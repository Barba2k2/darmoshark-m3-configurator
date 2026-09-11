//! The Tauri shell: thin commands over the `darmoshark` crate.
//!
//! - `state`: `DeviceGate`, which opens one HID handle per operation and lets
//!   only one operation hold it at a time (macOS opens it exclusively).
//! - `dto`: what crosses into the webview, serialised in camelCase.
//! - `commands`: the `#[tauri::command]` handlers the frontend invokes.

pub mod commands;
pub mod dto;
pub mod state;

use state::device_gate::DeviceGate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(DeviceGate::new())
    .invoke_handler(tauri::generate_handler![
      commands::read_commands::read_profile,
      commands::read_commands::read_device_state,
      commands::write_commands::write_dpi_levels,
      commands::write_commands::write_report_rate,
      commands::write_commands::write_lift_off,
      commands::write_commands::write_debounce,
      commands::write_commands::write_sleep_timer,
      commands::write_commands::restore_factory_defaults,
    ])
    .run(tauri::generate_context!())
    .expect("error while running the Darmoshark configurator");
}
