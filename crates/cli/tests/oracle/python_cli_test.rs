//! Output parity with the Python CLI this binary replaces.
//!
//! `fixtures/python_cli.json` holds what `src/cli.py` printed for the offline
//! commands and for `info` over fixed snapshots. The binary adds the trailing
//! newline when it prints, as `print` did.

use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::replies::base_info::BaseInfo;
use darmoshark::replies::base_snapshot::BaseSnapshot;
use darmoshark::replies::dongle_base_info::DongleBaseInfo;
use darmoshark_cli::commands::offline_commands::OfflineCommands;
use darmoshark_cli::commands::read_commands::ReadCommands;
use serde_json::Value;

fn oracle() -> Value {
  serde_json::from_str(include_str!("fixtures/python_cli.json")).unwrap()
}

fn printed(text: String) -> String {
  format!("{text}\n")
}

fn bytes(value: &Value) -> Vec<u8> {
  let text = value.as_str().unwrap();
  (0..text.len())
    .step_by(2)
    .map(|index| u8::from_str_radix(&text[index..index + 2], 16).unwrap())
    .collect()
}

#[test]
fn capabilities_match_the_python_cli() {
  let profile = DeviceProfile::load().unwrap();
  assert_eq!(
    printed(OfflineCommands::capabilities(&profile)),
    oracle()["capabilities"]
  );
}

#[test]
fn color_legend_matches_the_python_cli() {
  let profile = DeviceProfile::load().unwrap();
  assert_eq!(
    printed(OfflineCommands::color_legend(&profile)),
    oracle()["colors"]
  );
}

#[test]
fn snapshots_print_as_the_python_cli() {
  for case in oracle()["snapshots"].as_array().unwrap() {
    let input = bytes(&case["input"]);
    let snapshot = match case["kind"].as_str().unwrap() {
      "dongle" => BaseSnapshot::Dongle(DongleBaseInfo::parse(&input).unwrap()),
      _ => BaseSnapshot::Config(BaseInfo::parse(&input).unwrap()),
    };
    assert_eq!(
      printed(ReadCommands::describe_snapshot(&snapshot)),
      case["output"],
      "{}",
      case["input"]
    );
  }
}
