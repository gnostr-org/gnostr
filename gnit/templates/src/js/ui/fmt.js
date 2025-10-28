function linkify(text="") {
	return text.replace(URL_REGEX, function(match, p1, p2, p3) {
		const url = p2+p3;
		let parsed;
		try {
			parsed = new URL(url)
		} catch (err) {
			return match;
		}
		let markup = html`<a target="_blank" rel="noopener noreferrer" href="${url}">${url}</a>`;
		if (is_img_url(parsed.pathname)) {
			//markup += `<button action="open-attachments">Open Img</button>`;
		} else if (is_video_url(parsed.pathname)) {
			//markup += `<button action="open-attachments">Open Video</button>`;
		}
		return p1+markup;
	})
}

function format_content(model, ev) {
	if (ev.kind === KIND_REACTION) {
		if (ev.content === "" || ev.content === "+")
			return "❤️"
		return html`${ev.content.trim()}`;
	}
	const content = (ev.kind == KIND_DM ? ev.decrypted || ev.content : ev.content)
		.trim();
	const body = fmt_mentions(model, fmt_body(content), ev.tags);
	let cw = get_content_warning(ev.tags)
	if (cw !== null) {
		let cwHTML = "Content Warning"
		if (cw === "") {
			cwHTML += "."
		} else {
			cwHTML += html`: "<span>${cw}</span>".`
		}
		return `
		<details class="cw">
		  <summary class="event-message">${cwHTML}</summary>
		  ${body}
		</details>`;
	}
	return body;
}

function fmt_mentions(model, str, tags) {
	var buf = "";
	for (var i = 0; i < str.length; i++) {
		let c = str.charAt(i);
		let peek = str.charAt(i+1);
		if (!(c == "#" && peek == "[")) {
			buf += c;
			continue;
		}
		const start = i;
		i += 2;
		let x = "";
		for(;;) {
			c = str.charAt(i);
			if (c >= '0' && c <= '9') {
				x += c;
				i++;
			} else if (c == ']') {
				break;	
			} else {
				buf += x;
				x = "";
				break;
			}
		}
		if (x == "")
			continue;
		let tag = tags[parseInt(x)];
		if (!tag) {
			buf += `#[${x}]`
			continue;
		}
		let ref = tag[1];
		if (tag[0] == 'e') {
			buf += `<span class="clickable bold" action="open-thread"
			data-thread-id="${ref}">note:${fmt_pubkey(ref)}</span>`;
		} else if (tag[0] == 'p') {
			let profile = model_get_profile(model, ref);
			buf += `<span class="username clickable" action="open-profile" 
			data-pubkey="${ref}">${fmt_name(profile)}</span>`;
		}
	}
	return buf;
}

/* fmt_body will parse images, blockquotes, and sanitize the content.
 */
function fmt_body(content) {
	return newlines_to_br(linkify(content))
}

/* DEPRECATED: use fmt_name
 * format_profile_name takes in a profile and tries it's best to
 * return a string that is best suited for the profile. 
 */
function fmt_profile_name(profile={}, fallback="Anonymous") {
	const name = profile.display_name || profile.user || profile.name || 
		fallback	
	return html`${name}`;
}

function fmt_name(profile={data:{}}) {
	const { data } = profile;
	const name = data.display_name || data.user || data.name || 
		fmt_pubkey(profile.pubkey);
	return html`${name}`;
}

function fmt_pubkey(pk) {
	if (!pk)
		return "Unknown";
	return pk.slice(-8)
}

function fmt_datetime(d) {
	return d.getFullYear() + 
		"/" + ("0" + (d.getMonth()+1)).slice(-2) + 
		"/" + ("0" + d.getDate()).slice(-2) + 
		" " + ("0" + d.getHours()).slice(-2) + 
		":" + ("0" + d.getMinutes()).slice(-2);
}
