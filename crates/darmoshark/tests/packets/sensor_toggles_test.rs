use darmoshark::packets::sensor_toggles::SensorToggles;

#[test]
fn vendor_defaults_encode_as_the_vendor_packet() {
  assert_eq!(
    SensorToggles::vendorDefaults.packet_values(),
    [1, 2, 1, 1, 1]
  );
}

#[test]
fn scroll_and_e_sports_encode_on_as_two() {
  let all_on = SensorToggles {
    wave: true,
    line: true,
    motion: true,
    scroll: true,
    e_sports: true,
  };
  assert_eq!(all_on.packet_values(), [1, 1, 1, 2, 2]);
}

#[test]
fn wave_line_and_motion_encode_off_as_two() {
  let all_off = SensorToggles {
    wave: false,
    line: false,
    motion: false,
    scroll: false,
    e_sports: false,
  };
  assert_eq!(all_off.packet_values(), [2, 2, 2, 1, 1]);
}
