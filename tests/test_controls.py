"""Tests for capability-driven OSD controls."""

from enum import Enum

from src.core.controls import build_control_definitions, format_control_value


class FakeVcpCode(Enum):
    BRIGHTNESS = 0x10


def test_build_control_definitions_filters_supported_codes() -> None:
    capabilities = {
        "vcp": {
            0x10: {},
            0x12: {},
            0x14: {},
            0x60: {},
            0xAE: {},
            0xB2: {},
            0xB6: {},
        },
        "inputs": [15, 16],
        "color_presets": [5],
    }

    controls = build_control_definitions(capabilities)
    labels = {control.label for control in controls}

    assert "Brightness" in labels
    assert "Contrast" in labels
    assert "Input Source" in labels
    assert "Refresh Rate" in labels
    assert "Sharpness" not in labels
    assert "Blue Light Filter" not in labels
    assert "ECO" in labels
    assert any(control.label == "Adaptive-Sync" and control.experimental for control in controls)


def test_format_control_value_for_known_special_cases() -> None:
    controls = build_control_definitions({"vcp": {0x60: {}, 0xAE: {}, 0xDF: {}}, "inputs": [15]})
    input_control = next(control for control in controls if control.code == 0x60)
    refresh_control = next(control for control in controls if control.code == 0xAE)
    version_control = next(control for control in controls if control.code == 0xDF)

    assert format_control_value(input_control, 15) == "DisplayPort 1"
    assert format_control_value(refresh_control, 32000) == "320 Hz"
    assert format_control_value(version_control, 514) == "2.2"


def test_experimental_mapping_turns_unmapped_control_into_candidate_code() -> None:
    controls = build_control_definitions({"vcp": {}}, {"ECO": 0xE6})
    eco = next(control for control in controls if control.label == "ECO")

    assert eco.code == 0xE6
    assert eco.experimental is True


def test_build_control_definitions_accepts_enum_vcp_keys() -> None:
    controls = build_control_definitions({"vcp": {FakeVcpCode.BRIGHTNESS: {}}})

    assert any(control.label == "Brightness" for control in controls)


def test_falls_back_to_cmds_when_vcp_is_empty() -> None:
    controls = build_control_definitions({"vcp": {}, "cmds": {0x10: {}, 0x12: {}}})
    labels = {control.label for control in controls}

    assert "Brightness" in labels
    assert "Contrast" in labels
    assert "Sharpness" not in labels


def test_empty_capabilities_keep_standard_controls_visible() -> None:
    controls = build_control_definitions({})
    labels = {control.label for control in controls}

    assert "Brightness" in labels
    assert "Contrast" in labels


def test_no_duplicate_vcp_code_0xb6() -> None:
    capabilities = {"vcp": {0xB6: {}}, "inputs": [], "color_presets": []}
    controls = build_control_definitions(capabilities)
    b6_controls = [c for c in controls if c.code == 0xB6]
    assert len(b6_controls) == 1
    assert b6_controls[0].category == "Picture"


def test_input_source_is_dangerous() -> None:
    controls = build_control_definitions({"vcp": {0x60: {}}, "inputs": [15]})
    input_ctrl = next(c for c in controls if c.code == 0x60)
    assert input_ctrl.dangerous is True


def test_power_mode_is_dangerous() -> None:
    controls = build_control_definitions({"vcp": {0xD6: {}}, "inputs": []})
    power_ctrl = next(c for c in controls if c.code == 0xD6)
    assert power_ctrl.dangerous is True
    assert len(power_ctrl.values) == 5


def test_osd_language_has_discrete_values() -> None:
    controls = build_control_definitions({"vcp": {0xCC: {}}, "inputs": []})
    lang_ctrl = next(c for c in controls if c.code == 0xCC)
    assert lang_ctrl.values
    labels = [name for name, _ in lang_ctrl.values]
    assert "English" in labels
    assert lang_ctrl.experimental is False


def test_brightness_is_not_dangerous() -> None:
    controls = build_control_definitions({"vcp": {0x10: {}}, "inputs": []})
    brightness = next(c for c in controls if c.code == 0x10)
    assert brightness.dangerous is False
