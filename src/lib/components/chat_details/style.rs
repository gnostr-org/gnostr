use std::borrow::Cow;

use ratatui::text::Span;

use crate::{strings, ui::style::SharedTheme};

pub enum Detail {
    Author,
    Date,
    Committer,
    Sha,
    Message,
}

//displayed in home topiclist view
pub fn style_detail<'a>(theme: &'a SharedTheme, field: &Detail) -> Span<'a> {
    match field {
        Detail::Author => Span::styled(
            Cow::from(strings::commit::chat_details_author()),
            theme.text(false, false),
        ),
        Detail::Date => Span::styled(
            Cow::from(strings::commit::chat_details_date()),
            theme.text(false, false),
        ),
        Detail::Committer => Span::styled(
            Cow::from(strings::commit::chat_details_committer()),
            theme.text(false, false),
        ),
        //
        Detail::Sha => Span::styled(
            Cow::from(strings::commit::chat_details_sha()),
            theme.text(false, false),
        ),
        Detail::Message => Span::styled(
            Cow::from(strings::commit::chat_details_message()),
            theme.text(false, false),
        ),
    }
}
