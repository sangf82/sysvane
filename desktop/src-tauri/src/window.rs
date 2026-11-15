use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::{AppHandle, LogicalSize, PhysicalPosition, State, WebviewWindow, WindowEvent};

const FLYOUT: (f64, f64) = (320.0, 480.0);

pub struct Flyout(pub bool);

#[tauri::command]
pub fn is_flyout(mode: State<Flyout>) -> bool {
    mode.0
}

/// Opens the full dashboard as a new process, then closes the flyout.
#[tauri::command]
pub fn open_dashboard(app: AppHandle) {
    if let Ok(exe) = std::env::current_exe()
        && Command::new(exe).spawn().is_ok()
    {
        app.exit(0);
    }
}

/// Small frameless window above the taskbar, closed when it loses focus.
pub fn place_flyout(app: &AppHandle, win: &WebviewWindow) {
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
