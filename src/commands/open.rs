use std::fs;

use anyhow::{Context, Result};

use super::error::CommandError;

pub fn cmd_open(contest_name: &str, problem_name: Option<&str>) -> Result<()> {
  let cargo_toml_path = format!("{contest_name}/Cargo.toml");
  let cargo_toml_str =
    fs::read_to_string(&cargo_toml_path).with_context(|| format!("Cargo.toml が見つかりません: {cargo_toml_path}"))?;
  let cargo_toml: toml::Value =
    toml::from_str(&cargo_toml_str).with_context(|| format!("{cargo_toml_path} の TOML 解析に失敗しました"))?;

  let problems = cargo_toml
    .get("package")
    .and_then(|p| p.get("metadata"))
    .and_then(|m| m.get("atcoder-rust"))
    .and_then(|a| a.get("problems"))
    .and_then(|p| p.as_table())
    .ok_or_else(|| CommandError::MissingProblemsMetadata(cargo_toml_path.clone()))?;

  let url = match problem_name {
    Some(name) => {
      let entry = problems.get(name).ok_or_else(|| CommandError::ProblemNotFound(name.to_string()))?;
      entry
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| CommandError::ProblemUrlMissing(name.to_string()))?
        .to_string()
    },
    None => {
      format!("https://atcoder.jp/contests/{contest_name}/tasks")
    },
  };

  println!("【オープン】{url}");
  open::that(&url)?;

  Ok(())
}
