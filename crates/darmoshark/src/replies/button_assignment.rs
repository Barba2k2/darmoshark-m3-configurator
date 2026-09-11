/// One button's current assignment, as read back from the mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonAssignment {
  pub button: u8,
  pub kind: String,
  pub kind_value: u8,
  pub data: Vec<u8>,
}
