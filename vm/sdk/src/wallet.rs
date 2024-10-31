//! Definitions of the reference wallet interface.

use athena_interface::{payload::Payload, Address, MethodSelector};
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
  fn max_spend(&self, args: SpendArguments) -> u64;
}

pub fn encode_spend_inner(recipient: &Address, amount: u64) -> Vec<u8> {
  let args = SpendArguments {
    recipient: *recipient,
    amount,
  };
  args.encode()
}

pub fn encode_spend(recipient: &Address, amount: u64) -> Vec<u8> {
  let input = encode_spend_inner(recipient, amount);
  Payload::new(Some(MethodSelector::from("athexp_spend")), input).into()
}

pub fn encode_max_spend(recipient: &Address, amount: u64) -> Vec<u8> {
  let input = encode_spend_inner(recipient, amount);
  Payload::new(Some(MethodSelector::from("athexp_max_spend")), input).into()
}

pub fn encode_spawn(pubkey: &Pubkey) -> Vec<u8> {
  let payload = Payload::new(Some(MethodSelector::from("athexp_spawn")), pubkey.encode());
  payload.encode()
}

#[cfg(test)]
mod tests {
  use athena_interface::{payload::Payload, Address, MethodSelector};
  use parity_scale_codec::{Decode, IoReader};

  use super::SpendArguments;

  #[test]
  fn encode_decode_spend() {
    let args = SpendArguments {
      recipient: Address([22u8; 24]),
      amount: 800,
    };
    let encoded = super::encode_spend(&args.recipient, args.amount);

    let mut input_reader = IoReader(encoded.as_slice());
    let payload = Payload::decode(&mut input_reader).unwrap();
    assert_eq!(payload.selector, Some(MethodSelector::from("athexp_spend")));
    let decoded_args = SpendArguments::decode(&mut payload.input.as_slice()).unwrap();
    assert_eq!(decoded_args, args);
    assert!(input_reader.0.is_empty());
  }
}
