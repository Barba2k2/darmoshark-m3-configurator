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

  /// Round trip over the receiver.
  ///
  /// The reply lands in the feature report, which keeps the previous answer
  /// until the new one arrives. The echoed leading bytes -- the opcode, plus
  /// the slot for commands that address one (a button, a macro) -- tell which
  /// command an answer belongs to, but not whether it is this request's or an
  /// older one to the same command. So when the buffer already echoes this
  /// command, a primer read with a different echo is sent first; once the
  /// buffer holds the primer, an answer echoing this command can only be new.
  ///
  /// The 0xE4 acknowledgements on input 0x54 are not relied on: about one read
  /// in ten never posts "ready", and some commands (the bond read) post nothing.
  /// They are only watched for status 2, the dead link.
  pub fn request_dongle(
    &self,
    payload: &[u8],
    attempts: u32,
    echo_bytes: usize,
  ) -> DarmosharkResult<Option<Vec<u8>>> {
    let echo = &payload[..echo_bytes.min(payload.len())];
    let (feature_id, size) = Self::dongle_channel(payload.len())?;
    for attempt in 0..attempts {
      if self
        .get_feature(feature_id, size)
        .is_some_and(|reply| reply.get(1..1 + echo.len()) == Some(echo))
      {
        let primer = Self::primer(payload, echo_bytes);
        self.send_dongle(&primer)?;
        self.await_echo(feature_id, size, &primer[..echo.len()])?;
      }
      self.drain_input()?;
      self.send_dongle(payload)?;
      if let Some(reply) = self.await_echo(feature_id, size, echo)? {
        return Ok(Some(reply));
      }
      if attempt + 1 < attempts {
        thread::sleep(Duration::from_millis(300));
      }
    }
    Ok(None)
  }

  /// A read whose answer differs from `payload`'s: the same slot read aimed at
  /// another slot, or else the bond read -- the snapshot when the bond itself
  /// is asked. Its reply is discarded; it only has to replace the buffer.
  pub fn primer(payload: &[u8], echo_bytes: usize) -> Vec<u8> {
    if echo_bytes >= 2 {
      let mut primer = payload.to_vec();
      primer[1] = if primer[1] == 0 { 1 } else { 0 };
      return primer;
    }
    let mut primer = vec![0u8; DarmosharkProtocol::donglePayloadSize];
    primer[0] = if payload.first() == Some(&DmsCommands::getBondInfo) {
      DarmosharkProtocol::cmdDongleBaseInfo
    } else {
      DmsCommands::getBondInfo
    };
    primer
  }

  /// Delivers a dms payload over whichever transport this interface uses.
  ///
  /// The receiver takes the payload as feature report 0x51. The cable takes
  /// the identical bytes as feature report 0x52, zero padded to 64 bytes.
  ///
  /// The receiver relays a write to the mouse asynchronously and posts one
  /// 0xE4 frame when it went through, from a few ms to 800 ms later; a read
  /// before that still sees the old value. The write drains the queue first,
  /// so a leftover frame cannot end the wait early, then waits for its own
  /// frame so a read that follows is coherent. Silence is not an error -- the
  /// receiver does not promise the frame -- but a dead link is.
  pub fn send_command(&self, report_id: u8, payload: &[u8]) -> DarmosharkResult<()> {
    if self.uses_dongle_transport() {
      self.drain_input()?;
      self.send_dongle(payload)?;
      let deadline = Instant::now() + Duration::from_millis(1500);
      while let Some(frame) = self.read_input(deadline)? {
        if frame.len() > 2 && frame[1] == DmsCommands::ackOpcode {
          if frame[2] == DarmosharkProtocol::ackStatusLinkDown {
            return Err(DarmosharkError::Asleep);
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

  /// Polls the feature report until it echoes `echo`, for up to 1.5 s,
  /// failing at once if the receiver reports a dead link meanwhile.
  fn await_echo(
    &self,
    feature_id: u8,
    size: usize,
    echo: &[u8],
  ) -> DarmosharkResult<Option<Vec<u8>>> {
    let deadline = Instant::now() + Duration::from_millis(1500);
    loop {
      while let Some(frame) = self.read_input(Instant::now() + Duration::from_millis(1))? {
        if frame.len() > 2
          && frame[1] == DmsCommands::ackOpcode
          && frame[2] == DarmosharkProtocol::ackStatusLinkDown
        {
          return Err(DarmosharkError::Asleep);
        }
      }
      if let Some(reply) = self.get_feature(feature_id, size)
        && reply.get(1..1 + echo.len()) == Some(echo)
      {
        return Ok(Some(reply));
      }
      if Instant::now() >= deadline {
        return Ok(None);
      }
      thread::sleep(Duration::from_millis(5));
    }
  }

  /// Discards every input report already queued.
  fn drain_input(&self) -> DarmosharkResult<()> {
    while self
      .read_input(Instant::now() + Duration::from_millis(1))?
      .is_some()
    {}
    Ok(())
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
