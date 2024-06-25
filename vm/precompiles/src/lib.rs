pub mod io;
pub mod utils;
#[cfg(feature = "verify")]
pub mod verify;

extern "C" {
    pub fn syscall_halt(exit_code: u8) -> !;
    pub fn syscall_write(fd: u32, write_buf: *const u8, nbytes: usize);
    pub fn syscall_read(fd: u32, read_buf: *mut u8, nbytes: usize);
    pub fn syscall_hint_len() -> usize;
    pub fn syscall_hint_read(ptr: *mut u8, len: usize);
    pub fn sys_alloc_aligned(bytes: usize, align: usize) -> *mut u8;

    // host functions
    pub fn host_read_storage(address: *const u8, key: *const u8) -> *const u8;
    pub fn host_write_storage(address: *const u8, key: *const u8, value: *const u8) -> u32;
}
