use clap::Parser;
use darmoshark_cli::arguments::cli_arguments::CliArguments;
use darmoshark_cli::arguments::command::Command;

fn parse(arguments: &[&str]) -> Result<Command, clap::Error> {
  CliArguments::try_parse_from(std::iter::once("dms").chain(arguments.iter().copied()))
    .map(|parsed| parsed.command)
}

#[test]
fn dpi_takes_levels_and_an_active_index() {
  assert_eq!(
    parse(&["dpi", "400", "800", "--active", "1"]).unwrap(),
    Command::Dpi {
      values: vec![400, 800],
      active: 1
    }
  );
}

#[test]
fn dpi_activates_the_first_level_by_default() {
  assert_eq!(
    parse(&["dpi", "1600"]).unwrap(),
    Command::Dpi {
      values: vec![1600],
      active: 0
    }
  );
}

#[test]
fn dpi_needs_at_least_one_level() {
  assert!(parse(&["dpi"]).is_err());
}

#[test]
fn rate_takes_one_value_per_level() {
  assert_eq!(
    parse(&["rate", "1000", "500"]).unwrap(),
    Command::Rate {
      values: vec![1000, 500],
      active: 0
    }
  );
}

#[test]
fn lift_off_accepts_only_low_and_high() {
  assert_eq!(parse(&["lod", "2"]).unwrap(), Command::Lod { value: 2 });
  assert!(parse(&["lod", "3"]).is_err());
  assert!(parse(&["lod", "0"]).is_err());
}

#[test]
fn button_takes_a_known_kind_and_optional_bytes() {
  assert_eq!(
    parse(&["button", "4", "macro", "7", "7"]).unwrap(),
    Command::Button {
      index: 4,
      kind: "macro".into(),
      data: vec![7, 7]
    }
  );
  assert!(parse(&["button", "0", "teleport"]).is_err());
}

#[test]
fn negative_values_are_rejected_before_any_device_opens() {
  assert!(parse(&["debounce", "-1"]).is_err());
  assert!(parse(&["use", "-1"]).is_err());
}

#[test]
fn every_python_command_exists() {
  for name in [
    "list",
    "capabilities",
    "colors",
    "battery",
    "dfu",
    "info",
    "bond",
    "buttons",
    "reset",
  ] {
    assert!(parse(&[name]).is_ok(), "{name}");
  }
  for (name, value) in [
    ("use", "1"),
    ("debounce", "8"),
    ("sleep", "10"),
    ("profile", "1"),
  ] {
    assert!(parse(&[name, value]).is_ok(), "{name}");
  }
}
