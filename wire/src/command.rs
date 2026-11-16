use serde::{Deserialize, Serialize};

/// Every device the core can be told to change. A command names one of these, never a free string.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DeviceId {
    /// The core itself: settings that live in the core, such as starting with Windows.
    Core,
    /// Test device with no hardware behind it. Only debug builds register it.
    Fake,
    /// A DDC/CI monitor, numbered in the order Windows listed them at the last scan.
    Monitor(u8),
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ControlId {
    Autostart,
    /// 0 to 100. The fake device refuses anything else.
    Level,
    /// Monitor settings. The value is checked against the maximum the monitor reports.
    Brightness,
    Contrast,
    ColorPreset,
    RedGain,
    GreenGain,
    BlueGain,
    /// Apply the profile named by a `Text` value. Only the setting's own codes are written.
    Profile,
    /// Save the monitor's current settings as a profile named by a `Text` value.
    SaveProfile,
    /// Core: detect monitors again (`Bool(true)`), for one plugged in or woken after the core started.
    Scan,
    /// Resolution and refresh rate as `Text("2560x1440@165")`, one of `Device.modes`. Temporary until kept.
    DisplayMode,
    /// `Bool(true)` keeps a trial display mode, `Bool(false)` reverts it now. While a trial runs, the
    /// monitor reports this control as the whole seconds left before the core reverts it by itself.
    KeepDisplayMode,
    /// Quarter turns clockwise from landscape, `Int(0..=3)`. A trial like `DisplayMode`, kept the same way.
    Orientation,
    /// Colour through the GPU gamma ramp, for any monitor: each `Int(0..=100)`. Brightness, contrast and
    /// gamma are unchanged at 50, the channel gains at 100.
    GpuBrightness,
    GpuContrast,
    GpuGamma,
    GpuRed,
    GpuGreen,
    GpuBlue,
    /// Every GPU colour value back to neutral (`Bool(true)`).
    GpuReset,
    /// Dims the monitor three times through the GPU gamma ramp (`Bool(true)`), to tell monitors apart.
    Identify,
    /// A monitor image setting by VCP code, for a code the monitor lists beyond the named ones, such as
    /// volume or 6-axis saturation. A percentage of its maximum; the core takes only listed, allowlisted codes.
    Vcp(u8),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum ControlValue {
    Bool(bool),
    Int(i64),
    /// A name, such as a profile's. Limited by `MAX_COMMAND_BYTES`; the module validates it.
    Text(String),
}

/// One line on the command pipe. Unknown fields are refused, and so is another protocol number.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub protocol: u32,
    pub device: DeviceId,
    pub control: ControlId,
    pub value: ControlValue,
}
