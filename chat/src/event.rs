#[cfg(feature = "fractal")]
use std::path::PathBuf;

use crate::msg::Msg;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    ChatMessage(Msg),
    ShowErrorMsg(String),
    ShowInfoMsg(String),
    PeerConnected {
        peer_id: String,
        endpoint: String,
    },
    CrawlerSearch {
        nip: i32,
    },
    MessagePublished,
    #[cfg(feature = "fractal")]
    PerfectIpProgress {
        root: String,
        received: usize,
        total: usize,
    },
    #[cfg(feature = "fractal")]
    PerfectIpComplete {
        root: String,
        output_path: PathBuf,
    },
}
