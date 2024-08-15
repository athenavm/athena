use anyhow::{Context, Result};
use clap::Parser;
use core::str;
use std::{path::PathBuf, process::Command};

use crate::{get_target, CommandExecutor, RUSTUP_TOOLCHAIN_NAME};

#[derive(Parser)]
#[command(name = "build-toolchain", about = "Build the cargo-athena toolchain.")]
pub struct BuildToolchainCmd {}

impl BuildToolchainCmd {
  pub fn run(&self) -> Result<()> {
    // Get enviroment variables.
    let github_access_token = std::env::var("GITHUB_ACCESS_TOKEN");
    let build_dir = std::env::var("ATHENA_BUILD_DIR");

    // Clone our rust fork, if necessary.
    let toolchain_dir = match build_dir {
      Ok(build_dir) => {
        println!("Detected ATHENA_BUILD_DIR, skipping cloning rust-toolchain.");
        PathBuf::from(build_dir)
      }
      Err(_) => {
        let temp_dir = std::env::temp_dir();
        let dir = temp_dir.join("athena-toolchain");
        if dir.exists() {
          std::fs::remove_dir_all(&dir)?;
        }

        println!("No ATHENA_BUILD_DIR detected, cloning rust-toolchain.");
        let repo_url = match github_access_token.clone() {
          Ok(github_access_token) => {
            println!("Detected GITHUB_ACCESS_TOKEN, using it to clone rust.");
            format!(
              "https://{}@github.com/athenavm/rustc-rv32e-toolchain",
              github_access_token
            )
          }
          Err(_) => {
            println!("No GITHUB_ACCESS_TOKEN detected. If you get throttled by Github, set it to bypass the rate limit.");
            "ssh://git@github.com/athenavm/rustc-rv32e-toolchain".to_string()
          }
        };
        Command::new("git")
          .args(["clone", &repo_url, "athena-toolchain"])
          .current_dir(&temp_dir)
          .run()?;
        dir
      }
    };
    let rust_repo_url = match github_access_token {
      Ok(github_access_token) => {
        println!("Detected GITHUB_ACCESS_TOKEN, using it to clone rust.");
        format!(
          "https://{}@github.com/rust-lang/rust.git",
          github_access_token
        )
      }
      Err(_) => {
        println!("No GITHUB_ACCESS_TOKEN detected. If you get throttled by Github, set it to bypass the rate limit.");
        "https://github.com/rust-lang/rust.git".to_string()
      }
    };

    let rust_dir = toolchain_dir.join("rust");
    if !rust_dir.exists() {
      Command::new("git")
        .args(["clone", &rust_repo_url, "--depth=1"])
        .current_dir(&toolchain_dir)
        .run()?;
    }

    // Read the Rust commit hash
    let rust_commit = std::fs::read_to_string(toolchain_dir.join("rust_commit.txt"))?
      .trim()
      .to_string();

    Command::new("git")
      .args(["fetch", "--depth=1", "origin", &rust_commit])
      .current_dir(&rust_dir)
      .run()?;
    Command::new("git")
      .args(["checkout", "FETCH_HEAD"])
      .current_dir(&rust_dir)
      .run()?;
    Command::new("git")
      .args([
        "submodule",
        "update",
        "--init",
        "--recursive",
        "--depth=1",
        "--progress",
      ])
      .current_dir(&rust_dir)
      .run()?;

    // Install our config.toml.
    let ci = std::env::var("CI").unwrap_or("false".to_string()) == "true";
    let config_file_src = if ci {
      "patches/config.ci.toml"
    } else {
      "patches/config.toml"
    };
    std::fs::copy(
      toolchain_dir.join(config_file_src),
      rust_dir.join("config.toml"),
    )
    .with_context(|| {
      format!(
        "while copying configuration from {:?} to {:?}",
        toolchain_dir.join(config_file_src),
        rust_dir.join("config.toml")
      )
    })?;

    // Apply patches
    // We allow this to fail, but want to warn the user if it did.
    let patch_output = Command::new("patch")
      .args([
        "-f",
        "-N",
        "-p1",
        "-i",
        toolchain_dir.join("patches/rust.patch").to_str().unwrap(),
      ])
      .current_dir(&rust_dir)
      .output()
      .expect("Failed to run patch command");
    if !patch_output.status.success() {
      let stderr = str::from_utf8(&patch_output.stderr).unwrap_or("Failed to read stderr");
      println!("Failed to apply patches to rust with code: {:?}. This is expected if the patches have already been applied. Error output: {}", patch_output.status.code(), stderr);
    }

    // Create the custom target file.
    // Note: Rust doesn't actually read this file, it just needs to see that it exists
    // to get past the bootstrap phase.
    Command::new("touch")
      .arg(toolchain_dir.join("riscv32em-athena-zkvm-elf.json"))
      .run()?;

    // Build the toolchain (stage 1).
    Command::new("python3")
      .env(
        "CFLAGS_riscv32em_athena_zkvm_elf",
        "-ffunction-sections -fdata-sections -fPIC -target riscv32-unknown-elf",
      )
      .env(
        "CARGO_TARGET_RISCV32EM_ATHENA_ZKVM_ELF_RUSTFLAGS",
        "-Cpasses=loweratomic -Clink-arg=-march=rv32em -Clink-arg=-mabi=ilp32e",
      )
      .env("COMPILER_RT_DEFAULT_TARGET_TRIPLE", "riscv32-unknown-elf")
      .env("CC_riscv32em_athena_zkvm_elf", "clang")
      .env("CXX_riscv32em_athena_zkvm_elf", "clang++")
      .env("RUSTC_TARGET_ARG", "")
      .env("RUST_TARGET_PATH", &toolchain_dir)
      .args(["x.py", "build"])
      .current_dir(&rust_dir)
      .run()?;

    // Build the toolchain (stage 2).
    Command::new("python3")
      .env(
        "CFLAGS_riscv32em_athena_zkvm_elf",
        "-ffunction-sections -fdata-sections -fPIC -target riscv32-unknown-elf",
      )
      .env(
        "CARGO_TARGET_RISCV32EM_ATHENA_ZKVM_ELF_RUSTFLAGS",
        "-Cpasses=loweratomic -Clink-arg=-march=rv32em -Clink-arg=-mabi=ilp32e",
      )
      .env("COMPILER_RT_DEFAULT_TARGET_TRIPLE", "riscv32-unknown-elf")
      .env("CC_riscv32em_athena_zkvm_elf", "clang")
      .env("CXX_riscv32em_athena_zkvm_elf", "clang++")
      .env("RUSTC_TARGET_ARG", "")
      .env("RUST_TARGET_PATH", &toolchain_dir)
      .args(["x.py", "build", "--stage", "2"])
      .current_dir(&rust_dir)
      .run()?;

    // Remove the existing toolchain from rustup, if it exists.
    match Command::new("rustup")
      .args(["toolchain", "remove", RUSTUP_TOOLCHAIN_NAME])
      .run()
    {
      Ok(_) => println!("Successfully removed existing toolchain."),
      Err(_) => println!("No existing toolchain to remove."),
    }

    // Find the toolchain directory.
    let mut toolchain_dir = None;
    for wentry in std::fs::read_dir(rust_dir.join("build"))? {
      let entry = wentry?;
      let toolchain_dir_candidate = entry.path().join("stage2");
      if toolchain_dir_candidate.is_dir() {
        toolchain_dir = Some(toolchain_dir_candidate);
        break;
      }
    }
    let toolchain_dir = toolchain_dir.expect("Missing Rust toolchain directory");
    println!(
      "Found built toolchain directory at {}.",
      toolchain_dir.as_path().to_str().unwrap()
    );

    // Copy over the stage2-tools-bin directory to the toolchain bin directory.
    let tools_bin_dir = toolchain_dir.parent().unwrap().join("stage2-tools-bin");
    let target_bin_dir = toolchain_dir.join("bin");
    for tool in tools_bin_dir.read_dir()? {
      let tool = tool?;
      let tool_name = tool.file_name();
      std::fs::copy(tool.path(), target_bin_dir.join(tool_name))?;
    }

    // Link the toolchain to rustup.
    Command::new("rustup")
      .args(["toolchain", "link", RUSTUP_TOOLCHAIN_NAME])
      .arg(&toolchain_dir)
      .run()?;
    println!("Successfully linked the toolchain to rustup.");

    // Compressing toolchain directory to tar.gz.
    let target = get_target();
    let tar_gz_path = format!("rust-toolchain-{}.tar.gz", target);
    Command::new("tar")
      .args([
        "--exclude",
        "lib/rustlib/src",
        "--exclude",
        "lib/rustlib/rustc-src",
        "-hczvf",
        &tar_gz_path,
        "-C",
        toolchain_dir.to_str().unwrap(),
        ".",
      ])
      .run()?;
    println!("Successfully compressed the toolchain to {}.", tar_gz_path);

    Ok(())
  }
}
