use athena_runner::{vm::ExecutionContext, AthenaMessage, AthenaVm, VmInterface};
use athcon_sys as ffi;
use athcon_vm;

extern "C" fn destroy_vm(vm: *mut ffi::athcon_vm) {
  // Implementation for destroying the VM instance
  if vm.is_null() {
    return;
  } // Safety check to ensure the pointer is not null
  unsafe {
    let wrapper = &mut *(vm as *mut AthenaVmWrapper);
    drop(Box::from_raw(wrapper.vm));
    drop(Box::from_raw(wrapper));
  }
}

fn error_result() -> ffi::athcon_result {
  ffi::athcon_result {
    output_data: std::ptr::null_mut(),
    output_size: 0,
    gas_left: 0,
    create_address: ffi::athcon_address::default(),
    status_code: athcon_vm::StatusCode::ATHCON_FAILURE,
    release: None,
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
  // First, check for null pointers
  // For now we require them all to be non-null
  if msg.is_null() || host.is_null() || vm.is_null() {
    return error_result();
  }

  // SAFETY: We've checked that the pointers aren't null, so it's safe to dereference
  unsafe {
    // Unpack the VM
    let wrapper = &mut *(vm as *mut AthenaVmWrapper);
    let athena_vm = &mut *(wrapper.vm);

    // Unpack the context
    let ec_raw: &ffi::athcon_host_interface = &*host;
    let ec = ExecutionContext::new(ec_raw, context);

    // Convert the raw pointer to a reference
    let msg_ref: &ffi::athcon_message = &*msg;

    // Perform the conversion from `ffi::athcon_message` to `AthenaMessage`
    let athena_msg: AthenaMessage = (*msg_ref).into();

    // Execute the code and proxy the result back to the caller
    let execution_result = athena_vm.execute(
      ec,
      context,
      rev as u32,
      athena_msg,
      code,
      code_size,
    );
    let athcon_result: *const ffi::athcon_result = execution_result.into();
    *athcon_result
  }
}

extern "C" fn get_capabilities(_vm: *mut ffi::athcon_vm) -> ffi::athcon_capabilities_flagset {
  // Implementation for getting capabilities of the VM instance
  0
}

// Make this pub because it's not referenced inside the athcon_vm struct below,
// i.e., must be called separately
pub extern "C" fn result_dispose(result: *const ffi::athcon_result) {
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
  _value: *const ::std::os::raw::c_char,
) -> ffi::athcon_set_option_result {
  // Implementation for setting options of the VM instance
  return ffi::athcon_set_option_result::ATHCON_SET_OPTION_SUCCESS;
}

struct AthenaVmWrapper {
  base: ffi::athcon_vm,
  vm: *mut AthenaVm,
}

#[no_mangle]
pub extern "C" fn athcon_create() -> *mut ffi::athcon_vm {
  let athena_vm = Box::new(AthenaVm::new());
  let wrapper = Box::new(AthenaVmWrapper {
    base: ffi::athcon_vm {
      abi_version: 0,
      name: "Athena VM\0".as_ptr() as *const ::std::os::raw::c_char,
      version: "0.1.0\0".as_ptr() as *const ::std::os::raw::c_char,
      destroy: Some(destroy_vm),
      execute: Some(execute_code),
      get_capabilities: Some(get_capabilities),
      set_option: Some(set_option),
    },
    vm: Box::into_raw(athena_vm),
  });
  Box::into_raw(wrapper) as *mut ffi::athcon_vm
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::ptr;

  #[test]
  fn test_athcon_create() {
    unsafe {
      // Call the function to create a new VM instance
      let vm_ptr = athcon_create();

      // Ensure the returned pointer is not null
      assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");

      // Perform additional checks on the returned VM instance
      // For example, checking the abi_version or name if accessible
      let vm = &*vm_ptr;
      assert_eq!((*vm).abi_version, 0, "ABI version mismatch");
      assert_eq!(std::ffi::CStr::from_ptr((*vm).name).to_str().unwrap(), "Athena VM", "VM name mismatch");
      assert_eq!(std::ffi::CStr::from_ptr((*vm).version).to_str().unwrap(), "0.1.0", "Version mismatch");

      let wrapper = &mut *(vm_ptr as *mut AthenaVmWrapper);
      // let athena_vm = &mut *(wrapper.vm);

      // Test the FFI functions
      let set_option = (*vm).set_option.unwrap();
      assert_eq!(
        set_option(vm_ptr, "foo\0".as_ptr() as *const i8, "bar\0".as_ptr() as *const i8),
        ffi::athcon_set_option_result::ATHCON_SET_OPTION_SUCCESS
      );
      let get_capabilities = (*vm).get_capabilities.unwrap();
      assert_eq!(get_capabilities(vm_ptr), 0);
      let execute = (*vm).execute.unwrap();
      let result = execute(vm_ptr, ptr::null(), ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void, ffi::athcon_revision::ATHCON_FRONTIER, ptr::null(), ptr::null(), 0);
      assert_eq!(
        result.status_code,
        // failure expected due to input null pointers
        ffi::athcon_status_code::ATHCON_FAILURE
      );

      // Call them a second way
      assert_eq!(
        wrapper.base.set_option.unwrap()(vm_ptr, "foo\0".as_ptr() as *const i8, "bar\0".as_ptr() as *const i8),
        ffi::athcon_set_option_result::ATHCON_SET_OPTION_SUCCESS
      );
      assert_eq!(
        wrapper.base.get_capabilities.unwrap()(vm_ptr),
        0
      );
      assert_eq!(
        wrapper.base.execute.unwrap()(vm_ptr, ptr::null(), ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void, ffi::athcon_revision::ATHCON_FRONTIER, ptr::null(), ptr::null(), 0).status_code,
        // failure expected due to input null pointers
        ffi::athcon_status_code::ATHCON_FAILURE
      );

      // Cleanup: Destroy the VM instance to prevent memory leaks
      let destroy = (*vm).destroy.unwrap();
      destroy(vm_ptr);
    }
  }
}
