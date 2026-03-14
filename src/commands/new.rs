use std::{
  fs,
  path::{Path, PathBuf},
  time::Duration,
};

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use super::{TestCase, error::CommandError};

#[derive(Debug, Clone)]
struct Problem {
  id: String,
  title: String,
  url: String,
  time_limit: String,
  memory_limit: String,
}

pub fn cmd_new(contest_name: &str) -> Result<()> {
  let workspace_cargo_toml_path = Path::new("Cargo.toml");
  if workspace_member_exists(workspace_cargo_toml_path, contest_name)? {
    println!("{contest_name} はすでにワークスペースに存在しています。");
    return Ok(());
  }
  let template_content = load_or_create_template(Path::new("template.rs"))?;
  let contest_root = PathBuf::from(contest_name);
  create_contest_layout(&contest_root)?;

  let client = build_http_client()?;
  let (contest_title, problems) = fetch_contest_problems(&client, contest_name)?;
  if problems.is_empty() {
    return Err(CommandError::NoProblemsFetched.into());
  }

  write_contest_cargo_toml(&contest_root, contest_name, &contest_title, &problems)?;
  write_problem_sources(&contest_root, &problems, &template_content)?;
  fetch_and_write_test_cases(&client, &contest_root, &problems)?;
  add_workspace_member(workspace_cargo_toml_path, contest_name)?;

  Ok(())
}

fn workspace_member_exists(workspace_toml_path: &Path, contest_name: &str) -> Result<bool> {
  let workspace_cargo_toml = fs::read_to_string(workspace_toml_path)
    .with_context(|| format!("{} の読み込みに失敗しました", workspace_toml_path.display()))?;
  let cargo_toml: toml::Value = toml::from_str(&workspace_cargo_toml)
    .with_context(|| format!("{} の TOML 解析に失敗しました", workspace_toml_path.display()))?;
  let members = workspace_members(&cargo_toml)?;
  Ok(members.iter().any(|v| v.as_str() == Some(contest_name)))
}

fn add_workspace_member(workspace_toml_path: &Path, contest_name: &str) -> Result<()> {
  let workspace_cargo_toml = fs::read_to_string(workspace_toml_path)
    .with_context(|| format!("{} の読み込みに失敗しました", workspace_toml_path.display()))?;
  let mut cargo_toml: toml::Value = toml::from_str(&workspace_cargo_toml)
    .with_context(|| format!("{} の TOML 解析に失敗しました", workspace_toml_path.display()))?;
  let members = workspace_members_mut(&mut cargo_toml)?;
  members.push(toml::Value::String(contest_name.to_string()));

  let updated_cargo_toml_string =
    toml::to_string_pretty(&cargo_toml).context("workspace Cargo.toml の整形に失敗しました")?;
  fs::write(workspace_toml_path, updated_cargo_toml_string)
    .with_context(|| format!("{} の書き込みに失敗しました", workspace_toml_path.display()))?;
  Ok(())
}

fn workspace_members(cargo_toml: &toml::Value) -> Result<&Vec<toml::Value>> {
  cargo_toml
    .get("workspace")
    .and_then(|ws| ws.get("members"))
    .and_then(toml::Value::as_array)
    .ok_or_else(|| CommandError::WorkspaceMembersMissing.into())
}

fn workspace_members_mut(cargo_toml: &mut toml::Value) -> Result<&mut Vec<toml::Value>> {
  cargo_toml
    .get_mut("workspace")
    .and_then(|ws| ws.get_mut("members"))
    .and_then(toml::Value::as_array_mut)
    .ok_or_else(|| CommandError::WorkspaceMembersMissing.into())
}

fn load_or_create_template(template_path: &Path) -> Result<String> {
  if !template_path.exists() {
    fs::write(template_path, "// ここにコードを書いてください\n")
      .with_context(|| format!("{} の作成に失敗しました", template_path.display()))?;
    println!(
      "テンプレートファイル {} を作成しました。必要に応じて内容を編集してください。",
      template_path.display()
    );
  }
  fs::read_to_string(template_path).with_context(|| format!("{} の読み込みに失敗しました", template_path.display()))
}

fn create_contest_layout(contest_root: &Path) -> Result<()> {
  fs::create_dir_all(contest_root.join("src/bin"))
    .with_context(|| format!("{} の作成に失敗しました", contest_root.join("src/bin").display()))?;
  fs::create_dir_all(contest_root.join("test_cases"))
    .with_context(|| format!("{} の作成に失敗しました", contest_root.join("test_cases").display()))?;
  Ok(())
}

fn build_http_client() -> Result<Client> {
  Client::builder()
    .user_agent("atcoder-rust (https://github.com/Kumasan00/atcoder-rust)")
    .build()
    .context("HTTP クライアントの初期化に失敗しました")
}

fn fetch_contest_problems(client: &Client, contest_name: &str) -> Result<(String, Vec<Problem>)> {
  let tasks_url = format!("https://atcoder.jp/contests/{contest_name}/tasks");
  let response = client
    .get(&tasks_url)
    .send()
    .with_context(|| format!("問題一覧ページの取得に失敗しました: {tasks_url}"))?
    .text()
    .with_context(|| format!("問題一覧ページの本文取得に失敗しました: {tasks_url}"))?;
  parse_problems_from_tasks_html(&response)
}

fn parse_problems_from_tasks_html(html: &str) -> Result<(String, Vec<Problem>)> {
  let document = Html::parse_document(html);
  let a_contest_title_selector = parse_selector("a.contest-title")?;
  let tbody_selector = parse_selector("#main-container table tbody")?;
  let tr_selector = parse_selector("tr")?;
  let td_selector = parse_selector("td")?;
  let a_selector = parse_selector("a")?;

  let contest_title = document
    .select(&a_contest_title_selector)
    .next()
    .map(|element| element.text().collect::<String>().trim().to_string())
    .filter(|title| !title.is_empty())
    .ok_or_else(|| anyhow!("大会名が見つかりません"))?;

  let tbody = document.select(&tbody_selector).next().ok_or_else(|| anyhow!("問題一覧テーブルが見つかりません"))?;

  let mut problems = Vec::new();
  for tr in tbody.select(&tr_selector) {
    let mut tds = tr.select(&td_selector);
    let Some(first_td) = tds.next() else {
      continue;
    };
    let Some(second_td) = tds.next() else {
      continue;
    };
    let Some(third_td) = tds.next() else {
      continue;
    };
    let Some(fourth_td) = tds.next() else {
      continue;
    };

    let Some(problem_anchor) = first_td.select(&a_selector).next() else {
      continue;
    };
    let Some(problem_url) = problem_anchor.value().attr("href") else {
      continue;
    };

    let problem_id = first_td.text().collect::<String>().trim().to_string().to_lowercase();
    let problem_title = second_td.text().collect::<String>().trim().to_string();
    let problem_time_limit = third_td.text().collect::<String>().trim().to_string();
    let problem_memory_limit = fourth_td.text().collect::<String>().trim().to_string();
    problems.push(Problem {
      id: problem_id,
      title: problem_title,
      url: format!("https://atcoder.jp{problem_url}"),
      time_limit: problem_time_limit,
      memory_limit: problem_memory_limit,
    });
  }

  Ok((contest_title, problems))
}

fn write_contest_cargo_toml(
  contest_root: &Path,
  contest_name: &str,
  contest_title: &str,
  problems: &[Problem],
) -> Result<()> {
  let mut problems_meta = toml::value::Table::new();
  for problem in problems {
    let mut entry = toml::value::Table::new();
    entry.insert("title".to_string(), toml::Value::String(problem.title.clone()));
    entry.insert("url".to_string(), toml::Value::String(problem.url.clone()));
    entry.insert("time_limit".to_string(), toml::Value::String(problem.time_limit.clone()));
    entry.insert("memory_limit".to_string(), toml::Value::String(problem.memory_limit.clone()));
    problems_meta.insert(problem.id.clone(), toml::Value::Table(entry));
  }

  let base_cargo_toml = format!(
    "[package]\nname = \"{contest_name}\"\nversion = \"0.1.0\"\nedition.workspace = true\n\n[dependencies]\n\n[lints]\nworkspace = true\n"
  );
  let mut cargo_toml_val: toml::Value = toml::from_str(&base_cargo_toml)?;
  let package_table = cargo_toml_val
    .get_mut("package")
    .and_then(toml::Value::as_table_mut)
    .ok_or(CommandError::InvalidContestCargoToml("[package]"))?;
  let metadata = package_table
    .entry("metadata")
    .or_insert_with(|| toml::Value::Table(toml::value::Table::new()))
    .as_table_mut()
    .ok_or(CommandError::InvalidContestCargoToml("metadata"))?;
  let atcoder_rust = metadata
    .entry("atcoder-rust")
    .or_insert_with(|| toml::Value::Table(toml::value::Table::new()))
    .as_table_mut()
    .ok_or(CommandError::InvalidContestCargoToml("atcoder-rust metadata"))?;
  let mut contest_meta = toml::value::Table::new();
  contest_meta.insert("name".to_string(), toml::Value::String(contest_title.to_string()));
  contest_meta.insert("url".to_string(), toml::Value::String(format!("https://atcoder.jp/contests/{contest_name}")));
  atcoder_rust.insert("contest".to_string(), toml::Value::Table(contest_meta));
  atcoder_rust.insert("problems".to_string(), toml::Value::Table(problems_meta));

  fs::write(
    contest_root.join("Cargo.toml"),
    toml::to_string_pretty(&cargo_toml_val).context("contest Cargo.toml の整形に失敗しました")?,
  )
  .with_context(|| format!("{} の書き込みに失敗しました", contest_root.join("Cargo.toml").display()))?;
  Ok(())
}

fn write_problem_sources(contest_root: &Path, problems: &[Problem], template_content: &str) -> Result<()> {
  for problem in problems {
    let source_path = contest_root.join("src/bin").join(format!("{}.rs", problem.id));
    fs::write(&source_path, template_content)
      .with_context(|| format!("{} の書き込みに失敗しました", source_path.display()))?;
  }
  Ok(())
}

fn fetch_and_write_test_cases(client: &Client, contest_root: &Path, problems: &[Problem]) -> Result<()> {
  let total_problems = problems.len();
  for (idx, problem) in problems.iter().enumerate() {
    println!("【アクセス中】{}", problem.url);
    let response = match client.get(&problem.url).send() {
      Ok(res) => res,
      Err(e) => {
        println!("【通信エラー】{}: {e}", problem.url);
        continue;
      },
    };

    let text = match response.text() {
      Ok(text) => text,
      Err(e) => {
        println!("【テキスト取得エラー】{}: {e}", problem.url);
        continue;
      },
    };

    let test_cases = parse_test_cases_from_problem_html(&text)
      .with_context(|| format!("テストケース解析に失敗しました: {}", problem.url))?;
    let file_name = contest_root.join("test_cases").join(format!("{}.json", problem.id));
    fs::write(
      &file_name,
      serde_json::to_string_pretty(&test_cases)
        .with_context(|| format!("テストケース JSON の整形に失敗しました: {}", problem.id))?,
    )
    .with_context(|| format!("{} への書き込みに失敗しました", file_name.display()))?;
    println!("【成功】{} にJSONデータを保存しました", file_name.display());

    if idx + 1 < total_problems {
      std::thread::sleep(Duration::from_secs(1));
    }
  }

  Ok(())
}

fn parse_test_cases_from_problem_html(html: &str) -> Result<Vec<TestCase>> {
  let document = Html::parse_document(html);
  let section_selector = parse_selector("section")?;
  let h3_selector = parse_selector("h3")?;
  let pre_selector = parse_selector("pre")?;

  let mut inputs = Vec::new();
  let mut outputs = Vec::new();

  for section in document.select(&section_selector) {
    let Some(h3) = section.select(&h3_selector).next() else {
      continue;
    };
    let Some(pre) = section.select(&pre_selector).next() else {
      continue;
    };

    let title = h3.text().collect::<String>().trim().to_string();
    let content = pre.text().collect::<String>();
    if title.contains("入力例") {
      inputs.push((title, content));
    } else if title.contains("出力例") {
      outputs.push(content);
    }
  }

  let mut test_cases = Vec::new();
  for (idx, (name, input_text)) in inputs.into_iter().enumerate() {
    let output_text = outputs.get(idx).cloned().unwrap_or_default();
    test_cases.push(TestCase {
      name,
      input: input_text,
      output: output_text,
    });
  }

  Ok(test_cases)
}

fn parse_selector(selector: &str) -> Result<Selector> {
  Selector::parse(selector).map_err(|e| anyhow!("selector parse error ({selector}): {e}"))
}

#[cfg(test)]
mod tests {
  use super::{parse_problems_from_tasks_html, parse_test_cases_from_problem_html};

  #[test]
  fn parse_problems_extracts_id_title_and_url() {
    let html = r#"
      <a class="contest-title" href="/contests/abc999">AtCoder Beginner Contest 999</a>
      <div id="main-container">
        <table>
          <tbody>
            <tr>
              <td><a href="/contests/abc999/tasks/abc999_a">A</a></td>
              <td>AtCoder Quiz</td>
              <td>2 sec</td>
              <td>1024 MiB</td>
            </tr>
            <tr>
              <td><a href="/contests/abc999/tasks/abc999_b">B</a></td>
              <td>Many Oranges</td>
              <td>2 sec</td>
              <td>1024 MiB</td>
            </tr>
          </tbody>
        </table>
      </div>
    "#;

    let (contest_title, problems) = parse_problems_from_tasks_html(html).expect("failed to parse tasks html");
    assert_eq!(contest_title, "AtCoder Beginner Contest 999");
    assert_eq!(problems.len(), 2);
    assert_eq!(problems[0].id, "a");
    assert_eq!(problems[0].title, "AtCoder Quiz");
    assert_eq!(problems[0].url, "https://atcoder.jp/contests/abc999/tasks/abc999_a");
  }

  #[test]
  fn parse_test_cases_pairs_input_and_output() {
    let html = r"
      <section>
        <h3>入力例 1</h3>
        <pre>1 2\n</pre>
      </section>
      <section>
        <h3>出力例 1</h3>
        <pre>3\n</pre>
      </section>
      <section>
        <h3>入力例 2</h3>
        <pre>10 20\n</pre>
      </section>
      <section>
        <h3>出力例 2</h3>
        <pre>30\n</pre>
      </section>
    ";

    let test_cases = parse_test_cases_from_problem_html(html).expect("failed to parse test cases");
    assert_eq!(test_cases.len(), 2);
    assert_eq!(test_cases[0].name, "入力例 1");
    assert_eq!(test_cases[0].input, "1 2\\n");
    assert_eq!(test_cases[0].output, "3\\n");
  }
}
