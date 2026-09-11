/// Direction of the sleep timer command (opcode 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepMode {
  Set,
  Get,
}

impl SleepMode {
  pub fn marker(self) -> u8 {
    match self {
      Self::Set => 1,
      Self::Get => 2,
    }
  }
}
