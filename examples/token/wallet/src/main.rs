#![no_main]

use athena_interface::{Address, MethodSelector};
use athena_vm::entrypoint;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::Pubkey;
use parity_scale_codec::{Decode, Encode};
use wallet::ReceiveArguments;

const BALANCE_KEY: [u32; 8] = [0u32; 8];

#[derive(Debug, Decode, Encode)]
struct Wallet {
  owner: Pubkey,
  mint: Address,
  template: Address,
}

athena_vm::entrypoint!();

#[template]
impl Wallet {
  #[callable]
  fn spawn(state: Wallet) -> Address {
    athena_vm_sdk::spawn(&state.encode())
  }

  #[callable]
  fn receive(&self, args: ReceiveArguments) {
    // 1. Only accept calls from:
    // - the mint (exchange)
    // - another wallet
    let ctx = athena_vm::syscalls::context::context();
    // FIXME: This breaks if the mint isn't a singleton...
    assert!(ctx.caller == self.mint || ctx.caller_template == self.template);

    // 2. increase the balance
    let mut balance_words = athena_vm::syscalls::read_storage(&[0u32; 8]);
    let mut balance = (balance_words[0] as u64) + ((balance_words[1] as u64) << 32);
    balance = balance.saturating_add(args.amount);
    balance_words[0] = balance as u32;
    balance_words[1] = (balance >> 32) as u32;
    athena_vm::syscalls::write_storage(&BALANCE_KEY, &balance_words);
  }

  #[callable]
  fn spend(&self, recipient: Address, amount: u64) {
    // 1. Decrease the balance
    let mut balance_words = athena_vm::syscalls::read_storage(&[0u32; 8]);
    let mut balance = (balance_words[0] as u64) + ((balance_words[1] as u64) << 32);
    balance = balance.saturating_sub(amount);
    balance_words[0] = balance as u32;
    balance_words[1] = (balance >> 32) as u32;
    athena_vm::syscalls::write_storage(&BALANCE_KEY, &balance_words);

    // 2. Call receive() on the other wallet
    // It will increase its balance.
    athena_vm_sdk::call(
      recipient,
      Some(wallet::ReceiveArguments { amount }.encode()),
      Some(MethodSelector::from("athexp_receive")),
      0,
    );
  }

  #[callable]
  fn verify(&self, tx: Vec<u8>, signature: [u8; 64]) -> bool {
    athena_vm_sdk::precompiles::ed25519::verify(&tx, &self.owner.0, &signature)
  }
}
