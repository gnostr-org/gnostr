.PHONY:xcode
deps/secp256k1/.git:
	git submodule update --init --recursive
xcode:deps/secp256k1/.git## 	xcode
	cmake -G Xcode -S . -B xcode
xcodebuild: xcode## 	xcodebuild
## 	make xcode/Release/nostril
	cd xcode && xcodebuild -target nostril  -arch $(shell uname -m) && cd ..

xcode/Release/nostril:xcodebuild## 	xcode/Release/nostril

xcodebuild-list: xcode## 	xcodebuild-list
	cd xcode && xcodebuild -list && cd ..
xcodebuild-install: xcodebuild## 	xcodebuild-install
	install xcode/Release/nostril /usr/local/bin/ 2>/dev/null || \
        install xcode/RelWithDebInfo/nostril /usr/local/bin/
