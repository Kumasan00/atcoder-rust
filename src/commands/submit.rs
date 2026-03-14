use std::process::Command;

use anyhow::Context;
use arboard::Clipboard;

pub fn cmd_submit(contest_name: &str, problem_name: &str) -> anyhow::Result<()> {
  println!("【提出】{contest_name}/{problem_name}");
  let source_path = format!("{contest_name}/src/bin/{problem_name}.rs");
  let source_code = std::fs::read_to_string(&source_path)
    .with_context(|| format!("提出するソースコードが見つかりません: {source_path}"))?;
  let mut clipboard = Clipboard::new().context("クリップボードの初期化に失敗しました")?;
  clipboard.set_text(source_code).context("ソースコードのコピーに失敗しました")?;

  Command::new("open")
    .arg(format!("https://atcoder.jp/contests/{contest_name}/submit"))
    .status()
    .context("提出ページを開けませんでした")?;
  Ok(())
}
