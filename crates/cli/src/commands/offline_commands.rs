use darmoshark::error::darmoshark_error::{DarmosharkError, DarmosharkResult};
use darmoshark::profile::device_profile::DeviceProfile;
use darmoshark::transport::darmoshark_device::DarmosharkDevice;

/// Commands that need no open device: enumeration and the vendor profile.
pub struct OfflineCommands;

impl OfflineCommands {
  pub const colorNames: [(&'static str, &'static str); 5] = [
    ("#ff0000", "red"),
    ("#0060ff", "blue"),
    ("#12ff00", "green"),
    ("#e218ff", "magenta"),
    ("#f9e14c", "yellow"),
  ];

  pub fn list_interfaces() -> DarmosharkResult<String> {
    let entries = DarmosharkDevice::discover()?;
    if entries.is_empty() {
      return Err(DarmosharkError::Device(
        "no Darmoshark interface found.".into(),
      ));
    }
    Ok(
      entries
        .iter()
        .map(|entry| {
          let product = entry
            .product_string()
            .map_or_else(|| "None".to_string(), |name| format!("'{name}'"));
          format!(
            "{}  vid=0x{:04X} pid=0x{:04X}  iface={}  usagePage=0x{:04X} usage=0x{:02X}  {product}",
            entry.path().to_string_lossy(),
            entry.vendor_id(),
            entry.product_id(),
            entry.interface_number(),
            entry.usage_page(),
            entry.usage(),
          )
        })
        .collect::<Vec<_>>()
        .join("\n"),
    )
  }

  pub fn capabilities(profile: &DeviceProfile) -> String {
    let range = profile.dpi_range().map_or_else(
      || "None-None".to_string(),
      |(low, high)| format!("{low}-{high}"),
    );
    let rates = profile
      .report_rates()
      .iter()
      .map(|(hertz, _)| Self::show(hertz))
      .collect::<Vec<_>>()
      .join(", ");
    let steps = profile
      .lift_off_steps()
      .iter()
      .map(|(index, name)| {
        let label = name
          .as_deref()
          .and_then(|name| name.rsplit('.').next())
          .unwrap_or("None");
        format!("{}={label}", Self::show(index))
      })
      .collect::<Vec<_>>()
      .join(", ");
    let lighting = if profile.has_configurable_lighting() {
      "yes"
    } else {
      "no (indicator LED only)"
    };
    [
      format!("model          : {}", profile.name()),
      format!("dpi range      : {range}"),
      format!("default levels : {}", Self::join(&profile.dpi_levels())),
      format!("polling rates  : {rates} Hz"),
      format!("lift-off steps : {steps}"),
      format!("buttons        : {}", profile.button_count()),
      format!("rgb lighting   : {lighting}"),
    ]
    .join("\n")
  }

  pub fn color_legend(profile: &DeviceProfile) -> String {
    let colors = profile.dpi_colors();
    let mut lines = vec![
      "The indicator LED encodes the active setting. Colours are fixed in".to_string(),
      "firmware -- they are not configurable.\n".to_string(),
      "DPI level:".to_string(),
    ];
    for (index, dpi) in profile.dpi_levels().iter().enumerate() {
      let color = colors.get(index).map_or("?", String::as_str);
      lines.push(format!(
        "  level {index}  {dpi:>5} DPI   {color}  {}",
        Self::color_name(color)
      ));
    }
    lines.push("\nPolling rate:".to_string());
    for (hertz, color) in profile.report_rates() {
      let color = color.as_deref().unwrap_or("None");
      lines.push(format!(
        "  {:>4} Hz          {color}  {}",
        Self::show(&hertz),
        Self::color_name(color)
      ));
    }
    lines.join("\n")
  }

  pub fn color_name(color: &str) -> &'static str {
    Self::colorNames
      .iter()
      .find(|(code, _)| *code == color)
      .map_or("", |(_, name)| name)
  }

  /// Comma-separated values, the way every listing in the CLI prints them.
  pub fn join<T: ToString>(values: &[T]) -> String {
    values
      .iter()
      .map(ToString::to_string)
      .collect::<Vec<_>>()
      .join(", ")
  }

  fn show<T: ToString>(value: &Option<T>) -> String {
    value
      .as_ref()
      .map_or_else(|| "None".to_string(), ToString::to_string)
  }
}
