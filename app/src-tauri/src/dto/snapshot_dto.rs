use darmoshark::packets::report_rate_packet::ReportRatePacket;
use darmoshark::replies::dongle_base_info::DongleBaseInfo;
use serde::Serialize;

/// The stored configuration, as the receiver reports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDto {
  pub dpi_levels: Vec<u16>,
  pub active_level: u8,
  pub report_rate_hz: Option<u32>,
  pub debounce_ms: u8,
  pub lift_off: u8,
  pub sleep_minutes: u8,
}

impl SnapshotDto {
  pub fn from_dongle(info: &DongleBaseInfo) -> Self {
    Self {
      dpi_levels: info.dpi_levels.clone(),
      active_level: info.active_level,
      report_rate_hz: ReportRatePacket::code_to_rate(info.report_rate),
      debounce_ms: info.debounce_ms,
      lift_off: info.lift_off,
      sleep_minutes: info.sleep_minutes,
    }
  }
}
