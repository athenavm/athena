#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::address_to_32bit_words;
use athena_vm::types::ADDRESS_ALICE;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::call;

// Note: the test harness installs this contract code at ADDRESS_ALICE

pub fn main() {
  // recurse forever
  let address = address_to_32bit_words(ADDRESS_ALICE);
  let value: [u32; 2] = [0, 0];
  unsafe { athena_vm::host::call(address.as_ptr(), std::ptr::null(), 0, value.as_ptr()) };
}

pub struct EntrypointTest {};

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
#[template]
impl EntrypointTest {
  #[callable]
  fn test1() {
  }

  #[callable]
  fn test2() {
  }

  fn test3() {
  }
}
