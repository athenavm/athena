// E2E FFI test
// Note: this tests the vmlib crate as a compiled library. It must live outside that crate or else
// the extern function declaration below will resolve automatically to the local crate.
#[cfg(test)]
mod ffi_tests {
  use athcon_sys as ffi;

  // Declare the external functions you want to test
  extern "C" {
    fn athcon_create() -> *mut ffi::athcon_vm;
  }

  #[test]
  fn test_athcon_create() {
    unsafe {
      // Call the function to create a new VM instance
      let vm_ptr = athcon_create();

      // Ensure the returned pointer is not null
      assert!(!vm_ptr.is_null(), "VM creation returned a null pointer");

      // Perform additional checks and function calls as needed
      let vm = &*vm_ptr;
      assert_eq!((*vm).abi_version, 0, "ABI version mismatch");
      assert_eq!(std::ffi::CStr::from_ptr((*vm).name).to_str().unwrap(), "Athena VM", "VM name mismatch");
      assert_eq!(std::ffi::CStr::from_ptr((*vm).version).to_str().unwrap(), "0.1.0", "Version mismatch");

      // Cleanup: Destroy the VM instance to prevent memory leaks
      let destroy = (*vm).destroy.unwrap();
      destroy(vm_ptr);
    }
  }
}
