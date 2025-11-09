use std::error::Error;
use git2::Repository;
use libp2p::{gossipsub, kad};
use tracing::debug;

use super::behaviour::Behaviour;
use super::git_integration::{get_commit_diff_as_bytes, get_commit_id_of_tag};
use crate::p2p::args::Args;

pub async fn run_git_publisher(args: &Args, swarm: &mut libp2p::Swarm<Behaviour>) -> Result<(), Box<dyn Error>> {
    let path = args.flag_git_dir.as_ref().map_or(".", |s| &s[..]);
    let repo = Repository::discover(path)?;
    if let Ok(tag_names) = repo.tag_names(None) {
        for tag_name_opt in tag_names.iter() {
            if let Some(tag_name) = tag_name_opt {
                if let Ok(commit_id) = get_commit_id_of_tag(&repo, tag_name) {
                    let key = kad::RecordKey::new(&tag_name);
                    let record = kad::Record {
                        key: key.clone(),
                        value: commit_id.into_bytes(),
                        publisher: Some(swarm.local_peer_id().clone()),
                        expires: None,
                    };
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .put_record(record, kad::Quorum::Majority)?;
                    swarm
                        .behaviour_mut()
                        .kademlia
                        .start_providing(key.clone())?;

                    let topic = gossipsub::IdentTopic::new(tag_name);
                    debug!("subscribe topic={}", topic.clone());
                    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
                }
            }
        }
    }
    let mut revwalk = repo.revwalk()?;
    let base = if args.flag_reverse {
        git2::Sort::REVERSE
    } else {
        git2::Sort::NONE
    };
    let sorting = base
        | if args.flag_topo_order {
            git2::Sort::TOPOLOGICAL
        } else if args.flag_date_order {
            git2::Sort::TIME
        } else {
            git2::Sort::NONE
        };
    revwalk.set_sorting(sorting)?;

    if args.arg_commit.is_empty() {
        revwalk.push_head()?;
    } else {
        for commit_spec in &args.arg_commit {
            let obj = repo.revparse_single(commit_spec)?;
            revwalk.push(obj.id())?;
        }
    }

    let revwalk_iterator = revwalk
        .filter_map(Result::ok)
        .filter_map(|id| repo.find_commit(id).ok());

    for commit in revwalk_iterator.take(args.flag_max_count.unwrap_or(usize::MAX)) {
        let commit_id_str = commit.id().to_string();
        let msg_key = kad::RecordKey::new(&commit_id_str);
        let msg_record = kad::Record {
            key: msg_key.clone(),
            value: commit.message_bytes().to_vec(),
            publisher: Some(*swarm.local_peer_id()),
            expires: None,
        };
        swarm
            .behaviour_mut()
            .kademlia
            .put_record(msg_record, kad::Quorum::Majority)?;
        swarm.behaviour_mut().kademlia.start_providing(msg_key)?;
        if let Ok(diff_bytes) = get_commit_diff_as_bytes(&repo, &commit) {
            let diff_key_str = format!("{}/diff", commit_id_str);
            let diff_key = kad::RecordKey::new(&diff_key_str);
            let diff_record = kad::Record {
                key: diff_key.clone(),
                value: diff_bytes,
                publisher: Some(*swarm.local_peer_id()),
                expires: None,
            };
            swarm
                .behaviour_mut()
                .kademlia
                .put_record(diff_record, kad::Quorum::One)?;
            swarm.behaviour_mut().kademlia.start_providing(diff_key)?;
        }
    }

    Ok(())
}
