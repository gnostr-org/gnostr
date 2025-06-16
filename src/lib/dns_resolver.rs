use crate::global_rt::global_rt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::{RData, RecordType};
use trust_dns_resolver::TokioAsyncResolver;

use std::process::Command;
use std::str;

fn dns_resolver_sys() -> Result<String, Box<dyn std::error::Error>> {
    // Specify the dig command and its arguments
    let output = Command::new("dig")
        .arg("TXT")
        .arg("+short")
        .arg("o-o.myaddr.l.google.com")
        .arg("@ns1.google.com")
        .output()?; // Execute the command and capture its output

    // Check if the command executed successfully
    if output.status.success() {
        // Print the standard output, trimmed of leading/trailing whitespace
        let stdout = str::from_utf8(&output.stdout)?.trim().to_string();
        //println!("23:\n\n{}\n\n", stdout);
        Ok(stdout)
    } else {
        // Print the standard error if the command failed
        let stderr = str::from_utf8(&output.stderr)?.trim().to_string();
        eprintln!("dig command failed:");
        eprintln!("Status: {}", output.status);
        eprintln!("Error: {}", stderr);
        Ok(stderr)
    }
}

//fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let addresses = dns_resolver()?.clone();
//    if addresses.is_empty() {
//        //println!("No IP addresses found for www.example.com.");
//    } else {
//        //println!("IP Addresses for www.example.com:");
//        for address in &addresses.clone() {
//            //println!("42:\n{:?}", address);
//            // Note: Asserting specific IPs for www.example.com is brittle
//            // as they can change. The previous example's assertion was incorrect.
//            // You should typically just print and verify manually.
//        }
//    }
//    let addresses_sys = dns_resolver_sys()?.clone();
//    if addresses_sys.len() < 1 {
//        println!("No IP addresses found for www.example.com.");
//    } else {
//        //println!("IP Addresses for www.example.com:");
//        print!("{}", addresses_sys.replace("\"", ""));
//        // Note: Asserting specific IPs for www.example.com is brittle
//        // as they can change. The previous example's assertion was incorrect.
//        // You should typically just print and verify manually.
//    }
//    Ok(())
//}
fn dns_resolver() -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
    // Get the global Tokio runtime. This avoids creating multiple runtimes.
    let runtime = global_rt();

    // Block on the asynchronous operation within the global runtime.
    runtime.block_on(async {
        // --- 1. Configure the Resolver to use a specific nameserver (8.8.8.8) ---
        let mut config = ResolverConfig::new();
        let ns_ip = IpAddr::V4("8.8.8.8".parse().unwrap());
        let ns_sock_addr = SocketAddr::new(ns_ip, 53);
        config.add_name_server(NameServerConfig::new(ns_sock_addr, Protocol::Udp));

        // Create the resolver with the custom configuration
        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());

        // --- 2. Perform the DNS Lookup for TXT records of o-o.myaddr.l.google.com ---
        // This is based on your initial request to mimic 'dig TXT +short o-o.myaddr.l.google.com'
        //println!("Looking up TXT records for o-o.myaddr.l.google.com @8.8.8.8...");
        let txt_response = resolver
            .lookup("o-o.myaddr.l.google.com.", RecordType::TXT)
            .await?;

        let txt_records: Vec<String> = txt_response
            .iter()
            .filter_map(|rdata: &RData| match rdata {
                RData::TXT(txt_rdata) => Some(txt_rdata.to_string()),
                _ => None,
            })
            .collect();

        if txt_records.is_empty() {
            //println!("No TXT records found for o-o.myaddr.l.google.com.");
        } else {
            //println!("TXT Records for o-o.myaddr.l.google.com:");
            for txt in txt_records {
                //println!("96:\n\n{}\n\n", txt);
            }
        }

        //println!("---");

        // --- 3. Perform the DNS Lookup for IP addresses of www.example.com ---
        //println!("Looking up IP addresses for www.example.com @8.8.8.8...");
        let ip_response = resolver.lookup_ip("www.example.com.").await?;

        // Iterate over the IP addresses found
        let addresses: Vec<IpAddr> = ip_response.iter().collect();

        if addresses.is_empty() {
            //println!("No IP addresses found for www.example.com.");
        } else {
            //println!("IP Addresses for www.example.com:");
            for address in &addresses {
                //println!("114:\n\n{}\n\n", address);
                // Note: Asserting specific IPs for www.example.com is brittle
                // as they can change. The previous example's assertion was incorrect.
                // You should typically just print and verify manually.
            }
        }

        Ok(addresses)
    })
}
