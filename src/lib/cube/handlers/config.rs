use std::{
    fs::{create_dir_all, read_to_string, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use color_eyre::eyre::{bail, Error, Result};
use serde::{Deserialize, Serialize};

use crate::utils::pathing::config_path;

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
        let path_str = config_path("config.toml");

        let p = Path::new(&path_str);
        //println!("path_str:\n{:?}", p);

        if !p.exists() {
            create_dir_all(p.parent().unwrap()).unwrap();

            let default_toml_string = toml::to_string(&Self::default()).unwrap();
            let mut file = File::create(path_str.clone()).unwrap();
            file.write_all(default_toml_string.as_bytes()).unwrap();
            //println!("default_toml_string:\n{:?}", default_toml_string);

            let config: Self = toml::from_str(default_toml_string.as_str()).unwrap();
            print!("Configuration was generated at {path_str}, please fill it out with necessary information.");
            Ok(config)
        } else if let Ok(config_contents) = read_to_string(p) {
            let config: Self = toml::from_str(config_contents.as_str()).unwrap();

            // Remember to check for any important missing config items here!
            //println!("config_contents:\n{:?}", config);

            Ok(config)
        } else {
            bail!(
                "Configuration could not be read correctly. See the following link for the example config: {}",
                format!("{}/blob/main/default-config.toml", env!("CARGO_PKG_REPOSITORY"))
            )
        }
    }
}
