use std::fs::File;
use std::io::{Read, Write};
use std::mem::ManuallyDrop;
use std::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, RawFd};

/// Unix handle to an open terminal.
pub enum Terminal {
	/// Non-owning file for one of the standard I/O streams.
	Stdio(ManuallyDrop<File>),

	/// Owned file for `/dev/tty`.
	File(File),
}

#[derive(Copy, Clone)]
pub struct TerminalMode {
	termios: libc::termios,
}

impl Terminal {
	pub fn open() -> std::io::Result<Self> {
		if let Some(terminal) = open_fd_terminal(2) {
			Ok(terminal)
		} else if let Some(terminal) = open_fd_terminal(0) {
			Ok(terminal)
		} else if let Some(terminal) = open_fd_terminal(1) {
			Ok(terminal)
		} else {
			let file = std::fs::OpenOptions::new()
				.read(true)
				.write(true)
				.open("/dev/tty")?;
			if is_terminal(file.as_fd()) {
				Ok(Self::File(file))
			} else {
				Err(std::io::Error::from_raw_os_error(libc::ENOTTY))
			}
		}
	}

	pub fn get_terminal_mode(&self) -> std::io::Result<TerminalMode> {
		unsafe {
			let mut termios = std::mem::zeroed();
			check_ret(libc::tcgetattr(self.as_fd().as_raw_fd(), &mut termios))?;
			Ok(TerminalMode { termios })
		}
	}

	pub fn set_terminal_mode(&self, mode: &TerminalMode) -> std::io::Result<()> {
		unsafe {
			check_ret(libc::tcsetattr(
					self.as_fd().as_raw_fd(),
					libc::TCSANOW,
					&mode.termios,
			))?;
			Ok(())
		}
	}

	fn as_file(&self) -> &File {
		match self {
			Self::Stdio(io) => io,
			Self::File(io) => io,
		}
	}
}

fn open_fd_terminal(fd: RawFd) -> Option<Terminal> {
	let file = unsafe { ManuallyDrop::new(File::from_raw_fd(fd)) };
	if is_terminal(file.as_fd()) {
		Some(Terminal::Stdio(file))
	} else {
		None
	}
}

impl TerminalMode {
	pub fn enable_line_editing(&mut self) {
		self.termios.c_lflag |= libc::ICANON;
	}

	pub fn disable_echo(&mut self) {
		self.termios.c_lflag &= !libc::ECHO;
		self.termios.c_lflag &= !libc::ICANON;
	}

	pub fn enable_echo(&mut self) {
		self.termios.c_lflag |= libc::ECHO;
		self.termios.c_lflag |= !libc::ICANON;
	}

	pub fn is_echo_enabled(&self) -> bool {
		self.termios.c_lflag & libc::ECHO != 0
	}
}

fn is_terminal(fd: BorrowedFd) -> bool {
	unsafe {
		libc::isatty(fd.as_raw_fd()) == 1
	}
}

fn check_ret(input: i32) -> std::io::Result<()> {
	if input == 0 {
		Ok(())
	} else {
		Err(std::io::Error::last_os_error())
	}
}

impl AsFd for Terminal {
	fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
		match self {
			Self::Stdio(stdin) => stdin.as_fd(),
			Self::File(file) => file.as_fd(),
		}
	}
}

impl Read for Terminal {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		self.as_file().read(buf)
	}

	fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
		self.as_file().read_vectored(bufs)
	}
}

impl Write for Terminal {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.as_file().write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.as_file().flush()
	}

	fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
		self.as_file().write_vectored(bufs)
	}
}
