set -euo pipefail

# Xcode runs build phases in a non-login shell, so Cargo/Rustup may not be on PATH.
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:${PATH:-/usr/bin:/bin:/usr/sbin:/sbin}"
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo not found; install Rust or add cargo to PATH" >&2
    exit 127
fi

if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup not found; install Rust or add rustup to PATH" >&2
    exit 127
fi

MY_CRATE=rustylib
SWIFT_APP=swiftyapp
SWIFT_PROJECT=swiftyrustlib
SWIFT_PROJECT_NAME=RustyLib
SWIFT_CORE_NAME=RustyCore

cd $MY_CRATE

# step 1 - compile rust library and generate bindings
HEADERPATH="out/${MY_CRATE}FFI.h"
TARGETDIR="$(cargo metadata --no-deps --format-version 1 | tr -d '\n' | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
TARGETDIR="${TARGETDIR:-target}"
RELDIR="release"
STATIC_LIB_NAME="lib${MY_CRATE}.a"
NEW_HEADER_DIR="out/include"
XCFRAMEWORK_PATH="${MY_CRATE}_framework.xcframework"

DEVICE_TARGET="aarch64-apple-ios"

case "$(uname -m)" in
    arm64)
        SIMULATOR_TARGET="aarch64-apple-ios-sim"
        CATALYST_TARGET="aarch64-apple-ios-macabi"
        ;;
    x86_64)
        SIMULATOR_TARGET="x86_64-apple-ios"
        CATALYST_TARGET="x86_64-apple-ios-macabi"
        ;;
    *)
        echo "Unsupported host architecture: $(uname -m)" >&2
        exit 1
        ;;
esac

targets=("${DEVICE_TARGET}" "${SIMULATOR_TARGET}" "${CATALYST_TARGET}")

for target in "${targets[@]}"; do
    rustup target add ${target}
            cargo build --target "${target}" --release -j8
            cargo run --bin uniffi-bindgen generate --library "${TARGETDIR}/${target}/${RELDIR}/${STATIC_LIB_NAME}" --language swift --out-dir out
        done
# step 2 - create xcframework
mkdir -p "${NEW_HEADER_DIR}"
cp "${HEADERPATH}" "${NEW_HEADER_DIR}/"
cp "out/${MY_CRATE}FFI.modulemap" "${NEW_HEADER_DIR}/module.modulemap"

rm -rf "${XCFRAMEWORK_PATH}"

xcodebuild -create-xcframework \
    -library "${TARGETDIR}/${DEVICE_TARGET}/${RELDIR}/${STATIC_LIB_NAME}" -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/${SIMULATOR_TARGET}/${RELDIR}/${STATIC_LIB_NAME}" -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/${CATALYST_TARGET}/${RELDIR}/${STATIC_LIB_NAME}" -headers "${NEW_HEADER_DIR}" \
    -output "${XCFRAMEWORK_PATH}"

rm -rf "${NEW_HEADER_DIR}"

cd ../

SWIFT_LIB_PATH="./${SWIFT_APP}/Lib/${SWIFT_PROJECT}"
SWIFT_ARTIFACTS_PATH="${SWIFT_LIB_PATH}/artifacts"
SWIFT_SOURCES_PATH="${SWIFT_LIB_PATH}/Sources/${SWIFT_PROJECT_NAME}"

# step 3 - move to SwiftLib artifacts
mkdir -p "${SWIFT_ARTIFACTS_PATH}"
rm -rf "${SWIFT_ARTIFACTS_PATH}/${SWIFT_CORE_NAME}.xcframework"
cp -R "./${MY_CRATE}/${XCFRAMEWORK_PATH}" "${SWIFT_ARTIFACTS_PATH}/${SWIFT_CORE_NAME}.xcframework"

# step 4 - move to SwiftLib Sources
mkdir -p "${SWIFT_SOURCES_PATH}"
cp "./${MY_CRATE}/out/${MY_CRATE}.swift" "${SWIFT_SOURCES_PATH}/${SWIFT_PROJECT_NAME}.swift"
