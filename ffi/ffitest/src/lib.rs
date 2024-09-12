// E2E FFI test
// Note: this tests the vmlib crate as a compiled library. It must live outside that crate or else
// the extern function declaration below will resolve automatically to the local crate.
#[cfg(test)]
mod ffi_tests {
  use std::collections::BTreeMap;
  use std::rc::Rc;

  use athcon_client::host::HostContext as HostInterface;
  use athcon_client::types::{
    Address, Bytes, Bytes32, MessageKind, Revision, StatusCode, StorageStatus, ADDRESS_LENGTH,
    BYTES32_LENGTH,
  };
  use athcon_client::AthconVm;
  use athcon_sys as ffi;
  use athena_interface::ADDRESS_ALICE;

  const CONTRACT_CODE: &[u8] =
    include_bytes!("../../../tests/recursive_call/elf/recursive-call-test");
  const EMPTY_ADDRESS: Address = [0u8; ADDRESS_LENGTH];

  // Declare the external functions you want to test
  extern "C" {
    fn athcon_create_athenavmwrapper() -> *mut ffi::athcon_vm;
  }

  /// Perform the same tests as the athena_vmlib crate, but using the FFI interface.
  /// Note that these are raw tests, without a host interface. The host interface
  /// is set to null. No host calls are performed by these tests.
  #[test]
  fn test_athcon_create() {
    unsafe {
      athena_vmlib::vm_tests(athcon_create_athenavmwrapper());
    }
  }

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

    fn get_balance(&self, _addr: &Address) -> Bytes32 {
      println!("Host: get_balance");
      [0u8; BYTES32_LENGTH]
    }

    fn get_tx_context(&self) -> (Bytes32, Address, i64, i64, i64, Bytes32) {
      println!("Host: get_tx_context");
      (
        [0u8; BYTES32_LENGTH],
        EMPTY_ADDRESS,
        0,
        0,
        0,
        [0u8; BYTES32_LENGTH],
      )
    }

    fn get_block_hash(&self, _number: i64) -> Bytes32 {
      println!("Host: get_block_hash");
      [0u8; BYTES32_LENGTH]
    }

    fn spawn(&mut self, _blob: &[u8]) -> Address {
      println!("Host: spawn");
      ADDRESS_ALICE
    }

    fn call(
      &mut self,
      kind: MessageKind,
      destination: &Address,
      sender: &Address,
      value: &Bytes32,
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
          EMPTY_ADDRESS,
          StatusCode::ATHCON_CALL_DEPTH_EXCEEDED,
        );
      }

      // we recognize one destination address
      if destination != &ADDRESS_ALICE {
        return (
          vec![0u8; BYTES32_LENGTH],
          0,
          EMPTY_ADDRESS,
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
      (res.0.to_vec(), res.1, EMPTY_ADDRESS, res.2)
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
      // the value 3 as little-endian u32
      3u32.to_le_bytes().to_vec().as_slice(),
      &[0u8; BYTES32_LENGTH],
      CONTRACT_CODE,
    );
    println!("Output:  {:?}", hex::encode(&output));
    println!("GasLeft: {:?}", gas_left);
    println!("Status:  {:?}", status_code);
    assert_eq!(status_code, StatusCode::ATHCON_SUCCESS);
    assert_eq!(u32::from_le_bytes(output.as_slice().try_into().unwrap()), 2);
  }
}
