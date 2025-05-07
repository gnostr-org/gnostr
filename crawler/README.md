# Nostr Crawler 0

Simple nostr crawler, to 'crawl' relays to collect active public keys from a period.

`RecommendedRelay` (kind 2) events are used to discover other relays.
`ContactList` (kind 3) events are used to discover public keys.

__Warning!__ Use with discression, don't put too much load on the relays!

### Usage

Prerequisite: Rust (1.64)

Simply build & start

```
cargo run | tee output
```

See sample output below.

### Limitations

- No collected data is persisted, all runs start with empty state
- Data is collected from a past period, the previous period of N hours (6 hours by default), until the moment of starting the program.
- Number of relays used is capped at a maximum number.

### More Details

- Events are collected in a signle subscription, to which relays are added along the way
- Stop condition is somewhat arbitrary: EndOfStoredEvents are watched, but not all relays implement it.
- Process is started with a few hard-coded 'bootstrap' relays.
- Running time is on the order of few minutes
- Uses the `nostr-sdk` crate

### Sample Output (truncated)

```
Connecting to 3 relays ...
"wss://nos.lol/" "wss://relay.damus.io/" "wss://relay.plebstr.com/" 
Connected
Subscribed to relay events
Contacts 48
Contacts 42
Contacts 159
...
Contacts 1
Relays: 4
    wss://nos.lol/  wss://relay.plebstr.com/  wss://relay.damus.io/  wss://nostr-pub.wellorder.net/  
Contacts 48
...
Contacts 309
Relays: 26
    wss://nostr.zebedee.cloud/  wss://relay.plebstr.com/  wss://nostr.mikedilger.com/  wss://nostr.onsats.org/  wss://nostr.bitcoiner.social/  wss://eden.nostr.land/  wss://nostr.cercatrova.me/  wss://nostr.wine/  wss://bitcoiner.social/  wss://nostr-relay.wlvs.space/  wss://relay.thes.ai/  wss://nos.lol/  wss://relay.nostr.info/  wss://relay.nostrview.com/  wss://relay.nostrplebs.com/  wss://relay.snort.social/  wss://relay.nostr.ro/  wss://atlas.nostr.land/  wss://nostr.member.cash/  wss://nostr.walletofsatoshi.com/  wss://relay.mostr.pub/  wss://nostr-relay.nokotaro.com/  wss://nostr-pub.wellorder.net/  wss://relay.damus.io/  wss://nostr-dev.wellorder.net/  wss://relay.nostr.bg/  
Relays: 27
    wss://nostr.zebedee.cloud/  wss://relay.plebstr.com/  wss://nostr.mikedilger.com/  wss://nostr.onsats.org/  wss://nostr.bitcoiner.social/  wss://eden.nostr.land/  wss://nostr.cercatrova.me/  wss://nostr.wine/  wss://bitcoiner.social/  wss://nostr-pub.semisol.dev/  wss://nostr-relay.wlvs.space/  wss://relay.thes.ai/  wss://nos.lol/  wss://relay.nostr.info/  wss://relay.nostrview.com/  wss://relay.nostrplebs.com/  wss://relay.snort.social/  wss://relay.nostr.ro/  wss://atlas.nostr.land/  wss://nostr.member.cash/  wss://nostr.walletofsatoshi.com/  wss://relay.mostr.pub/  wss://nostr-relay.nokotaro.com/  wss://nostr-pub.wellorder.net/  wss://relay.damus.io/  wss://nostr-dev.wellorder.net/  wss://relay.nostr.bg/  
Contacts 172
Reconnect 25 27
Disconnected
Connecting to 27 relays ...
"wss://nostr-pub.semisol.dev/" "wss://relay.nostrplebs.com/" "wss://relay.nostr.ro/" "wss://nostr.walletofsatoshi.com/" "wss://nos.lol/" "wss://relay.nostr.bg/" "wss://relay.plebstr.com/" "wss://nostr.wine/" "wss://relay.nostrview.com/" "wss://nostr.mikedilger.com/" "wss://nostr.onsats.org/" "wss://relay.nostr.info/" "wss://atlas.nostr.land/" "wss://nostr-pub.wellorder.net/" "wss://nostr.cercatrova.me/" "wss://relay.snort.social/" "wss://relay.damus.io/" "wss://nostr-relay.nokotaro.com/" "wss://eden.nostr.land/" "wss://nostr.zebedee.cloud/" "wss://nostr-relay.wlvs.space/" "wss://relay.thes.ai/" "wss://nostr.bitcoiner.social/" "wss://relay.mostr.pub/" "wss://nostr-dev.wellorder.net/" "wss://bitcoiner.social/" "wss://nostr.member.cash/" 
Connected
Contacts 174
Contacts 680
Contacts 139
...
...
...
Contacts 25
Contacts 60
Contacts 53
Relays: 54
    wss://filter.nostr.wine/npub1aeh2zw4elewy5682lxc6xnlqzjnxksq303gwu2npfaxd49vmde6qcq4nwx?broadcast=true  wss://nostr.zebedee.cloud/  wss://offchain.pub/  wss://relay.plebstr.com/  wss://nostr.mikedilger.com/  wss://nostr.onsats.org/  wss://no.str.cr/  wss://eden.nostr.land/  wss://nostr-relay.untethr.me/  wss://nostr.wine/  wss://nostr.zerofeerouting.com/  wss://cloudnull.land/  wss://relay.nostr.pro/  wss://nostr-relay.wlvs.space/  wss://brb.io/  wss://nostr-verified.wellorder.net/  wss://nostr.developer.li/  wss://relay.nostr.info/  wss://relay.snort.social/  wss://relay.nostrplebs.com/  wss://atlas.nostr.land/  wss://relay.nostr.ro/  wss://nostr.member.cash/  ws://127.0.0.1:8080/  wss://nostr.fmt.wiz.biz/  wss://relay.mostr.pub/  wss://rsslay.fiatjaf.com/  wss://relay.nostr.bg/  wss://nostr.mom/  wss://nostr.bitcoiner.social/  wss://nostr.massmux.com/  wss://relay.nostrbr.online/  wss://nostr-pub.semisol.dev/  wss://nostr.cercatrova.me/  wss://nostr-relay.freeberty.net/  wss://nostr.v0l.io/  wss://bitcoiner.social/  wss://expensive-relay.fiatjaf.com/  wss://relay.nostr.band/  wss://relay.thes.ai/  wss://nos.lol/  wss://relay.nostrview.com/  wss://relay.nostr.ch/  wss://filter.nostr.wine/npub1xjsenwf98zypvyu039f7v767cm3j8sc0jf55qd38x94nn2zw5x9ska3rqt?broadcast=true  wss://at.nostrworks.com/  wss://nostr.walletofsatoshi.com/  wss://nostr.mutinywallet.com/  wss://nostr.plebchain.org/  wss://nostr-relay.nokotaro.com/  wss://nostr-pub.wellorder.net/  wss://relay.damus.io/  wss://nostr-dev.wellorder.net/  wss://nostr.oxtr.dev/  wss://relay.current.fyi/  
Contacts 2075
Contacts 97
Contacts 89
...
Received EOSE from wss://relay.nostr.bg/, total 19 (50 relays, 32 connected 6 connecting)
...
Contacts 2219
Contacts 2220
Contacts 2079
Received EOSE from wss://nostr.bitcoiner.social/, total 20 (50 relays, 34 connected 5 connecting)
...
Received EOSE from wss://relay.nostr.bg/, total 23 (50 relays, 33 connected 2 connecting)
STOPPING; There were some EOSE-s, and no events in the past 21 secs
Unsubscribed from relay events ...
Disconnected
STOPPED
======================================================

Relays: 54
  wss://filter.nostr.wine/npub1aeh2zw4elewy5682lxc6xnlqzjnxksq303gwu2npfaxd49vmde6qcq4nwx?broadcast=true
  wss://nostr.zebedee.cloud/
  wss://offchain.pub/
...
  wss://nostr-dev.wellorder.net/
  wss://nostr.oxtr.dev/
  wss://relay.current.fyi/

Number of ContactList events:      	 1127
Number of RecommendedRelay events: 	 27

Found  21183  public keys:
  npub1ew06xdf7...
  npub1uscmzple...
  npub14gw7cnxp...
...
  npub18qjehhpv...
  npub1073vwazf...
  npub1qqrcjx5z...
```
