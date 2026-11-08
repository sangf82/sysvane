"""Screen controllers for Toushou TUI."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal, Vertical, VerticalScroll
from textual.screen import Screen
from textual.widgets import (
    Button,
    ContentSwitcher,
    DataTable,
    Footer,
    Header,
    Input,
    Label,
    ListItem,
    ListView,
    LoadingIndicator,
    Static,
)

from ..core.controls import (
    ControlDefinition,
    build_control_definitions,
    format_control_value,
)
from ..core.monitor_manager import MonitorHandle, MonitorManager
from ..core.profiles import (
    DEFAULT_EXPERIMENTAL_MAPPINGS_PATH,
    ExperimentalMappingStore,
    Profile,
    ProfileStore,
)
from ..core.scanner import Scanner
from ..core.vcp_catalog import VCPCatalog
from .widgets import (
    ControlsPanel,
    MonitorDetail,
    MonitorList,
    RegisterTable,
    SidebarNav,
    StatusBar,
    VcpControlRow,
    control_meter,
)

if TYPE_CHECKING:
    pass

NAV_CONTROLS = "controls"
NAV_SCANNER = "scanner"
NAV_PROFILES = "profiles"


class DashboardScreen(Screen[None]):
    """Main screen with sidebar navigation and switchable content area."""

    BINDINGS = [
        ("q", "quit", "Quit"),
        ("question_mark", "show_help", "Help"),
        ("t", "toggle_theme", "Theme"),
        ("tab", "next_view", "Next Tab"),
        ("shift+tab", "prev_view", "Prev Tab"),
        ("f1", "go_controls", "Controls"),
        ("f2", "go_scanner", "Scanner"),
        ("f3", "go_profiles", "Profiles"),
        Binding("f4", "go_profiles", "Profiles", show=False),
        ("p", "quick_apply", "Apply Profile"),
        ("r", "refresh", "Refresh"),
        Binding("down", "select_next_control", "Next", priority=True),
        Binding("up", "select_previous_control", "Previous", priority=True),
        Binding("right", "increase_selected_control", "Increase", priority=True),
        Binding("left", "decrease_selected_control", "Decrease", priority=True),
        ("+", "increase_selected_control", "Increase"),
        ("-", "decrease_selected_control", "Decrease"),
        Binding("j", "select_next_control", "Next", show=False),
        Binding("k", "select_previous_control", "Previous", show=False),
        Binding("l", "increase_selected_control", "Increase", show=False),
        Binding("h", "decrease_selected_control", "Decrease", show=False),
        Binding("pageup", "page_up_control", "Page Up", show=False),
        Binding("pagedown", "page_down_control", "Page Down", show=False),
        Binding("home", "first_control", "Top", show=False),
        Binding("end", "last_control", "Bottom", show=False),
        Binding("enter", "confirm_action", "Confirm / Apply", show=False),
        ("1", "scanner_snap1", "Snap 1"),
        ("2", "scanner_snap2", "Snap 2"),
        ("a", "apply_selected_profile", "Apply"),
        ("n", "create_new_profile", "New"),
        ("d", "delete_selected_profile", "Delete"),
    ]

    CSS = """
    DashboardScreen { layout: vertical; }
    #dashboard-root { width: 100%; height: 100%; }
    #main-area { width: 100%; height: 1fr; layout: horizontal; }
    #sidebar { width: 32; height: 1fr; background: $background; }
    #content { width: 1fr; height: 1fr; background: $background; }
    MonitorList { height: 1fr; width: 1fr; }
    MonitorDetail { height: 9; width: 1fr; }
    SidebarNav { height: auto; width: 1fr; }
    SidebarNav ListItem:hover { background: $surface; }
    MonitorList ListItem:hover { background: $surface; }
    #top-status {
        height: 4; padding: 0 1;
        background: $background; color: $foreground;
        border: round $primary; border-title-color: $primary;
    }
    #view-controls {
        height: 1fr; overflow-y: auto;
        background: $background;
        border: round $primary; border-title-color: $primary;
        padding: 0 1;
    }
    #controls-help {
        height: 1; padding: 0 1;
        color: $secondary; background: $background;
    }
    .category-title {
        height: 1; margin-top: 1; padding: 0 1;
        background: $background; color: $primary; text-style: bold;
    }
    .control-row {
        height: 1; padding: 0 1;
        align: center middle;
        background: $background; color: $foreground;
    }
    .control-item { height: 1; }
    .control-item:hover { background: $surface; }
    .control-item.selected { background: $panel; }
    .control-item.selected:hover { background: $panel-lighten-1; }
    .control-item.selected .control-label { text-style: bold; color: $primary; }
    .control-marker { width: 2; color: $primary; text-style: bold; }
    .control-label { width: 24; }
    .control-value { width: 14; content-align: left middle; }
    .control-code { width: 6; color: $text-muted; }
    .control-meter { width: 13; color: $primary; content-align: right middle; }
    .row-step-btn {
        width: 3; color: $primary; text-style: bold; margin-left: 1;
    }
    .row-step-btn:hover {
        color: $accent; background: $surface-lighten-1;
    }

    #view-scanner {
        height: 1fr; padding: 0 1;
        background: $background;
        border: round $primary; border-title-color: $primary;
    }
    #scanner-actions {
        height: 3; align: left middle; margin-bottom: 1;
    }
    #scanner-actions Button { margin-right: 1; }
    #mapping-label { width: 24; margin-right: 1; }
    #scanner-status { width: 1fr; text-align: right; color: $secondary; }
    #scanner-loading { width: 4; display: none; }
    #scanner-loading.active { display: block; }
    #scanner-runbar {
        height: 1; color: $text-muted; margin-bottom: 1;
    }
    #scanner-diff { height: 1fr; }

    #view-profiles {
        height: 1fr; padding: 0 1;
        background: $background;
        border: round $primary; border-title-color: $primary;
    }
    #profiles-title { height: 1; color: $primary; margin-bottom: 1; }
    #profiles-table-container { height: 1fr; }
    #profiles-table { height: 1fr; }
    #profiles-actions {
        height: 3; align: left middle; margin-top: 1;
    }
    #profiles-actions Button { margin-right: 1; }
    """

    def __init__(self, manager: MonitorManager, store: ProfileStore) -> None:
        super().__init__()
        self.mgr = manager
        self.store = store
        self.mapping_store = ExperimentalMappingStore(DEFAULT_EXPERIMENTAL_MAPPINGS_PATH)
        self._current_view = NAV_CONTROLS
        self._selected_control_idx = 0
        self._control_defs: list[ControlDefinition] = []
        self._control_values: dict[str, int] = {}
        self._confirmed_experimental_codes: set[int] = set()
        self._confirmed_dangerous_codes: set[int] = set()
        self._register_values_loaded = False
        self._scanner: Scanner | None = None
        self._scanner_busy = False
        self._scanner_last_diff: dict[int, tuple[int | None, int | None]] = {}
        self._profiles_list: list[Profile] = []
        self._pending_delete: str | None = None

    # ------------------------------------------------------------------
    # layout
    # ------------------------------------------------------------------

    def compose(self) -> ComposeResult:
        with Vertical(id="dashboard-root"):
            with Horizontal(id="main-area"):
                with Vertical(id="sidebar"):
                    yield MonitorList()
                    yield MonitorDetail()
                    yield SidebarNav()
                with Vertical(id="content"):
                    yield Static(id="top-status")
                    with ContentSwitcher(initial="view-controls", id="content-switcher"):
                        yield ControlsPanel(id="view-controls")
                        with Vertical(id="view-scanner"):
                            with Horizontal(id="scanner-actions"):
                                yield Button("Snapshot 1", id="btn-snap1", variant="primary")
                                yield Button("Snapshot 2", id="btn-snap2", variant="warning")
                                yield Button("Reset", id="btn-reset", variant="error")
                                yield Input(placeholder="OSD label", id="mapping-label")
                                yield Button("Confirm", id="btn-confirm-map", variant="success")
                                yield LoadingIndicator(id="scanner-loading")
                                yield Static(id="scanner-status")
                            yield Static("Ready. Take Snapshot 1 before changing the physical monitor OSD.", id="scanner-runbar")
                            yield RegisterTable(id="scanner-diff")
                        with Vertical(id="view-profiles"):
                            yield Static("[bold]Profiles[/]  Apply, capture, or delete monitor presets.", id="profiles-title")
                            with VerticalScroll(id="profiles-table-container"):
                                yield DataTable(id="profiles-table")
                            with Horizontal(id="profiles-actions"):
                                yield Button("Apply Selected", id="btn-apply", variant="primary")
                                yield Button("New Profile", id="btn-new", variant="success")
                                yield Button("Delete Selected", id="btn-delete", variant="error")
                    yield Static(id="controls-help")
            yield StatusBar()

    # ------------------------------------------------------------------
    # lifecycle
    # ------------------------------------------------------------------

    def on_mount(self) -> None:
        self._show_startup_state()
        self.app.run_worker(
            self._discover_monitors,
            name="monitor-discovery", group="hardware", thread=True, exclusive=True,
        )

    def _show_startup_state(self) -> None:
        self.query_one(MonitorList).populate(["Detecting monitors..."], 0)
        self.query_one(MonitorDetail).update("[dim]Starting DDC/CI scan...[/dim]")

    def _discover_monitors(self) -> None:
        self.mgr.refresh(load_details=False)
        self.app.call_from_thread(self._finish_monitor_discovery)
        self.mgr.refresh(load_details=True)
        defs = self._definitions_for_active_monitor()
        vals = self._read_control_values_for(defs)
        scanner = Scanner(self.mgr)
        self.app.call_from_thread(self._finish_hardware_load, defs, vals, scanner)

    def _finish_monitor_discovery(self) -> None:
        self._refresh_controls_view()

    def _finish_hardware_load(
        self,
        defs: list[ControlDefinition],
        vals: dict[str, int],
        scanner: Scanner,
    ) -> None:
        self._control_defs = defs
        self._control_values = vals
        self._scanner = scanner
        self._register_values_loaded = True
        self._refresh_controls_view()

    # ------------------------------------------------------------------
    # navigation & mouse interaction
    # ------------------------------------------------------------------

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        lv_id = event.list_view.id
        if lv_id == "monitor-list":
            self._on_monitor_selected(event)
        elif lv_id == "nav-list":
            self._on_nav_selected(event)

    def _on_nav_selected(self, event: ListView.Selected) -> None:
        name = getattr(event.item, "name", None) or NAV_CONTROLS
        self._switch_view(name)

    def _switch_view(self, view: str) -> None:
        self._current_view = view
        switcher = self.query_one("#content-switcher", ContentSwitcher)
        if view == NAV_CONTROLS:
            switcher.current = "view-controls"
            self._rebuild_controls()
            self._update_control_help()
        elif view == NAV_SCANNER:
            switcher.current = "view-scanner"
            if self._scanner is None:
                self._scanner = Scanner(self.mgr)
            self._update_scanner_status()
            self.query_one("#controls-help", Static).update(
                "[bold]Scanner[/]  [bold]1[/] snap baseline  [bold]2[/] snap changed  "
                "[bold]r[/] reset"
            )
        elif view == NAV_PROFILES:
            switcher.current = "view-profiles"
            self._refresh_profiles_table()
            self.query_one("#controls-help", Static).update(
                "[bold]Profiles[/]  [bold]a[/] / [bold]Enter[/] apply  [bold]n[/] new  [bold]d[/] delete"
            )

        try:
            nav_lv = self.query_one("#nav-list", ListView)
            view_to_idx = {NAV_CONTROLS: 0, NAV_SCANNER: 1, NAV_PROFILES: 2}
            target_idx = view_to_idx.get(view, 0)
            if nav_lv.index != target_idx:
                nav_lv.index = target_idx
        except Exception:
            pass

    def action_go_controls(self) -> None:
        self._switch_view(NAV_CONTROLS)

    def action_go_scanner(self) -> None:
        self._switch_view(NAV_SCANNER)

    def action_go_profiles(self) -> None:
        self._switch_view(NAV_PROFILES)

    def action_next_view(self) -> None:
        views = [NAV_CONTROLS, NAV_SCANNER, NAV_PROFILES]
        cur = self._current_view if self._current_view in views else NAV_CONTROLS
        idx = (views.index(cur) + 1) % len(views)
        self._switch_view(views[idx])

    def action_prev_view(self) -> None:
        views = [NAV_CONTROLS, NAV_SCANNER, NAV_PROFILES]
        cur = self._current_view if self._current_view in views else NAV_CONTROLS
        idx = (views.index(cur) - 1) % len(views)
        self._switch_view(views[idx])

    def action_confirm_action(self) -> None:
        if self._current_view == NAV_CONTROLS:
            defn = self._selected_control()
            if defn and defn.values:
                self._adjust_control(defn.code_key, 1)
        elif self._current_view == NAV_PROFILES:
            self.action_apply_selected_profile()

    def action_show_help(self) -> None:
        self.app.push_screen(HelpScreen())

    def action_toggle_theme(self) -> None:
        toggle = getattr(self.app, 'action_toggle_theme', None)
        if callable(toggle):
            toggle()

    # ------------------------------------------------------------------
    # monitor selection
    # ------------------------------------------------------------------

    def _on_monitor_selected(self, event: ListView.Selected) -> None:
        if event.item is None or not self.mgr.available or event.index is None:
            return
        if event.index == self.mgr.active_idx:
            return
        self.mgr.active_idx = event.index
        self._update_detail()
        self._update_top_status()
        self._rebuild_monitor_list()
        self._control_defs = self._definitions_for_active_monitor()
        self._control_values = {}
        self._register_values_loaded = False
        self._refresh_controls_view()
        self.app.run_worker(
            self._load_active_monitor_controls,
            name="control-refresh", group="hardware", thread=True, exclusive=True,
        )

    def _load_active_monitor_controls(self) -> None:
        defs = self._definitions_for_active_monitor()
        vals = self._read_control_values_for(defs)
        scanner = Scanner(self.mgr)
        self.app.call_from_thread(self._finish_hardware_load, defs, vals, scanner)

    # ------------------------------------------------------------------
    # controls view
    # ------------------------------------------------------------------

    def _definitions_for_active_monitor(self) -> list[ControlDefinition]:
        h = self.mgr.active
        return build_control_definitions(h.capabilities if h else {}, self.mapping_store.load())

    def _read_control_values_for(self, defs: list[ControlDefinition]) -> dict[str, int]:
        values: dict[str, int] = {}
        h = self.mgr.active
        if h is None:
            return values
        for definition in defs:
            if definition.code is None:
                continue
            val = self.mgr.get_vcp(definition.code, h)
            if val is not None:
                values[definition.code_key] = val
        return values

    def _read_control_values(self) -> dict[str, int]:
        return self._read_control_values_for(self._control_defs)

    def _refresh_controls_view(self) -> None:
        self._rebuild_monitor_list()
        self._update_detail()
        self._update_top_status()
        if self._current_view == NAV_CONTROLS:
            self._rebuild_controls()
            self._update_control_help()

    def _rebuild_monitor_list(self) -> None:
        labels = [str(m.info) for m in self.mgr.monitors] if self.mgr.available else ["No monitors detected"]
        self.query_one(MonitorList).populate(labels, self.mgr.active_idx)

    def _update_detail(self) -> None:
        h = self.mgr.active
        detail = self.query_one(MonitorDetail)
        if h is None:
            detail.update("[dim]No monitor selected[/dim]")
            return
        info = h.info
        lines = [
            f"[b]Name:[/b] {info.display_name or 'N/A'}",
            f"[b]EDID:[/b] {info.edid_manufacturer or 'N/A'} {info.edid_product_code or ''}",
            f"[b]DDC:[/b] {info.model or 'N/A'}",
            f"[b]VCP:[/b] {info.vcp_version or 'N/A'}",
        ]
        detail.update("\n".join(lines))

    def _update_top_status(self) -> None:
        def value(code: int) -> str:
            key = f"0x{code:02X}"
            defn = next((d for d in self._control_defs if d.code == code), None)
            return format_control_value(defn, self._control_values.get(key)) if defn else "-"

        chips = [
            f"[b]Signal[/b] {value(0xAE)}",
            f"[b]Brightness[/b] {value(0x10)}",
            f"[b]Input[/b] {value(0x60)}",
        ]
        h = self.mgr.active
        title = str(h.info) if h is not None else "No monitor selected"
        status = self.query_one("#top-status", Static)
        status.border_title = title
        status.update("   ".join(chips))

    def _rebuild_controls(self) -> None:
        panel = self.query_one("#view-controls", ControlsPanel)
        panel.remove_children()

        if not self._control_defs:
            panel.mount(Label("[dim]Loading OSD controls...[/dim]"))
            return

        self._selected_control_idx = max(0, min(self._selected_control_idx, len(self._control_defs) - 1))
        category = ""
        for index, defn in enumerate(self._control_defs):
            if defn.category != category:
                category = defn.category
                panel.mount(Static(f"[b]{category}[/b]", classes="category-title"))
            val_formatted = format_control_value(defn, self._control_values.get(defn.code_key))
            panel.mount(
                VcpControlRow(
                    defn.label, defn.code_key, defn.min_val, defn.max_val,
                    val_formatted, defn.help_text, widget_key=defn.key,
                    discrete_values=list(defn.values),
                    read_only=defn.read_only, experimental=defn.experimental,
                    enabled=defn.is_writable,
                    selected=index == self._selected_control_idx,
                )
            )
        self._update_control_help()

    def _update_control_help(self) -> None:
        defn = self._selected_control()
        if defn is None:
            self.query_one("#controls-help", Static).update("[dim]No selected control.[/dim]")
            return
        code = defn.code_key if defn.code is not None else "----"
        val_formatted = format_control_value(defn, self._control_values.get(defn.code_key))
        if defn.experimental:
            state = "[bold]EXP[/] first adjustment arms this candidate before writing."
        elif defn.read_only:
            state = "[dim]read only[/dim]"
        elif not defn.is_writable:
            state = "scan first with Scanner"
        else:
            state = "writable with arrows, -/+, or click [-][+]"
        self.query_one("#controls-help", Static).update(
            f"[bold]{defn.label}[/] {code}  value {val_formatted}  {state}  {defn.help_text}"
        )

    # ------------------------------------------------------------------
    # control adjustment & mouse events
    # ------------------------------------------------------------------

    def _definition_for_code_key(self, code_key: str) -> ControlDefinition | None:
        return next((d for d in self._control_defs if d.code_key == code_key), None)

    def on_vcp_control_row_clicked(self, event: VcpControlRow.Clicked) -> None:
        rows = list(self.query(VcpControlRow))
        if event.row in rows:
            new_idx = rows.index(event.row)
            if new_idx != self._selected_control_idx:
                if 0 <= self._selected_control_idx < len(rows):
                    rows[self._selected_control_idx].set_selected(False)
                self._selected_control_idx = new_idx
                event.row.set_selected(True)
                event.row.scroll_visible(animate=False)
                self._update_control_help()

    def on_vcp_control_row_adjusted(self, event: VcpControlRow.Adjusted) -> None:
        rows = list(self.query(VcpControlRow))
        if event.row in rows:
            new_idx = rows.index(event.row)
            if new_idx != self._selected_control_idx:
                if 0 <= self._selected_control_idx < len(rows):
                    rows[self._selected_control_idx].set_selected(False)
                self._selected_control_idx = new_idx
                event.row.set_selected(True)
                event.row.scroll_visible(animate=False)
                self._update_control_help()
            defn = self._selected_control()
            if defn is not None:
                self._adjust_control(defn.code_key, event.delta)

    def on_button_pressed(self, event: Button.Pressed) -> None:
        btn_id = str(event.button.id or "")
        # Scanner buttons
        if btn_id == "btn-snap1":
            self._scanner_start_snapshot("snap1")
        elif btn_id == "btn-snap2":
            self._scanner_start_snapshot("snap2")
        elif btn_id == "btn-reset":
            self._scanner_reset()
        elif btn_id == "btn-confirm-map":
            self._scanner_confirm_mapping()
        # Profile buttons
        elif btn_id == "btn-apply":
            self.action_apply_selected_profile()
        elif btn_id == "btn-new":
            self.action_create_new_profile()
        elif btn_id == "btn-delete":
            self.action_delete_selected_profile()
        # Control +/- buttons
        elif btn_id.startswith("inc-") or btn_id.startswith("dec-"):
            code_key = btn_id.split("-", 1)[-1]
            delta = 1 if btn_id.startswith("inc-") else -1
            self._adjust_control(code_key, delta)

    def _adjust_control(self, code_key: str, delta: int) -> None:
        h = self.mgr.active
        if h is None:
            return
        defn = self._definition_for_code_key(code_key)
        if defn is None or defn.code is None or defn.read_only:
            if defn is not None:
                self.notify(f"{defn.label} cannot be adjusted.", severity="warning")
            return
        if defn.experimental and defn.code not in self._confirmed_experimental_codes:
            self._confirmed_experimental_codes.add(defn.code)
            self.notify(f"{defn.label} is experimental. Press again to write.", severity="warning")
            return

        # Never guess a starting value: writing before the first read would set
        # brightness to 0 (black screen) on a freshly opened or switched monitor.
        if not self._register_values_loaded:
            self.notify("Waiting for current values. Try again in a moment.", severity="warning")
            return
        if code_key not in self._control_values:
            self.notify(f"{defn.label}: current value unknown. Press r to refresh.", severity="warning")
            return

        current = self._control_values[code_key]
        if defn.values:
            values = [v for _n, v in defn.values]
            idx = values.index(current) if current in values else 0
            new_val = values[(idx + delta) % len(values)]
        else:
            new_val = max(defn.min_val, min(defn.max_val, current + delta))

        # Strict safety check: Block power off or standby to prevent permanent black screen
        if defn.code in (0xD6, 0xC6) and VCPCatalog.is_power_off_value(defn.code, new_val):
            target_label = format_control_value(defn, new_val)
            self.notify(
                f"Blocked: Power state '{target_label}' is disabled to prevent permanent display shutdown.",
                severity="error",
                timeout=5.0,
            )
            return

        # Input source or other dangerous operations: require confirmation
        if defn.dangerous and defn.code not in self._confirmed_dangerous_codes:
            target_label = format_control_value(defn, new_val)
            if defn.code == 0x60:
                self.notify(
                    f"Switch to {target_label}? Disconnected ports black out display. Press again to confirm.",
                    severity="warning",
                )
            else:
                self._confirmed_dangerous_codes.add(defn.code)
                self._do_write_control(defn, code_key, new_val, h)
                return
            self._confirmed_dangerous_codes.add(defn.code)
            return
        self._confirmed_dangerous_codes.discard(defn.code if defn.dangerous else -1)
        self._do_write_control(defn, code_key, new_val, h)

    def _do_write_control(self, defn: ControlDefinition, code_key: str, new_val: int, h: MonitorHandle) -> None:
        old_val = self._control_values.get(code_key)
        self._control_values[code_key] = new_val
        self._update_cached_value_label(defn, new_val)
        self.app.run_worker(
            lambda: self._write_vcp_and_report(defn, new_val, old_val, h),
            name=f"write-vcp-{code_key}", group="hardware-writes", thread=True,
        )
        self._update_top_status()

    def _write_vcp_and_report(self, defn: ControlDefinition, value: int, old_val: int | None, monitor: MonitorHandle) -> None:
        if defn.code is None:
            return
        if not self.mgr.set_vcp(defn.code, value, monitor):
            if old_val is not None:
                self._control_values[defn.code_key] = old_val
            else:
                self._control_values.pop(defn.code_key, None)
            self.app.call_from_thread(self.notify, f"Failed to write {defn.label} ({defn.code_key})", severity="error")
            self.app.call_from_thread(self._rebuild_controls)

    def _update_cached_value_label(self, defn: ControlDefinition, value: int) -> None:
        try:
            formatted = format_control_value(defn, value)
            self.query_one(f"#val-{defn.key}", Label).update(formatted)
            self.query_one(f"#meter-{defn.key}", Label).update(
                control_meter(formatted, defn.min_val, defn.max_val, list(defn.values), read_only=defn.read_only)
            )
        except Exception:
            self._rebuild_controls()

    def _move_selected_control(self, delta: int) -> None:
        if not self._control_defs or self._current_view != NAV_CONTROLS:
            return
        prev = self._selected_control_idx
        self._selected_control_idx = (self._selected_control_idx + delta) % len(self._control_defs)
        rows = list(self.query(VcpControlRow))
        if len(rows) == len(self._control_defs):
            rows[prev].set_selected(False)
            curr = rows[self._selected_control_idx]
            curr.set_selected(True)
            curr.scroll_visible(animate=False)
        else:
            self._rebuild_controls()
        self._update_control_help()

    def _selected_control(self) -> ControlDefinition | None:
        if not self._control_defs:
            return None
        return self._control_defs[self._selected_control_idx]

    def action_select_next_control(self) -> None:
        if self._current_view == NAV_CONTROLS:
            self._move_selected_control(1)
        elif self._current_view == NAV_PROFILES:
            try:
                self.query_one("#profiles-table", DataTable).action_cursor_down()
            except Exception:
                pass
        elif self._current_view == NAV_SCANNER:
            try:
                self.query_one("#scanner-diff", RegisterTable).action_cursor_down()
            except Exception:
                pass

    def action_select_previous_control(self) -> None:
        if self._current_view == NAV_CONTROLS:
            self._move_selected_control(-1)
        elif self._current_view == NAV_PROFILES:
            try:
                self.query_one("#profiles-table", DataTable).action_cursor_up()
            except Exception:
                pass
        elif self._current_view == NAV_SCANNER:
            try:
                self.query_one("#scanner-diff", RegisterTable).action_cursor_up()
            except Exception:
                pass

    def action_page_up_control(self) -> None:
        if self._current_view == NAV_CONTROLS:
            self._move_selected_control(-5)

    def action_page_down_control(self) -> None:
        if self._current_view == NAV_CONTROLS:
            self._move_selected_control(5)

    def action_first_control(self) -> None:
        if self._current_view == NAV_CONTROLS and self._control_defs:
            prev = self._selected_control_idx
            self._selected_control_idx = 0
            rows = list(self.query(VcpControlRow))
            if rows:
                rows[prev].set_selected(False)
                rows[0].set_selected(True)
                rows[0].scroll_visible(animate=False)
            self._update_control_help()

    def action_last_control(self) -> None:
        if self._current_view == NAV_CONTROLS and self._control_defs:
            prev = self._selected_control_idx
            self._selected_control_idx = len(self._control_defs) - 1
            rows = list(self.query(VcpControlRow))
            if rows:
                rows[prev].set_selected(False)
                rows[-1].set_selected(True)
                rows[-1].scroll_visible(animate=False)
            self._update_control_help()

    def action_increase_selected_control(self) -> None:
        if self._current_view != NAV_CONTROLS:
            return
        defn = self._selected_control()
        if defn is not None:
            self._adjust_control(defn.code_key, 1)

    def action_decrease_selected_control(self) -> None:
        if self._current_view != NAV_CONTROLS:
            return
        defn = self._selected_control()
        if defn is not None:
            self._adjust_control(defn.code_key, -1)

    def action_refresh(self) -> None:
        self.query_one(MonitorDetail).update("[dim]Refreshing...[/dim]")
        self.app.run_worker(self._refresh_hardware, name="hardware-refresh", group="hardware", thread=True, exclusive=True)

    def _refresh_hardware(self) -> None:
        self.mgr.refresh()
        defs = self._definitions_for_active_monitor()
        vals = self._read_control_values_for(defs)
        scanner = Scanner(self.mgr)
        self.app.call_from_thread(self._finish_hardware_refresh, defs, vals, scanner)

    def _finish_hardware_refresh(
        self,
        defs: list[ControlDefinition],
        vals: dict[str, int],
        scanner: Scanner,
    ) -> None:
        self._control_defs = defs
        self._control_values = vals
        self._scanner = scanner
        self._register_values_loaded = True
        self._refresh_controls_view()

    def action_quick_apply(self) -> None:
        self.app.push_screen(ProfilePickerScreen(self.mgr, self.store), self._on_picker_done)

    def _on_picker_done(self, _result: object) -> None:
        self._refresh_controls_view()

    # ------------------------------------------------------------------
    # scanner view (embedded)
    # ------------------------------------------------------------------

    def action_scanner_snap1(self) -> None:
        if self._current_view == NAV_SCANNER:
            self._scanner_start_snapshot("snap1")

    def action_scanner_snap2(self) -> None:
        if self._current_view == NAV_SCANNER:
            if self._scanner and not self._scanner.has_snap1:
                self.notify("Take Snapshot 1 first.", severity="warning")
                return
            self._scanner_start_snapshot("snap2")

    def _scanner_start_snapshot(self, which: str) -> None:
        if self._scanner_busy or self._scanner is None:
            return
        if which == "snap2" and not self._scanner.has_snap1:
            self.notify("Take Snapshot 1 first.", severity="warning")
            return
        self._scanner_set_busy(True, "Snapshot 1" if which == "snap1" else "Snapshot 2")
        self.app.run_worker(
            lambda: self._scanner_capture(which),
            name=f"scanner-{which}", group="scanner", thread=True, exclusive=True,
        )

    def _scanner_capture(self, which: str) -> None:
        try:
            if self._scanner is None:
                return
            if which == "snap1":
                self._scanner.take_snapshot1()
                self.app.call_from_thread(self._on_snap1_done)
            else:
                self._scanner.take_snapshot2()
                diff = self._scanner.diff()
                self.app.call_from_thread(self._on_snap2_done, diff)
        except Exception as e:
            self.app.call_from_thread(self.notify, f"Snapshot error: {e}", severity="error")
        finally:
            self.app.call_from_thread(self._scanner_set_busy, False)

    def _on_snap1_done(self) -> None:
        self._update_scanner_status()
        if self._scanner and self._scanner.snap1:
            self._scanner_show_snapshot(self._scanner.snap1.data, "Snapshot 1 Baseline")
        self.query_one("#scanner-runbar", Static).update("Snapshot 1 saved. Now change one OSD setting and click Snapshot 2.")

    def _on_snap2_done(self, diff: dict[int, tuple[int | None, int | None]]) -> None:
        self._update_scanner_status()
        self._scanner_last_diff = diff
        self._scanner_show_diff(diff)
        if diff:
            first_code = sorted(diff)[0]
            self.query_one("#scanner-runbar", Static).update(
                f"Diff found: {len(diff)} register(s) changed! Suggested: 0x{first_code:02X} {VCPCatalog.name(first_code)}"
            )
        else:
            self.query_one("#scanner-runbar", Static).update("No register changed. Change one OSD setting and retry.")

    def _scanner_set_busy(self, busy: bool, label: str = "") -> None:
        self._scanner_busy = busy
        try:
            self.query_one("#scanner-loading", LoadingIndicator).set_class(busy, "active")
            for sel in ("#btn-snap1", "#btn-snap2", "#btn-reset", "#btn-confirm-map"):
                self.query_one(sel, Button).disabled = busy
            if busy:
                self.query_one("#scanner-runbar", Static).update(f"Scanning registers for {label}...")
        except Exception:
            pass

    def _update_scanner_status(self) -> None:
        if self._scanner is None:
            return
        try:
            s1 = self._scanner.snap1.timestamp if self._scanner.has_snap1 and self._scanner.snap1 else "-"
            s2 = self._scanner.snap2.timestamp if self._scanner.has_snap2 and self._scanner.snap2 else "-"
            self.query_one("#scanner-status", Static).update(f"S1 {s1}  S2 {s2}")
        except Exception:
            pass

    def _scanner_show_snapshot(self, data: dict[int, int], title: str) -> None:
        table = self.query_one("#scanner-diff", RegisterTable)
        table.clear(columns=True)
        table.add_columns("Register", "Value")
        table.border_title = title
        for code in sorted(data):
            table.add_row(f"0x{code:02X} {VCPCatalog.name(code)}", str(data[code]), key=f"0x{code:02X}")

    def _scanner_show_diff(self, diff: dict[int, tuple[int | None, int | None]]) -> None:
        table = self.query_one("#scanner-diff", RegisterTable)
        table.clear(columns=True)
        table.add_columns("Register", "Before", "After", "Delta")
        table.border_title = f"Diff - {len(diff)} changed"
        for code, (before, after) in sorted(diff.items()):
            delta = str(after - before) if before is not None and after is not None else ""
            table.add_row(f"0x{code:02X} {VCPCatalog.name(code)}", str(before), str(after), delta, key=f"0x{code:02X}")

    def _scanner_reset(self) -> None:
        if self._scanner_busy or self._scanner is None:
            return
        self._scanner.reset()
        self._scanner_last_diff = {}
        try:
            self.query_one("#scanner-diff", RegisterTable).clear(columns=True)
            self.query_one("#scanner-runbar", Static).update("Ready. Take Snapshot 1 before changing OSD.")
            self._update_scanner_status()
        except Exception:
            pass

    def _scanner_confirm_mapping(self) -> None:
        try:
            label = self.query_one("#mapping-label", Input).value.strip()
        except Exception:
            return
        if not label:
            self.notify("Enter an OSD label first.", severity="error")
            return
        if not self._scanner_last_diff:
            self.notify("Take two snapshots first.", severity="error")
            return
        table = self.query_one("#scanner-diff", RegisterTable)
        code = sorted(self._scanner_last_diff)[0]
        if self._scanner_last_diff and table.row_count:
            try:
                row_key = table.coordinate_to_cell_key(table.cursor_coordinate).row_key
                if row_key is not None:
                    code = int(_table_row_key_value(row_key), 16)
            except (ValueError, Exception):
                pass
        self.mapping_store.save_mapping(code, label)
        self.notify(f"Mapped 0x{code:02X} -> '{label}'")
        try:
            self.query_one("#mapping-label", Input).value = ""
        except Exception:
            pass
        self._control_defs = self._definitions_for_active_monitor()
        self._refresh_controls_view()

    # ------------------------------------------------------------------
    # profiles view (embedded)
    # ------------------------------------------------------------------

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
        if event.data_table.id == "profiles-table":
            self.action_apply_selected_profile()

    def _refresh_profiles_table(self) -> None:
        table = self.query_one("#profiles-table", DataTable)
        table.clear(columns=True)
        table.cursor_type = "row"
        table.zebra_stripes = True
        table.add_columns("Name", "Controls", "Description")
        self._profiles_list = []
        for fp in self.store.list():
            p = self.store.load(fp)
            if p:
                self._profiles_list.append(p)
                table.add_row(p.name, str(len(p.settings)), p.description or "-", key=p.name)

    def _get_selected_profile(self) -> Profile | None:
        table = self.query_one("#profiles-table", DataTable)
        if not self._profiles_list or not table.row_count:
            return None
        try:
            row_key = table.coordinate_to_cell_key(table.cursor_coordinate).row_key
            if row_key is not None:
                key_str = _table_row_key_value(row_key)
                return next((p for p in self._profiles_list if p.name == key_str), None)
        except Exception:
            pass
        row_idx = table.cursor_coordinate.row
        return self._profiles_list[row_idx] if 0 <= row_idx < len(self._profiles_list) else None

    def action_apply_selected_profile(self) -> None:
        if self._current_view != NAV_PROFILES:
            return
        profile = self._get_selected_profile()
        if profile is None:
            self.notify("No profile selected.", severity="warning")
            return
        self.app.run_worker(
            lambda: self._apply_profile_worker(profile),
            name=f"apply-profile-{profile.name}", group="profiles", thread=True, exclusive=True,
        )

    def _apply_profile_worker(self, profile: Profile) -> None:
        count = profile.apply(self.mgr)
        self.app.call_from_thread(self.notify, f"Applied '{profile.name}' - {count} codes set")
        self.app.call_from_thread(self._refresh_controls_view)

    def action_create_new_profile(self) -> None:
        if self._current_view != NAV_PROFILES:
            return
        self.app.push_screen(CreateProfileScreen(self.mgr, self.store), self._on_profile_created)

    def _on_profile_created(self, _result: object) -> None:
        self._refresh_profiles_table()

    def action_delete_selected_profile(self) -> None:
        if self._current_view != NAV_PROFILES:
            return
        profile = self._get_selected_profile()
        if profile is None:
            return
        if self._pending_delete == profile.name:
            self.store.delete(self.store.directory / profile.filename)
            self.notify(f"Deleted '{profile.name}'")
            self._pending_delete = None
            self._refresh_profiles_table()
        else:
            self._pending_delete = profile.name
            self.notify(f"Press delete again to confirm deleting '{profile.name}'.", severity="warning")


def _table_row_key_value(row_key: object) -> str:
    value = getattr(row_key, "value", row_key)
    return str(value)


# ---------------------------------------------------------------------------
# Help Screen
# ---------------------------------------------------------------------------

class HelpScreen(Screen[None]):
    """Modal help screen showing keybindings and safety features."""

    BINDINGS = [
        ("escape", "app.pop_screen", "Close"),
        ("q", "app.pop_screen", "Close"),
        ("question_mark", "app.pop_screen", "Close"),
        ("enter", "app.pop_screen", "Close"),
    ]

    CSS = """
    HelpScreen {
        align: center middle;
        background: rgba(0, 0, 0, 0.7);
    }
    #help-dialog {
        width: 84;
        height: auto;
        max-height: 90%;
        padding: 1 2;
        background: $surface;
        border: round $primary;
        border-title-color: $primary;
    }
    .help-header {
        width: 100%;
        text-align: center;
        text-style: bold;
        color: $primary;
        margin-bottom: 1;
    }
    .help-subhead {
        text-style: bold;
        color: $secondary;
        margin-top: 1;
        margin-bottom: 0;
    }
    .help-row {
        height: 1;
    }
    .help-key {
        width: 26;
        color: $primary;
        text-style: bold;
    }
    .help-desc {
        width: 1fr;
        color: $foreground;
    }
    #help-close {
        margin-top: 1;
        width: 100%;
        text-align: center;
        color: $text-muted;
    }
    """

    def compose(self) -> ComposeResult:
        with Container(id="help-dialog"):
            yield Static("Toushou Monitor TUI - Help & Safety Guide", classes="help-header")

            yield Static("Navigation & Mouse Controls", classes="help-subhead")
            with Horizontal(classes="help-row"):
                yield Static("Mouse Click / [-][+]", classes="help-key")
                yield Static("Click row to select, click [-][+] to adjust value", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("Up / Down (k / j)", classes="help-key")
                yield Static("Select next / previous control (smooth auto-scroll)", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("Left / Right (h / l, - / +)", classes="help-key")
                yield Static("Adjust value or cycle discrete options", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("Tab / Shift+Tab", classes="help-key")
                yield Static("Cycle through Controls / Scanner / Profiles tabs", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("F1 / F2 / F3", classes="help-key")
                yield Static("Jump directly to Controls / Scanner / Profiles", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("PgUp / PgDn / Home / End", classes="help-key")
                yield Static("Fast scroll through controls list", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("Enter", classes="help-key")
                yield Static("Cycle discrete setting or apply selected profile", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("p / r / t / q", classes="help-key")
                yield Static("Profile picker / Refresh / Toggle theme / Quit", classes="help-desc")

            yield Static("Hardware Safety Protections", classes="help-subhead")
            with Horizontal(classes="help-row"):
                yield Static("[bold green]Power-Off Lock[/]", classes="help-key")
                yield Static("Standby/Off via DDC/CI is disabled (prevents black screen)", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("[bold yellow]Input Source Guard[/]", classes="help-key")
                yield Static("Changing input requires pressing twice to avoid lost signal", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("[bold red]Factory Reset Lock[/]", classes="help-key")
                yield Static("Destructive resets (0x04, 0x05, etc.) are excluded", classes="help-desc")
            with Horizontal(classes="help-row"):
                yield Static("[bold cyan]Profile Safety[/]", classes="help-key")
                yield Static("Applying profiles skips power off & read-only registers", classes="help-desc")

            yield Static("[dim]Press ?, Esc, or Enter to close[/dim]", id="help-close")


# ---------------------------------------------------------------------------
# Screens kept for backward compatibility (quick picker, create profile)
# ---------------------------------------------------------------------------

class ScannerScreen(Screen[None]):
    """Legacy scanner screen - kept for tests. Use sidebar nav instead."""
    BINDINGS = [("escape", "app.pop_screen", "Back")]
    def compose(self) -> ComposeResult:
        yield Static("Use the sidebar Scanner navigation instead.")
        yield Footer()


class ProfilesScreen(Screen[None]):
    """Legacy profiles screen - kept for tests. Use sidebar nav instead."""
    BINDINGS = [("escape", "app.pop_screen", "Back")]

    def __init__(self, manager: MonitorManager, store: ProfileStore) -> None:
        super().__init__()
        self.mgr = manager
        self.store = store

    def compose(self) -> ComposeResult:
        yield Static("Use the sidebar Profiles navigation instead.")
        yield Footer()


class ProfilePickerScreen(Screen[None]):
    """Quick profile picker overlay (modal-style) opened with 'p' key."""

    BINDINGS = [
        ("escape", "app.pop_screen", "Cancel"),
        ("q", "app.pop_screen", "Cancel"),
    ]

    CSS = """
    ProfilePickerScreen {
        align: center middle;
        background: rgba(0, 0, 0, 0.6);
    }
    #picker-box {
        width: 60; height: auto; max-height: 20;
        padding: 1 2;
        background: $panel;
        border: round $primary; border-title-color: $primary;
    }
    #picker-title { color: $primary; text-style: bold; margin-bottom: 1; }
    #picker-list { height: auto; max-height: 12; }
    #picker-empty { color: $text-muted; }
    """

    def __init__(self, manager: MonitorManager, store: ProfileStore) -> None:
        super().__init__()
        self.mgr = manager
        self.store = store
        self._profiles: list[Profile] = []

    def compose(self) -> ComposeResult:
        with Container(id="picker-box"):
            yield Static("Select Profile to Apply", id="picker-title")
            yield ListView(id="picker-list")
            yield Static("No profiles found. Create one with 'n' in Profiles view.", id="picker-empty")

    def on_mount(self) -> None:
        lv = self.query_one("#picker-list", ListView)
        self._profiles = []
        for fp in self.store.list():
            p = self.store.load(fp)
            if p:
                self._profiles.append(p)
                lv.append(ListItem(Label(f"[bold]{p.name}[/] ({len(p.settings)} settings)")))
        if self._profiles:
            self.query_one("#picker-empty", Static).display = False
        else:
            lv.display = False

    def on_list_view_selected(self, event: ListView.Selected) -> None:
        idx = getattr(event, "index", None)
        if idx is None and hasattr(event.list_view, "index"):
            idx = event.list_view.index
        if idx is not None and 0 <= idx < len(self._profiles):
            target = self._profiles[idx]
            self.app.pop_screen()
            self.app.run_worker(
                lambda: self._apply_worker(target),
                name=f"quick-apply-{target.name}", group="profiles", thread=True,
            )

    def _apply_worker(self, profile: Profile) -> None:
        count = profile.apply(self.mgr)
        self.app.call_from_thread(self.notify, f"Applied '{profile.name}' - {count} codes set")


class CreateProfileScreen(Screen[Path | None]):
    """Modal screen for naming and saving a new profile."""

    BINDINGS = [("escape", "app.pop_screen", "Cancel")]

    CSS = """
    CreateProfileScreen {
        align: center middle;
        background: rgba(0, 0, 0, 0.6);
    }
    #create-box {
        width: 50; height: auto;
        padding: 1 2;
        background: $panel;
        border: round $primary; border-title-color: $primary;
    }
    #create-title { color: $primary; text-style: bold; margin-bottom: 1; }
    #create-box Input { margin-bottom: 1; }
    #create-actions { height: 3; align: right middle; }
    #create-actions Button { margin-left: 1; }
    """

    def __init__(self, manager: MonitorManager, store: ProfileStore) -> None:
        super().__init__()
        self.mgr = manager
        self.store = store

    def compose(self) -> ComposeResult:
        with Container(id="create-box"):
            yield Static("Save Current Monitor Settings as Profile", id="create-title")
            yield Input(placeholder="Profile name (e.g. Gaming, Reading)", id="input-name")
            yield Input(placeholder="Description (optional)", id="input-desc")
            with Horizontal(id="create-actions"):
                yield Button("Save", id="btn-save", variant="success")
                yield Button("Cancel", id="btn-cancel")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "btn-save":
            self._save()
        else:
            self.dismiss(None)

    def on_input_submitted(self, event: Input.Submitted) -> None:
        if event.input.id == "input-name":
            self._save()

    def _save(self) -> None:
        name = self.query_one("#input-name", Input).value.strip()
        desc = self.query_one("#input-desc", Input).value.strip()
        if not name:
            self.notify("Profile name cannot be empty.", severity="error")
            return
        p = Profile.capture(name, self.mgr, description=desc)
        path = self.store.save(p)
        self.notify(f"Profile '{name}' saved ({len(p.settings)} settings)")
        self.dismiss(path)
