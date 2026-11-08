# VCP Code Reference — AL258Q

Acer AL258Q, VCP 2.2, CVTE controller, firmware 46507. Scanned 2026-05-29.
Only codes the monitor actually responds to are listed.

---

## All Responding VCP Codes

### Picture

| Code | Name | Current | Max | R/W | How to Configure |
|------|------|---------|-----|-----|------------------|
| 0x10 | Brightness | 53 | 100 | RW | Backlight intensity. Set 30-50 for normal room lighting. |
| 0x12 | Contrast | 50 | 100 | RW | White/black separation. Start at 50, increase until highlights clip, back off 5. |
| 0xB6 | Professional Mode | 1 | 9 | RW | Picture preset. 1=Standard, 2=E-Book, 3=Movie, 4=Picture. |

### Color

| Code | Name | Current | Max | R/W | How to Configure |
|------|------|---------|-----|-----|------------------|
| 0x14 | Color Preset | 5 | 14 | RW | White point. 5=6500K. 11=User 1 (unlocks manual RGB). |
| 0x0C | Color Temperature | 70 | 255 | RW | Fine kelvin within preset. |
| 0x16 | Red Gain | 50 | 100 | RW | Red channel. 50=factory default. |
| 0x18 | Green Gain | 50 | 100 | RW | Green channel. 50=factory default. |
| 0x1A | Blue Gain | 50 | 100 | RW | Blue channel. 50=factory default. |
| 0x59 | 6-Axis Saturation Red | 50 | 100 | RW | Red vividness. 50=neutral. |
| 0x5A | 6-Axis Saturation Yellow | 50 | 100 | RW | Yellow vividness. 50=neutral. |
| 0x5B | 6-Axis Saturation Green | 50 | 100 | RW | Green vividness. 50=neutral. |
| 0x5C | 6-Axis Saturation Cyan | 50 | 100 | RW | Cyan vividness. 50=neutral. |
| 0x5D | 6-Axis Saturation Blue | 50 | 100 | RW | Blue vividness. 50=neutral. |
| 0x5E | 6-Axis Saturation Magenta | 50 | 100 | RW | Magenta vividness. 50=neutral. |
| 0x9B | 6-Axis Hue Red | 50 | 100 | RW | Red tint shift. 50=neutral. |
| 0x9C | 6-Axis Hue Yellow | 50 | 100 | RW | Yellow tint shift. 50=neutral. |
| 0x9D | 6-Axis Hue Green | 50 | 100 | RW | Green tint shift. 50=neutral. |
| 0x9E | 6-Axis Hue Cyan | 50 | 100 | RW | Cyan tint shift. 50=neutral. |
| 0x9F | 6-Axis Hue Blue | 50 | 100 | RW | Blue tint shift. 50=neutral. |
| 0xA0 | 6-Axis Hue Magenta | 50 | 100 | RW | Magenta tint shift. 50=neutral. |

### Game

| Code | Name | Current | Max | R/W | How to Configure |
|------|------|---------|-----|-----|------------------|
| 0xB2 | Adaptive-Sync | 1 | 8 | RW | FreeSync/G-Sync toggle. 1=enabled. Experimental. |

### Audio

| Code | Name | Current | Max | R/W | How to Configure |
|------|------|---------|-----|-----|------------------|
| 0x62 | Volume | 30 | 100 | RW | Speaker volume. 0=mute, 100=max. |
| 0x8D | Audio Treble | 2 | 2 | RW | Fixed at 2. Not adjustable. |

### System

| Code | Name | Current | Max | R/W | How to Configure |
|------|------|---------|-----|-----|------------------|
| 0x60 | Input Source | 15 | 19 | RW | DANGEROUS. 15=DP1, 16=DP2, 17=HDMI1, 18=HDMI2. |
| 0xCC | OSD Language | 2 | 38 | RW | 2=English. 1-14 per MCCS, vendor extends to 38. |
| 0xD6 | Power Mode | 1 | 5 | RW | DANGEROUS. 1=On. 2-5 turn off the monitor. |
| 0xC6 | Power Mode (alt) | 90 | 255 | RW | Vendor power state. 90=normal. |

### Info (read-only)

| Code | Name | Value | Purpose |
|------|------|-------|---------|
| 0xAC | Horizontal Frequency | 15548 | Signal scan rate |
| 0xAE | Refresh Rate | 32000 (=320 Hz) | Current refresh rate x100 |
| 0xC8 | Controller ID | 9 | Scaler chip ID |
| 0xC9 | Firmware Level | 46507 | Firmware version |
| 0xDF | VCP Version | 514 (=2.2) | DDC protocol version |

### Internal / DDC Protocol

| Code | Name | Value | Purpose |
|------|------|-------|---------|
| 0x00 | Code Page | 0 | VCP page select. Always 0. |
| 0x02 | New Control Value | 1 | Flag for OSD changes. Write 1 to acknowledge. |
| 0x52 | Active Control | 0 | Currently active OSD control. |

### Write-Only (respond but no read value)

| Code | Name | Effect |
|------|------|--------|
| 0x04 | Restore Factory Defaults | Resets ALL settings. Irreversible. |
| 0x05 | Restore Luminance/Contrast | Resets brightness/contrast only. |
| 0x06 | Restore Color Defaults | Resets all color settings. |
| 0x08 | Restore Factory Defaults (alt) | Same as 0x04. |
| 0x1E | Auto Setup | Auto-adjust for VGA. No effect on DP/HDMI. |
| 0xA2 | Auto Color Setup | Triggers auto color calibration. |

### Manufacturer-Specific

| Code | Value | Max | Notes |
|------|-------|-----|-------|
| 0xE6 | 0 | 0 | Responds but function unknown. |

---

## Monitor Comparison (Mon 1 vs Mon 2)

All color codes identical. Only differences:

| Code | Mon 1 | Mon 2 |
|------|-------|-------|
| 0x60 | 15 (DP1) | 16 (DP2) |
| 0xCC | 2 (English) | 13 (Czech) |

Total responding codes: 35 readable + 6 write-only + 1 mfg = 42 codes.
