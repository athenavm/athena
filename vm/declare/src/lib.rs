extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, LitStr, Pat};

#[proc_macro_attribute]
pub fn template(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemImpl);
  let struct_name = &input.self_ty;

  let mut c_functions = vec![];

  for item in &input.items {
    if let ImplItem::Fn(method) = item {
      if method
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("callable"))
      {
        let method_name = &method.sig.ident;
        let c_func_name = format_ident!("athexp_{}", method_name);

        let (params, args): (Vec<_>, Vec<_>) = method
          .sig
          .inputs
          .iter()
          .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
              if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let ident = &pat_ident.ident;
                let ty = &pat_type.ty;
                Some((quote!(#ident: #ty), quote!(#ident)))
              } else {
                None
              }
            } else {
              None
            }
          })
          .unzip();

        let (self_param, call) = if is_static_method(&method.sig) {
          (None, quote! { #struct_name::#method_name(#(#args),*) })
        } else {
          (
            Some(quote!(vm_state: *const u8, vm_state_len: usize)),
            quote! {
              let obj = core::slice::from_raw_parts(vm_state, vm_state_len);
              let mut program = from_slice::<#struct_name>(&obj).expect("failed to deserialize program");
              program.#method_name(#(#args),*)
            },
          )
        };

        let all_params = self_param.into_iter().chain(params).collect::<Vec<_>>();
        let section_name =
          LitStr::new(&format!(".text.athexp.{}", method_name), method_name.span());
        let static_name = format_ident!("DUMMY_{}", method_name.to_string().to_uppercase());

        c_functions.push(quote! {
          #[cfg(all(any(target_arch = "riscv32", target_arch = "riscv64"), target_feature = "e"))]
          #[link_section = #section_name]
          #[no_mangle]
          pub unsafe extern "C" fn #c_func_name(#(#all_params),*) {
            #call;
            syscall_halt(0);
          }

          // This black magic ensures the function symbol makes it into the final binary.
          #[used]
          #[link_section = ".init_array"]
          static #static_name: unsafe extern "C" fn(#(#all_params),*) = #c_func_name;
        });
      }
    }
  }

  let output = quote! {
      // Basic Athena preamble, for use without a main entrypoint.
      use athena_vm::syscalls::syscall_halt;
      athena_vm::entrypoint!();

      #input

      #(#c_functions)*
  };

  output.into()
}

fn is_static_method(sig: &syn::Signature) -> bool {
  match sig.inputs.first() {
    Some(FnArg::Receiver(_)) => false,
    _ => true,
  }
}

// Define the callable attribute
#[proc_macro_attribute]
pub fn callable(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // This attribute doesn't modify the item it's applied to,
  // it just marks it for processing by the template macro
  item
}
