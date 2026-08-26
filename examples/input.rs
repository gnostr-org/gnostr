//! Demonstrates how to block read characters or a full line.
//! Just note that crossterm is not required to do this and can be done with `io::stdin()`.
//!
//! cargo run --example event-read-char-line

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal,
};

pub fn read_char() -> io::Result<char> {
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            return Ok(c);
        }
    }
}

pub fn read_line() -> io::Result<String> {
    let mut line = String::new();
    loop {
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char(c) => {
                    line.push(c);
                }
                _ => {}
            }
        }
    }

    Ok(line)
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;

    println!("Enter characters. Each character will be printed:\r");

    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("{}\r", c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("Enter pressed, exiting.\r");
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => {
                println!("Escape pressed, exiting.\r");
                break;
            }
            _ => {}
        }
    }

    //println!("read line:\r");
    //println!("{:?}\r", read_line());
    //println!("read char:\r");
    //println!("{:?}\r", read_char());
    //println!("read char again:\r");
    //println!("{:?}\r", read_char());

    terminal::disable_raw_mode()
}
