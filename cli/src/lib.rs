pub mod commands;
mod util;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::process::{Command, Stdio};

pub const RUSTUP_TOOLCHAIN_NAME: &str = "athena";

pub const ATHENA_VERSION_MESSAGE: &str = concat!(
  "athena",
  " (",
  env!("VERGEN_GIT_SHA"),
  " ",
  env!("VERGEN_BUILD_TIMESTAMP"),
  ")"
);

trait CommandExecutor {
  fn run(&mut self) -> Result<()>;
}

impl CommandExecutor for Command {
  fn run(&mut self) -> Result<()> {
    self
      .stderr(Stdio::inherit())
      .stdout(Stdio::inherit())
      .stdin(Stdio::inherit())
      .output()
      .with_context(|| format!("while executing `{:?}`", &self))
      .and_then(|output| {
        if output.status.success() {
          Ok(())
        } else {
          Err(anyhow::anyhow!(
            "Command failed with exit code: {}",
            output.status
          ))
        }
      })
  }
}

#[allow(unreachable_code)]
pub fn is_supported_target() -> bool {
  #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
  return true;

  #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
  return true;

  #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
  return true;

  #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
  return true;

  false
}

#[tracing::instrument(skip(client))]
pub fn get_toolchain_download_url(client: &Client, target: String) -> String {
  // Get latest tag and use it to construct the download URL.
  let json = client
    .get("https://api.github.com/repos/athenavm/rustc-rv32e-toolchain/releases/latest")
    .send()
    .unwrap()
    .json::<serde_json::Value>()
    .unwrap();
  tracing::debug!(%json, "got latest release response");
  let tag = json["tag_name"].as_str().expect(
    "Failed to download toolchain. Likely caused by GitHub rate limiting. Please try again.",
  );

  let url = format!(
        "https://github.com/athenavm/rustc-rv32e-toolchain/releases/download/{}/athena-rust-toolchain-{}-{}.tar.gz",
        tag, target, tag
    );

  url
}
