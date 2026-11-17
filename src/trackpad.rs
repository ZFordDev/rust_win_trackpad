use std::{num::NonZeroIsize, time::Instant};
use windows::{core::Result, Win32::Foundation::HWND};

use crate::{attachment::Attachment, convert::Converter, GestureEvent};

/// One Windows precision-touchpad session. Keep it on the window's owning thread.
///
/// The guard is neither `Send` nor `Sync`. Drop it to detach. Window destruction
/// also closes the backend, so dropping the guard afterward is harmless.
pub struct Trackpad {
    attachment: Attachment,
    converter: Converter,
}

impl Trackpad {
    /// Attach to a native HWND, with a callback that only requests later UI work.
    ///
    /// For `raw-window-handle` 0.6, pass `Win32WindowHandle::hwnd`. No winit or
    /// egui dependency is needed. The native timer pumps input independently.
    /// `wake` runs on the window thread; keep it short and do not render there.
    /// Duplicate attachment and native setup failures return an error.
    ///
    /// # Safety
    /// `window` must be a live HWND owned by the calling thread and must remain
    /// live throughout this call. Continue dispatching that thread's Windows
    /// messages normally. Do not bypass the installed subclass at destruction.
    pub unsafe fn attach(window: NonZeroIsize, wake: impl Fn() + 'static) -> Result<Self> {
        // SAFETY: The caller supplies a live owner-thread HWND during attachment.
        let attachment = unsafe { Attachment::attach(HWND(window.get() as *mut _), wake)? };
        Ok(Self {
            attachment,
            converter: Converter::default(),
        })
    }

    /// Drain each movement once, preserving callback order and observation times.
    ///
    /// Call after a wake, even if input is now closed: a previously delivered
    /// gesture then receives one `Cancel`. No rendering or native Update occurs.
    pub fn take_events(&mut self) -> Vec<GestureEvent> {
        let mut events = Vec::new();
        for sample in self.attachment.take_samples() {
            self.converter.append(sample, &mut events);
        }
        if self.is_closed() {
            self.converter.cancel(Instant::now(), &mut events);
        }
        events
    }

    /// Stop native input now. Drain once more to release any consumer gesture.
    pub fn close(&mut self) {
        self.attachment.close();
    }

    /// Whether explicit closure, a fatal error or window destruction stopped input.
    pub fn is_closed(&self) -> bool {
        self.attachment.is_closed()
    }

    /// The last native failure's HRESULT, including a rejected contact.
    ///
    /// A contact failure is recoverable; check `is_closed` separately.
    pub fn last_error(&self) -> Option<i32> {
        self.attachment.last_error().map(|code| code.0)
    }
}
