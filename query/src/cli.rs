use clap::{Arg, ArgMatches, Command};
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
                .help("Comma-separated list of kinds (integers)"),
        )
        .arg(
            Arg::new("generic")
                .short('g')
                .long("generic")
                .value_names(["tag", "value"])
                .number_of_values(2)
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
                //.help("-r wss://relay.damus.io")
                .default_value("wss://relay.damus.io"),
        )
        .arg(
            Arg::new("search").short('s').long("search").required(false), //.help("-r wss://relay.damus.io")
                                                                          //.default_value("gnostr"),
        )
        .get_matches();

    Ok(matches)
}
