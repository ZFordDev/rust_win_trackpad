use windows::{
    core::{s, w, Error, Result},
    Win32::{
        Foundation::{BOOL, E_NOTIMPL},
        Graphics::DirectManipulation::{DIRECTMANIPULATION_INERTIA, DIRECTMANIPULATION_RUNNING},
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
        UI::WindowsAndMessaging::{KillTimer, SetTimer, POINTER_INPUT_TYPE, PT_TOUCHPAD},
    },
};

use super::State;

const INTERVAL_MS: u32 = 10;
pub(super) const IDLE_UPDATES: u32 = 25;
pub(super) type GetPointerType = unsafe extern "system" fn(u32, *mut POINTER_INPUT_TYPE) -> BOOL;

pub(super) fn load_pointer_type() -> Result<GetPointerType> {
    // SAFETY: A live window has user32 loaded. The export has this documented
    // signature; resolving it dynamically avoids a new startup requirement.
    unsafe {
        let module = GetModuleHandleW(w!("user32.dll"))?;
        let address = GetProcAddress(module, s!("GetPointerType"))
            .ok_or_else(|| Error::from_hresult(E_NOTIMPL))?;
        Ok(std::mem::transmute::<
            unsafe extern "system" fn() -> isize,
            GetPointerType,
        >(address))
    }
}

impl State {
    pub(super) fn start_updates(&self) -> Result<()> {
        self.idle_updates.set(0);
        if self.timer.get() == 0 {
            // SAFETY: The live window owns this timer. The stable state address
            // gives it an ID distinct from other application timers.
            let timer =
                unsafe { SetTimer(self.window, self as *const Self as usize, INTERVAL_MS, None) };
            if timer == 0 {
                return Err(Error::from_win32());
            }
            self.timer.set(timer);
        }
        Ok(())
    }

    pub(super) fn stop_updates(&self) {
        let timer = self.timer.replace(0);
        if timer != 0 {
            // SAFETY: Stopped on the owner thread before window destruction.
            unsafe {
                let _ = KillTimer(self.window, timer);
            }
        }
    }

    pub(super) fn contact(&self, pointer_id: u32) {
        let mut kind = POINTER_INPUT_TYPE::default();
        // SAFETY: Windows supplies the pointer ID; the output is a valid local.
        if unsafe { (self.pointer_type)(pointer_id, &mut kind) }.as_bool() && kind == PT_TOUCHPAD {
            if let Err(error) = self.start_updates() {
                self.fail(error);
                self.wake();
                return;
            }
            // Failed contacts retain Windows' compatibility-wheel path.
            if let Err(error) = self.session.contact(pointer_id) {
                self.error.set(Some(error.code()));
            }
        }
    }

    pub(super) fn pump(&self) {
        if let Err(error) = self.session.update() {
            self.fail(error);
            self.wake();
            return;
        }
        if matches!(
            self.capture.status(),
            DIRECTMANIPULATION_RUNNING | DIRECTMANIPULATION_INERTIA
        ) {
            self.idle_updates.set(0);
        } else {
            self.idle_updates.set(self.idle_updates.get() + 1);
            // Callbacks can arrive after Update returns, so allow a short grace period.
            if self.idle_updates.get() >= IDLE_UPDATES {
                self.stop_updates();
            }
        }
    }
}
