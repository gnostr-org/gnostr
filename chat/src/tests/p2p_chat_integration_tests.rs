#[cfg(test)]
mod tests {
    use std::time::Duration;

    use libp2p::gossipsub;
    use tokio::sync::mpsc;

    use crate::{
        event::ChatEvent,
        evt_loop,
        msg::{Msg, MsgKind},
    };

    async fn next_chat_message(
        recv: &mut mpsc::Receiver<ChatEvent>,
        timeout: Duration,
    ) -> ChatEvent {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let now = tokio::time::Instant::now();
            let remaining = deadline.saturating_duration_since(now);
            let event = tokio::time::timeout(remaining, recv.recv())
                .await
                .expect("Timeout waiting for chat event")
                .expect("Channel closed before receiving chat event");

            if matches!(event, ChatEvent::ChatMessage(_)) {
                return event;
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_p2p_connectivity_two_nodes() {
        // Create channels for two chat instances
        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, mut recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-two-nodes");

        // Spawn the event loops for two peers
        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));

        // Give some time for peers to discover each other via mDNS.
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Send a message from peer 1
        let msg1_content = "Hello from peer 1";
        let msg1 = Msg {
            from: "peer1".to_string(),
            ..Msg::default()
        }
        .set_content(msg1_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx1
            .send(ChatEvent::ChatMessage(msg1))
            .await
            .unwrap();

        // Receive the message on peer 2's side
        let received_event = next_chat_message(&mut recv_rx2, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
            assert_eq!(received_msg.kind, MsgKind::Chat);
        } else {
            panic!("Received wrong event type on peer 2: {:?}", received_event);
        }

        // Send a message from peer 2
        let msg2_content = "Hello from peer 2";
        let msg2 = Msg {
            from: "peer2".to_string(),
            ..Msg::default()
        }
        .set_content(msg2_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx2
            .send(ChatEvent::ChatMessage(msg2))
            .await
            .unwrap();

        // Receive the message on peer 1's side
        let received_event_2 = next_chat_message(&mut recv_rx1, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg_2) = received_event_2 {
            assert_eq!(received_msg_2.from, "peer2");
            assert_eq!(received_msg_2.content[0], msg2_content);
            assert_eq!(received_msg_2.kind, MsgKind::Chat);
        } else {
            panic!(
                "Received wrong event type on peer 1: {:?}",
                received_event_2
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "long_tests")]
    #[ignore]
    async fn test_p2p_connectivity_three_nodes() {
        // Create channels for three chat instances
        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, mut _recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);
        let (send_tx3, send_rx3) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx3, mut recv_rx3) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-three-nodes");

        // Spawn the event loops for three peers
        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));
        tokio::spawn(evt_loop(send_rx3, recv_tx3, topic.clone()));

        // Give some time for peers to discover each other.
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Send a message from peer 1
        let msg1_content = "Hello from peer 1 to all";
        let msg1 = Msg {
            from: "peer1".to_string(),
            ..Msg::default()
        }
        .set_content(msg1_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx1
            .send(ChatEvent::ChatMessage(msg1))
            .await
            .unwrap();

        // Peer 2 should receive the message
        let received_event_2 = next_chat_message(&mut recv_rx2, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event_2 {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
        } else {
            panic!(
                "Received wrong event type on peer 2: {:?}",
                received_event_2
            );
        }

        // Peer 3 should also receive the message
        let received_event_3 = next_chat_message(&mut recv_rx3, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event_3 {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
        } else {
            panic!(
                "Received wrong event type on peer 3: {:?}",
                received_event_3
            );
        }

        // Test multi-sender scenarios: Send messages from peers 2 and 3
        let msg2_content = "Response from peer 2";
        let msg2 = Msg {
            from: "peer2".to_string(),
            ..Msg::default()
        }
        .set_content(msg2_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        let msg3_content = "Response from peer 3";
        let msg3 = Msg {
            from: "peer3".to_string(),
            ..Msg::default()
        }
        .set_content(msg3_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        // Send messages concurrently from peers 2 and 3
        let (tx2_result, tx3_result) = tokio::join!(
            send_tx2.send(ChatEvent::ChatMessage(msg2)),
            send_tx3.send(ChatEvent::ChatMessage(msg3))
        );
        tx2_result.unwrap();
        tx3_result.unwrap();

        // Peer 1 should receive messages from both peers 2 and 3
        let mut received_messages = Vec::new();
        for _ in 0..2 {
            let received_event = next_chat_message(&mut _recv_rx1, Duration::from_secs(10)).await;

            if let ChatEvent::ChatMessage(msg) = received_event {
                received_messages.push(msg);
            } else {
                panic!("Received wrong event type on peer 1: {:?}", received_event);
            }
        }

        // Verify we received both messages (order may vary due to network timing)
        let mut received_from_2 = false;
        let mut received_from_3 = false;
        for msg in &received_messages {
            if msg.from == "peer2" {
                assert_eq!(msg.content[0], msg2_content);
                received_from_2 = true;
            } else if msg.from == "peer3" {
                assert_eq!(msg.content[0], msg3_content);
                received_from_3 = true;
            }
        }
        assert!(received_from_2, "Did not receive message from peer 2");
        assert!(received_from_3, "Did not receive message from peer 3");
    }
}
