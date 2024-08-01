extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, FnArg, ItemFn, PatType, Result};

#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemFn);
  process_export(input)
    .unwrap_or_else(Error::into_compile_error)
    .into()
}

fn process_export(input: ItemFn) -> Result<proc_macro2::TokenStream> {
  let func_name = &input.sig.ident;
  let inputs = &input.sig.inputs;
  let output = &input.sig.output;

  let is_instance_method = inputs.iter().any(|arg| matches!(arg, FnArg::Receiver(_)));

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
      quote! { #pat }
    })
    .collect();

  let export_func_name = format_ident!("athexp_{}", func_name);

  let generated_func = if is_instance_method {
    quote! {
      extern "C" fn #export_func_name(vm_state: *const u8, vm_state_len: usize, #(#args),*) #output {
          let state = unsafe { std::slice::from_raw_parts(vm_state, vm_state_len) };
          let mut program = from_slice::<Self>(&state).expect("Failed to deserialize VM state");
          program.#func_name(#(#arg_names),*)
      }
    }
  } else {
    quote! {
        #[no_mangle]
        pub extern "C" fn #export_func_name(#(#args),*) #output {
            Self::#func_name(#(#arg_names),*)
        }
    }
  };

  Ok(quote! {
      // #input

      #generated_func
  })
}
