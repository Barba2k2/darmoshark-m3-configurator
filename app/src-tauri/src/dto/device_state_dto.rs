use serde::Serialize;

use crate::dto::snapshot_dto::SnapshotDto;
use crate::dto::transport::Transport;

/// What the window shows about the connected mouse. `snapshot` is present
/// only over the receiver: the cable cannot read the configuration back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStateDto {
  pub transport: Transport,
  pub firmware_version: String,
  pub battery_percent: u8,
  pub snapshot: Option<SnapshotDto>,
}
