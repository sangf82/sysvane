import { Select } from "@animate-ui/react/primitives/base-ui/select";
import { useState } from "react";
import { Glyph } from "./glyphs";
import { Icon } from "./layout";

/** One dropdown for the whole app: Animate UI's Base UI select, styled in styles.css. */
export function Dropdown<T extends string>({ value, onChange, options, label, icon }: { value: T; onChange: (v: T) => void; options: { value: T; label: string }[]; label: string; icon?: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  const items = options.map((o) => ({ ...o, label: o.label.charAt(0).toUpperCase() + o.label.slice(1) }));
  return (
    <Select.Root value={value} onValueChange={(v) => v && onChange(v as T)} items={items} open={open} onOpenChange={setOpen}>
      {/* The select opens on pointer down. A click with no pointer (detail 0) comes from a screen reader's
          default action or UI Automation; open for it too. Opening twice is harmless. */}
      <Select.Trigger className="dd-trigger" aria-label={label} onClick={(e) => e.detail === 0 && setOpen(true)}>
        {icon && <span className="dd-lead">{icon}</span>}
        <Select.Value />
        <Select.Icon className="dd-caret" aria-hidden>
          <Icon name="sortDesc" size={14} />
        </Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Positioner className="dd-pos" sideOffset={4} alignItemWithTrigger={false}>
          <Select.Popup className="dd-popup">
            <Select.List>
              {items.map((o) => (
                <Select.Item key={o.value} value={o.value} className="dd-item">
                  <Select.ItemText>{o.label}</Select.ItemText>
                  <Select.ItemIndicator className="dd-check" aria-hidden>
                    <Glyph name="check" />
                  </Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.List>
          </Select.Popup>
        </Select.Positioner>
      </Select.Portal>
    </Select.Root>
  );
}
