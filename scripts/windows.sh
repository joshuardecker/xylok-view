#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# ---------------------------------------------------------------------------
# Locate the VC++ 2022 CRT merge module (.msm) bundled with Visual Studio.
# All path discovery runs inside PowerShell to keep separators clean.
# vswhere.exe is always present on Windows CI runners and VS installations.
# ---------------------------------------------------------------------------
export VCToolsRedistMSM=$(powershell -Command "
  \$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
  \$vs = & \$vswhere -latest -products '*' -property installationPath
  \$base = Join-Path \$vs 'VC\Redist\MSVC'
  \$ver = (Get-ChildItem \$base | Sort-Object Name -Descending | Select-Object -First 1).Name
  \$msmDir = Join-Path \$base \"\$ver\MergeModules\"
  (Get-ChildItem \$msmDir -Filter 'Microsoft_VC*_CRT_x64.msm' | Select-Object -First 1 -ExpandProperty FullName)
" | tr -d '\r')

echo "==> VC Redist merge module: $VCToolsRedistMSM"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
# No --target flag: windows-latest is x86_64-pc-windows-msvc natively,
# so the binary lands in target/release/ where cargo-wix expects it.
echo "==> Building xylok-view (release)..."
cargo build --release

# ---------------------------------------------------------------------------
# Package (MSI)
# ---------------------------------------------------------------------------
echo "==> Building MSI installer..."
cargo wix --no-build --nocapture

MSI=$(find target/wix -name "*.msi" | head -1)
echo "==> Done: $MSI"
