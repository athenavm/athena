pub use parity_scale_codec::{Decode, Encode};

use crate::Address;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Transaction {
  pub nonce: u64,
  pub principal_account: Address,
  pub template: Option<Address>,
  pub method: Vec<u8>,
  pub args: Vec<u8>,
}

impl Transaction {
  pub fn new<M, A>(
    nonce: u64,
    principal_account: Address,
    template: Option<Address>,
    method: M,
    args: A,
  ) -> Self
  where
    M: Into<Vec<u8>>,
    A: Into<Vec<u8>>,
  {
    Self {
      nonce,
      principal_account,
      template,
      method: method.into(),
      args: args.into(),
    }
  }
}
