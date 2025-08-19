#!/usr/bin/env bash
set -euo pipefail

# Android emulator helper script
# - Installs required SDK components (if missing)
# - Creates an AVD (if missing)
# - Boots the emulator and waits until fully started
# - Can also kill/stop running emulators (see "kill" subcommand below)
#
# Configuration via environment variables (override as needed):
#   ANDROID_SDK_ROOT  Path to Android SDK (default: $HOME/Android/Sdk)
#   AVD_NAME          Name of the virtual device (default: bevy-x86-35)
#   API_LEVEL         Android API level (default: 35)
#   IMAGE_FLAVOR      System image flavor (default: google_apis)
#   ABI               ABI for the image (default: x86_64)
#   DEVICE            AVD device profile (default: pixel_5)
#   HEADLESS          1 to run headless (no window), else 0 (default: 0)
#   BOOT_TIMEOUT_SECS Max seconds to wait for boot completion (default: 300)

ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
export ANDROID_SDK_ROOT ANDROID_HOME="$ANDROID_SDK_ROOT"

AVD_NAME="${AVD_NAME:-bevy-x86-35}"
API_LEVEL="${API_LEVEL:-35}"
IMAGE_FLAVOR="${IMAGE_FLAVOR:-google_apis}"
ABI="${ABI:-x86_64}"
DEVICE="${DEVICE:-pixel_5}"
HEADLESS="${HEADLESS:-0}"
BOOT_TIMEOUT_SECS="${BOOT_TIMEOUT_SECS:-300}"

# Prefer the official "latest" layout, but include plain bin for compatibility
export PATH="$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:$ANDROID_SDK_ROOT/cmdline-tools/bin:$ANDROID_SDK_ROOT/platform-tools:$ANDROID_SDK_ROOT/emulator:$PATH"

log() { printf "[emu] %s\n" "$*"; }
die() { printf "[emu][ERROR] %s\n" "$*" >&2; exit 1; }

usage() {
  cat <<USAGE
Usage:
  $(basename "$0")                 # Install deps, create AVD (if needed), boot emulator and wait for boot
  $(basename "$0") kill             # Kill the only running emulator (or print choices if multiple)
  EMULATOR_SERIAL=emulator-5554 $(basename "$0") kill  # Kill a specific emulator by adb serial

Manual stop commands (equivalents):
  adb -e emu kill                    # If only one emulator is running
  adb -s emulator-5554 emu kill      # Target a specific emulator by serial
  pkill -f "emulator.*-avd"         # Force stop all emulators (last resort)

Note on xbuild (x):
  The crates.io release of xbuild can be outdated. If Android builds misbehave,
  install the latest from source:
    git clone https://github.com/rust-mobile/xbuild "$HOME/src/xbuild"
    cargo install --path "$HOME/src/xbuild/xbuild"
  Then verify:
    x --version
USAGE
}

ensure_tools() {
  if ! command -v sdkmanager >/dev/null 2>&1; then
    die "sdkmanager not found. Install Android commandline-tools (latest) under $ANDROID_SDK_ROOT/cmdline-tools.\n"\
        "See: https://developer.android.com/studio#cmdline-tools-only"
  fi
  if ! command -v avdmanager >/dev/null 2>&1; then
    die "avdmanager not found on PATH (cmdline-tools missing?)."
  fi
  if ! command -v emulator >/dev/null 2>&1; then
    log "Installing emulator binary via sdkmanager..."
    yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --install "emulator" || true
  fi
  if ! command -v adb >/dev/null 2>&1; then
    log "Installing platform-tools (adb)..."
    yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --install "platform-tools" || true
  fi
}

install_sdk_components() {
  log "Installing/Updating SDK components for API $API_LEVEL ($IMAGE_FLAVOR/$ABI)..."
  yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --licenses >/dev/null || true
  if ! yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --install \
    "platform-tools" \
    "platforms;android-$API_LEVEL" \
    "system-images;android-$API_LEVEL;$IMAGE_FLAVOR;$ABI" \
    >/dev/null; then
    log "Requested system image not available: android-$API_LEVEL;$IMAGE_FLAVOR;$ABI"
    log "Falling back to API 35 google_apis_playstore x86_64 (faster emulator)"
    API_LEVEL=35
    IMAGE_FLAVOR=google_apis_playstore
    ABI=x86_64
    yes | sdkmanager --sdk_root="$ANDROID_SDK_ROOT" --install \
      "platform-tools" \
      "platforms;android-$API_LEVEL" \
      "system-images;android-$API_LEVEL;$IMAGE_FLAVOR;$ABI" \
      >/dev/null || true
  fi
}

ensure_avd() {
  local avd_dir="$HOME/.android/avd/${AVD_NAME}.avd"
  if avdmanager list avd | grep -q "Name: ${AVD_NAME}" 2>/dev/null; then
    log "AVD '${AVD_NAME}' already exists."
    return
  fi
  log "Creating AVD '${AVD_NAME}' (device=${DEVICE}, api=${API_LEVEL}, image=${IMAGE_FLAVOR}/${ABI})..."
  echo "no" | avdmanager create avd -n "$AVD_NAME" \
    -k "system-images;android-$API_LEVEL;$IMAGE_FLAVOR;$ABI" \
    --device "$DEVICE" \
    --force
  if [[ ! -d "$avd_dir" ]]; then
    die "Failed to create AVD at $avd_dir"
  fi
}

boot_emulator() {
  log "Starting emulator: $AVD_NAME (headless=$HEADLESS)"
  local extra=()
  if [[ "$HEADLESS" == "1" ]]; then
    extra+=("-no-window" "-gpu" "swiftshader_indirect")
  else
    extra+=("-gpu" "host")
  fi

  # Start emulator in background
  emulator -avd "$AVD_NAME" -no-snapshot -no-boot-anim -accel auto -netdelay none -netspeed full "${extra[@]}" >/dev/null 2>&1 &

  # Wait for ADB to see the device
  log "Waiting for device to appear in ADB..."
  adb wait-for-device

  # Wait for boot completion
  log "Waiting for Android to finish booting (timeout ${BOOT_TIMEOUT_SECS}s)..."
  local start_ts now_ts elapsed
  start_ts=$(date +%s)
  while true; do
    if [[ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; then
      log "Boot completed."
      break
    fi
    now_ts=$(date +%s)
    elapsed=$((now_ts - start_ts))
    if (( elapsed > BOOT_TIMEOUT_SECS )); then
      die "Timed out waiting for emulator to boot."
    fi
    sleep 2
  done

  # Small extra wait for Home screen readiness
  sleep 2
  adb shell input keyevent 82 || true # unlock menu
  log "Emulator is ready."
  log "To stop this emulator later:"
  log "  adb -e emu kill    (if only one emulator)"
  log "  adb -s <serial> emu kill    (e.g., emulator-5554)"
}

kill_emulator() {
  # List running emulators
  mapfile -t serials < <(adb devices | awk '/^emulator-/{print $1}')
  if (( ${#serials[@]} == 0 )); then
    log "No running emulators found."
    exit 0
  fi

  if [[ -n "${EMULATOR_SERIAL:-}" ]]; then
    if printf '%s\n' "${serials[@]}" | grep -qx "$EMULATOR_SERIAL"; then
      log "Killing emulator $EMULATOR_SERIAL ..."
      adb -s "$EMULATOR_SERIAL" emu kill || true
      exit 0
    else
      die "EMULATOR_SERIAL '$EMULATOR_SERIAL' not in running list: ${serials[*]}"
    fi
  fi

  if (( ${#serials[@]} == 1 )); then
    log "Killing emulator ${serials[0]} ..."
    adb -s "${serials[0]}" emu kill || true
    exit 0
  fi

  log "Multiple emulators running: ${serials[*]}"
  log "Set EMULATOR_SERIAL=<one of above> and re-run: $0 kill"
  exit 2
}

main() {
  case "${1-}" in
    -h|--help|help) usage; exit 0 ;;
    kill|stop) ensure_tools; kill_emulator ;;
    "" ) : ;; # proceed to boot
    * ) log "Unknown argument: $1"; usage; exit 2 ;;
  esac

  log "SDK root: $ANDROID_SDK_ROOT"
  ensure_tools
  install_sdk_components
  ensure_avd
  boot_emulator
}

main "$@"
