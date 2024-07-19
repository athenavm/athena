#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::{address_to_32bit_words, bytes32_to_32bit_words};
use athena_vm::types::{
  StorageStatus::StorageAdded, StorageStatus::StorageModified, ADDRESS_ALICE, HELLO_WORLD,
};

// Note: the test harness installs this contract code at ADDRESS_ALICE

// storage status is returned as a 256-bit value
const STORAGE_ADDED: [u32; 8] = [StorageAdded as u32, 0, 0, 0, 0, 0, 0, 0];
const STORAGE_MODIFIED: [u32; 8] = [StorageModified as u32, 0, 0, 0, 0, 0, 0, 0];

// emulate a return value by writing the return value to storage.
// Athena doesn't support return values yet.
fn return_value(value: u32) {
  let mut key = bytes32_to_32bit_words(HELLO_WORLD);
  let val: [u32; 8] = [value, 0, 0, 0, 0, 0, 0, 0];
  let address = address_to_32bit_words(ADDRESS_ALICE);
  unsafe { athena_vm::host::write_storage(key.as_mut_ptr(), address.as_ptr(), val.as_ptr()) };
  assert!(
    key == STORAGE_ADDED || key == STORAGE_MODIFIED,
    "write_storage failed"
  );
}

fn recursive_call(value: u32) -> u32 {
  // we need a pointer to the value as an array
  let val: [u32; 1] = [value];
  let address = address_to_32bit_words(ADDRESS_ALICE);
  unsafe { athena_vm::host::call(address.as_ptr(), val.as_ptr(), 1) };

  // read the return value
  let mut key = bytes32_to_32bit_words(HELLO_WORLD);
  unsafe { athena_vm::host::read_storage(key.as_mut_ptr(), address.as_ptr()) };
  return key[0];
}

pub fn main() {
  // Read an input to the program.
  //
  // Behind the scenes, this compiles down to a custom system call which handles reading inputs.
  let n = athena_vm::io::read::<u32>();

  // Base case
  if n <= 1 {
    return_value(n);
    return;
  }

  // Recursive case
  let a = recursive_call(n - 1);
  let b = recursive_call(n - 2);
  return_value(a + b);
}
