//! Tiny library for prompting sensitive or non-sensitive data on the terminal.
//!
//! The only dependency is `libc` on Unix and `winapi` on Windows.
//!
//! See [`Terminal`] for the API documentation.
//!
//! # Example
//! Read a username and password from the terminal:
//! ```no_run
//! # fn main() -> std::io::Result<()> {
//! use terminal_prompt::Terminal;
//! let mut terminal = Terminal::open()?;
//! let username = terminal.prompt("Username: ")?;
//! let password = terminal.prompt_sensitive("Password: ")?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

use std::io::{BufReader, BufRead, Read, Write};

mod sys;

/// A handle to the terminal associated with the current process.
///
/// Once opened, you can use [`Self::prompt()`] to read non-sensitive data from the terminal,
/// and [`Self::prompt_sensitive()`] to read sensitive data like passwords.
///
/// Alternatively, you can manually call [`Self::enable_echo()`] and [`Self::disable_echo()`], and read/write from the terminal directly.
/// The terminal handle implements the standard [`Read`], [`Write`] and [`BufRead`] traits,
/// and it has a [`Self::read_line()`] convenience function that returns a new string.
///
/// # Terminal modes
/// When opened, the terminal will be put in line editing mode.
/// When dropped, the original mode of the terminal will be restored.
/// Note that the terminal is inherently a global resource,
/// so creating multiple terminal objects and dropping them in a different order can cause the terminal to be left in a different mode.
pub struct Terminal {
	/// The underlying terminal.
	terminal: BufReader<sys::Terminal>,

	/// The mode of the terminal when we opened it.
	initial_mode: sys::TerminalMode,
}

impl Terminal {
	/// Open the terminal associated with the current process.
	///
	/// The exact behavior is platform dependent.
	///
	/// On Unix platforms, if one of standard I/O streams is a terminal, that terminal is used.
	/// First standard error is tried, then standard input and finally standard output.
	/// If none of those work, the function tries to open `/dev/tty`.
	/// This means that on Unix platforms, the terminal prompt can still work, even when both standard input and standard output are connected to pipes instead of the terminal.
	///
	/// On Windows, if both standard input and standard error are connected to a terminal, those streams are used.
	///
	/// In all cases, if the function fails to find a terminal for the process, an error is returned.
	pub fn open() -> std::io::Result<Self> {
		// Open the terminal and retrieve the initial mode.
		let terminal = sys::Terminal::open()?;
		let initial_mode = terminal.get_terminal_mode()?;

		// Enable line editing mode.
		let mut mode = initial_mode;
		mode.enable_line_editing();
		terminal.set_terminal_mode(&mode)?;

		Ok(Self {
			terminal: BufReader::new(terminal),
			initial_mode,
		})
	}

	/// Check if the terminal is echoing input.
	///
	/// If enabled, any text typed on the terminal will be visible.
	pub fn is_echo_enabled(&self) -> std::io::Result<bool> {
		let mode = self.terminal.get_ref().get_terminal_mode()?;
		Ok(mode.is_echo_enabled())
	}

	/// Disable echoing of terminal input.
	///
	/// This will prevent text typed on the terminal from being visible.
	/// This can be used to hide passwords while they are being typed.
	pub fn disable_echo(&self) -> std::io::Result<()> {
		let mut mode = self.terminal.get_ref().get_terminal_mode()?;
		mode.disable_echo();
		self.terminal.get_ref().set_terminal_mode(&mode)?;
		Ok(())
	}

	/// Enable echoing of terminal input.
	///
	/// This will cause any text typed on the terminal to be visible.
	pub fn enable_echo(&mut self) -> std::io::Result<()> {
		let mut mode = self.terminal.get_ref().get_terminal_mode()?;
		mode.enable_echo();
		self.terminal.get_ref().set_terminal_mode(&mode)?;
		Ok(())
	}

	/// Read a line of input from the terminal.
	///
	/// If echoing is disabled, this will also print a newline character to visually indicate to the user.
	/// If this is not desired, use the [`BufRead::read_line()`] function instead.
	pub fn read_input_line(&mut self) -> std::io::Result<String> {
		let mut buffer = String::new();
		self.terminal.read_line(&mut buffer)?;

		if self.is_echo_enabled().ok() == Some(false) {
			writeln!(self).ok();
		}
		if buffer.ends_with('\n') {
			buffer.pop();
		}
		Ok(buffer)
	}

	/// Prompt the user on the terminal.
	///
	/// This function does not enable or disable echoing and should not normally be used for reading sensitive data like passwords.
	/// Consider [`Self::prompt_sensitive()`] instead.
	pub fn prompt(&mut self, prompt: impl std::fmt::Display) -> std::io::Result<String> {
		write!(self, "{prompt}")?;
		self.read_input_line()
	}

	/// Prompt the user for sensitive data (like passwords) on the terminal.
	///
	/// This function makes sure that echoing is disabled before the prompt is shown.
	/// If echoing was enabled, it is re-enabled after the response is read.
	///
	/// Use [`Self::prompt()`] to read non-sensitive data.
	pub fn prompt_sensitive(&mut self, prompt: impl std::fmt::Display) -> std::io::Result<String> {
		let old_mode = self.terminal.get_ref().get_terminal_mode()?;
		if old_mode.is_echo_enabled() {
			let mut new_mode = old_mode;
			new_mode.disable_echo();
			self.terminal.get_ref().set_terminal_mode(&new_mode)?;
		}
		write!(self, "{prompt}")?;
		let line = self.read_input_line();
		if old_mode.is_echo_enabled() {
			self.terminal.get_ref().set_terminal_mode(&old_mode).ok();
		}
		line
	}
}

impl Drop for Terminal {
	fn drop(&mut self) {
		self.terminal.get_ref().set_terminal_mode(&self.initial_mode).ok();
	}
}

impl Read for Terminal {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.terminal.read(buf)
	}

	fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
		self.terminal.read_vectored(bufs)
	}
}

impl BufRead for Terminal {
	fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
		self.terminal.fill_buf()
	}

	fn consume(&mut self, amt: usize) {
		self.terminal.consume(amt)
	}
}

impl Write for Terminal {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.terminal.get_mut().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.terminal.get_mut().flush()
	}

	fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
		self.terminal.get_mut().write_vectored(bufs)
	}
}
