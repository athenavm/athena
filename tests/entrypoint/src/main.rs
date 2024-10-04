#![no_main]
use athena_vm_declare::{callable, template};
use athena_vm_sdk::call;

pub struct EntrypointTest {}

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
#[template]
impl EntrypointTest {
  #[callable]
  fn test1() {
    let input = athena_vm::io::read_vec();
    let address =
      bincode::deserialize(&input).expect("input address malformed, failed to deserialize");
    let method = "athexp_test2".as_bytes().to_vec();

    // recursive call to self
    call(address, None, Some(method), 0);
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
