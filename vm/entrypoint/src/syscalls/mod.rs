mod halt;
mod host;
mod io;
mod memory;
mod sys;

pub use halt::*;
pub use host::*;
pub use io::*;
pub use memory::*;
pub use sys::*;

/// These codes MUST match the codes in `core/src/runtime/syscall.rs`. There is a derived test
/// that checks that the enum is consistent with the syscalls.

/// Halts the program.
pub const HALT: u32 = 0x00_00_00_00;

/// Writes to a file descriptor. Currently only used for `STDOUT/STDERR`.
pub const WRITE: u32 = 0x00_00_00_02;

/// Executes `HINT_LEN`.
pub const HINT_LEN: u32 = 0x00_00_00_F0;

/// Executes `HINT_READ`.
pub const HINT_READ: u32 = 0x00_00_00_F1;

/// Host functions
pub const HOST_READ: u32 = 0x00_00_00_A0;
pub const HOST_WRITE: u32 = 0x00_00_00_A1;
