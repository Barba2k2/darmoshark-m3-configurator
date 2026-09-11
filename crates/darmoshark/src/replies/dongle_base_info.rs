use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::sensor_toggles::SensorToggles;
use crate::replies::base_info::BaseInfo;

/// Stored configuration as the 2.4GHz receiver reports it (opcode 0x07).
///
/// This is the read the charging cable cannot do: over the cable every opcode
/// comes back as the identity block, so the mouse never tells what it holds.
/// Battery does not travel in this reply; it comes from the identify read.
///
/// Offsets follow getBaseInfo() of the "M" contract in the official bundle,
/// counted from the opcode echo (the report id is stripped first):
///
/// ```text
/// [0]      opcode echo (0x07)
/// [1]      active onboard profile
/// [2..4]   usb / 2.4GHz / bluetooth slots -- low nibble the active dpi
///          index, high nibble the report-rate index
/// [5..14]  five little-endian uint16 dpi values
/// [15]     sensor and system bits
/// [16]     number of enabled dpi levels
/// [17]     click debounce, in milliseconds
/// [18]     sleep timer, in minutes
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DongleBaseInfo {
  pub profile: u8,
  pub dpi_levels: Vec<u16>,
  pub active_level: u8,
  pub report_rate: u8,
  pub debounce_ms: u8,
  pub sleep_minutes: u8,
  pub lift_off: u8,
  pub wave: u8,
  pub line: u8,
  pub motion: u8,
  pub scroll: u8,
  pub e_sports: u8,
  pub raw: Vec<u8>,
}

impl DongleBaseInfo {
  pub const opcode: u8 = 0x07;

  /// The sensor switches as stored, ready to be written back unchanged.
  pub fn sensor_toggles(&self) -> SensorToggles {
    SensorToggles {
      wave: self.wave == 1,
      line: self.line == 1,
      motion: self.motion == 1,
      scroll: self.scroll == 1,
      e_sports: self.e_sports == 1,
    }
  }

  pub fn parse(reply: &[u8]) -> DarmosharkResult<Self> {
    if reply.len() < 20 {
      return Err(DarmosharkError::Invalid(format!(
        "snapshot reply too short: {} bytes",
        reply.len()
      )));
    }

    let body = &reply[1..];
    if body[0] != Self::opcode {
      return Err(DarmosharkError::Invalid(format!(
        "unexpected opcode echo 0x{:02X}, expected 0x{:02X}",
        body[0],
        Self::opcode
      )));
    }

    let gears = usize::from(body[16]);
    if !(1..=8).contains(&gears) {
      return Err(DarmosharkError::Invalid(format!(
        "implausible level count in reply: {gears}"
      )));
    }

    let offsets = &BaseInfo::dpiOffsets[..gears];
    if offsets[gears - 1].0 >= body.len() {
      return Err(DarmosharkError::Invalid(format!(
        "reply holds {} bytes, too few for {gears} dpi levels",
        body.len()
      )));
    }

    // Byte 3 is the 2.4GHz slot, the one in use while this read is possible.
    let radio_byte = body[3];
    let sensor_byte = body[15];

    Ok(Self {
      profile: body[1],
      dpi_levels: offsets
        .iter()
        .map(|&(high, low)| u16::from_le_bytes([body[low], body[high]]))
        .collect(),
      active_level: radio_byte & 0x0F,
      report_rate: (radio_byte >> 4) & 0x0F,
      debounce_ms: body[17],
      sleep_minutes: body[18],
      lift_off: sensor_byte & 0x03,
      wave: (sensor_byte >> 2) & 1,
      line: (sensor_byte >> 3) & 1,
      motion: (sensor_byte >> 4) & 1,
      scroll: (sensor_byte >> 6) & 1,
      e_sports: (sensor_byte >> 7) & 1,
      raw: reply.to_vec(),
    })
  }
}
