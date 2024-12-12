use anyhow::{Context, Result};
use clap::Parser;
use dirs::home_dir;
use indicatif::{ProgressBar, ProgressStyle};
use rand::{distributions::Alphanumeric, Rng};
use std::fs::{self};
use std::io::{Read, Write};
use std::process::Command;
use ureq::{MiddlewareNext, Request};

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

use crate::{get_toolchain_download_url, is_supported_target, RUSTUP_TOOLCHAIN_NAME};

#[derive(Parser)]
#[command(
  name = "install-toolchain",
  about = "Install the cargo-athena toolchain."
)]
pub struct InstallToolchainCmd {
  #[arg(short, long, env = "GITHUB_TOKEN")]
  pub token: Option<String>,
}

impl InstallToolchainCmd {
  pub fn run(&self) -> Result<()> {
    // Check if rust is installed.
    if Command::new("rustup")
      .arg("--version")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .status()
      .is_err()
    {
      return Err(anyhow::anyhow!(
        "Rust is not installed. Please install Rust from https://rustup.rs/ and try again."
      ));
    }

    // Setup client with optional token.
    let mut client_builder = ureq::AgentBuilder::new()
      .user_agent("Mozilla/5.0")
      .redirect_auth_headers(ureq::RedirectAuthHeaders::SameHost);
    if let Some(token) = &self.token {
      let value = format!("token {token}");
      client_builder = client_builder.middleware(move |req: Request, next: MiddlewareNext| {
        next.handle(req.set("Authorization", &value))
      });
    }
    let client = client_builder.build();

    // Setup variables.
    let root_dir = home_dir().unwrap().join(".athena");
    match fs::read_dir(&root_dir) {
      Ok(entries) =>
      {
        #[allow(clippy::manual_flatten)]
        for entry in entries {
          if let Ok(entry) = entry {
            let entry_path = entry.path();
            let entry_name = entry_path.file_name().unwrap();
            if entry_path.is_dir()
              && entry_name != "bin"
              && entry_name != "circuits"
              && entry_name != "toolchains"
            {
              if let Err(err) = fs::remove_dir_all(&entry_path) {
                println!("Failed to remove directory {:?}: {}", entry_path, err);
              }
            } else if entry_path.is_file() {
              if let Err(err) = fs::remove_file(&entry_path) {
                println!("Failed to remove file {:?}: {}", entry_path, err);
              }
            }
          }
        }
      }
      Err(_) => println!("No existing ~/.athena directory to remove."),
    }
    println!("Successfully cleaned up ~/.athena directory.");
    match fs::create_dir_all(&root_dir) {
      Ok(_) => println!("Successfully created ~/.athena directory."),
      Err(err) => println!("Failed to create ~/.athena directory: {}", err),
    };
    assert!(
      is_supported_target(),
      "Unsupported architecture. Please build the toolchain from source."
    );
    let target = target_lexicon::HOST.to_string();
    let toolchain_asset_name = format!("rust-toolchain-{target}.tar.gz");
    let toolchain_archive_path = root_dir.join(toolchain_asset_name.clone());
    let toolchain_dir = root_dir.join(&target);

    let toolchain_download_url = get_toolchain_download_url(&client, target.clone());
    client
      .head(&toolchain_download_url)
      .call()
      .with_context(|| {
        format!("checking availability for {target}. Your architecture might be unsupported.")
      })?;

    // Download the toolchain.
    let mut file = fs::File::create(toolchain_archive_path)?;
    download_file(&client, &toolchain_download_url, &mut file)?;

    // Remove the existing toolchain from rustup, if it exists.
    let mut child = Command::new("rustup")
      .current_dir(&root_dir)
      .args(["toolchain", "remove", RUSTUP_TOOLCHAIN_NAME])
      .stdout(std::process::Stdio::piped())
      .spawn()?;
    let res = child.wait();
    match res {
      Ok(_) => {
        let mut stdout = child.stdout.take().unwrap();
        let mut content = String::new();
        stdout.read_to_string(&mut content).unwrap();
        if !content.contains("no toolchain installed") {
          println!("Successfully removed existing toolchain.");
        }
      }
      Err(_) => println!("Failed to remove existing toolchain."),
    }

    // Unpack the toolchain.
    fs::create_dir_all(toolchain_dir.clone())?;
    Command::new("tar")
      .current_dir(&root_dir)
      .args([
        "-xzf",
        &toolchain_asset_name,
        "-C",
        &toolchain_dir.to_string_lossy(),
      ])
      .status()?;

    // Move the toolchain to a randomly named directory in the 'toolchains' folder
    let toolchains_dir = root_dir.join("toolchains");
    fs::create_dir_all(&toolchains_dir)?;
    let random_string: String = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(10)
      .map(char::from)
      .collect();
    let new_toolchain_dir = toolchains_dir.join(random_string);
    fs::rename(&toolchain_dir, &new_toolchain_dir)?;

    // Link the new toolchain directory to rustup
    Command::new("rustup")
      .current_dir(&root_dir)
      .args([
        "toolchain",
        "link",
        RUSTUP_TOOLCHAIN_NAME,
        &new_toolchain_dir.to_string_lossy(),
      ])
      .status()?;
    println!("Successfully linked toolchain to rustup.");

    // Ensure permissions.
    #[cfg(target_family = "unix")]
    {
      let bin_dir = new_toolchain_dir.join("bin");
      let rustlib_bin_dir = new_toolchain_dir.join(format!("lib/rustlib/{target}/bin"));
      for entry in fs::read_dir(bin_dir)?.chain(fs::read_dir(rustlib_bin_dir)?) {
        let entry = entry?;
        if entry.path().is_file() {
          let mut perms = entry.metadata()?.permissions();
          perms.set_mode(0o755);
          fs::set_permissions(entry.path(), perms)?;
        }
      }
    }

    Ok(())
  }
}

pub fn download_file(client: &ureq::Agent, url: &str, file: &mut fs::File) -> anyhow::Result<()> {
  let res = client
    .get(url)
    .call()
    .with_context(|| format!("getting '{url}'"))?;
  let total_size = res
    .header("Content-Length")
    .with_context(|| format!("getting content length from '{url}'"))?
    .parse::<u64>()?;

  let pb = ProgressBar::new(total_size);
  pb.set_style(ProgressStyle::default_bar()
      .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
      .progress_chars("#>-"));
  println!("Downloading {url}");

  let mut stream = res.into_reader();

  let mut buffer = vec![0; 8192];
  loop {
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
      break;
    }
    file.write_all(&buffer[..bytes_read])?;
    pb.inc(bytes_read as u64);
  }

  pb.finish();
  Ok(())
}
