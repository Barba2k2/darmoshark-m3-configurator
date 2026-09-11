import { NumberInputHelper } from "../helpers/number_input_helper";

interface NumberInputProps {
  label: string;
  className?: string;
  value: number;
  minimum: number;
  maximum: number;
  step?: number;
  disabled?: boolean;
  onCommit: (value: number) => void;
}

/**
 * A bare whole-number input that reports its value when editing ends (blur or
 * Enter), clamped to its range. It re-mounts whenever `value` changes, so a
 * value read back from the mouse always replaces what was typed.
 */
export function NumberInput({ label, className, value, minimum, maximum, step, disabled, onCommit }: NumberInputProps) {
  return (
    <input
      key={value}
      className={className}
      aria-label={label}
      type="number"
      inputMode="numeric"
      min={minimum}
      max={maximum}
      step={step}
      defaultValue={value}
      disabled={disabled}
      onKeyDown={(event) => event.key === "Enter" && event.currentTarget.blur()}
      onBlur={(event) => {
        const next = NumberInputHelper.clamp(event.currentTarget.value, minimum, maximum, value);
        event.currentTarget.value = String(next);
        if (next !== value) {
          onCommit(next);
        }
      }}
    />
  );
}
