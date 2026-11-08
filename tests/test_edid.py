"""Tests for EDID identity helpers."""

from src.core.edid import decode_edid_string


def test_decode_edid_string() -> None:
    assert decode_edid_string([65, 76, 50, 53, 56, 81, 0, 0]) == "AL258Q"
    assert decode_edid_string([73, 67, 68, 0]) == "ICD"
    assert decode_edid_string([50, 52, 53, 48, 0]) == "2450"
