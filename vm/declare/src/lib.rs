extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, Pat};

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
                let obj = std::slice::from_raw_parts(vm_state, vm_state_len);
                let mut program = from_slice::<#struct_name>(&obj).expect("failed to deserialize program");
                program.#method_name(#(#args),*)
            },
          )
        };

        let all_params = self_param.into_iter().chain(params).collect::<Vec<_>>();

        c_functions.push(quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #c_func_name(#(#all_params),*) {
                #call
            }
        });
      }
    }
  }

  let output = quote! {
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
