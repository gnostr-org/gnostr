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
