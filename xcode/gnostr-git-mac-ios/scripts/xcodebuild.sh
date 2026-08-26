#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PROJECT="$PROJECT_DIR/Git.xcodeproj"
IOS_DESTINATION="${XCODE_IOS_DESTINATION:-platform=iOS Simulator,name=iPhone 16}"

usage() {
  cat <<'EOF'
Usage: xcodebuild.sh <command>

Commands:
  mac-build   Build the Mac scheme
  mac-test    Run the Mac test scheme
  ios-build   Build the iOS scheme
  ios-test    Run the iOS test scheme
  resolve     Resolve Swift package dependencies
  clean       Remove the project's derived data folder
  reset       Remove derived data and package resolution state
  build       Build Mac and iOS
  test        Test Mac and iOS
  all         Build and then test Mac and iOS
  list        Show the available commands
EOF
}

run_xcodebuild() {
  local scheme="$1"
  shift
  xcodebuild -project "$PROJECT" -scheme "$scheme" -derivedDataPath "$(project_derived_data)" "$@"
}

project_derived_data() {
  printf '%s\n' "$PROJECT_DIR/.xcodebuild"
}

clean() {
  rm -rf "$(project_derived_data)"
}

reset() {
  clean
  rm -rf "$PROJECT_DIR/Git.xcodeproj/project.xcworkspace/xcuserdata"
  rm -rf "$PROJECT_DIR/Git.xcodeproj/project.xcworkspace/xcshareddata/swiftpm"
  rm -rf "$REPO_DIR/.xcodebuild"
}

resolve() {
  xcodebuild -project "$PROJECT" -resolvePackageDependencies
}

mac_build() {
  run_xcodebuild Mac -destination "generic/platform=macOS" build
}

mac_test() {
  run_xcodebuild Tests-MacOS -destination "$IOS_DESTINATION" test
}

ios_build() {
  run_xcodebuild iOS -destination "generic/platform=iOS Simulator" build
}

ios_test() {
  run_xcodebuild Tests-iOS -destination "$IOS_DESTINATION" test
}

case "${1:-}" in
  mac-build)
    mac_build
    ;;
  mac-test)
    mac_test
    ;;
  ios-build)
    ios_build
    ;;
  ios-test)
    ios_test
    ;;
  resolve)
    resolve
    ;;
  clean)
    clean
    ;;
  reset)
    reset
    ;;
  build)
    mac_build
    ios_build
    ;;
  test)
    mac_test
    ios_test
    ;;
  all)
    mac_build
    ios_build
    mac_test
    ios_test
    ;;
  list)
    usage
    ;;
  ""|-h|--help)
    usage
    ;;
  *)
    echo "Unsupported command: $1" >&2
    usage >&2
    exit 1
    ;;
esac
