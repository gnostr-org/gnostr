use crate::query::forms::landing_search_form;
use crate::relays::{get_config_dir_path, live_nips};
use std::fs;
use std::path::PathBuf;

pub fn write_index_html() -> std::io::Result<PathBuf> {
    let config_dir = get_config_dir_path();
    fs::create_dir_all(&config_dir)?;

    let mut nips = live_nips();
    if let Ok(entries) = fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(nip) = name.parse::<i32>() {
                        nips.push(nip);
                    }
                }
            }
        }
    }
    nips.sort_unstable();
    nips.dedup();
    let nip_links = if nips.is_empty() {
        "<li>No NIP buckets yet. Start serve and wait for the sniper service.</li>".to_string()
    } else {
        nips.iter()
            .map(|nip| {
                format!(
                    "<li><a href=\"/{0}\">NIP {0}</a> - <a href=\"/{0}/relays.json\">json</a> <a href=\"/{0}/relays.yaml\">yaml</a> <a href=\"/{0}/relays.txt\">txt</a></li>",
                    nip
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };
    let nav = [
        ("/", "gnostr/crawler"),
        ("/relays.json", "relays.json"),
        ("/relays.yaml", "relays.yaml"),
        ("/relays.txt", "relays.txt"),
    ];
    let body = format!("<section><h2>NIPs</h2><ul>{}</ul></section>", nip_links);
    let html = render_page_shell_with_header_right(
        "gnostr crawler",
        &nav,
        &body,
        &landing_search_form("/query"),
    );

    let path = config_dir.join("index.html");
    fs::write(&path, html)?;
    Ok(path)
}

pub fn render_page_shell(title: &str, nav: &[(&str, &str)], body: &str) -> String {
    render_page_shell_with_header_right(title, nav, body, "")
}

pub fn render_page_shell_with_header_right(
    title: &str,
    nav: &[(&str, &str)],
    body: &str,
    header_right: &str,
) -> String {
    let nav_html = nav
        .iter()
        .map(|(href, label)| format!("<a href=\"{}\">{}</a>", href, label))
        .collect::<Vec<_>>()
        .join("<span class=\"nav-sep\">/</span>");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{}</title>\
         <style>\
         :root{{color-scheme:dark light;}}\
         body{{font-family:system-ui,-apple-system,BlinkMacSystemFont,\"Segoe UI\",sans-serif;margin:0;line-height:1.5;}}\
         .site-header{{position:sticky;top:0;z-index:10;background:#111;border-bottom:1px solid #333;padding:0.9rem 1rem;display:flex;align-items:flex-start;justify-content:space-between;gap:1rem;}}\
         .site-header-main{{min-width:0;flex:1 1 auto;}}\
         .site-title{{margin:0;font-size:1.1rem;}}\
         .site-nav{{margin-top:0.35rem;display:flex;flex-wrap:wrap;gap:0.5rem;align-items:center;}}\
         .site-nav a{{color:inherit;text-decoration:none;padding:0.2rem 0.4rem;border-radius:0.35rem;background:rgba(255,255,255,0.06);}}\
         .nav-sep{{opacity:0.4;}}\
         .site-header-right{{display:flex;align-items:center;justify-content:flex-end;flex:0 0 auto;}}\
          .header-search{{display:flex;align-items:center;gap:0.35rem;}}\
          .header-search input{{min-width:12rem;max-width:18rem;}}\
          main{{padding:1rem;max-width:1100px;}}\
          section{{margin-bottom:1.5rem;}}\
          ul{{padding-left:1.2rem;}}\
          .relay-favorite-card{{outline:none;}}\
          .relay-favorite-card:focus-within{{box-shadow:0 0 0 2px #60a5fa;border-radius:0.35rem;}}\
          .relay-favorite-heart{{margin-right:0.35rem;min-width:1em;display:inline-block;color:#ef4444;}}\
          .relay-favorite-card.is-favorite .relay-favorite-heart{{opacity:1;}}\
          code{{background:rgba(255,255,255,0.08);padding:0.1rem 0.25rem;border-radius:0.25rem;}}\
          </style></head><body>\
           <header class=\"site-header\"><div class=\"site-header-main\"><nav class=\"site-nav\">{}</nav></div><div class=\"site-header-right\">{}</div></header>\
           <main>{}</main></body></html>",
        title, nav_html, header_right, body
    )
}
