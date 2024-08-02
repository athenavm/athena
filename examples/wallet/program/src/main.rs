//! The Spacemesh standard wallet template.
#![no_main]
athena_vm::entrypoint!(main);

use athena_interface::{Address, Balance};
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{call, Pubkey, VerifiableTemplate, WalletProgram};
use borsh::{from_slice, to_vec};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
// use std::hint::black_box;
// use wallet_common::{SendArguments, SpawnArguments};

pub fn main() {
  let wallet = Wallet::new([0u8; 32]);
  Wallet::spawn([0u8; 32]);
  wallet.send([0u8; 24], 0);
  wallet.verify(&[0u8; 32], &[0u8; 64]);
  unsafe { athexp_spawn([0u8; 32]) };
  let serialized = to_vec(&wallet).unwrap();
  let (state, statelen) = (serialized.as_ptr(), serialized.len());
  unsafe { athexp_send(state, statelen, [0u8; 24], 0) };
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Wallet {
  nonce: u64,
  balance: u64,
  owner: Pubkey,
}

impl Wallet {
  fn new(owner: Pubkey) -> Self {
    Wallet {
      nonce: 0,
      balance: 0,
      owner,
    }
  }
}

#[template]
impl WalletProgram for Wallet {
  #[callable]
  fn spawn(owner: Pubkey) {
    // for now this just tests the args
    Wallet::new(owner);

    // TODO: call spawn host function
  }

  #[callable]
  fn send(&self, recipient: Address, amount: Balance) {
    // Send coins
    // Note: error checking happens inside the host
    call(recipient, None, amount);
  }

  fn proxy(&self, _destination: Address, _args: Vec<u8>) {
    unimplemented!();
  }

  fn deploy(&self, _code: Vec<u8>) {
    unimplemented!();
  }
}

impl VerifiableTemplate for Wallet {
  fn verify(&self, tx: &[u8], signature: &[u8; 64]) -> bool {
    // Check that the transaction is signed by the owner
    let public_key = VerifyingKey::from_bytes(&self.owner).unwrap();
    let signature = Signature::from_bytes(signature);
    public_key.verify(&tx, &signature).is_ok()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_wallet() {
    let wallet = Wallet::new([0u8; 32]);
    Wallet::spawn([0u8; 32]);
    wallet.send([0u8; 24], 0);
    unsafe { athexp_spawn([0u8; 32]) };
    let serialized = to_vec(&wallet).unwrap();
    let (state, statelen) = (serialized.as_ptr(), serialized.len());
    unsafe { athexp_send(state, statelen, [0u8; 24], 0) };
  }
}
