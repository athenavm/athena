use athena_interface::Address;
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
