pub mod host;
pub mod types;
use crate::types::*;
use athcon_sys as ffi;
use std::ffi::CStr;

/// Athena VM wrapper
///
/// It owns the VM handle and provides a high-level interface to interact with the VM.
/// It also provides a host interface to the VM.
/// The VM is automatically destroyed when the wrapper is dropped.
pub struct AthconVm {
  handle: *mut ffi::athcon_vm,
  host_interface: ffi::athcon_host_interface,
}

impl AthconVm {
  pub fn new() -> Self {
    let handle = athena_vmlib::athcon_create_athenavmwrapper();
    assert!(!handle.is_null(), "Failed to create athena vm");

    AthconVm {
      handle,
      host_interface: host::get_athcon_host_interface(),
    }
  }

  pub fn get_abi_version(&self) -> i32 {
    unsafe { (*self.handle).abi_version }
  }

  pub fn get_name(&self) -> &str {
    unsafe {
      let c_str: &CStr = CStr::from_ptr((*self.handle).name);
      c_str.to_str().unwrap()
    }
  }

  pub fn get_version(&self) -> &str {
    unsafe {
      let c_str: &CStr = CStr::from_ptr((*self.handle).version);
      c_str.to_str().unwrap()
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub fn execute(
    &self,
    ctx: &mut dyn host::HostContext,
    rev: Revision,
    kind: MessageKind,
    depth: i32,
    gas: i64,
    destination: &Address,
    sender: &Address,
    input: &[u8],
    value: u64,
    code: &[u8],
  ) -> (Vec<u8>, i64, StatusCode) {
    let ext_ctx = host::ExtendedContext { hctx: ctx };
    let athcon_message = ffi::athcon_message {
      kind,
      depth,
      gas,
      recipient: ffi::athcon_address {
        bytes: *destination,
      },
      sender: ffi::athcon_address { bytes: *sender },
      input_data: input.as_ptr(),
      input_size: input.len(),
      value,
      code: code.as_ptr(),
      code_size: code.len(),
    };

    unsafe {
      let execute_func = (*self.handle).execute.unwrap();
      let result = execute_func(
        self.handle,
        &self.host_interface,
        // ext_ctx as *mut ffi::athcon_host_context,
        std::mem::transmute::<&host::ExtendedContext, *mut ffi::athcon_host_context>(&ext_ctx),
        rev,
        &athcon_message,
        code.as_ptr(),
        code.len(),
      );
      let output = if !result.output_data.is_null() && result.output_size > 0 {
        let data = std::slice::from_raw_parts(result.output_data, result.output_size);
        data.to_vec()
      } else {
        Vec::new()
      };
      if let Some(release) = result.release {
        release(&result);
      }
      (output, result.gas_left, result.status_code)
    }
  }
}

impl Drop for AthconVm {
  fn drop(&mut self) {
    unsafe { ((*self.handle).destroy.unwrap())(self.handle) }
  }
}

impl Default for AthconVm {
  fn default() -> Self {
    Self::new()
  }
}
