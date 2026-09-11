import { StatusBar } from "../../design-system/display/StatusBar";
import { useMouseStore } from "../../store/use_mouse_store";
import { ConfiguratorTextHelper } from "./helpers/configurator_text_helper";

/** The outcome of the last operation. */
export function StatusSection() {
  const status = useMouseStore((state) => state.status);
  const errorDetail = useMouseStore((state) => state.errorDetail);
  return (
    <StatusBar text={ConfiguratorTextHelper.status(status, errorDetail)} tone={ConfiguratorTextHelper.tone(status)} />
  );
}
