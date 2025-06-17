use gnostr::get_weeble;
use gnostr::{Command, Probe};
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{Filter, IdHex, RelayMessage, SubscriptionId};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let _ = args.next(); // program name
    let id: IdHex = match args.next() {
        Some(id) => IdHex::try_from_str(&id)?,
        None => "fbf73a17a4e0fe390aba1808a8d55f1b50717d5dd765b2904bf39eba18c51f7c"
            .to_string()
            .into(),
    };
    let relay_url = match args.next() {
        Some(u) => u,
        None => "wss://relay.damus.io".to_string(),
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
                    //println!("{}", serde_json::to_string(&e)?);
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
