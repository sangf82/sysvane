# Usage: powershell -NoProfile -File scripts\bump-version.ps1 0.0.3
# One version for the whole workspace: Cargo.toml and desktop/package.json. Does not commit or tag.
param([Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+(-[0-9A-Za-z.]+)?$')][string]$Version)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent

$cargo = Join-Path $root 'Cargo.toml'
$text = [IO.File]::ReadAllText($cargo)
$new = [regex]::Replace($text, '(?m)^version = "[^"]+"', "version = `"$Version`"", 1)
if ($new -eq $text) { throw 'Cargo.toml: workspace version not found or unchanged' }
[IO.File]::WriteAllText($cargo, $new)

$pkg = Join-Path $root 'desktop\package.json'
$text = [IO.File]::ReadAllText($pkg)
$new = [regex]::Replace($text, '(?m)^(\s*)"version": "[^"]+"', "`$1`"version`": `"$Version`"", 1)
[IO.File]::WriteAllText($pkg, $new)

Push-Location $root
cargo check -q -p sysvane-core -p sysvane-wire *> $null   # refreshes Cargo.lock
if ($LASTEXITCODE -ne 0) { throw 'cargo check failed after the bump' }
Pop-Location
Write-Output "Bumped to $Version. Next: commit, then 'git tag v$Version' and 'git push --tags'."
