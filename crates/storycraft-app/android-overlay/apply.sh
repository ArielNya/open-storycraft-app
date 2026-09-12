#!/bin/sh
# Copy Kotlin helpers into the generated Android project.
# Run from repo root after: cargo tauri android init
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
gen="$root/gen/android/app/src/main/java/dev/openstorycraft/app"
if [ ! -d "$root/gen/android" ]; then
  echo "missing gen/android — run: cargo tauri android init" >&2
  exit 1
fi
mkdir -p "$gen"
cp "$root/android-overlay/JobForegroundService.kt" "$gen/"
cp "$root/android-overlay/CustomTabs.kt" "$gen/"
echo "copied Kotlin helpers to $gen"
echo "merge AndroidManifest.permissions.xml into app/src/main/AndroidManifest.xml"
echo "add androidx.browser to app/build.gradle.kts for Custom Tabs"
