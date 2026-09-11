use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;

/// Builds the byte payload that programs the DPI levels.
///
/// Layout for the short form (<= 5 levels), opcode 0x40 on report 0xB5:
///
/// ```text
/// [0]      opcode 0x40
/// [1..3]   currently selected level index (repeated 3x)
/// [4..13]  five little-endian uint16 dpi values
/// [14]     number of enabled levels
/// ```
///
/// Every value is validated before it reaches the device: a malformed packet
/// can put the mouse into an inconsistent profile.
pub struct DpiPacket;

impl DpiPacket {
  pub fn build(
    dpi_values: &[u32],
    current_level: usize,
    enabled_levels: Option<usize>,
  ) -> DarmosharkResult<Packet> {
    Self::validate(dpi_values, current_level)?;
    let values: Vec<u16> = dpi_values.iter().map(|&value| value as u16).collect();

    let gears = enabled_levels.unwrap_or(values.len());
    if !(1..=values.len()).contains(&gears) {
      return Err(DarmosharkError::Invalid(format!(
        "enabledLevels must be between 1 and {}, got {gears}",
        values.len()
      )));
    }

    if values.len() > DarmosharkProtocol::maxShortLevels {
      return Ok(Self::build_long(&values, current_level));
    }
    Ok(Self::build_short(&values, current_level, gears))
  }

  fn build_short(values: &[u16], current_level: usize, gears: usize) -> Packet {
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DarmosharkProtocol::cmdSetDpiShort;
    payload[1..4].fill(current_level as u8);
    for (index, value) in values.iter().enumerate() {
      payload[4 + index * 2..6 + index * 2].copy_from_slice(&value.to_le_bytes());
    }
    payload[14] = gears as u8;
    Packet::new(DarmosharkProtocol::shortReportId, payload)
  }

  fn build_long(values: &[u16], current_level: usize) -> Packet {
    let mut payload = vec![0u8; DarmosharkProtocol::longPayloadSize];
    payload[0] = DarmosharkProtocol::cmdSetDpiLong;
    payload[1..4].fill(current_level as u8);
    payload[4] = values.len() as u8;
    for (index, value) in values.iter().enumerate() {
      payload[5 + index * 2..7 + index * 2].copy_from_slice(&value.to_le_bytes());
    }
    Packet::new(DarmosharkProtocol::longReportId, payload)
  }

  fn validate(values: &[u32], current_level: usize) -> DarmosharkResult<()> {
    if values.is_empty() {
      return Err(DarmosharkError::Invalid(
        "at least one DPI value is required".into(),
      ));
    }
    if values.len() > DarmosharkProtocol::maxLevels {
      return Err(DarmosharkError::Invalid(format!(
        "at most {} DPI levels are supported, got {}",
        DarmosharkProtocol::maxLevels,
        values.len()
      )));
    }
    let range =
      u32::from(DarmosharkProtocol::dpiMinimum)..=u32::from(DarmosharkProtocol::dpiMaximum);
    if let Some(value) = values.iter().find(|value| !range.contains(value)) {
      return Err(DarmosharkError::Invalid(format!(
        "DPI {value} is out of range ({}-{})",
        DarmosharkProtocol::dpiMinimum,
        DarmosharkProtocol::dpiMaximum
      )));
    }
    if current_level >= values.len() {
      return Err(DarmosharkError::Invalid(format!(
        "currentLevel must index one of the {} values, got {current_level}",
        values.len()
      )));
    }
    Ok(())
  }
}
