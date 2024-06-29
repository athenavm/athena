// E2E FFI test
// Note: this tests the vmlib crate as a compiled library. It must live outside that crate or else
// the extern function declaration below will resolve automatically to the local crate.
#[cfg(test)]
mod ffi_tests {
  use std::collections::BTreeMap;

  use athcon_client::create;
  use athcon_client::host::HostContext as HostInterface;
  use athcon_client::types::{
    Address, Bytes, Bytes32, MessageKind, Revision, StatusCode, StorageStatus, ADDRESS_LENGTH,
    BYTES32_LENGTH,
  };
  use athcon_sys as ffi;
  use athena_vmlib;

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
  }

  impl HostContext {
    fn new() -> HostContext {
      HostContext {
        storage: BTreeMap::new(),
      }
    }
  }

  // test all of the host functions the VM can handle
  impl HostInterface for HostContext {
    fn account_exists(&self, _addr: &Address) -> bool {
      println!("Host: account_exists");
      return true;
    }

    fn get_storage(&self, _addr: &Address, key: &Bytes32) -> Bytes32 {
      println!("Host: get_storage");
      let value = self.storage.get(key);
      let ret: Bytes32;
      match value {
        Some(value) => ret = value.to_owned(),
        None => ret = [0u8; BYTES32_LENGTH],
      }
      println!("{:?} -> {:?}", hex::encode(key), hex::encode(ret));
      return ret;
    }

    fn set_storage(&mut self, _addr: &Address, key: &Bytes32, value: &Bytes32) -> StorageStatus {
      println!("Host: set_storage");
      println!("{:?} -> {:?}", hex::encode(key), hex::encode(value));
      self.storage.insert(key.to_owned(), value.to_owned());
      return StorageStatus::ATHCON_STORAGE_MODIFIED;
    }

    fn get_balance(&self, _addr: &Address) -> Bytes32 {
      println!("Host: get_balance");
      return [0u8; BYTES32_LENGTH];
    }

    fn get_tx_context(&self) -> (Bytes32, Address, i64, i64, i64, Bytes32) {
      println!("Host: get_tx_context");
      return (
        [0u8; BYTES32_LENGTH],
        [0u8; ADDRESS_LENGTH],
        0,
        0,
        0,
        [0u8; BYTES32_LENGTH],
      );
    }

    fn get_block_hash(&self, _number: i64) -> Bytes32 {
      println!("Host: get_block_hash");
      return [0u8; BYTES32_LENGTH];
    }

    fn call(
      &mut self,
      _kind: MessageKind,
      _destination: &Address,
      _sender: &Address,
      _value: &Bytes32,
      _input: &Bytes,
      _gas: i64,
      _depth: i32,
    ) -> (Vec<u8>, i64, Address, StatusCode) {
      println!("Host: call");
      return (
        vec![0u8; BYTES32_LENGTH],
        _gas,
        [0u8; ADDRESS_LENGTH],
        StatusCode::ATHCON_SUCCESS,
      );
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
    let code = include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");
    let vm = create();
    println!("Instantiate: {:?}", (vm.get_name(), vm.get_version()));
    let mut host = HostContext::new();
    let (output, gas_left, status_code) = vm.execute(
      &mut host,
      Revision::ATHCON_FRONTIER,
      MessageKind::ATHCON_CALL,
      123,
      50000000,
      &[32u8; ADDRESS_LENGTH],
      &[128u8; ADDRESS_LENGTH],
      &[0u8; 0],
      &[0u8; BYTES32_LENGTH],
      &code[..],
    );
    println!("Output:  {:?}", hex::encode(output));
    println!("GasLeft: {:?}", gas_left);
    println!("Status:  {:?}", status_code);
    assert_eq!(status_code, StatusCode::ATHCON_SUCCESS);
    vm.destroy();
  }
}
