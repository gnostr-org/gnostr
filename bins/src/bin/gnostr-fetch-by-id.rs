use std::env;

use gnostr_types::IdHex;

fn main() {
    //gnostr-fetch-by-id wss://relay.damus.io
    // fc7aeb54b43215bff5b561bf59fb72128e36d67ef1092ba9c8ca7cdcfc68a439

    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage:\ngnostr-fetch-by-id <RelayURL> <EventID>"),
    };
    let id = match args.next() {
        Some(id) => id,
        None => panic!("Usage: fetch_by_id <RelayURL> <EventID>"),
    };

    let idhex = IdHex::try_from_str(&id).unwrap();

    if let Some(event) = gnostr_bins::fetch_by_id(&relay_url, idhex) {
        gnostr_bins::print_event(&event);
    } else {
        println!("Not found");
    }
}
