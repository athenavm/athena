//! athcon-declare is an attribute-style procedural macro to be used for automatic generation of FFI
//! code for the ATHCON API with minimal boilerplate.
//!
//! athcon-declare can be used by applying its attribute to any struct which implements the `AthconVm`
//! trait, from the athcon-vm crate.
//!
//! The macro takes three arguments: a valid UTF-8 stylized VM name, a comma-separated list of
//! capabilities, and a version string.
//!
//! # Example
//! ```
//! #[athcon_declare::athcon_declare_vm("This is an example VM name", "athena1", "1.2.3-custom")]
//! pub struct ExampleVM;
//!
//! impl athcon_vm::AthconVm for ExampleVM {
//!     fn init() -> Self {
//!       ExampleVM {}
//!     }
//!
//!     unsafe fn execute(
//!       &self,
//!       revision: athcon_vm::ffi::athcon_revision,
//!       code: &[u8],
//!       message: &athcon_vm::ExecutionMessage,
//!       host: *const athcon_vm::ffi::athcon_host_interface,
//!       context: *mut athcon_vm::ffi::athcon_host_context,
//!     ) -> athcon_vm::ExecutionResult {
//!       athcon_vm::ExecutionResult::success(1337, None)
//!     }
//! }
//! ```

// Set a higher recursion limit because parsing certain token trees might fail with the default of 64.
#![recursion_limit = "256"]

extern crate proc_macro;

use heck::AsShoutySnakeCase;
use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::AttributeArgs;
use syn::Ident;
use syn::ItemStruct;
use syn::Lit;
use syn::LitInt;
use syn::LitStr;
use syn::NestedMeta;

struct VMName(String);

struct VMMetaData {
  capabilities: u32,
  // Not included in VMName because it is parsed from the meta-item arguments.
  name_stylized: String,
  custom_version: String,
}

impl VMName {
  fn new(ident: String) -> Self {
    VMName(ident)
  }

  /// Get the struct's name as an explicit identifier to be interpolated with quote.
  fn as_type_ident(&self) -> Ident {
    Ident::new(&self.0, Span::call_site())
  }

  /// Get the lowercase name prepended with arbitrary text as an explicit ident.
  fn get_lowercase_as_ident_prepend(&self, prefix: &str) -> Ident {
    let concat = format!("{}{}", prefix, self.0.to_lowercase());
    Ident::new(&concat, Span::call_site())
  }

  /// Get the lowercase name appended with arbitrary text as an explicit ident.
  fn get_caps_as_ident_append(&self, suffix: &str) -> Ident {
    let concat = format!("{}{}", AsShoutySnakeCase(&self.0), suffix);
    Ident::new(&concat, Span::call_site())
  }
}

impl VMMetaData {
  fn new(args: AttributeArgs) -> Self {
    assert_eq!(args.len(), 3, "Incorrect number of arguments supplied");

    let vm_name_meta = &args[0];
    let vm_capabilities_meta = &args[1];
    let vm_version_meta = &args[2];

    let vm_name_string = match vm_name_meta {
      NestedMeta::Lit(lit) => {
        if let Lit::Str(s) = lit {
          // Add a null terminator here to ensure that it is handled correctly when
          // converted to a C String.
          let mut ret = s.value();
          ret.push('\0');
          ret
        } else {
          panic!("Literal argument type mismatch")
        }
      }
      _ => panic!("Argument 1 must be a string literal"),
    };

    let vm_capabilities_string = match vm_capabilities_meta {
      NestedMeta::Lit(lit) => {
        if let Lit::Str(s) = lit {
          s.value()
        } else {
          panic!("Literal argument type mismatch")
        }
      }
      _ => panic!("Argument 2 must be a string literal"),
    };

    // Parse the individual capabilities out of the list and prepare a capabilities flagset.
    // Prune spaces and underscores here to make a clean comma-separated list.
    let capabilities_list_pruned: String = vm_capabilities_string
      .chars()
      .filter(|c| *c != '_' && *c != ' ')
      .collect();
    let capabilities_flags = {
      let mut ret: u32 = 0;
      for capability in capabilities_list_pruned.split(',') {
        match capability {
          "athena1" => ret |= 1,
          // "ewasm" => ret |= 1 << 1,
          // "precompiles" => ret |= 1 << 2,
          _ => panic!("Invalid capability specified."),
        }
      }
      ret
    };

    let vm_version_string: String = if let NestedMeta::Lit(lit) = vm_version_meta {
      match lit {
        // Add a null terminator here to ensure that it is handled correctly when
        // converted to a C String.
        Lit::Str(s) => {
          let mut ret = s.value();
          ret.push('\0');
          ret
        }
        _ => panic!("Literal argument type mismatch"),
      }
    } else {
      panic!("Argument 3 must be a string literal")
    };

    // Make sure that the only null byte is the terminator we inserted in each string.
    assert_eq!(vm_name_string.matches('\0').count(), 1);
    assert_eq!(vm_version_string.matches('\0').count(), 1);

    VMMetaData {
      capabilities: capabilities_flags,
      name_stylized: vm_name_string,
      custom_version: vm_version_string,
    }
  }

  fn get_capabilities(&self) -> u32 {
    self.capabilities
  }

  fn get_name_stylized_nulterm(&self) -> &String {
    &self.name_stylized
  }

  fn get_custom_version_nulterm(&self) -> &String {
    &self.custom_version
  }
}

#[proc_macro_attribute]
pub fn athcon_declare_vm(args: TokenStream, item: TokenStream) -> TokenStream {
  // First, try to parse the input token stream into an AST node representing a struct
  // declaration.
  let input: ItemStruct = parse_macro_input!(item as ItemStruct);

  // Extract the identifier of the struct from the AST node.
  let vm_type_name: String = input.ident.to_string();

  let name = VMName::new(vm_type_name);

  // Parse the arguments for the macro.
  let meta_args = parse_macro_input!(args as AttributeArgs);
  let vm_data = VMMetaData::new(meta_args);

  // Get all the tokens from the respective helpers.
  let static_data_tokens = build_static_data(&name, &vm_data);
  let capabilities_tokens = build_capabilities_fn(vm_data.get_capabilities());
  let set_option_tokens = build_set_option_fn(&name);
  let create_tokens = build_create_fn(&name);
  let destroy_tokens = build_destroy_fn(&name);
  let execute_tokens = build_execute_fn(&name);

  let quoted = quote! {
      #input
      #static_data_tokens
      #capabilities_tokens
      #set_option_tokens
      #create_tokens
      #destroy_tokens
      #execute_tokens
  };

  quoted.into()
}

/// Generate tokens for the static data associated with an athcon VM.
fn build_static_data(name: &VMName, metadata: &VMMetaData) -> proc_macro2::TokenStream {
  // Stitch together the VM name and the suffix _NAME
  let static_name_ident = name.get_caps_as_ident_append("_NAME");
  let static_version_ident = name.get_caps_as_ident_append("_VERSION");

  // Turn the stylized VM name and version into string literals.
  let stylized_name_literal = LitStr::new(
    metadata.get_name_stylized_nulterm().as_str(),
    metadata.get_name_stylized_nulterm().as_str().span(),
  );

  // Turn the version into a string literal.
  let version_string = metadata.get_custom_version_nulterm();
  let version_literal = LitStr::new(version_string.as_str(), version_string.as_str().span());

  quote! {
      static #static_name_ident: &'static str = #stylized_name_literal;
      static #static_version_ident: &'static str = #version_literal;
  }
}

/// Takes a capabilities flag and builds the athcon_get_capabilities callback.
fn build_capabilities_fn(capabilities: u32) -> proc_macro2::TokenStream {
  let capabilities_string = capabilities.to_string();
  let capabilities_literal = LitInt::new(&capabilities_string, capabilities.span());

  quote! {
      extern "C" fn __athcon_get_capabilities(instance: *mut ::athcon_vm::ffi::athcon_vm) -> ::athcon_vm::ffi::athcon_capabilities_flagset {
          #capabilities_literal
      }
  }
}

fn build_set_option_fn(name: &VMName) -> proc_macro2::TokenStream {
  let type_name_ident = name.as_type_ident();

  quote! {
      extern "C" fn __athcon_set_option(
          instance: *mut ::athcon_vm::ffi::athcon_vm,
          key: *const std::os::raw::c_char,
          value: *const std::os::raw::c_char,
      ) -> ::athcon_vm::ffi::athcon_set_option_result
      {
          use athcon_vm::{AthconVm, SetOptionError};
          use std::ffi::CStr;

          assert!(!instance.is_null());

          if key.is_null() {
              return ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME;
          }

          let key = unsafe { CStr::from_ptr(key) };
          let key = match key.to_str() {
              Ok(k) => k,
              Err(e) => return ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME,
          };

          let value = if !value.is_null() {
              unsafe { CStr::from_ptr(value) }
          } else {
              unsafe { CStr::from_bytes_with_nul_unchecked(&[0]) }
          };

          let value = match value.to_str() {
              Ok(k) => k,
              Err(e) => return ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_VALUE,
          };

          let mut container = unsafe {
              // Acquire ownership from athcon.
              ::athcon_vm::AthconContainer::<#type_name_ident>::from_ffi_pointer(instance)
          };

          let result = match container.set_option(key, value) {
              Ok(()) => ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_SUCCESS,
              Err(SetOptionError::InvalidKey) => ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_NAME,
              Err(SetOptionError::InvalidValue) => ::athcon_vm::ffi::athcon_set_option_result::ATHCON_SET_OPTION_INVALID_VALUE,
          };

          unsafe {
              // Release ownership to athcon.
              ::athcon_vm::AthconContainer::into_ffi_pointer(container);
          }

          result
      }
  }
}

/// Takes an identifier and struct definition, builds an athcon_create_* function for FFI.
fn build_create_fn(name: &VMName) -> proc_macro2::TokenStream {
  let type_ident = name.as_type_ident();
  let fn_ident = name.get_lowercase_as_ident_prepend("athcon_create_");

  let static_version_ident = name.get_caps_as_ident_append("_VERSION");
  let static_name_ident = name.get_caps_as_ident_append("_NAME");

  // Note: we can get CStrs unchecked because we did the checks on instantiation of VMMetaData.
  quote! {
      #[no_mangle]
      extern "C" fn #fn_ident() -> *mut ::athcon_vm::ffi::athcon_vm {
          let new_instance = ::athcon_vm::ffi::athcon_vm {
              abi_version: ::athcon_vm::ffi::ATHCON_ABI_VERSION as i32,
              destroy: Some(__athcon_destroy),
              execute: Some(__athcon_execute),
              get_capabilities: Some(__athcon_get_capabilities),
              set_option: Some(__athcon_set_option),
              name: unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(#static_name_ident.as_bytes()).as_ptr() },
              version: unsafe { ::std::ffi::CStr::from_bytes_with_nul_unchecked(#static_version_ident.as_bytes()).as_ptr() },
          };

          let container = ::athcon_vm::AthconContainer::<#type_ident>::new(new_instance);

          unsafe {
              // Release ownership to athcon.
              ::athcon_vm::AthconContainer::into_ffi_pointer(container)
          }
      }
  }
}

/// Builds a callback to dispose of the VM instance.
fn build_destroy_fn(name: &VMName) -> proc_macro2::TokenStream {
  let type_ident = name.as_type_ident();

  quote! {
      extern "C" fn __athcon_destroy(instance: *mut ::athcon_vm::ffi::athcon_vm) {
          if instance.is_null() {
              // This is an irrecoverable error that violates the athcon spec.
              std::process::abort();
          }
          unsafe {
              // Acquire ownership from athcon. This will deallocate it also at the end of the scope.
              ::athcon_vm::AthconContainer::<#type_ident>::from_ffi_pointer(instance);
          }
      }
  }
}

/// Builds the main execution entry point.
fn build_execute_fn(name: &VMName) -> proc_macro2::TokenStream {
  let type_name_ident = name.as_type_ident();

  quote! {
      extern "C" fn __athcon_execute(
          instance: *mut ::athcon_vm::ffi::athcon_vm,
          host: *const ::athcon_vm::ffi::athcon_host_interface,
          context: *mut ::athcon_vm::ffi::athcon_host_context,
          revision: ::athcon_vm::ffi::athcon_revision,
          msg: *const ::athcon_vm::ffi::athcon_message,
          code: *const u8,
          code_size: usize
      ) -> ::athcon_vm::ffi::athcon_result
      {
          use athcon_vm::AthconVm;

          // TODO: context is optional in case of the "precompiles" capability
          if instance.is_null() || msg.is_null() || (code.is_null() && code_size != 0) {
              // These are irrecoverable errors that violate the athcon spec.
              std::process::abort();
          }

          assert!(!instance.is_null());
          assert!(!msg.is_null());

          let execution_message: ::athcon_vm::ExecutionMessage = unsafe {
              msg.as_ref().expect("athcon message is null").into()
          };

          let empty_code = [0u8;0];
          let code_ref: &[u8] = if code.is_null() {
              assert_eq!(code_size, 0);
              &empty_code
          } else {
              unsafe {
                  ::std::slice::from_raw_parts(code, code_size)
              }
          };

          let container = unsafe {
              // Acquire ownership from athcon.
              ::athcon_vm::AthconContainer::<#type_name_ident>::from_ffi_pointer(instance)
          };

          let result = unsafe {::std::panic::catch_unwind(|| {
              container.execute(revision, code_ref, &execution_message, host, context)
          })};

          let result = if result.is_err() {
              // Consider a panic an internal error.
              ::athcon_vm::ExecutionResult::new(::athcon_vm::ffi::athcon_status_code::ATHCON_INTERNAL_ERROR, 0, None)
          } else {
              result.unwrap()
          };

          unsafe {
              // Release ownership to athcon.
              ::athcon_vm::AthconContainer::into_ffi_pointer(container);
          }

          result.into()
      }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_vm_name_prepend_lowercase() {
    let name = super::VMName::new("ExampleVM".to_string());
    let ident = name.get_lowercase_as_ident_prepend("athcon_");
    assert_eq!(ident.to_string(), "athcon_examplevm");
  }
  #[test]
  fn test_vm_name_append_caps() {
    let name = super::VMName::new("ExampleVM".to_string());
    let ident = name.get_caps_as_ident_append("_NAME");
    assert_eq!(ident.to_string(), "EXAMPLE_VM_NAME");
  }
}
