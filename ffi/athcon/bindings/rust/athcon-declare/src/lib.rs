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
//!     fn execute(
//!       &self,
//!       revision: athcon_vm::ffi::athcon_revision,
//!       code: &[u8],
//!       message: &athcon_vm::ExecutionMessage,
//!       host: &athcon_vm::ffi::athcon_host_interface,
//!       context: *mut athcon_vm::ffi::athcon_host_context,
//!     ) -> athcon_vm::ExecutionResult {
//!       athcon_vm::ExecutionResult::success(1337, None)
//!     }
//! }
//! ```

// Set a higher recursion limit because parsing certain token trees might fail with the default of 64.
#![recursion_limit = "256"]

extern crate proc_macro;

use std::ffi::CString;

use heck::AsShoutySnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Ident;
use syn::ItemStruct;
use syn::Lit;
use syn::LitCStr;
use syn::LitInt;

struct VMName(String);

#[derive(Debug)]
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

  /// Get the lowercase name appended with arbitrary text as an explicit ident.
  fn get_caps_as_ident_append(&self, suffix: &str) -> Ident {
    let concat = format!("{}{}", AsShoutySnakeCase(&self.0), suffix);
    Ident::new(&concat, Span::call_site())
  }
}

impl Parse for VMMetaData {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let lits = syn::punctuated::Punctuated::<Lit, syn::Token![,]>::parse_terminated(input)?;

    if lits.len() != 3 {
      return Err(syn::Error::new_spanned(
        lits,
        "Expected exactly three arguments",
      ));
    }

    let name = match &lits[0] {
      Lit::Str(s) => s.value(),
      lit => {
        return Err(syn::Error::new_spanned(
          lit,
          "First argument must be a string literal",
        ))
      }
    };

    let capabilities_string = match &lits[1] {
      Lit::Str(s) => s.value(),
      lit => {
        return Err(syn::Error::new_spanned(
          lit,
          "Second argument must be a string literal",
        ))
      }
    };

    let capabilities_list_pruned: String = capabilities_string
      .chars()
      .filter(|c| *c != '_' && *c != ' ')
      .collect();
    let mut capabilities_flags = 0u32;
    for capability in capabilities_list_pruned.split(',') {
      match capability {
        "athena1" => capabilities_flags |= 1,
        _ => panic!("Invalid capability specified."),
      }
    }

    let version = match &lits[2] {
      Lit::Str(s) => s.value(),
      lit => {
        return Err(syn::Error::new_spanned(
          lit,
          "Third argument must be a string literal",
        ))
      }
    };

    Ok(VMMetaData {
      capabilities: capabilities_flags,
      name_stylized: name,
      custom_version: version,
    })
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
  let vm_data = parse_macro_input!(args as VMMetaData);

  // Get all the tokens from the respective helpers.
  let static_data_tokens = build_static_data(&name, &vm_data);
  let capabilities_tokens = build_capabilities_fn(vm_data.capabilities);
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
  let stylized_name = CString::new(metadata.name_stylized.as_str()).unwrap();
  let stylized_name_literal = LitCStr::new(stylized_name.as_c_str(), metadata.name_stylized.span());

  let version = CString::new(metadata.custom_version.as_str()).unwrap();
  let version_literal = LitCStr::new(version.as_c_str(), metadata.custom_version.span());

  quote! {
      static #static_name_ident: &'static core::ffi::CStr = #stylized_name_literal;
      static #static_version_ident: &'static core::ffi::CStr = #version_literal;
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
  let fn_ident = Ident::new("athcon_create", Span::call_site());

  let static_version_ident = name.get_caps_as_ident_append("_VERSION");
  let static_name_ident = name.get_caps_as_ident_append("_NAME");

  // Note: we can get CStrs unchecked because we did the checks on instantiation of VMMetaData.
  quote! {
      #[no_mangle]
      pub extern "C" fn #fn_ident() -> *mut ::athcon_vm::ffi::athcon_vm {
          let new_instance = ::athcon_vm::ffi::athcon_vm {
              abi_version: ::athcon_vm::ffi::ATHCON_ABI_VERSION as i32,
              destroy: Some(__athcon_destroy),
              execute: Some(__athcon_execute),
              get_capabilities: Some(__athcon_get_capabilities),
              set_option: Some(__athcon_set_option),
              name: #static_name_ident.as_ptr(),
              version: #static_version_ident.as_ptr(),
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
          let host = if let Some(host) = unsafe { host.as_ref() } {
            host
          } else {
            return athcon_vm::ExecutionResult::failure().into();
          };

          assert!(msg.is_aligned(), "msg must be properly aligned");
          let execution_message = unsafe { msg.as_ref() }.
            expect("msg must be non-null").
            try_into().
            expect("converting to ExecutionMessage");

          let code_ref = if code.is_null() {
            assert_eq!(code_size, 0, "code_size must be 0 if code is null");
            &[]
          } else {
            unsafe { ::std::slice::from_raw_parts(code, code_size) }
          };

          assert!(!instance.is_null(), "instance must be non-null");
          assert!(instance.is_aligned(), "instance must be properly aligned");
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
  fn test_vm_name_append_caps() {
    let name = super::VMName::new("ExampleVM".to_string());
    let ident = name.get_caps_as_ident_append("_NAME");
    assert_eq!(ident.to_string(), "EXAMPLE_VM_NAME");
  }

  #[test]
  fn test_vmmetadata_parsing() {
    let s = syn::parse_str::<super::VMMetaData>(
      r#""This is an example VM name", "athena1", "1.2.3-custom""#,
    )
    .unwrap();

    assert_eq!(s.capabilities, 1);
    assert_eq!(s.name_stylized, "This is an example VM name");
    assert_eq!(s.custom_version, "1.2.3-custom");
  }

  #[test]
  fn test_vmmetadata_parsing_needs_3_attributes() {
    let s = syn::parse_str::<super::VMMetaData>(r#""This is an example VM name", "athena1""#);

    let err = s.unwrap_err();
    assert_eq!(err.to_string(), "Expected exactly three arguments");
  }
}
