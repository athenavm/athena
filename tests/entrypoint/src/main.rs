#![no_main]
use athena_vm::{entrypoint, types::Address};
use athena_vm_declare::{callable, template};
use athena_vm_sdk::call;

pub struct EntrypointTest {}

athena_vm::entrypoint!();

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
#[template]
impl EntrypointTest {
  #[callable]
  fn test1() {
    let address = athena_vm::io::read::<[u8; 24]>();
    let address = Address::from(address);
    // recursive call to self
    call(address, None, Some("athexp_test2"), 0);
  }

  #[callable]
  fn test2() {
    // no-op
  }

  #[allow(dead_code)]
  fn test3() {
    // no-op
  }
}
