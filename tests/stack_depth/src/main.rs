#![no_main]
athena_vm::entrypoint!(main);

use athena_vm::helpers::address_to_32bit_words;
use athena_vm::types::ADDRESS_ALICE;

// Note: the test harness installs this contract code at ADDRESS_ALICE

pub fn main() {
  // recurse forever
  let address = address_to_32bit_words(ADDRESS_ALICE);
  unsafe { athena_vm::host::call(address.as_ptr(), std::ptr::null(), 0) };
}
