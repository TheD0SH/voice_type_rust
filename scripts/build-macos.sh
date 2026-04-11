#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ARCH="${1:-native}"
VERSION="0.5.1"
APP_NAME="Voice Type"
RELEASE_DIR="$PROJECT_ROOT/release"

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

detect_arch() {
    local machine
    machine=$(uname -m)
    case "$machine" in
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) err "Unknown architecture: $machine" ;;
    esac
}

resolve_target() {
    case "$ARCH" in
        native) detect_arch ;;
        arm64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "$ARCH" ;;
    esac
}

main() {
    check_macos

    local target
    target=$(resolve_target)
    step "Building for target: $target"

    step "Installing dependencies..."
    cd "$PROJECT_ROOT"
    bun install

    step "Building frontend..."
    bun run build

    step "Building Tauri app..."
    cargo tauri build --target "$target" --bundles app,dmg

    local arch_label
    case "$target" in
        aarch64-apple-darwin) arch_label="arm64" ;;
        x86_64-apple-darwin) arch_label="x86_64" ;;
        *) arch_label="$target" ;;
    esac

    local app_path="$PROJECT_ROOT/target/$target/release/bundle/macos/$APP_NAME.app"
    local dmg_glob="$PROJECT_ROOT/target/$target/release/bundle/dmg/"*.dmg

    mkdir -p "$RELEASE_DIR"

    if [[ -d "$app_path" ]]; then
        cp -R "$app_path" "$RELEASE_DIR/Voice-Type-$VERSION-$arch_label.app"
        step "App bundle copied to: $RELEASE_DIR/Voice-Type-$VERSION-$arch_label.app"
    else
        warn "App bundle not found at expected path: $app_path"
    fi

    for dmg in $dmg_glob; do
        if [[ -f "$dmg" ]]; then
            cp "$dmg" "$RELEASE_DIR/"
            step "DMG copied to: $RELEASE_DIR/$(basename "$dmg")"
        fi
    done

    step "Single-architecture build complete!"
    echo ""
    echo "  Output directory: $RELEASE_DIR"
}

main "$@"
