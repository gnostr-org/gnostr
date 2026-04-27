use regex::Regex;

use super::scan::FileInfo;

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

pub(crate) fn build_html(url: &str, files: &[FileInfo]) -> String {
    let mut toc = String::new();
    let mut sections = String::new();
    let mut cxml = String::from("&lt;documents&gt;\n");

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
                    "<pre style='background:#f6f8fa; padding:10px; border-radius:5px;'><code>{}</code></pre>",
                    highlight_code(c)
                )
            }
            None => format!(
                "<p style=\"color:red;\">Skipped: Binary or too large (Size: {} bytes).</p>",
                f.size
            ),
        };

        sections.push_str(&format!(
            "<section id='{}' style='border-top:1px solid #eee; margin-top:20px;'><h3>{}</h3>{}</section>",
            anchor, f.rel, body
        ));
    }
    cxml.push_str("&lt;/documents&gt;");

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{ font-family: sans-serif; display: grid; grid-template-columns: 250px 1fr; margin: 0; }}
nav {{ border-right: 1px solid #ccc; height: 100vh; overflow-y: auto; padding: 10px; position: sticky; top: 0; background: #f9f9f9; }}
main {{ padding: 20px; }}
textarea {{ width: 100%; height: 500px; }}
.view-btn {{ padding: 10px; cursor: pointer; }}
</style>
<script>
function show(id) {{
document.getElementById('human').style.display = id === 'h' ? 'block' : 'none';
document.getElementById('llm').style.display = id === 'l' ? 'block' : 'none';
}}
</script>
</head>
<body>
<nav>
<strong>Files</strong>
<ul style="padding-left:15px; font-size:12px;">{toc}</ul>
</nav>
<main>
<h1>Repo: {url}</h1>
<button class="view-btn" onclick="show('h')">👤 Human View</button>
<button class="view-btn" onclick="show('l')">🤖 LLM View</button>
<div id="human">{sections}</div>
<div id="llm" style="display:none;">
<h2>LLM CXML</h2>
<textarea readonly>{cxml}</textarea>
</div>
</main>
</body></html>
"#,
        url = url,
        toc = toc,
        sections = sections,
        cxml = cxml
    )
}
