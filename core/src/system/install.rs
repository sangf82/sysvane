//! The scheduled task that starts the core with administrator rights. Windows does not show a UAC
//! prompt for it, and a Run key cannot start an elevated exe at all. The installer registers the task
//! once (that needs admin). "Start with Windows" adds or removes its logon trigger.

use crate::system::win::hidden;
use sysvane_wire::CORE_TASK;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn task_xml(exe: &str, user: &str, autostart: bool) -> String {
    let trigger = if autostart {
        format!(
            "<Triggers><LogonTrigger><UserId>{}</UserId></LogonTrigger></Triggers>",
            xml_escape(user)
        )
    } else {
        String::new()
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Starts the SysVane core with administrator rights.</Description></RegistrationInfo>
  {trigger}
  <Principals><Principal id="Author"><UserId>{user}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>HighestAvailable</RunLevel></Principal></Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Hidden>true</Hidden>
  </Settings>
  <Actions Context="Author"><Exec><Command>{exe}</Command></Exec></Actions>
</Task>"#,
        user = xml_escape(user),
        exe = xml_escape(exe),
    )
}

fn schtasks(args: &[&str]) -> Result<String, String> {
    let out = hidden("schtasks")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
    }
}

/// Registers (or replaces) the task for the current user, pointing at this exe. Needs administrator rights.
pub fn register(autostart: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let user = format!(
        "{}\\{}",
        std::env::var("USERDOMAIN").unwrap_or_default(),
        std::env::var("USERNAME").unwrap_or_default()
    );
    let xml = task_xml(&exe.display().to_string(), &user, autostart);
    let path = std::env::temp_dir().join("sysvane-core-task.xml");
    let mut bytes = vec![0xFF, 0xFE];
    bytes.extend(xml.encode_utf16().flat_map(u16::to_le_bytes));
    std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
    let done = schtasks(&[
        "/create",
        "/tn",
        CORE_TASK,
        "/xml",
        &path.display().to_string(),
        "/f",
    ]);
    let _ = std::fs::remove_file(&path);
    done.map(drop)
}

pub fn unregister() -> Result<(), String> {
    schtasks(&["/delete", "/tn", CORE_TASK, "/f"]).map(drop)
}

/// Whether the task starts at logon. False when there is no task.
pub fn autostart_enabled() -> bool {
    schtasks(&["/query", "/tn", CORE_TASK, "/xml"]).is_ok_and(|xml| xml.contains("<LogonTrigger"))
}

/// Entry point of `Mode::RegisterTask`, run elevated by the installer. Pass \`--autostart\` to add the logon trigger.
pub fn run_register() {
    let autostart = std::env::args().any(|a| a == "--autostart");
    let result = register(autostart);
    report("register the core task", &result);
}

/// Entry point of `Mode::UnregisterTask`, run elevated by the uninstaller.
pub fn run_unregister() {
    let result = unregister();
    report("remove the core task", &result);
}

/// The installer reads the exit code; the log keeps the reason.
fn report(what: &str, result: &Result<(), String>) {
    crate::system::log::init();
    match result {
        Ok(()) => crate::system::log::info(&format!("{what}: done")),
        Err(e) => {
            crate::system::log::error(&format!("{what}: {e}"));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_runs_the_exe_elevated_and_autostart_is_only_the_logon_trigger() {
        let off = task_xml(r"C:\R&D\sysvane-core.exe", r"PC\me", false);
        assert!(off.contains(r"C:\R&amp;D\sysvane-core.exe"));
        assert!(off.contains("<RunLevel>HighestAvailable</RunLevel>"));
        assert!(!off.contains("<Triggers"));
        assert!(
            !off.contains("<Arguments>"),
            "the task starts the core itself, not a worker"
        );
        let on = task_xml(r"C:\sysvane-core.exe", r"PC\me", true);
        assert!(on.contains("<LogonTrigger><UserId>PC\\me</UserId></LogonTrigger>"));
    }
}
