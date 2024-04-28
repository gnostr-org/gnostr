xcode:## 	xcode
	cmake -G Xcode -S . -B build
xcodebuild:## 	xcodebuild
	cd build && xcodebuild -configuration Release && cd ..
xcodebuild-list:## 	xcodebuild-list
	cd build && xcodebuild -list && cd ..
xcodebuild-install:## 	xcodebuild-install
	cd build && xcodebuild -target install -configuration Release && cd ..
