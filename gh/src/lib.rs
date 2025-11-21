use clap::arg;
mod cmd_macro;
mod async_io;
use async_io::*;
use terminal_size::terminal_size;
use configparser::ini::Ini;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashSet,
    ops::Deref,
};
use thiserror::Error;



const gh_rs_github_username: &str = "gh0st-work";
const gh_rs_github_repo_name: &str = "gh.rs";
fn get_gh_rs_github_repo_path() -> String { 
    format!("{gh_rs_github_username}/{gh_rs_github_repo_name}")
}
fn get_gh_rs_github_url() -> String {
    format!("https://github.com/{gh_rs_github_username}/{gh_rs_github_repo_name}")
}
const ssh_key_name: &str = "gh_rs_ed25519.pem";

fn re_token() -> regex::Regex { 
    re(r"^ghp_[a-zA-Z0-9]+$")
}
fn re_username() -> regex::Regex {
    re(r"^[a-zA-Z0-9-_]+$")
}
fn re_repo_name() -> regex::Regex {
    re(r"^[a-zA-Z0-9-_\.]+$")
}

#[derive(Error, Debug)]
enum GhRsError {
    #[error("CLI only mode is enabled, but {0} is not specified")]
    CliOnly(String),

    #[error("GitHub API (octocrab) error: {0}")]
    OctocrabError(#[from] octocrab::Error),
    
    #[error("SSH key (ssh_key) error: {0}")]
    SshKeyError(#[from] ssh_key::Error),
    
    #[error("Git management (git2) error: {0}")]
    Git2Error(#[from] git2::Error),
    
    #[error("Command processing error: {0}")]
    CmdError(String),
}
type GhRsResult<T> = Result<T, GhRsError>;

macro_rules! return_cmd_err {
    ($($arg:tt)*) => (return Err(GhRsError::CmdError(format!($($arg)*))))
}
macro_rules! err_from {
    ($($arg:tt)*) => (Err(GhRsError::from($($arg)*)))
}

fn get_hr() -> String { 
    "─".repeat(
        terminal_size()
        .unwrap_or((terminal_size::Width(10), terminal_size::Height(10)))
        .0.0 as usize
    )
}


async fn get_github_config_from_machine() -> Option<Ini> {
    let mut config = Ini::new_cs();
    match config.load_async(path!("~/.gitconfig")).await {
        Ok(r) => Some(config),
        Err(e) => None
    }
}

async fn get_github_signature_from_machine<'s>() -> Option<git2::Signature<'s>> {
    if let Some(config) = get_github_config_from_machine().await {
        if let Some(name) = config.get("user", "name") {
            if let Some(email) = config.get("user", "email") {
                if let Ok(sig) = git2::Signature::now(&name, &email) {
                    return Some(sig);
                };
            };
        };
    };
    None
}


async fn get_github_token_from_machine(
    tried_env: &mut bool, 
    tried_config: &mut bool, 
    tried_credentials: &mut bool
) -> Option<String> {
    if !*tried_env {
        if let Ok(token_possible) = std::env::var("GITHUB_TOKEN") {
            if token_possible.len() == 40 && !re_token().is_match(&token_possible) {
                *tried_env = true;
                return Some(token_possible);
            }
        };
    }
    if !*tried_config {
        if let Some(config) = get_github_config_from_machine().await {
            if let Some(passoword) = config.get("user", "passoword") {
                let token_possible = passoword.trim_end_matches(':').to_string();
                if token_possible.len() == 40 && re_token().is_match(&token_possible) {
                    *tried_config = true;
                    return Some(token_possible);
                }
            };
        };
    }
    if !*tried_credentials {
        if let Some(token_possible) = find_regex_in_file_lines(&path!("~/.git-credentials"), re(r"(ghp_[a-zA-Z0-9]+):?"), 0).await {
            if token_possible.len() == 40 {
                *tried_credentials = true;
                return Some(token_possible);
            }
        };
    }
    None
}

async fn get_ssh_key_from_machine() -> Option<ssh_key::PrivateKey> {
    let path = path!("~/.ssh/{ssh_key_name}");
    if !path.is_file().await {
        return None;
    }
    let content: Vec<u8> = match fs::read(path).await {
        Ok(c) => c,
        Err(e) => return None,
    };
    let key_private = match ssh_key::PrivateKey::from_openssh(content) {
        Ok(k) => k,
        Err(e) => return None,
    };
    Some(key_private)
}

async fn create_ssh_key_on_machine() -> GhRsResult<ssh_key::PrivateKey> {
    let private_key = ssh_key::PrivateKey::random(&mut rand_core::OsRng, ssh_key::Algorithm::Ed25519)?;
    match fs::write(
        path!("~/.ssh/{ssh_key_name}"), 
        private_key.to_openssh(ssh_key::LineEnding::default())?
    ).await {
        Ok(r) => Ok(private_key),
        Err(error) => err_from!(ssh_key::Error::Io(error.kind()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OctocrabModelSshKey {
    pub key: String,
    pub id: i64,
    pub url: Option<url::Url>,
    pub title: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub verified: bool,
    pub read_only: bool,
}

async fn octocrab_ssh_keys_create(
    octocrab_client: &octocrab::Octocrab,
    private_key: &ssh_key::PrivateKey,
) -> GhRsResult<OctocrabModelSshKey> {
    let key: String = private_key.public_key().to_openssh()?;
    let octo_key: OctocrabModelSshKey = octocrab_client.post(
        "/user/keys",
        Some(&serde_json::json!({
            "title": "gh.rs",
            "key": key,
        })),
    ).await?;
    Ok(octo_key)
}


async fn get_or_create_ssh_key(
    octocrab_client: &octocrab::Octocrab,
) -> GhRsResult<ssh_key::PrivateKey> {
    if let Some(private_key_possible) = get_ssh_key_from_machine().await {
        match octocrab_ssh_keys_create(octocrab_client, &private_key_possible).await {
            Ok(r) => return Ok(private_key_possible), 
            Err(error) => {
                let error_string = error.to_string();
                if re(r#""message"\s*:\s*"key is already in use""#).is_match(&error_string) {
                    return Ok(private_key_possible);
                }
                aprintln!("Failed send ssh key to github, error: {error_string}");
            }
        }; 
    }

    match create_ssh_key_on_machine().await {
        Ok(private_key_possible) => match octocrab_ssh_keys_create(octocrab_client, &private_key_possible).await {
            Ok(k) => Ok(private_key_possible),
            Err(e) => return_cmd_err!("Failed to save GitHub SSH key, that was just created, error: {e}"),
        },
            Err(e) => return_cmd_err!("Failed to create SSH key, error: {e}"),
    }
}

async fn octocrab_repos_create(
    octocrab_client: &octocrab::Octocrab,
    is_public: &bool,
    name: &str,
    description: &str,
) -> GhRsResult<octocrab::models::Repository> {
    let gh_repo: octocrab::models::Repository = octocrab_client.post(
        "/user/repos",
        Some(&serde_json::json!({
            "private": !*is_public,
            "name": name,
            "description": description,
            "auto_init": false,
        })),
    ).await?;
    Ok(gh_repo)
}

async fn git2_add_and_commit<'repo, 'sig>(
    repo: &'repo git2::Repository,
    signature: &git2::Signature<'sig>,
    file_paths: &[&path::PathBuf],
    message: &str,
) -> GhRsResult<git2::Commit<'repo>> {
    let mut repo_index = repo.index()?;
    let repo_path = repo.path();
    for path in file_paths {
        let path_abs = match path.canonicalize().await {
            Ok(p) => p,
            Err(e) => return_cmd_err!(
                "Cannot canonicalize path {path_display}, error: {e}",
                path_display = path_display(path)
            )
        };
        let path_abs_sync = path_to_sync(&path_abs);
        let path_rel = match path_rel_sync(repo_path, path_abs_sync.as_path()) {
            Ok(p) => p,
            Err(e) => return_cmd_err!(
                "Cannot find file to repo relative path (repo: {repo_path_display}, file: {file_path_display}), error: {e}",
                repo_path_display = path_display_sync(&(repo_path.to_path_buf())),
                file_path_display = path_display_sync(&path_abs_sync)
            )

        };
        let path_rel = path_rel.strip_prefix("../").unwrap_or(&path_rel);
        repo_index.add_path(path_rel)?;
    }
    let tree_id = repo_index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let mut parents = vec![];
    if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            parents.push(commit);
        }
    }
    let pref: Vec<&git2::Commit> = parents.iter().collect();
    let commit_id = repo.commit(
        Some("HEAD"), 
        signature, 
        signature, 
        message,
        &tree,
        &pref,
    )?;
    let commit = repo.find_commit(commit_id)?;
    Ok(commit)
}

async fn git2_add_all_and_commit<'repo, 'sig>(
    repo: &'repo git2::Repository,
    signature: &git2::Signature<'sig>,
    pathspecs: impl IntoIterator<Item = impl git2::IntoCString>,
    message: &str,
) -> GhRsResult<git2::Commit<'repo>> {
    let mut repo_index = repo.index()?;
    repo_index.add_all(pathspecs, git2::IndexAddOption::CHECK_PATHSPEC, None)?;
    let tree_id = repo_index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let mut parents = vec![];
    if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            parents.push(commit);
        }
    }
    let pref: Vec<&git2::Commit> = parents.iter().collect();
    let commit_id = repo.commit(
        Some("HEAD"), 
        signature, 
        signature, 
        message,
        &tree,
        &pref,
    )?;
    let commit = repo.find_commit(commit_id)?;
    Ok(commit)
}

fn git2_set_branch_upstream(
    repo: &git2::Repository,
    branch_name: &str,
    remote_name: &str,
) -> GhRsResult<()> {
    let mut branch = repo.find_branch(
        branch_name, 
        git2::BranchType::Local
    )?;
    branch.set_upstream(Some(format!("{remote_name}/{branch_name}").as_str()))?;
    Ok(())
}

fn git2_credentials(
    url: &str,
    username_from_url: Option<&str>,
    allowed: git2::CredentialType,

    username: &str,
    password: &str,
    ssh_private_key: &ssh_key::PrivateKey
) -> Result<git2::Cred, git2::Error> {
    if allowed.contains(git2::CredentialType::SSH_MEMORY) {
        git2::Cred::ssh_key_from_memory(            
            username_from_url.expect("must be some"),
            Some(ssh_private_key.public_key().to_openssh().expect("must be able to convert to openssh format").deref()), 
            ssh_private_key.to_openssh(ssh_key::LineEnding::default()).expect("must be able to convert to openssh format").deref(), 
            None
        )
    } else if allowed.contains(git2::CredentialType::USERNAME) {
        git2::Cred::username(username)
    } else if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
        git2::Cred::userpass_plaintext(username, password)
    } else {
        Err(git2::Error::from_str(format!("no callbacks for type: {allowed:?}").as_str()))
    }
}


fn git2_remote_delete(
    repo: &git2::Repository,
    remote_name: &str,
) -> Result<(), git2::Error> {
    // fixes error when having multivar refspecs on remote in config
    
    fn remove_branch_config_related_entries(
        repo: &git2::Repository,
        remote_name: &str,
    ) -> Result<(), git2::Error> {
        let config = repo.config()?;
        // Some(r"branch\..+\.remote")
        let mut entries = config.entries(None)?;
        let mut entries_to_remove = HashSet::new();
        while let Some(entry) = entries.next() {
            let entry = entry?;
            let entry_value = match entry.value() {
                None => continue,
                Some(value) => {
                    if value != remote_name { continue; }
                    value
                }
            };
            let entry_name = match entry.name() {
                None => continue,
                Some(name) => name,
            };
            let branch = regex_groups(&re(r"branch\.(.+)\.remote"), entry_name)[1].as_str();
            entries_to_remove.insert(format!("branch.{branch}.merge"));
            entries_to_remove.insert(format!("branch.{branch}.remote"));
        }
        let mut config = repo.config()?;
        for entry in entries_to_remove {
            config.remove_multivar(entry.as_str(), r".*")?;
        }
        Ok(())
    }


    fn remove_refs(
        repo: &git2::Repository,
        spec: &git2::Refspec,
    ) -> Result<(), git2::Error> {
        let mut references_to_remove = HashSet::new();
        for reference in repo.references()? {
            let reference = reference?;
            let reference_name = match reference.name() {
                Some(n) => n,
                None => continue,
            };
            if !spec.dst_matches(reference_name) { continue; }
            references_to_remove.insert(reference_name.to_owned());
        }
        for reference_name in references_to_remove {
            repo.find_reference(reference_name.as_str())?.delete()?;
        }
        Ok(())
    }

    fn remove_remote_tracking(
        repo: &git2::Repository,
        remote_name: &str,
    ) -> Result<(), git2::Error> {
        let remote = repo.find_remote(remote_name)?;
        for refspec in remote.refspecs() {
            remove_refs(repo, &refspec)?;
        }
        Ok(())
    }
    
    fn remove_config_remote_section(
        repo: &git2::Repository,
        remote_name: &str,
    ) -> Result<(), git2::Error> {
        // this fn fails in libgit2, bcuz of git_config_delete_entry() used, not remove_multivar()
        let config = repo.config()?;
        let mut entries = config.entries(Some(format!(r"remote.{remote_name}\..*").as_str()))?;
        let mut entries_to_remove = HashSet::new();
        while let Some(entry) = entries.next() {
            let entry = entry?;
            let entry_name = entry.name().expect("must be some").to_owned();
            entries_to_remove.insert(entry_name);
        }
        let mut config = repo.config()?;
        for entry in entries_to_remove {
            config.remove_multivar(entry.as_str(), r".*")?; // fix is here
        }
        Ok(())
    }

    remove_branch_config_related_entries(repo, remote_name)?;
    remove_remote_tracking(repo, remote_name)?;
    
    // this fn fails in libgit2, bcuz of git_config_delete_entry() used, not remove_multivar()
    remove_config_remote_section(repo, remote_name)?;
    
    Ok(())
}

fn git2_remote_recreate<'repo>(
    repo: &'repo git2::Repository,
    name: &str,
    url: &str,
) -> Result<git2::Remote<'repo>, git2::Error> {
    if let Ok(r) = repo.find_remote(name) {
        git2_remote_delete(repo, name)?;
    }
    repo.remote(name, url)
}

fn git2_generate_refspecs_from_globbed(
    repo: &git2::Repository,
    refspecs_globbed: HashSet<String>,
) -> GhRsResult<HashSet<String>> {
    let refspecs = HashSet::new();
    for rg in refspecs_globbed {
        let mut rg = rg;
        let is_plus = rg.starts_with("+");
        if is_plus {
            rg = rg.strip_prefix("+").unwrap_or(&rg).to_owned();
        }
        if !rg.starts_with("refs") {
            return_cmd_err!("Refspec must start with \"refs\", here: {rg}");
        }
        let got_git_path = repo.path();
        
    }
    Ok(refspecs)
}

fn git2_default_branch_name(
    repo: &git2::Repository,
) -> GhRsResult<String> {
    let head = repo.head()?;
    let head_name = match head.name() {
        Some(n) => n, 
        None => return_cmd_err!("Failed to encode head name as utf8")
    };
    let branch_name = head_name.strip_prefix("refs/heads/").unwrap_or(head_name)
        .to_string();
    Ok(branch_name)
}

fn git2_push(
    repo: &git2::Repository,
    remote_name: &str,
    remote_url: &str,
    mirror: &bool,

    username: &str,
    password: &str,
    ssh_private_key: &ssh_key::PrivateKey,
) -> GhRsResult<()> {
    let mut remote = git2_remote_recreate(repo, remote_name, remote_url)?;
    let mut opts = git2::PushOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed| 
        git2_credentials(
            url, username_from_url, allowed,
            username, password, ssh_private_key
        )
     );
    opts.remote_callbacks(callbacks);
    let mut refspecs: Vec<String> = vec![];
    refspecs.push(format!("refs/heads/{}", git2_default_branch_name(&repo)?));
    if *mirror {
        repo.remote_add_push(remote_name, format!("refs/remotes/{remote_name}/*:refs/heads/*").as_str())?;
        repo.remote_add_push(remote_name, "refs/tags/*:refs/tags/*")?;
        let branches = repo.branches(Some(git2::BranchType::Remote))?;
        for branch in branches.flatten() {
            let name = branch.0.name();
            if let Ok(Some(name)) = name {
                let branch = &name[name.find('/').unwrap()..];
                refspecs.push(format!("refs/remotes/{name}:refs/heads{branch}"));
            }
        }
        let tags = repo.tag_names(None)?;
        for tag in tags.iter().flatten() {
            refspecs.push(format!("refs/tags/{tag}:refs/tags/{tag}"));
        }
    }
    remote.push(&refspecs, Some(&mut opts))?;
    Ok(())
}


fn git2_clone(
    clone_url: &str,
    clone_to_path: &path::PathBuf,
    clone_bare: &bool,

    username: &str,
    password: &str,
    ssh_private_key: &ssh_key::PrivateKey,
) -> GhRsResult<git2::Repository> {
    let mut fetch_options = git2::FetchOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed| 
        git2_credentials(
            url, username_from_url, allowed,
            username, password, ssh_private_key
        )
    );
    fetch_options.remote_callbacks(callbacks);
    let checkout_builder = git2::build::CheckoutBuilder::new();
    let repo = git2::build::RepoBuilder::new()
        .bare(*clone_bare)
        .remote_create(git2_remote_recreate)
        .fetch_options(fetch_options)
        .with_checkout(checkout_builder)
        .clone(clone_url, path_to_sync(clone_to_path).as_path())?;

    Ok(repo)
}

async fn git2_fetch_until_commit<'repo>(
    repo: &'repo git2::Repository,
    remote_name: &str,
    remote_url: &str,
    commit_id: &git2::Oid,
    retry_timeout_duration: &tokio::time::Duration,
    retries_limit: usize,

    username: &str,
    password: &str,
    ssh_private_key: &ssh_key::PrivateKey,
) -> GhRsResult<()> {
    if repo.find_commit(commit_id.to_owned()).is_ok() { return Ok(()); }

    let mut remote = git2_remote_recreate(repo, remote_name, remote_url)?;

    let mut retries = 0;
    while retries <= retries_limit {
        let started_at = tokio::time::Instant::now();
        remote.connect(git2::Direction::Fetch)?;
        let mut fetch_options = git2::FetchOptions::new();
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |url, username_from_url, allowed| 
            git2_credentials(
                url, username_from_url, allowed,
                username, password, ssh_private_key
            )
        );
        fetch_options.remote_callbacks(callbacks);
        remote.download(&[] as &[&str], Some(&mut fetch_options))?;
        remote.disconnect()?;
        remote.update_tips(None, true, git2::AutotagOption::Unspecified, None)?;

        if repo.find_commit(commit_id.to_owned()).is_ok() { return Ok(()); }
        
        tokio::time::sleep_until(started_at + retry_timeout_duration.to_owned()).await;
        retries += 1;
    }
    return_cmd_err!("Retries limit reached")
}
    

async fn get_auth(
    token_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<(octocrab::Octocrab, String, octocrab::models::Author)> {
    let cli_only_error = GhRsError::CliOnly("access token".to_string());

    async fn prompt_token() -> String {
        loop {
            let token_answer = prompt("GitHub access token", None).await.trim().to_string();
            if token_answer.len() != 40 || !re_token().is_match(&token_answer) {
                aprintln!("Invalid token format");
                continue;
            }
            return token_answer;
        }
    }
    
    let mut tried_env = false;
    let mut tried_config = false;
    let mut tried_credentials = false;
    let mut tried_opt = false;
    loop {
        let token_raw = match token_raw_opt {
            None => match get_github_token_from_machine(
                &mut tried_env, 
                &mut tried_config, 
                &mut tried_credentials
            ).await {
                Some(token_found) => token_found,
                None => match *cli_only {
                    true => return Err(cli_only_error),
                    false => prompt_token().await,
                },
            },
            Some(token_raw_possible) => {
                if !tried_opt && token_raw_possible.len() == 40 && re_token().is_match(token_raw_possible) {
                    tried_opt = true;
                    token_raw_possible.to_string()
                } else {
                    match get_github_token_from_machine(
                        &mut tried_env, 
                        &mut tried_config, 
                        &mut tried_credentials
                    ).await {
                        Some(token_found) => token_found,
                        None => match *cli_only {
                            true => return Err(cli_only_error),
                            false => prompt_token().await,
                        },
                    }
                }
            }
        };

        let octocrab_client = octocrab::OctocrabBuilder::default()
            .personal_token(token_raw.clone())
            .build()?;
        match octocrab_client.current().user().await {
            Ok(user) => {
                return Ok((octocrab_client, token_raw, user));
            }
            Err(e) => {
                aprintln!("Invalid answer received from API, possible access token is wrong, error: {e}");
                if *cli_only {
                    return Err(cli_only_error);
                }
                continue;
            }
        }
    }
}

async fn get_repo_name(
    octocrab_client: &octocrab::Octocrab,
    username: &str,
    repo_name_raw_opt: &Option<&str>,
    repo_name_prompt_default: Option<&str>,
    cli_only: &bool,
) -> GhRsResult<String> {
    let cli_only_error = GhRsError::CliOnly("new repo name".to_string());

    async fn prompt_repo_name(prompt_default: Option<&str>) -> String {
        loop {
            let repo_name_answer = prompt("GitHub new repo name", prompt_default).await.trim().to_string();
            if repo_name_answer.is_empty() {
                aprintln!("Invalid name format, empty string");
                continue;
            } else if !re_repo_name().is_match(&repo_name_answer) {
                aprintln!("Invalid name format");
                continue;
            } else if path!("./{repo_name_answer}").exists().await {
                aprintln!("Directory with name {repo_name_answer} already exists");
                continue;
            }
            return repo_name_answer;
        }
    }

    loop {
        let repo_name_raw = match repo_name_raw_opt {
            Some(repo_name_raw_possible) => match re_repo_name().is_match(repo_name_raw_possible) {
                true => repo_name_raw_possible.to_string(),
                false => {
                    aprintln!("Invalid name format");
                    match *cli_only {
                        true => return Err(cli_only_error),
                        false => prompt_repo_name(repo_name_prompt_default).await,
                    }
                },
            }
            None => match *cli_only {
                true => return Err(cli_only_error),
                false => prompt_repo_name(repo_name_prompt_default).await,
            },
        };
        match octocrab_client.repos(username, repo_name_raw.clone()).get().await {
            Err(e) => {
                return Ok(repo_name_raw);
            }
            Ok(repo) => {
                aprintln!("Repo with name {repo_name_raw} already exists");
                if *cli_only {
                    return Err(cli_only_error);
                }
                continue;
            }
        }
    }
}

async fn get_repo_description(
    repo_description_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<String> {
    let cli_only_error = GhRsError::CliOnly("new repo description".to_string());

    async fn prompt_repo_description() -> String {
        loop {
            let mut repo_description_answer = prompt("GitHub new repo description", None).await;
            repo_description_answer = repo_description_answer.trim().to_string();
            if repo_description_answer.len() <= 255 {
                return repo_description_answer;
            }
            aprintln!("Too long description, maximum lenght 255, now: {len}", len = repo_description_answer.len());
            continue;
        }
    }
    
    let repo_description = match repo_description_raw_opt {
        Some(repo_description_raw) => {
            if repo_description_raw.len() <= 255 {
                repo_description_raw.to_string()
            } else {
                aprintln!("Too long description, maximum lenght 255, now: {len}", len = repo_description_raw.len());
                match *cli_only {
                    true => return Err(cli_only_error),
                    false => prompt_repo_description().await,
                }
            }
        }
        None => match *cli_only {
            true => return Err(cli_only_error),
            false => prompt_repo_description().await,
        }
    };
    Ok(repo_description)
}

fn resolve_github_path<'path>(github_path_or_url: &'path str) -> Option<(&'path str, &'path str)> {
    let mut path = github_path_or_url;
    path = path.strip_prefix("https://").unwrap_or(path);
    path = path.strip_prefix("http://").unwrap_or(path);
    path = path.strip_prefix("github.com").unwrap_or(path);
    path = path.strip_prefix("/").unwrap_or(path);
    path = path.strip_suffix("/").unwrap_or(path);
    if path.contains("?") {
        path = path.split_once("?").expect("found one").0;
        path = path.strip_suffix("/").unwrap_or(path);
    }
    if path.contains("#") {
        path = path.split_once("#").expect("found one").0;
        path = path.strip_suffix("/").unwrap_or(path);
    }
    let mut path_splitter = path.splitn(3, "/");
    let username = match path_splitter.next() {
        Some(s) => s,
        None => return None,
    };
    let mut repo_name = match path_splitter.next() {
        Some(s) => s,
        None => return None,
    };
    repo_name = repo_name.strip_suffix(".git").unwrap_or(repo_name);
    if !re_username().is_match(username) { return None; }
    if !re_repo_name().is_match(repo_name) { return None; }
    Some((username, repo_name))
}

async fn get_external_path(
    octocrab_client: &octocrab::Octocrab,
    external_path_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<(String, String, octocrab::models::Repository)> {
    let cli_only_error = GhRsError::CliOnly("external repo url".to_string());

    async fn prompt_external_path() -> (String, String) {
        loop {
            let external_url_answer = prompt("GitHub external repo url", None).await.trim().to_string();
            match resolve_github_path(&external_url_answer) {
                Some((username_str, repo_name_str)) => return (username_str.to_string(), repo_name_str.to_string()),
                None => {
                    aprintln!(
                        "Invalid url format, expected {ex_url} or just {ex_repo_path}",
                        ex_url = get_gh_rs_github_url(),
                        ex_repo_path = get_gh_rs_github_repo_path(),
                    );
                    continue;
                }
            }
        }
    }

    loop {
        let (external_username, external_repo_name) = match external_path_raw_opt {
            Some(external_path_raw_possible) => match resolve_github_path(external_path_raw_possible) {
                None => {
                    aprintln!(
                        "Invalid url format, expected {ex_url} or just {ex_repo_path}",
                        ex_url = get_gh_rs_github_url(),
                        ex_repo_path = get_gh_rs_github_repo_path(),
                    );
                    match *cli_only {
                        true => return Err(cli_only_error),
                        false => prompt_external_path().await,
                    }
                },
                Some((external_username_str, external_repo_name_str)) => (external_username_str.to_string(), external_repo_name_str.to_string()),
            }
            None => match *cli_only {
                true => return Err(cli_only_error),
                false => prompt_external_path().await,
            }
        };
        match octocrab_client.repos(external_username.clone(), external_repo_name.clone()).get().await {
            Err(e) => {
                aprintln!(
                    "Repo {username}/{repo_name} is unavailable", 
                    username = external_username.clone(), 
                    repo_name = external_repo_name.clone()
                );
                if *cli_only {
                    return Err(cli_only_error);
                }
                continue;
            }
            Ok(repo) => {
                return Ok((external_username, external_repo_name, repo));
            }
        }
    }
}

async fn get_is_public(public_raw: &bool, default_is_public: bool, cli_only: &bool) -> bool {
    match *public_raw {
        true => *public_raw,
        false => match *cli_only {
            true => false,
            false => prompt_bool("Make it public?", Some(default_is_public)).await
        }
    }
}

fn get_readme_text(
    username: &str,
    repo_name: &str,
    repo_description: &str,
) -> String {
    format!("# {repo_name}
**{repo_description}**
[GitHub](https://github.com/{username}/{repo_name})

## Motivation
Add motivation here

## Installation
`add_installation_command_here`

## Usage
Add usage & examples here

## Development
### TODO:
- [x] Create project & repo
- [ ] Fill out that TODO list
- [ ] Fill out the Motivation section
- [ ] Fill out the Installation section
- [ ] Fill out the Usage section

## Credits:
- Local and remote (online) repo created from command line by [gh.rs]({gh_rs_github_url})


",
        gh_rs_github_url = get_gh_rs_github_url(),
    )
}


async fn run_new_cmd(
    repo_name_raw_opt: &Option<&str>,
    repo_description_raw_opt: &Option<&str>,
    public_raw: &bool,
    token_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<()> {
    let (octocrab_client, token, user) = get_auth(token_raw_opt, cli_only).await?;
    let username = user.login.clone();
    let repo_name = get_repo_name(&octocrab_client, &username, repo_name_raw_opt, None, cli_only).await?;
    let repo_description = get_repo_description(repo_description_raw_opt, cli_only).await?;
    let repo_public = get_is_public(public_raw, false, cli_only).await;

    let repo_path = path!("./{repo_name}");
    let _ = match fs::create_dir(&repo_path).await {
        Err(e) => return_cmd_err!("Failed to create directory ./{repo_name}, error: {e}"),
        Ok(r) => r,
    };
    let branch_name = "main";
    let repo = match git2::Repository::init_opts(
        &repo_path, 
        git2::RepositoryInitOptions::new()
            .initial_head("main")
    ) {
        Ok(r) => r,
        Err(e) => return_cmd_err!(
            "Failed to init repo in \"{repo_path_display}\", error: {e}",
            repo_path_display = path_display(&repo_path)
        ),
    };

    let readme_path = repo_path.join("README.md");
    let _ = match fs::write(&readme_path, get_readme_text(&username, &repo_name, &repo_description).as_bytes()).await {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to write README.md, error: {e}"),
    };

    let sig = match get_github_signature_from_machine().await {
        Some(sig_found) => sig_found,
        None => {
            if user.email.is_some() {
                match git2::Signature::now(
                    user.login.clone().as_str(), 
                    user.email.clone().expect("must be some").as_str()
                ) {
                    Ok(s) => s,
                    Err(e) => return_cmd_err!("Failed to create signature, error: {e}"),
                }
            } else {
                return_cmd_err!("Failed to find signature");
            }
        },
    };
    let _ = match git2_add_all_and_commit(
        &repo, 
        &sig, 
        ["."], 
        "Initial commit [gh.rs]",
    ).await {
        Err(e) => return_cmd_err!("Failed to commit, error: {e}"),
        Ok(r) => r,
    };
    let ssh_private_key = match get_or_create_ssh_key(&octocrab_client).await {
        Ok(k) => k,
        Err(e) => return_cmd_err!("Failed to create ssh key, error: {e}"),
    };
    let gh_repo: octocrab::models::Repository = match octocrab_repos_create(
        &octocrab_client,
        &repo_public,
        &repo_name,
        &repo_description,
    ).await {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to create repo, error: {e}"),
    };
    
    let remote_name = "origin";
    let remote_url = format!("https://github.com/{username}/{repo_name}.git");
    let _ = match git2_push(
        &repo,
        &remote_name,
        &remote_url,
        &false,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Err(e) => return_cmd_err!("Failed to push, error: {e}"),
        Ok(r) => r,
    };
    
    let _ = match git2_set_branch_upstream(&repo, branch_name, remote_name) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to set \"{branch_name}\" branch upstream, error: {e}"),
    };

    aprintln!("{hr}\n\nSUCCESS! Created {repo_name} repo.\nHappy hacking & have a nice day :)", hr = get_hr());
    Ok(())
}


async fn run_publish_cmd(
    repo_description_raw_opt: &Option<&str>,
    public_raw: &bool,
    token_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<()> {
    let (octocrab_client, token, user) = get_auth(token_raw_opt, cli_only).await?;
    let username = user.login.clone();
    
    let repo_description = get_repo_description(repo_description_raw_opt, cli_only).await?;
    let repo_public = get_is_public(public_raw, false, cli_only).await;
    
    let repo_path = path!("./").canonicalize().await.expect("must be able to canonicalize");
    let repo_name = repo_path.file_name().expect("must be able to get directory name").to_str().expect("must be able decode directory name as utf8");
    let branch_name = "main";
    let repo = match repo_path.join(".git").is_dir().await {
        true => match git2::Repository::open(&repo_path) {
            Ok(r) => r,
            Err(e) => return_cmd_err!(
                "Failed to open repo from \"{repo_path_display}\", error: {e}",
                repo_path_display = path_display(&repo_path)
            ),
        },
        false => match git2::Repository::init_opts(
            &repo_path, 
            git2::RepositoryInitOptions::new()
                .initial_head(branch_name)
        ) {
            Ok(r) => r,
            Err(e) => return_cmd_err!(
                "Failed to init repo in \"{repo_path_display}\", error: {e}",
                repo_path_display = path_display(&repo_path)
            ),
        },
    };

    let readme_path = repo_path.join("README.md");
    if !readme_path.exists().await {
        let _ = match fs::write(&readme_path, get_readme_text(&username, &repo_name, &repo_description).as_bytes()).await {
            Ok(r) => r,
            Err(e) => return_cmd_err!("Failed to write README.md, error: {e}"),
        };
    }
    
    let sig = match get_github_signature_from_machine().await {
        Some(sig_found) => sig_found,
        None => {
            if user.email.is_some() {
                match git2::Signature::now(
                    user.login.clone().as_str(), 
                    user.email.clone().expect("must be some").as_str()
                ) {
                    Ok(s) => s,
                    Err(e) => return_cmd_err!("Failed to create signature, error: {e}"),
                }
            } else {
                return_cmd_err!("Failed to find signature");
            }
        },
    };
    let _ = match git2_add_all_and_commit(
        &repo, 
        &sig, 
        ["."], 
        "Initial commit [gh.rs]",
    ).await {
        Err(e) => return_cmd_err!("Failed to commit, error: {e}"),
        Ok(r) => r,
    };
    let ssh_private_key = match get_or_create_ssh_key(&octocrab_client).await {
        Ok(k) => k,
        Err(e) => return_cmd_err!("Failed to create ssh key, error: {e}"),
    };
    let gh_repo: octocrab::models::Repository = match octocrab_repos_create(
        &octocrab_client,
        &repo_public,
        &repo_name,
        &repo_description,
    ).await {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to create repo, error: {e}"),
    };
    
    let remote_name = "origin";
    let remote_url = format!("https://github.com/{username}/{repo_name}.git");
    let _ = match git2_push(
        &repo,
        &remote_name,
        &remote_url,
        &false,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Err(e) => return_cmd_err!("Failed to push, error: {e}"),
        Ok(r) => r,
    };
    let _ = match git2_set_branch_upstream(&repo, branch_name, remote_name) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to set \"{branch_name}\" branch upstream, error: {e}"),
    };

    aprintln!("{hr}\n\nSUCCESS! Published {repo_name} repo.\nHappy hacking & have a nice day :)", hr = get_hr());
    Ok(())
}

async fn run_clone_cmd(
    external_path_raw_opt: &Option<&str>,
    token_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<()> {
    let (octocrab_client, token, user) = get_auth(token_raw_opt, cli_only).await?;
    let username = user.login.clone();
    let (external_username, external_repo_name, external_gh_repo) = get_external_path(&octocrab_client, external_path_raw_opt, cli_only).await?;
    
    let path = path!("./{external_repo_name}");
    let _ = match fs::create_dir(path.clone()).await {
        Err(e) => return_cmd_err!("Failed to create directory ./{external_repo_name}, error: {e}"),
        Ok(r) => r,
    };
    
    let ssh_private_key = match get_or_create_ssh_key(&octocrab_client).await {
        Ok(k) => k,
        Err(e) => return_cmd_err!("Failed to create ssh key, error: {e}"),
    };
    
    let repo_clone_url = format!("https://github.com/{external_username}/{external_repo_name}.git");
    let repo = match git2_clone(
        &repo_clone_url,
        &path,
        &false,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to clone repo, error: {e}"),
    };
    
    aprintln!("{hr}\n\nSUCCESS! Cloned {external_repo_name} repo.\nHappy hacking & have a nice day :)", hr = get_hr());
    Ok(())
}

async fn run_fork_cmd(
    external_path_raw_opt: &Option<&str>,
    repo_name_raw_opt: &Option<&str>,
    public_raw: &bool,
    token_raw_opt: &Option<&str>,
    cli_only: &bool,
) -> GhRsResult<()> {
    let (octocrab_client, token, user) = get_auth(token_raw_opt, cli_only).await?;
    let username = user.login.clone();
    let (external_username, external_repo_name, external_gh_repo) = get_external_path(&octocrab_client, external_path_raw_opt, cli_only).await?;

    let repo_name = get_repo_name(&octocrab_client, &username, repo_name_raw_opt, Some(&external_repo_name), cli_only).await?;
    let repo_public = get_is_public(public_raw, false, cli_only).await;


    let path = path!("./{repo_name}");
    let _ = match fs::create_dir(&path).await {
        Err(e) => return_cmd_err!("Failed to create directory ./{repo_name}, error: {e}"),
        Ok(r) => r,
    };
    
    let ssh_private_key = match get_or_create_ssh_key(&octocrab_client).await {
        Ok(k) => k,
        Err(e) => return_cmd_err!("Failed to create ssh key, error: {e}"),
    };

    let external_url = format!("https://github.com/{external_username}/{external_repo_name}.git");
    let external_repo = match git2_clone(
        &external_url,
        &path,
        &true,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to clone repo, error: {e}"),
    };
    
    let gh_repo: octocrab::models::Repository = match octocrab_repos_create(
        &octocrab_client,
        &repo_public,
        &repo_name,
        external_gh_repo.description.unwrap_or("".to_string()).as_str()
    ).await {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to create repo, error: {e}"),
    };
    
    let remote_name = "origin";
    let remote_url = format!("https://github.com/{username}/{repo_name}.git");
    let _ = match git2_push(
        &external_repo,
        &remote_name,
        &remote_url,
        &true,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Err(e) => return_cmd_err!("Failed to push, error: {e}"),
        Ok(r) => r,
    };
    
    let last_commit_id = external_repo.head().expect("has head")
        .peel_to_commit().expect("has commit")
        .id();
    let _ = match fs::remove_dir_all(&path).await {
        Err(e) => return_cmd_err!(
            "Failed to remove directory \"{path_display}\", error: {e}", 
            path_display = path_display(&path)
        ),
        Ok(r) => r,
    };

    let repo = match git2_clone(
        &remote_url,
        &path,
        &false,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to clone repo, error: {e}"),
    };
    
    let _ = match git2_fetch_until_commit(
        &repo,
        &remote_name,
        &remote_url,
        &last_commit_id,
        &tokio::time::Duration::new(0, 500_000_000),
        10,

        user.login.clone().as_str(), 
        &token, 
        &ssh_private_key,
    ).await {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to fetch remote until latest commit, error: {e}"),
    };

    let default_branch_name = match git2_default_branch_name(&repo) {
        Ok(n) => n,
        Err(e) => return_cmd_err!("Failed to get default branch name, error: {e}"),
    };

    let _ = match git2_set_branch_upstream(&repo, default_branch_name.as_str(), remote_name) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to set \"{default_branch_name}\" branch upstream, error: {e}"),
    };
    let remote_external = match repo.remote("external", &external_url) {
        Ok(r) => r,
        Err(e) => return_cmd_err!("Failed to create \"external\" remote, error: {e}"),
    };

    aprintln!("{hr}\n\nSUCCESS! Forked {external_repo_name} repo.\nHappy hacking & have a nice day :)", hr = get_hr());
    Ok(())
}

fn cmd_help_expanded_subcommands(
    root_cmd: &clap::Command, 
    subcommands: impl IntoIterator<Item = clap::Command> + Clone
) -> clap::builder::StyledStr {
    use std::fmt::Write;
    let mut subcmds_writer = clap::builder::StyledStr::new();
    
    let default_help_heading = clap::builder::Str::from("Commands");
    let help_heading = root_cmd
        .get_subcommand_help_heading()
        .unwrap_or(&default_help_heading);
    let header_style = root_cmd.get_styles().get_header();
    let _ = write!(
        subcmds_writer,
        "{}{help_heading}:{}\n",
        header_style.render(),
        header_style.render_reset()
    );
    
    const TAB: &str = "  ";
    const TAB_WIDTH: usize = TAB.len();
    fn render_native(cmd: &clap::Command, name: &str) -> clap::builder::StyledStr {
        cmd.clone().help_template(format!("{{{name}}}")).render_help()
    }
    fn ch_width(ch: char) -> usize {
        1 // or unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0) if unicode
    }
    fn display_width(text: &str) -> usize {
        let mut width = 0;

        let mut control_sequence = false;
        let control_terminate: char = 'm';

        for ch in text.chars() {
            if ch.is_ascii_control() {
                control_sequence = true;
            } else if control_sequence && ch == control_terminate {
                control_sequence = false;
                continue;
            }

            if !control_sequence {
                width += ch_width(ch);
            }
        }
        width
    }
    fn styled_str_display_width(st_str: &clap::builder::StyledStr) -> usize {
        let string = st_str.to_string();
        let iter = [string.as_str()].into_iter(); // or anstream::adapter::strip_str(&self.0) if color
        iter.fold(0, |w, c| w+display_width(c))
    }
    fn subcmd_names(cmd: &clap::Command) -> Vec<String> {
        let mut names = vec![cmd.get_name().to_string()];
        let mut aliases: Vec<String> = cmd.get_visible_aliases().map(|n| n.to_string()).collect();
        aliases.sort_by_key(|a| display_width(a));
        names.append(&mut aliases);
        names
    }
    fn subcmd_usage(cmd: &clap::Command) -> String {
        cmd
        .clone()
        .render_usage()
        .to_string()
        .replace(
            format!("Usage: {}", cmd.get_name()).as_str(),
            subcmd_names(cmd).join(", ").as_str()
        )
        
    }
    let longest = subcommands.clone().into_iter()
        .fold(2, |m, cmd| std::cmp::max(
            m, 
            display_width(subcmd_usage(&cmd).as_str())
        ));
    fn render_padding(amount: usize) -> String {
        let mut string = String::new();
        let _ = write!(string, "{:amount$}", "");
        string
    }
    for (i, subcmd) in subcommands.into_iter().enumerate() {
        if i > 0 {
            let _ = write!(subcmds_writer, "\n");
        }
        let usage = subcmd_usage(&subcmd);
        let padding_amount = longest - display_width(&usage) + TAB_WIDTH;
        let args = format!(
            "{TAB}{TAB}{}",
            render_native(&subcmd, "all-args").to_string()
                .replace("\n", format!("\n{TAB}{TAB}").as_str())
        );
        let _ = write!(
            subcmds_writer,
            "{TAB}{usage}{padding}{about}\n{args}",
            padding=render_padding(padding_amount),
            about=subcmd.get_about().unwrap_or(&(clap::builder::StyledStr::new())),
        );
    }

    let mut help_template = clap::builder::StyledStr::new();
    let _ = write!(
        help_template,
        "\
{{before-help}}{{about-with-newline}}
{{usage-heading}} {{usage}}

{subcmds_writer_str}{{after-help}}\
    ",
        subcmds_writer_str = subcmds_writer.ansi()
    );
    root_cmd.clone().help_template(help_template).render_help()
}

async fn async_main() {
    let name_arg = arg!(name: -n --name <name> "Set new repo name");
    let description_arg = arg!(description: -d --description <description> "Set new repo description")
        .visible_alias("descr");
    let public_arg = arg!(public: -p --public "Make repo public")
        .visible_aliases(["pub", "is-public", "make-public"]);
    let token_arg = arg!(token: -t --token <access_token> "Set GitHub access token, filled automatically if stored by git")
        .visible_aliases(["tok", "access-token"]);
    let cli_only_arg = arg!(cli_only: -c --"cli-only" "CLI-only mode, no prompts, will error if something is not specified, all bools will be set to false automatically")
        .visible_aliases(["co", "np", "no-prompt", "no-prompts", "no-prompting"]);
    let external_arg = arg!(external: -e --external <repo_url> "Set external repo url"); 
    
    let after_help = format!(
        "gh.rs GitHub: {url}",
        url = get_gh_rs_github_url(),
    );
    
    let new_cmd = cmd!(-n --new "Create new repo (local & GitHub)")
        .args([
            &name_arg,
            &description_arg,
            &public_arg,
            &token_arg,
            &cli_only_arg,
        ])
        .after_help(&after_help);
    
    let publish_cmd = cmd!(-p -pub --publish "Publish current directory to GitHub")
        .args([
            &description_arg,
            &public_arg,
            &token_arg,
            &cli_only_arg,
        ])
        .after_help(&after_help);

    let clone_cmd = cmd!(-c --clone "Clone GitHub repo")
        .args([
            &external_arg,
            &token_arg,
            &cli_only_arg,
        ])
        .after_help(&after_help);

    let fork_cmd = cmd!(-f --fork "Fork GitHub repo")
        .args([
            &external_arg,
            &name_arg,
            &public_arg,
            &token_arg,
            &cli_only_arg,
        ])
        .after_help(&after_help);
    
    let help_full_cmd = cmd!(--"help-full" "Print help fully, describing every command")
        .disable_help_flag(true);

    let subcommands = [
        new_cmd,
        publish_cmd,
        clone_cmd,
        fork_cmd,
        help_full_cmd,
    ];

    let root_cmd = cmd!(--"gh.rs")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommands(&subcommands)
        .after_help(&after_help);

    let result = match root_cmd.clone().get_matches().subcommand() {
        Some((subcmd, submatches)) => match subcmd {
            "new" => run_new_cmd(
                &submatches.get_one::<String>("name").map(|v| v.as_str()),
                &submatches.get_one::<String>("description").map(|v| v.as_str()),
                &submatches.get_flag("public"),
                &submatches.get_one::<String>("token").map(|v| v.as_str()),
                &submatches.get_flag("cli_only"),
            ).await,
            "publish" => run_publish_cmd(
                &submatches.get_one::<String>("description").map(|v| v.as_str()),
                &submatches.get_flag("public"),
                &submatches.get_one::<String>("token").map(|v| v.as_str()),
                &submatches.get_flag("cli_only"),
            ).await,
            "clone" => run_clone_cmd(
                &submatches.get_one::<String>("external").map(|v| v.as_str()),
                &submatches.get_one::<String>("token").map(|v| v.as_str()),
                &submatches.get_flag("cli_only"),
            ).await,
            "fork" => run_fork_cmd(
                &submatches.get_one::<String>("external").map(|v| v.as_str()),
                &submatches.get_one::<String>("name").map(|v| v.as_str()),
                &submatches.get_flag("public"),
                &submatches.get_one::<String>("token").map(|v| v.as_str()),
                &submatches.get_flag("cli_only"),
            ).await,
            "help-full" => {
                let st_str = cmd_help_expanded_subcommands(&root_cmd, subcommands);
                aprintln!("{}", st_str.ansi());
                Ok(())
            },
            _ => {
                aprintln!("Command \"{subcmd}\" not found");
                let help = root_cmd.clone().render_help();
                aprintln!("{help}");
                Ok(())
            },
        },
        None => {
            let help = root_cmd.clone().render_help();
            aprintln!("{help}");
            Ok(())
        },
    };

    match result {
        Ok(()) => (),
        Err(e) => aprintln!("{e}"),
    }
}

pub fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { async_main().await });
}
