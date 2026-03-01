use anyhow::{Context, Result};
use crate::ngit::{git::{remove_git_config_item, get_git_config_item}, login::{SignerInfoSource, existing::load_existing_login},};

use crate::{ngit::git::Repo, sub_commands::login::{format_items_as_list, get_global_login_config_items_set},};

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
                    let source_type = if source == crate::cli::SignerInfoSource::GitGlobal { "global" } else { "local" };
                    eprintln!(
                        "consider manually removing {source_type} git config items: {}",
                        format_items_as_list(&get_global_login_config_items_set())
                    );
                    return Ok(());
                }
            }
            let source_prefix = if source == crate::cli::SignerInfoSource::GitLocal { "from local git repository " } else { "" };
            println!(
                "logged out {source_prefix}as {}",
                user_ref.metadata.name
            );
            return Ok(());
        }
    }
    Ok(())
}

pub fn get_global_login_config_items_set() -> Vec<&'static str> {
    [
        "nostr.nsec",
        "nostr.npub",
        "nostr.bunker-uri",
        "nostr.bunker-app-key",
    ]
    .iter()
    .copied()
    .filter(|item| get_git_config_item(&None, item).is_ok_and(|item| item.is_some()))
    .collect::<Vec<&str>>()
}

pub fn format_items_as_list(items: &[&str]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].to_string(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let all_but_last = items[..items.len() - 1].join(", ");
            format!("{}, and {}", all_but_last, items[items.len() - 1])
        }
    }
}
