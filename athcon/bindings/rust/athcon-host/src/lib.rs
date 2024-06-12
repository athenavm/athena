 #[macro_use]
 extern crate enum_primitive;
 pub mod host;
 mod loader;
 pub mod types;
 pub use crate::loader::{load_and_create, athconLoaderErrorCode};
 use crate::types::*;
 use athcon_sys as ffi;
 use std::ffi::CStr;

 extern "C" {
     fn athcon_create() -> *mut ffi::athcon_vm;
 }

 pub struct athconVm {
     handle: *mut ffi::athcon_vm,
     host_interface: *mut ffi::athcon_host_interface,
 }

 impl athconVm {
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

     pub fn execute(
         &self,
         ctx: &mut dyn host::HostContext,
         rev: Revision,
         kind: MessageKind,
         is_static: bool,
         depth: i32,
         gas: i64,
         destination: &Address,
         sender: &Address,
         input: &Bytes,
         value: &Bytes32,
         code: &Bytes,
         create2_salt: &Bytes32,
     ) -> (&Bytes, i64, StatusCode) {
         let ext_ctx = host::ExtendedContext { hctx: ctx };
         let mut athcon_flags: u32 = 0;
         unsafe {
             if is_static {
                 athcon_flags |=
                     std::mem::transmute::<ffi::athcon_flags, u32>(ffi::athcon_flags::ATHCON_STATIC);
             }
         }
         let athcon_message = Box::into_raw(Box::new({
             ffi::athcon_message {
                 kind: kind,
                 flags: athcon_flags,
                 depth: depth,
                 gas: gas,
                 destination: ffi::athcon_address {
                     bytes: *destination,
                 },
                 sender: ffi::athcon_address { bytes: *sender },
                 input_data: input.as_ptr(),
                 input_size: input.len(),
                 value: ffi::athcon_uint256be { bytes: *value },
                 create2_salt: ffi::athcon_bytes32 {
                     bytes: *create2_salt,
                 },
             }
         }));
         unsafe {
             let result = ((*self.handle).execute.unwrap())(
                 self.handle,
                 self.host_interface,
                 // ext_ctx as *mut ffi::athcon_host_context,
                 std::mem::transmute::<&host::ExtendedContext, *mut ffi::athcon_host_context>(
                     &ext_ctx,
                 ),
                 rev,
                 athcon_message,
                 code.as_ptr(),
                 code.len(),
             );
             return (
                 std::slice::from_raw_parts(result.output_data, result.output_size),
                 result.gas_left,
                 result.status_code,
             );
         }
     }

     pub fn has_capability(&self, capability: Capabilities) -> bool {
         unsafe {
             std::mem::transmute::<Capabilities, u32>(capability)
                 == ((*self.handle).get_capabilities.unwrap())(self.handle)
         }
     }
 }

 pub fn load(fname: &str) -> (athconVm, Result<athconLoaderErrorCode, &'static str>) {
     let (instance, ec) = load_and_create(fname);
     (
         athconVm {
             handle: instance,
             host_interface: Box::into_raw(Box::new(host::get_athcon_host_interface())),
         },
         ec,
     )
 }

 pub fn create() -> athconVm {
     unsafe {
         athconVm {
             handle: athcon_create(),
             host_interface: Box::into_raw(Box::new(host::get_athcon_host_interface())),
         }
     }
 }
