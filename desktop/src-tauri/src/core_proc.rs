use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use sysvane_wire::{CORE_EXE, CORE_TASK};
use tauri::{AppHandle, Manager, State};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// How the last core start went, for the window to ask about; an event could fire before it listens.
pub struct CoreStart(Mutex<&'static str>);

impl Default for CoreStart {
    fn default() -> Self {
        Self(Mutex::new("ok"))
    }
}

/// Starts the core. The installed core runs from a scheduled task that carries its administrator
/// rights, so the task is asked first. A development build has no task, so its exe is started directly.
/// The core allows one instance, so starting it while it is already up is harmless.
/// Returns "ok", "unelevated" (the task would not run, so the core started without administrator
/// rights and cannot read CPU temperature) or "failed" (the core did not start at all).
pub fn spawn_core(app: &AppHandle) -> &'static str {
    let status = start();
    *app.state::<CoreStart>().0.lock().unwrap() = status;
    status
}

fn start() -> &'static str {
    // A debug build runs the core built next to it. The task points at the installed copy, so asking it
    // would put the old core behind the new window.
    #[cfg(debug_assertions)]
    let task: std::io::Result<std::process::ExitStatus> = Err(std::io::ErrorKind::Other.into());
    #[cfg(not(debug_assertions))]
    let task = Command::new("schtasks")
        .args(["/run", "/tn", CORE_TASK])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if task.is_ok_and(|s| s.success()) {
        return "ok";
    }
    match std::env::current_exe()
        .ok()
        .map(|p| p.with_file_name(CORE_EXE))
        .map(|core| Command::new(core).spawn())
    {
        // A debug build is expected to run without the task, so it is not a problem there.
        Some(Ok(_)) if cfg!(debug_assertions) => "ok",
        Some(Ok(_)) => "unelevated",
        _ => "failed",
    }
}

/// For the dashboard's "Start sysvane-core" button.
#[tauri::command]
pub fn start_core(app: AppHandle) -> &'static str {
    spawn_core(&app)
}

/// Restarts the core with administrator rights after the user accepts the Windows prompt: an elevated
/// PowerShell enables the scheduled task (or registers it when it is missing), then the unelevated core
/// is stopped and the task starts the elevated one. Nothing here takes a value from the webview.
#[tauri::command]
pub fn restart_core_elevated(app: AppHandle) -> &'static str {
    let core = std::env::current_exe()
        .ok()
        .map(|p| p.with_file_name(CORE_EXE))
        .map(|p| p.display().to_string().replace('\'', "''''"))
        .unwrap_or_default();
    let inner = format!(
        "schtasks /query /tn ''{CORE_TASK}'' *> $null; if ($LASTEXITCODE -eq 0) {{ schtasks /change /tn ''{CORE_TASK}'' /enable }} else {{ & ''{core}'' --register-task }}"
    );
    let outer = format!(
        "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '-NoProfile','-Command','{inner}'"
    );
    let approved = Command::new("powershell")
        .args(["-NoProfile", "-Command", &outer])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .is_ok_and(|s| s.success());
    if !approved {
        return "unelevated";
    }
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", CORE_EXE])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    std::thread::sleep(Duration::from_secs(1));
    spawn_core(&app)
}

#[tauri::command]
pub fn core_start_status(state: State<CoreStart>) -> &'static str {
    *state.0.lock().unwrap()
}
