use parity_scale_codec::{Decode, Encode};

#[derive(Decode, Encode)]
pub struct ReceiveArguments {
  pub amount: u64,
}
