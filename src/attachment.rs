//! The window owns one hook reference. Each callback borrows another so a
//! reentrant close cannot free its state midway through a message.

mod claim;
mod pump;

use std::{cell::Cell, rc::Rc, sync::Arc};

use windows::{
    core::{w, Error, Result, HRESULT},
    Win32::{
        Foundation::{GetLastError, SetLastError, E_FAIL, HWND, LPARAM, LRESULT, WPARAM},
        UI::{
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                RegisterWindowMessageW, DM_POINTERHITTEST, WM_NCDESTROY, WM_TIMER,
            },
        },
    },
};

use crate::{
    capture::{Capture, Sample},
    session::Session,
};
use claim::Claim;

const SUBCLASS_ID: usize = 0x52575450;

/// Native ownership behind the public gesture guard.
pub(crate) struct Attachment {
    state: Rc<State>,
}

struct State {
    session: Session,
    capture: Arc<Capture>,
    claim: Claim,
    window: HWND,
    message: u32,
    pointer_type: pump::GetPointerType,
    wake: Box<dyn Fn()>,
    timer: Cell<usize>,
    idle_updates: Cell<u32>,
    closed: Cell<bool>,
    attached: Cell<bool>,
    destroyed: Cell<bool>,
    error: Cell<Option<HRESULT>>,
}

impl Attachment {
    /// Attach on the window thread. The wake callback only requests later UI work.
    ///
    /// # Safety
    /// `window` must remain live throughout this call. The installed hook handles
    /// subsequent window destruction before the caller drops this guard.
    pub(crate) unsafe fn attach(window: HWND, wake: impl Fn() + 'static) -> Result<Self> {
        let claim = Claim::new(window)?;
        let pointer_type = pump::load_pointer_type()?;
        // SAFETY: Windows copies this constant message name.
        let message = unsafe { RegisterWindowMessageW(w!("rust_win_trackpad::input")) };
        if message == 0 {
            return Err(Error::from_win32());
        }
        let capture = Arc::new(Capture::for_window(window, message));
        // SAFETY: The caller keeps this owner-thread window alive during setup.
        let session = unsafe { Session::new(window)? };
        session.set_handler(&capture.handler())?;
        let state = Rc::new(State {
            session,
            capture,
            claim,
            window,
            message,
            pointer_type,
            wake: Box::new(wake),
            timer: Cell::new(0),
            idle_updates: Cell::new(0),
            closed: Cell::new(false),
            attached: Cell::new(false),
            destroyed: Cell::new(false),
            error: Cell::new(None),
        });
        let reference = Rc::into_raw(Rc::clone(&state)) as usize;
        // SAFETY: The hook owns this Rc reference until detachment or WM_NCDESTROY.
        if !unsafe { SetWindowSubclass(window, Some(subclass), SUBCLASS_ID, reference) }.as_bool() {
            let error = Error::from_win32();
            // SAFETY: Registration failed, so Windows never owned the reference.
            unsafe {
                drop(Rc::from_raw(reference as *const State));
            }
            return Err(error);
        }
        state.attached.set(true);
        let attachment = Self { state };
        attachment.state.session.enable()?;
        Ok(attachment)
    }

    pub(crate) fn take_samples(&self) -> Vec<Sample> {
        self.state.capture.take_samples()
    }

    pub(crate) fn last_error(&self) -> Option<HRESULT> {
        self.state.error.get()
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.state.closed.get()
    }

    pub(crate) fn close(&self) {
        self.state.close();
    }
}

impl State {
    fn close(&self) {
        if !self.closed.replace(true) {
            self.stop_updates();
            self.capture.close();
            self.session.close();
        }
    }

    fn fail(&self, error: Error) {
        self.error.set(Some(error.code()));
        self.close();
    }

    fn wake(&self) {
        // An application panic must not unwind through the native window procedure.
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.wake)())).is_err() {
            self.fail(Error::from_hresult(E_FAIL));
        }
    }

    fn detach(&self, destroying: bool) {
        if self.attached.get() {
            // SAFETY: This is the original owner-thread window and hook.
            let removed = unsafe { RemoveWindowSubclass(self.window, Some(subclass), SUBCLASS_ID) };
            if removed.as_bool() {
                self.attached.set(false);
                self.claim.release();
                // SAFETY: Release the hook's single reference. The caller holds
                // another Rc, including while WM_NCDESTROY is being forwarded.
                unsafe {
                    drop(Rc::from_raw(self as *const Self));
                }
            } else if destroying {
                self.claim.release();
            }
            // Failed removal retains the closed state. Even at destruction we
            // must not free a reference Windows might still use during reentry.
        }
    }
}

impl Drop for Attachment {
    fn drop(&mut self) {
        self.state.close();
        if !self.state.destroyed.get() {
            self.state.detach(false);
        }
    }
}

unsafe extern "system" fn subclass(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    reference: usize,
) -> LRESULT {
    let pointer = reference as *const State;
    // SAFETY: The installed hook holds a reference. This callback borrows one
    // more before any COM or application code can cause reentrant teardown.
    let state = unsafe {
        Rc::increment_strong_count(pointer);
        Rc::from_raw(pointer)
    };
    // SAFETY: These are thread-local error APIs with no pointer arguments.
    let previous_error = unsafe { GetLastError() };
    let mut handled = false;
    if message == WM_NCDESTROY {
        state.destroyed.set(true);
        state.close();
        state.detach(true);
    } else if !state.closed.get() {
        if message == DM_POINTERHITTEST {
            state.contact((wparam.0 & 0xffff) as u32);
        } else if message == WM_TIMER && wparam.0 != 0 && wparam.0 == state.timer.get() {
            state.pump();
            handled = true;
        } else if message == state.message {
            state.wake();
            handled = true;
        }
    }
    // SAFETY: Preserve the incoming error for downstream window procedures.
    unsafe {
        SetLastError(previous_error);
    }
    if handled {
        LRESULT(0)
    } else {
        // SAFETY: Forward every other message with its original arguments.
        unsafe { DefSubclassProc(window, message, wparam, lparam) }
    }
}

#[cfg(test)]
mod tests;
