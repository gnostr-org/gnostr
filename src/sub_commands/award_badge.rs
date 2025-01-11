use anyhow::{Context, Result};
use nostr::{FromBech32, PublicKey, ToBech32};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use super::send::send_events;
#[cfg(not(test))]
use crate::client::Client;
#[cfg(test)]
use crate::client::MockConnect;
use crate::{
    Cli,
    cli_interactor::{Interactor, InteractorPrompt, PromptInputParms},
    client::Connect,
    git::{Repo, RepoActions},
    login,
    repo_ref::{self, RepoRef, extract_pks, get_repo_config_from_yaml, save_repo_config_to_yaml},
};

use std::{process::exit, str::FromStr, time::Duration};

use clap::Args;
use nostr_sdk::prelude::*;

use crate::utils::{create_client, parse_private_key, weeble, blockheight, wobble};

use reqwest::get;
use reqwest::header::{AUTHORIZATION, HeaderMap};

#[derive(Debug, clap::Args)]
pub struct AwardBadgeSubCommandArgs {
    /// Badge definition event id
    #[arg(short, long)]
    badge_event_id: Option<String>,
    /// Awarded pubkeys
    #[arg(long, action = clap::ArgAction::Append)]
    ptag: Option<String>,
    //ptag: Option<Vec<String>>,
    //#[clap(short, long)]
    ///// name of repository
    //pub title: Option<String>,
    //#[clap(short, long)]
    ///// optional description
    //pub description: Option<String>,
    //#[clap(long)]
    ///// git server url users can clone from
    //pub clone_url: Vec<String>,
    //#[clap(long, value_parser, num_args = 1..)]
    ///// homepage
    //pub web: Vec<String>,
    #[clap(long, value_parser, num_args = 1..)]
    /// relays contributors push patches and comments to
    pub relays: Vec<String>,
    #[clap(short, long, value_parser, num_args = 1..)]
    /// npubs of other maintainers
    pub other_maintainers: Vec<String>,
    //#[clap(long)]
    ///// usually root commit but will be more recent commit for forks
    //pub earliest_unique_commit: Option<String>,
    //#[clap(short, long)]
    ///// shortname with no spaces or special characters
    //pub identifier: Option<String>,
    /// awarde_badge.rs Proof of work difficulty target¬                                         
    #[arg(long, action = clap::ArgAction::Append, default_value_t = 0u8)]
    difficulty_target: u8,
}

#[allow(clippy::too_many_lines)]
pub async fn launch(cli_args: &Cli, args: &AwardBadgeSubCommandArgs) -> Result<()> {
    if args.relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    #[cfg(not(test))]
    let mut client = Client::default();
    #[cfg(test)]
    let mut client = <MockConnect as std::default::Default>::default();
    let (keys, user_ref) = login::launch(&cli_args.nsec, &cli_args.password, Some(&client)).await?;
    client.set_keys(&keys).await;


    let my_keys = Keys::parse("0000000000000000000000000000000000000000000000000000000000000001")?;
    let mut nostr_sdk_client = nostr_sdk::Client::new(&my_keys);

    let proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));


        // Add relays
    nostr_sdk_client.add_relay("wss://relay.damus.io").await?;
    nostr_sdk_client.add_relay_with_opts(
        "wss://nos.lol", 
        RelayOptions::new().proxy(proxy).write(true)
    ).await?;
    nostr_sdk_client.add_relay_with_opts(
        "ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion",
        RelayOptions::new().proxy(proxy),
    ).await?;


    nostr_sdk_client.connect().await;

    let metadata = Metadata::new()
        .name("username")
        .display_name("My Username")
        .about("Description")
        .picture(Url::parse("https://example.com/avatar.png")?)
        .banner(Url::parse("https://example.com/banner.png")?)
        .nip05("username@example.com")
        .lud16("yuki@getalby.com")
        .custom_field("custom_field", "my value");

    // Update metadata
    nostr_sdk_client.set_metadata(&metadata).await?;

    let weeble_string = weeble().await;
    let wobble_string = wobble().await;
    let blockheight_string = blockheight().await;

     use std::borrow::Cow;
     let weeble_vec: Vec<String> = Tag::custom(TagKind::Custom(Cow::Borrowed("weeble")), [&weeble_string]).to_vec();
     let weeble_tag: Tag = Tag::parse(&weeble_vec).unwrap();

     let wobble_vec: Vec<String> = Tag::custom(TagKind::Custom(Cow::Borrowed("wobble")), [&wobble_string]).to_vec();
     let wobble_tag: Tag = Tag::parse(&wobble_vec).unwrap();

     let blockheight_vec: Vec<String> = Tag::custom(TagKind::Custom(Cow::Borrowed("blockheight")), [&blockheight_string]).to_vec();
     let blockheight_tag: Tag = Tag::parse(&blockheight_vec).unwrap();

     let gnostr: Vec<String> = Tag::custom(TagKind::Custom(Cow::Borrowed("client")), ["gnostr"]).to_vec();
     let gnostr_tag: Tag = Tag::parse(&gnostr).unwrap();
     //assert_eq!(gnostr_tag.content(), Some("gnostr"));
     let t: Tag = Tag::parse(&["gnostr", &weeble_string]).unwrap();
     //assert_eq!(t.content(), Some("gnostr"));


    // Publish a text note
    //nostr_sdk_client.publish_text_note("My first text note from rust-nostr!", [gnostr_tag, weeble_tag, blockheight_tag, wobble_tag]).await?;
    nostr_sdk_client.publish_text_note(format!("{}/{}/{}", weeble_string, blockheight_string, wobble_string), [gnostr_tag, weeble_tag, blockheight_tag, wobble_tag]).await?;

    // Create a POW text note
    //let event: Event = EventBuilder::text_note("POW text note from nostr-sdk", []).to_pow_event(&my_keys, 20)?;
    //client.send_event(event).await?; // Send to all relays
    // client.send_event_to(["wss://relay.damus.io"], event).await?; // Send to specific relay
    




    let git_repo = Repo::discover().context("cannot find a git repository")?;

    let repo_ref = if let Ok(rep_ref) = repo_ref::fetch(
        &git_repo,
        "".to_string(),
        &client,
        user_ref.relays.write(),
        false,
    )
    .await
    {
        Some(rep_ref)
    } else {
        None
    };

	let repo_config_result = get_repo_config_from_yaml(&git_repo);
    let root_commit = git_repo
        .get_root_commit()
        .context("failed to get root commit of the repository")?;

    // TODO: check for empty repo
    // TODO: check for existing maintaiers file


let maintainers: Vec<PublicKey> = {
        let mut dont_ask = !args.other_maintainers.is_empty();
        let mut maintainers_string = if !args.other_maintainers.is_empty() {
            [args.other_maintainers.clone()].concat().join(" ")
        } else if repo_ref.is_none() && repo_config_result.is_err() {
            keys.public_key().to_bech32()?
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
                vec![keys.public_key().to_bech32()?]
            };
            // add current user if not present
            if maintainers.iter().any(|m| {
                if let Ok(m_pubkey) = PublicKey::from_bech32(m) {
                    user_ref.public_key.eq(&m_pubkey)
                } else {
                    false
                }
            }) {
                maintainers.join(" ")
            } else {
                [maintainers, vec![keys.public_key().to_bech32()?]]
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
            let mut maintainers: Vec<PublicKey> = vec![];
            for m in maintainers_string.split(' ') {
                if let Ok(m_pubkey) = PublicKey::from_bech32(m) {
                    maintainers.push(m_pubkey);
                } else {
                    println!("not a valid set of npubs seperated by a space");
                    dont_ask = false;
                    continue 'outer;
                }
            }
            // add current user incase removed
            if !maintainers.iter().any(|m| user_ref.public_key.eq(m)) {
                maintainers.push(keys.public_key());
            }
            break maintainers;
        }
    };
	



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


    // if yaml file doesnt exist or needs updating
    if match &repo_config_result {
        Ok(config) => {
            !(extract_pks(config.maintainers.clone())?.eq(&maintainers)
                && config.relays.eq(&relays))
        }
        Err(_) => true,
    } {
        save_repo_config_to_yaml(&git_repo, maintainers.clone(), relays.clone())?;
        println!(
            "maintainers.yaml {}. commit and push.",
            if repo_config_result.is_err() {
                "created"
            } else {
                "updated"
            }
        );
        println!(
            "this optional file enables existing contributors to automatically fetch your repo event (instead of one from a pubkey pretending to be the maintainer)"
        );
    }





    let repo_ref = if let Ok(rep_ref) = repo_ref::fetch(
        &git_repo,
        root_commit.to_string(),
        &client,
        user_ref.relays.write(),
        false,
    )
    .await
    {
        Some(rep_ref)
    } else {
        None
    };

    let repo_config_result = get_repo_config_from_yaml(&git_repo);
    // TODO: check for other claims

    let badge_event_id = match &args.badge_event_id {
        Some(beid) => beid.clone(),
        None => Interactor::default().input(
            PromptInputParms::default()
                .with_prompt("badge_event_id")
                .with_default(if let Some(repo_ref) = &repo_ref {
                    repo_ref.name.clone()
                } else {
                    format!(
                        "{:?}/{:?}/{:?}-{}",
                        weeble().await,
                        blockheight().await,
                        wobble().await,
                        String::new()
                    )
                }),
        )?,
    };
    let ptag = match &args.ptag {
        Some(t) => t.clone(),
        None => Interactor::default().input(
            PromptInputParms::default()
                .with_prompt("ptag")
                .with_default(if let Some(repo_ref) = &repo_ref {
                    repo_ref.name.clone()
                } else {
                    format!(
                        "{:?}/{:?}/{:?}-{}",
                        weeble().await,
                        blockheight().await,
                        wobble().await,
                        String::new()
                    )
                }),
        )?,
    };

    Ok(())
}
