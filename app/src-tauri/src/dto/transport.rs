use serde::Serialize;

/// Which interface the operation went through; it decides what can be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Transport {
  Receiver,
  Cable,
  Other,
}
