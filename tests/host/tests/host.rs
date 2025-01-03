use std::{collections::BTreeMap, error::Error};

use athena_interface::{
  Address, AthenaContext, AthenaMessage, Balance, Bytes32, ExecutionResult, StorageStatus,
};
use athena_sdk::{host::HostInterface, AthenaStdin, ExecutionClient};

#[derive(Default)]
struct Host {
  storage: BTreeMap<(Address, [u8; 32]), [u8; 32]>,
}

impl HostInterface for Host {
  fn get_storage(&self, addr: &Address, key: &Bytes32) -> Bytes32 {
    self
      .storage
      .get(&(*addr, *key))
      .copied()
      .unwrap_or_default()
  }

  fn set_storage(&mut self, addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    match self.storage.insert((*addr, *key), *value) {
      None => StorageStatus::StorageAdded,
      Some(_) => StorageStatus::StorageModified,
    }
  }

  fn get_balance(&self, _: &Address) -> Balance {
    unimplemented!()
  }

  fn call(&mut self, _: AthenaMessage) -> ExecutionResult {
    unimplemented!()
  }

  fn spawn(&mut self, _: Vec<u8>) -> Address {
    unimplemented!()
  }

  fn deploy(&mut self, _: Vec<u8>) -> Result<Address, Box<dyn Error>> {
    unimplemented!()
  }
}

#[test]
fn test() {
  tracing_subscriber::fmt::init();

  let elf = include_bytes!("../elf/host-test");
  let mut host = Host::default();
  let stdin = AthenaStdin::new();
  let context = AthenaContext::new(Address::from([0xCC; 24]), Address::default(), 0);
  let result =
    ExecutionClient::new().execute(elf, stdin, Some(&mut host), Some(100_000), Some(context));
  // result will be Err if asserts in the test failed
  let (_, gas_left) = result.unwrap();

  assert!(gas_left.unwrap() < 100_000);
}
