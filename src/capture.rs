//! Capture what Windows tells us. Window hooks and UI conversion come later.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    time::Instant,
};

use windows::{
    core::{implement, Result},
    Win32::Graphics::DirectManipulation::*,
};

const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Raw internal samples; the public gesture API won't expose COM types.
pub(crate) enum NativeEvent {
    Status(DIRECTMANIPULATION_STATUS),
    Transform([f32; 6]),
}

pub(crate) struct Sample {
    // Observation time, not a hardware timestamp.
    pub observed_at: Instant,
    pub event: NativeEvent,
}

#[derive(Default)]
struct Queue {
    samples: Vec<Sample>,
    closed: bool,
}

/// Shared because callbacks can arrive after the update call has returned.
#[derive(Default)]
pub(crate) struct Capture {
    queue: Mutex<Queue>,
    // Resetting the native transform can re-enter the status callback.
    resetting: AtomicBool,
}

impl Capture {
    pub(crate) fn handler(self: &Arc<Self>) -> IDirectManipulationViewportEventHandler {
        Handler {
            capture: Arc::clone(self),
        }
        .into()
    }

    fn lock(&self) -> MutexGuard<'_, Queue> {
        self.queue.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn push(&self, event: NativeEvent) -> bool {
        let mut queue = self.lock();
        if queue.closed {
            return false;
        }
        queue.samples.push(Sample {
            observed_at: Instant::now(),
            event,
        });
        true
    }

    /// Draining releases the lock before the caller processes any input.
    pub(crate) fn take_samples(&self) -> Vec<Sample> {
        std::mem::take(&mut self.lock().samples)
    }

    /// Stop accepting callbacks before the session or native window closes.
    pub(crate) fn close(&self) {
        let mut queue = self.lock();
        queue.closed = true;
        queue.samples.clear();
    }
}

#[implement(IDirectManipulationViewportEventHandler)]
struct Handler {
    capture: Arc<Capture>,
}

#[allow(non_snake_case)]
impl IDirectManipulationViewportEventHandler_Impl for Handler_Impl {
    fn OnViewportStatusChanged(
        &self,
        viewport: Option<&IDirectManipulationViewport>,
        current: DIRECTMANIPULATION_STATUS,
        _previous: DIRECTMANIPULATION_STATUS,
    ) -> Result<()> {
        if !self.capture.push(NativeEvent::Status(current)) {
            return Ok(());
        }
        if current == DIRECTMANIPULATION_READY
            && !self.capture.resetting.swap(true, Ordering::Relaxed)
        {
            let result = if let Some(viewport) = viewport {
                // SAFETY: Windows retains the callback's viewport. No queue lock
                // is held across COM calls, which may re-enter this handler.
                unsafe {
                    viewport
                        .GetPrimaryContent::<IDirectManipulationContent>()
                        .and_then(|content| content.SyncContentTransform(&IDENTITY))
                }
            } else {
                Ok(())
            };
            self.capture.resetting.store(false, Ordering::Relaxed);
            result?;
        }
        Ok(())
    }

    fn OnViewportUpdated(&self, _viewport: Option<&IDirectManipulationViewport>) -> Result<()> {
        Ok(())
    }

    fn OnContentUpdated(
        &self,
        _viewport: Option<&IDirectManipulationViewport>,
        content: Option<&IDirectManipulationContent>,
    ) -> Result<()> {
        if self.capture.lock().closed {
            return Ok(());
        }
        if let Some(content) = content {
            let mut matrix = IDENTITY;
            // SAFETY: Windows retains the content; the output has six elements.
            unsafe { content.GetContentTransform(&mut matrix) }?;
            self.capture.push(NativeEvent::Transform(matrix));
        }
        Ok(())
    }
}
