use darmoshark::packets::sleep_mode::SleepMode;
use darmoshark::packets::tuning_packet::TuningPacket;

#[test]
fn debounce_layout() {
  let packet = TuningPacket::build_debounce(8).unwrap();
  assert_eq!(
    (packet.report_id, packet.payload[0], packet.payload[1]),
    (0xB5, 67, 8)
  );
}

#[test]
fn debounce_range_is_enforced() {
  assert!(TuningPacket::build_debounce(21).is_err());
}

#[test]
fn sensor_layout_matches_vendor_offsets() {
  let packet = TuningPacket::build_sensor(2, 1, 2, 1, 1, 1).unwrap();
  assert_eq!(packet.payload[0], 66);
  assert_eq!(packet.payload[1], 2);
  assert_eq!(packet.payload[6], 1);
  assert_eq!(packet.payload[7], 1);
}

#[test]
fn rejects_invalid_lift_off() {
  assert!(TuningPacket::build_sensor(9, 1, 2, 1, 1, 1).is_err());
}

#[test]
fn sleep_uses_set_mode_marker() {
  let packet = TuningPacket::build_sleep(10, SleepMode::Set).unwrap();
  assert_eq!(packet.payload[0..3], [10, 1, 10]);
}

#[test]
fn sleep_range_is_enforced() {
  assert!(TuningPacket::build_sleep(256, SleepMode::Set).is_err());
}

#[test]
fn scroll_layout() {
  let packet = TuningPacket::build_scroll(3, 1, 2);
  assert_eq!(packet.payload[0..4], [69, 3, 1, 2]);
}
