import { Switch as Primitive } from "@animate-ui/react/primitives/base-ui/switch";

/** One switch for the whole app: Animate UI's Base UI switch, styled in styles.css. */
export function Switch({ checked, onChange, label, disabled }: { checked: boolean; onChange: (on: boolean) => void; label: string; disabled?: boolean }) {
  return (
    <Primitive.Root className="switch" checked={checked} onCheckedChange={onChange} aria-label={label} disabled={disabled}>
      <Primitive.Thumb className="switch-thumb" />
    </Primitive.Root>
  );
}
