let DAMUS = new_model();

// TODO autogenerate these constants with a bash script
const IMG_EVENT_LIKED = "/icon/event-liked.svg";
const IMG_EVENT_LIKE  = "/icon/event-like.svg";
const IMG_NO_USER     = "/icon/no-user.svg";

const SID_META          = "meta";
const SID_HISTORY       = "hist";
const SID_NOTIFICATIONS = "noti";
const SID_DMS_OUT       = "dout";
const SID_DMS_IN        = "din";
const SID_PROFILES      = "prof";
const SID_THREAD        = "thrd";
const SID_FRIENDS       = "frds";
const SID_EVENT         = "evnt";

// This is our main entry.
// https://developer.mozilla.org/en-US/docs/Web/API/Window/DOMContentLoaded_event
addEventListener('DOMContentLoaded', (ev) => {
	damus_web_init();
	document.addEventListener("click", onclick_any);
});

async function damus_web_init() {
	let tries = 0;
	const interval = 20;
	function init() {
		if (window.nostr) {
			log_info("init after", tries);
			damus_web_init_ready();
			return;
		}
		// TODO if tries is too many say window.nostr not found.
		tries++;
		setTimeout(init, interval);
	}
	init();
}

async function damus_web_init_ready() {
	const model = DAMUS;
	model.pubkey = await get_pubkey(false);

	find_node("#container-busy").classList.add("hide");
	if (!model.pubkey) {
		find_node("#container-welcome").classList.remove("hide");
		return;
	}
	find_node("#container-app").classList.remove("hide");
	webapp_init();
}

async function signin() {
	const model = DAMUS;
	try {
		model.pubkey = await get_pubkey();
	} catch (err) {
		window.alert("An error occured trying to get your public key.");
		return;
	}
	if (!model.pubkey) {
		window.alert("No public key was aquired.");
		return;
	}
	find_node("#container-welcome").classList.add("hide");
	find_node("#container-app").classList.remove("hide");
	await webapp_init();
}

async function webapp_init() {
	let err;
	const model = DAMUS;

	// WARNING Order Matters!
	init_message_textareas();
	init_timeline(model);
	init_my_pfp(model);
	init_postbox(model);
	init_profile();
	view_show_spinner(true);

	// Load data from storage 
	await model_load_settings(model);
	init_settings(model);

	// Create our pool so that event processing functions can work
	const pool = nostrjs.RelayPool(model.relays);
	model.pool = pool
	pool.on("open", on_pool_open);
	pool.on("event", on_pool_event);
	pool.on("notice", on_pool_notice);
	pool.on("eose", on_pool_eose);
	pool.on("ok", on_pool_ok);

	var { mode, opts, valid } = parse_url_mode();
	view_timeline_apply_mode(model, mode, opts, !valid);
	on_timer_timestamps();
	on_timer_invalidations();
	on_timer_save();
	on_timer_tick();
	
	return pool;
}

function parse_url_mode() {
	var mode;
	var valid = true;
	var opts = {};
	var parts = window.location.pathname.split("/").slice(1);
	for (var key in VIEW_NAMES) {
		if (VIEW_NAMES[key].toLowerCase() == parts[0]) {
			mode = key;
			break;
		}
	}
	if (!mode) {
		mode = VM_FRIENDS;
		valid = false;
	}
	switch (mode) {
		case VM_FRIENDS:
			//opts.hide_replys = true;
			break;
		case VM_THREAD:
			opts.thread_id = parts[1];
			break;
		case VM_DM_THREAD:
		case VM_USER:
			opts.pubkey = parts[1];
			break;
	}
	return { mode, opts, valid };
}

function on_timer_timestamps() {
	setTimeout(() => {
		view_timeline_update_timestamps();
		on_timer_timestamps();
	}, 60 * 1000);
}

function on_timer_invalidations() {
	const model = DAMUS;
	setTimeout(async () => {
		if (model.dms_need_redraw && view_get_timeline_el().dataset.mode == VM_DM) {
			// if needs decryption do it
			await decrypt_dms(model);
			view_dm_update(model);
			model.dms_need_redraw = false;
			view_show_spinner(false);
		}
		if (model.invalidated.length > 0)
			view_timeline_update(model);
		on_timer_invalidations();
	}, 50);
}

function on_timer_save() {
	setTimeout(() => {
		const model = DAMUS;
		//model_save_events(model);
		model_save_settings(model);
		on_timer_save();
	}, 1 * 1000);
}

function on_timer_tick() {
	const model = DAMUS;
	setTimeout(async () => {
		update_notifications(model);
		model.relay_que.forEach((que, relay) => {
			model_fetch_next_profile(model, relay);
		});
		on_timer_tick();
	}, 1 * 1000);
}

/* on_pool_open occurs when a relay is opened. It then subscribes for the
 * relative REQ as needed.
 */
function on_pool_open(relay) {
	log_info(`OPEN(${relay.url})`);
	const model = DAMUS;
	const { pubkey } = model;

	// Get all our info & history, well close this after we get  it
	fetch_profile_info(pubkey, model.pool, relay);

	// Get our notifications
	relay.subscribe(SID_NOTIFICATIONS, [{
		kinds: PUBLIC_KINDS,
		"#p": [pubkey],
		limit: 5000,
	}]);

	// Get our dms. You have to do 2 separate queries: ours out and others in
	relay.subscribe(SID_DMS_IN, [{
		kinds: [KIND_DM],
		"#p": [pubkey],
	}]);
	relay.subscribe(SID_DMS_OUT, [{
		kinds: [KIND_DM],
		authors: [pubkey],
	}]);
}

function on_pool_notice(relay, notice) {
	log_info(`NOTICE(${relay.url}): ${notice}`);
}

// on_pool_eose occurs when all storage from a relay has been sent to the 
// client for a labeled (sub_id) REQ.
async function on_pool_eose(relay, sub_id) {
	log_info(`EOSE(${relay.url}): ${sub_id}`);
	const model = DAMUS;
	const { pool } = model;
	const index = sub_id.indexOf(":");
	const sid = sub_id.slice(0, index >= 0 ? index : sub_id.length);
	const identifier = sub_id.slice(index+1);
	switch (sid) {
		case SID_HISTORY:
		case SID_THREAD:
			view_timeline_refresh(model); 
			pool.unsubscribe(sub_id, relay);
			break
		case SID_FRIENDS:
			view_timeline_refresh(model); 
			break
		case SID_META:
			if (model.pubkey == identifier) {
				friends = Array.from(model.contacts.friends);
				friends.push(identifier);
				fetch_friends_history(friends, pool, relay);
				log_debug("Got our friends after no init & fetching our friends");
			}
		case SID_NOTIFICATIONS:
		case SID_PROFILES:
		case SID_EVENT:
			pool.unsubscribe(sub_id, relay);
			break;
		case SID_DMS_OUT:
		case SID_DMS_IN:
			break;
	}
}

function on_pool_event(relay, sub_id, ev) {
	const model = DAMUS;

	// Simply ignore any events that happened in the future.
	if (new Date(ev.created_at * 1000) > new Date()) {
		log_debug(`blocked event caust it was newer`, ev);
		return;	
	}
	model_process_event(model, relay, ev);
}

function on_pool_ok(relay, evid, status) {
	log_debug(`OK(${relay.url}): ${evid} = '${status}'`);
}

function fetch_profiles(pool, relay, pubkeys) {
	log_debug(`(${relay.url}) fetching '${pubkeys.length} profiles'`);
	pool.subscribe(SID_PROFILES, [{
		kinds: [KIND_METADATA],
		authors: pubkeys,
	}], relay);
}

function fetch_profile_info(pubkey, pool, relay) {
	const sid = `${SID_META}:${pubkey}`;
	pool.subscribe(sid, [{
		kinds: [KIND_METADATA, KIND_CONTACT, KIND_RELAY],
		authors: [pubkey],
	}], relay);
	return sid;
}

function fetch_profile(pubkey, pool, relay) {
	fetch_profile_info(pubkey, pool, relay);	
	pool.subscribe(`${SID_HISTORY}:${pubkey}`, [{
		kinds: PUBLIC_KINDS,
		authors: [pubkey],
		limit: 1000,
	}], relay);
}

function fetch_event(evid, pool) {
	const sid = `${SID_EVENT}:${evid}`;
	pool.subscribe(sid, [{
		ids: [evid]
	}]);
	log_debug(`fetching event ${sid}`);
}

function fetch_thread_history(evid, pool) {
	// TODO look up referenced relays for thread history 
	fetch_event(evid, pool);
	const sid = `${SID_THREAD}:${evid}`
	pool.subscribe(sid, [{
		kinds: PUBLIC_KINDS,
		"#e": [evid],
	}]);
	log_debug(`fetching thread ${sid}`);
}

function fetch_friends_history(friends, pool, relay) {
	// TODO fetch history of each friend by their desired relay 
	pool.subscribe(SID_FRIENDS, [{
		kinds: PUBLIC_KINDS,
		authors: friends,
		limit: 5000,
	}], relay);
	log_debug(`fetching friends history`);
}
