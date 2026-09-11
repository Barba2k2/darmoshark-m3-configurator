//! Opcode table of the Darmoshark "dms" configuration protocol.
//!
//! Names mirror the CMD_* symbols found in the official configurator bundle.
//! Commands travel as raw payloads on two output reports: the 20-byte short
//! report (0xB5) and the 63-byte long report (0xB3).

/// Opcodes, report routing and reply markers.
pub struct DmsCommands;

impl DmsCommands {
  // Base / device
  pub const getProtocol: u8 = 2;
  pub const getBondInfo: u8 = 3;
  pub const getDeviceString: u8 = 4;
  pub const getMouseInfo: u8 = 5;
  pub const getMouseExtInfo: u8 = 6; // the "base info" snapshot
  pub const deviceTime: u8 = 10; // sleep timer get/set
  pub const pairButton: u8 = 11;
  pub const profileSwitch: u8 = 14;
  pub const driverConfigRecovery: u8 = 15; // factory reset when value = 63

  // Lighting
  pub const getLightEffectParam: u8 = 35;
  pub const setLightEffectParam: u8 = 36;

  // Mouse tuning
  pub const setDpi: u8 = 64;
  pub const setReportRate: u8 = 65;
  pub const setSensorLiftCutoff: u8 = 66; // LOD and assorted sensor toggles
  pub const setButtonDebounce: u8 = 67;
  pub const setDpiExtended: u8 = 68; // more than 5 levels
  pub const setScroll: u8 = 69;

  // Buttons and macros
  pub const setButtonConfig: u8 = 82;
  pub const setMacroName: u8 = 83;
  pub const setMacroData: u8 = 84;
  pub const getAllButtonConfig: u8 = 97;
  pub const getButtonConfig: u8 = 98;
  pub const getMacroName: u8 = 99;
  pub const getMacroData: u8 = 100;

  // Long data transfer
  pub const longDataTransfer: u8 = 113;
  pub const longDataFlowControl: u8 = 114;

  // Reply markers
  pub const ackOpcode: u8 = 0xE4; // [0]=0xE4 [1]=status [2]=echoed opcode
  pub const ackStatusOk: u8 = 0;
  pub const lightChangedEvent: u8 = 225;
  pub const baseChangedEvent: u8 = 226;
  pub const profileChangedEvent: u8 = 229;

  pub const factoryResetValue: u8 = 63;

  /// Opcodes that must go on the long (0xB3) report.
  pub const longReportCommands: [u8; 11] = [
    Self::getMouseExtInfo,
    Self::getDeviceString,
    Self::setDpiExtended,
    Self::setButtonConfig,
    Self::setMacroName,
    Self::setMacroData,
    Self::getAllButtonConfig,
    Self::getButtonConfig,
    Self::getMacroName,
    Self::getMacroData,
    Self::longDataTransfer,
  ];

  pub fn uses_long_report(opcode: u8) -> bool {
    Self::longReportCommands.contains(&opcode)
  }
}
