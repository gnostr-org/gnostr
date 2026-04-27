use regex::Regex;

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
    min-height: 100vh;
    font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: var(--clrBg);
    color: var(--clrText);
}
.flat-shell {
    display: grid;
    grid-template-columns: minmax(260px, 320px) minmax(0, 1fr);
    min-height: 100vh;
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
}
.flat-header .flat-actions {
    margin: 0;
    flex: 0 0 auto;
}
.flat-sidebar {
    overflow: auto;
    padding: 20px;
    border-right: 1px solid var(--clrBorder);
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
    gap: 6px;
}
.flat-toc a {
    display: block;
    padding: 10px 12px;
    border-radius: 12px;
    color: var(--clrText);
    text-decoration: none;
    background: transparent;
    border: 1px solid transparent;
}
.flat-toc a:hover {
    background: var(--clrPanel);
    border-color: var(--clrBorder);
}
.flat-main {
    padding: 24px;
    min-width: 0;
}
.flat-panel {
    max-width: 100%;
}
.flat-section {
    margin-bottom: 20px;
}
.flat-section-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
}
.flat-section-head h2 {
    margin: 0;
    font-size: 1.2rem;
    word-break: break-word;
}
.flat-meta {
    color: var(--clrTextLight);
    font-size: 0.9rem;
    white-space: nowrap;
}
.flat-card {
    border: 1px solid var(--clrBorder);
    border-radius: 18px;
    background: var(--clrPanel);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.08);
    overflow: hidden;
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
    min-height: 500px;
    border: 1px solid var(--clrBorder);
    border-radius: 18px;
    background: var(--clrPanel);
    color: var(--clrText);
    padding: 18px;
    box-sizing: border-box;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
    font-size: 0.92rem;
}
@media (max-width: 900px) {
    .flat-header {
        flex-direction: column;
        align-items: flex-start;
    }
    .flat-shell {
        grid-template-columns: 1fr;
    }
    .flat-sidebar {
        border-right: 0;
        border-bottom: 1px solid var(--clrBorder);
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

pub(crate) fn build_html(url: &str, files: &[FileInfo]) -> String {
    let mut toc = String::new();
    let mut sections = String::new();
    let mut cxml = String::from("&lt;documents&gt;\n");
    let styles = build_flat_css();

    for (idx, f) in files.iter().enumerate() {
        let anchor = format!("f-{}", idx);
        toc.push_str(&format!("<li><a href='#{}'>{}</a></li>", anchor, f.rel));

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
            "<section id='{}' class='flat-section'><div class='flat-section-head'><h2>{}</h2><span class='flat-meta'>{} bytes</span></div><div class='flat-card'><div class='flat-card-body'>{}</div></div></section>",
            anchor, f.rel, f.size, body
        ));
    }
    cxml.push_str("&lt;/documents&gt;");

    format!(
        r#"
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
</script>
</head>
<body class="flat-app">
<header class="flat-header">
<div class="flat-header-left">
<h1>Flat view</h1>
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
<ul class="flat-toc">{toc}</ul>
</aside>
<main class="flat-main">
<div id="human" class="flat-panel">{sections}</div>
<div id="llm" class="flat-panel hide flat-llm">
<h2>LLM CXML</h2>
<textarea readonly>{cxml}</textarea>
</div>
</main>
</div>
</body></html>
"#,
        url = url,
        toc = toc,
        sections = sections,
        cxml = cxml,
        styles = styles
    )
}
