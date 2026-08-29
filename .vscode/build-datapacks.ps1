# Build every buildable conformance case into target/datapacks/<case>.zip, for
# in-game verification (ADR-0017). Diagnostics cases (no expected/ tree) are
# compile failures by design and are skipped.
#
# Run from the VS Code task "Datapacks: build all conformance cases", or directly:
#   powershell -ExecutionPolicy Bypass -File .vscode/build-datapacks.ps1

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$out = Join-Path $repo 'target/datapacks'
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force -Path $out | Out-Null

Get-ChildItem (Join-Path $repo 'conformance/cases') -Directory | ForEach-Object {
    $case = $_.Name
    if (-not (Test-Path (Join-Path $_.FullName 'expected'))) {
        Write-Host "--- $case (diagnostics case, skipped)"
        return
    }
    Write-Host "--- $case"
    $input = Join-Path $_.FullName 'input.json'
    $zip = Join-Path $out "$case.zip"
    & cargo run -q -p packsmith-cli -- build $input --target 26.2 --output $zip
}
