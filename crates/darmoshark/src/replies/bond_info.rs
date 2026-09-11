use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

/// Identity of the mouse the receiver is linked to, and the link state
/// (opcode 3, receiver only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BondInfo {
  pub vendor_id: u16,
  pub product_id: u16,
  pub linked: bool,
}

impl BondInfo {
  pub fn parse(reply: &[u8]) -> DarmosharkResult<Self> {
    if reply.len() < 8 {
      return Err(DarmosharkError::Invalid(format!(
        "bond reply too short: {} bytes",
        reply.len()
      )));
    }
    let body = &reply[1..];
    Ok(Self {
      vendor_id: u16::from_le_bytes([body[2], body[3]]),
      product_id: u16::from_le_bytes([body[4], body[5]]),
      linked: body[6] != 0,
    })
  }
}
