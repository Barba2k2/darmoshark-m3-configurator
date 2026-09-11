/** Mirrors `ProfileDto`: what the model supports, offline. */
export interface Profile {
  name: string;
  dpiLevels: number[];
  dpiColors: string[];
  dpiMinimum: number;
  dpiMaximum: number;
  reportRates: number[];
  liftOffSteps: number[];
  debounceMinimum: number;
  debounceMaximum: number;
  sleepMaximumMinutes: number;
}

/** Mirrors `Transport`: which interface answered. */
export type Transport = "receiver" | "cable" | "other";

/** Mirrors `SnapshotDto`: the stored configuration, receiver only. */
export interface Snapshot {
  dpiLevels: number[];
  activeLevel: number;
  reportRateHz: number | null;
  debounceMs: number;
  liftOff: number;
  sleepMinutes: number;
}

/** Mirrors `DeviceStateDto`. */
export interface DeviceState {
  transport: Transport;
  firmwareVersion: string;
  batteryPercent: number;
  snapshot: Snapshot | null;
}
