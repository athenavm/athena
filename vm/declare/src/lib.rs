extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, FnArg, ItemFn, Pat, PatType, Result};

#[proc_macro_attribute]
pub fn export_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemFn);
  process_export_method(input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn process_export_method(input: ItemFn) -> Result<TokenStream2> {
  let func_name = &input.sig.ident;
  let vis = &input.vis;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  let args: Vec<_> = inputs
    .iter()
    .filter_map(|arg| {
      if let FnArg::Typed(pat_type) = arg {
        Some(pat_type)
      } else {
        None
      }
    })
    .collect();

  let arg_names: Vec<_> = args
    .iter()
    .map(|arg| {
      let PatType { pat, .. } = arg;
      if let Pat::Ident(pat_ident) = &**pat {
        let ident = &pat_ident.ident;
        quote! { #ident }
      } else {
        quote! {}
      }
    })
    .collect();

  let export_func_name = format_ident!("athexpm_{}", func_name, span = Span::call_site());

  Ok(quote! {
    // all externally callable functions need entrypoint code
    athena_vm::entrypoint!(#export_func_name);

    #vis #input

    #[no_mangle]
    pub extern "C" fn #export_func_name(vm_state: *mut u8, vm_state_len: usize, #(#args),*) #output
    where
        Self: parity_scale_codec::Encode + parity_scale_codec::Decode,
    {
        unsafe {
            let state = std::slice::from_raw_parts(vm_state, vm_state_len);
            let mut program = Self::decode(&mut &state[..])
                .expect("Failed to deserialize VM state");
            let result = program.#func_name(#(#arg_names),*);

            // Serialize the updated state back to the VM
            let updated_state = program.encode();
            std::ptr::copy(updated_state.as_ptr(), vm_state, updated_state.len());

            result
        }
    }
  })
}

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemFn);
  let func_name = &input.sig.ident;
  let vis = &input.vis;
  let inputs = &input.sig.inputs;

  let export_func_name = format_ident!("athexp_{}", func_name);

  let output = quote! {
    // all externally callable functions need entrypoint code
    athena_vm::entrypoint!(#func_name);
    #vis #input

    #[no_mangle]
    pub extern "C" fn #export_func_name(#inputs) {
        Self::#func_name(#inputs)
    }
  };

  output.into()
}
