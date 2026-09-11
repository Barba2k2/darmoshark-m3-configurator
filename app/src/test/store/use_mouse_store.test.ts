import { beforeEach, describe, expect, it, vi } from "vitest";

import { MouseService } from "../../services/mouse_service";
import { useMouseStore } from "../../store/use_mouse_store";
import { DeviceFixtures } from "../fixtures/device_fixtures";

vi.mock("../../services/mouse_service");

const initialState = useMouseStore.getState();

describe("useMouseStore", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    useMouseStore.setState(initialState, true);
    vi.mocked(MouseService.readProfile).mockResolvedValue(DeviceFixtures.profile);
    vi.mocked(MouseService.readDeviceState).mockResolvedValue(DeviceFixtures.receiver);
  });

  it("loads the stored settings over the receiver", async () => {
    await useMouseStore.getState().load();
    const state = useMouseStore.getState();
    expect(state.connection).toBe("connected");
    expect(state.levels).toEqual([400, 800, 1600, 3200, 6400]);
    expect(state.activeLevel).toBe(3);
    expect(state.reportRate).toBe(500);
    expect(state.sleepMinutes).toBe(0);
  });

  it("keeps the profile defaults over the cable, which cannot read them", async () => {
    vi.mocked(MouseService.readDeviceState).mockResolvedValue(DeviceFixtures.cable);
    await useMouseStore.getState().load();
    const state = useMouseStore.getState();
    expect(state.connection).toBe("connected");
    expect(state.levels).toEqual(DeviceFixtures.profile.dpiLevels);
    expect(state.reportRate).toBe(1000);
  });

  it("reports a missing mouse", async () => {
    vi.mocked(MouseService.readDeviceState).mockResolvedValue(null);
    await useMouseStore.getState().load();
    expect(useMouseStore.getState().connection).toBe("disconnected");
    expect(useMouseStore.getState().status).toBe("noMouse");
  });

  it("selecting a level writes every level with the new active index", async () => {
    await useMouseStore.getState().load();
    await useMouseStore.getState().selectLevel(1);
    expect(MouseService.writeDpiLevels).toHaveBeenCalledWith([400, 800, 1600, 3200, 6400], 1);
    expect(useMouseStore.getState().status).toBe("applied");
    expect(useMouseStore.getState().busy).toBe(false);
  });

  it("changing the polling rate sends only the frequency and shows what the mouse reports", async () => {
    await useMouseStore.getState().load();
    const snapshot = { ...DeviceFixtures.receiverSnapshot, reportRateHz: 1000 };
    vi.mocked(MouseService.readDeviceState).mockResolvedValue({ ...DeviceFixtures.receiver, snapshot });
    await useMouseStore.getState().applyReportRate(1000);
    expect(MouseService.writeReportRate).toHaveBeenCalledWith(1000);
    expect(useMouseStore.getState().reportRate).toBe(1000);
  });

  it("shows the transport error when a write fails", async () => {
    await useMouseStore.getState().load();
    vi.mocked(MouseService.writeSleepTimer).mockRejectedValue("the receiver does not relay the sleep timer");
    await useMouseStore.getState().applySleep(5);
    const state = useMouseStore.getState();
    expect(state.status).toBe("error");
    expect(state.errorDetail).toBe("the receiver does not relay the sleep timer");
    expect(state.busy).toBe(false);
  });

  it("a failed read keeps the mouse connected and shows why", async () => {
    await useMouseStore.getState().load();
    vi.mocked(MouseService.readDeviceState).mockRejectedValue("the mouse fell asleep");
    await useMouseStore.getState().refreshDevice(false);
    const state = useMouseStore.getState();
    expect(state.connection).toBe("connected");
    expect(state.device).toEqual(DeviceFixtures.receiver);
    expect(state.status).toBe("error");
    expect(state.errorDetail).toBe("the mouse fell asleep");
  });

  it("a background refresh leaves fields being edited alone", async () => {
    await useMouseStore.getState().load();
    useMouseStore.getState().editLevel(0, 1200);
    await useMouseStore.getState().refreshDevice(false);
    expect(useMouseStore.getState().levels[0]).toBe(1200);
  });

  it("a confirmed reset closes the sheet and restores the defaults", async () => {
    await useMouseStore.getState().load();
    useMouseStore.getState().openResetSheet();
    await useMouseStore.getState().confirmReset();
    expect(useMouseStore.getState().resetSheetOpen).toBe(false);
    expect(MouseService.restoreFactoryDefaults).toHaveBeenCalledOnce();
  });
});
