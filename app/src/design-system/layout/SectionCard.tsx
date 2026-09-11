import type { ReactNode } from "react";

import styles from "./SectionCard.module.css";

interface SectionCardProps {
  title: string;
  children: ReactNode;
}

/** A titled surface grouping related controls. */
export function SectionCard({ title, children }: SectionCardProps) {
  return (
    <section className={styles.card} aria-label={title}>
      <h2 className={styles.heading}>{title}</h2>
      {children}
    </section>
  );
}
