use std::collections::BTreeMap;
use std::rc::Rc;

use athcon_client::host::HostContext as HostInterface;
use athcon_client::{
  types::{Address, Bytes, Bytes32, ADDRESS_LENGTH, BYTES32_LENGTH},
  AthconVm,
};
use athcon_vm::{MessageKind, Revision, StatusCode, StorageStatus};
use athena_interface::ADDRESS_ALICE;

const CONTRACT_CODE: &[u8] =
  include_bytes!("../../../../../../tests/recursive_call/elf/recursive-call-test");

struct HostContext {
  storage: BTreeMap<Bytes32, Bytes32>,
  vm: Rc<AthconVm>,
}

impl HostContext {
  fn new(vm: AthconVm) -> HostContext {
    HostContext {
      storage: BTreeMap::new(),
      vm: Rc::new(vm),
    }
  }
}

// An extremely simplistic host implementation. Note that we cannot use the MockHost
// from athena-interface because we need to work with FFI types here.
impl HostInterface for HostContext {
  fn account_exists(&self, _addr: &Address) -> bool {
    println!("Host: account_exists");
    true
  }

  fn get_storage(&self, _addr: &Address, key: &Bytes32) -> Bytes32 {
    println!("Host: get_storage");
    let value = self.storage.get(key);
    let ret: Bytes32 = match value {
      Some(value) => value.to_owned(),
      None => [0u8; BYTES32_LENGTH],
    };
    println!("{:?} -> {:?}", hex::encode(key), hex::encode(ret));
    ret
  }

  fn set_storage(&mut self, _addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
    println!("Host: set_storage");
    println!("{:?} -> {:?}", hex::encode(key), hex::encode(value));
    self.storage.insert(key.to_owned(), value.to_owned());
    StorageStatus::ATHCON_STORAGE_MODIFIED
  }

  fn get_balance(&self, _addr: &Address) -> u64 {
    println!("Host: get_balance");
    0
  }

  fn get_tx_context(&self) -> (u64, Address, i64, i64, i64, Bytes32) {
    println!("Host: get_tx_context");
    (0, [0u8; ADDRESS_LENGTH], 0, 0, 0, [0u8; BYTES32_LENGTH])
  }

  fn get_block_hash(&self, _number: i64) -> Bytes32 {
    println!("Host: get_block_hash");
    [0u8; BYTES32_LENGTH]
  }

  fn call(
    &mut self,
    kind: MessageKind,
    destination: &Address,
    sender: &Address,
    value: u64,
    input: &Bytes,
    gas: i64,
    depth: i32,
  ) -> (Vec<u8>, i64, Address, StatusCode) {
    println!("Host: call");
    // check depth
    if depth > 10 {
      return (
        vec![0u8; BYTES32_LENGTH],
        0,
        [0u8; ADDRESS_LENGTH],
        StatusCode::ATHCON_CALL_DEPTH_EXCEEDED,
      );
    }

    // we recognize one destination address
    if destination != &ADDRESS_ALICE {
      return (
        vec![0u8; BYTES32_LENGTH],
        0,
        [0u8; ADDRESS_LENGTH],
        StatusCode::ATHCON_CONTRACT_VALIDATION_FAILURE,
      );
    }

    let res = self.vm.clone().execute(
      self,
      Revision::ATHCON_FRONTIER,
      kind,
      depth + 1,
      gas,
      destination,
      sender,
      input,
      value,
      CONTRACT_CODE,
    );
    (res.0.to_vec(), res.1, [0u8; ADDRESS_LENGTH], res.2)
  }

  fn spawn(&mut self, _blob: &[u8]) -> Address {
    todo!()
  }

  fn deploy(&mut self, _code: &[u8]) -> Address {
    todo!()
  }
}

impl Drop for HostContext {
  fn drop(&mut self) {
    println!("Dump storage:");
    for (key, value) in &self.storage {
      println!("{:?} -> {:?}", hex::encode(key), hex::encode(value));
    }
  }
}

/// Test the Rust host interface to athcon
/// We don't use this in production since Athena provides only the VM, not the Host, but
/// it allows us to test talking to the VM via FFI, and that the host bindings work as expected.
#[test]
fn test_rust_host() {
  let vm = AthconVm::new();
  println!("Instantiate: {:?}", (vm.get_name(), vm.get_version()));

  let mut host = HostContext::new(vm);

  let (output, gas_left, status_code) = host.vm.clone().execute(
    &mut host,
    Revision::ATHCON_FRONTIER,
    MessageKind::ATHCON_CALL,
    0,
    50000000,
    &ADDRESS_ALICE,
    &[128u8; ADDRESS_LENGTH],
    // input payload consists of empty method selector (4 bytes) + simple LE integer argument (4 bytes)
    [0u8, 0u8, 0u8, 0u8, 3u8, 0u8, 0u8, 0u8].as_ref(),
    0,
    CONTRACT_CODE,
  );
  println!("Output:  {:?}", hex::encode(&output));
  println!("GasLeft: {:?}", gas_left);
  println!("Status:  {:?}", status_code);
  assert_eq!(status_code, StatusCode::ATHCON_SUCCESS);
  assert_eq!(u32::from_le_bytes(output.as_slice().try_into().unwrap()), 2);
}
