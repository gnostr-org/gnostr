// This file contains all methods related to rendering UI elements. Rendering
// is done by simple string manipulations & templates. If you need to write
// loops simply write it in code and return strings.

function render_replying_to(model, ev) {
	if (!(ev.refs && ev.refs.reply))
		return "";
	let pubkeys = ev.refs.pubkeys || []
	if (pubkeys.length === 0 && ev.refs.reply) {
		const replying_to = model.all_events[ev.refs.reply]
		// If there is no profile being replied to, it is simply a reply to an 
		// event itself, thus render it differently.
		if (!replying_to) {
			return html`<span class="replying-to small-txt">
				replying in thread 
				<span class="thread-id clickable" action="open-thread"
				data-thread-id="${ev.refs.reply}">
				${fmt_pubkey(ev.refs.reply)}</span></span>`;
		} else {
			pubkeys = [replying_to.pubkey];
		}
	}
	const names = pubkeys.map((pk) => {
		return render_name(pk, model_get_profile(model, pk).data);
	}).join(", ")
	return `
	<span class="replying-to small-txt">
		replying to ${names}
	</span>
	`
}

function render_share(model, ev, opts) {
	const shared_ev = model.all_events[ev.refs && ev.refs.root]
	// If the shared event hasn't been resolved or leads to a circular event 
	// kind we will skip out on it.
	if (!shared_ev || shared_ev.kind == KIND_SHARE)
		return "";
	opts.shared = {
		pubkey: ev.pubkey,
		profile: model_get_profile(model, ev.pubkey),
		share_time: ev.created_at,
		share_evid: ev.id,
	}
	return render_event(model, shared_ev, opts)
}

function render_shared_by(ev, opts) {
	if (!opts.shared)
		return "";
	const { profile, pubkey } = opts.shared
	return `<div class="shared-by">Shared by ${render_name(pubkey, profile)}
		</div>`
}

function render_collapsible_block(title, body, open=false, class_name="") {
    return `<details class="nip34-collapsible ${class_name}"${open ? " open" : ""}>
        <summary>${title}</summary>
        <div class="nip34-collapsible-body">${body}</div>
    </details>`;
}

function render_event(model, ev, opts={}) {
	switch(ev.kind) {
		case KIND_SHARE:
			return render_share(model, ev, opts);
		case KIND_DM:
			return render_dm(model, ev, opts);
	}

	const profile = model_get_profile(model, ev.pubkey);
	const delta = fmt_since_str(new Date().getTime(), ev.created_at*1000)
	const target_thread_id = ev.refs.root || ev.id;
	let classes = "event"
	if (!opts.is_composing)
		classes += " bottom-border";
	return html`<div id="ev${ev.id}" class="${classes}" action="open-thread"
	data-thread-id="${target_thread_id}">
		$${render_shared_by(ev, opts)}
		<div class="flex">
		<div class="userpic">
		$${render_profile_img(profile)}</div>
		<div class="event-content">
			<div class="info">
				$${render_name(ev.pubkey, profile.data)}
				<span class="timestamp" data-timestamp="${ev.created_at}">${delta}</span>
			</div>
			<div class="comment">
				$${render_event_body(model, ev, opts)}
			</div>
		</div>
		</div>
	</div>` 
}

function render_dm(model, ev, opts) {
	let classes = "event"
	if (ev.kind == KIND_DM) {
		classes += " dm";
		if (ev.pubkey == model.pubkey)
			classes += " mine";
	}
	const profile = model_get_profile(model, ev.pubkey);
	const delta = fmt_since_str(new Date().getTime(), ev.created_at*1000)
	let show_media = event_shows_media(model, ev, model.embeds);
	return html`<div id="ev${ev.id}" class="${classes}">
		<div class="wrap">
			<div class="body">
			<p>$${format_content(model, ev, show_media)}</p>
			</div>
			<div class="timestamp" data-timestamp="${ev.created_at}">${delta}</div>
		</div>
	</div>`
}

function event_body_should_fold(ev) {
	if (!ev) {
		return false;
	}

	const content = (ev.kind == KIND_DM ? ev.decrypted || ev.content : ev.content).trim();
	if (!content) {
		return false;
	}

	return content.length > 280
		|| /https?:\/\/\S{80,}/i.test(content)
		|| content.split(/\r?\n/).some((line) => line.length > 120);
}

function event_shows_media(model, ev, mode) {
	if (mode == "friends")
		return model.contacts.friends.has(ev.pubkey);
	return true;
}

function rerender_dm(model, ev, el) {
	let show_media = event_shows_media(model, ev, model.embeds);
	find_node(".body > p", el).innerHTML = format_content(model, ev, show_media);
}

function render_event_nointeract(model, ev, opts={}) {
	const profile = model_get_profile(model, ev.pubkey);
	const delta = fmt_since_str(new Date().getTime(), ev.created_at*1000)
	return html`<div class="event border-bottom">
		<div class="flex">
		<div class="userpic">
			$${render_profile_img(profile)}
		</div>	
		<div class="event-content">
			<div class="info">
				$${render_name(ev.pubkey, profile.data)}
				<span class="timestamp" data-timestamp="${ev.created_at}">${delta}</span>
			</div>
			<div class="comment">
				$${render_event_body(model, ev, opts)}
			</div>
		</div>
		</div>
	</div>`
}

function render_event_body(model, ev, opts) {
	const { shared } = opts;
	const can_delete = model.pubkey === ev.pubkey || 
		(opts.shared && model.pubkey == opts.shared.pubkey);
	// Only show media for content that is by friends.
	let show_media = true;
	if (opts.is_composing) {
		show_media = false;
	} else if (model.embeds == "friends") {
		show_media = model.contacts.friends.has(ev.pubkey);
	}
	let str = "<div>";
    if (is_nip34_repo_kind(ev.kind)) {
        str += render_repo_event_summary(model, ev);
    }
	str += render_replying_to(model, ev);
	const content = format_content(model, ev, show_media);
	if (!opts.is_composing && !is_nip34_repo_kind(ev.kind) && event_body_should_fold(ev)) {
		str += `</div>${render_collapsible_block(
			"Long event body",
			`<div class="event-fold-body"><p>${content}</p></div>`,
			false,
			"event-fold"
		)}`;
	} else {
		str += `</div><p>
		${content}
		</p>`;
	}
	str += render_reactions(model, ev);
	str += opts.nobar || ev.kind == KIND_DM ? "" : 
		render_action_bar(model, ev, {can_delete, shared});
	return str;
}

function render_react_onclick(our_pubkey, reacting_to, emoji, reactions) {
	const reaction = reactions[our_pubkey]
	if (!reaction) {
		return html`action="reply" data-emoji="${emoji}" data-to="${reacting_to}"`;
	} else {
		return html`action="delete" data-evid="${reaction.id}"`;
	}
}

function render_reaction_group(model, emoji, reactions, reacting_to) {
	let count = 0;
	for (const k in reactions) {
		count++;
	}
	let onclick = render_react_onclick(model.pubkey, 
		reacting_to.id, emoji, reactions);
	return html`
	<span $${onclick} class="reaction-group clickable">
		<span class="reaction-emoji">
		${emoji}
		</span>
		${count}
	</span>`;
}

function render_action_bar(model, ev, opts={}) {
	const { pubkey } = model;
	let { can_delete, shared } = opts;
	// TODO rewrite all of the toggle heart code. It's mine & I hate it.
	const thread_root = (ev.refs && ev.refs.root) || ev.id;
	const reaction = model_get_reacts_to(model, pubkey, ev.id, R_SHAKA);
	const liked = !!reaction;
	const reaction_id = reaction ? reaction.id : "";
	let str = html`<div class="action-bar">`;
	if (!shared && event_can_reply(ev)) {
		str += html`
		<button class="icon" title="Reply" action="reply-to" data-evid="${ev.id}">
			<img class="icon svg small" src="/images/event-reply.svg"/>
		</button>
		<button class="icon react heart ${ab(liked, 'liked', '')}"
			action="react-like"
			data-reaction-id="${reaction_id}"
			data-reacting-to="${ev.id}"
			title="$${ab(liked, 'Unlike', 'Like')}">
			<img class="icon svg small ${ab(!liked, 'hide', '')}"
				src="${IMG_EVENT_LIKE}"/>
			<img class="icon svg small ${ab(liked, 'hide', '')}"
				src="${IMG_EVENT_LIKED}"/>
		</button>`;
	}
	if (!shared) {
		str += html`<button class="icon" title="Share" data-evid="${ev.id}"
			action="share">
			<img class="icon svg small" src="/images/event-share.svg"/>
		</button>`;
	}
	str += `
	<button class="icon" title="More Options" action="open-event-options" data-evid="${ev.id}" onclick="console.log('More Options button clicked!', this.dataset.evid);">
		<img class="icon svg small" src="/images/event-options.svg"/>
	</button>`;
	return str + "</div>";
}

function render_reactions_inner(model, ev) {
	const groups = get_reactions(model, ev.id)
	let str = ""
	for (const emoji of Object.keys(groups)) {
		str += render_reaction_group(model, emoji, groups[emoji], ev)
	}
	return str;
}

function render_reactions(model, ev) {
	return html`<div class="reactions">$${render_reactions_inner(model, ev)}</div>`
}

function render_repo_event_summary(model, ev) {
    if (!ev) {
        console.warn("render_repo_event_summary: event object is undefined.");
        return "";
    }
    let a = "";
    let d = "";
    let r = "";
    let alt = "";
    let e = "";
    let p = "";
    let summary = "";
    let repo_id = "";
    let repo_url = "";
    let repo_name = "";
    let description = "";
    let clone_url = "";
    let flat_url = "";
    let relays = [];
    let maintainers = [];
    let status_tag = "";
    let issue_title = "";
    let patch_id = "";
    let pull_req_id = "";
    const state_refs = [];
    const json_body = render_collapsible_block("Raw JSON", html`<div class="nip34-json-card">
        <div class="nip34-json-card-head">
            <span class="nip34-json-label">Raw JSON</span>
        </div>
        <pre>${JSON.stringify(ev, null, 2)}</pre>
    </div>`);
    for (const tag of ev.tags) {
        if (tag[0] === "d") { // Repository Announcement Address
            repo_id = tag[1];
            repo_name = tag[1];
            d = tag[1];
        } else if (tag[0] === "d" && tag[1].includes("30618:")) { // Repository Announcement Address
            repo_id = tag[1];
            repo_name = tag[1];
            d = tag[1];
        } else if (tag[0] === "name") {
            //repo_name = tag[1];
        } else if (tag[0] === "url" || tag[0] === "web") {
            repo_url = tag[1];
        } else if (tag[0] === "description") {
            description = tag[1];
        } else if (tag[0] === "clone") {
            clone_url = tag[1];
            flat_url = `/flat?repo=${encodeURIComponent(tag[1])}`;
        } else if (tag[0] === "relays") {
            relays = tag.slice(1);
        } else if (tag[0] === "maintainers") {
            maintainers = tag.slice(1);
        } else if (tag[0] === "status") {
            status_tag = tag[1];
        } else if (tag[0] === "title") {
            issue_title = tag[1];
        } else if (tag[0] === "e" && tag[3] === "patch") { // Patch ID
            patch_id = tag[1];
        } else if (tag[0] === "e" && tag[3] === "pull_request") { // Pull Request ID
            pull_req_id = tag[1];
        } else if (tag[0] === "a") {
            a = tag[1];
        } else if (tag[0] === "r") {
            r = tag[1];
        } else if (tag[0] === "alt") {
            alt = tag[1];
        } else if (tag[0] === "e") {
            e = tag[1];
        } else if (tag[0] === "p") {
            p = tag[1];
        } else if (tag[0] && tag[0].startsWith("refs/") && tag[1]) {
            state_refs.push([tag[0], tag[1]]);
        }
    }

    switch (ev.kind) {
        case KIND_REPO_ANNOUNCE:
            summary = `Repository Announcement: <b>${repo_name || "309:Untitled Repository"}</b>`;
            if (flat_url) {
                summary += ` <a href="${flat_url}" target="_blank" rel="noreferrer">Flat View</a>`;
            }
            if (description) {
                summary += `<br>${description}`;
            }
            if (repo_url) {
                summary += `<br>Web: <a href="${repo_url}" target="_blank">${repo_url}</a>`;
            }
            if (clone_url) {
                summary += `<br>Clone: <a href="${clone_url}" target="_blank" rel="noreferrer">${clone_url}</a>`;
            }
            if (maintainers.length > 0) {
                summary += `<br>Maintainers: ${maintainers.map(pk => fmt_name(model_get_profile(model, pk))).join(", ")}`;
            }
            if (relays.length > 0) {
                summary += `<br>Relays: ${relays.join(", ")}`;
            }
            break;
        case KIND_REPO_STATE_ANNOUNCE:
            summary = html`<div class="nip34-state-summary">
                <div class="nip34-state-heading">
                    <strong>Repository State</strong>
                    <span class="nip34-state-count">${state_refs.length} refs</span>
                </div>
                <div class="nip34-state-name">${repo_name || "Unnamed repository state"}</div>
            </div>`;
            if (state_refs.length > 0) {
                const preview_refs = state_refs.slice(0, 20).map(([ref_name, ref_value]) => html`
                    <li class="nip34-state-ref-item">
                        <code class="nip34-state-ref-name">${ref_name}</code>
                        <span class="nip34-state-ref-arrow">→</span>
                        <code class="nip34-state-ref-value">${ref_value}</code>
                    </li>
                `).join("");
                const preview_note = state_refs.length > 20
                    ? html`<p class="nip34-state-preview-note">Showing 20 of ${state_refs.length} refs.</p>`
                    : "";
                summary += render_collapsible_block(
                    `Refs (${state_refs.length})`,
                    html`<div class="nip34-state-fold">
                        ${preview_note}
                        <ul class="nip34-state-ref-list">${preview_refs}</ul>
                    </div>`,
                    false,
                    "nip34-state-card"
                );
            }
            summary += json_body;
            break;
        case KIND_REPO_PATCH:
            //let patch_d = ev.tags.filter(tag => tag[0] === "d");
            let patch_content = ev.content;
            //if (patch_d.length > 0) {
            //    summary = `355:Repository Patch: ${patch_d.map(tag => tag[1]).join(", ")}</br>`;
            //} else {
            //    summary = `357:Repository Patch: <b>${ev.d || ev.id}</b>`;
            //}
            summary += render_collapsible_block(
                "Patch content",
                html`<pre class="nip34-patch-code">${patch_content}</pre>`,
                false,
                "nip34-patch-card"
            ); //TODO git commit formatting
            if (issue_title) {
                summary += `<br>Title: ${issue_title}`;
            }
            if (patch_id) {
                summary += `<br>Patch ID: ${fmt_note_id(patch_id)}`;
            } else {

                //summary += json_body; //`<br>Tracking stopped.`;
            break;
        }
        case KIND_REPO_PULL_REQ:
        case KIND_REPO_PULL_REQ_UPDATE:
            summary = `Pull Request for repository <b>${ev.repo_name || ev.repo_id || "Unknown"}</b>`;
            if (issue_title) {
                summary += `<br>Title: ${issue_title}`;
            }
            if (pull_req_id) {
                summary += `<br>PR ID: ${fmt_note_id(pull_req_id)}`;
            }
                summary += json_body; //`<br>Tracking stopped.`;
            break;
        case KIND_REPO_ISSUE:
            summary = `Issue for repository <b>${ev.repo_name || ev.repo_id || "Unknown"}</b>`;
            if (ev.issue_title) {
                summary += `<br>Title: ${ev.issue_title}`;
            }
                summary += json_body; //`<br>Tracking stopped.`;
            break;
        case KIND_REPO_STATUS_OPEN:
        case KIND_REPO_STATUS_APPLIED:
        case KIND_REPO_STATUS_CLOSED:
        case KIND_REPO_STATUS_DRAFT:
            summary = `Repository Status: <b>${ev.status_tag || "401:Unknown"}</b>`;
            if (repo_name) {
                summary += `<br>Repository: ${ev.repo_name}`;
            }
            if (issue_title) {
                summary += `<br>Related: ${ev.issue_title}`;
            } else if (patch_id) {
                summary += `<br>Related Patch: ${ev.fmt_note_id(patch_id)}`;
            }
            summary += json_body; //`<br>Tracking stopped.`;
            break;
    }

    return summary;
}

// Utility Methods

function render_pubkey(pk) {
	return fmt_pubkey(pk);
}

function render_username(pk, profile) {
	return (profile && profile.name) || render_pubkey(pk)
}

function render_mentioned_name(pk, profile) {
	return render_name(pk, profile, "");
}

function render_name(pk, profile, prefix="") {
	// Beware of whitespace.
	return html`<span>${prefix}<span class="username clickable" 
	action="open-profile" data-pubkey="${pk}"> 
		${fmt_profile_name(profile, fmt_pubkey(pk))}</span></span>`
}

function render_profile_img(profile, noclick=false) {
	const name = fmt_name(profile);
	let str = html`class="pfp clickable" action="open-profile"`;
	if (noclick)
		str = "class='pfp'";
	return html`<img 
	$${str}
	data-pubkey="${profile.pubkey}" 
	title="${name}" 
	src="${get_profile_pic(profile)}" onerror="this.onerror=null;this.src='${IMG_NO_USER}';"/>`
}
