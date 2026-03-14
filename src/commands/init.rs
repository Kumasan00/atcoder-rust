use std::{
  env, fs,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

pub fn cmd_init(dir_name: Option<&str>) -> Result<()> {
  let base_dir = if let Some(name) = dir_name {
    let dir = PathBuf::from(name);
    fs::create_dir_all(&dir).with_context(|| format!("ディレクトリの作成に失敗しました: {}", dir.display()))?;
    env::set_current_dir(&dir)
      .with_context(|| format!("カレントディレクトリの移動に失敗しました: {}", dir.display()))?;
    println!("【作成】{name}/");
    dir
  } else {
    PathBuf::from(".")
  };

  let cargo_toml_path = "Cargo.toml";
  let rust_toolchain_path = "rust-toolchain.toml";
  let template_path = "template.rs";

  if Path::new(cargo_toml_path).exists() {
    let display = base_dir.join(cargo_toml_path);
    bail!("{} はすでに存在しています。", display.display());
  }

  let cargo_toml_content = include_str!("../../templates/Cargo.toml");
  fs::write(cargo_toml_path, cargo_toml_content).context("Cargo.toml の書き込みに失敗しました")?;
  println!("【作成】{cargo_toml_path}");

  let rustfmt_content = include_str!("../../templates/rustfmt.toml");
  fs::write("rustfmt.toml", rustfmt_content).context("rustfmt.toml の書き込みに失敗しました")?;
  println!("【作成】rustfmt.toml");

  let rust_toolchain_content = include_str!("../../templates/rust-toolchain.toml");
  fs::write(rust_toolchain_path, rust_toolchain_content).context("rust-toolchain.toml の書き込みに失敗しました")?;
  println!("【作成】{rust_toolchain_path}");

  let template_content = include_str!("../../templates/template.rs");
  fs::write(template_path, template_content).context("template.rs の書き込みに失敗しました")?;
  println!("【作成】{template_path}");

  println!("\nワークスペースの初期化が完了しました。");
  println!("次は `atcoder-rust new <コンテスト名>` でコンテストをセットアップしてください。");

  Ok(())
}
