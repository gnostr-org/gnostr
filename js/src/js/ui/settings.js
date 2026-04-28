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
    const status = get_local_relay_status();
    const el = find_node("#relays #local-relay-dashboard");
    if (!el) {
        return;
    }
    find_node("[data-field='url']", el).textContent = status.url;
    find_node("[data-field='status']", el).textContent = status.connected ? "connected" : "stopped";
    find_node("[data-field='sent']", el).textContent = String(status.sent);
    find_node("[data-field='received']", el).textContent = String(status.received);
    find_node("[data-field='errors']", el).textContent = String(status.errors);
    find_node("[data-field='last_error']", el).textContent = status.last_error || "none";
    find_node("#local-relay-start", el.parentElement).disabled = status.connected;
    find_node("#local-relay-stop", el.parentElement).disabled = !status.connected;
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
        for (const key of ["name", "description", "pubkey", "software", "version", "supported_nips"]) {
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
	render_relay_discovery(model, relay_discovery);

	const rlist = find_node("#relays #nip65-relay-list tbody");
	if (!rlist) {
		return;
	}
	rlist.innerHTML = ''; // Clear existing NIP-65 relays

	nip65_relays
		.slice()
		.sort((left, right) => {
			const left_score = discovery_score(discovery_by_url.get(left[0]));
			const right_score = discovery_score(discovery_by_url.get(right[0]));
			if (right_score !== left_score) {
				return right_score - left_score;
			}
			return left[0].localeCompare(right[0]);
		})
		.forEach(([url, policy]) => {
			rlist.appendChild(new_nip65_relay_item(url, policy));
		});
}

function discovery_score(entry) {
	if (!entry || !Array.isArray(entry.supported_nips)) {
		return 0;
	}
	return entry.supported_nips.length;
}

function render_relay_discovery(model, relay_discovery) {
	const rlist = find_node("#relays #relay-discovery-list tbody");
	if (!rlist) {
		return;
	}

	rlist.innerHTML = '';
	if (!relay_discovery.length) {
		const tr = document.createElement('tr');
		const td = document.createElement('td');
		td.colSpan = 4;
		td.textContent = 'No crawler relay discovery available.';
		tr.appendChild(td);
		rlist.appendChild(tr);
		return;
	}

	relay_discovery
		.slice()
		.sort((left, right) => {
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

function new_nip65_relay_item(url, policy) {
    const is_read_only = policy.read && !policy.write;
    const tr = document.createElement('tr');
    if (is_read_only) {
        tr.classList.add('read-only-relay');
    }
    const policy_str = Object.keys(policy).length === 0 ? "" :
                       (policy.read && policy.write ? "read/write" :
                       (policy.read ? "read" : "write"));
    tr.innerHTML = `<td><a href="#" class="details-relay" data-address="${url}">${url}</a></td>
    <td>${policy_str}</td>
    <td>
    <button class="add-nip65-relay btn-text"
		data-address="${url}"
		role="add-nip65-relay">
		Add
	</button>
    </td>`;
	find_node("button", tr).addEventListener("click", on_click_add_nip65_relay);
	find_node(".details-relay", tr).addEventListener("click", on_click_details_relay);
	return tr;
}

function on_click_add_nip65_relay(ev) {
	add_relay_address(ev.target.dataset.address, "NIP-65 relay");
}

function on_click_add_discovered_relay(ev) {
	add_relay_address(ev.target.dataset.address, "discovered relay");
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
	find_node("#relay-list tbody").appendChild(new_relay_item(address));
	model_save_settings(model);
	log_info(`Added ${label}: ${address}`);
}

function new_relay_item(str) {
	const tr = document.createElement('tr');
	tr.classList.add("relay-card");
	tr.innerHTML = `
		<td class="relay-card-main">
			<a href="#" class="details-relay relay-address" data-address="${str}">${str}</a>
			<div class="relay-info">
				<div class="relay-info-line relay-info-name" data-relay-name>Loading relay info...</div>
				<div class="relay-info-line relay-info-software hide" data-relay-software></div>
				<div class="relay-info-line relay-info-nips hide" data-relay-nips></div>
				<div class="relay-info-line relay-info-description hide" data-relay-description></div>
			</div>
		</td>
		<td class="relay-card-action">
			<button class="remove-relay btn-text"
				data-address="${str}"
				role="remove-relay">
				<img class="icon svg small" src="/images/event-delete.svg"/>
			</button>
		</td>`;
	find_node(".remove-relay", tr).addEventListener("click", on_click_remove_relay);
	find_node(".details-relay", tr).addEventListener("click", on_click_details_relay);
	void hydrate_relay_item(tr, str);
	return tr;
}

async function hydrate_relay_item(tr, address) {
	const name_el = find_node("[data-relay-name]", tr);
	const software_el = find_node("[data-relay-software]", tr);
	const nips_el = find_node("[data-relay-nips]", tr);
	const description_el = find_node("[data-relay-description]", tr);
	if (!name_el || !software_el || !nips_el || !description_el) {
		return;
	}

	try {
		const url = new URL(address);
		const http_url = `http${url.protocol === 'wss:' ? 's' : ''}://${url.host}`;
		const response = await fetch(http_url, {
			headers: {
				'Accept': 'application/nostr+json',
			},
		});
		const data = await response.json();
		const label = data.name || data.pubkey || address;
		name_el.textContent = label;
		name_el.classList.toggle("hide", !label);

		const software_bits = [];
		if (data.software) {
			software_bits.push(data.software);
		}
		if (data.version) {
			software_bits.push(data.version);
		}
		software_el.textContent = software_bits.length ? software_bits.join(" ") : "";
		software_el.classList.toggle("hide", !software_bits.length);

		const supported_nips = Array.isArray(data.supported_nips) ? data.supported_nips : [];
		nips_el.textContent = supported_nips.length ? `Supports NIPs: ${supported_nips.join(", ")}` : "";
		nips_el.classList.toggle("hide", !supported_nips.length);

		const description = data.description || "";
		description_el.textContent = description;
		description_el.classList.toggle("hide", !description);
		tr.dataset.relayName = label;
		tr.dataset.relaySoftware = software_bits.join(" ");
		tr.dataset.relayNips = supported_nips.join(", ");
	} catch (error) {
		name_el.textContent = "Unable to load relay info";
		software_el.textContent = "";
		software_el.classList.add("hide");
		nips_el.textContent = "";
		nips_el.classList.add("hide");
		description_el.textContent = "";
		description_el.classList.add("hide");
		log_warn(`Failed to fetch relay info for ${address}:`, error);
	}
}

function new_relay_discovery_item(entry, model) {
	const tr = document.createElement('tr');
	const supported_nips = Array.isArray(entry.supported_nips) ? entry.supported_nips : [];
	const meta_bits = [];
	if (entry.name) {
		meta_bits.push(entry.name);
	}
	if (entry.software) {
		meta_bits.push(entry.version ? `${entry.software} ${entry.version}` : entry.software);
	} else if (entry.version) {
		meta_bits.push(entry.version);
	}

	const td_url = document.createElement('td');
	const link = document.createElement('a');
	link.href = '#';
	link.className = 'details-relay';
	link.dataset.address = entry.url;
	link.textContent = entry.url;
	td_url.appendChild(link);
	tr.appendChild(td_url);

	const td_kinds = document.createElement('td');
	td_kinds.title = supported_nips.length ? supported_nips.join(', ') : 'No supported NIPs reported';
	td_kinds.textContent = supported_nips.length ? `${supported_nips.length} kind${supported_nips.length === 1 ? '' : 's'}` : '—';
	tr.appendChild(td_kinds);

	const td_meta = document.createElement('td');
	if (meta_bits.length) {
		const meta_line = document.createElement('div');
		meta_line.textContent = meta_bits.join(' · ');
		td_meta.appendChild(meta_line);
	}
	if (entry.description) {
		const desc_line = document.createElement('div');
		desc_line.textContent = entry.description;
		td_meta.appendChild(desc_line);
	}
	if (Array.isArray(entry.supported_nip_extensions) && entry.supported_nip_extensions.length) {
		const ext_line = document.createElement('div');
		ext_line.textContent = `Extensions: ${entry.supported_nip_extensions.join(', ')}`;
		td_meta.appendChild(ext_line);
	}
	tr.appendChild(td_meta);

	const td_action = document.createElement('td');
	const button = document.createElement('button');
	button.className = 'add-nip65-relay btn-text';
	button.dataset.address = entry.url;
	button.textContent = model.relays.has(entry.url) ? 'Added' : 'Add';
	button.disabled = model.relays.has(entry.url);
	button.setAttribute('role', 'add-discovered-relay');
	button.addEventListener('click', on_click_add_discovered_relay);
	td_action.appendChild(button);
	tr.appendChild(td_action);

	return tr;
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
	model_save_settings(model);
}

function on_click_remove_relay(ev) {
	const model = GNOSTR;
	const address = ev.target.dataset.address;
	if (!model.pool.remove(address))
		return;
	model.relays.delete(address);
	let parent = ev.target;
	while (parent) {
		if (parent.matches("tr")) {
			parent.parentElement.removeChild(parent);
			break;
		}
		parent = parent.parentElement;
	}
	model_save_settings(model);
}

async function on_click_details_relay(ev) {
	const address = ev.target.dataset.address;
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
		if (parent.matches("tr")) {
			const is_already_open = parent.nextElementSibling && parent.nextElementSibling.classList.contains('relay-details');

			// Close all open details
			const all_details = document.querySelectorAll('.relay-details');
			all_details.forEach(row => row.remove());

			// If it wasn't already open, open it now.
			if (!is_already_open) {
				let details_row = document.createElement('tr');
				details_row.classList.add('relay-details');
				const td = document.createElement('td');
				td.colSpan = 2;
				details_row.appendChild(td);
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
				td.appendChild(dl);
			}
			break;
		}
		parent = parent.parentElement;
	}
}
