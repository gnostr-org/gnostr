MY_CRATE=rustylib
SWIFT_APP=swiftyapp
SWIFT_PROJECT=swiftyrustlib
SWIFT_PROJECT_NAME=RustyLib
SWIFT_CORE_NAME=RustyCore

cd $MY_CRATE

# step 1 - compile rust library and generate bindings
HEADERPATH="out/${MY_CRATE}FFI.h"
OUTDIR="../${MY_CRATE}"
RELDIR="release"
STATIC_LIB_NAME="lib${MY_CRATE}.a"
NEW_HEADER_DIR="out/include"
TARGETDIR="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
JOBS="$(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.logicalcpu 2>/dev/null || nproc 2>/dev/null || echo 8)"

targets=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios" "aarch64-apple-darwin")

for target in "${targets[@]}"; do
    rustup target add "${target}"
    cargo build --target "${target}" --release -j"${JOBS}"
    cargo run --bin uniffi-bindgen -- generate --library "${TARGETDIR}/${target}/release/librustylib.a" --language swift --out-dir out
done

SIM_DIR="${TARGETDIR}/ios-simulator"
SIM_LIB="${SIM_DIR}/${RELDIR}/${STATIC_LIB_NAME}"
mkdir -p "${SIM_DIR}/${RELDIR}"
lipo -create \
    "${TARGETDIR}/aarch64-apple-ios-sim/${RELDIR}/${STATIC_LIB_NAME}" \
    "${TARGETDIR}/x86_64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" \
    -output "${SIM_LIB}"

# step 2 - create xcframework
mkdir -p "${NEW_HEADER_DIR}"
cp "${HEADERPATH}" "${NEW_HEADER_DIR}/"
cat > "${NEW_HEADER_DIR}/module.modulemap" <<EOF
module ${MY_CRATE}FFI {
    header "${MY_CRATE}FFI.h"
    export *
    link "${MY_CRATE}"
}
EOF

rm -rf "${OUTDIR}/${MY_CRATE}_framework.xcframework"

xcodebuild -create-xcframework \
    -library "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" -headers "${NEW_HEADER_DIR}" \
    -library "${SIM_LIB}" -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/aarch64-apple-darwin/${RELDIR}/${STATIC_LIB_NAME}" -headers "${NEW_HEADER_DIR}" \
    -output "${OUTDIR}/${MY_CRATE}_framework.xcframework"

rm -rf "${NEW_HEADER_DIR}"

cd ../

SWIFT_LIB_PATH="./${SWIFT_APP}/Lib/${SWIFT_PROJECT}"

# step 3 - move to SwiftLib artifacts
if [ -d "${SWIFT_LIB_PATH}/artifacts" ]; then
    rm -rf "${SWIFT_LIB_PATH}/artifacts"
fi
mkdir "${SWIFT_LIB_PATH}/artifacts"
cp -R "./${MY_CRATE}/${MY_CRATE}_framework.xcframework" "${SWIFT_LIB_PATH}/artifacts"
mv "${SWIFT_LIB_PATH}/artifacts/${MY_CRATE}_framework.xcframework" "${SWIFT_LIB_PATH}/artifacts/${SWIFT_CORE_NAME}.xcframework"

# step 4 - move to SwiftLib Sources
if [ -d "${SWIFT_LIB_PATH}/Sources" ]; then
    rm -rf "${SWIFT_LIB_PATH}/Sources"
fi
mkdir "${SWIFT_LIB_PATH}/Sources"
mkdir "${SWIFT_LIB_PATH}/Sources/${SWIFT_PROJECT_NAME}"
cp "./${MY_CRATE}/out/${MY_CRATE}.swift" "${SWIFT_LIB_PATH}/Sources/${SWIFT_PROJECT_NAME}/${SWIFT_PROJECT_NAME}.swift"
