use std::{
	env,
	fs::{self, File},
	path::PathBuf,
};

use anyhow::{Result, anyhow};
use asyncgit::sync::RepoPath;
use clap::{
	Arg, Command as ClapApp, crate_authors, crate_description,
	crate_name,
};
use simplelog::{Config, LevelFilter, WriteLogger};

use crate::bug_report;

use crate::sub_commands::*;

use clap::{Parser, Subcommand};

use crate::sub_commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
	/// remote signer address
	#[arg(long, global = true)]
	pub bunker_uri: Option<String>,
	/// remote signer app secret key
	#[arg(long, global = true)]
	pub bunker_app_key: Option<String>,
	/// nsec or hex private key
	#[arg(short, long, global = true)]
	pub nsec: Option<String>,
	/// password to decrypt nsec
	#[arg(short, long, global = true)]
	pub password: Option<String>,
	/// disable spinner animations
	#[arg(long, action)]
	pub disable_cli_spinners: bool,
}

#[derive(Subcommand)]
pub enum Commands {
	/// update cache with latest updates from nostr
	Fetch(sub_commands::fetch::SubCommandArgs),
	/// signal you are this repo's maintainer accepting proposals via
	/// nostr
	Init(sub_commands::init::SubCommandArgs),
	/// issue commits as a proposal
	Send(sub_commands::send::SubCommandArgs),
	/// list proposals; checkout, apply or download selected
	List,
	/// send proposal revision
	Push(sub_commands::push::SubCommandArgs),
	/// fetch and apply new proposal commits / revisions linked to
	/// branch
	Pull,
	/// run with --nsec flag to change npub
	Login(sub_commands::login::SubCommandArgs),
}

pub struct CliArgs {
	pub theme: PathBuf,
	pub repo_path: RepoPath,
	pub notify_watcher: bool,
}

pub async fn process_cmdline() -> Result<CliArgs> {
	let app = app();

	let arg_matches = app.get_matches();

	let arg_theme = arg_matches
		.get_one::<String>("theme")
		.map_or_else(|| PathBuf::from("theme.ron"), PathBuf::from);

	let theme = get_app_config_path()?.join(arg_theme);

	let notify_watcher: bool =
		*arg_matches.get_one("watcher").unwrap_or(&false);


	if arg_matches.get_flag("bugreport") {
		bug_report::generate_bugreport();
		std::process::exit(0);
	}
	if arg_matches.get_flag("logging") {
		setup_logging()?;
	}

	let workdir =
		arg_matches.get_one::<String>("workdir").map(PathBuf::from);
	let gitdir = arg_matches
		.get_one::<String>("directory")
		.map_or_else(|| PathBuf::from("."), PathBuf::from);

	#[allow(clippy::option_if_let_else)]
	let repo_path = if let Some(w) = workdir {
		RepoPath::Workdir { gitdir, workdir: w }
	} else {

        //ngit functionality must be invoked from within the repo
       let cli = Cli::parse();
        //
       let _ = match &cli.command {
        //
               Commands::Fetch(args) => {
                       fetch::launch(&cli, &args).await
               }
        //
               Commands::Login(args) => {
                       login::launch(&cli, &args).await
               }
        //
               Commands::Init(args) => {
                       init::launch(&cli, &args).await
               }
        //
               Commands::Send(args) => {
                       send::launch(&cli, &args, false).await
               }
        //
               Commands::List => list::launch().await,
        //
               Commands::Pull => pull::launch().await,
        //
               Commands::Push(args) => {
        //
                       push::launch(&cli, &args).await
               }
        //
        };

	RepoPath::Path(gitdir)

	};

	Ok(CliArgs {
		theme,
		repo_path,
		notify_watcher,
	})
}

fn app() -> ClapApp {
	ClapApp::new(crate_name!())
		.author(crate_authors!())
		.version(env!("GITUI_BUILD_NAME"))
		.about(crate_description!())
		.help_template(
			"\
{before-help}gitui {version}
{author}
{about}

{usage-heading} {usage}

{all-args}{after-help}
		",
		)
		.arg(
			Arg::new("theme")
				.help("Set color theme filename loaded from config directory")
				.short('t')
				.long("theme")
				.value_name("THEME_FILE")
				.default_value("theme.ron")
				.num_args(1),
		)
		.arg(
			Arg::new("logging")
				.help("Stores logging output into a cache directory")
				.short('l')
				.long("logging")
				.num_args(0),
		)
		.arg(
			Arg::new("watcher")
				.help("Use notify-based file system watcher instead of tick-based update. This is more performant, but can cause issues on some platforms. See https://github.com/extrawurst/gitui/blob/master/FAQ.md#watcher for details.")
				.long("watcher")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new("bugreport")
				.help("Generate a bug report")
				.long("bugreport")
				.action(clap::ArgAction::SetTrue),
		)
		.arg(
			Arg::new("directory")
				.help("Set the git directory")
				.short('d')
				.long("directory")
				.env("GIT_DIR")
				.num_args(1),
		)
		.arg(
			Arg::new("workdir")
				.help("Set the working directory")
				.short('w')
				.long("workdir")
				.env("GIT_WORK_TREE")
				.num_args(1),
		)
}

fn setup_logging() -> Result<()> {
	let mut path = get_app_cache_path()?;
	path.push("gnostr-tui.log");

	println!("Logging enabled. log written to: {path:?}");

	WriteLogger::init(
		LevelFilter::Trace,
		Config::default(),
		File::create(path)?,
	)?;

	Ok(())
}

fn get_app_cache_path() -> Result<PathBuf> {
	let mut path = dirs::cache_dir()
		.ok_or_else(|| anyhow!("failed to find os cache dir."))?;

	path.push("gnostr-tui");
	fs::create_dir_all(&path)?;
	Ok(path)
}

pub fn get_app_config_path() -> Result<PathBuf> {
	let mut path = if cfg!(target_os = "macos") {
		dirs::home_dir().map(|h| h.join(".config"))
	} else {
		dirs::config_dir()
	}
	.ok_or_else(|| anyhow!("failed to find os config dir."))?;

	path.push("gitui");
	fs::create_dir_all(&path)?;
	Ok(path)
}

#[test]
fn verify_app() {
	app().debug_assert();
}
