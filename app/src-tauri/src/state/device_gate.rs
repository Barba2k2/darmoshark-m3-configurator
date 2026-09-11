use std::sync::{Mutex, PoisonError};

use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

/// Hands out a freshly opened configurator, one operation at a time.
///
/// A handle is opened per operation rather than held, so unplugging the mouse
/// cannot leave the window stuck on a dead handle. macOS opens the interface
/// exclusively, so the battery poll and a write must never overlap.
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
  ) -> Result<T, String> {
    let _turn = self.turn.lock().unwrap_or_else(PoisonError::into_inner);
    let configurator =
      MouseConfigurator::new(DarmosharkDevice::open().map_err(|error| error.to_string())?);
    action(&configurator).map_err(|error| error.to_string())
  }
}

impl Default for DeviceGate {
  fn default() -> Self {
    Self::new()
  }
}
