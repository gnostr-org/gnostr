use std::str::FromStr;
use anyhow::{bail, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CompleteConfig {
    /// Internal functionality
    pub terminal: TerminalConfig,
    /// What everything looks like to the user
    pub frontend: FrontendConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    /// How often the terminal will update
    pub tick_delay: u64,
}
impl Default for TerminalConfig {
    fn default() -> Self {
        Self { tick_delay: 3 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct FrontendConfig {
    /// The margin around the main window from to the terminal border
    pub margin: u16,
    /// The shape of the cursor in insert boxes.
    pub cursor_shape: CursorType,
    /// If the cursor should be blinking.
    pub blinking_cursor: bool,
    pub default_message: String,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            margin: 2,
            cursor_shape: CursorType::User,
            blinking_cursor: true,
            default_message: format!(
                "> defaults:\n> margin={}\n> Cursor::Type={:?}\n> blinking_cursor={}",
                2,
                CursorType::User,
                true
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CursorType {
    User,
    Line,
    Block,
    UnderScore,
}

impl FromStr for CursorType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "line" => Ok(Self::Line),
            "underscore" => Ok(Self::UnderScore),
            _ => Ok(Self::Block),
        }
    }
}

impl Default for CursorType {
    fn default() -> Self {
        Self::User
    }
}

impl CompleteConfig {
    pub fn new() -> Result<Self, Error> {
        Ok(Self::default())
    }
}
