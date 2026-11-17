use super::*;
use crate::capture::{Capture, NativeEvent};
use std::{sync::Arc, time::Instant};
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
    let capture = Arc::new(Capture::default());
    let handler = capture.handler();
    session.set_handler(&handler)?;
    session.enable()?;
    session.update()?;

    // Check the callback/queue boundary using real native content.
    let before = Instant::now();
    // SAFETY: The session and handler are live on this thread. The content's
    // local COM reference remains alive throughout the callback.
    unsafe {
        let content: IDirectManipulationContent = session.viewport.GetPrimaryContent()?;
        handler.OnContentUpdated(None::<&IDirectManipulationViewport>, &content)?;
    }
    let samples = capture.take_samples();
    assert!(samples.iter().any(|sample| {
        sample.observed_at >= before
            && matches!(sample.event, NativeEvent::Transform(matrix) if matrix == [1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
    }));
    assert!(capture.take_samples().is_empty(), "samples must drain once");
    capture.close();
    // A late callback must not restore input to a closed queue.
    // SAFETY: The retained handler is live; no viewport is supplied or accessed.
    unsafe {
        handler.OnViewportStatusChanged(
            None::<&IDirectManipulationViewport>,
            DIRECTMANIPULATION_READY,
            DIRECTMANIPULATION_RUNNING,
        )?;
    }
    assert!(capture.take_samples().is_empty());
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
    println!("Native session and COM callback: samples drained once, late input ignored, cleanup repeatable.");
    Ok(())
}
