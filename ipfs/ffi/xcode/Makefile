all: build

lib:
	cd ../swift && make build

build: lib
	swift build -Xlinker -L../swift -Xlinker -lkubo_ffi \
	  -Xlinker -framework -Xlinker Security \
	  -Xlinker -framework -Xlinker CoreFoundation \
	  -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl

xcodebuild: lib
	xcodebuild -scheme KuboXcode -destination 'platform=macOS' build

app: build
	rm -rf KuboXcode.app
	mkdir -p KuboXcode.app/Contents/MacOS KuboXcode.app/Contents/Resources
	cp .build/arm64-apple-macosx/debug/XcodeApp KuboXcode.app/Contents/MacOS/XcodeApp
	cp Info.plist KuboXcode.app/Contents/Info.plist
	codesign --force --sign - --entitlements Entitlements.plist KuboXcode.app
	@echo "Built KuboXcode.app"

run: app
	open KuboXcode.app

clean:
	rm -rf .build .swiftpm KuboXcode.app
.PHONY: all lib build xcodebuild app run clean