"""OSD-oriented control definitions built from monitor capabilities."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any

from .vcp_catalog import VCPCatalog


@dataclass(frozen=True)
class ControlDefinition:
    key: str
    label: str
    category: str
    code: int | None
    help_text: str
    min_val: int = 0
    max_val: int = 100
    values: tuple[tuple[str, int], ...] = ()
    read_only: bool = False
    experimental: bool = False
    confirmed: bool = True
    dangerous: bool = False

    @property
    def code_key(self) -> str:
        return f"0x{self.code:02X}" if self.code is not None else self.key

    @property
    def is_writable(self) -> bool:
        return self.code is not None and not self.read_only


def _enum_options(values: Any, fallback: dict[int, str]) -> tuple[tuple[str, int], ...]:
    options: list[tuple[str, int]] = []
    for value in values or []:
        raw = value.value if isinstance(value, Enum) else int(value)
        name = getattr(value, "name", fallback.get(raw, str(raw)))
        name = name.replace("COLOR_TEMP_", "").replace("_", " ").title()
        options.append((fallback.get(raw, name), raw))
    if not options:
        options = [(name, raw) for raw, name in sorted(fallback.items())]
    return tuple(options)


def _entry_bounds(code: int) -> tuple[int, int]:
    entry = VCPCatalog.get(code)
    if entry is None:
        return 0, 100
    return entry.min_val, entry.max_val


def _coerce_code(value: Any) -> int | None:
    try:
        raw = value.value if isinstance(value, Enum) else value
        return int(raw)
    except (TypeError, ValueError):
        return None


def _supported_codes(raw: Any) -> set[int]:
    if not isinstance(raw, dict):
        return set()
    return {code for key in raw if (code := _coerce_code(key)) is not None}


def build_control_definitions(
    capabilities: dict[str, Any] | None,
    experimental_mappings: dict[str, int] | None = None,
) -> list[ControlDefinition]:
    """Build OSD-ordered controls from parsed monitor capabilities."""
    caps = capabilities or {}
    mappings = experimental_mappings or {}
    # Some monitors list codes only in "cmds"; if neither is advertised, keep the
    # standard controls visible instead of rendering an empty panel.
    supported = _supported_codes(caps.get("vcp"))
    if not supported:
        supported = _supported_codes(caps.get("cmds"))
    input_values = _enum_options(caps.get("inputs"), VCPCatalog.INPUT_SOURCE_NAMES)
    color_values = _enum_options(caps.get("color_presets"), VCPCatalog.COLOR_PRESET_NAMES)

    definitions: list[ControlDefinition] = []

    def add(
        key: str,
        label: str,
        category: str,
        code: int | None,
        help_text: str,
        *,
        values: tuple[tuple[str, int], ...] = (),
        read_only: bool = False,
        experimental: bool = False,
        dangerous: bool = False,
        require_supported: bool = True,
    ) -> None:
        if supported and code is not None and require_supported and code not in supported:
            return
        min_val, max_val = _entry_bounds(code) if code is not None else (0, 100)
        definitions.append(
            ControlDefinition(
                key=key,
                label=label,
                category=category,
                code=code,
                help_text=help_text,
                min_val=min_val,
                max_val=max_val,
                values=values,
                read_only=read_only,
                experimental=experimental,
                confirmed=not experimental,
                dangerous=dangerous,
            )
        )

    add("brightness", "Brightness", "Picture", 0x10, "Backlight strength. Higher is brighter and uses more power.")
    add("backlight", "Backlight Control", "Picture", 0x13, "Separate backlight level on panels that split brightness from backlight.")
    add("contrast", "Contrast", "Picture", 0x12, "White/black separation. Too high can crush bright detail.")
    add("sharpness", "Sharpness", "Picture", 0x7C, "Edge sharpening filter. 0 is soft, 100 is sharpest.")
    add("blue-light", "Blue Light Filter", "Picture", 0xC2, "Reduces blue light for eye comfort. Higher values filter more blue.")
    add("overscan", "Overscan", "Picture", 0x87, "Overscan toggle. 0 = off (show full image), 1 = on (crop edges).")
    add("eco", "ECO", "Picture", mappings.get("ECO"), "Vendor OSD mode. Use Scanner first because this monitor does not advertise a DDC code.", experimental=True, require_supported=False)
    add("black-level", "Black Level", "Picture", mappings.get("Black Level"), "Adjusts dark shadow lift. Unmapped until Scanner finds the vendor DDC code.", experimental=True, require_supported=False)
    add("aspect-ratio", "Aspect Ratio", "Picture", mappings.get("Aspect Ratio"), "Scaling mode such as full or aspect. This is OSD-only until Scanner maps it.", experimental=True, require_supported=False)
    display_app_values = tuple((VCPCatalog.display_app_name(v), v) for v in [1, 2, 3, 4])
    add("professional-mode", "Professional Mode", "Picture", 0xB6, "Picture preset. Standard is neutral, E-Book reduces blue, Movie boosts contrast, Picture sharpens.", values=display_app_values, experimental=True)

    add("color-preset", "Color Preset", "Color", 0x14, "White point preset. 6500K is the usual neutral sRGB target.", values=color_values)
    add("color-temp", "Color Temperature", "Color", 0x0C, "Fine kelvin value when Color Preset is set to a user-defined mode.")
    add("gamma", "Gamma", "Color", 0x72, "Gamma curve. Controls midtone brightness. 2.2 is the sRGB standard.")
    add("red-gain", "Red Gain", "Color", 0x16, "Fine-tunes red channel strength for manual white balance.")
    add("green-gain", "Green Gain", "Color", 0x18, "Fine-tunes green channel strength for manual white balance.")
    add("blue-gain", "Blue Gain", "Color", 0x1A, "Fine-tunes blue channel strength for manual white balance.")
    for code, label, help_text in [
        (0x59, "Saturation Red", "Changes only red color intensity for 6-axis color tuning."),
        (0x5A, "Saturation Yellow", "Changes only yellow color intensity for 6-axis color tuning."),
        (0x5B, "Saturation Green", "Changes only green color intensity for 6-axis color tuning."),
        (0x5C, "Saturation Cyan", "Changes only cyan color intensity for 6-axis color tuning."),
        (0x5D, "Saturation Blue", "Changes only blue color intensity for 6-axis color tuning."),
        (0x5E, "Saturation Magenta", "Changes only magenta color intensity for 6-axis color tuning."),
        (0x9B, "Hue Red", "Shifts only red color tint for calibration work."),
        (0x9C, "Hue Yellow", "Shifts only yellow color tint for calibration work."),
        (0x9D, "Hue Green", "Shifts only green color tint for calibration work."),
        (0x9E, "Hue Cyan", "Shifts only cyan color tint for calibration work."),
        (0x9F, "Hue Blue", "Shifts only blue color tint for calibration work."),
        (0xA0, "Hue Magenta", "Shifts only magenta color tint for calibration work."),
    ]:
        add(f"vcp-{code:02x}", label, "Color", code, help_text)

    add("adaptive-sync", "Adaptive-Sync", "Game", 0xB2, "Variable refresh support candidate. Experimental because the monitor does not name this code clearly.", experimental=True)

    add("input-source", "Input Source", "System", 0x60, "Selects HDMI/DisplayPort input. Switching to a disconnected port may black out the display.", values=input_values, dangerous=True)
    add("volume", "Volume", "System", 0x62, "Monitor speaker/headphone volume if your model exposes audio control.")
    language_values = _enum_options(caps.get("osd_languages"), VCPCatalog.OSD_LANGUAGE_NAMES)
    add("osd-language", "OSD Language", "System", 0xCC, "OSD menu language. Values are per MCCS 2.2 spec but vendor support varies.", values=language_values)
    add("power-mode", "Power Mode", "System", 0xD6, "Monitor power state. Standby/Off will turn off the display.", values=tuple((VCPCatalog.power_mode_name(v), v) for v in [1, 2, 3, 4, 5]), dangerous=True)
    add("horizontal-frequency", "Horizontal Frequency", "System", 0xAC, "Signal information reported by the monitor. Change resolution/timing in Windows or GPU settings.", read_only=True)
    add("vertical-frequency", "Refresh Rate", "System", 0xAE, "Current signal refresh rate. DDC can read it, but Windows/NVIDIA controls the setting.", read_only=True)
    add("vcp-version", "VCP Version", "System", 0xDF, "DDC/CI protocol version reported by the monitor. Informational only.", read_only=True)

    return definitions


def format_control_value(definition: ControlDefinition, value: int | None) -> str:
    if value is None:
        return "Unmapped" if definition.code is None else "-"
    for label, option_value in definition.values:
        if option_value == value:
            return label
    if definition.code == 0xB6:
        return VCPCatalog.display_app_name(value)
    if definition.code == 0xAE:
        return f"{value // 100} Hz" if value >= 100 else f"{value} Hz"
    if definition.code == 0xDF:
        return f"{value >> 8}.{value & 0xFF}"
    return str(value)
