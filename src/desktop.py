"""Desktop GUI prototype: pywebview shell over the existing DDC/CI core.

The Python core stays the only hardware gateway; the window is plain HTML/CSS/JS
talking to :class:`DesktopBridge` through pywebview's JS API.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .core.controls import ControlDefinition, build_control_definitions, format_control_value
from .core.monitor_manager import MonitorHandle, MonitorManager
from .core.profiles import (
    DEFAULT_EXPERIMENTAL_MAPPINGS_PATH,
    ExperimentalMappingStore,
    ProfileStore,
)
from .core.vcp_catalog import VCPCatalog

UI_PATH = Path(__file__).resolve().parent / "ui" / "desktop.html"


class DesktopBridge:
    """JS-facing API. All monitor writes go through the same safety checks as the TUI."""

    def __init__(
        self,
        manager: MonitorManager | None = None,
        store: ProfileStore | None = None,
    ) -> None:
        self.mgr = manager or MonitorManager(auto_refresh=False)
        self.store = store or ProfileStore()
        self.mappings = ExperimentalMappingStore(DEFAULT_EXPERIMENTAL_MAPPINGS_PATH)
        self._defs: list[ControlDefinition] = []
        self._values: dict[str, int] = {}

    # ------------------------------------------------------------------
    # read side
    # ------------------------------------------------------------------

    def init(self) -> dict[str, Any]:
        self.mgr.refresh(load_details=True)
        return {"monitors": self._monitors(), "active": self.mgr.active_idx, **self.controls()}

    def refresh(self) -> dict[str, Any]:
        return self.init()

    def select_monitor(self, index: int) -> dict[str, Any]:
        try:
            index = int(index)
        except (TypeError, ValueError):
            return {"monitor": "", "controls": []}
        if 0 <= index < len(self.mgr.monitors):
            self.mgr.select_monitor(index)
        return self.controls()

    def controls(self) -> dict[str, Any]:
        handle = self.mgr.active
        self._defs = build_control_definitions(
            handle.capabilities if handle else {}, self.mappings.load()
        )
        self._values = {}
        if handle is not None:
            for definition in self._defs:
                if definition.code is None:
                    continue
                value = self.mgr.get_vcp(definition.code, handle)
                if value is not None:
                    self._values[definition.code_key] = value
        return {
            "monitor": str(handle.info) if handle is not None else "",
            "controls": [self._payload(definition) for definition in self._defs],
        }

    def _monitors(self) -> list[dict[str, Any]]:
        return [
            {
                "index": handle.info.index,
                "name": str(handle.info),
                "model": handle.info.model or handle.info.ddc_description or "-",
                "edid": " ".join(
                    part for part in (handle.info.edid_manufacturer, handle.info.edid_product_code) if part
                ) or "-",
            }
            for handle in self.mgr.monitors
        ]

    def _payload(self, definition: ControlDefinition) -> dict[str, Any]:
        value = self._values.get(definition.code_key)
        return {
            "key": definition.key,
            "label": definition.label,
            "category": definition.category,
            "code_key": definition.code_key,
            "value": value,
            "formatted": format_control_value(definition, value),
            "min": definition.min_val,
            "max": definition.max_val,
            "values": [{"label": label, "value": raw} for label, raw in definition.values],
            "read_only": definition.read_only,
            "experimental": definition.experimental,
            "dangerous": definition.dangerous,
            "writable": definition.is_writable and value is not None,
        }

    # ------------------------------------------------------------------
    # write side
    # ------------------------------------------------------------------

    def set_control(self, code_key: str, value: Any, confirm: bool = False) -> dict[str, Any]:
        definition = next((d for d in self._defs if d.code_key == code_key), None)
        if definition is None or definition.code is None:
            return {"ok": False, "message": "Unknown control."}
        if definition.read_only or not definition.is_writable:
            return {"ok": False, "message": f"{definition.label} is read-only."}
        try:
            value = int(value)
        except (TypeError, ValueError):
            return {"ok": False, "message": "Invalid value."}

        allowed = {raw for _, raw in definition.values}
        if allowed:
            if value not in allowed:
                return {"ok": False, "message": f"{value} is not a valid option."}
        elif not definition.min_val <= value <= definition.max_val:
            return {"ok": False, "message": f"Value must be {definition.min_val}-{definition.max_val}."}

        if VCPCatalog.is_power_off_value(definition.code, value):
            return {"ok": False, "message": "Blocked: that power state can black out the display."}
        if VCPCatalog.is_dangerous(definition.code):
            return {"ok": False, "message": "Blocked: factory-reset codes are disabled."}
        if definition.code == 0x60 and not confirm:
            return {
                "ok": False,
                "needsConfirm": True,
                "message": "Switching input can black out the display. Continue?",
            }
        if definition.experimental and not confirm:
            return {
                "ok": False,
                "needsConfirm": True,
                "message": f"{definition.label} is experimental. Write it anyway?",
            }

        handle = self.mgr.active
        if handle is None:
            return {"ok": False, "message": "No monitor selected."}
        if not self.mgr.set_vcp(definition.code, value, handle):
            return {"ok": False, "message": f"Write failed for {definition.label}."}

        self._values[definition.code_key] = value
        return {
            "ok": True,
            "message": f"{definition.label}: {format_control_value(definition, value)}",
            "value": value,
            "formatted": format_control_value(definition, value),
        }

    # ------------------------------------------------------------------
    # profiles
    # ------------------------------------------------------------------

    def profiles(self) -> list[dict[str, Any]]:
        result: list[dict[str, Any]] = []
        for path in self.store.list():
            profile = self.store.load(path)
            if profile is not None:
                result.append(
                    {
                        "name": profile.name,
                        "description": profile.description,
                        "count": len(profile.settings),
                    }
                )
        return result

    def apply_profile(self, name: str) -> dict[str, Any]:
        for path in self.store.list():
            profile = self.store.load(path)
            if profile is not None and profile.name.lower() == str(name).lower():
                count = profile.apply(self.mgr)
                return {
                    "ok": True,
                    "message": f"Applied '{profile.name}' ({count} codes).",
                    "controls": self.controls()["controls"],
                }
        return {"ok": False, "message": f"Profile '{name}' not found."}


def main() -> None:
    import webview

    bridge = DesktopBridge()
    webview.create_window(
        "Toushou Monitor",
        UI_PATH.as_uri(),
        js_api=bridge,
        width=1180,
        height=760,
        background_color="#0C0C0C",
    )
    webview.start()


if __name__ == "__main__":
    main()
