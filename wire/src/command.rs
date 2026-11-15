use serde::{Deserialize, Serialize};

/// Every device the core can be told to change. A command names one of these, never a free string.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum DeviceId {
    /// The core itself: settings that live in the core, such as starting with Windows.
    Core,
    /// Test device with no hardware behind it. Only debug builds register it.
    Fake,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ControlId {
    Autostart,
    /// 0 to 100. The fake device refuses anything else.
    Level,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(untagged)]
pub enum ControlValue {
    Bool(bool),
    Int(i64),
}

/// One line on the command pipe. Unknown fields are refused, and so is another protocol number.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub protocol: u32,
    pub device: DeviceId,
    pub control: ControlId,
    pub value: ControlValue,
}
