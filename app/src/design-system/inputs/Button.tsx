import styles from "./Button.module.css";

interface ButtonProps {
  label: string;
  variant: "primary" | "danger" | "quiet";
  disabled?: boolean;
  onPress: () => void;
}

/** A full-width action button. */
export function Button({ label, variant, disabled, onPress }: ButtonProps) {
  return (
    <button type="button" className={`${styles.button} ${styles[variant]}`} disabled={disabled} onClick={onPress}>
      {label}
    </button>
  );
}
