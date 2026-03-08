pub mod init;
pub mod new;
pub mod open;
pub mod test;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
  pub name: String,
  pub input: String,
  pub output: String,
}
