async function gnostr_web_start() {
	await gnostr_web_init();
	document.addEventListener("click", onclick_any);
}

async function gnostr_web_init() {
	await gnostrBrowserNostr.waitForProvider();
	gnostr_web_init_ready();
}

async function gnostr_web_init_ready() {
	const model = GNOSTR;
	model.pubkey = await gnostrBrowserNostr.getPublicKey(false);

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
		model.pubkey = await gnostrBrowserNostr.getPublicKey();
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
