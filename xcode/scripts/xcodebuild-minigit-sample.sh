#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
sample_root="${project_root}/MiniGitSample"
project_path="${sample_root}/MiniGitSample.xcodeproj"
scheme_name="MiniGitSample"
clibgit2_root="${project_root}/LibGit2-iOS"
clibgit2_bundle="${clibgit2_root}/Clibgit2.xcframework"
clibgit2_build_script="${project_root}/scripts/libgit2/build-libgit2-framework.sh"

usage() {
  cat <<EOF
usage: $(basename "$0") [--rebuild-clibgit2] [--verbose] [--] [xcodebuild args...]

Run xcodebuild for MiniGitSample with repo-local defaults:
  -project ${project_path}
  -scheme ${scheme_name}
  -configuration Debug
  -sdk iphonesimulator
  -destination 'generic/platform=iOS Simulator'

Before invoking xcodebuild, this script verifies that
LibGit2-iOS/Clibgit2.xcframework contains the binary artifacts referenced by
its Info.plist. If the bundle is missing or incomplete, the script rebuilds
it from source.

options:
  --rebuild-clibgit2  Rebuild Clibgit2.xcframework before running xcodebuild
  --verbose           Pass verbose output through to the Clibgit2 rebuild
  -h, --help          Show this help text

examples:
  $(basename "$0")
  $(basename "$0") build
  $(basename "$0") -list
  $(basename "$0") --rebuild-clibgit2 build
  $(basename "$0") -- build -configuration Release
EOF
}

log() {
  printf '==> %s\n' "$*"
}

has_arg() {
  local needle="$1"
  shift
  local arg

  for arg in "$@"; do
    if [[ "$arg" == "$needle" ]]; then
      return 0
    fi
  done

  return 1
}

append_default_option() {
  local flag="$1"
  local value="$2"

  if ! xcodebuild_has_arg "$flag"; then
    xcodebuild_args+=("$flag" "$value")
  fi
}

append_default_setting() {
  local setting="$1"

  if ! xcodebuild_has_arg "$setting"; then
    xcodebuild_args+=("$setting")
  fi
}

xcodebuild_has_arg() {
  local needle="$1"

  if ((${#xcodebuild_args[@]} == 0)); then
    return 1
  fi

  has_arg "$needle" "${xcodebuild_args[@]}"
}

collect_clibgit2_artifacts() {
  local bundle="$1"
  local plist="${bundle}/Info.plist"
  local index=0
  local identifier
  local library_path

  [[ -f "${plist}" ]] || return 1

  while identifier=$(/usr/libexec/PlistBuddy -c "Print :AvailableLibraries:${index}:LibraryIdentifier" "${plist}" 2>/dev/null); do
    library_path=$(/usr/libexec/PlistBuddy -c "Print :AvailableLibraries:${index}:LibraryPath" "${plist}" 2>/dev/null)
    printf '%s\n' "${bundle}/${identifier}/${library_path}"
    index=$((index + 1))
  done

  ((index > 0))
}

validate_clibgit2_bundle() {
  local bundle="$1"
  local artifact
  local missing=0

  [[ -d "${bundle}" ]] || return 1

  while IFS= read -r artifact; do
    if ! [[ -f "${artifact}" ]]; then
      printf 'missing Clibgit2 artifact: %s\n' "${artifact}" >&2
      missing=1
    fi
  done < <(collect_clibgit2_artifacts "${bundle}")

  ((missing == 0))
}

ensure_clibgit2_bundle() {
  if ((rebuild_clibgit2)); then
    log "Rebuilding Clibgit2.xcframework"
    if ((clibgit2_verbose)); then
      (cd "${project_root}" && "${clibgit2_build_script}" --verbose)
    else
      (cd "${project_root}" && "${clibgit2_build_script}")
    fi
  fi

  if validate_clibgit2_bundle "${clibgit2_bundle}"; then
    return 0
  fi

  log "Clibgit2.xcframework is missing or incomplete; rebuilding from source"
  if ((clibgit2_verbose)); then
    (cd "${project_root}" && "${clibgit2_build_script}" --verbose)
  else
    (cd "${project_root}" && "${clibgit2_build_script}")
  fi

  if validate_clibgit2_bundle "${clibgit2_bundle}"; then
    return 0
  fi

  cat >&2 <<EOF
Clibgit2.xcframework is missing required binary artifacts.

Expected bundle:
  ${clibgit2_bundle}

Try one of:
  $(basename "$0") --rebuild-clibgit2
  (cd "${project_root}" && scripts/libgit2/build-libgit2-framework.sh)
EOF
  return 1
}

rebuild_clibgit2=0
clibgit2_verbose=0
xcodebuild_args=()

while (($#)); do
  case "$1" in
    -h|--help|help)
      usage
      exit 0
      ;;
    --rebuild-clibgit2)
      rebuild_clibgit2=1
      ;;
    --verbose|-v)
      clibgit2_verbose=1
      ;;
    --)
      shift
      xcodebuild_args+=("$@")
      break
      ;;
    *)
      xcodebuild_args+=("$1")
      ;;
  esac
  shift
done

export GIT_CONFIG_COUNT="${GIT_CONFIG_COUNT:-1}"
export GIT_CONFIG_KEY_0="${GIT_CONFIG_KEY_0:-safe.bareRepository}"
export GIT_CONFIG_VALUE_0="${GIT_CONFIG_VALUE_0:-all}"

ensure_clibgit2_bundle

append_default_option "-project" "${project_path}"
append_default_option "-scheme" "${scheme_name}"

if ! xcodebuild_has_arg "-list" &&
   ! xcodebuild_has_arg "-showBuildSettings" &&
   ! xcodebuild_has_arg "-resolvePackageDependencies" &&
   ! xcodebuild_has_arg "-version" &&
   ! xcodebuild_has_arg "archive"; then
  append_default_option "-configuration" "Debug"
  append_default_option "-sdk" "iphonesimulator"
  append_default_option "-destination" "generic/platform=iOS Simulator"
  append_default_setting "CODE_SIGNING_ALLOWED=NO"
fi

log "Running xcodebuild ${xcodebuild_args[*]}"
exec xcodebuild "${xcodebuild_args[@]}"
