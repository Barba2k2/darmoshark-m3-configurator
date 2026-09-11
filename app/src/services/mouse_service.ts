import { invoke } from "@tauri-apps/api/core";

import type { DeviceState, Profile } from "../models/device_models";
import { AppRoutes } from "../routes/app_routes";

/** Every hardware call the window makes, through the Tauri commands. */
export class MouseService {
  private constructor() {}

  static readProfile(): Promise<Profile> {
    return invoke<Profile>(AppRoutes.readProfile);
  }

  /** `null` when no mouse or receiver is plugged in. */
  static readDeviceState(): Promise<DeviceState | null> {
    return invoke<DeviceState | null>(AppRoutes.readDeviceState);
  }

  static writeDpiLevels(values: number[], activeLevel: number): Promise<void> {
    return invoke(AppRoutes.writeDpiLevels, { values, activeLevel });
  }

  static writeReportRate(hertz: number): Promise<void> {
    return invoke(AppRoutes.writeReportRate, { hertz });
  }

  static writeLiftOff(value: number): Promise<void> {
    return invoke(AppRoutes.writeLiftOff, { value });
  }

  static writeDebounce(milliseconds: number): Promise<void> {
    return invoke(AppRoutes.writeDebounce, { milliseconds });
  }

  static writeSleepTimer(minutes: number): Promise<void> {
    return invoke(AppRoutes.writeSleepTimer, { minutes });
  }

  static restoreFactoryDefaults(): Promise<void> {
    return invoke(AppRoutes.restoreFactoryDefaults);
  }
}
