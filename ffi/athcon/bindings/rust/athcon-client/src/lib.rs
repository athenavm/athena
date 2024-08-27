extern crate enum_primitive;
pub mod host;
pub mod types;
use crate::types::*;
use athcon_sys as ffi;
use std::ffi::CStr;

extern "C" {
  fn athcon_create_athenavmwrapper() -> *mut ffi::athcon_vm;
}

// In principle it's safe to clone these handles, but the caller needs to be very careful to
// ensure the memory is freed properly, isn't double-freed, etc.
#[derive(Clone)]
pub struct AthconVm {
  handle: *mut ffi::athcon_vm,
  host_interface: ffi::athcon_host_interface,
}

impl AthconVm {
  pub fn get_abi_version(&self) -> i32 {
    unsafe {
      let version: i32 = (*self.handle).abi_version;
      version
    }
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

  pub fn destroy(&self) {
    unsafe { ((*self.handle).destroy.unwrap())(self.handle) }
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
    value: &[u8; 32],
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
      value: ffi::athcon_uint256be { bytes: *value },
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
      let data = std::slice::from_raw_parts(result.output_data, result.output_size);
      let output = data.to_vec();
      if let Some(release) = result.release {
        release(&result);
      }
      (output, result.gas_left, result.status_code)
    }
  }
}

pub fn create() -> AthconVm {
  unsafe {
    AthconVm {
      handle: athcon_create_athenavmwrapper(),
      host_interface: host::get_athcon_host_interface(),
    }
  }
}
