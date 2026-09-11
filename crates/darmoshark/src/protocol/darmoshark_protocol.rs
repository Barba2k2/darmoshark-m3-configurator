//! Wire protocol constants for Darmoshark mice (contract "dms").
//!
//! Reverse engineered from the official WebHID configurator bundle served by
//! darmoshark.cc (Angular app, Keychron launcher platform).

/// Report ids, opcodes and limits of the Darmoshark "dms" contract.
pub struct DarmosharkProtocol;

impl DarmosharkProtocol {
  pub const vendorId: u16 = 0x248A;
  pub const knownProductIds: [u16; 5] = [0xFF12, 0xFF18, 0xFF10, 0xFF30, 0xFF31];

  // Output report ids used to carry commands.
  pub const longReportId: u8 = 0xB3; // 63-byte payload (reads, >5 dpi levels)
  pub const shortReportId: u8 = 0xB5; // 20-byte payload (most writes)

  pub const longPayloadSize: usize = 63;
  pub const shortPayloadSize: usize = 20;

  // Opcodes (payload byte 0).
  pub const cmdGetBaseInfo: u8 = 0x06;
  pub const cmdSetDpiShort: u8 = 0x40; // up to 5 levels, sent on shortReportId
  pub const cmdSetDpiLong: u8 = 0x44; // more than 5 levels, sent on longReportId

  // Input report opcodes that carry a base-info reply.
  pub const baseInfoReplyOpcodes: [u8; 2] = [0x05, 0x06];

  // Both transports live on usage page 0x8C: the mouse exposes it over the
  // charging cable, the receiver exposes an identical descriptor of its own.
  // The report ids 0xB3 / 0xB5 exist in the protocol but not in this
  // descriptor -- writing them reaches nothing. Confirmed on hardware.
  pub const dfuUsagePage: u16 = 0x8C;
  pub const cableConfigFeatureId: u8 = 0x52;
  pub const cableConfigFeatureSize: usize = 64;

  // 2.4GHz receiver. It enumerates under its own product id and carries the
  // same 20-byte payloads as feature report 0x51, answering on input report
  // 0x54. Unlike the cable, it also reads configuration back.
  pub const dongleProductIds: [u16; 1] = [0xFF30];
  pub const dongleConfigFeatureId: u8 = 0x51;
  pub const donglePayloadSize: usize = 20;
  // Commands that do not fit the short report take the 64-byte one, the same
  // id the cable uses -- button reads and macro data travel here.
  pub const dongleLongFeatureId: u8 = 0x52;
  pub const dongleLongPayloadSize: usize = 64;
  pub const dongleAckInputId: u8 = 0x54;
  pub const cmdDongleBaseInfo: u8 = 0x07; // config snapshot; the cable contract uses 0x06

  // Status byte of the 0xE4 acknowledgement the receiver posts on 0x54.
  pub const ackStatusPending: u8 = 0; // command queued; "ready" may never follow
  pub const ackStatusReady: u8 = 1; // reply is waiting in the feature report
  pub const ackStatusLinkDown: u8 = 2; // receiver has no live link to the mouse
  pub const ackStatusBusy: u8 = 4; // same as pending, receiver still working

  pub const dpiMinimum: u16 = 50;
  pub const dpiMaximum: u16 = 26000;
  pub const maxShortLevels: usize = 5;
  pub const maxLevels: usize = 8;

  // Cable-side channel: identify/battery only, no DPI state.
  pub const identifyFeatureId: u8 = 0x51;
  pub const identifyPayloadSize: usize = 20;
  pub const cmdIdentify: u8 = 0x06;

  // DFU channel (cable interface, usage page 0x8C).
  pub const dfuOutputId: u8 = 0xB2;
  pub const dfuInputId: u8 = 0xB1;
  pub const dfuPayloadSize: usize = 32;
  pub const dfuHeaderByte: u8 = 0xAA;
  pub const dfuSendNoAck: u8 = 0x55;
  pub const dfuSendAck: u8 = 0x56;
  pub const cmdDfuModuleInfo: u8 = 96;
  pub const cmdDfuVersion: u8 = 97;
}
