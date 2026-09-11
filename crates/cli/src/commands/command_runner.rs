use darmoshark::configurator::mouse_configurator::MouseConfigurator;
use darmoshark::error::darmoshark_error::DarmosharkResult;
use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

use crate::arguments::command::Command;
use crate::commands::offline_commands::OfflineCommands;
use crate::commands::read_commands::ReadCommands;
use crate::commands::write_commands::WriteCommands;

/// Dispatches a parsed command, opening the device only when it needs one.
pub struct CommandRunner;

impl CommandRunner {
  pub fn run(command: &Command) -> DarmosharkResult<String> {
    let configurator = match command {
      Command::List => return OfflineCommands::list_interfaces(),
      Command::Capabilities => {
        return Ok(OfflineCommands::capabilities(&DeviceProfile::load()?));
      }
      Command::Colors => return Ok(OfflineCommands::color_legend(&DeviceProfile::load()?)),
      _ => MouseConfigurator::new(DarmosharkDevice::open()?),
    };

    match command {
      Command::Battery => ReadCommands::cable_info(&configurator),
      Command::Dfu => ReadCommands::dfu_info(&configurator),
      Command::Info => ReadCommands::base_info(&configurator),
      Command::Bond => ReadCommands::bond_info(&configurator),
      Command::Buttons => ReadCommands::buttons(&configurator),
      Command::Reset => WriteCommands::reset(&configurator),
      Command::Dpi { values, active } => WriteCommands::dpi(&configurator, values, *active),
      Command::Use { level } => WriteCommands::use_level(&configurator, *level),
      Command::Rate { values, active } => WriteCommands::rates(&configurator, values, *active),
      Command::Debounce { milliseconds } => WriteCommands::debounce(&configurator, *milliseconds),
      Command::Lod { value } => WriteCommands::lift_off(&configurator, *value),
      Command::Sleep { minutes } => WriteCommands::sleep(&configurator, *minutes),
      Command::Profile { index } => WriteCommands::profile(&configurator, *index),
      Command::Button { index, kind, data } => {
        WriteCommands::button(&configurator, *index, kind, data)
      }
      Command::List | Command::Capabilities | Command::Colors => {
        unreachable!("offline commands returned before the device was opened")
      }
    }
  }
}
