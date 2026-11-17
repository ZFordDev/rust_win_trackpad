//! One tiny Windows touchpad workaround, hopefully temporary.
//!
//! The native session and callback queue are in place. Window attachment and
//! input delivery come next, followed by an optional egui adapter. Windows
//! will own gesture momentum.
//!
//! The planned interface and window-lifetime rules live in `docs/api.md`.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

// These private pieces become the window attachment's implementation next.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod capture;
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod session;
