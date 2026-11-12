const encoder = new TextEncoder();

export function createPeerId(seed = '') {
  const bytes = encoder.encode(String(seed));
  let hash = 0n;
  for (const byte of bytes) {
    hash = (hash * 1099511628211n) ^ BigInt(byte);
  }
  return `peer-${hash.toString(16)}`;
}

export function normalizePeerId(value) {
  return value ? String(value) : createPeerId();
}
