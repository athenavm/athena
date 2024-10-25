mod halt;
mod host;
mod io;
mod memory;
mod sys;

pub mod precompiles;

pub use halt::*;
pub use host::*;
pub use io::*;
pub use memory::*;
pub use sys::*;

/// These codes MUST match the codes in `core/src/runtime/syscall.rs`. There is a derived test
/// that checks that the enum is consistent with the syscalls.
///
/// Halts the program.
pub const HALT: u32 = 0x00_00_00_00;

/// Writes to a file descriptor. Currently only used for `STDOUT/STDERR`.
pub const WRITE: u32 = 0x00_00_00_02;

/// Verifies ED25519 signature.
pub const PRECOMPILE_ED25519_VERIFY: u32 = 0x00_64_00_20;

/// Executes `HINT_LEN`.
pub const HINT_LEN: u32 = 0x00_00_00_F0;

/// Executes `HINT_READ`.
pub const HINT_READ: u32 = 0x00_00_00_F1;

/// Host functions
pub const HOST_READ: u32 = 0x00_00_00_A0;
pub const HOST_WRITE: u32 = 0x00_00_00_A1;
pub const HOST_CALL: u32 = 0x00_00_00_A2;
pub const HOST_GETBALANCE: u32 = 0x00_00_00_A3;
pub const HOST_SPAWN: u32 = 0x00_00_00_A4;
pub const HOST_DEPLOY: u32 = 0x00_00_00_A5;
