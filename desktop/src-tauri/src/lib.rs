mod core_proc;
mod icon;
mod relay;
mod window;

use sysvane_wire::FLYOUT_ARG;
use tauri::Manager;

pub fn run() {
    let flyout = std::env::args().any(|a| a == FLYOUT_ARG);
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(window::Flyout(flyout))
        .manage(core_proc::CoreStart::default())
        .invoke_handler(tauri::generate_handler![
            window::is_flyout,
            window::open_dashboard,
            core_proc::start_core,
            core_proc::core_start_status,
            relay::send_command,
            core_proc::restart_core_elevated,
            icon::exe_icon
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            std::thread::spawn(move || relay::pump(handle));
            if let Some(win) = app.get_webview_window("main") {
                if flyout {
                    window::place_flyout(app.handle(), &win);
                }
                let _ = win.show();
                let _ = win.set_focus();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
