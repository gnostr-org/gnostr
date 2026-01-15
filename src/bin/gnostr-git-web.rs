//#![cfg(feature = "gnostr-web")]

use clap::Parser;
use rocket::{get, launch, routes, State};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use tera::{Context, Tera};

use gnostr::web::*;

#[derive(Parser)]
struct Cli {
    #[clap(short, default_value = "8000")]
    port: u16,
    #[clap(short, long)]
    repos_config: Option<std::path::PathBuf>,
}

pub struct AppConfig {
    repos_config: Option<std::path::PathBuf>,
}

#[derive(Serialize)]
struct RepoInfo {
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Repo {
    info: RepoInfo,
    last_update: String,
}

#[get("/")]
fn index(tera: &State<Tera>) -> String {
    let mut context = Context::new();

    // Load repo.toml if it exists
    let repos = if std::path::Path::new("repo.toml").exists() {
        vec![Repo {
            info: RepoInfo {
                name: "gnostr".to_string(),
                description: "A Nostr-based Git implementation".to_string(),
            },
            last_update: "2024-01-14".to_string(),
        }]
    } else {
        vec![Repo {
            info: RepoInfo {
                name: "example".to_string(),
                description: "Example repository".to_string(),
            },
            last_update: "2024-01-01".to_string(),
        }]
    };

    context.insert("repos", &repos);

    tera.render("main.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e))
}

#[get("/repo/<repo>/<branch>")]
fn repo_detail(repo: String, branch: String, tera: &State<Tera>) -> String {
    let mut context = Context::new();
    context.insert("repo_name", &repo);
    context.insert("branch", &branch);

    tera.render("repo.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e))
}

#[get("/gnostr")]
fn gnostr_repo(tera: &State<Tera>) -> String {
    let mut context = Context::new();

    // Mock data for now
    let branches = vec!["main".to_string(), "develop".to_string()];
    let commits = vec![
        serde_json::json!({
            "oid": "abc123",
            "message": "Initial commit",
            "author": "Gnostr Team"
        }),
        serde_json::json!({
            "oid": "def456",
            "message": "Add web interface",
            "author": "Developer"
        }),
    ];

    context.insert("repo_name", &"gnostr");
    context.insert("branchName", &"main");
    context.insert("branches", &branches);
    context.insert("commits", &commits);
    context.insert("path", &"/repo/gnostr/");

    tera.render("repo.html", &context)
        .unwrap_or_else(|e| format!("Template error: {}", e))
}

#[launch]
fn rocket() -> _ {
    let cli = Cli::parse();

    let mut tera = Tera::default();
    tera.add_raw_template("main.html", include_str!("../../templates/main.html"))
        .expect("Failed to load main.html template");
    tera.add_raw_template("repo.html", include_str!("../../templates/repo.html"))
        .expect("Failed to load repo.html template");

    let config = rocket::Config::figment().merge(("port", cli.port));

    rocket::custom(config)
        .manage(tera)
        .manage(AppConfig {
            repos_config: cli.repos_config,
        })
        .mount("/", routes![index, repo_detail, gnostr_repo])
        .mount("/statics", rocket::fs::FileServer::from("statics"))
}
