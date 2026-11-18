use std::time::Instant;

/// What Windows says is driving this movement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The native viewport is running under finger contact.
    Contact,
    /// Windows is continuing the gesture after contact.
    Momentum,
}

/// A gesture's lifetime. Contact and momentum share one lifetime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// First movement is about to arrive in a separate `Move` event.
    Start,
    /// Apply the event's distance or scale once.
    Move,
    /// Windows has finished the gesture.
    End,
    /// The gesture was interrupted, replaced or the backend closed.
    Cancel,
}

/// Native movement, before any application scaling or boundary handling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    /// Content translation in physical pixels, positive right and down.
    Scroll {
        /// Incremental horizontal and vertical distance, not an absolute offset.
        delta: [f64; 2],
    },
    /// Pinch scaling. Anchor translation is deliberately excluded from scrolling.
    Pinch {
        /// Relative multiplier: greater than one zooms in.
        scale: f64,
    },
}

/// One owned event. `Start`, `End` and `Cancel` carry neutral movement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GestureEvent {
    /// Callback observation time on a monotonic clock, not a hardware timestamp.
    pub observed_at: Instant,
    /// Contact or Windows-managed momentum.
    pub source: Source,
    /// Begin, move, finish or interruption.
    pub phase: Phase,
    /// Physical-pixel translation or relative pinch scale.
    pub gesture: Gesture,
}
