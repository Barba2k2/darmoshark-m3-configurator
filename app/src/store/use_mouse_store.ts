import { create } from "zustand";

import type { DeviceState, Profile } from "../models/device_models";
import { MouseService } from "../services/mouse_service";

/** What the status line reports; the view turns it into text. */
export type StatusKind = "ready" | "working" | "applied" | "noMouse" | "error";

export type Connection = "loading" | "connected" | "disconnected";

interface MouseState {
  profile: Profile | null;
  device: DeviceState | null;
  connection: Connection;
  levels: number[];
  activeLevel: number;
  reportRate: number | null;
  liftOff: number;
  debounceMs: number;
  sleepMinutes: number;
  status: StatusKind;
  errorDetail: string;
  busy: boolean;
  resetSheetOpen: boolean;
  load: () => Promise<void>;
  refreshDevice: (syncSettings: boolean) => Promise<void>;
  editLevel: (index: number, value: number) => void;
  selectLevel: (index: number) => Promise<void>;
  applyLevels: () => Promise<void>;
  applyReportRate: (hertz: number) => Promise<void>;
  applyLiftOff: (value: number) => Promise<void>;
  applyDebounce: (milliseconds: number) => Promise<void>;
  applySleep: (minutes: number) => Promise<void>;
  openResetSheet: () => void;
  closeResetSheet: () => void;
  confirmReset: () => Promise<void>;
}

/**
 * The window's single store. Settings start from the vendor profile and are
 * replaced by the stored values whenever the receiver can read them; the
 * cable cannot, so over it the fields keep the profile defaults.
 */
export const useMouseStore = create<MouseState>()((set, get) => {
  const describeError = (error: unknown) => (error instanceof Error ? error.message : String(error));

  const runWrite = async (write: () => Promise<void>) => {
    set({ busy: true, status: "working", errorDetail: "" });
    try {
      await write();
      set({ status: "applied" });
      await get().refreshDevice(true);
    } catch (error) {
      set({ status: "error", errorDetail: describeError(error) });
    } finally {
      set({ busy: false });
    }
  };

  return {
    profile: null,
    device: null,
    connection: "loading",
    levels: [],
    activeLevel: 0,
    reportRate: null,
    liftOff: 1,
    debounceMs: 8,
    sleepMinutes: 10,
    status: "ready",
    errorDetail: "",
    busy: false,
    resetSheetOpen: false,

    load: async () => {
      try {
        const profile = await MouseService.readProfile();
        set({
          profile,
          levels: [...profile.dpiLevels],
          reportRate: profile.reportRates.at(-1) ?? null,
          liftOff: profile.liftOffSteps[0] ?? 1,
        });
      } catch (error) {
        set({ status: "error", errorDetail: describeError(error) });
        return;
      }
      await get().refreshDevice(true);
    },

    refreshDevice: async (syncSettings) => {
      let device: DeviceState | null;
      try {
        device = await MouseService.readDeviceState();
      } catch (error) {
        set({ device: null, connection: "disconnected", status: "error", errorDetail: describeError(error) });
        return;
      }
      if (device === null) {
        set({ device: null, connection: "disconnected", status: "noMouse" });
        return;
      }
      set((state) => ({
        device,
        connection: "connected",
        status: state.status === "noMouse" ? "ready" : state.status,
      }));
      const snapshot = device.snapshot;
      if (syncSettings && snapshot !== null) {
        set((state) => ({
          levels: [...snapshot.dpiLevels],
          activeLevel: snapshot.activeLevel,
          reportRate: snapshot.reportRateHz ?? state.reportRate,
          liftOff: snapshot.liftOff,
          debounceMs: snapshot.debounceMs,
          sleepMinutes: snapshot.sleepMinutes,
        }));
      }
    },

    editLevel: (index, value) => {
      set((state) => ({ levels: state.levels.map((level, position) => (position === index ? value : level)) }));
    },

    selectLevel: async (index) => {
      set({ activeLevel: index });
      await runWrite(() => MouseService.writeDpiLevels(get().levels, index));
    },

    applyLevels: () => runWrite(() => MouseService.writeDpiLevels(get().levels, get().activeLevel)),

    applyReportRate: async (hertz) => {
      set({ reportRate: hertz });
      await runWrite(() => MouseService.writeReportRate(hertz));
    },

    applyLiftOff: async (value) => {
      set({ liftOff: value });
      await runWrite(() => MouseService.writeLiftOff(value));
    },

    applyDebounce: async (milliseconds) => {
      set({ debounceMs: milliseconds });
      await runWrite(() => MouseService.writeDebounce(milliseconds));
    },

    applySleep: async (minutes) => {
      set({ sleepMinutes: minutes });
      await runWrite(() => MouseService.writeSleepTimer(minutes));
    },

    openResetSheet: () => set({ resetSheetOpen: true }),

    closeResetSheet: () => set({ resetSheetOpen: false }),

    confirmReset: async () => {
      set({ resetSheetOpen: false });
      await runWrite(() => MouseService.restoreFactoryDefaults());
    },
  };
});
