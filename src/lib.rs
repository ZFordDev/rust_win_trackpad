//! One tiny Windows touchpad workaround, hopefully temporary.
//!
//! This is just the crate skeleton for now. Native input is not wired up yet.
//! The plan is a small, framework-independent Direct Manipulation backend,
//! followed by an optional egui adapter. Windows will own gesture momentum.
//!
//! The planned interface and window-lifetime rules live in `docs/api.md`.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]
