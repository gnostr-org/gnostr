.PHONY:view
run:view
view:
	pushd view && make install && make run && popd
view-install:
	pushd view && make install && popd
view-run:
	pushd view && make run && popd

view-create-installer-mac:
	pushd view && make create-installer-mac && popd
view-package-mac:
	pushd view && make package-mac && popd
view-package-mac-x64:
	pushd view && make package-mac-x64 && popd
view-package-mac-arm64:
	pushd view && make package-mac-arm64 && popd
view-package-win:
	pushd view && make package-win && popd
view-start:
	pushd view && make start && popd
view-package-linux:
	pushd view && make view-package-linux && popd
view-package-mac-universal:
	pushd view && make package-mac-universal && popd
view-readme:
	pushd view && make readme && popd
