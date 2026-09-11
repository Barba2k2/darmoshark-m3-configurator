use darmoshark::packets::profile_packet::ProfilePacket;

#[test]
fn switch_layout() {
  let packet = ProfilePacket::build_switch(2).unwrap();
  assert_eq!((packet.payload[0], packet.payload[1]), (14, 2));
}

#[test]
fn factory_reset_uses_value_63() {
  let packet = ProfilePacket::build_factory_reset();
  assert_eq!((packet.payload[0], packet.payload[1]), (15, 63));
}

#[test]
fn rejects_unknown_profile() {
  assert!(ProfilePacket::build_switch(9).is_err());
}
