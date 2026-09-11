import type { CSSProperties } from "react";

import styles from "./DpiLevelEditor.module.css";
import { NumberInput } from "./NumberInput";

interface DpiLevelEditorProps {
  label: string;
  color: string;
  value: number;
  minimum: number;
  maximum: number;
  selected: boolean;
  badge?: string;
  disabled?: boolean;
  onSelect: () => void;
  onCommit: (value: number) => void;
}

/**
 * One DPI level: a button in the colour the mouse LED lights for it, over its
 * editable value. The badge slot is always reserved so a row of editors keeps
 * one baseline.
 */
export function DpiLevelEditor({
  label,
  color,
  value,
  minimum,
  maximum,
  selected,
  badge,
  disabled,
  onSelect,
  onCommit,
}: DpiLevelEditorProps) {
  const colorStyle = { "--level-color": color } as CSSProperties;
  return (
    <div className={styles.editor} style={colorStyle}>
      <button
        type="button"
        className={selected ? `${styles.level} ${styles.selected}` : styles.level}
        aria-label={label}
        aria-pressed={selected}
        disabled={disabled}
        onClick={onSelect}
      >
        {value}
      </button>
      <NumberInput
        label={label}
        className={styles.value}
        value={value}
        minimum={minimum}
        maximum={maximum}
        step={100}
        disabled={disabled}
        onCommit={onCommit}
      />
      <span className={styles.badge}>{badge}</span>
    </div>
  );
}
