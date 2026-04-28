#![deny(clippy::pedantic)]

pub mod chat;
//pub mod css;
pub mod database;
pub mod git;
//pub mod js;
pub mod kill_process;
pub mod layers;
pub mod layout_html;
pub mod methods;
pub mod syntax_highlight;
pub mod template_html;
pub mod unified_diff_builder;
pub mod websock_index_html;
pub use crate::app::{
    database::schema::{commit::Commit, repository::Repository, tag::Tag},
    git::Git,
    syntax_highlight::prime_highlighters,
    unified_diff_builder::UnifiedDiffBuilder,
};

pub mod assets;
pub mod startup;
pub use assets::*;
pub use startup::*;
