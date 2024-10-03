use athena_interface::Address;
use athena_vm::helpers::{address_to_32bit_words, bytes32_to_32bit_words};
use athena_vm::host;

pub fn read_storage(key: [u8; 32]) -> [u8; 32] {
  let mut key = bytes32_to_32bit_words(key);
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr()) };
  return key;
}

pub fn write_storage(key: [u8; 32], value: [u8; 32]) {
  let mut key = bytes32_to_32bit_words(key);
  let value = bytes32_to_32bit_words(value);
  unsafe { athena_vm::host::write_storage(key.as_mut_ptr(), value.as_mut_ptr()) };
}
