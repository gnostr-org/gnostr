function init_settings(model) {
	const el = find_node("#settings");
	if (!el) {
		return;
	}
	const pubkey_el = find_node("[name='settings-profile-pubkey']");
	if (pubkey_el && model.pubkey) {
		pubkey_el.textContent = model.pubkey;
	}
	if (model.pubkey) {
		view_update_cached_active_pfp(model);
	}
	render_settings_profile(model);
}

function format_bytes(bytes) {
	if (bytes == null || Number.isNaN(bytes)) {
		return "unknown";
	}
	if (bytes < 1024) {
		return `${bytes} B`;
	}
	const units = ["KiB", "MiB", "GiB", "TiB"];
	let value = bytes / 1024;
	let unit = 0;
	while (value >= 1024 && unit < units.length - 1) {
		value /= 1024;
		unit++;
	}
	return `${value.toFixed(value >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function ensure_local_relay_header_card() {
	const mount = find_node("#local-relay-header-card-mount");
	if (!mount || mount.querySelector("#local-relay-header-card")) {
		return mount;
	}
	const template = find_node("#local-relay-header-card-template");
	if (!template || !(template instanceof HTMLTemplateElement)) {
		return mount;
	}
	mount.appendChild(template.content.cloneNode(true));
	return mount;
}

async function fetch_relay_discovery() {
	const response = await fetch("/api/relay/discovery", {
		headers: {
			"Accept": "application/json",
		},
	});
	if (!response.ok) {
		throw new Error(`relay discovery request failed with ${response.status}`);
	}
	return await response.json();
}

function init_relays(model) {
	const el = find_node("#relays");
	if (!el) {
		return;
	}
	if (model.pubkey) {
		view_update_cached_active_pfp(model);
	}
	ensure_local_relay_header_card();
	find_node("#add-relay", el).addEventListener("click", on_click_add_relay);
	find_node("#local-relay-start", el).onclick = on_click_start_local_relay_sync;
	find_node("#local-relay-stop", el).onclick = on_click_stop_local_relay_sync;
	const rlist = find_node("#relay-list tbody", el);
	rlist.innerHTML = ''; // Clear existing relays to prevent duplicates
	model.relays.forEach((str) => {
		rlist.appendChild(new_relay_item(str));
	});

	render_relay_dashboard();
	render_local_relay_info();
	render_nip65_relays(model);
}

function render_relay_dashboard() {
    ensure_local_relay_header_card();
    const status = get_local_relay_status();
    const el = find_node("#relays #local-relay-header-card");
    if (!el) {
        return;
    }
    find_node("[data-field='url']", el).textContent = status.url;
    find_node("[data-field='status']", el).textContent = status.connected ? "connected" : "stopped";
    find_node("[data-field='net_io']", el).textContent = `${format_bytes(status.bytes_sent)} out / ${format_bytes(status.bytes_received)} in`;
    find_node("[data-field='disk_usage']", el).textContent = format_bytes(status.disk_usage_bytes);
    find_node("#local-relay-start", el).disabled = status.connected;
    find_node("#local-relay-stop", el).disabled = !status.connected;
    void refresh_local_relay_backend_status();
}

function sync_discovered_relay_state(address, model = GNOSTR) {
	const discovery_list = find_node("#relays #relay-discovery-list");
	if (!discovery_list) {
		return;
	}

    const is_added = model.relays.has(address);
    discovery_list.querySelectorAll("[data-relay-address]").forEach((address_el) => {
        if (address_el.textContent !== address) {
            return;
        }
        const item = address_el.closest("details");
		if (!item) {
			return;
		}
		item.open = false;
		item.dataset.relayState = is_added ? "added" : "available";
		const button = item.querySelector(".add-discovered-relay");
		if (button) {
			button.disabled = is_added;
        }
    });
}

function render_settings_profile(model) {
    const el = find_node("#settings-profile");
    if (!el || !model.pubkey) {
        return;
    }

    const profile = model_get_profile(model, model.pubkey);
    const name = fmt_name(profile);
    const image = find_node("img[name='settings-profile-image']", el);
    image.src = get_profile_pic(profile);
    image.classList.toggle("hide", !profile.data.picture);

    find_node("[name='settings-profile-name']", el).textContent = name;

    const nip05_el = find_node("[name='settings-profile-nip05']", el);
    nip05_el.textContent = profile.data.nip05 || "";
    nip05_el.classList.toggle("hide", !profile.data.nip05);

    const about_el = find_node("[name='settings-profile-about']", el);
    about_el.innerHTML = newlines_to_br(linkify(profile.data.about || ""));
    about_el.classList.toggle("hide", !profile.data.about);

    const pubkey_el = find_node("[name='settings-profile-pubkey']", el);
    pubkey_el.textContent = model.pubkey;
    pubkey_el.style.minHeight = "2.4em";
}

async function render_local_relay_info() {
    ensure_local_relay_header_card();
    const el = find_node("#relays #local-relay-info");
    if (!el) {
        return;
    }
    el.textContent = "Loading relay info...";

    try {
        const url = new URL(local_relay_url);
        const http_url = `http${url.protocol === 'wss:' ? 's' : ''}://${url.host}`;
        const response = await fetch(http_url, {
            headers: {
                'Accept': 'application/nostr+json',
            },
        });
        const data = await response.json();
        el.innerHTML = "";
        const dl = document.createElement("dl");
        for (const key of ["name", "description", "contact", "pubkey", "software", "version", "supported_nips", "supported_nip_extensions"]) {
            if (data[key] == null) {
                continue;
            }
            const dt = document.createElement("dt");
            dt.textContent = key;
            dl.appendChild(dt);

            const dd = document.createElement("dd");
            dd.textContent = Array.isArray(data[key]) ? data[key].join(", ") : data[key];
            dl.appendChild(dd);
        }
        el.appendChild(dl);
    } catch (error) {
        el.textContent = "Unable to load relay info.";
        log_error("Failed to fetch local relay info:", error);
    }
}

async function refresh_local_relay_backend_status() {
    ensure_local_relay_header_card();
    try {
        const response = await fetch("/api/relay/status", {
            headers: {
                "Accept": "application/json",
            },
        });
        if (!response.ok) {
            throw new Error(`relay status request failed with ${response.status}`);
        }
        const data = await response.json();
        local_relay_backend_stats.disk_usage_bytes = data.disk_usage_bytes ?? null;
        const el = find_node("#relays #local-relay-header-card");
        if (el) {
            const status = get_local_relay_status();
            find_node("[data-field='disk_usage']", el).textContent = format_bytes(status.disk_usage_bytes);
        }
    } catch (error) {
        log_warn("Failed to refresh local relay backend status:", error);
    }
}

async function on_click_start_local_relay_sync() {
    await start_local_relay_sync();
    render_relay_dashboard();
}

async function on_click_stop_local_relay_sync() {
    await stop_local_relay_sync();
    render_relay_dashboard();
}

async function render_nip65_relays(model) {
	const pubkey = model.pubkey;
	if (!pubkey) {
		log_warn("render_nip65_relays: No pubkey found, cannot fetch NIP-65 relays.");
		return;
	}

	const [nip65_relays, relay_discovery] = await Promise.all([
		get_nip65_relays_from_db(pubkey),
		fetch_relay_discovery().catch((error) => {
			log_warn("render_nip65_relays: Failed to fetch relay discovery", error);
			return [];
		}),
	]);
	const discovery_by_url = new Map(relay_discovery.map((entry) => [entry.url, entry]));
	model.relay_discovery = relay_discovery;
	render_relay_discovery(model, relay_discovery);
	sort_pool_relays_by_ping(model);
	const timeline = typeof view_get_timeline_el === "function" ? view_get_timeline_el() : null;
	if (model.search_query && timeline && timeline.dataset.mode === VM_SEARCH && typeof refresh_search_subscription === "function") {
		await refresh_search_subscription(model);
	}

	const rlist = find_node("#relays #nip65-relay-list tbody");
	if (!rlist) {
		return;
	}
	rlist.innerHTML = ''; // Clear existing NIP-65 relays

	nip65_relays
		.slice()
		.sort((left, right) => {
			const left_nip34 = relay_supports_nip34(model, left[0]) ? 1 : 0;
			const right_nip34 = relay_supports_nip34(model, right[0]) ? 1 : 0;
			if (right_nip34 !== left_nip34) {
				return right_nip34 - left_nip34;
			}
			const left_score = discovery_score(discovery_by_url.get(left[0]));
			const right_score = discovery_score(discovery_by_url.get(right[0]));
			if (right_score !== left_score) {
				return right_score - left_score;
			}
			return left[0].localeCompare(right[0]);
		})
		.forEach(([url, policy]) => {
			rlist.appendChild(new_nip65_relay_item(url, policy, discovery_by_url.get(url)));
		});
}

function discovery_score(entry) {
	if (!entry || !Array.isArray(entry.supported_nips)) {
		return 0;
	}
	return entry.supported_nips.length;
}

function render_relay_discovery(model, relay_discovery) {
	const rlist = find_node("#relays #relay-discovery-list");
	if (!rlist) {
		return;
	}

	rlist.innerHTML = '';
	if (!relay_discovery.length) {
		const empty = document.createElement('div');
		empty.className = 'relay-discovery-empty';
		empty.textContent = 'No crawler relay discovery available.';
		rlist.appendChild(empty);
		return;
	}

	relay_discovery
		.slice()
		.sort((left, right) => {
			const left_nip34 = relay_supports_nip34(model, left.url) ? 1 : 0;
			const right_nip34 = relay_supports_nip34(model, right.url) ? 1 : 0;
			if (right_nip34 !== left_nip34) {
				return right_nip34 - left_nip34;
			}
			const left_score = discovery_score(left);
			const right_score = discovery_score(right);
			if (right_score !== left_score) {
				return right_score - left_score;
			}
			return left.url.localeCompare(right.url);
		})
		.forEach((entry) => {
			rlist.appendChild(new_relay_discovery_item(entry, model));
		});
}

function new_nip65_relay_item(url, policy, discovery_entry) {
	const policy_str = Object.keys(policy).length === 0 ? "" :
		(policy.read && policy.write ? "read/write" :
		(policy.read ? "read" : "write"));
	const template = find_node("#nip65-relay-template");
	let tr = null;
	if (template instanceof HTMLTemplateElement && template.content.firstElementChild) {
		tr = template.content.firstElementChild.cloneNode(true);
	} else {
		tr = document.createElement("tr");
		tr.className = "relay-card nip65-relay";
		const td_address = document.createElement("td");
		td_address.className = "relay-card-main nip65-relay-address";
		td_address.innerHTML = `<a href="#" class="details-relay" data-address=""><span data-relay-address></span></a>`;
		const td_policy = document.createElement("td");
		td_policy.className = "nip65-relay-policy";
		td_policy.setAttribute("data-relay-policy", "");
		const td_action = document.createElement("td");
		td_action.className = "relay-card-action nip65-relay-action";
		td_action.innerHTML = `<button class="add-nip65-relay btn-text" data-address="" role="add-nip65-relay">Add</button>`;
		tr.append(td_address, td_policy, td_action);
	}
	tr.classList.add("nip65-relay");
	if (policy.read && !policy.write) {
		tr.classList.add("read-only-relay");
	}
	const link = find_node(".details-relay", tr);
	const address = find_node("[data-relay-address]", tr);
	const policy_el = find_node("[data-relay-policy]", tr);
	const name_el = find_node("[data-relay-name]", tr);
	const software_el = find_node("[data-relay-software]", tr);
	const nips_el = find_node("[data-relay-nips]", tr);
	const description_el = find_node("[data-relay-description]", tr);
	const button = find_node(".add-nip65-relay", tr);
	if (link) {
		link.dataset.address = url;
		link.setAttribute("data-address", url);
		link.addEventListener("click", on_click_details_relay);
	}
	if (address) {
		address.textContent = url;
	}
	if (policy_el) {
		policy_el.textContent = policy_str;
	}
	if (name_el) {
		const label = discovery_entry?.name || discovery_entry?.pubkey || url;
		name_el.textContent = label;
		name_el.classList.toggle("hide", !label);
	}
	if (software_el) {
		const software_bits = [];
		if (discovery_entry?.software) {
			software_bits.push(discovery_entry.software);
		}
		if (discovery_entry?.version) {
			software_bits.push(discovery_entry.version);
		}
		software_el.textContent = software_bits.join(" ");
		software_el.classList.toggle("hide", !software_bits.length);
	}
	if (nips_el) {
		const supported_nips = Array.isArray(discovery_entry?.supported_nips) ? discovery_entry.supported_nips : [];
		nips_el.textContent = supported_nips.length ? `Supports NIPs: ${supported_nips.join(", ")}` : "";
		nips_el.classList.toggle("hide", !supported_nips.length);
	}
	if (description_el) {
		description_el.textContent = discovery_entry?.description || "";
		description_el.classList.toggle("hide", !discovery_entry?.description);
	}
	if (button) {
		button.dataset.address = url;
		button.setAttribute("data-address", url);
		button.addEventListener("click", on_click_add_nip65_relay);
	}
	return tr;
}

function on_click_add_nip65_relay(ev) {
	ev.preventDefault();
	ev.stopPropagation();
	add_relay_address(ev.currentTarget.dataset.address, "NIP-65 relay");
}

function on_click_add_discovered_relay(ev) {
	ev.preventDefault();
	ev.stopPropagation();
	add_relay_address(ev.currentTarget.dataset.address, "discovered relay");
}

function add_relay_address(address, label) {
	const model = GNOSTR;

	if (model.relays.has(address)) {
		log_info(`Relay ${address} is already in the active list.`);
		return;
	}

	if (!model.pool.add(address)) {
		log_error(`Failed to add relay ${address} to pool.`);
		return;
	}
	model.relays.add(address);
	sort_pool_relays_by_ping(model);
	find_node("#relay-list tbody").appendChild(new_relay_item(address));
	sync_discovered_relay_state(address, model);
	model_save_settings(model);
	log_info(`Added ${label}: ${address}`);
}

function new_relay_item(str) {
	const template = find_node("#relay-card-template");
	let tr = null;
	if (template instanceof HTMLTemplateElement && template.content.firstElementChild) {
		tr = template.content.firstElementChild.cloneNode(true);
	} else {
		tr = document.createElement("tr");
		const cell = document.createElement("td");
		cell.colSpan = 2;
		cell.innerHTML = `
			<details class="relay-active-card">
				<summary class="relay-active-summary">
					<span class="relay-active-summary-text">
						<span class="relay-active-summary-name" data-relay-name></span>
						<span class="relay-active-summary-address" data-relay-address></span>
						<span class="relay-active-summary-ping" data-relay-ping></span>
						<img class="relay-active-summary-nip34 relay-active-summary-nip34-light icon svg small hide" src="/images/logo.svg" alt="NIP-34"/>
						<img class="relay-active-summary-nip34 relay-active-summary-nip34-dark icon svg small hide" src="/images/logo-inverted.svg" alt="NIP-34"/>
					</span>
					<button class="remove-relay btn-text" data-address="" role="remove-relay">
						<img class="icon svg small" src="/images/event-delete.svg"/>
					</button>
				</summary>
				<div class="relay-info">
					<div class="relay-info-line relay-info-name" data-relay-name></div>
					<div class="relay-info-line relay-info-ping hide" data-relay-ping></div>
					<div class="relay-info-line relay-info-software hide" data-relay-software></div>
					<div class="relay-info-line relay-info-nips hide" data-relay-nips></div>
					<div class="relay-info-line relay-info-description hide" data-relay-description></div>
				</div>
			</details>`;
		tr.append(cell);
	}
	tr.classList.add("relay-card");
	const card = find_node(".relay-active-card", tr);
	if (card) {
		card.open = false;
	}
	const remove = find_node(".remove-relay", tr);
	if (remove) {
		remove.dataset.address = str;
		remove.setAttribute("data-address", str);
		remove.addEventListener("click", on_click_remove_relay);
	}
	const link = find_node(".details-relay", tr);
	const address = find_node("[data-relay-address]", tr);
	if (link) {
		link.dataset.address = str;
		link.setAttribute("data-address", str);
		link.addEventListener("click", on_click_details_relay);
	}
	if (address) {
		address.textContent = str;
	}
	void hydrate_relay_item(tr, str);
	return tr;
}

async function hydrate_relay_item(tr, address) {
	const name_els = tr.querySelectorAll("[data-relay-name]");
	const ping_els = tr.querySelectorAll("[data-relay-ping]");
	const software_els = tr.querySelectorAll("[data-relay-software]");
	const nips_els = tr.querySelectorAll("[data-relay-nips]");
	const description_els = tr.querySelectorAll("[data-relay-description]");
	const nip34_icons = tr.querySelectorAll(".relay-active-summary-nip34");
	if (!name_els.length || !ping_els.length || !software_els.length || !nips_els.length || !description_els.length) {
		return;
	}

	try {
		const started = performance.now();
		const url = new URL(address);
		const http_url = `http${url.protocol === 'wss:' ? 's' : ''}://${url.host}`;
		const response = await fetch(http_url, {
			headers: {
				'Accept': 'application/nostr+json',
			},
		});
		const ping_ms = Math.round(performance.now() - started);
		const data = await response.json();
		const label = data.name || data.pubkey || address;
		name_els.forEach((el) => {
			el.textContent = label;
			el.classList.toggle("hide", !label);
		});
		ping_els.forEach((el) => {
			el.textContent = `Ping ${ping_ms} ms`;
			el.classList.toggle("hide", false);
		});

		const software_bits = [];
		if (data.software) {
			software_bits.push(data.software);
		}
		if (data.version) {
			software_bits.push(data.version);
		}
		software_els.forEach((el) => {
			el.textContent = software_bits.length ? software_bits.join(" ") : "";
			el.classList.toggle("hide", !software_bits.length);
		});

		const supported_nips = Array.isArray(data.supported_nips) ? data.supported_nips : [];
		nips_els.forEach((el) => {
			el.textContent = supported_nips.length ? `Supports NIPs: ${supported_nips.join(", ")}` : "";
			el.classList.toggle("hide", !supported_nips.length);
		});
		nip34_icons.forEach((icon) => {
			icon.classList.toggle("hide", !supported_nips.includes(34));
		});

		const description = data.description || "";
		description_els.forEach((el) => {
			el.textContent = description;
			el.classList.toggle("hide", !description);
		});
		tr.dataset.relayName = label;
		tr.dataset.relayPingMs = String(ping_ms);
		tr.dataset.relaySoftware = software_bits.join(" ");
		tr.dataset.relayNips = supported_nips.join(", ");
		sort_pool_relays_by_ping(GNOSTR);
	} catch (error) {
		name_el.textContent = "Unable to load relay info";
		software_el.textContent = "";
		ping_el.textContent = "";
		ping_el.classList.add("hide");
		software_el.classList.add("hide");
		nips_el.textContent = "";
		nips_el.classList.add("hide");
		description_el.textContent = "";
		description_el.classList.add("hide");
		log_warn(`Failed to fetch relay info for ${address}:`, error);
	}
}

function new_relay_discovery_item(entry, model) {
	const template = find_node("#relay-discovery-template");
	let item = null;
	if (template instanceof HTMLTemplateElement && template.content.firstElementChild) {
		item = template.content.firstElementChild.cloneNode(true);
	} else {
		item = document.createElement('details');
		item.className = 'relay-card relay-discovery-card';
		const summary = document.createElement('summary');
		summary.className = 'relay-discovery-summary';
		summary.innerHTML = `<a href="#" class="details-relay relay-address" data-address=""><span data-relay-address></span></a><img class="relay-discovery-nip34 relay-discovery-nip34-light icon svg small hide" src="/images/logo.svg" alt="NIP-34"/><img class="relay-discovery-nip34 relay-discovery-nip34-dark icon svg small hide" src="/images/logo-inverted.svg" alt="NIP-34"/><button class="add-discovered-relay btn-text" data-address="" role="add-discovered-relay" aria-label="Add relay" title="Add relay"><img class="icon svg small relay-add-icon" src="/images/add-relay.svg"/></button>`;
		const body = document.createElement('div');
		body.className = 'relay-discovery-body';
		body.innerHTML = `<div class="relay-info"><div class="relay-info-line relay-info-name" data-relay-name></div><div class="relay-info-line relay-info-ping hide" data-relay-ping></div><div class="relay-info-line relay-info-software hide" data-relay-software></div><div class="relay-info-line relay-info-nips hide" data-relay-nips></div><div class="relay-info-line relay-info-description hide" data-relay-description></div></div>`;
		item.append(summary, body);
	}
	item.classList.add('relay-card', 'relay-discovery-card');
	item.open = false;
	const supported_nips = Array.isArray(entry.supported_nips) ? entry.supported_nips : [];

	const address = find_node("[data-relay-address]", item);
	const link = find_node(".details-relay", item);
	const name_el = find_node("[data-relay-name]", item);
	const ping_el = find_node("[data-relay-ping]", item);
	const software_el = find_node("[data-relay-software]", item);
	const nips_el = find_node("[data-relay-nips]", item);
	const description_el = find_node("[data-relay-description]", item);
	const nip34_icons = item.querySelectorAll(".relay-discovery-nip34");
	const button = find_node(".add-discovered-relay", item);
	if (address) {
		address.textContent = entry.url;
	}
	if (link) {
		link.dataset.address = entry.url;
		link.setAttribute("data-address", entry.url);
		link.addEventListener("click", on_click_details_relay);
	}
	if (name_el) {
		name_el.textContent = entry.name || entry.pubkey || entry.url;
		name_el.classList.toggle("hide", !name_el.textContent);
	}
	if (ping_el) {
		ping_el.textContent = Number.isFinite(entry.ping_ms) ? `Ping: ${entry.ping_ms} ms` : "";
		ping_el.classList.toggle("hide", !ping_el.textContent);
	}
	if (software_el) {
		const software_bits = [];
		if (entry.software) {
			software_bits.push(entry.software);
		}
		if (entry.version) {
			software_bits.push(entry.version);
		}
		software_el.textContent = software_bits.join(" ");
		software_el.classList.toggle("hide", !software_bits.length);
	}
	if (nips_el) {
		nips_el.textContent = supported_nips.length ? `Supports NIPs: ${supported_nips.join(", ")}` : "";
		nips_el.classList.toggle("hide", !supported_nips.length);
	}
	nip34_icons.forEach((icon) => {
		icon.classList.toggle("hide", !supported_nips.includes(34));
	});
	if (description_el) {
		description_el.textContent = entry.description || "";
		description_el.classList.toggle("hide", !entry.description);
	}
	if (button) {
		button.dataset.address = entry.url;
		button.setAttribute("data-address", entry.url);
		button.disabled = model.relays.has(entry.url);
		button.setAttribute("aria-label", "Add relay");
		button.setAttribute("title", "Add relay");
		button.addEventListener('click', on_click_add_discovered_relay);
	}
	sync_discovered_relay_state(entry.url, model);

	return item;
}

function on_click_add_relay(ev) {
	const model = GNOSTR;
	const address = prompt("Please provide a websocket address:", "wss://");
	log_debug("got address", address);
	// TODO add relay validation
	if (!model.pool.add(address))
		return;
	model.relays.add(address);
	find_node("#relays #relay-list tbody").appendChild(new_relay_item(address));
	sync_discovered_relay_state(address, model);
	model_save_settings(model);
}

function on_click_remove_relay(ev) {
	ev.preventDefault();
	ev.stopPropagation();
	const model = GNOSTR;
	const address = ev.currentTarget.dataset.address;
	if (!model.pool.remove(address))
		return;
	model.relays.delete(address);
	sort_pool_relays_by_ping(model);
	let parent = ev.target;
	while (parent) {
		if (parent.matches("tr")) {
			parent.parentElement.removeChild(parent);
			break;
		}
		parent = parent.parentElement;
	}
	sync_discovered_relay_state(address, model);
	model_save_settings(model);
}

async function on_click_details_relay(ev) {
	ev.preventDefault();
	ev.stopPropagation();
	const address = ev.currentTarget.dataset.address;
	const url = new URL(address);
	const http_url = `http${url.protocol === 'wss:' ? 's' : ''}://${url.host}`;

	try {
		const response = await fetch(http_url, {
			headers: {
				'Accept': 'application/nostr+json',
			},
		});
		const data = await response.json();
		render_relay_details(data, ev.target);
	} catch (error) {
		log_error(`Failed to fetch relay details for ${address}:`, error);
	}
}

function render_relay_details(data, target_element) {
	let parent = target_element;
	while (parent) {
		if (parent.matches("tr") || parent.matches("details")) {
			const is_already_open = parent.nextElementSibling && parent.nextElementSibling.classList.contains('relay-details');

			// Close all open details
			const all_details = document.querySelectorAll('.relay-details');
			all_details.forEach(row => row.remove());

			// If it wasn't already open, open it now.
			if (!is_already_open) {
				const details_row = parent.matches("tr") ? document.createElement('tr') : document.createElement('div');
				details_row.classList.add('relay-details');
				let details_body = details_row;
				if (parent.matches("tr")) {
					const td = document.createElement('td');
					td.colSpan = 2;
					details_row.appendChild(td);
					details_body = td;
				}
				parent.insertAdjacentElement('afterend', details_row);

				const dl = document.createElement('dl');
				for (const key in data) {
					const dt = document.createElement('dt');
					dt.textContent = key;
					dl.appendChild(dt);
					const dd = document.createElement('dd');
					if (key === 'supported_nips' && Array.isArray(data[key])) {
						dd.textContent = data[key].join(', ');
					} else if (typeof data[key] === 'object' && data[key] !== null) {
						const innerDl = document.createElement('dl');
						for (const innerKey in data[key]) {
							const innerDt = document.createElement('dt');
							innerDt.textContent = innerKey;
							innerDl.appendChild(innerDt);
							const innerDd = document.createElement('dd');
							innerDd.textContent = JSON.stringify(data[key][innerKey], null, 2);
							innerDl.appendChild(innerDd);
						}
						dd.appendChild(innerDl);
					} else {
						dd.textContent = data[key];
					}
					dl.appendChild(dd);
				}
				details_body.appendChild(dl);
			}
			break;
		}
		parent = parent.parentElement;
	}
}
