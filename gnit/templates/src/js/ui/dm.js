function view_dm_update(model) {
	const el = find_node("#dms");
	const order = [];
	let reorder = false;
	model.dms.forEach((dm, pubkey, m) => {
		if (!dm.events.length)
			return;
		const i = arr_bsearch_insert(order, dm, dm_cmp);
		order.splice(i, 0, dm);
		if (!dm.needs_redraw)
			return;
		let gel = find_node(`[data-pubkey='${pubkey}']`, el);
		if (!gel) {
			gel = new_el_dmgroup(model, dm);
			gel.addEventListener("click", onclick_dm);
			find_node("img.pfp", gel).addEventListener("error", onerror_pfp);
			el.appendChild(gel);
		}
		update_el_dmgroup(model, dm, gel);
		dm.needs_redraw = false;
		reorder = true;
	});

	// I'm not sure what is faster, doing a frag update all at once OR just
	// updating individual positions. If they all update it's slower, but the 
	// chances of them all updating is is small and only garuenteed when it 
	// draws the first time.
	//const frag = new DocumentFragment();
	if (!reorder) return;
	for (let i = 0; i < order.length; i++) {
		let dm = order[i];
		let xel = el.children[i];
		if (dm.pubkey == xel.dataset.pubkey)
			continue;
		let gel = find_node(`[data-pubkey='${order[i].pubkey}']`, el);
		el.insertBefore(gel, xel);
		//frag.appendChild(gel);
	}
	//el.appendChild(frag);
}

function dm_cmp(a, b) {
	const x = a.events[0].created_at;
	const y = b.events[0].created_at;
	if (x > y)
		return -1;
	if (x < y)
		return 1;
	return 0;
}

function update_el_dmgroup(model, dm, el) {
	const ev = dm.events[0];
	const profile = model_get_profile(model, dm.pubkey);
	const message = ev.decrypted || ev.content || "No Message.";
	const time = fmt_datetime(new Date(ev.created_at * 1000));
	const cel = find_node(".count", el)
	cel.innerText = dm.new_count || dm.events.length;
	cel.classList.toggle("active", dm.new_count > 0);
	find_node(".time", el).innerText = time;
	find_node(".message", el).innerText = message;
	find_node(".username", el).innerText = fmt_name(profile);
}

function new_el_dmgroup(model, dm) {
	const profile = model_get_profile(model, dm.pubkey);
	return html2el(`<div class="dm-group bottom-border clickable" data-pubkey="${dm.pubkey}">
		<div>${render_profile_img(profile, true)}</div>
		<div class="content">
			<div class="count"></div>
			<label class="username" data-pubkey="${dm.pubkey}"></label>
			<p class="message"></p>
			<label class="time"></label>
		</div>
	</div>`);
}

function onclick_dm(ev) {
	const el = find_parent(ev.target, "[data-pubkey]");
	if (!el || !el.dataset.pubkey) {
		log_error("did not find dm pubkey");
		return;
	}
	view_timeline_apply_mode(DAMUS, VM_DM_THREAD, {pubkey: el.dataset.pubkey});
}
