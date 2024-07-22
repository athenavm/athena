extern "C" {
  pub fn call(address: *const u32, input: *const u32, len: usize);
  pub fn read_storage(key: *mut u32);
  pub fn write_storage(key: *mut u32, value: *const u32);
  pub fn get_balance(value: *mut u32);
}
