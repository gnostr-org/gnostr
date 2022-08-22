
# nostr

nostr nodejs library

## Example

```js
const RelayPool = require('nostr')

const jb55 = "32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245"
const damus = "wss://relay.damus.io"
const scsi = "wss://nostr-pub.wellorder.net"
const relays = [damus, scsi]

const pool = RelayPool(relays)

pool.on('open', relay => {
	relay.subscribe("subid", {limit: 2, kinds:[1], authors: [jb55]})
});

pool.on('eose', relay => {
	relay.close()
});

pool.on('event', (relay, sub_id, ev) => {
	console.log(ev)
});
```
