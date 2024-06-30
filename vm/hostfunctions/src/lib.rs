extern "C" {
  pub fn host_read_storage(key: *mut u32, address: *const u32);
  pub fn host_write_storage(key: *mut u32, address: *const u32, value: *const u32);
}
