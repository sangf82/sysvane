# PreToolUse guard: block DDC/CI commands that can black out or reset a monitor.
$ErrorActionPreference = 'Stop'

try {
    $raw = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($raw)) { exit 0 }
    $payload = $raw | ConvertFrom-Json
} catch {
    exit 0
}

if ($env:TOUSHOU_ALLOW_UNSAFE -eq '1') { exit 0 }

$cmd = [string]$payload.tool_input.command
if ([string]::IsNullOrWhiteSpace($cmd)) { exit 0 }
$c = $cmd.ToLowerInvariant()

function Stop-Unsafe([string]$reason) {
    [Console]::Error.WriteLine("Blocked by .claude/hooks/guard-monitor.ps1: $reason")
    [Console]::Error.WriteLine("No write was sent. Set TOUSHOU_ALLOW_UNSAFE=1 only if the user accepts losing the display.")
    exit 2
}

$isToushou = $c -match '\btoushou\b' -or $c -match 'src[./\\]cli' -or $c -match '-m\s+src\.cli'
$isSet = $c -match '\bset(-value)?\b'

if ($isToushou -and $isSet) {
    if ($c -match '0x0[4568]\b' -or $c -match '\bset(-value)?\s+(0x)?0[4568]\b') {
        Stop-Unsafe 'factory-reset codes 0x04/0x05/0x06/0x08 are disabled.'
    }
    if ($c -match '0x60\b' -or $c -match '\bset(-value)?\s+60\b') {
        Stop-Unsafe 'input-source writes (0x60) can black out the display.'
    }
    if ($c -match '0x(c6|d6)\b' -or $c -match '\bset(-value)?\s+(c6|d6)\b') {
        Stop-Unsafe 'power writes (0xC6/0xD6) can turn the display off.'
    }
    if ($c -match '--force') {
        Stop-Unsafe '--force bypasses the CLI monitor safety checks.'
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
