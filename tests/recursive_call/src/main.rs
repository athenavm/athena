#![cfg_attr(target_os = "zkvm", no_main)]

#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!(main);

#[cfg(target_os = "zkvm")]
use athena_vm_sdk::call;

#[cfg(target_os = "zkvm")]
fn return_value(value: u32) {
  athena_vm::io::write::<u32>(&value);
}

#[cfg(target_os = "zkvm")]
fn recursive_call(address: [u8; 24], value: u32) -> u32 {
  let mut input = address.to_vec();
  input.extend_from_slice(&value.to_le_bytes());
  let output = call(address, Some(input), None, 0);
  u32::from_le_bytes(output.as_slice().try_into().unwrap())
}

#[cfg(target_os = "zkvm")]
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

  // Recursive case
  let a = recursive_call(address, n - 1);
  // println!("Got {} for n - 1", a);
  let b = recursive_call(address, n - 2);
  // println!("Got {} for n - 2", b);
  return_value(a + b);
}

#[cfg(not(target_os = "zkvm"))]
fn main() {
  println!("Not running on zkVM");
}
