"""Async-safe, multi-monitor manager wrapping monitorcontrol.

Provides a single point of access for all connected DDC/CI monitors with
cached descriptor info and both high-level and low-level VCP read/write.
"""

from __future__ import annotations

import threading
from dataclasses import dataclass, field
from enum import Enum
from typing import Any

import monitorcontrol
from monitorcontrol import Monitor as _Monitor

from .edid import EdidIdentity, load_windows_edid_identities


@dataclass
class MonitorInfo:
    """Cached descriptor for a single physical monitor."""

    index: int
    display_name: str = ""
    model: str = ""
    serial: str = ""
    vcp_version: str = ""
    manufacturer_id: str = ""
    product_code: str = ""
    edid_manufacturer: str = ""
    edid_product_code: str = ""
    edid_serial: str = ""
    edid_instance: str = ""
    ddc_description: str = ""

    def __str__(self) -> str:
        return f"Display {self.index + 1}: {self.display_name or self.model or self.ddc_description or 'Generic Monitor'}"


@dataclass
class MonitorHandle:
    """Live handle to one monitor, with cached info."""

    info: MonitorInfo
    _native: _Monitor
    capabilities: dict[str, Any] = field(default_factory=dict)
    _lock: threading.Lock = field(default_factory=threading.Lock)


class MonitorManager:
    """Discovers and manages multiple DDC/CI monitors."""

    def __init__(self, auto_refresh: bool = False) -> None:
        self._monitors: list[MonitorHandle] = []
        self._active_idx: int = 0
        self._lock = threading.Lock()
        self._edid_identities: list[EdidIdentity] | None = None
        if auto_refresh:
            try:
                self.refresh()
            except Exception:
                pass

    # ------------------------------------------------------------------
    # discovery
    # ------------------------------------------------------------------

    def refresh(self, load_details: bool = True) -> list[MonitorHandle]:
        """(Re)enumerate all connected DDC/CI monitors."""
        raw_monitors = monitorcontrol.get_monitors()
        if self._edid_identities is None:
            try:
                self._edid_identities = load_windows_edid_identities()
            except Exception:
                self._edid_identities = []

        handles: list[MonitorHandle] = []
        for idx, m in enumerate(raw_monitors):
            edid_match = self._edid_identities[idx] if idx < len(self._edid_identities) else None
            info = MonitorInfo(
                index=idx,
                display_name=edid_match.name if edid_match else "",
                edid_manufacturer=edid_match.manufacturer if edid_match else "",
                edid_product_code=edid_match.product_code if edid_match else "",
                edid_serial=edid_match.serial if edid_match else "",
                edid_instance=edid_match.device_id if edid_match else "",
            )
            handle = MonitorHandle(info=info, _native=m)
            if load_details:
                self._probe_monitor(handle)
            handles.append(handle)

        with self._lock:
            self._monitors = handles
            if self._active_idx >= len(handles):
                self._active_idx = 0
        return handles

    def _probe_monitor(self, handle: MonitorHandle) -> None:
        try:
            with handle._native:
                try:
                    raw_caps = handle._native.get_vcp_capabilities()
                    caps: dict[str, Any] = {}
                    if isinstance(raw_caps, dict):
                        caps = dict(raw_caps)
                    elif isinstance(raw_caps, str):
                        parse_fn = getattr(monitorcontrol.vcp.vcp_codes, "parse_capabilities", None)
                        if callable(parse_fn):
                            parsed = parse_fn(raw_caps)
                            if isinstance(parsed, dict):
                                caps = dict(parsed)
                    handle.capabilities = caps
                    model = caps.get("model", "")
                    if model:
                        handle.info.model = str(model)
                    vcp_ver = caps.get("vcp_version", "") or caps.get("mccs_ver", "")
                    if vcp_ver:
                        handle.info.vcp_version = str(vcp_ver)
                    mfg = caps.get("mfg_id", "")
                    if mfg:
                        handle.info.manufacturer_id = str(mfg)
                    prod = caps.get("prod_id", "")
                    if prod:
                        handle.info.product_code = str(prod)
                except Exception:
                    pass

                if not handle.info.model:
                    try:
                        desc = getattr(handle._native, "description", "")
                        if desc:
                            handle.info.ddc_description = str(desc)
                    except Exception:
                        pass
        except Exception:
            pass

    # ------------------------------------------------------------------
    # active monitor access
    # ------------------------------------------------------------------

    @property
    def monitors(self) -> list[MonitorHandle]:
        with self._lock:
            return list(self._monitors)

    @property
    def active(self) -> MonitorHandle | None:
        with self._lock:
            if not self._monitors:
                return None
            return self._monitors[self._active_idx]

    @property
    def active_idx(self) -> int:
        return self._active_idx

    @active_idx.setter
    def active_idx(self, idx: int) -> None:
        with self._lock:
            if 0 <= idx < len(self._monitors):
                self._active_idx = idx

    def select_monitor(self, idx: int) -> None:
        self.active_idx = idx

    def get(self, idx: int) -> MonitorHandle | None:
        with self._lock:
            if 0 <= idx < len(self._monitors):
                return self._monitors[idx]
            return None

    # ------------------------------------------------------------------
    # high-level typed operations (passthrough to monitorcontrol)
    # ------------------------------------------------------------------

    def get_luminance(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_luminance())

    def set_luminance(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_luminance(value))

    def get_contrast(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_contrast())

    def set_contrast(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_contrast(value))

    def get_input_source(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_input_source())

    def set_input_source(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_input_source(value))

    def get_power_mode(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_power_mode())

    def set_power_mode(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_power_mode(value))

    def get_color_preset(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_color_preset())

    def set_color_preset(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_color_preset(value))

    def get_volume(self, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.get_volume())

    def set_volume(self, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.set_volume(value))

    # ------------------------------------------------------------------
    # low-level VCP read / write (arbitrary codes 0x00-0xFF)
    # ------------------------------------------------------------------

    def get_vcp(self, code: int, monitor: MonitorHandle | None = None) -> int | None:
        return self._with(monitor, lambda m: m.vcp.get_vcp_feature(code))

    def set_vcp(self, code: int, value: int, monitor: MonitorHandle | None = None) -> bool:
        return self._write(monitor, lambda m: m.vcp.set_vcp_feature(code, value))

    # ------------------------------------------------------------------
    # safety & status
    # ------------------------------------------------------------------

    @property
    def count(self) -> int:
        with self._lock:
            return len(self._monitors)

    @property
    def available(self) -> bool:
        with self._lock:
            return len(self._monitors) > 0

    # ------------------------------------------------------------------
    # internal helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _coerce_value(raw: Any) -> Any:
        if isinstance(raw, tuple):
            return raw[0]
        if isinstance(raw, Enum):
            return raw.value
        return raw

    def _resolve(self, monitor: MonitorHandle | None) -> MonitorHandle | None:
        if monitor is not None:
            return monitor
        return self.active

    def _with(self, monitor: MonitorHandle | None, fn: Any) -> Any:
        h = self._resolve(monitor)
        if h is None:
            return None
        with h._lock:
            try:
                with h._native:
                    raw = fn(h._native)
                    return MonitorManager._coerce_value(raw)
            except Exception:
                return None

    def _write(self, monitor: MonitorHandle | None, fn: Any) -> bool:
        h = self._resolve(monitor)
        if h is None:
            return False
        with h._lock:
            try:
                with h._native:
                    fn(h._native)
                    return True
            except Exception:
                return False
