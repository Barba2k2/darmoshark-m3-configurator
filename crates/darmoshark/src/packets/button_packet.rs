use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::packet::Packet;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;
use crate::replies::button_assignment::ButtonAssignment;

/// Reads and writes the per-button assignment.
///
/// Assignment kinds, as enumerated by the configurator:
///
/// ```text
/// 0 remove   1 Mouse    2 Keyboard  3 Media   4 Macro
/// 5 Dpi      6 Light    7 GameReinforce      8 ShortCut
/// 9 disable  10 profileSwitch
/// ```
pub struct ButtonPacket;

impl ButtonPacket {
  pub const kinds: [(&'static str, u8); 11] = [
    ("remove", 0),
    ("mouse", 1),
    ("keyboard", 2),
    ("media", 3),
    ("macro", 4),
    ("dpi", 5),
    ("light", 6),
    ("gameReinforce", 7),
    ("shortCut", 8),
    ("disable", 9),
    ("profileSwitch", 10),
  ];

  pub const buttonCount: u8 = 5;

  pub fn build_read(button_index: u8) -> DarmosharkResult<Packet> {
    Self::validate_index(button_index)?;
    let mut payload = vec![0u8; DarmosharkProtocol::longPayloadSize];
    payload[0] = DmsCommands::getButtonConfig;
    payload[1] = button_index;
    Ok(Packet::new(DarmosharkProtocol::longReportId, payload))
  }

  pub fn build_write(button_index: u8, kind: &str, data: &[u32]) -> DarmosharkResult<Packet> {
    Self::validate_index(button_index)?;
    let kind_value = Self::resolve_kind(kind)?;
    if data.len() > DarmosharkProtocol::longPayloadSize - 4 {
      return Err(DarmosharkError::Invalid(format!(
        "assignment data too long: {} bytes",
        data.len()
      )));
    }
    if let Some(byte) = data.iter().find(|&&byte| byte > 0xFF) {
      return Err(DarmosharkError::Invalid(format!(
        "assignment byte {byte} does not fit in a byte"
      )));
    }

    let mut payload = vec![0u8; DarmosharkProtocol::longPayloadSize];
    payload[0] = DmsCommands::setButtonConfig;
    payload[1] = button_index;
    payload[3] = kind_value;
    for (offset, &byte) in data.iter().enumerate() {
      payload[4 + offset] = byte as u8;
    }
    Ok(Packet::new(DarmosharkProtocol::longReportId, payload))
  }

  pub fn parse_read(reply: &[u8], button_index: u8) -> DarmosharkResult<ButtonAssignment> {
    if reply.len() < 5 {
      return Err(DarmosharkError::Invalid(format!(
        "button reply too short: {} bytes",
        reply.len()
      )));
    }
    if reply[0] != DmsCommands::getButtonConfig {
      return Err(DarmosharkError::Invalid(format!(
        "unexpected opcode 0x{:02X} in button reply",
        reply[0]
      )));
    }
    if reply[1] != button_index {
      return Err(DarmosharkError::Invalid(format!(
        "reply is for button {}, expected {button_index}",
        reply[1]
      )));
    }
    let kind_value = reply[3];
    Ok(ButtonAssignment {
      button: button_index,
      kind: Self::kind_name(kind_value),
      kind_value,
      data: reply[4..].to_vec(),
    })
  }

  pub fn resolve_kind(kind: &str) -> DarmosharkResult<u8> {
    Self::kinds
      .iter()
      .find(|(name, _)| *name == kind)
      .map(|&(_, value)| value)
      .ok_or_else(|| {
        let mut names: Vec<&str> = Self::kinds.iter().map(|&(name, _)| name).collect();
        names.sort_unstable();
        DarmosharkError::Invalid(format!(
          "unknown assignment kind {kind:?}, expected one of {names:?}"
        ))
      })
  }

  pub fn kind_name(value: u8) -> String {
    Self::kinds
      .iter()
      .find(|&&(_, candidate)| candidate == value)
      .map_or_else(
        || format!("unknown({value})"),
        |&(name, _)| name.to_string(),
      )
  }

  fn validate_index(button_index: u8) -> DarmosharkResult<()> {
    if button_index >= Self::buttonCount {
      return Err(DarmosharkError::Invalid(format!(
        "button must be 0-{}, got {button_index}",
        Self::buttonCount - 1
      )));
    }
    Ok(())
  }
}
