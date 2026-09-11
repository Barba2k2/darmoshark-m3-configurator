import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { BatteryGauge } from "../../design-system/display/BatteryGauge";
import { DpiLevelEditor } from "../../design-system/inputs/DpiLevelEditor";
import { NumberInput } from "../../design-system/inputs/NumberInput";
import { BottomSheet } from "../../design-system/overlays/BottomSheet";

describe("BatteryGauge", () => {
  it("turns low at 20 percent", () => {
    render(<BatteryGauge caption="battery" percent={20} valueLabel="20%" />);
    const meter = screen.getByRole("meter", { name: "battery" });
    expect(meter).toHaveAttribute("aria-valuenow", "20");
    expect(meter.firstElementChild?.className).toContain("low");
  });

  it("stays normal above 20 percent", () => {
    render(<BatteryGauge caption="battery" percent={21} valueLabel="21%" />);
    expect(screen.getByRole("meter").firstElementChild?.className).not.toContain("low");
  });
});

describe("NumberInput", () => {
  it("commits the clamped value when editing ends", () => {
    const onCommit = vi.fn();
    render(<NumberInput label="debounce" value={8} minimum={0} maximum={20} onCommit={onCommit} />);
    const input = screen.getByRole("spinbutton", { name: "debounce" });
    fireEvent.change(input, { target: { value: "99" } });
    fireEvent.blur(input);
    expect(onCommit).toHaveBeenCalledWith(20);
  });

  it("does not commit an unchanged value", () => {
    const onCommit = vi.fn();
    render(<NumberInput label="debounce" value={8} minimum={0} maximum={20} onCommit={onCommit} />);
    fireEvent.blur(screen.getByRole("spinbutton"));
    expect(onCommit).not.toHaveBeenCalled();
  });
});

describe("DpiLevelEditor", () => {
  it("marks the selected level and reports presses", () => {
    const onSelect = vi.fn();
    render(
      <DpiLevelEditor
        label="level 1"
        color="#ff0000"
        value={400}
        minimum={50}
        maximum={26000}
        selected
        onSelect={onSelect}
        onCommit={vi.fn()}
      />,
    );
    const button = screen.getByRole("button", { name: "level 1" });
    expect(button).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(button);
    expect(onSelect).toHaveBeenCalledOnce();
  });
});

describe("BottomSheet", () => {
  const sheet = (open: boolean, onConfirm = vi.fn(), onCancel = vi.fn()) =>
    render(
      <BottomSheet
        open={open}
        title="reset"
        body="sure?"
        confirmLabel="yes"
        cancelLabel="no"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    );

  it("renders nothing while closed", () => {
    sheet(false);
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("confirms and cancels through its buttons", () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    sheet(true, onConfirm, onCancel);
    fireEvent.click(screen.getByRole("button", { name: "yes" }));
    fireEvent.click(screen.getByRole("button", { name: "no" }));
    expect(onConfirm).toHaveBeenCalledOnce();
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("cancels when the dimmed backdrop is clicked, not the sheet", () => {
    const onCancel = vi.fn();
    sheet(true, vi.fn(), onCancel);
    fireEvent.click(screen.getByRole("dialog"));
    expect(onCancel).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("dialog").parentElement as HTMLElement);
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
