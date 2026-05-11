const gnostrBrowserNostr = (() => {
	const globalScope = typeof globalThis !== "undefined" ? globalThis : {};

	function getProvider() {
		return globalScope.window && globalScope.window.nostr ? globalScope.window.nostr : null;
	}

	function isAvailable() {
		return !!getProvider();
	}

	function waitForProvider(interval = 20) {
		return new Promise((resolve) => {
			let tries = 0;
			function poll() {
				if (isAvailable()) {
					log_info("browser nostr provider detected after", tries);
					resolve(true);
					return;
				}
				tries++;
				setTimeout(poll, interval);
			}
			poll();
		});
	}

	async function getPublicKey(use_prompt = true) {
		const provider = getProvider();
		if (!(provider && provider.getPublicKey)) {
			console.error("window.nostr.getPublicKey is unsupported");
			return;
		}
		return await provider.getPublicKey();
	}

	async function signEvent(ev) {
		const provider = getProvider();
		if (!(provider && provider.signEvent)) {
			console.error("window.nostr.signEvent is unsupported");
			return;
		}
		const signed = await provider.signEvent(ev);
		if (typeof signed === "string") {
			ev.sig = signed;
			return ev;
		}
		return signed;
	}

	function supportsNip04() {
		const provider = getProvider();
		return !!(provider && provider.nip04);
	}

	async function encrypt(pubkey, plaintext) {
		const provider = getProvider();
		if (!(provider && provider.nip04 && provider.nip04.encrypt)) {
			console.error("window.nostr.nip04.encrypt is unsupported");
			return;
		}
		return await provider.nip04.encrypt(pubkey, plaintext);
	}

	async function decrypt(pubkey, ciphertext) {
		const provider = getProvider();
		if (!(provider && provider.nip04 && provider.nip04.decrypt)) {
			console.error("window.nostr.nip04.decrypt is unsupported");
			return;
		}
		return await provider.nip04.decrypt(pubkey, ciphertext);
	}

	return {
		isAvailable,
		waitForProvider,
		getPublicKey,
		signEvent,
		supportsNip04,
		encrypt,
		decrypt,
	};
})();

if (typeof window !== "undefined") {
	window.gnostrBrowserNostr = gnostrBrowserNostr;
}

if (typeof globalThis !== "undefined") {
	globalThis.gnostrBrowserNostr = gnostrBrowserNostr;
}

if (typeof module !== "undefined" && module.exports) {
	module.exports = gnostrBrowserNostr;
}
