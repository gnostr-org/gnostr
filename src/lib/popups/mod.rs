mod blame_file;
mod branchlist;
mod chat;
mod commit;
mod compare_commits;
mod confirm;
mod create_branch;
mod display_chat;
mod externaleditor;
mod fetch;
mod file_revlog;
mod fuzzy_find;
mod help;
mod inspect_chat;
mod inspect_commit;
mod log_search;
mod msg;
mod openchat;
mod options;
mod pull;
mod push;
mod push_tags;
mod rename_branch;
mod reset;
mod revision_files;
mod stashmsg;
mod submodules;
mod tag_commit;
mod taglist;

pub use blame_file::{BlameFileOpen, BlameFilePopup};
pub use branchlist::BranchListPopup;
pub use chat::ChatPopup;
pub use commit::CommitPopup;
pub use compare_commits::CompareCommitsPopup;
pub use confirm::ConfirmPopup;
pub use create_branch::CreateBranchPopup;
pub use display_chat::{DisplayChatOpen, DisplayChatPopup};
pub use externaleditor::ExternalEditorPopup;
pub use fetch::FetchPopup;
pub use file_revlog::{FileRevOpen, FileRevlogPopup};
pub use fuzzy_find::FuzzyFindPopup;
pub use help::HelpPopup;
pub use inspect_chat::{InspectChatOpen, InspectChatPopup};
pub use inspect_commit::{InspectCommitOpen, InspectCommitPopup};
pub use log_search::LogSearchPopupPopup;
pub use msg::MsgPopup;
pub use openchat::ExternalChatPopup;
pub use options::{AppOption, OptionsPopup};
pub use pull::PullPopup;
pub use push::PushPopup;
pub use push_tags::PushTagsPopup;
use ratatui::{
    layout::Alignment,
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};
pub use rename_branch::RenameBranchPopup;
pub use reset::ResetPopup;
pub use revision_files::{FileTreeOpen, RevisionFilesPopup};
pub use stashmsg::StashMsgPopup;
pub use submodules::SubmodulesListPopup;
pub use tag_commit::TagCommitPopup;
pub use taglist::TagListPopup;

use crate::ui::style::Theme;

fn popup_paragraph<'a, T>(
    title: &'a str,
    content: T,
    theme: &Theme,
    focused: bool,
    block: bool,
) -> Paragraph<'a>
where
    T: Into<Text<'a>>,
{
    let paragraph = Paragraph::new(content.into())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    if block {
        paragraph.block(
            Block::default()
                .title(Span::styled(title, theme.title(focused)))
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(theme.block(focused)),
        )
    } else {
        paragraph
    }
}
