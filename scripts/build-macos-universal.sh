#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="Voice Type"
BUNDLE_ID="com.voicetype.desktop"
VERSION="0.5.1"

ARCH_ARM="aarch64-apple-darwin"
ARCH_X86="x86_64-apple-darwin"

BUILD_DIR="$PROJECT_ROOT/target/macos-release"
RELEASE_DIR="$PROJECT_ROOT/release"
ARM_APP="$BUILD_DIR/arm64/$APP_NAME.app"
X86_APP="$BUILD_DIR/x86_64/$APP_NAME.app"
UNIVERSAL_APP="$BUILD_DIR/universal/$APP_NAME.app"
DMG_PATH="$RELEASE_DIR/Voice-Type-$VERSION-universal.dmg"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

step() { echo -e "${GREEN}[==>]${NC} $1"; }
warn() { echo -e "${YELLOW}[!!]${NC} $1"; }
err() { echo -e "${RED}[ERR]${NC} $1" >&2; exit 1; }

check_macos() {
    [[ "$(uname)" == "Darwin" ]] || err "This script must be run on macOS"
}

check_deps() {
    for cmd in cargo bun node lipo; do
        command -v "$cmd" &>/dev/null || err "Missing required command: $cmd"
    done
}

install_rust_targets() {
    step "Installing Rust targets..."
    rustup target add "$ARCH_ARM" "$ARCH_X86" 2>/dev/null || true
}

install_deps() {
    step "Installing frontend dependencies..."
    cd "$PROJECT_ROOT"
    bun install
}

build_frontend() {
    step "Building frontend..."
    cd "$PROJECT_ROOT"
    bun run build
}

build_target() {
    local arch="$1"
    local target="$2"
    step "Building for $arch ($target)..."
    cd "$PROJECT_ROOT"
    cargo tauri build --target "$target" --bundles app
    local exit_code=$?
    if [[ $exit_code -ne 0 ]]; then
        err "Build failed for $arch (exit code $exit_code)"
    fi
}

create_universal_binary() {
    step "Creating universal binary..."

    mkdir -p "$BUILD_DIR/universal"

    local arm_binary="$ARM_APP/Contents/MacOS/$APP_NAME"
    local x86_binary="$X86_APP/Contents/MacOS/$APP_NAME"
    local universal_binary="$UNIVERSAL_APP/Contents/MacOS/$APP_NAME"

    if [[ ! -f "$arm_binary" ]]; then
        err "Apple Silicon binary not found at $arm_binary"
    fi
    if [[ ! -f "$x86_binary" ]]; then
        err "Intel binary not found at $x86_binary"
    fi

    cp -R "$ARM_APP" "$UNIVERSAL_APP"

    step "Merging binaries with lipo..."
    lipo -create -output "$universal_binary" "$arm_binary" "$x86_binary"

    local archs
    archs=$(lipo -archs "$universal_binary")
    step "Universal binary architectures: $archs"

    if [[ "$archs" != "x86_64 arm64" ]]; then
        err "Universal binary does not contain expected architectures. Got: $archs"
    fi

    merge_frameworks "$ARM_APP" "$X86_APP" "$UNIVERSAL_APP"
}

merge_frameworks() {
    local arm_app="$1"
    local x86_app="$2"
    local universal_app="$3"
    local frameworks_dir="$universal_app/Contents/Frameworks"

    if [[ -d "$arm_app/Contents/Frameworks" ]]; then
        while IFS= read -r -d '' framework; do
            local fname
            fname=$(basename "$framework")
            local arm_fw="$framework"
            local x86_fw="$x86_app/Contents/Frameworks/$fname"
            local universal_fw="$frameworks_dir/$fname"

            if [[ -f "$x86_fw" ]]; then
                mkdir -p "$frameworks_dir"
                lipo -create -output "$universal_fw" "$arm_fw" "$x86_fw" 2>/dev/null || {
                    cp "$arm_fw" "$universal_fw"
                    warn "Could not create universal framework for $fname, using arm64 only"
                }
            fi
        done < <(find "$arm_app/Contents/Frameworks" -name "*.dylib" -print0 2>/dev/null)
    fi
}

create_dmg() {
    step "Creating DMG installer..."
    mkdir -p "$RELEASE_DIR"

    local dmg_temp="$BUILD_DIR/dmg-temp"
    local staging="$dmg_temp/staging"
    rm -rf "$dmg_temp"
    mkdir -p "$staging"

    cp -R "$UNIVERSAL_APP" "$staging/"
    ln -s /Applications "$staging/Applications"

    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$staging" \
        -ov -format UDZO \
        "$DMG_PATH"

    rm -rf "$dmg_temp"
    step "DMG created: $DMG_PATH"
}

cleanup() {
    step "Cleaning up intermediate builds..."
    rm -rf "$BUILD_DIR/arm64"
    rm -rf "$BUILD_DIR/x86_64"
}

main() {
    echo "======================================"
    echo "  Voice Type - macOS Universal Build  "
    echo "  Version: $VERSION                   "
    echo "======================================"
    echo ""

    check_macos
    check_deps
    install_rust_targets
    install_deps
    build_frontend
    build_target "Apple Silicon" "$ARCH_ARM"
    build_target "Intel" "$ARCH_X86"
    create_universal_binary
    create_dmg
    cleanup

    echo ""
    echo "======================================"
    echo -e "  ${GREEN}Build complete!${NC}                   "
    echo "======================================"
    echo ""
    echo "  App bundle:  $UNIVERSAL_APP"
    echo "  DMG:         $DMG_PATH"
    echo ""
    echo "  To install:"
    echo "    1. Open the DMG"
    echo "    2. Drag Voice Type to /Applications"
    echo ""
    echo "  To test locally:"
    echo "    open \"$UNIVERSAL_APP\""
    echo ""
}

main "$@"
