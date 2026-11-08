"""Two-pass register diff engine for discovering manufacturer-specific VCP codes.

Workflow:
  1. User presses "Take Snapshot 1" -> reads all mfg registers (0xE0-0xFF).
  2. User changes a setting via physical OSD.
  3. User presses "Take Snapshot 2" -> reads again, computes delta.
  -> Changed registers are the discovered mappings.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .vcp_catalog import VCPCatalog


@dataclass
class Snapshot:
    """A single point-in-time register capture."""

    data: dict[int, int] = field(default_factory=dict)
    timestamp: str = ""


class Scanner:
    """Two-pass diff engine for mfg-specific VCP discovery."""

    def __init__(self, manager: Any) -> None:
        self._manager = manager
        self._snap1: Snapshot | None = None
        self._snap2: Snapshot | None = None

    # ------------------------------------------------------------------
    # snapshot operations
    # ------------------------------------------------------------------

    def take_snapshot1(self) -> Snapshot:
        self._snap1 = self._capture()
        self._snap2 = None
        return self._snap1

    def take_snapshot2(self) -> Snapshot:
        self._snap2 = self._capture()
        return self._snap2

    def take_snap1(self) -> Snapshot:
        """Alias for take_snapshot1."""
        return self.take_snapshot1()

    def take_snap2(self) -> Snapshot:
        """Alias for take_snapshot2."""
        return self.take_snapshot2()

    def reset(self) -> None:
        self._snap1 = None
        self._snap2 = None

    # ------------------------------------------------------------------
    # diff
    # ------------------------------------------------------------------

    def diff(self) -> dict[int, tuple[int | None, int | None]]:
        """Return {code: (before, after)} for registers that changed.

        Returns empty dict if snapshots aren't both captured.
        """
        if self._snap1 is None or self._snap2 is None:
            return {}
        result: dict[int, tuple[int | None, int | None]] = {}
        all_codes = set(self._snap1.data) | set(self._snap2.data)
        for code in all_codes:
            a = self._snap1.data.get(code)
            b = self._snap2.data.get(code)
            if a != b:
                result[code] = (a, b)
        return result

    @property
    def has_snap1(self) -> bool:
        return self._snap1 is not None

    @property
    def has_snap2(self) -> bool:
        return self._snap2 is not None

    @property
    def snap1(self) -> Snapshot | None:
        return self._snap1

    @property
    def snap2(self) -> Snapshot | None:
        return self._snap2

    # ------------------------------------------------------------------
    # helpers
    # ------------------------------------------------------------------

    def _capture(self) -> Snapshot:
        from datetime import datetime

        data: dict[int, int] = {}
        for code in VCPCatalog.mfg_codes():
            val = self._manager.get_vcp(code)
            if val is not None:
                data[code] = val
        return Snapshot(data=data, timestamp=datetime.now().strftime("%H:%M:%S"))
