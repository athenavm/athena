use anyhow::Result;
use clap::Parser;
use std::{fs, path::Path, process::Command};
use yansi::Paint;

#[derive(Parser)]
#[command(
  name = "new",
  about = "Setup a new project that runs inside the Athena VM."
)]
pub struct NewCmd {
  /// The name of the project.
  name: String,

  /// Version of athena-project-template to use (branch or tag).
  #[arg(long, default_value = "main")]
  version: String,
}

const TEMPLATE_REPOSITORY_URL: &str = "https://github.com/athenavm/athena-project-template";

impl NewCmd {
  pub fn run(&self) -> Result<()> {
    let root = Path::new(&self.name);

    // Create the root directory if it doesn't exist.
    if !root.exists() {
      fs::create_dir(&self.name)?;
    }

    // Clone the repository with the specified version.
    let output = Command::new("git")
      .arg("clone")
      .arg("--branch")
      .arg(&self.version)
      .arg(TEMPLATE_REPOSITORY_URL)
      .arg(root.as_os_str())
      .arg("--recurse-submodules")
      .arg("--depth=1")
      .output()
      .expect("failed to execute command");
    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(anyhow::anyhow!("failed to clone repository: {}", stderr));
    }

    // Remove the .git directory.
    fs::remove_dir_all(root.join(".git"))?;

    println!(
      "    \x1b[1m{}\x1b[0m {} ({})",
      Paint::green("Initialized"),
      self.name,
      std::fs::canonicalize(root)
        .expect("failed to canonicalize")
        .to_str()
        .unwrap()
    );

    Ok(())
  }
}
