let active_nip34_nip65_subscriptions = new Map();

async function poll_nip34_from_nip65_relays(model) {
	const pubkey = model.pubkey;
	if (!pubkey) {
		log_debug("poll_nip34_from_nip65_relays: No pubkey found, skipping NIP-65 polling.");
		return;
	}

	const nip65_relays_data = await get_nip65_relays_from_db(pubkey);
	const ordered_nip65_relays = sort_relay_pairs_by_ping(model, nip65_relays_data);
	const current_nip65_relay_urls = new Set(nip65_relays_data.map(r => r[0]));

	for (const [relay_url, sub_id] of active_nip34_nip65_subscriptions.entries()) {
		if (!current_nip65_relay_urls.has(relay_url)) {
			log_info(`Unsubscribing from NIP-34 events on removed NIP-65 relay: ${relay_url}`);
			model.pool.unsubscribe(sub_id, relay_url);
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

	for (const [relay_url, policy] of ordered_nip65_relays) {
		if (!policy.read) {
			continue;
		}

		if (active_nip34_nip65_subscriptions.has(relay_url)) {
			model.pool.unsubscribe(active_nip34_nip65_subscriptions.get(relay_url), relay_url);
			active_nip34_nip65_subscriptions.delete(relay_url);
		}

		const sub_id = `${SID_NIP65_RELAY_POLL}:${pubkey}:${relay_url}`;
		const filter = {
			kinds: nip34_kinds,
			limit: 100,
			since: Math.floor(Date.now() / 1000) - (60 * 60 * 24 * 7),
		};

		log_info(`Subscribing to NIP-34 events on NIP-65 relay ${relay_url} with sub_id: ${sub_id}`);
		model.pool.subscribe(sub_id, [filter], relay_url);
		active_nip34_nip65_subscriptions.set(relay_url, sub_id);
	}
}

function on_pool_open(relay) {
	log_info(`OPEN(${relay.url})`);
	const model = GNOSTR;
	const { pubkey } = model;

	fetch_profile_info(pubkey, model.pool, relay);

	relay.subscribe(SID_NOTIFICATIONS, [{
		kinds: PUBLIC_KINDS,
		"#p": [pubkey],
		limit: 5000,
	}]);

	refresh_dm_subscriptions(model);
}

function on_pool_notice(relay, notice) {
	log_info(`NOTICE(${relay.url}): ${notice}`);
}

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
			break;
		case SID_FRIENDS:
			view_timeline_refresh(model);
			break;
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
			view_show_spinner(false);
			pool.unsubscribe(sub_id, relay);
			break;
		case SID_DMS_OUT:
		case SID_DMS_IN:
			break;
	}
}

function on_pool_event(relay, sub_id, ev) {
	const model = GNOSTR;
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
	fetch_event(evid, pool);
	const sid = `${SID_THREAD}:${evid}`;
	pool.subscribe(sid, [{
		kinds: PUBLIC_KINDS,
		"#e": [evid],
	}]);
	log_debug(`fetching thread ${sid}`);
}

function fetch_friends_history(friends, pool, relay) {
	pool.subscribe(SID_FRIENDS, [{
		kinds: PUBLIC_KINDS,
		authors: friends,
		limit: 5000,
	}], relay);
	log_debug(`fetching friends history`);
}

function refresh_dm_subscriptions(model) {
    if (!model || !model.pool || !Array.isArray(model.pool.relays) || !model.pubkey) {
        return;
    }

    const relays = model.pool.relays.slice().sort((left, right) => {
        const left_local = relay_is_local(left.url) ? 0 : 1;
        const right_local = relay_is_local(right.url) ? 0 : 1;
        if (left_local !== right_local) {
            return left_local - right_local;
        }
        const left_ping = relay_ping_sort_value(model, left.url);
        const right_ping = relay_ping_sort_value(model, right.url);
        if (left_ping !== right_ping) {
            return left_ping - right_ping;
        }
        return left.url.localeCompare(right.url);
    });

    for (const relay of relays) {
        if (!relay || typeof relay.subscribe !== "function") {
            continue;
        }
        relay.subscribe(SID_DMS_IN, [{
            kinds: [KIND_DM],
            "#p": [model.pubkey],
            limit: 5000,
        }]);
        relay.subscribe(SID_DMS_OUT, [{
            kinds: [KIND_DM],
            authors: [model.pubkey],
            limit: 5000,
        }]);
    }
}
