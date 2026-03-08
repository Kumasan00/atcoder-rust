use std::process::Command;

pub fn cmd_open(contest_name: &str, problem_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
  let url = match problem_name {
    Some(name) => format!("https://atcoder.jp/contests/{contest_name}/tasks/{name}"),
    None => format!("https://atcoder.jp/contests/{contest_name}/tasks"),
  };

  println!("【オープン】{url}");
  Command::new("open").arg(&url).status()?;

  Ok(())
}
