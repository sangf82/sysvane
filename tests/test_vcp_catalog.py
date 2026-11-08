"""Tests for VCP catalog."""

import pytest
from src.core.vcp_catalog import POWER_CODES, VCPCatalog, VCPEntry


class TestVCPCatalog:
    def test_known_code(self) -> None:
        entry = VCPCatalog.get(0x10)
        assert entry is not None
        assert entry.name == "Luminance / Brightness"
        assert entry.max_val == 100

    def test_mfg_code(self) -> None:
        entry = VCPCatalog.get(0xE5)
        assert entry is not None
        assert entry.manufacturer_specific is True
        assert VCPCatalog.is_mfg(0xE5)

    def test_unknown_code(self) -> None:
        entry = VCPCatalog.get(0x200)
        assert entry is None

    def test_name_fallback(self) -> None:
        name = VCPCatalog.name(0xFE)
        assert "0xFE" in name

    def test_code_count(self) -> None:
        assert len(VCPCatalog.codes()) == 153
        assert len(VCPCatalog.standard_codes()) == 121
        assert len(VCPCatalog.mfg_codes()) == 32

    def test_input_source_name(self) -> None:
        assert VCPCatalog.input_source_name(0x0F) == "DisplayPort 1"
        assert VCPCatalog.input_source_name(0x11) == "HDMI 1"
        assert VCPCatalog.input_source_name(99) == "Input 99"

    def test_color_preset_name(self) -> None:
        assert VCPCatalog.color_preset_name(0x01) == "sRGB"
        assert VCPCatalog.color_preset_name(0x05) == "6500K"
        assert VCPCatalog.color_preset_name(99) == "Preset 99"

    def test_power_mode_name(self) -> None:
        assert VCPCatalog.power_mode_name(0x01) == "On"
        assert VCPCatalog.power_mode_name(0x04) == "Off (Soft)"

    def test_osd_language_name(self) -> None:
        assert VCPCatalog.osd_language_name(2) == "English"
        assert VCPCatalog.osd_language_name(99) == "Language 99"

    def test_dangerous_codes(self) -> None:
        assert VCPCatalog.is_dangerous(0x04) is True
        assert VCPCatalog.is_dangerous(0x05) is True
        assert VCPCatalog.is_dangerous(0x06) is True
        assert VCPCatalog.is_dangerous(0x08) is True
        assert VCPCatalog.is_dangerous(0x10) is False

    def test_confirm_codes(self) -> None:
        assert VCPCatalog.needs_confirm(0x60) is True
        assert VCPCatalog.needs_confirm(0xD6) is True

    def test_readonly_frequency_registers(self) -> None:
        ac = VCPCatalog.get(0xAC)
        ae = VCPCatalog.get(0xAE)
        assert ac is not None and ac.readable is True and ac.writeable is False
        assert ae is not None and ae.readable is True and ae.writeable is False

    def test_power_codes_and_power_off_detection(self) -> None:
        assert 0xD6 in POWER_CODES
        assert 0xC6 in POWER_CODES
        assert VCPCatalog.is_power_code(0xD6) is True
        assert VCPCatalog.is_power_code(0xC6) is True
        assert VCPCatalog.is_power_code(0x10) is False

        # 0xD6: 1 is On, 2..5 are standby/suspend/off
        assert VCPCatalog.is_power_off_value(0xD6, 1) is False
        assert VCPCatalog.is_power_off_value(0xD6, 2) is True
        assert VCPCatalog.is_power_off_value(0xD6, 3) is True
        assert VCPCatalog.is_power_off_value(0xD6, 4) is True
        assert VCPCatalog.is_power_off_value(0xD6, 5) is True

        # 0xC6: 1 is Full Power, 2 is Off
        assert VCPCatalog.is_power_off_value(0xC6, 1) is False
        assert VCPCatalog.is_power_off_value(0xC6, 2) is True

        # Brightness is not a power code
        assert VCPCatalog.is_power_off_value(0x10, 0) is False
