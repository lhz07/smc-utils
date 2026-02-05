//! A library for interacting with Apple System Management Control (SMC) on macOS.
//!
//! This library provides safe Rust wrappers around IOKit APIs to read and write
//! SMC keys, which control various hardware parameters such as temperatures,
//! fan speeds, battery status, and more.
//!

#![cfg(target_os = "macos")]
#![deny(clippy::unwrap_used)]

pub mod io;
pub mod structs;
pub mod value;
