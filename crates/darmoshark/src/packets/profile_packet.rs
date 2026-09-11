use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;

/// Profile selection and the factory-reset path.
///
/// Recovery is the same opcode for both: a partial reset targets one profile,
/// while value 63 wipes every stored profile.
pub struct ProfilePacket;

impl ProfilePacket {
  pub const profileCount: u8 = 4;

  pub fn build_switch(profile_index: u8) -> DarmosharkResult<Packet> {
    if profile_index >= Self::profileCount {
      return Err(DarmosharkError::Invalid(format!(
        "profile must be 0-{}, got {profile_index}",
        Self::profileCount - 1
      )));
    }
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::profileSwitch;
    payload[1] = profile_index;
    Ok(Packet::new(DarmosharkProtocol::shortReportId, payload))
  }

  pub fn build_recovery(value: u8, profile_index: u8, tag_value: u8) -> Packet {
    let mut payload = vec![0u8; DarmosharkProtocol::shortPayloadSize];
    payload[0] = DmsCommands::driverConfigRecovery;
    payload[1] = value;
    payload[2] = profile_index;
    payload[3] = tag_value;
    Packet::new(DarmosharkProtocol::shortReportId, payload)
  }

  pub fn build_factory_reset() -> Packet {
    Self::build_recovery(DmsCommands::factoryResetValue, 0, 0)
  }
}
