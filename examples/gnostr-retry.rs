use gnostr::utils::retry::GnostrRetry;

fn my_sync_fn(_n: &str) -> Result<(), std::io::Error> {
    println!("my_sync_fn({})", _n);
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "generic error",
    ))
}

fn main() {
    // Retry the operation with a linear strategy (1 second delay, 2 retries)
    let retry_strategy = GnostrRetry::new_linear(1, 2);
    let result = retry_strategy.run(|| my_sync_fn("Hi"));
    assert!(result.is_err());
}
