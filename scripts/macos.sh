#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
APP_NAME="Stig View"
APP_BUNDLE="StigView.app"
DMG="$PROJECT_ROOT/stig-view.dmg"
STAGING="$PROJECT_ROOT/dmg-staging"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
echo "==> Building stig-view $VERSION..."
cargo build --release

# ---------------------------------------------------------------------------
# Assemble .app bundle
# ---------------------------------------------------------------------------
echo "==> Assembling $APP_BUNDLE..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp target/release/stig-view "$APP_BUNDLE/Contents/MacOS/stig-view"

# Generate .icns from individual PNG sizes
ICONSET="$PROJECT_ROOT/StigView.iconset"
mkdir -p "$ICONSET"

# Map individual logo PNGs into the iconset at exact sizes
# Retina (@2x) sizes use the next resolution up
cp assets/logo/logo-16.png      "$ICONSET/icon_16x16.png"
cp assets/logo/logo-32.png      "$ICONSET/icon_16x16@2x.png"
cp assets/logo/logo-32.png      "$ICONSET/icon_32x32.png"
cp assets/logo/logo-64.png      "$ICONSET/icon_32x32@2x.png"
cp assets/logo/logo-128.png     "$ICONSET/icon_128x128.png"
cp assets/logo/logo-256.png     "$ICONSET/icon_128x128@2x.png"
cp assets/logo/logo-256.png     "$ICONSET/icon_256x256.png"
cp assets/logo/logo-512.png     "$ICONSET/icon_256x256@2x.png"
cp assets/logo/logo-512.png     "$ICONSET/icon_512x512.png"
cp assets/logo/logo-1024.png    "$ICONSET/icon_512x512@2x.png"

iconutil -c icns "$ICONSET" -o "$APP_BUNDLE/Contents/Resources/stig-view.icns"
rm -rf "$ICONSET"

cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>stig-view</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.joshuardecker.stig-view</string>
    <key>CFBundleName</key>
    <string>Stig View</string>
    <key>CFBundleDisplayName</key>
    <string>Stig View</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>stig-view</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# ---------------------------------------------------------------------------
# Sign (optional — skipped if credentials are not set)
# ---------------------------------------------------------------------------
if [[ -n "${SIGN_IDENTITY:-}" ]]; then
    echo "==> Signing $APP_BUNDLE..."
    codesign --deep --force --options runtime --sign "$SIGN_IDENTITY" "$APP_BUNDLE"
else
    echo "==> Skipping signing (SIGN_IDENTITY not set)"
fi

# ---------------------------------------------------------------------------
# Package DMG
# ---------------------------------------------------------------------------
echo "==> Creating DMG..."
rm -rf "$STAGING"
mkdir -p "$STAGING"
cp -r "$APP_BUNDLE" "$STAGING/"
ln -s /Applications "$STAGING/Applications"

hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$STAGING" \
    -ov \
    -format UDZO \
    "$DMG"

rm -rf "$STAGING" "$APP_BUNDLE"

echo "==> Done: $DMG"
