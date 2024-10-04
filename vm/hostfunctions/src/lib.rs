extern "C" {
  pub fn call(
    address: *const u32,
    input_ptr: *const u32,
    input_len: usize,
    method_ptr: *const u32,
    method_len: usize,
    amount: *const u32,
  );
  pub fn read_storage(key: *mut u32);
  pub fn write_storage(key: *mut u32, value: *const u32);
  pub fn get_balance(value: *mut u32);
  pub fn spawn(blob: *const u32, len: usize);
}
