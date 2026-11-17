//! Optional egui 0.37 input conversion. This module never pumps native input.
//!
//! Call [`Input::append`] before egui consumes `RawInput`, including on the
//! follow-up frame it requests. Keep ordinary wheel events in the same vector.
//! Only accepted native contacts replace Windows' compatibility-wheel route.

use ::egui::{Event, Modifiers, MouseWheelSource, MouseWheelUnit, TouchPhase, Vec2};

use crate::{Gesture, GestureEvent, Phase, Source};

/// Per-window egui input state, separate from the native session and repaint loop.
#[derive(Default)]
pub struct Input {
    modifiers: Option<Modifiers>,
    scrolling: bool,
    end_pending: Option<Phase>,
}

impl Input {
    /// Prepend native events to the frame's existing egui event vector.
    ///
    /// `pixels_per_point` must include both window DPI and egui's zoom factor.
    /// Modifiers latch across momentum; newly pressed modifiers are also honoured.
    /// Native source and phase remain available in the original `pending` slice.
    ///
    /// Returns `true` when the caller must request one follow-up frame to deliver
    /// a deferred terminal event. Empty, idle calls return `false`.
    ///
    /// # Panics
    /// Panics if `pixels_per_point` is non-finite or not positive.
    pub fn append(
        &mut self,
        pending: &[GestureEvent],
        pixels_per_point: f32,
        modifiers: Modifiers,
        events: &mut Vec<Event>,
    ) -> bool {
        assert!(pixels_per_point.is_finite() && pixels_per_point > 0.0);
        let mut native = Vec::new();
        if self.end_pending.is_some() {
            self.end(&mut native);
        }
        let mut started = false;
        let mut moved = false;
        for event in pending {
            if event.phase == Phase::Start {
                self.modifiers = None;
                started = false;
            }
            match (event.gesture, event.phase) {
                (Gesture::Scroll { delta }, Phase::Move) => {
                    let delta = Vec2::new(
                        (delta[0] / f64::from(pixels_per_point)) as f32,
                        (delta[1] / f64::from(pixels_per_point)) as f32,
                    );
                    if !delta.is_finite() || delta == Vec2::ZERO {
                        continue;
                    }
                    let latched = self.modifiers.get_or_insert(modifiers);
                    *latched |= modifiers;
                    let source = match event.source {
                        Source::Contact => MouseWheelSource::Trackpad,
                        Source::Momentum => MouseWheelSource::Momentum,
                    };
                    if !started {
                        // Windows already supplies momentum. Re-establish the phase
                        // each batch so egui's timeout cannot add smoothing after a gap.
                        native.push(wheel(Vec2::ZERO, TouchPhase::Start, source, *latched));
                        started = true;
                    }
                    native.push(wheel(delta, TouchPhase::Move, source, *latched));
                    self.scrolling = true;
                    self.end_pending = None;
                    moved = true;
                }
                (Gesture::Scroll { .. }, Phase::End | Phase::Cancel) => {
                    if self.scrolling {
                        self.end_pending = Some(event.phase);
                    }
                    self.modifiers = None;
                    started = false;
                }
                (Gesture::Pinch { scale }, Phase::Move) => {
                    let scale = scale as f32;
                    if scale.is_finite() && scale > 0.0 && scale != 1.0 {
                        native.push(Event::Zoom(scale));
                    }
                }
                _ => {}
            }
        }
        // egui's End clears this frame's wheel delta. Let movement reach the UI
        // first, then put the terminal event before any ordinary wheel next frame.
        if self.end_pending.is_some() && !moved {
            self.end(&mut native);
        }
        native.append(events);
        *events = native;
        self.end_pending.is_some()
    }

    /// Release egui input when disabling the backend. Call `append` once afterward.
    pub fn cancel(&mut self) {
        if self.scrolling {
            self.end_pending = Some(Phase::Cancel);
        }
    }

    fn end(&mut self, events: &mut Vec<Event>) {
        let phase = match self.end_pending.take() {
            Some(Phase::Cancel) => TouchPhase::Cancel,
            _ => TouchPhase::End,
        };
        events.push(wheel(
            Vec2::ZERO,
            phase,
            MouseWheelSource::Unknown,
            Modifiers::NONE,
        ));
        self.scrolling = false;
        self.modifiers = None;
    }
}

fn wheel(delta: Vec2, phase: TouchPhase, source: MouseWheelSource, modifiers: Modifiers) -> Event {
    Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta,
        phase,
        source,
        modifiers,
    }
}

#[cfg(test)]
mod tests;
