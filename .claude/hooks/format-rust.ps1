# PostToolUse: format a Rust file after Claude edits it. Never blocks.
try {
    $payload = [Console]::In.ReadToEnd() | ConvertFrom-Json
} catch {
    exit 0
}

$file = [string]$payload.tool_input.file_path
if ($file -like '*.rs' -and (Test-Path -LiteralPath $file)) {
    rustfmt --edition 2024 $file 2>$null | Out-Null
}
exit 0
