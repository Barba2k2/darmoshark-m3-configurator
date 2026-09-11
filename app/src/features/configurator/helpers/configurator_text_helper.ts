import { Labels } from "../../../labels/labels";
import type { DeviceState, Transport } from "../../../models/device_models";
import type { Connection, StatusKind } from "../../../store/use_mouse_store";

/** Turns store state into the text and tone the configurator shows. */
export class ConfiguratorTextHelper {
  private constructor() {}

  static transport(transport: Transport): string {
    switch (transport) {
      case "receiver":
        return Labels.transportReceiver;
      case "cable":
        return Labels.transportCable;
      case "other":
        return Labels.transportOther;
    }
  }

  static subtitle(connection: Connection, device: DeviceState | null): string {
    if (connection === "loading") {
      return Labels.subtitleLoading;
    }
    if (device === null) {
      return Labels.subtitleDisconnected;
    }
    return `firmware ${device.firmwareVersion} · ${ConfiguratorTextHelper.transport(device.transport)}`;
  }

  static battery(device: DeviceState | null): string {
    return device === null ? Labels.missingValue : `${device.batteryPercent}${Labels.unitPercent}`;
  }

  static status(kind: StatusKind, detail: string): string {
    switch (kind) {
      case "ready":
        return Labels.statusReady;
      case "working":
        return Labels.statusWorking;
      case "applied":
        return Labels.statusApplied;
      case "noMouse":
        return Labels.statusNoMouse;
      case "error":
        return detail;
    }
  }

  static tone(kind: StatusKind): "neutral" | "success" | "error" {
    if (kind === "applied") {
      return "success";
    }
    return kind === "error" ? "error" : "neutral";
  }

  static liftOff(step: number): string {
    if (step === 1) {
      return Labels.liftOffLow;
    }
    return step === 2 ? Labels.liftOffHigh : String(step);
  }
}
