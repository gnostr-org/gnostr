use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
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

#[derive(Clone, Debug)]
struct FileEntry {
    real: PathBuf,
    virtual_path: PathBuf,
    bucket: String,
    format: String,
}

pub struct CrawlerDiskTree {
    root: PathBuf,
    tree: FileTree,
    entries: Vec<FileEntry>,
    virtual_to_real: HashMap<PathBuf, PathBuf>,
    buckets: Vec<BucketSummary>,
}

impl CrawlerDiskTree {
    pub fn discover(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let files = collect_files(&root)?;
        let entries = build_entries(&root, &files)?;
        let buckets = summarize_buckets(&entries);
        let tree = build_tree(&entries)?;
        let virtual_to_real = entries
            .iter()
            .map(|entry| (entry.virtual_path.clone(), entry.real.clone()))
            .collect::<HashMap<_, _>>();

        Ok(Self {
            root,
            tree,
            entries,
            virtual_to_real,
            buckets,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        let next = Self::discover(&self.root)?;
        self.tree = next.tree;
        self.entries = next.entries;
        self.virtual_to_real = next.virtual_to_real;
        self.buckets = next.buckets;
        Ok(())
    }

    pub fn move_selection(&mut self, dir: MoveSelection) -> bool {
        self.tree.move_selection(dir)
    }

    pub fn selected_path(&self) -> Option<&Path> {
        self.tree
            .selected_file()
            .and_then(|info| self.virtual_to_real.get(info.full_path()))
            .map(PathBuf::as_path)
    }

    pub fn buckets(&self) -> &[BucketSummary] {
        &self.buckets
    }

    pub fn sorted_buckets(&self) -> Vec<BucketSummary> {
        let mut buckets = self.buckets.clone();
        buckets.sort_by(|a, b| a.name.cmp(&b.name));
        buckets
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
        let buckets = self.tree.sorted_buckets();
        let bucket_line = if buckets.is_empty() {
            String::from("no buckets")
        } else {
            buckets
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

fn build_entries(root: &Path, files: &[PathBuf]) -> Result<Vec<FileEntry>> {
    files.iter().map(|file| build_entry(root, file)).collect()
}

fn build_entry(root: &Path, real: &Path) -> Result<FileEntry> {
    let relative = real
        .strip_prefix(root)
        .with_context(|| format!("path {} is outside {}", real.display(), root.display()))?;

    let bucket = bucket_name(relative, real);
    let format = file_format(real);
    let virtual_path = virtual_path(root, relative, &bucket, &format, real)?;

    Ok(FileEntry {
        real: real.to_path_buf(),
        virtual_path,
        bucket,
        format,
    })
}

fn bucket_name(relative: &Path, real: &Path) -> String {
    if relative.parent().is_some_and(|parent| !parent.as_os_str().is_empty()) {
        relative
            .components()
            .next()
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("(root)"))
    } else {
        real.file_stem()
            .and_then(|stem| stem.to_str())
            .map(String::from)
            .unwrap_or_else(|| String::from("(root)"))
    }
}

fn file_format(real: &Path) -> String {
    match real
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => String::from("json"),
        Some("yaml") | Some("yml") => String::from("yaml"),
        Some("txt") => String::from("text"),
        Some(other) => other.to_string(),
        None => String::from("text"),
    }
}

fn virtual_path(
    root: &Path,
    relative: &Path,
    bucket: &str,
    format: &str,
    real: &Path,
) -> Result<PathBuf> {
    let mut virtual_path = PathBuf::from(root);
    virtual_path.push(bucket);
    virtual_path.push(format);

    let parent = relative.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        for component in parent.components().skip(1) {
            virtual_path.push(component.as_os_str());
        }
    }

    let stem = real
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("missing file stem")?;
    virtual_path.push(stem);
    Ok(virtual_path)
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

fn summarize_buckets(entries: &[FileEntry]) -> Vec<BucketSummary> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for entry in entries {
        *counts.entry(entry.bucket.clone()).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .map(|(name, files)| BucketSummary { name, files })
        .collect()
}

fn build_tree(entries: &[FileEntry]) -> Result<FileTree> {
    let refs = entries
        .iter()
        .map(|entry| entry.virtual_path.as_path())
        .collect::<Vec<_>>();
    let collapsed = BTreeSet::new();
    let mut tree = FileTree::new(&refs, &collapsed)?;
    tree.collapse_but_root();
    Ok(tree)
}
