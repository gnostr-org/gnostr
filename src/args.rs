use std::{
	env,
	fs::{self, File},
	path::PathBuf,
};

use anyhow::{Result, anyhow};
use asyncgit::sync::RepoPath;
//use clap::arg;
use clap::{
	Arg, Command as ClapApp, crate_authors, crate_description,
	crate_name,
};
use simplelog::{Config, LevelFilter, WriteLogger};

use crate::bug_report;

pub struct CliArgs {
	pub theme: PathBuf,
	pub repo_path: RepoPath,
	pub notify_watcher: bool,
}

pub fn process_cmdline() -> Result<CliArgs> {
	let app = app();

	let arg_matches = app.get_matches();

	let sec = arg_matches
		.get_one::<String>("sec")
		.map_or_else(|| String::from(""), String::from);
	let t = arg_matches
		.get_one::<String>("t")
		.map_or_else(|| String::from(""), String::from);
	let tag = arg_matches
		.get_one::<String>("tag")
		.map_or_else(|| String::from(""), String::from);

	if sec.len() != 0 {
		print!("sec={}\n", sec);
		print!(
			"BEGIN gnostr --sec --tag <string> <string> -t <string> --content \"<string>\" ect...\n"
		);
		if arg_matches.get_flag("t") {
			if sec.len() > 0 {
				print!("sec={}\n", sec);
				print!("t={}\n", t);
			} else {
				print!("sec={}\n", sec);
				print!("t={}\n", t);
			}
		}
		if arg_matches.get_flag("tag") {
			if sec.len() > 0 {
				print!("sec={}\n", sec);
				print!("t={}\n", tag);
			} else {
				print!("sec={}\n", sec);
				print!("t={}\n", tag);
			}
		}
	} else {
	}
	if arg_matches.get_flag("cli") {
		if sec.len() > 0 {
			print!("sec={}\n", sec);
			print!("cli!!");
		} else {
			print!("sec={}\n", sec);
			print!("cli!!");
		}
	}
	//std::process::exit(0);

	if arg_matches.get_flag("bugreport") {
		bug_report::generate_bugreport();
		std::process::exit(0);
	}
	if arg_matches.get_flag("logging") {
		setup_logging()?;
	}

	// //TODO: gnostr-tui cli sub-command
	if arg_matches.get_flag("cli") {
		//setup_logging()?;
		print!("cli!!");
		std::process::exit(0);
	}
	// if arg_matches.get_flag("sec") {
	// 	//setup_logging()?;
	// 	print!("handle --sec");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("tag") {
	// 	//setup_logging()?;
	// 	print!("handle a --tag");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("t") {
	// 	//setup_logging()?;
	// 	print!("handle a -t");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("envelope") {
	// 	//setup_logging()?;
	// 	print!("handle a --envelope");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("created-at") {
	// 	//setup_logging()?;
	// 	print!("handle --created-at");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("e") {
	// 	//setup_logging()?;
	// 	print!("handle -e");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("p") {
	// 	//setup_logging()?;
	// 	print!("handle -p");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("pow") {
	// 	//setup_logging()?;
	// 	print!("handle --pow");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("dm") {
	// 	//setup_logging()?;
	// 	print!("handle --dm");
	// 	std::process::exit(0);
	// }
	// if arg_matches.get_flag("kind") {
	// 	//setup_logging()?;
	// 	print!("handle --kind");
	// 	std::process::exit(0);
	// }

	let workdir =
		arg_matches.get_one::<String>("workdir").map(PathBuf::from);
	let gitdir = arg_matches
		.get_one::<String>("directory")
		.map_or_else(|| PathBuf::from("."), PathBuf::from);

	#[allow(clippy::option_if_let_else)]
	let repo_path = if let Some(w) = workdir {
		RepoPath::Workdir { gitdir, workdir: w }
	} else {
		RepoPath::Path(gitdir)
	};

	let arg_theme = arg_matches
		.get_one::<String>("theme")
		.map_or_else(|| PathBuf::from("theme.ron"), PathBuf::from);

	let theme = get_app_config_path()?.join(arg_theme);

	let notify_watcher: bool =
		*arg_matches.get_one("watcher").unwrap_or(&false);

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
			Arg::new("sec")
			.help("sec help")
			.value_name("SEC")
			.default_value("")
			.long("sec")
			.num_args(1)
			.action(clap::ArgAction::Append),
			)
		.arg(
			Arg::new("tag")
			.help("--tag <string> <string>")
			.value_name("TAG")
			.default_value("")
			.long("tag")
			.num_args(2)
			.action(clap::ArgAction::Append),
			)
		//.arg(
		//	Arg::new("t")
		//	.help("-t <string>")
		//	.value_name("T")
		//	.default_value("")
		//	.long("t")
		//	.num_args(1)
		//	.action(clap::ArgAction::Append),
		//	)
		.arg(
			Arg::new("content")
			.help("--content <string>")
			.value_name("CONTENT")
			.default_value("")
			.long("content")
			.num_args(1)
			.action(clap::ArgAction::Append),
			)
		.arg(
			Arg::new("cli")
				.help("Stores logging output into a cache directory")
				.long("cli")
				.num_args(0),
		)
		.arg(
			Arg::new("theme")
				.help("Set color theme filename loaded from config directory")
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
	path.push("gitui.log");

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

	path.push("gitui");
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
