//! Tray icon and its menu. Runs the Win32 message loop on the calling thread.

use crate::modules::devices::Devices;
use std::ffi::OsStr;
use std::process::Command;
use std::sync::Arc;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use sysvane_wire::{FLYOUT_ARG, UI_EXE};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, DispatchMessageW, GetMessageW, MSG, TranslateMessage,
};

/// Whether any UI window is open, however it was started. The flyout opens the dashboard itself,
/// so the core cannot track the windows it launched.
fn ui_running() -> bool {
    let Ok(mut sys) = System::new() else {
        return false;
    };
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys.processes_by_exact_name(OsStr::new(UI_EXE))
        .next()
        .is_some()
}

/// Starts the UI next to this exe, as the flyout or the dashboard, unless one is already open.
fn launch_ui(flyout: bool) {
    if ui_running() {
        return;
    }
    let Some(exe) = std::env::current_exe()
        .ok()
        .map(|p| p.with_file_name(UI_EXE))
    else {
        return;
    };
    let mut command = Command::new(exe);
    if flyout {
        command.arg(FLYOUT_ARG);
    }
    match command.spawn() {
        // The tray click gave this process the foreground; pass it on so the window can take focus.
        // SAFETY: plain Win32 call with the child's process id.
        Ok(child) => {
            unsafe { AllowSetForegroundWindow(child.id()) };
        }
        Err(e) => crate::system::log::error(&format!("cannot start {UI_EXE}: {e}")),
    }
}

/// 32x32 raw RGBA of the app icon, written by `branding/logo/scripts/build_icons.py`.
/// Raw bytes keep the core free of an image decoder.
// shortcut: one 32 px bitmap that Windows scales to the tray size; ship 16/20/24 px too if it looks soft on the taskbar.
fn icon() -> Icon {
    Icon::from_rgba(include_bytes!("../../assets/tray-32.rgba").to_vec(), 32, 32)
        .expect("icon buffer matches its size")
}

/// Zips the logs and a version note to the Desktop and selects the zip in Explorer. Nothing leaves the PC.
fn export_diagnostics() -> Result<(), String> {
    let stage = std::env::temp_dir().join("sysvane-diagnostics");
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(crate::system::log::dir())
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let _ = std::fs::copy(entry.path(), stage.join(entry.file_name()));
    }
    let note = format!(
        "SysVane core {}\nProtocol {}\nWindows {}\n",
        env!("CARGO_PKG_VERSION"),
        sysvane_wire::PROTOCOL,
        sysinfo::System::long_os_version().unwrap_or_default()
    );
    std::fs::write(stage.join("version.txt"), note).map_err(|e| e.to_string())?;
    // PowerShell knows the real Desktop folder, which OneDrive often moves away from %USERPROFILE%\Desktop.
    let out = crate::system::win::hidden("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "$z = Join-Path ([Environment]::GetFolderPath('Desktop')) 'sysvane-diagnostics.zip'; \
             Compress-Archive -Force -Path \"$env:SV_SRC\\*\" -DestinationPath $z -ErrorAction Stop; $z",
        ])
        .env("SV_SRC", &stage)
        .output()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&stage);
    if !out.status.success() {
        return Err(format!(
            "Compress-Archive failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let zip = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let _ = std::process::Command::new("explorer")
        .arg(format!("/select,{zip}"))
        .spawn();
    Ok(())
}

pub fn run(devices: Arc<Devices>) {
    let open = MenuItem::new("Open dashboard", true, None);
    let auto = CheckMenuItem::new("Start with Windows", true, devices.autostart(), None);
    let logs = MenuItem::new("Open logs folder", true, None);
    let diag = MenuItem::new("Export diagnostics to Desktop", true, None);
    let quit = MenuItem::new("Quit", true, None);
    let menu = Menu::new();
    menu.append_items(&[
        &open,
        &auto,
        &logs,
        &diag,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .expect("tray menu builds");
    let (open_id, auto_id, quit_id) = (open.id().clone(), auto.id().clone(), quit.id().clone());
    let (logs_id, diag_id) = (logs.id().clone(), diag.id().clone());

    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        if e.id == open_id {
            launch_ui(false);
        } else if e.id == auto_id {
            // The check mark already flipped, so set the trigger to the opposite of what the core holds.
            if let Err(err) = devices.set_autostart(!devices.autostart()) {
                crate::system::log::error(&format!("autostart change failed: {err}"));
            }
        } else if e.id == logs_id {
            let _ = std::process::Command::new("explorer")
                .arg(crate::system::log::dir())
                .spawn();
        } else if e.id == diag_id {
            if let Err(err) = export_diagnostics() {
                crate::system::log::error(&format!("diagnostics export failed: {err}"));
            }
        } else if e.id == quit_id {
            std::process::exit(0);
        }
    }));
    TrayIconEvent::set_event_handler(Some(|e: TrayIconEvent| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = e
        {
            launch_ui(true);
        }
    }));

    let _tray = TrayIconBuilder::new()
        .with_icon(icon())
        .with_tooltip("SysVane")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()
        .expect("tray icon builds");

    // SAFETY: standard message pump; `msg` is only read by the calls below.
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
