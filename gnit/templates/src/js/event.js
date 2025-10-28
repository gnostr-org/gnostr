/* EVENT */

/* event_refs_pubkey checks if the event (ev) is directed at the public key 
 * (pubkey) by checking its type, tags, and key.
 */
function event_refs_pubkey(ev, pubkey) {
	if (!(ev.kind === 1 || ev.kind === 42))
		return false
	if (ev.pubkey === pubkey)
		return false
	return event_tags_pubkey(ev, pubkey)
}

function event_contains_pubkey(ev, pubkey) {
	if (ev.pubkey == pubkey)
		return true;
	return event_tags_pubkey(ev, pubkey)
}

function event_tags_pubkey(ev, pubkey) {
	for (const tag of ev.tags) {
		if (tag.length >= 2 && tag[0] == "p" && tag[1] == pubkey)
			return true;
	}
	return false
}

function event_get_tagged_pubkeys(ev) {
	const keys = [];
	for (const tag of ev.tags) {
		if (tag.length >= 2 && tag[0] == "p")
			keys.push(tag[1]);
	}
	return keys;
}

function event_get_pubkeys(ev) {
	const keys = event_get_tagged_pubkeys(ev);
	keys.splice(0, 0, ev.pubkey);
	return keys;
}

function event_get_tag_values(ev) {
	const keys = [];
	for (const tag of ev.tags) {
		if (tag.length >= 2)
			keys.push(tag[1]);
	}
	return keys;
}

function event_calculate_pow(ev) {
	const id_bits = leading_zero_bits(ev.id)
	for (const tag of ev.tags) {
		if (tag.length >= 3 && tag[0] === "nonce") {
			const target = +tag[2]
			if (isNaN(target))
				return 0
			// if our nonce target is smaller than the difficulty,
			// then we use the nonce target as the actual difficulty
			return min(target, id_bits)
		}
	}
	// not a valid pow if we don't have a difficulty target
	return 0
}

/* event_can_reply returns a boolean value based on if you can "reply" to the
 * event in the manner of a chat.
 */
function event_can_reply(ev) {
	return ev.kind === KIND_NOTE; // || ev.kind === KIND_CHATROOM;
}

/* event_is_timeline returns a boolean based on if the event should be rendered
 * in a GUI
 */
function event_is_timeline(ev) {
	return ev.kind === KIND_NOTE || ev.kind === 42 || ev.kind === 6
}	

function event_get_tag_refs(tags) {
	let ids = []
	let pubkeys = []
	let root, reply
	for (const tag of tags) {
		if (tag.length >= 4 && tag[0] == "e") {
			ids.push(tag[1])
			if (tag[3] === "root") {
				root = tag[1]
			} else if (tag[3] === "reply") {
				reply = tag[1]
			}
		} else if (tag.length >= 2 && tag[0] == "e") {
			ids.push(tag[1])
		} else if (tag.length >= 2 && tag[0] == "p") {
			pubkeys.push(tag[1])
		}
	}
	if (!(root && reply) && ids.length > 0) {
		if (ids.length === 1)
			return {root: ids[0], reply: ids[0], pubkeys}
		else if (ids.length >= 2)
			return {root: ids[0], reply: ids[1], pubkeys}
		return {pubkeys}
	}
	return {root, reply, pubkeys}
}

function event_is_spam(ev, contacts, pow) {
	if (contacts.friend_of_friends.has(ev.pubkey))
		return true
	return ev.pow >= pow
}

function event_cmp_created(a={}, b={}) {
	if (a.created_at > b.created_at)
		return 1;
	if (a.created_at < b.created_at)
		return -1;
	return 0;
}

/* event_refs_event checks if event A references event B in its tags.
 */
function event_refs_event(a, b) {
	for (const tag of a.tags) {
		if (tag.length >= 2 && tag[0] === "e" && tag[1] == b.id)
			return true;
	}
	return false;
}

function event_get_last_tags(ev) {
	let o = {};
	for (const tag of ev.tags) {
		if (tag.length >= 2 && (tag[0] === "e" || tag[0] === "p"))
			o[tag[0]] = tag[1];
	}
	return o;
}

/* event_get_reacted_to returns the reacted to event & pubkey (e, p). Returns 
 * undefined if invalid or incomplete.
 */
function event_parse_reaction(ev) {
	if (!is_valid_reaction_content(ev.content) || ev.kind != KIND_REACTION) {
		return;
	}
	const o = event_get_last_tags(ev);	
	if (o["e"]) {
		return o;
	}
}

function event_is_dm(ev, mykey) {
	if (ev.kind != KIND_DM)
		return false;
	if (ev.pubkey != mykey && event_tags_pubkey(ev, mykey))
		return true;
	return ev.pubkey == mykey;
}

function event_is_reply(ev) {
	return !!ev.refs.reply;
}
