import styles from "./StatusBar.module.css";

interface StatusBarProps {
  text: string;
  tone: "neutral" | "success" | "error";
}

/** One line reporting the outcome of the last operation. */
export function StatusBar({ text, tone }: StatusBarProps) {
  const toneClass = tone === "neutral" ? "" : styles[tone];
  return (
    <p className={`${styles.status} ${toneClass}`} role="status">
      {text}
    </p>
  );
}
