#!/bin/bash

# Build script for kf-sdk Rust + Python wheels and sdist
# Usage: ./build_wheel.sh [options]
#
# Modes (pick one):
#   --local     Build wheels/sdist into dist/ only (default)
#   --upload    Build, then twine check + upload dist/* to PyPI

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

BUILD_TYPE="release"
PLATFORM="auto"
CLEAN=false
INSTALL=false
BUILD_SDIST=true
RUN_TEST=false
MODE="local"   # local | upload
HELP=false
TARGET_VERSIONS=()
DIST_DIR="dist"

# Matches pyproject.toml requires-python = ">=3.9"
SUPPORTED_VERSIONS=("3.9" "3.10" "3.11" "3.12" "3.13")

print_status()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

show_usage() {
    cat << EOF
Usage: $0 [MODE] [OPTIONS]

Build wheels (and optionally sdist) for kf-sdk — Rust + PyO3.
Targets Python 3.9–3.13; builds for every interpreter found on this machine.

MODES (choose one):
    --local             Build only — write artifacts to dist/ (default)
    -u, --upload        Build, then twine check + upload dist/* to PyPI

OPTIONS:
    -t, --type TYPE     Build type: debug, release (default: release)
    -p, --platform PLAT Target platform: auto, linux, macos, windows (default: auto)
    -v, --version VER   Build for one Python: 3.9, 3.10, 3.11, 3.12, or 3.13
    -c, --clean         Clean target/, dist/, build/ before building
    -s, --sdist         Include source distribution (default: on)
    --no-sdist          Wheels only (skip sdist)
    --test              pip install + import smoke test after build
    -i, --install       Install built wheel (implies --test)
    -h, --help          Show this help

EXAMPLES:
    $0                              # local wheels + sdist → dist/
    $0 --local --no-sdist -v 3.11   # single wheel, no sdist, no upload
    $0 --local -i                   # build, install, smoke test
    $0 --local --test               # build + smoke test (no permanent install intent)
    $0 -c -u                        # clean build + upload to PyPI
    $0 --upload -v 3.10 -v 3.11     # build selected versions, then upload

ENVIRONMENT VARIABLES:
    MATURIN_BUILD_ARGS  Extra flags passed to maturin build
    TWINE_REPOSITORY    PyPI index (default: pypi)
EOF
}

is_supported_version() {
    local ver="$1"
    for s in "${SUPPORTED_VERSIONS[@]}"; do
        [[ "$ver" == "$s" ]] && return 0
    done
    return 1
}

check_dependencies() {
    print_status "Checking dependencies..."

    if ! command -v maturin &> /dev/null; then
        print_warning "maturin not found — installing via pip..."
        pip install "maturin>=1.9,<2.0"
    fi

    if ! command -v cargo &> /dev/null; then
        print_error "cargo is not installed. Install Rust: https://rustup.rs"
        exit 1
    fi

    print_status "Rust toolchain: $(cargo --version 2>&1)"

    if [ ${#TARGET_VERSIONS[@]} -gt 0 ]; then
        for ver in "${TARGET_VERSIONS[@]}"; do
            if ! is_supported_version "$ver"; then
                print_error "Unsupported Python version: $ver (supported: ${SUPPORTED_VERSIONS[*]})"
                exit 1
            fi
            if ! command -v "python${ver}" &> /dev/null; then
                print_error "python${ver} is not available on this system."
                exit 1
            fi
        done
        ACTIVE_VERSIONS=("${TARGET_VERSIONS[@]}")
    else
        ACTIVE_VERSIONS=()
        for ver in "${SUPPORTED_VERSIONS[@]}"; do
            if command -v "python${ver}" &> /dev/null; then
                ACTIVE_VERSIONS+=("$ver")
                print_status "Found python${ver}: $(python${ver} --version)"
            else
                print_warning "python${ver} not found — skipping"
            fi
        done
    fi

    if [ ${#ACTIVE_VERSIONS[@]} -eq 0 ]; then
        print_error "No supported Python interpreter found (need one of: ${SUPPORTED_VERSIONS[*]})."
        exit 1
    fi

    print_success "Will build wheels for: ${ACTIVE_VERSIONS[*]}"
}

clean_build() {
    print_status "Cleaning build artifacts..."
    [ -d "target" ] && rm -rf target && print_success "Removed target/"
    [ -d "$DIST_DIR" ] && rm -rf "$DIST_DIR" && print_success "Removed ${DIST_DIR}/"
    [ -d "build" ]  && rm -rf build  && print_success "Removed build/"
    find . -maxdepth 1 -name "*.whl" -delete 2>/dev/null || true
    print_success "Clean complete"
}

maturin_out_args() {
    echo "-o ${DIST_DIR}"
}

maturin_wheel_args() {
    local args
    args="$(maturin_out_args)"
    [ "$BUILD_TYPE" = "release" ] && args="$args --release"
    [ "$PLATFORM" != "auto" ] && args="$args --target $PLATFORM"
    [ -n "${MATURIN_BUILD_ARGS:-}" ] && args="$args $MATURIN_BUILD_ARGS"
    echo "$args"
}

build_sdist() {
    print_status "Building source distribution (sdist)..."
    mkdir -p "$DIST_DIR"
    local cmd="maturin sdist $(maturin_out_args)"
    print_status "Running: $cmd"
    eval "$cmd"
    print_success "sdist built in ${DIST_DIR}/"
}

build_wheels() {
    mkdir -p "$DIST_DIR"
    local common
    common="$(maturin_wheel_args)"

    for ver in "${ACTIVE_VERSIONS[@]}"; do
        print_status "Building wheel for Python ${ver}..."
        local cmd="maturin build ${common} --interpreter python${ver}"
        print_status "Running: $cmd"
        if eval "$cmd"; then
            print_success "Python ${ver} wheel built"
        else
            print_error "Build failed for Python ${ver}"
            exit 1
        fi
    done

    print_status "Artifacts in ${DIST_DIR}/:"
    find "$DIST_DIR" -maxdepth 1 \( -name "*.whl" -o -name "*.tar.gz" \) -type f 2>/dev/null | sort | while read -r f; do
        echo "  - $(basename "$f")"
    done

    if [ -z "$(find "$DIST_DIR" -maxdepth 1 -name '*.whl' -type f 2>/dev/null)" ]; then
        print_error "No wheel files found in ${DIST_DIR}/"
        exit 1
    fi
}

install_wheel() {
    local ver="${ACTIVE_VERSIONS[0]}"
    local tag
    tag="$(echo "$ver" | tr -d '.')"
    local wheel
    wheel="$(find "$DIST_DIR" -maxdepth 1 -name "kf_sdk-*-cp${tag}-*.whl" -type f 2>/dev/null | sort | tail -1)"

    if [ -z "$wheel" ]; then
        wheel="$(find "$DIST_DIR" -maxdepth 1 -name "kf_sdk-*.whl" -type f 2>/dev/null | sort | tail -1)"
    fi

    if [ -z "$wheel" ]; then
        print_error "No wheel found to install in ${DIST_DIR}/"
        exit 1
    fi

    print_status "Installing: $(basename "$wheel")"
    python"${ver}" -m pip install --force-reinstall "$wheel"
    print_success "Installed $(basename "$wheel")"
}

upload_to_pypi() {
    if ! command -v twine &> /dev/null; then
        print_warning "twine not found — installing..."
        pip install twine
    fi

    local artifacts
    artifacts=$(find "$DIST_DIR" -maxdepth 1 \( -name "*.whl" -o -name "*.tar.gz" \) -type f 2>/dev/null | sort)
    if [ -z "$artifacts" ]; then
        print_error "No artifacts in ${DIST_DIR}/ to upload."
        exit 1
    fi

    print_status "Running twine check on ${DIST_DIR}/..."
    twine check "$DIST_DIR"/*

    print_status "Uploading to PyPI:"
    echo "$artifacts" | while read -r f; do echo "  - $(basename "$f")"; done

    twine upload "$DIST_DIR"/*
    print_success "Uploaded to PyPI successfully"
}

test_module() {
    local test_py="python${ACTIVE_VERSIONS[0]}"
    print_status "Testing module with ${test_py}..."

    "$test_py" - << 'PYEOF'
import sys

try:
    import kappa_apk
    ver = kappa_apk.version()
    print(f"  kappa_apk imported OK  — version: {ver}")

    for cls in [
        "KappaApkClient", "KappaDataset", "KappaDataLoader", "DataLoaderHelper",
        "BenchmarkVerification",
        "Dataset", "DatasetItem", "ItemFile",
        "DatasetVersionDetails", "DatasetDownloadDetails", "DatasetLabel",
        "NewDataset", "UpdateDatasetRequest",
        "NewDatasetEntity", "UpdateDatasetEntity",
        "UpdateDatasetLabel", "DeleteDatasetEntities", "NewDatasetVersion",
    ]:
        assert hasattr(kappa_apk, cls), f"Missing export: {cls}"
        print(f"  {cls} — OK")

    print("\nAll checks passed.")
except ImportError as e:
    print(f"FAIL: import error — {e}", file=sys.stderr)
    sys.exit(1)
except AssertionError as e:
    print(f"FAIL: {e}", file=sys.stderr)
    sys.exit(1)
except Exception as e:
    print(f"FAIL: {e}", file=sys.stderr)
    sys.exit(1)
PYEOF

    print_success "Module test passed"
}

run_smoke_test() {
    install_wheel
    test_module
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --local)         MODE="local"; shift ;;
        -u|--upload)     MODE="upload"; shift ;;
        -t|--type)       BUILD_TYPE="$2"; shift 2 ;;
        -p|--platform)   PLATFORM="$2"; shift 2 ;;
        -v|--version)    TARGET_VERSIONS+=("$2"); shift 2 ;;
        -c|--clean)      CLEAN=true; shift ;;
        -s|--sdist)      BUILD_SDIST=true; shift ;;
        --no-sdist)      BUILD_SDIST=false; shift ;;
        --test)          RUN_TEST=true; shift ;;
        -i|--install)    INSTALL=true; RUN_TEST=true; shift ;;
        -h|--help)       HELP=true; shift ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1 ;;
    esac
done

if [ "$HELP" = true ]; then show_usage; exit 0; fi

if [[ ! "$BUILD_TYPE" =~ ^(debug|release)$ ]]; then
    print_error "Invalid build type: $BUILD_TYPE"
    exit 1
fi

if [[ ! "$PLATFORM" =~ ^(auto|linux|macos|windows)$ ]]; then
    print_error "Invalid platform: $PLATFORM"
    exit 1
fi

if [ "$MODE" = "upload" ]; then
    RUN_TEST=true
    print_status "Mode: upload (build → smoke test → PyPI)"
else
    print_status "Mode: local (build only → dist/)"
fi

print_status "kf-sdk — build starting (output: ${DIST_DIR}/)"

check_dependencies
[ "$CLEAN" = true ] && clean_build
[ "$BUILD_SDIST" = true ] && build_sdist
build_wheels

if [ "$RUN_TEST" = true ]; then
    run_smoke_test
fi

if [ "$MODE" = "upload" ]; then
    upload_to_pypi
    print_success "Build and upload completed successfully!"
else
    print_success "Local build completed — artifacts in ${DIST_DIR}/"
    print_status "Install:  $0 -i"
    print_status "Upload:   $0 --upload   (or: twine upload ${DIST_DIR}/*)"
fi
