//! The Tauri shell: thin commands over the `darmoshark` crate.
//!
//! - `state`: `DeviceGate`, which opens one HID handle per operation and lets
//!   only one operation hold it at a time (macOS opens it exclusively).
//! - `dto`: what crosses into the webview, serialised in camelCase.
//! - `commands`: the `#[tauri::command]` handlers the frontend invokes.
//! - `tray`: the menu bar status item, the app's only permanent presence -- it has
//!   no Dock icon, and closing the window only hides it.

// Constants follow the project naming rule: camelCase, never SCREAMING_CASE.
#![allow(non_upper_case_globals)]

pub mod commands;
pub mod dto;
pub mod state;
pub mod tray;

use state::device_gate::DeviceGate;
use tauri::WindowEvent;
use tray::menu_bar::MenuBar;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(DeviceGate::new())
    .setup(|app| {
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);
      MenuBar::install(app.handle())?;
      Ok(())
    })
    .on_window_event(|window, event| {
      if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        if let Err(error) = window.hide() {
          eprintln!("could not hide the configurator: {error}");
        }
      }
    })
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
