use std::fs;

use anyhow::Context;
use arboard::Clipboard;

use super::error::CommandError;

pub fn cmd_submit(contest_name: &str, problem_name: &str) -> anyhow::Result<()> {
  println!("【提出】{contest_name}/{problem_name}");
  let source_path = format!("{contest_name}/src/bin/{problem_name}.rs");
  let source_code = std::fs::read_to_string(&source_path)
    .with_context(|| format!("提出するソースコードが見つかりません: {source_path}"))?;
  let mut clipboard = Clipboard::new().context("クリップボードの初期化に失敗しました")?;
  clipboard.set_text(source_code).context("ソースコードのコピーに失敗しました")?;

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
  let problem = problems.get(problem_name).ok_or_else(|| CommandError::ProblemNotFound(problem_name.to_string()))?;
  let problem_url = problem
    .get("url")
    .and_then(|u| u.as_str())
    .ok_or_else(|| CommandError::ProblemUrlMissing(problem_name.to_string()))?;
  open::that(problem_url)?;
  Ok(())
}
