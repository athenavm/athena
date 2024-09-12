use std::panic;

use athcon_sys as ffi;

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
    spawn: None,
  }
}

/// Perform the same tests as the athena_vmlib crate, but using the FFI interface.
/// Note that these are raw tests, without a host interface. The host interface
/// is set to null. No host calls are performed by these tests.
#[test]
fn test_athcon_create() {
  let vm_ptr = athena_vmlib::athcon_create_athenavmwrapper();
  // Ensure the returned pointer is not null
  assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");
  unsafe {
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

    // Cleanup: Destroy the VM instance to prevent memory leaks
    vm.destroy.unwrap()(vm_ptr);
  }
}
