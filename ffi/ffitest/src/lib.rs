// E2E FFI test
// Note: this tests the vmlib crate as a compiled library. It must live outside that crate or else
// the extern function declaration below will resolve automatically to the local crate.
#[cfg(test)]
mod ffi_tests {
  use athcon_sys as ffi;
  use athena_vmlib;

  // Declare the external functions you want to test
  extern "C" {
    fn athcon_create_athenavmwrapper() -> *mut ffi::athcon_vm;
  }

  #[test]
  fn test_athcon_create() {
    unsafe {
      athena_vmlib::vm_tests(athcon_create_athenavmwrapper());
    }
  }
}
