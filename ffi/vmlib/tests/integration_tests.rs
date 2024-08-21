use std::collections::BTreeMap;
use std::panic;

use athcon_client::host::HostContext as HostInterface;
use athcon_client::types::{
  Address, Bytes, Bytes32, MessageKind, Revision, StatusCode, StorageStatus, ADDRESS_LENGTH,
  BYTES32_LENGTH,
};
use athcon_client::{create, AthconVm};
use athcon_sys as ffi;
use athena_interface::ADDRESS_ALICE;

const CONTRACT_CODE: &[u8] =
  include_bytes!("../../../tests/recursive_call/elf/recursive-call-test");
const EMPTY_ADDRESS: Address = [0u8; ADDRESS_LENGTH];

struct AthenaVmWrapper {
  base: ffi::athcon_vm,
}

unsafe extern "C" fn get_dummy_tx_context(
  _context: *mut ffi::athcon_host_context,
) -> ffi::athcon_tx_context {
  ffi::athcon_tx_context {
    tx_gas_price: athcon_vm::Uint256 { bytes: [0u8; 32] },
    tx_origin: athcon_vm::Address { bytes: [0u8; 24] },
    block_height: 42,
    block_timestamp: 235117,
    block_gas_limit: 105023,
    chain_id: athcon_vm::Uint256::default(),
  }
}

// Update these when needed for tests
fn get_dummy_host_interface() -> ffi::athcon_host_interface {
  ffi::athcon_host_interface {
    account_exists: None,
    get_storage: None,
    set_storage: None,
    get_balance: None,
    call: None,
    get_tx_context: Some(get_dummy_tx_context),
    get_block_hash: None,
  }
}

/// Perform the same tests as the athena_vmlib crate, but using the FFI interface.
/// Note that these are raw tests, without a host interface. The host interface
/// is set to null. No host calls are performed by these tests.
#[test]
fn test_athcon_create() {
  unsafe {
    let vm_ptr = athena_vmlib::athcon_create_athenavmwrapper();
    // Ensure the returned pointer is not null
    assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");

    // Perform additional checks on the returned VM instance
    let vm = &*vm_ptr;
    assert_eq!(vm.abi_version, 0, "ABI version mismatch");
    assert_eq!(
      std::ffi::CStr::from_ptr(vm.name).to_str().unwrap(),
      "Athena",
      "VM name mismatch"
    );
    assert_eq!(
      std::ffi::CStr::from_ptr(vm.version).to_str().unwrap(),
      "0.1.0",
      "Version mismatch"
    );

    let wrapper = &mut *(vm_ptr as *mut AthenaVmWrapper);

    // Test the FFI functions
    assert_eq!(
      vm.set_option.unwrap()(
        vm_ptr,
        "foo\0".as_ptr() as *const std::os::raw::c_char,
        "bar\0".as_ptr() as *const std::os::raw::c_char
      ),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      vm.get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );

    // Construct mock host, context, message, and code objects for test
    let host_interface = get_dummy_host_interface();
    let code = include_bytes!("../../../examples/hello_world/program/elf/hello-world-program");
    let empty_code = &[0u8; 0];
    let message = ::athcon_sys::athcon_message {
      kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
      depth: 0,
      gas: 10000,
      recipient: ::athcon_sys::athcon_address::default(),
      sender: ::athcon_sys::athcon_address::default(),
      input_data: std::ptr::null(),
      input_size: 0,
      value: ::athcon_sys::athcon_uint256be::default(),
      code: code.as_ptr(),
      code_size: code.len(),
    };

    // this message is invalid because code_size doesn't match code length
    let bad_message = ::athcon_sys::athcon_message {
      kind: ::athcon_sys::athcon_call_kind::ATHCON_CALL,
      depth: 0,
      gas: 0,
      recipient: ::athcon_sys::athcon_address::default(),
      sender: ::athcon_sys::athcon_address::default(),
      input_data: std::ptr::null(),
      input_size: 0,
      value: ::athcon_sys::athcon_uint256be::default(),
      code: std::ptr::null(),
      code_size: 1,
    };

    // note: we cannot check for a null instance or message pointer here, as the VM wrapper code
    // calls `std::process::abort()`. this is a violation of the athcon spec.
    // host pointer is allowed to be null.

    // fail due to null host pointer
    assert_eq!(
      vm.execute.unwrap()(
        vm_ptr,
        // host can be null
        std::ptr::null(),
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // fail due to empty code
    assert_eq!(
      vm.execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        empty_code.as_ptr(),
        empty_code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // fail due to bad message
    // fails an assertion inside the VM macro code
    let result = panic::catch_unwind(|| {
      vm.execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &bad_message,
        code.as_ptr(),
        code.len(),
      )
    });
    assert!(result.is_err(), "Expected panic did not occur");

    // this one should succeed
    // note that host needs to be non-null, but the host context can be null.
    // the VM is unopinionated about it and doesn't rely on it directly.
    assert_eq!(
      vm.execute.unwrap()(
        vm_ptr,
        &host_interface,
        // host_context is unused and opaque
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_SUCCESS
    );

    // Call them a second way
    assert_eq!(
      wrapper.base.set_option.unwrap()(
        vm_ptr,
        "foo\0".as_ptr() as *const std::os::raw::c_char,
        "bar\0".as_ptr() as *const std::os::raw::c_char
      ),
      ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME
    );
    assert_eq!(
      wrapper.base.get_capabilities.unwrap()(vm_ptr),
      ffi::athcon_capabilities::ATHCON_CAPABILITY_Athena1 as u32
    );
    assert_eq!(
      wrapper.base.execute.unwrap()(
        vm_ptr,
        // host can be null
        std::ptr::null(),
        // host_context is an opaque pointer
        std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        ffi::athcon_revision::ATHCON_FRONTIER,
        &message,
        code.as_ptr(),
        code.len(),
      )
      .status_code,
      // failure expected due to input null pointers
      ffi::athcon_status_code::ATHCON_FAILURE
    );

    // Cleanup: Destroy the VM instance to prevent memory leaks
    vm.destroy.unwrap()(vm_ptr);
  }
}

struct HostContext {
  storage: BTreeMap<Bytes32, Bytes32>,
  vm: AthconVm,
}

impl HostContext {
  fn new(vm: AthconVm) -> HostContext {
    HostContext {
      storage: BTreeMap::new(),
      vm,
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

    // Create an owned copy of VM here to avoid borrow issues when passing self into execute
    // Note: this clone duplicates the FFI handles, but we don't attempt to destroy them here.
    // That'll be done using the original handles.
    let vm = self.vm.clone();
    let res = vm.execute(
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
  let vm = create();
  println!("Instantiate: {:?}", (vm.get_name(), vm.get_version()));

  // Same proviso as above: we're cloning the pointers here, which is fine as long as we
  // don't attempt to destroy them twice, or use the clone after we destroy the original.
  let mut host = HostContext::new(vm.clone());
  let (output, gas_left, status_code) = vm.execute(
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
  println!("Output:  {:?}", hex::encode(output));
  println!("GasLeft: {:?}", gas_left);
  println!("Status:  {:?}", status_code);
  assert_eq!(status_code, StatusCode::ATHCON_SUCCESS);
  assert_eq!(output, 2u32.to_le_bytes().to_vec().as_slice());
  vm.destroy();
}
