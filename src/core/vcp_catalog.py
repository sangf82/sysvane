"""VCP code catalog: human-readable names, ranges, and types for DDC/CI codes.

Reference: VESA MCCS 2.2a standard + CVTE/MStar vendor extensions.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import ClassVar


@dataclass(frozen=True)
class VCPEntry:
    """Metadata for a single VCP register."""

    code: int
    name: str
    category: str = "General"
    min_val: int = 0
    max_val: int = 100
    readable: bool = True
    writeable: bool = True
    is_continuous: bool = True
    discrete_values: dict[int, str] | None = None
    manufacturer_specific: bool = False


# Standard VESA MCCS 2.2a VCP codes
_VCP_CODES: dict[int, VCPEntry] = {
    # ── Image adjustment ──────────────────────────────────────────
    0x00: VCPEntry(0x00, "Code Page"),
    0x02: VCPEntry(0x02, "New Control Value"),
    0x04: VCPEntry(0x04, "Restore Factory Defaults", writeable=True, readable=False),
    0x05: VCPEntry(0x05, "Restore Factory Luminance/Contrast Defaults", writeable=True, readable=False),
    0x06: VCPEntry(0x06, "Restore Factory Color Defaults", writeable=True, readable=False),
    0x08: VCPEntry(0x08, "Restore Factory Defaults", writeable=True, readable=False),
    0x0B: VCPEntry(0x0B, "Color Temperature Increment"),
    0x0C: VCPEntry(0x0C, "Color Temperature Request"),
    0x10: VCPEntry(0x10, "Luminance / Brightness", max_val=100),
    0x12: VCPEntry(0x12, "Contrast", max_val=100),
    0x13: VCPEntry(0x13, "Backlight Control", max_val=100),
    0x14: VCPEntry(0x14, "Color Preset", min_val=1, max_val=12),
    0x16: VCPEntry(0x16, "Video Gain (Red)", max_val=100),
    0x18: VCPEntry(0x18, "Video Gain (Green)", max_val=100),
    0x1A: VCPEntry(0x1A, "Video Gain (Blue)", max_val=100),
    0x1C: VCPEntry(0x1C, "Focus"),
    0x1E: VCPEntry(0x1E, "Auto Setup", writeable=True, readable=False),
    0x20: VCPEntry(0x20, "Horizontal Position"),
    0x22: VCPEntry(0x22, "Horizontal Size"),
    0x24: VCPEntry(0x24, "Horizontal Pincushion"),
    0x26: VCPEntry(0x26, "Horizontal Pincushion Balance"),
    0x28: VCPEntry(0x28, "Horizontal Convergence"),
    0x2A: VCPEntry(0x2A, "Horizontal Linearity"),
    0x2C: VCPEntry(0x2C, "Horizontal Linearity Balance"),
    0x2E: VCPEntry(0x2E, "Gray Scale Expansion"),
    0x30: VCPEntry(0x30, "Vertical Position"),
    0x32: VCPEntry(0x32, "Vertical Size"),
    0x34: VCPEntry(0x34, "Vertical Pincushion"),
    0x36: VCPEntry(0x36, "Vertical Pincushion Balance"),
    0x38: VCPEntry(0x38, "Vertical Convergence"),
    0x3A: VCPEntry(0x3A, "Vertical Linearity"),
    0x3C: VCPEntry(0x3C, "Vertical Linearity Balance"),
    0x3E: VCPEntry(0x3E, "Clock"),
    0x40: VCPEntry(0x40, "Clock Phase"),
    0x42: VCPEntry(0x42, "Horizontal Parallelogram"),
    0x44: VCPEntry(0x44, "Horizontal Keystone"),
    0x46: VCPEntry(0x46, "Vertical Keystone"),
    0x48: VCPEntry(0x48, "Horizontal Rotation"),
    0x4A: VCPEntry(0x4A, "Vertical Rotation"),
    0x4C: VCPEntry(0x4C, "Video Black Level (Red)"),
    0x4E: VCPEntry(0x4E, "Video Black Level (Green)"),
    0x50: VCPEntry(0x50, "Video Black Level (Blue)"),
    0x52: VCPEntry(0x52, "Active Control"),
    0x54: VCPEntry(0x54, "Performance Preservation"),
    0x56: VCPEntry(0x56, "Horizontal Moire"),
    0x58: VCPEntry(0x58, "Vertical Moire"),
    0x59: VCPEntry(0x59, "6-Axis Saturation (Red)"),
    0x5A: VCPEntry(0x5A, "6-Axis Saturation (Yellow)"),
    0x5B: VCPEntry(0x5B, "6-Axis Saturation (Green)"),
    0x5C: VCPEntry(0x5C, "6-Axis Saturation (Cyan)"),
    0x5D: VCPEntry(0x5D, "6-Axis Saturation (Blue)"),
    0x5E: VCPEntry(0x5E, "6-Axis Saturation (Magenta)"),
    0x60: VCPEntry(0x60, "Input Source", min_val=1, max_val=18),
    0x62: VCPEntry(0x62, "Audio Volume", max_val=100),
    0x63: VCPEntry(0x63, "Audio Speaker Volume Adjust", max_val=100),
    0x64: VCPEntry(0x64, "Audio Microphone Volume", max_val=100),
    0x66: VCPEntry(0x66, "Ambient Light Sensor"),
    0x6C: VCPEntry(0x6C, "Video Black Level (Red)"),
    0x6E: VCPEntry(0x6E, "Video Black Level (Green)"),
    0x70: VCPEntry(0x70, "Video Black Level (Blue)"),
    0x72: VCPEntry(0x72, "Gamma"),
    0x73: VCPEntry(0x73, "LUT Size"),
    0x74: VCPEntry(0x74, "Single Point LUT Operation"),
    0x75: VCPEntry(0x75, "Block LUT Operation"),
    0x76: VCPEntry(0x76, "Remote Procedure Call"),
    0x78: VCPEntry(0x78, "Display Identification Data Operation"),
    0x7A: VCPEntry(0x7A, "Adjust Zoom"),
    0x7C: VCPEntry(0x7C, "Sharpness", max_val=100),
    0x7E: VCPEntry(0x7E, "Velocity Scan Modulation"),
    0x80: VCPEntry(0x80, "Flat Panel Sub-Pixel Layout"),
    0x82: VCPEntry(0x82, "Display Technology Type"),
    0x84: VCPEntry(0x84, "Horizontal Frequency (kHz)", readable=True, writeable=False),
    0x86: VCPEntry(0x86, "Vertical Frequency (Hz * 100)", readable=True, writeable=False),
    0x87: VCPEntry(0x87, "Overscan", min_val=0, max_val=1),
    0x8A: VCPEntry(0x8A, "Saturation (Red)"),
    0x8C: VCPEntry(0x8C, "Saturation (Green)"),
    0x8D: VCPEntry(0x8D, "Audio Treble"),
    0x8E: VCPEntry(0x8E, "Saturation (Blue)"),
    0x8F: VCPEntry(0x8F, "Audio Bass"),
    0x90: VCPEntry(0x90, "Hue (Red)"),
    0x91: VCPEntry(0x91, "Audio Microphone Balance"),
    0x92: VCPEntry(0x92, "Hue (Green)"),
    0x93: VCPEntry(0x93, "Audio Balance (L/R)"),
    0x94: VCPEntry(0x94, "Hue (Blue)"),
    0x95: VCPEntry(0x95, "Window Position (TL_X)"),
    0x96: VCPEntry(0x96, "Window Position (TL_Y)"),
    0x97: VCPEntry(0x97, "Window Position (BR_X)"),
    0x98: VCPEntry(0x98, "Window Position (BR_Y)"),
    0x9A: VCPEntry(0x9A, "Window Background"),
    0x9B: VCPEntry(0x9B, "6-Axis Hue (Red)"),
    0x9C: VCPEntry(0x9C, "6-Axis Hue (Yellow)"),
    0x9D: VCPEntry(0x9D, "6-Axis Hue (Green)"),
    0x9E: VCPEntry(0x9E, "6-Axis Hue (Cyan)"),
    0x9F: VCPEntry(0x9F, "6-Axis Hue (Blue)"),
    0xA0: VCPEntry(0xA0, "6-Axis Hue (Magenta)"),
    0xA2: VCPEntry(0xA2, "Auto Color Setup", writeable=True, readable=False),
    0xA4: VCPEntry(0xA4, "Window Mask Control"),
    0xA5: VCPEntry(0xA5, "Window Select"),
    0xA6: VCPEntry(0xA6, "Window Size"),
    0xA7: VCPEntry(0xA7, "Window Transparency"),
    0xAA: VCPEntry(0xAA, "Screen Orientation", min_val=1, max_val=4),
    0xAC: VCPEntry(0xAC, "Horizontal Frequency (coarse)", readable=True, writeable=False),
    0xAE: VCPEntry(0xAE, "Vertical Frequency (coarse)", readable=True, writeable=False),
    0xB0: VCPEntry(0xB0, "Settings LUT"),
    0xB2: VCPEntry(0xB2, "Flat Panel DPM Enable"),
    0xB4: VCPEntry(0xB4, "Vertical Black Level (Red)"),
    0xB6: VCPEntry(0xB6, "Display Application", min_val=1, max_val=4),
    0xB8: VCPEntry(0xB8, "Vertical Black Level (Green)"),
    0xBA: VCPEntry(0xBA, "Vertical Black Level (Blue)"),
    0xC0: VCPEntry(0xC0, "Display Usage Time (hours)", readable=True, writeable=False),
    0xC2: VCPEntry(0xC2, "Blue Light Filter", max_val=100),
    0xC6: VCPEntry(0xC6, "Power Mode", min_val=1, max_val=5),
    0xC8: VCPEntry(0xC8, "Display Controller ID", readable=True, writeable=False),
    0xC9: VCPEntry(0xC9, "Display Firmware Level", readable=True, writeable=False),
    0xCA: VCPEntry(0xCA, "OSD Language", min_val=1, max_val=14),
    0xCC: VCPEntry(0xCC, "OSD Button Level Control"),
    0xD0: VCPEntry(0xD0, "Output Select"),
    0xD4: VCPEntry(0xD4, "Stereo Video Mode"),
    0xD6: VCPEntry(0xD6, "Power Mode (On/Off)", min_val=1, max_val=5),
    0xDC: VCPEntry(0xDC, "Display Enable"),
    0xDF: VCPEntry(0xDF, "VCP Version", readable=True, writeable=False),
}

# Manufacturer-specific range 0xE0 - 0xFF
_MFG_RANGE: dict[int, VCPEntry] = {
    code: VCPEntry(
        code=code,
        name=f"Manufacturer Specific [0x{code:02X}]",
        category="Manufacturer",
        min_val=0,
        max_val=255,
        manufacturer_specific=True,
    )
    for code in range(0xE0, 0x100)
}

# Discrete value lookup tables
INPUT_SOURCE_NAMES: dict[int, str] = {
    0x01: "VGA 1",
    0x02: "VGA 2",
    0x03: "DVI 1",
    0x04: "DVI 2",
    0x05: "Composite 1",
    0x06: "Composite 2",
    0x07: "S-Video 1",
    0x08: "S-Video 2",
    0x09: "Tuner 1",
    0x0A: "Tuner 2",
    0x0B: "Component 1",
    0x0C: "Component 2",
    0x0D: "Component 3",
    0x0F: "DisplayPort 1",
    0x10: "DisplayPort 2",
    0x11: "HDMI 1",
    0x12: "HDMI 2",
    0x13: "USB-C",
}

COLOR_PRESET_NAMES: dict[int, str] = {
    0x01: "sRGB",
    0x02: "Display Native",
    0x03: "4000K",
    0x04: "5000K",
    0x05: "6500K",
    0x06: "7500K",
    0x07: "8200K",
    0x08: "9300K",
    0x09: "10000K",
    0x0A: "11500K",
    0x0B: "User 1",
    0x0C: "User 2",
    0x0D: "User 3",
}

OSD_LANGUAGE_NAMES: dict[int, str] = {
    1: "Chinese (Traditional)",
    2: "English",
    3: "French",
    4: "German",
    5: "Italian",
    6: "Japanese",
    7: "Korean",
    8: "Portuguese",
    9: "Russian",
    10: "Spanish",
    11: "Swedish",
    12: "Turkish",
    13: "Chinese (Simplified)",
    14: "Portuguese (Brazil)",
    15: "Arabic",
}

POWER_MODE_NAMES: dict[int, str] = {
    1: "On",
    2: "Standby",
    3: "Suspend",
    4: "Off (Soft)",
    5: "Off (Hard)",
}

DISPLAY_APP_NAMES: dict[int, str] = {
    1: "Standard",
    2: "E-Book",
    3: "Movie",
    4: "Picture",
}

DANGEROUS_CODES: frozenset[int] = frozenset({
    0x04,  # Restore Factory Defaults
    0x05,  # Restore Luminance/Contrast
    0x06,  # Restore Color Defaults
    0x08,  # Restore Factory Defaults (alt)
})

CONFIRM_CODES: frozenset[int] = frozenset({
    0x60,  # Input Source
    0xD6,  # Power Mode
})

POWER_CODES: frozenset[int] = frozenset({
    0xC6, 0xD6,
})


class VCPCatalog:
    """Centralised lookup of VCP codes and their metadata."""

    _full: ClassVar[dict[int, VCPEntry]] = {**_VCP_CODES, **_MFG_RANGE}
    INPUT_SOURCE_NAMES: ClassVar[dict[int, str]] = INPUT_SOURCE_NAMES
    COLOR_PRESET_NAMES: ClassVar[dict[int, str]] = COLOR_PRESET_NAMES
    OSD_LANGUAGE_NAMES: ClassVar[dict[int, str]] = OSD_LANGUAGE_NAMES
    DISPLAY_APP_NAMES: ClassVar[dict[int, str]] = DISPLAY_APP_NAMES
    DANGEROUS_CODES: ClassVar[frozenset[int]] = DANGEROUS_CODES
    CONFIRM_CODES: ClassVar[frozenset[int]] = CONFIRM_CODES
    POWER_CODES: ClassVar[frozenset[int]] = POWER_CODES

    @classmethod
    def get(cls, code: int) -> VCPEntry | None:
        return cls._full.get(code)

    @classmethod
    def codes(cls) -> list[int]:
        return sorted(cls._full)

    @classmethod
    def standard_codes(cls) -> list[int]:
        return sorted(_VCP_CODES)

    @classmethod
    def mfg_codes(cls) -> list[int]:
        return sorted(_MFG_RANGE)

    @classmethod
    def is_mfg(cls, code: int) -> bool:
        entry = cls._full.get(code)
        return entry is not None and entry.manufacturer_specific

    @classmethod
    def name(cls, code: int) -> str:
        entry = cls._full.get(code)
        return entry.name if entry else f"Unknown [0x{code:02X}]"

    @classmethod
    def category(cls, code: int) -> str:
        entry = cls._full.get(code)
        return entry.category if entry else "Unknown"

    @classmethod
    def input_source_name(cls, val: int) -> str:
        return INPUT_SOURCE_NAMES.get(val, f"Input {val}")

    @classmethod
    def color_preset_name(cls, val: int) -> str:
        return COLOR_PRESET_NAMES.get(val, f"Preset {val}")

    @classmethod
    def power_mode_name(cls, val: int) -> str:
        return POWER_MODE_NAMES.get(val, f"Mode {val}")

    @classmethod
    def osd_language_name(cls, val: int) -> str:
        return OSD_LANGUAGE_NAMES.get(val, f"Language {val}")

    @classmethod
    def display_app_name(cls, val: int) -> str:
        return DISPLAY_APP_NAMES.get(val, f"Mode {val}")

    @classmethod
    def is_dangerous(cls, code: int) -> bool:
        return code in DANGEROUS_CODES

    @classmethod
    def needs_confirm(cls, code: int) -> bool:
        return code in CONFIRM_CODES

    @classmethod
    def is_power_code(cls, code: int) -> bool:
        return code in POWER_CODES

    @classmethod
    def is_power_off_value(cls, code: int, val: int) -> bool:
        """Return True if writing `val` to `code` would put monitor in standby or turn it off."""
        # 1 is On for 0xD6 (2..5 are standby/suspend/off)
        # 90 is normal on CVTE/Acer, 1 on standard scalers for 0xC6
        return (code == 0xD6 and val != 1) or (code == 0xC6 and val not in (1, 90))
