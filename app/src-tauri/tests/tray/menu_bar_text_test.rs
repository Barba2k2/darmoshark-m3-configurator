use darmoshark::error::darmoshark_error::DarmosharkError;
use darmoshark_app_lib::dto::device_state_dto::DeviceStateDto;
use darmoshark_app_lib::dto::snapshot_dto::SnapshotDto;
use darmoshark_app_lib::dto::transport::Transport;
use darmoshark_app_lib::tray::menu_bar_labels::MenuBarLabels;
use darmoshark_app_lib::tray::menu_bar_text::MenuBarText;

fn receiver() -> DeviceStateDto {
  DeviceStateDto {
    transport: Transport::Receiver,
    firmware_version: "2.0.9".into(),
    battery_percent: 25,
    snapshot: Some(SnapshotDto {
      dpi_levels: vec![400, 800, 1600, 3200, 4800],
      active_level: 3,
      report_rate_hz: Some(500),
      debounce_ms: 8,
      lift_off: 1,
      sleep_minutes: 0,
    }),
  }
}

#[test]
fn title_shows_battery_and_active_dpi_over_the_receiver() {
  assert_eq!(MenuBarText::title(&Ok(Some(receiver()))), "25% · 3200");
}

#[test]
fn title_shows_only_battery_over_the_cable() {
  let cable = DeviceStateDto {
    transport: Transport::Cable,
    snapshot: None,
    ..receiver()
  };
  assert_eq!(MenuBarText::title(&Ok(Some(cable))), "25%");
}

#[test]
fn title_is_blank_without_a_reading() {
  assert_eq!(MenuBarText::title(&Ok(None)), MenuBarLabels::missingValue);
  assert_eq!(
    MenuBarText::title(&Err(DarmosharkError::Asleep)),
    MenuBarLabels::missingValue
  );
}

#[test]
fn status_line_names_battery_and_transport() {
  assert_eq!(
    MenuBarText::status_line(&Ok(Some(receiver()))),
    format!(
      "{} 25% · {}",
      MenuBarLabels::batteryPrefix,
      MenuBarLabels::transportReceiver
    )
  );
}

#[test]
fn status_line_asks_to_wake_a_sleeping_mouse() {
  assert_eq!(
    MenuBarText::status_line(&Err(DarmosharkError::Asleep)),
    MenuBarLabels::wakeMouse
  );
  assert_eq!(MenuBarText::status_line(&Ok(None)), MenuBarLabels::noMouse);
}
