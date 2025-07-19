use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use tui_input::backend::crossterm::EventHandler;
//use tui_input::*;

// Removed `use ratatui::prelude::*;` as it's not used in this standalone example
use tui_input::Input;
// Removed `use tui_input::backend::crossterm::EventHandler;` as it's implicit for `handle_event`

fn main() {
    // Scenario 1: Create an Input instance with an initial string value
    let initial_value = "Hello, TUI Input!";
    let mut input_field = Input::new(initial_value.to_string());

    println!("Initial Input Value: {:?}", input_field.value());
    // Cursor position is now obtained with `cursor_pos()`
    println!("Cursor position: {}", input_field.cursor());

    // IMPORTANT: The `Input` struct manages its cursor position internally based on events.
    // There is no direct `set_cursor_position` method to arbitrarily move it.
    // When you simulate typing via `handle_event`, the cursor will move automatically.
    // For TUI rendering, you use `input_field.cursor_pos()` to tell Ratatui where to draw the *terminal* cursor.

    // Scenario 2: Simulate typing into the input field
    // (This demonstrates how input is typically handled in a TUI event loop)
    println!("\nSimulating typing (appending to the end):");

    // Simulate typing 'W', 'o', 'r', 'l', 'd'
    // The cursor will advance automatically after each character.
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('W'),
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('o'),
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('r'),
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('l'),
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('d'),
        KeyModifiers::NONE,
    )));

    println!("Current Input Value: {:?}", input_field.value());
    println!("Cursor position: {}", input_field.cursor()); // Use cursor_pos() here

    // You can also simulate arrow keys to move the cursor
    println!("\nSimulating moving cursor left, then typing 'X':");
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Left,
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Left,
        KeyModifiers::NONE,
    )));
    input_field.handle_event(&Event::Key(KeyEvent::new(
        KeyCode::Char('X'),
        KeyModifiers::NONE,
    )));

    println!("Current Input Value: {:?}", input_field.value());
    println!("Cursor position: {}", input_field.cursor());

    // Scenario 3: Getting the string out of the Input instance
    let current_string: &str = input_field.value();
    let owned_string: String = input_field.value().to_string();

    println!("\nRetrieved string (str reference): {}", current_string);
    println!("Retrieved string (owned String): {}", owned_string);
}
