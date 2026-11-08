"""Custom Textual widgets for the monitor TUI."""

from __future__ import annotations

from typing import Any

from textual import events
from textual.app import ComposeResult
from textual.containers import Container, Horizontal
from textual.message import Message
from textual.widgets import (
    DataTable,
    Label,
    ListItem,
    ListView,
    Static,
)

CONTROL_MARKER_CLASS = "control-marker"

NAV_CONTROLS = "controls"
NAV_SCANNER = "scanner"
NAV_PROFILES = "profiles"


def control_meter(
    value: str,
    min_val: int,
    max_val: int,
    discrete_values: list[tuple[str, int]] | None = None,
    *,
    read_only: bool = False,
) -> str:
    """Return the compact row state/meter shown beside a VCP value."""
    if read_only:
        return "info"
    if discrete_values:
        return "cycle"
    if value in {"-", "Unmapped"} or max_val <= min_val:
        return "pending"
    try:
        numeric_value = int(str(value).split()[0])
    except ValueError:
        return "manual"
    ratio = max(0.0, min(1.0, (numeric_value - min_val) / (max_val - min_val)))
    filled = round(ratio * 10)
    return "[" + ("#" * filled).ljust(10, "-") + "]"


class MonitorList(Static):
    """Sidebar: list of detected monitors with active highlight."""

    BORDER_TITLE = "Monitors"

    def compose(self) -> ComposeResult:
        yield ListView(id="monitor-list")

    def populate(self, labels: list[str], active_idx: int = 0) -> None:
        lv = self.query_one("#monitor-list", ListView)
        lv.clear()
        for i, label in enumerate(labels):
            item = ListItem(Label(label))
            lv.append(item)
        if 0 <= active_idx < len(labels):
            lv.index = active_idx


class SidebarNav(Static):
    """Sidebar navigation block for switching views."""

    BORDER_TITLE = "Navigation"

    def compose(self) -> ComposeResult:
        yield ListView(id="nav-list")

    def on_mount(self) -> None:
        lv = self.query_one("#nav-list", ListView)
        lv.append(ListItem(Label("[1] Controls"), name=NAV_CONTROLS))
        lv.append(ListItem(Label("[2] Scanner"), name=NAV_SCANNER))
        lv.append(ListItem(Label("[3] Profiles"), name=NAV_PROFILES))


class MonitorDetail(Static):
    """Display model, serial, VCP version for the active monitor."""

    BORDER_TITLE = "Details"

    def compose(self) -> ComposeResult:
        yield Static(id="detail-content")

    def update(self, content: Any = "", *, layout: bool = True) -> None:
        try:
            self.query_one("#detail-content", Static).update(content, layout=layout)
        except Exception:
            super().update(content, layout=layout)


class VcpControlRow(Static):
    """A single named control row with responsive value and step buttons."""

    class Clicked(Message):
        """Posted when a control row is clicked with the mouse."""

        def __init__(self, row: VcpControlRow) -> None:
            super().__init__()
            self.row = row

    class Adjusted(Message):
        """Posted when a control value is adjusted via mouse click or scroll."""

        def __init__(self, row: VcpControlRow, delta: int) -> None:
            super().__init__()
            self.row = row
            self.delta = delta

    def __init__(
        self,
        label: str,
        code: str,
        min_val: int,
        max_val: int,
        value: str,
        help_text: str,
        widget_key: str | None = None,
        discrete_values: list[tuple[str, int]] | None = None,
        *,
        read_only: bool = False,
        experimental: bool = False,
        enabled: bool = True,
        selected: bool = False,
    ) -> None:
        classes = "control-item selected" if selected else "control-item"
        super().__init__(classes=classes)
        self._label = label
        self._code = code
        self._widget_key = widget_key or code
        self._min = min_val
        self._max = max_val
        self._value = value
        self._help_text = help_text
        self._discrete = discrete_values
        self._read_only = read_only
        self._experimental = experimental
        self._enabled = enabled
        self._selected = selected

    def _meter(self) -> str:
        return control_meter(self._value, self._min, self._max, self._discrete, read_only=self._read_only)

    def compose(self) -> ComposeResult:
        with Horizontal(classes="control-row"):
            marker = ">" if self._selected else " "
            yield Label(marker, classes=CONTROL_MARKER_CLASS)
            yield Label(self._label, classes="control-label")
            yield Label(self._code, classes="control-code")
            yield Label(str(self._value), classes="control-value", id=f"val-{self._widget_key}")
            yield Label(self._meter(), classes="control-meter", id=f"meter-{self._widget_key}")
            if not self._read_only and self._enabled:
                yield Label("[-]", classes="row-step-btn dec", id=f"dec-{self._widget_key}")
                yield Label("[+]", classes="row-step-btn inc", id=f"inc-{self._widget_key}")
            if self._experimental:
                state_cls = "control-state state-exp"
            elif self._read_only:
                state_cls = "control-state state-readonly"
            elif not self._enabled:
                state_cls = "control-state state-scan"
            else:
                state_cls = "control-state state-writable"
            state_text = "EXP" if self._experimental else (
                "read only" if self._read_only else (
                    "scan first" if not self._enabled else "writable"
                )
            )
            yield Label(state_text, classes=state_cls)

    def on_click(self, event: events.Click) -> None:
        event.stop()
        widget_id = getattr(event.widget, "id", "") or ""
        widget_classes = getattr(event.widget, "classes", set()) or set()
        if widget_id.startswith("dec-") or "dec" in widget_classes:
            self.post_message(self.Adjusted(self, -1))
            return
        if widget_id.startswith("inc-") or "inc" in widget_classes:
            self.post_message(self.Adjusted(self, 1))
            return
        self.post_message(self.Clicked(self))

    def set_selected(self, selected: bool) -> None:
        if self._selected == selected:
            return
        self._selected = selected
        self.set_class(selected, "selected")
        self.query_one(f".{CONTROL_MARKER_CLASS}", Label).update(">" if selected else " ")


class ControlsPanel(Container):
    """Scrollable container that holds VcpControlRow widgets."""

    BORDER_TITLE = "Controls"


class RegisterTable(DataTable[str]):
    """DataTable pre-configured for VCP register display."""

    def on_mount(self) -> None:
        self.cursor_type = "row"
        self.zebra_stripes = True


class StatusBar(Static):
    """Bottom bar with keybinding hints."""

    def compose(self) -> ComposeResult:
        yield Static(id="status-text")

    def on_mount(self) -> None:
        self.query_one("#status-text", Static).update(
            "[bold]Tab[/] switch view  [bold]↑/↓[/] select  [bold]←/→[/] adjust  "
            "[bold]Click / [-][+][/] mouse  [bold]?[/] help  [bold]q[/] quit"
        )
