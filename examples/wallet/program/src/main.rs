//! The Spacemesh standard wallet template.
#![no_main]
athena_vm::entrypoint!(main);

use athena_interface::{Address, Balance};
use athena_vm_declare::export;
use athena_vm_sdk::{call, Pubkey, VerifiableTemplate, WalletProgram, WalletTemplate};
use borsh::from_slice;
use borsh_derive::{BorshDeserialize, BorshSerialize};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
// use wallet_common::{SendArguments, SpawnArguments};

pub fn main() {}

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

impl WalletTemplate for Wallet {
  #[export]
  fn spawn(owner: Pubkey) {
    // for now this just tests the args
    Wallet::new(owner);

    // TODO: call spawn host function
  }
}

impl WalletProgram for Wallet {
  fn send(self, recipient: Address, amount: Balance) {
    // Send coins
    // Note: error checking happens inside the host
    call(recipient, None, amount);
  }

  fn proxy(self, _destination: Address, _args: Vec<u8>) {
    unimplemented!();
  }

  fn deploy(self, _code: Vec<u8>) {
    unimplemented!();
  }
}

impl VerifiableTemplate for Wallet {
  fn verify(self, tx: &[u8], signature: &[u8; 64]) -> bool {
    // Check that the transaction is signed by the owner
    let public_key = VerifyingKey::from_bytes(&self.owner).unwrap();
    let signature = Signature::from_bytes(signature);
    public_key.verify(&tx, &signature).is_ok()
  }
}
