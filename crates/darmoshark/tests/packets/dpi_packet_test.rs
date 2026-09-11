use darmoshark::packets::dpi_packet::DpiPacket;

#[test]
fn encodes_five_levels_little_endian() {
  let packet = DpiPacket::build(&[400, 800, 1600, 3200, 4800], 2, None).unwrap();
  assert_eq!(packet.report_id, 0xB5);
  assert_eq!(packet.payload[0], 0x40);
  assert_eq!(packet.payload[1..4], [2, 2, 2]);
  let decoded: Vec<u16> = (0..5)
    .map(|index| u16::from_le_bytes([packet.payload[4 + index * 2], packet.payload[5 + index * 2]]))
    .collect();
  assert_eq!(decoded, [400, 800, 1600, 3200, 4800]);
  assert_eq!(packet.payload[14], 5);
  assert_eq!(packet.payload.len(), 20);
}

#[test]
fn switches_to_long_form_beyond_five_levels() {
  let packet = DpiPacket::build(&[400, 800, 1200, 1600, 2400, 3200], 0, None).unwrap();
  assert_eq!(packet.report_id, 0xB3);
  assert_eq!(packet.payload[0], 0x44);
  assert_eq!(packet.payload[4], 6);
  assert_eq!(packet.payload.len(), 63);
}

#[test]
fn rejects_out_of_range_dpi() {
  for bad in [49, 26001] {
    assert!(DpiPacket::build(&[bad], 0, None).is_err());
  }
}

#[test]
fn rejects_active_level_outside_values() {
  assert!(DpiPacket::build(&[800, 1600], 5, None).is_err());
}

#[test]
fn rejects_too_many_levels() {
  assert!(DpiPacket::build(&[800; 9], 0, None).is_err());
}

#[test]
fn rejects_enabled_levels_outside_values() {
  assert!(DpiPacket::build(&[800, 1600], 0, Some(0)).is_err());
  assert!(DpiPacket::build(&[800, 1600], 0, Some(3)).is_err());
}
