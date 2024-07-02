.PHONY:xcode
xcode:## 	xcode
	cmake -G Xcode -S . -B xcode
xcodebuild: xcode## 	xcodebuild
	cd xcode && xcodebuild -configuration Release && cd ..
xcodebuild-list: xcode## 	xcodebuild-list
	cd xcode && xcodebuild -list && cd ..
xcodebuild-install: xcode## 	xcodebuild-install
	cd xcode && xcodebuild -target install -configuration Release && cd ..
