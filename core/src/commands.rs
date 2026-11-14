//! Command pipe, UI to core. One JSON `Command` per line; a line is checked by type (see the wire crate)
//! and queued for the device owner thread. The pipe thread never touches a device. A line that is too long
//! or not a valid command drops that client: the UI never sends one.

use crate::devices::Devices;
use crate::pipe::Access;
use crate::win::wide;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::os::windows::io::FromRawHandle;
use std::sync::Arc;
use sysvane_wire::{Command, MAX_COMMAND_BYTES};
use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_INBOUND;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

/// The next command from `reader`. `None` ends the client: closed, a line over the limit, or not a command.
fn next_command(reader: &mut impl BufRead, line: &mut Vec<u8>) -> Option<Command> {
    line.clear();
    let limit = MAX_COMMAND_BYTES as u64 + 1;
    let n = reader.by_ref().take(limit).read_until(b'\n', line).ok()?;
    if n == 0 || (n as u64 == limit && line.last() != Some(&b'\n')) {
        return None;
    }
    serde_json::from_slice(line.trim_ascii()).ok()
}

/// Accepts clients forever. Each client gets its own reader thread.
pub fn serve(devices: Arc<Devices>, pipe_name: &str) -> ! {
    let name = wide(pipe_name);
    let access = loop {
        match Access::current_user("GRGW") {
            Some(access) => break access,
            None => {
                crate::log::error("cannot build the command pipe access rules; not serving");
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    };
    loop {
        // SAFETY: `name` is NUL-terminated and `access` outlives the call.
        let handle = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                0,
                4096,
                0,
                &access.attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            // SAFETY: plain FFI call reading thread-local error state.
            crate::log::error(&format!("CreateNamedPipeW (commands) failed: {}", unsafe {
                GetLastError()
            }));
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }
        // SAFETY: valid handle just created; `File` takes ownership and closes it.
        let pipe = unsafe { File::from_raw_handle(handle as _) };
        // SAFETY: `handle` stays valid while `pipe` lives. ERROR_PIPE_CONNECTED means the client
        // connected between create and connect, which is success.
        let ok = unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) } != 0
            || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
        if !ok {
            continue;
        }
        let devices = devices.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(pipe);
            let mut line = Vec::with_capacity(MAX_COMMAND_BYTES + 1);
            while let Some(command) = next_command(&mut reader, &mut line) {
                devices.submit(command);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const GOOD: &str = r#"{"protocol":3,"device":"core","control":"autostart","value":true}"#;

    fn run(input: &[u8]) -> Vec<Command> {
        let mut reader = Cursor::new(input.to_vec());
        let mut line = Vec::new();
        std::iter::from_fn(|| next_command(&mut reader, &mut line)).collect()
    }

    #[test]
    fn reads_one_command_per_line() {
        assert_eq!(run(format!("{GOOD}\n{GOOD}\r\n").as_bytes()).len(), 2);
    }

    #[test]
    fn an_invalid_line_ends_the_client() {
        assert_eq!(
            run(format!("{GOOD}\nnot json\n{GOOD}\n").as_bytes()).len(),
            1
        );
    }

    #[test]
    fn a_line_over_the_limit_ends_the_client_without_being_read_whole() {
        let long = vec![b'x'; MAX_COMMAND_BYTES * 100];
        let mut input = long;
        input.extend_from_slice(format!("\n{GOOD}\n").as_bytes());
        assert!(run(&input).is_empty());
    }
}
