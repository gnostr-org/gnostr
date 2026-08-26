#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"
package_root="${repo_root}/LibP2P-iOS"
source_root="${package_root}/cpp-libp2p-0.1.37"
build_root="${package_root}/build"
install_root="${package_root}/install"
framework_root="${package_root}/LibP2P.xcframework"
framework_stage="${package_root}/LibP2P.stage.xcframework"
framework_zip="${package_root}/LibP2P.xcframework.zip"
modulemap_source="${package_root}/LibP2P_modulemap"

build_platforms=(macosx iphoneos iphonesimulator iphonesimulator-arm64)
xcframework_platforms=(macosx iphoneos iphonesimulator)

usage() {
  cat <<EOF
usage: $(basename "$0") [--verbose] [--configure-only]

Configure and build the vendored cpp-libp2p source tree for Apple targets.

The default build turns QUIC off because the upstream lsquic dependency is not
yet integrated into this repo's Apple toolchain path.

By default, the script also packages the merged static libraries as
LibP2P.xcframework.
EOF
}

verbose=0
configure_only=0

while (($#)); do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    --verbose|-v)
      verbose=1
      ;;
    --configure-only)
      configure_only=1
      ;;
    *)
      echo "usage: $(basename "$0") [--verbose] [--configure-only]" >&2
      exit 1
      ;;
  esac
  shift
done

base_cmake_args=(
  -DCMAKE_BUILD_TYPE=Release
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5
  -DBUILD_SHARED_LIBS=NO
  -DTESTING=OFF
  -DEXAMPLES=OFF
  -DCLANG_FORMAT=OFF
  -DCLANG_TIDY=OFF
  -DCOVERAGE=OFF
  -DASAN=OFF
  -DLSAN=OFF
  -DMSAN=OFF
  -DTSAN=OFF
  -DUBSAN=OFF
  -DMETRICS_ENABLED=OFF
  -DSQLITE_ENABLED=OFF
  -DLIBP2P_ENABLE_QUIC=OFF
)

rm -rf "${framework_stage}" "${framework_zip}"

setup_variables() {
  PLATFORM="$1"
  PLATFORM_CMAKE_ARGS=()

  case "${PLATFORM}" in
    "macosx")
      ARCH=arm64
      SYSROOT="$(xcodebuild -version -sdk macosx Path)"
      PLATFORM_CMAKE_ARGS+=(
        -DCMAKE_OSX_ARCHITECTURES="${ARCH}"
        -DCMAKE_OSX_SYSROOT="${SYSROOT}"
      )
      ;;
    "iphoneos")
      ARCH=arm64
      SYSROOT="$(xcodebuild -version -sdk iphoneos Path)"
      PLATFORM_CMAKE_ARGS+=(
        -DCMAKE_OSX_ARCHITECTURES="${ARCH}"
        -DCMAKE_OSX_SYSROOT="${SYSROOT}"
      )
      ;;
    "iphonesimulator")
      ARCH=x86_64
      SYSROOT="$(xcodebuild -version -sdk iphonesimulator Path)"
      PLATFORM_CMAKE_ARGS+=(
        -DCMAKE_OSX_ARCHITECTURES="${ARCH}"
        -DCMAKE_OSX_SYSROOT="${SYSROOT}"
      )
      ;;
    "iphonesimulator-arm64")
      ARCH=arm64
      SYSROOT="$(xcodebuild -version -sdk iphonesimulator Path)"
      PLATFORM_CMAKE_ARGS+=(
        -DCMAKE_OSX_ARCHITECTURES="${ARCH}"
        -DCMAKE_OSX_SYSROOT="${SYSROOT}"
      )
      ;;
    *)
      echo "Unsupported platform: ${PLATFORM}" >&2
      exit 1
      ;;
  esac
}

build_platform() {
  setup_variables "$1"

  local platform_build_root="${build_root}/${PLATFORM}"
  local platform_install_root="${install_root}/${PLATFORM}"
  local archives=()
  local archive
  local cmake_args=(
    -S "${source_root}"
    -B "${platform_build_root}"
    -DCMAKE_INSTALL_PREFIX="${platform_install_root}"
  )

  cmake_args+=("${base_cmake_args[@]}")
  cmake_args+=("${PLATFORM_CMAKE_ARGS[@]}")

  if ((verbose)); then
    printf '==> cmake'
    printf ' %q' "${cmake_args[@]}"
    printf '\n'
  fi

  cmake "${cmake_args[@]}"

  if ((configure_only)); then
    return 0
  fi

  if ((verbose)); then
    cmake --build "${platform_build_root}" --parallel 1
    cmake --install "${platform_build_root}"
  else
    cmake --build "${platform_build_root}" --parallel 1 >/dev/null
    cmake --install "${platform_build_root}" >/dev/null
  fi

  while IFS= read -r archive; do
    archives+=("${archive}")
  done < <(find "${platform_build_root}/src" -type f -name '*.a' | sort)

  if ((${#archives[@]} == 0)); then
    echo "No static libraries were produced for ${PLATFORM}" >&2
    exit 1
  fi

  libtool -static -o "${platform_install_root}/libp2p.a" "${archives[@]}"
}

package_framework() {
  local framework_args=()
  local platform
  local slice_headers

  for platform in "${xcframework_platforms[@]}"; do
    framework_args+=(
      -library "${install_root}/${platform}/libp2p.a"
      -headers "${source_root}/include"
    )
  done

  if ((verbose)); then
    printf '==> xcodebuild -create-xcframework'
    printf ' %q' "${framework_args[@]}"
    printf ' -output %q\n' "${framework_stage}"
  fi

  xcodebuild -create-xcframework "${framework_args[@]}" -output "${framework_stage}"
  mkdir -p "${framework_root}"
  cp -R "${framework_stage}/." "${framework_root}/"
  while IFS= read -r slice_headers; do
    cp "${modulemap_source}" "${slice_headers}/module.modulemap"
  done < <(find "${framework_root}" -mindepth 2 -maxdepth 2 -type d -name Headers | sort)
  mkdir -p "${framework_root}/Headers"
  cp "${modulemap_source}" "${framework_root}/Headers/module.modulemap"
  rm -rf "${framework_stage}"
  (cd "${package_root}" && zip -qr "${framework_zip}" "${framework_root##*/}")
}

for platform in "${build_platforms[@]}"; do
  build_platform "${platform}"
done

if ((configure_only)); then
  exit 0
fi

mkdir -p "${install_root}/iphonesimulator"
lipo -create \
  -output "${install_root}/iphonesimulator/libp2p.universal.a" \
  "${install_root}/iphonesimulator/libp2p.a" \
  "${install_root}/iphonesimulator-arm64/libp2p.a"
mv "${install_root}/iphonesimulator/libp2p.universal.a" "${install_root}/iphonesimulator/libp2p.a"

package_framework
