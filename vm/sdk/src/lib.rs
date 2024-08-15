use athena_hostfunctions;
use athena_interface::{Address, Balance, Bytes32, BYTES32_LENGTH};
use athena_vm::helpers::{address_to_32bit_words, balance_to_32bit_words};

pub type Pubkey = Bytes32;
pub const PUBKEY_LENGTH: usize = BYTES32_LENGTH;

pub fn call(address: Address, input: Option<Vec<u8>>, amount: Balance) {
  let address = address_to_32bit_words(address);
  let amount = balance_to_32bit_words(amount);

  // for now, require input to be word-aligned
  // we can pad the input but need to know more about the contents
  let (input, input_len) = if let Some(input) = input {
    assert!(input.len() % 4 == 0, "input is not byte-aligned");
    (
      input
        .chunks(4)
        .map(|chunk| {
          let mut bytes = [0u8; 4];
          bytes.copy_from_slice(chunk);
          u32::from_le_bytes(bytes)
        })
        .collect::<Vec<u32>>()
        .as_ptr(),
      input.len(),
    )
  } else {
    (std::ptr::null(), 0)
  };

  unsafe {
    athena_hostfunctions::call(address.as_ptr(), input, input_len, amount.as_ptr());
  }
}

// Template address is read from context
pub fn spawn(state_blob: Vec<u8>) {
  let state_blob_len = state_blob.len();
  let state_blob = state_blob
    .chunks(4)
    .map(|chunk| {
      let mut bytes = [0u8; 4];
      bytes.copy_from_slice(chunk);
      u32::from_le_bytes(bytes)
    })
    .collect::<Vec<u32>>()
    .as_ptr();

  unsafe {
    athena_hostfunctions::spawn(state_blob, state_blob_len);
  }
}

// These traits define the reference wallet interface.

pub trait VerifiableTemplate {
  fn verify(&self, tx: &[u8], signature: &[u8; 64]) -> bool;
}

pub trait WalletProgram {
  fn spawn(owner_blob: *const u8);
  fn send(&self, send_arguments_blob: *const u8, send_arguments_blob_len: usize);
  fn proxy(&self, destination: Address, args: &[u8]);
  fn deploy(&self, code: &[u8]);
}
