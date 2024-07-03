extern "C" {
  pub fn call(address: *const u32, input: *const u32, len: usize);
  pub fn read_storage(key: *mut u32, address: *const u32);
  pub fn write_storage(key: *mut u32, address: *const u32, value: *const u32);
}
