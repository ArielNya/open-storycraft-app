# Copy Kotlin helpers into the generated Android project (Windows twin of apply.sh).
# Run after: cargo tauri android init
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$gen = Join-Path $root 'gen\android\app\src\main\java\dev\openstorycraft\app'
if (-not (Test-Path (Join-Path $root 'gen\android'))) {
    Write-Error 'missing gen\android - run: cargo tauri android init'
}
New-Item -ItemType Directory -Force $gen | Out-Null
foreach ($file in 'JobForegroundService.kt', 'CustomTabs.kt', 'StorycraftPlugin.kt') {
    Copy-Item (Join-Path $PSScriptRoot $file) $gen
}
Write-Output "copied Kotlin helpers to $gen"
Write-Output 'merge AndroidManifest.permissions.xml into app\src\main\AndroidManifest.xml'
Write-Output 'add androidx.browser to app\build.gradle.kts for Custom Tabs'
