"""Tests for monitor manager value helpers."""

from enum import Enum

from src.core.monitor_manager import MonitorManager


class FakeEnum(Enum):
    VALUE = 7


def test_coerce_vcp_tuple_uses_current_value() -> None:
    assert MonitorManager._coerce_value((53, 100)) == 53


def test_coerce_enum_uses_enum_value() -> None:
    assert MonitorManager._coerce_value(FakeEnum.VALUE) == 7


class FakeNative:
    def __init__(self, *, fail: bool = False) -> None:
        self.fail = fail
        self.vcp = self
        self.writes: list[tuple[int, int]] = []

    def __enter__(self) -> "FakeNative":
        return self

    def __exit__(self, *_args: object) -> None:
        return None

    def set_vcp_feature(self, code: int, value: int) -> None:
        if self.fail:
            raise RuntimeError("write failed")
        self.writes.append((code, value))


def test_write_returns_true_on_success() -> None:
    mgr = MonitorManager(auto_refresh=False)
    native = FakeNative()

    assert mgr._write(None, lambda _monitor: native.set_vcp_feature(0x10, 50)) is False


def test_set_vcp_reports_success_and_failure() -> None:
    from src.core.monitor_manager import MonitorHandle, MonitorInfo

    success_native = FakeNative()
    fail_native = FakeNative(fail=True)
    success_handle = MonitorHandle(MonitorInfo(index=0), success_native)  # type: ignore[arg-type]
    fail_handle = MonitorHandle(MonitorInfo(index=1), fail_native)  # type: ignore[arg-type]
    mgr = MonitorManager(auto_refresh=False)

    assert mgr.set_vcp(0x10, 55, success_handle) is True
    assert success_native.writes == [(0x10, 55)]
    assert mgr.set_vcp(0x10, 55, fail_handle) is False
