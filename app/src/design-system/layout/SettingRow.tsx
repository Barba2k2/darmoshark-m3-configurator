import type { ReactNode } from "react";

import styles from "./SettingRow.module.css";

interface SettingRowProps {
  caption: string;
  note?: string;
  children: ReactNode;
}

/** A caption on the left, its control on the right. */
export function SettingRow({ caption, note, children }: SettingRowProps) {
  return (
    <div className={styles.row}>
      <span className={styles.caption}>
        {caption}
        {note && <span className={styles.note}>{note}</span>}
      </span>
      {children}
    </div>
  );
}
