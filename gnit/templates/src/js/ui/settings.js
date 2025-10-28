function init_settings(model) {
	const el = find_node("#settings");
	find_node("#add-relay", el).addEventListener("click", on_click_add_relay);
	const rlist = find_node("#relay-list tbody", el);
	model.relays.forEach((str) => {
		rlist.appendChild(new_relay_item(str));
	});
}

function new_relay_item(str) {
	const tr = document.createElement('tr');
	tr.innerHTML = `<td>${str}</td>
	<td>
	<button class="remove-relay btn-text" 
		data-address="${str}"
		role="remove-relay">
		<img class="icon svg small" src="/icon/event-delete.svg"/>
	</button>
	</td>`;
	find_node("button", tr).addEventListener("click", on_click_remove_relay);
	return tr;
}

function on_click_add_relay(ev) {
	const model = DAMUS;
	const address = prompt("Please provide a websocket address:", "wss://");
	log_debug("got address", address);
	// TODO add relay validation
	if (!model.pool.add(address))
		return;
	model.relays.add(address);
	find_node("#relay-list tbody").appendChild(new_relay_item(address));
	model_save_settings(model);
}

function on_click_remove_relay(ev) {
	const model = DAMUS;
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
