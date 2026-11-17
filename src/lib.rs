//! One tiny Windows touchpad workaround, hopefully temporary.
//!
//! The native session, callback queue and window pump are in place. The public
//! gesture API and optional egui adapter come next. Windows owns momentum.
//!
//! The planned interface and window-lifetime rules live in `docs/api.md`.

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

// The public gesture adapter will expose these private pieces next.
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod attachment;
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod capture;
#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod session;
