use super::*;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, SendMessageW, WM_GETTEXTLENGTH,
};

struct HiddenWindow(HWND);

impl HiddenWindow {
    fn new() -> Result<Self> {
        // SAFETY: STATIC is built in. The window is invisible and owned here.
        unsafe {
            CreateWindowExW(
                Default::default(),
                w!("STATIC"),
                w!("hook check"),
                Default::default(),
                0,
                0,
                100,
                100,
                None,
                None,
                None,
                None,
            )
            .map(Self)
        }
    }
}

impl Drop for HiddenWindow {
    fn drop(&mut self) {
        // SAFETY: This test owns the window on this thread.
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}

#[test]
fn hook_pump_and_both_destruction_orders() -> Result<()> {
    for window_first in [false, true] {
        let window = HiddenWindow::new()?;
        let wakes = Rc::new(Cell::new(0));
        let count = Rc::clone(&wakes);
        // SAFETY: The owner-thread window stays live through attachment.
        let attachment =
            unsafe { Attachment::attach(window.0, move || count.set(count.get() + 1))? };
        let state = Rc::downgrade(&attachment.state);
        // SAFETY: Same valid window; duplicate attachment must be rejected.
        assert!(unsafe { Attachment::attach(window.0, || {}) }.is_err());

        attachment.state.start_updates()?;
        for _ in 0..=pump::IDLE_UPDATES {
            attachment.state.pump();
        }
        assert_eq!(attachment.state.timer.get(), 0, "an idle pump must stop");
        assert!(!attachment.is_closed());
        assert!(attachment.last_error().is_none());
        let before = wakes.get();
        // SAFETY: These messages target the test window on its owner thread.
        unsafe {
            SendMessageW(window.0, attachment.state.message, None, None);
            assert_eq!(SendMessageW(window.0, WM_GETTEXTLENGTH, None, None).0, 10);
            SendMessageW(window.0, DM_POINTERHITTEST, None, None);
        }
        assert_eq!(wakes.get(), before + 1);
        assert_eq!(
            attachment.state.timer.get(),
            0,
            "an invalid contact must be ignored"
        );

        if window_first {
            drop(window);
            assert!(attachment.is_closed());
            assert!(attachment.state.destroyed.get());
            assert!(attachment.take_samples().is_empty());
            drop(attachment);
        } else {
            drop(attachment);
            // SAFETY: The original guard is gone and this window is still live.
            let replacement = unsafe { Attachment::attach(window.0, || {})? };
            drop(replacement);
        }
        assert!(state.upgrade().is_none(), "the hook must release its state");
    }
    println!("Window hook: wake, forwarding, idle pump, duplicate rejection and both teardown orders passed.");
    Ok(())
}
