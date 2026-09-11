use tauri::State;

use crate::state::device_gate::DeviceGate;

#[tauri::command]
pub async fn write_dpi_levels(
  gate: State<'_, DeviceGate>,
  values: Vec<u32>,
  active_level: usize,
) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.write_dpi_levels(&values, active_level, None))
}

#[tauri::command]
pub async fn write_report_rate(gate: State<'_, DeviceGate>, hertz: u32) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.write_report_rate(hertz))
}

#[tauri::command]
pub async fn write_lift_off(gate: State<'_, DeviceGate>, value: u8) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.write_lift_off(value).map(|_| ()))
}

#[tauri::command]
pub async fn write_debounce(gate: State<'_, DeviceGate>, milliseconds: u32) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.write_debounce(milliseconds))
}

#[tauri::command]
pub async fn write_sleep_timer(gate: State<'_, DeviceGate>, minutes: u32) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.write_sleep_timer(minutes))
}

#[tauri::command]
pub async fn restore_factory_defaults(gate: State<'_, DeviceGate>) -> Result<(), String> {
  gate.with_configurator(|configurator| configurator.restore_factory_defaults())
}
