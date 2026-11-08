"""Toushou Monitor TUI — Multi-monitor DDC/CI Controller.

Usage:
    python main.py              Launch the Textual TUI (default)
    python main.py --cli ...    Run CLI commands (e.g. --cli list, --cli set 0x10 85)
"""

from __future__ import annotations

import sys

from src.app import ToushouApp
from src.core.monitor_manager import MonitorManager
from src.core.profiles import ProfileStore


def run_tui() -> None:
    """Launch the full Textual TUI."""
    manager = MonitorManager(auto_refresh=False)
    store = ProfileStore()
    app = ToushouApp(manager, store)
    app.run()


def run_cli() -> None:
    """Run the Typer CLI (passthrough remaining args)."""
    from src.cli import app as cli_app

    cli_app()


def main() -> None:
    if "--cli" in sys.argv:
        sys.argv.remove("--cli")
        run_cli()
    else:
        run_tui()


if __name__ == "__main__":
    main()
