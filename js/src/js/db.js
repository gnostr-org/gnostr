const DB_NAME = 'gnostr_cache';
const DB_VERSION = 4; // Incremented DB version to trigger schema upgrade
const STORE_NIP34_EVENTS = 'nip34_events';
const STORE_NIP65_RELAYS = 'nip65_relays';
const INDEX_REPO_ID = 'repo_id_idx';

let db = null;
let local_relay = null;
const local_relay_synced_event_ids = new Set();
const local_relay_url = "ws://127.0.0.1:8080";
let local_relay_stats = {
    connected: false,
    sent: 0,
    received: 0,
    errors: 0,
    last_error: "",
};

function get_local_relay_status() {
    return {
        url: local_relay_url,
        connected: !!(local_relay && local_relay.ws && local_relay.ws.readyState === 1),
        sent: local_relay_stats.sent,
        received: local_relay_stats.received,
        errors: local_relay_stats.errors,
        last_error: local_relay_stats.last_error,
    };
}

function local_relay_is_connected() {
    return !!(local_relay && local_relay.ws && local_relay.ws.readyState === 1);
}

function local_relay_send_event(event) {
    if (!event || !event.id || !local_relay_is_connected()) {
        return false;
    }
    if (local_relay_synced_event_ids.has(event.id)) {
        return false;
    }

    local_relay_synced_event_ids.add(event.id);
    local_relay_stats.sent += 1;
    local_relay.send(["EVENT", event]);
    return true;
}

function model_has_nip34_events_for_pubkey(model, pubkey) {
    for (const id in model.all_events) {
        const ev = model.all_events[id];
        if (ev && ev.pubkey === pubkey && NIP_34_KINDS.includes(ev.kind)) {
            return true;
        }
    }
    return false;
}

function sync_related_user_metadata_to_local_relay(model, event) {
    if (!event || event.kind !== KIND_METADATA) {
        return false;
    }
    if (!model_has_nip34_events_for_pubkey(model, event.pubkey)) {
        return false;
    }
    return local_relay_send_event(event);
}

function sync_nip34_event_to_local_relay(model, event) {
    if (!event) {
        return false;
    }

    if (event.kind === KIND_METADATA) {
        return sync_related_user_metadata_to_local_relay(model, event);
    }

    if (!NIP_34_KINDS.includes(event.kind)) {
        return false;
    }

    let sent = local_relay_send_event(event);

    const profile = model_get_profile(model, event.pubkey);
    if (profile && profile.evid) {
        const metadata_event = model.all_events[profile.evid];
        if (metadata_event && metadata_event.kind === KIND_METADATA) {
            sent = local_relay_send_event(metadata_event) || sent;
        }
    }

    return sent;
}

function sync_all_nip34_events_to_local_relay(model) {
    let sent = false;
    for (const id in model.all_events) {
        sent = sync_nip34_event_to_local_relay(model, model.all_events[id]) || sent;
    }
    return sent;
}

async function relay_process_request(path) {
    const response = await fetch(path, { method: 'POST' });
    const data = await response.json();
    if (!response.ok) {
        throw new Error(data.message || data.error || `Request failed for ${path}`);
    }
    return data;
}

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
            local_relay_stats.connected = true;
            local_relay_stats.last_error = "";
            sync_all_nip34_events_to_local_relay(GNOSTR);
            if (typeof render_relay_dashboard === 'function') {
                render_relay_dashboard();
            }

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
            local_relay_stats.received += 1;
            add_nip34_event_to_db(ev, true);
        });

        relay.on('error', (err) => { 
            log_warn(`Could not connect to local relay at ${local_relay_url}:`, err);
            local_relay = null; 
            local_relay_stats.connected = false;
            local_relay_stats.errors += 1;
            local_relay_stats.last_error = String(err);
            if (typeof render_relay_dashboard === 'function') {
                render_relay_dashboard();
            }
        });

        relay.on('close', () => { 
            log_info(`Disconnected from local relay ${local_relay_url}`);
            local_relay = null; 
            local_relay_stats.connected = false;
            if (typeof render_relay_dashboard === 'function') {
                render_relay_dashboard();
            }
        });

    } catch (err) {
        log_error("Error initializing local relay sync connection:", err);
        local_relay_stats.connected = false;
        local_relay_stats.errors += 1;
        local_relay_stats.last_error = String(err);
        if (typeof render_relay_dashboard === 'function') {
            render_relay_dashboard();
        }
    }
}

async function stop_local_relay_sync() {
    const model = GNOSTR;
    model.local_relay_enabled = false;
    if (local_relay) {
        local_relay.close();
        local_relay = null;
    }
    try {
        const data = await relay_process_request("/api/relay/stop");
        log_info(data.message);
    } catch (error) {
        log_error("Failed to stop local relay process:", error);
    }
    local_relay_stats.connected = false;
    model_save_settings(model);
    if (typeof render_relay_dashboard === 'function') {
        render_relay_dashboard();
    }
}

async function start_local_relay_sync() {
    const model = GNOSTR;
    model.local_relay_enabled = true;
    model_save_settings(model);
    try {
        const data = await relay_process_request("/api/relay/start");
        log_info(data.message);
    } catch (error) {
        log_error("Failed to start local relay process:", error);
    }
    if (local_relay && local_relay.ws && local_relay.ws.readyState === 1) {
        return local_relay;
    }
    init_local_relay_sync();
    return local_relay;
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

        if (!is_from_local && local_relay_is_connected()) {
            log_debug(`Syncing event ${event.id} to local relay.`);
            local_relay_send_event(event);
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
