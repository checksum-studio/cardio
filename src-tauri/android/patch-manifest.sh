#!/usr/bin/env bash
# Patches the generated AndroidManifest.xml with foreground service declarations.
# Run after `tauri android init`.

set -euo pipefail

MANIFEST="src-tauri/gen/android/app/src/main/AndroidManifest.xml"

if [ ! -f "$MANIFEST" ]; then
  echo "Error: $MANIFEST not found. Run 'tauri android init' first."
  exit 1
fi

# Add permissions before <application> if not already present
if ! grep -q "FOREGROUND_SERVICE_CONNECTED_DEVICE" "$MANIFEST"; then
  sed -i 's|<application|<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />\n    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_CONNECTED_DEVICE" />\n    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />\n\n    <application|' "$MANIFEST"
fi

# Add service and receiver inside <application> if not already present
if ! grep -q "ForegroundService" "$MANIFEST"; then
  sed -i 's|</application>|<service\n            android:name=".ForegroundService"\n            android:exported="false"\n            android:foregroundServiceType="connectedDevice" />\n\n        <receiver\n            android:name=".StopReceiver"\n            android:exported="false">\n            <intent-filter>\n                <action android:name="space.checksum.cardio.ACTION_STOP" />\n            </intent-filter>\n        </receiver>\n\n    </application>|' "$MANIFEST"
fi

echo "AndroidManifest.xml patched successfully."
