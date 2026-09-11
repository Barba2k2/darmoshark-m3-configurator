import styles from "./Field.module.css";

interface SelectOption {
  value: number;
  label: string;
}

interface SelectFieldProps {
  label: string;
  options: SelectOption[];
  value: number | null;
  disabled?: boolean;
  onSelect: (value: number) => void;
}

/** A dropdown over numeric values, each with its own caption. */
export function SelectField({ label, options, value, disabled, onSelect }: SelectFieldProps) {
  return (
    <select
      className={styles.field}
      aria-label={label}
      value={value ?? ""}
      disabled={disabled}
      onChange={(event) => onSelect(Number(event.target.value))}
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}
