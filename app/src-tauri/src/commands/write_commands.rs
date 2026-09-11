use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use tauri::{AppHandle, State};

use crate::state::device_gate::DeviceGate;
use crate::tray::menu_bar::MenuBar;

/// Runs a write and, once it lands, redraws the menu bar so it matches.
fn apply(
  app: &AppHandle,
  gate: &DeviceGate,
  write: impl FnOnce(&MouseConfigurator) -> DarmosharkResult<()>,
) -> Result<(), String> {
  gate
    .with_configurator(write)
    .map_err(|error| error.to_string())?;
  MenuBar::refresh_later(app);
  Ok(())
}

#[tauri::command]
pub async fn write_dpi_levels(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
  values: Vec<u32>,
  active_level: usize,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.write_dpi_levels(&values, active_level, None)
  })
}

#[tauri::command]
pub async fn write_report_rate(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
  hertz: u32,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.write_report_rate(hertz)
  })
}

#[tauri::command]
pub async fn write_lift_off(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
  value: u8,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.write_lift_off(value).map(|_| ())
  })
}

#[tauri::command]
pub async fn write_debounce(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
  milliseconds: u32,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.write_debounce(milliseconds)
  })
}

#[tauri::command]
pub async fn write_sleep_timer(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
  minutes: u32,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.write_sleep_timer(minutes)
  })
}

#[tauri::command]
pub async fn restore_factory_defaults(
  app: AppHandle,
  gate: State<'_, DeviceGate>,
) -> Result<(), String> {
  apply(&app, &gate, |configurator| {
    configurator.restore_factory_defaults()
  })
}
