//! Named-pipe server: one JSON line per snapshot to each connected client. Only the current user,
//! administrators and SYSTEM may connect, so other users on the PC cannot read what the core sends.

use crate::hub::Hub;
use crate::win::wide;
use std::fs::File;
use std::io::Write;
use std::os::windows::io::FromRawHandle;
use std::sync::Arc;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_OUTBOUND;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Security attributes for a pipe: read for the current user, full access for administrators and SYSTEM.
pub struct Access {
    descriptor: PSECURITY_DESCRIPTOR,
    pub attributes: SECURITY_ATTRIBUTES,
}

impl Access {
    /// `rights` is the SDDL right string for the current user: `GR` to read the data pipe, `GRGW` to write commands (a client also needs to read attributes to open a pipe).
    /// The Medium label lets the UI, a normal-rights process, write to a pipe made by the elevated core.
    pub fn current_user(rights: &str) -> Option<Self> {
        let sddl = wide(&format!(
            "D:(A;;GA;;;SY)(A;;GA;;;BA)(A;;{rights};;;{})S:(ML;;NW;;;ME)",
            current_user_sid()?
        ));
        let mut descriptor = std::ptr::null_mut();
        // SAFETY: `sddl` is NUL-terminated; the descriptor is freed in Drop.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        };
        (ok != 0).then(|| Self {
            descriptor,
            attributes: SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: 0,
            },
        })
    }
}

impl Drop for Access {
    fn drop(&mut self) {
        // SAFETY: allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW, freed once.
        unsafe { LocalFree(self.descriptor) };
    }
}

/// The SID of the user this process runs as, in string form (`S-1-5-21-...`).
pub fn current_user_sid() -> Option<String> {
    // SAFETY: each buffer is sized by the preceding call; handles and strings are freed on every path.
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return None;
        }
        let mut len = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut len);
        let mut buf = vec![0u8; len as usize];
        let got = GetTokenInformation(token, TokenUser, buf.as_mut_ptr().cast(), len, &mut len);
        CloseHandle(token);
        if got == 0 {
            return None;
        }
        let user = &*(buf.as_ptr() as *const TOKEN_USER);
        let mut text = std::ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut text) == 0 {
            return None;
        }
        let mut n = 0;
        while *text.add(n) != 0 {
            n += 1;
        }
        let sid = String::from_utf16_lossy(std::slice::from_raw_parts(text, n));
        LocalFree(text.cast());
        Some(sid)
    }
}

/// Accepts clients forever. Each client gets its own writer thread.
pub fn serve(hub: Arc<Hub>, pipe_name: &str) -> ! {
    let name = wide(pipe_name);
    let access = loop {
        match Access::current_user("GR") {
            Some(access) => break access,
            None => {
                crate::log::error("cannot build the pipe access rules; not serving");
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
            crate::log::error(&format!("CreateNamedPipeW failed: {}", unsafe {
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
