"""Textual application entry point for Toushou Monitor TUI."""

from __future__ import annotations

from textual.app import App
from textual.binding import Binding
from textual.theme import Theme

from .core.monitor_manager import MonitorManager
from .core.profiles import ProfileStore
from .ui.screens import DashboardScreen


class ToushouApp(App[None]):
    """Main TUI application for multi-monitor DDC/CI control."""

    TITLE = "Toushou Monitor TUI"
    SUB_TITLE = "Multi-Monitor DDC/CI Controller"

    CSS = """
    Screen {
        background: $background;
        color: $foreground;
    }
    Header {
        background: $background;
        color: $foreground;
        border-bottom: round $primary;
    }
    MonitorList {
        width: 1fr;
        background: $background;
        color: $foreground;
        border: round $primary;
        border-title-color: $primary;
        padding: 0 1;
    }
    MonitorList ListView {
        height: 1fr;
        background: $background;
        color: $foreground;
    }
    SidebarNav {
        height: auto;
        background: $background;
        color: $foreground;
        border: round $primary;
        border-title-color: $primary;
        padding: 0 1;
    }
    SidebarNav ListView {
        height: auto;
        background: $background;
        color: $foreground;
    }
    ListView > ListItem {
        background: $background;
        color: $foreground;
    }
    ListView > ListItem.-highlight {
        background: $panel;
        color: $primary;
        text-style: bold;
    }
    ListView:focus > ListItem.-highlight {
        background: $secondary;
        color: $background;
        text-style: bold;
    }
    #monitor-list > ListItem.-highlight {
        background: $primary 20%;
        color: $primary;
        text-style: bold;
    }
    #monitor-list:focus > ListItem.-highlight {
        background: $primary;
        color: $background;
        text-style: bold;
    }
    MonitorDetail {
        width: 1fr;
        height: auto;
        background: $background;
        color: $foreground;
        border: round $primary;
        border-title-color: $primary;
        padding: 1;
    }
    #controls-panel {
        height: 1fr;
        overflow-y: auto;
    }
    .control-row {
        height: 1;
        padding: 0 1;
    }
    .control-label {
        width: 26;
    }
    .control-value {
        width: 16;
        content-align: left middle;
    }
    .control-code {
        width: 7;
        color: $text-muted;
    }
    .control-meter {
        width: 14;
        color: $secondary;
    }
    StatusBar {
        height: 1;
        background: $background;
        color: $secondary;
    }
    #status-text {
        background: $background;
        color: $secondary;
    }
    Button {
        margin: 0 1;
        background: $background;
        color: $primary;
        border: none;
    }
    Button:hover {
        background: $secondary;
        color: $background;
    }
    Button.-primary {
        background: $primary;
        color: $background;
    }
    Button.-success {
        background: $secondary;
        color: $background;
    }
    Button.-warning {
        background: $primary;
        color: $background;
    }
    Button.-error {
        background: $error;
        color: $background;
    }
    DataTable {
        background: $background;
        color: $foreground;
    }
    DataTable > .datatable--header {
        background: $background;
        color: $primary;
        text-style: bold;
    }
    DataTable > .datatable--even-row {
        background: $background;
    }
    DataTable > .datatable--cursor {
        background: $panel;
        color: $foreground;
    }
    DataTable:focus > .datatable--cursor {
        background: $secondary;
        color: $background;
    }
    """

    BINDINGS = [
        Binding("q", "quit", "Quit", show=True),
        Binding("r", "refresh", "Refresh", show=False),
        Binding("t", "toggle_theme", "Theme", show=False),
    ]

    THEME_NAMES = ["toushou-harlequin", "toushou-gruvbox"]

    def __init__(self, manager: MonitorManager, store: ProfileStore) -> None:
        super().__init__()
        self.register_theme(
            Theme(
                name="toushou-harlequin",
                primary="#FEFFAC",
                secondary="#45FFCA",
                warning="#FEFFAC",
                error="#FFB6D9",
                success="#45FFCA",
                accent="#D67BFF",
                foreground="#DDDDDD",
                background="#0C0C0C",
                surface="#0C0C0C",
                panel="#555555",
                dark=True,
            )
        )
        self.register_theme(
            Theme(
                name="toushou-gruvbox",
                primary="#fabd2f",
                secondary="#b8bb26",
                warning="#fe8019",
                error="#fb4934",
                success="#b8bb26",
                accent="#d3869b",
                foreground="#ebdbb2",
                background="#1d2021",
                surface="#1d2021",
                panel="#504945",
                dark=True,
            )
        )
        self.theme = "toushou-harlequin"
        self.mgr = manager
        self.store = store

    def on_mount(self) -> None:
        self.push_screen(DashboardScreen(self.mgr, self.store))

    def action_toggle_theme(self) -> None:
        names = self.THEME_NAMES
        idx = names.index(self.theme) if self.theme in names else -1
        self.theme = names[(idx + 1) % len(names)]
        self.notify(f"Theme: {self.theme}")


def main() -> None:
    """Main entry point for running the Toushou Monitor TUI application."""
    manager = MonitorManager(auto_refresh=False)
    store = ProfileStore()
    app = ToushouApp(manager, store)
    app.run()
