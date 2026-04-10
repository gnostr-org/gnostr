# Tabs API Documentation

This directory contains the main UI tabs for the gnostr-tui application. Each tab represents a major section of the interface (e.g., Revlog, Nostr, Files, Status) and is responsible for its own state, rendering, and event handling.

## Tab Structure

- Each tab is implemented as a Rust module and typically exposes a struct (e.g., `RevlogTab`, `NostrTab`) that implements the `Component` and `DrawableComponent` traits.
- Tabs coordinate one or more components to provide their UI and logic.
- Tabs are managed and switched by the main application, which delegates events and rendering to the active tab.

## Common Tabs

- **RevlogTab**: Displays the git revision log with navigation and selection.
- **NostrTab**: Shows Nostr events (patches, issues, announcements) with list navigation.
- **FilesTab**: File browser for the repository.
- **StatusTab**: Shows repository status and changes.

## Tab API

- `fn event(&mut self, ev: &Event) -> Result<EventState>`: Handle input events.
- `fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) -> Result<()>`: Render the tab.
- `fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking`: Register tab-specific commands.
- `fn is_visible(&self) -> bool`, `fn show(&mut self)`, `fn hide(&mut self)`: Visibility control.

## Adding New Tabs
- Create a new module in this directory.
- Implement the required traits and expose a public struct.
- Register the tab in `mod.rs` for use in the application.

See individual tab files for more details and implementation examples.
