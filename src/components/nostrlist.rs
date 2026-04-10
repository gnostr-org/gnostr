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

#[derive(Clone, Debug)]
pub struct NostrListComponent {
    items: Vec<crate::tabs::nostr_tab::NostrItem>,
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

    pub fn set_items(&mut self, items: Vec<crate::tabs::nostr_tab::NostrItem>) {
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

    pub fn push_patch(&mut self, patch: crate::tabs::nostr_tab::NostrItem) {
        self.items.push(patch);
        self.sort_items();
    }

    pub fn push_issue(&mut self, issue: crate::tabs::nostr_tab::NostrItem) {
        self.items.push(issue);
        self.sort_items();
    }

    pub fn push_announcement(&mut self, ann: crate::tabs::nostr_tab::NostrItem) {
        // Deduplicate by repo_id + pubkey
        let exists = self.items.iter().any(|item| {
            if let crate::tabs::nostr_tab::NostrItem::Announcement(a) = item {
                if let crate::tabs::nostr_tab::NostrItem::Announcement(new_a) = &ann {
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

    pub fn apply_status(&mut self, target_id: &str, status: crate::tabs::nostr_tab::PatchStatus) {
        for item in &mut self.items {
            if item.id() == target_id {
                match item {
                    crate::tabs::nostr_tab::NostrItem::Patch(p) => p.status = status.clone(),
                    crate::tabs::nostr_tab::NostrItem::Issue(i) => i.status = status.clone(),
                    crate::tabs::nostr_tab::NostrItem::Announcement(_) => {}
                }
            }
        }
    }

    pub fn selected_item(&self) -> Option<&crate::tabs::nostr_tab::NostrItem> {
        self.items.get(self.selected)
    }

    pub fn sort_items(&mut self) {
        self.items.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
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
            .map(|item| ListItem::new(item.clone()))
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Nostr Events"));
        f.render_stateful_widget(list, area, &mut *self.list_state.borrow_mut());
        Ok(())
    }
}

impl Component for NostrListComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            out.push(CommandInfo::new("Navigate up", true, self.visible || force_all));
            out.push(CommandInfo::new("Navigate down", true, self.visible || force_all));
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
