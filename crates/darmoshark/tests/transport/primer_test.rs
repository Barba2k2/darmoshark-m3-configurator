use darmoshark::packets::button_packet::ButtonPacket;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

fn read(opcode: u8) -> Vec<u8> {
  let mut payload = vec![0u8; 20];
  payload[0] = opcode;
  payload
}

#[test]
fn a_snapshot_read_is_primed_with_the_bond_read() {
  assert_eq!(DarmosharkDevice::primer(&read(0x07), 1)[0], 0x03);
}

#[test]
fn the_bond_read_is_primed_with_the_snapshot() {
  assert_eq!(DarmosharkDevice::primer(&read(0x03), 1)[0], 0x07);
}

#[test]
fn a_button_read_is_primed_with_another_button() {
  for index in 0..ButtonPacket::buttonCount {
    let payload = ButtonPacket::build_read(index).unwrap().payload;
    let primer = DarmosharkDevice::primer(&payload, 2);
    assert_eq!(primer[0], payload[0]);
    assert_ne!(primer[1], payload[1]);
    assert!(primer[1] < ButtonPacket::buttonCount);
    assert_eq!(primer.len(), payload.len());
  }
}
