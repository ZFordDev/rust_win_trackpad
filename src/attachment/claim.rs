use std::cell::Cell;

use windows::{
    core::{w, Error, Result},
    Win32::{
        Foundation::{E_INVALIDARG, E_UNEXPECTED, HANDLE, HWND},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{GetPropW, GetWindowThreadProcessId, RemovePropW, SetPropW},
    },
};

/// Reserve one attachment per window, including while COM setup can re-enter.
pub(super) struct Claim {
    window: HWND,
    held: Cell<bool>,
}

impl Claim {
    pub(super) fn new(window: HWND) -> Result<Self> {
        // SAFETY: These APIs validate the opaque handle. Nothing is dereferenced.
        unsafe {
            if GetWindowThreadProcessId(window, None) != GetCurrentThreadId() {
                return Err(Error::new(E_INVALIDARG, "attach on the window's thread"));
            }
            if !GetPropW(window, w!("rust_win_trackpad::attached"))
                .0
                .is_null()
            {
                return Err(Error::new(
                    E_UNEXPECTED,
                    "this window already has a trackpad hook",
                ));
            }
            // This is just a presence marker; Windows never dereferences it.
            SetPropW(
                window,
                w!("rust_win_trackpad::attached"),
                HANDLE(1usize as *mut _),
            )?;
        }
        Ok(Self {
            window,
            held: Cell::new(true),
        })
    }

    pub(super) fn release(&self) {
        if self.held.replace(false) {
            // SAFETY: Released on the owning thread, before window destruction.
            unsafe {
                let _ = RemovePropW(self.window, w!("rust_win_trackpad::attached"));
            }
        }
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.release();
    }
}
