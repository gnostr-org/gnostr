#!/usr/bin/env bash
set -euo pipefail

if ! bash -n "${BASH_SOURCE[0]}"; then
  exit 1
fi

WHOAMI="$(whoami)"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="all"
PROJECT_FILTER="all"
CONFIGURATION="${XCODE_CONFIGURATION:-Debug}"
DERIVED_DATA_ROOT="${XCODE_DERIVED_DATA_ROOT:-$ROOT_DIR/.xcodebuild}"
CLEAN=false

usage() {
  cat <<'EOF'
Usage: xcode-build.sh [--mode build|test|all|list] [--project relay|p2p|appwithtool|universal|git-mac-ios-mac|git-mac-ios-ios|all] [--configuration Debug|Release] [--clean]

Modes:
  build   Build the selected Xcode projects
  test    Run a build smoke test for the selected Xcode projects
  all     Build first, then test
  list    Print the known projects, schemes, and test coverage

Options:
  --project NAME        Select relay, p2p, appwithtool, universal, git-mac-ios-mac, git-mac-ios-ios, or all (default)
  --configuration NAME  Xcode build configuration (default: Debug)
  --clean               Remove per-project derived data before running
  --help                Show this help
EOF
}

clean_project_artifacts() {
  local project="$1"
  local project_root="$DERIVED_DATA_ROOT/$project"

  if [[ "$CLEAN" == true && -d "$project_root" ]]; then
    rm -rf "$project_root"
  fi
  if [[ "$MODE" == "all" ]]; then
    rm -rf /Users/$WHOAMI/Library/Developer/Xcode/DerivedData/**
  fi

}

project_path() {
  case "$1" in
    relay)
      printf '%s\n' "xcode/xcode-gnostr-relay/swiftyapp/gnostr-relay.xcodeproj"
      ;;
    p2p)
      printf '%s\n' "xcode/xcode-gnostr-p2p/swiftyapp/swiftyapp.xcodeproj"
      ;;
    appwithtool)
      printf '%s\n' "xcode/AppWithTool/AppWithTool.xcodeproj"
      ;;
    universal)
      printf '%s\n' "xcode/swift-universal/swiftyapp/Universal_App.xcodeproj"
      ;;
    git-mac-ios-mac|git-mac-ios-ios)
      printf '%s\n' "xcode/git-mac-ios/Git.xcodeproj"
      ;;
    *)
      echo "Unsupported project: $1" >&2
      exit 1
      ;;
  esac
}

project_build_scheme() {
  case "$1" in
    relay)
      printf '%s\n' "gnostr-relay"
      ;;
    p2p)
      printf '%s\n' "swiftyapp"
      ;;
    appwithtool)
      printf '%s\n' "AppWithTool"
      ;;
    universal)
      printf '%s\n' "Universal_App (iOS)"
      ;;
    git-mac-ios-mac)
      printf '%s\n' "Mac"
      ;;
    git-mac-ios-ios)
      printf '%s\n' "iOS"
      ;;
    *)
      echo "Unsupported project: $1" >&2
      exit 1
      ;;
  esac
}

project_test_scheme() {
  case "$1" in
    relay)
      printf '%s\n' "gnostr-relay"
      ;;
    p2p)
      printf '%s\n' "swiftyapp"
      ;;
    appwithtool)
      printf '%s\n' "AppWithTool"
      ;;
    universal)
      printf '%s\n' "Universal_App (iOS)"
      ;;
    git-mac-ios-mac)
      printf '%s\n' "Tests-MacOS"
      ;;
    git-mac-ios-ios)
      printf '%s\n' "Tests-iOS"
      ;;
    *)
      echo "Unsupported project: $1" >&2
      exit 1
      ;;
  esac
}

project_build_script() {
  case "$1" in
    relay)
      printf '%s\n' "xcode/xcode-gnostr-relay/build.sh"
      ;;
    p2p)
      printf '%s\n' "xcode/xcode-gnostr-p2p/build.sh"
      ;;
    universal)
      printf '%s\n' "xcode/swift-universal/build.sh"
      ;;
    *)
      return 1
      ;;
  esac
}

project_test_schemes() {
  case "$1" in
    appwithtool)
      printf '%s\n' "AppWithToolTests AppWithToolUITests"
      ;;
    git-mac-ios-mac)
      printf '%s\n' "Tests-MacOS"
      ;;
    git-mac-ios-ios)
      printf '%s\n' "Tests-iOS"
      ;;
    *)
      return 1
      ;;
  esac
}

resolve_test_destination() {
  if [[ -n "${XCODE_TEST_DESTINATION:-}" ]]; then
    printf '%s\n' "$XCODE_TEST_DESTINATION"
    return 0
  fi

  xcrun simctl list devices available -j | python3 -c "import json, sys
data = json.load(sys.stdin)
fallback = None
for runtime_name, devices in data.get('devices', {}).items():
    if 'iOS' not in runtime_name:
        continue
    for device in devices:
        if device.get('isAvailable') and device.get('name') and device.get('udid'):
            destination = f\"platform=iOS Simulator,id={device['udid']}\"
            if device['name'].startswith('iPhone'):
                print(destination)
                raise SystemExit(0)
            if fallback is None:
                fallback = destination
if fallback is not None:
    print(fallback)
    raise SystemExit(0)
raise SystemExit('no available iOS Simulator device found')"
}

project_test_destination() {
  case "$1" in
    appwithtool)
      printf '%s\n' "platform=macOS,id=00008103-0018790121B9001E"
      ;;
    *)
      resolve_test_destination
      ;;
  esac
}

project_build_destination() {
  if [[ -n "${XCODE_BUILD_DESTINATION:-}" ]]; then
    printf '%s\n' "$XCODE_BUILD_DESTINATION"
    return 0
  fi

  case "$1" in
    relay|p2p)
      printf '%s\n' "generic/platform=iOS Simulator"
      ;;
    appwithtool)
      printf '%s\n' "generic/platform=macOS"
      ;;
    universal|git-mac-ios-ios)
      printf '%s\n' "generic/platform=iOS Simulator"
      ;;
    git-mac-ios-mac)
      printf '%s\n' "generic/platform=macOS"
      ;;
    *)
      echo "Unsupported project: $1" >&2
      exit 1
      ;;
  esac
}

selected_projects() {
  case "$PROJECT_FILTER" in
    all)
      printf '%s\n' relay p2p appwithtool universal git-mac-ios-mac git-mac-ios-ios
      ;;
    relay|p2p|appwithtool|universal|git-mac-ios-mac|git-mac-ios-ios)
      printf '%s\n' "$PROJECT_FILTER"
      ;;
    *)
      echo "Unsupported project: $PROJECT_FILTER" >&2
      exit 1
      ;;
  esac
}

run_build_script() {
  local project="$1"
  local build_script

  if build_script="$(project_build_script "$project" 2>/dev/null)"; then
    [[ -f "$build_script" ]] || {
      echo "Missing build script for $project: $build_script" >&2
      exit 1
    }

    (
      cd "$(dirname "$build_script")"
      bash "./$(basename "$build_script")"
    )
  fi
}

project_has_tests() {
  project_test_schemes "$1" >/dev/null 2>&1
}

run_xcodebuild() {
  local action="$1"
  local project="$2"
  local scheme
  local project_path_value
  local derived_data_path

  if [[ "$action" == "build" ]]; then
    scheme="$(project_build_scheme "$project")"
  else
    scheme="$(project_test_scheme "$project")"
  fi
  project_path_value="$(project_path "$project")"
  derived_data_path="$DERIVED_DATA_ROOT/$project/$CONFIGURATION/$action"
  local build_destination

  mkdir -p "$derived_data_path"
  build_destination="$(project_build_destination "$project")"

  if [[ "$action" == "build" ]]; then
    xcodebuild \
      -project "$project_path_value" \
      -scheme "$scheme" \
      -configuration "$CONFIGURATION" \
      -derivedDataPath "$derived_data_path" \
      -destination "$build_destination" \
      build
  else
    xcodebuild \
      -project "$project_path_value" \
      -scheme "$scheme" \
      -configuration "$CONFIGURATION" \
      -derivedDataPath "$derived_data_path" \
      -destination "$(project_test_destination "$project")" \
      test
  fi
}

run_build_projects() {
  local project
  for project in $(selected_projects); do
    clean_project_artifacts "$project"
    run_build_script "$project"
    run_xcodebuild build "$project"
  done
}

run_test_projects() {
  local project

  for project in $(selected_projects); do
    clean_project_artifacts "$project"
    run_build_script "$project"

    if project_has_tests "$project"; then
      run_xcodebuild test "$project"
    else
      run_xcodebuild build "$project"
    fi
  done
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      shift
      [[ $# -gt 0 ]] || { echo "--mode requires a value" >&2; exit 1; }
      MODE="$1"
      ;;
    --project)
      shift
      [[ $# -gt 0 ]] || { echo "--project requires a value" >&2; exit 1; }
      PROJECT_FILTER="$1"
      ;;
    --configuration)
      shift
      [[ $# -gt 0 ]] || { echo "--configuration requires a value" >&2; exit 1; }
      CONFIGURATION="$1"
      ;;
    --clean)
      CLEAN=true
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unsupported flag: $1" >&2
      exit 1
      ;;
  esac
  shift
done

case "$MODE" in
  list)
  for project in relay p2p appwithtool universal git-mac-ios-mac git-mac-ios-ios; do
      printf '%s\t%s\t%s\t%s\n' \
        "$project" \
      "$(project_build_scheme "$project")" \
        "$(project_path "$project")" \
        "$(project_test_schemes "$project" 2>/dev/null || true)"
    done
    ;;
  build)
    run_build_projects
    ;;
  test)
    run_test_projects
    ;;
  all)
    run_build_projects
    run_test_projects
    ;;
  *)
    echo "Unsupported mode: $MODE" >&2
    exit 1
    ;;
esac


## xcodebuild -project xcode/git-mac-ios/Git.xcodeproj -scheme Tests-MacOS -destination 'platform=iOS Simulator,name=iPhone 16' \
##    -only-testing:Tests-MacOS/TestCheckout \
##    -only-testing:Tests-MacOS/TestClone \
##    -only-testing:Tests-MacOS/TestMerge \
##    -only-testing:Tests-MacOS/TestPull \
##    -only-testing:Tests-MacOS/TestUnpack \
##    -only-testing:Tests-MacOS/TestUser \
##    test

