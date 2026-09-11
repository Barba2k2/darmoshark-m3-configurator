import styles from "./BatteryGauge.module.css";

interface BatteryGaugeProps {
  caption: string;
  /** 0-100, or null while unknown. */
  percent: number | null;
  valueLabel: string;
}

/** A labelled charge bar that turns red at 20% and below. */
export function BatteryGauge({ caption, percent, valueLabel }: BatteryGaugeProps) {
  const level = Math.max(0, Math.min(100, percent ?? 0));
  const isLow = percent !== null && percent <= 20;
  return (
    <div className={styles.gauge}>
      <div className={styles.header}>
        <span className={styles.caption}>{caption}</span>
        <span>{valueLabel}</span>
      </div>
      <div
        className={styles.track}
        role="meter"
        aria-label={caption}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={level}
      >
        <div className={isLow ? `${styles.fill} ${styles.low}` : styles.fill} style={{ width: `${level}%` }} />
      </div>
    </div>
  );
}
