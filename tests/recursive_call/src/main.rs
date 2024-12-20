#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!(main);

use athena_interface::Address;
use athena_vm_sdk::call;

fn return_value(value: u32) {
  athena_vm::io::write::<u32>(&value);
}

fn recursive_call(address: Address, value: u32) -> u32 {
  let mut input = address.as_ref().to_vec();
  input.extend_from_slice(&value.to_le_bytes());
  let output = call(address, Some(input), None, 0);
  u32::from_le_bytes(output.as_slice().try_into().unwrap())
}

fn main() {
  // Read an input to the program.
  //
  // Behind the scenes, this compiles down to a custom system call which handles reading inputs.
  let (address, n) = athena_vm::io::read::<([u8; 24], u32)>();
  // Base case
  if n <= 1 {
    return_value(n);
    return;
  }
  let address = Address::from(address);

  // Recursive case
  let a = recursive_call(address, n - 1);
  // println!("Got {} for n - 1", a);
  let b = recursive_call(address, n - 2);
  // println!("Got {} for n - 2", b);
  return_value(a + b);
}
