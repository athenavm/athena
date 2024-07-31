use athena_interface::Address;
use athena_vm_sdk::Pubkey;
use parity_scale_codec::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct SendArguments {
  pub recipient: Address,
  pub amount: u64,
}

#[derive(Encode, Decode)]
pub struct SpawnArguments {
  pub owner: Pubkey,
}

// The method selectors
#[derive(PartialEq)]
pub enum MethodId {
  Spawn = 0,
  Send = 1,
  Proxy = 2,
  Deploy = 3,
}
