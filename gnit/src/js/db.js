const DB_NAME = 'gnostr_cache';
const DB_VERSION = 4; // Incremented DB version to trigger schema upgrade
const STORE_NIP34_EVENTS = 'nip34_events';
const STORE_NIP65_RELAYS = 'nip65_relays';
const INDEX_REPO_ID = 'repo_id_idx';

let db = null;
let local_relay = null;
const local_relay_url = "ws://127.0.0.1:8080";

function init_local_relay_sync() {
    log_info("Initializing local relay DB sync...");
    try {
        if (typeof nostrjs === 'undefined' || typeof nostrjs.Relay !== 'function') {
            log_error("nostrjs.Relay is not available. DB sync cannot start.");
            return;
        }

        const relay = nostrjs.Relay(local_relay_url, { reconnect: true });

        relay.on('open', () => {
            log_info(`Connected to local relay for DB sync: ${local_relay_url}`);
            local_relay = relay;

            const sub_id = `db-sync-${Math.random().toString(36).substring(7)}`;
            const nip34_kinds = [
            KIND_REPO_ANNOUNCE,
            KIND_REPO_STATE_ANNOUNCE,
            KIND_REPO_PATCH,
            KIND_REPO_PULL_REQ,
            KIND_REPO_PULL_REQ_UPDATE,
            KIND_REPO_ISSUE,
            KIND_REPO_STATUS_OPEN,
            KIND_REPO_STATUS_APPLIED,
            KIND_REPO_STATUS_CLOSED,
            KIND_REPO_STATUS_DRAFT,
        ];
            relay.subscribe(sub_id, [{ kinds: nip34_kinds }]);
        });

        relay.on('event', (sub_id, ev) => {
            add_nip34_event_to_db(ev, true);
        });

        relay.on('error', (err) => { 
            log_warn(`Could not connect to local relay at ${local_relay_url}:`, err);
            local_relay = null; 
        });

        relay.on('close', () => { 
            log_info(`Disconnected from local relay ${local_relay_url}`);
            local_relay = null; 
        });

    } catch (err) {
        log_error("Error initializing local relay sync connection:", err);
    }
}

function open_db() {
    return new Promise((resolve, reject) => {
        if (db) {
            resolve(db);
            return;
        }

        const request = indexedDB.open(DB_NAME, DB_VERSION);

        request.onupgradeneeded = (event) => {
            console.log("IndexedDB: Upgrading database.", event);
            const d = event.target.result;
            let store;
            if (!d.objectStoreNames.contains(STORE_NIP34_EVENTS)) {
                store = d.createObjectStore(STORE_NIP34_EVENTS, { keyPath: 'id' });
            } else {
                store = event.target.transaction.objectStore(STORE_NIP34_EVENTS);
            }

            if (!store.indexNames.contains(INDEX_REPO_ID)) {
                store.createIndex(INDEX_REPO_ID, 'repo_id', { unique: false });
            }

            if (!d.objectStoreNames.contains(STORE_NIP65_RELAYS)) {
                d.createObjectStore(STORE_NIP65_RELAYS, { keyPath: 'pubkey' });
            }
        };

        request.onsuccess = (event) => {
            db = event.target.result;
            resolve(db);
        };

        request.onerror = (event) => {
            console.error("IndexedDB: Error opening database", event);
            reject(event.target.error);
        };
    });
}

async function add_nip65_relays_to_db(pubkey, relays) {
    try {
        const d = await open_db();
        const tx = d.transaction(STORE_NIP65_RELAYS, 'readwrite');
        const store = tx.objectStore(STORE_NIP65_RELAYS);

        // Relays are stored as [[url, {read: true, write: true}], ...]
        const storable_relays = { pubkey, relays };
        store.put(storable_relays);
        await tx.complete;
    } catch (error) {
        console.error("IndexedDB: Error adding NIP-65 relays", error);
    }
}

async function get_nip65_relays_from_db(pubkey) {
    return new Promise(async (resolve, reject) => {
        try {
            const d = await open_db();
            const tx = d.transaction(STORE_NIP65_RELAYS, 'readonly');
            const store = tx.objectStore(STORE_NIP65_RELAYS);
            const request = store.get(pubkey);

            request.onsuccess = () => {
                resolve(request.result ? request.result.relays : []);
            };

            request.onerror = (event) => {
                console.error("IndexedDB: Error reading NIP-65 relays", event);
                reject(event.target.error);
            };

        } catch (error) {
            console.error("IndexedDB: Error in get_nip65_relays_from_db", error);
            reject(error);
        }
    });
}

async function add_nip34_event_to_db(event, is_from_local = false) {
    try {
        const d = await open_db();
        const tx = d.transaction(STORE_NIP34_EVENTS, 'readwrite');
        const store = tx.objectStore(STORE_NIP34_EVENTS);
        
        const storable_event = { ...event };
        
        const repo_tag = storable_event.tags.find(tag => tag[0] === 'a');
        if (repo_tag) {
            storable_event.repo_id = repo_tag[1];
        }

        store.put(storable_event);
        await tx.complete;

        if (!is_from_local && local_relay && local_relay.ws.readyState === 1) {
            log_debug(`Syncing event ${event.id} to local relay.`);
            local_relay.send(["EVENT", event]);
        }
    } catch (error) {
        console.error("IndexedDB: Error adding NIP-34 event", error);
    }
}

async function get_all_nip34_events_from_db() {
    return new Promise(async (resolve, reject) => {
        try {
            const d = await open_db();
            const tx = d.transaction(STORE_NIP34_EVENTS, 'readonly');
            const store = tx.objectStore(STORE_NIP34_EVENTS);
            const request = store.getAll();

            request.onsuccess = () => {
                resolve(request.result || []);
            };

            request.onerror = (event) => {
                console.error("IndexedDB: Error reading all NIP-34 events", event);
                reject(event.target.error);
            };

        } catch (error) {
            console.error("IndexedDB: Error in get_all_nip34_events_from_db", error);
            reject(error);
        }
    });
}

async function get_nip34_events_from_db(repo_id, until_timestamp = 0) {
    return new Promise(async (resolve, reject) => {
        const events = [];
        try {
            const d = await open_db();
            const tx = d.transaction(STORE_NIP34_EVENTS, 'readonly');
            const store = tx.objectStore(STORE_NIP34_EVENTS);
            const index = store.index(INDEX_REPO_ID);

            const request = index.openCursor(IDBKeyRange.only(repo_id), 'prev');

            request.onsuccess = (event) => {
                const cursor = event.target.result;
                if (cursor) {
                    const event = cursor.value;
                    if (until_timestamp === 0 || event.created_at < until_timestamp) {
                        events.push(event);
                    }
                    cursor.continue();
                } else {
                    resolve(events);
                }
            };

            request.onerror = (event) => {
                console.error("IndexedDB: Error reading NIP-34 events", event);
                reject(event.target.error);
            };

        } catch (error) {
            console.error("IndexedDB: Error in get_nip34_events_from_db", error);
            reject(error);
        }
    });
}

async function get_nip34_cache_size() {
    return new Promise(async (resolve, reject) => {
        let totalSize = 0;
        try {
            const d = await open_db();
            const tx = d.transaction(STORE_NIP34_EVENTS, 'readonly');
            const store = tx.objectStore(STORE_NIP34_EVENTS);
            const request = store.openCursor();

            request.onsuccess = (event) => {
                const cursor = event.target.result;
                if (cursor) {
                    const eventData = JSON.stringify(cursor.value);
                    totalSize += new TextEncoder().encode(eventData).length;
                    cursor.continue();
                } else {
                    let sizeStr;
                    if (totalSize < 1024) {
                        sizeStr = `${totalSize} B`;
                    } else if (totalSize < 1024 * 1024) {
                        sizeStr = `${(totalSize / 1024).toFixed(2)} KB`;
                    } else {
                        sizeStr = `${(totalSize / (1024 * 1024)).toFixed(2)} MB`;
                    }
                    resolve(sizeStr);
                }
            };

            request.onerror = (event) => {
                console.error("IndexedDB: Error calculating cache size", event);
                reject(event.target.error);
            };

        } catch (error) {
            console.error("IndexedDB: Error in get_nip34_cache_size", error);
            reject(error);
        }
    });
}
