# terminal-prompt

Tiny library for prompting sensitive or non-sensitive data on the terminal.

The only dependency is `libc` on Unix and `winapi` on Windows.

See [`Terminal`] for the API documentation.

## Example
Read a username and password from the terminal:
```rust
use terminal_prompt::Terminal;
let mut terminal = Terminal::open()?;
let username = terminal.prompt("Username: ")?;
let password = terminal.prompt_sensitive("Password: ")?;
```

[`Terminal`]: https://docs.rs/terminal-prompt/latest/terminal_prompt/struct.Terminal.html
