use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
  #[error("Cargo.toml に [workspace] members が見つかりません")]
  WorkspaceMembersMissing,

  #[error("問題一覧を取得できませんでした")]
  NoProblemsFetched,

  #[error("contest Cargo.toml の {0} が不正です")]
  InvalidContestCargoToml(&'static str),

  #[error("{0} に問題メタデータが見つかりません")]
  MissingProblemsMetadata(String),

  #[error("問題 '{0}' が見つかりません")]
  ProblemNotFound(String),

  #[error("問題 '{0}' の URL が見つかりません")]
  ProblemUrlMissing(String),

  #[error("ビルドに失敗しました: {contest}/{problem}")]
  BuildFailed { contest: String, problem: String },

  #[error("テスト失敗: {passed}/{total} 通過")]
  TestFailed { passed: usize, total: usize },
}
