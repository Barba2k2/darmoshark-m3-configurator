use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;

/// Programs the polling rate assigned to each DPI level (opcode 65).
///
/// Layout on the short report:
///
/// ```text
/// [0]     opcode 65
/// [1..2]  active level index
/// [3..8]  one rate code per level
/// [9]     number of enabled levels
/// ```
///
/// The device stores a rate *code*, not the frequency itself. Codes come from
/// the position of the value in the device profile (125/500/1000 Hz).
///
/// Unverified on hardware: neither this layout nor the bundle's uint16-Hz form
/// moves the rate nibble of the receiver snapshot. Do not swap it for the other
/// layout without evidence.
pub struct ReportRatePacket;

impl ReportRatePacket {
  pub const supportedRates: [u32; 3] = [125, 500, 1000];

  pub fn build(
    rate_codes: &[u32],
    active_level: usize,
    enabled_levels: Option<usize>,
  ) -> DarmosharkResult<Packet> {
    if rate_codes.is_empty() {
      return Err(DarmosharkError::Invalid(
        "at least one rate code is required".into(),
      ));
    }
    if rate_codes.len() > 6 {
      return Err(DarmosharkError::Invalid(format!(
        "at most 6 levels are supported, got {}",
        rate_codes.len()
      )));
    }
    if active_level >= rate_codes.len() {
      return Err(DarmosharkError::Invalid(format!(
        "activeLevel must index one of the {} levels, got {active_level}",
        rate_codes.len()
      )));
    }
    if let Some(code) = rate_codes.iter().find(|&&code| code > 0xFF) {
      return Err(DarmosharkError::Invalid(format!(
        "rate code {code} does not fit in a byte"
      )));
    }

    let gears = enabled_levels.unwrap_or(rate_codes.len());
    if !(1..=rate_codes.len()).contains(&gears) {
      return Err(DarmosharkError::Invalid(format!(
        "enabledLevels must be between 1 and {}, got {gears}",
        rate_codes.len()
      )));
    }

    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::setReportRate;
    payload[1] = active_level as u8;
    payload[2] = active_level as u8;
    for (index, &code) in rate_codes.iter().enumerate() {
      payload[3 + index] = code as u8;
    }
    payload[9] = gears as u8;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }

  /// Maps a frequency to its profile index, as the configurator does.
  pub fn rate_to_code(hertz: u32) -> DarmosharkResult<u32> {
    Self::supportedRates
      .iter()
      .position(|&rate| rate == hertz)
      .map(|index| index as u32)
      .ok_or_else(|| {
        DarmosharkError::Invalid(format!(
          "unsupported polling rate {hertz} Hz, expected one of {:?}",
          Self::supportedRates
        ))
      })
  }

  pub fn code_to_rate(code: u8) -> Option<u32> {
    Self::supportedRates.get(usize::from(code)).copied()
  }
}
