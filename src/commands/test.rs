use std::{
  fs,
  io::Write,
  process::{Command, Stdio},
};

use super::TestCase;

pub fn cmd_test(contest_name: &str, problem_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  // まずビルドする
  println!("【ビルド中】{contest_name}/{problem_name}");
  let build_status =
    Command::new("cargo").args(["build", "--package", contest_name, "--bin", problem_name]).status()?;
  if !build_status.success() {
    eprintln!("【ビルド失敗】");
    std::process::exit(1);
  }

  let json_path = format!("{contest_name}/test_cases/{problem_name}.json");
  let json_str =
    fs::read_to_string(&json_path).map_err(|_| format!("テストケースファイルが見つかりません: {json_path}"))?;
  let test_cases: Vec<TestCase> = serde_json::from_str(&json_str)?;

  if test_cases.is_empty() {
    println!("テストケースがありません。");
    return Ok(());
  }

  let binary = format!("./target/debug/{problem_name}");
  let mut passed = 0usize;
  let total = test_cases.len();

  for tc in &test_cases {
    let mut child = Command::new(&binary).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;

    child.stdin.as_mut().unwrap().write_all(tc.input.as_bytes())?;

    let output = child.wait_with_output()?;
    let actual = String::from_utf8_lossy(&output.stdout);
    let actual_trimmed = actual.trim();
    let expected_trimmed = tc.output.trim();

    if actual_trimmed == expected_trimmed {
      println!("【AC】{}", tc.name);
      passed += 1;
    } else {
      println!("【WA】{}", tc.name);
      println!("  期待: {expected_trimmed:?}");
      println!("  実際: {actual_trimmed:?}");
    }
  }

  println!("\n結果: {passed}/{total} 通過");
  if passed < total {
    std::process::exit(1);
  }

  Ok(())
}
