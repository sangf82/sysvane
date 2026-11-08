"""Profile engine: JSON-based save/load/apply of monitor VCP state."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, ClassVar

from .monitor_manager import MonitorHandle, MonitorManager
from .vcp_catalog import VCPCatalog


DEFAULT_PROFILES_DIR = Path(__file__).resolve().parents[2] / "profiles"
DEFAULT_EXPERIMENTAL_MAPPINGS_PATH = DEFAULT_PROFILES_DIR / "experimental_mappings.json"


@dataclass
class Profile:
    name: str
    description: str = ""
    settings: dict[str, int] = field(default_factory=dict)
    experimental_mappings: dict[str, int] = field(default_factory=dict)

    FILENAME_REPLACE: ClassVar[dict[int, Any]] = str.maketrans({" ": "_", "/": "-", "\\": "-", ":": ""})

    @classmethod
    def capture(
        cls,
        name: str,
        arg2: Any = "",
        arg3: Any = None,
        *,
        description: str = "",
        manager: MonitorManager | None = None,
        monitor: MonitorHandle | None = None,
        codes: list[int] | None = None,
    ) -> Profile:
        """Read current VCP values from the active monitor and build a Profile.

        Supports both:
          - Profile.capture(name, manager, description=desc)
          - Profile.capture(name, description, manager)
        """
        if isinstance(arg2, str):
            final_desc = description or arg2
            final_mgr = arg3 if arg3 is not None else manager
        else:
            final_mgr = arg2 if arg2 is not None else manager
            final_desc = description or (arg3 if isinstance(arg3, str) else "")

        if final_mgr is None:
            raise ValueError("MonitorManager must be provided to Profile.capture")

        explicit = codes is not None
        if codes is None:
            codes = VCPCatalog.standard_codes()
        settings: dict[str, int] = {}
        for code in codes:
            if not explicit and VCPCatalog.is_dangerous(code):
                continue
            if not explicit and VCPCatalog.is_power_code(code):
                continue
            entry = VCPCatalog.get(code)
            if not explicit and entry is not None and not entry.readable:
                continue
            val = final_mgr.get_vcp(code, monitor)
            if val is not None:
                settings[f"0x{code:02X}"] = val
        return cls(name=name, description=final_desc, settings=settings)

    def apply(
        self,
        manager: MonitorManager,
        monitor: MonitorHandle | None = None,
        *,
        include_power: bool = False,
        include_input: bool = False,
    ) -> int:
        """Write all stored VCP values. Returns count of successfully set codes.

        Skips dangerous codes (factory resets), non-writable codes, and power-off codes.
        Power codes (0xD6, 0xC6) are skipped unless `include_power=True`; input source
        (0x60) is skipped unless `include_input=True`, because switching to a port with
        no signal blacks out the display.
        """
        count = 0
        # Color preset first: CVTE scalers lock the RGB gain DAC while a factory preset
        # is active, so 0x14 must reach User mode before 0x16/0x18/0x1A.
        items = sorted(self.settings.items(), key=lambda item: 0 if int(item[0], 16) == 0x14 else 1)
        for key, val in items:
            if val is None:
                continue
            code = int(key, 16)
            if VCPCatalog.is_dangerous(code):
                continue
            if not include_power and VCPCatalog.is_power_code(code):
                continue
            if VCPCatalog.is_power_off_value(code, val):
                continue
            if not include_input and code == 0x60:
                continue
            entry = VCPCatalog.get(code)
            if entry is not None and not entry.writeable:
                continue
            if manager.set_vcp(code, val, monitor):
                count += 1
        return count

    @property
    def filename(self) -> str:
        safe = self.name.translate(self.FILENAME_REPLACE)
        return f"{safe}.json"

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "description": self.description,
            "settings": self.settings,
            "experimental_mappings": self.experimental_mappings,
        }

    @classmethod
    def from_dict(cls, d: dict[str, Any]) -> Profile:
        return cls(
            name=d.get("name", "Unnamed"),
            description=d.get("description", ""),
            settings=d.get("settings", {}),
            experimental_mappings=d.get("experimental_mappings", {}),
        )


class ProfileStore:
    """Manage saved profiles on disk."""

    def __init__(self, directory: str | Path | None = None) -> None:
        self._dir = Path(directory) if directory else DEFAULT_PROFILES_DIR

    @property
    def directory(self) -> Path:
        return self._dir

    def ensure_dir(self) -> None:
        self._dir.mkdir(parents=True, exist_ok=True)

    def list(self) -> list[Path]:
        """Return sorted list of .json profile file paths."""
        if not self._dir.exists():
            return []
        return sorted(self._dir.glob("*.json"))

    def load(self, path: str | Path) -> Profile | None:
        """Load a single profile from disk."""
        p = Path(path)
        if not p.exists():
            return None
        with open(p, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        # Dual-setup files hold "monitors" instead of "settings"; they are applied
        # through apply-dual and must not show up as empty single-monitor profiles.
        if not isinstance(data, dict) or "settings" not in data:
            return None
        return Profile.from_dict(data)

    def load_all(self) -> list[Profile]:
        profiles: list[Profile] = []
        for fp in self.list():
            p = self.load(fp)
            if p is not None:
                profiles.append(p)
        return profiles

    def save(self, profile: Profile) -> Path:
        """Persist a profile to its filename. Returns the saved path."""
        self.ensure_dir()
        dest = self._dir / profile.filename
        with open(dest, "w", encoding="utf-8") as fh:
            json.dump(profile.to_dict(), fh, indent=2, ensure_ascii=False)
        return dest

    def delete(self, path: str | Path) -> bool:
        p = Path(path)
        if p.exists():
            p.unlink()
            return True
        return False


class ExperimentalMappingStore:
    """Persist user-confirmed OSD label to VCP code mappings."""

    def __init__(self, path: str | Path | None = None) -> None:
        self._path = Path(path) if path else DEFAULT_EXPERIMENTAL_MAPPINGS_PATH

    @property
    def path(self) -> Path:
        return self._path

    def load(self) -> dict[str, int]:
        if not self._path.exists():
            return {}
        with open(self._path, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        return {str(key): int(value) for key, value in data.items()}

    def save(self, mappings: dict[str, int]) -> Path:
        self._path.parent.mkdir(parents=True, exist_ok=True)
        with open(self._path, "w", encoding="utf-8") as fh:
            json.dump(mappings, fh, indent=2, sort_keys=True)
        return self._path

    def set(self, label: str, code: int) -> Path:
        mappings = self.load()
        mappings[label.strip()] = code
        return self.save(mappings)

    def save_mapping(self, code: int, label: str) -> Path:
        return self.set(label, code)
