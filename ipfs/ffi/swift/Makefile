all: build

lib:
	cd .. && go build -buildmode=c-archive -o swift/libkubo_ffi.a .

build: lib
	swift build -Xlinker -L. -Xlinker -lkubo_ffi \
	  -Xlinker -framework -Xlinker Security \
	  -Xlinker -framework -Xlinker CoreFoundation \
	  -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl

test: build
	swift test -Xlinker -L. -Xlinker -lkubo_ffi \
	  -Xlinker -framework -Xlinker Security \
	  -Xlinker -framework -Xlinker CoreFoundation \
	  -Xlinker -lresolv -Xlinker -lpthread -Xlinker -ldl

clean:
	rm -f libkubo_ffi.a
	swift package clean
.PHONY: all lib build test clean