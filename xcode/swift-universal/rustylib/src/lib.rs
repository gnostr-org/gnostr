uniffi::setup_scaffolding!();
 
#[uniffi::export]
fn rust_hello() -> String {
    "rustylib/src/lib.rs:5:Hello from Rust!".to_string()
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
pub fn p2p_network_peers() -> String {
    gnostr_p2p::embedded_network::peers()
}

#[uniffi::export]
pub fn p2p_network_register_chat_topic(topic: String) -> String {
    gnostr_p2p::embedded_network::register_chat_topic(topic)
}

#[uniffi::export]
pub fn p2p_network_send_chat_message(topic: String, message: String) -> String {
    gnostr_p2p::embedded_network::send_chat_message(topic, message)
}
