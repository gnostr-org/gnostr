# gnostr-tui Source Directory

This directory contains the main source code for the gnostr-tui application, organized by feature and responsibility. Each subdirectory includes its own README with detailed API documentation and usage examples.

## Directory Overview

- [`components/`](components/README.md): Reusable UI and logic components (lists, popups, command info, etc.).
- [`tabs/`](tabs/README.md): Main UI tabs (Revlog, Nostr, Files, Status) implementing the core application views.
- [`keys/`](keys/README.md): Keyboard configuration, key bindings, and input handling.
- [`ui/`](ui/README.md): Shared UI utilities, themes, styles, and layout helpers.

## API Usage

- **Tabs and Components:**
  - Tabs are the main sections of the UI, each composed of one or more components.
  - Components encapsulate state, rendering, and event handling, and are reusable across tabs.
  - Both tabs and components implement trait-based APIs for event, draw, and command registration.

- **Key Handling:**
  - Key bindings are managed centrally and passed to tabs/components for consistent navigation and actions.

- **UI Theming and Layout:**
  - Shared themes and layout helpers ensure a consistent look and feel across the application.

## More Information

See the README in each subdirectory for detailed API documentation and usage examples:
- [components/README.md](components/README.md)
- [tabs/README.md](tabs/README.md)
- [keys/README.md](keys/README.md)
- [ui/README.md](ui/README.md)
