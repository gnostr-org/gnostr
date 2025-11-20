use std::io::{Read, Write};
use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle};

use winapi::um::consoleapi::{
	GetConsoleMode,
	SetConsoleMode,
};
use winapi::um::wincon::{
	ENABLE_LINE_INPUT,
	ENABLE_ECHO_INPUT,
};

use winapi::shared::minwindef::{BOOL, DWORD};

pub struct Terminal {
	input: std::io::Stdin,
	output: std::io::Stderr,
}

#[derive(Copy, Clone)]
pub struct TerminalMode {
	input_mode: DWORD,
}

impl Terminal {
	pub fn open() -> std::io::Result<Self> {
		let input = std::io::stdin();
		let output = std::io::stderr();
		if !is_terminal(input.as_handle()) {
			return Err(std::io::Error::new(std::io::ErrorKind::Other, "stdin is not a terminal"));
		}
		if !is_terminal(output.as_handle()) {
			return Err(std::io::Error::new(std::io::ErrorKind::Other, "stderr is not a terminal"));
		}
		Ok(Self {
			input,
			output,
		})
	}

	pub fn get_terminal_mode(&self) -> std::io::Result<TerminalMode> {
		unsafe {
			let mut input_mode = 0;
			check_ret(GetConsoleMode(self.input.as_raw_handle().cast(), &mut input_mode))?;
			Ok(TerminalMode {
				input_mode,
			})
		}
	}

	pub fn set_terminal_mode(&self, mode: &TerminalMode) -> std::io::Result<()> {
		unsafe {
			check_ret(SetConsoleMode(
				self.input.as_raw_handle().cast(),
				mode.input_mode,
			))?;
			Ok(())
		}
	}
}

impl TerminalMode {
	pub fn enable_line_editing(&mut self) {
		self.input_mode |= ENABLE_LINE_INPUT;
	}

	pub fn disable_echo(&mut self) {
		self.input_mode &= !ENABLE_ECHO_INPUT;
	}

	pub fn enable_echo(&mut self) {
		self.input_mode |= ENABLE_ECHO_INPUT;
	}

	pub fn is_echo_enabled(&self) -> bool {
		self.input_mode & ENABLE_ECHO_INPUT != 0
	}
}

fn is_terminal(handle: BorrowedHandle) -> bool {
	unsafe {
		let mut mode = 0;
		GetConsoleMode(handle.as_raw_handle().cast(), &mut mode) != 0
	}
}

fn check_ret(input: BOOL) -> std::io::Result<()> {
	if input != 0 {
		Ok(())
	} else {
		Err(std::io::Error::last_os_error())
	}
}

impl Read for Terminal {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.input.read(buf)
	}

	fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
		self.input.read_vectored(bufs)
	}
}

impl Write for Terminal {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.output.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.output.flush()
	}

	fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
		self.output.write_vectored(bufs)
	}
}
