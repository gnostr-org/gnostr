xcode:
	cmake -G Xcode -S . -B build
xcodebuild:
	cd build && xcodebuild && cd ..
