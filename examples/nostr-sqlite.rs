// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use std::time::Duration;

//use nostr_sdk_0_32_0::{EventBuilder, EventId, FromBech32, Keys, Kind, Metadata, SecretKey, Tag, Url};
use nostr_0_34_1::prelude::Tag;
use nostr_0_34_1::prelude::*;
use nostr_database_0_34_0::{
    nostr::event::Event, nostr::types::filter::Filter, NostrDatabase, Order,
};
use nostr_sqlite_0_34_0::SQLiteDatabase;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let secret_key =
        SecretKey::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .unwrap();
    let keys_a = Keys::new(secret_key);
    println!("Pubkey A: {}", keys_a.public_key());

    let secret_key =
        SecretKey::from_bech32("nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85")
            .unwrap();
    let keys_b = Keys::new(secret_key);
    println!("Pubkey B: {}", keys_b.public_key());

    let database = SQLiteDatabase::open(".git/nostr-cache.sqlite")
        .await
        .unwrap();

    println!(
        "Events stored: {}",
        database.count(vec![Filter::new()]).await.unwrap()
    );

    for i in 0..10000 {
        let event =
            EventBuilder::text_note(format!("Event #{i}"), [Tag::identifier(format!("myid{i}"))])
                .to_event(&keys_a)
                .unwrap();
        database.save_event(&event).await.unwrap();
        println!("{:?}", event);
        //}
        //
        let event = EventBuilder::text_note(
            format!("Reply to event #{i}"),
            [
                Tag::event(event.id),
                Tag::parse(&["p".to_string(), event.id.to_string()]).expect(""),
            ],
        )
        .to_event(&keys_b)
        .unwrap();
        database.save_event(&event).await.unwrap();
        println!("{:?}", event);
    }

    // for i in 0..100_000 {
    // let event = EventBuilder::text_note(format!("Event #{i}"), &[])
    // .to_event(&keys_a)
    // .unwrap();
    // database.save_event(&event).await.unwrap();
    //
    // let event = EventBuilder::text_note(
    // format!("Reply to event #{i}"),
    // &[
    // Tag::event(event.id),
    // Tag::PubKey(event.pubkey, None),
    // ],
    // )
    // .to_event(&keys_b)
    // .unwrap();
    // database.save_event(&event).await.unwrap();
    // }

    for i in 0..10 {
        let metadata = Metadata::new().name(format!("Name #{i}"));
        let event = EventBuilder::metadata(&metadata).to_event(&keys_a).unwrap();
        database.save_event(&event).await.unwrap();
    }

    // for i in 0..500_000 {
    // let event = EventBuilder::new(
    // Kind::Custom(123),
    // "Custom with d tag",
    // &[Tag::Identifier(format!("myid{i}"))],
    // )
    // .to_event(&keys_a)
    // .unwrap();
    // database.save_event(&event).await.unwrap();
    // }

    let event_id = EventId::all_zeros();
    database
        .event_id_seen(event_id, Url::parse("wss://relay.damus.io").unwrap())
        .await
        .unwrap();
    //database
    //    .event_id_seen(event_id, Url::parse("wss://relay.damus.io").unwrap())
    //    .await
    //    .unwrap();

    let relays = database.event_seen_on_relays(event_id).await.unwrap();
    println!("Seen on: {relays:?}");

    let events = database
        .query(
            vec![Filter::new()
                .kinds(vec![Kind::Metadata, Kind::Custom(123), Kind::TextNote])
                .limit(1000)
                //.kind(Kind::Custom(123))
                //.identifier("myid5000")
                .author(keys_a.public_key())],
            Order::Desc,
        )
        .await
        .unwrap();
    println!("Got {} events", events.len());

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await
    }
}
