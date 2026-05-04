use crate::msg::Msg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    ChatMessage(Msg),
    ShowErrorMsg(String),
    ShowInfoMsg(String),
    CrawlerSearch { nip: i32 },
}
