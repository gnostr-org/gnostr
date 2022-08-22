const WebSocket = require('ws')

function RelayPool(relays, opts)
{
	if (!(this instanceof RelayPool))
		return new RelayPool(relays)

	this.onfn = {}
	this.relays = []

	for (const relay of relays) {
		this.add(relay)
	}

	return this
}

RelayPool.prototype.close = function relayPoolClose() {
	for (const relay of this.relays) {
		relay.close()
	}
}

RelayPool.prototype.on = function relayPoolOn(method, fn) {
	for (const relay of this.relays) {
		this.onfn[method] = fn
		relay.onfn[method] = fn.bind(null, relay)
	}
}

RelayPool.prototype.has = function relayPoolHas(relayUrl) {
	for (const relay of this.relays) {
		if (relay.relay === relayUrl)
			return true
	}

	return false
}

RelayPool.prototype.setupHandlers = function relayPoolSetupHandlers(method, fn)
{
	// setup its message handlers with the ones we have already
	for (const handler of Object.keys(this.on)) {
		for (const relay of this.relays) {
			relay.onfn[handler] = this.onfn[handler].bind(null, relay)
		}
	}
}

RelayPool.prototype.remove = function relayPoolRemove(url) {
	let i = 0

	for (const relay of this.relays) {
		if (relay.url === url) {
			relay.ws && relay.ws.close()
			this.relays = this.replays.splice(i, 1)
			return true
		}

		i += 1
	}

	return false
}

RelayPool.prototype.subscribe = function relayPoolSubscribe(...args) {
	for (const relay of this.relays) {
		relay.subscribe(...args)
	}
}

RelayPool.prototype.unsubscribe = function relayPoolUnsubscibe(...args) {
	for (const relay of this.relays) {
		relay.unsubscribe(...args)
	}
}

RelayPool.prototype.add = function relayPoolAdd(relay) {
	if (relay instanceof Relay) {
		if (this.has(relay.url))
			return false

		this.relays.push(relay)
		this.setupHandlers()
		return true
	}

	if (this.has(relay))
		return false

	const r = Relay(relay, this.opts)
	this.relays.push(r)
	this.setupHandlers()
	return true
}

function Relay(relay, opts={})
{
	if (!(this instanceof Relay))
		return new Relay(relay, opts)

	this.url = relay
	this.opts = opts

	if (opts.reconnect == null)
		opts.reconnect = true

	const me = this
	me.onfn = {}

	init_websocket(me)

	return this
}

function init_websocket(me) {
	const ws = me.ws = new WebSocket(me.url);
	let resolved = false
	ws.onmessage = (m) => { handle_nostr_message(me, m) }
	ws.onclose = () => { 
		if (me.onfn.close) 
			me.onfn.close() 
		if (!me.manualClose && me.opts.reconnect)
			init_websocket(me)
	}
	ws.onerror = () => { 
		if (me.onfn.error)
			me.onfn.error() 
		if (me.opts.reconnect)
			init_websocket(me)
	}
	ws.onopen = () => {
		if (me.onfn.open)
			me.onfn.open()
	}
}

Relay.prototype.on = function relayOn(method, fn) {
	this.onfn[method] = fn
}

Relay.prototype.close = function relayClose() {
	if (this.ws) {
		this.manualClose = true
		this.ws.close()
	}
}

Relay.prototype.subscribe = function relay_subscribe(sub_id, ...filters) {
	const tosend = ["REQ", sub_id, ...filters]
	this.ws.send(JSON.stringify(tosend))
}

Relay.prototype.unsubscribe = function relay_unsubscribe(sub_id) {
	const tosend = ["CLOSE", sub_id]
	this.ws.send(JSON.stringify(tosend))
}

function handle_nostr_message(relay, msg)
{
	const data = JSON.parse(msg.data)
	if (data.length >= 2) {
		switch (data[0]) {
		case "EVENT":
			if (data.length < 3)
				return
			return relay.onfn.event && relay.onfn.event(data[1], data[2])
		case "EOSE":
			return relay.onfn.eose && relay.onfn.eose(data[1])
		case "NOTICE":
			return relay.onfn.notice && relay.onfn.notice(...data.slice(1))
		}
	}
}

module.exports = RelayPool
