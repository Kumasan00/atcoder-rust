# atcoder-rust

AtCoderのコンテスト問題をセットアップ・テストするためのCLIツールです。

コンテスト名を指定するだけで、問題ごとのソースファイルとテストケースを自動で生成し、ローカルでサンプルケースの検証ができます。

## 機能

- **`init`** — ワークスペースの初期化（`Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`, `template.rs` を生成）
- **`new`** — コンテストのセットアップ（Cargoワークスペースへの追加、テストケース取得、テンプレートからソースファイル生成）
- **`test`** — 保存済みテストケースで解答コードをビルド＆検証
- **`open`** — コンテストの問題ページをブラウザで開く
- **`submit`** — ソースコードをクリップボードにコピーし、提出ページをブラウザで開く

## 必要環境

- Rust (Edition 2024)
- Cargo

## インストール

```bash
cargo install --path .
```

## 使い方

### ワークスペースの初期化

```bash
atcoder-rust init [フォルダ名]
```

例:

```bash
atcoder-rust init my-atcoder
cd my-atcoder
```

フォルダ名を省略するとカレントディレクトリで初期化します。以下のファイルが生成されます:

- `Cargo.toml` — ワークスペース設定
- `rust-toolchain.toml` — Rustツールチェイン指定
- `rustfmt.toml` — フォーマッタ設定
- `template.rs` — 解答テンプレート

### コンテストのセットアップ

```bash
atcoder-rust new <コンテスト名>
```

例:

```bash
atcoder-rust new abc300
```

これにより以下が行われます:

1. ワークスペースの `Cargo.toml` にコンテストをメンバーとして追加
2. コンテストディレクトリと `Cargo.toml` を作成
3. AtCoderから問題一覧を取得し、各問題のテストケース（入出力例）をJSON形式で保存
4. `template.rs` をもとに各問題のソースファイルを `src/bin/` に生成

### テストの実行

```bash
atcoder-rust test <コンテスト名> <問題名>
```

例:

```bash
atcoder-rust test abc300 a
```

サンプルケースの入出力と実行結果を比較し、AC / WA を表示します。

### 問題ページを開く

```bash
atcoder-rust open <コンテスト名> [問題名]
```

例:

```bash
atcoder-rust open abc300 a   # 問題ページを開く
atcoder-rust open abc300     # 問題一覧ページを開く
```

### 解答の提出

```bash
atcoder-rust submit <コンテスト名> <問題名>
```

例:

```bash
atcoder-rust submit abc300 a
```

ソースコード（`<コンテスト名>/src/bin/<問題名>.rs`）をクリップボードにコピーし、問題の提出ページをブラウザで開きます。ページ上でコードを貼り付けて提出してください。

## プロジェクト構成

```text
├── Cargo.toml          # ワークスペースルート
├── rust-toolchain.toml # ツールチェイン指定
├── rustfmt.toml        # フォーマッタ設定
├── template.rs         # 問題ソースのテンプレート
└── <contest_name>/     # new で生成されるコンテストディレクトリ
    ├── Cargo.toml      # 問題メタデータ含む
    ├── src/bin/        # 問題ごとのソースファイル
    └── test_cases/     # 問題ごとのテストケースJSON
```

## ライセンス

MIT
