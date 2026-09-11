use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::packets::sleep_mode::SleepMode;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;

/// Debounce, lift-off distance, scroll and sleep timer packets.
///
/// All of them ride the 20-byte short report (0xB5) and are acknowledged with
/// an 0xE4 frame echoing the opcode.
pub struct TuningPacket;

impl TuningPacket {
  pub const debounceMinimum: u8 = 0;
  pub const debounceMaximum: u8 = 20;
  pub const liftOffValues: [u8; 2] = [1, 2]; // M3 profile declares only low/high
  pub const sleepMinimumMinutes: u8 = 0;
  pub const sleepMaximumMinutes: u8 = 255;

  pub fn build_debounce(milliseconds: u32) -> DarmosharkResult<Packet> {
    let range = u32::from(Self::debounceMinimum)..=u32::from(Self::debounceMaximum);
    if !range.contains(&milliseconds) {
      return Err(DarmosharkError::Invalid(format!(
        "debounce must be {}-{} ms, got {milliseconds}",
        Self::debounceMinimum,
        Self::debounceMaximum
      )));
    }
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::setButtonDebounce;
    payload[1] = milliseconds as u8;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }

  /// Sensor block: lift-off distance plus the assorted on/off toggles.
  ///
  /// The vendor defaults are `wave 1, line 2, motion 1, scroll 1, eSports 1`.
  pub fn build_sensor(
    lift_off: u8,
    wave: u8,
    line: u8,
    motion: u8,
    scroll: u8,
    e_sports: u8,
  ) -> DarmosharkResult<Packet> {
    if !Self::liftOffValues.contains(&lift_off) {
      return Err(DarmosharkError::Invalid(format!(
        "liftOff must be one of {:?}, got {lift_off}",
        Self::liftOffValues
      )));
    }
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::setSensorLiftCutoff;
    payload[1] = lift_off;
    payload[2] = wave;
    payload[3] = line;
    payload[4] = motion;
    payload[6] = scroll;
    payload[7] = e_sports;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }

  pub fn build_scroll(speed: u8, inertia: u8, spl: u8) -> Packet {
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::setScroll;
    payload[1] = speed;
    payload[2] = inertia;
    payload[3] = spl;
    Packet::new(DarmosharkProtocol::shortReportId, payload)
  }

  pub fn build_sleep(minutes: u32, mode: SleepMode) -> DarmosharkResult<Packet> {
    let range = u32::from(Self::sleepMinimumMinutes)..=u32::from(Self::sleepMaximumMinutes);
    if !range.contains(&minutes) {
      return Err(DarmosharkError::Invalid(format!(
        "sleep must be {}-{} minutes, got {minutes}",
        Self::sleepMinimumMinutes,
        Self::sleepMaximumMinutes
      )));
    }
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::deviceTime;
    payload[1] = mode.marker();
    payload[2] = minutes as u8;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }
}
