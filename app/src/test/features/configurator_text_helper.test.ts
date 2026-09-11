import { describe, expect, it } from "vitest";

import { ConfiguratorTextHelper } from "../../features/configurator/helpers/configurator_text_helper";
import { Labels } from "../../labels/labels";
import { DeviceFixtures } from "../fixtures/device_fixtures";

describe("ConfiguratorTextHelper", () => {
  it("names the firmware and the transport", () => {
    expect(ConfiguratorTextHelper.subtitle("connected", DeviceFixtures.receiver)).toBe(
      `firmware 2.0.9 · ${Labels.transportReceiver}`,
    );
  });

  it("says when no mouse is connected", () => {
    expect(ConfiguratorTextHelper.subtitle("disconnected", null)).toBe(Labels.subtitleDisconnected);
  });

  it("shows transport errors verbatim", () => {
    expect(ConfiguratorTextHelper.status("error", "link down")).toBe("link down");
    expect(ConfiguratorTextHelper.tone("error")).toBe("error");
    expect(ConfiguratorTextHelper.tone("applied")).toBe("success");
  });
});
