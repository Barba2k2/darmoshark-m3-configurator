import { useEffect } from "react";

import { DeviceSection } from "../features/configurator/DeviceSection";
import { DpiSection } from "../features/configurator/DpiSection";
import { ResetSection } from "../features/configurator/ResetSection";
import { SettingsSection } from "../features/configurator/SettingsSection";
import { StatusSection } from "../features/configurator/StatusSection";
import { useMouseStore } from "../store/use_mouse_store";
import styles from "./ConfiguratorScreen.module.css";

/** The single window: device, DPI, settings, reset and the status line. */
export function ConfiguratorScreen() {
  useEffect(() => {
    const { load, refreshDevice } = useMouseStore.getState();
    void load();
    // Battery and connection only; fields being edited are left alone.
    const poll = window.setInterval(() => void refreshDevice(false), 30000);
    return () => window.clearInterval(poll);
  }, []);

  return (
    <main className={styles.screen}>
      <DeviceSection />
      <DpiSection />
      <SettingsSection />
      <div className={styles.spacer} />
      <ResetSection />
      <StatusSection />
    </main>
  );
}
