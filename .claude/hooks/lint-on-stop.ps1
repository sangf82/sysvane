# Stop hook: lint what changed this session. Blocks the stop (exit 2) only when lint fails.
# Rust changed -> cargo fmt --check + clippy -D warnings. desktop/src changed -> tsc --noEmit.
try {
    $payload = [Console]::In.ReadToEnd() | ConvertFrom-Json
} catch {
    exit 0
}
# A second pass after a block must not loop.
if ($payload.stop_hook_active) { exit 0 }

Set-Location -LiteralPath $env:CLAUDE_PROJECT_DIR
$changed = git status --porcelain 2>$null | ForEach-Object { $_.Substring(3) }
# Lint only files touched since the last clean pass, not the whole dirty tree.
$stamp = Join-Path $env:TEMP 'sysvane-lint.stamp'
$since = if (Test-Path $stamp) { (Get-Item $stamp).LastWriteTime } else { [datetime]::MinValue }
$changed = $changed | Where-Object { (Test-Path -LiteralPath $_) -and (Get-Item -LiteralPath $_).LastWriteTime -gt $since }
$rust = $changed | Where-Object { $_ -match '\.rs$|Cargo\.toml$' }
$ts = $changed | Where-Object { $_ -match '^desktop/src/.*\.(ts|tsx)$' }
$errors = @()

if ($rust) {
    $out = cargo fmt --all -- --check 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) { $errors += "cargo fmt --check:`n$out" }
    $out = cargo clippy -p sysvane-core -p sysvane-wire --all-targets -- -D warnings 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) { $errors += "cargo clippy:`n$out" }
}
if ($ts) {
    Push-Location desktop
    $out = npx tsc --noEmit 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) { $errors += "tsc --noEmit:`n$out" }
    Pop-Location
}

if ($errors) {
    [Console]::Error.WriteLine(($errors -join "`n").Trim())
    exit 2
}
New-Item -ItemType File -Force $stamp | Out-Null
exit 0
