import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { SettingsSection } from "../../features/configurator/SettingsSection";
import { Labels } from "../../labels/labels";
import { useMouseStore } from "../../store/use_mouse_store";
import { DeviceFixtures } from "../fixtures/device_fixtures";

const initialState = useMouseStore.getState();

describe("SettingsSection", () => {
  beforeEach(() => {
    useMouseStore.setState({ ...initialState, profile: DeviceFixtures.profile }, true);
  });

  it("locks the sleep timer over the receiver, which does not relay it", () => {
    useMouseStore.setState({ device: DeviceFixtures.receiver, connection: "connected" });
    render(<SettingsSection />);
    expect(screen.getByRole("spinbutton", { name: Labels.sleepTimer })).toBeDisabled();
    expect(screen.getByText(Labels.sleepCableOnly)).toBeInTheDocument();
    expect(screen.getByRole("spinbutton", { name: Labels.debounce })).toBeEnabled();
  });

  it("offers the sleep timer over the cable", () => {
    useMouseStore.setState({ device: DeviceFixtures.cable, connection: "connected" });
    render(<SettingsSection />);
    expect(screen.getByRole("spinbutton", { name: Labels.sleepTimer })).toBeEnabled();
    expect(screen.queryByText(Labels.sleepCableOnly)).toBeNull();
  });
});
