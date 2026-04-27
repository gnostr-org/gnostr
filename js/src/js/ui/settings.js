function init_settings(model) {
	const el = find_node("#settings");
	if (!el) {
		return;
	}
	const pubkey_el = find_node("[name='settings-profile-pubkey']");
	if (pubkey_el && model.pubkey) {
		pubkey_el.textContent = model.pubkey;
	}
	render_settings_profile(model);
}

function init_relays(model) {
	const el = find_node("#relays");
	if (!el) {
		return;
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

    const nip65_relays = await get_nip65_relays_from_db(pubkey);
    const rlist = find_node("#relays #nip65-relay-list tbody");
    if (!rlist) {
        return;
    }
    rlist.innerHTML = ''; // Clear existing NIP-65 relays

    nip65_relays.forEach(([url, policy]) => {
        rlist.appendChild(new_nip65_relay_item(url, policy));
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
	const model = GNOSTR;
	const address = ev.target.dataset.address;

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
    log_info(`Added NIP-65 relay: ${address}`);
}

function new_relay_item(str) {
	const tr = document.createElement('tr');
	tr.innerHTML = `<td><a href="#" class="details-relay" data-address="${str}">${str}</a></td>
	<td>
	<button class="remove-relay btn-text"
		data-address="${str}"
		role="remove-relay">
		<img class="icon svg small" src="/images/event-delete.svg"/>
	</button>
	</td>`;
	find_node(".remove-relay", tr).addEventListener("click", on_click_remove_relay);
	find_node(".details-relay", tr).addEventListener("click", on_click_details_relay);
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
