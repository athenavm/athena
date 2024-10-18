//! Implements encoding athena method payloads

use athcon_sys as ffi;
use athena_vm_sdk::{encode_spawn, encode_spend, encode_verify, Pubkey};

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
  if pubkey.is_null() {
    return std::ptr::null_mut();
  }
  let payload = encode_spawn(&Pubkey((*pubkey).bytes));

  let (ptr, size) = crate::allocate_output_data(payload);
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
  if recipient.is_null() {
    return std::ptr::null_mut();
  }
  let recipient = unsafe { &(*recipient).bytes };
  let payload = encode_spend(recipient, amount);

  let (ptr, size) = crate::allocate_output_data(payload);
  Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
}

/// Encode Athena Verify() payload.
///
/// # Arguments
/// - `state`: Account state.
/// - `tx`: The raw transaction bytes.
/// - `signature`: The 64B signature of the transaction.
///
/// # Safety
/// The caller is responsible for freeing the returned bytes
/// via the `athcon_free_bytes` function.
#[no_mangle]
unsafe extern "C" fn athcon_encode_verify_tx(
  state: *const ffi::athcon_bytes,
  tx: *const ffi::athcon_bytes,
  signature: *const [u8; 64],
) -> *mut ffi::athcon_bytes {
  if state.is_null() || tx.is_null() || signature.is_null() {
    return std::ptr::null_mut(); // Return NULL if inputs are invalid
  }
  let state = unsafe { &*state }.as_slice().to_vec();
  let tx = unsafe { &*tx }.as_slice();
  let signature = unsafe { &*signature };

  let args = encode_verify(state, tx, signature);
  let payload = athena_interface::payload::Payload::new(
    Some(athena_interface::MethodSelector::from("athexp_verify")),
    args,
  );

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
  use parity_scale_codec::IoReader;

  use crate::encode_tx::{athcon_encode_tx_spawn, athcon_encode_tx_spend, athcon_encode_verify_tx};

  #[test]
  fn encoding_spawn_tx() {
    let pubkey = Pubkey([1; 32]);
    let encoded_bytes = unsafe { athcon_encode_tx_spawn(&ffi::athcon_bytes32 { bytes: pubkey.0 }) };

    let mut encoded_slice = unsafe { (*encoded_bytes).as_slice() };
    let tx = Payload::decode(&mut encoded_slice).unwrap();
    unsafe { crate::bytes::athcon_free_bytes(encoded_bytes) };

    assert_eq!(tx.selector, Some(MethodSelector::from("athexp_spawn")));
    assert_eq!(Pubkey::decode(&mut tx.input.as_slice()).unwrap(), pubkey);
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

    assert_eq!(tx.selector, Some(MethodSelector::from("athexp_spend")));

    let mut input_reader = IoReader(tx.input.as_slice());
    let args = SpendArguments::decode(&mut input_reader).unwrap();
    assert_eq!(args.amount, amount);
    assert_eq!(args.recipient, address);
  }

  #[test]
  fn encoding_verify_tx() {
    let wallet = Program {
      owner: Pubkey([0x17; 32]),
    };
    let wallet_state = wallet.encode();
    let tx = vec![0x01, 0x02, 0x03];
    let signature = [0x01; 64];
    let encoded_bytes = unsafe {
      athcon_encode_verify_tx(
        &ffi::athcon_bytes::from(wallet_state.as_slice()),
        &ffi::athcon_bytes::from(tx.as_slice()),
        &signature,
      )
    };

    let mut encoded_slice = unsafe { (*encoded_bytes).as_slice() };
    let payload = Payload::decode(&mut encoded_slice).unwrap();
    unsafe { crate::bytes::athcon_free_bytes(encoded_bytes) };

    assert_eq!(payload.method, Some(MethodSelector::from("athexp_verify")));

    let mut input_reader = IoReader(payload.args.as_slice());
    let decoded_wallet = Program::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_wallet, wallet);
    let decoded_tx = Vec::<u8>::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_tx, tx);
    let decoded_signature = <[u8; 64]>::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_signature, signature);

    assert!(input_reader.0.is_empty());
  }
}
