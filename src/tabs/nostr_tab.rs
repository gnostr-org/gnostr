use crate::components::{CommandBlocking, CommandInfo, Component, DrawableComponent, EventState, nostrlist::NostrListComponent};
use crate::components::nostr_types::NostrItem;
use asyncgit::nostr::PatchStatus;
use crate::keys::SharedKeyConfig;
use crate::ui::style::SharedTheme;
use anyhow::Result;
use crossterm::event::Event;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph},
    Frame,
};

/// NostrTab: displays Nostr events (patches, issues, announcements) with navigation like Revlog.
pub struct NostrTab {
    list: NostrListComponent,
    visible: bool,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
    pub status_msg: String,
}

impl NostrTab {
    pub fn new(theme: SharedTheme, key_config: SharedKeyConfig) -> Self {
        Self {
            list: NostrListComponent::new(theme.clone(), key_config.clone()),
            visible: false,
            theme,
            key_config,
            status_msg: String::from("disconnected"),
        }
    }

    pub fn push_patch(&mut self, patch: NostrItem) {
        self.list.push_patch(patch);
    }
    pub fn push_issue(&mut self, issue: NostrItem) {
        self.list.push_issue(issue);
    }
    pub fn push_announcement(&mut self, ann: NostrItem) {
        self.list.push_announcement(ann);
    }
    pub fn apply_status(&mut self, target_id: &str, status: crate::tabs::nostr_tab::PatchStatus) {
        self.list.apply_status(target_id, status);
    }
    pub fn selected_item(&self) -> Option<&crate::tabs::nostr_tab::NostrItem> {
        self.list.selected_item()
    }
    fn item_count(&self) -> usize {
        self.list.item_count()
    }
    fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let count = self.item_count();
        let msg = format!(" {} | {} items ", self.status_msg, count);
        let p = Paragraph::new(msg).style(self.theme.text(false, false));
        f.render_widget(p, area);
    }
}

impl NostrTab {
    /// Set the entire list of Nostr items (patches, issues, announcements)
    pub fn set_items(&mut self, items: Vec<NostrItem>) {
        self.list.set_items(items);
    }
}

impl DrawableComponent for NostrTab {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, rect: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(rect);
        self.list.draw(f, chunks[0])?;
        // Draw details of the selected item in chunks[1]
        if let Some(item) = self.selected_item() {
            let detail = format!(
                "Type: {}\nStatus: {}\nPubkey: {}\n\n{}",
                item.kind_label(),
                item.status_label(),
                item.pubkey_short(),
                item.content()
            );
            let p = Paragraph::new(detail).style(self.theme.text(false, false));
            f.render_widget(p, chunks[1]);
        }
        let footer_area = Rect {
            x: rect.x,
            y: rect.y + rect.height.saturating_sub(1),
            width: rect.width,
            height: 1,
        };
        self.draw_footer(f, footer_area);
        Ok(())
    }
}

impl Component for NostrTab {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            self.list.commands(out, force_all);
        }
        CommandBlocking::PassingOn
    }
    fn event(&mut self, ev: &Event) -> Result<EventState> {
        if !self.visible {
            return Ok(EventState::NotConsumed);
        }
        let list_result = self.list.event(ev)?;
        if list_result.is_consumed() {
            return Ok(EventState::Consumed);
        }
        Ok(EventState::NotConsumed)
    }
    fn is_visible(&self) -> bool {
        self.visible
    }
    fn hide(&mut self) {
        self.visible = false;
    }
    fn show(&mut self) -> Result<()> {
        self.visible = true;
        Ok(())
    }
}
