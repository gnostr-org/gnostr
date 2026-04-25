use clap::{Arg, ArgAction, ArgMatches, Args, Command};

use super::ConfigBuilder;

pub async fn cli() -> Result<ArgMatches, Box<dyn std::error::Error>> {
    let matches = Command::new("gnostr-query")
        .about("Construct nostr queries and send them over a websocket")
        .arg(
            Arg::new("authors")
                .short('a')
                .long("authors")
                .help("Comma-separated list of authors"),
        )
        .arg(
            Arg::new("mentions")
                .short('p')
                .long("mentions")
                .help("Comma-separated list of mentions"),
        )
        .arg(
            Arg::new("references")
                .short('e')
                .long("references")
                .help("Comma-separated list of references"),
        )
        .arg(
            Arg::new("hashtag")
                .short('t')
                .long("hashtag")
                .help("Comma-separated list of hashtags"),
        )
        .arg(
            Arg::new("ids")
                .short('i')
                .long("ids")
                .help("Comma-separated list of ids"),
        )
        .arg(
            Arg::new("kinds")
                .short('k')
                .long("kinds")
                .default_value("1630,1632,1621,30618,1633,1631,1617,30617")
                .help("Comma-separated list of kinds (integers)"),
        )
        .arg(
            Arg::new("generic")
                .short('g')
                .long("generic")
                .value_names(["tag", "value"])
                .num_args(2)
                .help("Generic tag query: #<tag>: value"),
        )
        .arg(
            Arg::new("limit")
                .short('l')
                .long("limit")
                .value_parser(clap::value_parser!(i32))
                .default_value("1")
                .help("Limit the number of results"),
        )
        .arg(
            Arg::new("relay")
                .short('r')
                .long("relay")
                .required(false)
                .value_delimiter(',')
                .action(ArgAction::Append),
        )
        .arg(Arg::new("search").short('s').long("search").required(false))
        .get_matches();

    Ok(matches)
}

#[derive(Args, Clone, Debug, Default)]
pub struct QuerySubCommand {
    #[arg(short, long)]
    #[arg(short = 'r', long, value_delimiter = ',', action = ArgAction::Append)]
    pub relay: Vec<String>,
    #[arg(short, long)]
    pub authors: Option<String>,
    #[arg(short, long)]
    pub ids: Option<String>,
    #[arg(short, long)]
    pub limit: Option<i32>,
    #[arg(long, num_args = 2, value_names = ["tag", "value"])]
    pub generic: Option<Vec<String>>,
    #[arg(short = 't', long)]
    pub hashtag: Option<String>,
    #[arg(short, long)]
    pub mentions: Option<String>,
    #[arg(short = 'e', long)]
    pub references: Option<String>,
    #[arg(
        short = 'k',
        long,
        default_value = "1630,1632,1621,30618,1633,1631,1617,30617"
    )]
    pub kinds: Option<String>,
    #[arg(short, long)]
    pub search: Option<Vec<String>>,
}

impl QuerySubCommand {
    pub fn into_config_builder(self) -> ConfigBuilder {
        let mut builder = ConfigBuilder::default();

        if let Some(relay) = self.relay.first() {
            builder = builder.host(relay);
        }
        if let Some(authors) = self.authors {
            builder = builder.authors(&authors);
        }
        if let Some(ids) = self.ids {
            builder = builder.ids(&ids);
        }
        if let Some(limit) = self.limit {
            builder = builder.limit(limit);
        }
        if let Some(generic) = self.generic {
            if generic.len() == 2 {
                builder = builder.generic(&generic[0], &generic[1]);
            }
        }
        if let Some(hashtag) = self.hashtag {
            builder = builder.hashtag(&hashtag);
        }
        if let Some(mentions) = self.mentions {
            builder = builder.mentions(&mentions);
        }
        if let Some(references) = self.references {
            builder = builder.references(&references);
        }
        if let Some(kinds) = self.kinds {
            builder = builder.kinds(&kinds);
        }
        if let Some(search) = self.search {
            if search.len() == 2 {
                builder = builder.search(&search[0], &search[1]);
            }
        }

        builder
    }
}
