import { BatteryGauge } from "../../design-system/display/BatteryGauge";
import { DeviceHeader } from "../../design-system/display/DeviceHeader";
import { Labels } from "../../labels/labels";
import { useMouseStore } from "../../store/use_mouse_store";
import { ConfiguratorTextHelper } from "./helpers/configurator_text_helper";

/** Which mouse is connected, through what, and how much charge it has. */
export function DeviceSection() {
  const connection = useMouseStore((state) => state.connection);
  const device = useMouseStore((state) => state.device);
  return (
    <>
      <DeviceHeader title={Labels.windowTitle} subtitle={ConfiguratorTextHelper.subtitle(connection, device)} />
      <BatteryGauge
        caption={Labels.batteryCaption}
        percent={device?.batteryPercent ?? null}
        valueLabel={ConfiguratorTextHelper.battery(device)}
      />
    </>
  );
}
