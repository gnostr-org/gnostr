use std::cell::RefCell;
///
/// Displays patches (kind 1617) and issues (kind 1621) received from
/// nostr relays for the current repository.  Navigation mirrors the
/// existing `StashList` / `Revlog` tab style.
#[cfg(feature = "nostr")]
use asyncgit::nostr::{GitIssue, GitPatch, PatchStatus};
use anyhow::Result;
use crossterm::event::{Event, KeyEventKind};
use ratatui::{
	backend::Backend,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Modifier, Style},
	text::{Line, Span},
	widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
	Frame,
};

use crate::{
	components::{
		CommandBlocking, CommandInfo, Component, DrawableComponent,
		EventState,
	},
	keys::{key_match, SharedKeyConfig},
	ui::style::SharedTheme,
};

/// A unified item in the NIP-34 timeline.
#[cfg(feature = "nostr")]
#[derive(Clone, Debug)]
pub enum NostrItem {
	/// A NIP-34 patch (kind 1617).
	Patch(GitPatch),
	/// A NIP-34 issue (kind 1621).
	Issue(GitIssue),
}

#[cfg(feature = "nostr")]
impl NostrItem {
	fn subject(&self) -> &str {
		match self {
			Self::Patch(p) => &p.subject,
			Self::Issue(i) => &i.subject,
		}
	}

	fn pubkey_short(&self) -> String {
		let pk = match self {
			Self::Patch(p) => &p.pubkey,
			Self::Issue(i) => &i.pubkey,
		};
		if pk.len() >= 8 {
			format!("{}…{}", &pk[..4], &pk[pk.len() - 4..])
		} else {
			pk.clone()
		}
	}

	fn kind_label(&self) -> &'static str {
		match self {
			Self::Patch(_) => "patch",
			Self::Issue(_) => "issue",
		}
	}

	fn status_label(&self) -> &'static str {
		match self {
			Self::Patch(p) => p.status.label(),
			Self::Issue(i) => i.status.label(),
		}
	}

	fn id(&self) -> &str {
		match self {
			Self::Patch(p) => &p.id,
			Self::Issue(i) => &i.id,
		}
	}

	fn content(&self) -> &str {
		match self {
			Self::Patch(p) => &p.content,
			Self::Issue(i) => &i.content,
		}
	}

	fn created_at(&self) -> u64 {
		match self {
			Self::Patch(p) => p.created_at,
			Self::Issue(i) => i.created_at,
		}
	}
}

/// NIP-34 tab: shows a list of patches and issues from nostr relays.
pub struct NostrTab {
	#[cfg(feature = "nostr")]
	items: Vec<NostrItem>,
	selected: usize,
	list_state: RefCell<ListState>,
	visible: bool,
	theme: SharedTheme,
	key_config: SharedKeyConfig,
	/// Status message shown in the footer (e.g. connection state).
	pub status_msg: String,
}

impl NostrTab {
	/// Create a new `NostrTab`.
	pub fn new(
		theme: SharedTheme,
		key_config: SharedKeyConfig,
	) -> Self {
		Self {
			#[cfg(feature = "nostr")]
			items: Vec::new(),
			selected: 0,
			list_state: RefCell::new(ListState::default()),
			visible: false,
			theme,
			key_config,
			status_msg: String::from("disconnected"),
		}
	}

	/// Add a received patch to the timeline.
	#[cfg(feature = "nostr")]
	pub fn push_patch(&mut self, patch: GitPatch) {
		self.items.push(NostrItem::Patch(patch));
		self.sort_items();
	}

	/// Add a received issue to the timeline.
	#[cfg(feature = "nostr")]
	pub fn push_issue(&mut self, issue: GitIssue) {
		self.items.push(NostrItem::Issue(issue));
		self.sort_items();
	}

	/// Apply a status update to an existing item by event id.
	#[cfg(feature = "nostr")]
	pub fn apply_status(&mut self, target_id: &str, status: PatchStatus) {
		for item in &mut self.items {
			if item.id() == target_id {
				match item {
					NostrItem::Patch(p) => p.status = status.clone(),
					NostrItem::Issue(i) => i.status = status.clone(),
				}
			}
		}
	}

	/// Return the currently selected item, if any.
	#[cfg(feature = "nostr")]
	#[allow(dead_code)]
	pub fn selected_item(&self) -> Option<&NostrItem> {
		self.items.get(self.selected)
	}

	#[cfg(feature = "nostr")]
	fn sort_items(&mut self) {
		self.items.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
	}

	fn item_count(&self) -> usize {
		#[cfg(feature = "nostr")]
		return self.items.len();
		#[cfg(not(feature = "nostr"))]
		return 0;
	}

	fn move_selection_up(&mut self) {
		if self.selected > 0 {
			self.selected -= 1;
			self.list_state.borrow_mut().select(Some(self.selected));
		}
	}

	fn move_selection_down(&mut self) {
		if self.item_count() > 0
			&& self.selected < self.item_count() - 1
		{
			self.selected += 1;
			self.list_state.borrow_mut().select(Some(self.selected));
		}
	}

	fn draw_list<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		#[cfg(feature = "nostr")]
		{
			let items: Vec<ListItem> = self
				.items
				.iter()
				.map(|item| {
					let kind_span = Span::styled(
						format!("[{}] ", item.kind_label()),
						self.theme.text(true, false),
					);
					let status_span = Span::styled(
						format!("[{}] ", item.status_label()),
						self.theme.text(false, false),
					);
					let subject_span = Span::styled(
						item.subject().to_owned(),
						self.theme.text(true, false),
					);
					let author_span = Span::styled(
						format!("  <{}>", item.pubkey_short()),
						self.theme.text(false, false),
					);
					ListItem::new(Line::from(vec![
						kind_span,
						status_span,
						subject_span,
						author_span,
					]))
				})
				.collect();

			let list = List::new(items)
				.block(
					Block::default()
						.borders(Borders::ALL)
						.border_type(BorderType::Plain)
						.title(" NIP-34: Patches & Issues "),
				)
				.highlight_style(
					Style::default().add_modifier(Modifier::REVERSED),
				);

			f.render_stateful_widget(list, area, &mut *self.list_state.borrow_mut());
		}
		#[cfg(not(feature = "nostr"))]
		{
			let p = Paragraph::new("nostr feature not enabled")
				.block(Block::default().borders(Borders::ALL));
			f.render_widget(p, area);
		}
	}

	fn draw_detail<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		#[cfg(feature = "nostr")]
		let body = self
			.items
			.get(self.selected)
			.map(|item| item.content().to_owned())
			.unwrap_or_default();
		#[cfg(not(feature = "nostr"))]
		let body = String::new();

		let p = Paragraph::new(body)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.title(" Detail "),
			)
			.wrap(Wrap { trim: false });
		f.render_widget(p, area);
	}

	fn draw_footer<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		let count = self.item_count();
		let msg = format!(
			" {} │ {} items ",
			self.status_msg, count
		);
		let p = Paragraph::new(msg)
			.style(self.theme.text(false, false));
		f.render_widget(p, area);
	}
}

impl DrawableComponent for NostrTab {
	fn draw<B: Backend>(
		&self,
		f: &mut Frame<B>,
		rect: Rect,
	) -> Result<()> {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Min(5),
				Constraint::Ratio(1, 3),
				Constraint::Length(1),
			])
			.split(rect);

		self.draw_list(f, chunks[0]);
		self.draw_detail(f, chunks[1]);
		self.draw_footer(f, chunks[2]);
		Ok(())
	}
}

impl Component for NostrTab {
	fn commands(
		&self,
		_out: &mut Vec<CommandInfo>,
		_force_all: bool,
	) -> CommandBlocking {
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
				} else if key_match(
					key,
					self.key_config.keys.move_down,
				) {
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
		if self.item_count() > 0 {
			self.list_state.borrow_mut().select(Some(self.selected));
		}
		Ok(())
	}
}
