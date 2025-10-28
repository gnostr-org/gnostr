// TODO track friend sphere another way by using graph nodes

function contact_is_friend(contacts, pk) {
	return contacts.friends.has(pk)
}

function contacts_process_event(contacts, our_pubkey, ev) {
	if (ev.pubkey !== our_pubkey)
		return;
	if (contacts.event && ev.created_at < contacts.event.created_at)
		return;
	contacts.event = ev
	contacts.friends = new Set();
	for (const tag of ev.tags) {
		if (tag.length > 1 && tag[0] === "p") {
			contacts.friends.add(tag[1])
		}
	}
}

/* contacts_save commits the contacts data to storage.
 */
async function contacts_save(contacts) {
	function _contacts_save(ev, resolve, reject) {
		const db = ev.target.result;
		let tx = db.transaction("friends", "readwrite");
		let store = tx.objectStore("friends");
		tx.oncomplete = (ev) => {
			db.close();
			resolve();
			log_debug("contacts saved successfully", contacts.friends.size);
		}
		tx.onerror = (ev) => {
			db.close();
			log_error(`tx errorCode: ${ev.request.errorCode}`);
			window.alert("An error occured saving contacts. Check console.");
			reject(ev);
		};

		store.clear().onsuccess = () => {
		 	contacts.friends.forEach((pubkey) => {
		 		//log_debug("storing", pubkey);
		 		store.put({pubkey});
		 	});
		};
	}
	return dbcall(_contacts_save);
}

async function contacts_load(model) {
	function _contacts_load(ev, resolve, reject) {
		const db = ev.target.result;
		const tx = db.transaction("friends", "readonly");
		const store = tx.objectStore("friends");
		const cursor = store.openCursor();
		cursor.onsuccess = (ev)=> {
			var cursor = ev.target.result;
			if (cursor) {
				//log_debug("cursor val", cursor.value);
				model.contacts.friends.add(cursor.value.pubkey);
				cursor.continue();
			} else {
				db.close();
				resolve();
				log_debug(`contacts loaded successfully ${model.contacts.friends.size}`);
			}
		}
		cursor.onerror = (ev) => {
			db.close();
			reject(ev);
			log_error("Could not load contacts.");
		}
	}
	return dbcall(_contacts_load);
}

// TODO move database methods to it's own file

async function dbcall(fn) {
	return new Promise((resolve, reject) => {
		var open = indexedDB.open("damus", 5);
		open.onupgradeneeded = (ev) => {
			const db = ev.target.result;
			if (!db.objectStoreNames.contains("friends"))
				db.createObjectStore("friends", {keyPath: "pubkey"});
			if (!db.objectStoreNames.contains("events"))
				db.createObjectStore("events", {keyPath: "id"});
			if (!db.objectStoreNames.contains("settings"))
				db.createObjectStore("settings", {keyPath: "pubkey"});
		};
		open.onsuccess = (ev) => {
			fn(ev, resolve, reject); 
		}
		open.onerror = (ev) => {
			reject(err);	
		};
	});
}

async function dbclear() {
	function _dbclear(ev, resolve, reject) {
		const stores = ["friends", "events", "settings"];
		const db = ev.target.result;
		const tx = db.transaction(stores, "readwrite");
		tx.oncomplete = (ev) => {
			db.close();
			resolve();
			log_debug("cleared database");
		}
		tx.onerror = (ev) => {
			db.close();
			log_error(`tx errorCode: ${ev.request.errorCode}`);
			reject(ev);
		};
		for (const store of stores) {
			tx.objectStore(store).clear();
		}
	}
	return dbcall(_dbclear);
}
