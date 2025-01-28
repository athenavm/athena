//! Multi-token wallet
//!
//! It keeps and allows spending various tokens
//! created by different mints. Every mint
//! instance creates a unique token, which
//! is identified by the address of the mint.

use athena_interface::Address;
use athena_vm_declare::{callable, template};
use athena_vm_sdk::Pubkey;
use parity_scale_codec::{Decode, Encode};
use wallet::{ReceiveArguments, SpendArguments};

#[derive(Debug, Decode, Encode)]
struct Wallet {
  owner: Pubkey,
  mint_template: Address,
  wallet_template: Address,
}

#[template]
impl Wallet {
  #[callable]
  fn spawn(state: Wallet) -> Address {
    athena_vm_sdk::spawn(&state.encode())
  }

  #[callable]
  fn receive(&self, args: ReceiveArguments) {
    // 1. Only accept calls from the following templates:
    // - the mint (exchange)
    // - another wallet (transfer)
    let ctx = athena_vm::syscalls::context::context();
    assert!(
      ctx.caller_template == self.mint_template || ctx.caller_template == self.wallet_template
    );

    // 2. increase the balance
    let balance_key = self.balance_key(&args.token_identifier);
    let mut balance_words = athena_vm::syscalls::read_storage(&balance_key);
    let mut balance = (balance_words[0] as u64) + ((balance_words[1] as u64) << 32);
    balance = balance.saturating_add(args.amount);
    balance_words[0] = balance as u32;
    balance_words[1] = (balance >> 32) as u32;
    athena_vm::syscalls::write_storage(&balance_key, &balance_words);
  }

  fn balance_key(&self, token_id: &Address) -> [u32; 8] {
    let mut balance_key = [0; 8];
    for (i, chunk) in token_id.0.chunks_exact(4).enumerate() {
      balance_key[i] = u32::from_le_bytes(chunk.try_into().unwrap())
    }
    balance_key
  }

  #[callable]
  fn send_token(&self, args: SpendArguments) {
    // 1. Decrease the balance
    let balance_key = self.balance_key(&args.token_identifier);
    let mut balance_words = athena_vm::syscalls::read_storage(&balance_key);
    let mut balance = (balance_words[0] as u64) + ((balance_words[1] as u64) << 32);
    assert!(args.amount <= balance);

    balance -= args.amount;
    balance_words[0] = balance as u32;
    balance_words[1] = (balance >> 32) as u32;
    athena_vm::syscalls::write_storage(&balance_key, &balance_words);

    // 2. Call receive() on the other wallet
    // It will increase its balance.
    athena_vm_sdk::call(
      args.recipient,
      Some(
        wallet::ReceiveArguments {
          token_identifier: args.token_identifier,
          amount: args.amount,
        }
        .encode(),
      ),
      Some(wallet::SELECTOR_RECEIVE),
      0,
    );
  }

  #[callable]
  fn spend(&self, args: athena_vm_sdk::wallet::SpendArguments) {
    athena_vm_sdk::call(args.recipient, None, None, args.amount);
  }

  #[callable]
  fn max_spend(&self, args: athena_vm_sdk::wallet::SpendArguments) -> u64 {
    args.amount
  }

  #[callable]
  fn verify(&self, tx: Vec<u8>, signature: [u8; 64]) -> bool {
    athena_vm_sdk::precompiles::ed25519::verify(&tx, &self.owner.0, &signature)
  }
}
