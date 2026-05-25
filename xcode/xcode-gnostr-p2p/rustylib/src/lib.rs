#[path = "../../../shared/chat_bridge.rs"]
mod chat_bridge;

uniffi::setup_scaffolding!();
 
#[uniffi::export]
fn rust_hello() -> String {
    "Hello from Rust!".to_string()
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

#[uniffi::export]
pub fn p2p_network_start() -> String {
    gnostr_p2p::embedded_network::start()
}

#[uniffi::export]
pub fn p2p_network_status() -> String {
    gnostr_p2p::embedded_network::status()
}

#[uniffi::export]
pub fn p2p_network_stop() -> String {
    gnostr_p2p::embedded_network::stop()
}

#[uniffi::export]
pub fn p2p_network_logs() -> String {
    gnostr_p2p::embedded_network::logs()
}

#[uniffi::export]
pub fn chat_current_topic() -> String {
    chat_bridge::chat_current_topic()
}

#[uniffi::export]
pub fn chat_start(topic: String) -> String {
    chat_bridge::chat_start(topic)
}

#[uniffi::export]
pub fn chat_status() -> String {
    chat_bridge::chat_status()
}

#[uniffi::export]
pub fn chat_logs() -> String {
    chat_bridge::chat_logs()
}

#[uniffi::export]
pub fn chat_send(text: String) -> String {
    chat_bridge::chat_send(text)
}

#[uniffi::export]
pub fn chat_stop() -> String {
    chat_bridge::chat_stop()
}
