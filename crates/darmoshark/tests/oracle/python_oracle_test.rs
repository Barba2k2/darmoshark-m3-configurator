//! Byte-for-byte parity with the Python implementation this crate replaces.
//!
//! `fixtures/python_oracle.json` was recorded by running the Python builders
//! and decoders over the same inputs. The Python frames are themselves checked
//! against the vendor bundle, so matching them keeps the Rust frames identical
//! to what the official configurator sends.

use darmoshark::packets::button_packet::ButtonPacket;
use darmoshark::packets::dpi_packet::DpiPacket;
use darmoshark::packets::packet::Packet;
use darmoshark::packets::profile_packet::ProfilePacket;
use darmoshark::packets::report_rate_packet::ReportRatePacket;
use darmoshark::packets::sleep_mode::SleepMode;
use darmoshark::packets::tuning_packet::TuningPacket;
use darmoshark::replies::base_info::BaseInfo;
use darmoshark::replies::cable_info::CableInfo;
use darmoshark::replies::dfu_info::DfuInfo;
use darmoshark::replies::dongle_base_info::DongleBaseInfo;
use serde_json::{Value, json};

fn oracle() -> Value {
  serde_json::from_str(include_str!("fixtures/python_oracle.json")).unwrap()
}

fn numbers(value: &Value) -> Vec<u32> {
  value
    .as_array()
    .unwrap()
    .iter()
    .map(|item| item.as_u64().unwrap() as u32)
    .collect()
}

fn number(value: &Value) -> u32 {
  value.as_u64().unwrap() as u32
}

fn optional(value: &Value) -> Option<usize> {
  value.as_u64().map(|number| number as usize)
}

fn bytes(value: &Value) -> Vec<u8> {
  let text = value.as_str().unwrap();
  (0..text.len())
    .step_by(2)
    .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
    .collect()
}

fn build(builder: &str, args: &[Value]) -> Packet {
  let byte = |index: usize| number(&args[index]) as u8;
  match builder {
    "dpi" => DpiPacket::build(
      &numbers(&args[0]),
      number(&args[1]) as usize,
      optional(&args[2]),
    ),
    "rate" => ReportRatePacket::build(number(&args[0])),
    "debounce" => TuningPacket::build_debounce(number(&args[0])),
    "sensor" => TuningPacket::build_sensor(byte(0), byte(1), byte(2), byte(3), byte(4), byte(5)),
    "scroll" => Ok(TuningPacket::build_scroll(byte(0), byte(1), byte(2))),
    "sleep" => {
      let mode = match args[1].as_str().unwrap() {
        "set" => SleepMode::Set,
        _ => SleepMode::Get,
      };
      TuningPacket::build_sleep(number(&args[0]), mode)
    }
    "profileSwitch" => ProfilePacket::build_switch(byte(0)),
    "recovery" => Ok(ProfilePacket::build_recovery(byte(0), byte(1), byte(2))),
    "factoryReset" => Ok(ProfilePacket::build_factory_reset()),
    "buttonRead" => ButtonPacket::build_read(byte(0)),
    "buttonWrite" => {
      ButtonPacket::build_write(byte(0), args[1].as_str().unwrap(), &numbers(&args[2]))
    }
    other => panic!("oracle names an unknown builder {other}"),
  }
  .unwrap()
}

#[test]
fn every_frame_matches_the_python_builder() {
  let oracle = oracle();
  let frames = oracle["frames"].as_array().unwrap();
  assert!(!frames.is_empty());
  for frame in frames {
    let builder = frame["builder"].as_str().unwrap();
    let packet = build(builder, frame["args"].as_array().unwrap());
    let expected = Packet::new(number(&frame["reportId"]) as u8, bytes(&frame["payload"]));
    assert_eq!(packet, expected, "{builder} {}", frame["args"]);
  }
}

#[test]
fn every_decoder_matches_the_python_decoder() {
  let oracle = oracle();
  for case in oracle["decoders"].as_array().unwrap() {
    let decoder = case["decoder"].as_str().unwrap();
    let decoded = match decoder {
      "dongleBaseInfo" => {
        let info = DongleBaseInfo::parse(&bytes(&case["input"])).unwrap();
        json!({
          "profile": info.profile, "dpiLevels": info.dpi_levels,
          "activeLevel": info.active_level, "reportRate": info.report_rate,
          "debounceMs": info.debounce_ms, "sleepMinutes": info.sleep_minutes,
          "liftOff": info.lift_off, "wave": info.wave, "line": info.line,
          "motion": info.motion, "scroll": info.scroll, "eSports": info.e_sports,
        })
      }
      "baseInfo" => {
        let info = BaseInfo::parse(&bytes(&case["input"])).unwrap();
        json!({
          "profile": info.profile, "dpiLevels": info.dpi_levels,
          "activeLevel": info.active_level, "reportRate": info.report_rate,
          "batteryPercent": info.battery_percent,
          "batteryCharging": info.battery_charging,
          "sleepMinutes": info.sleep_minutes,
        })
      }
      "cableInfo" => {
        let info = CableInfo::parse(&bytes(&case["input"])).unwrap();
        json!({
          "vendorId": info.vendor_id, "productId": info.product_id,
          "firmwareVersion": info.firmware_version,
          "batteryPercent": info.battery_percent,
        })
      }
      "dfuInfo" => {
        let packets: Vec<Vec<u8>> = case["input"]
          .as_array()
          .unwrap()
          .iter()
          .map(bytes)
          .collect();
        let info = DfuInfo::parse(&packets).unwrap();
        let raw: String = info.raw.iter().map(|byte| format!("{byte:02x}")).collect();
        json!({
          "moduleModel": info.module_model, "firmwareVersion": info.firmware_version,
          "hardwareVersion": info.hardware_version, "raw": raw,
        })
      }
      other => panic!("oracle names an unknown decoder {other}"),
    };
    assert_eq!(decoded, case["fields"], "{decoder}");
  }
}
