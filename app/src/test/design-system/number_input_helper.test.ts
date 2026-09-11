import { describe, expect, it } from "vitest";

import { NumberInputHelper } from "../../design-system/helpers/number_input_helper";

describe("NumberInputHelper.clamp", () => {
  it("keeps values inside the range", () => {
    expect(NumberInputHelper.clamp("800", 50, 26000, 400)).toBe(800);
  });

  it("clamps to the range edges", () => {
    expect(NumberInputHelper.clamp("49", 50, 26000, 400)).toBe(50);
    expect(NumberInputHelper.clamp("30000", 50, 26000, 400)).toBe(26000);
  });

  it("falls back when the text is not a number", () => {
    expect(NumberInputHelper.clamp("", 50, 26000, 400)).toBe(400);
  });
});
