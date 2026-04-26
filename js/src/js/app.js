async function gnostr_web_start() {
	gnostr_web_init();
	document.addEventListener("click", onclick_any);
}

async function gnostr_web_init() {
	let tries = 0;
	const interval = 20;
	function init() {
		if (window.nostr) {
			log_info("init after", tries);
			gnostr_web_init_ready();
			return;
		}
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
	const model = GNOSTR;

	init_message_textareas();
	init_timeline(model);
	init_search();
	init_my_pfp(model);
	init_postbox(model);
	init_profile();

	view_show_spinner(true);

	await model_load_settings(model);
	init_settings(model);

	const pool = nostrjs.RelayPool(model.relays);
	model.pool = pool;
	pool.on("open", on_pool_open);
	pool.on("event", on_pool_event);
	pool.on("notice", on_pool_notice);
	pool.on("eose", on_pool_eose);
	pool.on("ok", on_pool_ok);

	const { mode, opts, valid } = parse_url_mode();
	view_timeline_apply_mode(model, mode, opts, !valid);
	on_timer_timestamps();
	on_timer_invalidations();
	on_timer_save();
	on_timer_tick();

	setTimeout(() => init_local_relay_sync(), 0);
	await model_load_nip34_events(model);
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
			await decrypt_dms(model);
			view_dm_update(model);
			model.dms_need_redraw = false;
			view_show_spinner(false);
		}
		if (model.invalidated.length > 0) {
			view_timeline_update(model);
		}
		on_timer_invalidations();
	}, 50);
}

function on_timer_save() {
	setTimeout(() => {
		const model = GNOSTR;
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

		const cacheSizeEl = find_node("#nip34-cache-size");
		if (cacheSizeEl) {
			const size = await get_nip34_cache_size();
			log_debug(`NIP-34 Cache Size: ${size}`);
		}

		model.nip34_polling_counter++;
		if (model.nip34_polling_counter % 60 === 0) {
			subscribe_nip34_events(model);
			model.nip34_polling_counter = 0;
		}

		await poll_nip34_from_nip65_relays(model);
		on_timer_tick();
	}, 1 * 1000);
}
