use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use log::debug;

use crate::{
    get_weeble,
    types::{Filter, Id, IdHex, RelayMessage, SubscriptionId},
    Command, Probe,
};

#[derive(clap::Args, Debug, Clone)]
pub struct FetchByIdSubCommand {
    /// Event ID (hex) to fetch
    #[arg(short, long)]
    pub id: Option<String>,
    /// Relay URL to connect to
    #[arg(short, long)]
    pub relay: Option<String>,
}

pub async fn run_fetch_by_id(args: &FetchByIdSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    let id: IdHex = match args.id.clone() {
        Some(id_val) => {
            if id_val.starts_with("note1") {
                // bech32
                let id = Id::try_from_bech32_string(&id_val)?;
                id.into()
            } else {
                // hex
                IdHex::try_from_str(&id_val)?
            }
        }
        None => "fbf73a17a4e0fe390aba1808a8d55f1b50717d5dd765b2904bf39eba18c51f7c"
            .to_string()
            .into(),
    };
    let relay_url = match args.relay.clone() {
        Some(u) => u,
        None => BOOTSTRAP_RELAYS[0].to_string(),
    };

    let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
    let (to_main, mut from_probe) = tokio::sync::mpsc::channel::<RelayMessage>(100);
    let join_handle = tokio::spawn(async move {
        let mut probe = Probe::new(from_main, to_main);
        if let Err(e) = probe.connect_and_listen(&relay_url).await {
            eprintln!("{}", e);
        }
    });

    let mut filter = Filter::new();
    filter.add_id(&id);

    let our_sub_id = SubscriptionId(get_weeble().unwrap().to_string());

    to_probe
        .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
        .await?;

    loop {
        match from_probe.recv().await.unwrap() {
            RelayMessage::Eose(sub) => {
                if sub == our_sub_id {
                    to_probe.send(Command::Exit).await?;
                    break;
                }
            }
            RelayMessage::Event(sub, e) => {
                if sub == our_sub_id {
                    debug!("{}", serde_json::to_string(&e)?);
                }
            }
            RelayMessage::Closed(sub, _) => {
                if sub == our_sub_id {
                    to_probe.send(Command::Exit).await?;
                    break;
                }
            }
            RelayMessage::Notice(_) => {
                to_probe.send(Command::Exit).await?;
                break;
            }
            _ => {}
        }
    }

    Ok(join_handle.await?)
}
