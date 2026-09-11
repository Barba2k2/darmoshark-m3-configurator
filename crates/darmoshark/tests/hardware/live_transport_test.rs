//! Exercises the real transport against a plugged-in mouse or receiver.
//!
//! Runs only with `DARMOSHARK_HARDWARE=1`, so CI and machines without the
//! hardware skip it. Every write changes a setting, reads it back, then
//! restores the original, leaving the mouse configured as it was.

use std::sync::Mutex;

use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::packets::button_packet::ButtonPacket;
use darmoshark::packets::report_rate_packet::ReportRatePacket;
use darmoshark::packets::sensor_toggles::SensorToggles;
use darmoshark::protocol::darmoshark_protocol::DarmosharkProtocol;
use darmoshark::replies::base_snapshot::BaseSnapshot;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

// macOS opens HID interfaces exclusively: tests must take turns.
static hardware: Mutex<()> = Mutex::new(());

fn configurator() -> Option<MouseConfigurator> {
  if std::env::var("DARMOSHARK_HARDWARE").as_deref() != Ok("1") {
    eprintln!("skipped: set DARMOSHARK_HARDWARE=1 with the mouse or receiver plugged in");
    return None;
  }
  Some(MouseConfigurator::new(DarmosharkDevice::open().unwrap()))
}

fn receiver() -> Option<MouseConfigurator> {
  let configurator = configurator()?;
  if configurator.device().uses_dongle_transport() {
    return Some(configurator);
  }
  eprintln!("skipped: the open interface is not the 2.4GHz receiver");
  None
}

#[test]
fn identifies_the_mouse() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = configurator() else {
    return;
  };
  let info = configurator.read_cable_info().unwrap();
  eprintln!("{info:?}");
  assert_eq!(info.vendor_id, DarmosharkProtocol::vendorId);
  assert!(info.battery_percent <= 100);
}

#[test]
fn reads_the_stored_configuration() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let snapshot = configurator.read_base_info().unwrap();
  eprintln!("{snapshot:?}");
  let BaseSnapshot::Dongle(info) = &snapshot else {
    panic!("the receiver answered with a config-interface block");
  };
  let range = DarmosharkProtocol::dpiMinimum..=DarmosharkProtocol::dpiMaximum;
  assert!(info.dpi_levels.iter().all(|level| range.contains(level)));
  assert!(usize::from(info.active_level) < info.dpi_levels.len());
}

#[test]
fn reports_a_live_bond() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let bond = configurator.read_bond_info().unwrap();
  eprintln!("{bond:?}");
  assert_eq!(bond.vendor_id, DarmosharkProtocol::vendorId);
  assert!(bond.linked);
}

#[test]
fn reads_every_button() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let buttons = configurator.read_all_buttons().unwrap();
  eprintln!("{buttons:?}");
  let indexes: Vec<u8> = buttons.iter().map(|button| button.button).collect();
  assert_eq!(indexes, (0..ButtonPacket::buttonCount).collect::<Vec<_>>());
}

#[test]
fn reads_the_bootloader_identity() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = configurator() else {
    return;
  };
  let info = configurator.read_dfu_info().unwrap();
  eprintln!("{info:?}");
  assert!(!info.module_model.is_empty());
}

#[test]
fn switches_the_active_dpi_level_and_restores_it() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let before = configurator.read_base_info().unwrap();
  let original = usize::from(before.active_level());
  let other = (original + 1) % before.dpi_levels().len();
  configurator.select_dpi_level(other).unwrap();
  let switched = configurator.read_base_info().unwrap();
  configurator.select_dpi_level(original).unwrap();
  let restored = configurator.read_base_info().unwrap();

  assert_eq!(usize::from(switched.active_level()), other);
  assert_eq!(switched.dpi_levels(), before.dpi_levels());
  assert_eq!(restored.active_level(), before.active_level());
}

#[test]
fn changes_the_debounce_and_restores_it() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let debounce = |configurator: &MouseConfigurator| match configurator.read_base_info().unwrap() {
    BaseSnapshot::Dongle(info) => info.debounce_ms,
    BaseSnapshot::Config(_) => panic!("the receiver answered with a config-interface block"),
  };
  let original = debounce(&configurator);
  let other = if original <= 18 {
    original + 2
  } else {
    original - 2
  };
  configurator.write_debounce(u32::from(other)).unwrap();
  let changed = debounce(&configurator);
  configurator.write_debounce(u32::from(original)).unwrap();
  let restored = debounce(&configurator);

  assert_eq!(changed, other);
  assert_eq!(restored, original);
}

#[test]
fn each_sensor_switch_lands_on_its_own_bit_and_restores() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let stored = |configurator: &MouseConfigurator| configurator.read_dongle_base_info().unwrap();
  let before = stored(&configurator);
  let original = before.sensor_toggles();
  let flips: [fn(&mut SensorToggles); 5] = [
    |toggles| toggles.wave = !toggles.wave,
    |toggles| toggles.line = !toggles.line,
    |toggles| toggles.motion = !toggles.motion,
    |toggles| toggles.scroll = !toggles.scroll,
    |toggles| toggles.e_sports = !toggles.e_sports,
  ];
  let mut read_back = Vec::new();
  for flip in flips {
    let mut flipped = original;
    flip(&mut flipped);
    configurator
      .write_sensor_settings(before.lift_off, &flipped)
      .unwrap();
    read_back.push((flipped, stored(&configurator)));
  }
  configurator
    .write_sensor_settings(before.lift_off, &original)
    .unwrap();
  let restored = stored(&configurator);

  for (flipped, after) in read_back {
    assert_eq!(after.sensor_toggles(), flipped);
    assert_eq!(after.lift_off, before.lift_off);
  }
  assert_eq!(restored.sensor_toggles(), original);
}

#[test]
fn lift_off_keeps_the_stored_switches() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let before = configurator.read_dongle_base_info().unwrap();
  let other = if before.lift_off == 1 { 2 } else { 1 };
  let mut switched = before.sensor_toggles();
  switched.line = !switched.line;
  configurator
    .write_sensor_settings(before.lift_off, &switched)
    .unwrap();
  configurator.write_lift_off(other).unwrap();
  let after = configurator.read_dongle_base_info().unwrap();
  configurator
    .write_sensor_settings(before.lift_off, &before.sensor_toggles())
    .unwrap();
  let restored = configurator.read_dongle_base_info().unwrap();

  assert_eq!(after.lift_off, other);
  assert_eq!(after.sensor_toggles(), switched);
  assert_eq!(restored.lift_off, before.lift_off);
  assert_eq!(restored.sensor_toggles(), before.sensor_toggles());
}

#[test]
fn the_receiver_refuses_the_sleep_timer() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  assert!(configurator.write_sleep_timer(5).is_err());
}

#[test]
fn changes_the_polling_rate_and_restores_it() {
  let _turn = hardware.lock().unwrap_or_else(|poison| poison.into_inner());
  let Some(configurator) = receiver() else {
    return;
  };
  let rate =
    |configurator: &MouseConfigurator| configurator.read_dongle_base_info().unwrap().report_rate;
  let original = rate(&configurator);
  let original_hertz =
    ReportRatePacket::code_to_rate(original).expect("stored rate outside the profile");
  let other_hertz = if original_hertz == 1000 { 125 } else { 1000 };
  configurator.write_report_rate(other_hertz).unwrap();
  let changed = rate(&configurator);
  configurator.write_report_rate(original_hertz).unwrap();
  let restored = rate(&configurator);

  assert_eq!(ReportRatePacket::code_to_rate(changed), Some(other_hertz));
  assert_eq!(restored, original);
}
