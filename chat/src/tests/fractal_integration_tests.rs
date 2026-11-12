#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::Duration;

    use tempfile::NamedTempFile;

    use crate::session::ChatNotification;
    use crate::ChatSession;

    async fn next_notification(
        rx: &mut tokio::sync::broadcast::Receiver<ChatNotification>,
        timeout: Duration,
    ) -> ChatNotification {
        tokio::time::timeout(timeout, rx.recv())
            .await
            .expect("timeout waiting for notification")
            .expect("notification channel closed")
    }

    #[tokio::test]
    #[ignore = "networked p2p fractal test: run with --ignored --nocapture"]
    async fn fractal_file_roundtrip_between_two_sessions() {
        let mut file = NamedTempFile::new().expect("temp file");
        let payload = b"fractal integration test payload\n".repeat(128);
        file.write_all(&payload).expect("write payload");

        let topic = format!("fractal-test-{}", std::process::id());

        let mut sender = ChatSession::connect(topic.clone(), None)
            .await
            .expect("sender session");
        let mut receiver = ChatSession::connect(topic.clone(), None)
            .await
            .expect("receiver session");

        sender
            .wait_for_connected(Duration::from_secs(30))
            .await
            .expect("sender connect");
        receiver
            .wait_for_connected(Duration::from_secs(30))
            .await
            .expect("receiver connect");

        let mut sender_rx = sender.subscribe();
        let mut receiver_rx = receiver.subscribe();

        sender
            .send_perfect_ip_file(file.path(), false, 3)
            .await
            .expect("send fractal file");

        // Wait for the sender to publish the manifest before checking the receiver.
        let mut published = false;
        for _ in 0..60 {
            let notification = next_notification(&mut sender_rx, Duration::from_secs(5)).await;
            match notification {
                ChatNotification::MessagePublished => {
                    published = true;
                    break;
                }
                ChatNotification::Error(text) => panic!("sender publish error: {text}"),
                _ => continue,
            }
        }
        assert!(published, "sender did not publish fractal manifest");

        // Wait for the manifest announcement on the receiver.
        let mut manifest_seen = false;
        for _ in 0..60 {
            let notification = next_notification(&mut receiver_rx, Duration::from_secs(5)).await;
            match notification {
                ChatNotification::ChatMessage(msg) => {
                    if msg.kind == crate::msg::MsgKind::PerfectIpManifest {
                        manifest_seen = true;
                        break;
                    }
                }
                _ => continue,
            }
        }
        assert!(manifest_seen, "receiver never saw fractal manifest");

        // Wait for completion.
        let mut complete = None;
        for _ in 0..120 {
            let notification = next_notification(&mut receiver_rx, Duration::from_secs(5)).await;
            match notification {
                ChatNotification::PerfectIpComplete {
                    root: _,
                    output_path,
                } => {
                    complete = Some(output_path);
                    break;
                }
                ChatNotification::PerfectIpProgress { .. } => continue,
                ChatNotification::Error(text) => panic!("fractal transfer error: {text}"),
                _ => continue,
            }
        }

        let output_path = complete.expect("receiver completed fractal download");
        let reconstructed = std::fs::read(&output_path).expect("read reconstructed file");
        assert_eq!(reconstructed, payload);
    }
}
