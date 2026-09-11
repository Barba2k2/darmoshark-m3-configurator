import styles from "./Field.module.css";
import { NumberInput } from "./NumberInput";

interface NumberFieldProps {
  label: string;
  value: number;
  minimum: number;
  maximum: number;
  suffix: string;
  disabled?: boolean;
  onCommit: (value: number) => void;
}

/** A settings field: a whole-number input followed by its unit. */
export function NumberField({ label, value, minimum, maximum, suffix, disabled, onCommit }: NumberFieldProps) {
  return (
    <span>
      <NumberInput
        label={label}
        className={styles.field}
        value={value}
        minimum={minimum}
        maximum={maximum}
        disabled={disabled}
        onCommit={onCommit}
      />{" "}
      {suffix}
    </span>
  );
}
