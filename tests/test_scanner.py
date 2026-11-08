"""Tests for scanner engine."""

from src.core.scanner import Scanner


class FakeMonitorManager:
    def __init__(self, values: dict[int, int | None] | None = None) -> None:
        self.values = values or {}

    def get_vcp(self, code: int) -> int | None:
        return self.values.get(code)


class TestScanner:
    def test_initial_state(self) -> None:
        mgr = FakeMonitorManager()
        s = Scanner(mgr)
        assert s.has_snap1 is False
        assert s.has_snap2 is False
        assert s.snap1 is None
        assert s.snap2 is None
        assert s.diff() == {}

    def test_snapshot_zero_monitors(self) -> None:
        """Scanner handles no-monitor scenario gracefully."""
        mgr = FakeMonitorManager()
        s = Scanner(mgr)
        # Without real monitors, takes empty snapshots
        snap = s.take_snapshot1()
        assert snap.data == {}
        assert s.has_snap1 is True

    def test_reset(self) -> None:
        mgr = FakeMonitorManager()
        s = Scanner(mgr)
        s.take_snapshot1()
        s.take_snapshot2()
        assert s.has_snap1 is True
        assert s.has_snap2 is True
        s.reset()
        assert s.has_snap1 is False
        assert s.has_snap2 is False
        assert s.snap1 is None
        assert s.snap2 is None

    def test_diff_empty_snapshots(self) -> None:
        mgr = FakeMonitorManager()
        s = Scanner(mgr)
        s.take_snapshot1()
        s.take_snapshot2()
        # Both empty => diff also empty
        diff = s.diff()
        assert diff == {}

    def test_diff_no_snap2(self) -> None:
        mgr = FakeMonitorManager()
        s = Scanner(mgr)
        s.take_snapshot1()
        assert s.diff() == {}
