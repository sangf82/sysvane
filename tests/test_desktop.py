"""Tests for the desktop bridge safety and payloads."""

from pathlib import Path

from src.core.controls import build_control_definitions
from src.core.monitor_manager import MonitorHandle, MonitorInfo, MonitorManager
from src.core.profiles import Profile, ProfileStore
from src.desktop import DesktopBridge


class FakeNative:
    pass


class FakeManager(MonitorManager):
    def __init__(self) -> None:
        super().__init__()
        info = MonitorInfo(index=0, display_name="AL258Q", model="CVTE")
        capabilities = {
            "vcp": {0x10: {}, 0x12: {}, 0x60: {}, 0xD6: {}},
            "inputs": [15, 16],
            "color_presets": [5],
        }
        self._monitor = MonitorHandle(info=info, _native=FakeNative(), capabilities=capabilities)  # type: ignore[arg-type]
        self._values = {0x10: 50, 0x12: 50, 0x60: 15, 0xD6: 1}
        self._writes: list[tuple[int, int]] = []

    @property
    def monitors(self) -> list[MonitorHandle]:
        return [self._monitor]

    @property
    def active(self) -> MonitorHandle:
        return self._monitor

    @property
    def available(self) -> bool:
        return True

    @property
    def writes(self) -> list[tuple[int, int]]:
        return list(self._writes)

    def refresh(self, load_details: bool = True) -> list[MonitorHandle]:
        return [self._monitor]

    def get_vcp(self, code: int, monitor: MonitorHandle | None = None) -> int | None:
        return self._values.get(code)

    def set_vcp(self, code: int, value: int, monitor: MonitorHandle | None = None) -> bool:
        self._values[code] = value
        self._writes.append((code, value))
        return True


def make_bridge(tmp_path: Path) -> tuple[DesktopBridge, FakeManager]:
    manager = FakeManager()
    bridge = DesktopBridge(manager=manager, store=ProfileStore(tmp_path))
    bridge._defs = build_control_definitions(manager._monitor.capabilities)
    bridge._values = dict(manager._values)
    return bridge, manager


def test_controls_payload_includes_values(tmp_path: Path) -> None:
    bridge, _ = make_bridge(tmp_path)
    payload = bridge.controls()

    brightness = next(control for control in payload["controls"] if control["code_key"] == "0x10")
    assert brightness["value"] == 50
    assert brightness["writable"] is True
    assert payload["monitor"] == "Display 1: AL258Q"


def test_set_control_writes_safe_value(tmp_path: Path) -> None:
    bridge, manager = make_bridge(tmp_path)

    result = bridge.set_control("0x10", 60)

    assert result["ok"] is True
    assert manager.writes == [(0x10, 60)]


def test_set_control_blocks_power_off(tmp_path: Path) -> None:
    bridge, manager = make_bridge(tmp_path)

    result = bridge.set_control("0xD6", 4)

    assert result["ok"] is False
    assert "black out" in result["message"]
    assert manager.writes == []


def test_set_control_requires_confirmation_for_input(tmp_path: Path) -> None:
    bridge, manager = make_bridge(tmp_path)

    result = bridge.set_control("0x60", 16)

    assert result["needsConfirm"] is True
    assert manager.writes == []

    result = bridge.set_control("0x60", 16, confirm=True)
    assert result["ok"] is True
    assert manager.writes == [(0x60, 16)]


def test_apply_profile_skips_input_source(tmp_path: Path) -> None:
    manager = FakeManager()
    store = ProfileStore(tmp_path)
    store.save(Profile(name="Calibrated", settings={"0x10": 70, "0x60": 16}))
    bridge = DesktopBridge(manager=manager, store=store)
    bridge.controls()

    result = bridge.apply_profile("Calibrated")

    assert result["ok"] is True
    assert manager.writes == [(0x10, 70)]
