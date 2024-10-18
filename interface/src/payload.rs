pub use parity_scale_codec::{Decode, Encode};

use crate::MethodSelector;

/// Execution payload is passed as input to VM to execute a metod.
/// The optional state contains a serialized wallet state. It should be
/// None if the call doesn't require any state.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, Eq)]
pub struct ExecutionPayload {
  pub state: Vec<u8>,
  pub payload: Vec<u8>,
}

impl From<ExecutionPayload> for Vec<u8> {
  fn from(value: ExecutionPayload) -> Self {
    value.encode()
  }
}

#[derive(Default)]
pub struct ExecutionPayloadBuilder {
  payload: ExecutionPayload,
}

impl ExecutionPayloadBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with_state<P: Into<Vec<u8>>>(mut self, state: P) -> Self {
    self.payload.state = state.into();
    self
  }

  pub fn with_payload<P: Into<Vec<u8>>>(mut self, payload: P) -> Self {
    self.payload.payload = payload.into();
    self
  }

  pub fn build(self) -> ExecutionPayload {
    self.payload
  }
}

/// Payload included in the transaction.
/// It contains an optional method selector (if needed),
/// and optional program/function arguments.
#[derive(Debug, Default, Clone, Encode, Decode, PartialEq, Eq)]
pub struct Payload {
  pub selector: Option<MethodSelector>,
  pub input: Vec<u8>,
}

impl Payload {
  pub fn new<I>(selector: Option<MethodSelector>, input: I) -> Self
  where
    I: Into<Vec<u8>>,
  {
    Self {
      selector,
      input: input.into(),
    }
  }
}

impl From<Payload> for Vec<u8> {
  fn from(value: Payload) -> Self {
    value.encode()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encode_decode_payload() {
    let payload = ExecutionPayload::default();
    let encoded = payload.encode();
    dbg!(encoded);
  }
}
