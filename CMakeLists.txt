if (MSVC)
 cmake_minimum_required(VERSION 3.26)
else()
 cmake_minimum_required(VERSION 3.15)
endif() 
set(CMAKE_BUILD_TYPE Debug)
option(BUILD_RELEASE "BUILD_RELEASE" ON)
if(BUILD_RELEASE)
 set(CMAKE_BUILD_TYPE Release)
endif()

project (nostril C)

include_directories(${CMAKE_SOURCE_DIR}/ext/secp256k1/include)
#add_subdirectory(${CMAKE_SOURCE_DIR}/ext/secp256k1)
#set_target_properties(libsecp256k1.a PROPERTIES IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/ext/secp256k1/sec/libsecp256k1.a)
#add_library(${CMAKE_SOURCE_DIR}/libsecp256k1.a SHARED IMPORTED)

##set(lib_dep ${lib_dep} secp256k1)
# Provide the full path to the library, so CMake knows where to find it.
#set(lib_dep ${lib_dep} ${CMAKE_SOURCE_DIR}/libsecp256k1.a)


#//////////////////////////
#windows.h header file has macros for min and max, nostril defines max
#//////////////////////////

if (MSVC)
  add_definitions(-DNOMINMAX)
  add_definitions(-D_CRT_SECURE_NO_WARNINGS)
  add_definitions(-D_CRT_NONSTDC_NO_DEPRECATE)
endif()

#//////////////////////////
#sources 
#//////////////////////////

set(src ${src})
set(src ${src} hex.h)
set(src ${src} proof.h)
set(src ${src} cursor.h)
set(src ${src} endian.h)
set(src ${src} random.h)
set(src ${src} sha256.h)
set(src ${src} sha256.c)
set(src ${src} base64.h)
set(src ${src} base64.c)
set(src ${src} aes.h)
set(src ${src} aes.c)
if (MSVC)
  set(src ${src} clock_gettime.h)
endif()

#//////////////////////////
#link with libraries
#lib_dep contains a cascade definition of all the libraries needed to link
#//////////////////////////

set(lib_dep ${lib_dep})
if (MSVC)
  set(lib_dep ${lib_dep} ${CMAKE_BINARY_DIR}/ext/secp256k1/src/Debug/libsecp256k1.lib)
  set(lib_dep ${lib_dep} Bcrypt.lib)
else()
  ##set(lib_dep ${lib_dep} ${CMAKE_BINARY_DIR}/libsecp256k1.a)
  set(lib_dep ${lib_dep} ${CMAKE_SOURCE_DIR}/libsecp256k1.a)
endif()


#//////////////////////////
#executables
#//////////////////////////

# tool to generate file 'config.h' , generate from a shell with 
# configurator.exe > config.h
add_executable(configurator configurator.c)

add_executable(nostril ${src} nostril.c)
target_link_libraries (nostril ${lib_dep})

#//////////////////////////
# generate  config.h
#//////////////////////////

add_custom_command(
    TARGET configurator
    POST_BUILD
    COMMAND configurator > config.h
    COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_BINARY_DIR}/config.h" ${CMAKE_SOURCE_DIR}
    COMMENT "generating config.h"
)
## add_custom_command(
##     TARGET secp256k1
##     PRE_BUILD
##     COMMAND make secp256k1
##     COMMENT "make secp256k1"
## )

execute_process (
    COMMAND bash -c "git update-index --assume-unchanged Makefile"
    OUTPUT_VARIABLE outVar
)

#//////////////////////////
# Install
#//////////////////////////

install(TARGETS nostril
        CONFIGURATIONS Debug
        RUNTIME DESTINATION ${CMAKE_SOURCE_DIR}/bin)
install(TARGETS nostril
        CONFIGURATIONS Release
        RUNTIME DESTINATION /usr/local/bin)
