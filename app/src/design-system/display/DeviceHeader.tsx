import styles from "./DeviceHeader.module.css";

interface DeviceHeaderProps {
  title: string;
  subtitle: string;
}

/** Device name over a secondary line. */
export function DeviceHeader({ title, subtitle }: DeviceHeaderProps) {
  return (
    <header className={styles.header}>
      <h1 className={styles.title}>{title}</h1>
      <span className={styles.subtitle}>{subtitle}</span>
    </header>
  );
}
