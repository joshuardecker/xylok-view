#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "==> Building stig-view..."
cargo build --release

# ---------------------------------------------------------------------------
# Flatpak
# ---------------------------------------------------------------------------
if command -v flatpak-builder &>/dev/null; then
    REPO_DIR="$PROJECT_ROOT/flatpak-repo"
    BUILD_DIR="$PROJECT_ROOT/flatpak-build"
    BUNDLE="$PROJECT_ROOT/stig-view.flatpak"

    echo "==> Assembling Flatpak..."
    flatpak-builder \
        --repo "$REPO_DIR" \
        "$BUILD_DIR" \
        flatpak_builder.yml \
        --force-clean

    echo "==> Bundling Flatpak..."
    flatpak build-bundle "$REPO_DIR" "$BUNDLE" io.github.joshuardecker.stig-view

    echo "==> Done: $BUNDLE"
fi

# ---------------------------------------------------------------------------
# AppImage
# Requires: appimagetool on PATH.
# Built against the host glibc — run inside an old-glibc container (e.g.
# AlmaLinux 8 / glibc 2.28) for broad distro compatibility.
# ---------------------------------------------------------------------------
if command -v appimagetool &>/dev/null; then
    APPDIR="$PROJECT_ROOT/AppDir"
    APPIMAGE="$PROJECT_ROOT/stig-view.AppImage"

    echo "==> Assembling AppDir..."
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR/usr/bin"

    cp target/release/stig-view "$APPDIR/usr/bin/stig-view"
    cp assets/io.github.joshuardecker.stig-view.desktop "$APPDIR/io.github.joshuardecker.stig-view.desktop"
    cp assets/logo/logo-512.png "$APPDIR/io.github.joshuardecker.stig-view.png"

    cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE="${SELF%/*}"
exec "$HERE/usr/bin/stig-view" "$@"
APPRUN
    chmod +x "$APPDIR/AppRun"

    echo "==> Bundling AppImage..."
    # APPIMAGE_EXTRACT_AND_RUN=1 avoids needing FUSE (required in containers).
    ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 appimagetool "$APPDIR" "$APPIMAGE"

    echo "==> Done: $APPIMAGE"
fi
