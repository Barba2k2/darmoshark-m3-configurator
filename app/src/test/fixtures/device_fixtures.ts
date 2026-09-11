import type { DeviceState, Profile, Snapshot } from "../../models/device_models";

/** Shared inputs: the M3 vendor profile and states seen on hardware. */
export class DeviceFixtures {
  private constructor() {}

  static readonly profile: Profile = {
    name: "Darmoshark M3",
    dpiLevels: [400, 800, 1600, 3200, 4800],
    dpiColors: ["#ff0000", "#0060ff", "#12ff00", "#e218ff", "#f9e14c", "#e218ff"],
    dpiMinimum: 50,
    dpiMaximum: 26000,
    reportRates: [125, 500, 1000],
    liftOffSteps: [1, 2],
    debounceMinimum: 0,
    debounceMaximum: 20,
    sleepMaximumMinutes: 255,
  };

  static readonly receiverSnapshot: Snapshot = {
    dpiLevels: [400, 800, 1600, 3200, 6400],
    activeLevel: 3,
    reportRateHz: 500,
    debounceMs: 8,
    liftOff: 1,
    sleepMinutes: 0,
  };

  static readonly receiver: DeviceState = {
    transport: "receiver",
    firmwareVersion: "2.0.9",
    batteryPercent: 25,
    snapshot: DeviceFixtures.receiverSnapshot,
  };

  static readonly cable: DeviceState = {
    transport: "cable",
    firmwareVersion: "2.0.9",
    batteryPercent: 90,
    snapshot: null,
  };
}
