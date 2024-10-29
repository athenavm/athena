use anyhow::Result;
use athena_cli::{
  commands::{
    build::BuildCmd, build_toolchain::BuildToolchainCmd, execute::ExecuteCmd,
    install_toolchain::InstallToolchainCmd, new::NewCmd,
  },
  ATHENA_VERSION_MESSAGE,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum Cargo {
  Athena(AthenaCli),
}

#[derive(clap::Args)]
#[command(author, about, long_about = None, args_conflicts_with_subcommands = true, version = ATHENA_VERSION_MESSAGE)]
pub struct AthenaCli {
  #[clap(subcommand)]
  pub command: Option<AthenaCliCommands>,

  #[clap(flatten)]
  pub execute: ExecuteCmd,
}

#[derive(Subcommand)]
pub enum AthenaCliCommands {
  New(NewCmd),
  Build(BuildCmd),
  Execute(ExecuteCmd),
  BuildToolchain(BuildToolchainCmd),
  InstallToolchain(InstallToolchainCmd),
}

fn main() -> Result<()> {
  tracing_subscriber::fmt::init();

  let Cargo::Athena(args) = Cargo::parse();
  let command = args
    .command
    .unwrap_or(AthenaCliCommands::Execute(args.execute));
  match command {
    AthenaCliCommands::New(cmd) => cmd.run(),
    AthenaCliCommands::Build(cmd) => cmd.run(),
    AthenaCliCommands::Execute(cmd) => cmd.run(),
    AthenaCliCommands::BuildToolchain(cmd) => cmd.run(),
    AthenaCliCommands::InstallToolchain(cmd) => cmd.run(),
  }
}
