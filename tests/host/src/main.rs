#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::{address_to_32bit_words, bytes32_to_32bit_words};
use athena_vm::types::{
  StorageStatus::StorageAdded, StorageStatus::StorageModified, ADDRESS_CHARLIE, SOME_COINS,
  STORAGE_KEY, STORAGE_VALUE,
};

// storage status is returned as a 256-bit value
const STORAGE_ADDED: [u32; 8] = [StorageAdded as u32, 0, 0, 0, 0, 0, 0, 0];
const STORAGE_MODIFIED: [u32; 8] = [StorageModified as u32, 0, 0, 0, 0, 0, 0, 0];
const STORAGE_KEY_2: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

pub fn main() {
  let mut key = bytes32_to_32bit_words(STORAGE_KEY);
  let value = bytes32_to_32bit_words(STORAGE_VALUE);
  let value2: [u32; 8] = [0xaa; 8];
  let value_unset: [u32; 8] = [0; 8];
  let address_charlie = address_to_32bit_words(ADDRESS_CHARLIE);

  // note: for all of these calls, the result is written to the first argument, hence as_mut_ptr()

  // Alice already has a storage item
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr()) };
  assert_eq!(value, key, "read_storage failed");

  // Modify it
  let mut key = bytes32_to_32bit_words(STORAGE_KEY);
  unsafe { athena_vm::host::write_storage(key.as_mut_ptr(), value2.as_ptr()) };
  assert_eq!(key, STORAGE_MODIFIED, "write_storage failed");

  // Read the modified value
  let mut key = bytes32_to_32bit_words(STORAGE_KEY);
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr()) };
  assert_eq!(value2, key, "read_storage failed");

  // Try an empty key
  let mut key = STORAGE_KEY_2;
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr()) };
  assert_eq!(value_unset, key, "read_storage failed");

  // Write to the new key
  let mut key = STORAGE_KEY_2;
  unsafe { athena_vm::host::write_storage(key.as_mut_ptr(), value.as_ptr()) };
  assert_eq!(key, STORAGE_ADDED, "write_storage failed");

  // Read the new value
  let mut key = STORAGE_KEY_2;
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr()) };
  assert_eq!(value, key, "read_storage failed");

  // Alice does not accept calls
  // unsafe { athena_vm::host::call(address_alice.as_ptr(), std::ptr::null(), 0) };

  // Charlie does accept calls
  // Note: there is no way to check the result of a call
  // It either works, or it panics
  unsafe { athena_vm::host::call(address_charlie.as_ptr(), std::ptr::null(), 0) };

  println!("success");
}
