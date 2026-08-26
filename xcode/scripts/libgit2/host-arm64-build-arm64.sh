# Build libgit2 XCFramework
#
# This script assumes that
#  1. it is run at the root of the repo
#  2. the required tools (wget, ninja, cmake, autotools) are installed either globally via homebrew or locally in tools/bin using our other script build_tools.sh
#

unset SDKROOT
unset IPHONEOS_DEPLOYMENT_TARGET

export REPO_ROOT=`pwd`
export PATH=$PATH:$REPO_ROOT/tools/bin

# List of platforms-architecture that we support
# Note that there are limitations in `xcodebuild` command that disallows `maccatalyst` and `macosx` (native macOS lib) in the same xcframework.
AVAILABLE_PLATFORMS=(iphoneos iphonesimulator maccatalyst) # macosx macosx-arm64

### Setup common environment variables to run CMake for a given platform
### Usage:      setup_variables PLATFORM
### where PLATFORM is the platform to build for and should be one of
###    iphoneos  (implicitly arm64)
###    iphonesimulator-x86_64
###    iphonesimulator-arm64
###    maccatalyst-x86_64
###    maccatalyst-arm64
###    macosx, macosx-arm64
###
### After this function is executed, the variables
###    $PLATFORM
###    $ARCH
###    $SYSROOT
###    $CMAKE_ARGS
### providing basic/common CMake options will be set.
function setup_variables() {
	cd $REPO_ROOT
	PLATFORM=$1

	CMAKE_ARGS=(-DBUILD_SHARED_LIBS=NO \
		-DCMAKE_BUILD_TYPE=Release \
		-DCMAKE_C_COMPILER_WORKS=ON \
		-DCMAKE_CXX_COMPILER_WORKS=ON \
		-DCMAKE_POLICY_DEFAULT_CMP0026=NEW \
                -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
		-DCMAKE_INSTALL_PREFIX=$REPO_ROOT/install/$PLATFORM)
	export SDKROOT=$SYSROOT

	case $PLATFORM in
		"iphoneos")
			ARCH=arm64
			SYSROOT=`xcodebuild -version -sdk iphoneos Path`
			CMAKE_ARGS+=(-DCMAKE_OSX_ARCHITECTURES=$ARCH \
				-DCMAKE_OSX_SYSROOT=$SYSROOT);;

		"iphonesimulator")
			ARCH=arm64
			SYSROOT=`xcodebuild -version -sdk iphonesimulator Path`
			CMAKE_ARGS+=(-DCMAKE_OSX_ARCHITECTURES=$ARCH -DCMAKE_OSX_SYSROOT=$SYSROOT);;

		"maccatalyst")
			ARCH=arm64
			SYSROOT=`xcodebuild -version -sdk macosx Path`
			CMAKE_ARGS+=(-DCMAKE_C_FLAGS=-target\ $ARCH-apple-ios14.1-macabi);;

		"macosx")
			ARCH=arm64
			SYSROOT=`xcodebuild -version -sdk macosx Path`
			CMAKE_ARGS+=(-DCMAKE_OSX_ARCHITECTURES=$ARCH);;

		*)
			echo "Unsupported or missing platform! Must be one of" ${AVAILABLE_PLATFORMS[@]}
			exit 1;;
	esac
}

### Build libpcre for a given platform
function build_libpcre() {
	setup_variables $1

	##rm -rf pcre-8.45
	git clone https://github.com/light-tech/PCRE.git pcre-8.45 || true
	cd pcre-8.45

	rm -rf build && mkdir build && cd build
	CMAKE_ARGS+=(-DPCRE_BUILD_PCRECPP=NO \
		-DPCRE_BUILD_PCREGREP=NO \
		-DPCRE_BUILD_TESTS=NO \
		-DCMAKE_POLICY_DEFAULT_CMP0026=NEW \
                -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
		-DPCRE_SUPPORT_LIBBZ2=NO)

		cmake "${CMAKE_ARGS[@]}" ..
	
		cmake --build . --target install -j8
	}
	
	### Build openssl for a given platform
	function build_openssl() {
		setup_variables $1
	
		# It is better to remove and redownload the source since building make the source code directory dirty!
		## rm -rf openssl-3.0.4
		test -f openssl-3.0.4.tar.gz || curl -LO -s https://www.openssl.org/source/openssl-3.0.4.tar.gz
		tar xzf openssl-3.0.4.tar.gz
		cd openssl-3.0.4
	
		case $PLATFORM in
			"iphoneos")
				TARGET_OS=ios64-cross
				export CFLAGS="-isysroot $SYSROOT -arch $ARCH";;

			"iphonesimulator")
				TARGET_OS=darwin64-arm64-cc # Use specific target for arm64 simulator
				export CFLAGS="-isysroot $SYSROOT -arch $ARCH";;
	
			"maccatalyst"|"maccatalyst-arm64")
				TARGET_OS=darwin64-$ARCH-cc
				export CFLAGS="-isysroot $SYSROOT -target $ARCH-apple-ios14.1-macabi";;
	
			"macosx"|"macosx")
				TARGET_OS=darwin64-$ARCH-cc
				export CFLAGS="-isysroot $SYSROOT";;

			*)
				echo "Unsupported or missing platform!";;
		esac
	
		# See https://wiki.openssl.org/index.php/Compilation_and_Installation
		./Configure --prefix=$REPO_ROOT/install/$PLATFORM \
			--openssldir=$REPO_ROOT/install/$PLATFORM \
			$TARGET_OS no-shared no-dso no-hw no-engine
	
		make
		make install_sw install_ssldirs
		export -n CFLAGS
	}
	
	### Build libssh2 for a given platform (assume openssl was built)
	function build_libssh2() {
		setup_variables $1
	
		## rm -rf libssh2-1.10.0
		test -f libssh2-1.10.0.tar.gz || curl -LO -s https://www.libssh2.org/download/libssh2-1.10.0.tar.gz
		tar xzf libssh2-1.10.0.tar.gz
		cd libssh2-1.10.0
	
		rm -rf build && mkdir build && cd build
	
		CMAKE_ARGS+=(-DCRYPTO_BACKEND=OpenSSL \
			-DOPENSSL_ROOT_DIR=$REPO_ROOT/install/$PLATFORM \
			-DBUILD_EXAMPLES=OFF \
			-DCMAKE_POLICY_DEFAULT_CMP0026=NEW \
	                -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
			-DBUILD_TESTING=OFF)
	
		cmake "${CMAKE_ARGS[@]}" ..
	
		cmake --build . --target install -j8
	}
	
	### Build libgit2 for a single platform (given as the first and only argument)
	### See @setup_variables for the list of available platform names
	### Assume openssl and libssh2 was built
	function build_libgit2() {
	    setup_variables $1
	
	    ## rm -rf libgit2-1.3.1
	    test -f v1.3.1.zip || curl -LO -s https://github.com/libgit2/libgit2/archive/refs/tags/v1.3.1.zip
	    ditto -V -x -k --sequesterRsrc --rsrc v1.3.1.zip ./
	    cd libgit2-1.3.1
	
	    rm -rf build && mkdir build && cd build
	
	    CMAKE_ARGS+=(-DBUILD_CLAR=NO)
	
	    # See libgit2/cmake/FindPkgLibraries.cmake to understand how libgit2 looks for libssh2
	    # Basically, setting LIBSSH2_FOUND forces SSH support and since we are building static library,
	    # we only need the headers.
	    CMAKE_ARGS+=(-DOPENSSL_ROOT_DIR=$REPO_ROOT/install/$PLATFORM \
	        -DUSE_SSH=ON \
	        -DLIBSSH2_FOUND=YES \
			-DCMAKE_POLICY_DEFAULT_CMP0026=NEW \
	                -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
	        -DLIBSSH2_INCLUDE_DIRS=$REPO_ROOT/install/$PLATFORM/include)
	
	    cmake "${CMAKE_ARGS[@]}" ..
	
	    cmake --build . --target install -j8
	}		
		### Create xcframework for a given library
		function build_xcframework() {
			local FWNAME=$1
			shift
			local PLATFORMS=( "$@" )
			local FRAMEWORKS_ARGS=()
		
			echo "Building" $FWNAME "XCFramework containing" ${PLATFORMS[@]}
		
			for p in ${PLATFORMS[@]}; do
				FRAMEWORKS_ARGS+=("-library" "install/$p/$FWNAME.a" "-headers" "install/$p/include")
			done
		
			cd $REPO_ROOT
			xcodebuild -create-xcframework ${FRAMEWORKS_ARGS[@]} -output $FWNAME.xcframework
		}
		
		### Copy Clibgit2_modulemap to Clibgit2.xcframework/*/Headers
		### so that we can use libgit2 C API in Swift (e.g. via SwiftGit2)
		function copy_modulemap() {
		    local FWDIRS=$(find Clibgit2.xcframework -mindepth 1 -maxdepth 1 -type d)
		    for d in ${FWDIRS[@]}; do
		        echo $d
		        cp Clibgit2_modulemap $d/Headers/module.modulemap
		    done
		}
		
		### Build libgit2 and Clibgit2 frameworks for all available platforms
		
		for p in ${AVAILABLE_PLATFORMS[@]}; do
			echo "Build libraries for $p"
		    # Clean installation directory for the current platform to ensure a fresh build
		    rm -rf "$REPO_ROOT/install/$p"
			build_libpcre $p
			
		    cd $REPO_ROOT/install/$p
		    if [ -f lib/libpcre.a ]; then
		        echo "Architectures for install/$p/lib/libpcre.a:"
		        lipo -info lib/libpcre.a
		    else
		        echo "Warning: libpcre.a not found for platform $p"
		    fi
		    cd $REPO_ROOT # Go back to root before next build function
		
			build_openssl $p
		    cd $REPO_ROOT/install/$p
		    if [ -f lib/libcrypto.a ] && [ -f lib/libssl.a ]; then
		        echo "Architectures for install/$p/lib/libcrypto.a:"
		        lipo -info lib/libcrypto.a
		        echo "Architectures for install/$p/lib/libssl.a:"
		        lipo -info lib/libssl.a
		    else
		        echo "Warning: libcrypto.a or libssl.a not found for platform $p"
		    fi
		    cd $REPO_ROOT
		    
			build_libssh2 $p
		    cd $REPO_ROOT/install/$p
		    if [ -f lib/libssh2.a ]; then
		        echo "Architectures for install/$p/lib/libssh2.a:"
		        lipo -info lib/libssh2.a
		    else
		        echo "Warning: libssh2.a not found for platform $p"
		    fi
		    cd $REPO_ROOT
		    
			build_libgit2 $p
		    cd $REPO_ROOT/install/$p
		    if [ -f lib/libgit2.a ]; then
		        echo "Architectures for install/$p/lib/libgit2.a:"
		        lipo -info lib/libgit2.a
		    else
		        echo "Warning: libgit2.a not found for platform $p"
		    fi
		    cd $REPO_ROOT
		
			# Merge all static libs as Clibgit2.a since xcodebuild doesn't allow specifying multiple .a
			cd $REPO_ROOT/install/$p
			libtool -static -o libgit2.a lib/*.a
		    echo "Architectures for install/$p/libgit2.a (after libtool):"
		    lipo -info libgit2.a
		done
# Remove any explicit lipo commands that combine architectures prematurely.
# xcodebuild -create-xcframework will handle combining compatible architectures for the same platform.

# Build raw libgit2 XCFramework for Objective-C usage
#build_xcframework libgit2 ${AVAILABLE_PLATFORMS[@]}
#zip -r libgit2.xcframework.zip -i libgit2.xcframework/

# Build Clibgit2 XCFramework for use with SwiftGit2
build_xcframework libgit2 ${AVAILABLE_PLATFORMS[@]}
copy_modulemap libgit2
zip -r libgit2.xcframework.zip -i libgit2.xcframework/
rsync -r libgit2.xcframework/** ../libgit2.xcframework/
