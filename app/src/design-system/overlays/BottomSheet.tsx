import { Button } from "../inputs/Button";
import styles from "./BottomSheet.module.css";

interface BottomSheetProps {
  open: boolean;
  title: string;
  body: string;
  confirmLabel: string;
  cancelLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/** A confirmation that slides up from the bottom edge over a dimmed window. */
export function BottomSheet({ open, title, body, confirmLabel, cancelLabel, onConfirm, onCancel }: BottomSheetProps) {
  if (!open) {
    return null;
  }
  return (
    <div className={styles.backdrop} onClick={onCancel}>
      <div
        className={styles.sheet}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(event) => event.stopPropagation()}
      >
        <span className={styles.handle} />
        <h2 className={styles.title}>{title}</h2>
        <p className={styles.body}>{body}</p>
        <div className={styles.actions}>
          <Button label={confirmLabel} variant="danger" onPress={onConfirm} />
          <Button label={cancelLabel} variant="quiet" onPress={onCancel} />
        </div>
      </div>
    </div>
  );
}
