# PreToolUse guard: block DDC/CI commands that can black out or reset a monitor,
# and keep agent-desktop on the SysVane window. A guardrail, not a security boundary.
$ErrorActionPreference = 'Stop'

try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { exit 0 }
    $payload = $raw | ConvertFrom-Json
} catch {
    exit 0
}

if ($env:SYSVANE_ALLOW_UNSAFE -eq '1') { exit 0 }

$cmd = [string]$payload.tool_input.command
if ([string]::IsNullOrWhiteSpace($cmd)) { exit 0 }
$c = $cmd.ToLowerInvariant()

function Stop-Unsafe([string]$reason) {
    [Console]::Error.WriteLine("Blocked by .claude/hooks/guard-monitor.ps1: $reason")
    [Console]::Error.WriteLine("No write was sent. Set SYSVANE_ALLOW_UNSAFE=1 only if the user accepts losing the display.")
    exit 2
}

function Stop-Desktop([string]$reason) {
    [Console]::Error.WriteLine("Blocked by .claude/hooks/guard-monitor.ps1: $reason")
    [Console]::Error.WriteLine("agent-desktop may only operate SysVane, without physical input. See AGENTS.md.")
    exit 2
}

# Command position only (start, after ; & | ( quote or -command), so prose in a heredoc does not trip it.
if ($c -match '(^|[;&|(''"`\n]|-command|-c)\s*(&\s*)?agent-desktop(\.exe)?\b' -and $c -notmatch '--help') {
    if ($c -match '\b(close-app|batch|mcp|open-system-surface|clipboard-(get|set|clear)|list-notifications|dismiss-notification|dismiss-all-notifications|notification-action)\b') {
        Stop-Desktop 'this command reaches outside the SysVane window.'
    }
    if ($c -match '--(force|headed)\b') {
        Stop-Desktop '--force and --headed can send physical input to whatever has focus.'
    }
    $apps = [regex]::Matches($c, '--app[= ]+["'']?([^\s"'']+)')
    foreach ($app in $apps) {
        if ($app.Groups[1].Value -notmatch '^sysvane(-ui)?(\.exe)?$') {
            Stop-Desktop "--app $($app.Groups[1].Value) is not SysVane."
        }
    }
    if ($c -match '\b(snapshot|screenshot|find|press)\b' -and $apps.Count -eq 0 -and $c -notmatch '--root') {
        Stop-Desktop 'name the app with --app sysvane-ui.exe.'
    }
    if ($c -match '\blaunch\b' -and $c -notmatch '\blaunch\s+["'']?\S*sysvane-ui') {
        Stop-Desktop 'only sysvane-ui may be launched.'
    }
}

$isDdcCli = $c -match '\bsysvane-ddc\b' -or $c -match 'sysvane_ddc[./\\]cli'
$isSet = $c -match '\bset(-value)?\b'

if ($isDdcCli -and $isSet) {
    if ($c -match '0x0[4568]\b' -or $c -match '\bset(-value)?\s+(0x)?0[4568]\b') {
        Stop-Unsafe 'factory-reset codes 0x04/0x05/0x06/0x08 are disabled.'
    }
    if ($c -match '0x60\b' -or $c -match '\bset(-value)?\s+60\b') {
        Stop-Unsafe 'input-source writes (0x60) can black out the display.'
    }
    if ($c -match '0x(c6|d6)\b' -or $c -match '\bset(-value)?\s+(c6|d6)\b') {
        Stop-Unsafe 'power writes (0xC6/0xD6) can turn the display off.'
    }
    if ($c -match '--confirm-input') {
        Stop-Unsafe '--confirm-input allows an input switch that can black out the display.'
    }
}

if ($c -match 'monitorcontrol|set_vcp') {
    if ($c -match 'set_input_source|set_power_mode') {
        Stop-Unsafe 'direct input/power writes are disabled during development.'
    }
    if ($c -match 'set_vcp' -and $c -match '0x(60|c6|d6|0[4568])\b') {
        Stop-Unsafe 'direct writes to unsafe VCP codes are disabled.'
    }
}

if ($c -match 'apply-(dual|profile)' -and $c -match '--include-input') {
    Stop-Unsafe 'profile input switching is disabled during development.'
}

exit 0
