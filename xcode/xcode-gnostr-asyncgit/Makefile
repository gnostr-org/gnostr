SHELL := /bin/bash

PROJECT := swiftyapp/swiftyapp.xcodeproj
SCHEME := swiftyapp
BUILD_SCRIPT := ./build.sh
RUST_CRATE_DIR := rustylib
DERIVED_DATA ?= .build/xcode
CONFIGURATION ?= Debug
APP_NAME := swiftyapp.app
INSTALL_DIR ?= $(HOME)/Applications
SYMROOT := $(abspath $(DERIVED_DATA))
OBJROOT := $(SYMROOT)/Intermediates.noindex
HOST_ARCH := $(shell uname -m)
SIMULATOR_ARCH := $(if $(filter arm64,$(HOST_ARCH)),arm64,x86_64)
IOS_DESTINATION ?= generic/platform=iOS Simulator
CATALYST_DESTINATION ?= generic/platform=macOS,variant=Mac Catalyst
CATALYST_APP_PATH := $(SYMROOT)/$(CONFIGURATION)-maccatalyst/$(APP_NAME)

.DEFAULT_GOAL := help

.PHONY: help rust resolve app catalyst install clean

help:
	@printf '%s\n' \
		'make rust      - build the Rust library, bindings, and XCFramework' \
		'make resolve   - resolve local Swift package dependencies' \
		'make app       - build the iOS app for a generic simulator destination' \
		'make catalyst  - build the app for Mac Catalyst' \
		'make install   - install the Mac Catalyst app into ~/Applications' \
		'make clean     - clean Rust and Xcode build artifacts'

rust:
	$(BUILD_SCRIPT)

resolve:
	xcodebuild -resolvePackageDependencies -project "$(PROJECT)" -scheme "$(SCHEME)"

app: rust resolve
	xcodebuild \
		-project "$(PROJECT)" \
		-scheme "$(SCHEME)" \
		-configuration "$(CONFIGURATION)" \
		-derivedDataPath "$(DERIVED_DATA)" \
		SYMROOT="$(SYMROOT)" \
		OBJROOT="$(OBJROOT)" \
		-destination "$(IOS_DESTINATION)" \
		ARCHS="$(SIMULATOR_ARCH)" \
		ONLY_ACTIVE_ARCH=YES \
		build

catalyst: rust resolve
	xcodebuild \
		-project "$(PROJECT)" \
		-scheme "$(SCHEME)" \
		-configuration "$(CONFIGURATION)" \
		-derivedDataPath "$(DERIVED_DATA)" \
		SYMROOT="$(SYMROOT)" \
		OBJROOT="$(OBJROOT)" \
		-destination "$(CATALYST_DESTINATION)" \
		ARCHS="$(HOST_ARCH)" \
		ONLY_ACTIVE_ARCH=YES \
		build

install: catalyst
	mkdir -p "$(INSTALL_DIR)"
	rm -rf "$(INSTALL_DIR)/$(APP_NAME)"
	cp -R "$(CATALYST_APP_PATH)" "$(INSTALL_DIR)/$(APP_NAME)"

clean:
	cd "$(RUST_CRATE_DIR)" && cargo clean
	rm -rf "$(DERIVED_DATA)"