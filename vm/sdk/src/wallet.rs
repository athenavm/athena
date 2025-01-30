//! Definitions of the reference wallet interface.

use athena_interface::{Address, MethodSelector};
use parity_scale_codec::{Decode, Encode};

use crate::Pubkey;

#[derive(Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq)]
pub struct SpendArguments {
  pub recipient: Address,
  pub amount: u64,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct ProxyArguments {
  pub destination: Address,
  pub method: Option<MethodSelector>,
  pub args: Option<Vec<u8>>,
  pub amount: u64,
}

pub trait WalletProgram {
  fn spawn(owner: Pubkey) -> Address;
  fn spend(&self, args: SpendArguments);
  fn proxy(&self, args: ProxyArguments) -> Vec<u8>;
  fn deploy(&self, code: Vec<u8>) -> Address;
  fn max_spend(&self, args: SpendArguments) -> u64;
}
