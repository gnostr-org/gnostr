import { createPeerId, normalizePeerId } from './peer.js';
import { createKeyValueStore } from './kvs.js';

export class PureJsLibp2p extends EventTarget {
  constructor({ peerId, namespace = 'default' } = {}) {
    super();
    this.peerId = normalizePeerId(peerId);
    this.namespace = String(namespace);
    this.connections = new Set();
    this.topics = new Set();
    this.store = createKeyValueStore(this.namespace);
  }

  static create(options = {}) {
    return new PureJsLibp2p(options);
  }

  async start() {
    this.dispatchEvent(new Event('start'));
    return this;
  }

  async stop() {
    this.dispatchEvent(new Event('stop'));
    return this;
  }

  async connect(peerId) {
    const peer = normalizePeerId(peerId);
    this.connections.add(peer);
    this.dispatchEvent(new CustomEvent('connect', { detail: { peerId: peer } }));
    return peer;
  }

  async disconnect(peerId) {
    const peer = normalizePeerId(peerId);
    this.connections.delete(peer);
    this.dispatchEvent(new CustomEvent('disconnect', { detail: { peerId: peer } }));
    return peer;
  }

  async publish(topic, message) {
    const event = { topic: String(topic), message, peerId: this.peerId };
    this.dispatchEvent(new CustomEvent('message', { detail: event }));
    return event;
  }

  async subscribe(topic, handler) {
    const name = String(topic);
    this.topics.add(name);
    if (typeof handler === 'function') {
      const listener = (event) => {
        const detail = event.detail;
        if (detail?.topic === name) {
          handler(detail);
        }
      };
      this.addEventListener('message', listener);
      return () => this.removeEventListener('message', listener);
    }
    return name;
  }

  async request(key) {
    return this.store.get(key);
  }

  async provide(key, value) {
    return this.store.put(key, value);
  }
}

export function createLibp2p(options) {
  return PureJsLibp2p.create(options);
}

export { createPeerId };
