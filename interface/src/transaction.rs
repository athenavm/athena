pub use parity_scale_codec::{Decode, Encode};

use crate::MethodSelector;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Payload {
  pub method: Option<MethodSelector>,
  pub args: Vec<u8>,
}

impl Payload {
  pub fn new<A>(method: Option<MethodSelector>, args: A) -> Self
  where
    A: Into<Vec<u8>>,
  {
    Self {
      method,
      args: args.into(),
    }
  }
}
