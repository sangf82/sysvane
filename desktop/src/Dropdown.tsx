import { Select } from "@animate-ui/react/primitives/base-ui/select";

/** One dropdown for the whole app: Animate UI's Base UI select, styled in styles.css. */
export function Dropdown<T extends string>({ value, onChange, options, label }: { value: T; onChange: (v: T) => void; options: { value: T; label: string }[]; label: string }) {
  const items = options.map((o) => ({ ...o, label: o.label.charAt(0).toUpperCase() + o.label.slice(1) }));
  return (
    <Select.Root value={value} onValueChange={(v) => v && onChange(v as T)} items={items}>
      <Select.Trigger className="dd-trigger" aria-label={label}>
        <Select.Value />
        <Select.Icon className="dd-caret" aria-hidden>▾</Select.Icon>
      </Select.Trigger>
      <Select.Portal>
        <Select.Positioner sideOffset={4} alignItemWithTrigger={false}>
          <Select.Popup className="dd-popup">
            <Select.List>
              {items.map((o) => (
                <Select.Item key={o.value} value={o.value} className="dd-item">
                  <Select.ItemText>{o.label}</Select.ItemText>
                  <Select.ItemIndicator className="dd-check" aria-hidden>✓</Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.List>
          </Select.Popup>
        </Select.Positioner>
      </Select.Portal>
    </Select.Root>
  );
}
