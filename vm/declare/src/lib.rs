extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, FnArg, ItemFn, Pat, PatType, Result};

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemFn);
  process_export(input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn process_export(input: ItemFn) -> Result<TokenStream2> {
  let func_name = &input.sig.ident;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  let is_instance_method = inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)));

  let args: Vec<_> = inputs
    .iter()
    .filter_map(|arg| match arg {
      FnArg::Typed(pat_type) => Some(pat_type),
      _ => None,
    })
    .collect();

  let arg_names: Vec<_> = args
    .iter()
    .map(|arg| {
      let PatType { pat, .. } = arg;
      if let Pat::Ident(pat_ident) = pat.as_ref() {
        let ident = &pat_ident.ident;
        quote! { #ident }
      } else {
        quote! {}
      }
    })
    .collect();

  let export_func_name = format_ident!("athexp_{}", func_name, span = Span::call_site());

  if is_instance_method {
    Ok(quote! {
        #[no_mangle]
        extern "C" fn #export_func_name(vm_state: *const u8, vm_state_len: usize, #(#args),*) #output
        where
            Self: borsh::BorshDeserialize + borsh::BorshSerialize,
        {
            unsafe {
                let state = std::slice::from_raw_parts(vm_state, vm_state_len);
                let program = from_slice::<Self>(&state)
                    .expect("Failed to deserialize VM state");
                program.#func_name(#(#arg_names),*)
            }
        }
    })
  } else {
    Ok(quote! {
        #[no_mangle]
        extern "C" fn #export_func_name(#(#args),*) #output {
            Self::#func_name(#(#arg_names),*)
        }
    })
  }
}
