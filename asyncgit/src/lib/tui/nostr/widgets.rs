use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

use super::{
    Event, EventKind, EventReference, Id, IdHex, Metadata, NAddr, NEvent, NostrBech32,
    NostrUrl, Profile, PublicKey, PublicKeyHex, RelayInformationDocument, RelayList,
    RelayMessage, RelayUrl, RelayUsage, RelayUsageSet, Tag, Unixtime, UncheckedUrl, Url,
};

macro_rules! debug_widget {
    ($name:ident, $ty:ty) => {
        pub struct $name<'a> {
            value: &'a $ty,
            block: Option<Block<'static>>,
            style: Style,
        }

        impl<'a> $name<'a> {
            pub fn new(value: &'a $ty) -> Self {
                Self {
                    value,
                    block: None,
                    style: Style::default(),
                }
            }

            pub fn block(mut self, block: Block<'static>) -> Self {
                self.block = Some(block);
                self
            }

            pub fn style(mut self, style: Style) -> Self {
                self.style = style;
                self
            }
        }

        impl Widget for $name<'_> {
            fn render(self, area: Rect, buf: &mut Buffer) {
                let mut widget = Paragraph::new(format!("{:#?}", self.value)).style(self.style);
                if let Some(block) = self.block {
                    widget = widget.block(block);
                }
                widget.render(area, buf);
            }
        }
    };
}

debug_widget!(EventWidget, Event);
debug_widget!(EventKindWidget, EventKind);
debug_widget!(EventReferenceWidget, EventReference);
debug_widget!(IdWidget, Id);
debug_widget!(IdHexWidget, IdHex);
debug_widget!(MetadataWidget, Metadata);
debug_widget!(NAddrWidget, NAddr);
debug_widget!(NEventWidget, NEvent);
debug_widget!(NostrBech32Widget, NostrBech32);
debug_widget!(NostrUrlWidget, NostrUrl);
debug_widget!(ProfileWidget, Profile);
debug_widget!(PublicKeyWidget, PublicKey);
debug_widget!(PublicKeyHexWidget, PublicKeyHex);
debug_widget!(RelayInformationDocumentWidget, RelayInformationDocument);
debug_widget!(RelayListWidget, RelayList);
debug_widget!(RelayMessageWidget, RelayMessage);
debug_widget!(RelayUrlWidget, RelayUrl);
debug_widget!(RelayUsageWidget, RelayUsage);
debug_widget!(RelayUsageSetWidget, RelayUsageSet);
debug_widget!(TagWidget, Tag);
debug_widget!(UnixtimeWidget, Unixtime);
debug_widget!(UncheckedUrlWidget, UncheckedUrl);
debug_widget!(UrlWidget, Url);
