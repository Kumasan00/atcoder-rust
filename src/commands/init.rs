use std::{
  env, fs,
  path::{Path, PathBuf},
};

pub fn cmd_init(dir_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
  let base_dir = if let Some(name) = dir_name {
    let dir = PathBuf::from(name);
    fs::create_dir_all(&dir)?;
    env::set_current_dir(&dir)?;
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
    return Err(format!("{} はすでに存在しています。", display.display()).into());
  }

  let cargo_toml_content = include_str!("../../templates/Cargo.toml");
  fs::write(cargo_toml_path, cargo_toml_content)?;
  println!("【作成】{cargo_toml_path}");

  let rust_toolchain_content = include_str!("../../templates/rust-toolchain.toml");
  fs::write(rust_toolchain_path, rust_toolchain_content)?;
  println!("【作成】{rust_toolchain_path}");

  let template_content = include_str!("../../templates/template.rs");
  fs::write(template_path, template_content)?;
  println!("【作成】{template_path}");

  println!("\nワークスペースの初期化が完了しました。");
  println!("次は `atcoder-rust new <コンテスト名>` でコンテストをセットアップしてください。");

  Ok(())
}
