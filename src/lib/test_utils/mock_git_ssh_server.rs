// Mock Git SSH Server for Testing
// Solves hanging test issues by providing controlled, predictable behavior

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// Mock Git SSH Server that simulates git operations without complexity
/// Designed to prevent hanging and provide deterministic test behavior
pub struct MockGitSshServer {
    port: u16,
    responses: HashMap<String, String>,
    listener_handle: Option<tokio::task::JoinHandle<()>>,
    cancellation_token: Arc<tokio::sync::Notify>,
}

impl MockGitSshServer {
    /// Create a new mock Git SSH server
    pub fn new(port: u16) -> Self {
        let mut responses = HashMap::new();

        // Default git protocol responses
        responses.insert(
            "git-upload-pack".to_string(),
            "0000# service=git-upload-pack\\n0000\\n".to_string(),
        );
        responses.insert(
            "git-receive-pack".to_string(),
            "0000# service=git-receive-pack\\n0000\\n".to_string(),
        );
        responses.insert(
            "git-upload-archive".to_string(),
            "0000# service=git-upload-archive\\n0000\\n".to_string(),
        );

        Self {
            port,
            responses,
            listener_handle: None,
            cancellation_token: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Add custom response for a specific command
    pub fn add_response(&mut self, command: &str, response: &str) {
        self.responses
            .insert(command.to_string(), response.to_string());
    }

    /// Start the mock server
    pub async fn start(&mut self) -> Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .map_err(|e| anyhow!("Failed to bind to port {}: {}", self.port, e))?;

        info!("Mock Git SSH server started on port {}", self.port);

        let responses = self.responses.clone();
        let cancel_token = self.cancellation_token.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.notified() => {
                        info!("Mock Git SSH server received shutdown signal");
                        break;
                    }
                    result = listener.accept() => {
                        match result {
                            Ok((stream, _addr)) => {
                                let responses = responses.clone();
                                tokio::spawn(async move {
                                    Self::handle_connection(stream, responses).await;
                                });
                            }
                            Err(e) => {
                                info!("Error accepting connection: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });

        self.listener_handle = Some(handle);
        Ok(())
    }

    /// Stop the mock server
    pub async fn stop(&mut self) {
        if let Some(handle) = self.listener_handle.take() {
            self.cancellation_token.notify_one();
            let _ = handle.await;
        }
    }

    /// Handle individual git connections with deterministic responses
    async fn handle_connection(mut stream: TcpStream, responses: HashMap<String, String>) {
        let mut buffer = vec![0u8; 1024];

        // Simple SSH-like protocol handshake
        if let Ok(n) = stream.read(&mut buffer).await {
            if n > 0 {
                // Respond with SSH protocol version
                let response = "SSH-2.0-MockGitSshServer_1.0\\r\\n";
                if let Err(_) = stream.write_all(response.as_bytes()).await {
                    return;
                }

                // Skip to simple response phase
                if let Ok(_) = stream.read(&mut buffer).await {
                    // Try to match command from buffer
                    let request = String::from_utf8_lossy(&buffer);

                    for (command, response) in &responses {
                        if request.contains(command) {
                            let _ = stream.write_all(response.as_bytes()).await;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Get server address
    pub fn address(&self) -> String {
        format!("ssh://user@127.0.0.1:{}", self.port)
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{Duration, timeout};

    use super::*;

    #[tokio::test]
    async fn test_mock_server_basic_functionality() -> Result<()> {
        let mut server = MockGitSshServer::new(9999);

        // Add custom response
        server.add_response("custom-command", "mock-response");

        // Start server
        server.start().await?;

        // Give it time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop server
        server.stop().await;

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_server_timeout_prevention() -> Result<()> {
        let mut server = MockGitSshServer::new(9998);

        // Start server with timeout
        let mock_server_future = server.start();

        // Should complete quickly without hanging
        match timeout(Duration::from_secs(5), mock_server_future).await {
            Ok(_) => {
                server.stop().await;
                Ok(())
            }
            Err(_) => Err(anyhow!("Server setup timed out")),
        }
    }
}
