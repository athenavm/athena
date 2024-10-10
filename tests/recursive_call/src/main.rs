#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::bytes32_to_32bit_words;
use athena_vm::syscalls::{read_storage, write_storage};
use athena_vm::types::{
  StorageStatus::StorageAdded, StorageStatus::StorageModified, ADDRESS_ALICE, STORAGE_KEY,
};
use athena_vm_sdk::call;

// Note: the test harness installs this contract code at ADDRESS_ALICE

// emulate a return value by writing the return value to storage.
// Athena doesn't support return values yet.
fn return_value(value: u32) {
  let key = bytes32_to_32bit_words(STORAGE_KEY);
  let val: [u32; 8] = [value, 0, 0, 0, 0, 0, 0, 0];
  let status = write_storage(&key, &val);
  assert!(
    matches!(status, StorageAdded | StorageModified),
    "write_storage failed"
  );
  athena_vm::io::write::<u32>(&value);
}

fn recursive_call(value: u32) -> u32 {
  let value_bytes = value.to_le_bytes().to_vec();
  call(ADDRESS_ALICE, Some(value_bytes), None, 0);

  // read the return value
  let key = bytes32_to_32bit_words(STORAGE_KEY);
  let result = read_storage(&key);
  return result[0];
}

pub fn main() {
  // Read an input to the program.
  //
  // Behind the scenes, this compiles down to a custom system call which handles reading inputs.
  let n = athena_vm::io::read::<u32>();

  // println!("Calculating the {}th Fibonacci number", n);

  // Base case
  if n <= 1 {
    return_value(n);
    return;
  }

  // Recursive case
  let a = recursive_call(n - 1);
  // println!("Got {} for n - 1", a);
  let b = recursive_call(n - 2);
  // println!("Got {} for n - 2", b);
  return_value(a + b);
}
