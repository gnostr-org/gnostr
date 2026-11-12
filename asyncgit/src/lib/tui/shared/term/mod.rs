use crate::{tui::git::gitui_error::Error, tui::git::Res};
use crossterm::{
    event::Event,
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    backend::{Backend, ClearType, CrosstermBackend, IntoCrossterm},
    layout::Size,
    prelude::{backend::WindowSize, buffer::Cell, Position},
    Terminal,
};
use std::io::{self, stderr, Stderr};
use std::{fmt::Display, time::Duration};

///
pub type Term = Terminal<TermBackend>;

// TODO It would be more logical if the following top-level functions also were in 'TermBackend'.
//      However left here for now.

///
pub fn alternate_screen<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    stderr()
        .execute(EnterAlternateScreen)
        .map_err(|e| Error::Term(e.into()))?;
    let result = fun();
    stderr()
        .execute(LeaveAlternateScreen)
        .map_err(|e| Error::Term(e.into()))?;
    result
}

///
pub fn raw_mode<T, F: Fn() -> Res<T>>(fun: F) -> Res<T> {
    let was_raw_mode_enabled = is_raw_mode_enabled().map_err(|e| Error::Term(e.into()))?;

    if !was_raw_mode_enabled {
        enable_raw_mode().map_err(|e| Error::Term(e.into()))?;
    }

    let result = fun();

    if !was_raw_mode_enabled {
        disable_raw_mode().map_err(|e| Error::Term(e.into()))?;
    }

    result
}

///
pub fn cleanup_alternate_screen() {
    print_err(stderr().execute(LeaveAlternateScreen));
}

///
pub fn cleanup_raw_mode() {
    print_err(disable_raw_mode());
}

///
fn print_err<T, E: Display>(result: Result<T, E>) {
    match result {
        Ok(_) => (),
        Err(error) => eprintln!("Error: {}", error),
    };
}

///
pub fn backend() -> TermBackend {
    TermBackend::Crossterm(CrosstermBackend::new(stderr()))
}

/// Error type for [`TermBackend`].
#[derive(Debug)]
pub enum TermBackendError {
    ///
    Io(io::Error),
    ///
    Infallible(std::convert::Infallible),
}

impl std::fmt::Display for TermBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TermBackendError::Io(e) => e.fmt(f),
            TermBackendError::Infallible(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for TermBackendError {}

impl From<io::Error> for TermBackendError {
    fn from(e: io::Error) -> Self {
        TermBackendError::Io(e)
    }
}

impl From<std::convert::Infallible> for TermBackendError {
    fn from(e: std::convert::Infallible) -> Self {
        TermBackendError::Infallible(e)
    }
}

impl From<TermBackendError> for io::Error {
    fn from(e: TermBackendError) -> Self {
        match e {
            TermBackendError::Io(e) => e,
            TermBackendError::Infallible(e) => match e {},
        }
    }
}

///
pub enum TermBackend {
    ///
    Crossterm(CrosstermBackend<Stderr>),
    #[allow(dead_code)]
    ///
    Test {
        ///
        backend: ratatui::backend::TestBackend,
        ///
        events: Vec<Event>,
    },
}

///
impl Backend for TermBackend {
    type Error = TermBackendError;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        match self {
            TermBackend::Crossterm(t) => t.draw(content).map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.draw(content).map_err(Into::into),
        }
    }

    ///
    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.hide_cursor().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.hide_cursor().map_err(Into::into),
        }
    }

    ///
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.show_cursor().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.show_cursor().map_err(Into::into),
        }
    }

    ///
    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.get_cursor_position().map_err(Into::into),
            TermBackend::Test { backend, .. } => {
                backend.get_cursor_position().map_err(Into::into)
            }
        }
    }

    ///
    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.set_cursor_position(position).map_err(Into::into),
            TermBackend::Test { backend, .. } => {
                backend.set_cursor_position(position).map_err(Into::into)
            }
        }
    }

    ///
    fn clear(&mut self) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.clear().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.clear().map_err(Into::into),
        }
    }

    ///
    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.clear_region(clear_type).map_err(Into::into),
            TermBackend::Test { backend, .. } => {
                backend.clear_region(clear_type).map_err(Into::into)
            }
        }
    }

    ///
    fn size(&self) -> Result<Size, Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.size().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.size().map_err(Into::into),
        }
    }

    ///
    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.window_size().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.window_size().map_err(Into::into),
        }
    }

    ///
    fn flush(&mut self) -> Result<(), Self::Error> {
        match self {
            TermBackend::Crossterm(t) => t.flush().map_err(Into::into),
            TermBackend::Test { backend, .. } => backend.flush().map_err(Into::into),
        }
    }
}

///
impl TermBackend {
    ///
    pub fn enter_alternate_screen(&mut self) -> Res<()> {
        match self {
            TermBackend::Crossterm(c) => c
                .execute(EnterAlternateScreen)
                .map_err(|e| Error::Term(e.into()))
                .map(|_| ()),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    ///
    pub fn enable_raw_mode(&self) -> Res<()> {
        match self {
            TermBackend::Crossterm(_) => enable_raw_mode().map_err(|e| Error::Term(e.into())),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    ///
    pub fn disable_raw_mode(&self) -> Res<()> {
        match self {
            TermBackend::Crossterm(_) => disable_raw_mode().map_err(|e| Error::Term(e.into())),
            TermBackend::Test { .. } => Ok(()),
        }
    }

    ///
    pub fn poll_event(&self, timeout: Duration) -> Res<bool> {
        match self {
            TermBackend::Crossterm(_) => crossterm::event::poll(timeout).map_err(|e| Error::Term(e.into())),
            TermBackend::Test { events, .. } => {
                if events.is_empty() {
                    Err(Error::NoMoreEvents)
                } else {
                    Ok(true)
                }
            }
        }
    }

    ///
    pub fn read_event(&mut self) -> Res<Event> {
        match self {
            TermBackend::Crossterm(_) => crossterm::event::read().map_err(|e| Error::Term(e.into())),
            TermBackend::Test { events, .. } => events.pop().ok_or(Error::NoMoreEvents),
        }
    }
}
