use darmoshark::packets::report_rate_packet::ReportRatePacket;

#[test]
fn encodes_the_rate_index_twice() {
  let packet = ReportRatePacket::build(1000).unwrap();
  assert_eq!(packet.report_id, 0xB5);
  assert_eq!(packet.payload[0..3], [65, 2, 2]);
  assert!(packet.payload[3..].iter().all(|&byte| byte == 0));
}

#[test]
fn rate_code_round_trip() {
  for hertz in ReportRatePacket::supportedRates {
    let code = ReportRatePacket::rate_to_code(hertz).unwrap();
    assert_eq!(ReportRatePacket::code_to_rate(code), Some(hertz));
  }
}

#[test]
fn rejects_unsupported_rate() {
  assert!(ReportRatePacket::build(8000).is_err());
}

#[test]
fn unknown_code_has_no_rate() {
  assert_eq!(ReportRatePacket::code_to_rate(3), None);
}
