use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;
use tauri::State;

use crate::dto::device_state_dto::DeviceStateDto;
use crate::dto::profile_dto::ProfileDto;
use crate::dto::snapshot_dto::SnapshotDto;
use crate::dto::transport::Transport;
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
  if DarmosharkDevice::discover()
    .map_err(|error| error.to_string())?
    .is_empty()
  {
    return Ok(None);
  }
  gate
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
