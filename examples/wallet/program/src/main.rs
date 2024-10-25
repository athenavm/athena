//! The Spacemesh standard wallet template.
#![no_main]
#![no_std]
extern crate alloc;

use athena_interface::Address;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::{call, spawn, Pubkey, SpendArguments, VerifiableTemplate, WalletProgram};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use parity_scale_codec::{Decode, Encode};
#[derive(Debug, Encode, Decode)]
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

#[cfg(all(
  any(target_arch = "riscv32", target_arch = "riscv64"),
  target_feature = "e"
))]
#[template]
impl WalletProgram for Wallet {
  #[callable]
  fn spawn(owner: Pubkey) -> Address {
    let wallet = Wallet::new(owner);
    let serialized = wallet.encode();
    spawn(serialized)
  }

  #[callable]
  fn spend(&self, args: SpendArguments) {
    // Send coins
    // Note: error checking happens inside the host
    call(args.recipient, None, None, args.amount);
  }

  fn proxy(&self, _destination: Address, _args: &[u8]) {
    unimplemented!();
  }

  #[callable]
  fn deploy(&self, code: alloc::vec::Vec<u8>) -> Address {
    athena_vm_sdk::deploy(code)
  }

  #[callable]
  fn max_spend(&self, args: SpendArguments) -> u64 {
    args.amount
  }
}

impl VerifiableTemplate for Wallet {
  fn verify(&self, tx: alloc::vec::Vec<u8>, signature: [u8; 64]) -> bool {
    // Check that the transaction is signed by the owner
    let public_key = VerifyingKey::from_bytes(&self.owner.0).unwrap();
    let signature = Signature::from_bytes(&signature);
    public_key.verify(&tx, &signature).is_ok()
  }
}

#[link_section = ".text.athexp.verify"]
#[no_mangle]
pub unsafe extern "C" fn athexp_verify() {
  athena_vm::program::Method::call_method(Wallet::verify, &mut athena_vm::io::Io::default());
  syscall_halt(0);
}
#[used]
#[link_section = ".init_array"]
static DUMMY_VERIFY: unsafe extern "C" fn() = athexp_verify;
