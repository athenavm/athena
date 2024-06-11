mod build;
pub mod commands;
mod util;

use anyhow::{Context, Result};
use reqwest::Client;
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
        self.stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .output()
            .with_context(|| format!("while executing `{:?}`", &self))
            .map(|_| ())
    }
}

pub async fn url_exists(client: &Client, url: &str) -> bool {
    let res = client.head(url).send().await;
    res.is_ok()
}

pub fn get_target() -> String {
    target_lexicon::HOST.to_string()
}

pub async fn get_toolchain_download_url(client: &Client, target: String) -> String {
    // Get latest tag and use it to construct the download URL.
    let json = client
        .get("https://api.github.com/repos/athenavm/rustc-rv32e-toolchain/releases/latest")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let tag = json["tag_name"].as_str().unwrap();

    let url = format!(
        "https://github.com/athenavm/rustc-rv32e-toolchain/releases/download/{}/athena-rust-toolchain-{}-{}.tar.gz",
        tag, target, tag
    );

    url
}
