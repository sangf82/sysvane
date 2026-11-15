//! The same exe plays several roles. This is the one place that names the flags that pick them.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    /// Short-lived child that reads the NVIDIA GPU, so NVML's memory never lands in the core.
    GpuWorker,
    /// Child that reads the AMD CPU temperature through PawnIO. Elevated because its parent is.
    TempWorker,
    /// Elevated one-shot run by the installer: registers the scheduled task that starts the core.
    RegisterTask,
    UnregisterTask,
}

impl Mode {
    const ALL: [Mode; 4] = [
        Mode::GpuWorker,
        Mode::TempWorker,
        Mode::RegisterTask,
        Mode::UnregisterTask,
    ];

    pub fn flag(self) -> &'static str {
        match self {
            Mode::GpuWorker => "--gpu-worker",
            Mode::TempWorker => "--temp-worker",
            Mode::RegisterTask => "--register-task",
            Mode::UnregisterTask => "--unregister-task",
        }
    }

    /// The mode a command-line argument asks for. Anything else runs the core.
    pub fn parse(arg: &str) -> Option<Mode> {
        Self::ALL.into_iter().find(|m| m.flag() == arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_flag_parses_back_to_its_mode() {
        for mode in Mode::ALL {
            assert_eq!(Mode::parse(mode.flag()), Some(mode));
        }
    }
}
