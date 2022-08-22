
const test = require('tape')
const RelayPool = require('../')

const jb55 = "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
const damus = "wss://relay.damus.io"
const scsi = "wss://nostr-pub.wellorder.net"
const relays = [damus, scsi]

test('connect to multiple works', function (t) {
	t.plan(2)

	const pool = RelayPool(relays)

	pool.on('open', relay => {
		t.equal(true, relay.url === damus || relay.url === scsi, `connected to ${relay.url}`)

		relay.close()
	});
});

test('querying multiple works', function (t) {
	let per_relay = 2
	let expected = per_relay * relays.length

	t.plan(expected)

	let n = 0
	const pool = RelayPool(relays)

	pool.on('open', relay => {
		relay.subscribe("subid", {limit: per_relay, kinds:[1], authors: [jb55]})
	});

	pool.on('eose', relay => {
		relay.close()
	});

	pool.on('event', (relay, sub_id, ev) => {
		t.equal("subid", sub_id, `got event ${++n}/${expected} from ${relay.url}`)
	});
});

