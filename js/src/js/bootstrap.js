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
