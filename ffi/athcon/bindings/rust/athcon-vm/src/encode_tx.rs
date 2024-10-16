//! Implements encoding athena method payloads

use athcon_sys as ffi;
use athena_interface::{
  payload::{Encode, Payload},
  MethodSelector,
};
use athena_vm_sdk::{encode_spawn, encode_spend, Pubkey, SpendArguments};

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
  let args = encode_spawn(Pubkey((*pubkey).bytes));
  let payload = Payload::new(Some(MethodSelector::from("athexp_spawn")), args);

  let (ptr, size) = crate::allocate_output_data(payload.encode());
  Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
}

/// Encode Athena Spend transaction payload.
///
/// # Arguments
/// - `state`: Account state.
/// - `recipient`: The address of the recipient.
/// - `amount`: The amount of tokens to send.
///
/// # Safety
/// The caller is responsible for freeing the returned bytes
/// via the `athcon_free_bytes` function.
#[no_mangle]
unsafe extern "C" fn athcon_encode_tx_spend(
  state: *const ffi::athcon_bytes,
  recipient: *const ffi::athcon_address,
  amount: u64,
) -> *mut ffi::athcon_bytes {
  let args = SpendArguments {
    recipient: (*recipient).bytes,
    amount,
  };
  let state = unsafe { state.as_ref() }.expect("account state must be provded");
  let args = encode_spend(state.as_slice().to_vec(), args);
  let payload = Payload::new(Some(MethodSelector::from("athexp_spend")), args);

  let (ptr, size) = crate::allocate_output_data(payload.encode());
  Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
}

// /// Encode Athena Verify() payload.
// ///
// /// # Arguments
// /// - `state`: Account state.
// /// - `tx`: The raw transaction bytes.
// /// - `signature`: The 64B signature of the transaction.
// ///
// /// # Safety
// /// The caller is responsible for freeing the returned bytes
// /// via the `athcon_free_bytes` function.
// #[no_mangle]
// unsafe extern "C" fn athcon_encode_verify_tx(
//   state: ffi::athcon_bytes,
//   tx: ffi::athcon_bytes,
//   signature: *const u8,
// ) -> *mut ffi::athcon_bytes {
//   let signature: &[u8; 64] = slice::from_raw_parts(signature, 64).try_into().unwrap();
//   let args = encode_verify(state.as_slice(), tx.as_slice(), signature);

//   let payload = Payload::new(Some(MethodSelector::from("athexp_verify")), args);

//   let (ptr, size) = crate::allocate_output_data(payload.encode());
//   Box::into_raw(Box::new(ffi::athcon_bytes { ptr, size }))
// }

#[cfg(test)]
mod tests {
  use athcon_sys as ffi;
  use athena_interface::{
    payload::{Decode, Payload},
    MethodSelector,
  };
  use athena_vm_sdk::{Pubkey, SpendArguments};
  use parity_scale_codec::{Encode, IoReader};

  use crate::encode_tx::{athcon_encode_tx_spawn, athcon_encode_tx_spend};

  #[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
  struct Program {
    pub owner: Pubkey,
  }

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
    let wallet = Program {
      owner: Pubkey([0x17; 32]),
    };
    let wallet_state = wallet.encode();

    let address = [0x1C; 24];
    let amount = 781237;
    let encoded_bytes = unsafe {
      athcon_encode_tx_spend(
        &ffi::athcon_bytes {
          ptr: wallet_state.as_ptr(),
          size: wallet_state.len(),
        },
        &ffi::athcon_address { bytes: address },
        amount,
      )
    };

    let mut encoded_slice = unsafe { (*encoded_bytes).as_slice() };
    let tx = Payload::decode(&mut encoded_slice).unwrap();
    unsafe { crate::bytes::athcon_free_bytes(encoded_bytes) };

    assert_eq!(tx.method, Some(MethodSelector::from("athexp_spend")));

    let mut input_reader = IoReader(tx.args.as_slice());
    let decoded_wallet = Program::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_wallet, wallet);

    let args = SpendArguments::decode(&mut input_reader).unwrap();
    assert_eq!(args.amount, amount);
    assert_eq!(args.recipient, address);
  }
}
