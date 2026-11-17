use super::*;
use windows::{
    core::w,
    Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow},
};

// Declared before the session so the window also survives an early test return.
struct HiddenWindow(HWND);

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        // SAFETY: This test owns the hidden window on its creating thread.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

#[test]
fn native_session_opens_updates_and_closes() -> Result<()> {
    // SAFETY: STATIC is a built-in class. No visible-window style or borrowed
    // application data is supplied.
    let window = HiddenWindow(unsafe {
        CreateWindowExW(
            Default::default(),
            w!("STATIC"),
            w!("rust_win_trackpad session check"),
            Default::default(),
            0,
            0,
            100,
            100,
            None,
            None,
            None,
            None,
        )?
    });
    // SAFETY: The window belongs to this test thread and outlives the session.
    let session = unsafe { Session::new(window.0)? };
    session.enable()?;
    session.update()?;
    session.close();
    session.close();
    assert_eq!(
        session.update().err().map(|error| error.code()),
        Some(RO_E_CLOSED)
    );
    assert_eq!(
        session.enable().err().map(|error| error.code()),
        Some(RO_E_CLOSED)
    );
    println!("Native session: opened, updated, closed twice; later calls rejected.");
    Ok(())
}
