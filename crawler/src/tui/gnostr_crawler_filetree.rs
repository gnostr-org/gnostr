use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
pub use gnostr_filetreelist::MoveSelection;
use gnostr_filetreelist::FileTree;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BucketSummary {
    pub name: String,
    pub files: usize,
}

pub struct CrawlerDiskTree {
    root: PathBuf,
    tree: FileTree,
    files: Vec<PathBuf>,
    buckets: Vec<BucketSummary>,
}

impl CrawlerDiskTree {
    pub fn discover(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let files = collect_files(&root)?;
        let buckets = summarize_buckets(&root, &files);
        let tree = build_tree(&files)?;

        Ok(Self {
            root,
            tree,
            files,
            buckets,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        let next = Self::discover(&self.root)?;
        self.tree = next.tree;
        self.files = next.files;
        self.buckets = next.buckets;
        Ok(())
    }

    pub fn move_selection(&mut self, dir: MoveSelection) -> bool {
        self.tree.move_selection(dir)
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.tree.selected_file().map(|info| info.full_path())
    }

    pub fn buckets(&self) -> &[BucketSummary] {
        &self.buckets
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn visible_items(&self) -> Vec<ListItem<'static>> {
        let total = self.tree.iterate(0, self.tree.iterate(0, usize::MAX).count());
        total
            .map(|(item, selected)| {
                let indent = "  ".repeat(item.info().indent() as usize);
                let icon = if item.kind().is_path() {
                    if item.kind().is_path_collapsed() {
                        "▸"
                    } else {
                        "▾"
                    }
                } else {
                    " "
                };
                let label = format!("{indent}{icon} {}", item.info().path_str());
                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(label, style),
                    Span::raw("  "),
                    Span::styled(
                        if item.kind().is_path() { "bucket" } else { "file" },
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    }
}

pub struct CrawlerFileTreeWidget<'a> {
    pub tree: &'a CrawlerDiskTree,
    pub title: &'a str,
}

impl<'a> Widget for CrawlerFileTreeWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut rows = Vec::new();
        let bucket_line = if self.tree.buckets.is_empty() {
            String::from("no buckets")
        } else {
            self.tree
                .buckets
                .iter()
                .map(|bucket| format!("{}:{}", bucket.name, bucket.files))
                .collect::<Vec<_>>()
                .join("  ")
        };

        rows.push(ListItem::new(Line::from(vec![
            Span::styled(
                self.tree.root().display().to_string(),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(bucket_line, Style::default().fg(Color::DarkGray)),
        ])));
        rows.extend(self.tree.visible_items());

        List::new(rows)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .render(area, buf);
    }
}

fn collect_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if file_name == "target" || file_name == ".git" || file_name.starts_with('.') && path.is_dir() {
            continue;
        }
        let meta = entry.metadata()?;
        if meta.is_dir() {
            walk(&path, files)?;
        } else if meta.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn summarize_buckets(root: &Path, files: &[PathBuf]) -> Vec<BucketSummary> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for file in files {
        let label = file
            .strip_prefix(root)
            .ok()
            .and_then(|relative| relative.components().next())
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("(root)"));
        *counts.entry(label).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .map(|(name, files)| BucketSummary { name, files })
        .collect()
}

fn build_tree(files: &[PathBuf]) -> Result<FileTree> {
    let refs = files.iter().map(PathBuf::as_path).collect::<Vec<_>>();
    let collapsed = BTreeSet::new();
    let mut tree = FileTree::new(&refs, &collapsed)?;
    tree.collapse_but_root();
    Ok(tree)
}
