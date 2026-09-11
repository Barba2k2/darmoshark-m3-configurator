use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;

/// Programs the polling rate of the active connection slot (opcode 65).
///
/// Layout on the short report, the same in the cable and receiver contracts:
///
/// ```text
/// [0]     opcode 65
/// [1..2]  rate index, repeated
/// ```
///
/// The index is the position of the frequency in the device profile
/// (125/500/1000 Hz) and the same value the snapshot reports in the high nibble
/// of the slot byte. Measured on an M3 through the receiver: index 0 polls
/// every 8 ms, 1 every 2 ms, 2 every 1 ms. The receiver also accepts index 3,
/// which the M3 profile does not declare; it is not offered.
pub struct ReportRatePacket;

impl ReportRatePacket {
  pub const supportedRates: [u32; 3] = [125, 500, 1000];

  pub fn build(hertz: u32) -> DarmosharkResult<Packet> {
    let code = Self::rate_to_code(hertz)?;
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::setReportRate;
    payload[1] = code;
    payload[2] = code;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }

  /// Maps a frequency to its profile index, as the configurator does.
  pub fn rate_to_code(hertz: u32) -> DarmosharkResult<u8> {
    Self::supportedRates
      .iter()
      .position(|&rate| rate == hertz)
      .map(|index| index as u8)
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
