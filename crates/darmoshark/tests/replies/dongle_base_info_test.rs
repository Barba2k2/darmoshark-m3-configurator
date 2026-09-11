//! Bytes captured from a real receiver (M3, fw 2.0.9r) over 2.4GHz.

use darmoshark::replies::dongle_base_info::DongleBaseInfo;

const snapshot: [u8; 21] = [
  0x51, 0x07, 0x00, 0x13, 0x13, 0x03, 0x90, 0x01, 0x20, 0x03, 0x40, 0x06, 0x80, 0x0c, 0xc0, 0x12,
  0x35, 0x05, 0x08, 0x00, 0x00,
];

#[test]
fn decodes_the_captured_snapshot() {
  let info = DongleBaseInfo::parse(&snapshot).unwrap();
  assert_eq!(info.profile, 0);
  assert_eq!(info.dpi_levels, [400, 800, 1600, 3200, 4800]);
  assert_eq!(info.active_level, 3);
  assert_eq!(info.report_rate, 1);
  assert_eq!(info.debounce_ms, 8);
  assert_eq!(info.sleep_minutes, 0);
  assert_eq!(info.lift_off, 1);
}

#[test]
fn rejects_a_reply_from_another_opcode() {
  let mut other = snapshot;
  other[1] = 0x06;
  assert!(DongleBaseInfo::parse(&other).is_err());
}

#[test]
fn rejects_an_implausible_level_count() {
  let mut broken = snapshot;
  broken[17] = 9;
  assert!(DongleBaseInfo::parse(&broken).is_err());
}

#[test]
fn rejects_more_levels_than_the_reply_carries() {
  let mut truncated = snapshot;
  truncated[17] = 8;
  assert!(DongleBaseInfo::parse(&truncated).is_err());
}
