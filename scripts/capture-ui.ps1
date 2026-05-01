param(
    [string]$Out = "target\ui-shots\studio.png",
    [int]$Width = 1320,
    [int]$Height = 780,
    [switch]$DebugOverlay,
    [string]$RootId = "",
    [string]$WorkspaceId = "",
    [string]$ProjectId = "",
    [string]$GroupId = "",
    [string]$FlowId = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$outputPath = $Out
if (-not [System.IO.Path]::IsPathRooted($outputPath)) {
    $outputPath = Join-Path $repoRoot $outputPath
}
$outputPath = [System.IO.Path]::GetFullPath($outputPath)
$outputDir = Split-Path -Parent $outputPath

New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
if (Test-Path $outputPath) {
    Remove-Item -LiteralPath $outputPath
}

$env:EFRAME_SCREENSHOT_TO = $outputPath
$env:DES_UI_HARNESS_WIDTH = [string]$Width
$env:DES_UI_HARNESS_HEIGHT = [string]$Height
$env:DES_UI_HARNESS_TITLE = "Data Engine Studio UI Harness"
if ($DebugOverlay) {
    $env:DES_UI_DEBUG_OVERLAY = "1"
}
if ($RootId) {
    $env:DES_UI_SELECTED_ROOT = $RootId
}
if ($WorkspaceId) {
    $env:DES_UI_SELECTED_WORKSPACE = $WorkspaceId
}
if ($ProjectId) {
    $env:DES_UI_SELECTED_PROJECT = $ProjectId
}
if ($GroupId) {
    $env:DES_UI_SELECTED_GROUP = $GroupId
}
if ($FlowId) {
    $env:DES_UI_SELECTED_FLOW = $FlowId
}

try {
    cargo run -p des-ui-egui --features ui-screenshot --bin des-ui-shot
}
finally {
    Remove-Item Env:\EFRAME_SCREENSHOT_TO -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_HARNESS_WIDTH -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_HARNESS_HEIGHT -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_HARNESS_TITLE -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_DEBUG_OVERLAY -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_SELECTED_ROOT -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_SELECTED_WORKSPACE -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_SELECTED_PROJECT -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_SELECTED_GROUP -ErrorAction SilentlyContinue
    Remove-Item Env:\DES_UI_SELECTED_FLOW -ErrorAction SilentlyContinue
}

if (-not (Test-Path $outputPath)) {
    throw "UI screenshot was not created: $outputPath"
}

Write-Output $outputPath
