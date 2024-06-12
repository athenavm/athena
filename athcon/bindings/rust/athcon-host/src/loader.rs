 use athcon_sys as ffi;
 use std::ffi::{CStr, CString};
 use std::os::raw::c_char;
 use std::str;
 extern crate num;
 use num::FromPrimitive;

 #[link(name = "athcon-loader")]
 extern "C" {
     fn athcon_load_and_create(
         filename: *const c_char,
         athcon_loader_error_code: *mut i32,
     ) -> *mut ffi::athcon_vm;
     fn athcon_last_error_msg() -> *const c_char;
 }

 enum_from_primitive! {
 #[derive(Debug)]
 pub enum AthconLoaderErrorCode {
     /** The loader succeeded. */
     AthconLoaderSuccess = 0,

     /** The loader cannot open the given file name. */
     AthconLoaderCannotOpen = 1,

     /** The VM create function not found. */
     AthconLoaderSymbolNotFound = 2,

     /** The invalid argument value provided. */
     AthconLoaderInvalidArgument = 3,

     /** The creation of a VM instance has failed. */
     AthconLoaderInstanceCreationFailure = 4,

     /** The ABI version of the VM instance has mismatched. */
     AthconLoaderAbiVersionMismatch = 5,

     /** The VM option is invalid. */
     AthconLoaderInvalidOptionName = 6,

     /** The VM option value is invalid. */
     AthconLoaderInvalidOptionValue = 7,
 }
 }

 fn error(err: AthconLoaderErrorCode) -> Result<AthconLoaderErrorCode, &'static str> {
     match err {
         AthconLoaderErrorCode::AthconLoaderSuccess => Ok(AthconLoaderErrorCode::AthconLoaderSuccess),
         _ => unsafe { Err(CStr::from_ptr(athcon_last_error_msg()).to_str().unwrap()) },
     }
 }

 pub fn load_and_create(
     fname: &str,
 ) -> (*mut ffi::athcon_vm, Result<AthconLoaderErrorCode, &'static str>) {
     let c_str = CString::new(fname).unwrap();
     unsafe {
         let mut error_code: i32 = 0;
         let instance = athcon_load_and_create(c_str.as_ptr() as *const c_char, &mut error_code);
         return (
             instance,
             error(AthconLoaderErrorCode::from_i32(error_code).unwrap()),
         );
     }
 }
