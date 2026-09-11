use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::packets::button_packet::ButtonPacket;
use crate::packets::dpi_packet::DpiPacket;
use crate::packets::packet::Packet;
use crate::packets::profile_packet::ProfilePacket;
use crate::packets::report_rate_packet::ReportRatePacket;
use crate::packets::sensor_toggles::SensorToggles;
use crate::packets::sleep_mode::SleepMode;
use crate::packets::tuning_packet::TuningPacket;
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;
use crate::replies::base_info::BaseInfo;
use crate::replies::base_snapshot::BaseSnapshot;
use crate::replies::bond_info::BondInfo;
use crate::replies::button_assignment::ButtonAssignment;
use crate::replies::cable_info::CableInfo;
use crate::replies::dfu_info::DfuInfo;
use crate::replies::dongle_base_info::DongleBaseInfo;
use crate::transport::darmoshark_device::DarmosharkDevice;

/// Every setting the official configurator can change, over the dms channel.
///
/// Writes work over either transport. Reading the stored configuration back
/// needs the 2.4GHz receiver -- over the cable every opcode answers with the
/// identity block instead.
pub struct MouseConfigurator {
  device: DarmosharkDevice,
}

impl MouseConfigurator {
  pub fn new(device: DarmosharkDevice) -> Self {
    Self { device }
  }

  pub fn device(&self) -> &DarmosharkDevice {
    &self.device
  }

  // -- reads available over the cable -----------------------------------

  pub fn read_cable_info(&self) -> DarmosharkResult<CableInfo> {
    let mut payload = vec![0u8; DarmosharkProtocol::identifyPayloadSize];
    payload[0] = DarmosharkProtocol::cmdIdentify;
    let reply = self
      .device
      .request_feature(DarmosharkProtocol::identifyFeatureId, &payload, 3)?
      .ok_or_else(|| DarmosharkError::Device("mouse did not answer the identify request".into()))?;
    CableInfo::parse(&reply)
  }

  pub fn read_dfu_info(&self) -> DarmosharkResult<DfuInfo> {
    let packets = self
      .device
      .request_dfu(1, DarmosharkProtocol::cmdDfuModuleInfo)?;
    if packets.is_empty() {
      return Err(DarmosharkError::Device(
        "bootloader did not answer the module-info request".into(),
      ));
    }
    DfuInfo::parse(&packets)
  }

  // -- reads that need the 2.4GHz receiver -------------------------------

  /// Reads the stored configuration back, receiver only.
  pub fn read_dongle_base_info(&self) -> DarmosharkResult<DongleBaseInfo> {
    let reply = self
      .request_dongle_opcode(DarmosharkProtocol::cmdDongleBaseInfo)?
      .ok_or_else(|| {
        DarmosharkError::Device("receiver did not answer the configuration read".into())
      })?;
    DongleBaseInfo::parse(&reply)
  }

  /// Identity of the mouse the receiver is linked to, and the link state.
  pub fn read_bond_info(&self) -> DarmosharkResult<BondInfo> {
    let reply = self
      .request_dongle_opcode(DmsCommands::getBondInfo)?
      .ok_or_else(|| DarmosharkError::Device("receiver did not answer the bond request".into()))?;
    BondInfo::parse(&reply)
  }

  // -- reads that need the config interface ------------------------------

  pub fn read_base_info(&self) -> DarmosharkResult<BaseSnapshot> {
    if self.device.uses_dongle_transport() {
      return self.read_dongle_base_info().map(BaseSnapshot::Dongle);
    }

    let mut payload = vec![0u8; DarmosharkProtocol::longPayloadSize];
    payload[0] = DmsCommands::getMouseExtInfo;

    let reply = self
      .device
      .request_command(DarmosharkProtocol::longReportId, &payload)?
      .ok_or_else(|| {
        DarmosharkError::Device(
          "mouse did not answer the base-info request. The open interface is probably \
           the cable one — connect through the 2.4GHz dongle."
            .into(),
        )
      })?;
    if !DarmosharkProtocol::baseInfoReplyOpcodes.contains(&reply[0]) {
      return Err(DarmosharkError::Device(format!(
        "unexpected reply opcode 0x{:02X}, expected one of {:02X?}",
        reply[0],
        DarmosharkProtocol::baseInfoReplyOpcodes
      )));
    }
    BaseInfo::parse(&reply).map(BaseSnapshot::Config)
  }

  pub fn read_button(&self, button_index: u8) -> DarmosharkResult<ButtonAssignment> {
    let packet = ButtonPacket::build_read(button_index)?;
    let reply = if self.device.uses_dongle_transport() {
      // The opcode alone does not identify the answer here: every button
      // shares it, so the index has to be echoed back too.
      self
        .device
        .request_dongle(&packet.payload, 4, 2)?
        .map(|reply| reply[1..].to_vec())
    } else {
      self
        .device
        .request_command(packet.report_id, &packet.payload)?
    };
    let reply = reply
      .ok_or_else(|| DarmosharkError::Device(format!("no answer reading button {button_index}")))?;
    ButtonPacket::parse_read(&reply, button_index)
  }

  pub fn read_all_buttons(&self) -> DarmosharkResult<Vec<ButtonAssignment>> {
    (0..ButtonPacket::buttonCount)
      .map(|index| self.read_button(index))
      .collect()
  }

  // -- writes -------------------------------------------------------------

  pub fn write_dpi_levels(
    &self,
    dpi_values: &[u32],
    active_level: usize,
    enabled_levels: Option<usize>,
  ) -> DarmosharkResult<()> {
    let packet = DpiPacket::build(dpi_values, active_level, enabled_levels)?;
    self.device.send_command(packet.report_id, &packet.payload)
  }

  pub fn select_dpi_level(&self, level_index: usize) -> DarmosharkResult<BaseSnapshot> {
    let info = self.read_base_info()?;
    let levels: Vec<u32> = info
      .dpi_levels()
      .iter()
      .map(|&level| u32::from(level))
      .collect();
    if level_index >= levels.len() {
      return Err(DarmosharkError::Invalid(format!(
        "level {level_index} out of range, mouse has {}",
        levels.len()
      )));
    }
    self.write_dpi_levels(&levels, level_index, Some(levels.len()))?;
    Ok(info)
  }

  pub fn write_report_rate(&self, hertz: u32) -> DarmosharkResult<()> {
    let packet = ReportRatePacket::build(hertz)?;
    self.send_acknowledged(&packet, DmsCommands::setReportRate)
  }

  pub fn write_debounce(&self, milliseconds: u32) -> DarmosharkResult<()> {
    let packet = TuningPacket::build_debounce(milliseconds)?;
    self.send_acknowledged(&packet, DmsCommands::setButtonDebounce)
  }

  pub fn write_sensor_settings(
    &self,
    lift_off: u8,
    toggles: &SensorToggles,
  ) -> DarmosharkResult<()> {
    let [wave, line, motion, scroll, e_sports] = toggles.packet_values();
    let packet = TuningPacket::build_sensor(lift_off, wave, line, motion, scroll, e_sports)?;
    self.send_acknowledged(&packet, DmsCommands::setSensorLiftCutoff)
  }

  /// Changes the lift-off distance alone. Its packet also carries the sensor
  /// switches, so over the receiver the stored ones are read and sent back
  /// unchanged. The cable cannot read them: there they are reset to the vendor
  /// defaults, and the returned toggles say which ones were written.
  pub fn write_lift_off(&self, lift_off: u8) -> DarmosharkResult<SensorToggles> {
    let toggles = if self.device.uses_dongle_transport() {
      self.read_dongle_base_info()?.sensor_toggles()
    } else {
      SensorToggles::vendorDefaults
    };
    self.write_sensor_settings(lift_off, &toggles)?;
    Ok(toggles)
  }

  pub fn write_scroll_settings(&self, speed: u8, inertia: u8, spl: u8) -> DarmosharkResult<()> {
    let packet = TuningPacket::build_scroll(speed, inertia, spl);
    self.send_acknowledged(&packet, DmsCommands::setScroll)
  }

  /// Sets the idle sleep timer, cable only. The receiver contract in the
  /// vendor bundle never sends opcode 10 -- its sleep field is read-only -- and
  /// on hardware a write through the receiver leaves the snapshot unchanged
  /// while a `get` never turns ready.
  pub fn write_sleep_timer(&self, minutes: u32) -> DarmosharkResult<()> {
    if self.device.uses_dongle_transport() {
      return Err(DarmosharkError::Device(
        "the receiver does not relay the sleep timer; the vendor software sets it \
         only over the cable. Connect the charging cable and try again."
          .into(),
      ));
    }
    let packet = TuningPacket::build_sleep(minutes, SleepMode::Set)?;
    self.send_acknowledged(&packet, DmsCommands::deviceTime)
  }

  pub fn switch_profile(&self, profile_index: u8) -> DarmosharkResult<()> {
    let packet = ProfilePacket::build_switch(profile_index)?;
    self.send_acknowledged(&packet, DmsCommands::profileSwitch)
  }

  pub fn restore_factory_defaults(&self) -> DarmosharkResult<()> {
    let packet = ProfilePacket::build_factory_reset();
    self.send_acknowledged(&packet, DmsCommands::driverConfigRecovery)
  }

  pub fn write_button(&self, button_index: u8, kind: &str, data: &[u32]) -> DarmosharkResult<()> {
    let packet = ButtonPacket::build_write(button_index, kind, data)?;
    self.send_acknowledged(&packet, DmsCommands::setButtonConfig)
  }

  // -- internals ----------------------------------------------------------

  /// A short receiver command carrying nothing but its opcode.
  fn request_dongle_opcode(&self, opcode: u8) -> DarmosharkResult<Option<Vec<u8>>> {
    let mut payload = vec![0u8; DarmosharkProtocol::donglePayloadSize];
    payload[0] = opcode;
    self.device.request_dongle(&payload, 4, 1)
  }

  /// Sends a command and checks the 0xE4 acknowledgement frame.
  ///
  /// Both hardware transports issue writes fire-and-forget, the same way the
  /// vendor software does: the cable stays silent, and the receiver answers
  /// only that it queued the command. There is nothing to verify in either
  /// case, so a missing reply is not an error.
  fn send_acknowledged(&self, packet: &Packet, opcode: u8) -> DarmosharkResult<()> {
    if self.device.uses_cable_transport() || self.device.uses_dongle_transport() {
      return self.device.send_command(packet.report_id, &packet.payload);
    }

    let reply = self
      .device
      .request_command(packet.report_id, &packet.payload)?
      .ok_or_else(|| {
        DarmosharkError::Device(format!(
          "no acknowledgement for command {opcode}. Is the mouse on the 2.4GHz config \
           interface?"
        ))
      })?;
    if reply[0] == DmsCommands::ackOpcode && reply.len() > 1 && reply[1] != DmsCommands::ackStatusOk
    {
      return Err(DarmosharkError::Device(format!(
        "device rejected command {opcode} with status {}",
        reply[1]
      )));
    }
    Ok(())
  }
}
