
# nostr

nostr nodejs library

## Example

```js
const {RelayPool} = require('nostr')

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

## API

### RelayPool(urls, options)

Connect to a pool of relays. You should use this instead of `Relay` directly.

options: `{reconnect: true|false}`, defaults to `{reconnect: true}`

```
const relays = [`wss://relay1.com`, `wss://relay2.com`]
const pool = RelayPool(relays, {reconnect: false})
```

### Relay(url, options)

Connect to a relay

options: `{reconnect: true|false}`, defaults to `{reconnect: true}`

```
const relay = Relay(`wss://relay.damus.io`, {reconnect: false})
```

### async calculateId(event)

Calculate an id from an event

### async signId(privkey, id)

Create a signature for an id

### getPublicKey(privkey)

Get a public key from a privkey

### async createDelegation(privkey, pubkey, publisherPubkey, conditions)

Create a delegation. This gives `publisherPubkey` permission to create events
on the `privkey`s behalf subject to `conditions`

`privkey`: authorizer private key

`pubkey`: authorizer public key

`publisherKey`: delegate/publisher pubkey

`conditions`: `&` separated set of event creation permissions that the delegate
is required to adhere to when creating events

eg: `created_at>1669303873&created_at<1674574279&kind=1,7`

Returns:

```js
{pubkey, publisherPubkey, conditions, token}
```

Where `token` is the delegation token, which is a signature from the authorizer
of the delegation `nostr:delegation:publisher_key:`.

### async createDelegationEvent(publisherPrivkey, event, delegation)

Create a delegated event from a `delegation`. This is an event posted on behalf
of `delegation.pubkey` subject to `delegation.conditions`.

`delegation`: a delegation in the form returned by `createDelegation`

`publisherPrivkey`: the private key of the delegate, the entity posting on behalf of `delegation.pubkey`

`event`: The event to post as a delegate. The event `pubkey` will be overridden
by the `publisherPubkey`. The delegation tag will be upserted into the tag
list.


### async signDelegationToken(privkey, unsignedToken) 

Sign a delegation string in the form `nostr:delegation:...`.

Returns: a signature string called the delegation token.
