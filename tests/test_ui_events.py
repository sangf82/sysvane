"""Tests for Textual event API assumptions."""

import asyncio
from pathlib import Path
from typing import cast

from src.app import ToushouApp
from src.core.monitor_manager import MonitorHandle, MonitorInfo, MonitorManager
from src.core.profiles import Profile, ProfileStore
from src.ui.screens import DashboardScreen, HelpScreen, NAV_CONTROLS, NAV_PROFILES, NAV_SCANNER
from src.ui.widgets import VcpControlRow, control_meter
from textual.widgets import ListView


def test_list_view_selected_uses_index_attribute() -> None:
    assert hasattr(ListView.Selected, "control")
    assert not hasattr(ListView.Selected, "item_index")


class FakeNative:
    pass


class FakeManager(MonitorManager):
    def __init__(self) -> None:
        super().__init__()
        info = MonitorInfo(index=0, display_name="AL258Q", model="CVTE", vcp_version="2.2", ddc_description="DP-1")
        capabilities = {"vcp": {0x10: {}, 0x12: {}, 0xAE: {}, 0xD6: {}, 0xE0: {}}, "inputs": [], "color_presets": []}
        self._monitor = MonitorHandle(info=info, _native=FakeNative(), capabilities=capabilities)  # type: ignore[arg-type]
        self._values = {0x10: 50, 0x12: 70, 0xAE: 32000, 0xD6: 1, 0xE0: 10}
        self._writes: list[tuple[int, int]] = []
        self.active_idx = 0

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


def test_control_meter_formats_numeric_cycle_and_read_only_states() -> None:
    assert control_meter("50", 0, 100) == "[#####-----]"
    assert control_meter("DisplayPort 1", 0, 1, [("DisplayPort 1", 15), ("HDMI 1", 17)]) == "cycle"
    assert control_meter("320 Hz", 0, 65535, read_only=True) == "info"


def test_dashboard_keyboard_adjusts_selected_control(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            await pilot.press("right")
            await pilot.pause(0.1)

            assert manager.writes == [(0x10, 51)]

    asyncio.run(run())


def test_dashboard_keyboard_moves_selection_before_adjusting(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            await pilot.press("down")
            await pilot.pause(0.05)
            await pilot.press("+")
            await pilot.pause(0.1)

            assert manager.writes == [(0x12, 71)]

    asyncio.run(run())


def test_profiles_applies_via_embedded_view(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        store = ProfileStore(tmp_path)
        store.save(Profile(name="Preset", settings={"0x10": 42}))

        app = ToushouApp(manager, store)
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            await pilot.press("f3")  # F3 goes to Profiles
            await pilot.pause(0.1)
            await pilot.press("a")
            await pilot.pause(0.1)

            assert (0x10, 42) in manager.writes

    asyncio.run(run())


def test_theme_toggle(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            assert app.theme == "toushou-harlequin"
            await pilot.press("t")
            await pilot.pause(0.1)
            assert app.theme == "toushou-gruvbox"
            await pilot.press("t")
            await pilot.pause(0.1)
            assert app.theme == "toushou-harlequin"

    asyncio.run(run())


def test_help_screen_modal(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            await pilot.press("question_mark")
            await pilot.pause(0.1)
            assert isinstance(app.screen, HelpScreen)
            await pilot.press("escape")
            await pilot.pause(0.1)
            assert not isinstance(app.screen, HelpScreen)

    asyncio.run(run())


def test_dashboard_keyboard_blocks_power_off_adjustment(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            screen = cast(DashboardScreen, app.screen)
            power_idx = next(
                (i for i, d in enumerate(screen._control_defs) if d.code == 0xD6),
                None,
            )
            assert power_idx is not None
            screen._selected_control_idx = power_idx
            await pilot.press("right")
            await pilot.pause(0.1)
            d6_writes = [w for w in manager.writes if w[0] == 0xD6]
            assert d6_writes == []

    asyncio.run(run())


def test_adjust_blocked_until_values_loaded(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            screen = cast(DashboardScreen, app.screen)
            screen._register_values_loaded = False
            await pilot.press("right")
            await pilot.pause(0.1)
            assert manager.writes == []

            screen._register_values_loaded = True
            screen._control_values.pop("0x10", None)
            await pilot.press("right")
            await pilot.pause(0.1)
            assert manager.writes == []

    asyncio.run(run())


def test_mouse_click_selects_row_and_step_button_adjusts(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            rows = list(app.screen.query(VcpControlRow))
            assert len(rows) >= 2

            # Click second row (Contrast 0x12)
            await pilot.click(rows[1])
            await pilot.pause(0.05)
            screen = cast(DashboardScreen, app.screen)
            assert screen._selected_control_idx == 1

            # Click step [+] on the second row
            inc_btn = rows[1].query_one(".inc")
            await pilot.click(inc_btn)
            await pilot.pause(0.1)
            assert (0x12, 71) in manager.writes

            # Click step [-] on the second row
            dec_btn = rows[1].query_one(".dec")
            await pilot.click(dec_btn)
            await pilot.pause(0.1)
            assert (0x12, 70) in manager.writes

    asyncio.run(run())


def test_tab_and_function_key_view_navigation(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            screen = cast(DashboardScreen, app.screen)
            assert screen._current_view == NAV_CONTROLS

            # Tab cycles to Scanner
            await pilot.press("tab")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_SCANNER

            # Tab cycles to Profiles
            await pilot.press("tab")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_PROFILES

            # Tab cycles back to Controls
            await pilot.press("tab")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_CONTROLS

            # F2 jumps to Scanner
            await pilot.press("f2")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_SCANNER

            # F1 jumps to Controls
            await pilot.press("f1")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_CONTROLS

            # F3 jumps to Profiles
            await pilot.press("f3")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_PROFILES

    asyncio.run(run())


def test_home_end_page_navigation(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            screen = cast(DashboardScreen, app.screen)
            total = len(screen._control_defs)
            assert total > 1

            # End jumps to last control
            await pilot.press("end")
            await pilot.pause(0.05)
            assert screen._selected_control_idx == total - 1

            # Home jumps to first control
            await pilot.press("home")
            await pilot.pause(0.05)
            assert screen._selected_control_idx == 0

    asyncio.run(run())


def test_scanner_snapshot_workflow(tmp_path: Path) -> None:
    async def run() -> None:
        manager = FakeManager()
        app = ToushouApp(manager, ProfileStore(tmp_path))
        async with app.run_test(size=(120, 36)) as pilot:
            await pilot.pause(0.2)
            screen = cast(DashboardScreen, app.screen)
            # Jump to Scanner
            await pilot.press("f2")
            await pilot.pause(0.05)
            assert screen._current_view == NAV_SCANNER

            # Take Snapshot 1
            await pilot.click("#btn-snap1")
            await pilot.pause(0.2)
            assert screen._scanner is not None
            assert screen._scanner.has_snap1 is True

            # Simulate hardware register change (OSD changed by user)
            manager._values[0xE0] = 99

            # Take Snapshot 2
            await pilot.click("#btn-snap2")
            await pilot.pause(0.2)
            assert screen._scanner.has_snap2 is True
            assert 0xE0 in screen._scanner_last_diff
            assert screen._scanner_last_diff[0xE0] == (10, 99)

    asyncio.run(run())
