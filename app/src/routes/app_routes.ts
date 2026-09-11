/** Every Tauri command the window invokes. Names match the Rust handlers. */
export class AppRoutes {
  static readonly readProfile = "read_profile";
  static readonly readDeviceState = "read_device_state";
  static readonly writeDpiLevels = "write_dpi_levels";
  static readonly writeReportRate = "write_report_rate";
  static readonly writeLiftOff = "write_lift_off";
  static readonly writeDebounce = "write_debounce";
  static readonly writeSleepTimer = "write_sleep_timer";
  static readonly restoreFactoryDefaults = "restore_factory_defaults";
}
