use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

/// Parsed snapshot of the mouse configuration (base-info reply, opcode 0x06).
///
/// Field offsets follow handleInfo() in the official configurator bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseInfo {
  pub profile: u8,
  pub dpi_levels: Vec<u16>,
  pub active_level: u8,
  pub report_rate: u8,
  pub battery_percent: u8,
  pub battery_charging: bool,
  pub sleep_minutes: u8,
  pub raw: Vec<u8>,
}

impl BaseInfo {
  /// (high, low) byte offsets of each dpi level, in level order.
  pub const dpiOffsets: [(usize, usize); 8] = [
    (6, 5),
    (8, 7),
    (10, 9),
    (12, 11),
    (14, 13),
    (21, 20),
    (23, 22),
    (25, 24),
  ];

  pub fn parse(data: &[u8]) -> DarmosharkResult<Self> {
    if data.len() < 30 {
      return Err(DarmosharkError::Invalid(format!(
        "base-info reply too short: {} bytes",
        data.len()
      )));
    }

    let gears = usize::from(data[16]);
    if !(1..=8).contains(&gears) {
      return Err(DarmosharkError::Invalid(format!(
        "implausible level count in reply: {gears}"
      )));
    }

    // Byte 3 is the 2.4GHz slot: high nibble report rate, low nibble dpi index.
    let rf_byte = data[3];
    let power_byte = data[19];
    Ok(Self {
      profile: data[1],
      dpi_levels: Self::dpiOffsets[..gears]
        .iter()
        .map(|&(high, low)| u16::from_le_bytes([data[low], data[high]]))
        .collect(),
      active_level: rf_byte & 0x0F,
      report_rate: (rf_byte >> 4) & 0x0F,
      battery_percent: power_byte & 0x7F,
      battery_charging: power_byte & 0x80 != 0,
      sleep_minutes: data[18],
      raw: data.to_vec(),
    })
  }
}
