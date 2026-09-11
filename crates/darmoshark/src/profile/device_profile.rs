use serde_json::Value;

use crate::error::darmoshark_error::{DarmosharkError, DarmosharkResult};

/// Device capability profile, as published by the vendor for this model.
///
/// Wraps the official v3.json definition of the mouse, embedded at build time
/// from `reference/`. It declares what the hardware can actually do -- DPI
/// range, selectable polling rates, lift-off steps and whether the model has
/// controllable lighting. The M3 ships with `"light": null`, meaning no
/// configurable RGB.
pub struct DeviceProfile {
  data: Value,
}

impl DeviceProfile {
  pub fn load() -> DarmosharkResult<Self> {
    Self::parse(include_str!(
      "../../../../reference/darmoshark-m3-profile.json"
    ))
  }

  pub fn parse(text: &str) -> DarmosharkResult<Self> {
    serde_json::from_str(text)
      .map(|data| Self { data })
      .map_err(|error| {
        DarmosharkError::Profile(format!("device profile is not valid JSON: {error}"))
      })
  }

  pub fn name(&self) -> &str {
    self.data["name"].as_str().unwrap_or("unknown")
  }

  /// True only when the vendor declares a light block for this model.
  pub fn has_configurable_lighting(&self) -> bool {
    match &self.data["light"] {
      Value::Null => false,
      Value::Bool(flag) => *flag,
      Value::Object(block) => !block.is_empty(),
      Value::Array(block) => !block.is_empty(),
      Value::String(text) => !text.is_empty(),
      Value::Number(number) => number.as_f64() != Some(0.0),
    }
  }

  pub fn dpi_levels(&self) -> Vec<u32> {
    Self::items(&self.data["dpi"]["level"])
      .filter_map(|value| value.as_u64().map(|level| level as u32))
      .collect()
  }

  pub fn dpi_range(&self) -> Option<(u32, u32)> {
    match Self::items(&self.data["dpi"]["limit"]).collect::<Vec<_>>()[..] {
      [low, high] => Some((low.as_u64()? as u32, high.as_u64()? as u32)),
      _ => None,
    }
  }

  /// Indicator colour per DPI level, in level order.
  pub fn dpi_colors(&self) -> Vec<String> {
    Self::items(&self.data["dpi"]["colors"])
      .filter_map(|value| value.as_str().map(str::to_string))
      .collect()
  }

  /// (hertz, colour) for every selectable polling rate.
  pub fn report_rates(&self) -> Vec<(Option<u32>, Option<String>)> {
    Self::items(&self.data["dpi"]["reportRate"])
      .map(|entry| {
        (
          entry["value"].as_u64().map(|hertz| hertz as u32),
          entry["color"].as_str().map(str::to_string),
        )
      })
      .collect()
  }

  /// (index, label key) for every lift-off distance step.
  pub fn lift_off_steps(&self) -> Vec<(Option<u8>, Option<String>)> {
    Self::items(&self.data["sys"]["lod"])
      .map(|entry| {
        (
          entry["index"].as_u64().map(|index| index as u8),
          entry["value"].as_str().map(str::to_string),
        )
      })
      .collect()
  }

  pub fn button_count(&self) -> usize {
    Self::items(&self.data["keys"]).count()
  }

  fn items(value: &Value) -> impl Iterator<Item = &Value> {
    value.as_array().into_iter().flatten()
  }
}
