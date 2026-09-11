use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use darmoshark::packets::report_rate_packet::ReportRatePacket;
use darmoshark::replies::base_snapshot::BaseSnapshot;

use crate::commands::offline_commands::OfflineCommands;

/// Commands that read from the open device and print what came back.
pub struct ReadCommands;

impl ReadCommands {
  pub fn cable_info(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    let info = configurator.read_cable_info()?;
    Ok(
      [
        format!(
          "device   : 0x{:04X}:0x{:04X}",
          info.vendor_id, info.product_id
        ),
        format!("firmware : {}", info.firmware_version),
        format!("battery  : {}%", info.battery_percent),
      ]
      .join("\n"),
    )
  }

  pub fn dfu_info(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    let info = configurator.read_dfu_info()?;
    let mut lines = Vec::new();
    if configurator.device().uses_dongle_transport() {
      lines.push(
        "note: this is the receiver's own bootloader, not the mouse's. Read it over \
         the cable to get the mouse."
          .to_string(),
      );
    }
    lines.push(format!("module   : {}", info.module_model));
    lines.push(format!("firmware : {}", info.firmware_version));
    lines.push(format!("hardware : {}", info.hardware_version));
    Ok(lines.join("\n"))
  }

  pub fn base_info(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    Ok(Self::describe_snapshot(&configurator.read_base_info()?))
  }

  pub fn describe_snapshot(snapshot: &BaseSnapshot) -> String {
    let levels = snapshot.dpi_levels();
    let active = snapshot.active_level();
    let active_dpi = levels
      .get(usize::from(active))
      .map_or_else(String::new, |dpi| format!(" ({dpi} DPI)"));
    let rate = ReportRatePacket::code_to_rate(snapshot.report_rate()).map_or_else(
      || format!("code {}", snapshot.report_rate()),
      |hertz| hertz.to_string(),
    );
    let mut lines = vec![
      format!("profile      : {}", snapshot.profile()),
      format!("dpi levels   : {}", OfflineCommands::join(levels)),
      format!("active level : {active}{active_dpi}"),
      format!("polling rate : {rate} Hz"),
    ];
    if let BaseSnapshot::Config(info) = snapshot {
      let charging = if info.battery_charging {
        " (charging)"
      } else {
        ""
      };
      lines.push(format!(
        "battery      : {}%{charging}",
        info.battery_percent
      ));
    }
    lines.push(format!("sleep        : {} min", snapshot.sleep_minutes()));
    if let BaseSnapshot::Dongle(info) = snapshot {
      lines.push(format!("debounce     : {} ms", info.debounce_ms));
      lines.push(format!("lift-off     : {}", info.lift_off));
    }
    lines.join("\n")
  }

  pub fn bond_info(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    let bond = configurator.read_bond_info()?;
    Ok(
      [
        format!(
          "linked mouse : 0x{:04X}:0x{:04X}",
          bond.vendor_id, bond.product_id
        ),
        format!("link state   : {}", if bond.linked { "up" } else { "down" }),
      ]
      .join("\n"),
    )
  }

  pub fn buttons(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    Ok(
      configurator
        .read_all_buttons()?
        .iter()
        .map(|entry| {
          format!(
            "button {}: {} (0x{:02X})",
            entry.button, entry.kind, entry.kind_value
          )
        })
        .collect::<Vec<_>>()
        .join("\n"),
    )
  }
}
