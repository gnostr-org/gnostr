#[cfg(test)]
mod tests {
    use actix_web::App as WebApp;
    use actix_test::start;
    use gnostr_relay::App as GnostrRelayApp;
    use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
    use nostr_0_34_1::{EventBuilder, Kind, Keys, Tag};
    use serde_json::json;
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::connect_async;
    use futures_util::{StreamExt, SinkExt};
    use anyhow::Result;
    use std::fs;
    use tempfile::NamedTempFile;


    fn create_test_app_instance(test_name: &str) -> Result<(GnostrRelayApp, String)> {
        // Create a temporary config file
        let config_file = NamedTempFile::new().expect("Failed to create temp config file");
        let config_path = config_file.path().to_str().unwrap().to_owned();
        let default_config_content = format!(r#"
            [server]
            port = 0 # Use a random available port
            host = "127.0.0.1"

            [database]
            path = ":memory:" # Use in-memory database for tests
        "#);
        fs::write(&config_path, default_config_content).expect("Failed to write temp config");

        let app_data = GnostrRelayApp::create(
            Some(&config_path),
            true,
            Some("NOSTR".to_owned()),
            None,
        )
        .expect("Failed to create GnostrRelayApp");

        let r = app_data.setting.read();
        let server_address = format!("{}:{}", r.network.host, r.network.port);
        drop(r);

        Ok((app_data, server_address))
    }

    #[actix_web::test]
	#[ignore]
    async fn test_server_starts_and_websocket_connects() -> Result<()> {
        let srv = start(|| {
            let (app_data, _server_address) = create_test_app_instance("test_server_starts_and_websocket_connects").unwrap();
            app_data.web_app()
        });

        let ws_url = srv.url(&BOOTSTRAP_RELAYS[0]);
        let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect to websocket");

        // Send a ping and expect a pong
        ws_stream.send(Message::Ping(vec![1, 2, 3])).await?;
        let msg = ws_stream.next().await.unwrap()?;
        assert_eq!(msg, Message::Pong(vec![1, 2, 3]));

        srv.stop().await;
        Ok(())
    }

    #[actix_web::test]
    #[ignore]
    async fn test_event_submission_and_retrieval() -> Result<()> {
        let srv = start(|| {
            let (app_data, _server_address) = create_test_app_instance("test_event_submission_and_retrieval").unwrap();
            app_data.web_app()
        });

        let ws_url = srv.url("/");
        let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect to websocket");

        let keys = Keys::generate();
        let tags = vec![Tag::parse(&["t", "gnostr"]).unwrap(),Tag::parse(&["t", "nostr"]).unwrap()];
        let event = EventBuilder::new(
            Kind::TextNote,
            "Hello gostr from test!",
            tags.into_iter(),
        ).to_event(&keys).unwrap();

        let event_json = json!(["EVENT", event]).to_string();

        // Send event
        ws_stream.send(Message::Text(event_json.clone())).await?;

        // Expect OK message
        let msg = ws_stream.next().await.unwrap()?;
        let text = msg.to_text()?;
        assert!(text.contains("OK"));
        assert!(text.contains(&event.id.to_string()));

        // Send REQ to retrieve event
        let filter_json = json!(["REQ", "sub1", {"ids": [event.id]}]).to_string();
        ws_stream.send(Message::Text(filter_json)).await?;

        // Expect EVENT message
        let msg = ws_stream.next().await.unwrap()?;
        let text = msg.to_text()?;
        assert!(text.contains("EVENT"));
        assert!(text.contains(&event.id.to_string()));

        // Expect EOSE message
        let msg = ws_stream.next().await.unwrap()?;
        let text = msg.to_text()?;
        assert!(text.contains("EOSE"));

        srv.stop().await;
        Ok(())
    }
}
