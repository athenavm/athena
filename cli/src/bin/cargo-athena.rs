use anyhow::Result;
use clap::{Parser, Subcommand};
use athena_cli::{
    commands::{
        build::BuildCmd, build_toolchain::BuildToolchainCmd,
        install_toolchain::InstallToolchainCmd, new::NewCmd, execute::ExecuteCmd,
    },
    ATHENA_VERSION_MESSAGE,
};

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
    let Cargo::Athena(args) = Cargo::parse();
    let command = args.command.unwrap_or(AthenaCliCommands::Execute(args.execute));
    match command {
        AthenaCliCommands::New(cmd) => cmd.run(),
        AthenaCliCommands::Build(cmd) => cmd.run(),
        AthenaCliCommands::Execute(cmd) => cmd.run(),
        AthenaCliCommands::BuildToolchain(cmd) => cmd.run(),
        AthenaCliCommands::InstallToolchain(cmd) => cmd.run(),
    }
}
