use crate::weeble::{weeble_async, weeble_sync};
use crate::wobble_sync;
use anyhow::Result;
use chrono::{DateTime, Local};
use gnostr_asyncgit::sync::{
    self, checkout_commit, BranchDetails, BranchInfo, CommitId, RepoPathRef, Tags,
};
use indexmap::IndexSet;
use itertools::Itertools;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::env;
use std::{borrow::Cow, cell::Cell, cmp, collections::BTreeMap, rc::Rc, time::Instant};

use super::utils::logitems::{ItemBatch, LogEntry};
use crate::utils::truncate_chars;
use crate::{
    app::Environment,
    components::{
        utils::string_width_align, CommandBlocking, CommandInfo, Component, DrawableComponent,
        EventState, ScrollType,
    },
    keys::{key_match, SharedKeyConfig},
    queue::{InternalEvent, Queue},
    strings::{self, symbol},
    try_or_popup,
    ui::{
        calc_scroll_top,
        style::{SharedTheme, Theme},
    },
};

const ELEMENTS_PER_LINE: usize = 9;
const SLICE_SIZE: usize = 1200;

///
pub struct TopicList {
    repo: RepoPathRef,
    title: Box<str>,
    selection: usize,
    highlighted_selection: Option<usize>,
    items: ItemBatch,
    highlights: Option<Rc<IndexSet<CommitId>>>,
    commits: IndexSet<CommitId>,
    marked: Vec<(usize, CommitId)>,
    scroll_state: (Instant, f32),
    tags: Option<Tags>,
    local_branches: BTreeMap<CommitId, Vec<BranchInfo>>,
    remote_branches: BTreeMap<CommitId, Vec<BranchInfo>>,
    current_size: Cell<Option<(u16, u16)>>,
    scroll_top: Cell<usize>,
    theme: SharedTheme,
    queue: Queue,
    key_config: SharedKeyConfig,
}

impl TopicList {
    ///
    /// methods
    /// `copy_items`
    ///
    /// `clear_marked`
    ///
    /// `marked_commits`
    ///
    /// `set_commits`
    /// are never used
    pub fn new(env: &Environment, title: &str) -> Self {
        Self {
            repo: env.repo.clone(),
            items: ItemBatch::default(),
            marked: Vec::with_capacity(2),
            selection: 0,
            highlighted_selection: None,
            commits: IndexSet::new(),
            highlights: None,
            scroll_state: (Instant::now(), 0_f32),
            tags: None,
            local_branches: BTreeMap::default(),
            remote_branches: BTreeMap::default(),
            current_size: Cell::new(None),
            scroll_top: Cell::new(0),
            theme: env.theme.clone(),
            queue: env.queue.clone(),
            key_config: env.key_config.clone(),
            title: title.into(),
        }
    }

    ///
    pub const fn tags(&self) -> Option<&Tags> {
        self.tags.as_ref()
    }

    ///
    pub fn clear(&mut self) {
        self.items.clear();
        self.commits.clear();
    }

    ///
    pub fn copy_items(&self) -> Vec<CommitId> {
        self.commits.iter().copied().collect_vec()
    }

    ///
    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = Some(tags);
    }

    ///
    pub fn selected_entry(&self) -> Option<&LogEntry> {
        self.items
            .iter()
            .nth(self.selection.saturating_sub(self.items.index_offset()))
    }

    ///
    pub fn marked_count(&self) -> usize {
        self.marked.len()
    }

    ///
    pub fn marked(&self) -> &[(usize, CommitId)] {
        &self.marked
    }

    ///
    pub fn clear_marked(&mut self) {
        self.marked.clear();
    }

    ///
    pub fn marked_commits(&self) -> Vec<CommitId> {
        let (_, commits): (Vec<_>, Vec<CommitId>) = self.marked.iter().copied().unzip();
        commits
    }

    ///
    pub fn copy_commit_hash(&self) -> Result<()> {
        let marked = self.marked.as_slice();
        let yank: Option<String> = match marked {
            [] => self
                .items
                .iter()
                .nth(self.selection.saturating_sub(self.items.index_offset()))
                .map(|e| e.id.to_string()),
            [(_idx, commit)] => Some(commit.to_string()),
            [first, .., last] => {
                let marked_consecutive = marked.windows(2).all(|w| w[0].0 + 1 == w[1].0);

                let yank = if marked_consecutive {
                    format!("{}^..{}", first.1, last.1)
                } else {
                    marked
                        .iter()
                        .map(|(_idx, commit)| commit.to_string())
                        .join(" ")
                };
                Some(yank)
            }
        };

        if let Some(yank) = yank {
            crate::clipboard::copy_string(&yank)?;
            self.queue
                .push(InternalEvent::ShowInfoMsg(strings::copy_success(&yank)));
        }
        Ok(())
    }

    ///
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn checkout(&mut self) {
        if let Some(commit_hash) = self.selected_entry().map(|entry| entry.id) {
            try_or_popup!(
                self,
                "failed to checkout commit:",
                //checkout_commit
                checkout_commit(&self.repo.borrow(), commit_hash)
            );
        }
    }
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn comment(&mut self) {
        if let Some(commit_hash) = self.selected_entry().map(|entry| entry.id) {
            try_or_popup!(
                self,
                "failed to checkout commit:",
                //checkout_commit
                checkout_commit(&self.repo.borrow(), commit_hash)
            );
        }
    }

    ///
    pub fn set_local_branches(&mut self, local_branches: Vec<BranchInfo>) {
        self.local_branches.clear();

        for local_branch in local_branches {
            self.local_branches
                .entry(local_branch.top_commit)
                .or_default()
                .push(local_branch);
        }
    }

    ///
    pub fn set_remote_branches(&mut self, remote_branches: Vec<BranchInfo>) {
        self.remote_branches.clear();

        for remote_branch in remote_branches {
            self.remote_branches
                .entry(remote_branch.top_commit)
                .or_default()
                .push(remote_branch);
        }
    }

    ///
    pub fn set_commits(&mut self, commits: IndexSet<CommitId>) {
        // methods
        // `copy_items`
        //
        // `clear_marked`
        //
        // `marked_commits`
        //
        // `set_commits`
        // are never used
        if commits != self.commits {
            self.items.clear();
            self.commits = commits;
            self.fetch_commits(false);
        }
    }

    ///
    pub fn refresh_extend_data(&mut self, commits: Vec<CommitId>) {
        let new_commits = !commits.is_empty();
        self.commits.extend(commits);

        let selection = self.selection();
        let selection_max = self.selection_max();

        if self.needs_data(selection, selection_max) || new_commits {
            self.fetch_commits(false);
        }
    }

    ///
    pub fn set_highlighting(&mut self, highlighting: Option<Rc<IndexSet<CommitId>>>) {
        //note: set highlights to none if there is no highlight
        self.highlights = if highlighting.as_ref().is_some_and(|set| set.is_empty()) {
            None
        } else {
            highlighting
        };

        self.select_next_highlight();
        self.set_highlighted_selection_index();
        self.fetch_commits(true);
    }

    ///
    pub fn select_commit(&mut self, id: CommitId) -> Result<()> {
        let index = self.commits.get_index_of(&id);

        if let Some(index) = index {
            self.selection = index;
            self.set_highlighted_selection_index();
            Ok(())
        } else {
            anyhow::bail!(
				"Could not select commit. It might not be loaded yet or it might be on a different branch."
			);
        }
    }

    ///
    pub fn highlighted_selection_info(&self) -> (usize, usize) {
        let amount = self
            .highlights
            .as_ref()
            .map(|highlights| highlights.len())
            .unwrap_or_default();
        (self.highlighted_selection.unwrap_or_default(), amount)
    }

    fn set_highlighted_selection_index(&mut self) {
        self.highlighted_selection = self.highlights.as_ref().and_then(|highlights| {
            highlights
                .iter()
                .position(|entry| entry == &self.commits[self.selection])
        });
    }

    const fn selection(&self) -> usize {
        self.selection
    }

    /// will return view size or None before the first render
    fn current_size(&self) -> Option<(u16, u16)> {
        self.current_size.get()
    }

    fn selection_max(&self) -> usize {
        self.commits.len().saturating_sub(1)
    }

    fn selected_entry_marked(&self) -> bool {
        self.selected_entry()
            .and_then(|e| self.is_marked(&e.id))
            .unwrap_or_default()
    }

    fn move_selection(&mut self, scroll: ScrollType) -> Result<bool> {
        let needs_update = if self.items.highlighting() {
            self.move_selection_highlighting(scroll)?
        } else {
            self.move_selection_normal(scroll)?
        };

        Ok(needs_update)
    }

    fn move_selection_highlighting(&mut self, scroll: ScrollType) -> Result<bool> {
        let (current_index, selection_max) = self.highlighted_selection_info();

        let new_index = match scroll {
            ScrollType::Up => current_index.saturating_sub(1),
            ScrollType::Down => current_index.saturating_add(1),

            //TODO: support this?
            // ScrollType::Home => 0,
            // ScrollType::End => self.selection_max(),
            _ => return Ok(false),
        };

        let new_index = cmp::min(new_index, selection_max.saturating_sub(1));

        let index_changed = new_index != current_index;

        if !index_changed {
            return Ok(false);
        }

        let new_selected_commit = self
            .highlights
            .as_ref()
            .and_then(|highlights| highlights.iter().nth(new_index).copied());

        if let Some(c) = new_selected_commit {
            self.select_commit(c)?;
            return Ok(true);
        }

        Ok(false)
    }

    fn move_selection_normal(&mut self, scroll: ScrollType) -> Result<bool> {
        self.update_scroll_speed();

        #[allow(clippy::cast_possible_truncation)]
        let speed_int = usize::try_from(self.scroll_state.1 as i64)?.max(1);

        let page_offset =
            usize::from(self.current_size.get().unwrap_or_default().1).saturating_sub(1);

        let new_selection = match scroll {
            ScrollType::Up => self.selection.saturating_sub(speed_int),
            ScrollType::Down => self.selection.saturating_add(speed_int),
            ScrollType::PageUp => self.selection.saturating_sub(page_offset),
            ScrollType::PageDown => self.selection.saturating_add(page_offset),
            ScrollType::Home => 0,
            ScrollType::End => self.selection_max(),
        };

        let new_selection = cmp::min(new_selection, self.selection_max());
        let needs_update = new_selection != self.selection;

        self.selection = new_selection;

        Ok(needs_update)
    }

    // mark
    fn mark(&mut self) {
        if let Some(e) = self.selected_entry() {
            let id = e.id;
            let selected = self.selection.saturating_sub(self.items.index_offset());
            if self.is_marked(&id).unwrap_or_default() {
                self.marked.retain(|marked| marked.1 != id);
            } else {
                self.marked.push((selected, id));

                self.marked
                    .sort_unstable_by(|first, second| first.0.cmp(&second.0));
            }
        }
    }

    fn update_scroll_speed(&mut self) {
        const REPEATED_SCROLL_THRESHOLD_MILLIS: u128 = 300;
        const SCROLL_SPEED_START: f32 = 0.1_f32;
        const SCROLL_SPEED_MAX: f32 = 10_f32;
        const SCROLL_SPEED_MULTIPLIER: f32 = 1.05_f32;

        let now = Instant::now();

        let since_last_scroll = now.duration_since(self.scroll_state.0);

        self.scroll_state.0 = now;

        let speed = if since_last_scroll.as_millis() < REPEATED_SCROLL_THRESHOLD_MILLIS {
            self.scroll_state.1 * SCROLL_SPEED_MULTIPLIER
        } else {
            SCROLL_SPEED_START
        };

        self.scroll_state.1 = speed.min(SCROLL_SPEED_MAX);
    }

    fn is_marked(&self, id: &CommitId) -> Option<bool> {
        if self.marked.is_empty() {
            None
        } else {
            let found = self.marked.iter().any(|entry| entry.1 == *id);
            Some(found)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn get_entry_to_add<'a>(
        &self,
        e: &'a LogEntry,
        selected: bool,
        tags: Option<String>,
        local_branches: Option<String>,
        remote_branches: Option<String>,
        theme: &Theme,
        width: usize, //width
        now: DateTime<Local>,
        marked: Option<bool>,
    ) -> Line<'a> {
        //
        let mut txt: Vec<Span> =
            Vec::with_capacity(ELEMENTS_PER_LINE + if marked.is_some() { 2 } else { 2 });

        let normal = !self.items.highlighting() || (self.items.highlighting() && e.highlighted);

        let splitter_txt = Cow::from(symbol::EMPTY_SPACE);
        let splitter = Span::styled(
            splitter_txt,
            if normal {
                theme.text(true, selected)
            } else {
                Style::default()
            },
        );

        // marker
        if let Some(marked) = marked {
            txt.push(Span::styled(
                Cow::from(if marked {
                    symbol::CIRCLED_G_STR //offset in home component
                } else {
                    symbol::EMPTY_SPACE
                }),
                theme.log_marker(selected),
            ));
        } else {
            txt.push(Span::styled(
                Cow::from(symbol::EMPTY_SPACE),
                theme.log_marker(selected),
            ));
        }
        txt.push(splitter.clone());

        let style_hash = if normal {
            theme.commit_hash(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_time = if normal {
            theme.commit_time(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_author = if normal {
            theme.commit_author(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_tags = if normal {
            theme.tags(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_branches = if normal {
            theme.branch(selected, true)
        } else {
            theme.commit_unhighlighted()
        };
        let style_msg = if normal {
            theme.text(true, selected)
        } else {
            theme.commit_unhighlighted()
        };

        // commit hash
        //txt.push(Span::styled(Cow::from(&*e.hash_padded), style_hash));
        //txt.push(Span::styled(Cow::from(&*e.keys), style_hash));
        //txt.push(splitter.clone());
        txt.push(Span::styled(
            format!("{} ", &truncate_chars(&e.keys, 64_usize)),
            style_hash,
        ));
        txt.push(splitter.clone());
        //txt.push(Span::styled(Cow::from(&*e.hash_short), style_hash));
        //txt.push(splitter.clone());

        let author_width = (width.saturating_sub(0) / 3).clamp(3, 20);
        let message_width = width.saturating_sub(txt.iter().map(|span| span.content.len()).sum());

        //// commit msg
        //// commit msg
        //// commit msg
        //txt.push(splitter.clone());
        txt.push(Span::styled(
            format!("{:message_width$}", &e.msg),
            style_author,
        ));

        let author = string_width_align(&e.author, author_width);
        // commit author
        //txt.push(Span::styled(author, style_author));

        // commit tags
        if let Some(tags) = tags {
            //txt.push(Span::styled(tags, style_tags));
        }

        if let Some(local_branches) = local_branches {
            //txt.push(Span::styled(local_branches, style_branches));
        }
        //git-remote-nostr helper
        if let Some(remote_branches) = remote_branches {
            txt.push(Span::styled(remote_branches, style_branches));
        }

        Line::from(txt)
    }
    #[allow(clippy::too_many_arguments)]
    fn get_detail_to_add<'a>(
        &self,
        e: &'a LogEntry,
        selected: bool,
        tags: Option<String>,
        local_branches: Option<String>,
        remote_branches: Option<String>,
        theme: &Theme,
        width: usize, //width
        now: DateTime<Local>,
        marked: Option<bool>,
    ) -> Line<'a> {
        //
        let mut txt: Vec<Span> =
            Vec::with_capacity(ELEMENTS_PER_LINE + if marked.is_some() { 2 } else { 2 });

        let normal = !self.items.highlighting() || (self.items.highlighting() && e.highlighted);

        let splitter_txt = Cow::from(symbol::EMPTY_SPACE);
        let splitter = Span::styled(
            splitter_txt,
            if normal {
                theme.text(true, false)
                //Style::default()
            } else {
                theme.text(true, false)
                //Style::default()
            },
        );

        // marker
        if let Some(marked) = marked {
            //txt.push(Span::styled(
            //    Cow::from(if marked {
            //        //symbol::CIRCLED_G_STR //offset in home component
            //        symbol::EMPTY_SPACE
            //    } else {
            //        symbol::EMPTY_SPACE
            //    }),
            //    theme.log_marker(selected),
            //));
        } else {
            //txt.push(Span::styled(
            //    Cow::from(symbol::EMPTY_SPACE),
            //    theme.log_marker(selected),
            //));
        }
        txt.push(splitter.clone());

        let style_hash = if normal {
            theme.commit_hash(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_time = if normal {
            theme.commit_time(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_author = if normal {
            theme.commit_author(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_tags = if normal {
            theme.tags(selected)
        } else {
            theme.commit_unhighlighted()
        };
        let style_branches = if normal {
            theme.branch(selected, true)
        } else {
            theme.commit_unhighlighted()
        };
        let style_msg = if normal {
            theme.text(true, selected)
        } else {
            theme.commit_unhighlighted()
        };

        // commit hash
        //txt.push(Span::styled(Cow::from(&*""), style_hash));

        //txt.push(Span::styled(Cow::from(&*e.hash_padded), style_hash));
        //txt.push(Span::styled(Cow::from(&*e.keys), style_hash));
        //txt.push(splitter.clone());
        //txt.push(Span::styled(
        //    format!("{} ", &truncate_chars(&e.keys, 64 as usize)),
        //    style_hash,
        //));
        txt.push(splitter.clone());
        //txt.push(Span::styled(Cow::from(&*e.hash_short), style_hash));
        //txt.push(splitter.clone());

        let author_width = (width.saturating_sub(0) / 3).clamp(3, 20);
        let message_width = width.saturating_sub(txt.iter().map(|span| span.content.len()).sum());

        //// commit msg
        //// commit msg
        //// commit msg
        //txt.push(splitter.clone());

        //txt.push(Span::styled(
        //    format!("{:message_width$}", &e.msg),
        //    style_author,
        //));

        //let author = string_width_align(&e.author, author_width);
        // commit author
        //txt.push(Span::styled(author, style_author));

        // commit tags
        if let Some(tags) = tags {
            txt.push(Span::styled(tags, Style::default()));
            txt.push(splitter.clone());
        }

        if let Some(local_branches) = local_branches {
            //txt.push(Span::styled(local_branches, style_branches));
            txt.push(splitter.clone());
        }
        //git-remote-nostr helper
        if let Some(remote_branches) = remote_branches {
            txt.push(Span::styled(remote_branches, style_branches));
            txt.push(splitter.clone());
        }

        Line::from(txt)
    }

    fn get_detail_text(&self, height: usize, width: usize) -> Vec<Line> {
        let selection = self.relative_selection();
        let mut txt: Vec<Line> = Vec::with_capacity(height);
        let now = Local::now();
        let any_marked = !self.marked.is_empty();
        for (idx, e) in self
            .items
            .iter()
            .skip(self.scroll_top.get())
            .take(height)
            .enumerate()
        {
            let tags = self
                .tags
                .as_ref()
                .and_then(|t| t.get(&e.id))
                .map(|tags| tags.iter().map(|t| format!("<{}>", t.name)).join(" "));

            let local_branches = self.local_branches.get(&e.id).map(|local_branch| {
                local_branch
                    .iter()
                    .map(|local_branch| format!("{{{0}}}", local_branch.name))
                    .join(" ")
            });

            let marked = if any_marked {
                self.is_marked(&e.id)
            } else {
                None
            };

            //txt.push("topiclist:695:text".into());
            txt.push(
                format!(
                    "{}/{}/{}",
                    env::var("WEEBLE").unwrap(),
                    env::var("BLOCKHEIGHT").unwrap(),
                    env::var("WOBBLE").unwrap()
                )
                .into(), //wobble_sync().unwrap()).into()
            );
            //get_detail_to_add
            txt.push(self.get_detail_to_add(
                e,
                idx + self.scroll_top.get() == selection,
                tags,
                local_branches,
                self.remote_branches_string(e),
                &self.theme,
                width - 6_usize,
                now,
                marked,
            ));
            txt.push("topiclist:708:text".into());
        }

        txt
    }
    fn get_topic_text(&self, height: usize, width: usize) -> Vec<Line> {
        let selection = self.relative_selection();
        let mut txt: Vec<Line> = Vec::with_capacity(height);
        let now = Local::now();
        let any_marked = !self.marked.is_empty();
        for (idx, e) in self
            .items
            .iter()
            .skip(self.scroll_top.get())
            .take(height)
            .enumerate()
        {
            let tags = self
                .tags
                .as_ref()
                .and_then(|t| t.get(&e.id))
                .map(|tags| tags.iter().map(|t| format!("<{}>", t.name)).join(" "));

            let local_branches = self.local_branches.get(&e.id).map(|local_branch| {
                local_branch
                    .iter()
                    .map(|local_branch| format!("{{{0}}}", local_branch.name))
                    .join(" ")
            });

            let marked = if any_marked {
                self.is_marked(&e.id)
            } else {
                None
            };

            //get_entry_to_add
            txt.push(self.get_entry_to_add(
                e,
                idx + self.scroll_top.get() == selection,
                tags,
                local_branches,
                self.remote_branches_string(e),
                &self.theme,
                width - 6_usize,
                now,
                marked,
            ));
        }

        txt
    }

    fn remote_branches_string(&self, e: &LogEntry) -> Option<String> {
        self.remote_branches.get(&e.id).and_then(|remote_branches| {
            let filtered_branches: Vec<_> = remote_branches
                .iter()
                .filter(|remote_branch| {
                    self.local_branches.get(&e.id).is_none_or(|local_branch| {
                        local_branch.iter().any(|local_branch| {
                            let has_corresponding_local_branch = match &local_branch.details {
                                BranchDetails::Local(details) => {
                                    details.upstream.as_ref().is_some_and(|upstream| {
                                        upstream.reference == remote_branch.reference
                                    })
                                }
                                BranchDetails::Remote(_) => false,
                            };

                            !has_corresponding_local_branch
                        })
                    })
                })
                .map(|remote_branch| format!("[{0}]", remote_branch.name))
                .collect();

            if filtered_branches.is_empty() {
                None
            } else {
                Some(filtered_branches.join(" "))
            }
        })
    }

    fn relative_selection(&self) -> usize {
        self.selection.saturating_sub(self.items.index_offset())
    }

    fn select_next_highlight(&mut self) {
        if self.highlights.is_none() {
            return;
        }

        let old_selection = self.selection;

        let mut offset = 0;
        loop {
            let hit_upper_bound = old_selection + offset > self.selection_max();
            let hit_lower_bound = offset > old_selection;

            if !hit_upper_bound {
                self.selection = old_selection + offset;

                if self.selection_highlighted() {
                    break;
                }
            }

            if !hit_lower_bound {
                self.selection = old_selection - offset;

                if self.selection_highlighted() {
                    break;
                }
            }

            if hit_lower_bound && hit_upper_bound {
                self.selection = old_selection;
                break;
            }

            offset += 1;
        }
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    fn selection_highlighted(&mut self) -> bool {
        let commit = self.commits[self.selection];

        self.highlights
            .as_ref()
            .is_some_and(|highlights| highlights.contains(&commit))
    }

    fn needs_data(&self, idx: usize, idx_max: usize) -> bool {
        self.items.needs_data(idx, idx_max)
    }

    // checks if first entry in items is the same commit as we expect
    fn is_list_in_sync(&self) -> bool {
        self.items
            .index_offset_raw()
            .and_then(|index| {
                self.items
                    .iter()
                    .next()
                    .map(|item| item.id == self.commits[index])
            })
            .unwrap_or_default()
    }

    fn fetch_commits(&mut self, force: bool) {
        let want_min = self.selection().saturating_sub(SLICE_SIZE / 2);
        let commits = self.commits.len();

        let want_min = want_min.min(commits);

        let index_in_sync = self
            .items
            .index_offset_raw()
            .is_some_and(|index| want_min == index);

        if !index_in_sync || !self.is_list_in_sync() || force {
            let commits = sync::get_commits_info(
                &self.repo.borrow(),
                self.commits
                    .iter()
                    .skip(want_min)
                    .take(SLICE_SIZE)
                    .copied()
                    .collect_vec()
                    .as_slice(),
                self.current_size().map_or(100u16, |size| size.0).into(),
            );

            if let Ok(commits) = commits {
                self.items.set_items(want_min, commits, &self.highlights);
            }
        }
    }
    fn get_chat_text(&self, height: usize, width: usize) -> Vec<Line> {
        let selection = self.relative_selection();
        let mut txt: Vec<Line> = Vec::with_capacity(height);
        let now = Local::now();
        let any_marked = !self.marked.is_empty();
        for (idx, e) in self
            .items
            .iter()
            .skip(self.scroll_top.get())
            .take(height)
            .enumerate()
        {
            let tags = self
                .tags
                .as_ref()
                .and_then(|t| t.get(&e.id))
                .map(|tags| tags.iter().map(|t| format!("<{}>", t.name)).join(" "));

            let local_branches = self.local_branches.get(&e.id).map(|local_branch| {
                local_branch
                    .iter()
                    .map(|local_branch| format!("{{{0}}}", local_branch.name))
                    .join(" ")
            });

            let marked = if any_marked {
                self.is_marked(&e.id)
            } else {
                None
            };

            //get_entry_to_add
            txt.push(self.get_entry_to_add(
                e,
                idx + self.scroll_top.get() == selection,
                tags,
                local_branches,
                self.remote_branches_string(e),
                &self.theme,
                width - 6_usize,
                now,
                marked,
            ));
        }

        txt
    }
}

impl DrawableComponent for TopicList {
    fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            //0                                //1 initial width of commit detail
            .constraints([Constraint::Min(70), Constraint::Percentage(0)].as_ref())
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),       //help and tools height
                    Constraint::Length(3),       //timer
                    Constraint::Percentage(100), //table
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),       //0 topic
                    Constraint::Length(2),       //1 squares
                    Constraint::Percentage(100), //2 tools view
                ]
                .as_ref(),
            )
            .split(chunks[1]);

        let current_size = (
            area.width.saturating_sub(2),
            area.height.saturating_sub(1) - right_chunks.get(1).unwrap().height - 2,
        );
        self.current_size.set(Some(current_size));

        let topic_height_in_lines = 1_usize; //current_size.1 as usize;
        let selection = self.relative_selection();

        self.scroll_top.set(calc_scroll_top(
            self.scroll_top.get(),
            topic_height_in_lines,
            selection,
        ));

        let title = format!(
            "topiclist.rs:984: {} {}/{} ",
            self.title,
            self.commits.len().saturating_sub(self.selection),
            self.commits.len(),
        );
        //let more_text = format!(
        //    " {} {}/{} ",
        //    self.title,
        //    self.commits.len().saturating_sub(self.selection),
        //    self.commits.len(),
        //);

        //render commit info in topiclist
        //
        f.render_widget(
            Paragraph::new(
                self.get_topic_text(topic_height_in_lines, (current_size.0 + 10) as usize),
            )
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .title(Span::styled(
                        format!(
                            "self.get_topic_text:pubkey--->{:>}<---",
                            //"{}",
                            title.as_str().to_owned(),
                            //more_text.as_str()
                        ),
                        self.theme.title(true),
                    ))
                    .border_style(self.theme.block(false)),
            )
            .alignment(Alignment::Left),
            left_chunks[0],
        );
        //TODO nip-0034 git stuff display
        f.render_widget(
            Paragraph::new(self.get_detail_text(
                10_usize * topic_height_in_lines,
                (current_size.0 - 6) as usize,
            ))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
                    //.borders(Borders::ALL)
                    .title(Span::styled(
                        format!(
                            "1032:more_detail--->{:>}<---",
                            //"{}",
                            title.as_str().to_owned(),
                            //more_text.as_str()
                        ),
                        self.theme.title(true),
                    ))
                    .border_style(self.theme.block(false)),
            )
            .alignment(Alignment::Left),
            left_chunks[1],
        );
        //TODO p2p/nostr chat box
        f.render_widget(
            Paragraph::new(
                self.get_chat_text(current_size.0 as usize, (current_size.0 + 10) as usize),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        format!(
                            "1052:self.get_chat_text:chat widget--->{:>}",
                            title.as_str().to_owned(),
                            //more_text.as_str()
                        ),
                        self.theme.title(true),
                    ))
                    .border_style(self.theme.block(true)),
            )
            .alignment(Alignment::Left),
            left_chunks[2], //constrain to half width
        );

        //draw_scrollbar(
        //    f,
        //    area,
        //    &self.theme,
        //    self.commits.len(),
        //    self.selection,
        //    Orientation::Vertical,
        //);
        //
        Ok(())
    }
}

impl Component for TopicList {
    fn event(&mut self, ev: &crossterm::event::Event) -> Result<EventState> {
        if let crossterm::event::Event::Key(k) = ev {
            let selection_changed = if key_match(k, self.key_config.keys.move_up) {
                //
                //
                //
                //
                self.move_selection(ScrollType::Up)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.move_down) {
                //
                //
                //
                //
                self.move_selection(ScrollType::Down)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.shift_up)
                || key_match(k, self.key_config.keys.home)
            {
                //
                //
                //
                //
                self.move_selection(ScrollType::Home)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.shift_down)
                || key_match(k, self.key_config.keys.end)
            {
                //
                //
                //
                //
                self.move_selection(ScrollType::End)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.page_up) {
                //
                //
                //
                //
                self.move_selection(ScrollType::PageUp)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.page_down) {
                //
                //
                //
                //
                self.move_selection(ScrollType::PageDown)?
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.log_mark_commit) {
                //
                //
                //
                //
                self.mark();
                true
                //
                //
                //
                //
            } else if key_match(k, self.key_config.keys.log_checkout_commit) {
                //
                self.checkout();
                true
            } else if key_match(k, self.key_config.keys.log_comment_commit) {
                //
                //
                //
                //
                self.comment();
                true
                //
                //
                //
                //
            } else {
                //
                //
                //
                //
                false
                //
                //
                //
                //
            };
            return Ok(selection_changed.into());
        }

        Ok(EventState::NotConsumed)
    }

    fn commands(&self, out: &mut Vec<CommandInfo>, _force_all: bool) -> CommandBlocking {
        out.push(CommandInfo::new(
            strings::commands::scroll(&self.key_config),
            self.selected_entry().is_some(),
            true,
        ));
        out.push(CommandInfo::new(
            strings::commands::commit_list_mark(&self.key_config, self.selected_entry_marked()),
            true,
            true,
        ));
        CommandBlocking::PassingOn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_width_align() {
        assert_eq!(string_width_align("123", 3), "123");
        assert_eq!(string_width_align("123", 2), "..");
        assert_eq!(string_width_align("123", 3), "123");
        assert_eq!(string_width_align("12345", 6), "12345 ");
        assert_eq!(string_width_align("1234556", 4), "12..");
    }

    #[test]
    fn test_string_width_align_unicode() {
        assert_eq!(string_width_align("äste", 3), "ä..");
        assert_eq!(string_width_align("wüsten äste", 10), "wüsten ä..");
        assert_eq!(
            string_width_align("Jon Grythe Stødle", 19),
            "Jon Grythe Stødle  "
        );
    }
}
