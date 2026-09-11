/// The sensor switches that share the lift-off packet (opcode 66).
///
/// The receiver snapshot stores each as a bit, 1 meaning on. The packet uses
/// the vendor form values instead, and not all the same way: wave, line and
/// motion send 1 for on and 2 for off, while scroll and eSports send 2 for on
/// and 1 for off. The vendor defaults (wave and motion on, the rest off) are
/// what the tested M3 reports in sensor byte `0x35`; bit 5 of that byte is set
/// too, and no known field maps to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorToggles {
  pub wave: bool,
  pub line: bool,
  pub motion: bool,
  pub scroll: bool,
  pub e_sports: bool,
}

impl SensorToggles {
  pub const vendorDefaults: SensorToggles = SensorToggles {
    wave: true,
    line: false,
    motion: true,
    scroll: false,
    e_sports: false,
  };

  /// Packet bytes 2, 3, 4, 6 and 7, in that order.
  pub fn packet_values(&self) -> [u8; 5] {
    let on_is_one = |on: bool| if on { 1 } else { 2 };
    let on_is_two = |on: bool| if on { 2 } else { 1 };
    [
      on_is_one(self.wave),
      on_is_one(self.line),
      on_is_one(self.motion),
      on_is_two(self.scroll),
      on_is_two(self.e_sports),
    ]
  }
}
