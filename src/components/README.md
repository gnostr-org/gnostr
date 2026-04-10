# Components API Documentation

This directory contains reusable UI and logic components for the gnostr-tui application. Components encapsulate state, rendering, and event handling for specific UI elements or features. All components follow a trait-based API for integration with the main application.

## Component Traits

- **Component**: Core trait for all interactive UI elements. Provides methods for event handling, command registration, and visibility control.
  - `fn event(&mut self, ev: &Event) -> Result<EventState>`: Handle input events.
  - `fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking`: Register available commands for help menus.
  - `fn is_visible(&self) -> bool`: Query visibility.
  - `fn show(&mut self)`: Make component visible.
  - `fn hide(&mut self)`: Hide component.

- **DrawableComponent**: Extends `Component` with rendering support.
  - `fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) -> Result<()>`: Render the component in the given area.

## Common Components

- **NostrListComponent**: Displays and navigates a list of Nostr items (patches, issues, announcements). Supports up/down navigation, selection, and status updates.
- **CommandInfo/CommandText**: Structures for describing available commands and their help text.
- **StatusTreeComponent, BlameFileComponent, etc.**: Specialized components for git status, blame, and more.

## Usage

To use a component:
1. Create and configure the component (e.g., with theme and key config).
2. Integrate it into your tab or UI module.
3. Delegate event, draw, and command methods to the component as needed.

## Adding New Components
- Place new component modules in this directory.
- Implement the `Component` and/or `DrawableComponent` traits.
- Document public methods and expected usage.

See individual component files for more details and examples.
