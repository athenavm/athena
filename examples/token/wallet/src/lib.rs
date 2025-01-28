use athena_interface::{Address, MethodSelector};
use parity_scale_codec::{Decode, Encode};

#[derive(Decode, Encode)]
pub struct ReceiveArguments {
  /// Token identifier allows identifing a token.
  /// It is the address of the mint that produced this token.
  pub token_identifier: Address,
  pub amount: u64,
}

#[derive(Decode, Encode)]
pub struct SpendArguments {
  /// Token identifier allows identifing a token.
  /// It is the address of the mint that produced this token.
  pub token_identifier: Address,
  pub recipient: Address,
  pub amount: u64,
}

pub const SELECTOR_RECEIVE: MethodSelector = MethodSelector([208, 20, 84, 100]);

#[cfg(test)]
mod tests {
  use athena_interface::MethodSelector;

  use crate::SELECTOR_RECEIVE;

  #[test]
  fn receive_method_selector() {
    assert_eq!(SELECTOR_RECEIVE, MethodSelector::from("athexp_receive"));
  }
}
