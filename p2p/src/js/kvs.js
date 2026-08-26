const stores = new Map();

function bucket(name) {
  const key = String(name || 'default');
  if (!stores.has(key)) {
    stores.set(key, new Map());
  }
  return stores.get(key);
}

export class KeyValueStore {
  constructor(namespace = 'default') {
    this.namespace = String(namespace);
  }

  async put(key, value) {
    bucket(this.namespace).set(String(key), value);
    return true;
  }

  async get(key) {
    return bucket(this.namespace).get(String(key));
  }

  async delete(key) {
    return bucket(this.namespace).delete(String(key));
  }

  async keys() {
    return [...bucket(this.namespace).keys()];
  }

  async entries() {
    return [...bucket(this.namespace).entries()];
  }
}

export function createKeyValueStore(namespace) {
  return new KeyValueStore(namespace);
}
