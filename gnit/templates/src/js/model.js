/* model_process_event is the main point where events are post-processed from
 * a relay. Additionally other side effects happen such as notification checks
 * and fetching of unknown pubkey profiles.
 */
function model_process_event(model, relay, ev) {
	if (model.all_events[ev.id]) {
		return;
	}

	let fetch_profile = false;
	model.all_events[ev.id] = ev;
	ev.refs = event_get_tag_refs(ev.tags);
	ev.pow = event_calculate_pow(ev);

	// Process specific event needs based on it's kind. Not using a map because
	// integers can't be used.
	let fn;
	switch(ev.kind) {
		case KIND_NOTE:
		case KIND_SHARE:
			fetch_profile = true;
			break;
		case KIND_METADATA:
			fn = model_process_event_metadata;
			break;
		case KIND_CONTACT:
			fn = model_process_event_following;
			break;
		case KIND_DELETE:
			fn = model_process_event_deletion;
			break;
		case KIND_REACTION:
			fn = model_process_event_reaction;
			break;
		case KIND_DM:
			fetch_profile = true;
			fn = model_process_event_dm;
			break;
	}
	if (fn)
		fn(model, ev, !!relay);

	// Queue event for rendering  
	model.invalidated.push(ev.id);

	// If the processing did not come from a relay, but instead storage then
	// let us simply ignore fetching new things.
	if (!relay)
		return;

	// Request new profiles for unseen pubkeys of the event
	if (fetch_profile) {
		event_get_pubkeys(ev).forEach((pubkey) => {
			if (!model_has_profile(model, pubkey)) {
				model_que_profile(model, relay, pubkey);
			}
		});
	}
}

function model_get_relay_que(model, relay) {
	return map_get(model.relay_que, relay, {
		profiles: [],
		timestamp: 0,
		contacts_init: false,
	});
}

function model_que_profile(model, relay, pubkey) {
	const que = model_get_relay_que(model, relay);
	if (que.profiles.indexOf(pubkey) >= 0)
		return;
	que.profiles.push(pubkey);
}

function model_clear_profile_que(model, relay, sid) {
	const que = model_get_relay_que(model, relay);
	//log_debug(`cmp '${que.sid}' vs '${sid}'`);
	if (que.sid != sid)
		return;
	delete que.current;
	delete que.sid;
	que.timestamp = 0;
	log_debug("cleared qued");
}

function model_fetch_next_profile(model, relay) {
	const que = model_get_relay_que(model, relay);

	// Give up on existing case and add it back to the que
	if (que.current) {
		if ((new Date().getTime() - que.timestamp) / 1000 > 30) {
			que.profiles = que.profiles.concat(que.current);
			model.pool.unsubscribe(SID_PROFILES, relay);
			log_debug(`(${relay.url}) gave up on ${que.current.length} profiles`);
		} else {
			return;
		}
	}

	if (que.profiles.length == 0) {
		delete que.current;
		return;
	}
	log_debug(`(${relay.url}) has '${que.profiles.length} left profiles to fetch'`);

	const set = new Set(); 
	let i = 0;
	while (que.profiles.length > 0 && i < 100) {
		set.add(que.profiles.shift());
		i++;
	}
	que.timestamp = new Date().getTime();
	que.current = Array.from(set);
	fetch_profiles(model.pool, relay, que.current);
}

/* model_process_event_profile updates the matching profile with the contents found 
 * in the event.
 */
function model_process_event_metadata(model, ev, update_view) {
	const profile = model_get_profile(model, ev.pubkey);
	const evs = model.all_events;
	if (profile.evid && 
		evs[ev.id].created_at < evs[profile.evid].created_at)
		return;
	profile.evid = ev.id;
	profile.data = safe_parse_json(ev.content, "profile contents");
	if (update_view)
		view_timeline_update_profiles(model, profile.pubkey); 
	// If it's my pubkey let's redraw my pfp that is not located in the view
	// This has to happen regardless of update_view because of the it's not 
	// related to events
	/*if (profile.pubkey == model.pubkey) {
		redraw_my_pfp(model);
	}*/
}

function model_has_profile(model, pk) {
	return !!model_get_profile(model, pk).evid;
}

function model_get_profile(model, pubkey) {
	if (model.profiles.has(pubkey)) {
		return model.profiles.get(pubkey);
	}
	model.profiles.set(pubkey, {
		pubkey: pubkey,
		evid: "",
		relays: [],
		data: {},
	});
	return model.profiles.get(pubkey);
}

function model_process_event_following(model, ev, update_view) {
	contacts_process_event(model.contacts, model.pubkey, ev)
	// TODO support loading relays that are stored on the initial relay
	// I find this wierd since I may never want to do that and only have that
	// information provided by the client - to be better understood
//	load_our_relays(model.pubkey, model.pool, ev)
}

/* model_process_event_dm updates the internal dms hash map based on dms
 * targeted at the user.
 */
function model_process_event_dm(model, ev, update_view) {
	if (!event_is_dm(ev, model.pubkey))
		return;
	// We have to identify who the target DM is for since we are also in the 
	// chat. We simply use the first non-us key we find as the target. I am not
	// sure that multi-sig chats are possible at this time in the spec. If no 
	// target, it's a bad DM.
	let target;
	const keys = event_get_pubkeys(ev);
	for (let key of keys) {
		target = key;
		if (key == model.pubkey)
			continue;
		break;
	}
	if (!target)
		return;
	let dm = model_get_dm(model, target);
	dm.needs_decryption = true;
	dm.needs_redraw = true;
	// It may be faster to not use binary search due to the newest always being
	// at the front - but I could be totally wrong. Potentially it COULD be 
	// slower during history if history is imported ASCENDINGLY. But anything 
	// after this will always be faster and is insurance (race conditions).
	let i = 0;
	for (; i < dm.events.length; i++) {
		const b = dm.events[i];
		if (ev.created_at > b.created_at)
			break;
	}
	dm.events.splice(i, 0, ev);

	// Check if DM is new
	if (ev.created_at > dm.last_viewed) {
		dm.new_count++;
		// dirty hack but works well
		model.dms_need_redraw = true;
	}
}

function model_get_dm(model, target) {
	if (!model.dms.has(target)) {
		// TODO think about using "pubkey:subject" so we have threads
		model.dms.set(target, {
			pubkey: target,
			// events is an ordered list (new to old) of events referenced from
			// all_events. It should not be a copy to reduce memory.
			events: [], 
			// Last read event time by the client/user
			last_viewed: 0, 
			new_count: 0,
			// Notifies the renderer that this dm is out of date
			needs_redraw: false,
			needs_decryption: false,
		});
	}
	return model.dms.get(target);
}

function model_get_dms_seen(model) {
	const obj = {};
	for (let item of model.dms) {
		const dm = item[1];
		obj[dm.pubkey] = dm.last_viewed;
	}
	return obj;
}

function model_set_dms_seen(model, obj={}) {
	for (const pubkey in obj) {
		model_get_dm(model, pubkey).last_viewed = obj[pubkey];
	}
}

function model_dm_seen(model, target) {
	const dm = model_get_dm(model, target);
	if (!dm.events[0])
		return;
	dm.last_viewed = dm.events[0].created_at;
	dm.new_count = 0;
	dm.needs_redraw = true;
}

function model_mark_dms_seen(model) {
	model.dms.forEach((dm) => {
		model_dm_seen(model, dm.pubkey);
	});
	model.dms_need_redraw = true;
}

/* model_process_event_reaction updates the reactions dictionary
 */
function model_process_event_reaction(model, ev, update_view) {
	let reaction = event_parse_reaction(ev);
	if (!reaction) {
		return;
	}
	if (!model.reactions_to[reaction.e])
		model.reactions_to[reaction.e] = new Set();
	model.reactions_to[reaction.e].add(ev.id);	
	if (update_view)
		view_timeline_update_reaction(model, ev);
}

/* event_process_deletion updates the list of deleted events. Additionally
 * pushes event ids onto the invalidated stack for any found.
 */
function model_process_event_deletion(model, ev, update_view) {
	for (const tag of ev.tags) {
		if (tag.length >= 2 && tag[0] === "e" && tag[1]) {
			let evid = tag[1];
			model.invalidated.push(evid);
			model_remove_reaction(model, evid, update_view);
			if (model.deleted[evid])
				continue;
			let ds = model.deletions[evid] =
				(model.deletions[evid] || new Set());
			ds.add(ev.id);
		}
	}
}

function model_remove_reaction(model, evid, update_view) {
	// deleted_ev -> target_ev -> original_ev
	// Here we want to find the original react event to and remove it from our
	// reactions map, then we want to update the element on the page. If the 
	// server does not clean up events correctly the increment/decrement method
	// should work fine in theory.
	const target_ev = model.all_events[evid];
	if (!target_ev)
		return;
	const reaction = event_parse_reaction(target_ev);
	if (!reaction)
		return;
	if (model.reactions_to[reaction.e])
		model.reactions_to[reaction.e].delete(target_ev.id);
	if (update_view)
		view_timeline_update_reaction(model, target_ev);
}

function model_is_event_deleted(model, evid) {
	// we've already know it's deleted
	if (model.deleted[evid])
		return model.deleted[evid]

	const ev = model.all_events[evid]
	if (!ev)
		return false

	// all deletion events
	const ds = model.deletions[ev.id]
	if (!ds)
		return false

	// find valid deletion events
	for (const id of ds.keys()) {
		const d_ev = model.all_events[id]
		if (!d_ev)
			continue
		
		// only allow deletes from the user who created it
		if (d_ev.pubkey === ev.pubkey) {
			model.deleted[ev.id] = d_ev
			delete model.deletions[ev.id]
			return true
		}
	}
	return false
}

function model_has_event(model, evid) {
	return evid in model.all_events
}

function model_events_arr(model) {
	const events = model.all_events;
	let arr = [];
	for (const evid in events) {
		const ev = events[evid];
		const i = arr_bsearch_insert(arr, ev, event_cmp_created); 
		arr.splice(i, 0, ev);
	}
	return arr;
}

function test_model_events_arr() {
	const arr = model_events_arr({all_events: {
		"c": {name: "c", created_at: 2},
		"a": {name: "a", created_at: 0},
		"b": {name: "b", created_at: 1},
		"e": {name: "e", created_at: 4},
		"d": {name: "d", created_at: 3},
	}});
	let last;	
	while(arr.length > 0) {
		let ev = arr.pop();
		log_debug("test:", ev.name, ev.created_at);
		if (!last) {
			last = ev;
			continue;
		}
		if (ev.created_at > last.created_at) {
			log_error(`ev ${ev.name} should be before ${last.name}`);
		}
		last = ev;
	}
}

async function model_save_settings(model) {
	function _settings_save(ev, resolve, reject) {
		const db = ev.target.result;
		const tx = db.transaction("settings", "readwrite");
		const store = tx.objectStore("settings");
		tx.oncomplete = (ev) => {
			db.close();
			resolve();
			//log_debug("settings saved");
		};
		tx.onerror = (ev) => {
			db.close();
			log_error("failed to save events");
			reject(ev);
		};
		store.clear().onsuccess = () => {
			store.put({
				pubkey: model.pubkey,
				notifications_last_viewed: model.notifications.last_viewed,
				relays: Array.from(model.relays),
				dms_seen: model_get_dms_seen(model),
			});
		};
	}
	return dbcall(_settings_save);
}

async function model_load_settings(model) {
	function _settings_load(ev, resolve, reject) {
		const db = ev.target.result;
		const tx = db.transaction("settings", "readonly");
		const store = tx.objectStore("settings");
		const req = store.get(model.pubkey);
		req.onsuccess = (ev) => {
			const settings = ev.target.result;
			if (settings) {
				model.notifications.last_viewed = settings.notifications_last_viewed;
				if (settings.relays.length) 
					model.relays = new Set(settings.relays);
				model.embeds = settings.embeds ? settings.embeds : "friends";
				model_set_dms_seen(model, settings.dms_seen);
			}
			db.close();
			resolve();
			log_debug("Successfully loaded events");
		}
		req.onerror = (ev) => {
			db.close();
			reject(ev);
			log_error("Could not load settings.");
		};
	}
	return dbcall(_settings_load);	
}

async function model_save_events(model) {
	function _events_save(ev, resolve, reject) {
		const db = ev.target.result;
		let tx = db.transaction("events", "readwrite");
		let store = tx.objectStore("events");
		tx.oncomplete = (ev) => {
			db.close();
			resolve();
			//log_debug("saved events!");
		};
		tx.onerror = (ev) => {
			db.close();
			log_error("failed to save events");
			reject(ev);
		};
		store.clear().onsuccess = ()=> {
			for (const evid in model.all_events) {
				store.put(model.all_events[evid]);
			}
		}
	}
	return dbcall(_events_save);
}

async function model_load_events(model, fn) {
	function _events_load(ev, resolve, reject) {
		const db = ev.target.result;
		const tx = db.transaction("events", "readonly");
		const store = tx.objectStore("events");
		const cursor = store.openCursor();
		cursor.onsuccess = (ev) => {
			var cursor = ev.target.result;
			if (cursor) {
				fn(cursor.value);
				cursor.continue();
			} else {
				db.close();
				resolve();
				log_debug("Successfully loaded events");
			}
		}
		cursor.onerror = (ev) => {
			db.close();
			reject(ev);
			log_error("Could not load events.");
		};
	}
	return dbcall(_events_load);
}

function new_model() {
	return {
		all_events: {}, // our master list of all events
		notifications: {
			last_viewed: 0, // time since last looking at notifications
			count: 0, // the number not seen  since last looking
		},
		profiles: new Map(), // pubkey => profile data
		contacts: {
			event: null,
			friends: new Set(),
			friend_of_friends: new Set(),
		},
		dms: new Map(), // pubkey => event list
		invalidated: [], // event ids which should be added/removed
		elements: {}, // map of evid > rendered element
		relay_que: new Map(),
		relays: new Set([
			"wss://nostr.oxtr.dev",
			"wss://relay.damus.io",
			"wss://no.str.cr",
			"wss://relay.snort.social",
			"wss://nos.lol",
			"wss://relay.nostr.lucentlabs.co",
		]),
		
		max_depth: 2,
		reactions_to: {},
		deletions: {},
		deleted: {},
		pow: 0, // pow difficulty target
	};
}
