use athena_interface::{Address, Balance, Bytes32, BYTES32_LENGTH};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "zkvm")]
mod spawn;
#[cfg(target_os = "zkvm")]
pub use spawn::spawn;

// This type needs to be serializable
#[derive(Clone, Copy, Debug, Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Pubkey(pub Bytes32);
pub const PUBKEY_LENGTH: usize = BYTES32_LENGTH;

pub fn call(address: Address, input: Option<Vec<u8>>, method: Option<Vec<u8>>, amount: Balance) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  // for now, require input to be word-aligned
  // we can pad the input but need to know more about the contents
  let input32 = if let Some(input) = input {
    assert!(input.len() % 4 == 0, "input is not 4 byte-aligned");

    let v = input
      .chunks(4)
      .map(|chunk| {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(chunk);
        u32::from_le_bytes(bytes)
      })
      .collect::<Vec<u32>>();
    Some(v)
  } else {
    None
  };
  let (input, input_len) = input32.map_or((std::ptr::null(), 0), |v| (v.as_ptr(), v.len()));

  let method32 = if let Some(method) = method {
    let mut v = method
      .chunks(4)
      .map(|chunk| {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(chunk);
        u32::from_le_bytes(bytes)
      })
      .collect::<Vec<u32>>();
    let len = v.len();
    // pad method name to 4-byte alignment
    if len % 4 != 0 {
      v.extend(std::iter::repeat(0).take(4 - len % 4));
    }
    Some(v)
  } else {
    None
  };
  let (method, method_len) = method32.map_or((std::ptr::null(), 0), |v| (v.as_ptr(), v.len()));

  athena_vm::syscalls::call(
    address.as_ptr(),
    input,
    input_len,
    method,
    method_len,
    amount.as_ptr(),
  );
}

// These traits define the reference wallet interface.

pub trait VerifiableTemplate {
  fn verify(&self, tx: &[u8], signature: &[u8; 64]) -> bool;
}

#[derive(Clone, Copy, Debug, Default, BorshDeserialize, BorshSerialize)]
pub struct SendArguments {
  pub recipient: Address,
  pub amount: u64,
}

pub trait WalletProgram {
  fn spawn(owner: Pubkey) -> Address;
  fn send(&self, args: SendArguments);
  fn proxy(&self, destination: Address, args: &[u8]);
  fn deploy(&self, code: &[u8]);
}
