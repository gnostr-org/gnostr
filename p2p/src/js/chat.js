import { createLibp2p } from './libp2p.js';

export class ChatSession {
  constructor({ topic = 'gnostr', peerId, namespace } = {}) {
    this.topic = String(topic);
    this.node = createLibp2p({ peerId, namespace: namespace || this.topic });
  }

  async start() {
    await this.node.start();
    return this;
  }

  async join(handler) {
    return this.node.subscribe(this.topic, handler);
  }

  async send(message) {
    return this.node.publish(this.topic, message);
  }

  async save(key, value) {
    return this.node.provide(key, value);
  }

  async load(key) {
    return this.node.request(key);
  }
}

export function createChatSession(options) {
  return new ChatSession(options);
}
