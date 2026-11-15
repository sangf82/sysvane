//! Named-pipe server: one JSON line per snapshot to each connected client. Only the current user,
//! administrators and SYSTEM may connect, so other users on the PC cannot read what the core sends.

use super::access::{Access, is_elevated};
use crate::ipc::hub::Hub;
use crate::system::win::wide;
use std::fs::File;
use std::io::Write;
use std::os::windows::io::FromRawHandle;
use std::sync::Arc;
use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_OUTBOUND;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

/// Rights the current user gets on the snapshot pipe. An elevated core keeps read only: `BA` already gives it
/// every right it needs to open the next instance. An unelevated core (dev builds, before M1) is the same
/// user as the UI, so there is no boundary to protect, and Windows refuses its second instance with error 5
/// unless the user also holds the write rights (`GR` alone, or read plus 0x4, fails; checked by test).
fn data_pipe_rights() -> &'static str {
    if is_elevated() { "GR" } else { "GRGW" }
}

/// Accepts clients forever. Each client gets its own writer thread.
pub fn serve(hub: Arc<Hub>, pipe_name: &str) -> ! {
    let name = wide(pipe_name);
    let access = loop {
        match Access::current_user(data_pipe_rights()) {
            Some(access) => break access,
            None => {
                crate::system::log::error("cannot build the pipe access rules; not serving");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };
    loop {
        // SAFETY: `name` is NUL-terminated and `access` outlives the call.
        let handle = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_OUTBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                64 * 1024,
                0,
                0,
                &access.attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            // SAFETY: plain FFI call reading thread-local error state.
            crate::system::log::error(&format!("CreateNamedPipeW failed: {}", unsafe {
                GetLastError()
            }));
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }
        // SAFETY: valid handle just created; `File` takes ownership and closes it.
        let mut pipe = unsafe { File::from_raw_handle(handle as _) };
        // SAFETY: `handle` stays valid while `pipe` lives. ERROR_PIPE_CONNECTED means
        // the client connected between create and connect, which is success.
        let ok = unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) } != 0
            || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
        if !ok {
            continue;
        }
        let rx = hub.subscribe();
        std::thread::spawn(move || {
            for line in rx {
                if pipe
                    .write_all(line.as_bytes())
                    .and_then(|_| pipe.write_all(b"\n"))
                    .is_err()
                {
                    break;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::time::{Duration, Instant};
    /// A second instance of the data pipe must open while the first is in use, whatever the elevation.
    /// An elevated CI runner passes even with `GR`, so this guards the unelevated dev case only on a normal shell.
    #[test]
    fn a_second_instance_opens_with_the_rights_chosen_for_this_process() {
        let name = wide(&format!(r"\\.\pipe\sysvane-rights-{}", std::process::id()));
        let access = Access::current_user(data_pipe_rights()).unwrap();
        let create = || {
            // SAFETY: `name` is NUL-terminated and `access` outlives the call; the handle is leaked in this test.
            let h = unsafe {
                CreateNamedPipeW(
                    name.as_ptr(),
                    PIPE_ACCESS_OUTBOUND,
                    PIPE_TYPE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                    PIPE_UNLIMITED_INSTANCES,
                    4096,
                    0,
                    0,
                    &access.attributes,
                )
            };
            h != INVALID_HANDLE_VALUE
        };
        assert!(create(), "first instance");
        assert!(create(), "second instance");
    }

    const WAIT: Duration = Duration::from_secs(5);

    fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
        let start = Instant::now();
        while !ok() {
            assert!(start.elapsed() < WAIT, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The server may still be creating its next pipe instance, so retry for a moment.
    fn connect(name: &str) -> BufReader<File> {
        let start = Instant::now();
        loop {
            match File::open(name) {
                Ok(pipe) => return BufReader::new(pipe),
                Err(e) => assert!(start.elapsed() < WAIT, "cannot connect to {name}: {e}"),
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn read_line(pipe: &mut BufReader<File>) -> String {
        let mut line = String::new();
        pipe.read_line(&mut line).unwrap();
        line
    }

    #[test]
    fn lines_reach_each_client_and_a_closed_client_is_dropped() {
        let name = format!(r"\\.\pipe\sysvane-test-{}", std::process::id());
        let hub = Arc::new(Hub::default());
        let (h, n) = (hub.clone(), name.clone());
        std::thread::spawn(move || serve(h, &n));

        let mut first = connect(&name);
        wait_until("the first client to subscribe", || hub.has_clients());
        hub.broadcast(&"one".into());
        assert_eq!(read_line(&mut first), "one\n");

        drop(first);
        wait_until("the closed client to be dropped", || {
            hub.broadcast(&"x".into());
            !hub.has_clients()
        });

        let mut second = connect(&name);
        wait_until("the second client to subscribe", || hub.has_clients());
        hub.broadcast(&"two".into());
        assert_eq!(read_line(&mut second), "two\n");
    }
}
