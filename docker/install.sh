#!/bin/bash

UNAME_CMD="$(uname -m)"
case "$UNAME_CMD" in
x86_64) SUFFIX="x86_64" ;;
aarch64) SUFFIX="aarch64" ;;
armv6l) SUFFIX="armv6" ;;
esac

mkdir -p ${HOME}/bin

if [ -n "$SUFFIX" ]; then
	OXKER_GZ="oxker_linux_${SUFFIX}.tar.gz"
	curl -L -O "https://github.com/mrjackwills/oxker/releases/latest/download/${OXKER_GZ}"
	tar xzvf "${OXKER_GZ}" oxker
	chmod +x ./oxker
	install oxker "${HOME}/bin/"
	#rm "${OXKER_GZ}" oxker
fi
