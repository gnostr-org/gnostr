const KIND_METADATA = 0;
const KIND_NOTE     = 1;
const KIND_RELAY    = 2;
const KIND_CONTACT  = 3;
const KIND_DM       = 4;
const KIND_DELETE   = 5;
const KIND_SHARE    = 6;
const KIND_REACTION = 7;
const KIND_CHATROOM = 42;
const KIND_REPO_ANNOUNCE = 30617;
const KIND_REPO_STATE_ANNOUNCE = 30618;
const KIND_REPO_PATCH = 1617;
const KIND_REPO_PULL_REQ = 1618;
const KIND_REPO_PULL_REQ_UPDATE = 1619;
const KIND_REPO_ISSUE = 1621;
const KIND_REPO_REPLY = 1622;
const KIND_REPO_STATUS_OPEN = 1630;
const KIND_REPO_STATUS_APPLIED = 1631;
const KIND_REPO_STATUS_CLOSED = 1632;
const KIND_REPO_STATUS_DRAFT = 1633;
const KIND_RELAY_LIST = 10002;

const NIP34_KINDS = [
	KIND_REPO_ANNOUNCE,
	KIND_REPO_STATE_ANNOUNCE,
	KIND_REPO_PATCH,
	KIND_REPO_PULL_REQ,
	KIND_REPO_PULL_REQ_UPDATE,
	KIND_REPO_ISSUE,
	KIND_REPO_REPLY,
	KIND_REPO_STATUS_OPEN,
	KIND_REPO_STATUS_APPLIED,
	KIND_REPO_STATUS_CLOSED,
	KIND_REPO_STATUS_DRAFT,
];

const TAG_P = "#p";
const TAG_E = "#e";

const R_HEART = "❤️";
const R_SHAKA = "🤙";

function tag_name(tag) {
	return tag && tag.length > 0 ? tag[0] : "";
}

function tag_value(tag) {
	return tag && tag.length > 1 ? tag[1] : "";
}

function tag_marker(tag) {
	const name = tag_name(tag);
	if (name === "e") {
		return tag && tag.length > 3 ? tag[3] : "";
	}
	if (name === "a") {
		return tag && tag.length > 2 ? tag[2] : "";
	}
	return "";
}

function event_find_tag(tags, name) {
	return (tags || []).find((tag) => tag_name(tag) === name);
}

function event_find_tag_value(tags, name) {
	return tag_value(event_find_tag(tags, name));
}

function event_has_tag(tags, name) {
	return !!event_find_tag(tags, name);
}

function event_is_nip34_kind(kind) {
	return NIP34_KINDS.includes(kind);
}

const STANDARD_KINDS = [
	KIND_NOTE,
	KIND_DM,
	KIND_DELETE,
	KIND_REACTION,
	KIND_SHARE,
        //KIND_REPO_ANNOUNCE,
        //KIND_REPO_STATE_ANNOUNCE,
        //KIND_REPO_PATCH,
        //KIND_REPO_PULL_REQ,
        //KIND_REPO_PULL_REQ_UPDATE,
        //KIND_REPO_ISSUE,
        //KIND_REPO_STATUS_OPEN,
        //KIND_REPO_STATUS_APPLIED,
        //KIND_REPO_STATUS_CLOSED,
        //KIND_REPO_STATUS_DRAFT,
];
const PUBLIC_KINDS = [
	KIND_NOTE,
	KIND_DELETE,
	KIND_REACTION,
	KIND_SHARE,
        KIND_REPO_ANNOUNCE,
        KIND_REPO_STATE_ANNOUNCE,
        KIND_REPO_PATCH,
        KIND_REPO_PULL_REQ,
        KIND_REPO_PULL_REQ_UPDATE,
        KIND_REPO_ISSUE,
        KIND_REPO_REPLY,
        KIND_REPO_STATUS_OPEN,
        KIND_REPO_STATUS_APPLIED,
        KIND_REPO_STATUS_CLOSED,
        KIND_REPO_STATUS_DRAFT,
        KIND_RELAY_LIST,
];

async function broadcast_related_events(ev) {
	ev.tags.reduce((evs, tag) => {
		// cap it at something sane
		if (evs.length >= 5)
			return evs
		const ev = get_tag_event(tag)
		if (!ev)
			return evs
		return evs
	}, [])
	.forEach((ev, i) => {
		// so we don't get rate limited
		setTimeout(() => {
			log_debug("broadcasting related event", ev)
			broadcast_event(ev)
		}, (i+1)*1200)
	});
}

function broadcast_event(ev) {
	GNOSTR.pool.send(["EVENT", ev])
}

async function share(evid) {
	const model = GNOSTR;
	const e = model.all_events[evid];
	if (!e)
		return;
	let ev = {
		kind: KIND_SHARE,
		created_at: new_creation_time(),
		pubkey: model.pubkey,
		content: JSON.stringify(e),
		tags: [["e", e.id], ["p", e.pubkey]],
	}
	ev.id = await nostrjs.calculate_id(ev);
	ev = await sign_event(ev);
	broadcast_event(ev);
	return ev;
}

async function update_profile(profile={}) {
	let ev = {
		kind: KIND_METADATA,
		created_at: new_creation_time(),
		pubkey: GNOSTR.pubkey,
		content: JSON.stringify(profile),
		tags: [],
	};
	ev.id = await nostrjs.calculate_id(ev);
	ev = await sign_event(ev);
	broadcast_event(ev);
	return ev;
}

async function update_contacts() {
	const model = GNOSTR;
	const contacts = Array.from(model.contacts.friends);
	const tags = contacts.map((pubkey) => {
		return ["p", pubkey]
	});
	let ev = {
		kind: KIND_CONTACT,
		created_at: new_creation_time(),
		pubkey: model.pubkey,
		content: "",
		tags: tags,
	}
	ev.id = await nostrjs.calculate_id(ev);
	ev = await sign_event(ev);
	broadcast_event(ev);
	return ev;
}

async function sign_event(ev) {
	if (!(window.nostr && window.nostr.signEvent)) {
		console.error("window.nostr.signEvent is unsupported");
		return;
	}
	const signed = await window.nostr.signEvent(ev)
	if (typeof signed === 'string') {
		ev.sig = signed
		return ev
	}
	return signed
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
		// convert emoji replies into reactions
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
	reply.id = await nostrjs.calculate_id(reply)
	reply = await sign_event(reply)
	return reply
}

async function send_reply(content, replying_to, all=true) {
	const ev = GNOSTR.all_events[replying_to]
	if (!ev)
		return;

	const pubkey = await get_pubkey()
	let reply = await create_reply(pubkey, content, ev, all)

	broadcast_event(reply)
	broadcast_related_events(reply)
}

async function create_deletion_event(pubkey, target, content="") {
	const created_at = Math.floor(new Date().getTime() / 1000)
	let kind = 5

	const tags = [["e", target]]
	let del = { pubkey, tags, content, created_at, kind }

	del.id = await nostrjs.calculate_id(del)
	del = await sign_event(del)
	return del
}

async function delete_post(id, reason) {
	const ev = GNOSTR.all_events[id]
	if (!ev)
		return

	const pubkey = await get_pubkey()
	let del = await create_deletion_event(pubkey, id, reason)
	broadcast_event(del)
}

function model_get_reacts_to(model, pubkey, evid, emoji) {
	const r = model.reactions_to[evid];
	if (!r)
		return;
	for (const id of r.keys()) {
		if (model_is_event_deleted(model, id))
			continue;
		const reaction = model.all_events[id];
		if (!reaction || reaction.pubkey != pubkey)
			continue;
		if (emoji == get_reaction_emoji(reaction))
			return reaction;
	}
	return;
}

function get_reactions(model, evid) {
	const reactions_set = model.reactions_to[evid]
	if (!reactions_set)
		return ""

	let reactions = []
	for (const id of reactions_set.keys()) {
		if (model_is_event_deleted(model, id))
			continue
		const reaction = model.all_events[id]
		if (!reaction)
			continue
		reactions.push(reaction)
	}

	const groups = reactions.reduce((grp, r) => {
		const e = get_reaction_emoji(r)
		grp[e] = grp[e] || {}
		grp[e][r.pubkey] = r
		return grp
	}, {})

	return groups
}

function gather_reply_tags(pubkey, from) {
	let tags = []
	let ids = new Set()

	if (from.refs && from.refs.root) {
		tags.push(["e", from.refs.root, "", "root"])
		ids.add(from.refs.root)
	}

	tags.push(["e", from.id, "", "reply"])
	ids.add(from.id)

	for (const tag of from.tags) {
		if (tag.length >= 2) {
			if (tag_name(tag) === "p" && tag_value(tag) !== pubkey) {
				const value = tag_value(tag);
				if (!ids.has(value)) {
					tags.push(["p", value])
					ids.add(value)
				}
			}
		}
	}
	if (from.pubkey !== pubkey && !ids.has(from.pubkey)) {
		tags.push(["p", from.pubkey])
	}
	return tags
}

function get_tag_event(tag) {
	const model = GNOSTR;
	if (tag.length < 2)
		return null
	if (tag_name(tag) === "e")
		return model.all_events[tag_value(tag)]
	if (tag_name(tag) === "p") {
		let profile = model_get_profile(model, tag_value(tag));
		if (profile.evid)
			return model.all_events[profile.evid];
	}
	return null
}

function* yield_etags(tags) {
	for (const tag of tags) {
		if (tag.length >= 2 && tag_name(tag) === "e")
			yield tag
	}
}

function get_content_warning(tags) {
	for (const tag of tags) {
		if (tag.length >= 1 && tag_name(tag) === "content-warning")
			return tag_value(tag) || ""
	}
	return null
}

// New SID for NIP-34 detail view
const SID_NIP34_DETAIL = "nip34-detail";

async function get_nip05_pubkey(email) {
	const [user, host] = email.split("@")
	const url = `https://${host}/.well-known/nostr.json?name=${user}`
	try {
		const res = await fetch(url)
		const json = await res.json()
		log_debug("nip05 data", json)
		return json.names[user]
	} catch (e) {
		log_error("fetching nip05 entry for %s", email, e)
		throw e
	}
}

async function fetch_repo_events(repo_id, pool, until=Math.floor(Date.now() / 1000)) {
    view_show_spinner(true);
    console.log(`fetch_repo_events: Fetching NIP-34 events for repo_id: ${repo_id}, until: ${until}`);

    let cached_events = [];
    let since_timestamp = 0;

    // Try to load from cache first for initial load (when until is current time)
    if (until === Math.floor(Date.now() / 1000)) {
        console.log(`fetch_repo_events: Attempting to load NIP-34 events from cache for repo_id: ${repo_id}`);
        cached_events = await get_nip34_events_from_db(repo_id, until);
        if (cached_events.length > 0) {
            console.log(`fetch_repo_events: Loaded ${cached_events.length} events from cache.`);
            // Add cached events to model.all_events and trigger a refresh
            for (const ev of cached_events) {
                if (!GNOSTR.all_events[ev.id]) {
                    GNOSTR.all_events[ev.id] = ev;
                    GNOSTR.invalidated.push(ev.id);
                }
            }
            view_timeline_refresh(GNOSTR, VM_NIP34_DETAIL, { repo_id, is_showing_more: true, oldest_event_created_at: until });

            // Determine the 'since' timestamp for fetching newer events from relays
            const latest_cached_event = cached_events.sort((a, b) => b.created_at - a.created_at)[0];
            since_timestamp = latest_cached_event.created_at + 1; // Fetch events newer than the latest cached
            console.log(`fetch_repo_events: Will fetch from relays since: ${since_timestamp}`);
        } else {
            console.log("fetch_repo_events: No NIP-34 events found in cache.");
        }
    }

    const sid = `${SID_NIP34_DETAIL}:${repo_id}`;
    const filter = {
        kinds: NIP34_KINDS,
        "#a": [repo_id],
        limit: 100, // Increased limit for more frequent updates
        until: until, // Always use `until` for fetching older events
    };

    // If we loaded from cache, also fetch newer events from relays using 'since'
    if (since_timestamp > 0) {
        const new_filter = { ...filter, since: since_timestamp, until: undefined }; // Fetch new events up to now
        console.log("fetch_repo_events: Subscription filter for new events:", new_filter);
        pool.subscribe(`${sid}-new`, [new_filter]);
    }

    console.log("fetch_repo_events: Subscription filter for older/initial events:", filter);
    pool.subscribe(sid, [filter]);
    log_debug(`fetching NIP-34 events for repository ${repo_id} with SID ${sid}`);
}
