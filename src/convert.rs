//! Keep the working prototype's transform rules, without GUI or winit types.

use std::time::Instant;
use windows::Win32::Graphics::DirectManipulation::*;

use crate::{
    capture::{NativeEvent, Sample},
    Gesture, GestureEvent, Phase, Source,
};

const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Scroll,
    Pinch,
}

impl Kind {
    fn neutral(self) -> Gesture {
        match self {
            Self::Scroll => Gesture::Scroll { delta: [0.0; 2] },
            Self::Pinch => Gesture::Pinch { scale: 1.0 },
        }
    }
}

pub(crate) struct Converter {
    matrix: [f32; 6],
    source: Option<Source>,
    gesture: Option<Kind>,
    // Once scaling starts, anchor translation must not become scrolling.
    zooming: bool,
}

impl Default for Converter {
    fn default() -> Self {
        Self {
            matrix: IDENTITY,
            source: None,
            gesture: None,
            zooming: false,
        }
    }
}

impl Converter {
    pub(crate) fn append(&mut self, sample: Sample, events: &mut Vec<GestureEvent>) {
        match sample.event {
            NativeEvent::Status(status) => self.status(status, sample.observed_at, events),
            NativeEvent::Transform(matrix) => self.transform(matrix, sample.observed_at, events),
        }
    }

    pub(crate) fn cancel(&mut self, at: Instant, events: &mut Vec<GestureEvent>) {
        self.finish(Phase::Cancel, at, events);
        self.source = None;
    }

    fn finish(&mut self, phase: Phase, at: Instant, events: &mut Vec<GestureEvent>) {
        if let Some(kind) = self.gesture.take() {
            self.emit(kind.neutral(), phase, at, events);
        }
    }

    fn emit(&self, gesture: Gesture, phase: Phase, at: Instant, events: &mut Vec<GestureEvent>) {
        if let Some(source) = self.source {
            events.push(GestureEvent {
                observed_at: at,
                source,
                phase,
                gesture,
            });
        }
    }

    fn status(
        &mut self,
        status: DIRECTMANIPULATION_STATUS,
        at: Instant,
        events: &mut Vec<GestureEvent>,
    ) {
        match status {
            DIRECTMANIPULATION_RUNNING => {
                if self.source == Some(Source::Momentum) {
                    self.finish(Phase::Cancel, at, events);
                }
                self.source = Some(Source::Contact);
            }
            DIRECTMANIPULATION_INERTIA => self.source = Some(Source::Momentum),
            DIRECTMANIPULATION_READY => {
                self.finish(Phase::End, at, events);
                self.source = None;
                self.matrix = IDENTITY;
                self.zooming = false;
            }
            _ => self.cancel(at, events),
        }
    }

    fn transform(&mut self, matrix: [f32; 6], at: Instant, events: &mut Vec<GestureEvent>) {
        if self.source.is_none() || !matrix.iter().all(|v| v.is_finite()) || matrix[0] <= 0.0 {
            return;
        }
        let previous = std::mem::replace(&mut self.matrix, matrix);
        self.zooming |= (matrix[0] - 1.0).abs() > f32::EPSILON;
        let (kind, gesture) = if self.zooming {
            let scale = f64::from(matrix[0]) / f64::from(previous[0]);
            if scale == 1.0 {
                return;
            }
            (Kind::Pinch, Gesture::Pinch { scale })
        } else {
            let delta = [
                f64::from(matrix[4]) - f64::from(previous[4]),
                f64::from(matrix[5]) - f64::from(previous[5]),
            ];
            if delta == [0.0; 2] {
                return;
            }
            (Kind::Scroll, Gesture::Scroll { delta })
        };
        if self.gesture != Some(kind) {
            self.finish(Phase::Cancel, at, events);
            self.emit(kind.neutral(), Phase::Start, at, events);
            self.gesture = Some(kind);
        }
        self.emit(gesture, Phase::Move, at, events);
    }
}

#[cfg(test)]
mod tests;
