use darmoshark::packets::report_rate_packet::ReportRatePacket;

#[test]
fn encodes_rate_codes() {
  let codes: Vec<u32> = [1000, 1000, 500]
    .into_iter()
    .map(|hertz| ReportRatePacket::rate_to_code(hertz).unwrap())
    .collect();
  let packet = ReportRatePacket::build(&codes, 1, None).unwrap();
  assert_eq!(packet.report_id, 0xB5);
  assert_eq!(packet.payload[0], 65);
  assert_eq!(packet.payload[1..3], [1, 1]);
  assert_eq!(packet.payload[3..6], [2, 2, 1]);
  assert_eq!(packet.payload[9], 3);
}

#[test]
fn rate_code_round_trip() {
  for hertz in ReportRatePacket::supportedRates {
    let code = ReportRatePacket::rate_to_code(hertz).unwrap() as u8;
    assert_eq!(ReportRatePacket::code_to_rate(code), Some(hertz));
  }
}

#[test]
fn rejects_unsupported_rate() {
  assert!(ReportRatePacket::rate_to_code(8000).is_err());
}

#[test]
fn unknown_code_has_no_rate() {
  assert_eq!(ReportRatePacket::code_to_rate(3), None);
}
