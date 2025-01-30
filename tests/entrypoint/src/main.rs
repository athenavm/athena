use athena_vm::{
  entrypoint,
  types::{Address, MethodSelector},
};
use athena_vm_declare::{callable, template};
use athena_vm_sdk::call;
use std::io::Read;

pub struct EntrypointTest {}

athena_vm::entrypoint!();

#[template]
impl EntrypointTest {
  #[callable]
  fn test1() {
    let mut address = [0u8; 24];
    let n = athena_vm::io::Io::default().read(&mut address).unwrap();
    assert_eq!(24, n);
    let address = Address::from(address);

    // recursive call to self
    call(address, None, Some(MethodSelector::from("athexp_test2")), 0);
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
