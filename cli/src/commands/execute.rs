use anstyle::*;
use anyhow::Result;
use athena_core::io::AthenaStdin;
use athena_core::utils::{setup_logger, setup_tracer};
use athena_interface::MockHost;
use athena_sdk::ExecutionClient;
use clap::Parser;
use std::time::Instant;
use std::{env, fs::File, io::Read, path::PathBuf, str::FromStr};

use crate::{
  build::{build_program, BuildArgs},
  util::{elapsed, write_status},
};

#[derive(Debug, Clone)]
enum Input {
  FilePath(PathBuf),
  HexBytes(Vec<u8>),
}

fn is_valid_hex_string(s: &str) -> bool {
  if s.len() % 2 != 0 {
    return false;
  }
  // All hex digits with optional 0x prefix
  s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit())
    || s.chars().all(|c| c.is_ascii_hexdigit())
}

impl FromStr for Input {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if is_valid_hex_string(s) {
      // Remove 0x prefix if present
      let s = if s.starts_with("0x") {
        s.strip_prefix("0x").unwrap()
      } else {
        s
      };
      if s.is_empty() {
        return Ok(Input::HexBytes(Vec::new()));
      }
      if !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid hex string.".to_string());
      }
      let bytes = hex::decode(s).map_err(|e| e.to_string())?;
      Ok(Input::HexBytes(bytes))
    } else if PathBuf::from(s).exists() {
      Ok(Input::FilePath(PathBuf::from(s)))
    } else {
      Err("Input must be a valid file path or hex string.".to_string())
    }
  }
}

#[derive(Parser)]
#[command(name = "execute", about = "(default) Build and execute a program")]
pub struct ExecuteCmd {
  #[clap(long, value_parser)]
  input: Option<Input>,

  #[clap(long, action)]
  profile: bool,

  #[clap(long, action)]
  verbose: bool,

  #[clap(flatten)]
  build_args: BuildArgs,
}

impl ExecuteCmd {
  pub fn run(&self) -> Result<()> {
    let elf_path = build_program(&self.build_args)?;

    if !self.profile {
      match env::var("RUST_LOG") {
        Ok(_) => {}
        Err(_) => env::set_var("RUST_LOG", "info"),
      }
      setup_logger();
    } else {
      match env::var("RUST_TRACER") {
        Ok(_) => {}
        Err(_) => env::set_var("RUST_TRACER", "info"),
      }
      setup_tracer();
    }

    let mut elf = Vec::new();
    File::open(elf_path.as_path().as_str())
      .expect("failed to open input file")
      .read_to_end(&mut elf)
      .expect("failed to read from input file");

    let mut stdin = AthenaStdin::new();
    if let Some(ref input) = self.input {
      match input {
        Input::FilePath(ref path) => {
          let mut file = File::open(path).expect("failed to open input file");
          let mut bytes = Vec::new();
          file.read_to_end(&mut bytes)?;
          stdin.write_slice(&bytes);
        }
        Input::HexBytes(ref bytes) => {
          stdin.write_slice(bytes);
        }
      }
    }

    let start_time = Instant::now();
    let client = ExecutionClient::new();
    // no host interface needed for direct execution
    let (output, _gas_left) = client.execute::<MockHost>(&elf, stdin, None, 0).unwrap();

    let elapsed = elapsed(start_time.elapsed());
    let green = AnsiColor::Green.on_default().effects(Effects::BOLD);
    write_status(
      &green,
      "Finished",
      format!("executing in {}", elapsed).as_str(),
    );
    write_status(
      &green,
      "Finished",
      format!("received {} bytes output", output.as_slice().len()).as_str(),
    );

    Ok(())
  }
}
