//! Definitions of the reference wallet interface.

use athena_interface::Address;
use parity_scale_codec::{Decode, Encode};

use crate::Pubkey;

#[derive(Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq)]
pub struct SpendArguments {
  pub recipient: Address,
  pub amount: u64,
}

pub trait WalletProgram {
  fn spawn(owner: Pubkey) -> Address;
  fn spend(&self, args: SpendArguments);
  fn proxy(&self, destination: Address, args: &[u8]);
  fn deploy(&self, code: Vec<u8>) -> Address;
}

pub fn encode_spend(state: Vec<u8>, args: SpendArguments) -> Vec<u8> {
  let mut encoded = state;
  encoded.extend(args.encode());
  encoded
}

pub fn encode_spawn(pubkey: Pubkey) -> Vec<u8> {
  pubkey.encode()
}

pub fn encode_deploy(state: Vec<u8>, code: Vec<u8>) -> Vec<u8> {
  let mut encoded = state;
  encoded.extend(code.encode());
  encoded
}

#[cfg(test)]
mod tests {
  use parity_scale_codec::{Decode, Encode, IoReader};

  use super::SpendArguments;

  #[test]
  fn encode_decode_spend() {
    let wallet = Vec::<i32>::from([1, 2, 3, 4]);
    let wallet_state = wallet.encode();
    let args = SpendArguments {
      recipient: [22u8; 24],
      amount: 0,
    };
    let encoded = super::encode_spend(wallet_state, args);

    let mut input_reader = IoReader(encoded.as_slice());
    let decoded_wallet = Vec::<i32>::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_wallet, wallet);

    let decoded_args = SpendArguments::decode(&mut input_reader).unwrap();
    assert_eq!(decoded_args, args);
    assert!(input_reader.0.is_empty());
  }
}
