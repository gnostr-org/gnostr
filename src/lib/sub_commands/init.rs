use std::collections::HashMap;

use anyhow::{Context, Result};
use nostr_sdk_0_34_0::{PublicKey as NostrPublicKey, ToBech32};

use crate::types::{EventKind, NAddr, PublicKey};
use crate::{
    //cli::Cli,
    cli_interactor::{Interactor, InteractorPrompt, PromptInputParms},
    client::{Client, Connect, fetching_with_report, get_repo_ref_from_cache, send_events},
    git::{Repo, RepoActions, nostr_url::convert_clone_url_to_https},
    login,
    repo_ref::{
        RepoRef, extract_pks, get_repo_config_from_yaml, save_repo_config_to_yaml,
        try_and_get_repo_coordinates,
    },
};

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    #[arg(short, long)]
    /// name of repository
    pub title: Option<String>,
    #[arg(short, long)]
    /// optional description
    pub description: Option<String>,
    #[arg(long)]
    /// git server url users can clone from
    pub clone_url: Vec<String>,
    #[arg(short, long, num_args = 1..)]
    /// homepage
    pub web: Vec<String>,
    #[arg(short, long, num_args = 1..)]
    pub relays: Vec<String>,
    #[arg(short, long, num_args = 1..)]
    /// npubs of other maintainers
    pub other_maintainers: Vec<String>,
    #[arg(long)]
    /// usually root commit but will be more recent commit for forks
    pub earliest_unique_commit: Option<String>,
    #[arg(short, long)]
    /// shortname with no spaces or special characters
    pub identifier: Option<String>,
    #[arg(long)]
    /// shortname with no spaces or special characters
    pub disable_cli_spinners: bool,
    #[arg(long)]
    /// shortname with no spaces or special characters
    pub password: Option<String>,
    #[arg(long)]
    /// shortname with no spaces or special characters
    pub nsec: Option<String>,
    #[arg(long)]
    /// shortname with no spaces or special characters
    pub bunker_app_key: Option<String>,
    #[arg(long)]
    /// shortname with no spaces or special characters
    pub bunker_uri: Option<String>,
}

#[allow(clippy::too_many_lines)]
pub async fn launch(
    //cli_args: &Cli,
    args: &InitArgs,
) -> Result<()> {
    let git_repo = Repo::discover().context("cannot find a git repository")?;
    let git_repo_path = git_repo.get_path()?;

    let root_commit = git_repo
        .get_root_commit()
        .context("failed to get root commit of the repository")?;

    // TODO: check for empty repo
    // TODO: check for existing maintaiers file

    #[cfg(test)]
    let client: &mut crate::client::MockConnect = &mut Default::default();
    #[cfg(not(test))]
    let mut client = Client::default();

    let repo_coordinates = (try_and_get_repo_coordinates(&git_repo, &client, false).await).ok();

    let repo_ref = if let Some(repo_coordinates) = repo_coordinates {
        fetching_with_report(git_repo_path, &client, &repo_coordinates, true).await?;
        Some(get_repo_ref_from_cache(git_repo_path, &repo_coordinates).await?)
    } else {
        None
    };

    let (signer, user_ref) = login::launch(
        &git_repo,
        &args.bunker_uri,
        &args.bunker_app_key,
        &args.nsec,
        &args.password,
        Some(&client),
        false,
        false,
    )
    .await?;

    let repo_config_result = get_repo_config_from_yaml(&git_repo);
    // TODO: check for other claims

    let name = match &args.title {
        Some(t) => t.clone(),
        None => Interactor::default().input(
            PromptInputParms::default()
                .with_prompt("name")
                .with_default(if let Some(repo_ref) = &repo_ref {
                    repo_ref.name.clone()
                } else {
                    String::new()
                }),
        )?,
    };

    let identifier = match &args.identifier {
        Some(t) => t.clone(),
        None => Interactor::default().input(
            PromptInputParms::default()
                .with_prompt("identifier")
                .with_default(if let Some(repo_ref) = &repo_ref {
                    repo_ref.identifier.clone()
                } else {
                    let fallback = name
                        .clone()
                        .replace(' ', "-")
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c.eq(&'/') {
                                c
                            } else {
                                '-'
                            }
                        })
                        .collect();
                    if let Ok(config) = &repo_config_result {
                        if let Some(identifier) = &config.identifier {
                            identifier.to_string()
                        } else {
                            fallback
                        }
                    } else {
                        fallback
                    }
                }),
        )?,
    };

    let description = match &args.description {
        Some(t) => t.clone(),
        None => Interactor::default().input(
            PromptInputParms::default()
                .with_prompt("description")
                .with_default(if let Some(repo_ref) = &repo_ref {
                    repo_ref.description.clone()
                } else {
                    String::new()
                }),
        )?,
    };

    let git_server = if args.clone_url.is_empty() {
        Interactor::default()
            .input(
                PromptInputParms::default()
                    .with_prompt("clone url (for fetch)")
                    .with_default(if let Some(repo_ref) = &repo_ref {
                        repo_ref.git_server.clone().join(" ")
                    } else if let Ok(url) = git_repo.get_origin_url() {
                        if let Ok(fetch_url) = convert_clone_url_to_https(&url) {
                            fetch_url
                        } else {
                            // local repo or custom protocol
                            url
                        }
                    } else {
                        String::new()
                    }),
            )?
            .split(' ')
            .map(std::string::ToString::to_string)
            .collect()
    } else {
        args.clone_url.clone()
    };

    let web: Vec<String> = if args.web.is_empty() {
        Interactor::default()
            .input(
                PromptInputParms::default()
                    .with_prompt("web")
                    .optional()
                    .with_default(if let Some(repo_ref) = &repo_ref {
                        repo_ref.web.clone().join(" ")
                    } else {
                        format!("https://gitworkshop.dev/repo/{}", &identifier)
                    }),
            )?
            .split(' ')
            .map(std::string::ToString::to_string)
            .collect()
    } else {
        args.web.clone()
    };

    let maintainers: Vec<PublicKey> = {
        let mut dont_ask = !args.other_maintainers.is_empty();
        let mut maintainers_string = if !args.other_maintainers.is_empty() {
            [args.other_maintainers.clone()].concat().join(" ")
        } else if repo_ref.is_none() && repo_config_result.is_err() {
            signer.public_key().await?.to_bech32()?
        } else {
            let maintainers = if let Ok(config) = &repo_config_result {
                config.maintainers.clone()
            } else if let Some(repo_ref) = &repo_ref {
                repo_ref
                    .maintainers
                    .clone()
                    .iter()
                    .map(|k| k.to_bech32().unwrap())
                    .collect()
            } else {
                //unreachable
                vec![signer.public_key().await?.to_bech32()?]
            };
            // add current user if not present
            if maintainers
                .iter()
                .any(|m| user_ref.public_key.to_bech32().unwrap().eq(m))
            {
                maintainers.join(" ")
            } else {
                [maintainers, vec![signer.public_key().await?.to_bech32()?]]
                    .concat()
                    .join(" ")
            }
        };
        'outer: loop {
            if !dont_ask {
                println!("{}", &maintainers_string);
                maintainers_string = Interactor::default().input(
                    PromptInputParms::default()
                        .with_prompt("maintainers")
                        .with_default(maintainers_string),
                )?;
            }
            let mut maintainers: Vec<NostrPublicKey> = vec![];
            for m in maintainers_string.split(' ') {
                if let Ok(m_pubkey) = NostrPublicKey::parse(m) {
                    maintainers.push(m_pubkey);
                } else {
                    println!("not a valid set of npubs seperated by a space");
                    dont_ask = false;
                    continue 'outer;
                }
            }
            // add current user incase removed
            if !maintainers.iter().any(|m| user_ref.public_key.eq(m)) {
                maintainers.push(signer.public_key().await?);
            }
            // Convert nostr SDK PublicKeys to local PublicKeys for the outer scope
            break maintainers
                .into_iter()
                .map(|npk| {
                    PublicKey::from_bytes(&npk.to_bytes(), false)
                        .map_err(|e| anyhow::anyhow!("{}", e))
                })
                .collect::<Result<Vec<PublicKey>>>()?;
        }
    };

    // TODO: check if relays are free to post to so contributors can
    // submit patches TODO: recommend some reliable free ones
    let relays: Vec<String> = if args.relays.is_empty() {
        Interactor::default()
            .input(
                PromptInputParms::default()
                    .with_prompt("relays")
                    .with_default(if let Ok(config) = &repo_config_result {
                        config.relays.clone().join(" ")
                    } else if let Some(repo_ref) = &repo_ref {
                        repo_ref.relays.clone().join(" ")
                    } else {
                        user_ref.relays.write().join(" ")
                    }),
            )?
            .split(' ')
            .map(std::string::ToString::to_string)
            .collect()
    } else {
        args.relays.clone()
    };

    #[allow(clippy::single_match_else)]
    let earliest_unique_commit = match &args.earliest_unique_commit {
        Some(t) => t.clone(),
        None => {
            let mut earliest_unique_commit = if let Some(repo_ref) = &repo_ref {
                repo_ref.root_commit.clone()
            } else {
                root_commit.to_string()
            };
            loop {
                earliest_unique_commit = Interactor::default().input(
                    PromptInputParms::default()
                        .with_prompt("earliest unique commit")
                        .with_default(earliest_unique_commit.clone()),
                )?;
                if let Ok(exists) = git_repo.does_commit_exist(&earliest_unique_commit) {
                    if exists {
                        break earliest_unique_commit;
                    }
                    println!("commit does not exist on current repository");
                } else {
                    println!("commit id not formatted correctly");
                }
                if earliest_unique_commit.len().ne(&40) {
                    println!("commit id must be 40 characters long");
                }
            }
        }
    };

    println!("publishing repostory reference...");

    let repo_ref = RepoRef {
        identifier: identifier.clone(),
        name,
        description,
        root_commit: earliest_unique_commit,
        git_server,
        web,
        relays: relays.clone(),
        maintainers: maintainers
            .iter()
            .map(|local_pk| {
                NostrPublicKey::from_slice(&local_pk.as_bytes())
                    .map_err(|e| anyhow::anyhow!("Failed to convert local PublicKey: {}", e))
            })
            .collect::<Result<Vec<NostrPublicKey>>>()?,
        events: HashMap::new(),
    };
    let repo_event = repo_ref.to_event(&signer).await?;

    client.set_signer(signer).await;

    send_events(
        &client,
        git_repo_path,
        vec![repo_event],
        user_ref.relays.write(),
        relays.clone(),
        !args.disable_cli_spinners,
        false,
    )
    .await?;

    git_repo.save_git_config_item(
        "nostr.repo",
        &NAddr {
            kind: EventKind::GitRepoAnnouncement,
            author: crate::types::PublicKey::from_bytes(&user_ref.public_key.to_bytes(), false)?,
            d: identifier.clone(),
            relays: vec![],
        }
        .as_bech32_string(),
        false,
    )?;

    // if yaml file doesnt exist or needs updating
    if match &repo_config_result {
        Ok(config) => {
            !<std::option::Option<std::string::String> as Clone>::clone(&config.identifier)
                .unwrap_or_default()
                .eq(&identifier)
                || !extract_pks(config.maintainers.clone())?
                    .iter()
                    .zip(maintainers.iter())
                    .all(|(a, b)| a.to_string().eq(&b.to_string()))
                || !config.relays.eq(&relays)
        }
        Err(_) => true,
    } {
        let maintainers_for_yaml: Result<Vec<nostr_0_34_1::PublicKey>> = maintainers
            .iter()
            .map(|npk| nostr_0_34_1::PublicKey::from_slice(&npk.to_bytes()))
            .collect::<Result<Vec<_>>>()
            .map_err(|e| anyhow::anyhow!("Failed to convert to nostr PublicKey: {}", e));
        save_repo_config_to_yaml(
            &git_repo,
            identifier.clone(),
            maintainers_for_yaml?,
            relays.clone(),
        )?;
        println!(
            "maintainers.yaml {}. commit and push.",
            if repo_config_result.is_err() {
                "created"
            } else {
                "updated"
            }
        );
        println!(
            "this optional file helps in identifying who the maintainers are over time through the commit history"
        );
    }
    Ok(())
}
