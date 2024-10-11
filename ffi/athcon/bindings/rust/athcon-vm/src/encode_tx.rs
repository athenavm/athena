//! Implements athcon_encode_tx
use athcon_sys as ffi;
use athena_interface::transaction::{Encode, Transaction};

/// Encode Athena Transaction into bytes.
///
/// # Arguments
/// - `principal` - The transaction's principal account.
/// - `template` - Optional transaction's template account.
/// - (`method`, `method_size`) - Optional method to call
/// - `nonce` - The transaction's nonce
/// - (`args`, `args_size`) - Optional transaction's serialized arguments.
///
/// # Safety
/// - the caller is responsible for freeing the returned vector
///   via the `athcon_free_vector` function.
#[no_mangle]
unsafe extern "C" fn athcon_encode_tx(
  principal: *const ffi::athcon_address,
  template: *const ffi::athcon_address,
  method: *const u8,
  method_size: usize,
  nonce: u64,
  args: *const u8,
  args_size: usize,
) -> *mut ffi::athcon_vector {
  let principal = &(*principal).bytes;
  let template_o = template.as_ref().map(|t| t.bytes);
  let method = if method.is_null() {
    &[]
  } else {
    std::slice::from_raw_parts(method, method_size)
  };
  let args = if args.is_null() {
    &[]
  } else {
    std::slice::from_raw_parts(args, args_size)
  };

  let tx = Transaction::new(nonce, *principal, template_o, method, args);

  Box::into_raw(Box::new(ffi::athcon_vector::from_vec(tx.encode())))
}

#[cfg(test)]
mod tests {
  use athcon_sys as ffi;
  use athena_interface::transaction::Encode;

  #[test]
  fn encoding_tx() {
    let tx = athena_interface::transaction::Transaction::new(
      42,
      [12; 24],
      Some([34; 24]),
      vec![5, 6, 7],
      [8, 9, 0],
    );

    let encoded = tx.encode();

    // encode via the C interface
    let principal = ffi::athcon_address::from(tx.principal_account);
    let template = tx.template.map(ffi::athcon_address::from);
    let template_ptr = template
      .as_ref()
      .map(|a| a as *const _)
      .unwrap_or(std::ptr::null());
    let encoded_vec = unsafe {
      super::athcon_encode_tx(
        &principal as *const _,
        template_ptr,
        tx.method.as_ptr(),
        tx.method.len(),
        tx.nonce,
        tx.args.as_ptr(),
        tx.args.len(),
      )
    };

    let encoded_slice = unsafe { (*encoded_vec).as_slice() };
    assert_eq!(encoded_slice, encoded);
    unsafe { crate::vec::athcon_free_vector(encoded_vec) };
  }

  #[test]
  fn encoding_without_method_and_args() {
    let tx = athena_interface::transaction::Transaction::new(42, [12; 24], None, [], []);
    let encoded = tx.encode();

    // encode via the C interface
    let principal = ffi::athcon_address::from(tx.principal_account);
    let encoded_vec = unsafe {
      super::athcon_encode_tx(
        &principal as *const _,
        std::ptr::null(),
        std::ptr::null(),
        0,
        tx.nonce,
        std::ptr::null(),
        0,
      )
    };

    let encoded_slice = unsafe { (*encoded_vec).as_slice() };
    assert_eq!(encoded_slice, encoded);
    unsafe { crate::vec::athcon_free_vector(encoded_vec) };
  }
}
