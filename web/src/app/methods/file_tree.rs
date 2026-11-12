use std::path::{Path, PathBuf};

use askama::Template;
use axum::response::{IntoResponse, Response};

use super::filters;
use crate::app::into_response;

#[derive(Template)]
#[template(path = "file_tree.html")]
pub struct View {
    pub title: String,
    pub segments: Vec<(String, String)>,
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub name: String,
    pub href: String,
    pub is_dir: bool,
}

pub fn render(scan_path: &Path, relative_path: &Path) -> Option<Response> {
    let abs_path = scan_path.join(relative_path);
    if !abs_path.is_dir() {
        return None;
    }

    let mut entries = Vec::new();
    if let Ok(dir) = std::fs::read_dir(&abs_path) {
        for entry in dir.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" {
                continue;
            }
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let mut href = PathBuf::from("/");
            for component in relative_path.components() {
                href.push(component);
            }
            href.push(&name);
            let href = href.to_string_lossy().to_string();
            entries.push(Entry { name, href, is_dir });
        }
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    let title = if relative_path.as_os_str().is_empty() || relative_path == Path::new(".") {
        "gnostr.org".to_string()
    } else {
        format!("gnostr.org/{}", relative_path.display())
    };

    let segments = path_segments(relative_path);

    Some(
        into_response(View {
            title,
            segments,
            entries,
        })
        .into_response(),
    )
}

fn path_segments(path: &Path) -> Vec<(String, String)> {
    let mut segments = Vec::new();
    let mut href = String::new();
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy().to_string();
        if name.is_empty() {
            continue;
        }
        if !href.is_empty() {
            href.push('/');
        }
        href.push_str(&name);
        segments.push((name, format!("/{href}")));
    }
    segments
}
