use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
pub mod error;
pub mod init;
pub mod new;
pub mod open;
pub mod submit;
pub mod test;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
  pub name: String,
  pub input: String,
  pub output: String,
}

fn build_client() -> Result<reqwest::blocking::Client> {
  Client::builder()
    .user_agent("atcoder-rust (https://github.com/Kumasan00/atcoder-rust)")
    .build()
    .context("HTTP クライアントの初期化に失敗しました")
}
