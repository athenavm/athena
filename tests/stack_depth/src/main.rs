#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::address_to_32bit_words;
use athena_vm::syscalls::call;
use athena_vm::types::ADDRESS_ALICE;
use std::ptr::null;

// Note: the test harness installs this contract code at ADDRESS_ALICE

pub fn main() {
  // recurse forever
  let address = address_to_32bit_words(ADDRESS_ALICE);
  let value: [u32; 2] = [0, 0];
  call(address.as_ptr(), null(), 0, value.as_ptr());
}
