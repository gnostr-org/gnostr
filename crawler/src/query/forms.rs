pub fn generic_query_form(kinds_value: Option<&str>) -> String {
    let kinds_value = kinds_value.unwrap_or("");
    format!(
        r#"
        <section>
          <h2>Generic query</h2>
          <form action="/query" method="get">
            <label>Relay <input name="relay" type="text" placeholder="wss://relay.example.com"></label><br>
            <label>Authors <input name="authors" type="text" placeholder="pubkey1,pubkey2"></label><br>
            <label>IDs <input name="ids" type="text" placeholder="id1,id2"></label><br>
            <label>Generic tag <input name="generic_tag" type="text" placeholder="e"></label><br>
            <label>Generic value <input name="generic_value" type="text" placeholder="value"></label><br>
            <label>Hashtag <input name="hashtag" type="text" placeholder="root,reply"></label><br>
            <label>Mentions <input name="mentions" type="text" placeholder="pubkey1,pubkey2"></label><br>
            <label>References <input name="references" type="text" placeholder="event1,event2"></label><br>
            <label>NIPs <input name="kinds" type="text" value="{}"></label><br>
            <label>Limit <input name="limit" type="number" value="10" min="1"></label><br>
            <label>Search <input name="search" type="text" placeholder="keyword"></label><br>
            <button type="submit">Search</button>
          </form>
        </section>
    "#,
        kinds_value
    )
}

pub fn nip_query_form(
    nip_lower: i32,
    action: &str,
    relay_value: &str,
    kinds_value: Option<&str>,
) -> String {
    let relay_value = relay_value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    let kinds_value = kinds_value.unwrap_or("");
    format!(
        "<section><h2>NIP {} query</h2>\
         <form action=\"{}\" method=\"get\">\
         <label>Relay <input name=\"relay\" type=\"text\" placeholder=\"wss://relay.example.com\" value=\"{}\"></label><br>\
         <label>Authors <input name=\"authors\" type=\"text\" placeholder=\"pubkey1,pubkey2\"></label><br>\
         <label>IDs <input name=\"ids\" type=\"text\" placeholder=\"id1,id2\"></label><br>\
         <label>Generic tag <input name=\"generic_tag\" type=\"text\" placeholder=\"e\"></label><br>\
         <label>Generic value <input name=\"generic_value\" type=\"text\" placeholder=\"value\"></label><br>\
         <label>Hashtag <input name=\"hashtag\" type=\"text\" placeholder=\"root,reply\"></label><br>\
         <label>Mentions <input name=\"mentions\" type=\"text\" placeholder=\"pubkey1,pubkey2\"></label><br>\
         <label>References <input name=\"references\" type=\"text\" placeholder=\"event1,event2\"></label><br>\
         <label>NIPs <input name=\"kinds\" type=\"text\" value=\"{}\"></label><br>\
         <label>Limit <input name=\"limit\" type=\"number\" value=\"10\" min=\"1\"></label><br>\
         <label>Search <input name=\"search\" type=\"text\" placeholder=\"keyword\"></label><br>\
         <button type=\"submit\">Search</button>\
         </form></section>",
        nip_lower, action, relay_value, kinds_value
    )
}
