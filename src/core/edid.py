"""Windows EDID monitor identity helpers."""

from __future__ import annotations

import json
import platform
import subprocess
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class EdidIdentity:
    index: int
    instance_name: str = ""
    manufacturer: str = ""
    product_code: str = ""
    serial: str = ""
    user_friendly_name: str = ""

    @property
    def name(self) -> str:
        return self.user_friendly_name

    @property
    def device_id(self) -> str:
        return self.instance_name


def decode_edid_string(value: Any) -> str:
    """Decode a WMI EDID uint16 array into a display string."""
    if value is None:
        return ""
    chars: list[str] = []
    for item in value:
        try:
            codepoint = int(item)
        except (TypeError, ValueError):
            continue
        if codepoint:
            chars.append(chr(codepoint))
    return "".join(chars).strip()


def load_windows_edid_identities() -> list[EdidIdentity]:
    """Return active Windows monitor identities, best-effort."""
    if platform.system().lower() != "windows":
        return []

    command = (
        "Get-CimInstance -Namespace root\\wmi -ClassName WmiMonitorID | "
        "Where-Object { $_.Active } | "
        "Select-Object InstanceName,ManufacturerName,ProductCodeID,SerialNumberID,UserFriendlyName | "
        "ConvertTo-Json -Compress"
    )
    try:
        result = subprocess.run(
            ["powershell", "-NoProfile", "-Command", command],
            capture_output=True,
            check=False,
            text=True,
            timeout=5,
        )
    except Exception:
        return []
    if result.returncode != 0 or not result.stdout.strip():
        return []

    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError:
        return []
    if isinstance(payload, dict):
        rows = [payload]
    elif isinstance(payload, list):
        rows = payload
    else:
        return []

    identities: list[EdidIdentity] = []
    for idx, row in enumerate(rows):
        identities.append(
            EdidIdentity(
                index=idx,
                instance_name=str(row.get("InstanceName", "")),
                manufacturer=decode_edid_string(row.get("ManufacturerName")),
                product_code=decode_edid_string(row.get("ProductCodeID")),
                serial=decode_edid_string(row.get("SerialNumberID")),
                user_friendly_name=decode_edid_string(row.get("UserFriendlyName")),
            )
        )
    return identities
