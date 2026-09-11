use std::thread;
use std::time::{Duration, Instant};

use hidapi::{DeviceInfo, HidApi, HidDevice};

use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use crate::protocol::darmoshark_protocol::DarmosharkProtocol;
use crate::protocol::dms_commands::DmsCommands;

/// Opens the vendor config interface and exchanges reports with it.
///
/// Two interfaces carry the same payloads on usage page 0x8C: the mouse
/// itself over the charging cable (feature report 0x52, write only), and the
/// 2.4GHz receiver (feature report 0x51, reads included). This is the only
/// type in the crate that talks to `hidapi`.
pub struct DarmosharkDevice {
  handle: HidDevice,
  product_id: u16,
  usage_page: u16,
}

impl DarmosharkDevice {
  /// Returns every candidate config interface, best match first.
  pub fn discover() -> DarmosharkResult<Vec<DeviceInfo>> {
    let api = Self::api()?;
    let mut candidates = Vec::new();
    for entry in api
      .device_list()
      .filter(|entry| entry.vendor_id() == DarmosharkProtocol::vendorId)
    {
      if entry.usage_page() == DarmosharkProtocol::dfuUsagePage {
        candidates.insert(0, entry.clone());
      } else if entry.usage_page() >= 0xFF00 {
        candidates.push(entry.clone());
      }
    }
    Ok(candidates)
  }

  pub fn open() -> DarmosharkResult<Self> {
    let candidates = Self::discover()?;
    if candidates.is_empty() {
      return Err(DarmosharkError::Device(
        "no Darmoshark device found. Connect the charging cable, or plug in \
         the 2.4GHz receiver with the mouse switch set to 2.4G."
          .into(),
      ));
    }

    let api = Self::api()?;
    let mut errors = Vec::new();
    for entry in candidates {
      match api.open_path(entry.path()) {
        Ok(handle) => {
          return Ok(Self {
            handle,
            product_id: entry.product_id(),
            usage_page: entry.usage_page(),
          });
        }
        Err(error) => errors.push(format!("{}: {error}", entry.path().to_string_lossy())),
      }
    }
    Err(DarmosharkError::Device(format!(
      "could not open any interface:\n  {}",
      errors.join("\n  ")
    )))
  }

  /// True when the open interface belongs to the 2.4GHz receiver.
  pub fn uses_dongle_transport(&self) -> bool {
    DarmosharkProtocol::dongleProductIds.contains(&self.product_id)
  }

  /// True when the open interface is the mouse's own cable one.
  pub fn uses_cable_transport(&self) -> bool {
    self.usage_page == DarmosharkProtocol::dfuUsagePage && !self.uses_dongle_transport()
  }

  /// Sends a command on an output report and waits for the next input
  /// report, or `None` on timeout.
  pub fn request(&self, report_id: u8, payload: &[u8]) -> DarmosharkResult<Option<Vec<u8>>> {
    self.send(report_id, payload)?;
    self.read_input(Instant::now() + Duration::from_secs(1))
  }

  /// Feature-report round trip: write the command, read the answer back.
  pub fn request_feature(
    &self,
    report_id: u8,
    payload: &[u8],
    retries: u32,
  ) -> DarmosharkResult<Option<Vec<u8>>> {
    if self.uses_dongle_transport() {
      return self.request_dongle(payload, retries, 1);
    }

    for _ in 0..retries {
      self
        .handle
        .send_feature_report(&Self::frame(report_id, payload, payload.len()))
        .map_err(|error| {
          DarmosharkError::Device(format!(
            "failed to send feature report 0x{report_id:02X}: {error}"
          ))
        })?;
      thread::sleep(Duration::from_millis(80));
      if let Some(reply) = self.get_feature(report_id, payload.len()) {
        return Ok(Some(reply));
      }
      thread::sleep(Duration::from_millis(100));
    }
    Ok(None)
  }

  /// Sends an AA/55 DFU frame and collects the multi-packet reply, each
  /// packet without its echoed report id.
  pub fn request_dfu(&self, kind: u8, command: u8) -> DarmosharkResult<Vec<Vec<u8>>> {
    let mut frame = vec![0u8; DarmosharkProtocol::dfuPayloadSize];
    frame[0] = DarmosharkProtocol::dfuHeaderByte;
    frame[1] = DarmosharkProtocol::dfuSendNoAck;
    frame[2] = 3;
    frame[3] = !3u8;
    frame[4] = kind;
    frame[5] = command;
    frame[6] = command;

    self
      .handle
      .write(&Self::frame(
        DarmosharkProtocol::dfuOutputId,
        &frame,
        frame.len(),
      ))
      .map_err(|error| {
        DarmosharkError::Device(format!("failed to send DFU command {command}: {error}"))
      })?;

    let deadline = Instant::now() + Duration::from_millis(800);
    let mut packets = Vec::new();
    while let Some(reply) = self.read_input(deadline)? {
      packets.push(reply[1..].to_vec());
    }
    Ok(packets)
  }

  /// Picks the receiver report that fits the payload: 0x51 or 0x52.
  pub fn dongle_channel(payload_size: usize) -> DarmosharkResult<(u8, usize)> {
    if payload_size <= DarmosharkProtocol::donglePayloadSize {
      return Ok((
        DarmosharkProtocol::dongleConfigFeatureId,
        DarmosharkProtocol::donglePayloadSize,
      ));
    }
    if payload_size <= DarmosharkProtocol::dongleLongPayloadSize {
      return Ok((
        DarmosharkProtocol::dongleLongFeatureId,
        DarmosharkProtocol::dongleLongPayloadSize,
      ));
    }
    Err(DarmosharkError::Invalid(format!(
      "payload of {payload_size} bytes exceeds the {}-byte receiver report",
      DarmosharkProtocol::dongleLongPayloadSize
    )))
  }

  /// Delivers a payload as a receiver feature report, zero padded.
  pub fn send_dongle(&self, payload: &[u8]) -> DarmosharkResult<()> {
    let (feature_id, size) = Self::dongle_channel(payload.len())?;
    self
      .handle
      .send_feature_report(&Self::frame(feature_id, payload, size))
      .map_err(|error| DarmosharkError::Device(format!("failed to send receiver command: {error}")))
  }

  /// Round trip over the receiver, honouring its acknowledgement states.
  ///
  /// The receiver answers on input report 0x54 with 0xE4 <status>: pending
  /// and busy both mean "ask again in a moment", ready means the reply is
  /// sitting in the feature report. The ack carries no opcode, so frames left
  /// in the queue by an earlier command -- a write posts its pending ack about
  /// 600 ms later -- are drained before sending, and the feature report is
  /// read only once this request turns ready.
  ///
  /// The reply buffer keeps the previous answer until the new one lands, so
  /// the echoed leading bytes are what prove the data belongs to this request
  /// -- one byte is the opcode, and commands addressing a slot (a button, a
  /// macro) need two. Two reads of the same opcode echo the same bytes, which
  /// is why a request left pending is sent again rather than read. Some
  /// commands (the bond read) post no ack at all; their reply is read as is.
  pub fn request_dongle(
    &self,
    payload: &[u8],
    attempts: u32,
    echo_bytes: usize,
  ) -> DarmosharkResult<Option<Vec<u8>>> {
    let echo = &payload[..echo_bytes.min(payload.len())];
    let (feature_id, size) = Self::dongle_channel(payload.len())?;
    for attempt in 0..attempts {
      while self
        .read_input(Instant::now() + Duration::from_millis(1))?
        .is_some()
      {}
      self.send_dongle(payload)?;
      let status = self.await_dongle_ack()?;
      if status == Some(DarmosharkProtocol::ackStatusLinkDown) {
        return Err(Self::link_down());
      }
      if matches!(status, None | Some(DarmosharkProtocol::ackStatusReady))
        && let Some(reply) = self.get_feature(feature_id, size)
        && reply.get(1..1 + echo.len()) == Some(echo)
      {
        return Ok(Some(reply));
      }
      if attempt + 1 < attempts {
        thread::sleep(Duration::from_millis(300));
      }
    }
    Ok(None)
  }

  /// Delivers a dms payload over whichever transport this interface uses.
  ///
  /// The receiver takes the payload as feature report 0x51. The cable takes
  /// the identical bytes as feature report 0x52, zero padded to 64 bytes.
  ///
  /// The receiver relays a write to the mouse asynchronously and posts one
  /// 0xE4 frame when it went through, 300-800 ms later; a read before that
  /// still sees the old value. The write waits for that frame so a read that
  /// follows is coherent. Silence is not an error -- the receiver does not
  /// promise the frame -- but a dead link is.
  pub fn send_command(&self, report_id: u8, payload: &[u8]) -> DarmosharkResult<()> {
    if self.uses_dongle_transport() {
      self.send_dongle(payload)?;
      let deadline = Instant::now() + Duration::from_millis(1500);
      while let Some(frame) = self.read_input(deadline)? {
        if frame.len() > 2 && frame[1] == DmsCommands::ackOpcode {
          if frame[2] == DarmosharkProtocol::ackStatusLinkDown {
            return Err(Self::link_down());
          }
          break;
        }
      }
      return Ok(());
    }
    if !self.uses_cable_transport() {
      return self.send(report_id, payload);
    }

    let size = DarmosharkProtocol::cableConfigFeatureSize;
    if payload.len() > size {
      return Err(DarmosharkError::Invalid(format!(
        "payload of {} bytes exceeds the {size}-byte cable config report",
        payload.len()
      )));
    }
    self
      .handle
      .send_feature_report(&Self::frame(
        DarmosharkProtocol::cableConfigFeatureId,
        payload,
        size,
      ))
      .map_err(|error| DarmosharkError::Device(format!("failed to send config command: {error}")))
  }

  /// Sends a dms command and returns the reply, transport aware. Over the
  /// cable the echoed report id is stripped from the reply.
  pub fn request_command(
    &self,
    report_id: u8,
    payload: &[u8],
  ) -> DarmosharkResult<Option<Vec<u8>>> {
    if self.uses_dongle_transport() {
      return self.request_dongle(payload, 4, 1);
    }
    if !self.uses_cable_transport() {
      return self.request(report_id, payload);
    }

    self.send_command(report_id, payload)?;
    thread::sleep(Duration::from_millis(80));
    Ok(
      self
        .get_feature(
          DarmosharkProtocol::cableConfigFeatureId,
          DarmosharkProtocol::cableConfigFeatureSize,
        )
        .map(|reply| reply[1..].to_vec()),
    )
  }

  pub fn send(&self, report_id: u8, payload: &[u8]) -> DarmosharkResult<()> {
    self
      .handle
      .write(&Self::frame(report_id, payload, payload.len()))
      .map(|_| ())
      .map_err(|error| {
        DarmosharkError::Device(format!("failed to write report 0x{report_id:02X}: {error}"))
      })
  }

  fn api() -> DarmosharkResult<HidApi> {
    HidApi::new()
      .map_err(|error| DarmosharkError::Device(format!("could not initialise hidapi: {error}")))
  }

  /// Report id followed by the payload, zero padded to `size` bytes.
  fn frame(report_id: u8, payload: &[u8], size: usize) -> Vec<u8> {
    let mut framed = Vec::with_capacity(size + 1);
    framed.push(report_id);
    framed.extend_from_slice(payload);
    framed.resize(size.max(payload.len()) + 1, 0);
    framed
  }

  /// Waits for a settled 0xE4 status -- ready or link down -- skipping the
  /// pending and busy frames. On timeout, the last unsettled status seen, or
  /// `None` when no ack arrived at all.
  fn await_dongle_ack(&self) -> DarmosharkResult<Option<u8>> {
    let settled = [
      DarmosharkProtocol::ackStatusReady,
      DarmosharkProtocol::ackStatusLinkDown,
    ];
    let deadline = Instant::now() + Duration::from_millis(600);
    let mut last = None;
    while let Some(frame) = self.read_input(deadline)? {
      if frame.len() > 2 && frame[1] == DmsCommands::ackOpcode {
        if settled.contains(&frame[2]) {
          return Ok(Some(frame[2]));
        }
        last = Some(frame[2]);
      }
    }
    Ok(last)
  }

  fn link_down() -> DarmosharkError {
    DarmosharkError::Device(
      "the receiver reports no live link to the mouse. Set the switch to 2.4G, \
       then unplug and replug the receiver -- a dropped link does not recover \
       on its own."
        .into(),
    )
  }

  /// Next input report that arrives before `deadline`, report id included.
  fn read_input(&self, deadline: Instant) -> DarmosharkResult<Option<Vec<u8>>> {
    let mut buffer = [0u8; 64];
    loop {
      let left = deadline.saturating_duration_since(Instant::now());
      if left.is_zero() {
        return Ok(None);
      }
      let wait = left.min(Duration::from_millis(10)).as_millis().max(1) as i32;
      let count = self
        .handle
        .read_timeout(&mut buffer, wait)
        .map_err(|error| {
          DarmosharkError::Device(format!("failed to read input report: {error}"))
        })?;
      if count > 0 {
        return Ok(Some(buffer[..count].to_vec()));
      }
    }
  }

  /// Reads a feature report back; `None` when it fails or holds only zeros.
  /// The reply keeps the report id at index 0.
  fn get_feature(&self, report_id: u8, size: usize) -> Option<Vec<u8>> {
    let mut buffer = vec![0u8; size + 1];
    buffer[0] = report_id;
    let count = self.handle.get_feature_report(&mut buffer).ok()?;
    buffer.truncate(count);
    (buffer.len() > 2 && buffer[1..].iter().any(|&byte| byte != 0)).then_some(buffer)
  }
}
