let GNOSTR = new_model();

// TODO autogenerate these constants with a bash script
const IMG_EVENT_LIKED = "/images/event-liked.svg";
const IMG_EVENT_LIKE  = "/images/event-like.svg";
const IMG_NO_USER     = "/images/no-user.svg";

const SID_META          = "meta";
const SID_HISTORY       = "hist";
const SID_NOTIFICATIONS = "noti";
const SID_DMS_OUT       = "dout";
const SID_DMS_IN        = "din";
const SID_PROFILES      = "prof";
const SID_THREAD        = "thrd";
const SID_FRIENDS       = "frds";
const SID_EVENT         = "evnt";
const SID_NIP34_GLOBAL  = "nip34-global";
const SID_NIP65_RELAY_POLL = "nip65-relay-poll";

// This is our main entry.
// https://developer.mozilla.org/en-US/docs/Web/API/Window/DOMContentLoaded_event
addEventListener('DOMContentLoaded', (ev) => {
	gnostr_web_init();
	document.addEventListener("click", onclick_any);
});

async function gnostr_web_init() {
	let tries = 0;
	const interval = 20;
	function init() {
		if (window.nostr) {
			log_info("init after", tries);
			gnostr_web_init_ready();
			return;
		}
		// TODO if tries is too many say window.nostr not found.
		tries++;
		setTimeout(init, interval);
	}
	init();
}

async function gnostr_web_init_ready() {
	const model = GNOSTR;
	model.pubkey = await get_pubkey(false);

	find_node("#container-busy").classList.add("hide");
	if (!model.pubkey) {
		find_node("#container-welcome").classList.remove("hide");
		return;
	}
	find_node("#app-main").classList.remove("hide");
	webapp_init();
}

async function signin() {
	const model = GNOSTR;
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
	find_node("#app-main").classList.remove("hide");
	await webapp_init();
}

async function webapp_init() {
	let err;
	const model = GNOSTR;

	// WARNING Order Matters!
	init_message_textareas();
	init_timeline(model);
	init_search();
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

	// Start the DB sync in the background after the main UI is initialized
	//await model_load_nip34_events(model);
	setTimeout(() => init_local_relay_sync(), 0);
	await model_load_nip34_events(model);
    // Initial subscription to NIP-34 events
    subscribe_nip34_events(model);
	
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
        case VM_NIP34_DETAIL:
            opts.repo_id = parts[1];
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
	const model = GNOSTR;
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
		const model = GNOSTR;
		//model_save_events(model);
		model_save_settings(model);
		on_timer_save();
	}, 1 * 1000);
}

function on_timer_tick() {
	const model = GNOSTR;
	setTimeout(async () => {
		update_notifications(model);
		model.relay_que.forEach((que, relay) => {
			model_fetch_next_profile(model, relay);
		});

        // Update NIP-34 cache size in the footer
        const cacheSizeEl = find_node("#nip34-cache-size");
        if (cacheSizeEl) {
            const size = await get_nip34_cache_size();
            log_debug(`NIP-34 Cache Size: ${size}`);
        }

        // Periodically poll for NIP-34 events from all connected relays
        model.nip34_polling_counter++;
        if (model.nip34_polling_counter % 60 === 0) { // Every 60 seconds
            subscribe_nip34_events(model);
            model.nip34_polling_counter = 0;
        }

        await poll_nip34_from_nip65_relays(model);

		on_timer_tick();
	}, 1 * 1000);
}

// Stores active NIP-34 subscriptions made from NIP-65 relays
let active_nip34_nip65_subscriptions = new Map(); // Map: relay_url -> sub_id

async function poll_nip34_from_nip65_relays(model) {
    const pubkey = model.pubkey;
    if (!pubkey) {
        log_debug("poll_nip34_from_nip65_relays: No pubkey found, skipping NIP-65 polling.");
        return;
    }

    const nip65_relays_data = await get_nip65_relays_from_db(pubkey);
    const current_nip65_relay_urls = new Set(nip65_relays_data.map(r => r[0]));

    // Unsubscribe from relays that are no longer in the NIP-65 list
    for (const [relay_url, sub_id] of active_nip34_nip65_subscriptions.entries()) {
        if (!current_nip65_relay_urls.has(relay_url)) {
            log_info(`Unsubscribing from NIP-34 events on removed NIP-65 relay: ${relay_url}`);
            model.pool.unsubscribe(sub_id, relay_url); // Unsubscribe from the specific relay
            active_nip34_nip65_subscriptions.delete(relay_url);
        }
    }

    const nip34_kinds = [
        KIND_REPO_ANNOUNCE,
        KIND_REPO_STATE_ANNOUNCE,
        KIND_REPO_PATCH,
        KIND_REPO_PULL_REQ,
        KIND_REPO_PULL_REQ_UPDATE,
        KIND_REPO_ISSUE,
        KIND_REPO_STATUS_OPEN,
        KIND_REPO_STATUS_APPLIED,
        KIND_REPO_STATUS_CLOSED,
        KIND_REPO_STATUS_DRAFT,
    ];

    for (const [relay_url, policy] of nip65_relays_data) {
        // Only subscribe to relays marked for reading
        if (!policy.read) {
            continue;
        }

        // If already subscribed, update the subscription (simple unsubscribe/subscribe for now)
        if (active_nip34_nip65_subscriptions.has(relay_url)) {
            model.pool.unsubscribe(active_nip34_nip65_subscriptions.get(relay_url), relay_url);
            active_nip34_nip65_subscriptions.delete(relay_url);
        }

        const sub_id = `${SID_NIP65_RELAY_POLL}:${pubkey}:${relay_url}`;
        const filter = {
            kinds: nip34_kinds,
            // No authors filter here, as this is for *global* NIP-34 events from these relays
            limit: 100, // Fetch a reasonable number of events
            since: Math.floor(Date.now() / 1000) - (60 * 60 * 24 * 7), // Last 7 days, adjust as needed
        };

        log_info(`Subscribing to NIP-34 events on NIP-65 relay ${relay_url} with sub_id: ${sub_id}`);
        model.pool.subscribe(sub_id, [filter], relay_url); // Subscribe on this specific relay
        active_nip34_nip65_subscriptions.set(relay_url, sub_id);
    }
}

/* on_pool_open occurs when a relay is opened. It then subscribes for the
 * relative REQ as needed.
 */
function on_pool_open(relay) {
	log_info(`OPEN(${relay.url})`);
	const model = GNOSTR;
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
	const model = GNOSTR;
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
		case SID_NIP34_DETAIL:
			console.log(`on_pool_eose: NIP-34 Detail EOSE received for ${sub_id}. Refreshing timeline.`);
			view_timeline_refresh(model);
            view_show_spinner(false); // Hide spinner after refresh
			pool.unsubscribe(sub_id, relay);
			break;
		case SID_DMS_OUT:
		case SID_DMS_IN:
			break;
	}
}

function on_pool_event(relay, sub_id, ev) {
	const model = GNOSTR;

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
		kinds: [KIND_METADATA, KIND_CONTACT, KIND_RELAY, KIND_RELAY_LIST],
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
