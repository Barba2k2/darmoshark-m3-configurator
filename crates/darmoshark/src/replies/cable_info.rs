use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

/// Device identity and battery, read through feature report 0x51.
///
/// Answers opcode 0x06 with identity plus battery, but carries no DPI state.
///
/// Reply layout (after the echoed report id):
///
/// ```text
/// [0]     opcode echo (0x06)
/// [1]     status (1 = ok)
/// [3..4]  vendor id, little-endian
/// [5..6]  product id, little-endian
/// [7..8]  firmware version, little-endian (0x0209 -> 2.0.9)
/// [10]    battery percentage
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CableInfo {
  pub vendor_id: u16,
  pub product_id: u16,
  pub firmware_version: String,
  pub battery_percent: u8,
  pub raw: Vec<u8>,
}

impl CableInfo {
  pub fn parse(reply: &[u8]) -> DarmosharkResult<Self> {
    if reply.len() < 12 {
      return Err(DarmosharkError::Invalid(format!(
        "identify reply too short: {} bytes",
        reply.len()
      )));
    }

    let body = &reply[1..];
    if body[0] != 0x06 {
      return Err(DarmosharkError::Invalid(format!(
        "unexpected opcode echo 0x{:02X}, expected 0x06",
        body[0]
      )));
    }
    if body[1] != 1 {
      return Err(DarmosharkError::Invalid(format!(
        "device reported status {}, expected 1",
        body[1]
      )));
    }

    let firmware = u16::from_le_bytes([body[7], body[8]]);
    Ok(Self {
      vendor_id: u16::from_le_bytes([body[3], body[4]]),
      product_id: u16::from_le_bytes([body[5], body[6]]),
      firmware_version: format!(
        "{}.{}.{}",
        firmware >> 8,
        (firmware >> 4) & 0xF,
        firmware & 0xF
      ),
      battery_percent: body[10],
      raw: reply.to_vec(),
    })
  }
}
