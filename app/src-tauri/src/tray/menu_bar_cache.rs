use std::sync::{Mutex, PoisonError};

use crate::dto::device_state_dto::DeviceStateDto;

/// The last device state the menu bar read, so a click can redraw at once
/// instead of waiting for the hardware round trip.
pub struct MenuBarCache {
  last: Mutex<Option<DeviceStateDto>>,
}

impl MenuBarCache {
  pub fn new() -> Self {
    Self {
      last: Mutex::new(None),
    }
  }

  pub fn store(&self, device: DeviceStateDto) {
    *self.last.lock().unwrap_or_else(PoisonError::into_inner) = Some(device);
  }

  pub fn last(&self) -> Option<DeviceStateDto> {
    self
      .last
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .clone()
  }
}

impl Default for MenuBarCache {
  fn default() -> Self {
    Self::new()
  }
}
