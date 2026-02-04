#![cfg(target_os = "macos")]

use libc::mach_port_t;

pub(crate) mod cli;
pub use structs::SMC_BYTES_LEN;
pub mod io;
pub(crate) mod structs;
pub mod value;

unsafe extern "C" {
    pub fn mach_task_self() -> mach_port_t;
}
