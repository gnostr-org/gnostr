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
	init_relays(model);

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

	if (model.local_relay_enabled !== false) {
		setTimeout(() => start_local_relay_sync(), 0);
	}
	await model_load_events(model, (ev) => {
		model_process_event(model, null, ev);
	});
	await model_load_nip34_events(model);
	subscribe_nip34_events(model);

	return pool;
}

function parse_url_mode() {
	const parsed = view_path_to_mode(window.location.pathname);
	if (parsed) {
		if (parsed.mode == VM_SEARCH) {
			const params = new URLSearchParams(window.location.search);
			parsed.opts.query = params.get("search") || "";
		}
		return parsed;
	}
	return {
		mode: VM_FRIENDS,
		opts: {},
		valid: false,
	};
}
