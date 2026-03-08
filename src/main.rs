mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "AtCoderのコンテスト問題をセットアップ・テストするツール")]
struct Args {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// `AtCoder`用ワークスペースを初期化する (`Cargo.toml`, `rust-toolchain.toml`, `template.rs` を生成)
  Init {
    /// フォルダ名. 省略時はカレントディレクトリで初期化
    dir_name: Option<String>,
  },
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
  /// コンテストの問題ページをブラウザで開く
  Open {
    /// コンテスト名 (例: `abs`, `abc123`)
    contest_name: String,
    /// 問題名 (例: `a`, `b`). 省略時は問題一覧ページを開く
    problem_name: Option<String>,
  },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  match args.command {
    Commands::Init { dir_name } => {
      commands::init::cmd_init(dir_name.as_deref())?;
    },
    Commands::New { contest_name } => {
      commands::new::cmd_new(&contest_name).await?;
    },
    Commands::Test {
      contest_name,
      problem_name,
    } => {
      commands::test::cmd_test(&contest_name, &problem_name)?;
    },
    Commands::Open {
      contest_name,
      problem_name,
    } => {
      commands::open::cmd_open(&contest_name, problem_name.as_deref())?;
    },
  }

  Ok(())
}
