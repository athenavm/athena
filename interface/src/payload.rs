pub use parity_scale_codec::{Decode, Encode};

use crate::MethodSelector;

/// Execution payload is passed as input to VM to execute a metod.
/// The optional state contains a serialized wallet state. It should be
/// empty if the call doesn't require any state.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, Eq)]
pub struct ExecutionPayload {
  pub state: Vec<u8>,
  pub payload: Payload,
}

impl From<ExecutionPayload> for Vec<u8> {
  fn from(value: ExecutionPayload) -> Self {
    value.encode()
  }
}

impl ExecutionPayload {
  /// Manually encode using an already encoded payload.
  /// Effectively: encode(state) | payload
  pub fn encode_with_encoded_payload<S, P>(state: S, payload: P) -> Vec<u8>
  where
    S: AsRef<[u8]>,
    P: AsRef<[u8]>,
  {
    // create a dummy instance to get a compilation error when `ExecutionPayload` is changed
    // and to remember to update this method.
    ExecutionPayload {
      state: vec![],
      payload: Payload::default(),
    };

    let mut encoded = state.as_ref().encode();
    encoded.extend_from_slice(payload.as_ref());
    encoded
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

  pub fn with_payload(mut self, payload: Payload) -> Self {
    self.payload.payload = payload;
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
  fn test_manual_encoding() {
    let exe_payload = ExecutionPayload {
      state: vec![1, 2, 3, 4, 5, 6, 7, 8],
      payload: Payload {
        selector: Some(MethodSelector::from("abcd")),
        input: vec![9, 8, 7, 6],
      },
    };

    let manually_encoded = ExecutionPayload::encode_with_encoded_payload(
      exe_payload.state.clone(),
      exe_payload.payload.encode(),
    );

    assert_eq!(manually_encoded, exe_payload.encode());
  }
}
