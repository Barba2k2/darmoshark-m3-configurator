import { Button } from "../../design-system/inputs/Button";
import { BottomSheet } from "../../design-system/overlays/BottomSheet";
import { Labels } from "../../labels/labels";
import { useMouseStore } from "../../store/use_mouse_store";

/** Factory reset, behind a confirmation sheet. */
export function ResetSection() {
  const device = useMouseStore((state) => state.device);
  const busy = useMouseStore((state) => state.busy);
  const resetSheetOpen = useMouseStore((state) => state.resetSheetOpen);
  const openResetSheet = useMouseStore((state) => state.openResetSheet);
  const closeResetSheet = useMouseStore((state) => state.closeResetSheet);
  const confirmReset = useMouseStore((state) => state.confirmReset);
  return (
    <>
      <Button label={Labels.resetButton} variant="danger" disabled={busy || device === null} onPress={openResetSheet} />
      <BottomSheet
        open={resetSheetOpen}
        title={Labels.resetConfirmTitle}
        body={Labels.resetConfirmBody}
        confirmLabel={Labels.resetConfirm}
        cancelLabel={Labels.resetCancel}
        onConfirm={confirmReset}
        onCancel={closeResetSheet}
      />
    </>
  );
}
