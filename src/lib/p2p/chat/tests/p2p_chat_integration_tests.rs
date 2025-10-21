
#[cfg(test)]
mod tests {
    use super::super::super::super::queue::InternalEvent;
    use super::super::super::super::p2p::chat::msg::{Msg, MsgKind};
    use super::super::super::super::p2p::chat::ChatSubCommands;
    use super::super::super::super::p2p::evt_loop;
    use libp2p::gossipsub;
    use tokio::sync::mpsc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_evt_loop_message_sending_and_receiving() {
        // Create two chat instances
        let (send_tx1, send_rx1) = mpsc::channel::<Msg>(100);
        let (recv_tx1, mut recv_rx1) = mpsc::channel::<Msg>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<Msg>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<Msg>(100);

        let topic = gossipsub::IdentTopic::new("test-chat-topic");

        let args1 = ChatSubCommands {
            nsec: Some("0000000000000000000000000000000000000000000000000000000000000001".to_string()),
            password: None,
            name: Some("test_user_1".to_string()),
            topic: Some("test-chat-topic".to_string()),
            hash: None,
            disable_cli_spinners: false,
            info: false,
            debug: false,
            trace: false,
        };

        let args2 = ChatSubCommands {
            nsec: Some("0000000000000000000000000000000000000000000000000000000000000002".to_string()),
            password: None,
            name: Some("test_user_2".to_string()),
            topic: Some("test-chat-topic".to_string()),
            hash: None,
            disable_cli_spinners: false,
            info: false,
            debug: false,
            trace: false,
        };

        // Spawn the event loops
        tokio::spawn(async move {
            evt_loop(args1, send_rx1, recv_tx1, topic.clone()).await.unwrap();
        });
        tokio::spawn(async move {
            evt_loop(args2, send_rx2, recv_tx2, topic.clone()).await.unwrap();
        });

        tokio::time::sleep(Duration::from_secs(5)).await; // Give time for peers to discover each other

        // Send a message from user 1
        let chat_msg = Msg::default().set_content("Hello from user 1".to_string(), 0).set_kind(MsgKind::Chat);
        send_tx1.send(chat_msg.clone()).await.unwrap();

        // Receive the message on user 2's side
        let received_msg = tokio::time::timeout(Duration::from_secs(5), recv_rx2.recv()).await.unwrap().unwrap();
        assert_eq!(received_msg.content[0], "Hello from user 1");
        assert_eq!(received_msg.kind, MsgKind::Chat);

        // Send a message from user 2
        let chat_msg_2 = Msg::default().set_content("Hello from user 2".to_string(), 0).set_kind(MsgKind::Chat);
        send_tx2.send(chat_msg_2.clone()).await.unwrap();

        // Receive the message on user 1's side
        let received_msg_2 = tokio::time::timeout(Duration::from_secs(5), recv_rx1.recv()).await.unwrap().unwrap();
        assert_eq!(received_msg_2.content[0], "Hello from user 2");
        assert_eq!(received_msg_2.kind, MsgKind::Chat);
    }
}
