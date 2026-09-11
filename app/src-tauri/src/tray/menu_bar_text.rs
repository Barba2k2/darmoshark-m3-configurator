use darmoshark::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

use crate::dto::device_state_dto::DeviceStateDto;
use crate::dto::transport::Transport;
use crate::tray::menu_bar_labels::MenuBarLabels;

/// Turns a device read into the text the menu bar shows.
pub struct MenuBarText;

impl MenuBarText {
  /// Next to the icon: battery, plus the active DPI when it can be read
  /// (receiver only). `--` when there is nothing to show.
  pub fn title(state: &DarmosharkResult<Option<DeviceStateDto>>) -> String {
    let Ok(Some(device)) = state else {
      return MenuBarLabels::missingValue.to_string();
    };
    let active_dpi = device
      .snapshot
      .as_ref()
      .and_then(|snapshot| snapshot.dpi_levels.get(usize::from(snapshot.active_level)));
    match active_dpi {
      Some(dpi) => format!("{}% · {dpi}", device.battery_percent),
      None => format!("{}%", device.battery_percent),
    }
  }

  /// The first, disabled line of the menu: what is connected, or why not.
  pub fn status_line(state: &DarmosharkResult<Option<DeviceStateDto>>) -> String {
    match state {
      Ok(Some(device)) => format!(
        "{} {}% · {}",
        MenuBarLabels::batteryPrefix,
        device.battery_percent,
        Self::transport(device.transport)
      ),
      Ok(None) => MenuBarLabels::noMouse.to_string(),
      Err(DarmosharkError::Asleep) => MenuBarLabels::wakeMouse.to_string(),
      Err(error) => error.to_string(),
    }
  }

  pub fn transport(transport: Transport) -> &'static str {
    match transport {
      Transport::Receiver => MenuBarLabels::transportReceiver,
      Transport::Cable => MenuBarLabels::transportCable,
      Transport::Other => MenuBarLabels::transportOther,
    }
  }
}
