use std::{fs, process::Command};

pub fn cmd_open(contest_name: &str, problem_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
  let cargo_toml_path = format!("{contest_name}/Cargo.toml");
  let cargo_toml_str =
    fs::read_to_string(&cargo_toml_path).map_err(|_| format!("Cargo.toml が見つかりません: {cargo_toml_path}"))?;
  let cargo_toml: toml::Value = toml::from_str(&cargo_toml_str)?;

  let problems = cargo_toml
    .get("package")
    .and_then(|p| p.get("metadata"))
    .and_then(|m| m.get("atcoder-rust"))
    .and_then(|a| a.get("problems"))
    .and_then(|p| p.as_table())
    .ok_or(format!("{cargo_toml_path} に問題メタデータが見つかりません"))?;

  let url = match problem_name {
    Some(name) => {
      let entry = problems.get(name).ok_or(format!("問題 '{name}' が見つかりません"))?;
      entry
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or(format!("問題 '{name}' の URL が見つかりません"))?
        .to_string()
    },
    None => {
      format!("https://atcoder.jp/contests/{contest_name}/tasks")
    },
  };

  println!("【オープン】{url}");
  Command::new("open").arg(&url).status()?;

  Ok(())
}
