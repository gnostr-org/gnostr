use gnostr::dns_resolver::{dns_resolver, dns_resolver_sys};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addresses = dns_resolver()?.clone();
    if addresses.is_empty() {
        //println!("No IP addresses found for www.example.com.");
    } else {
        //println!("IP Addresses for www.example.com:");
        for _address in &addresses.clone() {
            //println!("42:\n{:?}", address);
            // Note: Asserting specific IPs for www.example.com is brittle
            // as they can change. The previous example's assertion was incorrect.
            // You should typically just print and verify manually.
        }
    }
    let addresses_sys = dns_resolver_sys()?.clone();
    if addresses_sys.len() < 1 {
        println!("No IP addresses found for www.example.com.");
    } else {
        //println!("IP Addresses for www.example.com:");
        print!("{}", addresses_sys.replace("\"", ""));
        // Note: Asserting specific IPs for www.example.com is brittle
        // as they can change. The previous example's assertion was incorrect.
        // You should typically just print and verify manually.
    }
    Ok(())
}
