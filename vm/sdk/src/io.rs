use athena_vm::helpers::{bytes32_to_32bit_words, words_to_bytes32};
use athena_vm::host;

pub fn read_storage(key: [u8; 32]) -> [u8; 32] {
  let mut key = bytes32_to_32bit_words(key);
  unsafe { host::read_storage(key.as_mut_ptr()) };
  return words_to_bytes32(key);
}

pub fn write_storage(key: [u8; 32], value: [u8; 32]) {
  let mut key = bytes32_to_32bit_words(key);
  let value = bytes32_to_32bit_words(value);
  unsafe { host::write_storage(key.as_mut_ptr(), value.as_ptr()) };
}
