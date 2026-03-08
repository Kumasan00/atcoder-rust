# atcoder-rust

AtCoderのコンテスト問題をセットアップ・テストするためのCLIツールです。

コンテスト名を指定するだけで、問題ごとのソースファイルとテストケースを自動で生成し、ローカルでサンプルケースの検証ができます。

## 機能

- **`new`** — コンテストのセットアップ（Cargoワークスペースへの追加、テストケース取得、テンプレートからソースファイル生成）
- **`test`** — 保存済みテストケースで解答コードをビルド＆検証

## 必要環境

- Rust (Edition 2024)
- Cargo

## インストール

```bash
cargo build --release
```

## 使い方

### コンテストのセットアップ

```bash
cargo run -- new <コンテスト名>
```

例:

```bash
cargo run -- new abc300
```

これにより以下が行われます:

1. ワークスペースの `Cargo.toml` にコンテストをメンバーとして追加
2. コンテストディレクトリと `Cargo.toml` を作成
3. AtCoderから問題一覧を取得し、各問題のテストケース（入出力例）をJSON形式で保存
4. `template.rs` をもとに各問題のソースファイルを `src/bin/` に生成

### テストの実行

```bash
cargo run -- test <コンテスト名> <問題名>
```

例:

```bash
cargo run -- test abc300 a
```

サンプルケースの入出力と実行結果を比較し、AC / WA を表示します。

## プロジェクト構成

```text
├── Cargo.toml          # ワークスペースルート
├── template.rs         # 問題ソースのテンプレート
├── src/main.rs         # CLIツール本体
└── <contest_name>/     # new で生成されるコンテストディレクトリ
    ├── Cargo.toml
    ├── src/bin/        # 問題ごとのソースファイル
    └── test_cases/     # 問題ごとのテストケースJSON
```

## ライセンス

MIT
