import { Button } from "../../design-system/inputs/Button";
import { DpiLevelEditor } from "../../design-system/inputs/DpiLevelEditor";
import { SectionCard } from "../../design-system/layout/SectionCard";
import { Labels } from "../../labels/labels";
import { useMouseStore } from "../../store/use_mouse_store";
import styles from "./DpiSection.module.css";

/**
 * The DPI levels. The last one carries the "free" badge: it is the slot meant
 * for experimenting with an arbitrary value.
 */
export function DpiSection() {
  const profile = useMouseStore((state) => state.profile);
  const device = useMouseStore((state) => state.device);
  const levels = useMouseStore((state) => state.levels);
  const activeLevel = useMouseStore((state) => state.activeLevel);
  const busy = useMouseStore((state) => state.busy);
  const editLevel = useMouseStore((state) => state.editLevel);
  const selectLevel = useMouseStore((state) => state.selectLevel);
  const applyLevels = useMouseStore((state) => state.applyLevels);
  if (profile === null) {
    return null;
  }
  const disabled = busy || device === null;
  return (
    <SectionCard title={Labels.dpiSection}>
      <p className={styles.hint}>{device?.transport === "cable" ? Labels.dpiHintCable : Labels.dpiHint}</p>
      <div className={styles.levels}>
        {levels.map((value, index) => (
          <DpiLevelEditor
            key={index}
            label={`${Labels.dpiSection} ${index + 1}`}
            color={profile.dpiColors[index] ?? "var(--color-accent-strong)"}
            value={value}
            minimum={profile.dpiMinimum}
            maximum={profile.dpiMaximum}
            selected={index === activeLevel}
            badge={index === levels.length - 1 ? Labels.dpiCustomBadge : undefined}
            disabled={disabled}
            onSelect={() => selectLevel(index)}
            onCommit={(next) => editLevel(index, next)}
          />
        ))}
      </div>
      <Button label={Labels.dpiApply} variant="primary" disabled={disabled} onPress={applyLevels} />
    </SectionCard>
  );
}
