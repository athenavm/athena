#![cfg_attr(target_os = "zkvm", no_main)]

mod contract;

#[cfg(target_os = "zkvm")]
use athena_vm::entrypoint;
#[cfg(target_os = "zkvm")]
athena_vm::entrypoint!();

// needed to make the main compile for tests
#[cfg(not(target_os = "zkvm"))]
pub fn main() {}
