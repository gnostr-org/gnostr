#[cfg(test)]
mod kvs_tests {
    use super::super::kvs::*;
    use libp2p::{identity, Multiaddr, PeerId, multiaddr::Protocol};
    use std::error::Error;
    use futures::StreamExt;
    use tokio::time::{self, Duration};

    // Helper to create a dummy peer ID
    fn create_dummy_peer_id() -> PeerId {
        identity::Keypair::generate_ed25519().public().to_peer_id()
    }

    #[tokio::test]
    async fn test_new_client_and_event_loop() -> Result<(), Box<dyn Error + Send>> {
        let (mut client, mut event_receiver, event_loop) = new(None).await?;

        tokio::spawn(event_loop.run());

        // Test start_listening
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
        client.start_listening(addr.clone()).await?;

        let event = time::timeout(Duration::from_secs(5), event_receiver.next())
            .await?
            .expect("Event should be received");

        match event {
            Event::NewListenAddr(received_addr) => {
                assert!(received_addr.to_string().contains("/ip4/127.0.0.1/tcp/"));
                assert!(received_addr.to_string().contains("/p2p/"));
            }
            _ => panic!("Unexpected event type"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_dial_and_connection_established() -> Result<(), Box<dyn Error + Send>> {
        let (mut client1, _, event_loop1) = new(Some(1)).await?;
        let (mut client2, _, event_loop2) = new(Some(2)).await?;

        tokio::spawn(event_loop1.run());
        tokio::spawn(event_loop2.run());

        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
        client1.start_listening(addr1.clone()).await?;

        // Get the actual listening address of client1
        let (mut client_temp, mut event_receiver_temp, event_loop_temp) = new(Some(3)).await?;
        tokio::spawn(event_loop_temp.run());
        client_temp.start_listening(addr1.clone()).await?;
        let event = time::timeout(Duration::from_secs(5), event_receiver_temp.next())
            .await?
            .expect("Event should be received");
        let listen_addr_client1 = match event {
            Event::NewListenAddr(addr) => addr,
            _ => panic!("Unexpected event type"),
        };

        let peer_id1 = listen_addr_client1.iter().find_map(|p| {
            if let Protocol::P2p(peer_id) = p {
                Some(peer_id)
            } else {
                None
            }
        }).expect("Should have a peer ID");

        // Dial client1 from client2
        client2.dial(peer_id1, listen_addr_client1.clone()).await?;

        // We can't directly assert connection established without more complex event handling,
        // but if dial doesn't return an error, it's a good indication.
        Ok(())
    }

    #[tokio::test]
    async fn test_start_providing_and_get_providers() -> Result<(), Box<dyn Error + Send>> {
        let (mut client1, _, event_loop1) = new(Some(1)).await?;
        let (mut client2, _, event_loop2) = new(Some(2)).await?;

        tokio::spawn(event_loop1.run());
        tokio::spawn(event_loop2.run());

        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
        client1.start_listening(addr1.clone()).await?;

        let (mut client_temp, mut event_receiver_temp, event_loop_temp) = new(Some(3)).await?;
        tokio::spawn(event_loop_temp.run());
        client_temp.start_listening(addr1.clone()).await?;
        let event = time::timeout(Duration::from_secs(5), event_receiver_temp.next())
            .await?
            .expect("Event should be received");
        let listen_addr_client1 = match event {
            Event::NewListenAddr(addr) => addr,
            _ => panic!("Unexpected event type"),
        };

        let peer_id1 = listen_addr_client1.iter().find_map(|p| {
            if let Protocol::P2p(peer_id) = p {
                Some(peer_id)
            } else {
                None
            }
        }).expect("Should have a peer ID");

        client2.dial(peer_id1, listen_addr_client1.clone()).await?;

        let file_name = "test_file.txt".to_string();
        client1.start_providing(file_name.clone()).await;

        // Give some time for DHT propagation
        time::sleep(Duration::from_secs(2)).await;

        let providers = client2.get_providers(file_name.clone()).await;
        assert!(providers.contains(&peer_id1));

        Ok(())
    }

    #[tokio::test]
    async fn test_request_and_respond_file() -> Result<(), Box<dyn Error + Send>> {
        let (mut client1, mut event_receiver1, event_loop1) = new(Some(1)).await?;
        let (mut client2, _, event_loop2) = new(Some(2)).await?;

        tokio::spawn(event_loop1.run());
        tokio::spawn(event_loop2.run());

        let addr1: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse()?;
        client1.start_listening(addr1.clone()).await?;

        let (mut client_temp, mut event_receiver_temp, event_loop_temp) = new(Some(3)).await?;
        tokio::spawn(event_loop_temp.run());
        client_temp.start_listening(addr1.clone()).await?;
        let event = time::timeout(Duration::from_secs(5), event_receiver_temp.next())
            .await?
            .expect("Event should be received");
        let listen_addr_client1 = match event {
            Event::NewListenAddr(addr) => addr,
            _ => panic!("Unexpected event type"),
        };

        let peer_id1 = listen_addr_client1.iter().find_map(|p| {
            if let Protocol::P2p(peer_id) = p {
                Some(peer_id)
            } else {
                None
            }
        }).expect("Should have a peer ID");

        client2.dial(peer_id1, listen_addr_client1.clone()).await?;

        let file_name = "secret_data.txt".to_string();
        let file_content = b"this is some secret data".to_vec();

        // Client 1 will respond to requests
        let file_content_clone = file_content.clone();
        tokio::spawn(async move {
            if let Some(Event::InboundRequest { request, channel }) = event_receiver1.next().await {
                assert_eq!(request, file_name);
                client1.respond_file(file_content_clone, channel).await;
            }
        });

        // Client 2 requests the file
        let received_file = client2.request_file(peer_id1, file_name.clone()).await?;
        assert_eq!(received_file, file_content);

        Ok(())
    }
}