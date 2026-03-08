use std::{fs, time::Duration};

use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use super::TestCase;

#[derive(Debug, Serialize, Deserialize)]
struct Problem {
  id: String,
  title: String,
  url: String,
}

pub async fn cmd_new(contest_name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

  let bin_dir = format!("{contest_name}/src/bin");
  fs::create_dir_all(&bin_dir)?;

  let client = Client::builder().user_agent("atcoder-rust (https://github.com/Kumasan00/atcoder-rust)").build()?;

  let tasks_url = format!("https://atcoder.jp/contests/{contest_name}/tasks");
  let response = client.get(tasks_url).send().await?.text().await?;

  let document = Html::parse_document(&response);

  let tbody_selector = Selector::parse("#main-container table tbody").unwrap();
  let tr_selector = Selector::parse("tr").unwrap();
  let td_selector = Selector::parse("td").unwrap();
  let a_selector = Selector::parse("a").unwrap();

  let mut problems = Vec::new();
  let tbody = document.select(&tbody_selector).next().unwrap();
  for tr in tbody.select(&tr_selector) {
    let mut tds = tr.select(&td_selector);
    let first_td = tds.next().unwrap();
    let second_td = tds.next().unwrap();
    let problem_id = first_td.text().collect::<String>().trim().to_string().to_lowercase();
    let problem_title = second_td.text().collect::<String>().trim().to_string();
    let problem_url = first_td.select(&a_selector).next().unwrap().value().attr("href").unwrap();
    let absolute_problem_url = format!("https://atcoder.jp{problem_url}");
    let problem = Problem {
      id: problem_id.clone(),
      title: problem_title,
      url: absolute_problem_url.clone(),
    };
    problems.push(problem);
  }

  let mut problems_meta = toml::value::Table::new();
  for problem in &problems {
    let mut entry = toml::value::Table::new();
    entry.insert("title".to_string(), toml::Value::String(problem.title.clone()));
    entry.insert("url".to_string(), toml::Value::String(problem.url.clone()));
    problems_meta.insert(problem.id.clone(), toml::Value::Table(entry));
  }
  let base_cargo_toml = format!(
    "[package]\nname = \"{contest_name}\"\nversion = \"0.1.0\"\nedition.workspace = true\nrust-version.workspace = true\n\n[dependencies]\n\n[lints]\nworkspace = true\n"
  );
  let mut cargo_toml_val: toml::Value = toml::from_str(&base_cargo_toml)?;
  cargo_toml_val
    .get_mut("package")
    .unwrap()
    .as_table_mut()
    .unwrap()
    .entry("metadata")
    .or_insert(toml::Value::Table(toml::value::Table::new()))
    .as_table_mut()
    .unwrap()
    .entry("atcoder-rust")
    .or_insert(toml::Value::Table(toml::value::Table::new()))
    .as_table_mut()
    .unwrap()
    .insert("problems".to_string(), toml::Value::Table(problems_meta));
  let cargo_toml_path = format!("{contest_name}/Cargo.toml");
  fs::write(&cargo_toml_path, toml::to_string_pretty(&cargo_toml_val)?)?;

  let test_case_folder = format!("{contest_name}/test_cases");
  fs::create_dir_all(&test_case_folder)?;

  for problem in problems {
    let url = problem.url;
    let id = problem.id;
    println!("【アクセス中】{url}");
    tokio::time::sleep(Duration::from_secs(1)).await;

    match client.get(&url).send().await {
      Ok(res) => match res.text().await {
        Ok(text) => {
          let test_cases = extract_io_examples(&text);
          let json_string = serde_json::to_string_pretty(&test_cases)?;

          let file_name = format!("{}/{}.json", &test_case_folder, id);
          fs::write(&file_name, json_string)?;
          println!("【成功】{file_name} にJSONデータを保存しました");

          let bin_file_name = format!("{contest_name}/src/bin/{id}.rs");

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
