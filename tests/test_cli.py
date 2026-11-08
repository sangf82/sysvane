"""Tests for Toushou CLI interface."""

from __future__ import annotations

from unittest.mock import MagicMock, patch
import pytest
from typer.testing import CliRunner

from src.cli import app

runner = CliRunner()


def test_cli_set_blocks_power_off_without_force() -> None:
    # 0xD6 with value 4 (Off) or 2 (Standby) must be blocked
    result = runner.invoke(app, ["set-value", "0xD6", "4"])
    assert result.exit_code == 3
    assert "Blocked: Setting" in result.output
    assert "would turn off the monitor" in result.output


def test_cli_set_blocks_dangerous_codes_without_force() -> None:
    # 0x04 is Factory Reset
    result = runner.invoke(app, ["set-value", "0x04", "1"])
    assert result.exit_code == 3
    assert "dangerous operation" in result.output


def test_cli_set_warns_input_switch_without_force() -> None:
    # 0x60 is Input Source
    result = runner.invoke(app, ["set-value", "0x60", "15"])
    assert result.exit_code == 3
    assert "Switching Input Source" in result.output


@patch("src.cli._manager_or_exit")
def test_cli_set_with_force_allows_execution(mock_mgr_or_exit: MagicMock) -> None:
    mock_mgr = MagicMock()
    mock_mgr.monitors = [MagicMock()]
    mock_mgr.set_vcp.return_value = True
    mock_mgr_or_exit.return_value = mock_mgr

    result = runner.invoke(app, ["set-value", "0x60", "15", "--force"])
    assert result.exit_code == 0
    mock_mgr.set_vcp.assert_called_once_with(0x60, 15)


@patch("src.cli._manager_or_exit")
def test_cli_safe_brightness_set(mock_mgr_or_exit: MagicMock) -> None:
    mock_mgr = MagicMock()
    mock_mgr.monitors = [MagicMock()]
    mock_mgr.set_vcp.return_value = True
    mock_mgr_or_exit.return_value = mock_mgr

    result = runner.invoke(app, ["set-value", "0x10", "80"])
    assert result.exit_code == 0
    mock_mgr.set_vcp.assert_called_once_with(0x10, 80)
    assert "Set Luminance / Brightness (0x10) -> 80" in result.output


@patch("src.cli._manager_or_exit")
def test_cli_get_value(mock_mgr_or_exit: MagicMock) -> None:
    mock_mgr = MagicMock()
    mock_mgr.monitors = [MagicMock()]
    mock_mgr.get_vcp.return_value = 75
    mock_mgr_or_exit.return_value = mock_mgr

    result = runner.invoke(app, ["get", "0x10"])
    assert result.exit_code == 0
    assert "Luminance / Brightness (0x10) = 75" in result.output
