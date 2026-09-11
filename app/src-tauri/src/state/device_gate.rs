use std::sync::{Mutex, PoisonError};

use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

use crate::dto::device_state_dto::DeviceStateDto;
use crate::dto::snapshot_dto::SnapshotDto;
use crate::dto::transport::Transport;

/// Hands out a freshly opened configurator, one operation at a time.
///
/// A handle is opened per operation rather than held, so unplugging the mouse
/// cannot leave the app stuck on a dead handle. macOS opens the interface
/// exclusively, so the window, the menu bar poll and any write must never
/// overlap.
pub struct DeviceGate {
  turn: Mutex<()>,
}

impl DeviceGate {
  pub fn new() -> Self {
    Self {
      turn: Mutex::new(()),
    }
  }

  pub fn with_configurator<T>(
    &self,
    action: impl FnOnce(&MouseConfigurator) -> DarmosharkResult<T>,
  ) -> DarmosharkResult<T> {
    let _turn = self.turn.lock().unwrap_or_else(PoisonError::into_inner);
    action(&MouseConfigurator::new(DarmosharkDevice::open()?))
  }

  /// What the window and the menu bar show. `None` when no Darmoshark
  /// interface is plugged in at all.
  pub fn read_device_state(&self) -> DarmosharkResult<Option<DeviceStateDto>> {
    if DarmosharkDevice::discover()?.is_empty() {
      return Ok(None);
    }
    self
      .with_configurator(|configurator| {
        let device = configurator.device();
        let identity = configurator.read_cable_info()?;
        let (transport, snapshot) = if device.uses_dongle_transport() {
          let snapshot = SnapshotDto::from_dongle(&configurator.read_dongle_base_info()?);
          (Transport::Receiver, Some(snapshot))
        } else if device.uses_cable_transport() {
          (Transport::Cable, None)
        } else {
          (Transport::Other, None)
        };
        Ok(DeviceStateDto {
          transport,
          firmware_version: identity.firmware_version,
          battery_percent: identity.battery_percent,
          snapshot,
        })
      })
      .map(Some)
  }
}

impl Default for DeviceGate {
  fn default() -> Self {
    Self::new()
  }
}
