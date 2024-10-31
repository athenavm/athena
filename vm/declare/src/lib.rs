extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ImplItem, ItemImpl, LitStr};

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

        // TODO: do we want to allow any args here at all?
        // or do we strictly want to allow input using io syscalls?
        let call = if is_static_method(&method.sig) {
          quote! {
            athena_vm::program::Function::call_func(#struct_name::#method_name, &mut athena_vm::io::Io::default())
          }
        } else {
          quote! {
            athena_vm::program::Method::call_method(#struct_name::#method_name, &mut athena_vm::io::Io::default())
          }
        };

        let section_name =
          LitStr::new(&format!(".text.athexp.{}", method_name), method_name.span());
        let static_name = format_ident!("DUMMY_{}", method_name.to_string().to_uppercase());

        c_functions.push(quote! {
          #[cfg(all(any(target_arch = "riscv32", target_arch = "riscv64"), target_feature = "e"))]
          #[link_section = #section_name]
          #[no_mangle]
          pub unsafe extern "C" fn #c_func_name() {
            #call;
            athena_vm::syscalls::syscall_halt(0);
          }

          // This black magic ensures the function symbol makes it into the final binary.
          #[used]
          #[link_section = ".init_array"]
          static #static_name: unsafe extern "C" fn() = #c_func_name;
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
  !matches!(sig.inputs.first(), Some(FnArg::Receiver(_)))
}

// Define the callable attribute
#[proc_macro_attribute]
pub fn callable(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // This attribute doesn't modify the item it's applied to,
  // it just marks it for processing by the template macro
  item
}
