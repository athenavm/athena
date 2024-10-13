use athena_interface::StorageStatus;
use athena_vm::helpers::{bytes32_to_32bit_words, words_to_bytes32};

pub fn read_storage(key: [u8; 32]) -> [u8; 32] {
  let key = bytes32_to_32bit_words(key);
  let result = athena_vm::syscalls::read_storage(&key);
  words_to_bytes32(result)
}

pub fn write_storage(key: [u8; 32], value: [u8; 32]) -> StorageStatus {
  let key = bytes32_to_32bit_words(key);
  let value = bytes32_to_32bit_words(value);
  athena_vm::syscalls::write_storage(&key, &value)
}
