/** Turns what was typed into a number the control accepts. */
export class NumberInputHelper {
  private constructor() {}

  /** Clamped whole number, or `fallback` when the text is not a number. */
  static clamp(text: string, minimum: number, maximum: number, fallback: number): number {
    const parsed = Number.parseInt(text, 10);
    if (Number.isNaN(parsed)) {
      return fallback;
    }
    return Math.min(maximum, Math.max(minimum, parsed));
  }
}
