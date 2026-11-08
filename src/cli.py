"""CLI interface using Typer for non-interactive monitor control."""

from __future__ import annotations

import sys
import typer

from .core.monitor_manager import MonitorManager
from .core.profiles import Profile, ProfileStore
from .core.vcp_catalog import VCPCatalog

app = typer.Typer(name="toushou", help="Multi-monitor DDC/CI controller CLI")


def _manager_or_exit(load_details: bool = False) -> MonitorManager:
    mgr = MonitorManager()
    mgr.refresh(load_details=load_details)
    if not mgr.available:
        typer.echo("No DDC/CI monitors detected.")
        raise typer.Exit(code=1)
    return mgr


def _parse_code(code: str) -> int:
    try:
        return int(code, 16)
    except ValueError:
        typer.echo(f"Invalid VCP code: {code!r}. Use hex like 0x10.")
        raise typer.Exit(code=2)


@app.command("list-monitors")
def list_monitors() -> None:
    """List all detected DDC/CI monitors."""
    mgr = _manager_or_exit(load_details=False)
    for i, h in enumerate(mgr.monitors):
        active = "*" if i == mgr.active_idx else " "
        typer.echo(f"[{active}] {i}: {h.info.display_name} - {h.info.model} ({h.info.ddc_description})")


@app.command("get")
@app.command("get-value")
def get_value(
    code: str = typer.Argument(..., help="VCP code in hex, e.g. 0x10"),
    monitor: int = typer.Option(0, "--monitor", "-m", help="Monitor index"),
) -> None:
    """Read a VCP register value from the specified monitor."""
    vcp_code = _parse_code(code)
    mgr = _manager_or_exit(load_details=False)
    if monitor >= len(mgr.monitors):
        typer.echo(f"Monitor index {monitor} out of range (found {len(mgr.monitors)}).")
        raise typer.Exit(code=1)
    mgr.select_monitor(monitor)
    val = mgr.get_vcp(vcp_code)
    name = VCPCatalog.name(vcp_code)
    if val is not None:
        typer.echo(f"{name} (0x{vcp_code:02X}) = {val}")
    else:
        typer.echo(f"0x{vcp_code:02X} ({name}) -> read failed")
        raise typer.Exit(code=1)


@app.command("set")
@app.command("set-value")
def set_value(
    code: str = typer.Argument(..., help="VCP code in hex, e.g. 0x10"),
    value: int = typer.Argument(..., help="Value to write (decimal)"),
    monitor: int = typer.Option(0, "--monitor", "-m", help="Monitor index"),
    force: bool = typer.Option(False, "--force", "-f", help="Bypass safety protection for dangerous codes"),
) -> None:
    """Write a value to a VCP register on the specified monitor."""
    vcp_code = _parse_code(code)
    name = VCPCatalog.name(vcp_code)

    # Safety checks
    if VCPCatalog.is_power_off_value(vcp_code, value):
        if not force:
            typer.echo(
                f"Blocked: Setting 0x{vcp_code:02X} ({name}) to {value} would turn off the monitor. "
                "Use --force to override if you are certain."
            )
            raise typer.Exit(code=3)

    if VCPCatalog.is_dangerous(vcp_code) and not force:
        typer.echo(
            f"Blocked: 0x{vcp_code:02X} ({name}) is a dangerous operation (factory reset). "
            "Pass --force to execute."
        )
        raise typer.Exit(code=3)

    if vcp_code == 0x60 and not force:
        typer.echo(
            f"Warning: Switching Input Source (0x60) to {value} may cause loss of video. "
            "Pass --force to confirm."
        )
        raise typer.Exit(code=3)

    mgr = _manager_or_exit(load_details=False)
    if monitor >= len(mgr.monitors):
        typer.echo(f"Monitor index {monitor} out of range.")
        raise typer.Exit(code=1)
    mgr.select_monitor(monitor)
    ok = mgr.set_vcp(vcp_code, value)
    if ok:
        typer.echo(f"Set {name} (0x{vcp_code:02X}) -> {value}")
    else:
        typer.echo(f"0x{vcp_code:02X} ({name}) <- {value} FAILED")
        raise typer.Exit(code=1)


@app.command("apply-profile")
def apply_profile(
    name: str = typer.Argument(..., help="Profile name"),
    monitor: int = typer.Option(0, "--monitor", "-m", help="Monitor index"),
    include_power: bool = typer.Option(False, "--include-power", help="Allow applying power register states"),
    include_input: bool = typer.Option(False, "--include-input", help="Allow switching the input source (0x60)"),
) -> None:
    """Apply a saved profile to the specified monitor."""
    store = ProfileStore()
    mgr = _manager_or_exit(load_details=False)
    if monitor >= len(mgr.monitors):
        typer.echo(f"Monitor index {monitor} out of range.")
        raise typer.Exit(code=1)
    mgr.select_monitor(monitor)

    for fp in store.list():
        p = store.load(fp)
        if p and p.name.lower() == name.lower():
            p.apply(mgr, include_power=include_power, include_input=include_input)
            typer.echo(f"Profile '{p.name}' applied to monitor {monitor}.")
            return

    typer.echo(f"Profile '{name}' not found.")
    raise typer.Exit(code=1)


@app.command("apply-dual")
def apply_dual(
    name: str = typer.Option("Calibrated-Dual-Setup", "--name", "-n", help="Dual setup file name"),
    include_power: bool = typer.Option(False, "--include-power", help="Allow applying power register states"),
    include_input: bool = typer.Option(False, "--include-input", help="Allow switching the input source (0x60)"),
) -> None:
    """Apply a dual-monitor setup profile to all monitors simultaneously."""
    import json
    store = ProfileStore()
    dual_file = store.directory / f"{name}.json"
    if not dual_file.exists():
        typer.echo(f"Dual profile file '{dual_file.name}' not found.")
        raise typer.Exit(code=1)
    
    data = json.loads(dual_file.read_text(encoding="utf-8"))
    monitors_data = data.get("monitors", [])
    if not monitors_data:
        typer.echo(f"No monitor configuration found in {dual_file.name}.")
        raise typer.Exit(code=1)

    mgr = _manager_or_exit(load_details=False)
    applied_count = 0
    for item in monitors_data:
        idx = item.get("index", 0)
        if idx < len(mgr.monitors):
            handle = mgr.monitors[idx]
            sub_prof = Profile(name=item.get("name", f"Monitor {idx}"), settings=item.get("settings", {}))
            sub_prof.apply(mgr, monitor=handle, include_power=include_power, include_input=include_input)
            applied_count += 1
            typer.echo(f"Applied calibration to {item.get('name', f'Monitor {idx}')} (index {idx}).")

    typer.echo(f"Successfully calibrated {applied_count} monitor(s) from '{name}'.")


@app.command("list-profiles")
def list_profiles() -> None:
    """List all saved profiles."""
    store = ProfileStore()
    profiles = store.list()
    if not profiles:
        typer.echo("No profiles saved yet.")
        return
    for fp in profiles:
        p = store.load(fp)
        if p:
            typer.echo(f"  {p.name:30s}  {len(p.settings):3d} codes  - {p.description}")


def main() -> None:
    if sys.platform == "win32":
        try:
            reconfig_out = getattr(sys.stdout, "reconfigure", None)
            if callable(reconfig_out):
                reconfig_out(encoding="utf-8", errors="replace")
            reconfig_err = getattr(sys.stderr, "reconfigure", None)
            if callable(reconfig_err):
                reconfig_err(encoding="utf-8", errors="replace")
        except Exception:
            pass
    app()


if __name__ == "__main__":
    main()
