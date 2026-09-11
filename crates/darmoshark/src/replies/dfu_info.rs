use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

/// Module model, firmware and hardware revision reported by the bootloader
/// (DFU cmd 96 on the config interface).
///
/// The reply is split across several 32-byte input reports. Byte 2 of the
/// first one carries the total payload length; the payload itself starts at
/// offset 5 and continues at offset 0 of the following reports.
///
/// Field offsets inside the reassembled payload:
///
/// ```text
/// [4..13]   module model, NUL padded ASCII
/// [16..25]  firmware version string
/// [26..35]  hardware version string
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DfuInfo {
  pub module_model: String,
  pub firmware_version: String,
  pub hardware_version: String,
  pub raw: Vec<u8>,
}

impl DfuInfo {
  pub fn parse(packets: &[Vec<u8>]) -> DarmosharkResult<Self> {
    let Some(first) = packets.first() else {
      return Err(DarmosharkError::Invalid("no DFU reply received".into()));
    };
    if first.len() < 5 {
      return Err(DarmosharkError::Invalid(format!(
        "DFU header truncated: {} bytes",
        first.len()
      )));
    }
    if first[0] != 0xAA || first[1] != 0x55 {
      return Err(DarmosharkError::Invalid(format!(
        "bad DFU header {:02X} {:02X}, expected AA 55",
        first[0], first[1]
      )));
    }
    if first[3] != !first[2] {
      return Err(DarmosharkError::Invalid(
        "DFU length checksum mismatch".into(),
      ));
    }

    let mut remaining = i64::from(first[2]);
    let mut payload = first[5..].to_vec();
    remaining -= payload.len() as i64;
    for packet in &packets[1..] {
      if remaining <= 0 {
        break;
      }
      let chunk = &packet[..packet.len().min(remaining as usize)];
      payload.extend_from_slice(chunk);
      remaining -= chunk.len() as i64;
    }

    if payload.len() < 36 {
      return Err(DarmosharkError::Invalid(format!(
        "DFU payload truncated: {} bytes",
        payload.len()
      )));
    }

    Ok(Self {
      module_model: Self::text(&payload[4..14]),
      firmware_version: Self::text(&payload[16..26]),
      hardware_version: Self::text(&payload[26..36]),
      raw: payload,
    })
  }

  /// NUL-stripped ASCII; every byte outside it becomes U+FFFD on its own, so a
  /// garbled reply never decodes into a plausible multi-byte character.
  fn text(chunk: &[u8]) -> String {
    chunk
      .iter()
      .filter(|&&byte| byte != 0)
      .map(|&byte| {
        if byte.is_ascii() {
          char::from(byte)
        } else {
          char::REPLACEMENT_CHARACTER
        }
      })
      .collect()
  }
}
