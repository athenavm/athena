#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!(main);

use athena_interface::ADDRESS_CHARLIE;
use athena_vm::helpers::{address_to_32bit_words, bytes32_to_32bit_words};
use athena_vm::syscalls::{call, get_balance, read_storage, write_storage};
use athena_vm::types::{
  StorageStatus::StorageAdded, StorageStatus::StorageModified, STORAGE_KEY, STORAGE_VALUE,
};

// storage status is returned as a 256-bit value
const STORAGE_KEY_2: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

fn main() {
  let key = bytes32_to_32bit_words(STORAGE_KEY);
  let value = bytes32_to_32bit_words(STORAGE_VALUE);
  let value2: [u32; 8] = [0xaa; 8];
  let address_charlie = address_to_32bit_words(ADDRESS_CHARLIE);

  // Alice already has a storage item
  let res = read_storage(&key);
  assert_eq!(value, res, "read_storage failed");

  // Modify it
  let status = write_storage(&key, &value2);
  assert_eq!(status, StorageModified, "write_storage failed");

  // Read the modified value
  let res = read_storage(&key);
  assert_eq!(value2, res, "read_storage failed");

  // Try an empty key
  let res = read_storage(&STORAGE_KEY_2);
  assert_eq!([0; 8], res, "read_storage failed");

  // Write to the new key
  let status = write_storage(&STORAGE_KEY_2, &value);
  assert_eq!(status, StorageAdded, "write_storage failed");

  // Read the new value
  let res = read_storage(&STORAGE_KEY_2);
  assert_eq!(value, res, "read_storage failed");

  // Charlie does accept calls
  // Note: there is no way to check the result of a call
  // It either works, or it panics
  let value: [u32; 2] = [0, 0];
  call(
    address_charlie.as_ptr(),
    std::ptr::null(),
    0,
    std::ptr::null_mut(),
    0,
    value.as_ptr(),
  );

  // Check balance
  let value = get_balance();
  assert_eq!(10000, value, "get_balance failed");
}
