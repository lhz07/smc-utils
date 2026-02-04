#![cfg(target_os = "macos")]

pub use structs::SMC_BYTES_LEN;
pub mod io;
pub(crate) mod structs;
pub mod value;

use libc::mach_port_t;

unsafe extern "C" {
    pub fn mach_task_self() -> mach_port_t;
}
