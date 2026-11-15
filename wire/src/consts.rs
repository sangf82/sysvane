use std::time::Duration;

pub const PIPE_NAME: &str = r"\\.\pipe\sysvane-core";
/// UI to core, one JSON `Command` per line. The core never answers on it: the result shows in the next snapshot.
pub const COMMAND_PIPE_NAME: &str = r"\\.\pipe\sysvane-core-cmd";
/// A command line longer than this is refused and its client dropped.
pub const MAX_COMMAND_BYTES: usize = 512;
pub const CORE_EXE: &str = "sysvane-core.exe";
pub const UI_EXE: &str = "sysvane-ui.exe";
/// The scheduled task that starts the core with administrator rights.
pub const CORE_TASK: &str = "SysVane Core";
/// Opens the UI as the small tray flyout. Without it the UI opens the dashboard.
pub const FLYOUT_ARG: &str = "--flyout";

/// Version of the pipe protocol: the snapshot and, later, the commands. Raise it when either changes
/// shape. The UI refuses to talk to a core that reports another number.
pub const PROTOCOL: u32 = 3;

/// Time between snapshots. The UI keeps a fixed number of them, so its "2 minutes" labels depend on this.
pub const TICK: Duration = Duration::from_secs(1);
