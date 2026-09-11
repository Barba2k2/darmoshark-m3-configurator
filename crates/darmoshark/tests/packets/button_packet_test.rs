use darmoshark::packets::button_packet::ButtonPacket;

#[test]
fn write_layout() {
  let packet = ButtonPacket::build_write(1, "dpi", &[]).unwrap();
  assert_eq!(packet.report_id, 0xB3);
  assert_eq!(packet.payload[0], 82);
  assert_eq!(packet.payload[1], 1);
  assert_eq!(
    packet.payload[3],
    ButtonPacket::resolve_kind("dpi").unwrap()
  );
}

#[test]
fn read_round_trip() {
  let packet = ButtonPacket::build_read(3).unwrap();
  assert_eq!((packet.payload[0], packet.payload[1]), (98, 3));
  let macro_kind = ButtonPacket::resolve_kind("macro").unwrap();
  let parsed = ButtonPacket::parse_read(&[98, 3, 0, macro_kind, 7, 7], 3).unwrap();
  assert_eq!(parsed.kind, "macro");
  assert_eq!(parsed.data, [7, 7]);
}

#[test]
fn rejects_unknown_kind() {
  assert!(ButtonPacket::build_write(0, "teleport", &[]).is_err());
}

#[test]
fn rejects_button_out_of_range() {
  assert!(ButtonPacket::build_read(9).is_err());
}

#[test]
fn rejects_mismatched_reply() {
  assert!(ButtonPacket::parse_read(&[98, 1, 0, 5, 0], 3).is_err());
}

#[test]
fn rejects_data_that_does_not_fit_a_byte() {
  assert!(ButtonPacket::build_write(0, "keyboard", &[256]).is_err());
}

#[test]
fn names_unknown_kinds_by_value() {
  assert_eq!(ButtonPacket::kind_name(42), "unknown(42)");
}
