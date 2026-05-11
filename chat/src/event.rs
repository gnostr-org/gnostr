use crate::msg::Msg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    ChatMessage(Msg),
    ShowErrorMsg(String),
    ShowInfoMsg(String),
    PeerConnected { peer_id: String, endpoint: String },
    CrawlerSearch { nip: i32 },
}
