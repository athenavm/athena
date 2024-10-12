//! Test harness for the Wallet program.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --package wallet-script --bin execute --release
//! ```

use std::error::Error;

use athena_interface::{
  Address, AthenaContext, HostDynamicContext, HostInterface, HostStaticContext, MethodSelector,
  MethodSelectorAsString, MockHost, ADDRESS_ALICE, ADDRESS_BOB, ADDRESS_CHARLIE,
};
use athena_sdk::{AthenaStdin, ExecutionClient};
use athena_vm_sdk::{Pubkey, SendArguments};
use clap::Parser;
use parity_scale_codec::Encode;

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

fn spawn(host: &mut MockHost, owner: Pubkey) -> Result<Address, Box<dyn Error>> {
  let mut stdin = AthenaStdin::new();
  stdin.write(&owner.0);

  // calculate method selector
  let method_selector = MethodSelector::from(MethodSelectorAsString::new("athexp_spawn"));

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
  let address = spawn(&mut host, args.owner).expect("spawning wallet program");
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
    .expect("getting wallet program instance")
    .clone();

  stdin.write_vec(wallet);

  let args = SendArguments {
    recipient: ADDRESS_CHARLIE,
    amount: 10,
  };
  stdin.write_slice(&args.encode());

  let alice_balance = host.get_balance(&ADDRESS_ALICE);
  assert!(alice_balance >= 10);
  println!(
    "sending {} coins {} -> {}",
    args.amount,
    hex::encode(context.address()),
    hex::encode(args.recipient),
  );
  // calculate method selector
  let method_selector = MethodSelector::from(MethodSelectorAsString::new("athexp_send"));
  let (_, gas_cost) = ExecutionClient::new()
    .execute_function(
      ELF,
      &method_selector,
      stdin,
      Some(&mut host),
      Some(25000),
      Some(context.clone()),
    )
    .expect("sending coins");

  let new_alice_balance = host.get_balance(&ADDRESS_ALICE);
  let charlie_balance = host.get_balance(&ADDRESS_CHARLIE);
  println!(
    "sent coins at gas cost {}, balances: alice: {}, charlie: {}",
    gas_cost.unwrap_or_default(),
    new_alice_balance,
    charlie_balance
  );
  assert!(gas_cost.is_some());
  assert_eq!(charlie_balance, 10);
  assert_eq!(new_alice_balance, alice_balance - 10);
}

#[cfg(test)]
mod tests {
  use athena_interface::{Address, HostDynamicContext, HostStaticContext, MockHost, ADDRESS_ALICE};
  use athena_sdk::{AthenaStdin, ExecutionClient};
  use athena_vm_sdk::Pubkey;
  use parity_scale_codec::Encode;

  #[test]
  fn deploy_template() {
    tracing_subscriber::fmt()
      .with_test_writer()
      .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
      .init();

    let mut host = MockHost::new_with_context(
      HostStaticContext::new(ADDRESS_ALICE, 0, ADDRESS_ALICE),
      HostDynamicContext::new([0u8; 24], ADDRESS_ALICE),
    );
    let address = super::spawn(&mut host, Pubkey::default()).unwrap();

    // deploy other contract
    let code = b"some really bad code".to_vec();
    let mut stdin = AthenaStdin::new();
    let wallet_state = host.get_program(&address).unwrap();
    stdin.write_slice(wallet_state);
    stdin.write_vec(code.encode());

    let result = ExecutionClient::new().execute_function(
      super::ELF,
      "athexp_deploy",
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
}
