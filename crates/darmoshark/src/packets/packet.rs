/// A built command: the report id it travels on and its raw payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
  pub report_id: u8,
  pub payload: Vec<u8>,
}

impl Packet {
  pub fn new(report_id: u8, payload: Vec<u8>) -> Self {
    Self { report_id, payload }
  }
}
