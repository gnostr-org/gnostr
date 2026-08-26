fn query_form_html(title: &str, action: &str, relay_value: &str, kinds_value: &str) -> String {
    let relay_value = relay_value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    format!(
        "<section><h2>{}</h2>\
         <form action=\"{}\" method=\"get\">\
         <label>Relay <input name=\"relay\" type=\"text\" placeholder=\"wss://relay.example.com\" value=\"{}\"></label><br>\
         <label>Authors <input name=\"authors\" type=\"text\" placeholder=\"pubkey1,pubkey2\"></label><br>\
         <label>IDs <input name=\"ids\" type=\"text\" placeholder=\"id1,id2\"></label><br>\
         <label>Generic tag <input name=\"generic_tag\" type=\"text\" placeholder=\"e\"></label><br>\
         <label>Generic value <input name=\"generic_value\" type=\"text\" placeholder=\"value\"></label><br>\
         <label>Hashtag <input name=\"hashtag\" type=\"text\" placeholder=\"root,reply\"></label><br>\
         <label>Mentions <input name=\"mentions\" type=\"text\" placeholder=\"pubkey1,pubkey2\"></label><br>\
         <label>References <input name=\"references\" type=\"text\" placeholder=\"event1,event2\"></label><br>\
         <label>NIP/Kind <input name=\"kinds\" type=\"text\" value=\"{}\"></label><br>\
         <label>Limit <input name=\"limit\" type=\"number\" value=\"100\" min=\"1\"></label><br>\
         <label>Search <input name=\"search\" type=\"text\" placeholder=\"keyword\"></label><br>\
         <button type=\"submit\">Search</button>\
         </form></section>",
        title, action, relay_value, kinds_value
    )
}

pub fn generic_query_form(action: &str, kinds_value: Option<&str>) -> String {
    query_form_html("Generic query", action, "", kinds_value.unwrap_or(""))
}

pub fn landing_search_form(action: &str) -> String {
    format!(
        "<form class=\"header-search\" action=\"{}\" method=\"get\">\
         <input type=\"hidden\" name=\"kinds\" value=\"1\">\
         <input type=\"hidden\" name=\"limit\" value=\"100\">\
         <input name=\"search\" type=\"search\" placeholder=\"Search\">\
         <button type=\"submit\">Go</button>\
         </form>",
        action
    )
}

pub fn nip_query_form(
    nip_lower: i32,
    action: &str,
    relay_value: &str,
    kinds_value: Option<&str>,
) -> String {
    query_form_html(
        &format!("NIP {} query", nip_lower),
        action,
        relay_value,
        kinds_value.unwrap_or(""),
    )
}
