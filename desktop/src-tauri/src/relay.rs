use crate::core_proc::spawn_core;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use sysvane_wire::{
    COMMAND_PIPE_NAME, Command as CoreCommand, ControlId, ControlValue, DeviceId, PIPE_NAME,
    PROTOCOL,
};
use tauri::{AppHandle, Emitter};

/// Relays the core's JSON lines to the webview. The UI never touches hardware.
pub fn pump(app: AppHandle) {
    let mut first = true;
    loop {
        match File::open(PIPE_NAME) {
            Ok(pipe) => {
                let _ = app.emit("core-status", true);
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    let _ = app.emit("snapshot", line);
                }
                let _ = app.emit("core-status", false);
            }
            // Only a core that is missing at launch is started. Later on a missing core means
            // the user chose Quit in the tray.
            Err(_) if first => {
                spawn_core(&app);
            }
            Err(_) => {}
        }
        first = false;
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// One command for the core. The webview can only name a device, a control and a value of the wire types;
/// the core checks them again. The core answers in its next snapshot, not here.
#[tauri::command]
pub fn send_command(
    device: DeviceId,
    control: ControlId,
    value: ControlValue,
) -> Result<(), String> {
    use std::io::Write;
    let line = serde_json::to_string(&CoreCommand {
        protocol: PROTOCOL,
        device,
        control,
        value,
    })
    .map_err(|e| e.to_string())?;
    let mut pipe = File::options()
        .write(true)
        .open(COMMAND_PIPE_NAME)
        .map_err(|e| e.to_string())?;
    writeln!(pipe, "{line}").map_err(|e| e.to_string())
}
