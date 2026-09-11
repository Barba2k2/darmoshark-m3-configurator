use darmoshark::packets::report_rate_packet::ReportRatePacket;
use darmoshark::packets::tuning_packet::TuningPacket;
use darmoshark::profile::device_profile::DeviceProfile;

fn profile() -> DeviceProfile {
  DeviceProfile::load().unwrap()
}

#[test]
fn model_is_the_m3() {
  assert_eq!(profile().name(), "Darmoshark M3");
}

#[test]
fn lighting_is_not_configurable() {
  assert!(!profile().has_configurable_lighting());
}

#[test]
fn declares_five_default_levels() {
  assert_eq!(profile().dpi_levels(), [400, 800, 1600, 3200, 4800]);
}

#[test]
fn declares_the_protocol_dpi_range() {
  assert_eq!(profile().dpi_range(), Some((50, 26000)));
}

#[test]
fn lift_off_matches_tuning_packet_range() {
  let declared: Vec<u8> = profile()
    .lift_off_steps()
    .into_iter()
    .filter_map(|(index, _)| index)
    .collect();
  assert_eq!(declared, TuningPacket::liftOffValues);
}

#[test]
fn polling_rates_match_report_rate_packet() {
  let declared: Vec<u32> = profile()
    .report_rates()
    .into_iter()
    .filter_map(|(hertz, _)| hertz)
    .collect();
  assert_eq!(declared, ReportRatePacket::supportedRates);
}

#[test]
fn declares_five_buttons() {
  assert_eq!(profile().button_count(), 5);
}

#[test]
fn rejects_invalid_json() {
  assert!(DeviceProfile::parse("{").is_err());
}
