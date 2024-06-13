use athcon_sys as ffi;

extern "C" fn destroy_vm(vm: *mut ffi::athcon_vm) {
  // Implementation for destroying the VM instance
  if vm.is_null() { return; } // Safety check to ensure the pointer is not null
  unsafe {
    // Convert the raw pointer back to a Box, allowing Rust to reclaim the memory
    drop(Box::from_raw(vm));
  }
}

extern "C" fn execute_code(
  vm: *mut ffi::athcon_vm,
  host: *const ffi::athcon_host_interface,
  context: *mut ffi::athcon_host_context,
  rev: ffi::athcon_revision,
  msg: *const ffi::athcon_message,
  code: *const u8,
  code_size: usize,
) -> ffi::athcon_result {
  // Implementation for executing code in the VM instance
  return ffi::athcon_result {
      status_code: ffi::athcon_status_code::ATHCON_SUCCESS,
      gas_left: 1337,
      output_data: Box::into_raw(Box::new([0xde, 0xad, 0xbe, 0xef])) as *const u8,
      output_size: 4,
      release: Some(result_dispose),
      create_address: ffi::athcon_address { bytes: [0u8; 24] },
  };
}

extern "C" fn get_capabilities(_vm: *mut ffi::athcon_vm) -> ffi::athcon_capabilities_flagset {
  // Implementation for getting capabilities of the VM instance
  0
}

extern "C" fn result_dispose(result: *const ffi::athcon_result) {
    unsafe {
        if !result.is_null() {
            let owned = *result;
            Vec::from_raw_parts(
                owned.output_data as *mut u8,
                owned.output_size,
                owned.output_size,
            );
        }
    }
}

extern "C" fn set_option(
  _vm: *mut ffi::athcon_vm,
  _name: *const ::std::os::raw::c_char,
  _value: *const ::std::os::raw::c_char
) -> ffi::athcon_set_option_result {
  // Implementation for setting options of the VM instance
  return ffi::athcon_set_option_result::ATHCON_SET_OPTION_SUCCESS;
}

#[no_mangle]
pub extern "C" fn athcon_create() -> *mut ffi::athcon_vm {
  // Implementation for creating an instance of AthconVm
  Box::into_raw(Box::new(ffi::athcon_vm {
    abi_version: 0,
    name: "Example VM\0".as_ptr() as *const ::std::os::raw::c_char,
    version: "0.1.0\0".as_ptr() as *const ::std::os::raw::c_char,
    destroy: Some(destroy_vm),
    execute: Some(execute_code),
    get_capabilities: Some(get_capabilities),
    set_option: Some(set_option),
  }))
}
