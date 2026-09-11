use darmoshark::profile::device_profile::DeviceProfile;
use tauri::State;

use crate::dto::device_state_dto::DeviceStateDto;
use crate::dto::profile_dto::ProfileDto;
use crate::state::device_gate::DeviceGate;

#[tauri::command]
pub async fn read_profile() -> Result<ProfileDto, String> {
  DeviceProfile::load()
    .map(|profile| ProfileDto::from_profile(&profile))
    .map_err(|error| error.to_string())
}

/// `None` when no Darmoshark interface is plugged in at all.
#[tauri::command]
pub async fn read_device_state(
  gate: State<'_, DeviceGate>,
) -> Result<Option<DeviceStateDto>, String> {
  gate.read_device_state().map_err(|error| error.to_string())
}
