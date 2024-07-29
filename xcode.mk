.PHONY:xcode
xcode:## 	xcode
	rm -rf CMakeCache.txt CMakeFiles
	cmake -S . -B xcode -G Xcode
xcodebuild: xcode## 	xcodebuild
	cd xcode && xcodebuild -target nostril -configuration Release && cd ..
xcodebuild-list: xcode## 	xcodebuild-list
	cd xcode && xcodebuild -list && cd ..
xcodebuild-install: xcode## 	xcodebuild-install
	cd xcode && xcodebuild -target nostril -configuration Release && cd ..
