use crate::{
	components::{
		visibility_blocking, CommandBlocking, CommandInfo, CommandText,
		CommitDetailsComponent, CommitList, Component,
		DrawableComponent, EventState, FileTreeOpen,
		InspectCommitOpen,
	},
	keys::{key_match, SharedKeyConfig},
	queue::{InternalEvent, Queue, StackablePopupOpen},
	strings::{self, order},
	try_or_popup,
	ui::style::{SharedTheme, Theme},
};
use anyhow::Result;
use asyncgit::{
	asyncjob::AsyncSingleJob,
	sync::{
		self, filter_commit_by_search, CommitId, LogFilterSearch,
		LogFilterSearchOptions, RepoPathRef,
	},
	AsyncBranchesJob, AsyncCommitFilterJob, AsyncGitNotification,
	AsyncLog, AsyncTags, CommitFilesParams, FetchStatus,
	ProgressPercent,
};
use crossbeam_channel::Sender;
use crossterm::event::Event;
use indexmap::IndexSet;
use ratatui::{
	backend::Backend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	text::Span,
	widgets::{Block, Borders, Paragraph},
	Frame,
};
use std::{rc::Rc, time::Duration};
use sync::CommitTags;

struct LogSearchResult {
	#[allow(dead_code)]
	options: LogFilterSearchOptions,
	#[allow(dead_code)]
	duration: Duration,
}

//TODO: deserves its own component
#[allow(dead_code)]
enum LogSearch {
	Off,
	Searching(
		AsyncSingleJob<AsyncCommitFilterJob>,
		LogFilterSearchOptions,
		Option<ProgressPercent>,
	),
	Results(LogSearchResult),
}

///
pub struct Nostr {
    /// Currently selected index in the Nostr item list
    selected_idx: usize,
	/// List of Nostr items (patches, issues, announcements)
	 nostr_items: Vec<crate::components::nostr_types::IndexedNostrItem>,
    
	pub status_msg: String,
	repo: RepoPathRef,
	commit_details: CommitDetailsComponent,
	list: CommitList,
	git_log: AsyncLog,
	search: LogSearch,
	git_tags: AsyncTags,
	git_local_branches: AsyncSingleJob<AsyncBranchesJob>,
	git_remote_branches: AsyncSingleJob<AsyncBranchesJob>,
	// Nostr async client integration
	#[allow(dead_code)]
	nostr_client: Option<asyncgit::nostr::AsyncNostr>,
	nostr_rx: Option<crossbeam_channel::Receiver<asyncgit::nostr::AsyncNostrNotification>>,
	queue: Queue,
	visible: bool,
	key_config: SharedKeyConfig,
	#[allow(dead_code)]
	sender: Sender<AsyncGitNotification>,
	theme: SharedTheme,
}

use asyncgit::nostr::PatchStatus;

impl Nostr {
	pub fn set_items(
		&mut self,
		_items: Vec<crate::components::nostr_types::NostrItem>,
	) {
	}

	pub fn push_patch(
		&mut self,
		patch: crate::components::nostr_types::NostrItem,
	) {
		let idx = self.nostr_items.len();
self.nostr_items.push(crate::components::nostr_types::IndexedNostrItem { idx, item: patch });
		self.sort_items();
	}
	pub fn push_issue(
		&mut self,
		issue: crate::components::nostr_types::NostrItem,
	) {
		let idx = self.nostr_items.len();
self.nostr_items.push(crate::components::nostr_types::IndexedNostrItem { idx, item: issue });
		self.sort_items();
	}
	pub fn push_announcement(
		&mut self,
		ann: crate::components::nostr_types::NostrItem,
	) {
		let idx = self.nostr_items.len();
self.nostr_items.push(crate::components::nostr_types::IndexedNostrItem { idx, item: ann });
		self.sort_items();
	}
	pub fn apply_status(
		&mut self,
		_target_id: &str,
		_status: PatchStatus,
	) {
	}

	fn sort_items(&mut self) {
		self.nostr_items.sort_by(|a, b| {
			b.item.created_at().cmp(&a.item.created_at())
		});
		// Update indices after sort
		for (idx, item) in self.nostr_items.iter_mut().enumerate() {
			item.idx = idx;
		}
	}

	fn item_count(&self) -> usize {
		self.nostr_items.len()
	}

	fn move_selection_up(&mut self) {
		if self.selected_idx > 0 {
			self.selected_idx -= 1;
		}
	}

	fn move_selection_down(&mut self) {
		if self.item_count() > 0 && self.selected_idx < self.item_count() - 1 {
			self.selected_idx += 1;
		}
	}

	fn draw_list<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		use ratatui::text::Line;
		use ratatui::style::{Modifier, Style};
		use ratatui::widgets::{List, ListItem, ListState};

		let items: Vec<ListItem> = self
			.nostr_items
			.iter()
			.map(|indexed_item| {
				let item = &indexed_item.item;
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
					.title(" NIP-34: Patches & Issues ")
					.borders(Borders::ALL),
			)
			.highlight_style(
				Style::default().add_modifier(Modifier::REVERSED),
			);

		let mut state = ListState::default();
		if !self.nostr_items.is_empty() {
			let idx = self.selected_idx.min(self.nostr_items.len().saturating_sub(1));
			state.select(Some(idx));
		}
		f.render_stateful_widget(list, area, &mut state.clone());
	}

	fn draw_detail<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		use ratatui::widgets::{Paragraph, Wrap};

		let body = self
			.nostr_items
			.get(self.selected_idx)
			.map(|indexed_item| indexed_item.item.content().to_owned())
			.unwrap_or_default();

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
		use ratatui::widgets::Paragraph;

		let count = self.item_count();
		let msg = format!(
			" {} │ {} items ",
			self.status_msg, count
		);
		let p = Paragraph::new(msg)
			.style(self.theme.text(false, false));
		f.render_widget(p, area);
	}

	///
	pub fn new(
		// status_msg is new for Nostr compatibility
		// with app.rs usage
		repo: &RepoPathRef,
		queue: &Queue,
		sender: &Sender<AsyncGitNotification>,
		theme: SharedTheme,
		key_config: SharedKeyConfig,
	) -> Self {
		let (nostr_tx, nostr_rx) = crossbeam_channel::unbounded();
let nostr_client = Some(asyncgit::nostr::AsyncNostr::new(nostr_tx));
Self {
			repo: repo.clone(),
			queue: queue.clone(),
			commit_details: CommitDetailsComponent::new(
				repo,
				queue,
				sender,
				theme.clone(),
				key_config.clone(),
			),
			list: CommitList::new(
				repo.clone(),
				&strings::log_title(&key_config),
				theme.clone(),
				queue.clone(),
				key_config.clone(),
			),
			git_log: AsyncLog::new(
				repo.borrow().clone(),
				sender,
				None,
			),
			search: LogSearch::Off,
			git_tags: AsyncTags::new(repo.borrow().clone(), sender),
			git_local_branches: AsyncSingleJob::new(sender.clone()),
			git_remote_branches: AsyncSingleJob::new(sender.clone()),
			nostr_client,
			nostr_rx: Some(nostr_rx),
			nostr_items: Vec::new(),
            selected_idx: 0, // Start with first item selected
			visible: false,
			key_config,
			sender: sender.clone(),
			theme,
			status_msg: String::new(),
		}
	}

	///
	#[allow(dead_code)]
	pub fn any_work_pending(&self) -> bool {
		self.git_log.is_pending()
			|| self.is_search_pending()
			|| self.git_tags.is_pending()
			|| self.git_local_branches.is_pending()
			|| self.git_remote_branches.is_pending()
			|| self.commit_details.any_work_pending()
	}

	const fn is_search_pending(&self) -> bool {
		matches!(self.search, LogSearch::Searching(_, _, _))
	}

	///
	pub fn update(&mut self) -> Result<()> {
		use asyncgit::nostr::AsyncNostrNotification;
		if let Some(rx) = &self.nostr_rx {
			let mut notifications = Vec::new();
			while let Ok(notification) = rx.try_recv() {
				notifications.push(notification);
			}
			for notification in notifications {
				match notification {
					AsyncNostrNotification::RepoPatch(patch) => {
						self.push_patch(crate::components::nostr_types::NostrItem::Patch(*patch));
					}
					AsyncNostrNotification::RepoIssue(issue) => {
						self.push_issue(crate::components::nostr_types::NostrItem::Issue(*issue));
					}
					AsyncNostrNotification::RepoAnnouncement(ann) => {
						self.push_announcement(crate::components::nostr_types::NostrItem::Announcement(*ann));
					}
					AsyncNostrNotification::RepoStatus { target_id, status } => {
						self.apply_status(&target_id, status);
					}
					_ => {}
				}
			}
		}

		if self.is_visible() {
			if self.git_log.fetch()? == FetchStatus::Started {
				self.list.clear();
			}

			self.list
				.refresh_extend_data(self.git_log.extract_items()?);

			self.git_tags.request(Duration::from_secs(3), false)?;

			if self.commit_details.is_visible() {
				let commit = self.selected_commit();
				let tags = self.selected_commit_tags(&commit);

				self.commit_details.set_commits(
					commit.map(CommitFilesParams::from),
					&tags,
				)?;
			}
		}

		Ok(())
	}

	///
	#[allow(dead_code)]
	pub fn update_git(
		&mut self,
		ev: AsyncGitNotification,
	) -> Result<()> {
		if self.visible {
			match ev {
				AsyncGitNotification::CommitFiles
				| AsyncGitNotification::Log => self.update()?,
				AsyncGitNotification::CommitFilter => {
					self.update_search_state();
				}
				AsyncGitNotification::Tags => {
					if let Some(tags) = self.git_tags.last()? {
						self.list.set_tags(tags);
						self.update()?;
					}
				}
				AsyncGitNotification::Branches => {
					if let Some(local_branches) =
						self.git_local_branches.take_last()
					{
						if let Some(Ok(local_branches)) =
							local_branches.result()
						{
							self.list
								.set_local_branches(local_branches);
							self.update()?;
						}
					}

					if let Some(remote_branches) =
						self.git_remote_branches.take_last()
					{
						if let Some(Ok(remote_branches)) =
							remote_branches.result()
						{
							self.list
								.set_remote_branches(remote_branches);
							self.update()?;
						}
					}
				}
				_ => (),
			}
		}

		Ok(())
	}

	fn selected_commit(&self) -> Option<CommitId> {
		self.list.selected_entry().map(|e| e.id)
	}

	fn selected_commit_tags(
		&self,
		commit: &Option<CommitId>,
	) -> Option<CommitTags> {
		let tags = self.list.tags();

		commit.and_then(|commit| {
			tags.and_then(|tags| tags.get(&commit).cloned())
		})
	}

	///
	#[allow(dead_code)]
	pub fn select_commit(&mut self, id: CommitId) -> Result<()> {
		self.list.select_commit(id)
	}

	fn revert_commit(&self) -> Result<()> {
		if let Some(c) = self.selected_commit() {
			sync::revert_commit(&self.repo.borrow(), c)?;
			self.queue.push(InternalEvent::TabSwitchStatus);
		}

		Ok(())
	}

	fn inspect_commit(&self) {
		if let Some(commit_id) = self.selected_commit() {
			let tags = self.selected_commit_tags(&Some(commit_id));
			self.queue.push(InternalEvent::OpenPopup(
				StackablePopupOpen::InspectCommit(
					InspectCommitOpen::new_with_tags(commit_id, tags),
				),
			));
		}
	}

	#[allow(dead_code)]
	pub fn search(&mut self, options: LogFilterSearchOptions) {
		if !self.can_start_search() {
			return;
		}

		if matches!(
			self.search,
			LogSearch::Off | LogSearch::Results(_)
		) {
			log::info!("start search: {:?}", options);

			let filter = filter_commit_by_search(
				LogFilterSearch::new(options.clone()),
			);

			let mut job = AsyncSingleJob::new(self.sender.clone());
			job.spawn(AsyncCommitFilterJob::new(
				self.repo.borrow().clone(),
				self.list.copy_items(),
				filter,
			));

			self.search = LogSearch::Searching(job, options, None);

			self.list.set_highlighting(None);
		}
	}

	#[allow(dead_code)]
	fn update_search_state(&mut self) {
		match &mut self.search {
			LogSearch::Off | LogSearch::Results(_) => (),
			LogSearch::Searching(search, options, progress) => {
				if search.is_pending() {
					//update progress
					*progress = search.progress();
				} else if let Some(search) = search
					.take_last()
					.and_then(|search| search.result())
				{
					match search {
						Ok(search) => {
							self.list.set_highlighting(Some(
								Rc::new(
									search
										.result
										.into_iter()
										.collect::<IndexSet<_>>(),
								),
							));

							self.search =
								LogSearch::Results(LogSearchResult {
									options: options.clone(),
									duration: search.duration,
								});
						}
						Err(err) => {
							self.queue.push(
								InternalEvent::ShowErrorMsg(format!(
									"search error: {err}",
								)),
							);

							self.search = LogSearch::Off;
						}
					}
				}
			}
		}
	}

	fn is_in_search_mode(&self) -> bool {
		!matches!(self.search, LogSearch::Off)
	}

	#[allow(dead_code)]
	fn draw_search<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
		let (text, title) = match &self.search {
			LogSearch::Searching(_, options, progress) => (
				format!("'{}'", options.search_pattern.clone()),
				format!(
					"({}%)",
					progress
						.map(|progress| progress.progress)
						.unwrap_or_default()
				),
			),
			LogSearch::Results(results) => {
				let info = self.list.highlighted_selection_info();

				(
					format!(
						"'{}' (duration: {:?})",
						results.options.search_pattern.clone(),
						results.duration,
					),
					format!(
						"({}/{})",
						(info.0 + 1).min(info.1),
						info.1
					),
				)
			}
			LogSearch::Off => (String::new(), String::new()),
		};

		f.render_widget(
			Paragraph::new(text)
				.block(
					Block::default()
						.title(Span::styled(
							format!(
								"{} {}",
								strings::POPUP_TITLE_LOG_SEARCH,
								title
							),
							self.theme.title(true),
						))
						.borders(Borders::ALL)
						.border_style(Theme::attention_block()),
				)
				.alignment(Alignment::Left),
			area,
		);
	}

	fn can_leave_search(&self) -> bool {
		self.is_in_search_mode() && !self.is_search_pending()
	}

	fn can_start_search(&self) -> bool {
		!self.git_log.is_pending()
	}
}

impl DrawableComponent for Nostr {
	fn draw<B: Backend>(
		&self,
		f: &mut Frame<B>,
		area: Rect,
	) -> Result<()> {
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Min(5),
				Constraint::Ratio(1, 3),
				Constraint::Length(1),
			])
			.split(area);

		self.draw_list(f, chunks[0]);
		self.draw_detail(f, chunks[1]);
		self.draw_footer(f, chunks[2]);
		Ok(())
	}
}

impl Component for Nostr {
	//TODO: cleanup
	#[allow(clippy::too_many_lines)]
	fn event(&mut self, ev: &Event) -> Result<EventState> {
		if self.visible {
			let event_used = self.list.event(ev)?;

			if event_used.is_consumed() {
				self.update()?;
				return Ok(EventState::Consumed);
			} else if let Event::Key(k) = ev {
				// Nostr item navigation
				if key_match(k, self.key_config.keys.move_up) {
					self.move_selection_up();
					return Ok(EventState::Consumed);
				} else if key_match(k, self.key_config.keys.move_down) {
					self.move_selection_down();
					return Ok(EventState::Consumed);
				} else if key_match(k, self.key_config.keys.enter) {
					self.commit_details.toggle_visible()?;
					self.update()?;
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.exit_popup,
				) {
					if self.can_leave_search() {
						self.search = LogSearch::Off;
						self.list.set_highlighting(None);
						return Ok(EventState::Consumed);
					}
				} else if key_match(k, self.key_config.keys.copy) {
					try_or_popup!(
						self,
						strings::POPUP_FAIL_COPY,
						self.list.copy_commit_hash()
					);
					return Ok(EventState::Consumed);
				} else if key_match(k, self.key_config.keys.push) {
					self.queue.push(InternalEvent::PushTags);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.log_tag_commit,
				) {
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue
								.push(InternalEvent::TagCommit(id));
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(
					k,
					self.key_config.keys.move_right,
				) && self.commit_details.is_visible()
				{
					self.inspect_commit();
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.select_branch,
				) {
					self.queue.push(InternalEvent::SelectBranch);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.status_reset_item,
				) {
					try_or_popup!(
						self,
						"revert error:",
						self.revert_commit()
					);

					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.open_file_tree,
				) {
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::OpenPopup(
									StackablePopupOpen::FileTree(
										FileTreeOpen::new(id),
									),
								),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(k, self.key_config.keys.tags) {
					self.queue.push(InternalEvent::Tags);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.log_reset_comit,
				) {
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::OpenResetPopup(id),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(
					k,
					self.key_config.keys.log_reword_comit,
				) {
					return self.selected_commit().map_or(
						Ok(EventState::NotConsumed),
						|id| {
							self.queue.push(
								InternalEvent::RewordCommit(id),
							);
							Ok(EventState::Consumed)
						},
					);
				} else if key_match(k, self.key_config.keys.log_find)
					&& self.can_start_search()
				{
					self.queue
						.push(InternalEvent::OpenLogSearchPopup);
					return Ok(EventState::Consumed);
				} else if key_match(
					k,
					self.key_config.keys.compare_commits,
				) && self.list.marked_count() > 0
				{
					if self.list.marked_count() == 1 {
						// compare against head
						self.queue.push(InternalEvent::OpenPopup(
							StackablePopupOpen::CompareCommits(
								InspectCommitOpen::new(
									self.list.marked()[0].1,
								),
							),
						));
						return Ok(EventState::Consumed);
					} else if self.list.marked_count() == 2 {
						//compare two marked commits
						let marked = self.list.marked();
						self.queue.push(InternalEvent::OpenPopup(
							StackablePopupOpen::CompareCommits(
								InspectCommitOpen {
									commit_id: marked[0].1,
									compare_id: Some(marked[1].1),
									tags: None,
								},
							),
						));
						return Ok(EventState::Consumed);
					}
				} else if key_match(
					k,
					self.key_config.keys.nostr_submit_patch,
				) && self.list.marked_count() > 0
				{
					#[cfg(feature = "nostr")]
					{
						let ids: Vec<_> = self
							.list
							.marked()
							.iter()
							.map(|(_, id)| *id)
							.collect();
						self.queue.push(
							InternalEvent::NostrSubmitPatches(ids),
						);
					}
					return Ok(EventState::Consumed);
				}
			}
		}

		Ok(EventState::NotConsumed)
	}

	fn commands(
		&self,
		out: &mut Vec<CommandInfo>,
		force_all: bool,
	) -> CommandBlocking {
		if self.visible || force_all {
			self.list.commands(out, force_all);
		}

		// Nostr navigation: show index when items are available
		if self.visible && self.item_count() > 0 {
			let nav_text = format!("↑↓ Navigate ({}/{})",
				self.selected_idx + 1,
				self.item_count()
			);
			out.push(CommandInfo::new(
				CommandText {
					name: nav_text,
					desc: "Navigate Nostr items",
					group: "Navigation",
					hide_help: false,
				},
				true,
				true,
			));
		}

		out.push(
			CommandInfo::new(
				strings::commands::log_close_search(&self.key_config),
				true,
				(self.visible && self.can_leave_search())
					|| force_all,
			)
			.order(order::PRIORITY),
		);

		out.push(CommandInfo::new(
			strings::commands::log_details_toggle(&self.key_config),
			true,
			self.visible,
		));

		out.push(CommandInfo::new(
			strings::commands::commit_details_open(&self.key_config),
			true,
			(self.visible && self.commit_details.is_visible())
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::open_branch_select_popup(
				&self.key_config,
			),
			true,
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::compare_with_head(&self.key_config),
			self.list.marked_count() == 1,
			(self.visible && self.list.marked_count() <= 1)
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::compare_commits(&self.key_config),
			true,
			(self.visible && self.list.marked_count() == 2)
				|| force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::copy_hash(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::log_tag_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::log_checkout_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::open_tags_popup(&self.key_config),
			true,
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::push_tags(&self.key_config),
			true,
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::inspect_file_tree(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::revert_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));

		out.push(CommandInfo::new(
			strings::commands::log_reset_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));
		out.push(CommandInfo::new(
			strings::commands::log_reword_commit(&self.key_config),
			self.selected_commit().is_some(),
			self.visible || force_all,
		));
		out.push(CommandInfo::new(
			strings::commands::log_find_commit(&self.key_config),
			self.can_start_search(),
			self.visible || force_all,
		));

		#[cfg(feature = "nostr")]
		out.push(CommandInfo::new(
			strings::commands::nostr_submit_patch(&self.key_config),
			self.list.marked_count() > 0,
			(self.visible && self.list.marked_count() > 0)
				|| force_all,
		));

		visibility_blocking(self)
	}

	fn is_visible(&self) -> bool {
		self.visible
	}

	fn hide(&mut self) {
		self.visible = false;
		self.git_log.set_background();
	}

	fn show(&mut self) -> Result<()> {
		self.visible = true;

		self.git_local_branches.spawn(AsyncBranchesJob::new(
			self.repo.borrow().clone(),
			true,
		));

		self.git_remote_branches.spawn(AsyncBranchesJob::new(
			self.repo.borrow().clone(),
			false,
		));

		self.update()?;

		Ok(())
	}
}
