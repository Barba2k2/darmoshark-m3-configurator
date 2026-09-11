import { NumberField } from "../../design-system/inputs/NumberField";
import { SelectField } from "../../design-system/inputs/SelectField";
import { SectionCard } from "../../design-system/layout/SectionCard";
import { SettingRow } from "../../design-system/layout/SettingRow";
import { Labels } from "../../labels/labels";
import { useMouseStore } from "../../store/use_mouse_store";
import { ConfiguratorTextHelper } from "./helpers/configurator_text_helper";

/** Polling rate, lift-off, debounce and the sleep timer (cable only). */
export function SettingsSection() {
  const profile = useMouseStore((state) => state.profile);
  const device = useMouseStore((state) => state.device);
  const busy = useMouseStore((state) => state.busy);
  const reportRate = useMouseStore((state) => state.reportRate);
  const liftOff = useMouseStore((state) => state.liftOff);
  const debounceMs = useMouseStore((state) => state.debounceMs);
  const sleepMinutes = useMouseStore((state) => state.sleepMinutes);
  const applyReportRate = useMouseStore((state) => state.applyReportRate);
  const applyLiftOff = useMouseStore((state) => state.applyLiftOff);
  const applyDebounce = useMouseStore((state) => state.applyDebounce);
  const applySleep = useMouseStore((state) => state.applySleep);
  if (profile === null) {
    return null;
  }
  const disabled = busy || device === null;
  const sleepOverReceiver = device?.transport === "receiver";
  return (
    <SectionCard title={Labels.settingsSection}>
      <SettingRow caption={Labels.reportRate}>
        <SelectField
          label={Labels.reportRate}
          options={profile.reportRates.map((hertz) => ({ value: hertz, label: `${hertz} ${Labels.unitHertz}` }))}
          value={reportRate}
          disabled={disabled}
          onSelect={applyReportRate}
        />
      </SettingRow>
      <SettingRow caption={Labels.liftOff}>
        <SelectField
          label={Labels.liftOff}
          options={profile.liftOffSteps.map((step) => ({ value: step, label: ConfiguratorTextHelper.liftOff(step) }))}
          value={liftOff}
          disabled={disabled}
          onSelect={applyLiftOff}
        />
      </SettingRow>
      <SettingRow caption={Labels.debounce}>
        <NumberField
          label={Labels.debounce}
          value={debounceMs}
          minimum={profile.debounceMinimum}
          maximum={profile.debounceMaximum}
          suffix={Labels.unitMilliseconds}
          disabled={disabled}
          onCommit={applyDebounce}
        />
      </SettingRow>
      <SettingRow caption={Labels.sleepTimer} note={sleepOverReceiver ? Labels.sleepCableOnly : undefined}>
        <NumberField
          label={Labels.sleepTimer}
          value={sleepMinutes}
          minimum={0}
          maximum={profile.sleepMaximumMinutes}
          suffix={Labels.unitMinutes}
          disabled={disabled || sleepOverReceiver}
          onCommit={applySleep}
        />
      </SettingRow>
    </SectionCard>
  );
}
