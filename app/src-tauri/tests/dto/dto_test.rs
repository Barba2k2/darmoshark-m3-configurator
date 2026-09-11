use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::replies::dongle_base_info::DongleBaseInfo;
use darmoshark_app_lib::dto::profile_dto::ProfileDto;
use darmoshark_app_lib::dto::snapshot_dto::SnapshotDto;

/// Bytes captured from a real receiver (M3, fw 2.0.9r).
const captured: [u8; 21] = [
  0x51, 0x07, 0x00, 0x13, 0x13, 0x03, 0x90, 0x01, 0x20, 0x03, 0x40, 0x06, 0x80, 0x0c, 0xc0, 0x12,
  0x35, 0x05, 0x08, 0x00, 0x00,
];

#[test]
fn snapshot_carries_the_stored_settings() {
  let snapshot = SnapshotDto::from_dongle(&DongleBaseInfo::parse(&captured).unwrap());
  assert_eq!(
    snapshot,
    SnapshotDto {
      dpi_levels: vec![400, 800, 1600, 3200, 4800],
      active_level: 3,
      report_rate_hz: Some(500),
      debounce_ms: 8,
      lift_off: 1,
      sleep_minutes: 0,
    }
  );
}

#[test]
fn snapshot_serialises_in_camel_case_for_the_webview() {
  let snapshot = SnapshotDto::from_dongle(&DongleBaseInfo::parse(&captured).unwrap());
  let json = serde_json::to_value(&snapshot).unwrap();
  assert_eq!(json["dpiLevels"][4], 4800);
  assert_eq!(json["reportRateHz"], 500);
  assert_eq!(json["activeLevel"], 3);
}

#[test]
fn profile_offers_only_what_the_builders_accept() {
  let profile = ProfileDto::from_profile(&DeviceProfile::load().unwrap());
  assert_eq!(profile.dpi_levels, [400, 800, 1600, 3200, 4800]);
  assert_eq!((profile.dpi_minimum, profile.dpi_maximum), (50, 26000));
  assert_eq!(profile.report_rates, [125, 500, 1000]);
  assert_eq!(profile.lift_off_steps, [1, 2]);
  assert_eq!(
    (profile.debounce_minimum, profile.debounce_maximum),
    (0, 20)
  );
}
