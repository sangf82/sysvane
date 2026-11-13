// GUI subsystem always: no console window when started from the tray, the Run key or the UI.
#![windows_subsystem = "windows"]

mod collector;
mod cputemp;
mod gpu;
mod hub;
mod install;
mod latest;
mod log;
mod mode;
mod perf;
mod pipe;
mod tray;
mod win;
mod worker;

use mode::Mode;
use std::sync::Arc;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError};
use windows_sys::Win32::System::Threading::CreateMutexW;

/// A second core would share the pipe and double the sampling, so it exits at once.
fn already_running() -> bool {
    let name = win::wide("Local\\SysVane-core");
    // SAFETY: `name` is NUL-terminated. The handle is kept for the whole process life on purpose.
    unsafe {
        CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        GetLastError() == ERROR_ALREADY_EXISTS
    }
}

fn main() {
    // `cargo run` still prints to the terminal that launched it.
    #[cfg(debug_assertions)]
    // SAFETY: plain Win32 call; failing (no parent console) is fine.
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
    }
    match std::env::args().nth(1).as_deref().and_then(Mode::parse) {
        Some(Mode::GpuWorker) => return gpu::worker_main(),
        Some(Mode::TempWorker) => return cputemp::worker_main(),
        Some(Mode::RegisterTask) => return install::run_register(),
        Some(Mode::UnregisterTask) => return install::run_unregister(),
        None => {}
    }
    if already_running() {
        return;
    }
    log::init();
    log::info(concat!("core ", env!("CARGO_PKG_VERSION"), " started"));
    let hub = Arc::new(hub::Hub::default());
    let h = hub.clone();
    std::thread::spawn(move || pipe::serve(h, sysvane_wire::PIPE_NAME));
    std::thread::spawn(move || {
        let make = || collector::Collector::new().expect("sysinfo cannot start on this system");
        hub.run(sysvane_wire::TICK, make, |c, dt| {
            serde_json::to_string(&c.sample(dt)).expect("snapshot serializes")
        })
    });
    tray::run();
}
