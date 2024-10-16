//! Implements athcon_encode_tx
use athcon_sys as ffi;
use athena_interface::{
  payload::{Encode, Payload},
  MethodSelector,
};
use athena_vm_sdk::{Pubkey, SpendArguments};

/// Encode Athena Spawn transaction payload.
///
/// # Arguments
/// - `pubkey`: The public key of the owner.
///
/// # Safety
/// The caller is responsible for freeing the returned bytes
/// via the `athcon_free_bytes` function.
#[no_mangle]
unsafe extern "C" fn athcon_encode_tx_spawn(
  pubkey: *const ffi::athcon_bytes32,
) -> *mut ffi::athcon_bytes {
  let args = Pubkey((*pubkey).bytes);
  let payload = Payload::new(Some(MethodSelector::from("athexp_spawn")), args.encode());

  let (ptr, size) = crate::allocate_output_data(payload.encode());
  Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
}

/// Encode Athena Spend transaction payload.
///
/// # Arguments
/// - `recipient`: The address of the recipient.
/// - `amount`: The amount of tokens to send.
///
/// # Safety
/// The caller is responsible for freeing the returned bytes
/// via the `athcon_free_bytes` function.
#[no_mangle]
unsafe extern "C" fn athcon_encode_tx_spend(
  recipient: *const ffi::athcon_address,
  amount: u64,
) -> *mut ffi::athcon_bytes {
  let args = SpendArguments {
    recipient: (*recipient).bytes,
    amount,
  };
  let payload = Payload::new(Some(MethodSelector::from("athexp_spend")), args.encode());

  let (ptr, size) = crate::allocate_output_data(payload.encode());
  Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
}

#[cfg(test)]
mod tests {
  use athcon_sys as ffi;
  use athena_interface::{
    payload::{Decode, Payload},
    MethodSelector,
  };
  use athena_vm_sdk::{Pubkey, SpendArguments};

  use crate::encode_tx::{athcon_encode_tx_spawn, athcon_encode_tx_spend};

  #[test]
  fn encoding_spawn_tx() {
    let pubkey = Pubkey([1; 32]);
    let encoded_bytes = unsafe { athcon_encode_tx_spawn(&ffi::athcon_bytes32 { bytes: pubkey.0 }) };

    let mut encoded_slice = unsafe { (*encoded_bytes).as_slice() };
    let tx = Payload::decode(&mut encoded_slice).unwrap();
    unsafe { crate::bytes::athcon_free_bytes(encoded_bytes) };

    assert_eq!(tx.method, Some(MethodSelector::from("athexp_spawn")));
    assert_eq!(Pubkey::decode(&mut tx.args.as_slice()).unwrap(), pubkey);
  }
  #[test]
  fn encoding_spend_tx() {
    let address = [0x1C; 24];
    let amount = 781237;
    let encoded_bytes =
      unsafe { athcon_encode_tx_spend(&ffi::athcon_address { bytes: address }, amount) };

    let mut encoded_slice = unsafe { (*encoded_bytes).as_slice() };
    let tx = Payload::decode(&mut encoded_slice).unwrap();
    unsafe { crate::bytes::athcon_free_bytes(encoded_bytes) };

    assert_eq!(tx.method, Some(MethodSelector::from("athexp_spend")));
    let args = SpendArguments::decode(&mut tx.args.as_slice()).unwrap();
    assert_eq!(args.amount, amount);
    assert_eq!(args.recipient, address);
  }
}
