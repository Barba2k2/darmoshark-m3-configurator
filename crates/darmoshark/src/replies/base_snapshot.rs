use crate::replies::base_info::BaseInfo;
use crate::replies::dongle_base_info::DongleBaseInfo;

/// The stored configuration, in whichever form the open transport returns it.
///
/// A config interface answers with the base-info block (battery included); the
/// receiver answers with its own snapshot (debounce and sensor bits included,
/// no battery). The accessors expose what both carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseSnapshot {
  Config(BaseInfo),
  Dongle(DongleBaseInfo),
}

impl BaseSnapshot {
  pub fn profile(&self) -> u8 {
    match self {
      Self::Config(info) => info.profile,
      Self::Dongle(info) => info.profile,
    }
  }

  pub fn dpi_levels(&self) -> &[u16] {
    match self {
      Self::Config(info) => &info.dpi_levels,
      Self::Dongle(info) => &info.dpi_levels,
    }
  }

  pub fn active_level(&self) -> u8 {
    match self {
      Self::Config(info) => info.active_level,
      Self::Dongle(info) => info.active_level,
    }
  }

  pub fn report_rate(&self) -> u8 {
    match self {
      Self::Config(info) => info.report_rate,
      Self::Dongle(info) => info.report_rate,
    }
  }

  pub fn sleep_minutes(&self) -> u8 {
    match self {
      Self::Config(info) => info.sleep_minutes,
      Self::Dongle(info) => info.sleep_minutes,
    }
  }
}
