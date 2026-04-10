use crate::components::{CommandBlocking, CommandInfo, Component, DrawableComponent, EventState};
use crate::keys::{key_match, SharedKeyConfig};
use crate::ui::style::SharedTheme;
use anyhow::Result;
use crossterm::event::{Event, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use std::cell::RefCell;
use crate::components::nostr_types::NostrItem;
use asyncgit::nostr::PatchStatus;
use crate::components::CommandText;

#[derive(Clone, Debug)]
pub struct NostrListComponent {
    items: Vec<NostrItem>,
    selected: usize,
    list_state: RefCell<ratatui::widgets::ListState>,
    theme: SharedTheme,
    key_config: SharedKeyConfig,
    visible: bool,
}

impl NostrListComponent {
    pub fn new(theme: SharedTheme, key_config: SharedKeyConfig) -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            list_state: RefCell::new(ratatui::widgets::ListState::default()),
            theme,
            key_config,
            visible: false,
        }
    }

    pub fn set_items(&mut self, items: Vec<NostrItem>) {
        self.items = items;
        self.selected = 0;
        self.list_state.borrow_mut().select(Some(self.selected));
    }

    pub fn move_selection_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.borrow_mut().select(Some(self.selected));
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.items.is_empty() && self.selected < self.items.len() - 1 {
            self.selected += 1;
            self.list_state.borrow_mut().select(Some(self.selected));
        }
    }

    pub fn push_patch(&mut self, patch: NostrItem) {
        self.items.push(patch);
        self.sort_items();
    }

    pub fn push_issue(&mut self, issue: NostrItem) {
        self.items.push(issue);
        self.sort_items();
    }

    pub fn push_announcement(&mut self, ann: NostrItem) {
        // Deduplicate by repo_id + pubkey
        let exists = self.items.iter().any(|item| {
            if let NostrItem::Announcement(a) = item {
                if let NostrItem::Announcement(new_a) = &ann {
                    a.pubkey == new_a.pubkey && a.repo_id == new_a.repo_id
                } else {
                    false
                }
            } else {
                false
            }
        });
        if !exists {
            self.items.push(ann);
            self.sort_items();
        }
    }

    pub fn apply_status(&mut self, target_id: &str, status: PatchStatus) {
        for item in &mut self.items {
            if NostrItem::id(item) == target_id {
                match item {
                    NostrItem::Patch(p) => p.status = status.clone(),
                    NostrItem::Issue(i) => i.status = status.clone(),
                    NostrItem::Announcement(_) => {}
                }
            }
        }
    }

    pub fn selected_item(&self) -> Option<&NostrItem> {
        self.items.get(self.selected)
    }

    pub fn sort_items(&mut self) {
        self.items.sort_by(|a: &NostrItem, b: &NostrItem| b.created_at().cmp(&a.created_at()));
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

impl DrawableComponent for NostrListComponent {
    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item: &NostrItem| ListItem::new(item.subject()))
            .collect();
        // Use theme for styling
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Nostr Events"))
            .style(self.theme.text(false, false));
        f.render_stateful_widget(list, area, &mut *self.list_state.borrow_mut());
        Ok(())
    }
}

impl Component for NostrListComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            out.push(CommandInfo::new(CommandText::new("Navigate up".to_owned(), "Move selection up", "navigation"), true, self.visible || force_all));
            out.push(CommandInfo::new(CommandText::new("Navigate down".to_owned(), "Move selection down", "navigation"), true, self.visible || force_all));
        }
        CommandBlocking::PassingOn
    }

    fn event(&mut self, ev: &Event) -> Result<EventState> {
        if !self.visible {
            return Ok(EventState::NotConsumed);
        }
        if let Event::Key(key) = ev {
            if key.kind == KeyEventKind::Press {
                if key_match(key, self.key_config.keys.move_up) {
                    self.move_selection_up();
                    return Ok(EventState::Consumed);
                } else if key_match(key, self.key_config.keys.move_down) {
                    self.move_selection_down();
                    return Ok(EventState::Consumed);
                }
            }
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
        if !self.items.is_empty() {
            self.list_state.borrow_mut().select(Some(self.selected));
        }
        Ok(())
    }
}
