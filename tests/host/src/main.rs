#![no_main]
athena_vm::entrypoint!(main);

pub fn main() {
  // use 32-bit words
  let address = [0u32; 6];
  let mut key = [2u32; 8];
  let mut key2 = [2u32; 8];

  // expected output value
  // [1u8; 32] i.e. 32 x 1-bytes
  // we convert [u8; 32] into [u32; 8] where each u32 is a 4 byte chunk
  // 0x01010101 == 16843009
  let value = [16843009u32; 8];

  // result will be written to key, overwriting input value
  unsafe { athena_vm::host::write_storage(key.as_mut_ptr(), address.as_ptr(), value.as_ptr()) };
  assert_eq!(key, [0u32; 8], "write_storage failed");

  // result will be written to key, overwriting input value
  unsafe { athena_vm::host::read_storage(key2.as_mut_ptr(), address.as_ptr()) };
  assert_eq!(value, key2, "read_storage failed");
  println!("success");
}
