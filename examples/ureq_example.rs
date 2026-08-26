use std::time::Duration;
use ureq::{Agent, AgentBuilder};

fn main() -> Result<(), ureq::Error> {
    let blockheight = blockheight();
    println!("{:?}", blockheight);
    // 1. Global Timeout for the entire request
    // This timeout covers the entire process from connecting to receiving the full body.
    let agent_with_global_timeout: Agent = AgentBuilder::new()
        .timeout(Duration::from_secs(1)) // Set global timeout to 1 second
        .build();

    match agent_with_global_timeout
        .get("http://httpbin.org/delay/2")
        .call()
    {
        // This request delays for 2 seconds
        Ok(_) => println!(
            "Global Timeout: Request succeeded (should not happen with a 1-second timeout)"
        ),
        Err(e) => println!("Global Timeout: Request failed with error: {:?}", e),
    }

    // 2. Connect Timeout
    // This timeout applies only to the initial connection phase.
    let agent_with_connect_timeout: Agent = AgentBuilder::new()
        .timeout_connect(Duration::from_secs(1)) // Set connect timeout to 1 second
        .build();

    match agent_with_connect_timeout
        .get("http://nonexistent.url")
        .call()
    {
        // This URL will likely fail to connect
        Ok(_) => println!("Connect Timeout: Request succeeded (should not happen)"),
        Err(e) => println!("Connect Timeout: Request failed with error: {:?}", e),
    }

    // 3. Read Timeout
    // This timeout applies to reading the response headers and body after the connection is established.
    let agent_with_read_timeout: Agent = AgentBuilder::new()
        .timeout_read(Duration::from_secs(1)) // Set read timeout to 1 second
        .build();

    match agent_with_read_timeout
        .get("http://httpbin.org/drip?delay=2&numbytes=100")
        .call()
    {
        // This request sends data slowly
        Ok(_) => {
            println!("Read Timeout: Request succeeded (should not happen with a 1-second timeout)")
        }
        Err(e) => println!("Read Timeout: Request failed with error: {:?}", e),
    }

    // 4. Write Timeout
    // This timeout applies to sending the request body.
    let agent_with_write_timeout: Agent = AgentBuilder::new()
        .timeout_write(Duration::from_secs(1)) // Set write timeout to 1 second
        .build();

    match agent_with_write_timeout.put("http://httpbin.org/post")
        .send_string("This is a long string that might cause a write timeout if the server is slow to receive it...")
    {
        Ok(_) => println!("Write Timeout: Request succeeded"),
        Err(e) => println!("Write Timeout: Request failed with error: {:?}", e),
    }

    let _ = print_result();
    Ok(())
}

fn print_result() -> Result<(), ureq::Error> {
    let url = "http://httpbin.org/get";

    // Handle the result of the call
    match ureq::get(url).call() {
        Ok(response) => {
            println!("Request to {} successful!", url);

            // Get the status code
            println!("Status: {}", response.status()); // Example

            // Get a specific header
            if let Some(content_type) = response.header("Content-Type") {
                // Example
                println!("Content-Type: {}", content_type);
            } else {
                println!("Content-Type header not found.");
            }

            // Print the response body as a string
            match response.into_string() {
                Ok(body) => println!("Body:\n{}", body),
                Err(e) => println!("Error reading response body: {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Request to {} failed with error: {:?}", url, e);
        }
    }

    Ok(())
}
fn blockheight() -> Result<(), ureq::Error> {
    let url = "https://mempool.space/api/blocks/tip/height";

    // Handle the result of the call
    match ureq::get(url).call() {
        Ok(response) => {
            println!("Request to {} successful!", url);

            // Get the status code
            println!("Status: {}", response.status()); // Example

            // Get a specific header
            if let Some(content_type) = response.header("Content-Type") {
                // Example
                println!("Content-Type: {}", content_type);
            } else {
                println!("Content-Type header not found.");
            }

            // Print the response body as a string
            match response.into_string() {
                Ok(body) => println!("Body:\n{}", body),
                Err(e) => println!("Error reading response body: {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Request to {} failed with error: {:?}", url, e);
        }
    }

    Ok(())
}
