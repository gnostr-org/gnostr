# Keys Module Documentation

This directory manages keyboard configuration and key handling for the gnostr-tui application.

## Key Features
- Defines key bindings for navigation and actions across the TUI.
- Supports user-customizable key mappings via configuration files.
- Provides utilities for matching and interpreting key events.

## Main Components
- **KeyConfig**: Struct holding all key bindings for the application.
- **key_match**: Utility function to compare input events to configured keys.
- **SharedKeyConfig**: Reference-counted wrapper for sharing key config across components.

## Usage
- Import `KeyConfig` and `key_match` to handle keyboard input in components and tabs.
- Use `SharedKeyConfig` to pass key configuration to UI elements.

See the source files for details on available keys and customization options.
