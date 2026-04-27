use regex::Regex;
use std::collections::BTreeMap;

use super::scan::FileInfo;

const FLAT_CSS_PARTS: [&[u8]; 4] = [
    include_bytes!("../css/vars.css"),
    include_bytes!("../css/utils.css"),
    include_bytes!("../css/styles.css"),
    include_bytes!("../css/responsive.css"),
];

const FLAT_LAYOUT_CSS: &str = r#"
:root {
    color-scheme: light dark;
}
body.flat-app {
    margin: 0;
    height: 100vh;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    --flat-sidebar-width: 320px;
    overflow: hidden;
    font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: var(--clrBg);
    color: var(--clrText);
}
.flat-shell {
    display: grid;
    grid-template-columns: minmax(220px, var(--flat-sidebar-width)) 8px minmax(0, 1fr);
    min-height: 0;
    overflow: hidden;
}
.flat-header {
    position: sticky;
    top: 0;
    z-index: 20;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 16px 20px;
    border-bottom: 1px solid var(--clrBorder);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.04), transparent 42%), var(--clrBg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
}
.flat-header h1 {
    margin: 0;
    font-size: 1.25rem;
}
.flat-header .flat-repo {
    margin: 4px 0 0;
    color: var(--clrTextLight);
    font-size: 0.95rem;
    word-break: break-all;
}
.flat-header .flat-header-left {
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 14px;
}
.flat-header .flat-actions {
    margin: 0;
    flex: 0 0 auto;
}
.flat-logo {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 44px;
    height: 44px;
    flex: 0 0 auto;
}
.flat-logo svg {
    width: 44px;
    height: 44px;
    display: block;
}
.flat-sidebar {
    height: 100%;
    overflow: auto;
    min-height: 0;
    padding: 20px;
    border-right: 1px solid var(--clrBorder);
}
.flat-resizer {
    cursor: col-resize;
    background: linear-gradient(180deg, transparent, var(--clrBorder) 50%, transparent);
    opacity: 0.8;
}
.flat-resizer:hover {
    opacity: 1;
}
body.flat-resizing,
body.flat-resizing * {
    cursor: col-resize !important;
    user-select: none !important;
}
.flat-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin: 12px 0 18px;
}
.view-btn {
    border: 1px solid var(--clrBorder);
    border-radius: 999px;
    background: var(--clrPanel);
    color: var(--clrText);
    padding: 10px 14px;
    cursor: pointer;
    font-weight: 700;
    transition: transform 0.15s ease, border-color 0.15s ease, background-color 0.15s ease;
}
.view-btn:hover {
    transform: translateY(-1px);
    border-color: var(--clrLink);
}
.flat-toc {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
}
.flat-tree {
    margin: 0;
    padding: 0;
}
.flat-tree ul {
    list-style: none;
    margin: 0;
    padding: 0 0 0 18px;
    border-left: 1px solid color-mix(in srgb, var(--clrBorder) 80%, transparent);
}
.flat-tree summary,
.flat-tree-file a {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    color: var(--clrText);
    text-decoration: none;
    background: transparent;
    border: 1px solid transparent;
}
.flat-tree summary {
    cursor: pointer;
    list-style: none;
}
.flat-tree summary::-webkit-details-marker {
    display: none;
}
.flat-tree summary::before,
.flat-tree-file a::before {
    content: "";
    width: 14px;
    height: 14px;
    flex: 0 0 auto;
    opacity: 0.85;
    background-repeat: no-repeat;
    background-position: center;
    background-size: contain;
}
.flat-tree summary::before {
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23888' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M1.5 4.5h4l1.5 2h7.5v5.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1v-7.5a1 1 0 0 1 1-1Z'/%3E%3C/svg%3E");
}
.flat-tree-file a::before {
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23888' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M3.5 1.5h5l4 4V14.5h-9z'/%3E%3Cpath d='M8.5 1.5V5.5H12.5'/%3E%3C/svg%3E");
}
.flat-tree summary:hover,
.flat-tree-file a:hover {
    background: var(--clrPanel);
    border-color: var(--clrBorder);
}
.flat-main {
    padding: 0 24px 24px;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: auto;
}
.flat-panel {
    max-width: 100%;
    min-height: 0;
    flex: 1 1 auto;
}
.flat-section {
    margin-bottom: 20px;
}
.flat-file-details {
    border: 1px solid var(--clrBorder);
    border-radius: 18px;
    background: var(--clrPanel);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.08);
    overflow: hidden;
}
.flat-file-details > summary {
    display: flex;
    align-items: baseline;
    justify-content: flex-start;
    gap: 12px;
    padding: 14px 18px;
    cursor: pointer;
    list-style: none;
}
.flat-file-details > summary::-webkit-details-marker {
    display: none;
}
.flat-file-details > summary::before {
    content: "";
    width: 14px;
    height: 14px;
    flex: 0 0 auto;
    opacity: 0.8;
    background-repeat: no-repeat;
    background-position: center;
    background-size: contain;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%23888' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M6 4l4 4-4 4'/%3E%3C/svg%3E");
    transition: transform 0.15s ease;
}
.flat-file-details[open] > summary::before {
    transform: rotate(90deg);
}
.flat-file-details > summary:hover {
    background: color-mix(in srgb, var(--clrPanel) 70%, var(--clrBg));
}
.flat-section-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
}
.flat-file-summary {
    min-width: 0;
    flex: 1 1 auto;
    display: flex;
    align-items: baseline;
    gap: 12px;
}
.flat-file-summary h2 {
    margin: 0;
    font-size: 1.2rem;
    word-break: break-word;
}
.flat-meta {
    color: var(--clrTextLight);
    font-size: 0.9rem;
    white-space: nowrap;
}
.flat-card-body {
    padding: 18px;
}
.flat-code {
    margin: 0;
    padding: 18px;
    background: var(--clrBg);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: anywhere;
    line-height: 1.6;
}
.flat-binary {
    margin: 0;
    color: #b91c1c;
    font-weight: 700;
}
.flat-llm textarea {
    width: 100%;
    min-height: 0;
    flex: 1 1 auto;
    border: 1px solid var(--clrBorder);
    border-radius: 18px;
    background: var(--clrPanel);
    color: var(--clrText);
    padding: 18px;
    box-sizing: border-box;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
    font-size: 0.92rem;
}
.flat-llm {
    display: flex;
    flex-direction: column;
}
.flat-llm h2 {
    margin: 0 0 12px;
}
.flat-footer {
    position: relative;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 12px 20px;
    border-top: 1px solid var(--clrBorder);
    background: linear-gradient(0deg, rgba(255, 255, 255, 0.04), transparent 42%), var(--clrBg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    color: var(--clrTextLight);
    font-size: 0.9rem;
}
.flat-footer a {
    color: var(--clrLink);
    text-decoration: none;
}
.flat-footer a:hover {
    text-decoration: underline;
}
@media (max-width: 900px) {
    .flat-header {
        flex-direction: column;
        align-items: flex-start;
    }
    .flat-shell {
        grid-template-columns: 1fr;
    }
    .flat-resizer {
        display: none;
    }
    .flat-sidebar {
        border-right: 0;
        border-bottom: 1px solid var(--clrBorder);
        height: auto;
    }
    .flat-main {
        padding: 16px;
    }
}
"#;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn highlight_code(code: &str) -> String {
    let escaped = escape_html(code);
    let re_kw = Regex::new(r"\b(fn|let|mut|var|const|if|else|return|import|export|class|struct|impl|pub|type|use|for|while|match)\b").unwrap();
    let re_str = Regex::new(r#"(&quot;.*?&quot;|&#39;.*?&#39;)"#).unwrap();
    let re_comment = Regex::new(r"((//|#).*?(\n|$))").unwrap();

    let first_pass = re_kw.replace_all(
        &escaped,
        r#"<span style=\"color: #d73a49; font-weight: bold;\">$1</span>"#,
    );
    let second_pass =
        re_str.replace_all(&first_pass, r#"<span style=\"color: #032f62;\">$1</span>"#);
    let third_pass = re_comment.replace_all(
        &second_pass,
        r#"<span style=\"color: #6a737d; font-style: italic;\">$1</span>"#,
    );

    third_pass.to_string()
}

fn build_flat_css() -> String {
    let mut css = String::new();

    for part in FLAT_CSS_PARTS {
        css.push_str(std::str::from_utf8(part).expect("flat CSS assets must be UTF-8"));
        css.push('\n');
    }

    css.push_str(FLAT_LAYOUT_CSS);
    css
}

fn build_flat_logo_svg() -> String {
    include_str!("../images/logo.svg")
        .lines()
        .filter(|line| !line.trim_start().starts_with("<?xml"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    file: Option<String>,
}

fn insert_tree(root: &mut TreeNode, rel: &str, anchor: String) {
    let mut node = root;
    let mut parts = rel.split('/').peekable();
    let mut anchor = Some(anchor);

    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            node.children.entry(part.to_string()).or_default().file = anchor.take();
        } else {
            node = node.children.entry(part.to_string()).or_default();
        }
    }
}

fn render_tree_node(name: &str, node: &TreeNode, path: &str, out: &mut String) {
    let current_path = if path.is_empty() {
        name.to_string()
    } else {
        format!("{path}/{name}")
    };

    if let Some(anchor) = &node.file {
        out.push_str(&format!(
            "<li class='flat-tree-file'><a href='#{anchor}' data-file-anchor='{anchor}' data-file-path='{current_path}'>{name}</a></li>"
        ));
        return;
    }

    out.push_str(&format!("<li class='flat-tree-dir'><details data-tree-path='{current_path}'><summary>"));
    out.push_str(name);
    out.push_str("</summary><ul>");
    for (child_name, child) in &node.children {
        render_tree_node(child_name, child, &current_path, out);
    }
    out.push_str("</ul></details></li>");
}

pub(crate) fn build_html(url: &str, files: &[FileInfo]) -> String {
    let mut tree = TreeNode::default();
    let mut toc = String::new();
    let mut sections = String::new();
    let mut cxml = String::from("&lt;documents&gt;\n");
    let styles = build_flat_css();
    let logo_svg = build_flat_logo_svg();

    for (idx, f) in files.iter().enumerate() {
        let anchor = format!("f-{}", idx);
        insert_tree(&mut tree, &f.rel, anchor.clone());

        let body = match &f.content {
            Some(c) => {
                cxml.push_str(&format!(
                    "&lt;document index='{}'&gt;\n&lt;source&gt;{}&lt;/source&gt;\n&lt;document_content&gt;\n{}\n&lt;/document_content&gt;\n&lt;/document&gt;\n",
                    idx + 1,
                    f.rel,
                    escape_html(c)
                ));
                format!(
                    "<pre class=\"flat-code\"><code>{}</code></pre>",
                    highlight_code(c)
                )
            }
            None => format!("<p class=\"flat-binary\">Skipped binary or large file ({})</p>", f.size),
        };

        sections.push_str(&format!(
            "<section id='{}' class='flat-section'><details class='flat-file-details'><summary><div class='flat-file-summary'><h2>{}</h2></div><span class='flat-meta'>{} bytes</span></summary><div class='flat-card-body'>{}</div></details></section>",
            anchor, f.rel, f.size, body
        ));
    }
    for (name, node) in &tree.children {
        render_tree_node(name, node, "", &mut toc);
    }
    cxml.push_str("&lt;/documents&gt;");

    format!(
        r##"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<style>{styles}</style>
<script>
function show(id) {{
document.getElementById('human').classList.toggle('hide', id !== 'h');
document.getElementById('llm').classList.toggle('hide', id !== 'l');
}}

function applySidebarWidth(width) {{
const value = Math.max(220, Math.min(width, window.innerWidth * 0.6));
document.body.style.setProperty('--flat-sidebar-width', `${{value}}px`);
localStorage.setItem('flat-sidebar-width', String(value));
}}

function treeDetails() {{
return Array.from(document.querySelectorAll('.flat-tree-dir > details'));
}}

function treeLinks() {{
return Array.from(document.querySelectorAll('.flat-tree-file a'));
}}

function openTreePath(filePath) {{
const parts = filePath.split('/').filter(Boolean);
let current = '';

for (let i = 0; i < parts.length - 1; i += 1) {{
    current = current ? `${{current}}/${{parts[i]}}` : parts[i];
    const node = treeDetails().find((details) => details.dataset.treePath === current);
    if (node) {{
        node.open = true;
    }}
}}
}}

function syncSelectedFile() {{
const hash = window.location.hash.slice(1);
const sections = document.querySelectorAll('.flat-file-details');
let target = null;
let selectedSection = null;

sections.forEach((details) => {{
    const section = details.closest('section');
    const isTarget = hash && section && section.id === hash;
    if (isTarget) {{
        target = details;
        selectedSection = section;
    }}
}});

sections.forEach((details) => {{
    details.open = target ? details === target : false;
}});

const treeLink = treeLinks().find((link) => link.dataset.fileAnchor === hash);
if (!treeLink) {{
    return;
}}

const filePath = treeLink.dataset.filePath || '';
openTreePath(filePath);

const pathParts = filePath.split('/').filter(Boolean);
const folderPath = pathParts.slice(0, -1).join('/');
const folderNode = folderPath
    ? treeDetails().find((details) => details.dataset.treePath === folderPath)
    : null;

if (folderNode) {{
    folderNode.scrollIntoView({{ block: 'start', inline: 'nearest' }});
}}

if (selectedSection) {{
    window.requestAnimationFrame(() => {{
        selectedSection.scrollIntoView({{ block: 'start', inline: 'nearest' }});
    }});
}}
}}

window.addEventListener('hashchange', syncSelectedFile);
window.addEventListener('DOMContentLoaded', syncSelectedFile);
window.addEventListener('DOMContentLoaded', () => {{
const saved = Number(localStorage.getItem('flat-sidebar-width'));
if (!Number.isNaN(saved) && saved > 0) {{
    applySidebarWidth(saved);
}}

const resizer = document.querySelector('.flat-resizer');
if (!resizer) {{
    return;
}}

let dragging = false;

const move = (event) => {{
    if (!dragging) {{
        return;
    }}
    applySidebarWidth(event.clientX);
}};

const stop = () => {{
    dragging = false;
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', stop);
    window.removeEventListener('pointercancel', stop);
    document.body.classList.remove('flat-resizing');
}};

resizer.addEventListener('pointerdown', (event) => {{
    if (event.button !== 0) {{
        return;
    }}
    dragging = true;
    document.body.classList.add('flat-resizing');
    resizer.setPointerCapture(event.pointerId);
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', stop);
    window.addEventListener('pointercancel', stop);
    applySidebarWidth(event.clientX);
}});
}});
</script>
</head>
<body class="flat-app" id="top">
<header class="flat-header">
<div class="flat-header-left">
<div class="flat-logo" role="img" aria-label="gnostr">{logo_svg}</div>
<p class="flat-repo">{url}</p>
</div>
<div class="flat-actions">
<button class="view-btn" onclick="show('h')">Human</button>
<button class="view-btn" onclick="show('l')">LLM</button>
</div>
</header>
<div class="flat-shell">
<aside class="flat-sidebar">
<strong>Files</strong>
<ul class="flat-toc flat-tree">{toc}</ul>
</aside>
<div class="flat-resizer" aria-hidden="true"></div>
<main class="flat-main">
<div id="human" class="flat-panel">{sections}</div>
<div id="llm" class="flat-panel hide flat-llm">
<h2>LLM CXML</h2>
<textarea readonly>{cxml}</textarea>
</div>
</main>
</div>
<footer class="flat-footer">
<span>gnostr flat view</span>
<a href="#top">Back to top</a>
</footer>
</body></html>
        "##,
        url = url,
        toc = toc,
        sections = sections,
        cxml = cxml,
        styles = styles
    )
}
