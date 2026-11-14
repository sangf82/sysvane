import { AlertDialog } from "@animate-ui/react/primitives/base-ui/alert-dialog";
import type { ReactNode } from "react";
import { t } from "./i18n";

/** One confirm dialog for the whole app: Animate UI's Base UI alert dialog, styled in styles.css.
 *  Without `confirm` it is a notice with a single OK button. */
export function Alert({ open, onClose, title, children, confirm }: { open: boolean; onClose: () => void; title: string; children: ReactNode; confirm?: { label: string; busy: boolean; onClick: () => void } }) {
  return (
    <AlertDialog.Root open={open} onOpenChange={(o) => !o && !confirm?.busy && onClose()}>
      <AlertDialog.Portal>
        <AlertDialog.Backdrop className="al-backdrop" />
        <AlertDialog.Viewport className="al-viewport">
          <AlertDialog.Popup className="al-popup">
            <AlertDialog.Title className="al-title">{title}</AlertDialog.Title>
            <AlertDialog.Description className="al-body" render={<div />}>
              {children}
            </AlertDialog.Description>
            <div className="row al-actions">
              <AlertDialog.Close disabled={confirm?.busy}>{confirm ? t("Later") : t("OK")}</AlertDialog.Close>
              {confirm && (
                <button className="primary" disabled={confirm.busy} onClick={confirm.onClick}>
                  {confirm.label}
                </button>
              )}
            </div>
          </AlertDialog.Popup>
        </AlertDialog.Viewport>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
