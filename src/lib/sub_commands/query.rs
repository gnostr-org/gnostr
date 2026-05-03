use crate::query::ConfigBuilder;
use anyhow::{anyhow, bail};
use log::{debug, error};
use serde_json::{json, to_string, Value};
use url::Url;

pub use crate::query::cli::QuerySubCommand;

// ... existing file content unchanged ...
