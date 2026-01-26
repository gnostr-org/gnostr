#[cfg(test)]
mod tests {
    use crate::utils::is_port_in_use;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_is_port_in_use_available() {
        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        // Ensure the port is not reported as in use after binding and before dropping
        assert!(!is_port_in_use(port).await);
    }

    #[tokio::test]
    async fn test_is_port_in_use_in_use() {
        // Bind to a port to ensure it's in use
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Check if the port is reported as in use
        assert!(is_port_in_use(port).await);
    }
}
