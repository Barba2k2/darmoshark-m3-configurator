use clap::Subcommand;
use clap::builder::PossibleValuesParser;
use darmoshark::packets::button_packet::ButtonPacket;

/// Every `dms` subcommand. Reads marked [receiver] need the 2.4GHz receiver.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
  /// list the HID interfaces exposed by the mouse
  List,
  /// what this model supports (offline)
  Capabilities,
  /// LED colour legend for DPI and polling rate
  Colors,
  /// [receiver] which mouse the receiver is linked to
  Bond,
  /// identity and battery (cable)
  Battery,
  /// bootloader, firmware and hardware revision (cable)
  Dfu,
  /// [receiver] full configuration snapshot
  Info,
  /// [receiver] current button assignments
  Buttons,
  /// restore factory defaults
  Reset,
  /// program the DPI levels
  Dpi {
    /// 50-26000 per level
    #[arg(required = true)]
    values: Vec<u32>,
    /// index of the level to activate
    #[arg(long, default_value_t = 0)]
    active: usize,
  },
  /// switch DPI level (reprograms with profile defaults over the cable)
  Use { level: usize },
  /// set the polling rate per level (affects wireless)
  Rate {
    /// one of 125, 500, 1000 per level
    #[arg(required = true)]
    values: Vec<u32>,
    #[arg(long, default_value_t = 0)]
    active: usize,
  },
  /// click debounce, in ms
  Debounce { milliseconds: u32 },
  /// lift-off distance
  Lod {
    /// 1 = low, 2 = high
    #[arg(value_parser = clap::value_parser!(u8).range(1..=2))]
    value: u8,
  },
  /// idle sleep timer, in minutes (cable only)
  Sleep { minutes: u32 },
  /// switch onboard profile
  Profile { index: u8 },
  /// remap a button
  Button {
    index: u8,
    #[arg(value_parser = PossibleValuesParser::new(ButtonPacket::kinds.map(|(name, _)| name)))]
    kind: String,
    /// optional assignment bytes
    data: Vec<u32>,
  },
}
