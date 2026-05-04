$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $repoRoot
try {
    cargo build -p des-ui-egui --bin des-ui-dev
    & ".\target\debug\des-ui-dev.exe"
}
finally {
    Pop-Location
}
