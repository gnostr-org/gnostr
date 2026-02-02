function load_our_relays(our_pubkey, pool, ev) {
	if (ev.pubkey != our_pubkey)
		return
	let relays
	try {
		relays = JSON.parse(ev.content)
	} catch (e) {
		log_error("error loading relays", e)
		return
	}
	for (const relay of Object.keys(relays)) {
		if (!pool.has(relay)) {
			log_debug("adding relay", relay)
			pool.add(relay)
		}
	}
}

/* DEPRECATED */

function is_deleted(model, evid) {
	log_warn("is_deleted deprecated, use model_is_event_deleted");
	return model_is_event_deleted(model, evid);
}
function process_event(model, ev) {
	log_warn("process_event deprecated, use event_process");
	return model_process_event(model, ev);
}
function calculate_pow(ev) {
	log_warn("calculate_pow deprecated, use event_calculate_pow");
	return event_calculate_pow(ev);
}
function can_reply(ev) {
	log_warn("can_reply is deprecated, use event_can_reply");
	return event_can_reply(ev);
}
function passes_spam_filter(contacts, ev, pow) {
	log_warn("passes_spam_filter deprecated, use event_is_spam");
	return !event_is_spam(ev, contacts, pow);
}

