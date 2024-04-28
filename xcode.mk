xcode:
	cmake -G Xcode -S . -B build
xcodebuild:
	cd build && xcodebuild && cd ..
xcodebuild-list:
	cd build && xcodebuild -list && cd ..
