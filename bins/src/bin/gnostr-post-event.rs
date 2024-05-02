use gnostr_types::Event;
use std::convert::TryInto;
use std::env;
use std::io::Read;
use std::process;

fn main() {
    //!
    //! Usage: in context of other gnostr utilities
    //!
    //! gnostr --sec $(gnostr-sha256) --content 'test'  | gnostr-post-event wss://relay.damus.io
    //!
    //! gnostr --sec $(gnostr-sha256) --content "$(gnostr-git show HEAD)" | gnostr-post-event wss://relay.damus.io
    //!
    //! gnostr --sec $(gnostr-sha256) --content "$(gnostr-git-reflog -gd)" | gnostr-post-event wss://relay.damus.io
    //!

    //! gnostr --sec $(gnostr-sha256 $(gnostr-weeble)) -t gnostr -t gnostr-get-relays --tag weeble $(gnostr-weeble) --tag wobble $(gnostr-wobble) --content "test" | ./target/debug/gnostr-post-event
    //!
    //! gnostr --sec $(gnostr-sha256 $(gnostr-weeble)) -t gnostr -t gnostr-get-relays --tag weeble $(gnostr-weeble) --tag wobble $(gnostr-wobble) --content "test" | ./target/debug/gnostr-post-event | sed 's/\\//g'

    //! Usage: in context of other gnostr utilities
    //!
    //! gnostr --sec $(gnostr-sha256 $(gnostr-weeble)) -t gnostr -t gnostr-get-relays --tag weeble $(gnostr-weeble) --tag wobble $(gnostr-wobble) --content "gnostr\/$(gnostr-weeble)\/$(gnostr-blockheight)\/$(gnostr-wobble)" | ./target/debug/gnostr-post-event
    //!
    //!
    //! gnostr --sec $(gnostr-sha256 $(gnostr-weeble)) -t gnostr -t gnostr-get-relays --tag weeble $(gnostr-weeble) --tag wobble $(gnostr-wobble) --content "#gnostr/$(gnostr-weeble)/$(gnostr-blockheight)/$(gnostr-wobble)" | ./target/debug/gnostr-post-event
    //!
    //!
    //! $(echo gnostr --sec $(gnostr-sha256)) | ./target/debug/gnostr-post-event
    //let v: Vec<u8> = vec![0, 1, 2, 3];
    //// The `Vec` type implements the `Index` trait so you can do:
    //println!("{:?}", v);
    //if v.len() == 0 {
    //    println!("v.len() = {}", 0);
    //};
    //if v.len() == 1 {
    //    println!("v.len() = {}", 1);
    //};
    //if v.len() == 2 {
    //    println!("v.len() = {}", 2);
    //};
    //if v.len() == 3 {
    //    println!("v.len() = {}", 3);
    //} else {
    //    println!("{}:{}", "zero", v[0]);
    //    println!("{}:{}", "one", v[1]);
    //    println!("{}:{}", "two", v[2]);
    //    println!("{}:{}", "three", v[3]);
    //    println!("v.len() = {}", v.len());
    //};

    let args_vector: Vec<String> = env::args().collect();
    //println!("args_vector = {:?}", args_vector);
    //println!("args_vector.len() = {:?}", args_vector.len());

    #[allow(unreachable_code)]
    for i in 0..args_vector.len() {
        //println!("58:i={}", i);
        if i == args_vector.len() {
            //println!("60:i={}", i);
            //println!("61:i-1={}", i-1);
            //println!("process::exit(0)");
            //process::exit(0);
            process::exit(i.try_into().unwrap());
            //unsafe { libc::exit(1); }
        } else {
            //println!("66:i={}", i);
            //println!("args_vector[{}]={}", i, args_vector[i]);

            if args_vector.len() == 0 {
                println!("args_vector.len() = {}", 0);
            };
            if args_vector.len() == 1 {
                //println!("args_vector[{}]={}", i, args_vector[i]);
                //no args case
                //no args case
                //no args case

                //println!("77:i={}", i);
                //println!("args_vector[{}]={}", i, args_vector[i]);
                //println!("args_vector.len() = {}", 1);

                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event
                let event: Event = serde_json::from_str(&s).unwrap();

                let relay_url = "wss://nos.lol";
                gnostr_bins::post_event(&relay_url, event);
            };
            if args_vector.len() == 2 {
                //println!("92:i={}", i);
                //println!("args_vector[{}]={}", i, args_vector[i]);
                //println!("args_vector.len() = {}", args_vector.len());

                //catch help
                if args_vector[1] == "-h" {
                    println!("-h HELP!");
                    process::exit(0);
                }
                if args_vector[1] == "--help" {
                    println!("--help HELP!");
                    process::exit(0);
                }
                //println!("i={}", i);
                //println!("args_vector[{}]={}", i, args_vector[i]);

                //catch version
                if args_vector[1] == "-v" {
                    println!("-v VERSION!");
                    process::exit(0);
                }
                if args_vector[1] == "--version" {
                    println!("--version VERSION!");
                    process::exit(0);
                }
                //catch missing url
                //catch missing url
                //catch missing url
                //catch missing url
                if args_vector[1] == "--relay" {
                    //println!("--relay RELAY!");
                    let relay_url = "wss://nos.lol";
                    //println!("relay_url={}", relay_url);
                    let mut s: String = String::new();
                    std::io::stdin().read_to_string(&mut s).unwrap();
                    println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event
                    let event: Event = serde_json::from_str(&s).unwrap();
                    gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }

                //else assume the second arg is the relay url
                let relay_url = &args_vector[1];
                //catch the stream
                //example:
                //gnostr --sec <privkey> --content "<string>" | gnostr-post-event <relay_url>
                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event
                let event: Event = serde_json::from_str(&s).unwrap();
                gnostr_bins::post_event(relay_url, event);
                process::exit(0);

                //if not -h --help or -v --version
                //assume the arg is an event

                //let mut s: String = String::new();
                //std::io::stdin().read_to_string(&mut s).unwrap();
                //println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event
                //let event: Event = serde_json::from_str(&s).unwrap();

                //let relay_url = "wss://nos.lol";
                //gnostr_bins::post_event(&relay_url, event);

                //println!("args_vector.len() = {}", 2);
                //let app: Vec<u8> = args_vector[0].clone().into();
                //println!("app.len() = {:?}", app.len());
                //println!("Searching for {:?}", app);
                //let relay: Vec<u8> = args_vector[1].clone().into();
                //println!("relay.len() = {:?}", relay.len());
                //println!("Searching for {:?}", relay);

                process::exit(0);
            };
            if args_vector.len() == 3 {
                //println!("args_vector.len() = {}", 3);
                //println!("args_vector[{}]={}", 2, args_vector[2]);
                if args_vector[1] == "--relay" {
                    //println!("--relay RELAY!");
                    let relay_url = &args_vector[2];
                    //println!("relay_url={}", relay_url);
                    let mut s: String = String::new();
                    std::io::stdin().read_to_string(&mut s).unwrap();
                    println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event
                    let event: Event = serde_json::from_str(&s).unwrap();
                    gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }

                let relay_url = &args_vector[3 - 1];
                //println!("relay_url={}", relay_url);

                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();

                println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event

                // TODO: detect { EVENT: } envelope
                let event: Event = serde_json::from_str(&s).unwrap();

                gnostr_bins::post_event(relay_url, event);
            };
            //process::exit(0);
        }
        //process::exit(0);
    }

    //let mut args = env::args();

    //let _ = args.next(); // program name
    //let relay_url = match args.next() {
    //    Some(u) => u,
    //    None => panic!("Usage:\ngnostr --sec $(gnostr-sha256) --content 'test'  | gnostr-post-event wss://relay.damus.io\ngnostr --sec $(gnostr-sha256) --content \"$(gnostr-git show HEAD)\ngnostr --sec $(gnostr-sha256) --content \"$(gnostr-git-reflog -gd)\" | gnostr-post-event wss://relay.damus.io | gnostr-post-event wss://relay.damus.io"),
    //};

    //let mut s: String = String::new();
    //std::io::stdin().read_to_string(&mut s).unwrap();

    //println!("{}", s); //TODO:write event to .gnostr/EVENT_HASH.event

    ////TODO: detect { EVENT: } envelope
    //let event: Event = serde_json::from_str(&s).unwrap();

    //gnostr_bins::post_event(&relay_url, event);
}
