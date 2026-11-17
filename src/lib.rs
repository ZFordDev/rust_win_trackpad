//! One tiny Windows touchpad workaround, hopefully temporary.
//!
//! Native session setup and cleanup are in place. Window attachment and input
//! delivery come next, followed by an optional egui adapter. Windows will own
//! gesture momentum.
//!
//! The planned interface and window-lifetime rules live in `docs/api.md`.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

// The window attachment stage will become this private session's caller.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod session;
