//! Test harness for the Wallet program.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --package wallet-script --bin execute --release
//! ```

use std::error::Error;

use athena_interface::{
  Address, AthenaContext, Encode, HostDynamicContext, HostInterface, HostStaticContext,
  MethodSelector, MockHost, ADDRESS_ALICE, ADDRESS_BOB, ADDRESS_CHARLIE,
};
use athena_sdk::{AthenaStdin, ExecutionClient};
use athena_vm_sdk::{Pubkey, SpendArguments};
use clap::Parser;

/// The ELF (executable and linkable format) file for the Athena RISC-V VM.
///
/// This file is generated by running `cargo athena build` inside the `program` directory.
pub const ELF: &[u8] = include_bytes!("../../../program/elf/wallet-template");

/// The arguments for the run command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct RunArgs {
  #[arg(
    long,
    default_value = "000dfb23b0979b4b000000000000000000000000000000000000000000000000",
    value_parser(parse_owner)
  )]
  owner: Pubkey,
}

fn parse_owner(data: &str) -> Result<Pubkey, hex::FromHexError> {
  let mut key = Pubkey::default();
  hex::decode_to_slice(data, &mut key.0 as &mut [u8])?;
  Ok(key)
}

fn spawn(host: &mut MockHost, owner: &Pubkey) -> Result<Address, Box<dyn Error>> {
  let mut stdin = AthenaStdin::new();
  stdin.write_vec(owner.encode());

  let method_selector = MethodSelector::from("athexp_spawn");

  let client = ExecutionClient::new();
  let (mut result, _) =
    client.execute_function(ELF, &method_selector, stdin, Some(host), None, None)?;

  Ok(result.read())
}

fn main() {
  tracing_subscriber::fmt::init();

  let args = RunArgs::parse();

  let mut host = MockHost::new_with_context(
    HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
    HostDynamicContext::new([0u8; 24], ADDRESS_ALICE),
  );
  let address = spawn(&mut host, &args.owner).expect("spawning wallet program");
  println!(
    "spawned a wallet program at {} for {}",
    hex::encode(address),
    hex::encode(args.owner.0),
  );

  // send some coins
  let context = AthenaContext::new(ADDRESS_ALICE, ADDRESS_BOB, 0);

  let mut stdin = AthenaStdin::new();
  let wallet = host
    .get_program(&address)
    .expect("getting wallet program instance");

  let args = SpendArguments {
    recipient: ADDRESS_CHARLIE,
    amount: 120,
  };

  stdin.write_vec(wallet.clone());
  stdin.write_vec(args.encode());

  let alice_balance = host.get_balance(&ADDRESS_ALICE);
  assert!(alice_balance >= 120);
  println!(
    "sending {} coins {} -> {}",
    args.amount,
    hex::encode(context.address()),
    hex::encode(args.recipient),
  );
  // calculate method selector
  let method_selector = MethodSelector::from("athexp_spend");
  let max_gas = 25000;
  let (_, gas_left) = ExecutionClient::new()
    .execute_function(
      ELF,
      &method_selector,
      stdin,
      Some(&mut host),
      Some(max_gas),
      Some(context.clone()),
    )
    .expect("sending coins");

  let new_alice_balance = host.get_balance(&ADDRESS_ALICE);
  let charlie_balance = host.get_balance(&ADDRESS_CHARLIE);
  println!(
    "sent coins at gas cost {}, balances: alice: {}, charlie: {}",
    max_gas - gas_left.unwrap_or_default(),
    new_alice_balance,
    charlie_balance
  );
  assert!(gas_left.is_some());
  assert_eq!(charlie_balance, 120);
  assert_eq!(new_alice_balance, alice_balance - 120);
}

#[cfg(test)]
mod tests {
  use athena_interface::{
    Address, Encode, HostDynamicContext, HostStaticContext, MethodSelector, MockHost, ADDRESS_ALICE,
  };
  use athena_sdk::{AthenaStdin, ExecutionClient};
  use athena_vm_sdk::Pubkey;

  fn setup_logger() {
    let _ = tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .try_init();
  }

  #[test]
  fn deploy_template() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new([0u8; 24], ADDRESS_ALICE),
    );
    let address = super::spawn(&mut host, &Pubkey::default()).unwrap();

    // deploy other contract
    let code = b"some really bad code".to_vec();
    let mut stdin = AthenaStdin::new();
    let wallet_state = host.get_program(&address).unwrap();
    stdin.write_vec(wallet_state.clone());
    stdin.write_vec(code.encode());

    let selector = MethodSelector::from("athexp_deploy");
    let result = ExecutionClient::new().execute_function(
      super::ELF,
      &selector,
      stdin.clone(),
      Some(&mut host),
      Some(25000000),
      None,
    );
    let (mut result, gas_cost) = result.unwrap();
    assert!(gas_cost.is_some());

    let address: Address = result.read();
    let template = host.template(&address);
    assert_eq!(template, Some(&code));
  }

  #[test]
  fn maxspend() {
    setup_logger();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new([0u8; 24], ADDRESS_ALICE),
    );
    let address = super::spawn(&mut host, Pubkey::default()).unwrap();

    let wallet = host.get_program(&address).unwrap();
    let args = athena_vm_sdk::SpendArguments {
      recipient: Address::default(),
      amount: 100,
    };

    let mut stdin = AthenaStdin::new();
    stdin.write_vec(athena_vm_sdk::encode_spend(wallet.clone(), args));

    let result = ExecutionClient::new().execute_function(
      super::ELF,
      "athexp_maxspend",
      stdin,
      Some(&mut host),
      Some(25000),
      None,
    );
    let (mut result, gas_cost) = result.unwrap();
    assert!(gas_cost.is_some());

    let max_spend: u64 = result.read();
    assert_eq!(max_spend, args.amount);
  }
}
