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

		const relaysEl = find_node("#relays");
		if (relaysEl && !relaysEl.classList.contains("hide") &&
			typeof render_relay_dashboard === 'function') {
			render_relay_dashboard();
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
