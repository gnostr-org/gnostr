use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use sha2::{Digest, Sha256};
use std::io;
use std::io::Write;

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;

    //println!("Enter input to hash Enter:\r");

    let mut input = String::new();

    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) => {
                input.push(c);
                print!("{}", c); // Echo the character.
                io::stdout().flush().unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("\r"); // Move to the next line.
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind: KeyEventKind::Press,
                ..
            }) => {
                input.pop();
                print!("\x08 \x08"); // Backspace in terminal
                io::stdout().flush().unwrap();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("\rEscape pressed, exiting.\r");
                disable_raw_mode()?;
                return Ok(());
            }
            _ => {}
        }
    }

    disable_raw_mode()?;

    // SHA-256 hashing
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    print!("{:x}", result);

    Ok(())
}
