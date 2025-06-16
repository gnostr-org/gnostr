use nostr_sdk_0_37_0::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Define the relay URL
    let relay_url = Url::parse("wss://relay.damus.io")?;

    // 2. Create a Client instance.
    // Keys::generate() creates a new random keypair.
    // The Client uses this keypair internally for signing events if you publish.
    let client = Client::new(Keys::generate()); 

    // 3. Add the relay to the client.
    // The `add_relay` method returns a Result, so we use `?` to propagate errors.
    client.add_relay(relay_url.clone()).await?;

    // 4. Connect to the relay.
    // The `connect` method returns a Result, so we use `?` to propagate connection errors.
    client.connect().await;

    // 5. Define the Filter for events.
    // We are looking for text notes (Kind 1) and requesting the latest 10.
    let filter = Filter::new()
        .kinds(vec![Kind::TextNote]) // Kind 1
        .limit(10); // Get the 10 most recent text notes

    // 6. Subscribe to events matching the filter.
    // The `subscribe` method now requires two arguments:
    // 1. A vector of filters (vec![filter] in this case).
    // 2. An `Option<SubscribeAutoCloseOptions>`. We pass `None` for no auto-closing.
    println!("Subscribing to kind 1 events from {}", relay_url);
    let subscription_id = client.subscribe(vec![filter], None).await?;

    // 7. Listen for events and notifications from the relay pool.
    // This loop will continue to receive events until the subscription is cancelled
    // or the client disconnects, or the `break` statement is hit.
    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        match notification {
            // When an Event is received
            RelayPoolNotification::Event { relay_url, event, subscription_id: received_sub_id } => {
                // Check if the received event belongs to our specific subscription
                if received_sub_id == *subscription_id {
                    println!("Received event from {}: {:?}", relay_url, event);
                    // You can add your event processing logic here.
                    // For example, print the content of a text note:
                    if event.kind == Kind::TextNote {
                        println!("  Content: {}", event.content);
                    }
                }
            },
            // When an End of Stored Events (EOSE) message is received for our subscription
            // The variant name for EOSE changed in recent nostr-sdk versions from `EndOfStoredEvents` to `Eose`.
            //RelayPoolNotification::Event { relay_url, subscription_id: received_sub_id } => {
            //    if received_sub_id == *subscription_id {
            //        println!("End of stored events from {} for subscription {}", relay_url, received_sub_id);
            //        // Once EOSE is received, it means the relay has sent all currently stored
            //        // events matching the filter. The subscription will remain active for new events.
            //        // If you only want the initial batch and then stop, you can unsubscribe.
            //        client.unsubscribe(*subscription_id).await; // Removed `vec!`
            //        break; // Exit the loop after receiving EOSE and unsubscribing for this example
            //    }
            //}
            // Ignore other types of notifications for this example
            _ => {},
        }
    }

    // 8. Disconnect from all relays.
    // The `disconnect` method also returns a Result.
    client.disconnect().await?;

    println!("Disconnected from relays.");

    Ok(()) // Indicate successful execution
}

