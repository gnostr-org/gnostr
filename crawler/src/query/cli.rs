use clap::{Arg, ArgAction, ArgMatches, Command};

fn command() -> Command {
    Command::new("gnostr-query")
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
                .default_value("30617,30618,1617,1621,1630,1631,1632,1633")
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
}

pub async fn cli() -> Result<ArgMatches, Box<dyn std::error::Error>> {
    let matches = command().get_matches();

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::command;

    #[test]
    fn parses_multiple_relays_and_csv_values() {
        let matches = command().get_matches_from([
            "gnostr-query",
            "-r",
            "wss://relay.damus.io",
            "-r",
            "wss://blossom.gnostr.cloud",
            "-r",
            "wss://nos.lol,wss://relay.nos.social",
        ]);

        let relays: Vec<String> = matches
            .get_many::<String>("relay")
            .expect("relay values")
            .cloned()
            .collect();

        assert_eq!(
            relays,
            vec![
                "wss://relay.damus.io".to_string(),
                "wss://blossom.gnostr.cloud".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nos.social".to_string(),
            ]
        );
    }
}
