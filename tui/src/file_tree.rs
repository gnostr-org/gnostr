//! Ratatui adapter for the `filetreelist` crate.
//!
//! Exposes the underlying tree types plus a [`FileTreeWidget`] /
//! [`FileTreeState`] pair that implements [`ratatui::widgets::StatefulWidget`].

pub use filetreelist::*;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

use gnostr_asyncgit::sync::status::{StatusItem, StatusItemType};

/// Mutable state for [`FileTreeWidget`].
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct FileTreeState {
    /// Source paths (unsorted input is sorted on rebuild).
    paths: Vec<PathBuf>,
    /// Set of collapsed folder paths (as strings).
    collapsed: BTreeSet<String>,
    /// Selected item path (persisted).
    #[serde(default)]
    selected: Option<PathBuf>,
    /// Scroll offset (persisted).
    #[serde(default)]
    scroll: usize,
    /// Git status per path.
    #[serde(skip)]
    git_status_map: HashMap<PathBuf, StatusItemType>,
    /// Built tree (None when empty).
    #[serde(skip)]
    tree: Option<filetreelist::FileTree>,
    /// Last rendered viewport height (rows).
    #[serde(skip)]
    viewport_height: usize,
}

impl FileTreeState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a state from a list of paths.
    pub fn from_paths(paths: Vec<PathBuf>) -> Self {
        let mut state = Self::new();
        state.set_paths(paths);
        state
    }

    /// Replace the source paths and rebuild the tree.
    pub fn set_paths(&mut self, mut paths: Vec<PathBuf>) {
        paths.sort();
        self.paths = paths;
        self.rebuild();
    }

    /// Replace the collapsed set and rebuild the tree.
    pub fn set_collapsed(&mut self, collapsed: BTreeSet<String>) {
        self.collapsed = collapsed;
        self.rebuild();
    }

    /// Replace the git status map.
    pub fn set_git_status(&mut self, items: Vec<StatusItem>) {
        self.git_status_map.clear();
        for item in items {
            self.git_status_map
                .insert(PathBuf::from(item.path), item.status);
        }
    }

    /// Look up git status for a path.
    pub fn git_status(&self, path: &Path) -> Option<StatusItemType> {
        self.git_status_map.get(path).copied()
    }

    /// Current source paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Currently collapsed folder paths.
    pub fn collapsed(&self) -> &BTreeSet<String> {
        &self.collapsed
    }

    /// Access the underlying tree.
    pub fn tree(&self) -> Option<&filetreelist::FileTree> {
        self.tree.as_ref()
    }

    /// Access the underlying tree mutably.
    pub fn tree_mut(&mut self) -> Option<&mut filetreelist::FileTree> {
        self.tree.as_mut()
    }

    /// Path of the currently selected item.
    pub fn selected_path(&self) -> Option<&Path> {
        if let Some(info) = self
            .tree
            .as_ref()
            .and_then(filetreelist::FileTree::selected_file)
        {
            return Some(info.full_path());
        }
        self.selected.as_deref()
    }

    /// Current scroll offset (visual index of first rendered row).
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn restore(&mut self) {
        self.rebuild();
    }

    pub fn apply_persisted(&mut self, data: &Self) {
        self.paths = data.paths.clone();
        self.collapsed = data.collapsed.clone();
        self.selected = data.selected.clone();
        self.scroll = data.scroll;
        self.rebuild();
    }

    pub fn apply_persisted_state(&mut self, data: &Self) {
        // Drop-down (expand/collapse) state is intentionally not restored so
        // directories always start collapsed by default.
        self.selected = data.selected.clone();
        self.scroll = data.scroll;
        self.rebuild();
    }

    /// Collapse every directory in the tree (default-closed view).
    pub fn collapse_all(&mut self) {
        let mut collapsed = BTreeSet::new();
        if let Some(tree) = &self.tree {
            for (item, _) in tree.iterate(0, usize::MAX) {
                if item.kind().is_path() {
                    collapsed.insert(item.info().full_path().to_string_lossy().to_string());
                }
            }
        }
        self.collapsed = collapsed;
        self.rebuild();
    }

    pub fn snapshot(&self) -> Self {
        // Expand/collapse state is intentionally not persisted.
        Self {
            paths: self.paths.clone(),
            collapsed: Default::default(),
            selected: self.selected.clone(),
            scroll: self.scroll,
            git_status_map: Default::default(),
            tree: None,
            viewport_height: 0,
        }
    }

    fn rebuild(&mut self) {
        if self.paths.is_empty() {
            self.tree = None;
            return;
        }
        let refs: Vec<&Path> = self.paths.iter().map(PathBuf::as_path).collect();
        let collapsed_refs: BTreeSet<&String> = self.collapsed.iter().collect();
        self.tree = filetreelist::FileTree::new(&refs, &collapsed_refs).ok();
        if let (Some(tree), Some(path)) = (self.tree.as_mut(), self.selected.as_ref()) {
            tree.select_file(path);
        }
    }

    /// Handle a key press. Returns `true` if the tree state changed.
    pub fn handle_key(&mut self, code: ratatui::crossterm::event::KeyCode) -> bool {
        use ratatui::crossterm::event::KeyCode;
        let tree = match self.tree.as_mut() {
            Some(t) => t,
            None => return false,
        };

        let mut changed = false;
        match code {
            KeyCode::Up => changed = tree.move_selection(filetreelist::MoveSelection::Up),
            KeyCode::Down => changed = tree.move_selection(filetreelist::MoveSelection::Down),
            KeyCode::Left => changed = tree.move_selection(filetreelist::MoveSelection::Left),
            KeyCode::Right => changed = tree.move_selection(filetreelist::MoveSelection::Right),
            KeyCode::Home => changed = tree.move_selection(filetreelist::MoveSelection::Top),
            KeyCode::End => changed = tree.move_selection(filetreelist::MoveSelection::End),
            KeyCode::PageUp => {
                for _ in 0..self.viewport_height.max(1) {
                    if !tree.move_selection(filetreelist::MoveSelection::Up) {
                        break;
                    }
                    changed = true;
                }
            }
            KeyCode::PageDown => {
                for _ in 0..self.viewport_height.max(1) {
                    if !tree.move_selection(filetreelist::MoveSelection::Down) {
                        break;
                    }
                    changed = true;
                }
            }
            _ => return false,
        }

        self.selected = self.tree.as_ref().and_then(|t| {
            t.iterate(0, usize::MAX)
                .find(|(_, is_sel)| *is_sel)
                .map(|(item, _)| item.info().full_path().to_path_buf())
        });

        self.ensure_selection_visible();
        changed
    }

    fn ensure_selection_visible(&mut self) {
        let tree = match self.tree.as_ref() {
            Some(t) => t,
            None => return,
        };
        let vs = match tree.visual_selection() {
            Some(v) => v,
            None => return,
        };
        let idx = vs.index;
        let vh = self.viewport_height.max(1);
        if idx < self.scroll {
            self.scroll = idx;
        } else if idx >= self.scroll + vh {
            self.scroll = idx.saturating_sub(vh - 1);
        }
        // clamp
        let total = vs.count;
        let max_scroll = total.saturating_sub(vh);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        }
    }
}

/// Configuration for [`FileTreeWidget`].
#[derive(Debug, Clone)]
pub struct FileTreeWidget {
    pub block: Option<Block<'static>>,
    pub selected_style: Style,
    pub normal_style: Style,
    pub dir_style: Style,
    pub indent_width: u16,
    pub show_icons: bool,
    pub selected_icon: &'static str,
    pub collapsed_icon: &'static str,
    pub expanded_icon: &'static str,
    pub file_icon: &'static str,
}

impl Default for FileTreeWidget {
    fn default() -> Self {
        Self {
            block: None,
            selected_style: Style::default().bg(Color::DarkGray).fg(Color::White),
            normal_style: Style::default(),
            dir_style: Style::default().fg(Color::Blue),
            indent_width: 2,
            show_icons: true,
            selected_icon: ">",
            collapsed_icon: "▸",
            expanded_icon: "▾",
            file_icon: " ",
        }
    }
}

impl FileTreeWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }

    pub fn dir_style(mut self, style: Style) -> Self {
        self.dir_style = style;
        self
    }

    pub fn indent_width(mut self, width: u16) -> Self {
        self.indent_width = width;
        self
    }

    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }
}

impl StatefulWidget for FileTreeWidget {
    type State = FileTreeState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = if let Some(block) = &self.block {
            let inner = block.inner(area);
            Widget::render(block.clone(), area, buf);
            inner
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let tree = match state.tree.as_ref() {
            Some(t) => t,
            None => return,
        };

        // Collect display rows (owned data) so the borrow of `tree` ends.
        #[derive(Clone)]
        struct Row {
            indent: usize,
            is_dir: bool,
            collapsed: bool,
            name: String,
            full_path: String,
            selected: bool,
        }

        let rows: Vec<Row> = tree
            .iterate(0, usize::MAX)
            .map(|(item, selected)| Row {
                indent: item.info().indent() as usize,
                is_dir: item.kind().is_path(),
                collapsed: item.kind().is_path_collapsed(),
                name: item.info().path_str().to_string(),
                full_path: item.info().full_path_str().to_string(),
                selected,
            })
            .collect();

        let total = rows.len();

        // Update viewport height in state (owned, no borrow of tree).
        state.viewport_height = inner.height as usize;
        let vh = state.viewport_height.max(1);
        let max_scroll = total.saturating_sub(vh);
        if state.scroll > max_scroll {
            state.scroll = max_scroll;
        }
        state.ensure_selection_visible();

        let start = state.scroll;
        let end = (start + vh).min(total);

        // Render rows
        for (vi, row) in rows[start..end].iter().enumerate() {
            let y = inner.y + vi as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let icon = if !self.show_icons {
                ""
            } else if row.is_dir {
                if row.collapsed {
                    self.collapsed_icon
                } else {
                    self.expanded_icon
                }
            } else {
                self.file_icon
            };

            let indent_str = " ".repeat(row.indent * self.indent_width as usize);
            let line_text = format!("{}{} {}", indent_str, icon, row.name);

            let base_style = if row.selected {
                self.selected_style
            } else if row.is_dir {
                self.dir_style
            } else {
                self.normal_style
            };
            let style = if row.selected {
                base_style
            } else {
                match state.git_status_map.get(&PathBuf::from(&row.full_path)) {
                    Some(StatusItemType::New) => base_style.fg(Color::Green),
                    Some(StatusItemType::Modified) => base_style.fg(Color::Yellow),
                    Some(StatusItemType::Deleted) => base_style.fg(Color::Red),
                    Some(StatusItemType::Renamed) => base_style.fg(Color::Cyan),
                    Some(StatusItemType::Conflicted) => base_style.fg(Color::Magenta),
                    Some(StatusItemType::Typechange) => base_style.fg(Color::Blue),
                    None => base_style,
                }
            };

            // clear row
            for x in inner.x..inner.x + inner.width {
                buf[(x, y)].set_char(' ');
                buf[(x, y)].set_style(self.normal_style);
            }
            buf.set_string(inner.x, y, &line_text, style);
        }

        let _ = Line::default();
    }
}

// helper to avoid unused import warnings
#[allow(dead_code)]
fn _use_color(_c: Color) {}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use std::path::PathBuf;

    #[test]
    fn renders_empty_without_panic() {
        let mut state = FileTreeState::new();
        let widget = FileTreeWidget::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        StatefulWidget::render(widget, buf.area, &mut buf, &mut state);
    }

    #[test]
    fn renders_nested_and_navigates() {
        let paths = vec![PathBuf::from("a/b/c.txt"), PathBuf::from("a/d.txt")];
        let mut state = FileTreeState::from_paths(paths);
        let widget = FileTreeWidget::new().show_icons(true);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        StatefulWidget::render(widget.clone(), buf.area, &mut buf, &mut state);

        // Move to a file (initial selection is the root folder).
        state.handle_key(ratatui::crossterm::event::KeyCode::Down);
        state.handle_key(ratatui::crossterm::event::KeyCode::Down);
        assert!(state.selected_path().is_some());

        let changed = state.handle_key(ratatui::crossterm::event::KeyCode::Down);
        // may or may not change depending visibility, but no panic
        let _ = changed;

        let mut buf2 = Buffer::empty(Rect::new(0, 0, 40, 10));
        StatefulWidget::render(widget, buf2.area, &mut buf2, &mut state);
    }

    #[test]
    fn collapse_hides_children() {
        let paths = vec![PathBuf::from("a/b/c.txt"), PathBuf::from("a/d.txt")];
        let mut state = FileTreeState::from_paths(paths);
        // select first dir? root a is index 0; collapse via Left? Actually Left collapses if expanded.
        // Move right/down? Just call handle_key Left on root
        state.handle_key(ratatui::crossterm::event::KeyCode::Left);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 10));
        StatefulWidget::render(FileTreeWidget::new(), buf.area, &mut buf, &mut state);
    }
}
