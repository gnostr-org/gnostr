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
	str += render_replying_to(model, ev);
	str += `</div><p>
	${format_content(model, ev, show_media)}
	</p>`;
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
			<img class="icon svg small" src="/icon/event-reply.svg"/>
		</button>
		<button class="icon react heart ${ab(liked, 'liked', '')}" 
			action="react-like"
			data-reaction-id="${reaction_id}"
			data-reacting-to="${ev.id}"
			title="$${ab(liked, 'Unlike', 'Like')}">
			<img class="icon svg small ${ab(liked, 'dark-noinvert', '')}" 
				src="$${ab(liked, IMG_EVENT_LIKED, IMG_EVENT_LIKE)}"/>
		</button>`;
	}
	if (!shared) {
		str += html`<button class="icon" title="Share" data-evid="${ev.id}" 
			action="share">
			<img class="icon svg small" src="/icon/event-share.svg"/>
		</button>`;
	}
	str += `
	<button class="icon" title="More Options" action="open-event-options">
		<img class="icon svg small" src="/icon/event-options.svg"/>
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
	src="${get_profile_pic(profile)}"/>`
	//onerror="this.onerror=null;this.src='${IMG_NO_USER}';" 
}

