"""Tests for profile engine."""

import json
import tempfile
from pathlib import Path

import pytest
from src.core.profiles import ExperimentalMappingStore, Profile, ProfileStore


class TestProfile:
    def test_serialization_roundtrip(self) -> None:
        p = Profile(
            name="Test Profile",
            description="For testing",
            settings={"0x10": 85, "0x12": 75, "0x60": 15},
            experimental_mappings={"ECO": 0xE6},
        )
        d = p.to_dict()
        p2 = Profile.from_dict(d)
        assert p2.name == "Test Profile"
        assert p2.description == "For testing"
        assert p2.settings == {"0x10": 85, "0x12": 75, "0x60": 15}
        assert p2.experimental_mappings == {"ECO": 0xE6}

    def test_filename_sanitization(self) -> None:
        p = Profile(name="My Gaming/Test profile", settings={})
        assert "/" not in p.filename
        assert " " not in p.filename

    def test_empty_profile(self) -> None:
        p = Profile(name="Empty", settings={})
        assert p.filename == "Empty.json"
        d = p.to_dict()
        assert d["settings"] == {}

    def test_apply_counts_only_successful_writes(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return code != 0x12

        mgr = FakeManager()
        p = Profile(name="Partial", settings={"0x10": 50, "0x12": 70})

        assert p.apply(mgr) == 1  # type: ignore[arg-type]
        assert mgr.writes == [(0x10, 50), (0x12, 70)]


class TestProfileStore:
    def test_save_and_load(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            store = ProfileStore(directory=tmp)
            p = Profile(name="My Profile", description="desc", settings={"0x10": 50})
            path = store.save(p)
            assert path.exists()
            loaded = store.load(path)
            assert loaded is not None
            assert loaded.name == "My Profile"
            assert loaded.settings == {"0x10": 50}

    def test_list_all(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            store = ProfileStore(directory=tmp)
            store.save(Profile(name="Alpha", settings={"0x10": 10}))
            store.save(Profile(name="Beta", settings={"0x10": 20}))
            store.save(Profile(name="Gamma", settings={"0x10": 30}))

            profiles = store.load_all()
            assert len(profiles) == 3
            names = {p.name for p in profiles}
            assert names == {"Alpha", "Beta", "Gamma"}

    def test_delete(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            store = ProfileStore(directory=tmp)
            path = store.save(Profile(name="DeleteMe", settings={}))
            assert path.exists()
            store.delete(path)
            assert not path.exists()

    def test_load_ignores_dual_setup_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dual = Path(tmp) / "Calibrated-Dual-Setup.json"
            dual.write_text('{"name": "Dual", "monitors": []}', encoding="utf-8")
            store = ProfileStore(directory=tmp)

            assert store.load(dual) is None
            assert store.load_all() == []

    def test_list_empty_directory(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            store = ProfileStore(directory=tmp)
            assert store.list() == []
            assert store.load_all() == []


class TestProfileSafety:
    def test_apply_skips_dangerous_codes(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="Unsafe", settings={
            "0x10": 50,
            "0x04": 1,
            "0x05": 1,
            "0x06": 1,
            "0x08": 1,
        })
        count = p.apply(mgr)  # type: ignore[arg-type]
        written_codes = [code for code, _ in mgr.writes]
        assert 0x10 in written_codes
        assert 0x04 not in written_codes
        assert 0x05 not in written_codes
        assert 0x06 not in written_codes
        assert 0x08 not in written_codes
        assert count == 1

    def test_apply_skips_read_only_codes(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="ReadOnly", settings={
            "0x10": 50,
            "0xAC": 67500,
            "0xAE": 144,
            "0xC0": 1000,
            "0xDF": 514,
        })
        count = p.apply(mgr)  # type: ignore[arg-type]
        written_codes = [code for code, _ in mgr.writes]
        assert 0x10 in written_codes
        assert 0xAC not in written_codes
        assert 0xAE not in written_codes
        assert 0xC0 not in written_codes
        assert 0xDF not in written_codes
        assert count == 1

    def test_apply_skips_power_codes_by_default(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="PowerSkip", settings={
            "0x10": 60,
            "0xD6": 1,
            "0xC6": 1,
        })
        count = p.apply(mgr, include_power=False)  # type: ignore[arg-type]
        written_codes = [code for code, _ in mgr.writes]
        assert 0x10 in written_codes
        assert 0xD6 not in written_codes
        assert 0xC6 not in written_codes
        assert count == 1

    def test_apply_skips_input_source_by_default(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="Input", settings={"0x10": 50, "0x60": 15})

        assert p.apply(mgr) == 1  # type: ignore[arg-type]
        assert mgr.writes == [(0x10, 50)]

        assert p.apply(mgr, include_input=True) == 2  # type: ignore[arg-type]
        assert (0x60, 15) in mgr.writes

    def test_apply_writes_color_preset_before_rgb_gains(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="CVTE", settings={"0x16": 47, "0x10": 70, "0x14": 11, "0x18": 49})
        p.apply(mgr)  # type: ignore[arg-type]

        assert mgr.writes[0] == (0x14, 11)

    def test_apply_blocks_power_off_even_if_include_power(self) -> None:
        class FakeManager:
            def __init__(self) -> None:
                self.writes: list[tuple[int, int]] = []

            def set_vcp(self, code: int, value: int, monitor: object | None = None) -> bool:
                self.writes.append((code, value))
                return True

        mgr = FakeManager()
        p = Profile(name="PowerOffBlock", settings={
            "0x10": 60,
            "0xD6": 4,  # Power Off (Soft)
        })
        count = p.apply(mgr, include_power=True)  # type: ignore[arg-type]
        written_codes = [code for code, _ in mgr.writes]
        assert 0x10 in written_codes
        assert 0xD6 not in written_codes
        assert count == 1

    def test_capture_excludes_dangerous_codes(self) -> None:
        class FakeManager:
            def get_vcp(self, code: int, monitor: object | None = None) -> int | None:
                return 42

        mgr = FakeManager()
        profile = Profile.capture("Test", "", mgr)  # type: ignore[arg-type]
        assert "0x04" not in profile.settings
        assert "0x05" not in profile.settings
        assert "0x06" not in profile.settings
        assert "0x08" not in profile.settings
        assert "0x10" in profile.settings

    def test_capture_excludes_power_codes(self) -> None:
        class FakeManager:
            def get_vcp(self, code: int, monitor: object | None = None) -> int | None:
                return 1

        mgr = FakeManager()
        profile = Profile.capture("Test", "", mgr)  # type: ignore[arg-type]
        assert "0xD6" not in profile.settings
        assert "0xC6" not in profile.settings

    def test_capture_excludes_non_readable_codes(self) -> None:
        class FakeManager:
            def get_vcp(self, code: int, monitor: object | None = None) -> int | None:
                return 42

        mgr = FakeManager()
        profile = Profile.capture("Test", "", mgr)  # type: ignore[arg-type]
        assert "0x1E" not in profile.settings
        assert "0xA2" not in profile.settings

    def test_capture_excludes_manufacturer_specific_by_default(self) -> None:
        class FakeManager:
            def get_vcp(self, code: int, monitor: object | None = None) -> int | None:
                return 42

        mgr = FakeManager()
        profile = Profile.capture("Test", "", mgr)  # type: ignore[arg-type]
        assert "0xE0" not in profile.settings
        assert "0xFF" not in profile.settings


class TestExperimentalMappingStore:
    def test_set_and_load(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "experimental_mappings.json"
            store = ExperimentalMappingStore(path)
            store.set("ECO", 0xE6)
            assert store.load() == {"ECO": 0xE6}
