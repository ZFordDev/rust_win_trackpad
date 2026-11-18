//! One tiny Windows touchpad workaround, hopefully temporary.
//!
//! [`GestureEvent`] carries physical-pixel scrolling and relative pinch scales.
//! On Windows, [`Trackpad`] attaches to one owner-thread HWND. Windows supplies
//! momentum; this crate does not predict movement or continuously repaint.
//!
//! Enable the `egui` feature for the egui 0.37 input adapter (requires Rust 1.95).
//! The core has no GUI dependency. See `docs/api.md` for the window contract.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod event;
pub use event::{Gesture, GestureEvent, Phase, Source};

#[cfg(feature = "egui")]
pub mod egui;

#[cfg(target_os = "windows")]
mod attachment;
#[cfg(target_os = "windows")]
mod capture;
#[cfg(target_os = "windows")]
mod convert;
#[cfg(target_os = "windows")]
mod session;
#[cfg(target_os = "windows")]
mod trackpad;
#[cfg(target_os = "windows")]
pub use trackpad::Trackpad;
