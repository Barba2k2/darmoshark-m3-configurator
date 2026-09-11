use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use darmoshark::profile::device_profile::DeviceProfile;

use crate::commands::offline_commands::OfflineCommands;

/// Commands that change the mouse and confirm what they sent.
pub struct WriteCommands;

impl WriteCommands {
  pub fn reset(configurator: &MouseConfigurator) -> DarmosharkResult<String> {
    configurator.restore_factory_defaults()?;
    Ok("factory defaults restored".into())
  }

  pub fn dpi(
    configurator: &MouseConfigurator,
    values: &[u32],
    active: usize,
  ) -> DarmosharkResult<String> {
    configurator.write_dpi_levels(values, active, None)?;
    Ok(format!(
      "programmed {} level(s): {}; active index {active}",
      values.len(),
      OfflineCommands::join(values)
    ))
  }

  /// Switches the active level, keeping the stored levels when they can be
  /// read back. The cable cannot read them, so there the levels are rewritten
  /// from the vendor profile defaults, and the output says so. A receiver
  /// failure is reported instead: rewriting defaults there would overwrite
  /// levels that were only temporarily unreadable.
  pub fn use_level(configurator: &MouseConfigurator, level: usize) -> DarmosharkResult<String> {
    match configurator.select_dpi_level(level) {
      Ok(info) => {
        return Ok(format!(
          "switched to level {level} ({} DPI)",
          info.dpi_levels()[level]
        ));
      }
      Err(error) if configurator.device().uses_dongle_transport() => return Err(error),
      Err(_) => {}
    }

    let levels = DeviceProfile::load()?.dpi_levels();
    if level >= levels.len() {
      return Err(DarmosharkError::Invalid(format!(
        "level {level} out of range, profile has {}",
        levels.len()
      )));
    }
    configurator.write_dpi_levels(&levels, level, None)?;
    Ok(format!(
      "switched to level {level} ({} DPI)\nnote: levels could not be read back over the \
       cable, so they were rewritten with the profile defaults ({}).",
      levels[level],
      OfflineCommands::join(&levels)
    ))
  }

  pub fn rates(
    configurator: &MouseConfigurator,
    values: &[u32],
    active: usize,
  ) -> DarmosharkResult<String> {
    configurator.write_report_rates(values, active, None)?;
    Ok(format!(
      "polling rates set to {} Hz",
      OfflineCommands::join(values)
    ))
  }

  pub fn debounce(configurator: &MouseConfigurator, milliseconds: u32) -> DarmosharkResult<String> {
    configurator.write_debounce(milliseconds)?;
    Ok(format!("debounce set to {milliseconds} ms"))
  }

  pub fn lift_off(configurator: &MouseConfigurator, value: u8) -> DarmosharkResult<String> {
    configurator.write_lift_off(value)?;
    if configurator.device().uses_dongle_transport() {
      return Ok(format!("lift-off distance set to {value}"));
    }
    Ok(format!(
      "lift-off distance set to {value}\nnote: the sensor switches could not be read \
       back over the cable, so they were reset to the vendor defaults (wave and motion \
       on, line, scroll and eSports off)."
    ))
  }

  pub fn sleep(configurator: &MouseConfigurator, minutes: u32) -> DarmosharkResult<String> {
    configurator.write_sleep_timer(minutes)?;
    Ok(format!("sleep timer set to {minutes} min"))
  }

  pub fn profile(configurator: &MouseConfigurator, index: u8) -> DarmosharkResult<String> {
    configurator.switch_profile(index)?;
    Ok(format!("switched to profile {index}"))
  }

  pub fn button(
    configurator: &MouseConfigurator,
    index: u8,
    kind: &str,
    data: &[u32],
  ) -> DarmosharkResult<String> {
    configurator.write_button(index, kind, data)?;
    let detail = if data.is_empty() {
      String::new()
    } else {
      format!(" with data {data:?}")
    };
    Ok(format!("button {index} assigned to {kind}{detail}"))
  }
}
