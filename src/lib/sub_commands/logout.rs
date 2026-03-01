use anyhow::{Context, Result};
use crate::ngit::{git::remove_git_config_item, login::{SignerInfoSource, existing::load_existing_login},};

use crate::{
    git::Repo,
    sub_commands::login::{format_items_as_list, get_global_login_config_items_set},
};

pub async fn launch() -> Result<()> {
    let git_repo_result = Repo::discover().context("failed to find a git repository");
    let git_repo = {
        git_repo_result.ok()
    };
    logout(git_repo.as_ref()).await
}

async fn logout(git_repo: Option<&Repo>) -> Result<()> {
    for source in if std::env::var("NGITTEST").is_ok() {
        vec![crate::cli::SignerInfoSource::GitLocal]
    } else {
        vec![crate::cli::SignerInfoSource::GitLocal, crate::cli::SignerInfoSource::GitGlobal]
    } {
        if let Ok((_, user_ref, source)) = load_existing_login(
            &git_repo,
            &None,
            &None,
            &Some(source),
            None,
            true,
            false,
            false,
        )
        .await
        {
            for item in [
                "nostr.nsec",
                "nostr.npub",
                "nostr.bunker-uri",
                "nostr.bunker-app-key",
            ] {
                if let Err(error) = remove_git_config_item(
                    if source == crate::cli::SignerInfoSource::GitLocal {
                        &git_repo
                    } else {
                        &None
                    },
                    item,
                ) {
                    eprintln!("{error:?}");
                    eprintln!(
                        "consider manually removing {} git config items: {}",
                        if source == crate::cli::SignerInfoSource::GitGlobal {
