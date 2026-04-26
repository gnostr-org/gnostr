async function create_note(pubkey, content) {
	let post = {
		pubkey,
		kind: KIND_NOTE,
		created_at: new_creation_time(),
		content,
		tags: [],
	};
	post.id = await nostrjs.calculate_id(post);
	post = await sign_event(post);
	return post;
}

async function send_note(content) {
	const pubkey = await get_pubkey();
	if (!pubkey) {
		return;
	}

	const post = await create_note(pubkey, content);
	broadcast_event(post);
	model_process_event(GNOSTR, null, post);
	view_timeline_update(GNOSTR);
	view_timeline_show_new(GNOSTR);
	return post;
}

function new_reply_tags(ev) {
	const tags = [["e", ev.id, "", "reply"]];
	if (ev.refs.root) {
		tags.push(["e", ev.refs.root, "", "root"]);
	}
	tags.push(["p", ev.pubkey]);
	return tags;
}

async function create_reply(pubkey, content, ev, all=true) {
	let kind = KIND_NOTE;
	let tags = [];
	if (is_valid_reaction_content(content)) {
		kind = KIND_REACTION;
		tags.push(["e", ev.id], ["p", ev.pubkey]);
	} else {
		tags = all ? gather_reply_tags(pubkey, ev) : new_reply_tags(ev);
	}
	const created_at = new_creation_time();
	let reply = {
		pubkey,
		tags,
		content,
		created_at,
		kind
	};
	reply.id = await nostrjs.calculate_id(reply);
	reply = await sign_event(reply);
	return reply;
}

async function send_reply(content, replying_to, all=true) {
	const ev = GNOSTR.all_events[replying_to];
	if (!ev) {
		return;
	}

	const pubkey = await get_pubkey();
	let reply = await create_reply(pubkey, content, ev, all);

	broadcast_event(reply);
	model_process_event(GNOSTR, null, reply);
	view_timeline_update(GNOSTR);
	view_timeline_show_new(GNOSTR);
	broadcast_related_events(reply);
}
