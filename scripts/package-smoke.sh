#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="${SCRIPT_DIR}/toolchain/targets"

selected_targets=()
expect_artifact=""
dry_run=0
output_root="${BLINC_PACKAGE_OUTPUT_ROOT:-target/package}"
build_cmd="${BLINC_BUILD_CMD:-blinc}"
version="${BLINC_PACKAGE_VERSION:-1.0.0}"

usage() {
  cat <<'EOF'
Usage:
  package-smoke.sh [--target macos|windows|linux] [--dry-run] [--output-root DIR]
                    [--expect-artifact PATH] [--version VERSION]

  --target TARGET        Filter targets. Repeatable. Defaults to all targets.
  --dry-run              Print commands only.
  --output-root DIR      Directory where artifacts are expected.
  --expect-artifact PATH Require this artifact to exist.
  --version VERSION      Version override used in output metadata.
  -h, --help            Show this help text.
EOF
}

is_valid_target() {
  case "$1" in
    macos|windows|linux)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --target)
      if [[ "$#" -lt 2 ]]; then
        echo "error: --target requires a target name" >&2
        usage
        exit 2
      fi
      selected_targets+=("$2")
      shift 2
      ;;
    --dry-run)
      dry_run=1
      shift
      ;;
    --output-root)
      if [[ "$#" -lt 2 ]]; then
        echo "error: --output-root requires a value" >&2
        usage
        exit 2
      fi
      output_root="$2"
      shift 2
      ;;
    --expect-artifact)
      if [[ "$#" -lt 2 ]]; then
        echo "error: --expect-artifact requires a path" >&2
        usage
        exit 2
      fi
      expect_artifact="$2"
      shift 2
      ;;
    --version)
      if [[ "$#" -lt 2 ]]; then
        echo "error: --version requires a value" >&2
        usage
        exit 2
      fi
      version="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ ${#selected_targets[@]} -eq 0 ]]; then
  selected_targets=(macos windows linux)
fi

validate_targets() {
  for t in "${selected_targets[@]}"; do
    if ! is_valid_target "$t"; then
      echo "error: unsupported target '$t'" >&2
      echo "valid targets: macos windows linux" >&2
      exit 2
    fi

    if [[ ! -f "${TARGET_DIR}/${t}.toml" ]]; then
      echo "error: missing config file ${TARGET_DIR}/${t}.toml" >&2
      exit 2
    fi
  done
}

get_target_meta() {
  local target=$1
  local config="${TARGET_DIR}/${target}.toml"

  python3 - "$target" "$config" "$version" <<'PY'
import re
import sys
import tomllib

target = sys.argv[1]
config_path = sys.argv[2]
version = sys.argv[3]

text = open(config_path, "rb").read()
config = tomllib.loads(text.decode("utf-8"))


def slugify(value: str) -> str:
    value = (value or "").strip().lower()
    value = re.sub(r"[^a-z0-9]+", "-", value)
    value = value.strip("-")
    return value or "blincapp"


if target == "macos":
    bundle = config.get("bundle", {})
    signing = config.get("signing", {})
    notarization = config.get("notarization", {})
    artifact = f"{bundle.get('bundle_name', 'BlincApp')}.app"
    print(f"bundle_name={bundle.get('bundle_name', 'BlincApp')}")
    print(f"artifact={artifact}")
    print(f"bundle_id={bundle.get('bundle_identifier', 'com.example.blincapp')}")
    print(f"bundle_version={bundle.get('bundle_version', version)}")
    print(f"identity={signing.get('identity', '-')}")
    print(f"hardened_runtime={signing.get('hardened_runtime', True)}")
    print(f"notarization_enabled={bool(notarization.get('apple_id'))}")
    print("signing_profile=")
elif target == "windows":
    executable = config.get("executable", {})
    installer = config.get("installer", {})
    signing = config.get("signing", {})
    artifact = executable.get("original_filename", "blincapp.exe")
    print(f"executable={artifact}")
    print(f"product_name={executable.get('product_name', 'BlincApp')}")
    print(f"file_version={executable.get('file_version', f'{version}.0')}")
    print(f"installer_type={installer.get('type', 'msix')}")
    print(f"publisher={installer.get('publisher', '')}")
    print(f"certificate={signing.get('certificate', '')}")
    print(f"timestamp_url={signing.get('timestamp_url', '')}")
    print("signing_profile=windows")
elif target == "linux":
    desktop = config.get("desktop", {})
    flatpak = config.get("flatpak", {})
    appimage = config.get("appimage", {})
    bundle = config.get("bundle", {})
    name = desktop.get("name", "Blinc App")
    slug = slugify(name)
    print(f"desktop_name={name}")
    print(f"desktop_slug={slug}")
    print(f"app_id={flatpak.get('app_id', 'com.example.BlincApp')}")
    print(f"bundle_version={bundle.get('bundle_version', version)}")
    print(f"appimage_enabled={appimage.get('enabled', True)}")
    print(f"artifact={slug}")
    print(f"appimage_artifact={slug}.AppImage")
    print(f"signing_profile={flatpak.get('app_id', 'com.example.BlincApp')}")
else:
    raise SystemExit(1)
PY
}

run_target() {
  local target=$1
  local out_dir="${output_root}/${target}"
  mkdir -p "$out_dir"

  local meta
  local kv_bundle_name=""
  local kv_bundle_id=""
  local kv_bundle_version=""
  local kv_identity=""
  local kv_hardened_runtime=""
  local kv_notarization_enabled=""
  local kv_executable=""
  local kv_product_name=""
  local kv_file_version=""
  local kv_installer_type=""
  local kv_publisher=""
  local kv_certificate=""
  local kv_timestamp_url=""
  local kv_desktop_name=""
  local kv_desktop_slug=""
  local kv_app_id=""
  local kv_appimage_enabled=""
  local kv_artifact=""
  local kv_appimage_artifact=""
  local kv_signing_profile=""

  meta=$(get_target_meta "$target")

  while IFS='=' read -r key value; do
    [ -z "$key" ] && continue
    case "$key" in
      bundle_name) kv_bundle_name="$value" ;;
      bundle_id) kv_bundle_id="$value" ;;
      bundle_version) kv_bundle_version="$value" ;;
      identity) kv_identity="$value" ;;
      hardened_runtime) kv_hardened_runtime="$value" ;;
      notarization_enabled) kv_notarization_enabled="$value" ;;
      executable) kv_executable="$value" ;;
      product_name) kv_product_name="$value" ;;
      file_version) kv_file_version="$value" ;;
      installer_type) kv_installer_type="$value" ;;
      publisher) kv_publisher="$value" ;;
      certificate) kv_certificate="$value" ;;
      timestamp_url) kv_timestamp_url="$value" ;;
      desktop_name) kv_desktop_name="$value" ;;
      desktop_slug) kv_desktop_slug="$value" ;;
      app_id) kv_app_id="$value" ;;
      appimage_enabled) kv_appimage_enabled="$value" ;;
      artifact) kv_artifact="$value" ;;
      appimage_artifact) kv_appimage_artifact="$value" ;;
      signing_profile) kv_signing_profile="$value" ;;
    esac
  done <<<"$meta"

  local command=("$build_cmd" "build" "--target" "$target" "--release")
  local command_str
  command_str="${command[*]}"
  local artifact=""

  case "$target" in
    macos)
      artifact="${out_dir}/${kv_artifact}"
      command+=("--output" "$artifact")
      echo "[dry-run:macos]"
      echo "  artifact: $artifact"
      echo "  bundle_id: ${kv_bundle_id}"
      echo "  version: ${kv_bundle_version}"
      echo "  signing: identity=${kv_identity} hardened_runtime=${kv_hardened_runtime} notarization=${kv_notarization_enabled}"
      ;;
    windows)
      local exe_name="${kv_executable}"
      local exe_path="${out_dir}/${exe_name}"
      local installer="${out_dir}/blincapp.${kv_installer_type}"
      artifact="$exe_path"
      command+=("--output" "$exe_path")
      echo "[dry-run:windows]"
      echo "  exe: $exe_path"
      echo "  installer: $installer"
      echo "  signing: certificate_set=${kv_certificate:+yes}; timestamp=${kv_timestamp_url}; publisher=${kv_publisher}"
      echo "  signing_profile=${kv_signing_profile}"
      ;;
    linux)
      local binary="${kv_desktop_slug}"
      local binary_path="${out_dir}/${binary}"
      local appimage_path="${out_dir}/${kv_appimage_artifact}"
      artifact="$binary_path"
      command+=("--output" "$binary_path")
      echo "[dry-run:linux]"
      echo "  desktop_entry: ${binary}.desktop"
      echo "  desktop_name: ${kv_desktop_name}"
      echo "  binary: $binary_path"
      echo "  appimage: $appimage_path (enabled=${kv_appimage_enabled})"
      echo "  app_id: ${kv_app_id}"
      echo "  signing_profile=${kv_signing_profile}"
      ;;
    *)
      echo "error: unsupported target '$target'" >&2
      return 1
      ;;
  esac

  command_str="${command[*]}"
  echo "  command: $command_str"
  echo "  expected_from_meta: ${artifact}"

  if [[ "$dry_run" == "1" ]]; then
    echo "  mode: dry-run"
  else
    echo "  mode: execute"
    "${command[@]}"
  fi

  if [[ -n "$expect_artifact" ]]; then
    if [[ ! -f "$expect_artifact" ]]; then
      echo "error: expected artifact not found: $expect_artifact" >&2
      return 1
    fi
    echo "  expect-artifact: ok ($expect_artifact)"
  fi

  echo
}

validate_targets
for target in "${selected_targets[@]}"; do
  run_target "$target"
done

echo "package-smoke complete"
