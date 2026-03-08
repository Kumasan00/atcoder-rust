use std::{
  fs,
  io::Write,
  process::{Command, Stdio},
  time::Duration,
};

use clap::{Parser, Subcommand};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(about = "AtCoderのコンテスト問題をセットアップ・テストするツール")]
struct Args {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// コンテストのセットアップ (ディレクトリ作成・テストケース取得)
  New {
    /// コンテスト名 (例: `abs`, `abc123`)
    contest_name: String,
  },
  /// 保存済みテストケースで解答コードを検証する
  Test {
    /// コンテスト名 (例: `abs`, `abc123`)
    contest_name: String,
    /// 問題名 (例: `a`, `b`, `abc123_a`)
    problem_name: String,
  },
}

#[derive(Debug, Serialize, Deserialize)]
struct TestCase {
  name: String,
  input: String,
  output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  match args.command {
    Commands::New { contest_name } => {
      cmd_new(&contest_name).await?;
    },
    Commands::Test {
      contest_name,
      problem_name,
    } => {
      cmd_test(&contest_name, &problem_name)?;
    },
  }

  Ok(())
}

async fn cmd_new(contest_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let workspace_cargo_toml_path = "Cargo.toml";
  let workspace_cargo_toml = fs::read_to_string(workspace_cargo_toml_path)?;
  let mut cargo_toml: toml::Value = toml::from_str(&workspace_cargo_toml)?;
  let members = cargo_toml
    .get_mut("workspace")
    .and_then(|ws| ws.get_mut("members"))
    .and_then(|m| m.as_array_mut())
    .ok_or("Cargo.toml に [workspace] members が見つかりません")?;
  if members.iter().any(|v| v.as_str() == Some(contest_name)) {
    println!("{contest_name} はすでにワークスペースに存在しています。");
    return Ok(());
  }
  members.push(toml::Value::String(contest_name.to_string()));
  let updated_cargo_toml_string = toml::to_string_pretty(&cargo_toml)?;
  fs::write(workspace_cargo_toml_path, updated_cargo_toml_string)?;

  let template_file_path = "template.rs";
  if !std::path::Path::new(template_file_path).exists() {
    fs::write(template_file_path, "// ここにコードを書いてください\n")?;
    println!("テンプレートファイル {template_file_path} を作成しました。必要に応じて内容を編集してください。");
    return Ok(());
  }
  let template_content = fs::read_to_string(template_file_path)?;

  fs::create_dir_all(contest_name)?;

  let cargo_toml_string = format!(
    "[package]\nname = \"{contest_name}\"\nversion = \"0.1.0\"\nedition.workspace = true\nrust-version.workspace = true\n\n[dependencies]\n\n[lints]\nworkspace = true\n"
  );
  let cargo_toml_path = format!("{contest_name}/Cargo.toml");
  fs::write(&cargo_toml_path, cargo_toml_string)?;

  let bin_dir = format!("{contest_name}/src/bin");
  fs::create_dir_all(&bin_dir)?;

  let client = Client::builder().user_agent("atcoder-rust (https://github.com/Kumasan00/Kumasan00)").build()?;

  let tasks_url = format!("https://atcoder.jp/contests/{contest_name}/tasks");
  let response = client.get(tasks_url).send().await?.text().await?;

  let document = Html::parse_document(&response);

  let tbody_selector = Selector::parse("#main-container table tbody").unwrap();
  let tr_selector = Selector::parse("tr").unwrap();
  let td_selector = Selector::parse("td").unwrap();
  let a_selector = Selector::parse("a").unwrap();

  let mut problem_urls = Vec::new();
  let tbody = document.select(&tbody_selector).next().unwrap();
  for tr in tbody.select(&tr_selector) {
    let td = tr.select(&td_selector).next().unwrap();
    let problem_name = td.text().collect::<String>().trim().to_string().to_lowercase();
    let problem_url = td.select(&a_selector).next().unwrap().value().attr("href").unwrap();
    let absolute_problem_url = format!("https://atcoder.jp{problem_url}");
    problem_urls.push((problem_name, absolute_problem_url));
  }

  let test_case_folder = format!("{contest_name}/test_cases");
  fs::create_dir_all(&test_case_folder)?;

  for (name, url) in problem_urls {
    println!("【アクセス中】{url}");
    tokio::time::sleep(Duration::from_secs(1)).await;

    match client.get(&url).send().await {
      Ok(res) => match res.text().await {
        Ok(text) => {
          let test_cases = extract_io_examples(&text);
          let json_string = serde_json::to_string_pretty(&test_cases)?;

          let file_name = format!("{}/{}.json", &test_case_folder, name);
          fs::write(&file_name, json_string)?;
          println!("【成功】{file_name} にJSONデータを保存しました");

          let bin_file_name = format!("{contest_name}/src/bin/{name}.rs");

          fs::write(&bin_file_name, &template_content)?;
        },
        Err(e) => println!("【テキスト取得エラー】{url}: {e}"),
      },
      Err(e) => println!("【通信エラー】{url}: {e}"),
    }
  }

  Ok(())
}

fn extract_io_examples(html_text: &str) -> Vec<TestCase> {
  let document = Html::parse_document(html_text);

  // 必要なセレクタを準備
  let section_selector = Selector::parse("section").unwrap();
  let h3_selector = Selector::parse("h3").unwrap();
  let pre_selector = Selector::parse("pre").unwrap();

  let mut inputs = Vec::new();
  let mut outputs = Vec::new();

  for section in document.select(&section_selector) {
    if let Some(h3) = section.select(&h3_selector).next() {
      let title = h3.text().collect::<String>().trim().to_string();

      if let Some(pre) = section.select(&pre_selector).next() {
        let content = pre.text().collect::<String>();

        if title.contains("入力例") {
          inputs.push((title, content));
        } else if title.contains("出力例") {
          outputs.push(content);
        }
      }
    }
  }

  let mut test_cases = Vec::new();
  for (i, (name, input_text)) in inputs.into_iter().enumerate() {
    let output_text = outputs.get(i).cloned().unwrap_or_default();
    test_cases.push(TestCase {
      name,
      input: input_text,
      output: output_text,
    });
  }

  test_cases
}

fn cmd_test(contest_name: &str, problem_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  // まずビルドする
  println!("【ビルド中】{contest_name}/{problem_name}");
  let build_status = Command::new("cargo").args(["build", "--bin", problem_name]).status()?;
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
