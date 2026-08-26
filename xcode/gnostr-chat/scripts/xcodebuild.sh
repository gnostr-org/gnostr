#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"

project="${XCODE_PROJECT:-gnostr-chat.xcodeproj}"
scheme="${XCODE_SCHEME:-gnostr-iphone}"
configuration="${XCODE_CONFIGURATION:-Debug}"
destination="${XCODE_DESTINATION:-platform=iOS Simulator,name=iPhone 16}"
derived_data_path="${XCODE_DERIVED_DATA_PATH:-${project_root}/.build/DerivedData}"
action="${1:-build}"
build_action="${action}"

if [[ "${action}" == "run" ]]; then
  build_action="build"
fi

if (($#)); then
  shift
fi

run_app() {
  local app_bundle

  if [[ "${destination}" == *"platform=macOS"* ]]; then
    app_bundle="$(find "${derived_data_path}/Build/Products/${configuration}" -maxdepth 1 -name '*.app' -print -quit)"
    if [[ -z "${app_bundle}" ]]; then
      echo "Unable to find built macOS app bundle under ${derived_data_path}/Build/Products/${configuration}" >&2
      exit 1
    fi
    open "${app_bundle}"
    return
  fi

  app_bundle="$(find "${derived_data_path}/Build/Products/${configuration}-iphonesimulator" -maxdepth 1 -name '*.app' -print -quit)"
  if [[ -z "${app_bundle}" ]]; then
    echo "Unable to find built simulator app bundle under ${derived_data_path}/Build/Products/${configuration}-iphonesimulator" >&2
    exit 1
  fi

  local bundle_id
  bundle_id="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "${app_bundle}/Info.plist")"

  local simulator_id
  if [[ "${destination}" =~ id=([^,]+) ]]; then
    simulator_id="${BASH_REMATCH[1]}"
  else
    local simulator_name
    if [[ "${destination}" =~ name=([^,]+) ]]; then
      simulator_name="${BASH_REMATCH[1]}"
    else
      simulator_name="iPhone 16"
    fi
    simulator_id="$(xcrun simctl list devices available | awk -v name="${simulator_name}" '$0 ~ name" \(" {gsub(/[()]/, "", $NF); print $NF; exit}')"
  fi

  if [[ -z "${simulator_id}" ]]; then
    echo "Unable to resolve a simulator ID from destination: ${destination}" >&2
    exit 1
  fi

  xcrun simctl boot "${simulator_id}" >/dev/null 2>&1 || true
  xcrun simctl bootstatus "${simulator_id}" -b >/dev/null
  xcrun simctl launch "${simulator_id}" "${bundle_id}"
}

xcodebuild \
  -project "${project_root}/${project}" \
  -scheme "${scheme}" \
  -configuration "${configuration}" \
  -destination "${destination}" \
  -derivedDataPath "${derived_data_path}" \
  "${build_action}" \
  "$@"

if [[ "${action}" == "run" ]]; then
  run_app
fi
