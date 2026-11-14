#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use sysvane_wire::{CORE_EXE, CORE_TASK, FLYOUT_ARG, PIPE_NAME};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, State, WebviewWindow, WindowEvent,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const FLYOUT: (f64, f64) = (320.0, 480.0);

struct Flyout(bool);

/// Starts the core. The installed core runs from a scheduled task that carries its administrator
/// rights, so the task is asked first. A development build has no task, so its exe is started directly.
/// The core allows one instance, so starting it while it is already up is harmless.
fn spawn_core() {
    let task = Command::new("schtasks")
        .args(["/run", "/tn", CORE_TASK])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    if task.is_ok_and(|s| s.success()) {
        return;
    }
    if let Some(core) = std::env::current_exe()
        .ok()
        .map(|p| p.with_file_name(CORE_EXE))
    {
        let _ = Command::new(core).spawn();
    }
}

/// Relays the core's JSON lines to the webview. The UI never touches hardware.
fn pump(app: AppHandle) {
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
            Err(_) if first => spawn_core(),
            Err(_) => {}
        }
        first = false;
        std::thread::sleep(Duration::from_secs(1));
    }
}

/// For the dashboard's "Start sysvane-core" button.
#[tauri::command]
fn start_core() {
    spawn_core();
}

#[tauri::command]
fn is_flyout(mode: State<Flyout>) -> bool {
    mode.0
}

/// Opens the full dashboard as a new process, then closes the flyout.
#[tauri::command]
fn open_dashboard(app: AppHandle) {
    if let Ok(exe) = std::env::current_exe()
        && Command::new(exe).spawn().is_ok()
    {
        app.exit(0);
    }
}

/// Small frameless window above the taskbar, closed when it loses focus.
fn place_flyout(app: &AppHandle, win: &WebviewWindow) {
    let _ = win.set_decorations(false);
    let _ = win.set_always_on_top(true);
    let _ = win.set_skip_taskbar(true);
    let _ = win.set_resizable(false);
    let _ = win.set_min_size(Some(LogicalSize::new(FLYOUT.0, FLYOUT.1)));
    let _ = win.set_size(LogicalSize::new(FLYOUT.0, FLYOUT.1));
    if let Ok(Some(monitor)) = win.primary_monitor() {
        let area = monitor.work_area();
        let scale = win.scale_factor().unwrap_or(1.0);
        let (w, h, gap) = (
            (FLYOUT.0 * scale) as i32,
            (FLYOUT.1 * scale) as i32,
            (12.0 * scale) as i32,
        );
        let _ = win.set_position(PhysicalPosition::new(
            area.position.x + area.size.width as i32 - w - gap,
            area.position.y + area.size.height as i32 - h - gap,
        ));
    }
    // Right after a tray click the taskbar takes the foreground back. During the first 1.5 s
    // a focus loss is taken back instead of closing, and a window that never had focus stays open.
    let opened = std::time::Instant::now();
    let had_focus = Arc::new(AtomicBool::new(false));
    let (app, same_win) = (app.clone(), win.clone());
    win.on_window_event(move |event| {
        if let WindowEvent::Focused(focused) = event {
            if *focused {
                had_focus.store(true, Ordering::Relaxed);
            } else if opened.elapsed() < Duration::from_millis(1500) {
                let _ = same_win.set_focus();
            } else if had_focus.load(Ordering::Relaxed) {
                app.exit(0);
            }
        }
    });
}

fn main() {
    let flyout = std::env::args().any(|a| a == FLYOUT_ARG);
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(Flyout(flyout))
        .invoke_handler(tauri::generate_handler![
            is_flyout,
            open_dashboard,
            start_core
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || pump(handle));
            if let Some(win) = app.get_webview_window("main") {
                if flyout {
                    place_flyout(app.handle(), &win);
                }
                let _ = win.show();
                let _ = win.set_focus();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
