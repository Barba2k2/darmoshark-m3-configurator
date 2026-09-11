use darmoshark::packets::tuning_packet::TuningPacket;
use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::protocol::darmoshark_protocol::DarmosharkProtocol;
use serde::Serialize;

/// What the model supports, offline: the vendor profile plus the limits the
/// packet builders enforce, so the window never offers a value they reject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDto {
  pub name: String,
  pub dpi_levels: Vec<u32>,
  pub dpi_colors: Vec<String>,
  pub dpi_minimum: u16,
  pub dpi_maximum: u16,
  pub report_rates: Vec<u32>,
  pub lift_off_steps: Vec<u8>,
  pub debounce_minimum: u8,
  pub debounce_maximum: u8,
  pub sleep_maximum_minutes: u8,
}

impl ProfileDto {
  pub fn from_profile(profile: &DeviceProfile) -> Self {
    Self {
      name: profile.name().to_string(),
      dpi_levels: profile.dpi_levels(),
      dpi_colors: profile.dpi_colors(),
      dpi_minimum: DarmosharkProtocol::dpiMinimum,
      dpi_maximum: DarmosharkProtocol::dpiMaximum,
      report_rates: profile
        .report_rates()
        .into_iter()
        .filter_map(|(hertz, _)| hertz)
        .collect(),
      lift_off_steps: profile
        .lift_off_steps()
        .into_iter()
        .filter_map(|(index, _)| index)
        .collect(),
      debounce_minimum: TuningPacket::debounceMinimum,
      debounce_maximum: TuningPacket::debounceMaximum,
      sleep_maximum_minutes: TuningPacket::sleepMaximumMinutes,
    }
  }
}
