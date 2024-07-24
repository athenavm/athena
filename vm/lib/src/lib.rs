//! System calls for the Athena VM.

pub mod io;
pub mod utils;

extern "C" {
  pub fn syscall_halt(exit_code: u8) -> !;
  pub fn syscall_write(fd: u32, write_buf: *const u8, nbytes: usize);
  pub fn syscall_read(fd: u32, read_buf: *mut u8, nbytes: usize);
  pub fn syscall_hint_len() -> usize;
  pub fn syscall_hint_read(ptr: *mut u8, len: usize);
  pub fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8;
}
