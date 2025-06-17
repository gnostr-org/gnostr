use gnostr::get_weeble;
use gnostr::{Command, Probe};
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{
    EventKind, Filter, IdHex, KeySigner, PreEvent, PrivateKey, RelayMessage, Signer,
    SubscriptionId, Unixtime,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => BOOTSTRAP_RELAYS[0].clone(),
    };

	println!("{}", relay_url);
    //if !relay_url.contains("damus") {
        // Create a new identity
        eprintln!("Generating keypair...");
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        let signer = KeySigner::from_private_key(private_key, "pass", 16).unwrap();

        let content = format!(
            "{}/{}/{}/{}",
			relay_url,
            gnostr::get_weeble().unwrap_or("0".to_string()),
            gnostr::get_blockheight().unwrap_or("0".to_string()),
            gnostr::get_wobble().unwrap_or("0".to_string())
        );
        // Create an event for testing the relay
        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            content: content.to_owned(),
            tags: vec![],
        };
        let event = signer.sign_event(pre_event).unwrap();
        event.verify(None).unwrap();

        // Connect to relay and handle commands
        let (to_probe, from_main) = tokio::sync::mpsc::channel::<Command>(100);
        let (to_main, mut from_probe) = tokio::sync::mpsc::channel::<RelayMessage>(100);
        let join_handle = tokio::spawn(async move {
            let mut probe = Probe::new(from_main, to_main);
            if let Err(e) = probe.connect_and_listen(&relay_url).await {
                eprintln!("{}", e);
            }
        });

        let id: IdHex = event.id.into();

        to_probe.send(Command::PostEvent(event.clone())).await?;

        loop {
            let message = Some(from_probe.recv().await.unwrap());
            println!("message:{:?}", message);
            match from_probe.recv().await.unwrap() {
                RelayMessage::Ok(id, _, _) => {
                    if id == event.id {
                        break;
                    }
                }
                RelayMessage::Notice(_) => {
                    to_probe.send(Command::Exit).await?;
                    return Ok(join_handle.await?);
                }
                _ => {}
            }
        }

        let our_sub_id = SubscriptionId(get_weeble().unwrap().to_string());
        let mut filter = Filter::new();
        filter.add_id(&id);
        to_probe
            .send(Command::FetchEvents(our_sub_id.clone(), vec![filter]))
            .await?;

        loop {
            match from_probe.recv().await.unwrap() {
                RelayMessage::Eose(subid) => {
                    if subid == our_sub_id {
                        to_probe.send(Command::Exit).await?;
                        break;
                    }
                }
                RelayMessage::Closed(subid, _) => {
                    if subid == our_sub_id {
                        to_probe.send(Command::Exit).await?;
                        break;
                    }
                }
                RelayMessage::Event(subid, e) => {
                    if subid == our_sub_id && e.id == event.id {
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
    //} else {Ok(())}
}
