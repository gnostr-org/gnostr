(() => {
  var __defProp = Object.defineProperty;
  var __export = (target, all) => {
    for (var name in all)
      __defProp(target, name, { get: all[name], enumerable: true });
  };

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/_assert.js
  function number(n) {
    if (!Number.isSafeInteger(n) || n < 0)
      throw new Error(`Wrong positive integer: ${n}`);
  }
  function bytes(b, ...lengths) {
    if (!(b instanceof Uint8Array))
      throw new Error("Expected Uint8Array");
    if (lengths.length > 0 && !lengths.includes(b.length))
      throw new Error(`Expected Uint8Array of length ${lengths}, not of length=${b.length}`);
  }
  function hash(hash3) {
    if (typeof hash3 !== "function" || typeof hash3.create !== "function")
      throw new Error("Hash should be wrapped by utils.wrapConstructor");
    number(hash3.outputLen);
    number(hash3.blockLen);
  }
  function exists(instance, checkFinished = true) {
    if (instance.destroyed)
      throw new Error("Hash instance has been destroyed");
    if (checkFinished && instance.finished)
      throw new Error("Hash#digest() has already been called");
  }
  function output(out, instance) {
    bytes(out);
    const min = instance.outputLen;
    if (out.length < min) {
      throw new Error(`digestInto() expects output buffer of length at least ${min}`);
    }
  }

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/crypto.js
  var crypto2 = typeof globalThis === "object" && "crypto" in globalThis ? globalThis.crypto : void 0;

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/utils.js
  var u8a = (a) => a instanceof Uint8Array;
  var createView = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
  var rotr = (word, shift) => word << 32 - shift | word >>> shift;
  var isLE = new Uint8Array(new Uint32Array([287454020]).buffer)[0] === 68;
  if (!isLE)
    throw new Error("Non little-endian hardware is not supported");
  function utf8ToBytes(str) {
    if (typeof str !== "string")
      throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
    return new Uint8Array(new TextEncoder().encode(str));
  }
  function toBytes(data) {
    if (typeof data === "string")
      data = utf8ToBytes(data);
    if (!u8a(data))
      throw new Error(`expected Uint8Array, got ${typeof data}`);
    return data;
  }
  function concatBytes(...arrays) {
    const r = new Uint8Array(arrays.reduce((sum, a) => sum + a.length, 0));
    let pad2 = 0;
    arrays.forEach((a) => {
      if (!u8a(a))
        throw new Error("Uint8Array expected");
      r.set(a, pad2);
      pad2 += a.length;
    });
    return r;
  }
  var Hash = class {
    // Safe version that clones internal state
    clone() {
      return this._cloneInto();
    }
  };
  var toStr = {}.toString;
  function wrapConstructor(hashCons) {
    const hashC = (msg) => hashCons().update(toBytes(msg)).digest();
    const tmp = hashCons();
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = () => hashCons();
    return hashC;
  }
  function randomBytes(bytesLength = 32) {
    if (crypto2 && typeof crypto2.getRandomValues === "function") {
      return crypto2.getRandomValues(new Uint8Array(bytesLength));
    }
    throw new Error("crypto.getRandomValues must be defined");
  }

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/_sha2.js
  function setBigUint64(view, byteOffset, value, isLE4) {
    if (typeof view.setBigUint64 === "function")
      return view.setBigUint64(byteOffset, value, isLE4);
    const _32n = BigInt(32);
    const _u32_max = BigInt(4294967295);
    const wh = Number(value >> _32n & _u32_max);
    const wl = Number(value & _u32_max);
    const h = isLE4 ? 4 : 0;
    const l = isLE4 ? 0 : 4;
    view.setUint32(byteOffset + h, wh, isLE4);
    view.setUint32(byteOffset + l, wl, isLE4);
  }
  var SHA2 = class extends Hash {
    constructor(blockLen, outputLen, padOffset, isLE4) {
      super();
      this.blockLen = blockLen;
      this.outputLen = outputLen;
      this.padOffset = padOffset;
      this.isLE = isLE4;
      this.finished = false;
      this.length = 0;
      this.pos = 0;
      this.destroyed = false;
      this.buffer = new Uint8Array(blockLen);
      this.view = createView(this.buffer);
    }
    update(data) {
      exists(this);
      const { view, buffer, blockLen } = this;
      data = toBytes(data);
      const len = data.length;
      for (let pos = 0; pos < len; ) {
        const take = Math.min(blockLen - this.pos, len - pos);
        if (take === blockLen) {
          const dataView = createView(data);
          for (; blockLen <= len - pos; pos += blockLen)
            this.process(dataView, pos);
          continue;
        }
        buffer.set(data.subarray(pos, pos + take), this.pos);
        this.pos += take;
        pos += take;
        if (this.pos === blockLen) {
          this.process(view, 0);
          this.pos = 0;
        }
      }
      this.length += data.length;
      this.roundClean();
      return this;
    }
    digestInto(out) {
      exists(this);
      output(out, this);
      this.finished = true;
      const { buffer, view, blockLen, isLE: isLE4 } = this;
      let { pos } = this;
      buffer[pos++] = 128;
      this.buffer.subarray(pos).fill(0);
      if (this.padOffset > blockLen - pos) {
        this.process(view, 0);
        pos = 0;
      }
      for (let i2 = pos; i2 < blockLen; i2++)
        buffer[i2] = 0;
      setBigUint64(view, blockLen - 8, BigInt(this.length * 8), isLE4);
      this.process(view, 0);
      const oview = createView(out);
      const len = this.outputLen;
      if (len % 4)
        throw new Error("_sha2: outputLen should be aligned to 32bit");
      const outLen = len / 4;
      const state = this.get();
      if (outLen > state.length)
        throw new Error("_sha2: outputLen bigger than state");
      for (let i2 = 0; i2 < outLen; i2++)
        oview.setUint32(4 * i2, state[i2], isLE4);
    }
    digest() {
      const { buffer, outputLen } = this;
      this.digestInto(buffer);
      const res = buffer.slice(0, outputLen);
      this.destroy();
      return res;
    }
    _cloneInto(to) {
      to || (to = new this.constructor());
      to.set(...this.get());
      const { blockLen, buffer, length, finished, destroyed, pos } = this;
      to.length = length;
      to.pos = pos;
      to.finished = finished;
      to.destroyed = destroyed;
      if (length % blockLen)
        to.buffer.set(buffer);
      return to;
    }
  };

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/sha256.js
  var Chi = (a, b, c) => a & b ^ ~a & c;
  var Maj = (a, b, c) => a & b ^ a & c ^ b & c;
  var SHA256_K = /* @__PURE__ */ new Uint32Array([
    1116352408,
    1899447441,
    3049323471,
    3921009573,
    961987163,
    1508970993,
    2453635748,
    2870763221,
    3624381080,
    310598401,
    607225278,
    1426881987,
    1925078388,
    2162078206,
    2614888103,
    3248222580,
    3835390401,
    4022224774,
    264347078,
    604807628,
    770255983,
    1249150122,
    1555081692,
    1996064986,
    2554220882,
    2821834349,
    2952996808,
    3210313671,
    3336571891,
    3584528711,
    113926993,
    338241895,
    666307205,
    773529912,
    1294757372,
    1396182291,
    1695183700,
    1986661051,
    2177026350,
    2456956037,
    2730485921,
    2820302411,
    3259730800,
    3345764771,
    3516065817,
    3600352804,
    4094571909,
    275423344,
    430227734,
    506948616,
    659060556,
    883997877,
    958139571,
    1322822218,
    1537002063,
    1747873779,
    1955562222,
    2024104815,
    2227730452,
    2361852424,
    2428436474,
    2756734187,
    3204031479,
    3329325298
  ]);
  var IV = /* @__PURE__ */ new Uint32Array([
    1779033703,
    3144134277,
    1013904242,
    2773480762,
    1359893119,
    2600822924,
    528734635,
    1541459225
  ]);
  var SHA256_W = /* @__PURE__ */ new Uint32Array(64);
  var SHA256 = class extends SHA2 {
    constructor() {
      super(64, 32, 8, false);
      this.A = IV[0] | 0;
      this.B = IV[1] | 0;
      this.C = IV[2] | 0;
      this.D = IV[3] | 0;
      this.E = IV[4] | 0;
      this.F = IV[5] | 0;
      this.G = IV[6] | 0;
      this.H = IV[7] | 0;
    }
    get() {
      const { A, B, C, D, E, F, G, H } = this;
      return [A, B, C, D, E, F, G, H];
    }
    // prettier-ignore
    set(A, B, C, D, E, F, G, H) {
      this.A = A | 0;
      this.B = B | 0;
      this.C = C | 0;
      this.D = D | 0;
      this.E = E | 0;
      this.F = F | 0;
      this.G = G | 0;
      this.H = H | 0;
    }
    process(view, offset) {
      for (let i2 = 0; i2 < 16; i2++, offset += 4)
        SHA256_W[i2] = view.getUint32(offset, false);
      for (let i2 = 16; i2 < 64; i2++) {
        const W15 = SHA256_W[i2 - 15];
        const W2 = SHA256_W[i2 - 2];
        const s0 = rotr(W15, 7) ^ rotr(W15, 18) ^ W15 >>> 3;
        const s1 = rotr(W2, 17) ^ rotr(W2, 19) ^ W2 >>> 10;
        SHA256_W[i2] = s1 + SHA256_W[i2 - 7] + s0 + SHA256_W[i2 - 16] | 0;
      }
      let { A, B, C, D, E, F, G, H } = this;
      for (let i2 = 0; i2 < 64; i2++) {
        const sigma1 = rotr(E, 6) ^ rotr(E, 11) ^ rotr(E, 25);
        const T1 = H + sigma1 + Chi(E, F, G) + SHA256_K[i2] + SHA256_W[i2] | 0;
        const sigma0 = rotr(A, 2) ^ rotr(A, 13) ^ rotr(A, 22);
        const T2 = sigma0 + Maj(A, B, C) | 0;
        H = G;
        G = F;
        F = E;
        E = D + T1 | 0;
        D = C;
        C = B;
        B = A;
        A = T1 + T2 | 0;
      }
      A = A + this.A | 0;
      B = B + this.B | 0;
      C = C + this.C | 0;
      D = D + this.D | 0;
      E = E + this.E | 0;
      F = F + this.F | 0;
      G = G + this.G | 0;
      H = H + this.H | 0;
      this.set(A, B, C, D, E, F, G, H);
    }
    roundClean() {
      SHA256_W.fill(0);
    }
    destroy() {
      this.set(0, 0, 0, 0, 0, 0, 0, 0);
      this.buffer.fill(0);
    }
  };
  var sha256 = /* @__PURE__ */ wrapConstructor(() => new SHA256());

  // node_modules/@noble/curves/esm/abstract/utils.js
  var utils_exports = {};
  __export(utils_exports, {
    bitGet: () => bitGet,
    bitLen: () => bitLen,
    bitMask: () => bitMask,
    bitSet: () => bitSet,
    bytesToHex: () => bytesToHex,
    bytesToNumberBE: () => bytesToNumberBE,
    bytesToNumberLE: () => bytesToNumberLE,
    concatBytes: () => concatBytes2,
    createHmacDrbg: () => createHmacDrbg,
    ensureBytes: () => ensureBytes,
    equalBytes: () => equalBytes,
    hexToBytes: () => hexToBytes,
    hexToNumber: () => hexToNumber,
    numberToBytesBE: () => numberToBytesBE,
    numberToBytesLE: () => numberToBytesLE,
    numberToHexUnpadded: () => numberToHexUnpadded,
    numberToVarBytesBE: () => numberToVarBytesBE,
    utf8ToBytes: () => utf8ToBytes2,
    validateObject: () => validateObject
  });
  var _0n = BigInt(0);
  var _1n = BigInt(1);
  var _2n = BigInt(2);
  var u8a2 = (a) => a instanceof Uint8Array;
  var hexes = /* @__PURE__ */ Array.from({ length: 256 }, (_, i2) => i2.toString(16).padStart(2, "0"));
  function bytesToHex(bytes4) {
    if (!u8a2(bytes4))
      throw new Error("Uint8Array expected");
    let hex2 = "";
    for (let i2 = 0; i2 < bytes4.length; i2++) {
      hex2 += hexes[bytes4[i2]];
    }
    return hex2;
  }
  function numberToHexUnpadded(num) {
    const hex2 = num.toString(16);
    return hex2.length & 1 ? `0${hex2}` : hex2;
  }
  function hexToNumber(hex2) {
    if (typeof hex2 !== "string")
      throw new Error("hex string expected, got " + typeof hex2);
    return BigInt(hex2 === "" ? "0" : `0x${hex2}`);
  }
  function hexToBytes(hex2) {
    if (typeof hex2 !== "string")
      throw new Error("hex string expected, got " + typeof hex2);
    const len = hex2.length;
    if (len % 2)
      throw new Error("padded hex string expected, got unpadded hex of length " + len);
    const array = new Uint8Array(len / 2);
    for (let i2 = 0; i2 < array.length; i2++) {
      const j = i2 * 2;
      const hexByte = hex2.slice(j, j + 2);
      const byte = Number.parseInt(hexByte, 16);
      if (Number.isNaN(byte) || byte < 0)
        throw new Error("Invalid byte sequence");
      array[i2] = byte;
    }
    return array;
  }
  function bytesToNumberBE(bytes4) {
    return hexToNumber(bytesToHex(bytes4));
  }
  function bytesToNumberLE(bytes4) {
    if (!u8a2(bytes4))
      throw new Error("Uint8Array expected");
    return hexToNumber(bytesToHex(Uint8Array.from(bytes4).reverse()));
  }
  function numberToBytesBE(n, len) {
    return hexToBytes(n.toString(16).padStart(len * 2, "0"));
  }
  function numberToBytesLE(n, len) {
    return numberToBytesBE(n, len).reverse();
  }
  function numberToVarBytesBE(n) {
    return hexToBytes(numberToHexUnpadded(n));
  }
  function ensureBytes(title, hex2, expectedLength) {
    let res;
    if (typeof hex2 === "string") {
      try {
        res = hexToBytes(hex2);
      } catch (e) {
        throw new Error(`${title} must be valid hex string, got "${hex2}". Cause: ${e}`);
      }
    } else if (u8a2(hex2)) {
      res = Uint8Array.from(hex2);
    } else {
      throw new Error(`${title} must be hex string or Uint8Array`);
    }
    const len = res.length;
    if (typeof expectedLength === "number" && len !== expectedLength)
      throw new Error(`${title} expected ${expectedLength} bytes, got ${len}`);
    return res;
  }
  function concatBytes2(...arrays) {
    const r = new Uint8Array(arrays.reduce((sum, a) => sum + a.length, 0));
    let pad2 = 0;
    arrays.forEach((a) => {
      if (!u8a2(a))
        throw new Error("Uint8Array expected");
      r.set(a, pad2);
      pad2 += a.length;
    });
    return r;
  }
  function equalBytes(b1, b2) {
    if (b1.length !== b2.length)
      return false;
    for (let i2 = 0; i2 < b1.length; i2++)
      if (b1[i2] !== b2[i2])
        return false;
    return true;
  }
  function utf8ToBytes2(str) {
    if (typeof str !== "string")
      throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
    return new Uint8Array(new TextEncoder().encode(str));
  }
  function bitLen(n) {
    let len;
    for (len = 0; n > _0n; n >>= _1n, len += 1)
      ;
    return len;
  }
  function bitGet(n, pos) {
    return n >> BigInt(pos) & _1n;
  }
  var bitSet = (n, pos, value) => {
    return n | (value ? _1n : _0n) << BigInt(pos);
  };
  var bitMask = (n) => (_2n << BigInt(n - 1)) - _1n;
  var u8n = (data) => new Uint8Array(data);
  var u8fr = (arr) => Uint8Array.from(arr);
  function createHmacDrbg(hashLen, qByteLen, hmacFn) {
    if (typeof hashLen !== "number" || hashLen < 2)
      throw new Error("hashLen must be a number");
    if (typeof qByteLen !== "number" || qByteLen < 2)
      throw new Error("qByteLen must be a number");
    if (typeof hmacFn !== "function")
      throw new Error("hmacFn must be a function");
    let v = u8n(hashLen);
    let k = u8n(hashLen);
    let i2 = 0;
    const reset = () => {
      v.fill(1);
      k.fill(0);
      i2 = 0;
    };
    const h = (...b) => hmacFn(k, v, ...b);
    const reseed = (seed = u8n()) => {
      k = h(u8fr([0]), seed);
      v = h();
      if (seed.length === 0)
        return;
      k = h(u8fr([1]), seed);
      v = h();
    };
    const gen = () => {
      if (i2++ >= 1e3)
        throw new Error("drbg: tried 1000 values");
      let len = 0;
      const out = [];
      while (len < qByteLen) {
        v = h();
        const sl = v.slice();
        out.push(sl);
        len += v.length;
      }
      return concatBytes2(...out);
    };
    const genUntil = (seed, pred) => {
      reset();
      reseed(seed);
      let res = void 0;
      while (!(res = pred(gen())))
        reseed();
      reset();
      return res;
    };
    return genUntil;
  }
  var validatorFns = {
    bigint: (val) => typeof val === "bigint",
    function: (val) => typeof val === "function",
    boolean: (val) => typeof val === "boolean",
    string: (val) => typeof val === "string",
    stringOrUint8Array: (val) => typeof val === "string" || val instanceof Uint8Array,
    isSafeInteger: (val) => Number.isSafeInteger(val),
    array: (val) => Array.isArray(val),
    field: (val, object) => object.Fp.isValid(val),
    hash: (val) => typeof val === "function" && Number.isSafeInteger(val.outputLen)
  };
  function validateObject(object, validators, optValidators = {}) {
    const checkField = (fieldName, type, isOptional) => {
      const checkVal = validatorFns[type];
      if (typeof checkVal !== "function")
        throw new Error(`Invalid validator "${type}", expected function`);
      const val = object[fieldName];
      if (isOptional && val === void 0)
        return;
      if (!checkVal(val, object)) {
        throw new Error(`Invalid param ${String(fieldName)}=${val} (${typeof val}), expected ${type}`);
      }
    };
    for (const [fieldName, type] of Object.entries(validators))
      checkField(fieldName, type, false);
    for (const [fieldName, type] of Object.entries(optValidators))
      checkField(fieldName, type, true);
    return object;
  }

  // node_modules/@noble/curves/esm/abstract/modular.js
  var _0n2 = BigInt(0);
  var _1n2 = BigInt(1);
  var _2n2 = BigInt(2);
  var _3n = BigInt(3);
  var _4n = BigInt(4);
  var _5n = BigInt(5);
  var _8n = BigInt(8);
  var _9n = BigInt(9);
  var _16n = BigInt(16);
  function mod(a, b) {
    const result = a % b;
    return result >= _0n2 ? result : b + result;
  }
  function pow(num, power, modulo) {
    if (modulo <= _0n2 || power < _0n2)
      throw new Error("Expected power/modulo > 0");
    if (modulo === _1n2)
      return _0n2;
    let res = _1n2;
    while (power > _0n2) {
      if (power & _1n2)
        res = res * num % modulo;
      num = num * num % modulo;
      power >>= _1n2;
    }
    return res;
  }
  function pow2(x, power, modulo) {
    let res = x;
    while (power-- > _0n2) {
      res *= res;
      res %= modulo;
    }
    return res;
  }
  function invert(number4, modulo) {
    if (number4 === _0n2 || modulo <= _0n2) {
      throw new Error(`invert: expected positive integers, got n=${number4} mod=${modulo}`);
    }
    let a = mod(number4, modulo);
    let b = modulo;
    let x = _0n2, y = _1n2, u = _1n2, v = _0n2;
    while (a !== _0n2) {
      const q = b / a;
      const r = b % a;
      const m = x - u * q;
      const n = y - v * q;
      b = a, a = r, x = u, y = v, u = m, v = n;
    }
    const gcd2 = b;
    if (gcd2 !== _1n2)
      throw new Error("invert: does not exist");
    return mod(x, modulo);
  }
  function tonelliShanks(P) {
    const legendreC = (P - _1n2) / _2n2;
    let Q, S, Z;
    for (Q = P - _1n2, S = 0; Q % _2n2 === _0n2; Q /= _2n2, S++)
      ;
    for (Z = _2n2; Z < P && pow(Z, legendreC, P) !== P - _1n2; Z++)
      ;
    if (S === 1) {
      const p1div4 = (P + _1n2) / _4n;
      return function tonelliFast(Fp2, n) {
        const root = Fp2.pow(n, p1div4);
        if (!Fp2.eql(Fp2.sqr(root), n))
          throw new Error("Cannot find square root");
        return root;
      };
    }
    const Q1div2 = (Q + _1n2) / _2n2;
    return function tonelliSlow(Fp2, n) {
      if (Fp2.pow(n, legendreC) === Fp2.neg(Fp2.ONE))
        throw new Error("Cannot find square root");
      let r = S;
      let g = Fp2.pow(Fp2.mul(Fp2.ONE, Z), Q);
      let x = Fp2.pow(n, Q1div2);
      let b = Fp2.pow(n, Q);
      while (!Fp2.eql(b, Fp2.ONE)) {
        if (Fp2.eql(b, Fp2.ZERO))
          return Fp2.ZERO;
        let m = 1;
        for (let t2 = Fp2.sqr(b); m < r; m++) {
          if (Fp2.eql(t2, Fp2.ONE))
            break;
          t2 = Fp2.sqr(t2);
        }
        const ge2 = Fp2.pow(g, _1n2 << BigInt(r - m - 1));
        g = Fp2.sqr(ge2);
        x = Fp2.mul(x, ge2);
        b = Fp2.mul(b, g);
        r = m;
      }
      return x;
    };
  }
  function FpSqrt(P) {
    if (P % _4n === _3n) {
      const p1div4 = (P + _1n2) / _4n;
      return function sqrt3mod4(Fp2, n) {
        const root = Fp2.pow(n, p1div4);
        if (!Fp2.eql(Fp2.sqr(root), n))
          throw new Error("Cannot find square root");
        return root;
      };
    }
    if (P % _8n === _5n) {
      const c1 = (P - _5n) / _8n;
      return function sqrt5mod8(Fp2, n) {
        const n2 = Fp2.mul(n, _2n2);
        const v = Fp2.pow(n2, c1);
        const nv = Fp2.mul(n, v);
        const i2 = Fp2.mul(Fp2.mul(nv, _2n2), v);
        const root = Fp2.mul(nv, Fp2.sub(i2, Fp2.ONE));
        if (!Fp2.eql(Fp2.sqr(root), n))
          throw new Error("Cannot find square root");
        return root;
      };
    }
    if (P % _16n === _9n) {
    }
    return tonelliShanks(P);
  }
  var FIELD_FIELDS = [
    "create",
    "isValid",
    "is0",
    "neg",
    "inv",
    "sqrt",
    "sqr",
    "eql",
    "add",
    "sub",
    "mul",
    "pow",
    "div",
    "addN",
    "subN",
    "mulN",
    "sqrN"
  ];
  function validateField(field) {
    const initial = {
      ORDER: "bigint",
      MASK: "bigint",
      BYTES: "isSafeInteger",
      BITS: "isSafeInteger"
    };
    const opts = FIELD_FIELDS.reduce((map, val) => {
      map[val] = "function";
      return map;
    }, initial);
    return validateObject(field, opts);
  }
  function FpPow(f, num, power) {
    if (power < _0n2)
      throw new Error("Expected power > 0");
    if (power === _0n2)
      return f.ONE;
    if (power === _1n2)
      return num;
    let p = f.ONE;
    let d = num;
    while (power > _0n2) {
      if (power & _1n2)
        p = f.mul(p, d);
      d = f.sqr(d);
      power >>= _1n2;
    }
    return p;
  }
  function FpInvertBatch(f, nums) {
    const tmp = new Array(nums.length);
    const lastMultiplied = nums.reduce((acc, num, i2) => {
      if (f.is0(num))
        return acc;
      tmp[i2] = acc;
      return f.mul(acc, num);
    }, f.ONE);
    const inverted = f.inv(lastMultiplied);
    nums.reduceRight((acc, num, i2) => {
      if (f.is0(num))
        return acc;
      tmp[i2] = f.mul(acc, tmp[i2]);
      return f.mul(acc, num);
    }, inverted);
    return tmp;
  }
  function nLength(n, nBitLength) {
    const _nBitLength = nBitLength !== void 0 ? nBitLength : n.toString(2).length;
    const nByteLength = Math.ceil(_nBitLength / 8);
    return { nBitLength: _nBitLength, nByteLength };
  }
  function Field(ORDER, bitLen2, isLE4 = false, redef = {}) {
    if (ORDER <= _0n2)
      throw new Error(`Expected Field ORDER > 0, got ${ORDER}`);
    const { nBitLength: BITS, nByteLength: BYTES } = nLength(ORDER, bitLen2);
    if (BYTES > 2048)
      throw new Error("Field lengths over 2048 bytes are not supported");
    const sqrtP = FpSqrt(ORDER);
    const f = Object.freeze({
      ORDER,
      BITS,
      BYTES,
      MASK: bitMask(BITS),
      ZERO: _0n2,
      ONE: _1n2,
      create: (num) => mod(num, ORDER),
      isValid: (num) => {
        if (typeof num !== "bigint")
          throw new Error(`Invalid field element: expected bigint, got ${typeof num}`);
        return _0n2 <= num && num < ORDER;
      },
      is0: (num) => num === _0n2,
      isOdd: (num) => (num & _1n2) === _1n2,
      neg: (num) => mod(-num, ORDER),
      eql: (lhs, rhs) => lhs === rhs,
      sqr: (num) => mod(num * num, ORDER),
      add: (lhs, rhs) => mod(lhs + rhs, ORDER),
      sub: (lhs, rhs) => mod(lhs - rhs, ORDER),
      mul: (lhs, rhs) => mod(lhs * rhs, ORDER),
      pow: (num, power) => FpPow(f, num, power),
      div: (lhs, rhs) => mod(lhs * invert(rhs, ORDER), ORDER),
      // Same as above, but doesn't normalize
      sqrN: (num) => num * num,
      addN: (lhs, rhs) => lhs + rhs,
      subN: (lhs, rhs) => lhs - rhs,
      mulN: (lhs, rhs) => lhs * rhs,
      inv: (num) => invert(num, ORDER),
      sqrt: redef.sqrt || ((n) => sqrtP(f, n)),
      invertBatch: (lst) => FpInvertBatch(f, lst),
      // TODO: do we really need constant cmov?
      // We don't have const-time bigints anyway, so probably will be not very useful
      cmov: (a, b, c) => c ? b : a,
      toBytes: (num) => isLE4 ? numberToBytesLE(num, BYTES) : numberToBytesBE(num, BYTES),
      fromBytes: (bytes4) => {
        if (bytes4.length !== BYTES)
          throw new Error(`Fp.fromBytes: expected ${BYTES}, got ${bytes4.length}`);
        return isLE4 ? bytesToNumberLE(bytes4) : bytesToNumberBE(bytes4);
      }
    });
    return Object.freeze(f);
  }
  function getFieldBytesLength(fieldOrder) {
    if (typeof fieldOrder !== "bigint")
      throw new Error("field order must be bigint");
    const bitLength = fieldOrder.toString(2).length;
    return Math.ceil(bitLength / 8);
  }
  function getMinHashLength(fieldOrder) {
    const length = getFieldBytesLength(fieldOrder);
    return length + Math.ceil(length / 2);
  }
  function mapHashToField(key, fieldOrder, isLE4 = false) {
    const len = key.length;
    const fieldLen = getFieldBytesLength(fieldOrder);
    const minLen = getMinHashLength(fieldOrder);
    if (len < 16 || len < minLen || len > 1024)
      throw new Error(`expected ${minLen}-1024 bytes of input, got ${len}`);
    const num = isLE4 ? bytesToNumberBE(key) : bytesToNumberLE(key);
    const reduced = mod(num, fieldOrder - _1n2) + _1n2;
    return isLE4 ? numberToBytesLE(reduced, fieldLen) : numberToBytesBE(reduced, fieldLen);
  }

  // node_modules/@noble/curves/esm/abstract/curve.js
  var _0n3 = BigInt(0);
  var _1n3 = BigInt(1);
  function wNAF(c, bits) {
    const constTimeNegate = (condition, item) => {
      const neg = item.negate();
      return condition ? neg : item;
    };
    const opts = (W) => {
      const windows = Math.ceil(bits / W) + 1;
      const windowSize = 2 ** (W - 1);
      return { windows, windowSize };
    };
    return {
      constTimeNegate,
      // non-const time multiplication ladder
      unsafeLadder(elm, n) {
        let p = c.ZERO;
        let d = elm;
        while (n > _0n3) {
          if (n & _1n3)
            p = p.add(d);
          d = d.double();
          n >>= _1n3;
        }
        return p;
      },
      /**
       * Creates a wNAF precomputation window. Used for caching.
       * Default window size is set by `utils.precompute()` and is equal to 8.
       * Number of precomputed points depends on the curve size:
       * 2^(𝑊−1) * (Math.ceil(𝑛 / 𝑊) + 1), where:
       * - 𝑊 is the window size
       * - 𝑛 is the bitlength of the curve order.
       * For a 256-bit curve and window size 8, the number of precomputed points is 128 * 33 = 4224.
       * @returns precomputed point tables flattened to a single array
       */
      precomputeWindow(elm, W) {
        const { windows, windowSize } = opts(W);
        const points = [];
        let p = elm;
        let base = p;
        for (let window = 0; window < windows; window++) {
          base = p;
          points.push(base);
          for (let i2 = 1; i2 < windowSize; i2++) {
            base = base.add(p);
            points.push(base);
          }
          p = base.double();
        }
        return points;
      },
      /**
       * Implements ec multiplication using precomputed tables and w-ary non-adjacent form.
       * @param W window size
       * @param precomputes precomputed tables
       * @param n scalar (we don't check here, but should be less than curve order)
       * @returns real and fake (for const-time) points
       */
      wNAF(W, precomputes, n) {
        const { windows, windowSize } = opts(W);
        let p = c.ZERO;
        let f = c.BASE;
        const mask = BigInt(2 ** W - 1);
        const maxNumber = 2 ** W;
        const shiftBy = BigInt(W);
        for (let window = 0; window < windows; window++) {
          const offset = window * windowSize;
          let wbits = Number(n & mask);
          n >>= shiftBy;
          if (wbits > windowSize) {
            wbits -= maxNumber;
            n += _1n3;
          }
          const offset1 = offset;
          const offset2 = offset + Math.abs(wbits) - 1;
          const cond1 = window % 2 !== 0;
          const cond2 = wbits < 0;
          if (wbits === 0) {
            f = f.add(constTimeNegate(cond1, precomputes[offset1]));
          } else {
            p = p.add(constTimeNegate(cond2, precomputes[offset2]));
          }
        }
        return { p, f };
      },
      wNAFCached(P, precomputesMap, n, transform) {
        const W = P._WINDOW_SIZE || 1;
        let comp = precomputesMap.get(P);
        if (!comp) {
          comp = this.precomputeWindow(P, W);
          if (W !== 1) {
            precomputesMap.set(P, transform(comp));
          }
        }
        return this.wNAF(W, comp, n);
      }
    };
  }
  function validateBasic(curve) {
    validateField(curve.Fp);
    validateObject(curve, {
      n: "bigint",
      h: "bigint",
      Gx: "field",
      Gy: "field"
    }, {
      nBitLength: "isSafeInteger",
      nByteLength: "isSafeInteger"
    });
    return Object.freeze({
      ...nLength(curve.n, curve.nBitLength),
      ...curve,
      ...{ p: curve.Fp.ORDER }
    });
  }

  // node_modules/@noble/curves/esm/abstract/weierstrass.js
  function validatePointOpts(curve) {
    const opts = validateBasic(curve);
    validateObject(opts, {
      a: "field",
      b: "field"
    }, {
      allowedPrivateKeyLengths: "array",
      wrapPrivateKey: "boolean",
      isTorsionFree: "function",
      clearCofactor: "function",
      allowInfinityPoint: "boolean",
      fromBytes: "function",
      toBytes: "function"
    });
    const { endo, Fp: Fp2, a } = opts;
    if (endo) {
      if (!Fp2.eql(a, Fp2.ZERO)) {
        throw new Error("Endomorphism can only be defined for Koblitz curves that have a=0");
      }
      if (typeof endo !== "object" || typeof endo.beta !== "bigint" || typeof endo.splitScalar !== "function") {
        throw new Error("Expected endomorphism with beta: bigint and splitScalar: function");
      }
    }
    return Object.freeze({ ...opts });
  }
  var { bytesToNumberBE: b2n, hexToBytes: h2b } = utils_exports;
  var DER = {
    // asn.1 DER encoding utils
    Err: class DERErr extends Error {
      constructor(m = "") {
        super(m);
      }
    },
    _parseInt(data) {
      const { Err: E } = DER;
      if (data.length < 2 || data[0] !== 2)
        throw new E("Invalid signature integer tag");
      const len = data[1];
      const res = data.subarray(2, len + 2);
      if (!len || res.length !== len)
        throw new E("Invalid signature integer: wrong length");
      if (res[0] & 128)
        throw new E("Invalid signature integer: negative");
      if (res[0] === 0 && !(res[1] & 128))
        throw new E("Invalid signature integer: unnecessary leading zero");
      return { d: b2n(res), l: data.subarray(len + 2) };
    },
    toSig(hex2) {
      const { Err: E } = DER;
      const data = typeof hex2 === "string" ? h2b(hex2) : hex2;
      if (!(data instanceof Uint8Array))
        throw new Error("ui8a expected");
      let l = data.length;
      if (l < 2 || data[0] != 48)
        throw new E("Invalid signature tag");
      if (data[1] !== l - 2)
        throw new E("Invalid signature: incorrect length");
      const { d: r, l: sBytes } = DER._parseInt(data.subarray(2));
      const { d: s, l: rBytesLeft } = DER._parseInt(sBytes);
      if (rBytesLeft.length)
        throw new E("Invalid signature: left bytes after parsing");
      return { r, s };
    },
    hexFromSig(sig) {
      const slice = (s2) => Number.parseInt(s2[0], 16) & 8 ? "00" + s2 : s2;
      const h = (num) => {
        const hex2 = num.toString(16);
        return hex2.length & 1 ? `0${hex2}` : hex2;
      };
      const s = slice(h(sig.s));
      const r = slice(h(sig.r));
      const shl = s.length / 2;
      const rhl = r.length / 2;
      const sl = h(shl);
      const rl = h(rhl);
      return `30${h(rhl + shl + 4)}02${rl}${r}02${sl}${s}`;
    }
  };
  var _0n4 = BigInt(0);
  var _1n4 = BigInt(1);
  var _2n3 = BigInt(2);
  var _3n2 = BigInt(3);
  var _4n2 = BigInt(4);
  function weierstrassPoints(opts) {
    const CURVE = validatePointOpts(opts);
    const { Fp: Fp2 } = CURVE;
    const toBytes4 = CURVE.toBytes || ((_c, point, _isCompressed) => {
      const a = point.toAffine();
      return concatBytes2(Uint8Array.from([4]), Fp2.toBytes(a.x), Fp2.toBytes(a.y));
    });
    const fromBytes = CURVE.fromBytes || ((bytes4) => {
      const tail = bytes4.subarray(1);
      const x = Fp2.fromBytes(tail.subarray(0, Fp2.BYTES));
      const y = Fp2.fromBytes(tail.subarray(Fp2.BYTES, 2 * Fp2.BYTES));
      return { x, y };
    });
    function weierstrassEquation(x) {
      const { a, b } = CURVE;
      const x2 = Fp2.sqr(x);
      const x3 = Fp2.mul(x2, x);
      return Fp2.add(Fp2.add(x3, Fp2.mul(x, a)), b);
    }
    if (!Fp2.eql(Fp2.sqr(CURVE.Gy), weierstrassEquation(CURVE.Gx)))
      throw new Error("bad generator point: equation left != right");
    function isWithinCurveOrder(num) {
      return typeof num === "bigint" && _0n4 < num && num < CURVE.n;
    }
    function assertGE(num) {
      if (!isWithinCurveOrder(num))
        throw new Error("Expected valid bigint: 0 < bigint < curve.n");
    }
    function normPrivateKeyToScalar(key) {
      const { allowedPrivateKeyLengths: lengths, nByteLength, wrapPrivateKey, n } = CURVE;
      if (lengths && typeof key !== "bigint") {
        if (key instanceof Uint8Array)
          key = bytesToHex(key);
        if (typeof key !== "string" || !lengths.includes(key.length))
          throw new Error("Invalid key");
        key = key.padStart(nByteLength * 2, "0");
      }
      let num;
      try {
        num = typeof key === "bigint" ? key : bytesToNumberBE(ensureBytes("private key", key, nByteLength));
      } catch (error) {
        throw new Error(`private key must be ${nByteLength} bytes, hex or bigint, not ${typeof key}`);
      }
      if (wrapPrivateKey)
        num = mod(num, n);
      assertGE(num);
      return num;
    }
    const pointPrecomputes = /* @__PURE__ */ new Map();
    function assertPrjPoint(other) {
      if (!(other instanceof Point2))
        throw new Error("ProjectivePoint expected");
    }
    class Point2 {
      constructor(px, py, pz) {
        this.px = px;
        this.py = py;
        this.pz = pz;
        if (px == null || !Fp2.isValid(px))
          throw new Error("x required");
        if (py == null || !Fp2.isValid(py))
          throw new Error("y required");
        if (pz == null || !Fp2.isValid(pz))
          throw new Error("z required");
      }
      // Does not validate if the point is on-curve.
      // Use fromHex instead, or call assertValidity() later.
      static fromAffine(p) {
        const { x, y } = p || {};
        if (!p || !Fp2.isValid(x) || !Fp2.isValid(y))
          throw new Error("invalid affine point");
        if (p instanceof Point2)
          throw new Error("projective point not allowed");
        const is0 = (i2) => Fp2.eql(i2, Fp2.ZERO);
        if (is0(x) && is0(y))
          return Point2.ZERO;
        return new Point2(x, y, Fp2.ONE);
      }
      get x() {
        return this.toAffine().x;
      }
      get y() {
        return this.toAffine().y;
      }
      /**
       * Takes a bunch of Projective Points but executes only one
       * inversion on all of them. Inversion is very slow operation,
       * so this improves performance massively.
       * Optimization: converts a list of projective points to a list of identical points with Z=1.
       */
      static normalizeZ(points) {
        const toInv = Fp2.invertBatch(points.map((p) => p.pz));
        return points.map((p, i2) => p.toAffine(toInv[i2])).map(Point2.fromAffine);
      }
      /**
       * Converts hash string or Uint8Array to Point.
       * @param hex short/long ECDSA hex
       */
      static fromHex(hex2) {
        const P = Point2.fromAffine(fromBytes(ensureBytes("pointHex", hex2)));
        P.assertValidity();
        return P;
      }
      // Multiplies generator point by privateKey.
      static fromPrivateKey(privateKey) {
        return Point2.BASE.multiply(normPrivateKeyToScalar(privateKey));
      }
      // "Private method", don't use it directly
      _setWindowSize(windowSize) {
        this._WINDOW_SIZE = windowSize;
        pointPrecomputes.delete(this);
      }
      // A point on curve is valid if it conforms to equation.
      assertValidity() {
        if (this.is0()) {
          if (CURVE.allowInfinityPoint && !Fp2.is0(this.py))
            return;
          throw new Error("bad point: ZERO");
        }
        const { x, y } = this.toAffine();
        if (!Fp2.isValid(x) || !Fp2.isValid(y))
          throw new Error("bad point: x or y not FE");
        const left = Fp2.sqr(y);
        const right = weierstrassEquation(x);
        if (!Fp2.eql(left, right))
          throw new Error("bad point: equation left != right");
        if (!this.isTorsionFree())
          throw new Error("bad point: not in prime-order subgroup");
      }
      hasEvenY() {
        const { y } = this.toAffine();
        if (Fp2.isOdd)
          return !Fp2.isOdd(y);
        throw new Error("Field doesn't support isOdd");
      }
      /**
       * Compare one point to another.
       */
      equals(other) {
        assertPrjPoint(other);
        const { px: X1, py: Y1, pz: Z1 } = this;
        const { px: X2, py: Y2, pz: Z2 } = other;
        const U1 = Fp2.eql(Fp2.mul(X1, Z2), Fp2.mul(X2, Z1));
        const U2 = Fp2.eql(Fp2.mul(Y1, Z2), Fp2.mul(Y2, Z1));
        return U1 && U2;
      }
      /**
       * Flips point to one corresponding to (x, -y) in Affine coordinates.
       */
      negate() {
        return new Point2(this.px, Fp2.neg(this.py), this.pz);
      }
      // Renes-Costello-Batina exception-free doubling formula.
      // There is 30% faster Jacobian formula, but it is not complete.
      // https://eprint.iacr.org/2015/1060, algorithm 3
      // Cost: 8M + 3S + 3*a + 2*b3 + 15add.
      double() {
        const { a, b } = CURVE;
        const b3 = Fp2.mul(b, _3n2);
        const { px: X1, py: Y1, pz: Z1 } = this;
        let X3 = Fp2.ZERO, Y3 = Fp2.ZERO, Z3 = Fp2.ZERO;
        let t0 = Fp2.mul(X1, X1);
        let t1 = Fp2.mul(Y1, Y1);
        let t2 = Fp2.mul(Z1, Z1);
        let t3 = Fp2.mul(X1, Y1);
        t3 = Fp2.add(t3, t3);
        Z3 = Fp2.mul(X1, Z1);
        Z3 = Fp2.add(Z3, Z3);
        X3 = Fp2.mul(a, Z3);
        Y3 = Fp2.mul(b3, t2);
        Y3 = Fp2.add(X3, Y3);
        X3 = Fp2.sub(t1, Y3);
        Y3 = Fp2.add(t1, Y3);
        Y3 = Fp2.mul(X3, Y3);
        X3 = Fp2.mul(t3, X3);
        Z3 = Fp2.mul(b3, Z3);
        t2 = Fp2.mul(a, t2);
        t3 = Fp2.sub(t0, t2);
        t3 = Fp2.mul(a, t3);
        t3 = Fp2.add(t3, Z3);
        Z3 = Fp2.add(t0, t0);
        t0 = Fp2.add(Z3, t0);
        t0 = Fp2.add(t0, t2);
        t0 = Fp2.mul(t0, t3);
        Y3 = Fp2.add(Y3, t0);
        t2 = Fp2.mul(Y1, Z1);
        t2 = Fp2.add(t2, t2);
        t0 = Fp2.mul(t2, t3);
        X3 = Fp2.sub(X3, t0);
        Z3 = Fp2.mul(t2, t1);
        Z3 = Fp2.add(Z3, Z3);
        Z3 = Fp2.add(Z3, Z3);
        return new Point2(X3, Y3, Z3);
      }
      // Renes-Costello-Batina exception-free addition formula.
      // There is 30% faster Jacobian formula, but it is not complete.
      // https://eprint.iacr.org/2015/1060, algorithm 1
      // Cost: 12M + 0S + 3*a + 3*b3 + 23add.
      add(other) {
        assertPrjPoint(other);
        const { px: X1, py: Y1, pz: Z1 } = this;
        const { px: X2, py: Y2, pz: Z2 } = other;
        let X3 = Fp2.ZERO, Y3 = Fp2.ZERO, Z3 = Fp2.ZERO;
        const a = CURVE.a;
        const b3 = Fp2.mul(CURVE.b, _3n2);
        let t0 = Fp2.mul(X1, X2);
        let t1 = Fp2.mul(Y1, Y2);
        let t2 = Fp2.mul(Z1, Z2);
        let t3 = Fp2.add(X1, Y1);
        let t4 = Fp2.add(X2, Y2);
        t3 = Fp2.mul(t3, t4);
        t4 = Fp2.add(t0, t1);
        t3 = Fp2.sub(t3, t4);
        t4 = Fp2.add(X1, Z1);
        let t5 = Fp2.add(X2, Z2);
        t4 = Fp2.mul(t4, t5);
        t5 = Fp2.add(t0, t2);
        t4 = Fp2.sub(t4, t5);
        t5 = Fp2.add(Y1, Z1);
        X3 = Fp2.add(Y2, Z2);
        t5 = Fp2.mul(t5, X3);
        X3 = Fp2.add(t1, t2);
        t5 = Fp2.sub(t5, X3);
        Z3 = Fp2.mul(a, t4);
        X3 = Fp2.mul(b3, t2);
        Z3 = Fp2.add(X3, Z3);
        X3 = Fp2.sub(t1, Z3);
        Z3 = Fp2.add(t1, Z3);
        Y3 = Fp2.mul(X3, Z3);
        t1 = Fp2.add(t0, t0);
        t1 = Fp2.add(t1, t0);
        t2 = Fp2.mul(a, t2);
        t4 = Fp2.mul(b3, t4);
        t1 = Fp2.add(t1, t2);
        t2 = Fp2.sub(t0, t2);
        t2 = Fp2.mul(a, t2);
        t4 = Fp2.add(t4, t2);
        t0 = Fp2.mul(t1, t4);
        Y3 = Fp2.add(Y3, t0);
        t0 = Fp2.mul(t5, t4);
        X3 = Fp2.mul(t3, X3);
        X3 = Fp2.sub(X3, t0);
        t0 = Fp2.mul(t3, t1);
        Z3 = Fp2.mul(t5, Z3);
        Z3 = Fp2.add(Z3, t0);
        return new Point2(X3, Y3, Z3);
      }
      subtract(other) {
        return this.add(other.negate());
      }
      is0() {
        return this.equals(Point2.ZERO);
      }
      wNAF(n) {
        return wnaf.wNAFCached(this, pointPrecomputes, n, (comp) => {
          const toInv = Fp2.invertBatch(comp.map((p) => p.pz));
          return comp.map((p, i2) => p.toAffine(toInv[i2])).map(Point2.fromAffine);
        });
      }
      /**
       * Non-constant-time multiplication. Uses double-and-add algorithm.
       * It's faster, but should only be used when you don't care about
       * an exposed private key e.g. sig verification, which works over *public* keys.
       */
      multiplyUnsafe(n) {
        const I = Point2.ZERO;
        if (n === _0n4)
          return I;
        assertGE(n);
        if (n === _1n4)
          return this;
        const { endo } = CURVE;
        if (!endo)
          return wnaf.unsafeLadder(this, n);
        let { k1neg, k1, k2neg, k2 } = endo.splitScalar(n);
        let k1p = I;
        let k2p = I;
        let d = this;
        while (k1 > _0n4 || k2 > _0n4) {
          if (k1 & _1n4)
            k1p = k1p.add(d);
          if (k2 & _1n4)
            k2p = k2p.add(d);
          d = d.double();
          k1 >>= _1n4;
          k2 >>= _1n4;
        }
        if (k1neg)
          k1p = k1p.negate();
        if (k2neg)
          k2p = k2p.negate();
        k2p = new Point2(Fp2.mul(k2p.px, endo.beta), k2p.py, k2p.pz);
        return k1p.add(k2p);
      }
      /**
       * Constant time multiplication.
       * Uses wNAF method. Windowed method may be 10% faster,
       * but takes 2x longer to generate and consumes 2x memory.
       * Uses precomputes when available.
       * Uses endomorphism for Koblitz curves.
       * @param scalar by which the point would be multiplied
       * @returns New point
       */
      multiply(scalar) {
        assertGE(scalar);
        let n = scalar;
        let point, fake;
        const { endo } = CURVE;
        if (endo) {
          const { k1neg, k1, k2neg, k2 } = endo.splitScalar(n);
          let { p: k1p, f: f1p } = this.wNAF(k1);
          let { p: k2p, f: f2p } = this.wNAF(k2);
          k1p = wnaf.constTimeNegate(k1neg, k1p);
          k2p = wnaf.constTimeNegate(k2neg, k2p);
          k2p = new Point2(Fp2.mul(k2p.px, endo.beta), k2p.py, k2p.pz);
          point = k1p.add(k2p);
          fake = f1p.add(f2p);
        } else {
          const { p, f } = this.wNAF(n);
          point = p;
          fake = f;
        }
        return Point2.normalizeZ([point, fake])[0];
      }
      /**
       * Efficiently calculate `aP + bQ`. Unsafe, can expose private key, if used incorrectly.
       * Not using Strauss-Shamir trick: precomputation tables are faster.
       * The trick could be useful if both P and Q are not G (not in our case).
       * @returns non-zero affine point
       */
      multiplyAndAddUnsafe(Q, a, b) {
        const G = Point2.BASE;
        const mul3 = (P, a2) => a2 === _0n4 || a2 === _1n4 || !P.equals(G) ? P.multiplyUnsafe(a2) : P.multiply(a2);
        const sum = mul3(this, a).add(mul3(Q, b));
        return sum.is0() ? void 0 : sum;
      }
      // Converts Projective point to affine (x, y) coordinates.
      // Can accept precomputed Z^-1 - for example, from invertBatch.
      // (x, y, z) ∋ (x=x/z, y=y/z)
      toAffine(iz) {
        const { px: x, py: y, pz: z } = this;
        const is0 = this.is0();
        if (iz == null)
          iz = is0 ? Fp2.ONE : Fp2.inv(z);
        const ax = Fp2.mul(x, iz);
        const ay = Fp2.mul(y, iz);
        const zz = Fp2.mul(z, iz);
        if (is0)
          return { x: Fp2.ZERO, y: Fp2.ZERO };
        if (!Fp2.eql(zz, Fp2.ONE))
          throw new Error("invZ was invalid");
        return { x: ax, y: ay };
      }
      isTorsionFree() {
        const { h: cofactor, isTorsionFree } = CURVE;
        if (cofactor === _1n4)
          return true;
        if (isTorsionFree)
          return isTorsionFree(Point2, this);
        throw new Error("isTorsionFree() has not been declared for the elliptic curve");
      }
      clearCofactor() {
        const { h: cofactor, clearCofactor } = CURVE;
        if (cofactor === _1n4)
          return this;
        if (clearCofactor)
          return clearCofactor(Point2, this);
        return this.multiplyUnsafe(CURVE.h);
      }
      toRawBytes(isCompressed = true) {
        this.assertValidity();
        return toBytes4(Point2, this, isCompressed);
      }
      toHex(isCompressed = true) {
        return bytesToHex(this.toRawBytes(isCompressed));
      }
    }
    Point2.BASE = new Point2(CURVE.Gx, CURVE.Gy, Fp2.ONE);
    Point2.ZERO = new Point2(Fp2.ZERO, Fp2.ONE, Fp2.ZERO);
    const _bits = CURVE.nBitLength;
    const wnaf = wNAF(Point2, CURVE.endo ? Math.ceil(_bits / 2) : _bits);
    return {
      CURVE,
      ProjectivePoint: Point2,
      normPrivateKeyToScalar,
      weierstrassEquation,
      isWithinCurveOrder
    };
  }
  function validateOpts(curve) {
    const opts = validateBasic(curve);
    validateObject(opts, {
      hash: "hash",
      hmac: "function",
      randomBytes: "function"
    }, {
      bits2int: "function",
      bits2int_modN: "function",
      lowS: "boolean"
    });
    return Object.freeze({ lowS: true, ...opts });
  }
  function weierstrass(curveDef) {
    const CURVE = validateOpts(curveDef);
    const { Fp: Fp2, n: CURVE_ORDER } = CURVE;
    const compressedLen = Fp2.BYTES + 1;
    const uncompressedLen = 2 * Fp2.BYTES + 1;
    function isValidFieldElement(num) {
      return _0n4 < num && num < Fp2.ORDER;
    }
    function modN2(a) {
      return mod(a, CURVE_ORDER);
    }
    function invN(a) {
      return invert(a, CURVE_ORDER);
    }
    const { ProjectivePoint: Point2, normPrivateKeyToScalar, weierstrassEquation, isWithinCurveOrder } = weierstrassPoints({
      ...CURVE,
      toBytes(_c, point, isCompressed) {
        const a = point.toAffine();
        const x = Fp2.toBytes(a.x);
        const cat = concatBytes2;
        if (isCompressed) {
          return cat(Uint8Array.from([point.hasEvenY() ? 2 : 3]), x);
        } else {
          return cat(Uint8Array.from([4]), x, Fp2.toBytes(a.y));
        }
      },
      fromBytes(bytes4) {
        const len = bytes4.length;
        const head = bytes4[0];
        const tail = bytes4.subarray(1);
        if (len === compressedLen && (head === 2 || head === 3)) {
          const x = bytesToNumberBE(tail);
          if (!isValidFieldElement(x))
            throw new Error("Point is not on curve");
          const y2 = weierstrassEquation(x);
          let y = Fp2.sqrt(y2);
          const isYOdd = (y & _1n4) === _1n4;
          const isHeadOdd = (head & 1) === 1;
          if (isHeadOdd !== isYOdd)
            y = Fp2.neg(y);
          return { x, y };
        } else if (len === uncompressedLen && head === 4) {
          const x = Fp2.fromBytes(tail.subarray(0, Fp2.BYTES));
          const y = Fp2.fromBytes(tail.subarray(Fp2.BYTES, 2 * Fp2.BYTES));
          return { x, y };
        } else {
          throw new Error(`Point of length ${len} was invalid. Expected ${compressedLen} compressed bytes or ${uncompressedLen} uncompressed bytes`);
        }
      }
    });
    const numToNByteStr = (num) => bytesToHex(numberToBytesBE(num, CURVE.nByteLength));
    function isBiggerThanHalfOrder(number4) {
      const HALF = CURVE_ORDER >> _1n4;
      return number4 > HALF;
    }
    function normalizeS(s) {
      return isBiggerThanHalfOrder(s) ? modN2(-s) : s;
    }
    const slcNum = (b, from, to) => bytesToNumberBE(b.slice(from, to));
    class Signature {
      constructor(r, s, recovery) {
        this.r = r;
        this.s = s;
        this.recovery = recovery;
        this.assertValidity();
      }
      // pair (bytes of r, bytes of s)
      static fromCompact(hex2) {
        const l = CURVE.nByteLength;
        hex2 = ensureBytes("compactSignature", hex2, l * 2);
        return new Signature(slcNum(hex2, 0, l), slcNum(hex2, l, 2 * l));
      }
      // DER encoded ECDSA signature
      // https://bitcoin.stackexchange.com/questions/57644/what-are-the-parts-of-a-bitcoin-transaction-input-script
      static fromDER(hex2) {
        const { r, s } = DER.toSig(ensureBytes("DER", hex2));
        return new Signature(r, s);
      }
      assertValidity() {
        if (!isWithinCurveOrder(this.r))
          throw new Error("r must be 0 < r < CURVE.n");
        if (!isWithinCurveOrder(this.s))
          throw new Error("s must be 0 < s < CURVE.n");
      }
      addRecoveryBit(recovery) {
        return new Signature(this.r, this.s, recovery);
      }
      recoverPublicKey(msgHash) {
        const { r, s, recovery: rec } = this;
        const h = bits2int_modN(ensureBytes("msgHash", msgHash));
        if (rec == null || ![0, 1, 2, 3].includes(rec))
          throw new Error("recovery id invalid");
        const radj = rec === 2 || rec === 3 ? r + CURVE.n : r;
        if (radj >= Fp2.ORDER)
          throw new Error("recovery id 2 or 3 invalid");
        const prefix = (rec & 1) === 0 ? "02" : "03";
        const R = Point2.fromHex(prefix + numToNByteStr(radj));
        const ir = invN(radj);
        const u1 = modN2(-h * ir);
        const u2 = modN2(s * ir);
        const Q = Point2.BASE.multiplyAndAddUnsafe(R, u1, u2);
        if (!Q)
          throw new Error("point at infinify");
        Q.assertValidity();
        return Q;
      }
      // Signatures should be low-s, to prevent malleability.
      hasHighS() {
        return isBiggerThanHalfOrder(this.s);
      }
      normalizeS() {
        return this.hasHighS() ? new Signature(this.r, modN2(-this.s), this.recovery) : this;
      }
      // DER-encoded
      toDERRawBytes() {
        return hexToBytes(this.toDERHex());
      }
      toDERHex() {
        return DER.hexFromSig({ r: this.r, s: this.s });
      }
      // padded bytes of r, then padded bytes of s
      toCompactRawBytes() {
        return hexToBytes(this.toCompactHex());
      }
      toCompactHex() {
        return numToNByteStr(this.r) + numToNByteStr(this.s);
      }
    }
    const utils = {
      isValidPrivateKey(privateKey) {
        try {
          normPrivateKeyToScalar(privateKey);
          return true;
        } catch (error) {
          return false;
        }
      },
      normPrivateKeyToScalar,
      /**
       * Produces cryptographically secure private key from random of size
       * (groupLen + ceil(groupLen / 2)) with modulo bias being negligible.
       */
      randomPrivateKey: () => {
        const length = getMinHashLength(CURVE.n);
        return mapHashToField(CURVE.randomBytes(length), CURVE.n);
      },
      /**
       * Creates precompute table for an arbitrary EC point. Makes point "cached".
       * Allows to massively speed-up `point.multiply(scalar)`.
       * @returns cached point
       * @example
       * const fast = utils.precompute(8, ProjectivePoint.fromHex(someonesPubKey));
       * fast.multiply(privKey); // much faster ECDH now
       */
      precompute(windowSize = 8, point = Point2.BASE) {
        point._setWindowSize(windowSize);
        point.multiply(BigInt(3));
        return point;
      }
    };
    function getPublicKey2(privateKey, isCompressed = true) {
      return Point2.fromPrivateKey(privateKey).toRawBytes(isCompressed);
    }
    function isProbPub(item) {
      const arr = item instanceof Uint8Array;
      const str = typeof item === "string";
      const len = (arr || str) && item.length;
      if (arr)
        return len === compressedLen || len === uncompressedLen;
      if (str)
        return len === 2 * compressedLen || len === 2 * uncompressedLen;
      if (item instanceof Point2)
        return true;
      return false;
    }
    function getSharedSecret(privateA, publicB, isCompressed = true) {
      if (isProbPub(privateA))
        throw new Error("first arg must be private key");
      if (!isProbPub(publicB))
        throw new Error("second arg must be public key");
      const b = Point2.fromHex(publicB);
      return b.multiply(normPrivateKeyToScalar(privateA)).toRawBytes(isCompressed);
    }
    const bits2int = CURVE.bits2int || function(bytes4) {
      const num = bytesToNumberBE(bytes4);
      const delta = bytes4.length * 8 - CURVE.nBitLength;
      return delta > 0 ? num >> BigInt(delta) : num;
    };
    const bits2int_modN = CURVE.bits2int_modN || function(bytes4) {
      return modN2(bits2int(bytes4));
    };
    const ORDER_MASK = bitMask(CURVE.nBitLength);
    function int2octets(num) {
      if (typeof num !== "bigint")
        throw new Error("bigint expected");
      if (!(_0n4 <= num && num < ORDER_MASK))
        throw new Error(`bigint expected < 2^${CURVE.nBitLength}`);
      return numberToBytesBE(num, CURVE.nByteLength);
    }
    function prepSig(msgHash, privateKey, opts = defaultSigOpts) {
      if (["recovered", "canonical"].some((k) => k in opts))
        throw new Error("sign() legacy options not supported");
      const { hash: hash3, randomBytes: randomBytes3 } = CURVE;
      let { lowS, prehash, extraEntropy: ent } = opts;
      if (lowS == null)
        lowS = true;
      msgHash = ensureBytes("msgHash", msgHash);
      if (prehash)
        msgHash = ensureBytes("prehashed msgHash", hash3(msgHash));
      const h1int = bits2int_modN(msgHash);
      const d = normPrivateKeyToScalar(privateKey);
      const seedArgs = [int2octets(d), int2octets(h1int)];
      if (ent != null) {
        const e = ent === true ? randomBytes3(Fp2.BYTES) : ent;
        seedArgs.push(ensureBytes("extraEntropy", e));
      }
      const seed = concatBytes2(...seedArgs);
      const m = h1int;
      function k2sig(kBytes) {
        const k = bits2int(kBytes);
        if (!isWithinCurveOrder(k))
          return;
        const ik = invN(k);
        const q = Point2.BASE.multiply(k).toAffine();
        const r = modN2(q.x);
        if (r === _0n4)
          return;
        const s = modN2(ik * modN2(m + r * d));
        if (s === _0n4)
          return;
        let recovery = (q.x === r ? 0 : 2) | Number(q.y & _1n4);
        let normS = s;
        if (lowS && isBiggerThanHalfOrder(s)) {
          normS = normalizeS(s);
          recovery ^= 1;
        }
        return new Signature(r, normS, recovery);
      }
      return { seed, k2sig };
    }
    const defaultSigOpts = { lowS: CURVE.lowS, prehash: false };
    const defaultVerOpts = { lowS: CURVE.lowS, prehash: false };
    function sign(msgHash, privKey, opts = defaultSigOpts) {
      const { seed, k2sig } = prepSig(msgHash, privKey, opts);
      const C = CURVE;
      const drbg = createHmacDrbg(C.hash.outputLen, C.nByteLength, C.hmac);
      return drbg(seed, k2sig);
    }
    Point2.BASE._setWindowSize(8);
    function verify(signature, msgHash, publicKey, opts = defaultVerOpts) {
      const sg = signature;
      msgHash = ensureBytes("msgHash", msgHash);
      publicKey = ensureBytes("publicKey", publicKey);
      if ("strict" in opts)
        throw new Error("options.strict was renamed to lowS");
      const { lowS, prehash } = opts;
      let _sig = void 0;
      let P;
      try {
        if (typeof sg === "string" || sg instanceof Uint8Array) {
          try {
            _sig = Signature.fromDER(sg);
          } catch (derError) {
            if (!(derError instanceof DER.Err))
              throw derError;
            _sig = Signature.fromCompact(sg);
          }
        } else if (typeof sg === "object" && typeof sg.r === "bigint" && typeof sg.s === "bigint") {
          const { r: r2, s: s2 } = sg;
          _sig = new Signature(r2, s2);
        } else {
          throw new Error("PARSE");
        }
        P = Point2.fromHex(publicKey);
      } catch (error) {
        if (error.message === "PARSE")
          throw new Error(`signature must be Signature instance, Uint8Array or hex string`);
        return false;
      }
      if (lowS && _sig.hasHighS())
        return false;
      if (prehash)
        msgHash = CURVE.hash(msgHash);
      const { r, s } = _sig;
      const h = bits2int_modN(msgHash);
      const is = invN(s);
      const u1 = modN2(h * is);
      const u2 = modN2(r * is);
      const R = Point2.BASE.multiplyAndAddUnsafe(P, u1, u2)?.toAffine();
      if (!R)
        return false;
      const v = modN2(R.x);
      return v === r;
    }
    return {
      CURVE,
      getPublicKey: getPublicKey2,
      getSharedSecret,
      sign,
      verify,
      ProjectivePoint: Point2,
      Signature,
      utils
    };
  }

  // node_modules/@noble/curves/node_modules/@noble/hashes/esm/hmac.js
  var HMAC = class extends Hash {
    constructor(hash3, _key) {
      super();
      this.finished = false;
      this.destroyed = false;
      hash(hash3);
      const key = toBytes(_key);
      this.iHash = hash3.create();
      if (typeof this.iHash.update !== "function")
        throw new Error("Expected instance of class which extends utils.Hash");
      this.blockLen = this.iHash.blockLen;
      this.outputLen = this.iHash.outputLen;
      const blockLen = this.blockLen;
      const pad2 = new Uint8Array(blockLen);
      pad2.set(key.length > blockLen ? hash3.create().update(key).digest() : key);
      for (let i2 = 0; i2 < pad2.length; i2++)
        pad2[i2] ^= 54;
      this.iHash.update(pad2);
      this.oHash = hash3.create();
      for (let i2 = 0; i2 < pad2.length; i2++)
        pad2[i2] ^= 54 ^ 92;
      this.oHash.update(pad2);
      pad2.fill(0);
    }
    update(buf) {
      exists(this);
      this.iHash.update(buf);
      return this;
    }
    digestInto(out) {
      exists(this);
      bytes(out, this.outputLen);
      this.finished = true;
      this.iHash.digestInto(out);
      this.oHash.update(out);
      this.oHash.digestInto(out);
      this.destroy();
    }
    digest() {
      const out = new Uint8Array(this.oHash.outputLen);
      this.digestInto(out);
      return out;
    }
    _cloneInto(to) {
      to || (to = Object.create(Object.getPrototypeOf(this), {}));
      const { oHash, iHash, finished, destroyed, blockLen, outputLen } = this;
      to = to;
      to.finished = finished;
      to.destroyed = destroyed;
      to.blockLen = blockLen;
      to.outputLen = outputLen;
      to.oHash = oHash._cloneInto(to.oHash);
      to.iHash = iHash._cloneInto(to.iHash);
      return to;
    }
    destroy() {
      this.destroyed = true;
      this.oHash.destroy();
      this.iHash.destroy();
    }
  };
  var hmac = (hash3, key, message) => new HMAC(hash3, key).update(message).digest();
  hmac.create = (hash3, key) => new HMAC(hash3, key);

  // node_modules/@noble/curves/esm/_shortw_utils.js
  function getHash(hash3) {
    return {
      hash: hash3,
      hmac: (key, ...msgs) => hmac(hash3, key, concatBytes(...msgs)),
      randomBytes
    };
  }
  function createCurve(curveDef, defHash) {
    const create = (hash3) => weierstrass({ ...curveDef, ...getHash(hash3) });
    return Object.freeze({ ...create(defHash), create });
  }

  // node_modules/@noble/curves/esm/secp256k1.js
  var secp256k1P = BigInt("0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
  var secp256k1N = BigInt("0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
  var _1n5 = BigInt(1);
  var _2n4 = BigInt(2);
  var divNearest = (a, b) => (a + b / _2n4) / b;
  function sqrtMod(y) {
    const P = secp256k1P;
    const _3n3 = BigInt(3), _6n = BigInt(6), _11n = BigInt(11), _22n = BigInt(22);
    const _23n = BigInt(23), _44n = BigInt(44), _88n = BigInt(88);
    const b2 = y * y * y % P;
    const b3 = b2 * b2 * y % P;
    const b6 = pow2(b3, _3n3, P) * b3 % P;
    const b9 = pow2(b6, _3n3, P) * b3 % P;
    const b11 = pow2(b9, _2n4, P) * b2 % P;
    const b22 = pow2(b11, _11n, P) * b11 % P;
    const b44 = pow2(b22, _22n, P) * b22 % P;
    const b88 = pow2(b44, _44n, P) * b44 % P;
    const b176 = pow2(b88, _88n, P) * b88 % P;
    const b220 = pow2(b176, _44n, P) * b44 % P;
    const b223 = pow2(b220, _3n3, P) * b3 % P;
    const t1 = pow2(b223, _23n, P) * b22 % P;
    const t2 = pow2(t1, _6n, P) * b2 % P;
    const root = pow2(t2, _2n4, P);
    if (!Fp.eql(Fp.sqr(root), y))
      throw new Error("Cannot find square root");
    return root;
  }
  var Fp = Field(secp256k1P, void 0, void 0, { sqrt: sqrtMod });
  var secp256k1 = createCurve({
    a: BigInt(0),
    b: BigInt(7),
    Fp,
    n: secp256k1N,
    // Base point (x, y) aka generator point
    Gx: BigInt("55066263022277343669578718895168534326250603453777594175500187360389116729240"),
    Gy: BigInt("32670510020758816978083085130507043184471273380659243275938904335757337482424"),
    h: BigInt(1),
    lowS: true,
    /**
     * secp256k1 belongs to Koblitz curves: it has efficiently computable endomorphism.
     * Endomorphism uses 2x less RAM, speeds up precomputation by 2x and ECDH / key recovery by 20%.
     * For precomputed wNAF it trades off 1/2 init time & 1/3 ram for 20% perf hit.
     * Explanation: https://gist.github.com/paulmillr/eb670806793e84df628a7c434a873066
     */
    endo: {
      beta: BigInt("0x7ae96a2b657c07106e64479eac3434e99cf0497512f58995c1396c28719501ee"),
      splitScalar: (k) => {
        const n = secp256k1N;
        const a1 = BigInt("0x3086d221a7d46bcde86c90e49284eb15");
        const b1 = -_1n5 * BigInt("0xe4437ed6010e88286f547fa90abfe4c3");
        const a2 = BigInt("0x114ca50f7a8e2f3f657c1108d9d44cfd8");
        const b2 = a1;
        const POW_2_128 = BigInt("0x100000000000000000000000000000000");
        const c1 = divNearest(b2 * k, n);
        const c2 = divNearest(-b1 * k, n);
        let k1 = mod(k - c1 * a1 - c2 * a2, n);
        let k2 = mod(-c1 * b1 - c2 * b2, n);
        const k1neg = k1 > POW_2_128;
        const k2neg = k2 > POW_2_128;
        if (k1neg)
          k1 = n - k1;
        if (k2neg)
          k2 = n - k2;
        if (k1 > POW_2_128 || k2 > POW_2_128) {
          throw new Error("splitScalar: Endomorphism failed, k=" + k);
        }
        return { k1neg, k1, k2neg, k2 };
      }
    }
  }, sha256);
  var _0n5 = BigInt(0);
  var fe = (x) => typeof x === "bigint" && _0n5 < x && x < secp256k1P;
  var ge = (x) => typeof x === "bigint" && _0n5 < x && x < secp256k1N;
  var TAGGED_HASH_PREFIXES = {};
  function taggedHash(tag, ...messages) {
    let tagP = TAGGED_HASH_PREFIXES[tag];
    if (tagP === void 0) {
      const tagH = sha256(Uint8Array.from(tag, (c) => c.charCodeAt(0)));
      tagP = concatBytes2(tagH, tagH);
      TAGGED_HASH_PREFIXES[tag] = tagP;
    }
    return sha256(concatBytes2(tagP, ...messages));
  }
  var pointToBytes = (point) => point.toRawBytes(true).slice(1);
  var numTo32b = (n) => numberToBytesBE(n, 32);
  var modP = (x) => mod(x, secp256k1P);
  var modN = (x) => mod(x, secp256k1N);
  var Point = secp256k1.ProjectivePoint;
  var GmulAdd = (Q, a, b) => Point.BASE.multiplyAndAddUnsafe(Q, a, b);
  function schnorrGetExtPubKey(priv) {
    let d_ = secp256k1.utils.normPrivateKeyToScalar(priv);
    let p = Point.fromPrivateKey(d_);
    const scalar = p.hasEvenY() ? d_ : modN(-d_);
    return { scalar, bytes: pointToBytes(p) };
  }
  function lift_x(x) {
    if (!fe(x))
      throw new Error("bad x: need 0 < x < p");
    const xx = modP(x * x);
    const c = modP(xx * x + BigInt(7));
    let y = sqrtMod(c);
    if (y % _2n4 !== _0n5)
      y = modP(-y);
    const p = new Point(x, y, _1n5);
    p.assertValidity();
    return p;
  }
  function challenge(...args) {
    return modN(bytesToNumberBE(taggedHash("BIP0340/challenge", ...args)));
  }
  function schnorrGetPublicKey(privateKey) {
    return schnorrGetExtPubKey(privateKey).bytes;
  }
  function schnorrSign(message, privateKey, auxRand = randomBytes(32)) {
    const m = ensureBytes("message", message);
    const { bytes: px, scalar: d } = schnorrGetExtPubKey(privateKey);
    const a = ensureBytes("auxRand", auxRand, 32);
    const t = numTo32b(d ^ bytesToNumberBE(taggedHash("BIP0340/aux", a)));
    const rand = taggedHash("BIP0340/nonce", t, px, m);
    const k_ = modN(bytesToNumberBE(rand));
    if (k_ === _0n5)
      throw new Error("sign failed: k is zero");
    const { bytes: rx, scalar: k } = schnorrGetExtPubKey(k_);
    const e = challenge(rx, px, m);
    const sig = new Uint8Array(64);
    sig.set(rx, 0);
    sig.set(numTo32b(modN(k + e * d)), 32);
    if (!schnorrVerify(sig, m, px))
      throw new Error("sign: Invalid signature produced");
    return sig;
  }
  function schnorrVerify(signature, message, publicKey) {
    const sig = ensureBytes("signature", signature, 64);
    const m = ensureBytes("message", message);
    const pub = ensureBytes("publicKey", publicKey, 32);
    try {
      const P = lift_x(bytesToNumberBE(pub));
      const r = bytesToNumberBE(sig.subarray(0, 32));
      if (!fe(r))
        return false;
      const s = bytesToNumberBE(sig.subarray(32, 64));
      if (!ge(s))
        return false;
      const e = challenge(numTo32b(r), pointToBytes(P), m);
      const R = GmulAdd(P, s, modN(-e));
      if (!R || !R.hasEvenY() || R.toAffine().x !== r)
        return false;
      return true;
    } catch (error) {
      return false;
    }
  }
  var schnorr = /* @__PURE__ */ (() => ({
    getPublicKey: schnorrGetPublicKey,
    sign: schnorrSign,
    verify: schnorrVerify,
    utils: {
      randomPrivateKey: secp256k1.utils.randomPrivateKey,
      lift_x,
      pointToBytes,
      numberToBytesBE,
      bytesToNumberBE,
      taggedHash,
      mod
    }
  }))();

  // node_modules/@noble/hashes/esm/crypto.js
  var crypto3 = typeof globalThis === "object" && "crypto" in globalThis ? globalThis.crypto : void 0;

  // node_modules/@noble/hashes/esm/utils.js
  var u8a3 = (a) => a instanceof Uint8Array;
  var createView2 = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
  var rotr2 = (word, shift) => word << 32 - shift | word >>> shift;
  var isLE2 = new Uint8Array(new Uint32Array([287454020]).buffer)[0] === 68;
  if (!isLE2)
    throw new Error("Non little-endian hardware is not supported");
  var hexes2 = Array.from({ length: 256 }, (v, i2) => i2.toString(16).padStart(2, "0"));
  function bytesToHex2(bytes4) {
    if (!u8a3(bytes4))
      throw new Error("Uint8Array expected");
    let hex2 = "";
    for (let i2 = 0; i2 < bytes4.length; i2++) {
      hex2 += hexes2[bytes4[i2]];
    }
    return hex2;
  }
  function hexToBytes2(hex2) {
    if (typeof hex2 !== "string")
      throw new Error("hex string expected, got " + typeof hex2);
    const len = hex2.length;
    if (len % 2)
      throw new Error("padded hex string expected, got unpadded hex of length " + len);
    const array = new Uint8Array(len / 2);
    for (let i2 = 0; i2 < array.length; i2++) {
      const j = i2 * 2;
      const hexByte = hex2.slice(j, j + 2);
      const byte = Number.parseInt(hexByte, 16);
      if (Number.isNaN(byte) || byte < 0)
        throw new Error("Invalid byte sequence");
      array[i2] = byte;
    }
    return array;
  }
  function utf8ToBytes3(str) {
    if (typeof str !== "string")
      throw new Error(`utf8ToBytes expected string, got ${typeof str}`);
    return new Uint8Array(new TextEncoder().encode(str));
  }
  function toBytes2(data) {
    if (typeof data === "string")
      data = utf8ToBytes3(data);
    if (!u8a3(data))
      throw new Error(`expected Uint8Array, got ${typeof data}`);
    return data;
  }
  function concatBytes3(...arrays) {
    const r = new Uint8Array(arrays.reduce((sum, a) => sum + a.length, 0));
    let pad2 = 0;
    arrays.forEach((a) => {
      if (!u8a3(a))
        throw new Error("Uint8Array expected");
      r.set(a, pad2);
      pad2 += a.length;
    });
    return r;
  }
  var Hash2 = class {
    // Safe version that clones internal state
    clone() {
      return this._cloneInto();
    }
  };
  function wrapConstructor2(hashCons) {
    const hashC = (msg) => hashCons().update(toBytes2(msg)).digest();
    const tmp = hashCons();
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = () => hashCons();
    return hashC;
  }
  function randomBytes2(bytesLength = 32) {
    if (crypto3 && typeof crypto3.getRandomValues === "function") {
      return crypto3.getRandomValues(new Uint8Array(bytesLength));
    }
    throw new Error("crypto.getRandomValues must be defined");
  }

  // node_modules/@noble/hashes/esm/_assert.js
  function number2(n) {
    if (!Number.isSafeInteger(n) || n < 0)
      throw new Error(`Wrong positive integer: ${n}`);
  }
  function bool(b) {
    if (typeof b !== "boolean")
      throw new Error(`Expected boolean, not ${b}`);
  }
  function bytes2(b, ...lengths) {
    if (!(b instanceof Uint8Array))
      throw new Error("Expected Uint8Array");
    if (lengths.length > 0 && !lengths.includes(b.length))
      throw new Error(`Expected Uint8Array of length ${lengths}, not of length=${b.length}`);
  }
  function hash2(hash3) {
    if (typeof hash3 !== "function" || typeof hash3.create !== "function")
      throw new Error("Hash should be wrapped by utils.wrapConstructor");
    number2(hash3.outputLen);
    number2(hash3.blockLen);
  }
  function exists2(instance, checkFinished = true) {
    if (instance.destroyed)
      throw new Error("Hash instance has been destroyed");
    if (checkFinished && instance.finished)
      throw new Error("Hash#digest() has already been called");
  }
  function output2(out, instance) {
    bytes2(out);
    const min = instance.outputLen;
    if (out.length < min) {
      throw new Error(`digestInto() expects output buffer of length at least ${min}`);
    }
  }
  var assert = {
    number: number2,
    bool,
    bytes: bytes2,
    hash: hash2,
    exists: exists2,
    output: output2
  };
  var assert_default = assert;

  // node_modules/@noble/hashes/esm/_sha2.js
  function setBigUint642(view, byteOffset, value, isLE4) {
    if (typeof view.setBigUint64 === "function")
      return view.setBigUint64(byteOffset, value, isLE4);
    const _32n = BigInt(32);
    const _u32_max = BigInt(4294967295);
    const wh = Number(value >> _32n & _u32_max);
    const wl = Number(value & _u32_max);
    const h = isLE4 ? 4 : 0;
    const l = isLE4 ? 0 : 4;
    view.setUint32(byteOffset + h, wh, isLE4);
    view.setUint32(byteOffset + l, wl, isLE4);
  }
  var SHA22 = class extends Hash2 {
    constructor(blockLen, outputLen, padOffset, isLE4) {
      super();
      this.blockLen = blockLen;
      this.outputLen = outputLen;
      this.padOffset = padOffset;
      this.isLE = isLE4;
      this.finished = false;
      this.length = 0;
      this.pos = 0;
      this.destroyed = false;
      this.buffer = new Uint8Array(blockLen);
      this.view = createView2(this.buffer);
    }
    update(data) {
      assert_default.exists(this);
      const { view, buffer, blockLen } = this;
      data = toBytes2(data);
      const len = data.length;
      for (let pos = 0; pos < len; ) {
        const take = Math.min(blockLen - this.pos, len - pos);
        if (take === blockLen) {
          const dataView = createView2(data);
          for (; blockLen <= len - pos; pos += blockLen)
            this.process(dataView, pos);
          continue;
        }
        buffer.set(data.subarray(pos, pos + take), this.pos);
        this.pos += take;
        pos += take;
        if (this.pos === blockLen) {
          this.process(view, 0);
          this.pos = 0;
        }
      }
      this.length += data.length;
      this.roundClean();
      return this;
    }
    digestInto(out) {
      assert_default.exists(this);
      assert_default.output(out, this);
      this.finished = true;
      const { buffer, view, blockLen, isLE: isLE4 } = this;
      let { pos } = this;
      buffer[pos++] = 128;
      this.buffer.subarray(pos).fill(0);
      if (this.padOffset > blockLen - pos) {
        this.process(view, 0);
        pos = 0;
      }
      for (let i2 = pos; i2 < blockLen; i2++)
        buffer[i2] = 0;
      setBigUint642(view, blockLen - 8, BigInt(this.length * 8), isLE4);
      this.process(view, 0);
      const oview = createView2(out);
      const len = this.outputLen;
      if (len % 4)
        throw new Error("_sha2: outputLen should be aligned to 32bit");
      const outLen = len / 4;
      const state = this.get();
      if (outLen > state.length)
        throw new Error("_sha2: outputLen bigger than state");
      for (let i2 = 0; i2 < outLen; i2++)
        oview.setUint32(4 * i2, state[i2], isLE4);
    }
    digest() {
      const { buffer, outputLen } = this;
      this.digestInto(buffer);
      const res = buffer.slice(0, outputLen);
      this.destroy();
      return res;
    }
    _cloneInto(to) {
      to || (to = new this.constructor());
      to.set(...this.get());
      const { blockLen, buffer, length, finished, destroyed, pos } = this;
      to.length = length;
      to.pos = pos;
      to.finished = finished;
      to.destroyed = destroyed;
      if (length % blockLen)
        to.buffer.set(buffer);
      return to;
    }
  };

  // node_modules/@noble/hashes/esm/sha256.js
  var Chi2 = (a, b, c) => a & b ^ ~a & c;
  var Maj2 = (a, b, c) => a & b ^ a & c ^ b & c;
  var SHA256_K2 = new Uint32Array([
    1116352408,
    1899447441,
    3049323471,
    3921009573,
    961987163,
    1508970993,
    2453635748,
    2870763221,
    3624381080,
    310598401,
    607225278,
    1426881987,
    1925078388,
    2162078206,
    2614888103,
    3248222580,
    3835390401,
    4022224774,
    264347078,
    604807628,
    770255983,
    1249150122,
    1555081692,
    1996064986,
    2554220882,
    2821834349,
    2952996808,
    3210313671,
    3336571891,
    3584528711,
    113926993,
    338241895,
    666307205,
    773529912,
    1294757372,
    1396182291,
    1695183700,
    1986661051,
    2177026350,
    2456956037,
    2730485921,
    2820302411,
    3259730800,
    3345764771,
    3516065817,
    3600352804,
    4094571909,
    275423344,
    430227734,
    506948616,
    659060556,
    883997877,
    958139571,
    1322822218,
    1537002063,
    1747873779,
    1955562222,
    2024104815,
    2227730452,
    2361852424,
    2428436474,
    2756734187,
    3204031479,
    3329325298
  ]);
  var IV2 = new Uint32Array([
    1779033703,
    3144134277,
    1013904242,
    2773480762,
    1359893119,
    2600822924,
    528734635,
    1541459225
  ]);
  var SHA256_W2 = new Uint32Array(64);
  var SHA2562 = class extends SHA22 {
    constructor() {
      super(64, 32, 8, false);
      this.A = IV2[0] | 0;
      this.B = IV2[1] | 0;
      this.C = IV2[2] | 0;
      this.D = IV2[3] | 0;
      this.E = IV2[4] | 0;
      this.F = IV2[5] | 0;
      this.G = IV2[6] | 0;
      this.H = IV2[7] | 0;
    }
    get() {
      const { A, B, C, D, E, F, G, H } = this;
      return [A, B, C, D, E, F, G, H];
    }
    // prettier-ignore
    set(A, B, C, D, E, F, G, H) {
      this.A = A | 0;
      this.B = B | 0;
      this.C = C | 0;
      this.D = D | 0;
      this.E = E | 0;
      this.F = F | 0;
      this.G = G | 0;
      this.H = H | 0;
    }
    process(view, offset) {
      for (let i2 = 0; i2 < 16; i2++, offset += 4)
        SHA256_W2[i2] = view.getUint32(offset, false);
      for (let i2 = 16; i2 < 64; i2++) {
        const W15 = SHA256_W2[i2 - 15];
        const W2 = SHA256_W2[i2 - 2];
        const s0 = rotr2(W15, 7) ^ rotr2(W15, 18) ^ W15 >>> 3;
        const s1 = rotr2(W2, 17) ^ rotr2(W2, 19) ^ W2 >>> 10;
        SHA256_W2[i2] = s1 + SHA256_W2[i2 - 7] + s0 + SHA256_W2[i2 - 16] | 0;
      }
      let { A, B, C, D, E, F, G, H } = this;
      for (let i2 = 0; i2 < 64; i2++) {
        const sigma1 = rotr2(E, 6) ^ rotr2(E, 11) ^ rotr2(E, 25);
        const T1 = H + sigma1 + Chi2(E, F, G) + SHA256_K2[i2] + SHA256_W2[i2] | 0;
        const sigma0 = rotr2(A, 2) ^ rotr2(A, 13) ^ rotr2(A, 22);
        const T2 = sigma0 + Maj2(A, B, C) | 0;
        H = G;
        G = F;
        F = E;
        E = D + T1 | 0;
        D = C;
        C = B;
        B = A;
        A = T1 + T2 | 0;
      }
      A = A + this.A | 0;
      B = B + this.B | 0;
      C = C + this.C | 0;
      D = D + this.D | 0;
      E = E + this.E | 0;
      F = F + this.F | 0;
      G = G + this.G | 0;
      H = H + this.H | 0;
      this.set(A, B, C, D, E, F, G, H);
    }
    roundClean() {
      SHA256_W2.fill(0);
    }
    destroy() {
      this.set(0, 0, 0, 0, 0, 0, 0, 0);
      this.buffer.fill(0);
    }
  };
  var SHA224 = class extends SHA2562 {
    constructor() {
      super();
      this.A = 3238371032 | 0;
      this.B = 914150663 | 0;
      this.C = 812702999 | 0;
      this.D = 4144912697 | 0;
      this.E = 4290775857 | 0;
      this.F = 1750603025 | 0;
      this.G = 1694076839 | 0;
      this.H = 3204075428 | 0;
      this.outputLen = 28;
    }
  };
  var sha2562 = wrapConstructor2(() => new SHA2562());
  var sha224 = wrapConstructor2(() => new SHA224());

  // node_modules/@scure/base/lib/esm/index.js
  function assertNumber(n) {
    if (!Number.isSafeInteger(n))
      throw new Error(`Wrong integer: ${n}`);
  }
  function chain(...args) {
    const wrap2 = (a, b) => (c) => a(b(c));
    const encode = Array.from(args).reverse().reduce((acc, i2) => acc ? wrap2(acc, i2.encode) : i2.encode, void 0);
    const decode2 = args.reduce((acc, i2) => acc ? wrap2(acc, i2.decode) : i2.decode, void 0);
    return { encode, decode: decode2 };
  }
  function alphabet(alphabet2) {
    return {
      encode: (digits) => {
        if (!Array.isArray(digits) || digits.length && typeof digits[0] !== "number")
          throw new Error("alphabet.encode input should be an array of numbers");
        return digits.map((i2) => {
          assertNumber(i2);
          if (i2 < 0 || i2 >= alphabet2.length)
            throw new Error(`Digit index outside alphabet: ${i2} (alphabet: ${alphabet2.length})`);
          return alphabet2[i2];
        });
      },
      decode: (input) => {
        if (!Array.isArray(input) || input.length && typeof input[0] !== "string")
          throw new Error("alphabet.decode input should be array of strings");
        return input.map((letter) => {
          if (typeof letter !== "string")
            throw new Error(`alphabet.decode: not string element=${letter}`);
          const index = alphabet2.indexOf(letter);
          if (index === -1)
            throw new Error(`Unknown letter: "${letter}". Allowed: ${alphabet2}`);
          return index;
        });
      }
    };
  }
  function join(separator = "") {
    if (typeof separator !== "string")
      throw new Error("join separator should be string");
    return {
      encode: (from) => {
        if (!Array.isArray(from) || from.length && typeof from[0] !== "string")
          throw new Error("join.encode input should be array of strings");
        for (let i2 of from)
          if (typeof i2 !== "string")
            throw new Error(`join.encode: non-string input=${i2}`);
        return from.join(separator);
      },
      decode: (to) => {
        if (typeof to !== "string")
          throw new Error("join.decode input should be string");
        return to.split(separator);
      }
    };
  }
  function padding(bits, chr = "=") {
    assertNumber(bits);
    if (typeof chr !== "string")
      throw new Error("padding chr should be string");
    return {
      encode(data) {
        if (!Array.isArray(data) || data.length && typeof data[0] !== "string")
          throw new Error("padding.encode input should be array of strings");
        for (let i2 of data)
          if (typeof i2 !== "string")
            throw new Error(`padding.encode: non-string input=${i2}`);
        while (data.length * bits % 8)
          data.push(chr);
        return data;
      },
      decode(input) {
        if (!Array.isArray(input) || input.length && typeof input[0] !== "string")
          throw new Error("padding.encode input should be array of strings");
        for (let i2 of input)
          if (typeof i2 !== "string")
            throw new Error(`padding.decode: non-string input=${i2}`);
        let end = input.length;
        if (end * bits % 8)
          throw new Error("Invalid padding: string should have whole number of bytes");
        for (; end > 0 && input[end - 1] === chr; end--) {
          if (!((end - 1) * bits % 8))
            throw new Error("Invalid padding: string has too much padding");
        }
        return input.slice(0, end);
      }
    };
  }
  function normalize(fn) {
    if (typeof fn !== "function")
      throw new Error("normalize fn should be function");
    return { encode: (from) => from, decode: (to) => fn(to) };
  }
  function convertRadix(data, from, to) {
    if (from < 2)
      throw new Error(`convertRadix: wrong from=${from}, base cannot be less than 2`);
    if (to < 2)
      throw new Error(`convertRadix: wrong to=${to}, base cannot be less than 2`);
    if (!Array.isArray(data))
      throw new Error("convertRadix: data should be array");
    if (!data.length)
      return [];
    let pos = 0;
    const res = [];
    const digits = Array.from(data);
    digits.forEach((d) => {
      assertNumber(d);
      if (d < 0 || d >= from)
        throw new Error(`Wrong integer: ${d}`);
    });
    while (true) {
      let carry = 0;
      let done = true;
      for (let i2 = pos; i2 < digits.length; i2++) {
        const digit = digits[i2];
        const digitBase = from * carry + digit;
        if (!Number.isSafeInteger(digitBase) || from * carry / from !== carry || digitBase - digit !== from * carry) {
          throw new Error("convertRadix: carry overflow");
        }
        carry = digitBase % to;
        digits[i2] = Math.floor(digitBase / to);
        if (!Number.isSafeInteger(digits[i2]) || digits[i2] * to + carry !== digitBase)
          throw new Error("convertRadix: carry overflow");
        if (!done)
          continue;
        else if (!digits[i2])
          pos = i2;
        else
          done = false;
      }
      res.push(carry);
      if (done)
        break;
    }
    for (let i2 = 0; i2 < data.length - 1 && data[i2] === 0; i2++)
      res.push(0);
    return res.reverse();
  }
  var gcd = (a, b) => !b ? a : gcd(b, a % b);
  var radix2carry = (from, to) => from + (to - gcd(from, to));
  function convertRadix2(data, from, to, padding2) {
    if (!Array.isArray(data))
      throw new Error("convertRadix2: data should be array");
    if (from <= 0 || from > 32)
      throw new Error(`convertRadix2: wrong from=${from}`);
    if (to <= 0 || to > 32)
      throw new Error(`convertRadix2: wrong to=${to}`);
    if (radix2carry(from, to) > 32) {
      throw new Error(`convertRadix2: carry overflow from=${from} to=${to} carryBits=${radix2carry(from, to)}`);
    }
    let carry = 0;
    let pos = 0;
    const mask = 2 ** to - 1;
    const res = [];
    for (const n of data) {
      assertNumber(n);
      if (n >= 2 ** from)
        throw new Error(`convertRadix2: invalid data word=${n} from=${from}`);
      carry = carry << from | n;
      if (pos + from > 32)
        throw new Error(`convertRadix2: carry overflow pos=${pos} from=${from}`);
      pos += from;
      for (; pos >= to; pos -= to)
        res.push((carry >> pos - to & mask) >>> 0);
      carry &= 2 ** pos - 1;
    }
    carry = carry << to - pos & mask;
    if (!padding2 && pos >= from)
      throw new Error("Excess padding");
    if (!padding2 && carry)
      throw new Error(`Non-zero padding: ${carry}`);
    if (padding2 && pos > 0)
      res.push(carry >>> 0);
    return res;
  }
  function radix(num) {
    assertNumber(num);
    return {
      encode: (bytes4) => {
        if (!(bytes4 instanceof Uint8Array))
          throw new Error("radix.encode input should be Uint8Array");
        return convertRadix(Array.from(bytes4), 2 ** 8, num);
      },
      decode: (digits) => {
        if (!Array.isArray(digits) || digits.length && typeof digits[0] !== "number")
          throw new Error("radix.decode input should be array of strings");
        return Uint8Array.from(convertRadix(digits, num, 2 ** 8));
      }
    };
  }
  function radix2(bits, revPadding = false) {
    assertNumber(bits);
    if (bits <= 0 || bits > 32)
      throw new Error("radix2: bits should be in (0..32]");
    if (radix2carry(8, bits) > 32 || radix2carry(bits, 8) > 32)
      throw new Error("radix2: carry overflow");
    return {
      encode: (bytes4) => {
        if (!(bytes4 instanceof Uint8Array))
          throw new Error("radix2.encode input should be Uint8Array");
        return convertRadix2(Array.from(bytes4), 8, bits, !revPadding);
      },
      decode: (digits) => {
        if (!Array.isArray(digits) || digits.length && typeof digits[0] !== "number")
          throw new Error("radix2.decode input should be array of strings");
        return Uint8Array.from(convertRadix2(digits, bits, 8, revPadding));
      }
    };
  }
  function unsafeWrapper(fn) {
    if (typeof fn !== "function")
      throw new Error("unsafeWrapper fn should be function");
    return function(...args) {
      try {
        return fn.apply(null, args);
      } catch (e) {
      }
    };
  }
  var base16 = chain(radix2(4), alphabet("0123456789ABCDEF"), join(""));
  var base32 = chain(radix2(5), alphabet("ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"), padding(5), join(""));
  var base32hex = chain(radix2(5), alphabet("0123456789ABCDEFGHIJKLMNOPQRSTUV"), padding(5), join(""));
  var base32crockford = chain(radix2(5), alphabet("0123456789ABCDEFGHJKMNPQRSTVWXYZ"), join(""), normalize((s) => s.toUpperCase().replace(/O/g, "0").replace(/[IL]/g, "1")));
  var base64 = chain(radix2(6), alphabet("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"), padding(6), join(""));
  var base64url = chain(radix2(6), alphabet("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"), padding(6), join(""));
  var genBase58 = (abc) => chain(radix(58), alphabet(abc), join(""));
  var base58 = genBase58("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
  var base58flickr = genBase58("123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ");
  var base58xrp = genBase58("rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz");
  var XMR_BLOCK_LEN = [0, 2, 3, 5, 6, 7, 9, 10, 11];
  var base58xmr = {
    encode(data) {
      let res = "";
      for (let i2 = 0; i2 < data.length; i2 += 8) {
        const block = data.subarray(i2, i2 + 8);
        res += base58.encode(block).padStart(XMR_BLOCK_LEN[block.length], "1");
      }
      return res;
    },
    decode(str) {
      let res = [];
      for (let i2 = 0; i2 < str.length; i2 += 11) {
        const slice = str.slice(i2, i2 + 11);
        const blockLen = XMR_BLOCK_LEN.indexOf(slice.length);
        const block = base58.decode(slice);
        for (let j = 0; j < block.length - blockLen; j++) {
          if (block[j] !== 0)
            throw new Error("base58xmr: wrong padding");
        }
        res = res.concat(Array.from(block.slice(block.length - blockLen)));
      }
      return Uint8Array.from(res);
    }
  };
  var BECH_ALPHABET = chain(alphabet("qpzry9x8gf2tvdw0s3jn54khce6mua7l"), join(""));
  var POLYMOD_GENERATORS = [996825010, 642813549, 513874426, 1027748829, 705979059];
  function bech32Polymod(pre) {
    const b = pre >> 25;
    let chk = (pre & 33554431) << 5;
    for (let i2 = 0; i2 < POLYMOD_GENERATORS.length; i2++) {
      if ((b >> i2 & 1) === 1)
        chk ^= POLYMOD_GENERATORS[i2];
    }
    return chk;
  }
  function bechChecksum(prefix, words, encodingConst = 1) {
    const len = prefix.length;
    let chk = 1;
    for (let i2 = 0; i2 < len; i2++) {
      const c = prefix.charCodeAt(i2);
      if (c < 33 || c > 126)
        throw new Error(`Invalid prefix (${prefix})`);
      chk = bech32Polymod(chk) ^ c >> 5;
    }
    chk = bech32Polymod(chk);
    for (let i2 = 0; i2 < len; i2++)
      chk = bech32Polymod(chk) ^ prefix.charCodeAt(i2) & 31;
    for (let v of words)
      chk = bech32Polymod(chk) ^ v;
    for (let i2 = 0; i2 < 6; i2++)
      chk = bech32Polymod(chk);
    chk ^= encodingConst;
    return BECH_ALPHABET.encode(convertRadix2([chk % 2 ** 30], 30, 5, false));
  }
  function genBech32(encoding) {
    const ENCODING_CONST = encoding === "bech32" ? 1 : 734539939;
    const _words = radix2(5);
    const fromWords = _words.decode;
    const toWords = _words.encode;
    const fromWordsUnsafe = unsafeWrapper(fromWords);
    function encode(prefix, words, limit2 = 90) {
      if (typeof prefix !== "string")
        throw new Error(`bech32.encode prefix should be string, not ${typeof prefix}`);
      if (!Array.isArray(words) || words.length && typeof words[0] !== "number")
        throw new Error(`bech32.encode words should be array of numbers, not ${typeof words}`);
      const actualLength = prefix.length + 7 + words.length;
      if (limit2 !== false && actualLength > limit2)
        throw new TypeError(`Length ${actualLength} exceeds limit ${limit2}`);
      prefix = prefix.toLowerCase();
      return `${prefix}1${BECH_ALPHABET.encode(words)}${bechChecksum(prefix, words, ENCODING_CONST)}`;
    }
    function decode2(str, limit2 = 90) {
      if (typeof str !== "string")
        throw new Error(`bech32.decode input should be string, not ${typeof str}`);
      if (str.length < 8 || limit2 !== false && str.length > limit2)
        throw new TypeError(`Wrong string length: ${str.length} (${str}). Expected (8..${limit2})`);
      const lowered = str.toLowerCase();
      if (str !== lowered && str !== str.toUpperCase())
        throw new Error(`String must be lowercase or uppercase`);
      str = lowered;
      const sepIndex = str.lastIndexOf("1");
      if (sepIndex === 0 || sepIndex === -1)
        throw new Error(`Letter "1" must be present between prefix and data only`);
      const prefix = str.slice(0, sepIndex);
      const _words2 = str.slice(sepIndex + 1);
      if (_words2.length < 6)
        throw new Error("Data must be at least 6 characters long");
      const words = BECH_ALPHABET.decode(_words2).slice(0, -6);
      const sum = bechChecksum(prefix, words, ENCODING_CONST);
      if (!_words2.endsWith(sum))
        throw new Error(`Invalid checksum in ${str}: expected "${sum}"`);
      return { prefix, words };
    }
    const decodeUnsafe = unsafeWrapper(decode2);
    function decodeToBytes(str) {
      const { prefix, words } = decode2(str, false);
      return { prefix, words, bytes: fromWords(words) };
    }
    return { encode, decode: decode2, decodeToBytes, decodeUnsafe, fromWords, fromWordsUnsafe, toWords };
  }
  var bech32 = genBech32("bech32");
  var bech32m = genBech32("bech32m");
  var utf8 = {
    encode: (data) => new TextDecoder().decode(data),
    decode: (str) => new TextEncoder().encode(str)
  };
  var hex = chain(radix2(4), alphabet("0123456789abcdef"), join(""), normalize((s) => {
    if (typeof s !== "string" || s.length % 2)
      throw new TypeError(`hex.decode: expected string, got ${typeof s} with length ${s.length}`);
    return s.toLowerCase();
  }));
  var CODERS = {
    utf8,
    hex,
    base16,
    base32,
    base64,
    base64url,
    base58,
    base58xmr
  };
  var coderTypeError = `Invalid encoding type. Available types: ${Object.keys(CODERS).join(", ")}`;

  // node_modules/@noble/ciphers/esm/_assert.js
  function number3(n) {
    if (!Number.isSafeInteger(n) || n < 0)
      throw new Error(`positive integer expected, not ${n}`);
  }
  function bool2(b) {
    if (typeof b !== "boolean")
      throw new Error(`boolean expected, not ${b}`);
  }
  function isBytes(a) {
    return a instanceof Uint8Array || a != null && typeof a === "object" && a.constructor.name === "Uint8Array";
  }
  function bytes3(b, ...lengths) {
    if (!isBytes(b))
      throw new Error("Uint8Array expected");
    if (lengths.length > 0 && !lengths.includes(b.length))
      throw new Error(`Uint8Array expected of length ${lengths}, not of length=${b.length}`);
  }
  function exists3(instance, checkFinished = true) {
    if (instance.destroyed)
      throw new Error("Hash instance has been destroyed");
    if (checkFinished && instance.finished)
      throw new Error("Hash#digest() has already been called");
  }
  function output3(out, instance) {
    bytes3(out);
    const min = instance.outputLen;
    if (out.length < min) {
      throw new Error(`digestInto() expects output buffer of length at least ${min}`);
    }
  }

  // node_modules/@noble/ciphers/esm/utils.js
  var u8 = (arr) => new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength);
  var u32 = (arr) => new Uint32Array(arr.buffer, arr.byteOffset, Math.floor(arr.byteLength / 4));
  var createView3 = (arr) => new DataView(arr.buffer, arr.byteOffset, arr.byteLength);
  var isLE3 = new Uint8Array(new Uint32Array([287454020]).buffer)[0] === 68;
  if (!isLE3)
    throw new Error("Non little-endian hardware is not supported");
  function utf8ToBytes4(str) {
    if (typeof str !== "string")
      throw new Error(`string expected, got ${typeof str}`);
    return new Uint8Array(new TextEncoder().encode(str));
  }
  function toBytes3(data) {
    if (typeof data === "string")
      data = utf8ToBytes4(data);
    else if (isBytes(data))
      data = data.slice();
    else
      throw new Error(`Uint8Array expected, got ${typeof data}`);
    return data;
  }
  function checkOpts(defaults, opts) {
    if (opts == null || typeof opts !== "object")
      throw new Error("options must be defined");
    const merged = Object.assign(defaults, opts);
    return merged;
  }
  function equalBytes2(a, b) {
    if (a.length !== b.length)
      return false;
    let diff = 0;
    for (let i2 = 0; i2 < a.length; i2++)
      diff |= a[i2] ^ b[i2];
    return diff === 0;
  }
  var wrapCipher = /* @__NO_SIDE_EFFECTS__ */ (params, c) => {
    Object.assign(c, params);
    return c;
  };
  function setBigUint643(view, byteOffset, value, isLE4) {
    if (typeof view.setBigUint64 === "function")
      return view.setBigUint64(byteOffset, value, isLE4);
    const _32n = BigInt(32);
    const _u32_max = BigInt(4294967295);
    const wh = Number(value >> _32n & _u32_max);
    const wl = Number(value & _u32_max);
    const h = isLE4 ? 4 : 0;
    const l = isLE4 ? 0 : 4;
    view.setUint32(byteOffset + h, wh, isLE4);
    view.setUint32(byteOffset + l, wl, isLE4);
  }

  // node_modules/@noble/ciphers/esm/_polyval.js
  var BLOCK_SIZE = 16;
  var ZEROS16 = /* @__PURE__ */ new Uint8Array(16);
  var ZEROS32 = u32(ZEROS16);
  var POLY = 225;
  var mul2 = (s0, s1, s2, s3) => {
    const hiBit = s3 & 1;
    return {
      s3: s2 << 31 | s3 >>> 1,
      s2: s1 << 31 | s2 >>> 1,
      s1: s0 << 31 | s1 >>> 1,
      s0: s0 >>> 1 ^ POLY << 24 & -(hiBit & 1)
      // reduce % poly
    };
  };
  var swapLE = (n) => (n >>> 0 & 255) << 24 | (n >>> 8 & 255) << 16 | (n >>> 16 & 255) << 8 | n >>> 24 & 255 | 0;
  function _toGHASHKey(k) {
    k.reverse();
    const hiBit = k[15] & 1;
    let carry = 0;
    for (let i2 = 0; i2 < k.length; i2++) {
      const t = k[i2];
      k[i2] = t >>> 1 | carry;
      carry = (t & 1) << 7;
    }
    k[0] ^= -hiBit & 225;
    return k;
  }
  var estimateWindow = (bytes4) => {
    if (bytes4 > 64 * 1024)
      return 8;
    if (bytes4 > 1024)
      return 4;
    return 2;
  };
  var GHASH = class {
    // We select bits per window adaptively based on expectedLength
    constructor(key, expectedLength) {
      this.blockLen = BLOCK_SIZE;
      this.outputLen = BLOCK_SIZE;
      this.s0 = 0;
      this.s1 = 0;
      this.s2 = 0;
      this.s3 = 0;
      this.finished = false;
      key = toBytes3(key);
      bytes3(key, 16);
      const kView = createView3(key);
      let k0 = kView.getUint32(0, false);
      let k1 = kView.getUint32(4, false);
      let k2 = kView.getUint32(8, false);
      let k3 = kView.getUint32(12, false);
      const doubles = [];
      for (let i2 = 0; i2 < 128; i2++) {
        doubles.push({ s0: swapLE(k0), s1: swapLE(k1), s2: swapLE(k2), s3: swapLE(k3) });
        ({ s0: k0, s1: k1, s2: k2, s3: k3 } = mul2(k0, k1, k2, k3));
      }
      const W = estimateWindow(expectedLength || 1024);
      if (![1, 2, 4, 8].includes(W))
        throw new Error(`ghash: wrong window size=${W}, should be 2, 4 or 8`);
      this.W = W;
      const bits = 128;
      const windows = bits / W;
      const windowSize = this.windowSize = 2 ** W;
      const items = [];
      for (let w = 0; w < windows; w++) {
        for (let byte = 0; byte < windowSize; byte++) {
          let s0 = 0, s1 = 0, s2 = 0, s3 = 0;
          for (let j = 0; j < W; j++) {
            const bit = byte >>> W - j - 1 & 1;
            if (!bit)
              continue;
            const { s0: d0, s1: d1, s2: d2, s3: d3 } = doubles[W * w + j];
            s0 ^= d0, s1 ^= d1, s2 ^= d2, s3 ^= d3;
          }
          items.push({ s0, s1, s2, s3 });
        }
      }
      this.t = items;
    }
    _updateBlock(s0, s1, s2, s3) {
      s0 ^= this.s0, s1 ^= this.s1, s2 ^= this.s2, s3 ^= this.s3;
      const { W, t, windowSize } = this;
      let o0 = 0, o1 = 0, o2 = 0, o3 = 0;
      const mask = (1 << W) - 1;
      let w = 0;
      for (const num of [s0, s1, s2, s3]) {
        for (let bytePos = 0; bytePos < 4; bytePos++) {
          const byte = num >>> 8 * bytePos & 255;
          for (let bitPos = 8 / W - 1; bitPos >= 0; bitPos--) {
            const bit = byte >>> W * bitPos & mask;
            const { s0: e0, s1: e1, s2: e2, s3: e3 } = t[w * windowSize + bit];
            o0 ^= e0, o1 ^= e1, o2 ^= e2, o3 ^= e3;
            w += 1;
          }
        }
      }
      this.s0 = o0;
      this.s1 = o1;
      this.s2 = o2;
      this.s3 = o3;
    }
    update(data) {
      data = toBytes3(data);
      exists3(this);
      const b32 = u32(data);
      const blocks = Math.floor(data.length / BLOCK_SIZE);
      const left = data.length % BLOCK_SIZE;
      for (let i2 = 0; i2 < blocks; i2++) {
        this._updateBlock(b32[i2 * 4 + 0], b32[i2 * 4 + 1], b32[i2 * 4 + 2], b32[i2 * 4 + 3]);
      }
      if (left) {
        ZEROS16.set(data.subarray(blocks * BLOCK_SIZE));
        this._updateBlock(ZEROS32[0], ZEROS32[1], ZEROS32[2], ZEROS32[3]);
        ZEROS32.fill(0);
      }
      return this;
    }
    destroy() {
      const { t } = this;
      for (const elm of t) {
        elm.s0 = 0, elm.s1 = 0, elm.s2 = 0, elm.s3 = 0;
      }
    }
    digestInto(out) {
      exists3(this);
      output3(out, this);
      this.finished = true;
      const { s0, s1, s2, s3 } = this;
      const o32 = u32(out);
      o32[0] = s0;
      o32[1] = s1;
      o32[2] = s2;
      o32[3] = s3;
      return out;
    }
    digest() {
      const res = new Uint8Array(BLOCK_SIZE);
      this.digestInto(res);
      this.destroy();
      return res;
    }
  };
  var Polyval = class extends GHASH {
    constructor(key, expectedLength) {
      key = toBytes3(key);
      const ghKey = _toGHASHKey(key.slice());
      super(ghKey, expectedLength);
      ghKey.fill(0);
    }
    update(data) {
      data = toBytes3(data);
      exists3(this);
      const b32 = u32(data);
      const left = data.length % BLOCK_SIZE;
      const blocks = Math.floor(data.length / BLOCK_SIZE);
      for (let i2 = 0; i2 < blocks; i2++) {
        this._updateBlock(swapLE(b32[i2 * 4 + 3]), swapLE(b32[i2 * 4 + 2]), swapLE(b32[i2 * 4 + 1]), swapLE(b32[i2 * 4 + 0]));
      }
      if (left) {
        ZEROS16.set(data.subarray(blocks * BLOCK_SIZE));
        this._updateBlock(swapLE(ZEROS32[3]), swapLE(ZEROS32[2]), swapLE(ZEROS32[1]), swapLE(ZEROS32[0]));
        ZEROS32.fill(0);
      }
      return this;
    }
    digestInto(out) {
      exists3(this);
      output3(out, this);
      this.finished = true;
      const { s0, s1, s2, s3 } = this;
      const o32 = u32(out);
      o32[0] = s0;
      o32[1] = s1;
      o32[2] = s2;
      o32[3] = s3;
      return out.reverse();
    }
  };
  function wrapConstructorWithKey(hashCons) {
    const hashC = (msg, key) => hashCons(key, msg.length).update(toBytes3(msg)).digest();
    const tmp = hashCons(new Uint8Array(16), 0);
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = (key, expectedLength) => hashCons(key, expectedLength);
    return hashC;
  }
  var ghash = wrapConstructorWithKey((key, expectedLength) => new GHASH(key, expectedLength));
  var polyval = wrapConstructorWithKey((key, expectedLength) => new Polyval(key, expectedLength));

  // node_modules/@noble/ciphers/esm/aes.js
  var BLOCK_SIZE2 = 16;
  var BLOCK_SIZE32 = 4;
  var EMPTY_BLOCK = new Uint8Array(BLOCK_SIZE2);
  var POLY2 = 283;
  function mul22(n) {
    return n << 1 ^ POLY2 & -(n >> 7);
  }
  function mul(a, b) {
    let res = 0;
    for (; b > 0; b >>= 1) {
      res ^= a & -(b & 1);
      a = mul22(a);
    }
    return res;
  }
  var sbox = /* @__PURE__ */ (() => {
    let t = new Uint8Array(256);
    for (let i2 = 0, x = 1; i2 < 256; i2++, x ^= mul22(x))
      t[i2] = x;
    const box = new Uint8Array(256);
    box[0] = 99;
    for (let i2 = 0; i2 < 255; i2++) {
      let x = t[255 - i2];
      x |= x << 8;
      box[t[i2]] = (x ^ x >> 4 ^ x >> 5 ^ x >> 6 ^ x >> 7 ^ 99) & 255;
    }
    return box;
  })();
  var invSbox = /* @__PURE__ */ sbox.map((_, j) => sbox.indexOf(j));
  var rotr32_8 = (n) => n << 24 | n >>> 8;
  var rotl32_8 = (n) => n << 8 | n >>> 24;
  function genTtable(sbox2, fn) {
    if (sbox2.length !== 256)
      throw new Error("Wrong sbox length");
    const T0 = new Uint32Array(256).map((_, j) => fn(sbox2[j]));
    const T1 = T0.map(rotl32_8);
    const T2 = T1.map(rotl32_8);
    const T3 = T2.map(rotl32_8);
    const T01 = new Uint32Array(256 * 256);
    const T23 = new Uint32Array(256 * 256);
    const sbox22 = new Uint16Array(256 * 256);
    for (let i2 = 0; i2 < 256; i2++) {
      for (let j = 0; j < 256; j++) {
        const idx = i2 * 256 + j;
        T01[idx] = T0[i2] ^ T1[j];
        T23[idx] = T2[i2] ^ T3[j];
        sbox22[idx] = sbox2[i2] << 8 | sbox2[j];
      }
    }
    return { sbox: sbox2, sbox2: sbox22, T0, T1, T2, T3, T01, T23 };
  }
  var tableEncoding = /* @__PURE__ */ genTtable(sbox, (s) => mul(s, 3) << 24 | s << 16 | s << 8 | mul(s, 2));
  var tableDecoding = /* @__PURE__ */ genTtable(invSbox, (s) => mul(s, 11) << 24 | mul(s, 13) << 16 | mul(s, 9) << 8 | mul(s, 14));
  var xPowers = /* @__PURE__ */ (() => {
    const p = new Uint8Array(16);
    for (let i2 = 0, x = 1; i2 < 16; i2++, x = mul22(x))
      p[i2] = x;
    return p;
  })();
  function expandKeyLE(key) {
    bytes3(key);
    const len = key.length;
    if (![16, 24, 32].includes(len))
      throw new Error(`aes: wrong key size: should be 16, 24 or 32, got: ${len}`);
    const { sbox2 } = tableEncoding;
    const k32 = u32(key);
    const Nk = k32.length;
    const subByte = (n) => applySbox(sbox2, n, n, n, n);
    const xk = new Uint32Array(len + 28);
    xk.set(k32);
    for (let i2 = Nk; i2 < xk.length; i2++) {
      let t = xk[i2 - 1];
      if (i2 % Nk === 0)
        t = subByte(rotr32_8(t)) ^ xPowers[i2 / Nk - 1];
      else if (Nk > 6 && i2 % Nk === 4)
        t = subByte(t);
      xk[i2] = xk[i2 - Nk] ^ t;
    }
    return xk;
  }
  function expandKeyDecLE(key) {
    const encKey = expandKeyLE(key);
    const xk = encKey.slice();
    const Nk = encKey.length;
    const { sbox2 } = tableEncoding;
    const { T0, T1, T2, T3 } = tableDecoding;
    for (let i2 = 0; i2 < Nk; i2 += 4) {
      for (let j = 0; j < 4; j++)
        xk[i2 + j] = encKey[Nk - i2 - 4 + j];
    }
    encKey.fill(0);
    for (let i2 = 4; i2 < Nk - 4; i2++) {
      const x = xk[i2];
      const w = applySbox(sbox2, x, x, x, x);
      xk[i2] = T0[w & 255] ^ T1[w >>> 8 & 255] ^ T2[w >>> 16 & 255] ^ T3[w >>> 24];
    }
    return xk;
  }
  function apply0123(T01, T23, s0, s1, s2, s3) {
    return T01[s0 << 8 & 65280 | s1 >>> 8 & 255] ^ T23[s2 >>> 8 & 65280 | s3 >>> 24 & 255];
  }
  function applySbox(sbox2, s0, s1, s2, s3) {
    return sbox2[s0 & 255 | s1 & 65280] | sbox2[s2 >>> 16 & 255 | s3 >>> 16 & 65280] << 16;
  }
  function encrypt(xk, s0, s1, s2, s3) {
    const { sbox2, T01, T23 } = tableEncoding;
    let k = 0;
    s0 ^= xk[k++], s1 ^= xk[k++], s2 ^= xk[k++], s3 ^= xk[k++];
    const rounds = xk.length / 4 - 2;
    for (let i2 = 0; i2 < rounds; i2++) {
      const t02 = xk[k++] ^ apply0123(T01, T23, s0, s1, s2, s3);
      const t12 = xk[k++] ^ apply0123(T01, T23, s1, s2, s3, s0);
      const t22 = xk[k++] ^ apply0123(T01, T23, s2, s3, s0, s1);
      const t32 = xk[k++] ^ apply0123(T01, T23, s3, s0, s1, s2);
      s0 = t02, s1 = t12, s2 = t22, s3 = t32;
    }
    const t0 = xk[k++] ^ applySbox(sbox2, s0, s1, s2, s3);
    const t1 = xk[k++] ^ applySbox(sbox2, s1, s2, s3, s0);
    const t2 = xk[k++] ^ applySbox(sbox2, s2, s3, s0, s1);
    const t3 = xk[k++] ^ applySbox(sbox2, s3, s0, s1, s2);
    return { s0: t0, s1: t1, s2: t2, s3: t3 };
  }
  function decrypt(xk, s0, s1, s2, s3) {
    const { sbox2, T01, T23 } = tableDecoding;
    let k = 0;
    s0 ^= xk[k++], s1 ^= xk[k++], s2 ^= xk[k++], s3 ^= xk[k++];
    const rounds = xk.length / 4 - 2;
    for (let i2 = 0; i2 < rounds; i2++) {
      const t02 = xk[k++] ^ apply0123(T01, T23, s0, s3, s2, s1);
      const t12 = xk[k++] ^ apply0123(T01, T23, s1, s0, s3, s2);
      const t22 = xk[k++] ^ apply0123(T01, T23, s2, s1, s0, s3);
      const t32 = xk[k++] ^ apply0123(T01, T23, s3, s2, s1, s0);
      s0 = t02, s1 = t12, s2 = t22, s3 = t32;
    }
    const t0 = xk[k++] ^ applySbox(sbox2, s0, s3, s2, s1);
    const t1 = xk[k++] ^ applySbox(sbox2, s1, s0, s3, s2);
    const t2 = xk[k++] ^ applySbox(sbox2, s2, s1, s0, s3);
    const t3 = xk[k++] ^ applySbox(sbox2, s3, s2, s1, s0);
    return { s0: t0, s1: t1, s2: t2, s3: t3 };
  }
  function getDst(len, dst) {
    if (!dst)
      return new Uint8Array(len);
    bytes3(dst);
    if (dst.length < len)
      throw new Error(`aes: wrong destination length, expected at least ${len}, got: ${dst.length}`);
    return dst;
  }
  function ctrCounter(xk, nonce, src, dst) {
    bytes3(nonce, BLOCK_SIZE2);
    bytes3(src);
    const srcLen = src.length;
    dst = getDst(srcLen, dst);
    const ctr3 = nonce;
    const c32 = u32(ctr3);
    let { s0, s1, s2, s3 } = encrypt(xk, c32[0], c32[1], c32[2], c32[3]);
    const src32 = u32(src);
    const dst32 = u32(dst);
    for (let i2 = 0; i2 + 4 <= src32.length; i2 += 4) {
      dst32[i2 + 0] = src32[i2 + 0] ^ s0;
      dst32[i2 + 1] = src32[i2 + 1] ^ s1;
      dst32[i2 + 2] = src32[i2 + 2] ^ s2;
      dst32[i2 + 3] = src32[i2 + 3] ^ s3;
      let carry = 1;
      for (let i3 = ctr3.length - 1; i3 >= 0; i3--) {
        carry = carry + (ctr3[i3] & 255) | 0;
        ctr3[i3] = carry & 255;
        carry >>>= 8;
      }
      ({ s0, s1, s2, s3 } = encrypt(xk, c32[0], c32[1], c32[2], c32[3]));
    }
    const start = BLOCK_SIZE2 * Math.floor(src32.length / BLOCK_SIZE32);
    if (start < srcLen) {
      const b32 = new Uint32Array([s0, s1, s2, s3]);
      const buf = u8(b32);
      for (let i2 = start, pos = 0; i2 < srcLen; i2++, pos++)
        dst[i2] = src[i2] ^ buf[pos];
    }
    return dst;
  }
  function ctr32(xk, isLE4, nonce, src, dst) {
    bytes3(nonce, BLOCK_SIZE2);
    bytes3(src);
    dst = getDst(src.length, dst);
    const ctr3 = nonce;
    const c32 = u32(ctr3);
    const view = createView3(ctr3);
    const src32 = u32(src);
    const dst32 = u32(dst);
    const ctrPos = isLE4 ? 0 : 12;
    const srcLen = src.length;
    let ctrNum = view.getUint32(ctrPos, isLE4);
    let { s0, s1, s2, s3 } = encrypt(xk, c32[0], c32[1], c32[2], c32[3]);
    for (let i2 = 0; i2 + 4 <= src32.length; i2 += 4) {
      dst32[i2 + 0] = src32[i2 + 0] ^ s0;
      dst32[i2 + 1] = src32[i2 + 1] ^ s1;
      dst32[i2 + 2] = src32[i2 + 2] ^ s2;
      dst32[i2 + 3] = src32[i2 + 3] ^ s3;
      ctrNum = ctrNum + 1 >>> 0;
      view.setUint32(ctrPos, ctrNum, isLE4);
      ({ s0, s1, s2, s3 } = encrypt(xk, c32[0], c32[1], c32[2], c32[3]));
    }
    const start = BLOCK_SIZE2 * Math.floor(src32.length / BLOCK_SIZE32);
    if (start < srcLen) {
      const b32 = new Uint32Array([s0, s1, s2, s3]);
      const buf = u8(b32);
      for (let i2 = start, pos = 0; i2 < srcLen; i2++, pos++)
        dst[i2] = src[i2] ^ buf[pos];
    }
    return dst;
  }
  var ctr = wrapCipher({ blockSize: 16, nonceLength: 16 }, function ctr2(key, nonce) {
    bytes3(key);
    bytes3(nonce, BLOCK_SIZE2);
    function processCtr(buf, dst) {
      const xk = expandKeyLE(key);
      const n = nonce.slice();
      const out = ctrCounter(xk, n, buf, dst);
      xk.fill(0);
      n.fill(0);
      return out;
    }
    return {
      encrypt: (plaintext, dst) => processCtr(plaintext, dst),
      decrypt: (ciphertext, dst) => processCtr(ciphertext, dst)
    };
  });
  function validateBlockDecrypt(data) {
    bytes3(data);
    if (data.length % BLOCK_SIZE2 !== 0) {
      throw new Error(`aes/(cbc-ecb).decrypt ciphertext should consist of blocks with size ${BLOCK_SIZE2}`);
    }
  }
  function validateBlockEncrypt(plaintext, pcks5, dst) {
    let outLen = plaintext.length;
    const remaining = outLen % BLOCK_SIZE2;
    if (!pcks5 && remaining !== 0)
      throw new Error("aec/(cbc-ecb): unpadded plaintext with disabled padding");
    const b = u32(plaintext);
    if (pcks5) {
      let left = BLOCK_SIZE2 - remaining;
      if (!left)
        left = BLOCK_SIZE2;
      outLen = outLen + left;
    }
    const out = getDst(outLen, dst);
    const o = u32(out);
    return { b, o, out };
  }
  function validatePCKS(data, pcks5) {
    if (!pcks5)
      return data;
    const len = data.length;
    if (!len)
      throw new Error(`aes/pcks5: empty ciphertext not allowed`);
    const lastByte = data[len - 1];
    if (lastByte <= 0 || lastByte > 16)
      throw new Error(`aes/pcks5: wrong padding byte: ${lastByte}`);
    const out = data.subarray(0, -lastByte);
    for (let i2 = 0; i2 < lastByte; i2++)
      if (data[len - i2 - 1] !== lastByte)
        throw new Error(`aes/pcks5: wrong padding`);
    return out;
  }
  function padPCKS(left) {
    const tmp = new Uint8Array(16);
    const tmp32 = u32(tmp);
    tmp.set(left);
    const paddingByte = BLOCK_SIZE2 - left.length;
    for (let i2 = BLOCK_SIZE2 - paddingByte; i2 < BLOCK_SIZE2; i2++)
      tmp[i2] = paddingByte;
    return tmp32;
  }
  var ecb = wrapCipher({ blockSize: 16 }, function ecb2(key, opts = {}) {
    bytes3(key);
    const pcks5 = !opts.disablePadding;
    return {
      encrypt: (plaintext, dst) => {
        bytes3(plaintext);
        const { b, o, out: _out } = validateBlockEncrypt(plaintext, pcks5, dst);
        const xk = expandKeyLE(key);
        let i2 = 0;
        for (; i2 + 4 <= b.length; ) {
          const { s0, s1, s2, s3 } = encrypt(xk, b[i2 + 0], b[i2 + 1], b[i2 + 2], b[i2 + 3]);
          o[i2++] = s0, o[i2++] = s1, o[i2++] = s2, o[i2++] = s3;
        }
        if (pcks5) {
          const tmp32 = padPCKS(plaintext.subarray(i2 * 4));
          const { s0, s1, s2, s3 } = encrypt(xk, tmp32[0], tmp32[1], tmp32[2], tmp32[3]);
          o[i2++] = s0, o[i2++] = s1, o[i2++] = s2, o[i2++] = s3;
        }
        xk.fill(0);
        return _out;
      },
      decrypt: (ciphertext, dst) => {
        validateBlockDecrypt(ciphertext);
        const xk = expandKeyDecLE(key);
        const out = getDst(ciphertext.length, dst);
        const b = u32(ciphertext);
        const o = u32(out);
        for (let i2 = 0; i2 + 4 <= b.length; ) {
          const { s0, s1, s2, s3 } = decrypt(xk, b[i2 + 0], b[i2 + 1], b[i2 + 2], b[i2 + 3]);
          o[i2++] = s0, o[i2++] = s1, o[i2++] = s2, o[i2++] = s3;
        }
        xk.fill(0);
        return validatePCKS(out, pcks5);
      }
    };
  });
  var cbc = wrapCipher({ blockSize: 16, nonceLength: 16 }, function cbc2(key, iv, opts = {}) {
    bytes3(key);
    bytes3(iv, 16);
    const pcks5 = !opts.disablePadding;
    return {
      encrypt: (plaintext, dst) => {
        const xk = expandKeyLE(key);
        const { b, o, out: _out } = validateBlockEncrypt(plaintext, pcks5, dst);
        const n32 = u32(iv);
        let s0 = n32[0], s1 = n32[1], s2 = n32[2], s3 = n32[3];
        let i2 = 0;
        for (; i2 + 4 <= b.length; ) {
          s0 ^= b[i2 + 0], s1 ^= b[i2 + 1], s2 ^= b[i2 + 2], s3 ^= b[i2 + 3];
          ({ s0, s1, s2, s3 } = encrypt(xk, s0, s1, s2, s3));
          o[i2++] = s0, o[i2++] = s1, o[i2++] = s2, o[i2++] = s3;
        }
        if (pcks5) {
          const tmp32 = padPCKS(plaintext.subarray(i2 * 4));
          s0 ^= tmp32[0], s1 ^= tmp32[1], s2 ^= tmp32[2], s3 ^= tmp32[3];
          ({ s0, s1, s2, s3 } = encrypt(xk, s0, s1, s2, s3));
          o[i2++] = s0, o[i2++] = s1, o[i2++] = s2, o[i2++] = s3;
        }
        xk.fill(0);
        return _out;
      },
      decrypt: (ciphertext, dst) => {
        validateBlockDecrypt(ciphertext);
        const xk = expandKeyDecLE(key);
        const n32 = u32(iv);
        const out = getDst(ciphertext.length, dst);
        const b = u32(ciphertext);
        const o = u32(out);
        let s0 = n32[0], s1 = n32[1], s2 = n32[2], s3 = n32[3];
        for (let i2 = 0; i2 + 4 <= b.length; ) {
          const ps0 = s0, ps1 = s1, ps2 = s2, ps3 = s3;
          s0 = b[i2 + 0], s1 = b[i2 + 1], s2 = b[i2 + 2], s3 = b[i2 + 3];
          const { s0: o0, s1: o1, s2: o2, s3: o3 } = decrypt(xk, s0, s1, s2, s3);
          o[i2++] = o0 ^ ps0, o[i2++] = o1 ^ ps1, o[i2++] = o2 ^ ps2, o[i2++] = o3 ^ ps3;
        }
        xk.fill(0);
        return validatePCKS(out, pcks5);
      }
    };
  });
  var cfb = wrapCipher({ blockSize: 16, nonceLength: 16 }, function cfb2(key, iv) {
    bytes3(key);
    bytes3(iv, 16);
    function processCfb(src, isEncrypt, dst) {
      const xk = expandKeyLE(key);
      const srcLen = src.length;
      dst = getDst(srcLen, dst);
      const src32 = u32(src);
      const dst32 = u32(dst);
      const next32 = isEncrypt ? dst32 : src32;
      const n32 = u32(iv);
      let s0 = n32[0], s1 = n32[1], s2 = n32[2], s3 = n32[3];
      for (let i2 = 0; i2 + 4 <= src32.length; ) {
        const { s0: e0, s1: e1, s2: e2, s3: e3 } = encrypt(xk, s0, s1, s2, s3);
        dst32[i2 + 0] = src32[i2 + 0] ^ e0;
        dst32[i2 + 1] = src32[i2 + 1] ^ e1;
        dst32[i2 + 2] = src32[i2 + 2] ^ e2;
        dst32[i2 + 3] = src32[i2 + 3] ^ e3;
        s0 = next32[i2++], s1 = next32[i2++], s2 = next32[i2++], s3 = next32[i2++];
      }
      const start = BLOCK_SIZE2 * Math.floor(src32.length / BLOCK_SIZE32);
      if (start < srcLen) {
        ({ s0, s1, s2, s3 } = encrypt(xk, s0, s1, s2, s3));
        const buf = u8(new Uint32Array([s0, s1, s2, s3]));
        for (let i2 = start, pos = 0; i2 < srcLen; i2++, pos++)
          dst[i2] = src[i2] ^ buf[pos];
        buf.fill(0);
      }
      xk.fill(0);
      return dst;
    }
    return {
      encrypt: (plaintext, dst) => processCfb(plaintext, true, dst),
      decrypt: (ciphertext, dst) => processCfb(ciphertext, false, dst)
    };
  });
  function computeTag(fn, isLE4, key, data, AAD) {
    const h = fn.create(key, data.length + (AAD?.length || 0));
    if (AAD)
      h.update(AAD);
    h.update(data);
    const num = new Uint8Array(16);
    const view = createView3(num);
    if (AAD)
      setBigUint643(view, 0, BigInt(AAD.length * 8), isLE4);
    setBigUint643(view, 8, BigInt(data.length * 8), isLE4);
    h.update(num);
    return h.digest();
  }
  var gcm = wrapCipher({ blockSize: 16, nonceLength: 12, tagLength: 16 }, function gcm2(key, nonce, AAD) {
    bytes3(nonce);
    if (nonce.length === 0)
      throw new Error("aes/gcm: empty nonce");
    const tagLength = 16;
    function _computeTag(authKey, tagMask, data) {
      const tag = computeTag(ghash, false, authKey, data, AAD);
      for (let i2 = 0; i2 < tagMask.length; i2++)
        tag[i2] ^= tagMask[i2];
      return tag;
    }
    function deriveKeys() {
      const xk = expandKeyLE(key);
      const authKey = EMPTY_BLOCK.slice();
      const counter = EMPTY_BLOCK.slice();
      ctr32(xk, false, counter, counter, authKey);
      if (nonce.length === 12) {
        counter.set(nonce);
      } else {
        const nonceLen = EMPTY_BLOCK.slice();
        const view = createView3(nonceLen);
        setBigUint643(view, 8, BigInt(nonce.length * 8), false);
        ghash.create(authKey).update(nonce).update(nonceLen).digestInto(counter);
      }
      const tagMask = ctr32(xk, false, counter, EMPTY_BLOCK);
      return { xk, authKey, counter, tagMask };
    }
    return {
      encrypt: (plaintext) => {
        bytes3(plaintext);
        const { xk, authKey, counter, tagMask } = deriveKeys();
        const out = new Uint8Array(plaintext.length + tagLength);
        ctr32(xk, false, counter, plaintext, out);
        const tag = _computeTag(authKey, tagMask, out.subarray(0, out.length - tagLength));
        out.set(tag, plaintext.length);
        xk.fill(0);
        return out;
      },
      decrypt: (ciphertext) => {
        bytes3(ciphertext);
        if (ciphertext.length < tagLength)
          throw new Error(`aes/gcm: ciphertext less than tagLen (${tagLength})`);
        const { xk, authKey, counter, tagMask } = deriveKeys();
        const data = ciphertext.subarray(0, -tagLength);
        const passedTag = ciphertext.subarray(-tagLength);
        const tag = _computeTag(authKey, tagMask, data);
        if (!equalBytes2(tag, passedTag))
          throw new Error("aes/gcm: invalid ghash tag");
        const out = ctr32(xk, false, counter, data);
        authKey.fill(0);
        tagMask.fill(0);
        xk.fill(0);
        return out;
      }
    };
  });
  var limit = (name, min, max) => (value) => {
    if (!Number.isSafeInteger(value) || min > value || value > max)
      throw new Error(`${name}: invalid value=${value}, must be [${min}..${max}]`);
  };
  var siv = wrapCipher({ blockSize: 16, nonceLength: 12, tagLength: 16 }, function siv2(key, nonce, AAD) {
    const tagLength = 16;
    const AAD_LIMIT = limit("AAD", 0, 2 ** 36);
    const PLAIN_LIMIT = limit("plaintext", 0, 2 ** 36);
    const NONCE_LIMIT = limit("nonce", 12, 12);
    const CIPHER_LIMIT = limit("ciphertext", 16, 2 ** 36 + 16);
    bytes3(nonce);
    NONCE_LIMIT(nonce.length);
    if (AAD) {
      bytes3(AAD);
      AAD_LIMIT(AAD.length);
    }
    function deriveKeys() {
      const len = key.length;
      if (len !== 16 && len !== 24 && len !== 32)
        throw new Error(`key length must be 16, 24 or 32 bytes, got: ${len} bytes`);
      const xk = expandKeyLE(key);
      const encKey = new Uint8Array(len);
      const authKey = new Uint8Array(16);
      const n32 = u32(nonce);
      let s0 = 0, s1 = n32[0], s2 = n32[1], s3 = n32[2];
      let counter = 0;
      for (const derivedKey of [authKey, encKey].map(u32)) {
        const d32 = u32(derivedKey);
        for (let i2 = 0; i2 < d32.length; i2 += 2) {
          const { s0: o0, s1: o1 } = encrypt(xk, s0, s1, s2, s3);
          d32[i2 + 0] = o0;
          d32[i2 + 1] = o1;
          s0 = ++counter;
        }
      }
      xk.fill(0);
      return { authKey, encKey: expandKeyLE(encKey) };
    }
    function _computeTag(encKey, authKey, data) {
      const tag = computeTag(polyval, true, authKey, data, AAD);
      for (let i2 = 0; i2 < 12; i2++)
        tag[i2] ^= nonce[i2];
      tag[15] &= 127;
      const t32 = u32(tag);
      let s0 = t32[0], s1 = t32[1], s2 = t32[2], s3 = t32[3];
      ({ s0, s1, s2, s3 } = encrypt(encKey, s0, s1, s2, s3));
      t32[0] = s0, t32[1] = s1, t32[2] = s2, t32[3] = s3;
      return tag;
    }
    function processSiv(encKey, tag, input) {
      let block = tag.slice();
      block[15] |= 128;
      return ctr32(encKey, true, block, input);
    }
    return {
      encrypt: (plaintext) => {
        bytes3(plaintext);
        PLAIN_LIMIT(plaintext.length);
        const { encKey, authKey } = deriveKeys();
        const tag = _computeTag(encKey, authKey, plaintext);
        const out = new Uint8Array(plaintext.length + tagLength);
        out.set(tag, plaintext.length);
        out.set(processSiv(encKey, tag, plaintext));
        encKey.fill(0);
        authKey.fill(0);
        return out;
      },
      decrypt: (ciphertext) => {
        bytes3(ciphertext);
        CIPHER_LIMIT(ciphertext.length);
        const tag = ciphertext.subarray(-tagLength);
        const { encKey, authKey } = deriveKeys();
        const plaintext = processSiv(encKey, tag, ciphertext.subarray(0, -tagLength));
        const expectedTag = _computeTag(encKey, authKey, plaintext);
        encKey.fill(0);
        authKey.fill(0);
        if (!equalBytes2(tag, expectedTag))
          throw new Error("invalid polyval tag");
        return plaintext;
      }
    };
  });

  // node_modules/@noble/ciphers/esm/_poly1305.js
  var u8to16 = (a, i2) => a[i2++] & 255 | (a[i2++] & 255) << 8;
  var Poly1305 = class {
    constructor(key) {
      this.blockLen = 16;
      this.outputLen = 16;
      this.buffer = new Uint8Array(16);
      this.r = new Uint16Array(10);
      this.h = new Uint16Array(10);
      this.pad = new Uint16Array(8);
      this.pos = 0;
      this.finished = false;
      key = toBytes3(key);
      bytes3(key, 32);
      const t0 = u8to16(key, 0);
      const t1 = u8to16(key, 2);
      const t2 = u8to16(key, 4);
      const t3 = u8to16(key, 6);
      const t4 = u8to16(key, 8);
      const t5 = u8to16(key, 10);
      const t6 = u8to16(key, 12);
      const t7 = u8to16(key, 14);
      this.r[0] = t0 & 8191;
      this.r[1] = (t0 >>> 13 | t1 << 3) & 8191;
      this.r[2] = (t1 >>> 10 | t2 << 6) & 7939;
      this.r[3] = (t2 >>> 7 | t3 << 9) & 8191;
      this.r[4] = (t3 >>> 4 | t4 << 12) & 255;
      this.r[5] = t4 >>> 1 & 8190;
      this.r[6] = (t4 >>> 14 | t5 << 2) & 8191;
      this.r[7] = (t5 >>> 11 | t6 << 5) & 8065;
      this.r[8] = (t6 >>> 8 | t7 << 8) & 8191;
      this.r[9] = t7 >>> 5 & 127;
      for (let i2 = 0; i2 < 8; i2++)
        this.pad[i2] = u8to16(key, 16 + 2 * i2);
    }
    process(data, offset, isLast = false) {
      const hibit = isLast ? 0 : 1 << 11;
      const { h, r } = this;
      const r0 = r[0];
      const r1 = r[1];
      const r2 = r[2];
      const r3 = r[3];
      const r4 = r[4];
      const r5 = r[5];
      const r6 = r[6];
      const r7 = r[7];
      const r8 = r[8];
      const r9 = r[9];
      const t0 = u8to16(data, offset + 0);
      const t1 = u8to16(data, offset + 2);
      const t2 = u8to16(data, offset + 4);
      const t3 = u8to16(data, offset + 6);
      const t4 = u8to16(data, offset + 8);
      const t5 = u8to16(data, offset + 10);
      const t6 = u8to16(data, offset + 12);
      const t7 = u8to16(data, offset + 14);
      let h0 = h[0] + (t0 & 8191);
      let h1 = h[1] + ((t0 >>> 13 | t1 << 3) & 8191);
      let h2 = h[2] + ((t1 >>> 10 | t2 << 6) & 8191);
      let h3 = h[3] + ((t2 >>> 7 | t3 << 9) & 8191);
      let h4 = h[4] + ((t3 >>> 4 | t4 << 12) & 8191);
      let h5 = h[5] + (t4 >>> 1 & 8191);
      let h6 = h[6] + ((t4 >>> 14 | t5 << 2) & 8191);
      let h7 = h[7] + ((t5 >>> 11 | t6 << 5) & 8191);
      let h8 = h[8] + ((t6 >>> 8 | t7 << 8) & 8191);
      let h9 = h[9] + (t7 >>> 5 | hibit);
      let c = 0;
      let d0 = c + h0 * r0 + h1 * (5 * r9) + h2 * (5 * r8) + h3 * (5 * r7) + h4 * (5 * r6);
      c = d0 >>> 13;
      d0 &= 8191;
      d0 += h5 * (5 * r5) + h6 * (5 * r4) + h7 * (5 * r3) + h8 * (5 * r2) + h9 * (5 * r1);
      c += d0 >>> 13;
      d0 &= 8191;
      let d1 = c + h0 * r1 + h1 * r0 + h2 * (5 * r9) + h3 * (5 * r8) + h4 * (5 * r7);
      c = d1 >>> 13;
      d1 &= 8191;
      d1 += h5 * (5 * r6) + h6 * (5 * r5) + h7 * (5 * r4) + h8 * (5 * r3) + h9 * (5 * r2);
      c += d1 >>> 13;
      d1 &= 8191;
      let d2 = c + h0 * r2 + h1 * r1 + h2 * r0 + h3 * (5 * r9) + h4 * (5 * r8);
      c = d2 >>> 13;
      d2 &= 8191;
      d2 += h5 * (5 * r7) + h6 * (5 * r6) + h7 * (5 * r5) + h8 * (5 * r4) + h9 * (5 * r3);
      c += d2 >>> 13;
      d2 &= 8191;
      let d3 = c + h0 * r3 + h1 * r2 + h2 * r1 + h3 * r0 + h4 * (5 * r9);
      c = d3 >>> 13;
      d3 &= 8191;
      d3 += h5 * (5 * r8) + h6 * (5 * r7) + h7 * (5 * r6) + h8 * (5 * r5) + h9 * (5 * r4);
      c += d3 >>> 13;
      d3 &= 8191;
      let d4 = c + h0 * r4 + h1 * r3 + h2 * r2 + h3 * r1 + h4 * r0;
      c = d4 >>> 13;
      d4 &= 8191;
      d4 += h5 * (5 * r9) + h6 * (5 * r8) + h7 * (5 * r7) + h8 * (5 * r6) + h9 * (5 * r5);
      c += d4 >>> 13;
      d4 &= 8191;
      let d5 = c + h0 * r5 + h1 * r4 + h2 * r3 + h3 * r2 + h4 * r1;
      c = d5 >>> 13;
      d5 &= 8191;
      d5 += h5 * r0 + h6 * (5 * r9) + h7 * (5 * r8) + h8 * (5 * r7) + h9 * (5 * r6);
      c += d5 >>> 13;
      d5 &= 8191;
      let d6 = c + h0 * r6 + h1 * r5 + h2 * r4 + h3 * r3 + h4 * r2;
      c = d6 >>> 13;
      d6 &= 8191;
      d6 += h5 * r1 + h6 * r0 + h7 * (5 * r9) + h8 * (5 * r8) + h9 * (5 * r7);
      c += d6 >>> 13;
      d6 &= 8191;
      let d7 = c + h0 * r7 + h1 * r6 + h2 * r5 + h3 * r4 + h4 * r3;
      c = d7 >>> 13;
      d7 &= 8191;
      d7 += h5 * r2 + h6 * r1 + h7 * r0 + h8 * (5 * r9) + h9 * (5 * r8);
      c += d7 >>> 13;
      d7 &= 8191;
      let d8 = c + h0 * r8 + h1 * r7 + h2 * r6 + h3 * r5 + h4 * r4;
      c = d8 >>> 13;
      d8 &= 8191;
      d8 += h5 * r3 + h6 * r2 + h7 * r1 + h8 * r0 + h9 * (5 * r9);
      c += d8 >>> 13;
      d8 &= 8191;
      let d9 = c + h0 * r9 + h1 * r8 + h2 * r7 + h3 * r6 + h4 * r5;
      c = d9 >>> 13;
      d9 &= 8191;
      d9 += h5 * r4 + h6 * r3 + h7 * r2 + h8 * r1 + h9 * r0;
      c += d9 >>> 13;
      d9 &= 8191;
      c = (c << 2) + c | 0;
      c = c + d0 | 0;
      d0 = c & 8191;
      c = c >>> 13;
      d1 += c;
      h[0] = d0;
      h[1] = d1;
      h[2] = d2;
      h[3] = d3;
      h[4] = d4;
      h[5] = d5;
      h[6] = d6;
      h[7] = d7;
      h[8] = d8;
      h[9] = d9;
    }
    finalize() {
      const { h, pad: pad2 } = this;
      const g = new Uint16Array(10);
      let c = h[1] >>> 13;
      h[1] &= 8191;
      for (let i2 = 2; i2 < 10; i2++) {
        h[i2] += c;
        c = h[i2] >>> 13;
        h[i2] &= 8191;
      }
      h[0] += c * 5;
      c = h[0] >>> 13;
      h[0] &= 8191;
      h[1] += c;
      c = h[1] >>> 13;
      h[1] &= 8191;
      h[2] += c;
      g[0] = h[0] + 5;
      c = g[0] >>> 13;
      g[0] &= 8191;
      for (let i2 = 1; i2 < 10; i2++) {
        g[i2] = h[i2] + c;
        c = g[i2] >>> 13;
        g[i2] &= 8191;
      }
      g[9] -= 1 << 13;
      let mask = (c ^ 1) - 1;
      for (let i2 = 0; i2 < 10; i2++)
        g[i2] &= mask;
      mask = ~mask;
      for (let i2 = 0; i2 < 10; i2++)
        h[i2] = h[i2] & mask | g[i2];
      h[0] = (h[0] | h[1] << 13) & 65535;
      h[1] = (h[1] >>> 3 | h[2] << 10) & 65535;
      h[2] = (h[2] >>> 6 | h[3] << 7) & 65535;
      h[3] = (h[3] >>> 9 | h[4] << 4) & 65535;
      h[4] = (h[4] >>> 12 | h[5] << 1 | h[6] << 14) & 65535;
      h[5] = (h[6] >>> 2 | h[7] << 11) & 65535;
      h[6] = (h[7] >>> 5 | h[8] << 8) & 65535;
      h[7] = (h[8] >>> 8 | h[9] << 5) & 65535;
      let f = h[0] + pad2[0];
      h[0] = f & 65535;
      for (let i2 = 1; i2 < 8; i2++) {
        f = (h[i2] + pad2[i2] | 0) + (f >>> 16) | 0;
        h[i2] = f & 65535;
      }
    }
    update(data) {
      exists3(this);
      const { buffer, blockLen } = this;
      data = toBytes3(data);
      const len = data.length;
      for (let pos = 0; pos < len; ) {
        const take = Math.min(blockLen - this.pos, len - pos);
        if (take === blockLen) {
          for (; blockLen <= len - pos; pos += blockLen)
            this.process(data, pos);
          continue;
        }
        buffer.set(data.subarray(pos, pos + take), this.pos);
        this.pos += take;
        pos += take;
        if (this.pos === blockLen) {
          this.process(buffer, 0, false);
          this.pos = 0;
        }
      }
      return this;
    }
    destroy() {
      this.h.fill(0);
      this.r.fill(0);
      this.buffer.fill(0);
      this.pad.fill(0);
    }
    digestInto(out) {
      exists3(this);
      output3(out, this);
      this.finished = true;
      const { buffer, h } = this;
      let { pos } = this;
      if (pos) {
        buffer[pos++] = 1;
        for (; pos < 16; pos++)
          buffer[pos] = 0;
        this.process(buffer, 0, true);
      }
      this.finalize();
      let opos = 0;
      for (let i2 = 0; i2 < 8; i2++) {
        out[opos++] = h[i2] >>> 0;
        out[opos++] = h[i2] >>> 8;
      }
      return out;
    }
    digest() {
      const { buffer, outputLen } = this;
      this.digestInto(buffer);
      const res = buffer.slice(0, outputLen);
      this.destroy();
      return res;
    }
  };
  function wrapConstructorWithKey2(hashCons) {
    const hashC = (msg, key) => hashCons(key).update(toBytes3(msg)).digest();
    const tmp = hashCons(new Uint8Array(32));
    hashC.outputLen = tmp.outputLen;
    hashC.blockLen = tmp.blockLen;
    hashC.create = (key) => hashCons(key);
    return hashC;
  }
  var poly1305 = wrapConstructorWithKey2((key) => new Poly1305(key));

  // node_modules/@noble/ciphers/esm/_arx.js
  var _utf8ToBytes = (str) => Uint8Array.from(str.split("").map((c) => c.charCodeAt(0)));
  var sigma16 = _utf8ToBytes("expand 16-byte k");
  var sigma32 = _utf8ToBytes("expand 32-byte k");
  var sigma16_32 = u32(sigma16);
  var sigma32_32 = u32(sigma32);
  var sigma = sigma32_32.slice();
  function rotl(a, b) {
    return a << b | a >>> 32 - b;
  }
  function isAligned32(b) {
    return b.byteOffset % 4 === 0;
  }
  var BLOCK_LEN = 64;
  var BLOCK_LEN32 = 16;
  var MAX_COUNTER = 2 ** 32 - 1;
  var U32_EMPTY = new Uint32Array();
  function runCipher(core, sigma2, key, nonce, data, output4, counter, rounds) {
    const len = data.length;
    const block = new Uint8Array(BLOCK_LEN);
    const b32 = u32(block);
    const isAligned = isAligned32(data) && isAligned32(output4);
    const d32 = isAligned ? u32(data) : U32_EMPTY;
    const o32 = isAligned ? u32(output4) : U32_EMPTY;
    for (let pos = 0; pos < len; counter++) {
      core(sigma2, key, nonce, b32, counter, rounds);
      if (counter >= MAX_COUNTER)
        throw new Error("arx: counter overflow");
      const take = Math.min(BLOCK_LEN, len - pos);
      if (isAligned && take === BLOCK_LEN) {
        const pos32 = pos / 4;
        if (pos % 4 !== 0)
          throw new Error("arx: invalid block position");
        for (let j = 0, posj; j < BLOCK_LEN32; j++) {
          posj = pos32 + j;
          o32[posj] = d32[posj] ^ b32[j];
        }
        pos += BLOCK_LEN;
        continue;
      }
      for (let j = 0, posj; j < take; j++) {
        posj = pos + j;
        output4[posj] = data[posj] ^ block[j];
      }
      pos += take;
    }
  }
  function createCipher(core, opts) {
    const { allowShortKeys, extendNonceFn, counterLength, counterRight, rounds } = checkOpts({ allowShortKeys: false, counterLength: 8, counterRight: false, rounds: 20 }, opts);
    if (typeof core !== "function")
      throw new Error("core must be a function");
    number3(counterLength);
    number3(rounds);
    bool2(counterRight);
    bool2(allowShortKeys);
    return (key, nonce, data, output4, counter = 0) => {
      bytes3(key);
      bytes3(nonce);
      bytes3(data);
      const len = data.length;
      if (!output4)
        output4 = new Uint8Array(len);
      bytes3(output4);
      number3(counter);
      if (counter < 0 || counter >= MAX_COUNTER)
        throw new Error("arx: counter overflow");
      if (output4.length < len)
        throw new Error(`arx: output (${output4.length}) is shorter than data (${len})`);
      const toClean = [];
      let l = key.length, k, sigma2;
      if (l === 32) {
        k = key.slice();
        toClean.push(k);
        sigma2 = sigma32_32;
      } else if (l === 16 && allowShortKeys) {
        k = new Uint8Array(32);
        k.set(key);
        k.set(key, 16);
        sigma2 = sigma16_32;
        toClean.push(k);
      } else {
        throw new Error(`arx: invalid 32-byte key, got length=${l}`);
      }
      if (!isAligned32(nonce)) {
        nonce = nonce.slice();
        toClean.push(nonce);
      }
      const k32 = u32(k);
      if (extendNonceFn) {
        if (nonce.length !== 24)
          throw new Error(`arx: extended nonce must be 24 bytes`);
        extendNonceFn(sigma2, k32, u32(nonce.subarray(0, 16)), k32);
        nonce = nonce.subarray(16);
      }
      const nonceNcLen = 16 - counterLength;
      if (nonceNcLen !== nonce.length)
        throw new Error(`arx: nonce must be ${nonceNcLen} or 16 bytes`);
      if (nonceNcLen !== 12) {
        const nc = new Uint8Array(12);
        nc.set(nonce, counterRight ? 0 : 12 - nonce.length);
        nonce = nc;
        toClean.push(nonce);
      }
      const n32 = u32(nonce);
      runCipher(core, sigma2, k32, n32, data, output4, counter, rounds);
      while (toClean.length > 0)
        toClean.pop().fill(0);
      return output4;
    };
  }

  // node_modules/@noble/ciphers/esm/chacha.js
  function chachaCore(s, k, n, out, cnt, rounds = 20) {
    let y00 = s[0], y01 = s[1], y02 = s[2], y03 = s[3], y04 = k[0], y05 = k[1], y06 = k[2], y07 = k[3], y08 = k[4], y09 = k[5], y10 = k[6], y11 = k[7], y12 = cnt, y13 = n[0], y14 = n[1], y15 = n[2];
    let x00 = y00, x01 = y01, x02 = y02, x03 = y03, x04 = y04, x05 = y05, x06 = y06, x07 = y07, x08 = y08, x09 = y09, x10 = y10, x11 = y11, x12 = y12, x13 = y13, x14 = y14, x15 = y15;
    for (let r = 0; r < rounds; r += 2) {
      x00 = x00 + x04 | 0;
      x12 = rotl(x12 ^ x00, 16);
      x08 = x08 + x12 | 0;
      x04 = rotl(x04 ^ x08, 12);
      x00 = x00 + x04 | 0;
      x12 = rotl(x12 ^ x00, 8);
      x08 = x08 + x12 | 0;
      x04 = rotl(x04 ^ x08, 7);
      x01 = x01 + x05 | 0;
      x13 = rotl(x13 ^ x01, 16);
      x09 = x09 + x13 | 0;
      x05 = rotl(x05 ^ x09, 12);
      x01 = x01 + x05 | 0;
      x13 = rotl(x13 ^ x01, 8);
      x09 = x09 + x13 | 0;
      x05 = rotl(x05 ^ x09, 7);
      x02 = x02 + x06 | 0;
      x14 = rotl(x14 ^ x02, 16);
      x10 = x10 + x14 | 0;
      x06 = rotl(x06 ^ x10, 12);
      x02 = x02 + x06 | 0;
      x14 = rotl(x14 ^ x02, 8);
      x10 = x10 + x14 | 0;
      x06 = rotl(x06 ^ x10, 7);
      x03 = x03 + x07 | 0;
      x15 = rotl(x15 ^ x03, 16);
      x11 = x11 + x15 | 0;
      x07 = rotl(x07 ^ x11, 12);
      x03 = x03 + x07 | 0;
      x15 = rotl(x15 ^ x03, 8);
      x11 = x11 + x15 | 0;
      x07 = rotl(x07 ^ x11, 7);
      x00 = x00 + x05 | 0;
      x15 = rotl(x15 ^ x00, 16);
      x10 = x10 + x15 | 0;
      x05 = rotl(x05 ^ x10, 12);
      x00 = x00 + x05 | 0;
      x15 = rotl(x15 ^ x00, 8);
      x10 = x10 + x15 | 0;
      x05 = rotl(x05 ^ x10, 7);
      x01 = x01 + x06 | 0;
      x12 = rotl(x12 ^ x01, 16);
      x11 = x11 + x12 | 0;
      x06 = rotl(x06 ^ x11, 12);
      x01 = x01 + x06 | 0;
      x12 = rotl(x12 ^ x01, 8);
      x11 = x11 + x12 | 0;
      x06 = rotl(x06 ^ x11, 7);
      x02 = x02 + x07 | 0;
      x13 = rotl(x13 ^ x02, 16);
      x08 = x08 + x13 | 0;
      x07 = rotl(x07 ^ x08, 12);
      x02 = x02 + x07 | 0;
      x13 = rotl(x13 ^ x02, 8);
      x08 = x08 + x13 | 0;
      x07 = rotl(x07 ^ x08, 7);
      x03 = x03 + x04 | 0;
      x14 = rotl(x14 ^ x03, 16);
      x09 = x09 + x14 | 0;
      x04 = rotl(x04 ^ x09, 12);
      x03 = x03 + x04 | 0;
      x14 = rotl(x14 ^ x03, 8);
      x09 = x09 + x14 | 0;
      x04 = rotl(x04 ^ x09, 7);
    }
    let oi = 0;
    out[oi++] = y00 + x00 | 0;
    out[oi++] = y01 + x01 | 0;
    out[oi++] = y02 + x02 | 0;
    out[oi++] = y03 + x03 | 0;
    out[oi++] = y04 + x04 | 0;
    out[oi++] = y05 + x05 | 0;
    out[oi++] = y06 + x06 | 0;
    out[oi++] = y07 + x07 | 0;
    out[oi++] = y08 + x08 | 0;
    out[oi++] = y09 + x09 | 0;
    out[oi++] = y10 + x10 | 0;
    out[oi++] = y11 + x11 | 0;
    out[oi++] = y12 + x12 | 0;
    out[oi++] = y13 + x13 | 0;
    out[oi++] = y14 + x14 | 0;
    out[oi++] = y15 + x15 | 0;
  }
  function hchacha(s, k, i2, o32) {
    let x00 = s[0], x01 = s[1], x02 = s[2], x03 = s[3], x04 = k[0], x05 = k[1], x06 = k[2], x07 = k[3], x08 = k[4], x09 = k[5], x10 = k[6], x11 = k[7], x12 = i2[0], x13 = i2[1], x14 = i2[2], x15 = i2[3];
    for (let r = 0; r < 20; r += 2) {
      x00 = x00 + x04 | 0;
      x12 = rotl(x12 ^ x00, 16);
      x08 = x08 + x12 | 0;
      x04 = rotl(x04 ^ x08, 12);
      x00 = x00 + x04 | 0;
      x12 = rotl(x12 ^ x00, 8);
      x08 = x08 + x12 | 0;
      x04 = rotl(x04 ^ x08, 7);
      x01 = x01 + x05 | 0;
      x13 = rotl(x13 ^ x01, 16);
      x09 = x09 + x13 | 0;
      x05 = rotl(x05 ^ x09, 12);
      x01 = x01 + x05 | 0;
      x13 = rotl(x13 ^ x01, 8);
      x09 = x09 + x13 | 0;
      x05 = rotl(x05 ^ x09, 7);
      x02 = x02 + x06 | 0;
      x14 = rotl(x14 ^ x02, 16);
      x10 = x10 + x14 | 0;
      x06 = rotl(x06 ^ x10, 12);
      x02 = x02 + x06 | 0;
      x14 = rotl(x14 ^ x02, 8);
      x10 = x10 + x14 | 0;
      x06 = rotl(x06 ^ x10, 7);
      x03 = x03 + x07 | 0;
      x15 = rotl(x15 ^ x03, 16);
      x11 = x11 + x15 | 0;
      x07 = rotl(x07 ^ x11, 12);
      x03 = x03 + x07 | 0;
      x15 = rotl(x15 ^ x03, 8);
      x11 = x11 + x15 | 0;
      x07 = rotl(x07 ^ x11, 7);
      x00 = x00 + x05 | 0;
      x15 = rotl(x15 ^ x00, 16);
      x10 = x10 + x15 | 0;
      x05 = rotl(x05 ^ x10, 12);
      x00 = x00 + x05 | 0;
      x15 = rotl(x15 ^ x00, 8);
      x10 = x10 + x15 | 0;
      x05 = rotl(x05 ^ x10, 7);
      x01 = x01 + x06 | 0;
      x12 = rotl(x12 ^ x01, 16);
      x11 = x11 + x12 | 0;
      x06 = rotl(x06 ^ x11, 12);
      x01 = x01 + x06 | 0;
      x12 = rotl(x12 ^ x01, 8);
      x11 = x11 + x12 | 0;
      x06 = rotl(x06 ^ x11, 7);
      x02 = x02 + x07 | 0;
      x13 = rotl(x13 ^ x02, 16);
      x08 = x08 + x13 | 0;
      x07 = rotl(x07 ^ x08, 12);
      x02 = x02 + x07 | 0;
      x13 = rotl(x13 ^ x02, 8);
      x08 = x08 + x13 | 0;
      x07 = rotl(x07 ^ x08, 7);
      x03 = x03 + x04 | 0;
      x14 = rotl(x14 ^ x03, 16);
      x09 = x09 + x14 | 0;
      x04 = rotl(x04 ^ x09, 12);
      x03 = x03 + x04 | 0;
      x14 = rotl(x14 ^ x03, 8);
      x09 = x09 + x14 | 0;
      x04 = rotl(x04 ^ x09, 7);
    }
    let oi = 0;
    o32[oi++] = x00;
    o32[oi++] = x01;
    o32[oi++] = x02;
    o32[oi++] = x03;
    o32[oi++] = x12;
    o32[oi++] = x13;
    o32[oi++] = x14;
    o32[oi++] = x15;
  }
  var chacha20 = /* @__PURE__ */ createCipher(chachaCore, {
    counterRight: false,
    counterLength: 4,
    allowShortKeys: false
  });
  var xchacha20 = /* @__PURE__ */ createCipher(chachaCore, {
    counterRight: false,
    counterLength: 8,
    extendNonceFn: hchacha,
    allowShortKeys: false
  });
  var ZEROS162 = /* @__PURE__ */ new Uint8Array(16);
  var updatePadded = (h, msg) => {
    h.update(msg);
    const left = msg.length % 16;
    if (left)
      h.update(ZEROS162.subarray(left));
  };
  var ZEROS322 = /* @__PURE__ */ new Uint8Array(32);
  function computeTag2(fn, key, nonce, data, AAD) {
    const authKey = fn(key, nonce, ZEROS322);
    const h = poly1305.create(authKey);
    if (AAD)
      updatePadded(h, AAD);
    updatePadded(h, data);
    const num = new Uint8Array(16);
    const view = createView3(num);
    setBigUint643(view, 0, BigInt(AAD ? AAD.length : 0), true);
    setBigUint643(view, 8, BigInt(data.length), true);
    h.update(num);
    const res = h.digest();
    authKey.fill(0);
    return res;
  }
  var _poly1305_aead = (xorStream) => (key, nonce, AAD) => {
    const tagLength = 16;
    bytes3(key, 32);
    bytes3(nonce);
    return {
      encrypt: (plaintext, output4) => {
        const plength = plaintext.length;
        const clength = plength + tagLength;
        if (output4) {
          bytes3(output4, clength);
        } else {
          output4 = new Uint8Array(clength);
        }
        xorStream(key, nonce, plaintext, output4, 1);
        const tag = computeTag2(xorStream, key, nonce, output4.subarray(0, -tagLength), AAD);
        output4.set(tag, plength);
        return output4;
      },
      decrypt: (ciphertext, output4) => {
        const clength = ciphertext.length;
        const plength = clength - tagLength;
        if (clength < tagLength)
          throw new Error(`encrypted data must be at least ${tagLength} bytes`);
        if (output4) {
          bytes3(output4, plength);
        } else {
          output4 = new Uint8Array(plength);
        }
        const data = ciphertext.subarray(0, -tagLength);
        const passedTag = ciphertext.subarray(-tagLength);
        const tag = computeTag2(xorStream, key, nonce, data, AAD);
        if (!equalBytes2(passedTag, tag))
          throw new Error("invalid tag");
        xorStream(key, nonce, data, output4, 1);
        return output4;
      }
    };
  };
  var chacha20poly1305 = /* @__PURE__ */ wrapCipher({ blockSize: 64, nonceLength: 12, tagLength: 16 }, _poly1305_aead(chacha20));
  var xchacha20poly1305 = /* @__PURE__ */ wrapCipher({ blockSize: 64, nonceLength: 24, tagLength: 16 }, _poly1305_aead(xchacha20));

  // node_modules/@noble/hashes/esm/hmac.js
  var HMAC2 = class extends Hash2 {
    constructor(hash3, _key) {
      super();
      this.finished = false;
      this.destroyed = false;
      assert_default.hash(hash3);
      const key = toBytes2(_key);
      this.iHash = hash3.create();
      if (typeof this.iHash.update !== "function")
        throw new Error("Expected instance of class which extends utils.Hash");
      this.blockLen = this.iHash.blockLen;
      this.outputLen = this.iHash.outputLen;
      const blockLen = this.blockLen;
      const pad2 = new Uint8Array(blockLen);
      pad2.set(key.length > blockLen ? hash3.create().update(key).digest() : key);
      for (let i2 = 0; i2 < pad2.length; i2++)
        pad2[i2] ^= 54;
      this.iHash.update(pad2);
      this.oHash = hash3.create();
      for (let i2 = 0; i2 < pad2.length; i2++)
        pad2[i2] ^= 54 ^ 92;
      this.oHash.update(pad2);
      pad2.fill(0);
    }
    update(buf) {
      assert_default.exists(this);
      this.iHash.update(buf);
      return this;
    }
    digestInto(out) {
      assert_default.exists(this);
      assert_default.bytes(out, this.outputLen);
      this.finished = true;
      this.iHash.digestInto(out);
      this.oHash.update(out);
      this.oHash.digestInto(out);
      this.destroy();
    }
    digest() {
      const out = new Uint8Array(this.oHash.outputLen);
      this.digestInto(out);
      return out;
    }
    _cloneInto(to) {
      to || (to = Object.create(Object.getPrototypeOf(this), {}));
      const { oHash, iHash, finished, destroyed, blockLen, outputLen } = this;
      to = to;
      to.finished = finished;
      to.destroyed = destroyed;
      to.blockLen = blockLen;
      to.outputLen = outputLen;
      to.oHash = oHash._cloneInto(to.oHash);
      to.iHash = iHash._cloneInto(to.iHash);
      return to;
    }
    destroy() {
      this.destroyed = true;
      this.oHash.destroy();
      this.iHash.destroy();
    }
  };
  var hmac2 = (hash3, key, message) => new HMAC2(hash3, key).update(message).digest();
  hmac2.create = (hash3, key) => new HMAC2(hash3, key);

  // node_modules/@noble/hashes/esm/hkdf.js
  function extract(hash3, ikm, salt) {
    assert_default.hash(hash3);
    if (salt === void 0)
      salt = new Uint8Array(hash3.outputLen);
    return hmac2(hash3, toBytes2(salt), toBytes2(ikm));
  }
  var HKDF_COUNTER = new Uint8Array([0]);
  var EMPTY_BUFFER = new Uint8Array();
  function expand(hash3, prk, info, length = 32) {
    assert_default.hash(hash3);
    assert_default.number(length);
    if (length > 255 * hash3.outputLen)
      throw new Error("Length should be <= 255*HashLen");
    const blocks = Math.ceil(length / hash3.outputLen);
    if (info === void 0)
      info = EMPTY_BUFFER;
    const okm = new Uint8Array(blocks * hash3.outputLen);
    const HMAC3 = hmac2.create(hash3, prk);
    const HMACTmp = HMAC3._cloneInto();
    const T = new Uint8Array(HMAC3.outputLen);
    for (let counter = 0; counter < blocks; counter++) {
      HKDF_COUNTER[0] = counter + 1;
      HMACTmp.update(counter === 0 ? EMPTY_BUFFER : T).update(info).update(HKDF_COUNTER).digestInto(T);
      okm.set(T, hash3.outputLen * counter);
      HMAC3._cloneInto(HMACTmp);
    }
    HMAC3.destroy();
    HMACTmp.destroy();
    T.fill(0);
    HKDF_COUNTER.fill(0);
    return okm.slice(0, length);
  }

  // node_modules/nostr-tools/lib/esm/index.js
  var __defProp2 = Object.defineProperty;
  var __export2 = (target, all) => {
    for (var name in all)
      __defProp2(target, name, { get: all[name], enumerable: true });
  };
  var verifiedSymbol = Symbol("verified");
  var isRecord = (obj) => obj instanceof Object;
  function validateEvent(event) {
    if (!isRecord(event))
      return false;
    if (typeof event.kind !== "number")
      return false;
    if (typeof event.content !== "string")
      return false;
    if (typeof event.created_at !== "number")
      return false;
    if (typeof event.pubkey !== "string")
      return false;
    if (!event.pubkey.match(/^[a-f0-9]{64}$/))
      return false;
    if (!Array.isArray(event.tags))
      return false;
    for (let i2 = 0; i2 < event.tags.length; i2++) {
      let tag = event.tags[i2];
      if (!Array.isArray(tag))
        return false;
      for (let j = 0; j < tag.length; j++) {
        if (typeof tag[j] !== "string")
          return false;
      }
    }
    return true;
  }
  var utils_exports2 = {};
  __export2(utils_exports2, {
    Queue: () => Queue,
    QueueNode: () => QueueNode,
    binarySearch: () => binarySearch,
    bytesToHex: () => bytesToHex2,
    hexToBytes: () => hexToBytes2,
    insertEventIntoAscendingList: () => insertEventIntoAscendingList,
    insertEventIntoDescendingList: () => insertEventIntoDescendingList,
    normalizeURL: () => normalizeURL,
    utf8Decoder: () => utf8Decoder,
    utf8Encoder: () => utf8Encoder
  });
  var utf8Decoder = new TextDecoder("utf-8");
  var utf8Encoder = new TextEncoder();
  function normalizeURL(url) {
    try {
      if (url.indexOf("://") === -1)
        url = "wss://" + url;
      let p = new URL(url);
      p.pathname = p.pathname.replace(/\/+/g, "/");
      if (p.pathname.endsWith("/"))
        p.pathname = p.pathname.slice(0, -1);
      if (p.port === "80" && p.protocol === "ws:" || p.port === "443" && p.protocol === "wss:")
        p.port = "";
      p.searchParams.sort();
      p.hash = "";
      return p.toString();
    } catch (e) {
      throw new Error(`Invalid URL: ${url}`);
    }
  }
  function insertEventIntoDescendingList(sortedArray, event) {
    const [idx, found] = binarySearch(sortedArray, (b) => {
      if (event.id === b.id)
        return 0;
      if (event.created_at === b.created_at)
        return -1;
      return b.created_at - event.created_at;
    });
    if (!found) {
      sortedArray.splice(idx, 0, event);
    }
    return sortedArray;
  }
  function insertEventIntoAscendingList(sortedArray, event) {
    const [idx, found] = binarySearch(sortedArray, (b) => {
      if (event.id === b.id)
        return 0;
      if (event.created_at === b.created_at)
        return -1;
      return event.created_at - b.created_at;
    });
    if (!found) {
      sortedArray.splice(idx, 0, event);
    }
    return sortedArray;
  }
  function binarySearch(arr, compare) {
    let start = 0;
    let end = arr.length - 1;
    while (start <= end) {
      const mid = Math.floor((start + end) / 2);
      const cmp = compare(arr[mid]);
      if (cmp === 0) {
        return [mid, true];
      }
      if (cmp < 0) {
        end = mid - 1;
      } else {
        start = mid + 1;
      }
    }
    return [start, false];
  }
  var QueueNode = class {
    value;
    next = null;
    prev = null;
    constructor(message) {
      this.value = message;
    }
  };
  var Queue = class {
    first;
    last;
    constructor() {
      this.first = null;
      this.last = null;
    }
    enqueue(value) {
      const newNode = new QueueNode(value);
      if (!this.last) {
        this.first = newNode;
        this.last = newNode;
      } else if (this.last === this.first) {
        this.last = newNode;
        this.last.prev = this.first;
        this.first.next = newNode;
      } else {
        newNode.prev = this.last;
        this.last.next = newNode;
        this.last = newNode;
      }
      return true;
    }
    dequeue() {
      if (!this.first)
        return null;
      if (this.first === this.last) {
        const target2 = this.first;
        this.first = null;
        this.last = null;
        return target2.value;
      }
      const target = this.first;
      this.first = target.next;
      if (this.first) {
        this.first.prev = null;
      }
      return target.value;
    }
  };
  var JS = class {
    generateSecretKey() {
      return schnorr.utils.randomPrivateKey();
    }
    getPublicKey(secretKey) {
      return bytesToHex2(schnorr.getPublicKey(secretKey));
    }
    finalizeEvent(t, secretKey) {
      const event = t;
      event.pubkey = bytesToHex2(schnorr.getPublicKey(secretKey));
      event.id = getEventHash(event);
      event.sig = bytesToHex2(schnorr.sign(getEventHash(event), secretKey));
      event[verifiedSymbol] = true;
      return event;
    }
    verifyEvent(event) {
      if (typeof event[verifiedSymbol] === "boolean")
        return event[verifiedSymbol];
      const hash3 = getEventHash(event);
      if (hash3 !== event.id) {
        event[verifiedSymbol] = false;
        return false;
      }
      try {
        const valid = schnorr.verify(event.sig, hash3, event.pubkey);
        event[verifiedSymbol] = valid;
        return valid;
      } catch (err) {
        event[verifiedSymbol] = false;
        return false;
      }
    }
  };
  function serializeEvent(evt) {
    if (!validateEvent(evt))
      throw new Error("can't serialize event with wrong or missing properties");
    return JSON.stringify([0, evt.pubkey, evt.created_at, evt.kind, evt.tags, evt.content]);
  }
  function getEventHash(event) {
    let eventHash = sha2562(utf8Encoder.encode(serializeEvent(event)));
    return bytesToHex2(eventHash);
  }
  var i = new JS();
  var generateSecretKey = i.generateSecretKey;
  var getPublicKey = i.getPublicKey;
  var finalizeEvent = i.finalizeEvent;
  var verifyEvent = i.verifyEvent;
  var kinds_exports = {};
  __export2(kinds_exports, {
    Application: () => Application,
    BadgeAward: () => BadgeAward,
    BadgeDefinition: () => BadgeDefinition,
    BlockedRelaysList: () => BlockedRelaysList,
    BookmarkList: () => BookmarkList,
    Bookmarksets: () => Bookmarksets,
    Calendar: () => Calendar,
    CalendarEventRSVP: () => CalendarEventRSVP,
    ChannelCreation: () => ChannelCreation,
    ChannelHideMessage: () => ChannelHideMessage,
    ChannelMessage: () => ChannelMessage,
    ChannelMetadata: () => ChannelMetadata,
    ChannelMuteUser: () => ChannelMuteUser,
    ClassifiedListing: () => ClassifiedListing,
    ClientAuth: () => ClientAuth,
    CommunitiesList: () => CommunitiesList,
    CommunityDefinition: () => CommunityDefinition,
    CommunityPostApproval: () => CommunityPostApproval,
    Contacts: () => Contacts,
    CreateOrUpdateProduct: () => CreateOrUpdateProduct,
    CreateOrUpdateStall: () => CreateOrUpdateStall,
    Curationsets: () => Curationsets,
    Date: () => Date2,
    DirectMessageRelaysList: () => DirectMessageRelaysList,
    DraftClassifiedListing: () => DraftClassifiedListing,
    DraftLong: () => DraftLong,
    Emojisets: () => Emojisets,
    EncryptedDirectMessage: () => EncryptedDirectMessage,
    EventDeletion: () => EventDeletion,
    FileMetadata: () => FileMetadata,
    FileServerPreference: () => FileServerPreference,
    Followsets: () => Followsets,
    GenericRepost: () => GenericRepost,
    Genericlists: () => Genericlists,
    GiftWrap: () => GiftWrap,
    HTTPAuth: () => HTTPAuth,
    Handlerinformation: () => Handlerinformation,
    Handlerrecommendation: () => Handlerrecommendation,
    Highlights: () => Highlights,
    InterestsList: () => InterestsList,
    Interestsets: () => Interestsets,
    JobFeedback: () => JobFeedback,
    JobRequest: () => JobRequest,
    JobResult: () => JobResult,
    Label: () => Label,
    LightningPubRPC: () => LightningPubRPC,
    LiveChatMessage: () => LiveChatMessage,
    LiveEvent: () => LiveEvent,
    LongFormArticle: () => LongFormArticle,
    Metadata: () => Metadata,
    Mutelist: () => Mutelist,
    NWCWalletInfo: () => NWCWalletInfo,
    NWCWalletRequest: () => NWCWalletRequest,
    NWCWalletResponse: () => NWCWalletResponse,
    NostrConnect: () => NostrConnect,
    OpenTimestamps: () => OpenTimestamps,
    Pinlist: () => Pinlist,
    PrivateDirectMessage: () => PrivateDirectMessage,
    ProblemTracker: () => ProblemTracker,
    ProfileBadges: () => ProfileBadges,
    PublicChatsList: () => PublicChatsList,
    Reaction: () => Reaction,
    RecommendRelay: () => RecommendRelay,
    RelayList: () => RelayList,
    Relaysets: () => Relaysets,
    Report: () => Report,
    Reporting: () => Reporting,
    Repost: () => Repost,
    Seal: () => Seal,
    SearchRelaysList: () => SearchRelaysList,
    ShortTextNote: () => ShortTextNote,
    Time: () => Time,
    UserEmojiList: () => UserEmojiList,
    UserStatuses: () => UserStatuses,
    Zap: () => Zap,
    ZapGoal: () => ZapGoal,
    ZapRequest: () => ZapRequest,
    classifyKind: () => classifyKind,
    isAddressableKind: () => isAddressableKind,
    isEphemeralKind: () => isEphemeralKind,
    isKind: () => isKind,
    isParameterizedReplaceableKind: () => isParameterizedReplaceableKind,
    isRegularKind: () => isRegularKind,
    isReplaceableKind: () => isReplaceableKind
  });
  function isRegularKind(kind) {
    return 1e3 <= kind && kind < 1e4 || [1, 2, 4, 5, 6, 7, 8, 16, 40, 41, 42, 43, 44].includes(kind);
  }
  function isReplaceableKind(kind) {
    return [0, 3].includes(kind) || 1e4 <= kind && kind < 2e4;
  }
  function isEphemeralKind(kind) {
    return 2e4 <= kind && kind < 3e4;
  }
  function isAddressableKind(kind) {
    return 3e4 <= kind && kind < 4e4;
  }
  var isParameterizedReplaceableKind = isAddressableKind;
  function classifyKind(kind) {
    if (isRegularKind(kind))
      return "regular";
    if (isReplaceableKind(kind))
      return "replaceable";
    if (isEphemeralKind(kind))
      return "ephemeral";
    if (isAddressableKind(kind))
      return "parameterized";
    return "unknown";
  }
  function isKind(event, kind) {
    const kindAsArray = kind instanceof Array ? kind : [kind];
    return validateEvent(event) && kindAsArray.includes(event.kind) || false;
  }
  var Metadata = 0;
  var ShortTextNote = 1;
  var RecommendRelay = 2;
  var Contacts = 3;
  var EncryptedDirectMessage = 4;
  var EventDeletion = 5;
  var Repost = 6;
  var Reaction = 7;
  var BadgeAward = 8;
  var Seal = 13;
  var PrivateDirectMessage = 14;
  var GenericRepost = 16;
  var ChannelCreation = 40;
  var ChannelMetadata = 41;
  var ChannelMessage = 42;
  var ChannelHideMessage = 43;
  var ChannelMuteUser = 44;
  var OpenTimestamps = 1040;
  var GiftWrap = 1059;
  var FileMetadata = 1063;
  var LiveChatMessage = 1311;
  var ProblemTracker = 1971;
  var Report = 1984;
  var Reporting = 1984;
  var Label = 1985;
  var CommunityPostApproval = 4550;
  var JobRequest = 5999;
  var JobResult = 6999;
  var JobFeedback = 7e3;
  var ZapGoal = 9041;
  var ZapRequest = 9734;
  var Zap = 9735;
  var Highlights = 9802;
  var Mutelist = 1e4;
  var Pinlist = 10001;
  var RelayList = 10002;
  var BookmarkList = 10003;
  var CommunitiesList = 10004;
  var PublicChatsList = 10005;
  var BlockedRelaysList = 10006;
  var SearchRelaysList = 10007;
  var InterestsList = 10015;
  var UserEmojiList = 10030;
  var DirectMessageRelaysList = 10050;
  var FileServerPreference = 10096;
  var NWCWalletInfo = 13194;
  var LightningPubRPC = 21e3;
  var ClientAuth = 22242;
  var NWCWalletRequest = 23194;
  var NWCWalletResponse = 23195;
  var NostrConnect = 24133;
  var HTTPAuth = 27235;
  var Followsets = 3e4;
  var Genericlists = 30001;
  var Relaysets = 30002;
  var Bookmarksets = 30003;
  var Curationsets = 30004;
  var ProfileBadges = 30008;
  var BadgeDefinition = 30009;
  var Interestsets = 30015;
  var CreateOrUpdateStall = 30017;
  var CreateOrUpdateProduct = 30018;
  var LongFormArticle = 30023;
  var DraftLong = 30024;
  var Emojisets = 30030;
  var Application = 30078;
  var LiveEvent = 30311;
  var UserStatuses = 30315;
  var ClassifiedListing = 30402;
  var DraftClassifiedListing = 30403;
  var Date2 = 31922;
  var Time = 31923;
  var Calendar = 31924;
  var CalendarEventRSVP = 31925;
  var Handlerrecommendation = 31989;
  var Handlerinformation = 31990;
  var CommunityDefinition = 34550;
  var fakejson_exports = {};
  __export2(fakejson_exports, {
    getHex64: () => getHex64,
    getInt: () => getInt,
    getSubscriptionId: () => getSubscriptionId,
    matchEventId: () => matchEventId,
    matchEventKind: () => matchEventKind,
    matchEventPubkey: () => matchEventPubkey
  });
  function getHex64(json, field) {
    let len = field.length + 3;
    let idx = json.indexOf(`"${field}":`) + len;
    let s = json.slice(idx).indexOf(`"`) + idx + 1;
    return json.slice(s, s + 64);
  }
  function getInt(json, field) {
    let len = field.length;
    let idx = json.indexOf(`"${field}":`) + len + 3;
    let sliced = json.slice(idx);
    let end = Math.min(sliced.indexOf(","), sliced.indexOf("}"));
    return parseInt(sliced.slice(0, end), 10);
  }
  function getSubscriptionId(json) {
    let idx = json.slice(0, 22).indexOf(`"EVENT"`);
    if (idx === -1)
      return null;
    let pstart = json.slice(idx + 7 + 1).indexOf(`"`);
    if (pstart === -1)
      return null;
    let start = idx + 7 + 1 + pstart;
    let pend = json.slice(start + 1, 80).indexOf(`"`);
    if (pend === -1)
      return null;
    let end = start + 1 + pend;
    return json.slice(start + 1, end);
  }
  function matchEventId(json, id) {
    return id === getHex64(json, "id");
  }
  function matchEventPubkey(json, pubkey) {
    return pubkey === getHex64(json, "pubkey");
  }
  function matchEventKind(json, kind) {
    return kind === getInt(json, "kind");
  }
  var nip42_exports = {};
  __export2(nip42_exports, {
    makeAuthEvent: () => makeAuthEvent
  });
  function makeAuthEvent(relayURL, challenge2) {
    return {
      kind: ClientAuth,
      created_at: Math.floor(Date.now() / 1e3),
      tags: [
        ["relay", relayURL],
        ["challenge", challenge2]
      ],
      content: ""
    };
  }
  var _WebSocket;
  try {
    _WebSocket = WebSocket;
  } catch {
  }
  var _WebSocket2;
  try {
    _WebSocket2 = WebSocket;
  } catch {
  }
  var nip19_exports = {};
  __export2(nip19_exports, {
    BECH32_REGEX: () => BECH32_REGEX,
    Bech32MaxSize: () => Bech32MaxSize,
    NostrTypeGuard: () => NostrTypeGuard,
    decode: () => decode,
    decodeNostrURI: () => decodeNostrURI,
    encodeBytes: () => encodeBytes,
    naddrEncode: () => naddrEncode,
    neventEncode: () => neventEncode,
    noteEncode: () => noteEncode,
    nprofileEncode: () => nprofileEncode,
    npubEncode: () => npubEncode,
    nsecEncode: () => nsecEncode
  });
  var NostrTypeGuard = {
    isNProfile: (value) => /^nprofile1[a-z\d]+$/.test(value || ""),
    isNEvent: (value) => /^nevent1[a-z\d]+$/.test(value || ""),
    isNAddr: (value) => /^naddr1[a-z\d]+$/.test(value || ""),
    isNSec: (value) => /^nsec1[a-z\d]{58}$/.test(value || ""),
    isNPub: (value) => /^npub1[a-z\d]{58}$/.test(value || ""),
    isNote: (value) => /^note1[a-z\d]+$/.test(value || ""),
    isNcryptsec: (value) => /^ncryptsec1[a-z\d]+$/.test(value || "")
  };
  var Bech32MaxSize = 5e3;
  var BECH32_REGEX = /[\x21-\x7E]{1,83}1[023456789acdefghjklmnpqrstuvwxyz]{6,}/;
  function integerToUint8Array(number4) {
    const uint8Array = new Uint8Array(4);
    uint8Array[0] = number4 >> 24 & 255;
    uint8Array[1] = number4 >> 16 & 255;
    uint8Array[2] = number4 >> 8 & 255;
    uint8Array[3] = number4 & 255;
    return uint8Array;
  }
  function decodeNostrURI(nip19code) {
    try {
      if (nip19code.startsWith("nostr:"))
        nip19code = nip19code.substring(6);
      return decode(nip19code);
    } catch (_err) {
      return { type: "invalid", data: null };
    }
  }
  function decode(code) {
    let { prefix, words } = bech32.decode(code, Bech32MaxSize);
    let data = new Uint8Array(bech32.fromWords(words));
    switch (prefix) {
      case "nprofile": {
        let tlv = parseTLV(data);
        if (!tlv[0]?.[0])
          throw new Error("missing TLV 0 for nprofile");
        if (tlv[0][0].length !== 32)
          throw new Error("TLV 0 should be 32 bytes");
        return {
          type: "nprofile",
          data: {
            pubkey: bytesToHex2(tlv[0][0]),
            relays: tlv[1] ? tlv[1].map((d) => utf8Decoder.decode(d)) : []
          }
        };
      }
      case "nevent": {
        let tlv = parseTLV(data);
        if (!tlv[0]?.[0])
          throw new Error("missing TLV 0 for nevent");
        if (tlv[0][0].length !== 32)
          throw new Error("TLV 0 should be 32 bytes");
        if (tlv[2] && tlv[2][0].length !== 32)
          throw new Error("TLV 2 should be 32 bytes");
        if (tlv[3] && tlv[3][0].length !== 4)
          throw new Error("TLV 3 should be 4 bytes");
        return {
          type: "nevent",
          data: {
            id: bytesToHex2(tlv[0][0]),
            relays: tlv[1] ? tlv[1].map((d) => utf8Decoder.decode(d)) : [],
            author: tlv[2]?.[0] ? bytesToHex2(tlv[2][0]) : void 0,
            kind: tlv[3]?.[0] ? parseInt(bytesToHex2(tlv[3][0]), 16) : void 0
          }
        };
      }
      case "naddr": {
        let tlv = parseTLV(data);
        if (!tlv[0]?.[0])
          throw new Error("missing TLV 0 for naddr");
        if (!tlv[2]?.[0])
          throw new Error("missing TLV 2 for naddr");
        if (tlv[2][0].length !== 32)
          throw new Error("TLV 2 should be 32 bytes");
        if (!tlv[3]?.[0])
          throw new Error("missing TLV 3 for naddr");
        if (tlv[3][0].length !== 4)
          throw new Error("TLV 3 should be 4 bytes");
        return {
          type: "naddr",
          data: {
            identifier: utf8Decoder.decode(tlv[0][0]),
            pubkey: bytesToHex2(tlv[2][0]),
            kind: parseInt(bytesToHex2(tlv[3][0]), 16),
            relays: tlv[1] ? tlv[1].map((d) => utf8Decoder.decode(d)) : []
          }
        };
      }
      case "nsec":
        return { type: prefix, data };
      case "npub":
      case "note":
        return { type: prefix, data: bytesToHex2(data) };
      default:
        throw new Error(`unknown prefix ${prefix}`);
    }
  }
  function parseTLV(data) {
    let result = {};
    let rest = data;
    while (rest.length > 0) {
      let t = rest[0];
      let l = rest[1];
      let v = rest.slice(2, 2 + l);
      rest = rest.slice(2 + l);
      if (v.length < l)
        throw new Error(`not enough data to read on TLV ${t}`);
      result[t] = result[t] || [];
      result[t].push(v);
    }
    return result;
  }
  function nsecEncode(key) {
    return encodeBytes("nsec", key);
  }
  function npubEncode(hex2) {
    return encodeBytes("npub", hexToBytes2(hex2));
  }
  function noteEncode(hex2) {
    return encodeBytes("note", hexToBytes2(hex2));
  }
  function encodeBech32(prefix, data) {
    let words = bech32.toWords(data);
    return bech32.encode(prefix, words, Bech32MaxSize);
  }
  function encodeBytes(prefix, bytes4) {
    return encodeBech32(prefix, bytes4);
  }
  function nprofileEncode(profile) {
    let data = encodeTLV({
      0: [hexToBytes2(profile.pubkey)],
      1: (profile.relays || []).map((url) => utf8Encoder.encode(url))
    });
    return encodeBech32("nprofile", data);
  }
  function neventEncode(event) {
    let kindArray;
    if (event.kind !== void 0) {
      kindArray = integerToUint8Array(event.kind);
    }
    let data = encodeTLV({
      0: [hexToBytes2(event.id)],
      1: (event.relays || []).map((url) => utf8Encoder.encode(url)),
      2: event.author ? [hexToBytes2(event.author)] : [],
      3: kindArray ? [new Uint8Array(kindArray)] : []
    });
    return encodeBech32("nevent", data);
  }
  function naddrEncode(addr) {
    let kind = new ArrayBuffer(4);
    new DataView(kind).setUint32(0, addr.kind, false);
    let data = encodeTLV({
      0: [utf8Encoder.encode(addr.identifier)],
      1: (addr.relays || []).map((url) => utf8Encoder.encode(url)),
      2: [hexToBytes2(addr.pubkey)],
      3: [new Uint8Array(kind)]
    });
    return encodeBech32("naddr", data);
  }
  function encodeTLV(tlv) {
    let entries = [];
    Object.entries(tlv).reverse().forEach(([t, vs]) => {
      vs.forEach((v) => {
        let entry = new Uint8Array(v.length + 2);
        entry.set([parseInt(t)], 0);
        entry.set([v.length], 1);
        entry.set(v, 2);
        entries.push(entry);
      });
    });
    return concatBytes3(...entries);
  }
  var nip04_exports = {};
  __export2(nip04_exports, {
    decrypt: () => decrypt2,
    encrypt: () => encrypt2
  });
  function encrypt2(secretKey, pubkey, text) {
    const privkey = secretKey instanceof Uint8Array ? bytesToHex2(secretKey) : secretKey;
    const key = secp256k1.getSharedSecret(privkey, "02" + pubkey);
    const normalizedKey = getNormalizedX(key);
    let iv = Uint8Array.from(randomBytes2(16));
    let plaintext = utf8Encoder.encode(text);
    let ciphertext = cbc(normalizedKey, iv).encrypt(plaintext);
    let ctb64 = base64.encode(new Uint8Array(ciphertext));
    let ivb64 = base64.encode(new Uint8Array(iv.buffer));
    return `${ctb64}?iv=${ivb64}`;
  }
  function decrypt2(secretKey, pubkey, data) {
    const privkey = secretKey instanceof Uint8Array ? bytesToHex2(secretKey) : secretKey;
    let [ctb64, ivb64] = data.split("?iv=");
    let key = secp256k1.getSharedSecret(privkey, "02" + pubkey);
    let normalizedKey = getNormalizedX(key);
    let iv = base64.decode(ivb64);
    let ciphertext = base64.decode(ctb64);
    let plaintext = cbc(normalizedKey, iv).decrypt(ciphertext);
    return utf8Decoder.decode(plaintext);
  }
  function getNormalizedX(key) {
    return key.slice(1, 33);
  }
  var nip05_exports = {};
  __export2(nip05_exports, {
    NIP05_REGEX: () => NIP05_REGEX,
    isNip05: () => isNip05,
    isValid: () => isValid,
    queryProfile: () => queryProfile,
    searchDomain: () => searchDomain,
    useFetchImplementation: () => useFetchImplementation
  });
  var NIP05_REGEX = /^(?:([\w.+-]+)@)?([\w_-]+(\.[\w_-]+)+)$/;
  var isNip05 = (value) => NIP05_REGEX.test(value || "");
  var _fetch;
  try {
    _fetch = fetch;
  } catch (_) {
    null;
  }
  function useFetchImplementation(fetchImplementation) {
    _fetch = fetchImplementation;
  }
  async function searchDomain(domain, query = "") {
    try {
      const url = `https://${domain}/.well-known/nostr.json?name=${query}`;
      const res = await _fetch(url, { redirect: "manual" });
      if (res.status !== 200) {
        throw Error("Wrong response code");
      }
      const json = await res.json();
      return json.names;
    } catch (_) {
      return {};
    }
  }
  async function queryProfile(fullname) {
    const match = fullname.match(NIP05_REGEX);
    if (!match)
      return null;
    const [, name = "_", domain] = match;
    try {
      const url = `https://${domain}/.well-known/nostr.json?name=${name}`;
      const res = await _fetch(url, { redirect: "manual" });
      if (res.status !== 200) {
        throw Error("Wrong response code");
      }
      const json = await res.json();
      const pubkey = json.names[name];
      return pubkey ? { pubkey, relays: json.relays?.[pubkey] } : null;
    } catch (_e) {
      return null;
    }
  }
  async function isValid(pubkey, nip05) {
    const res = await queryProfile(nip05);
    return res ? res.pubkey === pubkey : false;
  }
  var nip10_exports = {};
  __export2(nip10_exports, {
    parse: () => parse
  });
  function parse(event) {
    const result = {
      reply: void 0,
      root: void 0,
      mentions: [],
      profiles: [],
      quotes: []
    };
    let maybeParent;
    let maybeRoot;
    for (let i2 = event.tags.length - 1; i2 >= 0; i2--) {
      const tag = event.tags[i2];
      if (tag[0] === "e" && tag[1]) {
        const [_, eTagEventId, eTagRelayUrl, eTagMarker, eTagAuthor] = tag;
        const eventPointer = {
          id: eTagEventId,
          relays: eTagRelayUrl ? [eTagRelayUrl] : [],
          author: eTagAuthor
        };
        if (eTagMarker === "root") {
          result.root = eventPointer;
          continue;
        }
        if (eTagMarker === "reply") {
          result.reply = eventPointer;
          continue;
        }
        if (eTagMarker === "mention") {
          result.mentions.push(eventPointer);
          continue;
        }
        if (!maybeParent) {
          maybeParent = eventPointer;
        } else {
          maybeRoot = eventPointer;
        }
        result.mentions.push(eventPointer);
        continue;
      }
      if (tag[0] === "q" && tag[1]) {
        const [_, eTagEventId, eTagRelayUrl] = tag;
        result.quotes.push({
          id: eTagEventId,
          relays: eTagRelayUrl ? [eTagRelayUrl] : []
        });
      }
      if (tag[0] === "p" && tag[1]) {
        result.profiles.push({
          pubkey: tag[1],
          relays: tag[2] ? [tag[2]] : []
        });
        continue;
      }
    }
    if (!result.root) {
      result.root = maybeRoot || maybeParent || result.reply;
    }
    if (!result.reply) {
      result.reply = maybeParent || result.root;
    }
    ;
    [result.reply, result.root].forEach((ref) => {
      if (!ref)
        return;
      let idx = result.mentions.indexOf(ref);
      if (idx !== -1) {
        result.mentions.splice(idx, 1);
      }
      if (ref.author) {
        let author = result.profiles.find((p) => p.pubkey === ref.author);
        if (author && author.relays) {
          if (!ref.relays) {
            ref.relays = [];
          }
          author.relays.forEach((url) => {
            if (ref.relays?.indexOf(url) === -1)
              ref.relays.push(url);
          });
          author.relays = ref.relays;
        }
      }
    });
    result.mentions.forEach((ref) => {
      if (ref.author) {
        let author = result.profiles.find((p) => p.pubkey === ref.author);
        if (author && author.relays) {
          if (!ref.relays) {
            ref.relays = [];
          }
          author.relays.forEach((url) => {
            if (ref.relays.indexOf(url) === -1)
              ref.relays.push(url);
          });
          author.relays = ref.relays;
        }
      }
    });
    return result;
  }
  var nip11_exports = {};
  __export2(nip11_exports, {
    fetchRelayInformation: () => fetchRelayInformation,
    useFetchImplementation: () => useFetchImplementation2
  });
  var _fetch2;
  try {
    _fetch2 = fetch;
  } catch {
  }
  function useFetchImplementation2(fetchImplementation) {
    _fetch2 = fetchImplementation;
  }
  async function fetchRelayInformation(url) {
    return await (await fetch(url.replace("ws://", "http://").replace("wss://", "https://"), {
      headers: { Accept: "application/nostr+json" }
    })).json();
  }
  var nip13_exports = {};
  __export2(nip13_exports, {
    fastEventHash: () => fastEventHash,
    getPow: () => getPow,
    minePow: () => minePow
  });
  function getPow(hex2) {
    let count = 0;
    for (let i2 = 0; i2 < 64; i2 += 8) {
      const nibble = parseInt(hex2.substring(i2, i2 + 8), 16);
      if (nibble === 0) {
        count += 32;
      } else {
        count += Math.clz32(nibble);
        break;
      }
    }
    return count;
  }
  function minePow(unsigned, difficulty) {
    let count = 0;
    const event = unsigned;
    const tag = ["nonce", count.toString(), difficulty.toString()];
    event.tags.push(tag);
    while (true) {
      const now2 = Math.floor((/* @__PURE__ */ new Date()).getTime() / 1e3);
      if (now2 !== event.created_at) {
        count = 0;
        event.created_at = now2;
      }
      tag[1] = (++count).toString();
      event.id = fastEventHash(event);
      if (getPow(event.id) >= difficulty) {
        break;
      }
    }
    return event;
  }
  function fastEventHash(evt) {
    return bytesToHex2(
      sha2562(utf8Encoder.encode(JSON.stringify([0, evt.pubkey, evt.created_at, evt.kind, evt.tags, evt.content])))
    );
  }
  var nip17_exports = {};
  __export2(nip17_exports, {
    unwrapEvent: () => unwrapEvent2,
    unwrapManyEvents: () => unwrapManyEvents2,
    wrapEvent: () => wrapEvent2,
    wrapManyEvents: () => wrapManyEvents2
  });
  var nip59_exports = {};
  __export2(nip59_exports, {
    createRumor: () => createRumor,
    createSeal: () => createSeal,
    createWrap: () => createWrap,
    unwrapEvent: () => unwrapEvent,
    unwrapManyEvents: () => unwrapManyEvents,
    wrapEvent: () => wrapEvent,
    wrapManyEvents: () => wrapManyEvents
  });
  var nip44_exports = {};
  __export2(nip44_exports, {
    decrypt: () => decrypt22,
    encrypt: () => encrypt22,
    getConversationKey: () => getConversationKey,
    v2: () => v2
  });
  var minPlaintextSize = 1;
  var maxPlaintextSize = 65535;
  function getConversationKey(privkeyA, pubkeyB) {
    const sharedX = secp256k1.getSharedSecret(privkeyA, "02" + pubkeyB).subarray(1, 33);
    return extract(sha2562, sharedX, "nip44-v2");
  }
  function getMessageKeys(conversationKey, nonce) {
    const keys = expand(sha2562, conversationKey, nonce, 76);
    return {
      chacha_key: keys.subarray(0, 32),
      chacha_nonce: keys.subarray(32, 44),
      hmac_key: keys.subarray(44, 76)
    };
  }
  function calcPaddedLen(len) {
    if (!Number.isSafeInteger(len) || len < 1)
      throw new Error("expected positive integer");
    if (len <= 32)
      return 32;
    const nextPower = 1 << Math.floor(Math.log2(len - 1)) + 1;
    const chunk = nextPower <= 256 ? 32 : nextPower / 8;
    return chunk * (Math.floor((len - 1) / chunk) + 1);
  }
  function writeU16BE(num) {
    if (!Number.isSafeInteger(num) || num < minPlaintextSize || num > maxPlaintextSize)
      throw new Error("invalid plaintext size: must be between 1 and 65535 bytes");
    const arr = new Uint8Array(2);
    new DataView(arr.buffer).setUint16(0, num, false);
    return arr;
  }
  function pad(plaintext) {
    const unpadded = utf8Encoder.encode(plaintext);
    const unpaddedLen = unpadded.length;
    const prefix = writeU16BE(unpaddedLen);
    const suffix = new Uint8Array(calcPaddedLen(unpaddedLen) - unpaddedLen);
    return concatBytes3(prefix, unpadded, suffix);
  }
  function unpad(padded) {
    const unpaddedLen = new DataView(padded.buffer).getUint16(0);
    const unpadded = padded.subarray(2, 2 + unpaddedLen);
    if (unpaddedLen < minPlaintextSize || unpaddedLen > maxPlaintextSize || unpadded.length !== unpaddedLen || padded.length !== 2 + calcPaddedLen(unpaddedLen))
      throw new Error("invalid padding");
    return utf8Decoder.decode(unpadded);
  }
  function hmacAad(key, message, aad) {
    if (aad.length !== 32)
      throw new Error("AAD associated data must be 32 bytes");
    const combined = concatBytes3(aad, message);
    return hmac2(sha2562, key, combined);
  }
  function decodePayload(payload) {
    if (typeof payload !== "string")
      throw new Error("payload must be a valid string");
    const plen = payload.length;
    if (plen < 132 || plen > 87472)
      throw new Error("invalid payload length: " + plen);
    if (payload[0] === "#")
      throw new Error("unknown encryption version");
    let data;
    try {
      data = base64.decode(payload);
    } catch (error) {
      throw new Error("invalid base64: " + error.message);
    }
    const dlen = data.length;
    if (dlen < 99 || dlen > 65603)
      throw new Error("invalid data length: " + dlen);
    const vers = data[0];
    if (vers !== 2)
      throw new Error("unknown encryption version " + vers);
    return {
      nonce: data.subarray(1, 33),
      ciphertext: data.subarray(33, -32),
      mac: data.subarray(-32)
    };
  }
  function encrypt22(plaintext, conversationKey, nonce = randomBytes2(32)) {
    const { chacha_key, chacha_nonce, hmac_key } = getMessageKeys(conversationKey, nonce);
    const padded = pad(plaintext);
    const ciphertext = chacha20(chacha_key, chacha_nonce, padded);
    const mac = hmacAad(hmac_key, ciphertext, nonce);
    return base64.encode(concatBytes3(new Uint8Array([2]), nonce, ciphertext, mac));
  }
  function decrypt22(payload, conversationKey) {
    const { nonce, ciphertext, mac } = decodePayload(payload);
    const { chacha_key, chacha_nonce, hmac_key } = getMessageKeys(conversationKey, nonce);
    const calculatedMac = hmacAad(hmac_key, ciphertext, nonce);
    if (!equalBytes2(calculatedMac, mac))
      throw new Error("invalid MAC");
    const padded = chacha20(chacha_key, chacha_nonce, ciphertext);
    return unpad(padded);
  }
  var v2 = {
    utils: {
      getConversationKey,
      calcPaddedLen
    },
    encrypt: encrypt22,
    decrypt: decrypt22
  };
  var TWO_DAYS = 2 * 24 * 60 * 60;
  var now = () => Math.round(Date.now() / 1e3);
  var randomNow = () => Math.round(now() - Math.random() * TWO_DAYS);
  var nip44ConversationKey = (privateKey, publicKey) => getConversationKey(privateKey, publicKey);
  var nip44Encrypt = (data, privateKey, publicKey) => encrypt22(JSON.stringify(data), nip44ConversationKey(privateKey, publicKey));
  var nip44Decrypt = (data, privateKey) => JSON.parse(decrypt22(data.content, nip44ConversationKey(privateKey, data.pubkey)));
  function createRumor(event, privateKey) {
    const rumor = {
      created_at: now(),
      content: "",
      tags: [],
      ...event,
      pubkey: getPublicKey(privateKey)
    };
    rumor.id = getEventHash(rumor);
    return rumor;
  }
  function createSeal(rumor, privateKey, recipientPublicKey) {
    return finalizeEvent(
      {
        kind: Seal,
        content: nip44Encrypt(rumor, privateKey, recipientPublicKey),
        created_at: randomNow(),
        tags: []
      },
      privateKey
    );
  }
  function createWrap(seal, recipientPublicKey) {
    const randomKey = generateSecretKey();
    return finalizeEvent(
      {
        kind: GiftWrap,
        content: nip44Encrypt(seal, randomKey, recipientPublicKey),
        created_at: randomNow(),
        tags: [["p", recipientPublicKey]]
      },
      randomKey
    );
  }
  function wrapEvent(event, senderPrivateKey, recipientPublicKey) {
    const rumor = createRumor(event, senderPrivateKey);
    const seal = createSeal(rumor, senderPrivateKey, recipientPublicKey);
    return createWrap(seal, recipientPublicKey);
  }
  function wrapManyEvents(event, senderPrivateKey, recipientsPublicKeys) {
    if (!recipientsPublicKeys || recipientsPublicKeys.length === 0) {
      throw new Error("At least one recipient is required.");
    }
    const senderPublicKey = getPublicKey(senderPrivateKey);
    const wrappeds = [wrapEvent(event, senderPrivateKey, senderPublicKey)];
    recipientsPublicKeys.forEach((recipientPublicKey) => {
      wrappeds.push(wrapEvent(event, senderPrivateKey, recipientPublicKey));
    });
    return wrappeds;
  }
  function unwrapEvent(wrap2, recipientPrivateKey) {
    const unwrappedSeal = nip44Decrypt(wrap2, recipientPrivateKey);
    return nip44Decrypt(unwrappedSeal, recipientPrivateKey);
  }
  function unwrapManyEvents(wrappedEvents, recipientPrivateKey) {
    let unwrappedEvents = [];
    wrappedEvents.forEach((e) => {
      unwrappedEvents.push(unwrapEvent(e, recipientPrivateKey));
    });
    unwrappedEvents.sort((a, b) => a.created_at - b.created_at);
    return unwrappedEvents;
  }
  function createEvent(recipients, message, conversationTitle, replyTo) {
    const baseEvent = {
      created_at: Math.ceil(Date.now() / 1e3),
      kind: PrivateDirectMessage,
      tags: [],
      content: message
    };
    const recipientsArray = Array.isArray(recipients) ? recipients : [recipients];
    recipientsArray.forEach(({ publicKey, relayUrl }) => {
      baseEvent.tags.push(relayUrl ? ["p", publicKey, relayUrl] : ["p", publicKey]);
    });
    if (replyTo) {
      baseEvent.tags.push(["e", replyTo.eventId, replyTo.relayUrl || "", "reply"]);
    }
    if (conversationTitle) {
      baseEvent.tags.push(["subject", conversationTitle]);
    }
    return baseEvent;
  }
  function wrapEvent2(senderPrivateKey, recipient, message, conversationTitle, replyTo) {
    const event = createEvent(recipient, message, conversationTitle, replyTo);
    return wrapEvent(event, senderPrivateKey, recipient.publicKey);
  }
  function wrapManyEvents2(senderPrivateKey, recipients, message, conversationTitle, replyTo) {
    if (!recipients || recipients.length === 0) {
      throw new Error("At least one recipient is required.");
    }
    const senderPublicKey = getPublicKey(senderPrivateKey);
    return [{ publicKey: senderPublicKey }, ...recipients].map(
      (recipient) => wrapEvent2(senderPrivateKey, recipient, message, conversationTitle, replyTo)
    );
  }
  var unwrapEvent2 = unwrapEvent;
  var unwrapManyEvents2 = unwrapManyEvents;
  var nip18_exports = {};
  __export2(nip18_exports, {
    finishRepostEvent: () => finishRepostEvent,
    getRepostedEvent: () => getRepostedEvent,
    getRepostedEventPointer: () => getRepostedEventPointer
  });
  function finishRepostEvent(t, reposted, relayUrl, privateKey) {
    let kind;
    const tags = [...t.tags ?? [], ["e", reposted.id, relayUrl], ["p", reposted.pubkey]];
    if (reposted.kind === ShortTextNote) {
      kind = Repost;
    } else {
      kind = GenericRepost;
      tags.push(["k", String(reposted.kind)]);
    }
    return finalizeEvent(
      {
        kind,
        tags,
        content: t.content === "" || reposted.tags?.find((tag) => tag[0] === "-") ? "" : JSON.stringify(reposted),
        created_at: t.created_at
      },
      privateKey
    );
  }
  function getRepostedEventPointer(event) {
    if (![Repost, GenericRepost].includes(event.kind)) {
      return void 0;
    }
    let lastETag;
    let lastPTag;
    for (let i2 = event.tags.length - 1; i2 >= 0 && (lastETag === void 0 || lastPTag === void 0); i2--) {
      const tag = event.tags[i2];
      if (tag.length >= 2) {
        if (tag[0] === "e" && lastETag === void 0) {
          lastETag = tag;
        } else if (tag[0] === "p" && lastPTag === void 0) {
          lastPTag = tag;
        }
      }
    }
    if (lastETag === void 0) {
      return void 0;
    }
    return {
      id: lastETag[1],
      relays: [lastETag[2], lastPTag?.[2]].filter((x) => typeof x === "string"),
      author: lastPTag?.[1]
    };
  }
  function getRepostedEvent(event, { skipVerification } = {}) {
    const pointer = getRepostedEventPointer(event);
    if (pointer === void 0 || event.content === "") {
      return void 0;
    }
    let repostedEvent;
    try {
      repostedEvent = JSON.parse(event.content);
    } catch (error) {
      return void 0;
    }
    if (repostedEvent.id !== pointer.id) {
      return void 0;
    }
    if (!skipVerification && !verifyEvent(repostedEvent)) {
      return void 0;
    }
    return repostedEvent;
  }
  var nip21_exports = {};
  __export2(nip21_exports, {
    NOSTR_URI_REGEX: () => NOSTR_URI_REGEX,
    parse: () => parse2,
    test: () => test
  });
  var NOSTR_URI_REGEX = new RegExp(`nostr:(${BECH32_REGEX.source})`);
  function test(value) {
    return typeof value === "string" && new RegExp(`^${NOSTR_URI_REGEX.source}$`).test(value);
  }
  function parse2(uri) {
    const match = uri.match(new RegExp(`^${NOSTR_URI_REGEX.source}$`));
    if (!match)
      throw new Error(`Invalid Nostr URI: ${uri}`);
    return {
      uri: match[0],
      value: match[1],
      decoded: decode(match[1])
    };
  }
  var nip25_exports = {};
  __export2(nip25_exports, {
    finishReactionEvent: () => finishReactionEvent,
    getReactedEventPointer: () => getReactedEventPointer
  });
  function finishReactionEvent(t, reacted, privateKey) {
    const inheritedTags = reacted.tags.filter((tag) => tag.length >= 2 && (tag[0] === "e" || tag[0] === "p"));
    return finalizeEvent(
      {
        ...t,
        kind: Reaction,
        tags: [...t.tags ?? [], ...inheritedTags, ["e", reacted.id], ["p", reacted.pubkey]],
        content: t.content ?? "+"
      },
      privateKey
    );
  }
  function getReactedEventPointer(event) {
    if (event.kind !== Reaction) {
      return void 0;
    }
    let lastETag;
    let lastPTag;
    for (let i2 = event.tags.length - 1; i2 >= 0 && (lastETag === void 0 || lastPTag === void 0); i2--) {
      const tag = event.tags[i2];
      if (tag.length >= 2) {
        if (tag[0] === "e" && lastETag === void 0) {
          lastETag = tag;
        } else if (tag[0] === "p" && lastPTag === void 0) {
          lastPTag = tag;
        }
      }
    }
    if (lastETag === void 0 || lastPTag === void 0) {
      return void 0;
    }
    return {
      id: lastETag[1],
      relays: [lastETag[2], lastPTag[2]].filter((x) => x !== void 0),
      author: lastPTag[1]
    };
  }
  var nip27_exports = {};
  __export2(nip27_exports, {
    parse: () => parse3
  });
  var noCharacter = /\W/m;
  var noURLCharacter = /\W |\W$|$|,| /m;
  function* parse3(content) {
    const max = content.length;
    let prevIndex = 0;
    let index = 0;
    while (index < max) {
      let u = content.indexOf(":", index);
      if (u === -1) {
        break;
      }
      if (content.substring(u - 5, u) === "nostr") {
        const m = content.substring(u + 60).match(noCharacter);
        const end = m ? u + 60 + m.index : max;
        try {
          let pointer;
          let { data, type } = decode(content.substring(u + 1, end));
          switch (type) {
            case "npub":
              pointer = { pubkey: data };
              break;
            case "nsec":
            case "note":
              index = end + 1;
              continue;
            default:
              pointer = data;
          }
          if (prevIndex !== u - 5) {
            yield { type: "text", text: content.substring(prevIndex, u - 5) };
          }
          yield { type: "reference", pointer };
          index = end;
          prevIndex = index;
          continue;
        } catch (_err) {
          index = u + 1;
          continue;
        }
      } else if (content.substring(u - 5, u) === "https" || content.substring(u - 4, u) === "http") {
        const m = content.substring(u + 4).match(noURLCharacter);
        const end = m ? u + 4 + m.index : max;
        const prefixLen = content[u - 1] === "s" ? 5 : 4;
        try {
          let url = new URL(content.substring(u - prefixLen, end));
          if (url.hostname.indexOf(".") === -1) {
            throw new Error("invalid url");
          }
          if (prevIndex !== u - prefixLen) {
            yield { type: "text", text: content.substring(prevIndex, u - prefixLen) };
          }
          if (url.pathname.endsWith(".png") || url.pathname.endsWith(".jpg") || url.pathname.endsWith(".jpeg") || url.pathname.endsWith(".gif") || url.pathname.endsWith(".webp")) {
            yield { type: "image", url: url.toString() };
            index = end;
            prevIndex = index;
            continue;
          }
          if (url.pathname.endsWith(".mp4") || url.pathname.endsWith(".avi") || url.pathname.endsWith(".webm") || url.pathname.endsWith(".mkv")) {
            yield { type: "video", url: url.toString() };
            index = end;
            prevIndex = index;
            continue;
          }
          if (url.pathname.endsWith(".mp3") || url.pathname.endsWith(".aac") || url.pathname.endsWith(".ogg") || url.pathname.endsWith(".opus")) {
            yield { type: "audio", url: url.toString() };
            index = end;
            prevIndex = index;
            continue;
          }
          yield { type: "url", url: url.toString() };
          index = end;
          prevIndex = index;
          continue;
        } catch (_err) {
          index = end + 1;
          continue;
        }
      } else if (content.substring(u - 3, u) === "wss" || content.substring(u - 2, u) === "ws") {
        const m = content.substring(u + 4).match(noURLCharacter);
        const end = m ? u + 4 + m.index : max;
        const prefixLen = content[u - 1] === "s" ? 3 : 2;
        try {
          let url = new URL(content.substring(u - prefixLen, end));
          if (url.hostname.indexOf(".") === -1) {
            throw new Error("invalid ws url");
          }
          if (prevIndex !== u - prefixLen) {
            yield { type: "text", text: content.substring(prevIndex, u - prefixLen) };
          }
          yield { type: "relay", url: url.toString() };
          index = end;
          prevIndex = index;
          continue;
        } catch (_err) {
          index = end + 1;
          continue;
        }
      } else {
        index = u + 1;
        continue;
      }
    }
    if (prevIndex !== max) {
      yield { type: "text", text: content.substring(prevIndex) };
    }
  }
  var nip28_exports = {};
  __export2(nip28_exports, {
    channelCreateEvent: () => channelCreateEvent,
    channelHideMessageEvent: () => channelHideMessageEvent,
    channelMessageEvent: () => channelMessageEvent,
    channelMetadataEvent: () => channelMetadataEvent,
    channelMuteUserEvent: () => channelMuteUserEvent
  });
  var channelCreateEvent = (t, privateKey) => {
    let content;
    if (typeof t.content === "object") {
      content = JSON.stringify(t.content);
    } else if (typeof t.content === "string") {
      content = t.content;
    } else {
      return void 0;
    }
    return finalizeEvent(
      {
        kind: ChannelCreation,
        tags: [...t.tags ?? []],
        content,
        created_at: t.created_at
      },
      privateKey
    );
  };
  var channelMetadataEvent = (t, privateKey) => {
    let content;
    if (typeof t.content === "object") {
      content = JSON.stringify(t.content);
    } else if (typeof t.content === "string") {
      content = t.content;
    } else {
      return void 0;
    }
    return finalizeEvent(
      {
        kind: ChannelMetadata,
        tags: [["e", t.channel_create_event_id], ...t.tags ?? []],
        content,
        created_at: t.created_at
      },
      privateKey
    );
  };
  var channelMessageEvent = (t, privateKey) => {
    const tags = [["e", t.channel_create_event_id, t.relay_url, "root"]];
    if (t.reply_to_channel_message_event_id) {
      tags.push(["e", t.reply_to_channel_message_event_id, t.relay_url, "reply"]);
    }
    return finalizeEvent(
      {
        kind: ChannelMessage,
        tags: [...tags, ...t.tags ?? []],
        content: t.content,
        created_at: t.created_at
      },
      privateKey
    );
  };
  var channelHideMessageEvent = (t, privateKey) => {
    let content;
    if (typeof t.content === "object") {
      content = JSON.stringify(t.content);
    } else if (typeof t.content === "string") {
      content = t.content;
    } else {
      return void 0;
    }
    return finalizeEvent(
      {
        kind: ChannelHideMessage,
        tags: [["e", t.channel_message_event_id], ...t.tags ?? []],
        content,
        created_at: t.created_at
      },
      privateKey
    );
  };
  var channelMuteUserEvent = (t, privateKey) => {
    let content;
    if (typeof t.content === "object") {
      content = JSON.stringify(t.content);
    } else if (typeof t.content === "string") {
      content = t.content;
    } else {
      return void 0;
    }
    return finalizeEvent(
      {
        kind: ChannelMuteUser,
        tags: [["p", t.pubkey_to_mute], ...t.tags ?? []],
        content,
        created_at: t.created_at
      },
      privateKey
    );
  };
  var nip30_exports = {};
  __export2(nip30_exports, {
    EMOJI_SHORTCODE_REGEX: () => EMOJI_SHORTCODE_REGEX,
    matchAll: () => matchAll,
    regex: () => regex,
    replaceAll: () => replaceAll
  });
  var EMOJI_SHORTCODE_REGEX = /:(\w+):/;
  var regex = () => new RegExp(`\\B${EMOJI_SHORTCODE_REGEX.source}\\B`, "g");
  function* matchAll(content) {
    const matches = content.matchAll(regex());
    for (const match of matches) {
      try {
        const [shortcode, name] = match;
        yield {
          shortcode,
          name,
          start: match.index,
          end: match.index + shortcode.length
        };
      } catch (_e) {
      }
    }
  }
  function replaceAll(content, replacer) {
    return content.replaceAll(regex(), (shortcode, name) => {
      return replacer({
        shortcode,
        name
      });
    });
  }
  var nip39_exports = {};
  __export2(nip39_exports, {
    useFetchImplementation: () => useFetchImplementation3,
    validateGithub: () => validateGithub
  });
  var _fetch3;
  try {
    _fetch3 = fetch;
  } catch {
  }
  function useFetchImplementation3(fetchImplementation) {
    _fetch3 = fetchImplementation;
  }
  async function validateGithub(pubkey, username, proof) {
    try {
      let res = await (await _fetch3(`https://gist.github.com/${username}/${proof}/raw`)).text();
      return res === `Verifying that I control the following Nostr public key: ${pubkey}`;
    } catch (_) {
      return false;
    }
  }
  var nip47_exports = {};
  __export2(nip47_exports, {
    makeNwcRequestEvent: () => makeNwcRequestEvent,
    parseConnectionString: () => parseConnectionString
  });
  function parseConnectionString(connectionString) {
    const { pathname, searchParams } = new URL(connectionString);
    const pubkey = pathname;
    const relay = searchParams.get("relay");
    const secret = searchParams.get("secret");
    if (!pubkey || !relay || !secret) {
      throw new Error("invalid connection string");
    }
    return { pubkey, relay, secret };
  }
  async function makeNwcRequestEvent(pubkey, secretKey, invoice) {
    const content = {
      method: "pay_invoice",
      params: {
        invoice
      }
    };
    const encryptedContent = encrypt2(secretKey, pubkey, JSON.stringify(content));
    const eventTemplate = {
      kind: NWCWalletRequest,
      created_at: Math.round(Date.now() / 1e3),
      content: encryptedContent,
      tags: [["p", pubkey]]
    };
    return finalizeEvent(eventTemplate, secretKey);
  }
  var nip54_exports = {};
  __export2(nip54_exports, {
    normalizeIdentifier: () => normalizeIdentifier
  });
  function normalizeIdentifier(name) {
    name = name.trim().toLowerCase();
    name = name.normalize("NFKC");
    return Array.from(name).map((char) => {
      if (/\p{Letter}/u.test(char) || /\p{Number}/u.test(char)) {
        return char;
      }
      return "-";
    }).join("");
  }
  var nip57_exports = {};
  __export2(nip57_exports, {
    getSatoshisAmountFromBolt11: () => getSatoshisAmountFromBolt11,
    getZapEndpoint: () => getZapEndpoint,
    makeZapReceipt: () => makeZapReceipt,
    makeZapRequest: () => makeZapRequest,
    useFetchImplementation: () => useFetchImplementation4,
    validateZapRequest: () => validateZapRequest
  });
  var _fetch4;
  try {
    _fetch4 = fetch;
  } catch {
  }
  function useFetchImplementation4(fetchImplementation) {
    _fetch4 = fetchImplementation;
  }
  async function getZapEndpoint(metadata) {
    try {
      let lnurl = "";
      let { lud06, lud16 } = JSON.parse(metadata.content);
      if (lud06) {
        let { words } = bech32.decode(lud06, 1e3);
        let data = bech32.fromWords(words);
        lnurl = utf8Decoder.decode(data);
      } else if (lud16) {
        let [name, domain] = lud16.split("@");
        lnurl = new URL(`/.well-known/lnurlp/${name}`, `https://${domain}`).toString();
      } else {
        return null;
      }
      let res = await _fetch4(lnurl);
      let body = await res.json();
      if (body.allowsNostr && body.nostrPubkey) {
        return body.callback;
      }
    } catch (err) {
    }
    return null;
  }
  function makeZapRequest({
    profile,
    event,
    amount,
    relays,
    comment = ""
  }) {
    if (!amount)
      throw new Error("amount not given");
    if (!profile)
      throw new Error("profile not given");
    let zr = {
      kind: 9734,
      created_at: Math.round(Date.now() / 1e3),
      content: comment,
      tags: [
        ["p", profile],
        ["amount", amount.toString()],
        ["relays", ...relays]
      ]
    };
    if (event && typeof event === "string") {
      zr.tags.push(["e", event]);
    }
    if (event && typeof event === "object") {
      if (isReplaceableKind(event.kind)) {
        const a = ["a", `${event.kind}:${event.pubkey}:`];
        zr.tags.push(a);
      } else if (isAddressableKind(event.kind)) {
        let d = event.tags.find(([t, v]) => t === "d" && v);
        if (!d)
          throw new Error("d tag not found or is empty");
        const a = ["a", `${event.kind}:${event.pubkey}:${d[1]}`];
        zr.tags.push(a);
      }
    }
    return zr;
  }
  function validateZapRequest(zapRequestString) {
    let zapRequest;
    try {
      zapRequest = JSON.parse(zapRequestString);
    } catch (err) {
      return "Invalid zap request JSON.";
    }
    if (!validateEvent(zapRequest))
      return "Zap request is not a valid Nostr event.";
    if (!verifyEvent(zapRequest))
      return "Invalid signature on zap request.";
    let p = zapRequest.tags.find(([t, v]) => t === "p" && v);
    if (!p)
      return "Zap request doesn't have a 'p' tag.";
    if (!p[1].match(/^[a-f0-9]{64}$/))
      return "Zap request 'p' tag is not valid hex.";
    let e = zapRequest.tags.find(([t, v]) => t === "e" && v);
    if (e && !e[1].match(/^[a-f0-9]{64}$/))
      return "Zap request 'e' tag is not valid hex.";
    let relays = zapRequest.tags.find(([t, v]) => t === "relays" && v);
    if (!relays)
      return "Zap request doesn't have a 'relays' tag.";
    return null;
  }
  function makeZapReceipt({
    zapRequest,
    preimage,
    bolt11,
    paidAt
  }) {
    let zr = JSON.parse(zapRequest);
    let tagsFromZapRequest = zr.tags.filter(([t]) => t === "e" || t === "p" || t === "a");
    let zap = {
      kind: 9735,
      created_at: Math.round(paidAt.getTime() / 1e3),
      content: "",
      tags: [...tagsFromZapRequest, ["P", zr.pubkey], ["bolt11", bolt11], ["description", zapRequest]]
    };
    if (preimage) {
      zap.tags.push(["preimage", preimage]);
    }
    return zap;
  }
  function getSatoshisAmountFromBolt11(bolt11) {
    if (bolt11.length < 50) {
      return 0;
    }
    bolt11 = bolt11.substring(0, 50);
    const idx = bolt11.lastIndexOf("1");
    if (idx === -1) {
      return 0;
    }
    const hrp = bolt11.substring(0, idx);
    if (!hrp.startsWith("lnbc")) {
      return 0;
    }
    const amount = hrp.substring(4);
    if (amount.length < 1) {
      return 0;
    }
    const char = amount[amount.length - 1];
    const digit = char.charCodeAt(0) - "0".charCodeAt(0);
    const isDigit = digit >= 0 && digit <= 9;
    let cutPoint = amount.length - 1;
    if (isDigit) {
      cutPoint++;
    }
    if (cutPoint < 1) {
      return 0;
    }
    const num = parseInt(amount.substring(0, cutPoint));
    switch (char) {
      case "m":
        return num * 1e5;
      case "u":
        return num * 100;
      case "n":
        return num / 10;
      case "p":
        return num / 1e4;
      default:
        return num * 1e8;
    }
  }
  var nip98_exports = {};
  __export2(nip98_exports, {
    getToken: () => getToken,
    hashPayload: () => hashPayload,
    unpackEventFromToken: () => unpackEventFromToken,
    validateEvent: () => validateEvent2,
    validateEventKind: () => validateEventKind,
    validateEventMethodTag: () => validateEventMethodTag,
    validateEventPayloadTag: () => validateEventPayloadTag,
    validateEventTimestamp: () => validateEventTimestamp,
    validateEventUrlTag: () => validateEventUrlTag,
    validateToken: () => validateToken
  });
  var _authorizationScheme = "Nostr ";
  async function getToken(loginUrl, httpMethod, sign, includeAuthorizationScheme = false, payload) {
    const event = {
      kind: HTTPAuth,
      tags: [
        ["u", loginUrl],
        ["method", httpMethod]
      ],
      created_at: Math.round((/* @__PURE__ */ new Date()).getTime() / 1e3),
      content: ""
    };
    if (payload) {
      event.tags.push(["payload", hashPayload(payload)]);
    }
    const signedEvent = await sign(event);
    const authorizationScheme = includeAuthorizationScheme ? _authorizationScheme : "";
    return authorizationScheme + base64.encode(utf8Encoder.encode(JSON.stringify(signedEvent)));
  }
  async function validateToken(token, url, method) {
    const event = await unpackEventFromToken(token).catch((error) => {
      throw error;
    });
    const valid = await validateEvent2(event, url, method).catch((error) => {
      throw error;
    });
    return valid;
  }
  async function unpackEventFromToken(token) {
    if (!token) {
      throw new Error("Missing token");
    }
    token = token.replace(_authorizationScheme, "");
    const eventB64 = utf8Decoder.decode(base64.decode(token));
    if (!eventB64 || eventB64.length === 0 || !eventB64.startsWith("{")) {
      throw new Error("Invalid token");
    }
    const event = JSON.parse(eventB64);
    return event;
  }
  function validateEventTimestamp(event) {
    if (!event.created_at) {
      return false;
    }
    return Math.round((/* @__PURE__ */ new Date()).getTime() / 1e3) - event.created_at < 60;
  }
  function validateEventKind(event) {
    return event.kind === HTTPAuth;
  }
  function validateEventUrlTag(event, url) {
    const urlTag = event.tags.find((t) => t[0] === "u");
    if (!urlTag) {
      return false;
    }
    return urlTag.length > 0 && urlTag[1] === url;
  }
  function validateEventMethodTag(event, method) {
    const methodTag = event.tags.find((t) => t[0] === "method");
    if (!methodTag) {
      return false;
    }
    return methodTag.length > 0 && methodTag[1].toLowerCase() === method.toLowerCase();
  }
  function hashPayload(payload) {
    const hash3 = sha2562(utf8Encoder.encode(JSON.stringify(payload)));
    return bytesToHex2(hash3);
  }
  function validateEventPayloadTag(event, payload) {
    const payloadTag = event.tags.find((t) => t[0] === "payload");
    if (!payloadTag) {
      return false;
    }
    const payloadHash = hashPayload(payload);
    return payloadTag.length > 0 && payloadTag[1] === payloadHash;
  }
  async function validateEvent2(event, url, method, body) {
    if (!verifyEvent(event)) {
      throw new Error("Invalid nostr event, signature invalid");
    }
    if (!validateEventKind(event)) {
      throw new Error("Invalid nostr event, kind invalid");
    }
    if (!validateEventTimestamp(event)) {
      throw new Error("Invalid nostr event, created_at timestamp invalid");
    }
    if (!validateEventUrlTag(event, url)) {
      throw new Error("Invalid nostr event, url tag invalid");
    }
    if (!validateEventMethodTag(event, method)) {
      throw new Error("Invalid nostr event, method tag invalid");
    }
    if (Boolean(body) && typeof body === "object" && Object.keys(body).length > 0) {
      if (!validateEventPayloadTag(event, body)) {
        throw new Error("Invalid nostr event, payload tag does not match request body hash");
      }
    }
    return true;
  }

  // node_modules/async-mutex/index.mjs
  var E_TIMEOUT = new Error("timeout while waiting for mutex to become available");
  var E_ALREADY_LOCKED = new Error("mutex already locked");
  var E_CANCELED = new Error("request for lock canceled");
  var __awaiter$2 = function(thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P ? value : new P(function(resolve) {
        resolve(value);
      });
    }
    return new (P || (P = Promise))(function(resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
  var Semaphore = class {
    constructor(_value, _cancelError = E_CANCELED) {
      this._value = _value;
      this._cancelError = _cancelError;
      this._queue = [];
      this._weightedWaiters = [];
    }
    acquire(weight = 1, priority = 0) {
      if (weight <= 0)
        throw new Error(`invalid weight ${weight}: must be positive`);
      return new Promise((resolve, reject) => {
        const task = { resolve, reject, weight, priority };
        const i2 = findIndexFromEnd(this._queue, (other) => priority <= other.priority);
        if (i2 === -1 && weight <= this._value) {
          this._dispatchItem(task);
        } else {
          this._queue.splice(i2 + 1, 0, task);
        }
      });
    }
    runExclusive(callback_1) {
      return __awaiter$2(this, arguments, void 0, function* (callback, weight = 1, priority = 0) {
        const [value, release] = yield this.acquire(weight, priority);
        try {
          return yield callback(value);
        } finally {
          release();
        }
      });
    }
    waitForUnlock(weight = 1, priority = 0) {
      if (weight <= 0)
        throw new Error(`invalid weight ${weight}: must be positive`);
      if (this._couldLockImmediately(weight, priority)) {
        return Promise.resolve();
      } else {
        return new Promise((resolve) => {
          if (!this._weightedWaiters[weight - 1])
            this._weightedWaiters[weight - 1] = [];
          insertSorted(this._weightedWaiters[weight - 1], { resolve, priority });
        });
      }
    }
    isLocked() {
      return this._value <= 0;
    }
    getValue() {
      return this._value;
    }
    setValue(value) {
      this._value = value;
      this._dispatchQueue();
    }
    release(weight = 1) {
      if (weight <= 0)
        throw new Error(`invalid weight ${weight}: must be positive`);
      this._value += weight;
      this._dispatchQueue();
    }
    cancel() {
      this._queue.forEach((entry) => entry.reject(this._cancelError));
      this._queue = [];
    }
    _dispatchQueue() {
      this._drainUnlockWaiters();
      while (this._queue.length > 0 && this._queue[0].weight <= this._value) {
        this._dispatchItem(this._queue.shift());
        this._drainUnlockWaiters();
      }
    }
    _dispatchItem(item) {
      const previousValue = this._value;
      this._value -= item.weight;
      item.resolve([previousValue, this._newReleaser(item.weight)]);
    }
    _newReleaser(weight) {
      let called = false;
      return () => {
        if (called)
          return;
        called = true;
        this.release(weight);
      };
    }
    _drainUnlockWaiters() {
      if (this._queue.length === 0) {
        for (let weight = this._value; weight > 0; weight--) {
          const waiters = this._weightedWaiters[weight - 1];
          if (!waiters)
            continue;
          waiters.forEach((waiter) => waiter.resolve());
          this._weightedWaiters[weight - 1] = [];
        }
      } else {
        const queuedPriority = this._queue[0].priority;
        for (let weight = this._value; weight > 0; weight--) {
          const waiters = this._weightedWaiters[weight - 1];
          if (!waiters)
            continue;
          const i2 = waiters.findIndex((waiter) => waiter.priority <= queuedPriority);
          (i2 === -1 ? waiters : waiters.splice(0, i2)).forEach((waiter) => waiter.resolve());
        }
      }
    }
    _couldLockImmediately(weight, priority) {
      return (this._queue.length === 0 || this._queue[0].priority < priority) && weight <= this._value;
    }
  };
  function insertSorted(a, v) {
    const i2 = findIndexFromEnd(a, (other) => v.priority <= other.priority);
    a.splice(i2 + 1, 0, v);
  }
  function findIndexFromEnd(a, predicate) {
    for (let i2 = a.length - 1; i2 >= 0; i2--) {
      if (predicate(a[i2])) {
        return i2;
      }
    }
    return -1;
  }
  var __awaiter$1 = function(thisArg, _arguments, P, generator) {
    function adopt(value) {
      return value instanceof P ? value : new P(function(resolve) {
        resolve(value);
      });
    }
    return new (P || (P = Promise))(function(resolve, reject) {
      function fulfilled(value) {
        try {
          step(generator.next(value));
        } catch (e) {
          reject(e);
        }
      }
      function rejected(value) {
        try {
          step(generator["throw"](value));
        } catch (e) {
          reject(e);
        }
      }
      function step(result) {
        result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected);
      }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
  };
  var Mutex = class {
    constructor(cancelError) {
      this._semaphore = new Semaphore(1, cancelError);
    }
    acquire() {
      return __awaiter$1(this, arguments, void 0, function* (priority = 0) {
        const [, releaser] = yield this._semaphore.acquire(1, priority);
        return releaser;
      });
    }
    runExclusive(callback, priority = 0) {
      return this._semaphore.runExclusive(() => callback(), 1, priority);
    }
    isLocked() {
      return this._semaphore.isLocked();
    }
    waitForUnlock(priority = 0) {
      return this._semaphore.waitForUnlock(1, priority);
    }
    release() {
      if (this._semaphore.isLocked())
        this._semaphore.release();
    }
    cancel() {
      return this._semaphore.cancel();
    }
  };

  // Shared (Extension)/Resources/utilities/utils.js
  var storage = browser.storage.local;
  var RECOMMENDED_RELAYS = [
    new URL("wss://relay.damus.io"),
    new URL("wss://relay.snort.social"),
    new URL("wss://nos.lol"),
    new URL("wss://relay.primal.net"),
    new URL("wss://relay.nostr.band"),
    new URL("wss://nostr.orangepill.dev")
  ];
  async function getProfiles() {
    let profiles = await storage.get({ profiles: [] });
    return profiles.profiles;
  }
  async function getProfile(index) {
    let profiles = await getProfiles();
    return profiles[index];
  }
  async function getProfileIndex() {
    const index = await storage.get({ profileIndex: 0 });
    return index.profileIndex;
  }
  async function get(item) {
    return (await storage.get(item))[item];
  }
  async function getPermission(host, action) {
    let index = await getProfileIndex();
    let profile = await getProfile(index);
    console.log(host, action);
    console.log("profile: ", profile);
    return profile.hosts?.[host]?.[action] || "ask";
  }
  async function setPermission(host, action, perm, index = null) {
    let profiles = await getProfiles();
    if (!index) {
      index = await getProfileIndex();
    }
    let profile = profiles[index];
    let newPerms = profile.hosts[host] || {};
    newPerms = { ...newPerms, [action]: perm };
    profile.hosts[host] = newPerms;
    profiles[index] = profile;
    await storage.set({ profiles });
  }

  // node_modules/idb/build/index.js
  var instanceOfAny = (object, constructors) => constructors.some((c) => object instanceof c);
  var idbProxyableTypes;
  var cursorAdvanceMethods;
  function getIdbProxyableTypes() {
    return idbProxyableTypes || (idbProxyableTypes = [
      IDBDatabase,
      IDBObjectStore,
      IDBIndex,
      IDBCursor,
      IDBTransaction
    ]);
  }
  function getCursorAdvanceMethods() {
    return cursorAdvanceMethods || (cursorAdvanceMethods = [
      IDBCursor.prototype.advance,
      IDBCursor.prototype.continue,
      IDBCursor.prototype.continuePrimaryKey
    ]);
  }
  var transactionDoneMap = /* @__PURE__ */ new WeakMap();
  var transformCache = /* @__PURE__ */ new WeakMap();
  var reverseTransformCache = /* @__PURE__ */ new WeakMap();
  function promisifyRequest(request) {
    const promise = new Promise((resolve, reject) => {
      const unlisten = () => {
        request.removeEventListener("success", success);
        request.removeEventListener("error", error);
      };
      const success = () => {
        resolve(wrap(request.result));
        unlisten();
      };
      const error = () => {
        reject(request.error);
        unlisten();
      };
      request.addEventListener("success", success);
      request.addEventListener("error", error);
    });
    reverseTransformCache.set(promise, request);
    return promise;
  }
  function cacheDonePromiseForTransaction(tx) {
    if (transactionDoneMap.has(tx))
      return;
    const done = new Promise((resolve, reject) => {
      const unlisten = () => {
        tx.removeEventListener("complete", complete2);
        tx.removeEventListener("error", error);
        tx.removeEventListener("abort", error);
      };
      const complete2 = () => {
        resolve();
        unlisten();
      };
      const error = () => {
        reject(tx.error || new DOMException("AbortError", "AbortError"));
        unlisten();
      };
      tx.addEventListener("complete", complete2);
      tx.addEventListener("error", error);
      tx.addEventListener("abort", error);
    });
    transactionDoneMap.set(tx, done);
  }
  var idbProxyTraps = {
    get(target, prop, receiver) {
      if (target instanceof IDBTransaction) {
        if (prop === "done")
          return transactionDoneMap.get(target);
        if (prop === "store") {
          return receiver.objectStoreNames[1] ? void 0 : receiver.objectStore(receiver.objectStoreNames[0]);
        }
      }
      return wrap(target[prop]);
    },
    set(target, prop, value) {
      target[prop] = value;
      return true;
    },
    has(target, prop) {
      if (target instanceof IDBTransaction && (prop === "done" || prop === "store")) {
        return true;
      }
      return prop in target;
    }
  };
  function replaceTraps(callback) {
    idbProxyTraps = callback(idbProxyTraps);
  }
  function wrapFunction(func) {
    if (getCursorAdvanceMethods().includes(func)) {
      return function(...args) {
        func.apply(unwrap(this), args);
        return wrap(this.request);
      };
    }
    return function(...args) {
      return wrap(func.apply(unwrap(this), args));
    };
  }
  function transformCachableValue(value) {
    if (typeof value === "function")
      return wrapFunction(value);
    if (value instanceof IDBTransaction)
      cacheDonePromiseForTransaction(value);
    if (instanceOfAny(value, getIdbProxyableTypes()))
      return new Proxy(value, idbProxyTraps);
    return value;
  }
  function wrap(value) {
    if (value instanceof IDBRequest)
      return promisifyRequest(value);
    if (transformCache.has(value))
      return transformCache.get(value);
    const newValue = transformCachableValue(value);
    if (newValue !== value) {
      transformCache.set(value, newValue);
      reverseTransformCache.set(newValue, value);
    }
    return newValue;
  }
  var unwrap = (value) => reverseTransformCache.get(value);
  function openDB(name, version, { blocked, upgrade, blocking, terminated } = {}) {
    const request = indexedDB.open(name, version);
    const openPromise = wrap(request);
    if (upgrade) {
      request.addEventListener("upgradeneeded", (event) => {
        upgrade(wrap(request.result), event.oldVersion, event.newVersion, wrap(request.transaction), event);
      });
    }
    if (blocked) {
      request.addEventListener("blocked", (event) => blocked(
        // Casting due to https://github.com/microsoft/TypeScript-DOM-lib-generator/pull/1405
        event.oldVersion,
        event.newVersion,
        event
      ));
    }
    openPromise.then((db) => {
      if (terminated)
        db.addEventListener("close", () => terminated());
      if (blocking) {
        db.addEventListener("versionchange", (event) => blocking(event.oldVersion, event.newVersion, event));
      }
    }).catch(() => {
    });
    return openPromise;
  }
  var readMethods = ["get", "getKey", "getAll", "getAllKeys", "count"];
  var writeMethods = ["put", "add", "delete", "clear"];
  var cachedMethods = /* @__PURE__ */ new Map();
  function getMethod(target, prop) {
    if (!(target instanceof IDBDatabase && !(prop in target) && typeof prop === "string")) {
      return;
    }
    if (cachedMethods.get(prop))
      return cachedMethods.get(prop);
    const targetFuncName = prop.replace(/FromIndex$/, "");
    const useIndex = prop !== targetFuncName;
    const isWrite = writeMethods.includes(targetFuncName);
    if (
      // Bail if the target doesn't exist on the target. Eg, getAll isn't in Edge.
      !(targetFuncName in (useIndex ? IDBIndex : IDBObjectStore).prototype) || !(isWrite || readMethods.includes(targetFuncName))
    ) {
      return;
    }
    const method = async function(storeName, ...args) {
      const tx = this.transaction(storeName, isWrite ? "readwrite" : "readonly");
      let target2 = tx.store;
      if (useIndex)
        target2 = target2.index(args.shift());
      return (await Promise.all([
        target2[targetFuncName](...args),
        isWrite && tx.done
      ]))[0];
    };
    cachedMethods.set(prop, method);
    return method;
  }
  replaceTraps((oldTraps) => ({
    ...oldTraps,
    get: (target, prop, receiver) => getMethod(target, prop) || oldTraps.get(target, prop, receiver),
    has: (target, prop) => !!getMethod(target, prop) || oldTraps.has(target, prop)
  }));
  var advanceMethodProps = ["continue", "continuePrimaryKey", "advance"];
  var methodMap = {};
  var advanceResults = /* @__PURE__ */ new WeakMap();
  var ittrProxiedCursorToOriginalProxy = /* @__PURE__ */ new WeakMap();
  var cursorIteratorTraps = {
    get(target, prop) {
      if (!advanceMethodProps.includes(prop))
        return target[prop];
      let cachedFunc = methodMap[prop];
      if (!cachedFunc) {
        cachedFunc = methodMap[prop] = function(...args) {
          advanceResults.set(this, ittrProxiedCursorToOriginalProxy.get(this)[prop](...args));
        };
      }
      return cachedFunc;
    }
  };
  async function* iterate(...args) {
    let cursor = this;
    if (!(cursor instanceof IDBCursor)) {
      cursor = await cursor.openCursor(...args);
    }
    if (!cursor)
      return;
    cursor = cursor;
    const proxiedCursor = new Proxy(cursor, cursorIteratorTraps);
    ittrProxiedCursorToOriginalProxy.set(proxiedCursor, cursor);
    reverseTransformCache.set(proxiedCursor, unwrap(cursor));
    while (cursor) {
      yield proxiedCursor;
      cursor = await (advanceResults.get(proxiedCursor) || cursor.continue());
      advanceResults.delete(proxiedCursor);
    }
  }
  function isIteratorProp(target, prop) {
    return prop === Symbol.asyncIterator && instanceOfAny(target, [IDBIndex, IDBObjectStore, IDBCursor]) || prop === "iterate" && instanceOfAny(target, [IDBIndex, IDBObjectStore]);
  }
  replaceTraps((oldTraps) => ({
    ...oldTraps,
    get(target, prop, receiver) {
      if (isIteratorProp(target, prop))
        return iterate;
      return oldTraps.get(target, prop, receiver);
    },
    has(target, prop) {
      return isIteratorProp(target, prop) || oldTraps.has(target, prop);
    }
  }));

  // Shared (Extension)/Resources/utilities/db.js
  async function openEventsDb() {
    return await openDB("events", 1, {
      upgrade(db) {
        const events = db.createObjectStore("events", {
          keyPath: "event.id"
        });
        events.createIndex("pubkey", "event.pubkey");
        events.createIndex("created_at", "event.created_at");
        events.createIndex("kind", "event.kind");
        events.createIndex("host", "metadata.host");
      }
    });
  }
  async function saveEvent(event) {
    let db = await openEventsDb();
    return db.put("events", event);
  }

  // Shared (Extension)/Resources/background.js
  var storage2 = browser.storage.local;
  var log = (msg) => console.log("Background: ", msg);
  var validations = {};
  var prompt = { mutex: new Mutex(), release: null, tabId: null };
  browser.runtime.onMessage.addListener((message, _sender, sendResponse2) => {
    log(message);
    let uuid = crypto.randomUUID();
    let sr;
    switch (message.kind) {
      // General
      case "closePrompt":
        prompt.release?.();
        return Promise.resolve(true);
      case "allowed":
        complete(message);
        return Promise.resolve(true);
      case "denied":
        deny(message);
        return Promise.resolve(true);
      case "generatePrivateKey":
        return Promise.resolve(generatePrivateKey_());
      case "savePrivateKey":
        return savePrivateKey(message.payload);
      case "getNpub":
        return getNpub(message.payload);
      case "getNsec":
        return getNsec(message.payload);
      case "calcPubKey":
        return Promise.resolve(getPublicKey(message.payload));
      case "npubEncode":
        return Promise.resolve(nip19_exports.npubEncode(message.payload));
      case "copy":
        return navigator.clipboard.writeText(message.payload);
      // window.nostr
      case "getPubKey":
      case "signEvent":
      case "nip04.encrypt":
      case "nip04.decrypt":
      case "nip44.encrypt":
      case "nip44.decrypt":
      case "getRelays":
        validations[uuid] = sendResponse2;
        ask(uuid, message);
        setTimeout(() => {
          prompt.release?.();
        }, 1e4);
        return true;
      default:
        return Promise.resolve();
    }
  });
  async function forceRelease() {
    if (prompt.tabId !== null) {
      try {
        await browser.tabs.get(prompt.tabId);
      } catch (error) {
        prompt.release?.();
        prompt.tabId = null;
      }
    }
  }
  async function generatePrivateKey_() {
    const sk = generateSecretKey();
    return bytesToHex2(sk);
  }
  async function ask(uuid, { kind, host, payload }) {
    await forceRelease();
    prompt.release = await prompt.mutex.acquire();
    let mKind = kind === "signEvent" ? `signEvent:${payload.kind}` : kind;
    let permission = await getPermission(host, mKind);
    if (permission === "allow") {
      complete({
        payload: uuid,
        origKind: kind,
        event: payload,
        remember: false,
        host
      });
      prompt.release();
      return;
    }
    if (permission === "deny") {
      deny({ payload: uuid, origKind: kind, host });
      prompt.release();
      return;
    }
    let qs = new URLSearchParams({
      uuid,
      kind,
      host,
      payload: JSON.stringify(payload || false)
    });
    let tab = await browser.tabs.getCurrent();
    let p = await browser.tabs.create({
      url: `/permission/permission.html?${qs.toString()}`,
      openerTabId: tab.id
    });
    prompt.tabId = p.id;
    return true;
  }
  function complete({ payload, origKind, event, remember, host }) {
    sendResponse = validations[payload];
    if (remember) {
      let mKind = origKind === "signEvent" ? `signEvent:${event.kind}` : origKind;
      setPermission(host, mKind, "allow");
    }
    if (sendResponse) {
      switch (origKind) {
        case "getPubKey":
          getPubKey().then((pk) => {
            sendResponse(pk);
          });
          break;
        case "signEvent":
          signEvent_(event, host).then((e) => sendResponse(e));
          break;
        case "nip04.encrypt":
          nip04Encrypt(event).then((e) => sendResponse(e));
          break;
        case "nip04.decrypt":
          nip04Decrypt(event).then((e) => sendResponse(e));
          break;
        case "nip44.encrypt":
          nip44Encrypt2(event).then((e) => sendResponse(e));
          break;
        case "nip44.decrypt":
          nip44Decrypt2(event).then((e) => sendResponse(e));
          break;
        case "getRelays":
          getRelays().then((e) => sendResponse(e));
          break;
      }
    }
  }
  function deny({ origKind, host, payload, remember, event }) {
    sendResponse = validations[payload];
    if (remember) {
      let mKind = origKind === "signEvent" ? `signEvent:${event.kind}` : origKind;
      setPermission(host, mKind, "deny");
    }
    sendResponse?.(void 0);
    return false;
  }
  async function savePrivateKey([index, privKey]) {
    if (privKey.startsWith("nsec")) {
      privKey = nip19_exports.decode(privKey).data;
    }
    let profiles = await get("profiles");
    profiles[index].privKey = bytesToHex2(privKey);
    await storage2.set({ profiles });
    return true;
  }
  async function getNsec(index) {
    let profile = await getProfile(index);
    let nsec = nip19_exports.nsecEncode(hexToBytes2(profile.privKey));
    return nsec;
  }
  async function getNpub(index) {
    let profile = await getProfile(index);
    let pubKey = getPublicKey(hexToBytes2(profile.privKey));
    let npub = nip19_exports.npubEncode(pubKey);
    return npub;
  }
  async function getPrivKey() {
    let profile = await currentProfile();
    return hexToBytes2(profile.privKey);
  }
  async function getPubKey() {
    let pi = await getProfileIndex();
    let profile = await getProfile(pi);
    let privKey = await getPrivKey();
    let pubKey = getPublicKey(privKey);
    return pubKey;
  }
  async function currentProfile() {
    let index = await getProfileIndex();
    let profiles = await get("profiles");
    return profiles[index];
  }
  async function signEvent_(event, host) {
    event = JSON.parse(JSON.stringify(event));
    let sk = await getPrivKey();
    event = finalizeEvent(event, sk);
    saveEvent({
      event,
      metadata: { host, signed_at: Math.round(Date.now() / 1e3) }
    });
    return event;
  }
  async function nip04Encrypt({ pubKey, plainText }) {
    let privKey = await getPrivKey();
    return nip04_exports.encrypt(privKey, pubKey, plainText);
  }
  async function nip04Decrypt({ pubKey, cipherText }) {
    let privKey = await getPrivKey();
    return nip04_exports.decrypt(privKey, pubKey, cipherText);
  }
  async function nip44Encrypt2({ pubKey, plainText }) {
    let privKey = await getPrivKey();
    let conversationKey = nip44_exports.getConversationKey(privKey, pubKey);
    return nip44_exports.encrypt(plainText, conversationKey);
  }
  async function nip44Decrypt2({ pubKey, cipherText }) {
    let privKey = await getPrivKey();
    let conversationKey = nip44_exports.getConversationKey(privKey, pubKey);
    return nip44_exports.decrypt(cipherText, conversationKey);
  }
  async function getRelays() {
    let profile = await currentProfile();
    let relays = profile.relays;
    let relayObj = {};
    relays.forEach((relay) => {
      let { url, read, write } = relay;
      relayObj[url] = { read, write };
    });
    return relayObj;
  }
})();
/*! Bundled license information:

@noble/hashes/esm/utils.js:
@noble/hashes/esm/utils.js:
  (*! noble-hashes - MIT License (c) 2022 Paul Miller (paulmillr.com) *)

@noble/curves/esm/abstract/utils.js:
@noble/curves/esm/abstract/modular.js:
@noble/curves/esm/abstract/curve.js:
@noble/curves/esm/abstract/weierstrass.js:
@noble/curves/esm/_shortw_utils.js:
@noble/curves/esm/secp256k1.js:
  (*! noble-curves - MIT License (c) 2022 Paul Miller (paulmillr.com) *)

@scure/base/lib/esm/index.js:
  (*! scure-base - MIT License (c) 2022 Paul Miller (paulmillr.com) *)

@noble/ciphers/esm/utils.js:
  (*! noble-ciphers - MIT License (c) 2023 Paul Miller (paulmillr.com) *)
*/
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvbm9kZV9tb2R1bGVzL0Bub2JsZS9oYXNoZXMvc3JjL19hc3NlcnQudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvbm9kZV9tb2R1bGVzL0Bub2JsZS9oYXNoZXMvc3JjL2NyeXB0by50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2N1cnZlcy9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvdXRpbHMudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvbm9kZV9tb2R1bGVzL0Bub2JsZS9oYXNoZXMvc3JjL19zaGEyLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY3VydmVzL25vZGVfbW9kdWxlcy9Abm9ibGUvaGFzaGVzL3NyYy9zaGEyNTYudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvc3JjL2Fic3RyYWN0L3V0aWxzLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY3VydmVzL3NyYy9hYnN0cmFjdC9tb2R1bGFyLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY3VydmVzL3NyYy9hYnN0cmFjdC9jdXJ2ZS50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2N1cnZlcy9zcmMvYWJzdHJhY3Qvd2VpZXJzdHJhc3MudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvbm9kZV9tb2R1bGVzL0Bub2JsZS9oYXNoZXMvc3JjL2htYWMudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvc3JjL19zaG9ydHdfdXRpbHMudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jdXJ2ZXMvc3JjL3NlY3AyNTZrMS50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvY3J5cHRvLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvaGFzaGVzL3NyYy91dGlscy50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvX2Fzc2VydC50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvX3NoYTIudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9oYXNoZXMvc3JjL3NoYTI1Ni50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQHNjdXJlL2Jhc2UvbGliL2VzbS9pbmRleC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2NpcGhlcnMvc3JjL19hc3NlcnQudHMiLCAiLi4vLi4vbm9kZV9tb2R1bGVzL0Bub2JsZS9jaXBoZXJzL3NyYy91dGlscy50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2NpcGhlcnMvc3JjL19wb2x5dmFsLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY2lwaGVycy9zcmMvYWVzLnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY2lwaGVycy9zcmMvX3BvbHkxMzA1LnRzIiwgIi4uLy4uL25vZGVfbW9kdWxlcy9Abm9ibGUvY2lwaGVycy9zcmMvX2FyeC50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2NpcGhlcnMvc3JjL2NoYWNoYS50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvaG1hYy50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvQG5vYmxlL2hhc2hlcy9zcmMvaGtkZi50cyIsICIuLi8uLi9ub2RlX21vZHVsZXMvbm9zdHItdG9vbHMvbGliL2VzbS9pbmRleC5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvYXN5bmMtbXV0ZXgvaW5kZXgubWpzIiwgInV0aWxpdGllcy91dGlscy5qcyIsICIuLi8uLi9ub2RlX21vZHVsZXMvaWRiL2J1aWxkL2luZGV4LmpzIiwgInV0aWxpdGllcy9kYi5qcyIsICJiYWNrZ3JvdW5kLmpzIl0sCiAgInNvdXJjZXNDb250ZW50IjogWyJmdW5jdGlvbiBudW1iZXIobjogbnVtYmVyKSB7XG4gIGlmICghTnVtYmVyLmlzU2FmZUludGVnZXIobikgfHwgbiA8IDApIHRocm93IG5ldyBFcnJvcihgV3JvbmcgcG9zaXRpdmUgaW50ZWdlcjogJHtufWApO1xufVxuXG5mdW5jdGlvbiBib29sKGI6IGJvb2xlYW4pIHtcbiAgaWYgKHR5cGVvZiBiICE9PSAnYm9vbGVhbicpIHRocm93IG5ldyBFcnJvcihgRXhwZWN0ZWQgYm9vbGVhbiwgbm90ICR7Yn1gKTtcbn1cblxuZnVuY3Rpb24gYnl0ZXMoYjogVWludDhBcnJheSB8IHVuZGVmaW5lZCwgLi4ubGVuZ3RoczogbnVtYmVyW10pIHtcbiAgaWYgKCEoYiBpbnN0YW5jZW9mIFVpbnQ4QXJyYXkpKSB0aHJvdyBuZXcgRXJyb3IoJ0V4cGVjdGVkIFVpbnQ4QXJyYXknKTtcbiAgaWYgKGxlbmd0aHMubGVuZ3RoID4gMCAmJiAhbGVuZ3Rocy5pbmNsdWRlcyhiLmxlbmd0aCkpXG4gICAgdGhyb3cgbmV3IEVycm9yKGBFeHBlY3RlZCBVaW50OEFycmF5IG9mIGxlbmd0aCAke2xlbmd0aHN9LCBub3Qgb2YgbGVuZ3RoPSR7Yi5sZW5ndGh9YCk7XG59XG5cbnR5cGUgSGFzaCA9IHtcbiAgKGRhdGE6IFVpbnQ4QXJyYXkpOiBVaW50OEFycmF5O1xuICBibG9ja0xlbjogbnVtYmVyO1xuICBvdXRwdXRMZW46IG51bWJlcjtcbiAgY3JlYXRlOiBhbnk7XG59O1xuZnVuY3Rpb24gaGFzaChoYXNoOiBIYXNoKSB7XG4gIGlmICh0eXBlb2YgaGFzaCAhPT0gJ2Z1bmN0aW9uJyB8fCB0eXBlb2YgaGFzaC5jcmVhdGUgIT09ICdmdW5jdGlvbicpXG4gICAgdGhyb3cgbmV3IEVycm9yKCdIYXNoIHNob3VsZCBiZSB3cmFwcGVkIGJ5IHV0aWxzLndyYXBDb25zdHJ1Y3RvcicpO1xuICBudW1iZXIoaGFzaC5vdXRwdXRMZW4pO1xuICBudW1iZXIoaGFzaC5ibG9ja0xlbik7XG59XG5cbmZ1bmN0aW9uIGV4aXN0cyhpbnN0YW5jZTogYW55LCBjaGVja0ZpbmlzaGVkID0gdHJ1ZSkge1xuICBpZiAoaW5zdGFuY2UuZGVzdHJveWVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2ggaW5zdGFuY2UgaGFzIGJlZW4gZGVzdHJveWVkJyk7XG4gIGlmIChjaGVja0ZpbmlzaGVkICYmIGluc3RhbmNlLmZpbmlzaGVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2gjZGlnZXN0KCkgaGFzIGFscmVhZHkgYmVlbiBjYWxsZWQnKTtcbn1cbmZ1bmN0aW9uIG91dHB1dChvdXQ6IGFueSwgaW5zdGFuY2U6IGFueSkge1xuICBieXRlcyhvdXQpO1xuICBjb25zdCBtaW4gPSBpbnN0YW5jZS5vdXRwdXRMZW47XG4gIGlmIChvdXQubGVuZ3RoIDwgbWluKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKGBkaWdlc3RJbnRvKCkgZXhwZWN0cyBvdXRwdXQgYnVmZmVyIG9mIGxlbmd0aCBhdCBsZWFzdCAke21pbn1gKTtcbiAgfVxufVxuXG5leHBvcnQgeyBudW1iZXIsIGJvb2wsIGJ5dGVzLCBoYXNoLCBleGlzdHMsIG91dHB1dCB9O1xuXG5jb25zdCBhc3NlcnQgPSB7IG51bWJlciwgYm9vbCwgYnl0ZXMsIGhhc2gsIGV4aXN0cywgb3V0cHV0IH07XG5leHBvcnQgZGVmYXVsdCBhc3NlcnQ7XG4iLCAiLy8gV2UgdXNlIFdlYkNyeXB0byBha2EgZ2xvYmFsVGhpcy5jcnlwdG8sIHdoaWNoIGV4aXN0cyBpbiBicm93c2VycyBhbmQgbm9kZS5qcyAxNisuXG4vLyBTZWUgdXRpbHMudHMgZm9yIGRldGFpbHMuXG5kZWNsYXJlIGNvbnN0IGdsb2JhbFRoaXM6IFJlY29yZDxzdHJpbmcsIGFueT4gfCB1bmRlZmluZWQ7XG5leHBvcnQgY29uc3QgY3J5cHRvID1cbiAgdHlwZW9mIGdsb2JhbFRoaXMgPT09ICdvYmplY3QnICYmICdjcnlwdG8nIGluIGdsb2JhbFRoaXMgPyBnbG9iYWxUaGlzLmNyeXB0byA6IHVuZGVmaW5lZDtcbiIsICIvKiEgbm9ibGUtaGFzaGVzIC0gTUlUIExpY2Vuc2UgKGMpIDIwMjIgUGF1bCBNaWxsZXIgKHBhdWxtaWxsci5jb20pICovXG5cbi8vIFdlIHVzZSBXZWJDcnlwdG8gYWthIGdsb2JhbFRoaXMuY3J5cHRvLCB3aGljaCBleGlzdHMgaW4gYnJvd3NlcnMgYW5kIG5vZGUuanMgMTYrLlxuLy8gbm9kZS5qcyB2ZXJzaW9ucyBlYXJsaWVyIHRoYW4gdjE5IGRvbid0IGRlY2xhcmUgaXQgaW4gZ2xvYmFsIHNjb3BlLlxuLy8gRm9yIG5vZGUuanMsIHBhY2thZ2UuanNvbiNleHBvcnRzIGZpZWxkIG1hcHBpbmcgcmV3cml0ZXMgaW1wb3J0XG4vLyBmcm9tIGBjcnlwdG9gIHRvIGBjcnlwdG9Ob2RlYCwgd2hpY2ggaW1wb3J0cyBuYXRpdmUgbW9kdWxlLlxuLy8gTWFrZXMgdGhlIHV0aWxzIHVuLWltcG9ydGFibGUgaW4gYnJvd3NlcnMgd2l0aG91dCBhIGJ1bmRsZXIuXG4vLyBPbmNlIG5vZGUuanMgMTggaXMgZGVwcmVjYXRlZCwgd2UgY2FuIGp1c3QgZHJvcCB0aGUgaW1wb3J0LlxuaW1wb3J0IHsgY3J5cHRvIH0gZnJvbSAnQG5vYmxlL2hhc2hlcy9jcnlwdG8nO1xuXG4vLyBwcmV0dGllci1pZ25vcmVcbmV4cG9ydCB0eXBlIFR5cGVkQXJyYXkgPSBJbnQ4QXJyYXkgfCBVaW50OENsYW1wZWRBcnJheSB8IFVpbnQ4QXJyYXkgfFxuICBVaW50MTZBcnJheSB8IEludDE2QXJyYXkgfCBVaW50MzJBcnJheSB8IEludDMyQXJyYXk7XG5cbmNvbnN0IHU4YSA9IChhOiBhbnkpOiBhIGlzIFVpbnQ4QXJyYXkgPT4gYSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXk7XG4vLyBDYXN0IGFycmF5IHRvIGRpZmZlcmVudCB0eXBlXG5leHBvcnQgY29uc3QgdTggPSAoYXJyOiBUeXBlZEFycmF5KSA9PiBuZXcgVWludDhBcnJheShhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgYXJyLmJ5dGVMZW5ndGgpO1xuZXhwb3J0IGNvbnN0IHUzMiA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBVaW50MzJBcnJheShhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgTWF0aC5mbG9vcihhcnIuYnl0ZUxlbmd0aCAvIDQpKTtcblxuLy8gQ2FzdCBhcnJheSB0byB2aWV3XG5leHBvcnQgY29uc3QgY3JlYXRlVmlldyA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBEYXRhVmlldyhhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgYXJyLmJ5dGVMZW5ndGgpO1xuXG4vLyBUaGUgcm90YXRlIHJpZ2h0IChjaXJjdWxhciByaWdodCBzaGlmdCkgb3BlcmF0aW9uIGZvciB1aW50MzJcbmV4cG9ydCBjb25zdCByb3RyID0gKHdvcmQ6IG51bWJlciwgc2hpZnQ6IG51bWJlcikgPT4gKHdvcmQgPDwgKDMyIC0gc2hpZnQpKSB8ICh3b3JkID4+PiBzaGlmdCk7XG5cbi8vIGJpZy1lbmRpYW4gaGFyZHdhcmUgaXMgcmFyZS4gSnVzdCBpbiBjYXNlIHNvbWVvbmUgc3RpbGwgZGVjaWRlcyB0byBydW4gaGFzaGVzOlxuLy8gZWFybHktdGhyb3cgYW4gZXJyb3IgYmVjYXVzZSB3ZSBkb24ndCBzdXBwb3J0IEJFIHlldC5cbmV4cG9ydCBjb25zdCBpc0xFID0gbmV3IFVpbnQ4QXJyYXkobmV3IFVpbnQzMkFycmF5KFsweDExMjIzMzQ0XSkuYnVmZmVyKVswXSA9PT0gMHg0NDtcbmlmICghaXNMRSkgdGhyb3cgbmV3IEVycm9yKCdOb24gbGl0dGxlLWVuZGlhbiBoYXJkd2FyZSBpcyBub3Qgc3VwcG9ydGVkJyk7XG5cbmNvbnN0IGhleGVzID0gLyogQF9fUFVSRV9fICovIEFycmF5LmZyb20oeyBsZW5ndGg6IDI1NiB9LCAoXywgaSkgPT5cbiAgaS50b1N0cmluZygxNikucGFkU3RhcnQoMiwgJzAnKVxuKTtcbi8qKlxuICogQGV4YW1wbGUgYnl0ZXNUb0hleChVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKSkgLy8gJ2NhZmUwMTIzJ1xuICovXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb0hleChieXRlczogVWludDhBcnJheSk6IHN0cmluZyB7XG4gIGlmICghdThhKGJ5dGVzKSkgdGhyb3cgbmV3IEVycm9yKCdVaW50OEFycmF5IGV4cGVjdGVkJyk7XG4gIC8vIHByZS1jYWNoaW5nIGltcHJvdmVzIHRoZSBzcGVlZCA2eFxuICBsZXQgaGV4ID0gJyc7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgYnl0ZXMubGVuZ3RoOyBpKyspIHtcbiAgICBoZXggKz0gaGV4ZXNbYnl0ZXNbaV1dO1xuICB9XG4gIHJldHVybiBoZXg7XG59XG5cbi8qKlxuICogQGV4YW1wbGUgaGV4VG9CeXRlcygnY2FmZTAxMjMnKSAvLyBVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKVxuICovXG5leHBvcnQgZnVuY3Rpb24gaGV4VG9CeXRlcyhoZXg6IHN0cmluZyk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIGhleCAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcignaGV4IHN0cmluZyBleHBlY3RlZCwgZ290ICcgKyB0eXBlb2YgaGV4KTtcbiAgY29uc3QgbGVuID0gaGV4Lmxlbmd0aDtcbiAgaWYgKGxlbiAlIDIpIHRocm93IG5ldyBFcnJvcigncGFkZGVkIGhleCBzdHJpbmcgZXhwZWN0ZWQsIGdvdCB1bnBhZGRlZCBoZXggb2YgbGVuZ3RoICcgKyBsZW4pO1xuICBjb25zdCBhcnJheSA9IG5ldyBVaW50OEFycmF5KGxlbiAvIDIpO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGFycmF5Lmxlbmd0aDsgaSsrKSB7XG4gICAgY29uc3QgaiA9IGkgKiAyO1xuICAgIGNvbnN0IGhleEJ5dGUgPSBoZXguc2xpY2UoaiwgaiArIDIpO1xuICAgIGNvbnN0IGJ5dGUgPSBOdW1iZXIucGFyc2VJbnQoaGV4Qnl0ZSwgMTYpO1xuICAgIGlmIChOdW1iZXIuaXNOYU4oYnl0ZSkgfHwgYnl0ZSA8IDApIHRocm93IG5ldyBFcnJvcignSW52YWxpZCBieXRlIHNlcXVlbmNlJyk7XG4gICAgYXJyYXlbaV0gPSBieXRlO1xuICB9XG4gIHJldHVybiBhcnJheTtcbn1cblxuLy8gVGhlcmUgaXMgbm8gc2V0SW1tZWRpYXRlIGluIGJyb3dzZXIgYW5kIHNldFRpbWVvdXQgaXMgc2xvdy5cbi8vIGNhbGwgb2YgYXN5bmMgZm4gd2lsbCByZXR1cm4gUHJvbWlzZSwgd2hpY2ggd2lsbCBiZSBmdWxsZmlsZWQgb25seSBvblxuLy8gbmV4dCBzY2hlZHVsZXIgcXVldWUgcHJvY2Vzc2luZyBzdGVwIGFuZCB0aGlzIGlzIGV4YWN0bHkgd2hhdCB3ZSBuZWVkLlxuZXhwb3J0IGNvbnN0IG5leHRUaWNrID0gYXN5bmMgKCkgPT4ge307XG5cbi8vIFJldHVybnMgY29udHJvbCB0byB0aHJlYWQgZWFjaCAndGljaycgbXMgdG8gYXZvaWQgYmxvY2tpbmdcbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBhc3luY0xvb3AoaXRlcnM6IG51bWJlciwgdGljazogbnVtYmVyLCBjYjogKGk6IG51bWJlcikgPT4gdm9pZCkge1xuICBsZXQgdHMgPSBEYXRlLm5vdygpO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGl0ZXJzOyBpKyspIHtcbiAgICBjYihpKTtcbiAgICAvLyBEYXRlLm5vdygpIGlzIG5vdCBtb25vdG9uaWMsIHNvIGluIGNhc2UgaWYgY2xvY2sgZ29lcyBiYWNrd2FyZHMgd2UgcmV0dXJuIHJldHVybiBjb250cm9sIHRvb1xuICAgIGNvbnN0IGRpZmYgPSBEYXRlLm5vdygpIC0gdHM7XG4gICAgaWYgKGRpZmYgPj0gMCAmJiBkaWZmIDwgdGljaykgY29udGludWU7XG4gICAgYXdhaXQgbmV4dFRpY2soKTtcbiAgICB0cyArPSBkaWZmO1xuICB9XG59XG5cbi8vIEdsb2JhbCBzeW1ib2xzIGluIGJvdGggYnJvd3NlcnMgYW5kIE5vZGUuanMgc2luY2UgdjExXG4vLyBTZWUgaHR0cHM6Ly9naXRodWIuY29tL21pY3Jvc29mdC9UeXBlU2NyaXB0L2lzc3Vlcy8zMTUzNVxuZGVjbGFyZSBjb25zdCBUZXh0RW5jb2RlcjogYW55O1xuXG4vKipcbiAqIEBleGFtcGxlIHV0ZjhUb0J5dGVzKCdhYmMnKSAvLyBuZXcgVWludDhBcnJheShbOTcsIDk4LCA5OV0pXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiB1dGY4VG9CeXRlcyhzdHI6IHN0cmluZyk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIHN0ciAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcihgdXRmOFRvQnl0ZXMgZXhwZWN0ZWQgc3RyaW5nLCBnb3QgJHt0eXBlb2Ygc3RyfWApO1xuICByZXR1cm4gbmV3IFVpbnQ4QXJyYXkobmV3IFRleHRFbmNvZGVyKCkuZW5jb2RlKHN0cikpOyAvLyBodHRwczovL2J1Z3ppbC5sYS8xNjgxODA5XG59XG5cbmV4cG9ydCB0eXBlIElucHV0ID0gVWludDhBcnJheSB8IHN0cmluZztcbi8qKlxuICogTm9ybWFsaXplcyAobm9uLWhleCkgc3RyaW5nIG9yIFVpbnQ4QXJyYXkgdG8gVWludDhBcnJheS5cbiAqIFdhcm5pbmc6IHdoZW4gVWludDhBcnJheSBpcyBwYXNzZWQsIGl0IHdvdWxkIE5PVCBnZXQgY29waWVkLlxuICogS2VlcCBpbiBtaW5kIGZvciBmdXR1cmUgbXV0YWJsZSBvcGVyYXRpb25zLlxuICovXG5leHBvcnQgZnVuY3Rpb24gdG9CeXRlcyhkYXRhOiBJbnB1dCk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIGRhdGEgPT09ICdzdHJpbmcnKSBkYXRhID0gdXRmOFRvQnl0ZXMoZGF0YSk7XG4gIGlmICghdThhKGRhdGEpKSB0aHJvdyBuZXcgRXJyb3IoYGV4cGVjdGVkIFVpbnQ4QXJyYXksIGdvdCAke3R5cGVvZiBkYXRhfWApO1xuICByZXR1cm4gZGF0YTtcbn1cblxuLyoqXG4gKiBDb3BpZXMgc2V2ZXJhbCBVaW50OEFycmF5cyBpbnRvIG9uZS5cbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIGNvbmNhdEJ5dGVzKC4uLmFycmF5czogVWludDhBcnJheVtdKTogVWludDhBcnJheSB7XG4gIGNvbnN0IHIgPSBuZXcgVWludDhBcnJheShhcnJheXMucmVkdWNlKChzdW0sIGEpID0+IHN1bSArIGEubGVuZ3RoLCAwKSk7XG4gIGxldCBwYWQgPSAwOyAvLyB3YWxrIHRocm91Z2ggZWFjaCBpdGVtLCBlbnN1cmUgdGhleSBoYXZlIHByb3BlciB0eXBlXG4gIGFycmF5cy5mb3JFYWNoKChhKSA9PiB7XG4gICAgaWYgKCF1OGEoYSkpIHRocm93IG5ldyBFcnJvcignVWludDhBcnJheSBleHBlY3RlZCcpO1xuICAgIHIuc2V0KGEsIHBhZCk7XG4gICAgcGFkICs9IGEubGVuZ3RoO1xuICB9KTtcbiAgcmV0dXJuIHI7XG59XG5cbi8vIEZvciBydW50aW1lIGNoZWNrIGlmIGNsYXNzIGltcGxlbWVudHMgaW50ZXJmYWNlXG5leHBvcnQgYWJzdHJhY3QgY2xhc3MgSGFzaDxUIGV4dGVuZHMgSGFzaDxUPj4ge1xuICBhYnN0cmFjdCBibG9ja0xlbjogbnVtYmVyOyAvLyBCeXRlcyBwZXIgYmxvY2tcbiAgYWJzdHJhY3Qgb3V0cHV0TGVuOiBudW1iZXI7IC8vIEJ5dGVzIGluIG91dHB1dFxuICBhYnN0cmFjdCB1cGRhdGUoYnVmOiBJbnB1dCk6IHRoaXM7XG4gIC8vIFdyaXRlcyBkaWdlc3QgaW50byBidWZcbiAgYWJzdHJhY3QgZGlnZXN0SW50byhidWY6IFVpbnQ4QXJyYXkpOiB2b2lkO1xuICBhYnN0cmFjdCBkaWdlc3QoKTogVWludDhBcnJheTtcbiAgLyoqXG4gICAqIFJlc2V0cyBpbnRlcm5hbCBzdGF0ZS4gTWFrZXMgSGFzaCBpbnN0YW5jZSB1bnVzYWJsZS5cbiAgICogUmVzZXQgaXMgaW1wb3NzaWJsZSBmb3Iga2V5ZWQgaGFzaGVzIGlmIGtleSBpcyBjb25zdW1lZCBpbnRvIHN0YXRlLiBJZiBkaWdlc3QgaXMgbm90IGNvbnN1bWVkXG4gICAqIGJ5IHVzZXIsIHRoZXkgd2lsbCBuZWVkIHRvIG1hbnVhbGx5IGNhbGwgYGRlc3Ryb3koKWAgd2hlbiB6ZXJvaW5nIGlzIG5lY2Vzc2FyeS5cbiAgICovXG4gIGFic3RyYWN0IGRlc3Ryb3koKTogdm9pZDtcbiAgLyoqXG4gICAqIENsb25lcyBoYXNoIGluc3RhbmNlLiBVbnNhZmU6IGRvZXNuJ3QgY2hlY2sgd2hldGhlciBgdG9gIGlzIHZhbGlkLiBDYW4gYmUgdXNlZCBhcyBgY2xvbmUoKWBcbiAgICogd2hlbiBubyBvcHRpb25zIGFyZSBwYXNzZWQuXG4gICAqIFJlYXNvbnMgdG8gdXNlIGBfY2xvbmVJbnRvYCBpbnN0ZWFkIG9mIGNsb25lOiAxKSBwZXJmb3JtYW5jZSAyKSByZXVzZSBpbnN0YW5jZSA9PiBhbGwgaW50ZXJuYWxcbiAgICogYnVmZmVycyBhcmUgb3ZlcndyaXR0ZW4gPT4gY2F1c2VzIGJ1ZmZlciBvdmVyd3JpdGUgd2hpY2ggaXMgdXNlZCBmb3IgZGlnZXN0IGluIHNvbWUgY2FzZXMuXG4gICAqIFRoZXJlIGFyZSBubyBndWFyYW50ZWVzIGZvciBjbGVhbi11cCBiZWNhdXNlIGl0J3MgaW1wb3NzaWJsZSBpbiBKUy5cbiAgICovXG4gIGFic3RyYWN0IF9jbG9uZUludG8odG8/OiBUKTogVDtcbiAgLy8gU2FmZSB2ZXJzaW9uIHRoYXQgY2xvbmVzIGludGVybmFsIHN0YXRlXG4gIGNsb25lKCk6IFQge1xuICAgIHJldHVybiB0aGlzLl9jbG9uZUludG8oKTtcbiAgfVxufVxuXG4vKipcbiAqIFhPRjogc3RyZWFtaW5nIEFQSSB0byByZWFkIGRpZ2VzdCBpbiBjaHVua3MuXG4gKiBTYW1lIGFzICdzcXVlZXplJyBpbiBrZWNjYWsvazEyIGFuZCAnc2VlaycgaW4gYmxha2UzLCBidXQgbW9yZSBnZW5lcmljIG5hbWUuXG4gKiBXaGVuIGhhc2ggdXNlZCBpbiBYT0YgbW9kZSBpdCBpcyB1cCB0byB1c2VyIHRvIGNhbGwgJy5kZXN0cm95JyBhZnRlcndhcmRzLCBzaW5jZSB3ZSBjYW5ub3RcbiAqIGRlc3Ryb3kgc3RhdGUsIG5leHQgY2FsbCBjYW4gcmVxdWlyZSBtb3JlIGJ5dGVzLlxuICovXG5leHBvcnQgdHlwZSBIYXNoWE9GPFQgZXh0ZW5kcyBIYXNoPFQ+PiA9IEhhc2g8VD4gJiB7XG4gIHhvZihieXRlczogbnVtYmVyKTogVWludDhBcnJheTsgLy8gUmVhZCAnYnl0ZXMnIGJ5dGVzIGZyb20gZGlnZXN0IHN0cmVhbVxuICB4b2ZJbnRvKGJ1ZjogVWludDhBcnJheSk6IFVpbnQ4QXJyYXk7IC8vIHJlYWQgYnVmLmxlbmd0aCBieXRlcyBmcm9tIGRpZ2VzdCBzdHJlYW0gaW50byBidWZcbn07XG5cbmNvbnN0IHRvU3RyID0ge30udG9TdHJpbmc7XG50eXBlIEVtcHR5T2JqID0ge307XG5leHBvcnQgZnVuY3Rpb24gY2hlY2tPcHRzPFQxIGV4dGVuZHMgRW1wdHlPYmosIFQyIGV4dGVuZHMgRW1wdHlPYmo+KFxuICBkZWZhdWx0czogVDEsXG4gIG9wdHM/OiBUMlxuKTogVDEgJiBUMiB7XG4gIGlmIChvcHRzICE9PSB1bmRlZmluZWQgJiYgdG9TdHIuY2FsbChvcHRzKSAhPT0gJ1tvYmplY3QgT2JqZWN0XScpXG4gICAgdGhyb3cgbmV3IEVycm9yKCdPcHRpb25zIHNob3VsZCBiZSBvYmplY3Qgb3IgdW5kZWZpbmVkJyk7XG4gIGNvbnN0IG1lcmdlZCA9IE9iamVjdC5hc3NpZ24oZGVmYXVsdHMsIG9wdHMpO1xuICByZXR1cm4gbWVyZ2VkIGFzIFQxICYgVDI7XG59XG5cbmV4cG9ydCB0eXBlIENIYXNoID0gUmV0dXJuVHlwZTx0eXBlb2Ygd3JhcENvbnN0cnVjdG9yPjtcblxuZXhwb3J0IGZ1bmN0aW9uIHdyYXBDb25zdHJ1Y3RvcjxUIGV4dGVuZHMgSGFzaDxUPj4oaGFzaENvbnM6ICgpID0+IEhhc2g8VD4pIHtcbiAgY29uc3QgaGFzaEMgPSAobXNnOiBJbnB1dCk6IFVpbnQ4QXJyYXkgPT4gaGFzaENvbnMoKS51cGRhdGUodG9CeXRlcyhtc2cpKS5kaWdlc3QoKTtcbiAgY29uc3QgdG1wID0gaGFzaENvbnMoKTtcbiAgaGFzaEMub3V0cHV0TGVuID0gdG1wLm91dHB1dExlbjtcbiAgaGFzaEMuYmxvY2tMZW4gPSB0bXAuYmxvY2tMZW47XG4gIGhhc2hDLmNyZWF0ZSA9ICgpID0+IGhhc2hDb25zKCk7XG4gIHJldHVybiBoYXNoQztcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIHdyYXBDb25zdHJ1Y3RvcldpdGhPcHRzPEggZXh0ZW5kcyBIYXNoPEg+LCBUIGV4dGVuZHMgT2JqZWN0PihcbiAgaGFzaENvbnM6IChvcHRzPzogVCkgPT4gSGFzaDxIPlxuKSB7XG4gIGNvbnN0IGhhc2hDID0gKG1zZzogSW5wdXQsIG9wdHM/OiBUKTogVWludDhBcnJheSA9PiBoYXNoQ29ucyhvcHRzKS51cGRhdGUodG9CeXRlcyhtc2cpKS5kaWdlc3QoKTtcbiAgY29uc3QgdG1wID0gaGFzaENvbnMoe30gYXMgVCk7XG4gIGhhc2hDLm91dHB1dExlbiA9IHRtcC5vdXRwdXRMZW47XG4gIGhhc2hDLmJsb2NrTGVuID0gdG1wLmJsb2NrTGVuO1xuICBoYXNoQy5jcmVhdGUgPSAob3B0czogVCkgPT4gaGFzaENvbnMob3B0cyk7XG4gIHJldHVybiBoYXNoQztcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIHdyYXBYT0ZDb25zdHJ1Y3RvcldpdGhPcHRzPEggZXh0ZW5kcyBIYXNoWE9GPEg+LCBUIGV4dGVuZHMgT2JqZWN0PihcbiAgaGFzaENvbnM6IChvcHRzPzogVCkgPT4gSGFzaFhPRjxIPlxuKSB7XG4gIGNvbnN0IGhhc2hDID0gKG1zZzogSW5wdXQsIG9wdHM/OiBUKTogVWludDhBcnJheSA9PiBoYXNoQ29ucyhvcHRzKS51cGRhdGUodG9CeXRlcyhtc2cpKS5kaWdlc3QoKTtcbiAgY29uc3QgdG1wID0gaGFzaENvbnMoe30gYXMgVCk7XG4gIGhhc2hDLm91dHB1dExlbiA9IHRtcC5vdXRwdXRMZW47XG4gIGhhc2hDLmJsb2NrTGVuID0gdG1wLmJsb2NrTGVuO1xuICBoYXNoQy5jcmVhdGUgPSAob3B0czogVCkgPT4gaGFzaENvbnMob3B0cyk7XG4gIHJldHVybiBoYXNoQztcbn1cblxuLyoqXG4gKiBTZWN1cmUgUFJORy4gVXNlcyBgY3J5cHRvLmdldFJhbmRvbVZhbHVlc2AsIHdoaWNoIGRlZmVycyB0byBPUy5cbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIHJhbmRvbUJ5dGVzKGJ5dGVzTGVuZ3RoID0gMzIpOiBVaW50OEFycmF5IHtcbiAgaWYgKGNyeXB0byAmJiB0eXBlb2YgY3J5cHRvLmdldFJhbmRvbVZhbHVlcyA9PT0gJ2Z1bmN0aW9uJykge1xuICAgIHJldHVybiBjcnlwdG8uZ2V0UmFuZG9tVmFsdWVzKG5ldyBVaW50OEFycmF5KGJ5dGVzTGVuZ3RoKSk7XG4gIH1cbiAgdGhyb3cgbmV3IEVycm9yKCdjcnlwdG8uZ2V0UmFuZG9tVmFsdWVzIG11c3QgYmUgZGVmaW5lZCcpO1xufVxuIiwgImltcG9ydCB7IGV4aXN0cywgb3V0cHV0IH0gZnJvbSAnLi9fYXNzZXJ0LmpzJztcbmltcG9ydCB7IEhhc2gsIGNyZWF0ZVZpZXcsIElucHV0LCB0b0J5dGVzIH0gZnJvbSAnLi91dGlscy5qcyc7XG5cbi8vIFBvbHlmaWxsIGZvciBTYWZhcmkgMTRcbmZ1bmN0aW9uIHNldEJpZ1VpbnQ2NCh2aWV3OiBEYXRhVmlldywgYnl0ZU9mZnNldDogbnVtYmVyLCB2YWx1ZTogYmlnaW50LCBpc0xFOiBib29sZWFuKTogdm9pZCB7XG4gIGlmICh0eXBlb2Ygdmlldy5zZXRCaWdVaW50NjQgPT09ICdmdW5jdGlvbicpIHJldHVybiB2aWV3LnNldEJpZ1VpbnQ2NChieXRlT2Zmc2V0LCB2YWx1ZSwgaXNMRSk7XG4gIGNvbnN0IF8zMm4gPSBCaWdJbnQoMzIpO1xuICBjb25zdCBfdTMyX21heCA9IEJpZ0ludCgweGZmZmZmZmZmKTtcbiAgY29uc3Qgd2ggPSBOdW1iZXIoKHZhbHVlID4+IF8zMm4pICYgX3UzMl9tYXgpO1xuICBjb25zdCB3bCA9IE51bWJlcih2YWx1ZSAmIF91MzJfbWF4KTtcbiAgY29uc3QgaCA9IGlzTEUgPyA0IDogMDtcbiAgY29uc3QgbCA9IGlzTEUgPyAwIDogNDtcbiAgdmlldy5zZXRVaW50MzIoYnl0ZU9mZnNldCArIGgsIHdoLCBpc0xFKTtcbiAgdmlldy5zZXRVaW50MzIoYnl0ZU9mZnNldCArIGwsIHdsLCBpc0xFKTtcbn1cblxuLy8gQmFzZSBTSEEyIGNsYXNzIChSRkMgNjIzNClcbmV4cG9ydCBhYnN0cmFjdCBjbGFzcyBTSEEyPFQgZXh0ZW5kcyBTSEEyPFQ+PiBleHRlbmRzIEhhc2g8VD4ge1xuICBwcm90ZWN0ZWQgYWJzdHJhY3QgcHJvY2VzcyhidWY6IERhdGFWaWV3LCBvZmZzZXQ6IG51bWJlcik6IHZvaWQ7XG4gIHByb3RlY3RlZCBhYnN0cmFjdCBnZXQoKTogbnVtYmVyW107XG4gIHByb3RlY3RlZCBhYnN0cmFjdCBzZXQoLi4uYXJnczogbnVtYmVyW10pOiB2b2lkO1xuICBhYnN0cmFjdCBkZXN0cm95KCk6IHZvaWQ7XG4gIHByb3RlY3RlZCBhYnN0cmFjdCByb3VuZENsZWFuKCk6IHZvaWQ7XG4gIC8vIEZvciBwYXJ0aWFsIHVwZGF0ZXMgbGVzcyB0aGFuIGJsb2NrIHNpemVcbiAgcHJvdGVjdGVkIGJ1ZmZlcjogVWludDhBcnJheTtcbiAgcHJvdGVjdGVkIHZpZXc6IERhdGFWaWV3O1xuICBwcm90ZWN0ZWQgZmluaXNoZWQgPSBmYWxzZTtcbiAgcHJvdGVjdGVkIGxlbmd0aCA9IDA7XG4gIHByb3RlY3RlZCBwb3MgPSAwO1xuICBwcm90ZWN0ZWQgZGVzdHJveWVkID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3IoXG4gICAgcmVhZG9ubHkgYmxvY2tMZW46IG51bWJlcixcbiAgICBwdWJsaWMgb3V0cHV0TGVuOiBudW1iZXIsXG4gICAgcmVhZG9ubHkgcGFkT2Zmc2V0OiBudW1iZXIsXG4gICAgcmVhZG9ubHkgaXNMRTogYm9vbGVhblxuICApIHtcbiAgICBzdXBlcigpO1xuICAgIHRoaXMuYnVmZmVyID0gbmV3IFVpbnQ4QXJyYXkoYmxvY2tMZW4pO1xuICAgIHRoaXMudmlldyA9IGNyZWF0ZVZpZXcodGhpcy5idWZmZXIpO1xuICB9XG4gIHVwZGF0ZShkYXRhOiBJbnB1dCk6IHRoaXMge1xuICAgIGV4aXN0cyh0aGlzKTtcbiAgICBjb25zdCB7IHZpZXcsIGJ1ZmZlciwgYmxvY2tMZW4gfSA9IHRoaXM7XG4gICAgZGF0YSA9IHRvQnl0ZXMoZGF0YSk7XG4gICAgY29uc3QgbGVuID0gZGF0YS5sZW5ndGg7XG4gICAgZm9yIChsZXQgcG9zID0gMDsgcG9zIDwgbGVuOyApIHtcbiAgICAgIGNvbnN0IHRha2UgPSBNYXRoLm1pbihibG9ja0xlbiAtIHRoaXMucG9zLCBsZW4gLSBwb3MpO1xuICAgICAgLy8gRmFzdCBwYXRoOiB3ZSBoYXZlIGF0IGxlYXN0IG9uZSBibG9jayBpbiBpbnB1dCwgY2FzdCBpdCB0byB2aWV3IGFuZCBwcm9jZXNzXG4gICAgICBpZiAodGFrZSA9PT0gYmxvY2tMZW4pIHtcbiAgICAgICAgY29uc3QgZGF0YVZpZXcgPSBjcmVhdGVWaWV3KGRhdGEpO1xuICAgICAgICBmb3IgKDsgYmxvY2tMZW4gPD0gbGVuIC0gcG9zOyBwb3MgKz0gYmxvY2tMZW4pIHRoaXMucHJvY2VzcyhkYXRhVmlldywgcG9zKTtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9XG4gICAgICBidWZmZXIuc2V0KGRhdGEuc3ViYXJyYXkocG9zLCBwb3MgKyB0YWtlKSwgdGhpcy5wb3MpO1xuICAgICAgdGhpcy5wb3MgKz0gdGFrZTtcbiAgICAgIHBvcyArPSB0YWtlO1xuICAgICAgaWYgKHRoaXMucG9zID09PSBibG9ja0xlbikge1xuICAgICAgICB0aGlzLnByb2Nlc3ModmlldywgMCk7XG4gICAgICAgIHRoaXMucG9zID0gMDtcbiAgICAgIH1cbiAgICB9XG4gICAgdGhpcy5sZW5ndGggKz0gZGF0YS5sZW5ndGg7XG4gICAgdGhpcy5yb3VuZENsZWFuKCk7XG4gICAgcmV0dXJuIHRoaXM7XG4gIH1cbiAgZGlnZXN0SW50byhvdXQ6IFVpbnQ4QXJyYXkpIHtcbiAgICBleGlzdHModGhpcyk7XG4gICAgb3V0cHV0KG91dCwgdGhpcyk7XG4gICAgdGhpcy5maW5pc2hlZCA9IHRydWU7XG4gICAgLy8gUGFkZGluZ1xuICAgIC8vIFdlIGNhbiBhdm9pZCBhbGxvY2F0aW9uIG9mIGJ1ZmZlciBmb3IgcGFkZGluZyBjb21wbGV0ZWx5IGlmIGl0XG4gICAgLy8gd2FzIHByZXZpb3VzbHkgbm90IGFsbG9jYXRlZCBoZXJlLiBCdXQgaXQgd29uJ3QgY2hhbmdlIHBlcmZvcm1hbmNlLlxuICAgIGNvbnN0IHsgYnVmZmVyLCB2aWV3LCBibG9ja0xlbiwgaXNMRSB9ID0gdGhpcztcbiAgICBsZXQgeyBwb3MgfSA9IHRoaXM7XG4gICAgLy8gYXBwZW5kIHRoZSBiaXQgJzEnIHRvIHRoZSBtZXNzYWdlXG4gICAgYnVmZmVyW3BvcysrXSA9IDBiMTAwMDAwMDA7XG4gICAgdGhpcy5idWZmZXIuc3ViYXJyYXkocG9zKS5maWxsKDApO1xuICAgIC8vIHdlIGhhdmUgbGVzcyB0aGFuIHBhZE9mZnNldCBsZWZ0IGluIGJ1ZmZlciwgc28gd2UgY2Fubm90IHB1dCBsZW5ndGggaW4gY3VycmVudCBibG9jaywgbmVlZCBwcm9jZXNzIGl0IGFuZCBwYWQgYWdhaW5cbiAgICBpZiAodGhpcy5wYWRPZmZzZXQgPiBibG9ja0xlbiAtIHBvcykge1xuICAgICAgdGhpcy5wcm9jZXNzKHZpZXcsIDApO1xuICAgICAgcG9zID0gMDtcbiAgICB9XG4gICAgLy8gUGFkIHVudGlsIGZ1bGwgYmxvY2sgYnl0ZSB3aXRoIHplcm9zXG4gICAgZm9yIChsZXQgaSA9IHBvczsgaSA8IGJsb2NrTGVuOyBpKyspIGJ1ZmZlcltpXSA9IDA7XG4gICAgLy8gTm90ZTogc2hhNTEyIHJlcXVpcmVzIGxlbmd0aCB0byBiZSAxMjhiaXQgaW50ZWdlciwgYnV0IGxlbmd0aCBpbiBKUyB3aWxsIG92ZXJmbG93IGJlZm9yZSB0aGF0XG4gICAgLy8gWW91IG5lZWQgdG8gd3JpdGUgYXJvdW5kIDIgZXhhYnl0ZXMgKHU2NF9tYXggLyA4IC8gKDEwMjQqKjYpKSBmb3IgdGhpcyB0byBoYXBwZW4uXG4gICAgLy8gU28gd2UganVzdCB3cml0ZSBsb3dlc3QgNjQgYml0cyBvZiB0aGF0IHZhbHVlLlxuICAgIHNldEJpZ1VpbnQ2NCh2aWV3LCBibG9ja0xlbiAtIDgsIEJpZ0ludCh0aGlzLmxlbmd0aCAqIDgpLCBpc0xFKTtcbiAgICB0aGlzLnByb2Nlc3ModmlldywgMCk7XG4gICAgY29uc3Qgb3ZpZXcgPSBjcmVhdGVWaWV3KG91dCk7XG4gICAgY29uc3QgbGVuID0gdGhpcy5vdXRwdXRMZW47XG4gICAgLy8gTk9URTogd2UgZG8gZGl2aXNpb24gYnkgNCBsYXRlciwgd2hpY2ggc2hvdWxkIGJlIGZ1c2VkIGluIHNpbmdsZSBvcCB3aXRoIG1vZHVsbyBieSBKSVRcbiAgICBpZiAobGVuICUgNCkgdGhyb3cgbmV3IEVycm9yKCdfc2hhMjogb3V0cHV0TGVuIHNob3VsZCBiZSBhbGlnbmVkIHRvIDMyYml0Jyk7XG4gICAgY29uc3Qgb3V0TGVuID0gbGVuIC8gNDtcbiAgICBjb25zdCBzdGF0ZSA9IHRoaXMuZ2V0KCk7XG4gICAgaWYgKG91dExlbiA+IHN0YXRlLmxlbmd0aCkgdGhyb3cgbmV3IEVycm9yKCdfc2hhMjogb3V0cHV0TGVuIGJpZ2dlciB0aGFuIHN0YXRlJyk7XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCBvdXRMZW47IGkrKykgb3ZpZXcuc2V0VWludDMyKDQgKiBpLCBzdGF0ZVtpXSwgaXNMRSk7XG4gIH1cbiAgZGlnZXN0KCkge1xuICAgIGNvbnN0IHsgYnVmZmVyLCBvdXRwdXRMZW4gfSA9IHRoaXM7XG4gICAgdGhpcy5kaWdlc3RJbnRvKGJ1ZmZlcik7XG4gICAgY29uc3QgcmVzID0gYnVmZmVyLnNsaWNlKDAsIG91dHB1dExlbik7XG4gICAgdGhpcy5kZXN0cm95KCk7XG4gICAgcmV0dXJuIHJlcztcbiAgfVxuICBfY2xvbmVJbnRvKHRvPzogVCk6IFQge1xuICAgIHRvIHx8PSBuZXcgKHRoaXMuY29uc3RydWN0b3IgYXMgYW55KSgpIGFzIFQ7XG4gICAgdG8uc2V0KC4uLnRoaXMuZ2V0KCkpO1xuICAgIGNvbnN0IHsgYmxvY2tMZW4sIGJ1ZmZlciwgbGVuZ3RoLCBmaW5pc2hlZCwgZGVzdHJveWVkLCBwb3MgfSA9IHRoaXM7XG4gICAgdG8ubGVuZ3RoID0gbGVuZ3RoO1xuICAgIHRvLnBvcyA9IHBvcztcbiAgICB0by5maW5pc2hlZCA9IGZpbmlzaGVkO1xuICAgIHRvLmRlc3Ryb3llZCA9IGRlc3Ryb3llZDtcbiAgICBpZiAobGVuZ3RoICUgYmxvY2tMZW4pIHRvLmJ1ZmZlci5zZXQoYnVmZmVyKTtcbiAgICByZXR1cm4gdG87XG4gIH1cbn1cbiIsICJpbXBvcnQgeyBTSEEyIH0gZnJvbSAnLi9fc2hhMi5qcyc7XG5pbXBvcnQgeyByb3RyLCB3cmFwQ29uc3RydWN0b3IgfSBmcm9tICcuL3V0aWxzLmpzJztcblxuLy8gU0hBMi0yNTYgbmVlZCB0byB0cnkgMl4xMjggaGFzaGVzIHRvIGV4ZWN1dGUgYmlydGhkYXkgYXR0YWNrLlxuLy8gQlRDIG5ldHdvcmsgaXMgZG9pbmcgMl42NyBoYXNoZXMvc2VjIGFzIHBlciBlYXJseSAyMDIzLlxuXG4vLyBDaG9pY2U6IGEgPyBiIDogY1xuY29uc3QgQ2hpID0gKGE6IG51bWJlciwgYjogbnVtYmVyLCBjOiBudW1iZXIpID0+IChhICYgYikgXiAofmEgJiBjKTtcbi8vIE1ham9yaXR5IGZ1bmN0aW9uLCB0cnVlIGlmIGFueSB0d28gaW5wdXN0IGlzIHRydWVcbmNvbnN0IE1haiA9IChhOiBudW1iZXIsIGI6IG51bWJlciwgYzogbnVtYmVyKSA9PiAoYSAmIGIpIF4gKGEgJiBjKSBeIChiICYgYyk7XG5cbi8vIFJvdW5kIGNvbnN0YW50czpcbi8vIGZpcnN0IDMyIGJpdHMgb2YgdGhlIGZyYWN0aW9uYWwgcGFydHMgb2YgdGhlIGN1YmUgcm9vdHMgb2YgdGhlIGZpcnN0IDY0IHByaW1lcyAyLi4zMTEpXG4vLyBwcmV0dGllci1pZ25vcmVcbmNvbnN0IFNIQTI1Nl9LID0gLyogQF9fUFVSRV9fICovbmV3IFVpbnQzMkFycmF5KFtcbiAgMHg0MjhhMmY5OCwgMHg3MTM3NDQ5MSwgMHhiNWMwZmJjZiwgMHhlOWI1ZGJhNSwgMHgzOTU2YzI1YiwgMHg1OWYxMTFmMSwgMHg5MjNmODJhNCwgMHhhYjFjNWVkNSxcbiAgMHhkODA3YWE5OCwgMHgxMjgzNWIwMSwgMHgyNDMxODViZSwgMHg1NTBjN2RjMywgMHg3MmJlNWQ3NCwgMHg4MGRlYjFmZSwgMHg5YmRjMDZhNywgMHhjMTliZjE3NCxcbiAgMHhlNDliNjljMSwgMHhlZmJlNDc4NiwgMHgwZmMxOWRjNiwgMHgyNDBjYTFjYywgMHgyZGU5MmM2ZiwgMHg0YTc0ODRhYSwgMHg1Y2IwYTlkYywgMHg3NmY5ODhkYSxcbiAgMHg5ODNlNTE1MiwgMHhhODMxYzY2ZCwgMHhiMDAzMjdjOCwgMHhiZjU5N2ZjNywgMHhjNmUwMGJmMywgMHhkNWE3OTE0NywgMHgwNmNhNjM1MSwgMHgxNDI5Mjk2NyxcbiAgMHgyN2I3MGE4NSwgMHgyZTFiMjEzOCwgMHg0ZDJjNmRmYywgMHg1MzM4MGQxMywgMHg2NTBhNzM1NCwgMHg3NjZhMGFiYiwgMHg4MWMyYzkyZSwgMHg5MjcyMmM4NSxcbiAgMHhhMmJmZThhMSwgMHhhODFhNjY0YiwgMHhjMjRiOGI3MCwgMHhjNzZjNTFhMywgMHhkMTkyZTgxOSwgMHhkNjk5MDYyNCwgMHhmNDBlMzU4NSwgMHgxMDZhYTA3MCxcbiAgMHgxOWE0YzExNiwgMHgxZTM3NmMwOCwgMHgyNzQ4Nzc0YywgMHgzNGIwYmNiNSwgMHgzOTFjMGNiMywgMHg0ZWQ4YWE0YSwgMHg1YjljY2E0ZiwgMHg2ODJlNmZmMyxcbiAgMHg3NDhmODJlZSwgMHg3OGE1NjM2ZiwgMHg4NGM4NzgxNCwgMHg4Y2M3MDIwOCwgMHg5MGJlZmZmYSwgMHhhNDUwNmNlYiwgMHhiZWY5YTNmNywgMHhjNjcxNzhmMlxuXSk7XG5cbi8vIEluaXRpYWwgc3RhdGUgKGZpcnN0IDMyIGJpdHMgb2YgdGhlIGZyYWN0aW9uYWwgcGFydHMgb2YgdGhlIHNxdWFyZSByb290cyBvZiB0aGUgZmlyc3QgOCBwcmltZXMgMi4uMTkpOlxuLy8gcHJldHRpZXItaWdub3JlXG5jb25zdCBJViA9IC8qIEBfX1BVUkVfXyAqL25ldyBVaW50MzJBcnJheShbXG4gIDB4NmEwOWU2NjcsIDB4YmI2N2FlODUsIDB4M2M2ZWYzNzIsIDB4YTU0ZmY1M2EsIDB4NTEwZTUyN2YsIDB4OWIwNTY4OGMsIDB4MWY4M2Q5YWIsIDB4NWJlMGNkMTlcbl0pO1xuXG4vLyBUZW1wb3JhcnkgYnVmZmVyLCBub3QgdXNlZCB0byBzdG9yZSBhbnl0aGluZyBiZXR3ZWVuIHJ1bnNcbi8vIE5hbWVkIHRoaXMgd2F5IGJlY2F1c2UgaXQgbWF0Y2hlcyBzcGVjaWZpY2F0aW9uLlxuY29uc3QgU0hBMjU2X1cgPSAvKiBAX19QVVJFX18gKi8gbmV3IFVpbnQzMkFycmF5KDY0KTtcbmNsYXNzIFNIQTI1NiBleHRlbmRzIFNIQTI8U0hBMjU2PiB7XG4gIC8vIFdlIGNhbm5vdCB1c2UgYXJyYXkgaGVyZSBzaW5jZSBhcnJheSBhbGxvd3MgaW5kZXhpbmcgYnkgdmFyaWFibGVcbiAgLy8gd2hpY2ggbWVhbnMgb3B0aW1pemVyL2NvbXBpbGVyIGNhbm5vdCB1c2UgcmVnaXN0ZXJzLlxuICBBID0gSVZbMF0gfCAwO1xuICBCID0gSVZbMV0gfCAwO1xuICBDID0gSVZbMl0gfCAwO1xuICBEID0gSVZbM10gfCAwO1xuICBFID0gSVZbNF0gfCAwO1xuICBGID0gSVZbNV0gfCAwO1xuICBHID0gSVZbNl0gfCAwO1xuICBIID0gSVZbN10gfCAwO1xuXG4gIGNvbnN0cnVjdG9yKCkge1xuICAgIHN1cGVyKDY0LCAzMiwgOCwgZmFsc2UpO1xuICB9XG4gIHByb3RlY3RlZCBnZXQoKTogW251bWJlciwgbnVtYmVyLCBudW1iZXIsIG51bWJlciwgbnVtYmVyLCBudW1iZXIsIG51bWJlciwgbnVtYmVyXSB7XG4gICAgY29uc3QgeyBBLCBCLCBDLCBELCBFLCBGLCBHLCBIIH0gPSB0aGlzO1xuICAgIHJldHVybiBbQSwgQiwgQywgRCwgRSwgRiwgRywgSF07XG4gIH1cbiAgLy8gcHJldHRpZXItaWdub3JlXG4gIHByb3RlY3RlZCBzZXQoXG4gICAgQTogbnVtYmVyLCBCOiBudW1iZXIsIEM6IG51bWJlciwgRDogbnVtYmVyLCBFOiBudW1iZXIsIEY6IG51bWJlciwgRzogbnVtYmVyLCBIOiBudW1iZXJcbiAgKSB7XG4gICAgdGhpcy5BID0gQSB8IDA7XG4gICAgdGhpcy5CID0gQiB8IDA7XG4gICAgdGhpcy5DID0gQyB8IDA7XG4gICAgdGhpcy5EID0gRCB8IDA7XG4gICAgdGhpcy5FID0gRSB8IDA7XG4gICAgdGhpcy5GID0gRiB8IDA7XG4gICAgdGhpcy5HID0gRyB8IDA7XG4gICAgdGhpcy5IID0gSCB8IDA7XG4gIH1cbiAgcHJvdGVjdGVkIHByb2Nlc3ModmlldzogRGF0YVZpZXcsIG9mZnNldDogbnVtYmVyKTogdm9pZCB7XG4gICAgLy8gRXh0ZW5kIHRoZSBmaXJzdCAxNiB3b3JkcyBpbnRvIHRoZSByZW1haW5pbmcgNDggd29yZHMgd1sxNi4uNjNdIG9mIHRoZSBtZXNzYWdlIHNjaGVkdWxlIGFycmF5XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCAxNjsgaSsrLCBvZmZzZXQgKz0gNCkgU0hBMjU2X1dbaV0gPSB2aWV3LmdldFVpbnQzMihvZmZzZXQsIGZhbHNlKTtcbiAgICBmb3IgKGxldCBpID0gMTY7IGkgPCA2NDsgaSsrKSB7XG4gICAgICBjb25zdCBXMTUgPSBTSEEyNTZfV1tpIC0gMTVdO1xuICAgICAgY29uc3QgVzIgPSBTSEEyNTZfV1tpIC0gMl07XG4gICAgICBjb25zdCBzMCA9IHJvdHIoVzE1LCA3KSBeIHJvdHIoVzE1LCAxOCkgXiAoVzE1ID4+PiAzKTtcbiAgICAgIGNvbnN0IHMxID0gcm90cihXMiwgMTcpIF4gcm90cihXMiwgMTkpIF4gKFcyID4+PiAxMCk7XG4gICAgICBTSEEyNTZfV1tpXSA9IChzMSArIFNIQTI1Nl9XW2kgLSA3XSArIHMwICsgU0hBMjU2X1dbaSAtIDE2XSkgfCAwO1xuICAgIH1cbiAgICAvLyBDb21wcmVzc2lvbiBmdW5jdGlvbiBtYWluIGxvb3AsIDY0IHJvdW5kc1xuICAgIGxldCB7IEEsIEIsIEMsIEQsIEUsIEYsIEcsIEggfSA9IHRoaXM7XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCA2NDsgaSsrKSB7XG4gICAgICBjb25zdCBzaWdtYTEgPSByb3RyKEUsIDYpIF4gcm90cihFLCAxMSkgXiByb3RyKEUsIDI1KTtcbiAgICAgIGNvbnN0IFQxID0gKEggKyBzaWdtYTEgKyBDaGkoRSwgRiwgRykgKyBTSEEyNTZfS1tpXSArIFNIQTI1Nl9XW2ldKSB8IDA7XG4gICAgICBjb25zdCBzaWdtYTAgPSByb3RyKEEsIDIpIF4gcm90cihBLCAxMykgXiByb3RyKEEsIDIyKTtcbiAgICAgIGNvbnN0IFQyID0gKHNpZ21hMCArIE1haihBLCBCLCBDKSkgfCAwO1xuICAgICAgSCA9IEc7XG4gICAgICBHID0gRjtcbiAgICAgIEYgPSBFO1xuICAgICAgRSA9IChEICsgVDEpIHwgMDtcbiAgICAgIEQgPSBDO1xuICAgICAgQyA9IEI7XG4gICAgICBCID0gQTtcbiAgICAgIEEgPSAoVDEgKyBUMikgfCAwO1xuICAgIH1cbiAgICAvLyBBZGQgdGhlIGNvbXByZXNzZWQgY2h1bmsgdG8gdGhlIGN1cnJlbnQgaGFzaCB2YWx1ZVxuICAgIEEgPSAoQSArIHRoaXMuQSkgfCAwO1xuICAgIEIgPSAoQiArIHRoaXMuQikgfCAwO1xuICAgIEMgPSAoQyArIHRoaXMuQykgfCAwO1xuICAgIEQgPSAoRCArIHRoaXMuRCkgfCAwO1xuICAgIEUgPSAoRSArIHRoaXMuRSkgfCAwO1xuICAgIEYgPSAoRiArIHRoaXMuRikgfCAwO1xuICAgIEcgPSAoRyArIHRoaXMuRykgfCAwO1xuICAgIEggPSAoSCArIHRoaXMuSCkgfCAwO1xuICAgIHRoaXMuc2V0KEEsIEIsIEMsIEQsIEUsIEYsIEcsIEgpO1xuICB9XG4gIHByb3RlY3RlZCByb3VuZENsZWFuKCkge1xuICAgIFNIQTI1Nl9XLmZpbGwoMCk7XG4gIH1cbiAgZGVzdHJveSgpIHtcbiAgICB0aGlzLnNldCgwLCAwLCAwLCAwLCAwLCAwLCAwLCAwKTtcbiAgICB0aGlzLmJ1ZmZlci5maWxsKDApO1xuICB9XG59XG4vLyBDb25zdGFudHMgZnJvbSBodHRwczovL252bHB1YnMubmlzdC5nb3YvbmlzdHB1YnMvRklQUy9OSVNULkZJUFMuMTgwLTQucGRmXG5jbGFzcyBTSEEyMjQgZXh0ZW5kcyBTSEEyNTYge1xuICBBID0gMHhjMTA1OWVkOCB8IDA7XG4gIEIgPSAweDM2N2NkNTA3IHwgMDtcbiAgQyA9IDB4MzA3MGRkMTcgfCAwO1xuICBEID0gMHhmNzBlNTkzOSB8IDA7XG4gIEUgPSAweGZmYzAwYjMxIHwgMDtcbiAgRiA9IDB4Njg1ODE1MTEgfCAwO1xuICBHID0gMHg2NGY5OGZhNyB8IDA7XG4gIEggPSAweGJlZmE0ZmE0IHwgMDtcbiAgY29uc3RydWN0b3IoKSB7XG4gICAgc3VwZXIoKTtcbiAgICB0aGlzLm91dHB1dExlbiA9IDI4O1xuICB9XG59XG5cbi8qKlxuICogU0hBMi0yNTYgaGFzaCBmdW5jdGlvblxuICogQHBhcmFtIG1lc3NhZ2UgLSBkYXRhIHRoYXQgd291bGQgYmUgaGFzaGVkXG4gKi9cbmV4cG9ydCBjb25zdCBzaGEyNTYgPSAvKiBAX19QVVJFX18gKi8gd3JhcENvbnN0cnVjdG9yKCgpID0+IG5ldyBTSEEyNTYoKSk7XG5leHBvcnQgY29uc3Qgc2hhMjI0ID0gLyogQF9fUFVSRV9fICovIHdyYXBDb25zdHJ1Y3RvcigoKSA9PiBuZXcgU0hBMjI0KCkpO1xuIiwgIi8qISBub2JsZS1jdXJ2ZXMgLSBNSVQgTGljZW5zZSAoYykgMjAyMiBQYXVsIE1pbGxlciAocGF1bG1pbGxyLmNvbSkgKi9cbi8vIDEwMCBsaW5lcyBvZiBjb2RlIGluIHRoZSBmaWxlIGFyZSBkdXBsaWNhdGVkIGZyb20gbm9ibGUtaGFzaGVzICh1dGlscykuXG4vLyBUaGlzIGlzIE9LOiBgYWJzdHJhY3RgIGRpcmVjdG9yeSBkb2VzIG5vdCB1c2Ugbm9ibGUtaGFzaGVzLlxuLy8gVXNlciBtYXkgb3B0LWluIGludG8gdXNpbmcgZGlmZmVyZW50IGhhc2hpbmcgbGlicmFyeS4gVGhpcyB3YXksIG5vYmxlLWhhc2hlc1xuLy8gd29uJ3QgYmUgaW5jbHVkZWQgaW50byB0aGVpciBidW5kbGUuXG5jb25zdCBfMG4gPSBCaWdJbnQoMCk7XG5jb25zdCBfMW4gPSBCaWdJbnQoMSk7XG5jb25zdCBfMm4gPSBCaWdJbnQoMik7XG5jb25zdCB1OGEgPSAoYTogYW55KTogYSBpcyBVaW50OEFycmF5ID0+IGEgaW5zdGFuY2VvZiBVaW50OEFycmF5O1xuZXhwb3J0IHR5cGUgSGV4ID0gVWludDhBcnJheSB8IHN0cmluZzsgLy8gaGV4IHN0cmluZ3MgYXJlIGFjY2VwdGVkIGZvciBzaW1wbGljaXR5XG5leHBvcnQgdHlwZSBQcml2S2V5ID0gSGV4IHwgYmlnaW50OyAvLyBiaWdpbnRzIGFyZSBhY2NlcHRlZCB0byBlYXNlIGxlYXJuaW5nIGN1cnZlXG5leHBvcnQgdHlwZSBDSGFzaCA9IHtcbiAgKG1lc3NhZ2U6IFVpbnQ4QXJyYXkgfCBzdHJpbmcpOiBVaW50OEFycmF5O1xuICBibG9ja0xlbjogbnVtYmVyO1xuICBvdXRwdXRMZW46IG51bWJlcjtcbiAgY3JlYXRlKG9wdHM/OiB7IGRrTGVuPzogbnVtYmVyIH0pOiBhbnk7IC8vIEZvciBzaGFrZVxufTtcbmV4cG9ydCB0eXBlIEZIYXNoID0gKG1lc3NhZ2U6IFVpbnQ4QXJyYXkgfCBzdHJpbmcpID0+IFVpbnQ4QXJyYXk7XG5cbmNvbnN0IGhleGVzID0gLyogQF9fUFVSRV9fICovIEFycmF5LmZyb20oeyBsZW5ndGg6IDI1NiB9LCAoXywgaSkgPT5cbiAgaS50b1N0cmluZygxNikucGFkU3RhcnQoMiwgJzAnKVxuKTtcbi8qKlxuICogQGV4YW1wbGUgYnl0ZXNUb0hleChVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKSkgLy8gJ2NhZmUwMTIzJ1xuICovXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb0hleChieXRlczogVWludDhBcnJheSk6IHN0cmluZyB7XG4gIGlmICghdThhKGJ5dGVzKSkgdGhyb3cgbmV3IEVycm9yKCdVaW50OEFycmF5IGV4cGVjdGVkJyk7XG4gIC8vIHByZS1jYWNoaW5nIGltcHJvdmVzIHRoZSBzcGVlZCA2eFxuICBsZXQgaGV4ID0gJyc7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgYnl0ZXMubGVuZ3RoOyBpKyspIHtcbiAgICBoZXggKz0gaGV4ZXNbYnl0ZXNbaV1dO1xuICB9XG4gIHJldHVybiBoZXg7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBudW1iZXJUb0hleFVucGFkZGVkKG51bTogbnVtYmVyIHwgYmlnaW50KTogc3RyaW5nIHtcbiAgY29uc3QgaGV4ID0gbnVtLnRvU3RyaW5nKDE2KTtcbiAgcmV0dXJuIGhleC5sZW5ndGggJiAxID8gYDAke2hleH1gIDogaGV4O1xufVxuXG5leHBvcnQgZnVuY3Rpb24gaGV4VG9OdW1iZXIoaGV4OiBzdHJpbmcpOiBiaWdpbnQge1xuICBpZiAodHlwZW9mIGhleCAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcignaGV4IHN0cmluZyBleHBlY3RlZCwgZ290ICcgKyB0eXBlb2YgaGV4KTtcbiAgLy8gQmlnIEVuZGlhblxuICByZXR1cm4gQmlnSW50KGhleCA9PT0gJycgPyAnMCcgOiBgMHgke2hleH1gKTtcbn1cblxuLyoqXG4gKiBAZXhhbXBsZSBoZXhUb0J5dGVzKCdjYWZlMDEyMycpIC8vIFVpbnQ4QXJyYXkuZnJvbShbMHhjYSwgMHhmZSwgMHgwMSwgMHgyM10pXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBoZXhUb0J5dGVzKGhleDogc3RyaW5nKTogVWludDhBcnJheSB7XG4gIGlmICh0eXBlb2YgaGV4ICE9PSAnc3RyaW5nJykgdGhyb3cgbmV3IEVycm9yKCdoZXggc3RyaW5nIGV4cGVjdGVkLCBnb3QgJyArIHR5cGVvZiBoZXgpO1xuICBjb25zdCBsZW4gPSBoZXgubGVuZ3RoO1xuICBpZiAobGVuICUgMikgdGhyb3cgbmV3IEVycm9yKCdwYWRkZWQgaGV4IHN0cmluZyBleHBlY3RlZCwgZ290IHVucGFkZGVkIGhleCBvZiBsZW5ndGggJyArIGxlbik7XG4gIGNvbnN0IGFycmF5ID0gbmV3IFVpbnQ4QXJyYXkobGVuIC8gMik7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgYXJyYXkubGVuZ3RoOyBpKyspIHtcbiAgICBjb25zdCBqID0gaSAqIDI7XG4gICAgY29uc3QgaGV4Qnl0ZSA9IGhleC5zbGljZShqLCBqICsgMik7XG4gICAgY29uc3QgYnl0ZSA9IE51bWJlci5wYXJzZUludChoZXhCeXRlLCAxNik7XG4gICAgaWYgKE51bWJlci5pc05hTihieXRlKSB8fCBieXRlIDwgMCkgdGhyb3cgbmV3IEVycm9yKCdJbnZhbGlkIGJ5dGUgc2VxdWVuY2UnKTtcbiAgICBhcnJheVtpXSA9IGJ5dGU7XG4gIH1cbiAgcmV0dXJuIGFycmF5O1xufVxuXG4vLyBCRTogQmlnIEVuZGlhbiwgTEU6IExpdHRsZSBFbmRpYW5cbmV4cG9ydCBmdW5jdGlvbiBieXRlc1RvTnVtYmVyQkUoYnl0ZXM6IFVpbnQ4QXJyYXkpOiBiaWdpbnQge1xuICByZXR1cm4gaGV4VG9OdW1iZXIoYnl0ZXNUb0hleChieXRlcykpO1xufVxuZXhwb3J0IGZ1bmN0aW9uIGJ5dGVzVG9OdW1iZXJMRShieXRlczogVWludDhBcnJheSk6IGJpZ2ludCB7XG4gIGlmICghdThhKGJ5dGVzKSkgdGhyb3cgbmV3IEVycm9yKCdVaW50OEFycmF5IGV4cGVjdGVkJyk7XG4gIHJldHVybiBoZXhUb051bWJlcihieXRlc1RvSGV4KFVpbnQ4QXJyYXkuZnJvbShieXRlcykucmV2ZXJzZSgpKSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBudW1iZXJUb0J5dGVzQkUobjogbnVtYmVyIHwgYmlnaW50LCBsZW46IG51bWJlcik6IFVpbnQ4QXJyYXkge1xuICByZXR1cm4gaGV4VG9CeXRlcyhuLnRvU3RyaW5nKDE2KS5wYWRTdGFydChsZW4gKiAyLCAnMCcpKTtcbn1cbmV4cG9ydCBmdW5jdGlvbiBudW1iZXJUb0J5dGVzTEUobjogbnVtYmVyIHwgYmlnaW50LCBsZW46IG51bWJlcik6IFVpbnQ4QXJyYXkge1xuICByZXR1cm4gbnVtYmVyVG9CeXRlc0JFKG4sIGxlbikucmV2ZXJzZSgpO1xufVxuLy8gVW5wYWRkZWQsIHJhcmVseSB1c2VkXG5leHBvcnQgZnVuY3Rpb24gbnVtYmVyVG9WYXJCeXRlc0JFKG46IG51bWJlciB8IGJpZ2ludCk6IFVpbnQ4QXJyYXkge1xuICByZXR1cm4gaGV4VG9CeXRlcyhudW1iZXJUb0hleFVucGFkZGVkKG4pKTtcbn1cblxuLyoqXG4gKiBUYWtlcyBoZXggc3RyaW5nIG9yIFVpbnQ4QXJyYXksIGNvbnZlcnRzIHRvIFVpbnQ4QXJyYXkuXG4gKiBWYWxpZGF0ZXMgb3V0cHV0IGxlbmd0aC5cbiAqIFdpbGwgdGhyb3cgZXJyb3IgZm9yIG90aGVyIHR5cGVzLlxuICogQHBhcmFtIHRpdGxlIGRlc2NyaXB0aXZlIHRpdGxlIGZvciBhbiBlcnJvciBlLmcuICdwcml2YXRlIGtleSdcbiAqIEBwYXJhbSBoZXggaGV4IHN0cmluZyBvciBVaW50OEFycmF5XG4gKiBAcGFyYW0gZXhwZWN0ZWRMZW5ndGggb3B0aW9uYWwsIHdpbGwgY29tcGFyZSB0byByZXN1bHQgYXJyYXkncyBsZW5ndGhcbiAqIEByZXR1cm5zXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBlbnN1cmVCeXRlcyh0aXRsZTogc3RyaW5nLCBoZXg6IEhleCwgZXhwZWN0ZWRMZW5ndGg/OiBudW1iZXIpOiBVaW50OEFycmF5IHtcbiAgbGV0IHJlczogVWludDhBcnJheTtcbiAgaWYgKHR5cGVvZiBoZXggPT09ICdzdHJpbmcnKSB7XG4gICAgdHJ5IHtcbiAgICAgIHJlcyA9IGhleFRvQnl0ZXMoaGV4KTtcbiAgICB9IGNhdGNoIChlKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYCR7dGl0bGV9IG11c3QgYmUgdmFsaWQgaGV4IHN0cmluZywgZ290IFwiJHtoZXh9XCIuIENhdXNlOiAke2V9YCk7XG4gICAgfVxuICB9IGVsc2UgaWYgKHU4YShoZXgpKSB7XG4gICAgLy8gVWludDhBcnJheS5mcm9tKCkgaW5zdGVhZCBvZiBoYXNoLnNsaWNlKCkgYmVjYXVzZSBub2RlLmpzIEJ1ZmZlclxuICAgIC8vIGlzIGluc3RhbmNlIG9mIFVpbnQ4QXJyYXksIGFuZCBpdHMgc2xpY2UoKSBjcmVhdGVzICoqbXV0YWJsZSoqIGNvcHlcbiAgICByZXMgPSBVaW50OEFycmF5LmZyb20oaGV4KTtcbiAgfSBlbHNlIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoYCR7dGl0bGV9IG11c3QgYmUgaGV4IHN0cmluZyBvciBVaW50OEFycmF5YCk7XG4gIH1cbiAgY29uc3QgbGVuID0gcmVzLmxlbmd0aDtcbiAgaWYgKHR5cGVvZiBleHBlY3RlZExlbmd0aCA9PT0gJ251bWJlcicgJiYgbGVuICE9PSBleHBlY3RlZExlbmd0aClcbiAgICB0aHJvdyBuZXcgRXJyb3IoYCR7dGl0bGV9IGV4cGVjdGVkICR7ZXhwZWN0ZWRMZW5ndGh9IGJ5dGVzLCBnb3QgJHtsZW59YCk7XG4gIHJldHVybiByZXM7XG59XG5cbi8qKlxuICogQ29waWVzIHNldmVyYWwgVWludDhBcnJheXMgaW50byBvbmUuXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBjb25jYXRCeXRlcyguLi5hcnJheXM6IFVpbnQ4QXJyYXlbXSk6IFVpbnQ4QXJyYXkge1xuICBjb25zdCByID0gbmV3IFVpbnQ4QXJyYXkoYXJyYXlzLnJlZHVjZSgoc3VtLCBhKSA9PiBzdW0gKyBhLmxlbmd0aCwgMCkpO1xuICBsZXQgcGFkID0gMDsgLy8gd2FsayB0aHJvdWdoIGVhY2ggaXRlbSwgZW5zdXJlIHRoZXkgaGF2ZSBwcm9wZXIgdHlwZVxuICBhcnJheXMuZm9yRWFjaCgoYSkgPT4ge1xuICAgIGlmICghdThhKGEpKSB0aHJvdyBuZXcgRXJyb3IoJ1VpbnQ4QXJyYXkgZXhwZWN0ZWQnKTtcbiAgICByLnNldChhLCBwYWQpO1xuICAgIHBhZCArPSBhLmxlbmd0aDtcbiAgfSk7XG4gIHJldHVybiByO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gZXF1YWxCeXRlcyhiMTogVWludDhBcnJheSwgYjI6IFVpbnQ4QXJyYXkpIHtcbiAgLy8gV2UgZG9uJ3QgY2FyZSBhYm91dCB0aW1pbmcgYXR0YWNrcyBoZXJlXG4gIGlmIChiMS5sZW5ndGggIT09IGIyLmxlbmd0aCkgcmV0dXJuIGZhbHNlO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGIxLmxlbmd0aDsgaSsrKSBpZiAoYjFbaV0gIT09IGIyW2ldKSByZXR1cm4gZmFsc2U7XG4gIHJldHVybiB0cnVlO1xufVxuXG4vLyBHbG9iYWwgc3ltYm9scyBpbiBib3RoIGJyb3dzZXJzIGFuZCBOb2RlLmpzIHNpbmNlIHYxMVxuLy8gU2VlIGh0dHBzOi8vZ2l0aHViLmNvbS9taWNyb3NvZnQvVHlwZVNjcmlwdC9pc3N1ZXMvMzE1MzVcbmRlY2xhcmUgY29uc3QgVGV4dEVuY29kZXI6IGFueTtcblxuLyoqXG4gKiBAZXhhbXBsZSB1dGY4VG9CeXRlcygnYWJjJykgLy8gbmV3IFVpbnQ4QXJyYXkoWzk3LCA5OCwgOTldKVxuICovXG5leHBvcnQgZnVuY3Rpb24gdXRmOFRvQnl0ZXMoc3RyOiBzdHJpbmcpOiBVaW50OEFycmF5IHtcbiAgaWYgKHR5cGVvZiBzdHIgIT09ICdzdHJpbmcnKSB0aHJvdyBuZXcgRXJyb3IoYHV0ZjhUb0J5dGVzIGV4cGVjdGVkIHN0cmluZywgZ290ICR7dHlwZW9mIHN0cn1gKTtcbiAgcmV0dXJuIG5ldyBVaW50OEFycmF5KG5ldyBUZXh0RW5jb2RlcigpLmVuY29kZShzdHIpKTsgLy8gaHR0cHM6Ly9idWd6aWwubGEvMTY4MTgwOVxufVxuXG4vLyBCaXQgb3BlcmF0aW9uc1xuXG4vKipcbiAqIENhbGN1bGF0ZXMgYW1vdW50IG9mIGJpdHMgaW4gYSBiaWdpbnQuXG4gKiBTYW1lIGFzIGBuLnRvU3RyaW5nKDIpLmxlbmd0aGBcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIGJpdExlbihuOiBiaWdpbnQpIHtcbiAgbGV0IGxlbjtcbiAgZm9yIChsZW4gPSAwOyBuID4gXzBuOyBuID4+PSBfMW4sIGxlbiArPSAxKTtcbiAgcmV0dXJuIGxlbjtcbn1cblxuLyoqXG4gKiBHZXRzIHNpbmdsZSBiaXQgYXQgcG9zaXRpb24uXG4gKiBOT1RFOiBmaXJzdCBiaXQgcG9zaXRpb24gaXMgMCAoc2FtZSBhcyBhcnJheXMpXG4gKiBTYW1lIGFzIGAhIStBcnJheS5mcm9tKG4udG9TdHJpbmcoMikpLnJldmVyc2UoKVtwb3NdYFxuICovXG5leHBvcnQgZnVuY3Rpb24gYml0R2V0KG46IGJpZ2ludCwgcG9zOiBudW1iZXIpIHtcbiAgcmV0dXJuIChuID4+IEJpZ0ludChwb3MpKSAmIF8xbjtcbn1cblxuLyoqXG4gKiBTZXRzIHNpbmdsZSBiaXQgYXQgcG9zaXRpb24uXG4gKi9cbmV4cG9ydCBjb25zdCBiaXRTZXQgPSAobjogYmlnaW50LCBwb3M6IG51bWJlciwgdmFsdWU6IGJvb2xlYW4pID0+IHtcbiAgcmV0dXJuIG4gfCAoKHZhbHVlID8gXzFuIDogXzBuKSA8PCBCaWdJbnQocG9zKSk7XG59O1xuXG4vKipcbiAqIENhbGN1bGF0ZSBtYXNrIGZvciBOIGJpdHMuIE5vdCB1c2luZyAqKiBvcGVyYXRvciB3aXRoIGJpZ2ludHMgYmVjYXVzZSBvZiBvbGQgZW5naW5lcy5cbiAqIFNhbWUgYXMgQmlnSW50KGAwYiR7QXJyYXkoaSkuZmlsbCgnMScpLmpvaW4oJycpfWApXG4gKi9cbmV4cG9ydCBjb25zdCBiaXRNYXNrID0gKG46IG51bWJlcikgPT4gKF8ybiA8PCBCaWdJbnQobiAtIDEpKSAtIF8xbjtcblxuLy8gRFJCR1xuXG5jb25zdCB1OG4gPSAoZGF0YT86IGFueSkgPT4gbmV3IFVpbnQ4QXJyYXkoZGF0YSk7IC8vIGNyZWF0ZXMgVWludDhBcnJheVxuY29uc3QgdThmciA9IChhcnI6IGFueSkgPT4gVWludDhBcnJheS5mcm9tKGFycik7IC8vIGFub3RoZXIgc2hvcnRjdXRcbnR5cGUgUHJlZDxUPiA9ICh2OiBVaW50OEFycmF5KSA9PiBUIHwgdW5kZWZpbmVkO1xuLyoqXG4gKiBNaW5pbWFsIEhNQUMtRFJCRyBmcm9tIE5JU1QgODAwLTkwIGZvciBSRkM2OTc5IHNpZ3MuXG4gKiBAcmV0dXJucyBmdW5jdGlvbiB0aGF0IHdpbGwgY2FsbCBEUkJHIHVudGlsIDJuZCBhcmcgcmV0dXJucyBzb21ldGhpbmcgbWVhbmluZ2Z1bFxuICogQGV4YW1wbGVcbiAqICAgY29uc3QgZHJiZyA9IGNyZWF0ZUhtYWNEUkJHPEtleT4oMzIsIDMyLCBobWFjKTtcbiAqICAgZHJiZyhzZWVkLCBieXRlc1RvS2V5KTsgLy8gYnl0ZXNUb0tleSBtdXN0IHJldHVybiBLZXkgb3IgdW5kZWZpbmVkXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBjcmVhdGVIbWFjRHJiZzxUPihcbiAgaGFzaExlbjogbnVtYmVyLFxuICBxQnl0ZUxlbjogbnVtYmVyLFxuICBobWFjRm46IChrZXk6IFVpbnQ4QXJyYXksIC4uLm1lc3NhZ2VzOiBVaW50OEFycmF5W10pID0+IFVpbnQ4QXJyYXlcbik6IChzZWVkOiBVaW50OEFycmF5LCBwcmVkaWNhdGU6IFByZWQ8VD4pID0+IFQge1xuICBpZiAodHlwZW9mIGhhc2hMZW4gIT09ICdudW1iZXInIHx8IGhhc2hMZW4gPCAyKSB0aHJvdyBuZXcgRXJyb3IoJ2hhc2hMZW4gbXVzdCBiZSBhIG51bWJlcicpO1xuICBpZiAodHlwZW9mIHFCeXRlTGVuICE9PSAnbnVtYmVyJyB8fCBxQnl0ZUxlbiA8IDIpIHRocm93IG5ldyBFcnJvcigncUJ5dGVMZW4gbXVzdCBiZSBhIG51bWJlcicpO1xuICBpZiAodHlwZW9mIGhtYWNGbiAhPT0gJ2Z1bmN0aW9uJykgdGhyb3cgbmV3IEVycm9yKCdobWFjRm4gbXVzdCBiZSBhIGZ1bmN0aW9uJyk7XG4gIC8vIFN0ZXAgQiwgU3RlcCBDOiBzZXQgaGFzaExlbiB0byA4KmNlaWwoaGxlbi84KVxuICBsZXQgdiA9IHU4bihoYXNoTGVuKTsgLy8gTWluaW1hbCBub24tZnVsbC1zcGVjIEhNQUMtRFJCRyBmcm9tIE5JU1QgODAwLTkwIGZvciBSRkM2OTc5IHNpZ3MuXG4gIGxldCBrID0gdThuKGhhc2hMZW4pOyAvLyBTdGVwcyBCIGFuZCBDIG9mIFJGQzY5NzkgMy4yOiBzZXQgaGFzaExlbiwgaW4gb3VyIGNhc2UgYWx3YXlzIHNhbWVcbiAgbGV0IGkgPSAwOyAvLyBJdGVyYXRpb25zIGNvdW50ZXIsIHdpbGwgdGhyb3cgd2hlbiBvdmVyIDEwMDBcbiAgY29uc3QgcmVzZXQgPSAoKSA9PiB7XG4gICAgdi5maWxsKDEpO1xuICAgIGsuZmlsbCgwKTtcbiAgICBpID0gMDtcbiAgfTtcbiAgY29uc3QgaCA9ICguLi5iOiBVaW50OEFycmF5W10pID0+IGhtYWNGbihrLCB2LCAuLi5iKTsgLy8gaG1hYyhrKSh2LCAuLi52YWx1ZXMpXG4gIGNvbnN0IHJlc2VlZCA9IChzZWVkID0gdThuKCkpID0+IHtcbiAgICAvLyBITUFDLURSQkcgcmVzZWVkKCkgZnVuY3Rpb24uIFN0ZXBzIEQtR1xuICAgIGsgPSBoKHU4ZnIoWzB4MDBdKSwgc2VlZCk7IC8vIGsgPSBobWFjKGsgfHwgdiB8fCAweDAwIHx8IHNlZWQpXG4gICAgdiA9IGgoKTsgLy8gdiA9IGhtYWMoayB8fCB2KVxuICAgIGlmIChzZWVkLmxlbmd0aCA9PT0gMCkgcmV0dXJuO1xuICAgIGsgPSBoKHU4ZnIoWzB4MDFdKSwgc2VlZCk7IC8vIGsgPSBobWFjKGsgfHwgdiB8fCAweDAxIHx8IHNlZWQpXG4gICAgdiA9IGgoKTsgLy8gdiA9IGhtYWMoayB8fCB2KVxuICB9O1xuICBjb25zdCBnZW4gPSAoKSA9PiB7XG4gICAgLy8gSE1BQy1EUkJHIGdlbmVyYXRlKCkgZnVuY3Rpb25cbiAgICBpZiAoaSsrID49IDEwMDApIHRocm93IG5ldyBFcnJvcignZHJiZzogdHJpZWQgMTAwMCB2YWx1ZXMnKTtcbiAgICBsZXQgbGVuID0gMDtcbiAgICBjb25zdCBvdXQ6IFVpbnQ4QXJyYXlbXSA9IFtdO1xuICAgIHdoaWxlIChsZW4gPCBxQnl0ZUxlbikge1xuICAgICAgdiA9IGgoKTtcbiAgICAgIGNvbnN0IHNsID0gdi5zbGljZSgpO1xuICAgICAgb3V0LnB1c2goc2wpO1xuICAgICAgbGVuICs9IHYubGVuZ3RoO1xuICAgIH1cbiAgICByZXR1cm4gY29uY2F0Qnl0ZXMoLi4ub3V0KTtcbiAgfTtcbiAgY29uc3QgZ2VuVW50aWwgPSAoc2VlZDogVWludDhBcnJheSwgcHJlZDogUHJlZDxUPik6IFQgPT4ge1xuICAgIHJlc2V0KCk7XG4gICAgcmVzZWVkKHNlZWQpOyAvLyBTdGVwcyBELUdcbiAgICBsZXQgcmVzOiBUIHwgdW5kZWZpbmVkID0gdW5kZWZpbmVkOyAvLyBTdGVwIEg6IGdyaW5kIHVudGlsIGsgaXMgaW4gWzEuLm4tMV1cbiAgICB3aGlsZSAoIShyZXMgPSBwcmVkKGdlbigpKSkpIHJlc2VlZCgpO1xuICAgIHJlc2V0KCk7XG4gICAgcmV0dXJuIHJlcztcbiAgfTtcbiAgcmV0dXJuIGdlblVudGlsO1xufVxuXG4vLyBWYWxpZGF0aW5nIGN1cnZlcyBhbmQgZmllbGRzXG5cbmNvbnN0IHZhbGlkYXRvckZucyA9IHtcbiAgYmlnaW50OiAodmFsOiBhbnkpID0+IHR5cGVvZiB2YWwgPT09ICdiaWdpbnQnLFxuICBmdW5jdGlvbjogKHZhbDogYW55KSA9PiB0eXBlb2YgdmFsID09PSAnZnVuY3Rpb24nLFxuICBib29sZWFuOiAodmFsOiBhbnkpID0+IHR5cGVvZiB2YWwgPT09ICdib29sZWFuJyxcbiAgc3RyaW5nOiAodmFsOiBhbnkpID0+IHR5cGVvZiB2YWwgPT09ICdzdHJpbmcnLFxuICBzdHJpbmdPclVpbnQ4QXJyYXk6ICh2YWw6IGFueSkgPT4gdHlwZW9mIHZhbCA9PT0gJ3N0cmluZycgfHwgdmFsIGluc3RhbmNlb2YgVWludDhBcnJheSxcbiAgaXNTYWZlSW50ZWdlcjogKHZhbDogYW55KSA9PiBOdW1iZXIuaXNTYWZlSW50ZWdlcih2YWwpLFxuICBhcnJheTogKHZhbDogYW55KSA9PiBBcnJheS5pc0FycmF5KHZhbCksXG4gIGZpZWxkOiAodmFsOiBhbnksIG9iamVjdDogYW55KSA9PiAob2JqZWN0IGFzIGFueSkuRnAuaXNWYWxpZCh2YWwpLFxuICBoYXNoOiAodmFsOiBhbnkpID0+IHR5cGVvZiB2YWwgPT09ICdmdW5jdGlvbicgJiYgTnVtYmVyLmlzU2FmZUludGVnZXIodmFsLm91dHB1dExlbiksXG59IGFzIGNvbnN0O1xudHlwZSBWYWxpZGF0b3IgPSBrZXlvZiB0eXBlb2YgdmFsaWRhdG9yRm5zO1xudHlwZSBWYWxNYXA8VCBleHRlbmRzIFJlY29yZDxzdHJpbmcsIGFueT4+ID0geyBbSyBpbiBrZXlvZiBUXT86IFZhbGlkYXRvciB9O1xuLy8gdHlwZSBSZWNvcmQ8SyBleHRlbmRzIHN0cmluZyB8IG51bWJlciB8IHN5bWJvbCwgVD4gPSB7IFtQIGluIEtdOiBUOyB9XG5cbmV4cG9ydCBmdW5jdGlvbiB2YWxpZGF0ZU9iamVjdDxUIGV4dGVuZHMgUmVjb3JkPHN0cmluZywgYW55Pj4oXG4gIG9iamVjdDogVCxcbiAgdmFsaWRhdG9yczogVmFsTWFwPFQ+LFxuICBvcHRWYWxpZGF0b3JzOiBWYWxNYXA8VD4gPSB7fVxuKSB7XG4gIGNvbnN0IGNoZWNrRmllbGQgPSAoZmllbGROYW1lOiBrZXlvZiBULCB0eXBlOiBWYWxpZGF0b3IsIGlzT3B0aW9uYWw6IGJvb2xlYW4pID0+IHtcbiAgICBjb25zdCBjaGVja1ZhbCA9IHZhbGlkYXRvckZuc1t0eXBlXTtcbiAgICBpZiAodHlwZW9mIGNoZWNrVmFsICE9PSAnZnVuY3Rpb24nKVxuICAgICAgdGhyb3cgbmV3IEVycm9yKGBJbnZhbGlkIHZhbGlkYXRvciBcIiR7dHlwZX1cIiwgZXhwZWN0ZWQgZnVuY3Rpb25gKTtcblxuICAgIGNvbnN0IHZhbCA9IG9iamVjdFtmaWVsZE5hbWUgYXMga2V5b2YgdHlwZW9mIG9iamVjdF07XG4gICAgaWYgKGlzT3B0aW9uYWwgJiYgdmFsID09PSB1bmRlZmluZWQpIHJldHVybjtcbiAgICBpZiAoIWNoZWNrVmFsKHZhbCwgb2JqZWN0KSkge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgICBgSW52YWxpZCBwYXJhbSAke1N0cmluZyhmaWVsZE5hbWUpfT0ke3ZhbH0gKCR7dHlwZW9mIHZhbH0pLCBleHBlY3RlZCAke3R5cGV9YFxuICAgICAgKTtcbiAgICB9XG4gIH07XG4gIGZvciAoY29uc3QgW2ZpZWxkTmFtZSwgdHlwZV0gb2YgT2JqZWN0LmVudHJpZXModmFsaWRhdG9ycykpIGNoZWNrRmllbGQoZmllbGROYW1lLCB0eXBlISwgZmFsc2UpO1xuICBmb3IgKGNvbnN0IFtmaWVsZE5hbWUsIHR5cGVdIG9mIE9iamVjdC5lbnRyaWVzKG9wdFZhbGlkYXRvcnMpKSBjaGVja0ZpZWxkKGZpZWxkTmFtZSwgdHlwZSEsIHRydWUpO1xuICByZXR1cm4gb2JqZWN0O1xufVxuLy8gdmFsaWRhdGUgdHlwZSB0ZXN0c1xuLy8gY29uc3QgbzogeyBhOiBudW1iZXI7IGI6IG51bWJlcjsgYzogbnVtYmVyIH0gPSB7IGE6IDEsIGI6IDUsIGM6IDYgfTtcbi8vIGNvbnN0IHowID0gdmFsaWRhdGVPYmplY3QobywgeyBhOiAnaXNTYWZlSW50ZWdlcicgfSwgeyBjOiAnYmlnaW50JyB9KTsgLy8gT2shXG4vLyAvLyBTaG91bGQgZmFpbCB0eXBlLWNoZWNrXG4vLyBjb25zdCB6MSA9IHZhbGlkYXRlT2JqZWN0KG8sIHsgYTogJ3RtcCcgfSwgeyBjOiAnenonIH0pO1xuLy8gY29uc3QgejIgPSB2YWxpZGF0ZU9iamVjdChvLCB7IGE6ICdpc1NhZmVJbnRlZ2VyJyB9LCB7IGM6ICd6eicgfSk7XG4vLyBjb25zdCB6MyA9IHZhbGlkYXRlT2JqZWN0KG8sIHsgdGVzdDogJ2Jvb2xlYW4nLCB6OiAnYnVnJyB9KTtcbi8vIGNvbnN0IHo0ID0gdmFsaWRhdGVPYmplY3QobywgeyBhOiAnYm9vbGVhbicsIHo6ICdidWcnIH0pO1xuIiwgIi8qISBub2JsZS1jdXJ2ZXMgLSBNSVQgTGljZW5zZSAoYykgMjAyMiBQYXVsIE1pbGxlciAocGF1bG1pbGxyLmNvbSkgKi9cbi8vIFV0aWxpdGllcyBmb3IgbW9kdWxhciBhcml0aG1ldGljcyBhbmQgZmluaXRlIGZpZWxkc1xuaW1wb3J0IHtcbiAgYml0TWFzayxcbiAgbnVtYmVyVG9CeXRlc0JFLFxuICBudW1iZXJUb0J5dGVzTEUsXG4gIGJ5dGVzVG9OdW1iZXJCRSxcbiAgYnl0ZXNUb051bWJlckxFLFxuICBlbnN1cmVCeXRlcyxcbiAgdmFsaWRhdGVPYmplY3QsXG59IGZyb20gJy4vdXRpbHMuanMnO1xuLy8gcHJldHRpZXItaWdub3JlXG5jb25zdCBfMG4gPSBCaWdJbnQoMCksIF8xbiA9IEJpZ0ludCgxKSwgXzJuID0gQmlnSW50KDIpLCBfM24gPSBCaWdJbnQoMyk7XG4vLyBwcmV0dGllci1pZ25vcmVcbmNvbnN0IF80biA9IEJpZ0ludCg0KSwgXzVuID0gQmlnSW50KDUpLCBfOG4gPSBCaWdJbnQoOCk7XG4vLyBwcmV0dGllci1pZ25vcmVcbmNvbnN0IF85biA9IEJpZ0ludCg5KSwgXzE2biA9IEJpZ0ludCgxNik7XG5cbi8vIENhbGN1bGF0ZXMgYSBtb2R1bG8gYlxuZXhwb3J0IGZ1bmN0aW9uIG1vZChhOiBiaWdpbnQsIGI6IGJpZ2ludCk6IGJpZ2ludCB7XG4gIGNvbnN0IHJlc3VsdCA9IGEgJSBiO1xuICByZXR1cm4gcmVzdWx0ID49IF8wbiA/IHJlc3VsdCA6IGIgKyByZXN1bHQ7XG59XG4vKipcbiAqIEVmZmljaWVudGx5IHJhaXNlIG51bSB0byBwb3dlciBhbmQgZG8gbW9kdWxhciBkaXZpc2lvbi5cbiAqIFVuc2FmZSBpbiBzb21lIGNvbnRleHRzOiB1c2VzIGxhZGRlciwgc28gY2FuIGV4cG9zZSBiaWdpbnQgYml0cy5cbiAqIEBleGFtcGxlXG4gKiBwb3coMm4sIDZuLCAxMW4pIC8vIDY0biAlIDExbiA9PSA5blxuICovXG4vLyBUT0RPOiB1c2UgZmllbGQgdmVyc2lvbiAmJiByZW1vdmVcbmV4cG9ydCBmdW5jdGlvbiBwb3cobnVtOiBiaWdpbnQsIHBvd2VyOiBiaWdpbnQsIG1vZHVsbzogYmlnaW50KTogYmlnaW50IHtcbiAgaWYgKG1vZHVsbyA8PSBfMG4gfHwgcG93ZXIgPCBfMG4pIHRocm93IG5ldyBFcnJvcignRXhwZWN0ZWQgcG93ZXIvbW9kdWxvID4gMCcpO1xuICBpZiAobW9kdWxvID09PSBfMW4pIHJldHVybiBfMG47XG4gIGxldCByZXMgPSBfMW47XG4gIHdoaWxlIChwb3dlciA+IF8wbikge1xuICAgIGlmIChwb3dlciAmIF8xbikgcmVzID0gKHJlcyAqIG51bSkgJSBtb2R1bG87XG4gICAgbnVtID0gKG51bSAqIG51bSkgJSBtb2R1bG87XG4gICAgcG93ZXIgPj49IF8xbjtcbiAgfVxuICByZXR1cm4gcmVzO1xufVxuXG4vLyBEb2VzIHggXiAoMiBeIHBvd2VyKSBtb2QgcC4gcG93MigzMCwgNCkgPT0gMzAgXiAoMiBeIDQpXG5leHBvcnQgZnVuY3Rpb24gcG93Mih4OiBiaWdpbnQsIHBvd2VyOiBiaWdpbnQsIG1vZHVsbzogYmlnaW50KTogYmlnaW50IHtcbiAgbGV0IHJlcyA9IHg7XG4gIHdoaWxlIChwb3dlci0tID4gXzBuKSB7XG4gICAgcmVzICo9IHJlcztcbiAgICByZXMgJT0gbW9kdWxvO1xuICB9XG4gIHJldHVybiByZXM7XG59XG5cbi8vIEludmVyc2VzIG51bWJlciBvdmVyIG1vZHVsb1xuZXhwb3J0IGZ1bmN0aW9uIGludmVydChudW1iZXI6IGJpZ2ludCwgbW9kdWxvOiBiaWdpbnQpOiBiaWdpbnQge1xuICBpZiAobnVtYmVyID09PSBfMG4gfHwgbW9kdWxvIDw9IF8wbikge1xuICAgIHRocm93IG5ldyBFcnJvcihgaW52ZXJ0OiBleHBlY3RlZCBwb3NpdGl2ZSBpbnRlZ2VycywgZ290IG49JHtudW1iZXJ9IG1vZD0ke21vZHVsb31gKTtcbiAgfVxuICAvLyBFdWNsaWRlYW4gR0NEIGh0dHBzOi8vYnJpbGxpYW50Lm9yZy93aWtpL2V4dGVuZGVkLWV1Y2xpZGVhbi1hbGdvcml0aG0vXG4gIC8vIEZlcm1hdCdzIGxpdHRsZSB0aGVvcmVtIFwiQ1QtbGlrZVwiIHZlcnNpb24gaW52KG4pID0gbl4obS0yKSBtb2QgbSBpcyAzMHggc2xvd2VyLlxuICBsZXQgYSA9IG1vZChudW1iZXIsIG1vZHVsbyk7XG4gIGxldCBiID0gbW9kdWxvO1xuICAvLyBwcmV0dGllci1pZ25vcmVcbiAgbGV0IHggPSBfMG4sIHkgPSBfMW4sIHUgPSBfMW4sIHYgPSBfMG47XG4gIHdoaWxlIChhICE9PSBfMG4pIHtcbiAgICAvLyBKSVQgYXBwbGllcyBvcHRpbWl6YXRpb24gaWYgdGhvc2UgdHdvIGxpbmVzIGZvbGxvdyBlYWNoIG90aGVyXG4gICAgY29uc3QgcSA9IGIgLyBhO1xuICAgIGNvbnN0IHIgPSBiICUgYTtcbiAgICBjb25zdCBtID0geCAtIHUgKiBxO1xuICAgIGNvbnN0IG4gPSB5IC0gdiAqIHE7XG4gICAgLy8gcHJldHRpZXItaWdub3JlXG4gICAgYiA9IGEsIGEgPSByLCB4ID0gdSwgeSA9IHYsIHUgPSBtLCB2ID0gbjtcbiAgfVxuICBjb25zdCBnY2QgPSBiO1xuICBpZiAoZ2NkICE9PSBfMW4pIHRocm93IG5ldyBFcnJvcignaW52ZXJ0OiBkb2VzIG5vdCBleGlzdCcpO1xuICByZXR1cm4gbW9kKHgsIG1vZHVsbyk7XG59XG5cbi8qKlxuICogVG9uZWxsaS1TaGFua3Mgc3F1YXJlIHJvb3Qgc2VhcmNoIGFsZ29yaXRobS5cbiAqIDEuIGh0dHBzOi8vZXByaW50LmlhY3Iub3JnLzIwMTIvNjg1LnBkZiAocGFnZSAxMilcbiAqIDIuIFNxdWFyZSBSb290cyBmcm9tIDE7IDI0LCA1MSwgMTAgdG8gRGFuIFNoYW5rc1xuICogV2lsbCBzdGFydCBhbiBpbmZpbml0ZSBsb29wIGlmIGZpZWxkIG9yZGVyIFAgaXMgbm90IHByaW1lLlxuICogQHBhcmFtIFAgZmllbGQgb3JkZXJcbiAqIEByZXR1cm5zIGZ1bmN0aW9uIHRoYXQgdGFrZXMgZmllbGQgRnAgKGNyZWF0ZWQgZnJvbSBQKSBhbmQgbnVtYmVyIG5cbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIHRvbmVsbGlTaGFua3MoUDogYmlnaW50KSB7XG4gIC8vIExlZ2VuZHJlIGNvbnN0YW50OiB1c2VkIHRvIGNhbGN1bGF0ZSBMZWdlbmRyZSBzeW1ib2wgKGEgfCBwKSxcbiAgLy8gd2hpY2ggZGVub3RlcyB0aGUgdmFsdWUgb2YgYV4oKHAtMSkvMikgKG1vZCBwKS5cbiAgLy8gKGEgfCBwKSBcdTIyNjEgMSAgICBpZiBhIGlzIGEgc3F1YXJlIChtb2QgcClcbiAgLy8gKGEgfCBwKSBcdTIyNjEgLTEgICBpZiBhIGlzIG5vdCBhIHNxdWFyZSAobW9kIHApXG4gIC8vIChhIHwgcCkgXHUyMjYxIDAgICAgaWYgYSBcdTIyNjEgMCAobW9kIHApXG4gIGNvbnN0IGxlZ2VuZHJlQyA9IChQIC0gXzFuKSAvIF8ybjtcblxuICBsZXQgUTogYmlnaW50LCBTOiBudW1iZXIsIFo6IGJpZ2ludDtcbiAgLy8gU3RlcCAxOiBCeSBmYWN0b3Jpbmcgb3V0IHBvd2VycyBvZiAyIGZyb20gcCAtIDEsXG4gIC8vIGZpbmQgcSBhbmQgcyBzdWNoIHRoYXQgcCAtIDEgPSBxKigyXnMpIHdpdGggcSBvZGRcbiAgZm9yIChRID0gUCAtIF8xbiwgUyA9IDA7IFEgJSBfMm4gPT09IF8wbjsgUSAvPSBfMm4sIFMrKyk7XG5cbiAgLy8gU3RlcCAyOiBTZWxlY3QgYSBub24tc3F1YXJlIHogc3VjaCB0aGF0ICh6IHwgcCkgXHUyMjYxIC0xIGFuZCBzZXQgYyBcdTIyNjEgenFcbiAgZm9yIChaID0gXzJuOyBaIDwgUCAmJiBwb3coWiwgbGVnZW5kcmVDLCBQKSAhPT0gUCAtIF8xbjsgWisrKTtcblxuICAvLyBGYXN0LXBhdGhcbiAgaWYgKFMgPT09IDEpIHtcbiAgICBjb25zdCBwMWRpdjQgPSAoUCArIF8xbikgLyBfNG47XG4gICAgcmV0dXJuIGZ1bmN0aW9uIHRvbmVsbGlGYXN0PFQ+KEZwOiBJRmllbGQ8VD4sIG46IFQpIHtcbiAgICAgIGNvbnN0IHJvb3QgPSBGcC5wb3cobiwgcDFkaXY0KTtcbiAgICAgIGlmICghRnAuZXFsKEZwLnNxcihyb290KSwgbikpIHRocm93IG5ldyBFcnJvcignQ2Fubm90IGZpbmQgc3F1YXJlIHJvb3QnKTtcbiAgICAgIHJldHVybiByb290O1xuICAgIH07XG4gIH1cblxuICAvLyBTbG93LXBhdGhcbiAgY29uc3QgUTFkaXYyID0gKFEgKyBfMW4pIC8gXzJuO1xuICByZXR1cm4gZnVuY3Rpb24gdG9uZWxsaVNsb3c8VD4oRnA6IElGaWVsZDxUPiwgbjogVCk6IFQge1xuICAgIC8vIFN0ZXAgMDogQ2hlY2sgdGhhdCBuIGlzIGluZGVlZCBhIHNxdWFyZTogKG4gfCBwKSBzaG91bGQgbm90IGJlIFx1MjI2MSAtMVxuICAgIGlmIChGcC5wb3cobiwgbGVnZW5kcmVDKSA9PT0gRnAubmVnKEZwLk9ORSkpIHRocm93IG5ldyBFcnJvcignQ2Fubm90IGZpbmQgc3F1YXJlIHJvb3QnKTtcbiAgICBsZXQgciA9IFM7XG4gICAgLy8gVE9ETzogd2lsbCBmYWlsIGF0IEZwMi9ldGNcbiAgICBsZXQgZyA9IEZwLnBvdyhGcC5tdWwoRnAuT05FLCBaKSwgUSk7IC8vIHdpbGwgdXBkYXRlIGJvdGggeCBhbmQgYlxuICAgIGxldCB4ID0gRnAucG93KG4sIFExZGl2Mik7IC8vIGZpcnN0IGd1ZXNzIGF0IHRoZSBzcXVhcmUgcm9vdFxuICAgIGxldCBiID0gRnAucG93KG4sIFEpOyAvLyBmaXJzdCBndWVzcyBhdCB0aGUgZnVkZ2UgZmFjdG9yXG5cbiAgICB3aGlsZSAoIUZwLmVxbChiLCBGcC5PTkUpKSB7XG4gICAgICBpZiAoRnAuZXFsKGIsIEZwLlpFUk8pKSByZXR1cm4gRnAuWkVSTzsgLy8gaHR0cHM6Ly9lbi53aWtpcGVkaWEub3JnL3dpa2kvVG9uZWxsaSVFMiU4MCU5M1NoYW5rc19hbGdvcml0aG0gKDQuIElmIHQgPSAwLCByZXR1cm4gciA9IDApXG4gICAgICAvLyBGaW5kIG0gc3VjaCBiXigyXm0pPT0xXG4gICAgICBsZXQgbSA9IDE7XG4gICAgICBmb3IgKGxldCB0MiA9IEZwLnNxcihiKTsgbSA8IHI7IG0rKykge1xuICAgICAgICBpZiAoRnAuZXFsKHQyLCBGcC5PTkUpKSBicmVhaztcbiAgICAgICAgdDIgPSBGcC5zcXIodDIpOyAvLyB0MiAqPSB0MlxuICAgICAgfVxuICAgICAgLy8gTk9URTogci1tLTEgY2FuIGJlIGJpZ2dlciB0aGFuIDMyLCBuZWVkIHRvIGNvbnZlcnQgdG8gYmlnaW50IGJlZm9yZSBzaGlmdCwgb3RoZXJ3aXNlIHRoZXJlIHdpbGwgYmUgb3ZlcmZsb3dcbiAgICAgIGNvbnN0IGdlID0gRnAucG93KGcsIF8xbiA8PCBCaWdJbnQociAtIG0gLSAxKSk7IC8vIGdlID0gMl4oci1tLTEpXG4gICAgICBnID0gRnAuc3FyKGdlKTsgLy8gZyA9IGdlICogZ2VcbiAgICAgIHggPSBGcC5tdWwoeCwgZ2UpOyAvLyB4ICo9IGdlXG4gICAgICBiID0gRnAubXVsKGIsIGcpOyAvLyBiICo9IGdcbiAgICAgIHIgPSBtO1xuICAgIH1cbiAgICByZXR1cm4geDtcbiAgfTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIEZwU3FydChQOiBiaWdpbnQpIHtcbiAgLy8gTk9URTogZGlmZmVyZW50IGFsZ29yaXRobXMgY2FuIGdpdmUgZGlmZmVyZW50IHJvb3RzLCBpdCBpcyB1cCB0byB1c2VyIHRvIGRlY2lkZSB3aGljaCBvbmUgdGhleSB3YW50LlxuICAvLyBGb3IgZXhhbXBsZSB0aGVyZSBpcyBGcFNxcnRPZGQvRnBTcXJ0RXZlbiB0byBjaG9pY2Ugcm9vdCBiYXNlZCBvbiBvZGRuZXNzICh1c2VkIGZvciBoYXNoLXRvLWN1cnZlKS5cblxuICAvLyBQIFx1MjI2MSAzIChtb2QgNClcbiAgLy8gXHUyMjFBbiA9IG5eKChQKzEpLzQpXG4gIGlmIChQICUgXzRuID09PSBfM24pIHtcbiAgICAvLyBOb3QgYWxsIHJvb3RzIHBvc3NpYmxlIVxuICAgIC8vIGNvbnN0IE9SREVSID1cbiAgICAvLyAgIDB4MWEwMTExZWEzOTdmZTY5YTRiMWJhN2I2NDM0YmFjZDc2NDc3NGI4NGYzODUxMmJmNjczMGQyYTBmNmIwZjYyNDFlYWJmZmZlYjE1M2ZmZmZiOWZlZmZmZmZmZmZhYWFibjtcbiAgICAvLyBjb25zdCBOVU0gPSA3MjA1NzU5NDAzNzkyNzgxNm47XG4gICAgY29uc3QgcDFkaXY0ID0gKFAgKyBfMW4pIC8gXzRuO1xuICAgIHJldHVybiBmdW5jdGlvbiBzcXJ0M21vZDQ8VD4oRnA6IElGaWVsZDxUPiwgbjogVCkge1xuICAgICAgY29uc3Qgcm9vdCA9IEZwLnBvdyhuLCBwMWRpdjQpO1xuICAgICAgLy8gVGhyb3cgaWYgcm9vdCoqMiAhPSBuXG4gICAgICBpZiAoIUZwLmVxbChGcC5zcXIocm9vdCksIG4pKSB0aHJvdyBuZXcgRXJyb3IoJ0Nhbm5vdCBmaW5kIHNxdWFyZSByb290Jyk7XG4gICAgICByZXR1cm4gcm9vdDtcbiAgICB9O1xuICB9XG5cbiAgLy8gQXRraW4gYWxnb3JpdGhtIGZvciBxIFx1MjI2MSA1IChtb2QgOCksIGh0dHBzOi8vZXByaW50LmlhY3Iub3JnLzIwMTIvNjg1LnBkZiAocGFnZSAxMClcbiAgaWYgKFAgJSBfOG4gPT09IF81bikge1xuICAgIGNvbnN0IGMxID0gKFAgLSBfNW4pIC8gXzhuO1xuICAgIHJldHVybiBmdW5jdGlvbiBzcXJ0NW1vZDg8VD4oRnA6IElGaWVsZDxUPiwgbjogVCkge1xuICAgICAgY29uc3QgbjIgPSBGcC5tdWwobiwgXzJuKTtcbiAgICAgIGNvbnN0IHYgPSBGcC5wb3cobjIsIGMxKTtcbiAgICAgIGNvbnN0IG52ID0gRnAubXVsKG4sIHYpO1xuICAgICAgY29uc3QgaSA9IEZwLm11bChGcC5tdWwobnYsIF8ybiksIHYpO1xuICAgICAgY29uc3Qgcm9vdCA9IEZwLm11bChudiwgRnAuc3ViKGksIEZwLk9ORSkpO1xuICAgICAgaWYgKCFGcC5lcWwoRnAuc3FyKHJvb3QpLCBuKSkgdGhyb3cgbmV3IEVycm9yKCdDYW5ub3QgZmluZCBzcXVhcmUgcm9vdCcpO1xuICAgICAgcmV0dXJuIHJvb3Q7XG4gICAgfTtcbiAgfVxuXG4gIC8vIFAgXHUyMjYxIDkgKG1vZCAxNilcbiAgaWYgKFAgJSBfMTZuID09PSBfOW4pIHtcbiAgICAvLyBOT1RFOiB0b25lbGxpIGlzIHRvbyBzbG93IGZvciBibHMtRnAyIGNhbGN1bGF0aW9ucyBldmVuIG9uIHN0YXJ0XG4gICAgLy8gTWVhbnMgd2UgY2Fubm90IHVzZSBzcXJ0IGZvciBjb25zdGFudHMgYXQgYWxsIVxuICAgIC8vXG4gICAgLy8gY29uc3QgYzEgPSBGcC5zcXJ0KEZwLm5lZ2F0ZShGcC5PTkUpKTsgLy8gIDEuIGMxID0gc3FydCgtMSkgaW4gRiwgaS5lLiwgKGMxXjIpID09IC0xIGluIEZcbiAgICAvLyBjb25zdCBjMiA9IEZwLnNxcnQoYzEpOyAgICAgICAgICAgICAgICAvLyAgMi4gYzIgPSBzcXJ0KGMxKSBpbiBGLCBpLmUuLCAoYzJeMikgPT0gYzEgaW4gRlxuICAgIC8vIGNvbnN0IGMzID0gRnAuc3FydChGcC5uZWdhdGUoYzEpKTsgICAgIC8vICAzLiBjMyA9IHNxcnQoLWMxKSBpbiBGLCBpLmUuLCAoYzNeMikgPT0gLWMxIGluIEZcbiAgICAvLyBjb25zdCBjNCA9IChQICsgXzduKSAvIF8xNm47ICAgICAgICAgICAvLyAgNC4gYzQgPSAocSArIDcpIC8gMTYgICAgICAgICMgSW50ZWdlciBhcml0aG1ldGljXG4gICAgLy8gc3FydCA9ICh4KSA9PiB7XG4gICAgLy8gICBsZXQgdHYxID0gRnAucG93KHgsIGM0KTsgICAgICAgICAgICAgLy8gIDEuIHR2MSA9IHheYzRcbiAgICAvLyAgIGxldCB0djIgPSBGcC5tdWwoYzEsIHR2MSk7ICAgICAgICAgICAvLyAgMi4gdHYyID0gYzEgKiB0djFcbiAgICAvLyAgIGNvbnN0IHR2MyA9IEZwLm11bChjMiwgdHYxKTsgICAgICAgICAvLyAgMy4gdHYzID0gYzIgKiB0djFcbiAgICAvLyAgIGxldCB0djQgPSBGcC5tdWwoYzMsIHR2MSk7ICAgICAgICAgICAvLyAgNC4gdHY0ID0gYzMgKiB0djFcbiAgICAvLyAgIGNvbnN0IGUxID0gRnAuZXF1YWxzKEZwLnNxdWFyZSh0djIpLCB4KTsgLy8gIDUuICBlMSA9ICh0djJeMikgPT0geFxuICAgIC8vICAgY29uc3QgZTIgPSBGcC5lcXVhbHMoRnAuc3F1YXJlKHR2MyksIHgpOyAvLyAgNi4gIGUyID0gKHR2M14yKSA9PSB4XG4gICAgLy8gICB0djEgPSBGcC5jbW92KHR2MSwgdHYyLCBlMSk7IC8vICA3LiB0djEgPSBDTU9WKHR2MSwgdHYyLCBlMSkgICMgU2VsZWN0IHR2MiBpZiAodHYyXjIpID09IHhcbiAgICAvLyAgIHR2MiA9IEZwLmNtb3YodHY0LCB0djMsIGUyKTsgLy8gIDguIHR2MiA9IENNT1YodHY0LCB0djMsIGUyKSAgIyBTZWxlY3QgdHYzIGlmICh0djNeMikgPT0geFxuICAgIC8vICAgY29uc3QgZTMgPSBGcC5lcXVhbHMoRnAuc3F1YXJlKHR2MiksIHgpOyAvLyAgOS4gIGUzID0gKHR2Ml4yKSA9PSB4XG4gICAgLy8gICByZXR1cm4gRnAuY21vdih0djEsIHR2MiwgZTMpOyAvLyAgMTAuICB6ID0gQ01PVih0djEsIHR2MiwgZTMpICAjIFNlbGVjdCB0aGUgc3FydCBmcm9tIHR2MSBhbmQgdHYyXG4gICAgLy8gfVxuICB9XG5cbiAgLy8gT3RoZXIgY2FzZXM6IFRvbmVsbGktU2hhbmtzIGFsZ29yaXRobVxuICByZXR1cm4gdG9uZWxsaVNoYW5rcyhQKTtcbn1cblxuLy8gTGl0dGxlLWVuZGlhbiBjaGVjayBmb3IgZmlyc3QgTEUgYml0IChsYXN0IEJFIGJpdCk7XG5leHBvcnQgY29uc3QgaXNOZWdhdGl2ZUxFID0gKG51bTogYmlnaW50LCBtb2R1bG86IGJpZ2ludCkgPT4gKG1vZChudW0sIG1vZHVsbykgJiBfMW4pID09PSBfMW47XG5cbi8vIEZpZWxkIGlzIG5vdCBhbHdheXMgb3ZlciBwcmltZTogZm9yIGV4YW1wbGUsIEZwMiBoYXMgT1JERVIocSk9cF5tXG5leHBvcnQgaW50ZXJmYWNlIElGaWVsZDxUPiB7XG4gIE9SREVSOiBiaWdpbnQ7XG4gIEJZVEVTOiBudW1iZXI7XG4gIEJJVFM6IG51bWJlcjtcbiAgTUFTSzogYmlnaW50O1xuICBaRVJPOiBUO1xuICBPTkU6IFQ7XG4gIC8vIDEtYXJnXG4gIGNyZWF0ZTogKG51bTogVCkgPT4gVDtcbiAgaXNWYWxpZDogKG51bTogVCkgPT4gYm9vbGVhbjtcbiAgaXMwOiAobnVtOiBUKSA9PiBib29sZWFuO1xuICBuZWcobnVtOiBUKTogVDtcbiAgaW52KG51bTogVCk6IFQ7XG4gIHNxcnQobnVtOiBUKTogVDtcbiAgc3FyKG51bTogVCk6IFQ7XG4gIC8vIDItYXJnc1xuICBlcWwobGhzOiBULCByaHM6IFQpOiBib29sZWFuO1xuICBhZGQobGhzOiBULCByaHM6IFQpOiBUO1xuICBzdWIobGhzOiBULCByaHM6IFQpOiBUO1xuICBtdWwobGhzOiBULCByaHM6IFQgfCBiaWdpbnQpOiBUO1xuICBwb3cobGhzOiBULCBwb3dlcjogYmlnaW50KTogVDtcbiAgZGl2KGxoczogVCwgcmhzOiBUIHwgYmlnaW50KTogVDtcbiAgLy8gTiBmb3IgTm9uTm9ybWFsaXplZCAoZm9yIG5vdylcbiAgYWRkTihsaHM6IFQsIHJoczogVCk6IFQ7XG4gIHN1Yk4obGhzOiBULCByaHM6IFQpOiBUO1xuICBtdWxOKGxoczogVCwgcmhzOiBUIHwgYmlnaW50KTogVDtcbiAgc3FyTihudW06IFQpOiBUO1xuXG4gIC8vIE9wdGlvbmFsXG4gIC8vIFNob3VsZCBiZSBzYW1lIGFzIHNnbjAgZnVuY3Rpb24gaW5cbiAgLy8gW1JGQzkzODBdKGh0dHBzOi8vd3d3LnJmYy1lZGl0b3Iub3JnL3JmYy9yZmM5MzgwI3NlY3Rpb24tNC4xKS5cbiAgLy8gTk9URTogc2duMCBpcyAnbmVnYXRpdmUgaW4gTEUnLCB3aGljaCBpcyBzYW1lIGFzIG9kZC4gQW5kIG5lZ2F0aXZlIGluIExFIGlzIGtpbmRhIHN0cmFuZ2UgZGVmaW5pdGlvbiBhbnl3YXkuXG4gIGlzT2RkPyhudW06IFQpOiBib29sZWFuOyAvLyBPZGQgaW5zdGVhZCBvZiBldmVuIHNpbmNlIHdlIGhhdmUgaXQgZm9yIEZwMlxuICAvLyBsZWdlbmRyZT8obnVtOiBUKTogVDtcbiAgcG93KGxoczogVCwgcG93ZXI6IGJpZ2ludCk6IFQ7XG4gIGludmVydEJhdGNoOiAobHN0OiBUW10pID0+IFRbXTtcbiAgdG9CeXRlcyhudW06IFQpOiBVaW50OEFycmF5O1xuICBmcm9tQnl0ZXMoYnl0ZXM6IFVpbnQ4QXJyYXkpOiBUO1xuICAvLyBJZiBjIGlzIEZhbHNlLCBDTU9WIHJldHVybnMgYSwgb3RoZXJ3aXNlIGl0IHJldHVybnMgYi5cbiAgY21vdihhOiBULCBiOiBULCBjOiBib29sZWFuKTogVDtcbn1cbi8vIHByZXR0aWVyLWlnbm9yZVxuY29uc3QgRklFTERfRklFTERTID0gW1xuICAnY3JlYXRlJywgJ2lzVmFsaWQnLCAnaXMwJywgJ25lZycsICdpbnYnLCAnc3FydCcsICdzcXInLFxuICAnZXFsJywgJ2FkZCcsICdzdWInLCAnbXVsJywgJ3BvdycsICdkaXYnLFxuICAnYWRkTicsICdzdWJOJywgJ211bE4nLCAnc3FyTidcbl0gYXMgY29uc3Q7XG5leHBvcnQgZnVuY3Rpb24gdmFsaWRhdGVGaWVsZDxUPihmaWVsZDogSUZpZWxkPFQ+KSB7XG4gIGNvbnN0IGluaXRpYWwgPSB7XG4gICAgT1JERVI6ICdiaWdpbnQnLFxuICAgIE1BU0s6ICdiaWdpbnQnLFxuICAgIEJZVEVTOiAnaXNTYWZlSW50ZWdlcicsXG4gICAgQklUUzogJ2lzU2FmZUludGVnZXInLFxuICB9IGFzIFJlY29yZDxzdHJpbmcsIHN0cmluZz47XG4gIGNvbnN0IG9wdHMgPSBGSUVMRF9GSUVMRFMucmVkdWNlKChtYXAsIHZhbDogc3RyaW5nKSA9PiB7XG4gICAgbWFwW3ZhbF0gPSAnZnVuY3Rpb24nO1xuICAgIHJldHVybiBtYXA7XG4gIH0sIGluaXRpYWwpO1xuICByZXR1cm4gdmFsaWRhdGVPYmplY3QoZmllbGQsIG9wdHMpO1xufVxuXG4vLyBHZW5lcmljIGZpZWxkIGZ1bmN0aW9uc1xuXG4vKipcbiAqIFNhbWUgYXMgYHBvd2AgYnV0IGZvciBGcDogbm9uLWNvbnN0YW50LXRpbWUuXG4gKiBVbnNhZmUgaW4gc29tZSBjb250ZXh0czogdXNlcyBsYWRkZXIsIHNvIGNhbiBleHBvc2UgYmlnaW50IGJpdHMuXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBGcFBvdzxUPihmOiBJRmllbGQ8VD4sIG51bTogVCwgcG93ZXI6IGJpZ2ludCk6IFQge1xuICAvLyBTaG91bGQgaGF2ZSBzYW1lIHNwZWVkIGFzIHBvdyBmb3IgYmlnaW50c1xuICAvLyBUT0RPOiBiZW5jaG1hcmshXG4gIGlmIChwb3dlciA8IF8wbikgdGhyb3cgbmV3IEVycm9yKCdFeHBlY3RlZCBwb3dlciA+IDAnKTtcbiAgaWYgKHBvd2VyID09PSBfMG4pIHJldHVybiBmLk9ORTtcbiAgaWYgKHBvd2VyID09PSBfMW4pIHJldHVybiBudW07XG4gIGxldCBwID0gZi5PTkU7XG4gIGxldCBkID0gbnVtO1xuICB3aGlsZSAocG93ZXIgPiBfMG4pIHtcbiAgICBpZiAocG93ZXIgJiBfMW4pIHAgPSBmLm11bChwLCBkKTtcbiAgICBkID0gZi5zcXIoZCk7XG4gICAgcG93ZXIgPj49IF8xbjtcbiAgfVxuICByZXR1cm4gcDtcbn1cblxuLyoqXG4gKiBFZmZpY2llbnRseSBpbnZlcnQgYW4gYXJyYXkgb2YgRmllbGQgZWxlbWVudHMuXG4gKiBgaW52KDApYCB3aWxsIHJldHVybiBgdW5kZWZpbmVkYCBoZXJlOiBtYWtlIHN1cmUgdG8gdGhyb3cgYW4gZXJyb3IuXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBGcEludmVydEJhdGNoPFQ+KGY6IElGaWVsZDxUPiwgbnVtczogVFtdKTogVFtdIHtcbiAgY29uc3QgdG1wID0gbmV3IEFycmF5KG51bXMubGVuZ3RoKTtcbiAgLy8gV2FsayBmcm9tIGZpcnN0IHRvIGxhc3QsIG11bHRpcGx5IHRoZW0gYnkgZWFjaCBvdGhlciBNT0QgcFxuICBjb25zdCBsYXN0TXVsdGlwbGllZCA9IG51bXMucmVkdWNlKChhY2MsIG51bSwgaSkgPT4ge1xuICAgIGlmIChmLmlzMChudW0pKSByZXR1cm4gYWNjO1xuICAgIHRtcFtpXSA9IGFjYztcbiAgICByZXR1cm4gZi5tdWwoYWNjLCBudW0pO1xuICB9LCBmLk9ORSk7XG4gIC8vIEludmVydCBsYXN0IGVsZW1lbnRcbiAgY29uc3QgaW52ZXJ0ZWQgPSBmLmludihsYXN0TXVsdGlwbGllZCk7XG4gIC8vIFdhbGsgZnJvbSBsYXN0IHRvIGZpcnN0LCBtdWx0aXBseSB0aGVtIGJ5IGludmVydGVkIGVhY2ggb3RoZXIgTU9EIHBcbiAgbnVtcy5yZWR1Y2VSaWdodCgoYWNjLCBudW0sIGkpID0+IHtcbiAgICBpZiAoZi5pczAobnVtKSkgcmV0dXJuIGFjYztcbiAgICB0bXBbaV0gPSBmLm11bChhY2MsIHRtcFtpXSk7XG4gICAgcmV0dXJuIGYubXVsKGFjYywgbnVtKTtcbiAgfSwgaW52ZXJ0ZWQpO1xuICByZXR1cm4gdG1wO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gRnBEaXY8VD4oZjogSUZpZWxkPFQ+LCBsaHM6IFQsIHJoczogVCB8IGJpZ2ludCk6IFQge1xuICByZXR1cm4gZi5tdWwobGhzLCB0eXBlb2YgcmhzID09PSAnYmlnaW50JyA/IGludmVydChyaHMsIGYuT1JERVIpIDogZi5pbnYocmhzKSk7XG59XG5cbi8vIFRoaXMgZnVuY3Rpb24gcmV0dXJucyBUcnVlIHdoZW5ldmVyIHRoZSB2YWx1ZSB4IGlzIGEgc3F1YXJlIGluIHRoZSBmaWVsZCBGLlxuZXhwb3J0IGZ1bmN0aW9uIEZwSXNTcXVhcmU8VD4oZjogSUZpZWxkPFQ+KSB7XG4gIGNvbnN0IGxlZ2VuZHJlQ29uc3QgPSAoZi5PUkRFUiAtIF8xbikgLyBfMm47IC8vIEludGVnZXIgYXJpdGhtZXRpY1xuICByZXR1cm4gKHg6IFQpOiBib29sZWFuID0+IHtcbiAgICBjb25zdCBwID0gZi5wb3coeCwgbGVnZW5kcmVDb25zdCk7XG4gICAgcmV0dXJuIGYuZXFsKHAsIGYuWkVSTykgfHwgZi5lcWwocCwgZi5PTkUpO1xuICB9O1xufVxuXG4vLyBDVVJWRS5uIGxlbmd0aHNcbmV4cG9ydCBmdW5jdGlvbiBuTGVuZ3RoKG46IGJpZ2ludCwgbkJpdExlbmd0aD86IG51bWJlcikge1xuICAvLyBCaXQgc2l6ZSwgYnl0ZSBzaXplIG9mIENVUlZFLm5cbiAgY29uc3QgX25CaXRMZW5ndGggPSBuQml0TGVuZ3RoICE9PSB1bmRlZmluZWQgPyBuQml0TGVuZ3RoIDogbi50b1N0cmluZygyKS5sZW5ndGg7XG4gIGNvbnN0IG5CeXRlTGVuZ3RoID0gTWF0aC5jZWlsKF9uQml0TGVuZ3RoIC8gOCk7XG4gIHJldHVybiB7IG5CaXRMZW5ndGg6IF9uQml0TGVuZ3RoLCBuQnl0ZUxlbmd0aCB9O1xufVxuXG50eXBlIEZwRmllbGQgPSBJRmllbGQ8YmlnaW50PiAmIFJlcXVpcmVkPFBpY2s8SUZpZWxkPGJpZ2ludD4sICdpc09kZCc+Pjtcbi8qKlxuICogSW5pdGlhbGl6ZXMgYSBmaW5pdGUgZmllbGQgb3ZlciBwcmltZS4gKipOb24tcHJpbWVzIGFyZSBub3Qgc3VwcG9ydGVkLioqXG4gKiBEbyBub3QgaW5pdCBpbiBsb29wOiBzbG93LiBWZXJ5IGZyYWdpbGU6IGFsd2F5cyBydW4gYSBiZW5jaG1hcmsgb24gYSBjaGFuZ2UuXG4gKiBNYWpvciBwZXJmb3JtYW5jZSBvcHRpbWl6YXRpb25zOlxuICogKiBhKSBkZW5vcm1hbGl6ZWQgb3BlcmF0aW9ucyBsaWtlIG11bE4gaW5zdGVhZCBvZiBtdWxcbiAqICogYikgc2FtZSBvYmplY3Qgc2hhcGU6IG5ldmVyIGFkZCBvciByZW1vdmUga2V5c1xuICogKiBjKSBPYmplY3QuZnJlZXplXG4gKiBAcGFyYW0gT1JERVIgcHJpbWUgcG9zaXRpdmUgYmlnaW50XG4gKiBAcGFyYW0gYml0TGVuIGhvdyBtYW55IGJpdHMgdGhlIGZpZWxkIGNvbnN1bWVzXG4gKiBAcGFyYW0gaXNMRSAoZGVmOiBmYWxzZSkgaWYgZW5jb2RpbmcgLyBkZWNvZGluZyBzaG91bGQgYmUgaW4gbGl0dGxlLWVuZGlhblxuICogQHBhcmFtIHJlZGVmIG9wdGlvbmFsIGZhc3RlciByZWRlZmluaXRpb25zIG9mIHNxcnQgYW5kIG90aGVyIG1ldGhvZHNcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIEZpZWxkKFxuICBPUkRFUjogYmlnaW50LFxuICBiaXRMZW4/OiBudW1iZXIsXG4gIGlzTEUgPSBmYWxzZSxcbiAgcmVkZWY6IFBhcnRpYWw8SUZpZWxkPGJpZ2ludD4+ID0ge31cbik6IFJlYWRvbmx5PEZwRmllbGQ+IHtcbiAgaWYgKE9SREVSIDw9IF8wbikgdGhyb3cgbmV3IEVycm9yKGBFeHBlY3RlZCBGaWVsZCBPUkRFUiA+IDAsIGdvdCAke09SREVSfWApO1xuICBjb25zdCB7IG5CaXRMZW5ndGg6IEJJVFMsIG5CeXRlTGVuZ3RoOiBCWVRFUyB9ID0gbkxlbmd0aChPUkRFUiwgYml0TGVuKTtcbiAgaWYgKEJZVEVTID4gMjA0OCkgdGhyb3cgbmV3IEVycm9yKCdGaWVsZCBsZW5ndGhzIG92ZXIgMjA0OCBieXRlcyBhcmUgbm90IHN1cHBvcnRlZCcpO1xuICBjb25zdCBzcXJ0UCA9IEZwU3FydChPUkRFUik7XG4gIGNvbnN0IGY6IFJlYWRvbmx5PEZwRmllbGQ+ID0gT2JqZWN0LmZyZWV6ZSh7XG4gICAgT1JERVIsXG4gICAgQklUUyxcbiAgICBCWVRFUyxcbiAgICBNQVNLOiBiaXRNYXNrKEJJVFMpLFxuICAgIFpFUk86IF8wbixcbiAgICBPTkU6IF8xbixcbiAgICBjcmVhdGU6IChudW0pID0+IG1vZChudW0sIE9SREVSKSxcbiAgICBpc1ZhbGlkOiAobnVtKSA9PiB7XG4gICAgICBpZiAodHlwZW9mIG51bSAhPT0gJ2JpZ2ludCcpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgSW52YWxpZCBmaWVsZCBlbGVtZW50OiBleHBlY3RlZCBiaWdpbnQsIGdvdCAke3R5cGVvZiBudW19YCk7XG4gICAgICByZXR1cm4gXzBuIDw9IG51bSAmJiBudW0gPCBPUkRFUjsgLy8gMCBpcyB2YWxpZCBlbGVtZW50LCBidXQgaXQncyBub3QgaW52ZXJ0aWJsZVxuICAgIH0sXG4gICAgaXMwOiAobnVtKSA9PiBudW0gPT09IF8wbixcbiAgICBpc09kZDogKG51bSkgPT4gKG51bSAmIF8xbikgPT09IF8xbixcbiAgICBuZWc6IChudW0pID0+IG1vZCgtbnVtLCBPUkRFUiksXG4gICAgZXFsOiAobGhzLCByaHMpID0+IGxocyA9PT0gcmhzLFxuXG4gICAgc3FyOiAobnVtKSA9PiBtb2QobnVtICogbnVtLCBPUkRFUiksXG4gICAgYWRkOiAobGhzLCByaHMpID0+IG1vZChsaHMgKyByaHMsIE9SREVSKSxcbiAgICBzdWI6IChsaHMsIHJocykgPT4gbW9kKGxocyAtIHJocywgT1JERVIpLFxuICAgIG11bDogKGxocywgcmhzKSA9PiBtb2QobGhzICogcmhzLCBPUkRFUiksXG4gICAgcG93OiAobnVtLCBwb3dlcikgPT4gRnBQb3coZiwgbnVtLCBwb3dlciksXG4gICAgZGl2OiAobGhzLCByaHMpID0+IG1vZChsaHMgKiBpbnZlcnQocmhzLCBPUkRFUiksIE9SREVSKSxcblxuICAgIC8vIFNhbWUgYXMgYWJvdmUsIGJ1dCBkb2Vzbid0IG5vcm1hbGl6ZVxuICAgIHNxck46IChudW0pID0+IG51bSAqIG51bSxcbiAgICBhZGROOiAobGhzLCByaHMpID0+IGxocyArIHJocyxcbiAgICBzdWJOOiAobGhzLCByaHMpID0+IGxocyAtIHJocyxcbiAgICBtdWxOOiAobGhzLCByaHMpID0+IGxocyAqIHJocyxcblxuICAgIGludjogKG51bSkgPT4gaW52ZXJ0KG51bSwgT1JERVIpLFxuICAgIHNxcnQ6IHJlZGVmLnNxcnQgfHwgKChuKSA9PiBzcXJ0UChmLCBuKSksXG4gICAgaW52ZXJ0QmF0Y2g6IChsc3QpID0+IEZwSW52ZXJ0QmF0Y2goZiwgbHN0KSxcbiAgICAvLyBUT0RPOiBkbyB3ZSByZWFsbHkgbmVlZCBjb25zdGFudCBjbW92P1xuICAgIC8vIFdlIGRvbid0IGhhdmUgY29uc3QtdGltZSBiaWdpbnRzIGFueXdheSwgc28gcHJvYmFibHkgd2lsbCBiZSBub3QgdmVyeSB1c2VmdWxcbiAgICBjbW92OiAoYSwgYiwgYykgPT4gKGMgPyBiIDogYSksXG4gICAgdG9CeXRlczogKG51bSkgPT4gKGlzTEUgPyBudW1iZXJUb0J5dGVzTEUobnVtLCBCWVRFUykgOiBudW1iZXJUb0J5dGVzQkUobnVtLCBCWVRFUykpLFxuICAgIGZyb21CeXRlczogKGJ5dGVzKSA9PiB7XG4gICAgICBpZiAoYnl0ZXMubGVuZ3RoICE9PSBCWVRFUylcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBGcC5mcm9tQnl0ZXM6IGV4cGVjdGVkICR7QllURVN9LCBnb3QgJHtieXRlcy5sZW5ndGh9YCk7XG4gICAgICByZXR1cm4gaXNMRSA/IGJ5dGVzVG9OdW1iZXJMRShieXRlcykgOiBieXRlc1RvTnVtYmVyQkUoYnl0ZXMpO1xuICAgIH0sXG4gIH0gYXMgRnBGaWVsZCk7XG4gIHJldHVybiBPYmplY3QuZnJlZXplKGYpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gRnBTcXJ0T2RkPFQ+KEZwOiBJRmllbGQ8VD4sIGVsbTogVCkge1xuICBpZiAoIUZwLmlzT2RkKSB0aHJvdyBuZXcgRXJyb3IoYEZpZWxkIGRvZXNuJ3QgaGF2ZSBpc09kZGApO1xuICBjb25zdCByb290ID0gRnAuc3FydChlbG0pO1xuICByZXR1cm4gRnAuaXNPZGQocm9vdCkgPyByb290IDogRnAubmVnKHJvb3QpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gRnBTcXJ0RXZlbjxUPihGcDogSUZpZWxkPFQ+LCBlbG06IFQpIHtcbiAgaWYgKCFGcC5pc09kZCkgdGhyb3cgbmV3IEVycm9yKGBGaWVsZCBkb2Vzbid0IGhhdmUgaXNPZGRgKTtcbiAgY29uc3Qgcm9vdCA9IEZwLnNxcnQoZWxtKTtcbiAgcmV0dXJuIEZwLmlzT2RkKHJvb3QpID8gRnAubmVnKHJvb3QpIDogcm9vdDtcbn1cblxuLyoqXG4gKiBcIkNvbnN0YW50LXRpbWVcIiBwcml2YXRlIGtleSBnZW5lcmF0aW9uIHV0aWxpdHkuXG4gKiBTYW1lIGFzIG1hcEtleVRvRmllbGQsIGJ1dCBhY2NlcHRzIGxlc3MgYnl0ZXMgKDQwIGluc3RlYWQgb2YgNDggZm9yIDMyLWJ5dGUgZmllbGQpLlxuICogV2hpY2ggbWFrZXMgaXQgc2xpZ2h0bHkgbW9yZSBiaWFzZWQsIGxlc3Mgc2VjdXJlLlxuICogQGRlcHJlY2F0ZWQgdXNlIG1hcEtleVRvRmllbGQgaW5zdGVhZFxuICovXG5leHBvcnQgZnVuY3Rpb24gaGFzaFRvUHJpdmF0ZVNjYWxhcihcbiAgaGFzaDogc3RyaW5nIHwgVWludDhBcnJheSxcbiAgZ3JvdXBPcmRlcjogYmlnaW50LFxuICBpc0xFID0gZmFsc2Vcbik6IGJpZ2ludCB7XG4gIGhhc2ggPSBlbnN1cmVCeXRlcygncHJpdmF0ZUhhc2gnLCBoYXNoKTtcbiAgY29uc3QgaGFzaExlbiA9IGhhc2gubGVuZ3RoO1xuICBjb25zdCBtaW5MZW4gPSBuTGVuZ3RoKGdyb3VwT3JkZXIpLm5CeXRlTGVuZ3RoICsgODtcbiAgaWYgKG1pbkxlbiA8IDI0IHx8IGhhc2hMZW4gPCBtaW5MZW4gfHwgaGFzaExlbiA+IDEwMjQpXG4gICAgdGhyb3cgbmV3IEVycm9yKGBoYXNoVG9Qcml2YXRlU2NhbGFyOiBleHBlY3RlZCAke21pbkxlbn0tMTAyNCBieXRlcyBvZiBpbnB1dCwgZ290ICR7aGFzaExlbn1gKTtcbiAgY29uc3QgbnVtID0gaXNMRSA/IGJ5dGVzVG9OdW1iZXJMRShoYXNoKSA6IGJ5dGVzVG9OdW1iZXJCRShoYXNoKTtcbiAgcmV0dXJuIG1vZChudW0sIGdyb3VwT3JkZXIgLSBfMW4pICsgXzFuO1xufVxuXG4vKipcbiAqIFJldHVybnMgdG90YWwgbnVtYmVyIG9mIGJ5dGVzIGNvbnN1bWVkIGJ5IHRoZSBmaWVsZCBlbGVtZW50LlxuICogRm9yIGV4YW1wbGUsIDMyIGJ5dGVzIGZvciB1c3VhbCAyNTYtYml0IHdlaWVyc3RyYXNzIGN1cnZlLlxuICogQHBhcmFtIGZpZWxkT3JkZXIgbnVtYmVyIG9mIGZpZWxkIGVsZW1lbnRzLCB1c3VhbGx5IENVUlZFLm5cbiAqIEByZXR1cm5zIGJ5dGUgbGVuZ3RoIG9mIGZpZWxkXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBnZXRGaWVsZEJ5dGVzTGVuZ3RoKGZpZWxkT3JkZXI6IGJpZ2ludCk6IG51bWJlciB7XG4gIGlmICh0eXBlb2YgZmllbGRPcmRlciAhPT0gJ2JpZ2ludCcpIHRocm93IG5ldyBFcnJvcignZmllbGQgb3JkZXIgbXVzdCBiZSBiaWdpbnQnKTtcbiAgY29uc3QgYml0TGVuZ3RoID0gZmllbGRPcmRlci50b1N0cmluZygyKS5sZW5ndGg7XG4gIHJldHVybiBNYXRoLmNlaWwoYml0TGVuZ3RoIC8gOCk7XG59XG5cbi8qKlxuICogUmV0dXJucyBtaW5pbWFsIGFtb3VudCBvZiBieXRlcyB0aGF0IGNhbiBiZSBzYWZlbHkgcmVkdWNlZFxuICogYnkgZmllbGQgb3JkZXIuXG4gKiBTaG91bGQgYmUgMl4tMTI4IGZvciAxMjgtYml0IGN1cnZlIHN1Y2ggYXMgUDI1Ni5cbiAqIEBwYXJhbSBmaWVsZE9yZGVyIG51bWJlciBvZiBmaWVsZCBlbGVtZW50cywgdXN1YWxseSBDVVJWRS5uXG4gKiBAcmV0dXJucyBieXRlIGxlbmd0aCBvZiB0YXJnZXQgaGFzaFxuICovXG5leHBvcnQgZnVuY3Rpb24gZ2V0TWluSGFzaExlbmd0aChmaWVsZE9yZGVyOiBiaWdpbnQpOiBudW1iZXIge1xuICBjb25zdCBsZW5ndGggPSBnZXRGaWVsZEJ5dGVzTGVuZ3RoKGZpZWxkT3JkZXIpO1xuICByZXR1cm4gbGVuZ3RoICsgTWF0aC5jZWlsKGxlbmd0aCAvIDIpO1xufVxuXG4vKipcbiAqIFwiQ29uc3RhbnQtdGltZVwiIHByaXZhdGUga2V5IGdlbmVyYXRpb24gdXRpbGl0eS5cbiAqIENhbiB0YWtlIChuICsgbi8yKSBvciBtb3JlIGJ5dGVzIG9mIHVuaWZvcm0gaW5wdXQgZS5nLiBmcm9tIENTUFJORyBvciBLREZcbiAqIGFuZCBjb252ZXJ0IHRoZW0gaW50byBwcml2YXRlIHNjYWxhciwgd2l0aCB0aGUgbW9kdWxvIGJpYXMgYmVpbmcgbmVnbGlnaWJsZS5cbiAqIE5lZWRzIGF0IGxlYXN0IDQ4IGJ5dGVzIG9mIGlucHV0IGZvciAzMi1ieXRlIHByaXZhdGUga2V5LlxuICogaHR0cHM6Ly9yZXNlYXJjaC5rdWRlbHNraXNlY3VyaXR5LmNvbS8yMDIwLzA3LzI4L3RoZS1kZWZpbml0aXZlLWd1aWRlLXRvLW1vZHVsby1iaWFzLWFuZC1ob3ctdG8tYXZvaWQtaXQvXG4gKiBGSVBTIDE4Ni01LCBBLjIgaHR0cHM6Ly9jc3JjLm5pc3QuZ292L3B1YmxpY2F0aW9ucy9kZXRhaWwvZmlwcy8xODYvNS9maW5hbFxuICogUkZDIDkzODAsIGh0dHBzOi8vd3d3LnJmYy1lZGl0b3Iub3JnL3JmYy9yZmM5MzgwI3NlY3Rpb24tNVxuICogQHBhcmFtIGhhc2ggaGFzaCBvdXRwdXQgZnJvbSBTSEEzIG9yIGEgc2ltaWxhciBmdW5jdGlvblxuICogQHBhcmFtIGdyb3VwT3JkZXIgc2l6ZSBvZiBzdWJncm91cCAtIChlLmcuIHNlY3AyNTZrMS5DVVJWRS5uKVxuICogQHBhcmFtIGlzTEUgaW50ZXJwcmV0IGhhc2ggYnl0ZXMgYXMgTEUgbnVtXG4gKiBAcmV0dXJucyB2YWxpZCBwcml2YXRlIHNjYWxhclxuICovXG5leHBvcnQgZnVuY3Rpb24gbWFwSGFzaFRvRmllbGQoa2V5OiBVaW50OEFycmF5LCBmaWVsZE9yZGVyOiBiaWdpbnQsIGlzTEUgPSBmYWxzZSk6IFVpbnQ4QXJyYXkge1xuICBjb25zdCBsZW4gPSBrZXkubGVuZ3RoO1xuICBjb25zdCBmaWVsZExlbiA9IGdldEZpZWxkQnl0ZXNMZW5ndGgoZmllbGRPcmRlcik7XG4gIGNvbnN0IG1pbkxlbiA9IGdldE1pbkhhc2hMZW5ndGgoZmllbGRPcmRlcik7XG4gIC8vIE5vIHNtYWxsIG51bWJlcnM6IG5lZWQgdG8gdW5kZXJzdGFuZCBiaWFzIHN0b3J5LiBObyBodWdlIG51bWJlcnM6IGVhc2llciB0byBkZXRlY3QgSlMgdGltaW5ncy5cbiAgaWYgKGxlbiA8IDE2IHx8IGxlbiA8IG1pbkxlbiB8fCBsZW4gPiAxMDI0KVxuICAgIHRocm93IG5ldyBFcnJvcihgZXhwZWN0ZWQgJHttaW5MZW59LTEwMjQgYnl0ZXMgb2YgaW5wdXQsIGdvdCAke2xlbn1gKTtcbiAgY29uc3QgbnVtID0gaXNMRSA/IGJ5dGVzVG9OdW1iZXJCRShrZXkpIDogYnl0ZXNUb051bWJlckxFKGtleSk7XG4gIC8vIGBtb2QoeCwgMTEpYCBjYW4gc29tZXRpbWVzIHByb2R1Y2UgMC4gYG1vZCh4LCAxMCkgKyAxYCBpcyB0aGUgc2FtZSwgYnV0IG5vIDBcbiAgY29uc3QgcmVkdWNlZCA9IG1vZChudW0sIGZpZWxkT3JkZXIgLSBfMW4pICsgXzFuO1xuICByZXR1cm4gaXNMRSA/IG51bWJlclRvQnl0ZXNMRShyZWR1Y2VkLCBmaWVsZExlbikgOiBudW1iZXJUb0J5dGVzQkUocmVkdWNlZCwgZmllbGRMZW4pO1xufVxuIiwgIi8qISBub2JsZS1jdXJ2ZXMgLSBNSVQgTGljZW5zZSAoYykgMjAyMiBQYXVsIE1pbGxlciAocGF1bG1pbGxyLmNvbSkgKi9cbi8vIEFiZWxpYW4gZ3JvdXAgdXRpbGl0aWVzXG5pbXBvcnQgeyBJRmllbGQsIHZhbGlkYXRlRmllbGQsIG5MZW5ndGggfSBmcm9tICcuL21vZHVsYXIuanMnO1xuaW1wb3J0IHsgdmFsaWRhdGVPYmplY3QgfSBmcm9tICcuL3V0aWxzLmpzJztcbmNvbnN0IF8wbiA9IEJpZ0ludCgwKTtcbmNvbnN0IF8xbiA9IEJpZ0ludCgxKTtcblxuZXhwb3J0IHR5cGUgQWZmaW5lUG9pbnQ8VD4gPSB7XG4gIHg6IFQ7XG4gIHk6IFQ7XG59ICYgeyB6PzogbmV2ZXI7IHQ/OiBuZXZlciB9O1xuXG5leHBvcnQgaW50ZXJmYWNlIEdyb3VwPFQgZXh0ZW5kcyBHcm91cDxUPj4ge1xuICBkb3VibGUoKTogVDtcbiAgbmVnYXRlKCk6IFQ7XG4gIGFkZChvdGhlcjogVCk6IFQ7XG4gIHN1YnRyYWN0KG90aGVyOiBUKTogVDtcbiAgZXF1YWxzKG90aGVyOiBUKTogYm9vbGVhbjtcbiAgbXVsdGlwbHkoc2NhbGFyOiBiaWdpbnQpOiBUO1xufVxuXG5leHBvcnQgdHlwZSBHcm91cENvbnN0cnVjdG9yPFQ+ID0ge1xuICBCQVNFOiBUO1xuICBaRVJPOiBUO1xufTtcbmV4cG9ydCB0eXBlIE1hcHBlcjxUPiA9IChpOiBUW10pID0+IFRbXTtcblxuLy8gRWxsaXB0aWMgY3VydmUgbXVsdGlwbGljYXRpb24gb2YgUG9pbnQgYnkgc2NhbGFyLiBGcmFnaWxlLlxuLy8gU2NhbGFycyBzaG91bGQgYWx3YXlzIGJlIGxlc3MgdGhhbiBjdXJ2ZSBvcmRlcjogdGhpcyBzaG91bGQgYmUgY2hlY2tlZCBpbnNpZGUgb2YgYSBjdXJ2ZSBpdHNlbGYuXG4vLyBDcmVhdGVzIHByZWNvbXB1dGF0aW9uIHRhYmxlcyBmb3IgZmFzdCBtdWx0aXBsaWNhdGlvbjpcbi8vIC0gcHJpdmF0ZSBzY2FsYXIgaXMgc3BsaXQgYnkgZml4ZWQgc2l6ZSB3aW5kb3dzIG9mIFcgYml0c1xuLy8gLSBldmVyeSB3aW5kb3cgcG9pbnQgaXMgY29sbGVjdGVkIGZyb20gd2luZG93J3MgdGFibGUgJiBhZGRlZCB0byBhY2N1bXVsYXRvclxuLy8gLSBzaW5jZSB3aW5kb3dzIGFyZSBkaWZmZXJlbnQsIHNhbWUgcG9pbnQgaW5zaWRlIHRhYmxlcyB3b24ndCBiZSBhY2Nlc3NlZCBtb3JlIHRoYW4gb25jZSBwZXIgY2FsY1xuLy8gLSBlYWNoIG11bHRpcGxpY2F0aW9uIGlzICdNYXRoLmNlaWwoQ1VSVkVfT1JERVIgLyBcdUQ4MzVcdURDNEEpICsgMScgcG9pbnQgYWRkaXRpb25zIChmaXhlZCBmb3IgYW55IHNjYWxhcilcbi8vIC0gKzEgd2luZG93IGlzIG5lY2Nlc3NhcnkgZm9yIHdOQUZcbi8vIC0gd05BRiByZWR1Y2VzIHRhYmxlIHNpemU6IDJ4IGxlc3MgbWVtb3J5ICsgMnggZmFzdGVyIGdlbmVyYXRpb24sIGJ1dCAxMCUgc2xvd2VyIG11bHRpcGxpY2F0aW9uXG4vLyBUT0RPOiBSZXNlYXJjaCByZXR1cm5pbmcgMmQgSlMgYXJyYXkgb2Ygd2luZG93cywgaW5zdGVhZCBvZiBhIHNpbmdsZSB3aW5kb3cuIFRoaXMgd291bGQgYWxsb3dcbi8vIHdpbmRvd3MgdG8gYmUgaW4gZGlmZmVyZW50IG1lbW9yeSBsb2NhdGlvbnNcbmV4cG9ydCBmdW5jdGlvbiB3TkFGPFQgZXh0ZW5kcyBHcm91cDxUPj4oYzogR3JvdXBDb25zdHJ1Y3RvcjxUPiwgYml0czogbnVtYmVyKSB7XG4gIGNvbnN0IGNvbnN0VGltZU5lZ2F0ZSA9IChjb25kaXRpb246IGJvb2xlYW4sIGl0ZW06IFQpOiBUID0+IHtcbiAgICBjb25zdCBuZWcgPSBpdGVtLm5lZ2F0ZSgpO1xuICAgIHJldHVybiBjb25kaXRpb24gPyBuZWcgOiBpdGVtO1xuICB9O1xuICBjb25zdCBvcHRzID0gKFc6IG51bWJlcikgPT4ge1xuICAgIGNvbnN0IHdpbmRvd3MgPSBNYXRoLmNlaWwoYml0cyAvIFcpICsgMTsgLy8gKzEsIGJlY2F1c2VcbiAgICBjb25zdCB3aW5kb3dTaXplID0gMiAqKiAoVyAtIDEpOyAvLyAtMSBiZWNhdXNlIHdlIHNraXAgemVyb1xuICAgIHJldHVybiB7IHdpbmRvd3MsIHdpbmRvd1NpemUgfTtcbiAgfTtcbiAgcmV0dXJuIHtcbiAgICBjb25zdFRpbWVOZWdhdGUsXG4gICAgLy8gbm9uLWNvbnN0IHRpbWUgbXVsdGlwbGljYXRpb24gbGFkZGVyXG4gICAgdW5zYWZlTGFkZGVyKGVsbTogVCwgbjogYmlnaW50KSB7XG4gICAgICBsZXQgcCA9IGMuWkVSTztcbiAgICAgIGxldCBkOiBUID0gZWxtO1xuICAgICAgd2hpbGUgKG4gPiBfMG4pIHtcbiAgICAgICAgaWYgKG4gJiBfMW4pIHAgPSBwLmFkZChkKTtcbiAgICAgICAgZCA9IGQuZG91YmxlKCk7XG4gICAgICAgIG4gPj49IF8xbjtcbiAgICAgIH1cbiAgICAgIHJldHVybiBwO1xuICAgIH0sXG5cbiAgICAvKipcbiAgICAgKiBDcmVhdGVzIGEgd05BRiBwcmVjb21wdXRhdGlvbiB3aW5kb3cuIFVzZWQgZm9yIGNhY2hpbmcuXG4gICAgICogRGVmYXVsdCB3aW5kb3cgc2l6ZSBpcyBzZXQgYnkgYHV0aWxzLnByZWNvbXB1dGUoKWAgYW5kIGlzIGVxdWFsIHRvIDguXG4gICAgICogTnVtYmVyIG9mIHByZWNvbXB1dGVkIHBvaW50cyBkZXBlbmRzIG9uIHRoZSBjdXJ2ZSBzaXplOlxuICAgICAqIDJeKFx1RDgzNVx1REM0QVx1MjIxMjEpICogKE1hdGguY2VpbChcdUQ4MzVcdURDNUIgLyBcdUQ4MzVcdURDNEEpICsgMSksIHdoZXJlOlxuICAgICAqIC0gXHVEODM1XHVEQzRBIGlzIHRoZSB3aW5kb3cgc2l6ZVxuICAgICAqIC0gXHVEODM1XHVEQzVCIGlzIHRoZSBiaXRsZW5ndGggb2YgdGhlIGN1cnZlIG9yZGVyLlxuICAgICAqIEZvciBhIDI1Ni1iaXQgY3VydmUgYW5kIHdpbmRvdyBzaXplIDgsIHRoZSBudW1iZXIgb2YgcHJlY29tcHV0ZWQgcG9pbnRzIGlzIDEyOCAqIDMzID0gNDIyNC5cbiAgICAgKiBAcmV0dXJucyBwcmVjb21wdXRlZCBwb2ludCB0YWJsZXMgZmxhdHRlbmVkIHRvIGEgc2luZ2xlIGFycmF5XG4gICAgICovXG4gICAgcHJlY29tcHV0ZVdpbmRvdyhlbG06IFQsIFc6IG51bWJlcik6IEdyb3VwPFQ+W10ge1xuICAgICAgY29uc3QgeyB3aW5kb3dzLCB3aW5kb3dTaXplIH0gPSBvcHRzKFcpO1xuICAgICAgY29uc3QgcG9pbnRzOiBUW10gPSBbXTtcbiAgICAgIGxldCBwOiBUID0gZWxtO1xuICAgICAgbGV0IGJhc2UgPSBwO1xuICAgICAgZm9yIChsZXQgd2luZG93ID0gMDsgd2luZG93IDwgd2luZG93czsgd2luZG93KyspIHtcbiAgICAgICAgYmFzZSA9IHA7XG4gICAgICAgIHBvaW50cy5wdXNoKGJhc2UpO1xuICAgICAgICAvLyA9MSwgYmVjYXVzZSB3ZSBza2lwIHplcm9cbiAgICAgICAgZm9yIChsZXQgaSA9IDE7IGkgPCB3aW5kb3dTaXplOyBpKyspIHtcbiAgICAgICAgICBiYXNlID0gYmFzZS5hZGQocCk7XG4gICAgICAgICAgcG9pbnRzLnB1c2goYmFzZSk7XG4gICAgICAgIH1cbiAgICAgICAgcCA9IGJhc2UuZG91YmxlKCk7XG4gICAgICB9XG4gICAgICByZXR1cm4gcG9pbnRzO1xuICAgIH0sXG5cbiAgICAvKipcbiAgICAgKiBJbXBsZW1lbnRzIGVjIG11bHRpcGxpY2F0aW9uIHVzaW5nIHByZWNvbXB1dGVkIHRhYmxlcyBhbmQgdy1hcnkgbm9uLWFkamFjZW50IGZvcm0uXG4gICAgICogQHBhcmFtIFcgd2luZG93IHNpemVcbiAgICAgKiBAcGFyYW0gcHJlY29tcHV0ZXMgcHJlY29tcHV0ZWQgdGFibGVzXG4gICAgICogQHBhcmFtIG4gc2NhbGFyICh3ZSBkb24ndCBjaGVjayBoZXJlLCBidXQgc2hvdWxkIGJlIGxlc3MgdGhhbiBjdXJ2ZSBvcmRlcilcbiAgICAgKiBAcmV0dXJucyByZWFsIGFuZCBmYWtlIChmb3IgY29uc3QtdGltZSkgcG9pbnRzXG4gICAgICovXG4gICAgd05BRihXOiBudW1iZXIsIHByZWNvbXB1dGVzOiBUW10sIG46IGJpZ2ludCk6IHsgcDogVDsgZjogVCB9IHtcbiAgICAgIC8vIFRPRE86IG1heWJlIGNoZWNrIHRoYXQgc2NhbGFyIGlzIGxlc3MgdGhhbiBncm91cCBvcmRlcj8gd05BRiBiZWhhdmlvdXMgaXMgdW5kZWZpbmVkIG90aGVyd2lzZVxuICAgICAgLy8gQnV0IG5lZWQgdG8gY2FyZWZ1bGx5IHJlbW92ZSBvdGhlciBjaGVja3MgYmVmb3JlIHdOQUYuIE9SREVSID09IGJpdHMgaGVyZVxuICAgICAgY29uc3QgeyB3aW5kb3dzLCB3aW5kb3dTaXplIH0gPSBvcHRzKFcpO1xuXG4gICAgICBsZXQgcCA9IGMuWkVSTztcbiAgICAgIGxldCBmID0gYy5CQVNFO1xuXG4gICAgICBjb25zdCBtYXNrID0gQmlnSW50KDIgKiogVyAtIDEpOyAvLyBDcmVhdGUgbWFzayB3aXRoIFcgb25lczogMGIxMTExIGZvciBXPTQgZXRjLlxuICAgICAgY29uc3QgbWF4TnVtYmVyID0gMiAqKiBXO1xuICAgICAgY29uc3Qgc2hpZnRCeSA9IEJpZ0ludChXKTtcblxuICAgICAgZm9yIChsZXQgd2luZG93ID0gMDsgd2luZG93IDwgd2luZG93czsgd2luZG93KyspIHtcbiAgICAgICAgY29uc3Qgb2Zmc2V0ID0gd2luZG93ICogd2luZG93U2l6ZTtcbiAgICAgICAgLy8gRXh0cmFjdCBXIGJpdHMuXG4gICAgICAgIGxldCB3Yml0cyA9IE51bWJlcihuICYgbWFzayk7XG5cbiAgICAgICAgLy8gU2hpZnQgbnVtYmVyIGJ5IFcgYml0cy5cbiAgICAgICAgbiA+Pj0gc2hpZnRCeTtcblxuICAgICAgICAvLyBJZiB0aGUgYml0cyBhcmUgYmlnZ2VyIHRoYW4gbWF4IHNpemUsIHdlJ2xsIHNwbGl0IHRob3NlLlxuICAgICAgICAvLyArMjI0ID0+IDI1NiAtIDMyXG4gICAgICAgIGlmICh3Yml0cyA+IHdpbmRvd1NpemUpIHtcbiAgICAgICAgICB3Yml0cyAtPSBtYXhOdW1iZXI7XG4gICAgICAgICAgbiArPSBfMW47XG4gICAgICAgIH1cblxuICAgICAgICAvLyBUaGlzIGNvZGUgd2FzIGZpcnN0IHdyaXR0ZW4gd2l0aCBhc3N1bXB0aW9uIHRoYXQgJ2YnIGFuZCAncCcgd2lsbCBuZXZlciBiZSBpbmZpbml0eSBwb2ludDpcbiAgICAgICAgLy8gc2luY2UgZWFjaCBhZGRpdGlvbiBpcyBtdWx0aXBsaWVkIGJ5IDIgKiogVywgaXQgY2Fubm90IGNhbmNlbCBlYWNoIG90aGVyLiBIb3dldmVyLFxuICAgICAgICAvLyB0aGVyZSBpcyBuZWdhdGUgbm93OiBpdCBpcyBwb3NzaWJsZSB0aGF0IG5lZ2F0ZWQgZWxlbWVudCBmcm9tIGxvdyB2YWx1ZVxuICAgICAgICAvLyB3b3VsZCBiZSB0aGUgc2FtZSBhcyBoaWdoIGVsZW1lbnQsIHdoaWNoIHdpbGwgY3JlYXRlIGNhcnJ5IGludG8gbmV4dCB3aW5kb3cuXG4gICAgICAgIC8vIEl0J3Mgbm90IG9idmlvdXMgaG93IHRoaXMgY2FuIGZhaWwsIGJ1dCBzdGlsbCB3b3J0aCBpbnZlc3RpZ2F0aW5nIGxhdGVyLlxuXG4gICAgICAgIC8vIENoZWNrIGlmIHdlJ3JlIG9udG8gWmVybyBwb2ludC5cbiAgICAgICAgLy8gQWRkIHJhbmRvbSBwb2ludCBpbnNpZGUgY3VycmVudCB3aW5kb3cgdG8gZi5cbiAgICAgICAgY29uc3Qgb2Zmc2V0MSA9IG9mZnNldDtcbiAgICAgICAgY29uc3Qgb2Zmc2V0MiA9IG9mZnNldCArIE1hdGguYWJzKHdiaXRzKSAtIDE7IC8vIC0xIGJlY2F1c2Ugd2Ugc2tpcCB6ZXJvXG4gICAgICAgIGNvbnN0IGNvbmQxID0gd2luZG93ICUgMiAhPT0gMDtcbiAgICAgICAgY29uc3QgY29uZDIgPSB3Yml0cyA8IDA7XG4gICAgICAgIGlmICh3Yml0cyA9PT0gMCkge1xuICAgICAgICAgIC8vIFRoZSBtb3N0IGltcG9ydGFudCBwYXJ0IGZvciBjb25zdC10aW1lIGdldFB1YmxpY0tleVxuICAgICAgICAgIGYgPSBmLmFkZChjb25zdFRpbWVOZWdhdGUoY29uZDEsIHByZWNvbXB1dGVzW29mZnNldDFdKSk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgcCA9IHAuYWRkKGNvbnN0VGltZU5lZ2F0ZShjb25kMiwgcHJlY29tcHV0ZXNbb2Zmc2V0Ml0pKTtcbiAgICAgICAgfVxuICAgICAgfVxuICAgICAgLy8gSklULWNvbXBpbGVyIHNob3VsZCBub3QgZWxpbWluYXRlIGYgaGVyZSwgc2luY2UgaXQgd2lsbCBsYXRlciBiZSB1c2VkIGluIG5vcm1hbGl6ZVooKVxuICAgICAgLy8gRXZlbiBpZiB0aGUgdmFyaWFibGUgaXMgc3RpbGwgdW51c2VkLCB0aGVyZSBhcmUgc29tZSBjaGVja3Mgd2hpY2ggd2lsbFxuICAgICAgLy8gdGhyb3cgYW4gZXhjZXB0aW9uLCBzbyBjb21waWxlciBuZWVkcyB0byBwcm92ZSB0aGV5IHdvbid0IGhhcHBlbiwgd2hpY2ggaXMgaGFyZC5cbiAgICAgIC8vIEF0IHRoaXMgcG9pbnQgdGhlcmUgaXMgYSB3YXkgdG8gRiBiZSBpbmZpbml0eS1wb2ludCBldmVuIGlmIHAgaXMgbm90LFxuICAgICAgLy8gd2hpY2ggbWFrZXMgaXQgbGVzcyBjb25zdC10aW1lOiBhcm91bmQgMSBiaWdpbnQgbXVsdGlwbHkuXG4gICAgICByZXR1cm4geyBwLCBmIH07XG4gICAgfSxcblxuICAgIHdOQUZDYWNoZWQoUDogVCwgcHJlY29tcHV0ZXNNYXA6IE1hcDxULCBUW10+LCBuOiBiaWdpbnQsIHRyYW5zZm9ybTogTWFwcGVyPFQ+KTogeyBwOiBUOyBmOiBUIH0ge1xuICAgICAgLy8gQHRzLWlnbm9yZVxuICAgICAgY29uc3QgVzogbnVtYmVyID0gUC5fV0lORE9XX1NJWkUgfHwgMTtcbiAgICAgIC8vIENhbGN1bGF0ZSBwcmVjb21wdXRlcyBvbiBhIGZpcnN0IHJ1biwgcmV1c2UgdGhlbSBhZnRlclxuICAgICAgbGV0IGNvbXAgPSBwcmVjb21wdXRlc01hcC5nZXQoUCk7XG4gICAgICBpZiAoIWNvbXApIHtcbiAgICAgICAgY29tcCA9IHRoaXMucHJlY29tcHV0ZVdpbmRvdyhQLCBXKSBhcyBUW107XG4gICAgICAgIGlmIChXICE9PSAxKSB7XG4gICAgICAgICAgcHJlY29tcHV0ZXNNYXAuc2V0KFAsIHRyYW5zZm9ybShjb21wKSk7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICAgIHJldHVybiB0aGlzLndOQUYoVywgY29tcCwgbik7XG4gICAgfSxcbiAgfTtcbn1cblxuLy8gR2VuZXJpYyBCYXNpY0N1cnZlIGludGVyZmFjZTogd29ya3MgZXZlbiBmb3IgcG9seW5vbWlhbCBmaWVsZHMgKEJMUyk6IFAsIG4sIGggd291bGQgYmUgb2suXG4vLyBUaG91Z2ggZ2VuZXJhdG9yIGNhbiBiZSBkaWZmZXJlbnQgKEZwMiAvIEZwNiBmb3IgQkxTKS5cbmV4cG9ydCB0eXBlIEJhc2ljQ3VydmU8VD4gPSB7XG4gIEZwOiBJRmllbGQ8VD47IC8vIEZpZWxkIG92ZXIgd2hpY2ggd2UnbGwgZG8gY2FsY3VsYXRpb25zIChGcClcbiAgbjogYmlnaW50OyAvLyBDdXJ2ZSBvcmRlciwgdG90YWwgY291bnQgb2YgdmFsaWQgcG9pbnRzIGluIHRoZSBmaWVsZFxuICBuQml0TGVuZ3RoPzogbnVtYmVyOyAvLyBiaXQgbGVuZ3RoIG9mIGN1cnZlIG9yZGVyXG4gIG5CeXRlTGVuZ3RoPzogbnVtYmVyOyAvLyBieXRlIGxlbmd0aCBvZiBjdXJ2ZSBvcmRlclxuICBoOiBiaWdpbnQ7IC8vIGNvZmFjdG9yLiB3ZSBjYW4gYXNzaWduIGRlZmF1bHQ9MSwgYnV0IHVzZXJzIHdpbGwganVzdCBpZ25vcmUgaXQgdy9vIHZhbGlkYXRpb25cbiAgaEVmZj86IGJpZ2ludDsgLy8gTnVtYmVyIHRvIG11bHRpcGx5IHRvIGNsZWFyIGNvZmFjdG9yXG4gIEd4OiBUOyAvLyBiYXNlIHBvaW50IFggY29vcmRpbmF0ZVxuICBHeTogVDsgLy8gYmFzZSBwb2ludCBZIGNvb3JkaW5hdGVcbiAgYWxsb3dJbmZpbml0eVBvaW50PzogYm9vbGVhbjsgLy8gYmxzMTItMzgxIHJlcXVpcmVzIGl0LiBaRVJPIHBvaW50IGlzIHZhbGlkLCBidXQgaW52YWxpZCBwdWJrZXlcbn07XG5cbmV4cG9ydCBmdW5jdGlvbiB2YWxpZGF0ZUJhc2ljPEZQLCBUPihjdXJ2ZTogQmFzaWNDdXJ2ZTxGUD4gJiBUKSB7XG4gIHZhbGlkYXRlRmllbGQoY3VydmUuRnApO1xuICB2YWxpZGF0ZU9iamVjdChcbiAgICBjdXJ2ZSxcbiAgICB7XG4gICAgICBuOiAnYmlnaW50JyxcbiAgICAgIGg6ICdiaWdpbnQnLFxuICAgICAgR3g6ICdmaWVsZCcsXG4gICAgICBHeTogJ2ZpZWxkJyxcbiAgICB9LFxuICAgIHtcbiAgICAgIG5CaXRMZW5ndGg6ICdpc1NhZmVJbnRlZ2VyJyxcbiAgICAgIG5CeXRlTGVuZ3RoOiAnaXNTYWZlSW50ZWdlcicsXG4gICAgfVxuICApO1xuICAvLyBTZXQgZGVmYXVsdHNcbiAgcmV0dXJuIE9iamVjdC5mcmVlemUoe1xuICAgIC4uLm5MZW5ndGgoY3VydmUubiwgY3VydmUubkJpdExlbmd0aCksXG4gICAgLi4uY3VydmUsXG4gICAgLi4ueyBwOiBjdXJ2ZS5GcC5PUkRFUiB9LFxuICB9IGFzIGNvbnN0KTtcbn1cbiIsICIvKiEgbm9ibGUtY3VydmVzIC0gTUlUIExpY2Vuc2UgKGMpIDIwMjIgUGF1bCBNaWxsZXIgKHBhdWxtaWxsci5jb20pICovXG4vLyBTaG9ydCBXZWllcnN0cmFzcyBjdXJ2ZS4gVGhlIGZvcm11bGEgaXM6IHlcdTAwQjIgPSB4XHUwMEIzICsgYXggKyBiXG5pbXBvcnQgKiBhcyBtb2QgZnJvbSAnLi9tb2R1bGFyLmpzJztcbmltcG9ydCAqIGFzIHV0IGZyb20gJy4vdXRpbHMuanMnO1xuaW1wb3J0IHsgQ0hhc2gsIEhleCwgUHJpdktleSwgZW5zdXJlQnl0ZXMgfSBmcm9tICcuL3V0aWxzLmpzJztcbmltcG9ydCB7IEdyb3VwLCBHcm91cENvbnN0cnVjdG9yLCB3TkFGLCBCYXNpY0N1cnZlLCB2YWxpZGF0ZUJhc2ljLCBBZmZpbmVQb2ludCB9IGZyb20gJy4vY3VydmUuanMnO1xuXG5leHBvcnQgdHlwZSB7IEFmZmluZVBvaW50IH07XG50eXBlIEhtYWNGblN5bmMgPSAoa2V5OiBVaW50OEFycmF5LCAuLi5tZXNzYWdlczogVWludDhBcnJheVtdKSA9PiBVaW50OEFycmF5O1xudHlwZSBFbmRvbW9ycGhpc21PcHRzID0ge1xuICBiZXRhOiBiaWdpbnQ7XG4gIHNwbGl0U2NhbGFyOiAoazogYmlnaW50KSA9PiB7IGsxbmVnOiBib29sZWFuOyBrMTogYmlnaW50OyBrMm5lZzogYm9vbGVhbjsgazI6IGJpZ2ludCB9O1xufTtcbmV4cG9ydCB0eXBlIEJhc2ljV0N1cnZlPFQ+ID0gQmFzaWNDdXJ2ZTxUPiAmIHtcbiAgLy8gUGFyYW1zOiBhLCBiXG4gIGE6IFQ7XG4gIGI6IFQ7XG5cbiAgLy8gT3B0aW9uYWwgcGFyYW1zXG4gIGFsbG93ZWRQcml2YXRlS2V5TGVuZ3Rocz86IHJlYWRvbmx5IG51bWJlcltdOyAvLyBmb3IgUDUyMVxuICB3cmFwUHJpdmF0ZUtleT86IGJvb2xlYW47IC8vIGJsczEyLTM4MSByZXF1aXJlcyBtb2QobikgaW5zdGVhZCBvZiByZWplY3Rpbmcga2V5cyA+PSBuXG4gIGVuZG8/OiBFbmRvbW9ycGhpc21PcHRzOyAvLyBFbmRvbW9ycGhpc20gb3B0aW9ucyBmb3IgS29ibGl0eiBjdXJ2ZXNcbiAgLy8gV2hlbiBhIGNvZmFjdG9yICE9IDEsIHRoZXJlIGNhbiBiZSBhbiBlZmZlY3RpdmUgbWV0aG9kcyB0bzpcbiAgLy8gMS4gRGV0ZXJtaW5lIHdoZXRoZXIgYSBwb2ludCBpcyB0b3JzaW9uLWZyZWVcbiAgaXNUb3JzaW9uRnJlZT86IChjOiBQcm9qQ29uc3RydWN0b3I8VD4sIHBvaW50OiBQcm9qUG9pbnRUeXBlPFQ+KSA9PiBib29sZWFuO1xuICAvLyAyLiBDbGVhciB0b3JzaW9uIGNvbXBvbmVudFxuICBjbGVhckNvZmFjdG9yPzogKGM6IFByb2pDb25zdHJ1Y3RvcjxUPiwgcG9pbnQ6IFByb2pQb2ludFR5cGU8VD4pID0+IFByb2pQb2ludFR5cGU8VD47XG59O1xuXG50eXBlIEVudHJvcHkgPSBIZXggfCB0cnVlO1xuZXhwb3J0IHR5cGUgU2lnbk9wdHMgPSB7IGxvd1M/OiBib29sZWFuOyBleHRyYUVudHJvcHk/OiBFbnRyb3B5OyBwcmVoYXNoPzogYm9vbGVhbiB9O1xuZXhwb3J0IHR5cGUgVmVyT3B0cyA9IHsgbG93Uz86IGJvb2xlYW47IHByZWhhc2g/OiBib29sZWFuIH07XG5cbi8qKlxuICogIyMjIERlc2lnbiByYXRpb25hbGUgZm9yIHR5cGVzXG4gKlxuICogKiBJbnRlcmFjdGlvbiBiZXR3ZWVuIGNsYXNzZXMgZnJvbSBkaWZmZXJlbnQgY3VydmVzIHNob3VsZCBmYWlsOlxuICogICBgazI1Ni5Qb2ludC5CQVNFLmFkZChwMjU2LlBvaW50LkJBU0UpYFxuICogKiBGb3IgdGhpcyBwdXJwb3NlIHdlIHdhbnQgdG8gdXNlIGBpbnN0YW5jZW9mYCBvcGVyYXRvciwgd2hpY2ggaXMgZmFzdCBhbmQgd29ya3MgZHVyaW5nIHJ1bnRpbWVcbiAqICogRGlmZmVyZW50IGNhbGxzIG9mIGBjdXJ2ZSgpYCB3b3VsZCByZXR1cm4gZGlmZmVyZW50IGNsYXNzZXMgLVxuICogICBgY3VydmUocGFyYW1zKSAhPT0gY3VydmUocGFyYW1zKWA6IGlmIHNvbWVib2R5IGRlY2lkZWQgdG8gbW9ua2V5LXBhdGNoIHRoZWlyIGN1cnZlLFxuICogICBpdCB3b24ndCBhZmZlY3Qgb3RoZXJzXG4gKlxuICogVHlwZVNjcmlwdCBjYW4ndCBpbmZlciB0eXBlcyBmb3IgY2xhc3NlcyBjcmVhdGVkIGluc2lkZSBhIGZ1bmN0aW9uLiBDbGFzc2VzIGlzIG9uZSBpbnN0YW5jZSBvZiBub21pbmF0aXZlIHR5cGVzIGluIFR5cGVTY3JpcHQgYW5kIGludGVyZmFjZXMgb25seSBjaGVjayBmb3Igc2hhcGUsIHNvIGl0J3MgaGFyZCB0byBjcmVhdGUgdW5pcXVlIHR5cGUgZm9yIGV2ZXJ5IGZ1bmN0aW9uIGNhbGwuXG4gKlxuICogV2UgY2FuIHVzZSBnZW5lcmljIHR5cGVzIHZpYSBzb21lIHBhcmFtLCBsaWtlIGN1cnZlIG9wdHMsIGJ1dCB0aGF0IHdvdWxkOlxuICogICAgIDEuIEVuYWJsZSBpbnRlcmFjdGlvbiBiZXR3ZWVuIGBjdXJ2ZShwYXJhbXMpYCBhbmQgYGN1cnZlKHBhcmFtcylgIChjdXJ2ZXMgb2Ygc2FtZSBwYXJhbXMpXG4gKiAgICAgd2hpY2ggaXMgaGFyZCB0byBkZWJ1Zy5cbiAqICAgICAyLiBQYXJhbXMgY2FuIGJlIGdlbmVyaWMgYW5kIHdlIGNhbid0IGVuZm9yY2UgdGhlbSB0byBiZSBjb25zdGFudCB2YWx1ZTpcbiAqICAgICBpZiBzb21lYm9keSBjcmVhdGVzIGN1cnZlIGZyb20gbm9uLWNvbnN0YW50IHBhcmFtcyxcbiAqICAgICBpdCB3b3VsZCBiZSBhbGxvd2VkIHRvIGludGVyYWN0IHdpdGggb3RoZXIgY3VydmVzIHdpdGggbm9uLWNvbnN0YW50IHBhcmFtc1xuICpcbiAqIFRPRE86IGh0dHBzOi8vd3d3LnR5cGVzY3JpcHRsYW5nLm9yZy9kb2NzL2hhbmRib29rL3JlbGVhc2Utbm90ZXMvdHlwZXNjcmlwdC0yLTcuaHRtbCN1bmlxdWUtc3ltYm9sXG4gKi9cblxuLy8gSW5zdGFuY2UgZm9yIDNkIFhZWiBwb2ludHNcbmV4cG9ydCBpbnRlcmZhY2UgUHJvalBvaW50VHlwZTxUPiBleHRlbmRzIEdyb3VwPFByb2pQb2ludFR5cGU8VD4+IHtcbiAgcmVhZG9ubHkgcHg6IFQ7XG4gIHJlYWRvbmx5IHB5OiBUO1xuICByZWFkb25seSBwejogVDtcbiAgZ2V0IHgoKTogVDtcbiAgZ2V0IHkoKTogVDtcbiAgbXVsdGlwbHkoc2NhbGFyOiBiaWdpbnQpOiBQcm9qUG9pbnRUeXBlPFQ+O1xuICB0b0FmZmluZShpej86IFQpOiBBZmZpbmVQb2ludDxUPjtcbiAgaXNUb3JzaW9uRnJlZSgpOiBib29sZWFuO1xuICBjbGVhckNvZmFjdG9yKCk6IFByb2pQb2ludFR5cGU8VD47XG4gIGFzc2VydFZhbGlkaXR5KCk6IHZvaWQ7XG4gIGhhc0V2ZW5ZKCk6IGJvb2xlYW47XG4gIHRvUmF3Qnl0ZXMoaXNDb21wcmVzc2VkPzogYm9vbGVhbik6IFVpbnQ4QXJyYXk7XG4gIHRvSGV4KGlzQ29tcHJlc3NlZD86IGJvb2xlYW4pOiBzdHJpbmc7XG5cbiAgbXVsdGlwbHlVbnNhZmUoc2NhbGFyOiBiaWdpbnQpOiBQcm9qUG9pbnRUeXBlPFQ+O1xuICBtdWx0aXBseUFuZEFkZFVuc2FmZShROiBQcm9qUG9pbnRUeXBlPFQ+LCBhOiBiaWdpbnQsIGI6IGJpZ2ludCk6IFByb2pQb2ludFR5cGU8VD4gfCB1bmRlZmluZWQ7XG4gIF9zZXRXaW5kb3dTaXplKHdpbmRvd1NpemU6IG51bWJlcik6IHZvaWQ7XG59XG4vLyBTdGF0aWMgbWV0aG9kcyBmb3IgM2QgWFlaIHBvaW50c1xuZXhwb3J0IGludGVyZmFjZSBQcm9qQ29uc3RydWN0b3I8VD4gZXh0ZW5kcyBHcm91cENvbnN0cnVjdG9yPFByb2pQb2ludFR5cGU8VD4+IHtcbiAgbmV3ICh4OiBULCB5OiBULCB6OiBUKTogUHJvalBvaW50VHlwZTxUPjtcbiAgZnJvbUFmZmluZShwOiBBZmZpbmVQb2ludDxUPik6IFByb2pQb2ludFR5cGU8VD47XG4gIGZyb21IZXgoaGV4OiBIZXgpOiBQcm9qUG9pbnRUeXBlPFQ+O1xuICBmcm9tUHJpdmF0ZUtleShwcml2YXRlS2V5OiBQcml2S2V5KTogUHJvalBvaW50VHlwZTxUPjtcbiAgbm9ybWFsaXplWihwb2ludHM6IFByb2pQb2ludFR5cGU8VD5bXSk6IFByb2pQb2ludFR5cGU8VD5bXTtcbn1cblxuZXhwb3J0IHR5cGUgQ3VydmVQb2ludHNUeXBlPFQ+ID0gQmFzaWNXQ3VydmU8VD4gJiB7XG4gIC8vIEJ5dGVzXG4gIGZyb21CeXRlcz86IChieXRlczogVWludDhBcnJheSkgPT4gQWZmaW5lUG9pbnQ8VD47XG4gIHRvQnl0ZXM/OiAoYzogUHJvakNvbnN0cnVjdG9yPFQ+LCBwb2ludDogUHJvalBvaW50VHlwZTxUPiwgaXNDb21wcmVzc2VkOiBib29sZWFuKSA9PiBVaW50OEFycmF5O1xufTtcblxuZnVuY3Rpb24gdmFsaWRhdGVQb2ludE9wdHM8VD4oY3VydmU6IEN1cnZlUG9pbnRzVHlwZTxUPikge1xuICBjb25zdCBvcHRzID0gdmFsaWRhdGVCYXNpYyhjdXJ2ZSk7XG4gIHV0LnZhbGlkYXRlT2JqZWN0KFxuICAgIG9wdHMsXG4gICAge1xuICAgICAgYTogJ2ZpZWxkJyxcbiAgICAgIGI6ICdmaWVsZCcsXG4gICAgfSxcbiAgICB7XG4gICAgICBhbGxvd2VkUHJpdmF0ZUtleUxlbmd0aHM6ICdhcnJheScsXG4gICAgICB3cmFwUHJpdmF0ZUtleTogJ2Jvb2xlYW4nLFxuICAgICAgaXNUb3JzaW9uRnJlZTogJ2Z1bmN0aW9uJyxcbiAgICAgIGNsZWFyQ29mYWN0b3I6ICdmdW5jdGlvbicsXG4gICAgICBhbGxvd0luZmluaXR5UG9pbnQ6ICdib29sZWFuJyxcbiAgICAgIGZyb21CeXRlczogJ2Z1bmN0aW9uJyxcbiAgICAgIHRvQnl0ZXM6ICdmdW5jdGlvbicsXG4gICAgfVxuICApO1xuICBjb25zdCB7IGVuZG8sIEZwLCBhIH0gPSBvcHRzO1xuICBpZiAoZW5kbykge1xuICAgIGlmICghRnAuZXFsKGEsIEZwLlpFUk8pKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoJ0VuZG9tb3JwaGlzbSBjYW4gb25seSBiZSBkZWZpbmVkIGZvciBLb2JsaXR6IGN1cnZlcyB0aGF0IGhhdmUgYT0wJyk7XG4gICAgfVxuICAgIGlmIChcbiAgICAgIHR5cGVvZiBlbmRvICE9PSAnb2JqZWN0JyB8fFxuICAgICAgdHlwZW9mIGVuZG8uYmV0YSAhPT0gJ2JpZ2ludCcgfHxcbiAgICAgIHR5cGVvZiBlbmRvLnNwbGl0U2NhbGFyICE9PSAnZnVuY3Rpb24nXG4gICAgKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoJ0V4cGVjdGVkIGVuZG9tb3JwaGlzbSB3aXRoIGJldGE6IGJpZ2ludCBhbmQgc3BsaXRTY2FsYXI6IGZ1bmN0aW9uJyk7XG4gICAgfVxuICB9XG4gIHJldHVybiBPYmplY3QuZnJlZXplKHsgLi4ub3B0cyB9IGFzIGNvbnN0KTtcbn1cblxuZXhwb3J0IHR5cGUgQ3VydmVQb2ludHNSZXM8VD4gPSB7XG4gIFByb2plY3RpdmVQb2ludDogUHJvakNvbnN0cnVjdG9yPFQ+O1xuICBub3JtUHJpdmF0ZUtleVRvU2NhbGFyOiAoa2V5OiBQcml2S2V5KSA9PiBiaWdpbnQ7XG4gIHdlaWVyc3RyYXNzRXF1YXRpb246ICh4OiBUKSA9PiBUO1xuICBpc1dpdGhpbkN1cnZlT3JkZXI6IChudW06IGJpZ2ludCkgPT4gYm9vbGVhbjtcbn07XG5cbi8vIEFTTi4xIERFUiBlbmNvZGluZyB1dGlsaXRpZXNcbmNvbnN0IHsgYnl0ZXNUb051bWJlckJFOiBiMm4sIGhleFRvQnl0ZXM6IGgyYiB9ID0gdXQ7XG5leHBvcnQgY29uc3QgREVSID0ge1xuICAvLyBhc24uMSBERVIgZW5jb2RpbmcgdXRpbHNcbiAgRXJyOiBjbGFzcyBERVJFcnIgZXh0ZW5kcyBFcnJvciB7XG4gICAgY29uc3RydWN0b3IobSA9ICcnKSB7XG4gICAgICBzdXBlcihtKTtcbiAgICB9XG4gIH0sXG4gIF9wYXJzZUludChkYXRhOiBVaW50OEFycmF5KTogeyBkOiBiaWdpbnQ7IGw6IFVpbnQ4QXJyYXkgfSB7XG4gICAgY29uc3QgeyBFcnI6IEUgfSA9IERFUjtcbiAgICBpZiAoZGF0YS5sZW5ndGggPCAyIHx8IGRhdGFbMF0gIT09IDB4MDIpIHRocm93IG5ldyBFKCdJbnZhbGlkIHNpZ25hdHVyZSBpbnRlZ2VyIHRhZycpO1xuICAgIGNvbnN0IGxlbiA9IGRhdGFbMV07XG4gICAgY29uc3QgcmVzID0gZGF0YS5zdWJhcnJheSgyLCBsZW4gKyAyKTtcbiAgICBpZiAoIWxlbiB8fCByZXMubGVuZ3RoICE9PSBsZW4pIHRocm93IG5ldyBFKCdJbnZhbGlkIHNpZ25hdHVyZSBpbnRlZ2VyOiB3cm9uZyBsZW5ndGgnKTtcbiAgICAvLyBodHRwczovL2NyeXB0by5zdGFja2V4Y2hhbmdlLmNvbS9hLzU3NzM0IExlZnRtb3N0IGJpdCBvZiBmaXJzdCBieXRlIGlzICduZWdhdGl2ZScgZmxhZyxcbiAgICAvLyBzaW5jZSB3ZSBhbHdheXMgdXNlIHBvc2l0aXZlIGludGVnZXJzIGhlcmUuIEl0IG11c3QgYWx3YXlzIGJlIGVtcHR5OlxuICAgIC8vIC0gYWRkIHplcm8gYnl0ZSBpZiBleGlzdHNcbiAgICAvLyAtIGlmIG5leHQgYnl0ZSBkb2Vzbid0IGhhdmUgYSBmbGFnLCBsZWFkaW5nIHplcm8gaXMgbm90IGFsbG93ZWQgKG1pbmltYWwgZW5jb2RpbmcpXG4gICAgaWYgKHJlc1swXSAmIDBiMTAwMDAwMDApIHRocm93IG5ldyBFKCdJbnZhbGlkIHNpZ25hdHVyZSBpbnRlZ2VyOiBuZWdhdGl2ZScpO1xuICAgIGlmIChyZXNbMF0gPT09IDB4MDAgJiYgIShyZXNbMV0gJiAwYjEwMDAwMDAwKSlcbiAgICAgIHRocm93IG5ldyBFKCdJbnZhbGlkIHNpZ25hdHVyZSBpbnRlZ2VyOiB1bm5lY2Vzc2FyeSBsZWFkaW5nIHplcm8nKTtcbiAgICByZXR1cm4geyBkOiBiMm4ocmVzKSwgbDogZGF0YS5zdWJhcnJheShsZW4gKyAyKSB9OyAvLyBkIGlzIGRhdGEsIGwgaXMgbGVmdFxuICB9LFxuICB0b1NpZyhoZXg6IHN0cmluZyB8IFVpbnQ4QXJyYXkpOiB7IHI6IGJpZ2ludDsgczogYmlnaW50IH0ge1xuICAgIC8vIHBhcnNlIERFUiBzaWduYXR1cmVcbiAgICBjb25zdCB7IEVycjogRSB9ID0gREVSO1xuICAgIGNvbnN0IGRhdGEgPSB0eXBlb2YgaGV4ID09PSAnc3RyaW5nJyA/IGgyYihoZXgpIDogaGV4O1xuICAgIGlmICghKGRhdGEgaW5zdGFuY2VvZiBVaW50OEFycmF5KSkgdGhyb3cgbmV3IEVycm9yKCd1aThhIGV4cGVjdGVkJyk7XG4gICAgbGV0IGwgPSBkYXRhLmxlbmd0aDtcbiAgICBpZiAobCA8IDIgfHwgZGF0YVswXSAhPSAweDMwKSB0aHJvdyBuZXcgRSgnSW52YWxpZCBzaWduYXR1cmUgdGFnJyk7XG4gICAgaWYgKGRhdGFbMV0gIT09IGwgLSAyKSB0aHJvdyBuZXcgRSgnSW52YWxpZCBzaWduYXR1cmU6IGluY29ycmVjdCBsZW5ndGgnKTtcbiAgICBjb25zdCB7IGQ6IHIsIGw6IHNCeXRlcyB9ID0gREVSLl9wYXJzZUludChkYXRhLnN1YmFycmF5KDIpKTtcbiAgICBjb25zdCB7IGQ6IHMsIGw6IHJCeXRlc0xlZnQgfSA9IERFUi5fcGFyc2VJbnQoc0J5dGVzKTtcbiAgICBpZiAockJ5dGVzTGVmdC5sZW5ndGgpIHRocm93IG5ldyBFKCdJbnZhbGlkIHNpZ25hdHVyZTogbGVmdCBieXRlcyBhZnRlciBwYXJzaW5nJyk7XG4gICAgcmV0dXJuIHsgciwgcyB9O1xuICB9LFxuICBoZXhGcm9tU2lnKHNpZzogeyByOiBiaWdpbnQ7IHM6IGJpZ2ludCB9KTogc3RyaW5nIHtcbiAgICAvLyBBZGQgbGVhZGluZyB6ZXJvIGlmIGZpcnN0IGJ5dGUgaGFzIG5lZ2F0aXZlIGJpdCBlbmFibGVkLiBNb3JlIGRldGFpbHMgaW4gJ19wYXJzZUludCdcbiAgICBjb25zdCBzbGljZSA9IChzOiBzdHJpbmcpOiBzdHJpbmcgPT4gKE51bWJlci5wYXJzZUludChzWzBdLCAxNikgJiAwYjEwMDAgPyAnMDAnICsgcyA6IHMpO1xuICAgIGNvbnN0IGggPSAobnVtOiBudW1iZXIgfCBiaWdpbnQpID0+IHtcbiAgICAgIGNvbnN0IGhleCA9IG51bS50b1N0cmluZygxNik7XG4gICAgICByZXR1cm4gaGV4Lmxlbmd0aCAmIDEgPyBgMCR7aGV4fWAgOiBoZXg7XG4gICAgfTtcbiAgICBjb25zdCBzID0gc2xpY2UoaChzaWcucykpO1xuICAgIGNvbnN0IHIgPSBzbGljZShoKHNpZy5yKSk7XG4gICAgY29uc3Qgc2hsID0gcy5sZW5ndGggLyAyO1xuICAgIGNvbnN0IHJobCA9IHIubGVuZ3RoIC8gMjtcbiAgICBjb25zdCBzbCA9IGgoc2hsKTtcbiAgICBjb25zdCBybCA9IGgocmhsKTtcbiAgICByZXR1cm4gYDMwJHtoKHJobCArIHNobCArIDQpfTAyJHtybH0ke3J9MDIke3NsfSR7c31gO1xuICB9LFxufTtcblxuLy8gQmUgZnJpZW5kbHkgdG8gYmFkIEVDTUFTY3JpcHQgcGFyc2VycyBieSBub3QgdXNpbmcgYmlnaW50IGxpdGVyYWxzXG4vLyBwcmV0dGllci1pZ25vcmVcbmNvbnN0IF8wbiA9IEJpZ0ludCgwKSwgXzFuID0gQmlnSW50KDEpLCBfMm4gPSBCaWdJbnQoMiksIF8zbiA9IEJpZ0ludCgzKSwgXzRuID0gQmlnSW50KDQpO1xuXG5leHBvcnQgZnVuY3Rpb24gd2VpZXJzdHJhc3NQb2ludHM8VD4ob3B0czogQ3VydmVQb2ludHNUeXBlPFQ+KSB7XG4gIGNvbnN0IENVUlZFID0gdmFsaWRhdGVQb2ludE9wdHMob3B0cyk7XG4gIGNvbnN0IHsgRnAgfSA9IENVUlZFOyAvLyBBbGwgY3VydmVzIGhhcyBzYW1lIGZpZWxkIC8gZ3JvdXAgbGVuZ3RoIGFzIGZvciBub3csIGJ1dCB0aGV5IGNhbiBkaWZmZXJcblxuICBjb25zdCB0b0J5dGVzID1cbiAgICBDVVJWRS50b0J5dGVzIHx8XG4gICAgKChfYzogUHJvakNvbnN0cnVjdG9yPFQ+LCBwb2ludDogUHJvalBvaW50VHlwZTxUPiwgX2lzQ29tcHJlc3NlZDogYm9vbGVhbikgPT4ge1xuICAgICAgY29uc3QgYSA9IHBvaW50LnRvQWZmaW5lKCk7XG4gICAgICByZXR1cm4gdXQuY29uY2F0Qnl0ZXMoVWludDhBcnJheS5mcm9tKFsweDA0XSksIEZwLnRvQnl0ZXMoYS54KSwgRnAudG9CeXRlcyhhLnkpKTtcbiAgICB9KTtcbiAgY29uc3QgZnJvbUJ5dGVzID1cbiAgICBDVVJWRS5mcm9tQnl0ZXMgfHxcbiAgICAoKGJ5dGVzOiBVaW50OEFycmF5KSA9PiB7XG4gICAgICAvLyBjb25zdCBoZWFkID0gYnl0ZXNbMF07XG4gICAgICBjb25zdCB0YWlsID0gYnl0ZXMuc3ViYXJyYXkoMSk7XG4gICAgICAvLyBpZiAoaGVhZCAhPT0gMHgwNCkgdGhyb3cgbmV3IEVycm9yKCdPbmx5IG5vbi1jb21wcmVzc2VkIGVuY29kaW5nIGlzIHN1cHBvcnRlZCcpO1xuICAgICAgY29uc3QgeCA9IEZwLmZyb21CeXRlcyh0YWlsLnN1YmFycmF5KDAsIEZwLkJZVEVTKSk7XG4gICAgICBjb25zdCB5ID0gRnAuZnJvbUJ5dGVzKHRhaWwuc3ViYXJyYXkoRnAuQllURVMsIDIgKiBGcC5CWVRFUykpO1xuICAgICAgcmV0dXJuIHsgeCwgeSB9O1xuICAgIH0pO1xuXG4gIC8qKlxuICAgKiB5XHUwMEIyID0geFx1MDBCMyArIGF4ICsgYjogU2hvcnQgd2VpZXJzdHJhc3MgY3VydmUgZm9ybXVsYVxuICAgKiBAcmV0dXJucyB5XHUwMEIyXG4gICAqL1xuICBmdW5jdGlvbiB3ZWllcnN0cmFzc0VxdWF0aW9uKHg6IFQpOiBUIHtcbiAgICBjb25zdCB7IGEsIGIgfSA9IENVUlZFO1xuICAgIGNvbnN0IHgyID0gRnAuc3FyKHgpOyAvLyB4ICogeFxuICAgIGNvbnN0IHgzID0gRnAubXVsKHgyLCB4KTsgLy8geDIgKiB4XG4gICAgcmV0dXJuIEZwLmFkZChGcC5hZGQoeDMsIEZwLm11bCh4LCBhKSksIGIpOyAvLyB4MyArIGEgKiB4ICsgYlxuICB9XG4gIC8vIFZhbGlkYXRlIHdoZXRoZXIgdGhlIHBhc3NlZCBjdXJ2ZSBwYXJhbXMgYXJlIHZhbGlkLlxuICAvLyBXZSBjaGVjayBpZiBjdXJ2ZSBlcXVhdGlvbiB3b3JrcyBmb3IgZ2VuZXJhdG9yIHBvaW50LlxuICAvLyBgYXNzZXJ0VmFsaWRpdHkoKWAgd29uJ3Qgd29yazogYGlzVG9yc2lvbkZyZWUoKWAgaXMgbm90IGF2YWlsYWJsZSBhdCB0aGlzIHBvaW50IGluIGJsczEyLTM4MS5cbiAgLy8gUHJvamVjdGl2ZVBvaW50IGNsYXNzIGhhcyBub3QgYmVlbiBpbml0aWFsaXplZCB5ZXQuXG4gIGlmICghRnAuZXFsKEZwLnNxcihDVVJWRS5HeSksIHdlaWVyc3RyYXNzRXF1YXRpb24oQ1VSVkUuR3gpKSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoJ2JhZCBnZW5lcmF0b3IgcG9pbnQ6IGVxdWF0aW9uIGxlZnQgIT0gcmlnaHQnKTtcblxuICAvLyBWYWxpZCBncm91cCBlbGVtZW50cyByZXNpZGUgaW4gcmFuZ2UgMS4ubi0xXG4gIGZ1bmN0aW9uIGlzV2l0aGluQ3VydmVPcmRlcihudW06IGJpZ2ludCk6IGJvb2xlYW4ge1xuICAgIHJldHVybiB0eXBlb2YgbnVtID09PSAnYmlnaW50JyAmJiBfMG4gPCBudW0gJiYgbnVtIDwgQ1VSVkUubjtcbiAgfVxuICBmdW5jdGlvbiBhc3NlcnRHRShudW06IGJpZ2ludCkge1xuICAgIGlmICghaXNXaXRoaW5DdXJ2ZU9yZGVyKG51bSkpIHRocm93IG5ldyBFcnJvcignRXhwZWN0ZWQgdmFsaWQgYmlnaW50OiAwIDwgYmlnaW50IDwgY3VydmUubicpO1xuICB9XG4gIC8vIFZhbGlkYXRlcyBpZiBwcml2IGtleSBpcyB2YWxpZCBhbmQgY29udmVydHMgaXQgdG8gYmlnaW50LlxuICAvLyBTdXBwb3J0cyBvcHRpb25zIGFsbG93ZWRQcml2YXRlS2V5TGVuZ3RocyBhbmQgd3JhcFByaXZhdGVLZXkuXG4gIGZ1bmN0aW9uIG5vcm1Qcml2YXRlS2V5VG9TY2FsYXIoa2V5OiBQcml2S2V5KTogYmlnaW50IHtcbiAgICBjb25zdCB7IGFsbG93ZWRQcml2YXRlS2V5TGVuZ3RoczogbGVuZ3RocywgbkJ5dGVMZW5ndGgsIHdyYXBQcml2YXRlS2V5LCBuIH0gPSBDVVJWRTtcbiAgICBpZiAobGVuZ3RocyAmJiB0eXBlb2Yga2V5ICE9PSAnYmlnaW50Jykge1xuICAgICAgaWYgKGtleSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXkpIGtleSA9IHV0LmJ5dGVzVG9IZXgoa2V5KTtcbiAgICAgIC8vIE5vcm1hbGl6ZSB0byBoZXggc3RyaW5nLCBwYWQuIEUuZy4gUDUyMSB3b3VsZCBub3JtIDEzMC0xMzIgY2hhciBoZXggdG8gMTMyLWNoYXIgYnl0ZXNcbiAgICAgIGlmICh0eXBlb2Yga2V5ICE9PSAnc3RyaW5nJyB8fCAhbGVuZ3Rocy5pbmNsdWRlcyhrZXkubGVuZ3RoKSkgdGhyb3cgbmV3IEVycm9yKCdJbnZhbGlkIGtleScpO1xuICAgICAga2V5ID0ga2V5LnBhZFN0YXJ0KG5CeXRlTGVuZ3RoICogMiwgJzAnKTtcbiAgICB9XG4gICAgbGV0IG51bTogYmlnaW50O1xuICAgIHRyeSB7XG4gICAgICBudW0gPVxuICAgICAgICB0eXBlb2Yga2V5ID09PSAnYmlnaW50J1xuICAgICAgICAgID8ga2V5XG4gICAgICAgICAgOiB1dC5ieXRlc1RvTnVtYmVyQkUoZW5zdXJlQnl0ZXMoJ3ByaXZhdGUga2V5Jywga2V5LCBuQnl0ZUxlbmd0aCkpO1xuICAgIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYHByaXZhdGUga2V5IG11c3QgYmUgJHtuQnl0ZUxlbmd0aH0gYnl0ZXMsIGhleCBvciBiaWdpbnQsIG5vdCAke3R5cGVvZiBrZXl9YCk7XG4gICAgfVxuICAgIGlmICh3cmFwUHJpdmF0ZUtleSkgbnVtID0gbW9kLm1vZChudW0sIG4pOyAvLyBkaXNhYmxlZCBieSBkZWZhdWx0LCBlbmFibGVkIGZvciBCTFNcbiAgICBhc3NlcnRHRShudW0pOyAvLyBudW0gaW4gcmFuZ2UgWzEuLk4tMV1cbiAgICByZXR1cm4gbnVtO1xuICB9XG5cbiAgY29uc3QgcG9pbnRQcmVjb21wdXRlcyA9IG5ldyBNYXA8UG9pbnQsIFBvaW50W10+KCk7XG4gIGZ1bmN0aW9uIGFzc2VydFByalBvaW50KG90aGVyOiB1bmtub3duKSB7XG4gICAgaWYgKCEob3RoZXIgaW5zdGFuY2VvZiBQb2ludCkpIHRocm93IG5ldyBFcnJvcignUHJvamVjdGl2ZVBvaW50IGV4cGVjdGVkJyk7XG4gIH1cbiAgLyoqXG4gICAqIFByb2plY3RpdmUgUG9pbnQgd29ya3MgaW4gM2QgLyBwcm9qZWN0aXZlIChob21vZ2VuZW91cykgY29vcmRpbmF0ZXM6ICh4LCB5LCB6KSBcdTIyMEIgKHg9eC96LCB5PXkveilcbiAgICogRGVmYXVsdCBQb2ludCB3b3JrcyBpbiAyZCAvIGFmZmluZSBjb29yZGluYXRlczogKHgsIHkpXG4gICAqIFdlJ3JlIGRvaW5nIGNhbGN1bGF0aW9ucyBpbiBwcm9qZWN0aXZlLCBiZWNhdXNlIGl0cyBvcGVyYXRpb25zIGRvbid0IHJlcXVpcmUgY29zdGx5IGludmVyc2lvbi5cbiAgICovXG4gIGNsYXNzIFBvaW50IGltcGxlbWVudHMgUHJvalBvaW50VHlwZTxUPiB7XG4gICAgc3RhdGljIHJlYWRvbmx5IEJBU0UgPSBuZXcgUG9pbnQoQ1VSVkUuR3gsIENVUlZFLkd5LCBGcC5PTkUpO1xuICAgIHN0YXRpYyByZWFkb25seSBaRVJPID0gbmV3IFBvaW50KEZwLlpFUk8sIEZwLk9ORSwgRnAuWkVSTyk7XG5cbiAgICBjb25zdHJ1Y3RvcihyZWFkb25seSBweDogVCwgcmVhZG9ubHkgcHk6IFQsIHJlYWRvbmx5IHB6OiBUKSB7XG4gICAgICBpZiAocHggPT0gbnVsbCB8fCAhRnAuaXNWYWxpZChweCkpIHRocm93IG5ldyBFcnJvcigneCByZXF1aXJlZCcpO1xuICAgICAgaWYgKHB5ID09IG51bGwgfHwgIUZwLmlzVmFsaWQocHkpKSB0aHJvdyBuZXcgRXJyb3IoJ3kgcmVxdWlyZWQnKTtcbiAgICAgIGlmIChweiA9PSBudWxsIHx8ICFGcC5pc1ZhbGlkKHB6KSkgdGhyb3cgbmV3IEVycm9yKCd6IHJlcXVpcmVkJyk7XG4gICAgfVxuXG4gICAgLy8gRG9lcyBub3QgdmFsaWRhdGUgaWYgdGhlIHBvaW50IGlzIG9uLWN1cnZlLlxuICAgIC8vIFVzZSBmcm9tSGV4IGluc3RlYWQsIG9yIGNhbGwgYXNzZXJ0VmFsaWRpdHkoKSBsYXRlci5cbiAgICBzdGF0aWMgZnJvbUFmZmluZShwOiBBZmZpbmVQb2ludDxUPik6IFBvaW50IHtcbiAgICAgIGNvbnN0IHsgeCwgeSB9ID0gcCB8fCB7fTtcbiAgICAgIGlmICghcCB8fCAhRnAuaXNWYWxpZCh4KSB8fCAhRnAuaXNWYWxpZCh5KSkgdGhyb3cgbmV3IEVycm9yKCdpbnZhbGlkIGFmZmluZSBwb2ludCcpO1xuICAgICAgaWYgKHAgaW5zdGFuY2VvZiBQb2ludCkgdGhyb3cgbmV3IEVycm9yKCdwcm9qZWN0aXZlIHBvaW50IG5vdCBhbGxvd2VkJyk7XG4gICAgICBjb25zdCBpczAgPSAoaTogVCkgPT4gRnAuZXFsKGksIEZwLlpFUk8pO1xuICAgICAgLy8gZnJvbUFmZmluZSh4OjAsIHk6MCkgd291bGQgcHJvZHVjZSAoeDowLCB5OjAsIHo6MSksIGJ1dCB3ZSBuZWVkICh4OjAsIHk6MSwgejowKVxuICAgICAgaWYgKGlzMCh4KSAmJiBpczAoeSkpIHJldHVybiBQb2ludC5aRVJPO1xuICAgICAgcmV0dXJuIG5ldyBQb2ludCh4LCB5LCBGcC5PTkUpO1xuICAgIH1cblxuICAgIGdldCB4KCk6IFQge1xuICAgICAgcmV0dXJuIHRoaXMudG9BZmZpbmUoKS54O1xuICAgIH1cbiAgICBnZXQgeSgpOiBUIHtcbiAgICAgIHJldHVybiB0aGlzLnRvQWZmaW5lKCkueTtcbiAgICB9XG5cbiAgICAvKipcbiAgICAgKiBUYWtlcyBhIGJ1bmNoIG9mIFByb2plY3RpdmUgUG9pbnRzIGJ1dCBleGVjdXRlcyBvbmx5IG9uZVxuICAgICAqIGludmVyc2lvbiBvbiBhbGwgb2YgdGhlbS4gSW52ZXJzaW9uIGlzIHZlcnkgc2xvdyBvcGVyYXRpb24sXG4gICAgICogc28gdGhpcyBpbXByb3ZlcyBwZXJmb3JtYW5jZSBtYXNzaXZlbHkuXG4gICAgICogT3B0aW1pemF0aW9uOiBjb252ZXJ0cyBhIGxpc3Qgb2YgcHJvamVjdGl2ZSBwb2ludHMgdG8gYSBsaXN0IG9mIGlkZW50aWNhbCBwb2ludHMgd2l0aCBaPTEuXG4gICAgICovXG4gICAgc3RhdGljIG5vcm1hbGl6ZVoocG9pbnRzOiBQb2ludFtdKTogUG9pbnRbXSB7XG4gICAgICBjb25zdCB0b0ludiA9IEZwLmludmVydEJhdGNoKHBvaW50cy5tYXAoKHApID0+IHAucHopKTtcbiAgICAgIHJldHVybiBwb2ludHMubWFwKChwLCBpKSA9PiBwLnRvQWZmaW5lKHRvSW52W2ldKSkubWFwKFBvaW50LmZyb21BZmZpbmUpO1xuICAgIH1cblxuICAgIC8qKlxuICAgICAqIENvbnZlcnRzIGhhc2ggc3RyaW5nIG9yIFVpbnQ4QXJyYXkgdG8gUG9pbnQuXG4gICAgICogQHBhcmFtIGhleCBzaG9ydC9sb25nIEVDRFNBIGhleFxuICAgICAqL1xuICAgIHN0YXRpYyBmcm9tSGV4KGhleDogSGV4KTogUG9pbnQge1xuICAgICAgY29uc3QgUCA9IFBvaW50LmZyb21BZmZpbmUoZnJvbUJ5dGVzKGVuc3VyZUJ5dGVzKCdwb2ludEhleCcsIGhleCkpKTtcbiAgICAgIFAuYXNzZXJ0VmFsaWRpdHkoKTtcbiAgICAgIHJldHVybiBQO1xuICAgIH1cblxuICAgIC8vIE11bHRpcGxpZXMgZ2VuZXJhdG9yIHBvaW50IGJ5IHByaXZhdGVLZXkuXG4gICAgc3RhdGljIGZyb21Qcml2YXRlS2V5KHByaXZhdGVLZXk6IFByaXZLZXkpIHtcbiAgICAgIHJldHVybiBQb2ludC5CQVNFLm11bHRpcGx5KG5vcm1Qcml2YXRlS2V5VG9TY2FsYXIocHJpdmF0ZUtleSkpO1xuICAgIH1cblxuICAgIC8vIFdlIGNhbGN1bGF0ZSBwcmVjb21wdXRlcyBmb3IgZWxsaXB0aWMgY3VydmUgcG9pbnQgbXVsdGlwbGljYXRpb25cbiAgICAvLyB1c2luZyB3aW5kb3dlZCBtZXRob2QuIFRoaXMgc3BlY2lmaWVzIHdpbmRvdyBzaXplIGFuZFxuICAgIC8vIHN0b3JlcyBwcmVjb21wdXRlZCB2YWx1ZXMuIFVzdWFsbHkgb25seSBiYXNlIHBvaW50IHdvdWxkIGJlIHByZWNvbXB1dGVkLlxuICAgIF9XSU5ET1dfU0laRT86IG51bWJlcjtcblxuICAgIC8vIFwiUHJpdmF0ZSBtZXRob2RcIiwgZG9uJ3QgdXNlIGl0IGRpcmVjdGx5XG4gICAgX3NldFdpbmRvd1NpemUod2luZG93U2l6ZTogbnVtYmVyKSB7XG4gICAgICB0aGlzLl9XSU5ET1dfU0laRSA9IHdpbmRvd1NpemU7XG4gICAgICBwb2ludFByZWNvbXB1dGVzLmRlbGV0ZSh0aGlzKTtcbiAgICB9XG5cbiAgICAvLyBBIHBvaW50IG9uIGN1cnZlIGlzIHZhbGlkIGlmIGl0IGNvbmZvcm1zIHRvIGVxdWF0aW9uLlxuICAgIGFzc2VydFZhbGlkaXR5KCk6IHZvaWQge1xuICAgICAgaWYgKHRoaXMuaXMwKCkpIHtcbiAgICAgICAgLy8gKDAsIDEsIDApIGFrYSBaRVJPIGlzIGludmFsaWQgaW4gbW9zdCBjb250ZXh0cy5cbiAgICAgICAgLy8gSW4gQkxTLCBaRVJPIGNhbiBiZSBzZXJpYWxpemVkLCBzbyB3ZSBhbGxvdyBpdC5cbiAgICAgICAgLy8gKDAsIDAsIDApIGlzIHdyb25nIHJlcHJlc2VudGF0aW9uIG9mIFpFUk8gYW5kIGlzIGFsd2F5cyBpbnZhbGlkLlxuICAgICAgICBpZiAoQ1VSVkUuYWxsb3dJbmZpbml0eVBvaW50ICYmICFGcC5pczAodGhpcy5weSkpIHJldHVybjtcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdiYWQgcG9pbnQ6IFpFUk8nKTtcbiAgICAgIH1cbiAgICAgIC8vIFNvbWUgM3JkLXBhcnR5IHRlc3QgdmVjdG9ycyByZXF1aXJlIGRpZmZlcmVudCB3b3JkaW5nIGJldHdlZW4gaGVyZSAmIGBmcm9tQ29tcHJlc3NlZEhleGBcbiAgICAgIGNvbnN0IHsgeCwgeSB9ID0gdGhpcy50b0FmZmluZSgpO1xuICAgICAgLy8gQ2hlY2sgaWYgeCwgeSBhcmUgdmFsaWQgZmllbGQgZWxlbWVudHNcbiAgICAgIGlmICghRnAuaXNWYWxpZCh4KSB8fCAhRnAuaXNWYWxpZCh5KSkgdGhyb3cgbmV3IEVycm9yKCdiYWQgcG9pbnQ6IHggb3IgeSBub3QgRkUnKTtcbiAgICAgIGNvbnN0IGxlZnQgPSBGcC5zcXIoeSk7IC8vIHlcdTAwQjJcbiAgICAgIGNvbnN0IHJpZ2h0ID0gd2VpZXJzdHJhc3NFcXVhdGlvbih4KTsgLy8geFx1MDBCMyArIGF4ICsgYlxuICAgICAgaWYgKCFGcC5lcWwobGVmdCwgcmlnaHQpKSB0aHJvdyBuZXcgRXJyb3IoJ2JhZCBwb2ludDogZXF1YXRpb24gbGVmdCAhPSByaWdodCcpO1xuICAgICAgaWYgKCF0aGlzLmlzVG9yc2lvbkZyZWUoKSkgdGhyb3cgbmV3IEVycm9yKCdiYWQgcG9pbnQ6IG5vdCBpbiBwcmltZS1vcmRlciBzdWJncm91cCcpO1xuICAgIH1cbiAgICBoYXNFdmVuWSgpOiBib29sZWFuIHtcbiAgICAgIGNvbnN0IHsgeSB9ID0gdGhpcy50b0FmZmluZSgpO1xuICAgICAgaWYgKEZwLmlzT2RkKSByZXR1cm4gIUZwLmlzT2RkKHkpO1xuICAgICAgdGhyb3cgbmV3IEVycm9yKFwiRmllbGQgZG9lc24ndCBzdXBwb3J0IGlzT2RkXCIpO1xuICAgIH1cblxuICAgIC8qKlxuICAgICAqIENvbXBhcmUgb25lIHBvaW50IHRvIGFub3RoZXIuXG4gICAgICovXG4gICAgZXF1YWxzKG90aGVyOiBQb2ludCk6IGJvb2xlYW4ge1xuICAgICAgYXNzZXJ0UHJqUG9pbnQob3RoZXIpO1xuICAgICAgY29uc3QgeyBweDogWDEsIHB5OiBZMSwgcHo6IFoxIH0gPSB0aGlzO1xuICAgICAgY29uc3QgeyBweDogWDIsIHB5OiBZMiwgcHo6IFoyIH0gPSBvdGhlcjtcbiAgICAgIGNvbnN0IFUxID0gRnAuZXFsKEZwLm11bChYMSwgWjIpLCBGcC5tdWwoWDIsIFoxKSk7XG4gICAgICBjb25zdCBVMiA9IEZwLmVxbChGcC5tdWwoWTEsIFoyKSwgRnAubXVsKFkyLCBaMSkpO1xuICAgICAgcmV0dXJuIFUxICYmIFUyO1xuICAgIH1cblxuICAgIC8qKlxuICAgICAqIEZsaXBzIHBvaW50IHRvIG9uZSBjb3JyZXNwb25kaW5nIHRvICh4LCAteSkgaW4gQWZmaW5lIGNvb3JkaW5hdGVzLlxuICAgICAqL1xuICAgIG5lZ2F0ZSgpOiBQb2ludCB7XG4gICAgICByZXR1cm4gbmV3IFBvaW50KHRoaXMucHgsIEZwLm5lZyh0aGlzLnB5KSwgdGhpcy5weik7XG4gICAgfVxuXG4gICAgLy8gUmVuZXMtQ29zdGVsbG8tQmF0aW5hIGV4Y2VwdGlvbi1mcmVlIGRvdWJsaW5nIGZvcm11bGEuXG4gICAgLy8gVGhlcmUgaXMgMzAlIGZhc3RlciBKYWNvYmlhbiBmb3JtdWxhLCBidXQgaXQgaXMgbm90IGNvbXBsZXRlLlxuICAgIC8vIGh0dHBzOi8vZXByaW50LmlhY3Iub3JnLzIwMTUvMTA2MCwgYWxnb3JpdGhtIDNcbiAgICAvLyBDb3N0OiA4TSArIDNTICsgMyphICsgMipiMyArIDE1YWRkLlxuICAgIGRvdWJsZSgpIHtcbiAgICAgIGNvbnN0IHsgYSwgYiB9ID0gQ1VSVkU7XG4gICAgICBjb25zdCBiMyA9IEZwLm11bChiLCBfM24pO1xuICAgICAgY29uc3QgeyBweDogWDEsIHB5OiBZMSwgcHo6IFoxIH0gPSB0aGlzO1xuICAgICAgbGV0IFgzID0gRnAuWkVSTywgWTMgPSBGcC5aRVJPLCBaMyA9IEZwLlpFUk87IC8vIHByZXR0aWVyLWlnbm9yZVxuICAgICAgbGV0IHQwID0gRnAubXVsKFgxLCBYMSk7IC8vIHN0ZXAgMVxuICAgICAgbGV0IHQxID0gRnAubXVsKFkxLCBZMSk7XG4gICAgICBsZXQgdDIgPSBGcC5tdWwoWjEsIFoxKTtcbiAgICAgIGxldCB0MyA9IEZwLm11bChYMSwgWTEpO1xuICAgICAgdDMgPSBGcC5hZGQodDMsIHQzKTsgLy8gc3RlcCA1XG4gICAgICBaMyA9IEZwLm11bChYMSwgWjEpO1xuICAgICAgWjMgPSBGcC5hZGQoWjMsIFozKTtcbiAgICAgIFgzID0gRnAubXVsKGEsIFozKTtcbiAgICAgIFkzID0gRnAubXVsKGIzLCB0Mik7XG4gICAgICBZMyA9IEZwLmFkZChYMywgWTMpOyAvLyBzdGVwIDEwXG4gICAgICBYMyA9IEZwLnN1Yih0MSwgWTMpO1xuICAgICAgWTMgPSBGcC5hZGQodDEsIFkzKTtcbiAgICAgIFkzID0gRnAubXVsKFgzLCBZMyk7XG4gICAgICBYMyA9IEZwLm11bCh0MywgWDMpO1xuICAgICAgWjMgPSBGcC5tdWwoYjMsIFozKTsgLy8gc3RlcCAxNVxuICAgICAgdDIgPSBGcC5tdWwoYSwgdDIpO1xuICAgICAgdDMgPSBGcC5zdWIodDAsIHQyKTtcbiAgICAgIHQzID0gRnAubXVsKGEsIHQzKTtcbiAgICAgIHQzID0gRnAuYWRkKHQzLCBaMyk7XG4gICAgICBaMyA9IEZwLmFkZCh0MCwgdDApOyAvLyBzdGVwIDIwXG4gICAgICB0MCA9IEZwLmFkZChaMywgdDApO1xuICAgICAgdDAgPSBGcC5hZGQodDAsIHQyKTtcbiAgICAgIHQwID0gRnAubXVsKHQwLCB0Myk7XG4gICAgICBZMyA9IEZwLmFkZChZMywgdDApO1xuICAgICAgdDIgPSBGcC5tdWwoWTEsIFoxKTsgLy8gc3RlcCAyNVxuICAgICAgdDIgPSBGcC5hZGQodDIsIHQyKTtcbiAgICAgIHQwID0gRnAubXVsKHQyLCB0Myk7XG4gICAgICBYMyA9IEZwLnN1YihYMywgdDApO1xuICAgICAgWjMgPSBGcC5tdWwodDIsIHQxKTtcbiAgICAgIFozID0gRnAuYWRkKFozLCBaMyk7IC8vIHN0ZXAgMzBcbiAgICAgIFozID0gRnAuYWRkKFozLCBaMyk7XG4gICAgICByZXR1cm4gbmV3IFBvaW50KFgzLCBZMywgWjMpO1xuICAgIH1cblxuICAgIC8vIFJlbmVzLUNvc3RlbGxvLUJhdGluYSBleGNlcHRpb24tZnJlZSBhZGRpdGlvbiBmb3JtdWxhLlxuICAgIC8vIFRoZXJlIGlzIDMwJSBmYXN0ZXIgSmFjb2JpYW4gZm9ybXVsYSwgYnV0IGl0IGlzIG5vdCBjb21wbGV0ZS5cbiAgICAvLyBodHRwczovL2VwcmludC5pYWNyLm9yZy8yMDE1LzEwNjAsIGFsZ29yaXRobSAxXG4gICAgLy8gQ29zdDogMTJNICsgMFMgKyAzKmEgKyAzKmIzICsgMjNhZGQuXG4gICAgYWRkKG90aGVyOiBQb2ludCk6IFBvaW50IHtcbiAgICAgIGFzc2VydFByalBvaW50KG90aGVyKTtcbiAgICAgIGNvbnN0IHsgcHg6IFgxLCBweTogWTEsIHB6OiBaMSB9ID0gdGhpcztcbiAgICAgIGNvbnN0IHsgcHg6IFgyLCBweTogWTIsIHB6OiBaMiB9ID0gb3RoZXI7XG4gICAgICBsZXQgWDMgPSBGcC5aRVJPLCBZMyA9IEZwLlpFUk8sIFozID0gRnAuWkVSTzsgLy8gcHJldHRpZXItaWdub3JlXG4gICAgICBjb25zdCBhID0gQ1VSVkUuYTtcbiAgICAgIGNvbnN0IGIzID0gRnAubXVsKENVUlZFLmIsIF8zbik7XG4gICAgICBsZXQgdDAgPSBGcC5tdWwoWDEsIFgyKTsgLy8gc3RlcCAxXG4gICAgICBsZXQgdDEgPSBGcC5tdWwoWTEsIFkyKTtcbiAgICAgIGxldCB0MiA9IEZwLm11bChaMSwgWjIpO1xuICAgICAgbGV0IHQzID0gRnAuYWRkKFgxLCBZMSk7XG4gICAgICBsZXQgdDQgPSBGcC5hZGQoWDIsIFkyKTsgLy8gc3RlcCA1XG4gICAgICB0MyA9IEZwLm11bCh0MywgdDQpO1xuICAgICAgdDQgPSBGcC5hZGQodDAsIHQxKTtcbiAgICAgIHQzID0gRnAuc3ViKHQzLCB0NCk7XG4gICAgICB0NCA9IEZwLmFkZChYMSwgWjEpO1xuICAgICAgbGV0IHQ1ID0gRnAuYWRkKFgyLCBaMik7IC8vIHN0ZXAgMTBcbiAgICAgIHQ0ID0gRnAubXVsKHQ0LCB0NSk7XG4gICAgICB0NSA9IEZwLmFkZCh0MCwgdDIpO1xuICAgICAgdDQgPSBGcC5zdWIodDQsIHQ1KTtcbiAgICAgIHQ1ID0gRnAuYWRkKFkxLCBaMSk7XG4gICAgICBYMyA9IEZwLmFkZChZMiwgWjIpOyAvLyBzdGVwIDE1XG4gICAgICB0NSA9IEZwLm11bCh0NSwgWDMpO1xuICAgICAgWDMgPSBGcC5hZGQodDEsIHQyKTtcbiAgICAgIHQ1ID0gRnAuc3ViKHQ1LCBYMyk7XG4gICAgICBaMyA9IEZwLm11bChhLCB0NCk7XG4gICAgICBYMyA9IEZwLm11bChiMywgdDIpOyAvLyBzdGVwIDIwXG4gICAgICBaMyA9IEZwLmFkZChYMywgWjMpO1xuICAgICAgWDMgPSBGcC5zdWIodDEsIFozKTtcbiAgICAgIFozID0gRnAuYWRkKHQxLCBaMyk7XG4gICAgICBZMyA9IEZwLm11bChYMywgWjMpO1xuICAgICAgdDEgPSBGcC5hZGQodDAsIHQwKTsgLy8gc3RlcCAyNVxuICAgICAgdDEgPSBGcC5hZGQodDEsIHQwKTtcbiAgICAgIHQyID0gRnAubXVsKGEsIHQyKTtcbiAgICAgIHQ0ID0gRnAubXVsKGIzLCB0NCk7XG4gICAgICB0MSA9IEZwLmFkZCh0MSwgdDIpO1xuICAgICAgdDIgPSBGcC5zdWIodDAsIHQyKTsgLy8gc3RlcCAzMFxuICAgICAgdDIgPSBGcC5tdWwoYSwgdDIpO1xuICAgICAgdDQgPSBGcC5hZGQodDQsIHQyKTtcbiAgICAgIHQwID0gRnAubXVsKHQxLCB0NCk7XG4gICAgICBZMyA9IEZwLmFkZChZMywgdDApO1xuICAgICAgdDAgPSBGcC5tdWwodDUsIHQ0KTsgLy8gc3RlcCAzNVxuICAgICAgWDMgPSBGcC5tdWwodDMsIFgzKTtcbiAgICAgIFgzID0gRnAuc3ViKFgzLCB0MCk7XG4gICAgICB0MCA9IEZwLm11bCh0MywgdDEpO1xuICAgICAgWjMgPSBGcC5tdWwodDUsIFozKTtcbiAgICAgIFozID0gRnAuYWRkKFozLCB0MCk7IC8vIHN0ZXAgNDBcbiAgICAgIHJldHVybiBuZXcgUG9pbnQoWDMsIFkzLCBaMyk7XG4gICAgfVxuXG4gICAgc3VidHJhY3Qob3RoZXI6IFBvaW50KSB7XG4gICAgICByZXR1cm4gdGhpcy5hZGQob3RoZXIubmVnYXRlKCkpO1xuICAgIH1cblxuICAgIHByaXZhdGUgaXMwKCkge1xuICAgICAgcmV0dXJuIHRoaXMuZXF1YWxzKFBvaW50LlpFUk8pO1xuICAgIH1cbiAgICBwcml2YXRlIHdOQUYobjogYmlnaW50KTogeyBwOiBQb2ludDsgZjogUG9pbnQgfSB7XG4gICAgICByZXR1cm4gd25hZi53TkFGQ2FjaGVkKHRoaXMsIHBvaW50UHJlY29tcHV0ZXMsIG4sIChjb21wOiBQb2ludFtdKSA9PiB7XG4gICAgICAgIGNvbnN0IHRvSW52ID0gRnAuaW52ZXJ0QmF0Y2goY29tcC5tYXAoKHApID0+IHAucHopKTtcbiAgICAgICAgcmV0dXJuIGNvbXAubWFwKChwLCBpKSA9PiBwLnRvQWZmaW5lKHRvSW52W2ldKSkubWFwKFBvaW50LmZyb21BZmZpbmUpO1xuICAgICAgfSk7XG4gICAgfVxuXG4gICAgLyoqXG4gICAgICogTm9uLWNvbnN0YW50LXRpbWUgbXVsdGlwbGljYXRpb24uIFVzZXMgZG91YmxlLWFuZC1hZGQgYWxnb3JpdGhtLlxuICAgICAqIEl0J3MgZmFzdGVyLCBidXQgc2hvdWxkIG9ubHkgYmUgdXNlZCB3aGVuIHlvdSBkb24ndCBjYXJlIGFib3V0XG4gICAgICogYW4gZXhwb3NlZCBwcml2YXRlIGtleSBlLmcuIHNpZyB2ZXJpZmljYXRpb24sIHdoaWNoIHdvcmtzIG92ZXIgKnB1YmxpYyoga2V5cy5cbiAgICAgKi9cbiAgICBtdWx0aXBseVVuc2FmZShuOiBiaWdpbnQpOiBQb2ludCB7XG4gICAgICBjb25zdCBJID0gUG9pbnQuWkVSTztcbiAgICAgIGlmIChuID09PSBfMG4pIHJldHVybiBJO1xuICAgICAgYXNzZXJ0R0Uobik7IC8vIFdpbGwgdGhyb3cgb24gMFxuICAgICAgaWYgKG4gPT09IF8xbikgcmV0dXJuIHRoaXM7XG4gICAgICBjb25zdCB7IGVuZG8gfSA9IENVUlZFO1xuICAgICAgaWYgKCFlbmRvKSByZXR1cm4gd25hZi51bnNhZmVMYWRkZXIodGhpcywgbik7XG5cbiAgICAgIC8vIEFwcGx5IGVuZG9tb3JwaGlzbVxuICAgICAgbGV0IHsgazFuZWcsIGsxLCBrMm5lZywgazIgfSA9IGVuZG8uc3BsaXRTY2FsYXIobik7XG4gICAgICBsZXQgazFwID0gSTtcbiAgICAgIGxldCBrMnAgPSBJO1xuICAgICAgbGV0IGQ6IFBvaW50ID0gdGhpcztcbiAgICAgIHdoaWxlIChrMSA+IF8wbiB8fCBrMiA+IF8wbikge1xuICAgICAgICBpZiAoazEgJiBfMW4pIGsxcCA9IGsxcC5hZGQoZCk7XG4gICAgICAgIGlmIChrMiAmIF8xbikgazJwID0gazJwLmFkZChkKTtcbiAgICAgICAgZCA9IGQuZG91YmxlKCk7XG4gICAgICAgIGsxID4+PSBfMW47XG4gICAgICAgIGsyID4+PSBfMW47XG4gICAgICB9XG4gICAgICBpZiAoazFuZWcpIGsxcCA9IGsxcC5uZWdhdGUoKTtcbiAgICAgIGlmIChrMm5lZykgazJwID0gazJwLm5lZ2F0ZSgpO1xuICAgICAgazJwID0gbmV3IFBvaW50KEZwLm11bChrMnAucHgsIGVuZG8uYmV0YSksIGsycC5weSwgazJwLnB6KTtcbiAgICAgIHJldHVybiBrMXAuYWRkKGsycCk7XG4gICAgfVxuXG4gICAgLyoqXG4gICAgICogQ29uc3RhbnQgdGltZSBtdWx0aXBsaWNhdGlvbi5cbiAgICAgKiBVc2VzIHdOQUYgbWV0aG9kLiBXaW5kb3dlZCBtZXRob2QgbWF5IGJlIDEwJSBmYXN0ZXIsXG4gICAgICogYnV0IHRha2VzIDJ4IGxvbmdlciB0byBnZW5lcmF0ZSBhbmQgY29uc3VtZXMgMnggbWVtb3J5LlxuICAgICAqIFVzZXMgcHJlY29tcHV0ZXMgd2hlbiBhdmFpbGFibGUuXG4gICAgICogVXNlcyBlbmRvbW9ycGhpc20gZm9yIEtvYmxpdHogY3VydmVzLlxuICAgICAqIEBwYXJhbSBzY2FsYXIgYnkgd2hpY2ggdGhlIHBvaW50IHdvdWxkIGJlIG11bHRpcGxpZWRcbiAgICAgKiBAcmV0dXJucyBOZXcgcG9pbnRcbiAgICAgKi9cbiAgICBtdWx0aXBseShzY2FsYXI6IGJpZ2ludCk6IFBvaW50IHtcbiAgICAgIGFzc2VydEdFKHNjYWxhcik7XG4gICAgICBsZXQgbiA9IHNjYWxhcjtcbiAgICAgIGxldCBwb2ludDogUG9pbnQsIGZha2U6IFBvaW50OyAvLyBGYWtlIHBvaW50IGlzIHVzZWQgdG8gY29uc3QtdGltZSBtdWx0XG4gICAgICBjb25zdCB7IGVuZG8gfSA9IENVUlZFO1xuICAgICAgaWYgKGVuZG8pIHtcbiAgICAgICAgY29uc3QgeyBrMW5lZywgazEsIGsybmVnLCBrMiB9ID0gZW5kby5zcGxpdFNjYWxhcihuKTtcbiAgICAgICAgbGV0IHsgcDogazFwLCBmOiBmMXAgfSA9IHRoaXMud05BRihrMSk7XG4gICAgICAgIGxldCB7IHA6IGsycCwgZjogZjJwIH0gPSB0aGlzLndOQUYoazIpO1xuICAgICAgICBrMXAgPSB3bmFmLmNvbnN0VGltZU5lZ2F0ZShrMW5lZywgazFwKTtcbiAgICAgICAgazJwID0gd25hZi5jb25zdFRpbWVOZWdhdGUoazJuZWcsIGsycCk7XG4gICAgICAgIGsycCA9IG5ldyBQb2ludChGcC5tdWwoazJwLnB4LCBlbmRvLmJldGEpLCBrMnAucHksIGsycC5weik7XG4gICAgICAgIHBvaW50ID0gazFwLmFkZChrMnApO1xuICAgICAgICBmYWtlID0gZjFwLmFkZChmMnApO1xuICAgICAgfSBlbHNlIHtcbiAgICAgICAgY29uc3QgeyBwLCBmIH0gPSB0aGlzLndOQUYobik7XG4gICAgICAgIHBvaW50ID0gcDtcbiAgICAgICAgZmFrZSA9IGY7XG4gICAgICB9XG4gICAgICAvLyBOb3JtYWxpemUgYHpgIGZvciBib3RoIHBvaW50cywgYnV0IHJldHVybiBvbmx5IHJlYWwgb25lXG4gICAgICByZXR1cm4gUG9pbnQubm9ybWFsaXplWihbcG9pbnQsIGZha2VdKVswXTtcbiAgICB9XG5cbiAgICAvKipcbiAgICAgKiBFZmZpY2llbnRseSBjYWxjdWxhdGUgYGFQICsgYlFgLiBVbnNhZmUsIGNhbiBleHBvc2UgcHJpdmF0ZSBrZXksIGlmIHVzZWQgaW5jb3JyZWN0bHkuXG4gICAgICogTm90IHVzaW5nIFN0cmF1c3MtU2hhbWlyIHRyaWNrOiBwcmVjb21wdXRhdGlvbiB0YWJsZXMgYXJlIGZhc3Rlci5cbiAgICAgKiBUaGUgdHJpY2sgY291bGQgYmUgdXNlZnVsIGlmIGJvdGggUCBhbmQgUSBhcmUgbm90IEcgKG5vdCBpbiBvdXIgY2FzZSkuXG4gICAgICogQHJldHVybnMgbm9uLXplcm8gYWZmaW5lIHBvaW50XG4gICAgICovXG4gICAgbXVsdGlwbHlBbmRBZGRVbnNhZmUoUTogUG9pbnQsIGE6IGJpZ2ludCwgYjogYmlnaW50KTogUG9pbnQgfCB1bmRlZmluZWQge1xuICAgICAgY29uc3QgRyA9IFBvaW50LkJBU0U7IC8vIE5vIFN0cmF1c3MtU2hhbWlyIHRyaWNrOiB3ZSBoYXZlIDEwJSBmYXN0ZXIgRyBwcmVjb21wdXRlc1xuICAgICAgY29uc3QgbXVsID0gKFxuICAgICAgICBQOiBQb2ludCxcbiAgICAgICAgYTogYmlnaW50IC8vIFNlbGVjdCBmYXN0ZXIgbXVsdGlwbHkoKSBtZXRob2RcbiAgICAgICkgPT4gKGEgPT09IF8wbiB8fCBhID09PSBfMW4gfHwgIVAuZXF1YWxzKEcpID8gUC5tdWx0aXBseVVuc2FmZShhKSA6IFAubXVsdGlwbHkoYSkpO1xuICAgICAgY29uc3Qgc3VtID0gbXVsKHRoaXMsIGEpLmFkZChtdWwoUSwgYikpO1xuICAgICAgcmV0dXJuIHN1bS5pczAoKSA/IHVuZGVmaW5lZCA6IHN1bTtcbiAgICB9XG5cbiAgICAvLyBDb252ZXJ0cyBQcm9qZWN0aXZlIHBvaW50IHRvIGFmZmluZSAoeCwgeSkgY29vcmRpbmF0ZXMuXG4gICAgLy8gQ2FuIGFjY2VwdCBwcmVjb21wdXRlZCBaXi0xIC0gZm9yIGV4YW1wbGUsIGZyb20gaW52ZXJ0QmF0Y2guXG4gICAgLy8gKHgsIHksIHopIFx1MjIwQiAoeD14L3osIHk9eS96KVxuICAgIHRvQWZmaW5lKGl6PzogVCk6IEFmZmluZVBvaW50PFQ+IHtcbiAgICAgIGNvbnN0IHsgcHg6IHgsIHB5OiB5LCBwejogeiB9ID0gdGhpcztcbiAgICAgIGNvbnN0IGlzMCA9IHRoaXMuaXMwKCk7XG4gICAgICAvLyBJZiBpbnZaIHdhcyAwLCB3ZSByZXR1cm4gemVybyBwb2ludC4gSG93ZXZlciB3ZSBzdGlsbCB3YW50IHRvIGV4ZWN1dGVcbiAgICAgIC8vIGFsbCBvcGVyYXRpb25zLCBzbyB3ZSByZXBsYWNlIGludlogd2l0aCBhIHJhbmRvbSBudW1iZXIsIDEuXG4gICAgICBpZiAoaXogPT0gbnVsbCkgaXogPSBpczAgPyBGcC5PTkUgOiBGcC5pbnYoeik7XG4gICAgICBjb25zdCBheCA9IEZwLm11bCh4LCBpeik7XG4gICAgICBjb25zdCBheSA9IEZwLm11bCh5LCBpeik7XG4gICAgICBjb25zdCB6eiA9IEZwLm11bCh6LCBpeik7XG4gICAgICBpZiAoaXMwKSByZXR1cm4geyB4OiBGcC5aRVJPLCB5OiBGcC5aRVJPIH07XG4gICAgICBpZiAoIUZwLmVxbCh6eiwgRnAuT05FKSkgdGhyb3cgbmV3IEVycm9yKCdpbnZaIHdhcyBpbnZhbGlkJyk7XG4gICAgICByZXR1cm4geyB4OiBheCwgeTogYXkgfTtcbiAgICB9XG4gICAgaXNUb3JzaW9uRnJlZSgpOiBib29sZWFuIHtcbiAgICAgIGNvbnN0IHsgaDogY29mYWN0b3IsIGlzVG9yc2lvbkZyZWUgfSA9IENVUlZFO1xuICAgICAgaWYgKGNvZmFjdG9yID09PSBfMW4pIHJldHVybiB0cnVlOyAvLyBObyBzdWJncm91cHMsIGFsd2F5cyB0b3JzaW9uLWZyZWVcbiAgICAgIGlmIChpc1RvcnNpb25GcmVlKSByZXR1cm4gaXNUb3JzaW9uRnJlZShQb2ludCwgdGhpcyk7XG4gICAgICB0aHJvdyBuZXcgRXJyb3IoJ2lzVG9yc2lvbkZyZWUoKSBoYXMgbm90IGJlZW4gZGVjbGFyZWQgZm9yIHRoZSBlbGxpcHRpYyBjdXJ2ZScpO1xuICAgIH1cbiAgICBjbGVhckNvZmFjdG9yKCk6IFBvaW50IHtcbiAgICAgIGNvbnN0IHsgaDogY29mYWN0b3IsIGNsZWFyQ29mYWN0b3IgfSA9IENVUlZFO1xuICAgICAgaWYgKGNvZmFjdG9yID09PSBfMW4pIHJldHVybiB0aGlzOyAvLyBGYXN0LXBhdGhcbiAgICAgIGlmIChjbGVhckNvZmFjdG9yKSByZXR1cm4gY2xlYXJDb2ZhY3RvcihQb2ludCwgdGhpcykgYXMgUG9pbnQ7XG4gICAgICByZXR1cm4gdGhpcy5tdWx0aXBseVVuc2FmZShDVVJWRS5oKTtcbiAgICB9XG5cbiAgICB0b1Jhd0J5dGVzKGlzQ29tcHJlc3NlZCA9IHRydWUpOiBVaW50OEFycmF5IHtcbiAgICAgIHRoaXMuYXNzZXJ0VmFsaWRpdHkoKTtcbiAgICAgIHJldHVybiB0b0J5dGVzKFBvaW50LCB0aGlzLCBpc0NvbXByZXNzZWQpO1xuICAgIH1cblxuICAgIHRvSGV4KGlzQ29tcHJlc3NlZCA9IHRydWUpOiBzdHJpbmcge1xuICAgICAgcmV0dXJuIHV0LmJ5dGVzVG9IZXgodGhpcy50b1Jhd0J5dGVzKGlzQ29tcHJlc3NlZCkpO1xuICAgIH1cbiAgfVxuICBjb25zdCBfYml0cyA9IENVUlZFLm5CaXRMZW5ndGg7XG4gIGNvbnN0IHduYWYgPSB3TkFGKFBvaW50LCBDVVJWRS5lbmRvID8gTWF0aC5jZWlsKF9iaXRzIC8gMikgOiBfYml0cyk7XG4gIC8vIFZhbGlkYXRlIGlmIGdlbmVyYXRvciBwb2ludCBpcyBvbiBjdXJ2ZVxuICByZXR1cm4ge1xuICAgIENVUlZFLFxuICAgIFByb2plY3RpdmVQb2ludDogUG9pbnQgYXMgUHJvakNvbnN0cnVjdG9yPFQ+LFxuICAgIG5vcm1Qcml2YXRlS2V5VG9TY2FsYXIsXG4gICAgd2VpZXJzdHJhc3NFcXVhdGlvbixcbiAgICBpc1dpdGhpbkN1cnZlT3JkZXIsXG4gIH07XG59XG5cbi8vIEluc3RhbmNlXG5leHBvcnQgaW50ZXJmYWNlIFNpZ25hdHVyZVR5cGUge1xuICByZWFkb25seSByOiBiaWdpbnQ7XG4gIHJlYWRvbmx5IHM6IGJpZ2ludDtcbiAgcmVhZG9ubHkgcmVjb3Zlcnk/OiBudW1iZXI7XG4gIGFzc2VydFZhbGlkaXR5KCk6IHZvaWQ7XG4gIGFkZFJlY292ZXJ5Qml0KHJlY292ZXJ5OiBudW1iZXIpOiBSZWNvdmVyZWRTaWduYXR1cmVUeXBlO1xuICBoYXNIaWdoUygpOiBib29sZWFuO1xuICBub3JtYWxpemVTKCk6IFNpZ25hdHVyZVR5cGU7XG4gIHJlY292ZXJQdWJsaWNLZXkobXNnSGFzaDogSGV4KTogUHJvalBvaW50VHlwZTxiaWdpbnQ+O1xuICB0b0NvbXBhY3RSYXdCeXRlcygpOiBVaW50OEFycmF5O1xuICB0b0NvbXBhY3RIZXgoKTogc3RyaW5nO1xuICAvLyBERVItZW5jb2RlZFxuICB0b0RFUlJhd0J5dGVzKGlzQ29tcHJlc3NlZD86IGJvb2xlYW4pOiBVaW50OEFycmF5O1xuICB0b0RFUkhleChpc0NvbXByZXNzZWQ/OiBib29sZWFuKTogc3RyaW5nO1xufVxuZXhwb3J0IHR5cGUgUmVjb3ZlcmVkU2lnbmF0dXJlVHlwZSA9IFNpZ25hdHVyZVR5cGUgJiB7XG4gIHJlYWRvbmx5IHJlY292ZXJ5OiBudW1iZXI7XG59O1xuLy8gU3RhdGljIG1ldGhvZHNcbmV4cG9ydCB0eXBlIFNpZ25hdHVyZUNvbnN0cnVjdG9yID0ge1xuICBuZXcgKHI6IGJpZ2ludCwgczogYmlnaW50KTogU2lnbmF0dXJlVHlwZTtcbiAgZnJvbUNvbXBhY3QoaGV4OiBIZXgpOiBTaWduYXR1cmVUeXBlO1xuICBmcm9tREVSKGhleDogSGV4KTogU2lnbmF0dXJlVHlwZTtcbn07XG50eXBlIFNpZ25hdHVyZUxpa2UgPSB7IHI6IGJpZ2ludDsgczogYmlnaW50IH07XG5cbmV4cG9ydCB0eXBlIFB1YktleSA9IEhleCB8IFByb2pQb2ludFR5cGU8YmlnaW50PjtcblxuZXhwb3J0IHR5cGUgQ3VydmVUeXBlID0gQmFzaWNXQ3VydmU8YmlnaW50PiAmIHtcbiAgaGFzaDogQ0hhc2g7IC8vIENIYXNoIG5vdCBGSGFzaCBiZWNhdXNlIHdlIG5lZWQgb3V0cHV0TGVuIGZvciBEUkJHXG4gIGhtYWM6IEhtYWNGblN5bmM7XG4gIHJhbmRvbUJ5dGVzOiAoYnl0ZXNMZW5ndGg/OiBudW1iZXIpID0+IFVpbnQ4QXJyYXk7XG4gIGxvd1M/OiBib29sZWFuO1xuICBiaXRzMmludD86IChieXRlczogVWludDhBcnJheSkgPT4gYmlnaW50O1xuICBiaXRzMmludF9tb2ROPzogKGJ5dGVzOiBVaW50OEFycmF5KSA9PiBiaWdpbnQ7XG59O1xuXG5mdW5jdGlvbiB2YWxpZGF0ZU9wdHMoY3VydmU6IEN1cnZlVHlwZSkge1xuICBjb25zdCBvcHRzID0gdmFsaWRhdGVCYXNpYyhjdXJ2ZSk7XG4gIHV0LnZhbGlkYXRlT2JqZWN0KFxuICAgIG9wdHMsXG4gICAge1xuICAgICAgaGFzaDogJ2hhc2gnLFxuICAgICAgaG1hYzogJ2Z1bmN0aW9uJyxcbiAgICAgIHJhbmRvbUJ5dGVzOiAnZnVuY3Rpb24nLFxuICAgIH0sXG4gICAge1xuICAgICAgYml0czJpbnQ6ICdmdW5jdGlvbicsXG4gICAgICBiaXRzMmludF9tb2ROOiAnZnVuY3Rpb24nLFxuICAgICAgbG93UzogJ2Jvb2xlYW4nLFxuICAgIH1cbiAgKTtcbiAgcmV0dXJuIE9iamVjdC5mcmVlemUoeyBsb3dTOiB0cnVlLCAuLi5vcHRzIH0gYXMgY29uc3QpO1xufVxuXG5leHBvcnQgdHlwZSBDdXJ2ZUZuID0ge1xuICBDVVJWRTogUmV0dXJuVHlwZTx0eXBlb2YgdmFsaWRhdGVPcHRzPjtcbiAgZ2V0UHVibGljS2V5OiAocHJpdmF0ZUtleTogUHJpdktleSwgaXNDb21wcmVzc2VkPzogYm9vbGVhbikgPT4gVWludDhBcnJheTtcbiAgZ2V0U2hhcmVkU2VjcmV0OiAocHJpdmF0ZUE6IFByaXZLZXksIHB1YmxpY0I6IEhleCwgaXNDb21wcmVzc2VkPzogYm9vbGVhbikgPT4gVWludDhBcnJheTtcbiAgc2lnbjogKG1zZ0hhc2g6IEhleCwgcHJpdktleTogUHJpdktleSwgb3B0cz86IFNpZ25PcHRzKSA9PiBSZWNvdmVyZWRTaWduYXR1cmVUeXBlO1xuICB2ZXJpZnk6IChzaWduYXR1cmU6IEhleCB8IFNpZ25hdHVyZUxpa2UsIG1zZ0hhc2g6IEhleCwgcHVibGljS2V5OiBIZXgsIG9wdHM/OiBWZXJPcHRzKSA9PiBib29sZWFuO1xuICBQcm9qZWN0aXZlUG9pbnQ6IFByb2pDb25zdHJ1Y3RvcjxiaWdpbnQ+O1xuICBTaWduYXR1cmU6IFNpZ25hdHVyZUNvbnN0cnVjdG9yO1xuICB1dGlsczoge1xuICAgIG5vcm1Qcml2YXRlS2V5VG9TY2FsYXI6IChrZXk6IFByaXZLZXkpID0+IGJpZ2ludDtcbiAgICBpc1ZhbGlkUHJpdmF0ZUtleShwcml2YXRlS2V5OiBQcml2S2V5KTogYm9vbGVhbjtcbiAgICByYW5kb21Qcml2YXRlS2V5OiAoKSA9PiBVaW50OEFycmF5O1xuICAgIHByZWNvbXB1dGU6ICh3aW5kb3dTaXplPzogbnVtYmVyLCBwb2ludD86IFByb2pQb2ludFR5cGU8YmlnaW50PikgPT4gUHJvalBvaW50VHlwZTxiaWdpbnQ+O1xuICB9O1xufTtcblxuZXhwb3J0IGZ1bmN0aW9uIHdlaWVyc3RyYXNzKGN1cnZlRGVmOiBDdXJ2ZVR5cGUpOiBDdXJ2ZUZuIHtcbiAgY29uc3QgQ1VSVkUgPSB2YWxpZGF0ZU9wdHMoY3VydmVEZWYpIGFzIFJldHVyblR5cGU8dHlwZW9mIHZhbGlkYXRlT3B0cz47XG4gIGNvbnN0IHsgRnAsIG46IENVUlZFX09SREVSIH0gPSBDVVJWRTtcbiAgY29uc3QgY29tcHJlc3NlZExlbiA9IEZwLkJZVEVTICsgMTsgLy8gZS5nLiAzMyBmb3IgMzJcbiAgY29uc3QgdW5jb21wcmVzc2VkTGVuID0gMiAqIEZwLkJZVEVTICsgMTsgLy8gZS5nLiA2NSBmb3IgMzJcblxuICBmdW5jdGlvbiBpc1ZhbGlkRmllbGRFbGVtZW50KG51bTogYmlnaW50KTogYm9vbGVhbiB7XG4gICAgcmV0dXJuIF8wbiA8IG51bSAmJiBudW0gPCBGcC5PUkRFUjsgLy8gMCBpcyBiYW5uZWQgc2luY2UgaXQncyBub3QgaW52ZXJ0aWJsZSBGRVxuICB9XG4gIGZ1bmN0aW9uIG1vZE4oYTogYmlnaW50KSB7XG4gICAgcmV0dXJuIG1vZC5tb2QoYSwgQ1VSVkVfT1JERVIpO1xuICB9XG4gIGZ1bmN0aW9uIGludk4oYTogYmlnaW50KSB7XG4gICAgcmV0dXJuIG1vZC5pbnZlcnQoYSwgQ1VSVkVfT1JERVIpO1xuICB9XG5cbiAgY29uc3Qge1xuICAgIFByb2plY3RpdmVQb2ludDogUG9pbnQsXG4gICAgbm9ybVByaXZhdGVLZXlUb1NjYWxhcixcbiAgICB3ZWllcnN0cmFzc0VxdWF0aW9uLFxuICAgIGlzV2l0aGluQ3VydmVPcmRlcixcbiAgfSA9IHdlaWVyc3RyYXNzUG9pbnRzKHtcbiAgICAuLi5DVVJWRSxcbiAgICB0b0J5dGVzKF9jLCBwb2ludCwgaXNDb21wcmVzc2VkOiBib29sZWFuKTogVWludDhBcnJheSB7XG4gICAgICBjb25zdCBhID0gcG9pbnQudG9BZmZpbmUoKTtcbiAgICAgIGNvbnN0IHggPSBGcC50b0J5dGVzKGEueCk7XG4gICAgICBjb25zdCBjYXQgPSB1dC5jb25jYXRCeXRlcztcbiAgICAgIGlmIChpc0NvbXByZXNzZWQpIHtcbiAgICAgICAgcmV0dXJuIGNhdChVaW50OEFycmF5LmZyb20oW3BvaW50Lmhhc0V2ZW5ZKCkgPyAweDAyIDogMHgwM10pLCB4KTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHJldHVybiBjYXQoVWludDhBcnJheS5mcm9tKFsweDA0XSksIHgsIEZwLnRvQnl0ZXMoYS55KSk7XG4gICAgICB9XG4gICAgfSxcbiAgICBmcm9tQnl0ZXMoYnl0ZXM6IFVpbnQ4QXJyYXkpIHtcbiAgICAgIGNvbnN0IGxlbiA9IGJ5dGVzLmxlbmd0aDtcbiAgICAgIGNvbnN0IGhlYWQgPSBieXRlc1swXTtcbiAgICAgIGNvbnN0IHRhaWwgPSBieXRlcy5zdWJhcnJheSgxKTtcbiAgICAgIC8vIHRoaXMuYXNzZXJ0VmFsaWRpdHkoKSBpcyBkb25lIGluc2lkZSBvZiBmcm9tSGV4XG4gICAgICBpZiAobGVuID09PSBjb21wcmVzc2VkTGVuICYmIChoZWFkID09PSAweDAyIHx8IGhlYWQgPT09IDB4MDMpKSB7XG4gICAgICAgIGNvbnN0IHggPSB1dC5ieXRlc1RvTnVtYmVyQkUodGFpbCk7XG4gICAgICAgIGlmICghaXNWYWxpZEZpZWxkRWxlbWVudCh4KSkgdGhyb3cgbmV3IEVycm9yKCdQb2ludCBpcyBub3Qgb24gY3VydmUnKTtcbiAgICAgICAgY29uc3QgeTIgPSB3ZWllcnN0cmFzc0VxdWF0aW9uKHgpOyAvLyB5XHUwMEIyID0geFx1MDBCMyArIGF4ICsgYlxuICAgICAgICBsZXQgeSA9IEZwLnNxcnQoeTIpOyAvLyB5ID0geVx1MDBCMiBeIChwKzEpLzRcbiAgICAgICAgY29uc3QgaXNZT2RkID0gKHkgJiBfMW4pID09PSBfMW47XG4gICAgICAgIC8vIEVDRFNBXG4gICAgICAgIGNvbnN0IGlzSGVhZE9kZCA9IChoZWFkICYgMSkgPT09IDE7XG4gICAgICAgIGlmIChpc0hlYWRPZGQgIT09IGlzWU9kZCkgeSA9IEZwLm5lZyh5KTtcbiAgICAgICAgcmV0dXJuIHsgeCwgeSB9O1xuICAgICAgfSBlbHNlIGlmIChsZW4gPT09IHVuY29tcHJlc3NlZExlbiAmJiBoZWFkID09PSAweDA0KSB7XG4gICAgICAgIGNvbnN0IHggPSBGcC5mcm9tQnl0ZXModGFpbC5zdWJhcnJheSgwLCBGcC5CWVRFUykpO1xuICAgICAgICBjb25zdCB5ID0gRnAuZnJvbUJ5dGVzKHRhaWwuc3ViYXJyYXkoRnAuQllURVMsIDIgKiBGcC5CWVRFUykpO1xuICAgICAgICByZXR1cm4geyB4LCB5IH07XG4gICAgICB9IGVsc2Uge1xuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXG4gICAgICAgICAgYFBvaW50IG9mIGxlbmd0aCAke2xlbn0gd2FzIGludmFsaWQuIEV4cGVjdGVkICR7Y29tcHJlc3NlZExlbn0gY29tcHJlc3NlZCBieXRlcyBvciAke3VuY29tcHJlc3NlZExlbn0gdW5jb21wcmVzc2VkIGJ5dGVzYFxuICAgICAgICApO1xuICAgICAgfVxuICAgIH0sXG4gIH0pO1xuICBjb25zdCBudW1Ub05CeXRlU3RyID0gKG51bTogYmlnaW50KTogc3RyaW5nID0+XG4gICAgdXQuYnl0ZXNUb0hleCh1dC5udW1iZXJUb0J5dGVzQkUobnVtLCBDVVJWRS5uQnl0ZUxlbmd0aCkpO1xuXG4gIGZ1bmN0aW9uIGlzQmlnZ2VyVGhhbkhhbGZPcmRlcihudW1iZXI6IGJpZ2ludCkge1xuICAgIGNvbnN0IEhBTEYgPSBDVVJWRV9PUkRFUiA+PiBfMW47XG4gICAgcmV0dXJuIG51bWJlciA+IEhBTEY7XG4gIH1cblxuICBmdW5jdGlvbiBub3JtYWxpemVTKHM6IGJpZ2ludCkge1xuICAgIHJldHVybiBpc0JpZ2dlclRoYW5IYWxmT3JkZXIocykgPyBtb2ROKC1zKSA6IHM7XG4gIH1cbiAgLy8gc2xpY2UgYnl0ZXMgbnVtXG4gIGNvbnN0IHNsY051bSA9IChiOiBVaW50OEFycmF5LCBmcm9tOiBudW1iZXIsIHRvOiBudW1iZXIpID0+IHV0LmJ5dGVzVG9OdW1iZXJCRShiLnNsaWNlKGZyb20sIHRvKSk7XG5cbiAgLyoqXG4gICAqIEVDRFNBIHNpZ25hdHVyZSB3aXRoIGl0cyAociwgcykgcHJvcGVydGllcy4gU3VwcG9ydHMgREVSICYgY29tcGFjdCByZXByZXNlbnRhdGlvbnMuXG4gICAqL1xuICBjbGFzcyBTaWduYXR1cmUgaW1wbGVtZW50cyBTaWduYXR1cmVUeXBlIHtcbiAgICBjb25zdHJ1Y3RvcihyZWFkb25seSByOiBiaWdpbnQsIHJlYWRvbmx5IHM6IGJpZ2ludCwgcmVhZG9ubHkgcmVjb3Zlcnk/OiBudW1iZXIpIHtcbiAgICAgIHRoaXMuYXNzZXJ0VmFsaWRpdHkoKTtcbiAgICB9XG5cbiAgICAvLyBwYWlyIChieXRlcyBvZiByLCBieXRlcyBvZiBzKVxuICAgIHN0YXRpYyBmcm9tQ29tcGFjdChoZXg6IEhleCkge1xuICAgICAgY29uc3QgbCA9IENVUlZFLm5CeXRlTGVuZ3RoO1xuICAgICAgaGV4ID0gZW5zdXJlQnl0ZXMoJ2NvbXBhY3RTaWduYXR1cmUnLCBoZXgsIGwgKiAyKTtcbiAgICAgIHJldHVybiBuZXcgU2lnbmF0dXJlKHNsY051bShoZXgsIDAsIGwpLCBzbGNOdW0oaGV4LCBsLCAyICogbCkpO1xuICAgIH1cblxuICAgIC8vIERFUiBlbmNvZGVkIEVDRFNBIHNpZ25hdHVyZVxuICAgIC8vIGh0dHBzOi8vYml0Y29pbi5zdGFja2V4Y2hhbmdlLmNvbS9xdWVzdGlvbnMvNTc2NDQvd2hhdC1hcmUtdGhlLXBhcnRzLW9mLWEtYml0Y29pbi10cmFuc2FjdGlvbi1pbnB1dC1zY3JpcHRcbiAgICBzdGF0aWMgZnJvbURFUihoZXg6IEhleCkge1xuICAgICAgY29uc3QgeyByLCBzIH0gPSBERVIudG9TaWcoZW5zdXJlQnl0ZXMoJ0RFUicsIGhleCkpO1xuICAgICAgcmV0dXJuIG5ldyBTaWduYXR1cmUociwgcyk7XG4gICAgfVxuXG4gICAgYXNzZXJ0VmFsaWRpdHkoKTogdm9pZCB7XG4gICAgICAvLyBjYW4gdXNlIGFzc2VydEdFIGhlcmVcbiAgICAgIGlmICghaXNXaXRoaW5DdXJ2ZU9yZGVyKHRoaXMucikpIHRocm93IG5ldyBFcnJvcignciBtdXN0IGJlIDAgPCByIDwgQ1VSVkUubicpO1xuICAgICAgaWYgKCFpc1dpdGhpbkN1cnZlT3JkZXIodGhpcy5zKSkgdGhyb3cgbmV3IEVycm9yKCdzIG11c3QgYmUgMCA8IHMgPCBDVVJWRS5uJyk7XG4gICAgfVxuXG4gICAgYWRkUmVjb3ZlcnlCaXQocmVjb3Zlcnk6IG51bWJlcik6IFJlY292ZXJlZFNpZ25hdHVyZSB7XG4gICAgICByZXR1cm4gbmV3IFNpZ25hdHVyZSh0aGlzLnIsIHRoaXMucywgcmVjb3ZlcnkpIGFzIFJlY292ZXJlZFNpZ25hdHVyZTtcbiAgICB9XG5cbiAgICByZWNvdmVyUHVibGljS2V5KG1zZ0hhc2g6IEhleCk6IHR5cGVvZiBQb2ludC5CQVNFIHtcbiAgICAgIGNvbnN0IHsgciwgcywgcmVjb3Zlcnk6IHJlYyB9ID0gdGhpcztcbiAgICAgIGNvbnN0IGggPSBiaXRzMmludF9tb2ROKGVuc3VyZUJ5dGVzKCdtc2dIYXNoJywgbXNnSGFzaCkpOyAvLyBUcnVuY2F0ZSBoYXNoXG4gICAgICBpZiAocmVjID09IG51bGwgfHwgIVswLCAxLCAyLCAzXS5pbmNsdWRlcyhyZWMpKSB0aHJvdyBuZXcgRXJyb3IoJ3JlY292ZXJ5IGlkIGludmFsaWQnKTtcbiAgICAgIGNvbnN0IHJhZGogPSByZWMgPT09IDIgfHwgcmVjID09PSAzID8gciArIENVUlZFLm4gOiByO1xuICAgICAgaWYgKHJhZGogPj0gRnAuT1JERVIpIHRocm93IG5ldyBFcnJvcigncmVjb3ZlcnkgaWQgMiBvciAzIGludmFsaWQnKTtcbiAgICAgIGNvbnN0IHByZWZpeCA9IChyZWMgJiAxKSA9PT0gMCA/ICcwMicgOiAnMDMnO1xuICAgICAgY29uc3QgUiA9IFBvaW50LmZyb21IZXgocHJlZml4ICsgbnVtVG9OQnl0ZVN0cihyYWRqKSk7XG4gICAgICBjb25zdCBpciA9IGludk4ocmFkaik7IC8vIHJeLTFcbiAgICAgIGNvbnN0IHUxID0gbW9kTigtaCAqIGlyKTsgLy8gLWhyXi0xXG4gICAgICBjb25zdCB1MiA9IG1vZE4ocyAqIGlyKTsgLy8gc3JeLTFcbiAgICAgIGNvbnN0IFEgPSBQb2ludC5CQVNFLm11bHRpcGx5QW5kQWRkVW5zYWZlKFIsIHUxLCB1Mik7IC8vIChzcl4tMSlSLShocl4tMSlHID0gLShocl4tMSlHICsgKHNyXi0xKVxuICAgICAgaWYgKCFRKSB0aHJvdyBuZXcgRXJyb3IoJ3BvaW50IGF0IGluZmluaWZ5Jyk7IC8vIHVuc2FmZSBpcyBmaW5lOiBubyBwcml2IGRhdGEgbGVha2VkXG4gICAgICBRLmFzc2VydFZhbGlkaXR5KCk7XG4gICAgICByZXR1cm4gUTtcbiAgICB9XG5cbiAgICAvLyBTaWduYXR1cmVzIHNob3VsZCBiZSBsb3ctcywgdG8gcHJldmVudCBtYWxsZWFiaWxpdHkuXG4gICAgaGFzSGlnaFMoKTogYm9vbGVhbiB7XG4gICAgICByZXR1cm4gaXNCaWdnZXJUaGFuSGFsZk9yZGVyKHRoaXMucyk7XG4gICAgfVxuXG4gICAgbm9ybWFsaXplUygpIHtcbiAgICAgIHJldHVybiB0aGlzLmhhc0hpZ2hTKCkgPyBuZXcgU2lnbmF0dXJlKHRoaXMuciwgbW9kTigtdGhpcy5zKSwgdGhpcy5yZWNvdmVyeSkgOiB0aGlzO1xuICAgIH1cblxuICAgIC8vIERFUi1lbmNvZGVkXG4gICAgdG9ERVJSYXdCeXRlcygpIHtcbiAgICAgIHJldHVybiB1dC5oZXhUb0J5dGVzKHRoaXMudG9ERVJIZXgoKSk7XG4gICAgfVxuICAgIHRvREVSSGV4KCkge1xuICAgICAgcmV0dXJuIERFUi5oZXhGcm9tU2lnKHsgcjogdGhpcy5yLCBzOiB0aGlzLnMgfSk7XG4gICAgfVxuXG4gICAgLy8gcGFkZGVkIGJ5dGVzIG9mIHIsIHRoZW4gcGFkZGVkIGJ5dGVzIG9mIHNcbiAgICB0b0NvbXBhY3RSYXdCeXRlcygpIHtcbiAgICAgIHJldHVybiB1dC5oZXhUb0J5dGVzKHRoaXMudG9Db21wYWN0SGV4KCkpO1xuICAgIH1cbiAgICB0b0NvbXBhY3RIZXgoKSB7XG4gICAgICByZXR1cm4gbnVtVG9OQnl0ZVN0cih0aGlzLnIpICsgbnVtVG9OQnl0ZVN0cih0aGlzLnMpO1xuICAgIH1cbiAgfVxuICB0eXBlIFJlY292ZXJlZFNpZ25hdHVyZSA9IFNpZ25hdHVyZSAmIHsgcmVjb3Zlcnk6IG51bWJlciB9O1xuXG4gIGNvbnN0IHV0aWxzID0ge1xuICAgIGlzVmFsaWRQcml2YXRlS2V5KHByaXZhdGVLZXk6IFByaXZLZXkpIHtcbiAgICAgIHRyeSB7XG4gICAgICAgIG5vcm1Qcml2YXRlS2V5VG9TY2FsYXIocHJpdmF0ZUtleSk7XG4gICAgICAgIHJldHVybiB0cnVlO1xuICAgICAgfSBjYXRjaCAoZXJyb3IpIHtcbiAgICAgICAgcmV0dXJuIGZhbHNlO1xuICAgICAgfVxuICAgIH0sXG4gICAgbm9ybVByaXZhdGVLZXlUb1NjYWxhcjogbm9ybVByaXZhdGVLZXlUb1NjYWxhcixcblxuICAgIC8qKlxuICAgICAqIFByb2R1Y2VzIGNyeXB0b2dyYXBoaWNhbGx5IHNlY3VyZSBwcml2YXRlIGtleSBmcm9tIHJhbmRvbSBvZiBzaXplXG4gICAgICogKGdyb3VwTGVuICsgY2VpbChncm91cExlbiAvIDIpKSB3aXRoIG1vZHVsbyBiaWFzIGJlaW5nIG5lZ2xpZ2libGUuXG4gICAgICovXG4gICAgcmFuZG9tUHJpdmF0ZUtleTogKCk6IFVpbnQ4QXJyYXkgPT4ge1xuICAgICAgY29uc3QgbGVuZ3RoID0gbW9kLmdldE1pbkhhc2hMZW5ndGgoQ1VSVkUubik7XG4gICAgICByZXR1cm4gbW9kLm1hcEhhc2hUb0ZpZWxkKENVUlZFLnJhbmRvbUJ5dGVzKGxlbmd0aCksIENVUlZFLm4pO1xuICAgIH0sXG5cbiAgICAvKipcbiAgICAgKiBDcmVhdGVzIHByZWNvbXB1dGUgdGFibGUgZm9yIGFuIGFyYml0cmFyeSBFQyBwb2ludC4gTWFrZXMgcG9pbnQgXCJjYWNoZWRcIi5cbiAgICAgKiBBbGxvd3MgdG8gbWFzc2l2ZWx5IHNwZWVkLXVwIGBwb2ludC5tdWx0aXBseShzY2FsYXIpYC5cbiAgICAgKiBAcmV0dXJucyBjYWNoZWQgcG9pbnRcbiAgICAgKiBAZXhhbXBsZVxuICAgICAqIGNvbnN0IGZhc3QgPSB1dGlscy5wcmVjb21wdXRlKDgsIFByb2plY3RpdmVQb2ludC5mcm9tSGV4KHNvbWVvbmVzUHViS2V5KSk7XG4gICAgICogZmFzdC5tdWx0aXBseShwcml2S2V5KTsgLy8gbXVjaCBmYXN0ZXIgRUNESCBub3dcbiAgICAgKi9cbiAgICBwcmVjb21wdXRlKHdpbmRvd1NpemUgPSA4LCBwb2ludCA9IFBvaW50LkJBU0UpOiB0eXBlb2YgUG9pbnQuQkFTRSB7XG4gICAgICBwb2ludC5fc2V0V2luZG93U2l6ZSh3aW5kb3dTaXplKTtcbiAgICAgIHBvaW50Lm11bHRpcGx5KEJpZ0ludCgzKSk7IC8vIDMgaXMgYXJiaXRyYXJ5LCBqdXN0IG5lZWQgYW55IG51bWJlciBoZXJlXG4gICAgICByZXR1cm4gcG9pbnQ7XG4gICAgfSxcbiAgfTtcblxuICAvKipcbiAgICogQ29tcHV0ZXMgcHVibGljIGtleSBmb3IgYSBwcml2YXRlIGtleS4gQ2hlY2tzIGZvciB2YWxpZGl0eSBvZiB0aGUgcHJpdmF0ZSBrZXkuXG4gICAqIEBwYXJhbSBwcml2YXRlS2V5IHByaXZhdGUga2V5XG4gICAqIEBwYXJhbSBpc0NvbXByZXNzZWQgd2hldGhlciB0byByZXR1cm4gY29tcGFjdCAoZGVmYXVsdCksIG9yIGZ1bGwga2V5XG4gICAqIEByZXR1cm5zIFB1YmxpYyBrZXksIGZ1bGwgd2hlbiBpc0NvbXByZXNzZWQ9ZmFsc2U7IHNob3J0IHdoZW4gaXNDb21wcmVzc2VkPXRydWVcbiAgICovXG4gIGZ1bmN0aW9uIGdldFB1YmxpY0tleShwcml2YXRlS2V5OiBQcml2S2V5LCBpc0NvbXByZXNzZWQgPSB0cnVlKTogVWludDhBcnJheSB7XG4gICAgcmV0dXJuIFBvaW50LmZyb21Qcml2YXRlS2V5KHByaXZhdGVLZXkpLnRvUmF3Qnl0ZXMoaXNDb21wcmVzc2VkKTtcbiAgfVxuXG4gIC8qKlxuICAgKiBRdWljayBhbmQgZGlydHkgY2hlY2sgZm9yIGl0ZW0gYmVpbmcgcHVibGljIGtleS4gRG9lcyBub3QgdmFsaWRhdGUgaGV4LCBvciBiZWluZyBvbi1jdXJ2ZS5cbiAgICovXG4gIGZ1bmN0aW9uIGlzUHJvYlB1YihpdGVtOiBQcml2S2V5IHwgUHViS2V5KTogYm9vbGVhbiB7XG4gICAgY29uc3QgYXJyID0gaXRlbSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXk7XG4gICAgY29uc3Qgc3RyID0gdHlwZW9mIGl0ZW0gPT09ICdzdHJpbmcnO1xuICAgIGNvbnN0IGxlbiA9IChhcnIgfHwgc3RyKSAmJiAoaXRlbSBhcyBIZXgpLmxlbmd0aDtcbiAgICBpZiAoYXJyKSByZXR1cm4gbGVuID09PSBjb21wcmVzc2VkTGVuIHx8IGxlbiA9PT0gdW5jb21wcmVzc2VkTGVuO1xuICAgIGlmIChzdHIpIHJldHVybiBsZW4gPT09IDIgKiBjb21wcmVzc2VkTGVuIHx8IGxlbiA9PT0gMiAqIHVuY29tcHJlc3NlZExlbjtcbiAgICBpZiAoaXRlbSBpbnN0YW5jZW9mIFBvaW50KSByZXR1cm4gdHJ1ZTtcbiAgICByZXR1cm4gZmFsc2U7XG4gIH1cblxuICAvKipcbiAgICogRUNESCAoRWxsaXB0aWMgQ3VydmUgRGlmZmllIEhlbGxtYW4pLlxuICAgKiBDb21wdXRlcyBzaGFyZWQgcHVibGljIGtleSBmcm9tIHByaXZhdGUga2V5IGFuZCBwdWJsaWMga2V5LlxuICAgKiBDaGVja3M6IDEpIHByaXZhdGUga2V5IHZhbGlkaXR5IDIpIHNoYXJlZCBrZXkgaXMgb24tY3VydmUuXG4gICAqIERvZXMgTk9UIGhhc2ggdGhlIHJlc3VsdC5cbiAgICogQHBhcmFtIHByaXZhdGVBIHByaXZhdGUga2V5XG4gICAqIEBwYXJhbSBwdWJsaWNCIGRpZmZlcmVudCBwdWJsaWMga2V5XG4gICAqIEBwYXJhbSBpc0NvbXByZXNzZWQgd2hldGhlciB0byByZXR1cm4gY29tcGFjdCAoZGVmYXVsdCksIG9yIGZ1bGwga2V5XG4gICAqIEByZXR1cm5zIHNoYXJlZCBwdWJsaWMga2V5XG4gICAqL1xuICBmdW5jdGlvbiBnZXRTaGFyZWRTZWNyZXQocHJpdmF0ZUE6IFByaXZLZXksIHB1YmxpY0I6IEhleCwgaXNDb21wcmVzc2VkID0gdHJ1ZSk6IFVpbnQ4QXJyYXkge1xuICAgIGlmIChpc1Byb2JQdWIocHJpdmF0ZUEpKSB0aHJvdyBuZXcgRXJyb3IoJ2ZpcnN0IGFyZyBtdXN0IGJlIHByaXZhdGUga2V5Jyk7XG4gICAgaWYgKCFpc1Byb2JQdWIocHVibGljQikpIHRocm93IG5ldyBFcnJvcignc2Vjb25kIGFyZyBtdXN0IGJlIHB1YmxpYyBrZXknKTtcbiAgICBjb25zdCBiID0gUG9pbnQuZnJvbUhleChwdWJsaWNCKTsgLy8gY2hlY2sgZm9yIGJlaW5nIG9uLWN1cnZlXG4gICAgcmV0dXJuIGIubXVsdGlwbHkobm9ybVByaXZhdGVLZXlUb1NjYWxhcihwcml2YXRlQSkpLnRvUmF3Qnl0ZXMoaXNDb21wcmVzc2VkKTtcbiAgfVxuXG4gIC8vIFJGQzY5Nzk6IGVuc3VyZSBFQ0RTQSBtc2cgaXMgWCBieXRlcyBhbmQgPCBOLiBSRkMgc3VnZ2VzdHMgb3B0aW9uYWwgdHJ1bmNhdGluZyB2aWEgYml0czJvY3RldHMuXG4gIC8vIEZJUFMgMTg2LTQgNC42IHN1Z2dlc3RzIHRoZSBsZWZ0bW9zdCBtaW4obkJpdExlbiwgb3V0TGVuKSBiaXRzLCB3aGljaCBtYXRjaGVzIGJpdHMyaW50LlxuICAvLyBiaXRzMmludCBjYW4gcHJvZHVjZSByZXM+Tiwgd2UgY2FuIGRvIG1vZChyZXMsIE4pIHNpbmNlIHRoZSBiaXRMZW4gaXMgdGhlIHNhbWUuXG4gIC8vIGludDJvY3RldHMgY2FuJ3QgYmUgdXNlZDsgcGFkcyBzbWFsbCBtc2dzIHdpdGggMDogdW5hY2NlcHRhdGJsZSBmb3IgdHJ1bmMgYXMgcGVyIFJGQyB2ZWN0b3JzXG4gIGNvbnN0IGJpdHMyaW50ID1cbiAgICBDVVJWRS5iaXRzMmludCB8fFxuICAgIGZ1bmN0aW9uIChieXRlczogVWludDhBcnJheSk6IGJpZ2ludCB7XG4gICAgICAvLyBGb3IgY3VydmVzIHdpdGggbkJpdExlbmd0aCAlIDggIT09IDA6IGJpdHMyb2N0ZXRzKGJpdHMyb2N0ZXRzKG0pKSAhPT0gYml0czJvY3RldHMobSlcbiAgICAgIC8vIGZvciBzb21lIGNhc2VzLCBzaW5jZSBieXRlcy5sZW5ndGggKiA4IGlzIG5vdCBhY3R1YWwgYml0TGVuZ3RoLlxuICAgICAgY29uc3QgbnVtID0gdXQuYnl0ZXNUb051bWJlckJFKGJ5dGVzKTsgLy8gY2hlY2sgZm9yID09IHU4IGRvbmUgaGVyZVxuICAgICAgY29uc3QgZGVsdGEgPSBieXRlcy5sZW5ndGggKiA4IC0gQ1VSVkUubkJpdExlbmd0aDsgLy8gdHJ1bmNhdGUgdG8gbkJpdExlbmd0aCBsZWZ0bW9zdCBiaXRzXG4gICAgICByZXR1cm4gZGVsdGEgPiAwID8gbnVtID4+IEJpZ0ludChkZWx0YSkgOiBudW07XG4gICAgfTtcbiAgY29uc3QgYml0czJpbnRfbW9kTiA9XG4gICAgQ1VSVkUuYml0czJpbnRfbW9kTiB8fFxuICAgIGZ1bmN0aW9uIChieXRlczogVWludDhBcnJheSk6IGJpZ2ludCB7XG4gICAgICByZXR1cm4gbW9kTihiaXRzMmludChieXRlcykpOyAvLyBjYW4ndCB1c2UgYnl0ZXNUb051bWJlckJFIGhlcmVcbiAgICB9O1xuICAvLyBOT1RFOiBwYWRzIG91dHB1dCB3aXRoIHplcm8gYXMgcGVyIHNwZWNcbiAgY29uc3QgT1JERVJfTUFTSyA9IHV0LmJpdE1hc2soQ1VSVkUubkJpdExlbmd0aCk7XG4gIC8qKlxuICAgKiBDb252ZXJ0cyB0byBieXRlcy4gQ2hlY2tzIGlmIG51bSBpbiBgWzAuLk9SREVSX01BU0stMV1gIGUuZy46IGBbMC4uMl4yNTYtMV1gLlxuICAgKi9cbiAgZnVuY3Rpb24gaW50Mm9jdGV0cyhudW06IGJpZ2ludCk6IFVpbnQ4QXJyYXkge1xuICAgIGlmICh0eXBlb2YgbnVtICE9PSAnYmlnaW50JykgdGhyb3cgbmV3IEVycm9yKCdiaWdpbnQgZXhwZWN0ZWQnKTtcbiAgICBpZiAoIShfMG4gPD0gbnVtICYmIG51bSA8IE9SREVSX01BU0spKVxuICAgICAgdGhyb3cgbmV3IEVycm9yKGBiaWdpbnQgZXhwZWN0ZWQgPCAyXiR7Q1VSVkUubkJpdExlbmd0aH1gKTtcbiAgICAvLyB3b3JrcyB3aXRoIG9yZGVyLCBjYW4gaGF2ZSBkaWZmZXJlbnQgc2l6ZSB0aGFuIG51bVRvRmllbGQhXG4gICAgcmV0dXJuIHV0Lm51bWJlclRvQnl0ZXNCRShudW0sIENVUlZFLm5CeXRlTGVuZ3RoKTtcbiAgfVxuXG4gIC8vIFN0ZXBzIEEsIEQgb2YgUkZDNjk3OSAzLjJcbiAgLy8gQ3JlYXRlcyBSRkM2OTc5IHNlZWQ7IGNvbnZlcnRzIG1zZy9wcml2S2V5IHRvIG51bWJlcnMuXG4gIC8vIFVzZWQgb25seSBpbiBzaWduLCBub3QgaW4gdmVyaWZ5LlxuICAvLyBOT1RFOiB3ZSBjYW5ub3QgYXNzdW1lIGhlcmUgdGhhdCBtc2dIYXNoIGhhcyBzYW1lIGFtb3VudCBvZiBieXRlcyBhcyBjdXJ2ZSBvcmRlciwgdGhpcyB3aWxsIGJlIHdyb25nIGF0IGxlYXN0IGZvciBQNTIxLlxuICAvLyBBbHNvIGl0IGNhbiBiZSBiaWdnZXIgZm9yIFAyMjQgKyBTSEEyNTZcbiAgZnVuY3Rpb24gcHJlcFNpZyhtc2dIYXNoOiBIZXgsIHByaXZhdGVLZXk6IFByaXZLZXksIG9wdHMgPSBkZWZhdWx0U2lnT3B0cykge1xuICAgIGlmIChbJ3JlY292ZXJlZCcsICdjYW5vbmljYWwnXS5zb21lKChrKSA9PiBrIGluIG9wdHMpKVxuICAgICAgdGhyb3cgbmV3IEVycm9yKCdzaWduKCkgbGVnYWN5IG9wdGlvbnMgbm90IHN1cHBvcnRlZCcpO1xuICAgIGNvbnN0IHsgaGFzaCwgcmFuZG9tQnl0ZXMgfSA9IENVUlZFO1xuICAgIGxldCB7IGxvd1MsIHByZWhhc2gsIGV4dHJhRW50cm9weTogZW50IH0gPSBvcHRzOyAvLyBnZW5lcmF0ZXMgbG93LXMgc2lncyBieSBkZWZhdWx0XG4gICAgaWYgKGxvd1MgPT0gbnVsbCkgbG93UyA9IHRydWU7IC8vIFJGQzY5NzkgMy4yOiB3ZSBza2lwIHN0ZXAgQSwgYmVjYXVzZSB3ZSBhbHJlYWR5IHByb3ZpZGUgaGFzaFxuICAgIG1zZ0hhc2ggPSBlbnN1cmVCeXRlcygnbXNnSGFzaCcsIG1zZ0hhc2gpO1xuICAgIGlmIChwcmVoYXNoKSBtc2dIYXNoID0gZW5zdXJlQnl0ZXMoJ3ByZWhhc2hlZCBtc2dIYXNoJywgaGFzaChtc2dIYXNoKSk7XG5cbiAgICAvLyBXZSBjYW4ndCBsYXRlciBjYWxsIGJpdHMyb2N0ZXRzLCBzaW5jZSBuZXN0ZWQgYml0czJpbnQgaXMgYnJva2VuIGZvciBjdXJ2ZXNcbiAgICAvLyB3aXRoIG5CaXRMZW5ndGggJSA4ICE9PSAwLiBCZWNhdXNlIG9mIHRoYXQsIHdlIHVud3JhcCBpdCBoZXJlIGFzIGludDJvY3RldHMgY2FsbC5cbiAgICAvLyBjb25zdCBiaXRzMm9jdGV0cyA9IChiaXRzKSA9PiBpbnQyb2N0ZXRzKGJpdHMyaW50X21vZE4oYml0cykpXG4gICAgY29uc3QgaDFpbnQgPSBiaXRzMmludF9tb2ROKG1zZ0hhc2gpO1xuICAgIGNvbnN0IGQgPSBub3JtUHJpdmF0ZUtleVRvU2NhbGFyKHByaXZhdGVLZXkpOyAvLyB2YWxpZGF0ZSBwcml2YXRlIGtleSwgY29udmVydCB0byBiaWdpbnRcbiAgICBjb25zdCBzZWVkQXJncyA9IFtpbnQyb2N0ZXRzKGQpLCBpbnQyb2N0ZXRzKGgxaW50KV07XG4gICAgLy8gZXh0cmFFbnRyb3B5LiBSRkM2OTc5IDMuNjogYWRkaXRpb25hbCBrJyAob3B0aW9uYWwpLlxuICAgIGlmIChlbnQgIT0gbnVsbCkge1xuICAgICAgLy8gSyA9IEhNQUNfSyhWIHx8IDB4MDAgfHwgaW50Mm9jdGV0cyh4KSB8fCBiaXRzMm9jdGV0cyhoMSkgfHwgaycpXG4gICAgICBjb25zdCBlID0gZW50ID09PSB0cnVlID8gcmFuZG9tQnl0ZXMoRnAuQllURVMpIDogZW50OyAvLyBnZW5lcmF0ZSByYW5kb20gYnl0ZXMgT1IgcGFzcyBhcy1pc1xuICAgICAgc2VlZEFyZ3MucHVzaChlbnN1cmVCeXRlcygnZXh0cmFFbnRyb3B5JywgZSkpOyAvLyBjaGVjayBmb3IgYmVpbmcgYnl0ZXNcbiAgICB9XG4gICAgY29uc3Qgc2VlZCA9IHV0LmNvbmNhdEJ5dGVzKC4uLnNlZWRBcmdzKTsgLy8gU3RlcCBEIG9mIFJGQzY5NzkgMy4yXG4gICAgY29uc3QgbSA9IGgxaW50OyAvLyBOT1RFOiBubyBuZWVkIHRvIGNhbGwgYml0czJpbnQgc2Vjb25kIHRpbWUgaGVyZSwgaXQgaXMgaW5zaWRlIHRydW5jYXRlSGFzaCFcbiAgICAvLyBDb252ZXJ0cyBzaWduYXR1cmUgcGFyYW1zIGludG8gcG9pbnQgdyByL3MsIGNoZWNrcyByZXN1bHQgZm9yIHZhbGlkaXR5LlxuICAgIGZ1bmN0aW9uIGsyc2lnKGtCeXRlczogVWludDhBcnJheSk6IFJlY292ZXJlZFNpZ25hdHVyZSB8IHVuZGVmaW5lZCB7XG4gICAgICAvLyBSRkMgNjk3OSBTZWN0aW9uIDMuMiwgc3RlcCAzOiBrID0gYml0czJpbnQoVClcbiAgICAgIGNvbnN0IGsgPSBiaXRzMmludChrQnl0ZXMpOyAvLyBDYW5ub3QgdXNlIGZpZWxkcyBtZXRob2RzLCBzaW5jZSBpdCBpcyBncm91cCBlbGVtZW50XG4gICAgICBpZiAoIWlzV2l0aGluQ3VydmVPcmRlcihrKSkgcmV0dXJuOyAvLyBJbXBvcnRhbnQ6IGFsbCBtb2QoKSBjYWxscyBoZXJlIG11c3QgYmUgZG9uZSBvdmVyIE5cbiAgICAgIGNvbnN0IGlrID0gaW52TihrKTsgLy8ga14tMSBtb2QgblxuICAgICAgY29uc3QgcSA9IFBvaW50LkJBU0UubXVsdGlwbHkoaykudG9BZmZpbmUoKTsgLy8gcSA9IEdrXG4gICAgICBjb25zdCByID0gbW9kTihxLngpOyAvLyByID0gcS54IG1vZCBuXG4gICAgICBpZiAociA9PT0gXzBuKSByZXR1cm47XG4gICAgICAvLyBDYW4gdXNlIHNjYWxhciBibGluZGluZyBiXi0xKGJtICsgYmRyKSB3aGVyZSBiIFx1MjIwOCBbMSxxXHUyMjEyMV0gYWNjb3JkaW5nIHRvXG4gICAgICAvLyBodHRwczovL3RjaGVzLmlhY3Iub3JnL2luZGV4LnBocC9UQ0hFUy9hcnRpY2xlL3ZpZXcvNzMzNy82NTA5LiBXZSd2ZSBkZWNpZGVkIGFnYWluc3QgaXQ6XG4gICAgICAvLyBhKSBkZXBlbmRlbmN5IG9uIENTUFJORyBiKSAxNSUgc2xvd2Rvd24gYykgZG9lc24ndCByZWFsbHkgaGVscCBzaW5jZSBiaWdpbnRzIGFyZSBub3QgQ1RcbiAgICAgIGNvbnN0IHMgPSBtb2ROKGlrICogbW9kTihtICsgciAqIGQpKTsgLy8gTm90IHVzaW5nIGJsaW5kaW5nIGhlcmVcbiAgICAgIGlmIChzID09PSBfMG4pIHJldHVybjtcbiAgICAgIGxldCByZWNvdmVyeSA9IChxLnggPT09IHIgPyAwIDogMikgfCBOdW1iZXIocS55ICYgXzFuKTsgLy8gcmVjb3ZlcnkgYml0ICgyIG9yIDMsIHdoZW4gcS54ID4gbilcbiAgICAgIGxldCBub3JtUyA9IHM7XG4gICAgICBpZiAobG93UyAmJiBpc0JpZ2dlclRoYW5IYWxmT3JkZXIocykpIHtcbiAgICAgICAgbm9ybVMgPSBub3JtYWxpemVTKHMpOyAvLyBpZiBsb3dTIHdhcyBwYXNzZWQsIGVuc3VyZSBzIGlzIGFsd2F5c1xuICAgICAgICByZWNvdmVyeSBePSAxOyAvLyAvLyBpbiB0aGUgYm90dG9tIGhhbGYgb2YgTlxuICAgICAgfVxuICAgICAgcmV0dXJuIG5ldyBTaWduYXR1cmUociwgbm9ybVMsIHJlY292ZXJ5KSBhcyBSZWNvdmVyZWRTaWduYXR1cmU7IC8vIHVzZSBub3JtUywgbm90IHNcbiAgICB9XG4gICAgcmV0dXJuIHsgc2VlZCwgazJzaWcgfTtcbiAgfVxuICBjb25zdCBkZWZhdWx0U2lnT3B0czogU2lnbk9wdHMgPSB7IGxvd1M6IENVUlZFLmxvd1MsIHByZWhhc2g6IGZhbHNlIH07XG4gIGNvbnN0IGRlZmF1bHRWZXJPcHRzOiBWZXJPcHRzID0geyBsb3dTOiBDVVJWRS5sb3dTLCBwcmVoYXNoOiBmYWxzZSB9O1xuXG4gIC8qKlxuICAgKiBTaWducyBtZXNzYWdlIGhhc2ggd2l0aCBhIHByaXZhdGUga2V5LlxuICAgKiBgYGBcbiAgICogc2lnbihtLCBkLCBrKSB3aGVyZVxuICAgKiAgICh4LCB5KSA9IEcgXHUwMEQ3IGtcbiAgICogICByID0geCBtb2QgblxuICAgKiAgIHMgPSAobSArIGRyKS9rIG1vZCBuXG4gICAqIGBgYFxuICAgKiBAcGFyYW0gbXNnSGFzaCBOT1QgbWVzc2FnZS4gbXNnIG5lZWRzIHRvIGJlIGhhc2hlZCB0byBgbXNnSGFzaGAsIG9yIHVzZSBgcHJlaGFzaGAuXG4gICAqIEBwYXJhbSBwcml2S2V5IHByaXZhdGUga2V5XG4gICAqIEBwYXJhbSBvcHRzIGxvd1MgZm9yIG5vbi1tYWxsZWFibGUgc2lncy4gZXh0cmFFbnRyb3B5IGZvciBtaXhpbmcgcmFuZG9tbmVzcyBpbnRvIGsuIHByZWhhc2ggd2lsbCBoYXNoIGZpcnN0IGFyZy5cbiAgICogQHJldHVybnMgc2lnbmF0dXJlIHdpdGggcmVjb3ZlcnkgcGFyYW1cbiAgICovXG4gIGZ1bmN0aW9uIHNpZ24obXNnSGFzaDogSGV4LCBwcml2S2V5OiBQcml2S2V5LCBvcHRzID0gZGVmYXVsdFNpZ09wdHMpOiBSZWNvdmVyZWRTaWduYXR1cmUge1xuICAgIGNvbnN0IHsgc2VlZCwgazJzaWcgfSA9IHByZXBTaWcobXNnSGFzaCwgcHJpdktleSwgb3B0cyk7IC8vIFN0ZXBzIEEsIEQgb2YgUkZDNjk3OSAzLjIuXG4gICAgY29uc3QgQyA9IENVUlZFO1xuICAgIGNvbnN0IGRyYmcgPSB1dC5jcmVhdGVIbWFjRHJiZzxSZWNvdmVyZWRTaWduYXR1cmU+KEMuaGFzaC5vdXRwdXRMZW4sIEMubkJ5dGVMZW5ndGgsIEMuaG1hYyk7XG4gICAgcmV0dXJuIGRyYmcoc2VlZCwgazJzaWcpOyAvLyBTdGVwcyBCLCBDLCBELCBFLCBGLCBHXG4gIH1cblxuICAvLyBFbmFibGUgcHJlY29tcHV0ZXMuIFNsb3dzIGRvd24gZmlyc3QgcHVibGljS2V5IGNvbXB1dGF0aW9uIGJ5IDIwbXMuXG4gIFBvaW50LkJBU0UuX3NldFdpbmRvd1NpemUoOCk7XG4gIC8vIHV0aWxzLnByZWNvbXB1dGUoOCwgUHJvamVjdGl2ZVBvaW50LkJBU0UpXG5cbiAgLyoqXG4gICAqIFZlcmlmaWVzIGEgc2lnbmF0dXJlIGFnYWluc3QgbWVzc2FnZSBoYXNoIGFuZCBwdWJsaWMga2V5LlxuICAgKiBSZWplY3RzIGxvd1Mgc2lnbmF0dXJlcyBieSBkZWZhdWx0OiB0byBvdmVycmlkZSxcbiAgICogc3BlY2lmeSBvcHRpb24gYHtsb3dTOiBmYWxzZX1gLiBJbXBsZW1lbnRzIHNlY3Rpb24gNC4xLjQgZnJvbSBodHRwczovL3d3dy5zZWNnLm9yZy9zZWMxLXYyLnBkZjpcbiAgICpcbiAgICogYGBgXG4gICAqIHZlcmlmeShyLCBzLCBoLCBQKSB3aGVyZVxuICAgKiAgIFUxID0gaHNeLTEgbW9kIG5cbiAgICogICBVMiA9IHJzXi0xIG1vZCBuXG4gICAqICAgUiA9IFUxXHUyMkM1RyAtIFUyXHUyMkM1UFxuICAgKiAgIG1vZChSLngsIG4pID09IHJcbiAgICogYGBgXG4gICAqL1xuICBmdW5jdGlvbiB2ZXJpZnkoXG4gICAgc2lnbmF0dXJlOiBIZXggfCBTaWduYXR1cmVMaWtlLFxuICAgIG1zZ0hhc2g6IEhleCxcbiAgICBwdWJsaWNLZXk6IEhleCxcbiAgICBvcHRzID0gZGVmYXVsdFZlck9wdHNcbiAgKTogYm9vbGVhbiB7XG4gICAgY29uc3Qgc2cgPSBzaWduYXR1cmU7XG4gICAgbXNnSGFzaCA9IGVuc3VyZUJ5dGVzKCdtc2dIYXNoJywgbXNnSGFzaCk7XG4gICAgcHVibGljS2V5ID0gZW5zdXJlQnl0ZXMoJ3B1YmxpY0tleScsIHB1YmxpY0tleSk7XG4gICAgaWYgKCdzdHJpY3QnIGluIG9wdHMpIHRocm93IG5ldyBFcnJvcignb3B0aW9ucy5zdHJpY3Qgd2FzIHJlbmFtZWQgdG8gbG93UycpO1xuICAgIGNvbnN0IHsgbG93UywgcHJlaGFzaCB9ID0gb3B0cztcblxuICAgIGxldCBfc2lnOiBTaWduYXR1cmUgfCB1bmRlZmluZWQgPSB1bmRlZmluZWQ7XG4gICAgbGV0IFA6IFByb2pQb2ludFR5cGU8YmlnaW50PjtcbiAgICB0cnkge1xuICAgICAgaWYgKHR5cGVvZiBzZyA9PT0gJ3N0cmluZycgfHwgc2cgaW5zdGFuY2VvZiBVaW50OEFycmF5KSB7XG4gICAgICAgIC8vIFNpZ25hdHVyZSBjYW4gYmUgcmVwcmVzZW50ZWQgaW4gMiB3YXlzOiBjb21wYWN0ICgyKm5CeXRlTGVuZ3RoKSAmIERFUiAodmFyaWFibGUtbGVuZ3RoKS5cbiAgICAgICAgLy8gU2luY2UgREVSIGNhbiBhbHNvIGJlIDIqbkJ5dGVMZW5ndGggYnl0ZXMsIHdlIGNoZWNrIGZvciBpdCBmaXJzdC5cbiAgICAgICAgdHJ5IHtcbiAgICAgICAgICBfc2lnID0gU2lnbmF0dXJlLmZyb21ERVIoc2cpO1xuICAgICAgICB9IGNhdGNoIChkZXJFcnJvcikge1xuICAgICAgICAgIGlmICghKGRlckVycm9yIGluc3RhbmNlb2YgREVSLkVycikpIHRocm93IGRlckVycm9yO1xuICAgICAgICAgIF9zaWcgPSBTaWduYXR1cmUuZnJvbUNvbXBhY3Qoc2cpO1xuICAgICAgICB9XG4gICAgICB9IGVsc2UgaWYgKHR5cGVvZiBzZyA9PT0gJ29iamVjdCcgJiYgdHlwZW9mIHNnLnIgPT09ICdiaWdpbnQnICYmIHR5cGVvZiBzZy5zID09PSAnYmlnaW50Jykge1xuICAgICAgICBjb25zdCB7IHIsIHMgfSA9IHNnO1xuICAgICAgICBfc2lnID0gbmV3IFNpZ25hdHVyZShyLCBzKTtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIHRocm93IG5ldyBFcnJvcignUEFSU0UnKTtcbiAgICAgIH1cbiAgICAgIFAgPSBQb2ludC5mcm9tSGV4KHB1YmxpY0tleSk7XG4gICAgfSBjYXRjaCAoZXJyb3IpIHtcbiAgICAgIGlmICgoZXJyb3IgYXMgRXJyb3IpLm1lc3NhZ2UgPT09ICdQQVJTRScpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgc2lnbmF0dXJlIG11c3QgYmUgU2lnbmF0dXJlIGluc3RhbmNlLCBVaW50OEFycmF5IG9yIGhleCBzdHJpbmdgKTtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gICAgaWYgKGxvd1MgJiYgX3NpZy5oYXNIaWdoUygpKSByZXR1cm4gZmFsc2U7XG4gICAgaWYgKHByZWhhc2gpIG1zZ0hhc2ggPSBDVVJWRS5oYXNoKG1zZ0hhc2gpO1xuICAgIGNvbnN0IHsgciwgcyB9ID0gX3NpZztcbiAgICBjb25zdCBoID0gYml0czJpbnRfbW9kTihtc2dIYXNoKTsgLy8gQ2Fubm90IHVzZSBmaWVsZHMgbWV0aG9kcywgc2luY2UgaXQgaXMgZ3JvdXAgZWxlbWVudFxuICAgIGNvbnN0IGlzID0gaW52TihzKTsgLy8gc14tMVxuICAgIGNvbnN0IHUxID0gbW9kTihoICogaXMpOyAvLyB1MSA9IGhzXi0xIG1vZCBuXG4gICAgY29uc3QgdTIgPSBtb2ROKHIgKiBpcyk7IC8vIHUyID0gcnNeLTEgbW9kIG5cbiAgICBjb25zdCBSID0gUG9pbnQuQkFTRS5tdWx0aXBseUFuZEFkZFVuc2FmZShQLCB1MSwgdTIpPy50b0FmZmluZSgpOyAvLyBSID0gdTFcdTIyQzVHICsgdTJcdTIyQzVQXG4gICAgaWYgKCFSKSByZXR1cm4gZmFsc2U7XG4gICAgY29uc3QgdiA9IG1vZE4oUi54KTtcbiAgICByZXR1cm4gdiA9PT0gcjtcbiAgfVxuICByZXR1cm4ge1xuICAgIENVUlZFLFxuICAgIGdldFB1YmxpY0tleSxcbiAgICBnZXRTaGFyZWRTZWNyZXQsXG4gICAgc2lnbixcbiAgICB2ZXJpZnksXG4gICAgUHJvamVjdGl2ZVBvaW50OiBQb2ludCxcbiAgICBTaWduYXR1cmUsXG4gICAgdXRpbHMsXG4gIH07XG59XG5cbi8qKlxuICogSW1wbGVtZW50YXRpb24gb2YgdGhlIFNoYWxsdWUgYW5kIHZhbiBkZSBXb2VzdGlqbmUgbWV0aG9kIGZvciBhbnkgd2VpZXJzdHJhc3MgY3VydmUuXG4gKiBUT0RPOiBjaGVjayBpZiB0aGVyZSBpcyBhIHdheSB0byBtZXJnZSB0aGlzIHdpdGggdXZSYXRpbyBpbiBFZHdhcmRzOyBtb3ZlIHRvIG1vZHVsYXIuXG4gKiBiID0gVHJ1ZSBhbmQgeSA9IHNxcnQodSAvIHYpIGlmICh1IC8gdikgaXMgc3F1YXJlIGluIEYsIGFuZFxuICogYiA9IEZhbHNlIGFuZCB5ID0gc3FydChaICogKHUgLyB2KSkgb3RoZXJ3aXNlLlxuICogQHBhcmFtIEZwXG4gKiBAcGFyYW0gWlxuICogQHJldHVybnNcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIFNXVUZwU3FydFJhdGlvPFQ+KEZwOiBtb2QuSUZpZWxkPFQ+LCBaOiBUKSB7XG4gIC8vIEdlbmVyaWMgaW1wbGVtZW50YXRpb25cbiAgY29uc3QgcSA9IEZwLk9SREVSO1xuICBsZXQgbCA9IF8wbjtcbiAgZm9yIChsZXQgbyA9IHEgLSBfMW47IG8gJSBfMm4gPT09IF8wbjsgbyAvPSBfMm4pIGwgKz0gXzFuO1xuICBjb25zdCBjMSA9IGw7IC8vIDEuIGMxLCB0aGUgbGFyZ2VzdCBpbnRlZ2VyIHN1Y2ggdGhhdCAyXmMxIGRpdmlkZXMgcSAtIDEuXG4gIC8vIFdlIG5lZWQgMm4gKiogYzEgYW5kIDJuICoqIChjMS0xKS4gV2UgY2FuJ3QgdXNlICoqOyBidXQgd2UgY2FuIHVzZSA8PC5cbiAgLy8gMm4gKiogYzEgPT0gMm4gPDwgKGMxLTEpXG4gIGNvbnN0IF8ybl9wb3dfYzFfMSA9IF8ybiA8PCAoYzEgLSBfMW4gLSBfMW4pO1xuICBjb25zdCBfMm5fcG93X2MxID0gXzJuX3Bvd19jMV8xICogXzJuO1xuICBjb25zdCBjMiA9IChxIC0gXzFuKSAvIF8ybl9wb3dfYzE7IC8vIDIuIGMyID0gKHEgLSAxKSAvICgyXmMxKSAgIyBJbnRlZ2VyIGFyaXRobWV0aWNcbiAgY29uc3QgYzMgPSAoYzIgLSBfMW4pIC8gXzJuOyAvLyAzLiBjMyA9IChjMiAtIDEpIC8gMiAgICAgICAgICAgICMgSW50ZWdlciBhcml0aG1ldGljXG4gIGNvbnN0IGM0ID0gXzJuX3Bvd19jMSAtIF8xbjsgLy8gNC4gYzQgPSAyXmMxIC0gMSAgICAgICAgICAgICAgICAjIEludGVnZXIgYXJpdGhtZXRpY1xuICBjb25zdCBjNSA9IF8ybl9wb3dfYzFfMTsgLy8gNS4gYzUgPSAyXihjMSAtIDEpICAgICAgICAgICAgICAgICAgIyBJbnRlZ2VyIGFyaXRobWV0aWNcbiAgY29uc3QgYzYgPSBGcC5wb3coWiwgYzIpOyAvLyA2LiBjNiA9IFpeYzJcbiAgY29uc3QgYzcgPSBGcC5wb3coWiwgKGMyICsgXzFuKSAvIF8ybik7IC8vIDcuIGM3ID0gWl4oKGMyICsgMSkgLyAyKVxuICBsZXQgc3FydFJhdGlvID0gKHU6IFQsIHY6IFQpOiB7IGlzVmFsaWQ6IGJvb2xlYW47IHZhbHVlOiBUIH0gPT4ge1xuICAgIGxldCB0djEgPSBjNjsgLy8gMS4gdHYxID0gYzZcbiAgICBsZXQgdHYyID0gRnAucG93KHYsIGM0KTsgLy8gMi4gdHYyID0gdl5jNFxuICAgIGxldCB0djMgPSBGcC5zcXIodHYyKTsgLy8gMy4gdHYzID0gdHYyXjJcbiAgICB0djMgPSBGcC5tdWwodHYzLCB2KTsgLy8gNC4gdHYzID0gdHYzICogdlxuICAgIGxldCB0djUgPSBGcC5tdWwodSwgdHYzKTsgLy8gNS4gdHY1ID0gdSAqIHR2M1xuICAgIHR2NSA9IEZwLnBvdyh0djUsIGMzKTsgLy8gNi4gdHY1ID0gdHY1XmMzXG4gICAgdHY1ID0gRnAubXVsKHR2NSwgdHYyKTsgLy8gNy4gdHY1ID0gdHY1ICogdHYyXG4gICAgdHYyID0gRnAubXVsKHR2NSwgdik7IC8vIDguIHR2MiA9IHR2NSAqIHZcbiAgICB0djMgPSBGcC5tdWwodHY1LCB1KTsgLy8gOS4gdHYzID0gdHY1ICogdVxuICAgIGxldCB0djQgPSBGcC5tdWwodHYzLCB0djIpOyAvLyAxMC4gdHY0ID0gdHYzICogdHYyXG4gICAgdHY1ID0gRnAucG93KHR2NCwgYzUpOyAvLyAxMS4gdHY1ID0gdHY0XmM1XG4gICAgbGV0IGlzUVIgPSBGcC5lcWwodHY1LCBGcC5PTkUpOyAvLyAxMi4gaXNRUiA9IHR2NSA9PSAxXG4gICAgdHYyID0gRnAubXVsKHR2MywgYzcpOyAvLyAxMy4gdHYyID0gdHYzICogYzdcbiAgICB0djUgPSBGcC5tdWwodHY0LCB0djEpOyAvLyAxNC4gdHY1ID0gdHY0ICogdHYxXG4gICAgdHYzID0gRnAuY21vdih0djIsIHR2MywgaXNRUik7IC8vIDE1LiB0djMgPSBDTU9WKHR2MiwgdHYzLCBpc1FSKVxuICAgIHR2NCA9IEZwLmNtb3YodHY1LCB0djQsIGlzUVIpOyAvLyAxNi4gdHY0ID0gQ01PVih0djUsIHR2NCwgaXNRUilcbiAgICAvLyAxNy4gZm9yIGkgaW4gKGMxLCBjMSAtIDEsIC4uLiwgMik6XG4gICAgZm9yIChsZXQgaSA9IGMxOyBpID4gXzFuOyBpLS0pIHtcbiAgICAgIGxldCB0djUgPSBpIC0gXzJuOyAvLyAxOC4gICAgdHY1ID0gaSAtIDJcbiAgICAgIHR2NSA9IF8ybiA8PCAodHY1IC0gXzFuKTsgLy8gMTkuICAgIHR2NSA9IDJedHY1XG4gICAgICBsZXQgdHZ2NSA9IEZwLnBvdyh0djQsIHR2NSk7IC8vIDIwLiAgICB0djUgPSB0djRedHY1XG4gICAgICBjb25zdCBlMSA9IEZwLmVxbCh0dnY1LCBGcC5PTkUpOyAvLyAyMS4gICAgZTEgPSB0djUgPT0gMVxuICAgICAgdHYyID0gRnAubXVsKHR2MywgdHYxKTsgLy8gMjIuICAgIHR2MiA9IHR2MyAqIHR2MVxuICAgICAgdHYxID0gRnAubXVsKHR2MSwgdHYxKTsgLy8gMjMuICAgIHR2MSA9IHR2MSAqIHR2MVxuICAgICAgdHZ2NSA9IEZwLm11bCh0djQsIHR2MSk7IC8vIDI0LiAgICB0djUgPSB0djQgKiB0djFcbiAgICAgIHR2MyA9IEZwLmNtb3YodHYyLCB0djMsIGUxKTsgLy8gMjUuICAgIHR2MyA9IENNT1YodHYyLCB0djMsIGUxKVxuICAgICAgdHY0ID0gRnAuY21vdih0dnY1LCB0djQsIGUxKTsgLy8gMjYuICAgIHR2NCA9IENNT1YodHY1LCB0djQsIGUxKVxuICAgIH1cbiAgICByZXR1cm4geyBpc1ZhbGlkOiBpc1FSLCB2YWx1ZTogdHYzIH07XG4gIH07XG4gIGlmIChGcC5PUkRFUiAlIF80biA9PT0gXzNuKSB7XG4gICAgLy8gc3FydF9yYXRpb18zbW9kNCh1LCB2KVxuICAgIGNvbnN0IGMxID0gKEZwLk9SREVSIC0gXzNuKSAvIF80bjsgLy8gMS4gYzEgPSAocSAtIDMpIC8gNCAgICAgIyBJbnRlZ2VyIGFyaXRobWV0aWNcbiAgICBjb25zdCBjMiA9IEZwLnNxcnQoRnAubmVnKFopKTsgLy8gMi4gYzIgPSBzcXJ0KC1aKVxuICAgIHNxcnRSYXRpbyA9ICh1OiBULCB2OiBUKSA9PiB7XG4gICAgICBsZXQgdHYxID0gRnAuc3FyKHYpOyAvLyAxLiB0djEgPSB2XjJcbiAgICAgIGNvbnN0IHR2MiA9IEZwLm11bCh1LCB2KTsgLy8gMi4gdHYyID0gdSAqIHZcbiAgICAgIHR2MSA9IEZwLm11bCh0djEsIHR2Mik7IC8vIDMuIHR2MSA9IHR2MSAqIHR2MlxuICAgICAgbGV0IHkxID0gRnAucG93KHR2MSwgYzEpOyAvLyA0LiB5MSA9IHR2MV5jMVxuICAgICAgeTEgPSBGcC5tdWwoeTEsIHR2Mik7IC8vIDUuIHkxID0geTEgKiB0djJcbiAgICAgIGNvbnN0IHkyID0gRnAubXVsKHkxLCBjMik7IC8vIDYuIHkyID0geTEgKiBjMlxuICAgICAgY29uc3QgdHYzID0gRnAubXVsKEZwLnNxcih5MSksIHYpOyAvLyA3LiB0djMgPSB5MV4yOyA4LiB0djMgPSB0djMgKiB2XG4gICAgICBjb25zdCBpc1FSID0gRnAuZXFsKHR2MywgdSk7IC8vIDkuIGlzUVIgPSB0djMgPT0gdVxuICAgICAgbGV0IHkgPSBGcC5jbW92KHkyLCB5MSwgaXNRUik7IC8vIDEwLiB5ID0gQ01PVih5MiwgeTEsIGlzUVIpXG4gICAgICByZXR1cm4geyBpc1ZhbGlkOiBpc1FSLCB2YWx1ZTogeSB9OyAvLyAxMS4gcmV0dXJuIChpc1FSLCB5KSBpc1FSID8geSA6IHkqYzJcbiAgICB9O1xuICB9XG4gIC8vIE5vIGN1cnZlcyB1c2VzIHRoYXRcbiAgLy8gaWYgKEZwLk9SREVSICUgXzhuID09PSBfNW4pIC8vIHNxcnRfcmF0aW9fNW1vZDhcbiAgcmV0dXJuIHNxcnRSYXRpbztcbn1cbi8qKlxuICogU2ltcGxpZmllZCBTaGFsbHVlLXZhbiBkZSBXb2VzdGlqbmUtVWxhcyBNZXRob2RcbiAqIGh0dHBzOi8vd3d3LnJmYy1lZGl0b3Iub3JnL3JmYy9yZmM5MzgwI3NlY3Rpb24tNi42LjJcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIG1hcFRvQ3VydmVTaW1wbGVTV1U8VD4oXG4gIEZwOiBtb2QuSUZpZWxkPFQ+LFxuICBvcHRzOiB7XG4gICAgQTogVDtcbiAgICBCOiBUO1xuICAgIFo6IFQ7XG4gIH1cbikge1xuICBtb2QudmFsaWRhdGVGaWVsZChGcCk7XG4gIGlmICghRnAuaXNWYWxpZChvcHRzLkEpIHx8ICFGcC5pc1ZhbGlkKG9wdHMuQikgfHwgIUZwLmlzVmFsaWQob3B0cy5aKSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoJ21hcFRvQ3VydmVTaW1wbGVTV1U6IGludmFsaWQgb3B0cycpO1xuICBjb25zdCBzcXJ0UmF0aW8gPSBTV1VGcFNxcnRSYXRpbyhGcCwgb3B0cy5aKTtcbiAgaWYgKCFGcC5pc09kZCkgdGhyb3cgbmV3IEVycm9yKCdGcC5pc09kZCBpcyBub3QgaW1wbGVtZW50ZWQhJyk7XG4gIC8vIElucHV0OiB1LCBhbiBlbGVtZW50IG9mIEYuXG4gIC8vIE91dHB1dDogKHgsIHkpLCBhIHBvaW50IG9uIEUuXG4gIHJldHVybiAodTogVCk6IHsgeDogVDsgeTogVCB9ID0+IHtcbiAgICAvLyBwcmV0dGllci1pZ25vcmVcbiAgICBsZXQgdHYxLCB0djIsIHR2MywgdHY0LCB0djUsIHR2NiwgeCwgeTtcbiAgICB0djEgPSBGcC5zcXIodSk7IC8vIDEuICB0djEgPSB1XjJcbiAgICB0djEgPSBGcC5tdWwodHYxLCBvcHRzLlopOyAvLyAyLiAgdHYxID0gWiAqIHR2MVxuICAgIHR2MiA9IEZwLnNxcih0djEpOyAvLyAzLiAgdHYyID0gdHYxXjJcbiAgICB0djIgPSBGcC5hZGQodHYyLCB0djEpOyAvLyA0LiAgdHYyID0gdHYyICsgdHYxXG4gICAgdHYzID0gRnAuYWRkKHR2MiwgRnAuT05FKTsgLy8gNS4gIHR2MyA9IHR2MiArIDFcbiAgICB0djMgPSBGcC5tdWwodHYzLCBvcHRzLkIpOyAvLyA2LiAgdHYzID0gQiAqIHR2M1xuICAgIHR2NCA9IEZwLmNtb3Yob3B0cy5aLCBGcC5uZWcodHYyKSwgIUZwLmVxbCh0djIsIEZwLlpFUk8pKTsgLy8gNy4gIHR2NCA9IENNT1YoWiwgLXR2MiwgdHYyICE9IDApXG4gICAgdHY0ID0gRnAubXVsKHR2NCwgb3B0cy5BKTsgLy8gOC4gIHR2NCA9IEEgKiB0djRcbiAgICB0djIgPSBGcC5zcXIodHYzKTsgLy8gOS4gIHR2MiA9IHR2M14yXG4gICAgdHY2ID0gRnAuc3FyKHR2NCk7IC8vIDEwLiB0djYgPSB0djReMlxuICAgIHR2NSA9IEZwLm11bCh0djYsIG9wdHMuQSk7IC8vIDExLiB0djUgPSBBICogdHY2XG4gICAgdHYyID0gRnAuYWRkKHR2MiwgdHY1KTsgLy8gMTIuIHR2MiA9IHR2MiArIHR2NVxuICAgIHR2MiA9IEZwLm11bCh0djIsIHR2Myk7IC8vIDEzLiB0djIgPSB0djIgKiB0djNcbiAgICB0djYgPSBGcC5tdWwodHY2LCB0djQpOyAvLyAxNC4gdHY2ID0gdHY2ICogdHY0XG4gICAgdHY1ID0gRnAubXVsKHR2Niwgb3B0cy5CKTsgLy8gMTUuIHR2NSA9IEIgKiB0djZcbiAgICB0djIgPSBGcC5hZGQodHYyLCB0djUpOyAvLyAxNi4gdHYyID0gdHYyICsgdHY1XG4gICAgeCA9IEZwLm11bCh0djEsIHR2Myk7IC8vIDE3LiAgIHggPSB0djEgKiB0djNcbiAgICBjb25zdCB7IGlzVmFsaWQsIHZhbHVlIH0gPSBzcXJ0UmF0aW8odHYyLCB0djYpOyAvLyAxOC4gKGlzX2d4MV9zcXVhcmUsIHkxKSA9IHNxcnRfcmF0aW8odHYyLCB0djYpXG4gICAgeSA9IEZwLm11bCh0djEsIHUpOyAvLyAxOS4gICB5ID0gdHYxICogdSAgLT4gWiAqIHVeMyAqIHkxXG4gICAgeSA9IEZwLm11bCh5LCB2YWx1ZSk7IC8vIDIwLiAgIHkgPSB5ICogeTFcbiAgICB4ID0gRnAuY21vdih4LCB0djMsIGlzVmFsaWQpOyAvLyAyMS4gICB4ID0gQ01PVih4LCB0djMsIGlzX2d4MV9zcXVhcmUpXG4gICAgeSA9IEZwLmNtb3YoeSwgdmFsdWUsIGlzVmFsaWQpOyAvLyAyMi4gICB5ID0gQ01PVih5LCB5MSwgaXNfZ3gxX3NxdWFyZSlcbiAgICBjb25zdCBlMSA9IEZwLmlzT2RkISh1KSA9PT0gRnAuaXNPZGQhKHkpOyAvLyAyMy4gIGUxID0gc2duMCh1KSA9PSBzZ24wKHkpXG4gICAgeSA9IEZwLmNtb3YoRnAubmVnKHkpLCB5LCBlMSk7IC8vIDI0LiAgIHkgPSBDTU9WKC15LCB5LCBlMSlcbiAgICB4ID0gRnAuZGl2KHgsIHR2NCk7IC8vIDI1LiAgIHggPSB4IC8gdHY0XG4gICAgcmV0dXJuIHsgeCwgeSB9O1xuICB9O1xufVxuIiwgImltcG9ydCB7IGhhc2ggYXMgYXNzZXJ0SGFzaCwgYnl0ZXMgYXMgYXNzZXJ0Qnl0ZXMsIGV4aXN0cyBhcyBhc3NlcnRFeGlzdHMgfSBmcm9tICcuL19hc3NlcnQuanMnO1xuaW1wb3J0IHsgSGFzaCwgQ0hhc2gsIElucHV0LCB0b0J5dGVzIH0gZnJvbSAnLi91dGlscy5qcyc7XG4vLyBITUFDIChSRkMgMjEwNClcbmV4cG9ydCBjbGFzcyBITUFDPFQgZXh0ZW5kcyBIYXNoPFQ+PiBleHRlbmRzIEhhc2g8SE1BQzxUPj4ge1xuICBvSGFzaDogVDtcbiAgaUhhc2g6IFQ7XG4gIGJsb2NrTGVuOiBudW1iZXI7XG4gIG91dHB1dExlbjogbnVtYmVyO1xuICBwcml2YXRlIGZpbmlzaGVkID0gZmFsc2U7XG4gIHByaXZhdGUgZGVzdHJveWVkID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3IoaGFzaDogQ0hhc2gsIF9rZXk6IElucHV0KSB7XG4gICAgc3VwZXIoKTtcbiAgICBhc3NlcnRIYXNoKGhhc2gpO1xuICAgIGNvbnN0IGtleSA9IHRvQnl0ZXMoX2tleSk7XG4gICAgdGhpcy5pSGFzaCA9IGhhc2guY3JlYXRlKCkgYXMgVDtcbiAgICBpZiAodHlwZW9mIHRoaXMuaUhhc2gudXBkYXRlICE9PSAnZnVuY3Rpb24nKVxuICAgICAgdGhyb3cgbmV3IEVycm9yKCdFeHBlY3RlZCBpbnN0YW5jZSBvZiBjbGFzcyB3aGljaCBleHRlbmRzIHV0aWxzLkhhc2gnKTtcbiAgICB0aGlzLmJsb2NrTGVuID0gdGhpcy5pSGFzaC5ibG9ja0xlbjtcbiAgICB0aGlzLm91dHB1dExlbiA9IHRoaXMuaUhhc2gub3V0cHV0TGVuO1xuICAgIGNvbnN0IGJsb2NrTGVuID0gdGhpcy5ibG9ja0xlbjtcbiAgICBjb25zdCBwYWQgPSBuZXcgVWludDhBcnJheShibG9ja0xlbik7XG4gICAgLy8gYmxvY2tMZW4gY2FuIGJlIGJpZ2dlciB0aGFuIG91dHB1dExlblxuICAgIHBhZC5zZXQoa2V5Lmxlbmd0aCA+IGJsb2NrTGVuID8gaGFzaC5jcmVhdGUoKS51cGRhdGUoa2V5KS5kaWdlc3QoKSA6IGtleSk7XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCBwYWQubGVuZ3RoOyBpKyspIHBhZFtpXSBePSAweDM2O1xuICAgIHRoaXMuaUhhc2gudXBkYXRlKHBhZCk7XG4gICAgLy8gQnkgZG9pbmcgdXBkYXRlIChwcm9jZXNzaW5nIG9mIGZpcnN0IGJsb2NrKSBvZiBvdXRlciBoYXNoIGhlcmUgd2UgY2FuIHJlLXVzZSBpdCBiZXR3ZWVuIG11bHRpcGxlIGNhbGxzIHZpYSBjbG9uZVxuICAgIHRoaXMub0hhc2ggPSBoYXNoLmNyZWF0ZSgpIGFzIFQ7XG4gICAgLy8gVW5kbyBpbnRlcm5hbCBYT1IgJiYgYXBwbHkgb3V0ZXIgWE9SXG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCBwYWQubGVuZ3RoOyBpKyspIHBhZFtpXSBePSAweDM2IF4gMHg1YztcbiAgICB0aGlzLm9IYXNoLnVwZGF0ZShwYWQpO1xuICAgIHBhZC5maWxsKDApO1xuICB9XG4gIHVwZGF0ZShidWY6IElucHV0KSB7XG4gICAgYXNzZXJ0RXhpc3RzKHRoaXMpO1xuICAgIHRoaXMuaUhhc2gudXBkYXRlKGJ1Zik7XG4gICAgcmV0dXJuIHRoaXM7XG4gIH1cbiAgZGlnZXN0SW50byhvdXQ6IFVpbnQ4QXJyYXkpIHtcbiAgICBhc3NlcnRFeGlzdHModGhpcyk7XG4gICAgYXNzZXJ0Qnl0ZXMob3V0LCB0aGlzLm91dHB1dExlbik7XG4gICAgdGhpcy5maW5pc2hlZCA9IHRydWU7XG4gICAgdGhpcy5pSGFzaC5kaWdlc3RJbnRvKG91dCk7XG4gICAgdGhpcy5vSGFzaC51cGRhdGUob3V0KTtcbiAgICB0aGlzLm9IYXNoLmRpZ2VzdEludG8ob3V0KTtcbiAgICB0aGlzLmRlc3Ryb3koKTtcbiAgfVxuICBkaWdlc3QoKSB7XG4gICAgY29uc3Qgb3V0ID0gbmV3IFVpbnQ4QXJyYXkodGhpcy5vSGFzaC5vdXRwdXRMZW4pO1xuICAgIHRoaXMuZGlnZXN0SW50byhvdXQpO1xuICAgIHJldHVybiBvdXQ7XG4gIH1cbiAgX2Nsb25lSW50byh0bz86IEhNQUM8VD4pOiBITUFDPFQ+IHtcbiAgICAvLyBDcmVhdGUgbmV3IGluc3RhbmNlIHdpdGhvdXQgY2FsbGluZyBjb25zdHJ1Y3RvciBzaW5jZSBrZXkgYWxyZWFkeSBpbiBzdGF0ZSBhbmQgd2UgZG9uJ3Qga25vdyBpdC5cbiAgICB0byB8fD0gT2JqZWN0LmNyZWF0ZShPYmplY3QuZ2V0UHJvdG90eXBlT2YodGhpcyksIHt9KTtcbiAgICBjb25zdCB7IG9IYXNoLCBpSGFzaCwgZmluaXNoZWQsIGRlc3Ryb3llZCwgYmxvY2tMZW4sIG91dHB1dExlbiB9ID0gdGhpcztcbiAgICB0byA9IHRvIGFzIHRoaXM7XG4gICAgdG8uZmluaXNoZWQgPSBmaW5pc2hlZDtcbiAgICB0by5kZXN0cm95ZWQgPSBkZXN0cm95ZWQ7XG4gICAgdG8uYmxvY2tMZW4gPSBibG9ja0xlbjtcbiAgICB0by5vdXRwdXRMZW4gPSBvdXRwdXRMZW47XG4gICAgdG8ub0hhc2ggPSBvSGFzaC5fY2xvbmVJbnRvKHRvLm9IYXNoKTtcbiAgICB0by5pSGFzaCA9IGlIYXNoLl9jbG9uZUludG8odG8uaUhhc2gpO1xuICAgIHJldHVybiB0bztcbiAgfVxuICBkZXN0cm95KCkge1xuICAgIHRoaXMuZGVzdHJveWVkID0gdHJ1ZTtcbiAgICB0aGlzLm9IYXNoLmRlc3Ryb3koKTtcbiAgICB0aGlzLmlIYXNoLmRlc3Ryb3koKTtcbiAgfVxufVxuXG4vKipcbiAqIEhNQUM6IFJGQzIxMDQgbWVzc2FnZSBhdXRoZW50aWNhdGlvbiBjb2RlLlxuICogQHBhcmFtIGhhc2ggLSBmdW5jdGlvbiB0aGF0IHdvdWxkIGJlIHVzZWQgZS5nLiBzaGEyNTZcbiAqIEBwYXJhbSBrZXkgLSBtZXNzYWdlIGtleVxuICogQHBhcmFtIG1lc3NhZ2UgLSBtZXNzYWdlIGRhdGFcbiAqL1xuZXhwb3J0IGNvbnN0IGhtYWMgPSAoaGFzaDogQ0hhc2gsIGtleTogSW5wdXQsIG1lc3NhZ2U6IElucHV0KTogVWludDhBcnJheSA9PlxuICBuZXcgSE1BQzxhbnk+KGhhc2gsIGtleSkudXBkYXRlKG1lc3NhZ2UpLmRpZ2VzdCgpO1xuaG1hYy5jcmVhdGUgPSAoaGFzaDogQ0hhc2gsIGtleTogSW5wdXQpID0+IG5ldyBITUFDPGFueT4oaGFzaCwga2V5KTtcbiIsICIvKiEgbm9ibGUtY3VydmVzIC0gTUlUIExpY2Vuc2UgKGMpIDIwMjIgUGF1bCBNaWxsZXIgKHBhdWxtaWxsci5jb20pICovXG5pbXBvcnQgeyBobWFjIH0gZnJvbSAnQG5vYmxlL2hhc2hlcy9obWFjJztcbmltcG9ydCB7IGNvbmNhdEJ5dGVzLCByYW5kb21CeXRlcyB9IGZyb20gJ0Bub2JsZS9oYXNoZXMvdXRpbHMnO1xuaW1wb3J0IHsgd2VpZXJzdHJhc3MsIEN1cnZlVHlwZSB9IGZyb20gJy4vYWJzdHJhY3Qvd2VpZXJzdHJhc3MuanMnO1xuaW1wb3J0IHsgQ0hhc2ggfSBmcm9tICcuL2Fic3RyYWN0L3V0aWxzLmpzJztcblxuLy8gY29ubmVjdHMgbm9ibGUtY3VydmVzIHRvIG5vYmxlLWhhc2hlc1xuZXhwb3J0IGZ1bmN0aW9uIGdldEhhc2goaGFzaDogQ0hhc2gpIHtcbiAgcmV0dXJuIHtcbiAgICBoYXNoLFxuICAgIGhtYWM6IChrZXk6IFVpbnQ4QXJyYXksIC4uLm1zZ3M6IFVpbnQ4QXJyYXlbXSkgPT4gaG1hYyhoYXNoLCBrZXksIGNvbmNhdEJ5dGVzKC4uLm1zZ3MpKSxcbiAgICByYW5kb21CeXRlcyxcbiAgfTtcbn1cbi8vIFNhbWUgQVBJIGFzIEBub2JsZS9oYXNoZXMsIHdpdGggYWJpbGl0eSB0byBjcmVhdGUgY3VydmUgd2l0aCBjdXN0b20gaGFzaFxudHlwZSBDdXJ2ZURlZiA9IFJlYWRvbmx5PE9taXQ8Q3VydmVUeXBlLCAnaGFzaCcgfCAnaG1hYycgfCAncmFuZG9tQnl0ZXMnPj47XG5leHBvcnQgZnVuY3Rpb24gY3JlYXRlQ3VydmUoY3VydmVEZWY6IEN1cnZlRGVmLCBkZWZIYXNoOiBDSGFzaCkge1xuICBjb25zdCBjcmVhdGUgPSAoaGFzaDogQ0hhc2gpID0+IHdlaWVyc3RyYXNzKHsgLi4uY3VydmVEZWYsIC4uLmdldEhhc2goaGFzaCkgfSk7XG4gIHJldHVybiBPYmplY3QuZnJlZXplKHsgLi4uY3JlYXRlKGRlZkhhc2gpLCBjcmVhdGUgfSk7XG59XG4iLCAiLyohIG5vYmxlLWN1cnZlcyAtIE1JVCBMaWNlbnNlIChjKSAyMDIyIFBhdWwgTWlsbGVyIChwYXVsbWlsbHIuY29tKSAqL1xuaW1wb3J0IHsgc2hhMjU2IH0gZnJvbSAnQG5vYmxlL2hhc2hlcy9zaGEyNTYnO1xuaW1wb3J0IHsgcmFuZG9tQnl0ZXMgfSBmcm9tICdAbm9ibGUvaGFzaGVzL3V0aWxzJztcbmltcG9ydCB7IEZpZWxkLCBtb2QsIHBvdzIgfSBmcm9tICcuL2Fic3RyYWN0L21vZHVsYXIuanMnO1xuaW1wb3J0IHsgUHJvalBvaW50VHlwZSBhcyBQb2ludFR5cGUsIG1hcFRvQ3VydmVTaW1wbGVTV1UgfSBmcm9tICcuL2Fic3RyYWN0L3dlaWVyc3RyYXNzLmpzJztcbmltcG9ydCB0eXBlIHsgSGV4LCBQcml2S2V5IH0gZnJvbSAnLi9hYnN0cmFjdC91dGlscy5qcyc7XG5pbXBvcnQgeyBieXRlc1RvTnVtYmVyQkUsIGNvbmNhdEJ5dGVzLCBlbnN1cmVCeXRlcywgbnVtYmVyVG9CeXRlc0JFIH0gZnJvbSAnLi9hYnN0cmFjdC91dGlscy5qcyc7XG5pbXBvcnQgeyBjcmVhdGVIYXNoZXIsIGlzb2dlbnlNYXAgfSBmcm9tICcuL2Fic3RyYWN0L2hhc2gtdG8tY3VydmUuanMnO1xuaW1wb3J0IHsgY3JlYXRlQ3VydmUgfSBmcm9tICcuL19zaG9ydHdfdXRpbHMuanMnO1xuXG5jb25zdCBzZWNwMjU2azFQID0gQmlnSW50KCcweGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZlZmZmZmZjMmYnKTtcbmNvbnN0IHNlY3AyNTZrMU4gPSBCaWdJbnQoJzB4ZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmViYWFlZGNlNmFmNDhhMDNiYmZkMjVlOGNkMDM2NDE0MScpO1xuY29uc3QgXzFuID0gQmlnSW50KDEpO1xuY29uc3QgXzJuID0gQmlnSW50KDIpO1xuY29uc3QgZGl2TmVhcmVzdCA9IChhOiBiaWdpbnQsIGI6IGJpZ2ludCkgPT4gKGEgKyBiIC8gXzJuKSAvIGI7XG5cbi8qKlxuICogXHUyMjFBbiA9IG5eKChwKzEpLzQpIGZvciBmaWVsZHMgcCA9IDMgbW9kIDQuIFdlIHVud3JhcCB0aGUgbG9vcCBhbmQgbXVsdGlwbHkgYml0LWJ5LWJpdC5cbiAqIChQKzFuLzRuKS50b1N0cmluZygyKSB3b3VsZCBwcm9kdWNlIGJpdHMgWzIyM3ggMSwgMCwgMjJ4IDEsIDR4IDAsIDExLCAwMF1cbiAqL1xuZnVuY3Rpb24gc3FydE1vZCh5OiBiaWdpbnQpOiBiaWdpbnQge1xuICBjb25zdCBQID0gc2VjcDI1NmsxUDtcbiAgLy8gcHJldHRpZXItaWdub3JlXG4gIGNvbnN0IF8zbiA9IEJpZ0ludCgzKSwgXzZuID0gQmlnSW50KDYpLCBfMTFuID0gQmlnSW50KDExKSwgXzIybiA9IEJpZ0ludCgyMik7XG4gIC8vIHByZXR0aWVyLWlnbm9yZVxuICBjb25zdCBfMjNuID0gQmlnSW50KDIzKSwgXzQ0biA9IEJpZ0ludCg0NCksIF84OG4gPSBCaWdJbnQoODgpO1xuICBjb25zdCBiMiA9ICh5ICogeSAqIHkpICUgUDsgLy8geF4zLCAxMVxuICBjb25zdCBiMyA9IChiMiAqIGIyICogeSkgJSBQOyAvLyB4XjdcbiAgY29uc3QgYjYgPSAocG93MihiMywgXzNuLCBQKSAqIGIzKSAlIFA7XG4gIGNvbnN0IGI5ID0gKHBvdzIoYjYsIF8zbiwgUCkgKiBiMykgJSBQO1xuICBjb25zdCBiMTEgPSAocG93MihiOSwgXzJuLCBQKSAqIGIyKSAlIFA7XG4gIGNvbnN0IGIyMiA9IChwb3cyKGIxMSwgXzExbiwgUCkgKiBiMTEpICUgUDtcbiAgY29uc3QgYjQ0ID0gKHBvdzIoYjIyLCBfMjJuLCBQKSAqIGIyMikgJSBQO1xuICBjb25zdCBiODggPSAocG93MihiNDQsIF80NG4sIFApICogYjQ0KSAlIFA7XG4gIGNvbnN0IGIxNzYgPSAocG93MihiODgsIF84OG4sIFApICogYjg4KSAlIFA7XG4gIGNvbnN0IGIyMjAgPSAocG93MihiMTc2LCBfNDRuLCBQKSAqIGI0NCkgJSBQO1xuICBjb25zdCBiMjIzID0gKHBvdzIoYjIyMCwgXzNuLCBQKSAqIGIzKSAlIFA7XG4gIGNvbnN0IHQxID0gKHBvdzIoYjIyMywgXzIzbiwgUCkgKiBiMjIpICUgUDtcbiAgY29uc3QgdDIgPSAocG93Mih0MSwgXzZuLCBQKSAqIGIyKSAlIFA7XG4gIGNvbnN0IHJvb3QgPSBwb3cyKHQyLCBfMm4sIFApO1xuICBpZiAoIUZwLmVxbChGcC5zcXIocm9vdCksIHkpKSB0aHJvdyBuZXcgRXJyb3IoJ0Nhbm5vdCBmaW5kIHNxdWFyZSByb290Jyk7XG4gIHJldHVybiByb290O1xufVxuXG5jb25zdCBGcCA9IEZpZWxkKHNlY3AyNTZrMVAsIHVuZGVmaW5lZCwgdW5kZWZpbmVkLCB7IHNxcnQ6IHNxcnRNb2QgfSk7XG5cbmV4cG9ydCBjb25zdCBzZWNwMjU2azEgPSBjcmVhdGVDdXJ2ZShcbiAge1xuICAgIGE6IEJpZ0ludCgwKSwgLy8gZXF1YXRpb24gcGFyYW1zOiBhLCBiXG4gICAgYjogQmlnSW50KDcpLCAvLyBTZWVtIHRvIGJlIHJpZ2lkOiBiaXRjb2ludGFsay5vcmcvaW5kZXgucGhwP3RvcGljPTI4OTc5NS5tc2czMTgzOTc1I21zZzMxODM5NzVcbiAgICBGcCwgLy8gRmllbGQncyBwcmltZTogMm4qKjI1Nm4gLSAybioqMzJuIC0gMm4qKjluIC0gMm4qKjhuIC0gMm4qKjduIC0gMm4qKjZuIC0gMm4qKjRuIC0gMW5cbiAgICBuOiBzZWNwMjU2azFOLCAvLyBDdXJ2ZSBvcmRlciwgdG90YWwgY291bnQgb2YgdmFsaWQgcG9pbnRzIGluIHRoZSBmaWVsZFxuICAgIC8vIEJhc2UgcG9pbnQgKHgsIHkpIGFrYSBnZW5lcmF0b3IgcG9pbnRcbiAgICBHeDogQmlnSW50KCc1NTA2NjI2MzAyMjI3NzM0MzY2OTU3ODcxODg5NTE2ODUzNDMyNjI1MDYwMzQ1Mzc3NzU5NDE3NTUwMDE4NzM2MDM4OTExNjcyOTI0MCcpLFxuICAgIEd5OiBCaWdJbnQoJzMyNjcwNTEwMDIwNzU4ODE2OTc4MDgzMDg1MTMwNTA3MDQzMTg0NDcxMjczMzgwNjU5MjQzMjc1OTM4OTA0MzM1NzU3MzM3NDgyNDI0JyksXG4gICAgaDogQmlnSW50KDEpLCAvLyBDb2ZhY3RvclxuICAgIGxvd1M6IHRydWUsIC8vIEFsbG93IG9ubHkgbG93LVMgc2lnbmF0dXJlcyBieSBkZWZhdWx0IGluIHNpZ24oKSBhbmQgdmVyaWZ5KClcbiAgICAvKipcbiAgICAgKiBzZWNwMjU2azEgYmVsb25ncyB0byBLb2JsaXR6IGN1cnZlczogaXQgaGFzIGVmZmljaWVudGx5IGNvbXB1dGFibGUgZW5kb21vcnBoaXNtLlxuICAgICAqIEVuZG9tb3JwaGlzbSB1c2VzIDJ4IGxlc3MgUkFNLCBzcGVlZHMgdXAgcHJlY29tcHV0YXRpb24gYnkgMnggYW5kIEVDREggLyBrZXkgcmVjb3ZlcnkgYnkgMjAlLlxuICAgICAqIEZvciBwcmVjb21wdXRlZCB3TkFGIGl0IHRyYWRlcyBvZmYgMS8yIGluaXQgdGltZSAmIDEvMyByYW0gZm9yIDIwJSBwZXJmIGhpdC5cbiAgICAgKiBFeHBsYW5hdGlvbjogaHR0cHM6Ly9naXN0LmdpdGh1Yi5jb20vcGF1bG1pbGxyL2ViNjcwODA2NzkzZTg0ZGY2MjhhN2M0MzRhODczMDY2XG4gICAgICovXG4gICAgZW5kbzoge1xuICAgICAgYmV0YTogQmlnSW50KCcweDdhZTk2YTJiNjU3YzA3MTA2ZTY0NDc5ZWFjMzQzNGU5OWNmMDQ5NzUxMmY1ODk5NWMxMzk2YzI4NzE5NTAxZWUnKSxcbiAgICAgIHNwbGl0U2NhbGFyOiAoazogYmlnaW50KSA9PiB7XG4gICAgICAgIGNvbnN0IG4gPSBzZWNwMjU2azFOO1xuICAgICAgICBjb25zdCBhMSA9IEJpZ0ludCgnMHgzMDg2ZDIyMWE3ZDQ2YmNkZTg2YzkwZTQ5Mjg0ZWIxNScpO1xuICAgICAgICBjb25zdCBiMSA9IC1fMW4gKiBCaWdJbnQoJzB4ZTQ0MzdlZDYwMTBlODgyODZmNTQ3ZmE5MGFiZmU0YzMnKTtcbiAgICAgICAgY29uc3QgYTIgPSBCaWdJbnQoJzB4MTE0Y2E1MGY3YThlMmYzZjY1N2MxMTA4ZDlkNDRjZmQ4Jyk7XG4gICAgICAgIGNvbnN0IGIyID0gYTE7XG4gICAgICAgIGNvbnN0IFBPV18yXzEyOCA9IEJpZ0ludCgnMHgxMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAnKTsgLy8gKDJuKioxMjhuKS50b1N0cmluZygxNilcblxuICAgICAgICBjb25zdCBjMSA9IGRpdk5lYXJlc3QoYjIgKiBrLCBuKTtcbiAgICAgICAgY29uc3QgYzIgPSBkaXZOZWFyZXN0KC1iMSAqIGssIG4pO1xuICAgICAgICBsZXQgazEgPSBtb2QoayAtIGMxICogYTEgLSBjMiAqIGEyLCBuKTtcbiAgICAgICAgbGV0IGsyID0gbW9kKC1jMSAqIGIxIC0gYzIgKiBiMiwgbik7XG4gICAgICAgIGNvbnN0IGsxbmVnID0gazEgPiBQT1dfMl8xMjg7XG4gICAgICAgIGNvbnN0IGsybmVnID0gazIgPiBQT1dfMl8xMjg7XG4gICAgICAgIGlmIChrMW5lZykgazEgPSBuIC0gazE7XG4gICAgICAgIGlmIChrMm5lZykgazIgPSBuIC0gazI7XG4gICAgICAgIGlmIChrMSA+IFBPV18yXzEyOCB8fCBrMiA+IFBPV18yXzEyOCkge1xuICAgICAgICAgIHRocm93IG5ldyBFcnJvcignc3BsaXRTY2FsYXI6IEVuZG9tb3JwaGlzbSBmYWlsZWQsIGs9JyArIGspO1xuICAgICAgICB9XG4gICAgICAgIHJldHVybiB7IGsxbmVnLCBrMSwgazJuZWcsIGsyIH07XG4gICAgICB9LFxuICAgIH0sXG4gIH0sXG4gIHNoYTI1NlxuKTtcblxuLy8gU2Nobm9yciBzaWduYXR1cmVzIGFyZSBzdXBlcmlvciB0byBFQ0RTQSBmcm9tIGFib3ZlLiBCZWxvdyBpcyBTY2hub3JyLXNwZWNpZmljIEJJUDAzNDAgY29kZS5cbi8vIGh0dHBzOi8vZ2l0aHViLmNvbS9iaXRjb2luL2JpcHMvYmxvYi9tYXN0ZXIvYmlwLTAzNDAubWVkaWF3aWtpXG5jb25zdCBfMG4gPSBCaWdJbnQoMCk7XG5jb25zdCBmZSA9ICh4OiBiaWdpbnQpID0+IHR5cGVvZiB4ID09PSAnYmlnaW50JyAmJiBfMG4gPCB4ICYmIHggPCBzZWNwMjU2azFQO1xuY29uc3QgZ2UgPSAoeDogYmlnaW50KSA9PiB0eXBlb2YgeCA9PT0gJ2JpZ2ludCcgJiYgXzBuIDwgeCAmJiB4IDwgc2VjcDI1NmsxTjtcbi8qKiBBbiBvYmplY3QgbWFwcGluZyB0YWdzIHRvIHRoZWlyIHRhZ2dlZCBoYXNoIHByZWZpeCBvZiBbU0hBMjU2KHRhZykgfCBTSEEyNTYodGFnKV0gKi9cbmNvbnN0IFRBR0dFRF9IQVNIX1BSRUZJWEVTOiB7IFt0YWc6IHN0cmluZ106IFVpbnQ4QXJyYXkgfSA9IHt9O1xuZnVuY3Rpb24gdGFnZ2VkSGFzaCh0YWc6IHN0cmluZywgLi4ubWVzc2FnZXM6IFVpbnQ4QXJyYXlbXSk6IFVpbnQ4QXJyYXkge1xuICBsZXQgdGFnUCA9IFRBR0dFRF9IQVNIX1BSRUZJWEVTW3RhZ107XG4gIGlmICh0YWdQID09PSB1bmRlZmluZWQpIHtcbiAgICBjb25zdCB0YWdIID0gc2hhMjU2KFVpbnQ4QXJyYXkuZnJvbSh0YWcsIChjKSA9PiBjLmNoYXJDb2RlQXQoMCkpKTtcbiAgICB0YWdQID0gY29uY2F0Qnl0ZXModGFnSCwgdGFnSCk7XG4gICAgVEFHR0VEX0hBU0hfUFJFRklYRVNbdGFnXSA9IHRhZ1A7XG4gIH1cbiAgcmV0dXJuIHNoYTI1Nihjb25jYXRCeXRlcyh0YWdQLCAuLi5tZXNzYWdlcykpO1xufVxuXG4vLyBFQ0RTQSBjb21wYWN0IHBvaW50cyBhcmUgMzMtYnl0ZS4gU2Nobm9yciBpcyAzMjogd2Ugc3RyaXAgZmlyc3QgYnl0ZSAweDAyIG9yIDB4MDNcbmNvbnN0IHBvaW50VG9CeXRlcyA9IChwb2ludDogUG9pbnRUeXBlPGJpZ2ludD4pID0+IHBvaW50LnRvUmF3Qnl0ZXModHJ1ZSkuc2xpY2UoMSk7XG5jb25zdCBudW1UbzMyYiA9IChuOiBiaWdpbnQpID0+IG51bWJlclRvQnl0ZXNCRShuLCAzMik7XG5jb25zdCBtb2RQID0gKHg6IGJpZ2ludCkgPT4gbW9kKHgsIHNlY3AyNTZrMVApO1xuY29uc3QgbW9kTiA9ICh4OiBiaWdpbnQpID0+IG1vZCh4LCBzZWNwMjU2azFOKTtcbmNvbnN0IFBvaW50ID0gc2VjcDI1NmsxLlByb2plY3RpdmVQb2ludDtcbmNvbnN0IEdtdWxBZGQgPSAoUTogUG9pbnRUeXBlPGJpZ2ludD4sIGE6IGJpZ2ludCwgYjogYmlnaW50KSA9PlxuICBQb2ludC5CQVNFLm11bHRpcGx5QW5kQWRkVW5zYWZlKFEsIGEsIGIpO1xuXG4vLyBDYWxjdWxhdGUgcG9pbnQsIHNjYWxhciBhbmQgYnl0ZXNcbmZ1bmN0aW9uIHNjaG5vcnJHZXRFeHRQdWJLZXkocHJpdjogUHJpdktleSkge1xuICBsZXQgZF8gPSBzZWNwMjU2azEudXRpbHMubm9ybVByaXZhdGVLZXlUb1NjYWxhcihwcml2KTsgLy8gc2FtZSBtZXRob2QgZXhlY3V0ZWQgaW4gZnJvbVByaXZhdGVLZXlcbiAgbGV0IHAgPSBQb2ludC5mcm9tUHJpdmF0ZUtleShkXyk7IC8vIFAgPSBkJ1x1MjJDNUc7IDAgPCBkJyA8IG4gY2hlY2sgaXMgZG9uZSBpbnNpZGVcbiAgY29uc3Qgc2NhbGFyID0gcC5oYXNFdmVuWSgpID8gZF8gOiBtb2ROKC1kXyk7XG4gIHJldHVybiB7IHNjYWxhcjogc2NhbGFyLCBieXRlczogcG9pbnRUb0J5dGVzKHApIH07XG59XG4vKipcbiAqIGxpZnRfeCBmcm9tIEJJUDM0MC4gQ29udmVydCAzMi1ieXRlIHggY29vcmRpbmF0ZSB0byBlbGxpcHRpYyBjdXJ2ZSBwb2ludC5cbiAqIEByZXR1cm5zIHZhbGlkIHBvaW50IGNoZWNrZWQgZm9yIGJlaW5nIG9uLWN1cnZlXG4gKi9cbmZ1bmN0aW9uIGxpZnRfeCh4OiBiaWdpbnQpOiBQb2ludFR5cGU8YmlnaW50PiB7XG4gIGlmICghZmUoeCkpIHRocm93IG5ldyBFcnJvcignYmFkIHg6IG5lZWQgMCA8IHggPCBwJyk7IC8vIEZhaWwgaWYgeCBcdTIyNjUgcC5cbiAgY29uc3QgeHggPSBtb2RQKHggKiB4KTtcbiAgY29uc3QgYyA9IG1vZFAoeHggKiB4ICsgQmlnSW50KDcpKTsgLy8gTGV0IGMgPSB4XHUwMEIzICsgNyBtb2QgcC5cbiAgbGV0IHkgPSBzcXJ0TW9kKGMpOyAvLyBMZXQgeSA9IGNeKHArMSkvNCBtb2QgcC5cbiAgaWYgKHkgJSBfMm4gIT09IF8wbikgeSA9IG1vZFAoLXkpOyAvLyBSZXR1cm4gdGhlIHVuaXF1ZSBwb2ludCBQIHN1Y2ggdGhhdCB4KFApID0geCBhbmRcbiAgY29uc3QgcCA9IG5ldyBQb2ludCh4LCB5LCBfMW4pOyAvLyB5KFApID0geSBpZiB5IG1vZCAyID0gMCBvciB5KFApID0gcC15IG90aGVyd2lzZS5cbiAgcC5hc3NlcnRWYWxpZGl0eSgpO1xuICByZXR1cm4gcDtcbn1cbi8qKlxuICogQ3JlYXRlIHRhZ2dlZCBoYXNoLCBjb252ZXJ0IGl0IHRvIGJpZ2ludCwgcmVkdWNlIG1vZHVsby1uLlxuICovXG5mdW5jdGlvbiBjaGFsbGVuZ2UoLi4uYXJnczogVWludDhBcnJheVtdKTogYmlnaW50IHtcbiAgcmV0dXJuIG1vZE4oYnl0ZXNUb051bWJlckJFKHRhZ2dlZEhhc2goJ0JJUDAzNDAvY2hhbGxlbmdlJywgLi4uYXJncykpKTtcbn1cblxuLyoqXG4gKiBTY2hub3JyIHB1YmxpYyBrZXkgaXMganVzdCBgeGAgY29vcmRpbmF0ZSBvZiBQb2ludCBhcyBwZXIgQklQMzQwLlxuICovXG5mdW5jdGlvbiBzY2hub3JyR2V0UHVibGljS2V5KHByaXZhdGVLZXk6IEhleCk6IFVpbnQ4QXJyYXkge1xuICByZXR1cm4gc2Nobm9yckdldEV4dFB1YktleShwcml2YXRlS2V5KS5ieXRlczsgLy8gZCc9aW50KHNrKS4gRmFpbCBpZiBkJz0wIG9yIGQnXHUyMjY1bi4gUmV0IGJ5dGVzKGQnXHUyMkM1Rylcbn1cblxuLyoqXG4gKiBDcmVhdGVzIFNjaG5vcnIgc2lnbmF0dXJlIGFzIHBlciBCSVAzNDAuIFZlcmlmaWVzIGl0c2VsZiBiZWZvcmUgcmV0dXJuaW5nIGFueXRoaW5nLlxuICogYXV4UmFuZCBpcyBvcHRpb25hbCBhbmQgaXMgbm90IHRoZSBzb2xlIHNvdXJjZSBvZiBrIGdlbmVyYXRpb246IGJhZCBDU1BSTkcgd29uJ3QgYmUgZGFuZ2Vyb3VzLlxuICovXG5mdW5jdGlvbiBzY2hub3JyU2lnbihcbiAgbWVzc2FnZTogSGV4LFxuICBwcml2YXRlS2V5OiBQcml2S2V5LFxuICBhdXhSYW5kOiBIZXggPSByYW5kb21CeXRlcygzMilcbik6IFVpbnQ4QXJyYXkge1xuICBjb25zdCBtID0gZW5zdXJlQnl0ZXMoJ21lc3NhZ2UnLCBtZXNzYWdlKTtcbiAgY29uc3QgeyBieXRlczogcHgsIHNjYWxhcjogZCB9ID0gc2Nobm9yckdldEV4dFB1YktleShwcml2YXRlS2V5KTsgLy8gY2hlY2tzIGZvciBpc1dpdGhpbkN1cnZlT3JkZXJcbiAgY29uc3QgYSA9IGVuc3VyZUJ5dGVzKCdhdXhSYW5kJywgYXV4UmFuZCwgMzIpOyAvLyBBdXhpbGlhcnkgcmFuZG9tIGRhdGEgYTogYSAzMi1ieXRlIGFycmF5XG4gIGNvbnN0IHQgPSBudW1UbzMyYihkIF4gYnl0ZXNUb051bWJlckJFKHRhZ2dlZEhhc2goJ0JJUDAzNDAvYXV4JywgYSkpKTsgLy8gTGV0IHQgYmUgdGhlIGJ5dGUtd2lzZSB4b3Igb2YgYnl0ZXMoZCkgYW5kIGhhc2gvYXV4KGEpXG4gIGNvbnN0IHJhbmQgPSB0YWdnZWRIYXNoKCdCSVAwMzQwL25vbmNlJywgdCwgcHgsIG0pOyAvLyBMZXQgcmFuZCA9IGhhc2gvbm9uY2UodCB8fCBieXRlcyhQKSB8fCBtKVxuICBjb25zdCBrXyA9IG1vZE4oYnl0ZXNUb051bWJlckJFKHJhbmQpKTsgLy8gTGV0IGsnID0gaW50KHJhbmQpIG1vZCBuXG4gIGlmIChrXyA9PT0gXzBuKSB0aHJvdyBuZXcgRXJyb3IoJ3NpZ24gZmFpbGVkOiBrIGlzIHplcm8nKTsgLy8gRmFpbCBpZiBrJyA9IDAuXG4gIGNvbnN0IHsgYnl0ZXM6IHJ4LCBzY2FsYXI6IGsgfSA9IHNjaG5vcnJHZXRFeHRQdWJLZXkoa18pOyAvLyBMZXQgUiA9IGsnXHUyMkM1Ry5cbiAgY29uc3QgZSA9IGNoYWxsZW5nZShyeCwgcHgsIG0pOyAvLyBMZXQgZSA9IGludChoYXNoL2NoYWxsZW5nZShieXRlcyhSKSB8fCBieXRlcyhQKSB8fCBtKSkgbW9kIG4uXG4gIGNvbnN0IHNpZyA9IG5ldyBVaW50OEFycmF5KDY0KTsgLy8gTGV0IHNpZyA9IGJ5dGVzKFIpIHx8IGJ5dGVzKChrICsgZWQpIG1vZCBuKS5cbiAgc2lnLnNldChyeCwgMCk7XG4gIHNpZy5zZXQobnVtVG8zMmIobW9kTihrICsgZSAqIGQpKSwgMzIpO1xuICAvLyBJZiBWZXJpZnkoYnl0ZXMoUCksIG0sIHNpZykgKHNlZSBiZWxvdykgcmV0dXJucyBmYWlsdXJlLCBhYm9ydFxuICBpZiAoIXNjaG5vcnJWZXJpZnkoc2lnLCBtLCBweCkpIHRocm93IG5ldyBFcnJvcignc2lnbjogSW52YWxpZCBzaWduYXR1cmUgcHJvZHVjZWQnKTtcbiAgcmV0dXJuIHNpZztcbn1cblxuLyoqXG4gKiBWZXJpZmllcyBTY2hub3JyIHNpZ25hdHVyZS5cbiAqIFdpbGwgc3dhbGxvdyBlcnJvcnMgJiByZXR1cm4gZmFsc2UgZXhjZXB0IGZvciBpbml0aWFsIHR5cGUgdmFsaWRhdGlvbiBvZiBhcmd1bWVudHMuXG4gKi9cbmZ1bmN0aW9uIHNjaG5vcnJWZXJpZnkoc2lnbmF0dXJlOiBIZXgsIG1lc3NhZ2U6IEhleCwgcHVibGljS2V5OiBIZXgpOiBib29sZWFuIHtcbiAgY29uc3Qgc2lnID0gZW5zdXJlQnl0ZXMoJ3NpZ25hdHVyZScsIHNpZ25hdHVyZSwgNjQpO1xuICBjb25zdCBtID0gZW5zdXJlQnl0ZXMoJ21lc3NhZ2UnLCBtZXNzYWdlKTtcbiAgY29uc3QgcHViID0gZW5zdXJlQnl0ZXMoJ3B1YmxpY0tleScsIHB1YmxpY0tleSwgMzIpO1xuICB0cnkge1xuICAgIGNvbnN0IFAgPSBsaWZ0X3goYnl0ZXNUb051bWJlckJFKHB1YikpOyAvLyBQID0gbGlmdF94KGludChwaykpOyBmYWlsIGlmIHRoYXQgZmFpbHNcbiAgICBjb25zdCByID0gYnl0ZXNUb051bWJlckJFKHNpZy5zdWJhcnJheSgwLCAzMikpOyAvLyBMZXQgciA9IGludChzaWdbMDozMl0pOyBmYWlsIGlmIHIgXHUyMjY1IHAuXG4gICAgaWYgKCFmZShyKSkgcmV0dXJuIGZhbHNlO1xuICAgIGNvbnN0IHMgPSBieXRlc1RvTnVtYmVyQkUoc2lnLnN1YmFycmF5KDMyLCA2NCkpOyAvLyBMZXQgcyA9IGludChzaWdbMzI6NjRdKTsgZmFpbCBpZiBzIFx1MjI2NSBuLlxuICAgIGlmICghZ2UocykpIHJldHVybiBmYWxzZTtcbiAgICBjb25zdCBlID0gY2hhbGxlbmdlKG51bVRvMzJiKHIpLCBwb2ludFRvQnl0ZXMoUCksIG0pOyAvLyBpbnQoY2hhbGxlbmdlKGJ5dGVzKHIpfHxieXRlcyhQKXx8bSkpJW5cbiAgICBjb25zdCBSID0gR211bEFkZChQLCBzLCBtb2ROKC1lKSk7IC8vIFIgPSBzXHUyMkM1RyAtIGVcdTIyQzVQXG4gICAgaWYgKCFSIHx8ICFSLmhhc0V2ZW5ZKCkgfHwgUi50b0FmZmluZSgpLnggIT09IHIpIHJldHVybiBmYWxzZTsgLy8gLWVQID09IChuLWUpUFxuICAgIHJldHVybiB0cnVlOyAvLyBGYWlsIGlmIGlzX2luZmluaXRlKFIpIC8gbm90IGhhc19ldmVuX3koUikgLyB4KFIpIFx1MjI2MCByLlxuICB9IGNhdGNoIChlcnJvcikge1xuICAgIHJldHVybiBmYWxzZTtcbiAgfVxufVxuXG5leHBvcnQgY29uc3Qgc2Nobm9yciA9IC8qIEBfX1BVUkVfXyAqLyAoKCkgPT4gKHtcbiAgZ2V0UHVibGljS2V5OiBzY2hub3JyR2V0UHVibGljS2V5LFxuICBzaWduOiBzY2hub3JyU2lnbixcbiAgdmVyaWZ5OiBzY2hub3JyVmVyaWZ5LFxuICB1dGlsczoge1xuICAgIHJhbmRvbVByaXZhdGVLZXk6IHNlY3AyNTZrMS51dGlscy5yYW5kb21Qcml2YXRlS2V5LFxuICAgIGxpZnRfeCxcbiAgICBwb2ludFRvQnl0ZXMsXG4gICAgbnVtYmVyVG9CeXRlc0JFLFxuICAgIGJ5dGVzVG9OdW1iZXJCRSxcbiAgICB0YWdnZWRIYXNoLFxuICAgIG1vZCxcbiAgfSxcbn0pKSgpO1xuXG5jb25zdCBpc29NYXAgPSAvKiBAX19QVVJFX18gKi8gKCgpID0+XG4gIGlzb2dlbnlNYXAoXG4gICAgRnAsXG4gICAgW1xuICAgICAgLy8geE51bVxuICAgICAgW1xuICAgICAgICAnMHg4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZTM4ZGFhYWFhOGM3JyxcbiAgICAgICAgJzB4N2QzZDRjODBiYzMyMWQ1YjlmMzE1Y2VhN2ZkNDRjNWQ1OTVkMmZjMGJmNjNiOTJkZmZmMTA0NGYxN2M2NTgxJyxcbiAgICAgICAgJzB4NTM0YzMyOGQyM2YyMzRlNmUyYTQxM2RlY2EyNWNhZWNlNDUwNjE0NDAzN2M0MDMxNGVjYmQwYjUzZDlkZDI2MicsXG4gICAgICAgICcweDhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhlMzhkYWFhYWE4OGMnLFxuICAgICAgXSxcbiAgICAgIC8vIHhEZW5cbiAgICAgIFtcbiAgICAgICAgJzB4ZDM1NzcxMTkzZDk0OTE4YTljYTM0Y2NiYjdiNjQwZGQ4NmNkNDA5NTQyZjg0ODdkOWZlNmI3NDU3ODFlYjQ5YicsXG4gICAgICAgICcweGVkYWRjNmY2NDM4M2RjMWRmN2M0YjJkNTFiNTQyMjU0MDZkMzZiNjQxZjVlNDFiYmM1MmE1NjYxMmE4YzZkMTQnLFxuICAgICAgICAnMHgwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAxJywgLy8gTEFTVCAxXG4gICAgICBdLFxuICAgICAgLy8geU51bVxuICAgICAgW1xuICAgICAgICAnMHg0YmRhMTJmNjg0YmRhMTJmNjg0YmRhMTJmNjg0YmRhMTJmNjg0YmRhMTJmNjg0YmRhMTJmNjg0YjhlMzhlMjNjJyxcbiAgICAgICAgJzB4Yzc1ZTBjMzJkNWNiN2MwZmE5ZDBhNTRiMTJhMGE2ZDU2NDdhYjA0NmQ2ODZkYTZmZGZmYzkwZmMyMDFkNzFhMycsXG4gICAgICAgICcweDI5YTYxOTQ2OTFmOTFhNzM3MTUyMDllZjY1MTJlNTc2NzIyODMwYTIwMWJlMjAxOGE3NjVlODVhOWVjZWU5MzEnLFxuICAgICAgICAnMHgyZjY4NGJkYTEyZjY4NGJkYTEyZjY4NGJkYTEyZjY4NGJkYTEyZjY4NGJkYTEyZjY4NGJkYTEyZjM4ZTM4ZDg0JyxcbiAgICAgIF0sXG4gICAgICAvLyB5RGVuXG4gICAgICBbXG4gICAgICAgICcweGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZlZmZmZmY5M2InLFxuICAgICAgICAnMHg3YTA2NTM0YmI4YmRiNDlmZDVlOWU2NjMyNzIyYzI5ODk0NjdjMWJmYzhlOGQ5NzhkZmI0MjVkMjY4NWMyNTczJyxcbiAgICAgICAgJzB4NjQ4NGFhNzE2NTQ1Y2EyY2YzYTcwYzNmYThmZTMzN2UwYTNkMjExNjJmMGQ2Mjk5YTdiZjgxOTJiZmQyYTc2ZicsXG4gICAgICAgICcweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDEnLCAvLyBMQVNUIDFcbiAgICAgIF0sXG4gICAgXS5tYXAoKGkpID0+IGkubWFwKChqKSA9PiBCaWdJbnQoaikpKSBhcyBbYmlnaW50W10sIGJpZ2ludFtdLCBiaWdpbnRbXSwgYmlnaW50W11dXG4gICkpKCk7XG5jb25zdCBtYXBTV1UgPSAvKiBAX19QVVJFX18gKi8gKCgpID0+XG4gIG1hcFRvQ3VydmVTaW1wbGVTV1UoRnAsIHtcbiAgICBBOiBCaWdJbnQoJzB4M2Y4NzMxYWJkZDY2MWFkY2EwOGE1NTU4ZjBmNWQyNzJlOTUzZDM2M2NiNmYwZTVkNDA1NDQ3YzAxYTQ0NDUzMycpLFxuICAgIEI6IEJpZ0ludCgnMTc3MScpLFxuICAgIFo6IEZwLmNyZWF0ZShCaWdJbnQoJy0xMScpKSxcbiAgfSkpKCk7XG5jb25zdCBodGYgPSAvKiBAX19QVVJFX18gKi8gKCgpID0+XG4gIGNyZWF0ZUhhc2hlcihcbiAgICBzZWNwMjU2azEuUHJvamVjdGl2ZVBvaW50LFxuICAgIChzY2FsYXJzOiBiaWdpbnRbXSkgPT4ge1xuICAgICAgY29uc3QgeyB4LCB5IH0gPSBtYXBTV1UoRnAuY3JlYXRlKHNjYWxhcnNbMF0pKTtcbiAgICAgIHJldHVybiBpc29NYXAoeCwgeSk7XG4gICAgfSxcbiAgICB7XG4gICAgICBEU1Q6ICdzZWNwMjU2azFfWE1EOlNIQS0yNTZfU1NXVV9ST18nLFxuICAgICAgZW5jb2RlRFNUOiAnc2VjcDI1NmsxX1hNRDpTSEEtMjU2X1NTV1VfTlVfJyxcbiAgICAgIHA6IEZwLk9SREVSLFxuICAgICAgbTogMSxcbiAgICAgIGs6IDEyOCxcbiAgICAgIGV4cGFuZDogJ3htZCcsXG4gICAgICBoYXNoOiBzaGEyNTYsXG4gICAgfVxuICApKSgpO1xuZXhwb3J0IGNvbnN0IGhhc2hUb0N1cnZlID0gLyogQF9fUFVSRV9fICovICgoKSA9PiBodGYuaGFzaFRvQ3VydmUpKCk7XG5leHBvcnQgY29uc3QgZW5jb2RlVG9DdXJ2ZSA9IC8qIEBfX1BVUkVfXyAqLyAoKCkgPT4gaHRmLmVuY29kZVRvQ3VydmUpKCk7XG4iLCAiLy8gV2UgdXNlIFdlYkNyeXB0byBha2EgZ2xvYmFsVGhpcy5jcnlwdG8sIHdoaWNoIGV4aXN0cyBpbiBicm93c2VycyBhbmQgbm9kZS5qcyAxNisuXG4vLyBTZWUgdXRpbHMudHMgZm9yIGRldGFpbHMuXG5kZWNsYXJlIGNvbnN0IGdsb2JhbFRoaXM6IFJlY29yZDxzdHJpbmcsIGFueT4gfCB1bmRlZmluZWQ7XG5leHBvcnQgY29uc3QgY3J5cHRvID1cbiAgdHlwZW9mIGdsb2JhbFRoaXMgPT09ICdvYmplY3QnICYmICdjcnlwdG8nIGluIGdsb2JhbFRoaXMgPyBnbG9iYWxUaGlzLmNyeXB0byA6IHVuZGVmaW5lZDtcbiIsICIvKiEgbm9ibGUtaGFzaGVzIC0gTUlUIExpY2Vuc2UgKGMpIDIwMjIgUGF1bCBNaWxsZXIgKHBhdWxtaWxsci5jb20pICovXG5cbi8vIFdlIHVzZSBXZWJDcnlwdG8gYWthIGdsb2JhbFRoaXMuY3J5cHRvLCB3aGljaCBleGlzdHMgaW4gYnJvd3NlcnMgYW5kIG5vZGUuanMgMTYrLlxuLy8gbm9kZS5qcyB2ZXJzaW9ucyBlYXJsaWVyIHRoYW4gdjE5IGRvbid0IGRlY2xhcmUgaXQgaW4gZ2xvYmFsIHNjb3BlLlxuLy8gRm9yIG5vZGUuanMsIHBhY2thZ2UuanNvbiNleHBvcnRzIGZpZWxkIG1hcHBpbmcgcmV3cml0ZXMgaW1wb3J0XG4vLyBmcm9tIGBjcnlwdG9gIHRvIGBjcnlwdG9Ob2RlYCwgd2hpY2ggaW1wb3J0cyBuYXRpdmUgbW9kdWxlLlxuLy8gTWFrZXMgdGhlIHV0aWxzIHVuLWltcG9ydGFibGUgaW4gYnJvd3NlcnMgd2l0aG91dCBhIGJ1bmRsZXIuXG4vLyBPbmNlIG5vZGUuanMgMTggaXMgZGVwcmVjYXRlZCwgd2UgY2FuIGp1c3QgZHJvcCB0aGUgaW1wb3J0LlxuaW1wb3J0IHsgY3J5cHRvIH0gZnJvbSAnQG5vYmxlL2hhc2hlcy9jcnlwdG8nO1xuXG4vLyBwcmV0dGllci1pZ25vcmVcbmV4cG9ydCB0eXBlIFR5cGVkQXJyYXkgPSBJbnQ4QXJyYXkgfCBVaW50OENsYW1wZWRBcnJheSB8IFVpbnQ4QXJyYXkgfFxuICBVaW50MTZBcnJheSB8IEludDE2QXJyYXkgfCBVaW50MzJBcnJheSB8IEludDMyQXJyYXk7XG5cbmNvbnN0IHU4YSA9IChhOiBhbnkpOiBhIGlzIFVpbnQ4QXJyYXkgPT4gYSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXk7XG4vLyBDYXN0IGFycmF5IHRvIGRpZmZlcmVudCB0eXBlXG5leHBvcnQgY29uc3QgdTggPSAoYXJyOiBUeXBlZEFycmF5KSA9PiBuZXcgVWludDhBcnJheShhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgYXJyLmJ5dGVMZW5ndGgpO1xuZXhwb3J0IGNvbnN0IHUzMiA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBVaW50MzJBcnJheShhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgTWF0aC5mbG9vcihhcnIuYnl0ZUxlbmd0aCAvIDQpKTtcblxuLy8gQ2FzdCBhcnJheSB0byB2aWV3XG5leHBvcnQgY29uc3QgY3JlYXRlVmlldyA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBEYXRhVmlldyhhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgYXJyLmJ5dGVMZW5ndGgpO1xuXG4vLyBUaGUgcm90YXRlIHJpZ2h0IChjaXJjdWxhciByaWdodCBzaGlmdCkgb3BlcmF0aW9uIGZvciB1aW50MzJcbmV4cG9ydCBjb25zdCByb3RyID0gKHdvcmQ6IG51bWJlciwgc2hpZnQ6IG51bWJlcikgPT4gKHdvcmQgPDwgKDMyIC0gc2hpZnQpKSB8ICh3b3JkID4+PiBzaGlmdCk7XG5cbi8vIGJpZy1lbmRpYW4gaGFyZHdhcmUgaXMgcmFyZS4gSnVzdCBpbiBjYXNlIHNvbWVvbmUgc3RpbGwgZGVjaWRlcyB0byBydW4gaGFzaGVzOlxuLy8gZWFybHktdGhyb3cgYW4gZXJyb3IgYmVjYXVzZSB3ZSBkb24ndCBzdXBwb3J0IEJFIHlldC5cbmV4cG9ydCBjb25zdCBpc0xFID0gbmV3IFVpbnQ4QXJyYXkobmV3IFVpbnQzMkFycmF5KFsweDExMjIzMzQ0XSkuYnVmZmVyKVswXSA9PT0gMHg0NDtcbmlmICghaXNMRSkgdGhyb3cgbmV3IEVycm9yKCdOb24gbGl0dGxlLWVuZGlhbiBoYXJkd2FyZSBpcyBub3Qgc3VwcG9ydGVkJyk7XG5cbmNvbnN0IGhleGVzID0gQXJyYXkuZnJvbSh7IGxlbmd0aDogMjU2IH0sICh2LCBpKSA9PiBpLnRvU3RyaW5nKDE2KS5wYWRTdGFydCgyLCAnMCcpKTtcbi8qKlxuICogQGV4YW1wbGUgYnl0ZXNUb0hleChVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKSkgLy8gJ2NhZmUwMTIzJ1xuICovXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb0hleChieXRlczogVWludDhBcnJheSk6IHN0cmluZyB7XG4gIGlmICghdThhKGJ5dGVzKSkgdGhyb3cgbmV3IEVycm9yKCdVaW50OEFycmF5IGV4cGVjdGVkJyk7XG4gIC8vIHByZS1jYWNoaW5nIGltcHJvdmVzIHRoZSBzcGVlZCA2eFxuICBsZXQgaGV4ID0gJyc7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgYnl0ZXMubGVuZ3RoOyBpKyspIHtcbiAgICBoZXggKz0gaGV4ZXNbYnl0ZXNbaV1dO1xuICB9XG4gIHJldHVybiBoZXg7XG59XG5cbi8qKlxuICogQGV4YW1wbGUgaGV4VG9CeXRlcygnY2FmZTAxMjMnKSAvLyBVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKVxuICovXG5leHBvcnQgZnVuY3Rpb24gaGV4VG9CeXRlcyhoZXg6IHN0cmluZyk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIGhleCAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcignaGV4IHN0cmluZyBleHBlY3RlZCwgZ290ICcgKyB0eXBlb2YgaGV4KTtcbiAgY29uc3QgbGVuID0gaGV4Lmxlbmd0aDtcbiAgaWYgKGxlbiAlIDIpIHRocm93IG5ldyBFcnJvcigncGFkZGVkIGhleCBzdHJpbmcgZXhwZWN0ZWQsIGdvdCB1bnBhZGRlZCBoZXggb2YgbGVuZ3RoICcgKyBsZW4pO1xuICBjb25zdCBhcnJheSA9IG5ldyBVaW50OEFycmF5KGxlbiAvIDIpO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGFycmF5Lmxlbmd0aDsgaSsrKSB7XG4gICAgY29uc3QgaiA9IGkgKiAyO1xuICAgIGNvbnN0IGhleEJ5dGUgPSBoZXguc2xpY2UoaiwgaiArIDIpO1xuICAgIGNvbnN0IGJ5dGUgPSBOdW1iZXIucGFyc2VJbnQoaGV4Qnl0ZSwgMTYpO1xuICAgIGlmIChOdW1iZXIuaXNOYU4oYnl0ZSkgfHwgYnl0ZSA8IDApIHRocm93IG5ldyBFcnJvcignSW52YWxpZCBieXRlIHNlcXVlbmNlJyk7XG4gICAgYXJyYXlbaV0gPSBieXRlO1xuICB9XG4gIHJldHVybiBhcnJheTtcbn1cblxuLy8gVGhlcmUgaXMgbm8gc2V0SW1tZWRpYXRlIGluIGJyb3dzZXIgYW5kIHNldFRpbWVvdXQgaXMgc2xvdy5cbi8vIGNhbGwgb2YgYXN5bmMgZm4gd2lsbCByZXR1cm4gUHJvbWlzZSwgd2hpY2ggd2lsbCBiZSBmdWxsZmlsZWQgb25seSBvblxuLy8gbmV4dCBzY2hlZHVsZXIgcXVldWUgcHJvY2Vzc2luZyBzdGVwIGFuZCB0aGlzIGlzIGV4YWN0bHkgd2hhdCB3ZSBuZWVkLlxuZXhwb3J0IGNvbnN0IG5leHRUaWNrID0gYXN5bmMgKCkgPT4ge307XG5cbi8vIFJldHVybnMgY29udHJvbCB0byB0aHJlYWQgZWFjaCAndGljaycgbXMgdG8gYXZvaWQgYmxvY2tpbmdcbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBhc3luY0xvb3AoaXRlcnM6IG51bWJlciwgdGljazogbnVtYmVyLCBjYjogKGk6IG51bWJlcikgPT4gdm9pZCkge1xuICBsZXQgdHMgPSBEYXRlLm5vdygpO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGl0ZXJzOyBpKyspIHtcbiAgICBjYihpKTtcbiAgICAvLyBEYXRlLm5vdygpIGlzIG5vdCBtb25vdG9uaWMsIHNvIGluIGNhc2UgaWYgY2xvY2sgZ29lcyBiYWNrd2FyZHMgd2UgcmV0dXJuIHJldHVybiBjb250cm9sIHRvb1xuICAgIGNvbnN0IGRpZmYgPSBEYXRlLm5vdygpIC0gdHM7XG4gICAgaWYgKGRpZmYgPj0gMCAmJiBkaWZmIDwgdGljaykgY29udGludWU7XG4gICAgYXdhaXQgbmV4dFRpY2soKTtcbiAgICB0cyArPSBkaWZmO1xuICB9XG59XG5cbi8vIEdsb2JhbCBzeW1ib2xzIGluIGJvdGggYnJvd3NlcnMgYW5kIE5vZGUuanMgc2luY2UgdjExXG4vLyBTZWUgaHR0cHM6Ly9naXRodWIuY29tL21pY3Jvc29mdC9UeXBlU2NyaXB0L2lzc3Vlcy8zMTUzNVxuZGVjbGFyZSBjb25zdCBUZXh0RW5jb2RlcjogYW55O1xuXG4vKipcbiAqIEBleGFtcGxlIHV0ZjhUb0J5dGVzKCdhYmMnKSAvLyBuZXcgVWludDhBcnJheShbOTcsIDk4LCA5OV0pXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiB1dGY4VG9CeXRlcyhzdHI6IHN0cmluZyk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIHN0ciAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcihgdXRmOFRvQnl0ZXMgZXhwZWN0ZWQgc3RyaW5nLCBnb3QgJHt0eXBlb2Ygc3RyfWApO1xuICByZXR1cm4gbmV3IFVpbnQ4QXJyYXkobmV3IFRleHRFbmNvZGVyKCkuZW5jb2RlKHN0cikpOyAvLyBodHRwczovL2J1Z3ppbC5sYS8xNjgxODA5XG59XG5cbmV4cG9ydCB0eXBlIElucHV0ID0gVWludDhBcnJheSB8IHN0cmluZztcbi8qKlxuICogTm9ybWFsaXplcyAobm9uLWhleCkgc3RyaW5nIG9yIFVpbnQ4QXJyYXkgdG8gVWludDhBcnJheS5cbiAqIFdhcm5pbmc6IHdoZW4gVWludDhBcnJheSBpcyBwYXNzZWQsIGl0IHdvdWxkIE5PVCBnZXQgY29waWVkLlxuICogS2VlcCBpbiBtaW5kIGZvciBmdXR1cmUgbXV0YWJsZSBvcGVyYXRpb25zLlxuICovXG5leHBvcnQgZnVuY3Rpb24gdG9CeXRlcyhkYXRhOiBJbnB1dCk6IFVpbnQ4QXJyYXkge1xuICBpZiAodHlwZW9mIGRhdGEgPT09ICdzdHJpbmcnKSBkYXRhID0gdXRmOFRvQnl0ZXMoZGF0YSk7XG4gIGlmICghdThhKGRhdGEpKSB0aHJvdyBuZXcgRXJyb3IoYGV4cGVjdGVkIFVpbnQ4QXJyYXksIGdvdCAke3R5cGVvZiBkYXRhfWApO1xuICByZXR1cm4gZGF0YTtcbn1cblxuLyoqXG4gKiBDb3BpZXMgc2V2ZXJhbCBVaW50OEFycmF5cyBpbnRvIG9uZS5cbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIGNvbmNhdEJ5dGVzKC4uLmFycmF5czogVWludDhBcnJheVtdKTogVWludDhBcnJheSB7XG4gIGNvbnN0IHIgPSBuZXcgVWludDhBcnJheShhcnJheXMucmVkdWNlKChzdW0sIGEpID0+IHN1bSArIGEubGVuZ3RoLCAwKSk7XG4gIGxldCBwYWQgPSAwOyAvLyB3YWxrIHRocm91Z2ggZWFjaCBpdGVtLCBlbnN1cmUgdGhleSBoYXZlIHByb3BlciB0eXBlXG4gIGFycmF5cy5mb3JFYWNoKChhKSA9PiB7XG4gICAgaWYgKCF1OGEoYSkpIHRocm93IG5ldyBFcnJvcignVWludDhBcnJheSBleHBlY3RlZCcpO1xuICAgIHIuc2V0KGEsIHBhZCk7XG4gICAgcGFkICs9IGEubGVuZ3RoO1xuICB9KTtcbiAgcmV0dXJuIHI7XG59XG5cbi8vIEZvciBydW50aW1lIGNoZWNrIGlmIGNsYXNzIGltcGxlbWVudHMgaW50ZXJmYWNlXG5leHBvcnQgYWJzdHJhY3QgY2xhc3MgSGFzaDxUIGV4dGVuZHMgSGFzaDxUPj4ge1xuICBhYnN0cmFjdCBibG9ja0xlbjogbnVtYmVyOyAvLyBCeXRlcyBwZXIgYmxvY2tcbiAgYWJzdHJhY3Qgb3V0cHV0TGVuOiBudW1iZXI7IC8vIEJ5dGVzIGluIG91dHB1dFxuICBhYnN0cmFjdCB1cGRhdGUoYnVmOiBJbnB1dCk6IHRoaXM7XG4gIC8vIFdyaXRlcyBkaWdlc3QgaW50byBidWZcbiAgYWJzdHJhY3QgZGlnZXN0SW50byhidWY6IFVpbnQ4QXJyYXkpOiB2b2lkO1xuICBhYnN0cmFjdCBkaWdlc3QoKTogVWludDhBcnJheTtcbiAgLyoqXG4gICAqIFJlc2V0cyBpbnRlcm5hbCBzdGF0ZS4gTWFrZXMgSGFzaCBpbnN0YW5jZSB1bnVzYWJsZS5cbiAgICogUmVzZXQgaXMgaW1wb3NzaWJsZSBmb3Iga2V5ZWQgaGFzaGVzIGlmIGtleSBpcyBjb25zdW1lZCBpbnRvIHN0YXRlLiBJZiBkaWdlc3QgaXMgbm90IGNvbnN1bWVkXG4gICAqIGJ5IHVzZXIsIHRoZXkgd2lsbCBuZWVkIHRvIG1hbnVhbGx5IGNhbGwgYGRlc3Ryb3koKWAgd2hlbiB6ZXJvaW5nIGlzIG5lY2Vzc2FyeS5cbiAgICovXG4gIGFic3RyYWN0IGRlc3Ryb3koKTogdm9pZDtcbiAgLyoqXG4gICAqIENsb25lcyBoYXNoIGluc3RhbmNlLiBVbnNhZmU6IGRvZXNuJ3QgY2hlY2sgd2hldGhlciBgdG9gIGlzIHZhbGlkLiBDYW4gYmUgdXNlZCBhcyBgY2xvbmUoKWBcbiAgICogd2hlbiBubyBvcHRpb25zIGFyZSBwYXNzZWQuXG4gICAqIFJlYXNvbnMgdG8gdXNlIGBfY2xvbmVJbnRvYCBpbnN0ZWFkIG9mIGNsb25lOiAxKSBwZXJmb3JtYW5jZSAyKSByZXVzZSBpbnN0YW5jZSA9PiBhbGwgaW50ZXJuYWxcbiAgICogYnVmZmVycyBhcmUgb3ZlcndyaXR0ZW4gPT4gY2F1c2VzIGJ1ZmZlciBvdmVyd3JpdGUgd2hpY2ggaXMgdXNlZCBmb3IgZGlnZXN0IGluIHNvbWUgY2FzZXMuXG4gICAqIFRoZXJlIGFyZSBubyBndWFyYW50ZWVzIGZvciBjbGVhbi11cCBiZWNhdXNlIGl0J3MgaW1wb3NzaWJsZSBpbiBKUy5cbiAgICovXG4gIGFic3RyYWN0IF9jbG9uZUludG8odG8/OiBUKTogVDtcbiAgLy8gU2FmZSB2ZXJzaW9uIHRoYXQgY2xvbmVzIGludGVybmFsIHN0YXRlXG4gIGNsb25lKCk6IFQge1xuICAgIHJldHVybiB0aGlzLl9jbG9uZUludG8oKTtcbiAgfVxufVxuXG4vKipcbiAqIFhPRjogc3RyZWFtaW5nIEFQSSB0byByZWFkIGRpZ2VzdCBpbiBjaHVua3MuXG4gKiBTYW1lIGFzICdzcXVlZXplJyBpbiBrZWNjYWsvazEyIGFuZCAnc2VlaycgaW4gYmxha2UzLCBidXQgbW9yZSBnZW5lcmljIG5hbWUuXG4gKiBXaGVuIGhhc2ggdXNlZCBpbiBYT0YgbW9kZSBpdCBpcyB1cCB0byB1c2VyIHRvIGNhbGwgJy5kZXN0cm95JyBhZnRlcndhcmRzLCBzaW5jZSB3ZSBjYW5ub3RcbiAqIGRlc3Ryb3kgc3RhdGUsIG5leHQgY2FsbCBjYW4gcmVxdWlyZSBtb3JlIGJ5dGVzLlxuICovXG5leHBvcnQgdHlwZSBIYXNoWE9GPFQgZXh0ZW5kcyBIYXNoPFQ+PiA9IEhhc2g8VD4gJiB7XG4gIHhvZihieXRlczogbnVtYmVyKTogVWludDhBcnJheTsgLy8gUmVhZCAnYnl0ZXMnIGJ5dGVzIGZyb20gZGlnZXN0IHN0cmVhbVxuICB4b2ZJbnRvKGJ1ZjogVWludDhBcnJheSk6IFVpbnQ4QXJyYXk7IC8vIHJlYWQgYnVmLmxlbmd0aCBieXRlcyBmcm9tIGRpZ2VzdCBzdHJlYW0gaW50byBidWZcbn07XG5cbi8vIENoZWNrIGlmIG9iamVjdCBkb2Vucyd0IGhhdmUgY3VzdG9tIGNvbnN0cnVjdG9yIChsaWtlIFVpbnQ4QXJyYXkvQXJyYXkpXG5jb25zdCBpc1BsYWluT2JqZWN0ID0gKG9iajogYW55KSA9PlxuICBPYmplY3QucHJvdG90eXBlLnRvU3RyaW5nLmNhbGwob2JqKSA9PT0gJ1tvYmplY3QgT2JqZWN0XScgJiYgb2JqLmNvbnN0cnVjdG9yID09PSBPYmplY3Q7XG5cbnR5cGUgRW1wdHlPYmogPSB7fTtcbmV4cG9ydCBmdW5jdGlvbiBjaGVja09wdHM8VDEgZXh0ZW5kcyBFbXB0eU9iaiwgVDIgZXh0ZW5kcyBFbXB0eU9iaj4oXG4gIGRlZmF1bHRzOiBUMSxcbiAgb3B0cz86IFQyXG4pOiBUMSAmIFQyIHtcbiAgaWYgKG9wdHMgIT09IHVuZGVmaW5lZCAmJiAodHlwZW9mIG9wdHMgIT09ICdvYmplY3QnIHx8ICFpc1BsYWluT2JqZWN0KG9wdHMpKSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoJ09wdGlvbnMgc2hvdWxkIGJlIG9iamVjdCBvciB1bmRlZmluZWQnKTtcbiAgY29uc3QgbWVyZ2VkID0gT2JqZWN0LmFzc2lnbihkZWZhdWx0cywgb3B0cyk7XG4gIHJldHVybiBtZXJnZWQgYXMgVDEgJiBUMjtcbn1cblxuZXhwb3J0IHR5cGUgQ0hhc2ggPSBSZXR1cm5UeXBlPHR5cGVvZiB3cmFwQ29uc3RydWN0b3I+O1xuXG5leHBvcnQgZnVuY3Rpb24gd3JhcENvbnN0cnVjdG9yPFQgZXh0ZW5kcyBIYXNoPFQ+PihoYXNoQ29uczogKCkgPT4gSGFzaDxUPikge1xuICBjb25zdCBoYXNoQyA9IChtc2c6IElucHV0KTogVWludDhBcnJheSA9PiBoYXNoQ29ucygpLnVwZGF0ZSh0b0J5dGVzKG1zZykpLmRpZ2VzdCgpO1xuICBjb25zdCB0bXAgPSBoYXNoQ29ucygpO1xuICBoYXNoQy5vdXRwdXRMZW4gPSB0bXAub3V0cHV0TGVuO1xuICBoYXNoQy5ibG9ja0xlbiA9IHRtcC5ibG9ja0xlbjtcbiAgaGFzaEMuY3JlYXRlID0gKCkgPT4gaGFzaENvbnMoKTtcbiAgcmV0dXJuIGhhc2hDO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gd3JhcENvbnN0cnVjdG9yV2l0aE9wdHM8SCBleHRlbmRzIEhhc2g8SD4sIFQgZXh0ZW5kcyBPYmplY3Q+KFxuICBoYXNoQ29uczogKG9wdHM/OiBUKSA9PiBIYXNoPEg+XG4pIHtcbiAgY29uc3QgaGFzaEMgPSAobXNnOiBJbnB1dCwgb3B0cz86IFQpOiBVaW50OEFycmF5ID0+IGhhc2hDb25zKG9wdHMpLnVwZGF0ZSh0b0J5dGVzKG1zZykpLmRpZ2VzdCgpO1xuICBjb25zdCB0bXAgPSBoYXNoQ29ucyh7fSBhcyBUKTtcbiAgaGFzaEMub3V0cHV0TGVuID0gdG1wLm91dHB1dExlbjtcbiAgaGFzaEMuYmxvY2tMZW4gPSB0bXAuYmxvY2tMZW47XG4gIGhhc2hDLmNyZWF0ZSA9IChvcHRzOiBUKSA9PiBoYXNoQ29ucyhvcHRzKTtcbiAgcmV0dXJuIGhhc2hDO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gd3JhcFhPRkNvbnN0cnVjdG9yV2l0aE9wdHM8SCBleHRlbmRzIEhhc2hYT0Y8SD4sIFQgZXh0ZW5kcyBPYmplY3Q+KFxuICBoYXNoQ29uczogKG9wdHM/OiBUKSA9PiBIYXNoWE9GPEg+XG4pIHtcbiAgY29uc3QgaGFzaEMgPSAobXNnOiBJbnB1dCwgb3B0cz86IFQpOiBVaW50OEFycmF5ID0+IGhhc2hDb25zKG9wdHMpLnVwZGF0ZSh0b0J5dGVzKG1zZykpLmRpZ2VzdCgpO1xuICBjb25zdCB0bXAgPSBoYXNoQ29ucyh7fSBhcyBUKTtcbiAgaGFzaEMub3V0cHV0TGVuID0gdG1wLm91dHB1dExlbjtcbiAgaGFzaEMuYmxvY2tMZW4gPSB0bXAuYmxvY2tMZW47XG4gIGhhc2hDLmNyZWF0ZSA9IChvcHRzOiBUKSA9PiBoYXNoQ29ucyhvcHRzKTtcbiAgcmV0dXJuIGhhc2hDO1xufVxuXG4vKipcbiAqIFNlY3VyZSBQUk5HLiBVc2VzIGBjcnlwdG8uZ2V0UmFuZG9tVmFsdWVzYCwgd2hpY2ggZGVmZXJzIHRvIE9TLlxuICovXG5leHBvcnQgZnVuY3Rpb24gcmFuZG9tQnl0ZXMoYnl0ZXNMZW5ndGggPSAzMik6IFVpbnQ4QXJyYXkge1xuICBpZiAoY3J5cHRvICYmIHR5cGVvZiBjcnlwdG8uZ2V0UmFuZG9tVmFsdWVzID09PSAnZnVuY3Rpb24nKSB7XG4gICAgcmV0dXJuIGNyeXB0by5nZXRSYW5kb21WYWx1ZXMobmV3IFVpbnQ4QXJyYXkoYnl0ZXNMZW5ndGgpKTtcbiAgfVxuICB0aHJvdyBuZXcgRXJyb3IoJ2NyeXB0by5nZXRSYW5kb21WYWx1ZXMgbXVzdCBiZSBkZWZpbmVkJyk7XG59XG4iLCAiZXhwb3J0IGZ1bmN0aW9uIG51bWJlcihuOiBudW1iZXIpIHtcbiAgaWYgKCFOdW1iZXIuaXNTYWZlSW50ZWdlcihuKSB8fCBuIDwgMCkgdGhyb3cgbmV3IEVycm9yKGBXcm9uZyBwb3NpdGl2ZSBpbnRlZ2VyOiAke259YCk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBib29sKGI6IGJvb2xlYW4pIHtcbiAgaWYgKHR5cGVvZiBiICE9PSAnYm9vbGVhbicpIHRocm93IG5ldyBFcnJvcihgRXhwZWN0ZWQgYm9vbGVhbiwgbm90ICR7Yn1gKTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGJ5dGVzKGI6IFVpbnQ4QXJyYXkgfCB1bmRlZmluZWQsIC4uLmxlbmd0aHM6IG51bWJlcltdKSB7XG4gIGlmICghKGIgaW5zdGFuY2VvZiBVaW50OEFycmF5KSkgdGhyb3cgbmV3IEVycm9yKCdFeHBlY3RlZCBVaW50OEFycmF5Jyk7XG4gIGlmIChsZW5ndGhzLmxlbmd0aCA+IDAgJiYgIWxlbmd0aHMuaW5jbHVkZXMoYi5sZW5ndGgpKVxuICAgIHRocm93IG5ldyBFcnJvcihgRXhwZWN0ZWQgVWludDhBcnJheSBvZiBsZW5ndGggJHtsZW5ndGhzfSwgbm90IG9mIGxlbmd0aD0ke2IubGVuZ3RofWApO1xufVxuXG50eXBlIEhhc2ggPSB7XG4gIChkYXRhOiBVaW50OEFycmF5KTogVWludDhBcnJheTtcbiAgYmxvY2tMZW46IG51bWJlcjtcbiAgb3V0cHV0TGVuOiBudW1iZXI7XG4gIGNyZWF0ZTogYW55O1xufTtcbmV4cG9ydCBmdW5jdGlvbiBoYXNoKGhhc2g6IEhhc2gpIHtcbiAgaWYgKHR5cGVvZiBoYXNoICE9PSAnZnVuY3Rpb24nIHx8IHR5cGVvZiBoYXNoLmNyZWF0ZSAhPT0gJ2Z1bmN0aW9uJylcbiAgICB0aHJvdyBuZXcgRXJyb3IoJ0hhc2ggc2hvdWxkIGJlIHdyYXBwZWQgYnkgdXRpbHMud3JhcENvbnN0cnVjdG9yJyk7XG4gIG51bWJlcihoYXNoLm91dHB1dExlbik7XG4gIG51bWJlcihoYXNoLmJsb2NrTGVuKTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIGV4aXN0cyhpbnN0YW5jZTogYW55LCBjaGVja0ZpbmlzaGVkID0gdHJ1ZSkge1xuICBpZiAoaW5zdGFuY2UuZGVzdHJveWVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2ggaW5zdGFuY2UgaGFzIGJlZW4gZGVzdHJveWVkJyk7XG4gIGlmIChjaGVja0ZpbmlzaGVkICYmIGluc3RhbmNlLmZpbmlzaGVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2gjZGlnZXN0KCkgaGFzIGFscmVhZHkgYmVlbiBjYWxsZWQnKTtcbn1cbmV4cG9ydCBmdW5jdGlvbiBvdXRwdXQob3V0OiBhbnksIGluc3RhbmNlOiBhbnkpIHtcbiAgYnl0ZXMob3V0KTtcbiAgY29uc3QgbWluID0gaW5zdGFuY2Uub3V0cHV0TGVuO1xuICBpZiAob3V0Lmxlbmd0aCA8IG1pbikge1xuICAgIHRocm93IG5ldyBFcnJvcihgZGlnZXN0SW50bygpIGV4cGVjdHMgb3V0cHV0IGJ1ZmZlciBvZiBsZW5ndGggYXQgbGVhc3QgJHttaW59YCk7XG4gIH1cbn1cblxuY29uc3QgYXNzZXJ0ID0ge1xuICBudW1iZXIsXG4gIGJvb2wsXG4gIGJ5dGVzLFxuICBoYXNoLFxuICBleGlzdHMsXG4gIG91dHB1dCxcbn07XG5cbmV4cG9ydCBkZWZhdWx0IGFzc2VydDtcbiIsICJpbXBvcnQgYXNzZXJ0IGZyb20gJy4vX2Fzc2VydC5qcyc7XG5pbXBvcnQgeyBIYXNoLCBjcmVhdGVWaWV3LCBJbnB1dCwgdG9CeXRlcyB9IGZyb20gJy4vdXRpbHMuanMnO1xuXG4vLyBQb2x5ZmlsbCBmb3IgU2FmYXJpIDE0XG5mdW5jdGlvbiBzZXRCaWdVaW50NjQodmlldzogRGF0YVZpZXcsIGJ5dGVPZmZzZXQ6IG51bWJlciwgdmFsdWU6IGJpZ2ludCwgaXNMRTogYm9vbGVhbik6IHZvaWQge1xuICBpZiAodHlwZW9mIHZpZXcuc2V0QmlnVWludDY0ID09PSAnZnVuY3Rpb24nKSByZXR1cm4gdmlldy5zZXRCaWdVaW50NjQoYnl0ZU9mZnNldCwgdmFsdWUsIGlzTEUpO1xuICBjb25zdCBfMzJuID0gQmlnSW50KDMyKTtcbiAgY29uc3QgX3UzMl9tYXggPSBCaWdJbnQoMHhmZmZmZmZmZik7XG4gIGNvbnN0IHdoID0gTnVtYmVyKCh2YWx1ZSA+PiBfMzJuKSAmIF91MzJfbWF4KTtcbiAgY29uc3Qgd2wgPSBOdW1iZXIodmFsdWUgJiBfdTMyX21heCk7XG4gIGNvbnN0IGggPSBpc0xFID8gNCA6IDA7XG4gIGNvbnN0IGwgPSBpc0xFID8gMCA6IDQ7XG4gIHZpZXcuc2V0VWludDMyKGJ5dGVPZmZzZXQgKyBoLCB3aCwgaXNMRSk7XG4gIHZpZXcuc2V0VWludDMyKGJ5dGVPZmZzZXQgKyBsLCB3bCwgaXNMRSk7XG59XG5cbi8vIEJhc2UgU0hBMiBjbGFzcyAoUkZDIDYyMzQpXG5leHBvcnQgYWJzdHJhY3QgY2xhc3MgU0hBMjxUIGV4dGVuZHMgU0hBMjxUPj4gZXh0ZW5kcyBIYXNoPFQ+IHtcbiAgcHJvdGVjdGVkIGFic3RyYWN0IHByb2Nlc3MoYnVmOiBEYXRhVmlldywgb2Zmc2V0OiBudW1iZXIpOiB2b2lkO1xuICBwcm90ZWN0ZWQgYWJzdHJhY3QgZ2V0KCk6IG51bWJlcltdO1xuICBwcm90ZWN0ZWQgYWJzdHJhY3Qgc2V0KC4uLmFyZ3M6IG51bWJlcltdKTogdm9pZDtcbiAgYWJzdHJhY3QgZGVzdHJveSgpOiB2b2lkO1xuICBwcm90ZWN0ZWQgYWJzdHJhY3Qgcm91bmRDbGVhbigpOiB2b2lkO1xuICAvLyBGb3IgcGFydGlhbCB1cGRhdGVzIGxlc3MgdGhhbiBibG9jayBzaXplXG4gIHByb3RlY3RlZCBidWZmZXI6IFVpbnQ4QXJyYXk7XG4gIHByb3RlY3RlZCB2aWV3OiBEYXRhVmlldztcbiAgcHJvdGVjdGVkIGZpbmlzaGVkID0gZmFsc2U7XG4gIHByb3RlY3RlZCBsZW5ndGggPSAwO1xuICBwcm90ZWN0ZWQgcG9zID0gMDtcbiAgcHJvdGVjdGVkIGRlc3Ryb3llZCA9IGZhbHNlO1xuXG4gIGNvbnN0cnVjdG9yKFxuICAgIHJlYWRvbmx5IGJsb2NrTGVuOiBudW1iZXIsXG4gICAgcHVibGljIG91dHB1dExlbjogbnVtYmVyLFxuICAgIHJlYWRvbmx5IHBhZE9mZnNldDogbnVtYmVyLFxuICAgIHJlYWRvbmx5IGlzTEU6IGJvb2xlYW5cbiAgKSB7XG4gICAgc3VwZXIoKTtcbiAgICB0aGlzLmJ1ZmZlciA9IG5ldyBVaW50OEFycmF5KGJsb2NrTGVuKTtcbiAgICB0aGlzLnZpZXcgPSBjcmVhdGVWaWV3KHRoaXMuYnVmZmVyKTtcbiAgfVxuICB1cGRhdGUoZGF0YTogSW5wdXQpOiB0aGlzIHtcbiAgICBhc3NlcnQuZXhpc3RzKHRoaXMpO1xuICAgIGNvbnN0IHsgdmlldywgYnVmZmVyLCBibG9ja0xlbiB9ID0gdGhpcztcbiAgICBkYXRhID0gdG9CeXRlcyhkYXRhKTtcbiAgICBjb25zdCBsZW4gPSBkYXRhLmxlbmd0aDtcbiAgICBmb3IgKGxldCBwb3MgPSAwOyBwb3MgPCBsZW47ICkge1xuICAgICAgY29uc3QgdGFrZSA9IE1hdGgubWluKGJsb2NrTGVuIC0gdGhpcy5wb3MsIGxlbiAtIHBvcyk7XG4gICAgICAvLyBGYXN0IHBhdGg6IHdlIGhhdmUgYXQgbGVhc3Qgb25lIGJsb2NrIGluIGlucHV0LCBjYXN0IGl0IHRvIHZpZXcgYW5kIHByb2Nlc3NcbiAgICAgIGlmICh0YWtlID09PSBibG9ja0xlbikge1xuICAgICAgICBjb25zdCBkYXRhVmlldyA9IGNyZWF0ZVZpZXcoZGF0YSk7XG4gICAgICAgIGZvciAoOyBibG9ja0xlbiA8PSBsZW4gLSBwb3M7IHBvcyArPSBibG9ja0xlbikgdGhpcy5wcm9jZXNzKGRhdGFWaWV3LCBwb3MpO1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICAgIGJ1ZmZlci5zZXQoZGF0YS5zdWJhcnJheShwb3MsIHBvcyArIHRha2UpLCB0aGlzLnBvcyk7XG4gICAgICB0aGlzLnBvcyArPSB0YWtlO1xuICAgICAgcG9zICs9IHRha2U7XG4gICAgICBpZiAodGhpcy5wb3MgPT09IGJsb2NrTGVuKSB7XG4gICAgICAgIHRoaXMucHJvY2Vzcyh2aWV3LCAwKTtcbiAgICAgICAgdGhpcy5wb3MgPSAwO1xuICAgICAgfVxuICAgIH1cbiAgICB0aGlzLmxlbmd0aCArPSBkYXRhLmxlbmd0aDtcbiAgICB0aGlzLnJvdW5kQ2xlYW4oKTtcbiAgICByZXR1cm4gdGhpcztcbiAgfVxuICBkaWdlc3RJbnRvKG91dDogVWludDhBcnJheSkge1xuICAgIGFzc2VydC5leGlzdHModGhpcyk7XG4gICAgYXNzZXJ0Lm91dHB1dChvdXQsIHRoaXMpO1xuICAgIHRoaXMuZmluaXNoZWQgPSB0cnVlO1xuICAgIC8vIFBhZGRpbmdcbiAgICAvLyBXZSBjYW4gYXZvaWQgYWxsb2NhdGlvbiBvZiBidWZmZXIgZm9yIHBhZGRpbmcgY29tcGxldGVseSBpZiBpdFxuICAgIC8vIHdhcyBwcmV2aW91c2x5IG5vdCBhbGxvY2F0ZWQgaGVyZS4gQnV0IGl0IHdvbid0IGNoYW5nZSBwZXJmb3JtYW5jZS5cbiAgICBjb25zdCB7IGJ1ZmZlciwgdmlldywgYmxvY2tMZW4sIGlzTEUgfSA9IHRoaXM7XG4gICAgbGV0IHsgcG9zIH0gPSB0aGlzO1xuICAgIC8vIGFwcGVuZCB0aGUgYml0ICcxJyB0byB0aGUgbWVzc2FnZVxuICAgIGJ1ZmZlcltwb3MrK10gPSAwYjEwMDAwMDAwO1xuICAgIHRoaXMuYnVmZmVyLnN1YmFycmF5KHBvcykuZmlsbCgwKTtcbiAgICAvLyB3ZSBoYXZlIGxlc3MgdGhhbiBwYWRPZmZzZXQgbGVmdCBpbiBidWZmZXIsIHNvIHdlIGNhbm5vdCBwdXQgbGVuZ3RoIGluIGN1cnJlbnQgYmxvY2ssIG5lZWQgcHJvY2VzcyBpdCBhbmQgcGFkIGFnYWluXG4gICAgaWYgKHRoaXMucGFkT2Zmc2V0ID4gYmxvY2tMZW4gLSBwb3MpIHtcbiAgICAgIHRoaXMucHJvY2Vzcyh2aWV3LCAwKTtcbiAgICAgIHBvcyA9IDA7XG4gICAgfVxuICAgIC8vIFBhZCB1bnRpbCBmdWxsIGJsb2NrIGJ5dGUgd2l0aCB6ZXJvc1xuICAgIGZvciAobGV0IGkgPSBwb3M7IGkgPCBibG9ja0xlbjsgaSsrKSBidWZmZXJbaV0gPSAwO1xuICAgIC8vIE5vdGU6IHNoYTUxMiByZXF1aXJlcyBsZW5ndGggdG8gYmUgMTI4Yml0IGludGVnZXIsIGJ1dCBsZW5ndGggaW4gSlMgd2lsbCBvdmVyZmxvdyBiZWZvcmUgdGhhdFxuICAgIC8vIFlvdSBuZWVkIHRvIHdyaXRlIGFyb3VuZCAyIGV4YWJ5dGVzICh1NjRfbWF4IC8gOCAvICgxMDI0Kio2KSkgZm9yIHRoaXMgdG8gaGFwcGVuLlxuICAgIC8vIFNvIHdlIGp1c3Qgd3JpdGUgbG93ZXN0IDY0IGJpdHMgb2YgdGhhdCB2YWx1ZS5cbiAgICBzZXRCaWdVaW50NjQodmlldywgYmxvY2tMZW4gLSA4LCBCaWdJbnQodGhpcy5sZW5ndGggKiA4KSwgaXNMRSk7XG4gICAgdGhpcy5wcm9jZXNzKHZpZXcsIDApO1xuICAgIGNvbnN0IG92aWV3ID0gY3JlYXRlVmlldyhvdXQpO1xuICAgIGNvbnN0IGxlbiA9IHRoaXMub3V0cHV0TGVuO1xuICAgIC8vIE5PVEU6IHdlIGRvIGRpdmlzaW9uIGJ5IDQgbGF0ZXIsIHdoaWNoIHNob3VsZCBiZSBmdXNlZCBpbiBzaW5nbGUgb3Agd2l0aCBtb2R1bG8gYnkgSklUXG4gICAgaWYgKGxlbiAlIDQpIHRocm93IG5ldyBFcnJvcignX3NoYTI6IG91dHB1dExlbiBzaG91bGQgYmUgYWxpZ25lZCB0byAzMmJpdCcpO1xuICAgIGNvbnN0IG91dExlbiA9IGxlbiAvIDQ7XG4gICAgY29uc3Qgc3RhdGUgPSB0aGlzLmdldCgpO1xuICAgIGlmIChvdXRMZW4gPiBzdGF0ZS5sZW5ndGgpIHRocm93IG5ldyBFcnJvcignX3NoYTI6IG91dHB1dExlbiBiaWdnZXIgdGhhbiBzdGF0ZScpO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgb3V0TGVuOyBpKyspIG92aWV3LnNldFVpbnQzMig0ICogaSwgc3RhdGVbaV0sIGlzTEUpO1xuICB9XG4gIGRpZ2VzdCgpIHtcbiAgICBjb25zdCB7IGJ1ZmZlciwgb3V0cHV0TGVuIH0gPSB0aGlzO1xuICAgIHRoaXMuZGlnZXN0SW50byhidWZmZXIpO1xuICAgIGNvbnN0IHJlcyA9IGJ1ZmZlci5zbGljZSgwLCBvdXRwdXRMZW4pO1xuICAgIHRoaXMuZGVzdHJveSgpO1xuICAgIHJldHVybiByZXM7XG4gIH1cbiAgX2Nsb25lSW50byh0bz86IFQpOiBUIHtcbiAgICB0byB8fD0gbmV3ICh0aGlzLmNvbnN0cnVjdG9yIGFzIGFueSkoKSBhcyBUO1xuICAgIHRvLnNldCguLi50aGlzLmdldCgpKTtcbiAgICBjb25zdCB7IGJsb2NrTGVuLCBidWZmZXIsIGxlbmd0aCwgZmluaXNoZWQsIGRlc3Ryb3llZCwgcG9zIH0gPSB0aGlzO1xuICAgIHRvLmxlbmd0aCA9IGxlbmd0aDtcbiAgICB0by5wb3MgPSBwb3M7XG4gICAgdG8uZmluaXNoZWQgPSBmaW5pc2hlZDtcbiAgICB0by5kZXN0cm95ZWQgPSBkZXN0cm95ZWQ7XG4gICAgaWYgKGxlbmd0aCAlIGJsb2NrTGVuKSB0by5idWZmZXIuc2V0KGJ1ZmZlcik7XG4gICAgcmV0dXJuIHRvO1xuICB9XG59XG4iLCAiaW1wb3J0IHsgU0hBMiB9IGZyb20gJy4vX3NoYTIuanMnO1xuaW1wb3J0IHsgcm90ciwgd3JhcENvbnN0cnVjdG9yIH0gZnJvbSAnLi91dGlscy5qcyc7XG5cbi8vIENob2ljZTogYSA/IGIgOiBjXG5jb25zdCBDaGkgPSAoYTogbnVtYmVyLCBiOiBudW1iZXIsIGM6IG51bWJlcikgPT4gKGEgJiBiKSBeICh+YSAmIGMpO1xuLy8gTWFqb3JpdHkgZnVuY3Rpb24sIHRydWUgaWYgYW55IHR3byBpbnB1c3QgaXMgdHJ1ZVxuY29uc3QgTWFqID0gKGE6IG51bWJlciwgYjogbnVtYmVyLCBjOiBudW1iZXIpID0+IChhICYgYikgXiAoYSAmIGMpIF4gKGIgJiBjKTtcblxuLy8gUm91bmQgY29uc3RhbnRzOlxuLy8gZmlyc3QgMzIgYml0cyBvZiB0aGUgZnJhY3Rpb25hbCBwYXJ0cyBvZiB0aGUgY3ViZSByb290cyBvZiB0aGUgZmlyc3QgNjQgcHJpbWVzIDIuLjMxMSlcbi8vIHByZXR0aWVyLWlnbm9yZVxuY29uc3QgU0hBMjU2X0sgPSBuZXcgVWludDMyQXJyYXkoW1xuICAweDQyOGEyZjk4LCAweDcxMzc0NDkxLCAweGI1YzBmYmNmLCAweGU5YjVkYmE1LCAweDM5NTZjMjViLCAweDU5ZjExMWYxLCAweDkyM2Y4MmE0LCAweGFiMWM1ZWQ1LFxuICAweGQ4MDdhYTk4LCAweDEyODM1YjAxLCAweDI0MzE4NWJlLCAweDU1MGM3ZGMzLCAweDcyYmU1ZDc0LCAweDgwZGViMWZlLCAweDliZGMwNmE3LCAweGMxOWJmMTc0LFxuICAweGU0OWI2OWMxLCAweGVmYmU0Nzg2LCAweDBmYzE5ZGM2LCAweDI0MGNhMWNjLCAweDJkZTkyYzZmLCAweDRhNzQ4NGFhLCAweDVjYjBhOWRjLCAweDc2Zjk4OGRhLFxuICAweDk4M2U1MTUyLCAweGE4MzFjNjZkLCAweGIwMDMyN2M4LCAweGJmNTk3ZmM3LCAweGM2ZTAwYmYzLCAweGQ1YTc5MTQ3LCAweDA2Y2E2MzUxLCAweDE0MjkyOTY3LFxuICAweDI3YjcwYTg1LCAweDJlMWIyMTM4LCAweDRkMmM2ZGZjLCAweDUzMzgwZDEzLCAweDY1MGE3MzU0LCAweDc2NmEwYWJiLCAweDgxYzJjOTJlLCAweDkyNzIyYzg1LFxuICAweGEyYmZlOGExLCAweGE4MWE2NjRiLCAweGMyNGI4YjcwLCAweGM3NmM1MWEzLCAweGQxOTJlODE5LCAweGQ2OTkwNjI0LCAweGY0MGUzNTg1LCAweDEwNmFhMDcwLFxuICAweDE5YTRjMTE2LCAweDFlMzc2YzA4LCAweDI3NDg3NzRjLCAweDM0YjBiY2I1LCAweDM5MWMwY2IzLCAweDRlZDhhYTRhLCAweDViOWNjYTRmLCAweDY4MmU2ZmYzLFxuICAweDc0OGY4MmVlLCAweDc4YTU2MzZmLCAweDg0Yzg3ODE0LCAweDhjYzcwMjA4LCAweDkwYmVmZmZhLCAweGE0NTA2Y2ViLCAweGJlZjlhM2Y3LCAweGM2NzE3OGYyXG5dKTtcblxuLy8gSW5pdGlhbCBzdGF0ZSAoZmlyc3QgMzIgYml0cyBvZiB0aGUgZnJhY3Rpb25hbCBwYXJ0cyBvZiB0aGUgc3F1YXJlIHJvb3RzIG9mIHRoZSBmaXJzdCA4IHByaW1lcyAyLi4xOSk6XG4vLyBwcmV0dGllci1pZ25vcmVcbmNvbnN0IElWID0gbmV3IFVpbnQzMkFycmF5KFtcbiAgMHg2YTA5ZTY2NywgMHhiYjY3YWU4NSwgMHgzYzZlZjM3MiwgMHhhNTRmZjUzYSwgMHg1MTBlNTI3ZiwgMHg5YjA1Njg4YywgMHgxZjgzZDlhYiwgMHg1YmUwY2QxOVxuXSk7XG5cbi8vIFRlbXBvcmFyeSBidWZmZXIsIG5vdCB1c2VkIHRvIHN0b3JlIGFueXRoaW5nIGJldHdlZW4gcnVuc1xuLy8gTmFtZWQgdGhpcyB3YXkgYmVjYXVzZSBpdCBtYXRjaGVzIHNwZWNpZmljYXRpb24uXG5jb25zdCBTSEEyNTZfVyA9IG5ldyBVaW50MzJBcnJheSg2NCk7XG5jbGFzcyBTSEEyNTYgZXh0ZW5kcyBTSEEyPFNIQTI1Nj4ge1xuICAvLyBXZSBjYW5ub3QgdXNlIGFycmF5IGhlcmUgc2luY2UgYXJyYXkgYWxsb3dzIGluZGV4aW5nIGJ5IHZhcmlhYmxlXG4gIC8vIHdoaWNoIG1lYW5zIG9wdGltaXplci9jb21waWxlciBjYW5ub3QgdXNlIHJlZ2lzdGVycy5cbiAgQSA9IElWWzBdIHwgMDtcbiAgQiA9IElWWzFdIHwgMDtcbiAgQyA9IElWWzJdIHwgMDtcbiAgRCA9IElWWzNdIHwgMDtcbiAgRSA9IElWWzRdIHwgMDtcbiAgRiA9IElWWzVdIHwgMDtcbiAgRyA9IElWWzZdIHwgMDtcbiAgSCA9IElWWzddIHwgMDtcblxuICBjb25zdHJ1Y3RvcigpIHtcbiAgICBzdXBlcig2NCwgMzIsIDgsIGZhbHNlKTtcbiAgfVxuICBwcm90ZWN0ZWQgZ2V0KCk6IFtudW1iZXIsIG51bWJlciwgbnVtYmVyLCBudW1iZXIsIG51bWJlciwgbnVtYmVyLCBudW1iZXIsIG51bWJlcl0ge1xuICAgIGNvbnN0IHsgQSwgQiwgQywgRCwgRSwgRiwgRywgSCB9ID0gdGhpcztcbiAgICByZXR1cm4gW0EsIEIsIEMsIEQsIEUsIEYsIEcsIEhdO1xuICB9XG4gIC8vIHByZXR0aWVyLWlnbm9yZVxuICBwcm90ZWN0ZWQgc2V0KFxuICAgIEE6IG51bWJlciwgQjogbnVtYmVyLCBDOiBudW1iZXIsIEQ6IG51bWJlciwgRTogbnVtYmVyLCBGOiBudW1iZXIsIEc6IG51bWJlciwgSDogbnVtYmVyXG4gICkge1xuICAgIHRoaXMuQSA9IEEgfCAwO1xuICAgIHRoaXMuQiA9IEIgfCAwO1xuICAgIHRoaXMuQyA9IEMgfCAwO1xuICAgIHRoaXMuRCA9IEQgfCAwO1xuICAgIHRoaXMuRSA9IEUgfCAwO1xuICAgIHRoaXMuRiA9IEYgfCAwO1xuICAgIHRoaXMuRyA9IEcgfCAwO1xuICAgIHRoaXMuSCA9IEggfCAwO1xuICB9XG4gIHByb3RlY3RlZCBwcm9jZXNzKHZpZXc6IERhdGFWaWV3LCBvZmZzZXQ6IG51bWJlcik6IHZvaWQge1xuICAgIC8vIEV4dGVuZCB0aGUgZmlyc3QgMTYgd29yZHMgaW50byB0aGUgcmVtYWluaW5nIDQ4IHdvcmRzIHdbMTYuLjYzXSBvZiB0aGUgbWVzc2FnZSBzY2hlZHVsZSBhcnJheVxuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgMTY7IGkrKywgb2Zmc2V0ICs9IDQpIFNIQTI1Nl9XW2ldID0gdmlldy5nZXRVaW50MzIob2Zmc2V0LCBmYWxzZSk7XG4gICAgZm9yIChsZXQgaSA9IDE2OyBpIDwgNjQ7IGkrKykge1xuICAgICAgY29uc3QgVzE1ID0gU0hBMjU2X1dbaSAtIDE1XTtcbiAgICAgIGNvbnN0IFcyID0gU0hBMjU2X1dbaSAtIDJdO1xuICAgICAgY29uc3QgczAgPSByb3RyKFcxNSwgNykgXiByb3RyKFcxNSwgMTgpIF4gKFcxNSA+Pj4gMyk7XG4gICAgICBjb25zdCBzMSA9IHJvdHIoVzIsIDE3KSBeIHJvdHIoVzIsIDE5KSBeIChXMiA+Pj4gMTApO1xuICAgICAgU0hBMjU2X1dbaV0gPSAoczEgKyBTSEEyNTZfV1tpIC0gN10gKyBzMCArIFNIQTI1Nl9XW2kgLSAxNl0pIHwgMDtcbiAgICB9XG4gICAgLy8gQ29tcHJlc3Npb24gZnVuY3Rpb24gbWFpbiBsb29wLCA2NCByb3VuZHNcbiAgICBsZXQgeyBBLCBCLCBDLCBELCBFLCBGLCBHLCBIIH0gPSB0aGlzO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgNjQ7IGkrKykge1xuICAgICAgY29uc3Qgc2lnbWExID0gcm90cihFLCA2KSBeIHJvdHIoRSwgMTEpIF4gcm90cihFLCAyNSk7XG4gICAgICBjb25zdCBUMSA9IChIICsgc2lnbWExICsgQ2hpKEUsIEYsIEcpICsgU0hBMjU2X0tbaV0gKyBTSEEyNTZfV1tpXSkgfCAwO1xuICAgICAgY29uc3Qgc2lnbWEwID0gcm90cihBLCAyKSBeIHJvdHIoQSwgMTMpIF4gcm90cihBLCAyMik7XG4gICAgICBjb25zdCBUMiA9IChzaWdtYTAgKyBNYWooQSwgQiwgQykpIHwgMDtcbiAgICAgIEggPSBHO1xuICAgICAgRyA9IEY7XG4gICAgICBGID0gRTtcbiAgICAgIEUgPSAoRCArIFQxKSB8IDA7XG4gICAgICBEID0gQztcbiAgICAgIEMgPSBCO1xuICAgICAgQiA9IEE7XG4gICAgICBBID0gKFQxICsgVDIpIHwgMDtcbiAgICB9XG4gICAgLy8gQWRkIHRoZSBjb21wcmVzc2VkIGNodW5rIHRvIHRoZSBjdXJyZW50IGhhc2ggdmFsdWVcbiAgICBBID0gKEEgKyB0aGlzLkEpIHwgMDtcbiAgICBCID0gKEIgKyB0aGlzLkIpIHwgMDtcbiAgICBDID0gKEMgKyB0aGlzLkMpIHwgMDtcbiAgICBEID0gKEQgKyB0aGlzLkQpIHwgMDtcbiAgICBFID0gKEUgKyB0aGlzLkUpIHwgMDtcbiAgICBGID0gKEYgKyB0aGlzLkYpIHwgMDtcbiAgICBHID0gKEcgKyB0aGlzLkcpIHwgMDtcbiAgICBIID0gKEggKyB0aGlzLkgpIHwgMDtcbiAgICB0aGlzLnNldChBLCBCLCBDLCBELCBFLCBGLCBHLCBIKTtcbiAgfVxuICBwcm90ZWN0ZWQgcm91bmRDbGVhbigpIHtcbiAgICBTSEEyNTZfVy5maWxsKDApO1xuICB9XG4gIGRlc3Ryb3koKSB7XG4gICAgdGhpcy5zZXQoMCwgMCwgMCwgMCwgMCwgMCwgMCwgMCk7XG4gICAgdGhpcy5idWZmZXIuZmlsbCgwKTtcbiAgfVxufVxuLy8gQ29uc3RhbnRzIGZyb20gaHR0cHM6Ly9udmxwdWJzLm5pc3QuZ292L25pc3RwdWJzL0ZJUFMvTklTVC5GSVBTLjE4MC00LnBkZlxuY2xhc3MgU0hBMjI0IGV4dGVuZHMgU0hBMjU2IHtcbiAgQSA9IDB4YzEwNTllZDggfCAwO1xuICBCID0gMHgzNjdjZDUwNyB8IDA7XG4gIEMgPSAweDMwNzBkZDE3IHwgMDtcbiAgRCA9IDB4ZjcwZTU5MzkgfCAwO1xuICBFID0gMHhmZmMwMGIzMSB8IDA7XG4gIEYgPSAweDY4NTgxNTExIHwgMDtcbiAgRyA9IDB4NjRmOThmYTcgfCAwO1xuICBIID0gMHhiZWZhNGZhNCB8IDA7XG4gIGNvbnN0cnVjdG9yKCkge1xuICAgIHN1cGVyKCk7XG4gICAgdGhpcy5vdXRwdXRMZW4gPSAyODtcbiAgfVxufVxuXG4vKipcbiAqIFNIQTItMjU2IGhhc2ggZnVuY3Rpb25cbiAqIEBwYXJhbSBtZXNzYWdlIC0gZGF0YSB0aGF0IHdvdWxkIGJlIGhhc2hlZFxuICovXG5leHBvcnQgY29uc3Qgc2hhMjU2ID0gd3JhcENvbnN0cnVjdG9yKCgpID0+IG5ldyBTSEEyNTYoKSk7XG5leHBvcnQgY29uc3Qgc2hhMjI0ID0gd3JhcENvbnN0cnVjdG9yKCgpID0+IG5ldyBTSEEyMjQoKSk7XG4iLCAiLyohIHNjdXJlLWJhc2UgLSBNSVQgTGljZW5zZSAoYykgMjAyMiBQYXVsIE1pbGxlciAocGF1bG1pbGxyLmNvbSkgKi9cbmV4cG9ydCBmdW5jdGlvbiBhc3NlcnROdW1iZXIobikge1xuICAgIGlmICghTnVtYmVyLmlzU2FmZUludGVnZXIobikpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgV3JvbmcgaW50ZWdlcjogJHtufWApO1xufVxuZnVuY3Rpb24gY2hhaW4oLi4uYXJncykge1xuICAgIGNvbnN0IHdyYXAgPSAoYSwgYikgPT4gKGMpID0+IGEoYihjKSk7XG4gICAgY29uc3QgZW5jb2RlID0gQXJyYXkuZnJvbShhcmdzKVxuICAgICAgICAucmV2ZXJzZSgpXG4gICAgICAgIC5yZWR1Y2UoKGFjYywgaSkgPT4gKGFjYyA/IHdyYXAoYWNjLCBpLmVuY29kZSkgOiBpLmVuY29kZSksIHVuZGVmaW5lZCk7XG4gICAgY29uc3QgZGVjb2RlID0gYXJncy5yZWR1Y2UoKGFjYywgaSkgPT4gKGFjYyA/IHdyYXAoYWNjLCBpLmRlY29kZSkgOiBpLmRlY29kZSksIHVuZGVmaW5lZCk7XG4gICAgcmV0dXJuIHsgZW5jb2RlLCBkZWNvZGUgfTtcbn1cbmZ1bmN0aW9uIGFscGhhYmV0KGFscGhhYmV0KSB7XG4gICAgcmV0dXJuIHtcbiAgICAgICAgZW5jb2RlOiAoZGlnaXRzKSA9PiB7XG4gICAgICAgICAgICBpZiAoIUFycmF5LmlzQXJyYXkoZGlnaXRzKSB8fCAoZGlnaXRzLmxlbmd0aCAmJiB0eXBlb2YgZGlnaXRzWzBdICE9PSAnbnVtYmVyJykpXG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdhbHBoYWJldC5lbmNvZGUgaW5wdXQgc2hvdWxkIGJlIGFuIGFycmF5IG9mIG51bWJlcnMnKTtcbiAgICAgICAgICAgIHJldHVybiBkaWdpdHMubWFwKChpKSA9PiB7XG4gICAgICAgICAgICAgICAgYXNzZXJ0TnVtYmVyKGkpO1xuICAgICAgICAgICAgICAgIGlmIChpIDwgMCB8fCBpID49IGFscGhhYmV0Lmxlbmd0aClcbiAgICAgICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBEaWdpdCBpbmRleCBvdXRzaWRlIGFscGhhYmV0OiAke2l9IChhbHBoYWJldDogJHthbHBoYWJldC5sZW5ndGh9KWApO1xuICAgICAgICAgICAgICAgIHJldHVybiBhbHBoYWJldFtpXTtcbiAgICAgICAgICAgIH0pO1xuICAgICAgICB9LFxuICAgICAgICBkZWNvZGU6IChpbnB1dCkgPT4ge1xuICAgICAgICAgICAgaWYgKCFBcnJheS5pc0FycmF5KGlucHV0KSB8fCAoaW5wdXQubGVuZ3RoICYmIHR5cGVvZiBpbnB1dFswXSAhPT0gJ3N0cmluZycpKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignYWxwaGFiZXQuZGVjb2RlIGlucHV0IHNob3VsZCBiZSBhcnJheSBvZiBzdHJpbmdzJyk7XG4gICAgICAgICAgICByZXR1cm4gaW5wdXQubWFwKChsZXR0ZXIpID0+IHtcbiAgICAgICAgICAgICAgICBpZiAodHlwZW9mIGxldHRlciAhPT0gJ3N0cmluZycpXG4gICAgICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgYWxwaGFiZXQuZGVjb2RlOiBub3Qgc3RyaW5nIGVsZW1lbnQ9JHtsZXR0ZXJ9YCk7XG4gICAgICAgICAgICAgICAgY29uc3QgaW5kZXggPSBhbHBoYWJldC5pbmRleE9mKGxldHRlcik7XG4gICAgICAgICAgICAgICAgaWYgKGluZGV4ID09PSAtMSlcbiAgICAgICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBVbmtub3duIGxldHRlcjogXCIke2xldHRlcn1cIi4gQWxsb3dlZDogJHthbHBoYWJldH1gKTtcbiAgICAgICAgICAgICAgICByZXR1cm4gaW5kZXg7XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgfSxcbiAgICB9O1xufVxuZnVuY3Rpb24gam9pbihzZXBhcmF0b3IgPSAnJykge1xuICAgIGlmICh0eXBlb2Ygc2VwYXJhdG9yICE9PSAnc3RyaW5nJylcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdqb2luIHNlcGFyYXRvciBzaG91bGQgYmUgc3RyaW5nJyk7XG4gICAgcmV0dXJuIHtcbiAgICAgICAgZW5jb2RlOiAoZnJvbSkgPT4ge1xuICAgICAgICAgICAgaWYgKCFBcnJheS5pc0FycmF5KGZyb20pIHx8IChmcm9tLmxlbmd0aCAmJiB0eXBlb2YgZnJvbVswXSAhPT0gJ3N0cmluZycpKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignam9pbi5lbmNvZGUgaW5wdXQgc2hvdWxkIGJlIGFycmF5IG9mIHN0cmluZ3MnKTtcbiAgICAgICAgICAgIGZvciAobGV0IGkgb2YgZnJvbSlcbiAgICAgICAgICAgICAgICBpZiAodHlwZW9mIGkgIT09ICdzdHJpbmcnKVxuICAgICAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYGpvaW4uZW5jb2RlOiBub24tc3RyaW5nIGlucHV0PSR7aX1gKTtcbiAgICAgICAgICAgIHJldHVybiBmcm9tLmpvaW4oc2VwYXJhdG9yKTtcbiAgICAgICAgfSxcbiAgICAgICAgZGVjb2RlOiAodG8pID0+IHtcbiAgICAgICAgICAgIGlmICh0eXBlb2YgdG8gIT09ICdzdHJpbmcnKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignam9pbi5kZWNvZGUgaW5wdXQgc2hvdWxkIGJlIHN0cmluZycpO1xuICAgICAgICAgICAgcmV0dXJuIHRvLnNwbGl0KHNlcGFyYXRvcik7XG4gICAgICAgIH0sXG4gICAgfTtcbn1cbmZ1bmN0aW9uIHBhZGRpbmcoYml0cywgY2hyID0gJz0nKSB7XG4gICAgYXNzZXJ0TnVtYmVyKGJpdHMpO1xuICAgIGlmICh0eXBlb2YgY2hyICE9PSAnc3RyaW5nJylcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdwYWRkaW5nIGNociBzaG91bGQgYmUgc3RyaW5nJyk7XG4gICAgcmV0dXJuIHtcbiAgICAgICAgZW5jb2RlKGRhdGEpIHtcbiAgICAgICAgICAgIGlmICghQXJyYXkuaXNBcnJheShkYXRhKSB8fCAoZGF0YS5sZW5ndGggJiYgdHlwZW9mIGRhdGFbMF0gIT09ICdzdHJpbmcnKSlcbiAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ3BhZGRpbmcuZW5jb2RlIGlucHV0IHNob3VsZCBiZSBhcnJheSBvZiBzdHJpbmdzJyk7XG4gICAgICAgICAgICBmb3IgKGxldCBpIG9mIGRhdGEpXG4gICAgICAgICAgICAgICAgaWYgKHR5cGVvZiBpICE9PSAnc3RyaW5nJylcbiAgICAgICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBwYWRkaW5nLmVuY29kZTogbm9uLXN0cmluZyBpbnB1dD0ke2l9YCk7XG4gICAgICAgICAgICB3aGlsZSAoKGRhdGEubGVuZ3RoICogYml0cykgJSA4KVxuICAgICAgICAgICAgICAgIGRhdGEucHVzaChjaHIpO1xuICAgICAgICAgICAgcmV0dXJuIGRhdGE7XG4gICAgICAgIH0sXG4gICAgICAgIGRlY29kZShpbnB1dCkge1xuICAgICAgICAgICAgaWYgKCFBcnJheS5pc0FycmF5KGlucHV0KSB8fCAoaW5wdXQubGVuZ3RoICYmIHR5cGVvZiBpbnB1dFswXSAhPT0gJ3N0cmluZycpKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcigncGFkZGluZy5lbmNvZGUgaW5wdXQgc2hvdWxkIGJlIGFycmF5IG9mIHN0cmluZ3MnKTtcbiAgICAgICAgICAgIGZvciAobGV0IGkgb2YgaW5wdXQpXG4gICAgICAgICAgICAgICAgaWYgKHR5cGVvZiBpICE9PSAnc3RyaW5nJylcbiAgICAgICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBwYWRkaW5nLmRlY29kZTogbm9uLXN0cmluZyBpbnB1dD0ke2l9YCk7XG4gICAgICAgICAgICBsZXQgZW5kID0gaW5wdXQubGVuZ3RoO1xuICAgICAgICAgICAgaWYgKChlbmQgKiBiaXRzKSAlIDgpXG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdJbnZhbGlkIHBhZGRpbmc6IHN0cmluZyBzaG91bGQgaGF2ZSB3aG9sZSBudW1iZXIgb2YgYnl0ZXMnKTtcbiAgICAgICAgICAgIGZvciAoOyBlbmQgPiAwICYmIGlucHV0W2VuZCAtIDFdID09PSBjaHI7IGVuZC0tKSB7XG4gICAgICAgICAgICAgICAgaWYgKCEoKChlbmQgLSAxKSAqIGJpdHMpICUgOCkpXG4gICAgICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignSW52YWxpZCBwYWRkaW5nOiBzdHJpbmcgaGFzIHRvbyBtdWNoIHBhZGRpbmcnKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIHJldHVybiBpbnB1dC5zbGljZSgwLCBlbmQpO1xuICAgICAgICB9LFxuICAgIH07XG59XG5mdW5jdGlvbiBub3JtYWxpemUoZm4pIHtcbiAgICBpZiAodHlwZW9mIGZuICE9PSAnZnVuY3Rpb24nKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ25vcm1hbGl6ZSBmbiBzaG91bGQgYmUgZnVuY3Rpb24nKTtcbiAgICByZXR1cm4geyBlbmNvZGU6IChmcm9tKSA9PiBmcm9tLCBkZWNvZGU6ICh0bykgPT4gZm4odG8pIH07XG59XG5mdW5jdGlvbiBjb252ZXJ0UmFkaXgoZGF0YSwgZnJvbSwgdG8pIHtcbiAgICBpZiAoZnJvbSA8IDIpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgY29udmVydFJhZGl4OiB3cm9uZyBmcm9tPSR7ZnJvbX0sIGJhc2UgY2Fubm90IGJlIGxlc3MgdGhhbiAyYCk7XG4gICAgaWYgKHRvIDwgMilcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBjb252ZXJ0UmFkaXg6IHdyb25nIHRvPSR7dG99LCBiYXNlIGNhbm5vdCBiZSBsZXNzIHRoYW4gMmApO1xuICAgIGlmICghQXJyYXkuaXNBcnJheShkYXRhKSlcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdjb252ZXJ0UmFkaXg6IGRhdGEgc2hvdWxkIGJlIGFycmF5Jyk7XG4gICAgaWYgKCFkYXRhLmxlbmd0aClcbiAgICAgICAgcmV0dXJuIFtdO1xuICAgIGxldCBwb3MgPSAwO1xuICAgIGNvbnN0IHJlcyA9IFtdO1xuICAgIGNvbnN0IGRpZ2l0cyA9IEFycmF5LmZyb20oZGF0YSk7XG4gICAgZGlnaXRzLmZvckVhY2goKGQpID0+IHtcbiAgICAgICAgYXNzZXJ0TnVtYmVyKGQpO1xuICAgICAgICBpZiAoZCA8IDAgfHwgZCA+PSBmcm9tKVxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBXcm9uZyBpbnRlZ2VyOiAke2R9YCk7XG4gICAgfSk7XG4gICAgd2hpbGUgKHRydWUpIHtcbiAgICAgICAgbGV0IGNhcnJ5ID0gMDtcbiAgICAgICAgbGV0IGRvbmUgPSB0cnVlO1xuICAgICAgICBmb3IgKGxldCBpID0gcG9zOyBpIDwgZGlnaXRzLmxlbmd0aDsgaSsrKSB7XG4gICAgICAgICAgICBjb25zdCBkaWdpdCA9IGRpZ2l0c1tpXTtcbiAgICAgICAgICAgIGNvbnN0IGRpZ2l0QmFzZSA9IGZyb20gKiBjYXJyeSArIGRpZ2l0O1xuICAgICAgICAgICAgaWYgKCFOdW1iZXIuaXNTYWZlSW50ZWdlcihkaWdpdEJhc2UpIHx8XG4gICAgICAgICAgICAgICAgKGZyb20gKiBjYXJyeSkgLyBmcm9tICE9PSBjYXJyeSB8fFxuICAgICAgICAgICAgICAgIGRpZ2l0QmFzZSAtIGRpZ2l0ICE9PSBmcm9tICogY2FycnkpIHtcbiAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ2NvbnZlcnRSYWRpeDogY2Fycnkgb3ZlcmZsb3cnKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGNhcnJ5ID0gZGlnaXRCYXNlICUgdG87XG4gICAgICAgICAgICBkaWdpdHNbaV0gPSBNYXRoLmZsb29yKGRpZ2l0QmFzZSAvIHRvKTtcbiAgICAgICAgICAgIGlmICghTnVtYmVyLmlzU2FmZUludGVnZXIoZGlnaXRzW2ldKSB8fCBkaWdpdHNbaV0gKiB0byArIGNhcnJ5ICE9PSBkaWdpdEJhc2UpXG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdjb252ZXJ0UmFkaXg6IGNhcnJ5IG92ZXJmbG93Jyk7XG4gICAgICAgICAgICBpZiAoIWRvbmUpXG4gICAgICAgICAgICAgICAgY29udGludWU7XG4gICAgICAgICAgICBlbHNlIGlmICghZGlnaXRzW2ldKVxuICAgICAgICAgICAgICAgIHBvcyA9IGk7XG4gICAgICAgICAgICBlbHNlXG4gICAgICAgICAgICAgICAgZG9uZSA9IGZhbHNlO1xuICAgICAgICB9XG4gICAgICAgIHJlcy5wdXNoKGNhcnJ5KTtcbiAgICAgICAgaWYgKGRvbmUpXG4gICAgICAgICAgICBicmVhaztcbiAgICB9XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCBkYXRhLmxlbmd0aCAtIDEgJiYgZGF0YVtpXSA9PT0gMDsgaSsrKVxuICAgICAgICByZXMucHVzaCgwKTtcbiAgICByZXR1cm4gcmVzLnJldmVyc2UoKTtcbn1cbmNvbnN0IGdjZCA9IChhLCBiKSA9PiAoIWIgPyBhIDogZ2NkKGIsIGEgJSBiKSk7XG5jb25zdCByYWRpeDJjYXJyeSA9IChmcm9tLCB0bykgPT4gZnJvbSArICh0byAtIGdjZChmcm9tLCB0bykpO1xuZnVuY3Rpb24gY29udmVydFJhZGl4MihkYXRhLCBmcm9tLCB0bywgcGFkZGluZykge1xuICAgIGlmICghQXJyYXkuaXNBcnJheShkYXRhKSlcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdjb252ZXJ0UmFkaXgyOiBkYXRhIHNob3VsZCBiZSBhcnJheScpO1xuICAgIGlmIChmcm9tIDw9IDAgfHwgZnJvbSA+IDMyKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYGNvbnZlcnRSYWRpeDI6IHdyb25nIGZyb209JHtmcm9tfWApO1xuICAgIGlmICh0byA8PSAwIHx8IHRvID4gMzIpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihgY29udmVydFJhZGl4Mjogd3JvbmcgdG89JHt0b31gKTtcbiAgICBpZiAocmFkaXgyY2FycnkoZnJvbSwgdG8pID4gMzIpIHtcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBjb252ZXJ0UmFkaXgyOiBjYXJyeSBvdmVyZmxvdyBmcm9tPSR7ZnJvbX0gdG89JHt0b30gY2FycnlCaXRzPSR7cmFkaXgyY2FycnkoZnJvbSwgdG8pfWApO1xuICAgIH1cbiAgICBsZXQgY2FycnkgPSAwO1xuICAgIGxldCBwb3MgPSAwO1xuICAgIGNvbnN0IG1hc2sgPSAyICoqIHRvIC0gMTtcbiAgICBjb25zdCByZXMgPSBbXTtcbiAgICBmb3IgKGNvbnN0IG4gb2YgZGF0YSkge1xuICAgICAgICBhc3NlcnROdW1iZXIobik7XG4gICAgICAgIGlmIChuID49IDIgKiogZnJvbSlcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgY29udmVydFJhZGl4MjogaW52YWxpZCBkYXRhIHdvcmQ9JHtufSBmcm9tPSR7ZnJvbX1gKTtcbiAgICAgICAgY2FycnkgPSAoY2FycnkgPDwgZnJvbSkgfCBuO1xuICAgICAgICBpZiAocG9zICsgZnJvbSA+IDMyKVxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBjb252ZXJ0UmFkaXgyOiBjYXJyeSBvdmVyZmxvdyBwb3M9JHtwb3N9IGZyb209JHtmcm9tfWApO1xuICAgICAgICBwb3MgKz0gZnJvbTtcbiAgICAgICAgZm9yICg7IHBvcyA+PSB0bzsgcG9zIC09IHRvKVxuICAgICAgICAgICAgcmVzLnB1c2goKChjYXJyeSA+PiAocG9zIC0gdG8pKSAmIG1hc2spID4+PiAwKTtcbiAgICAgICAgY2FycnkgJj0gMiAqKiBwb3MgLSAxO1xuICAgIH1cbiAgICBjYXJyeSA9IChjYXJyeSA8PCAodG8gLSBwb3MpKSAmIG1hc2s7XG4gICAgaWYgKCFwYWRkaW5nICYmIHBvcyA+PSBmcm9tKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ0V4Y2VzcyBwYWRkaW5nJyk7XG4gICAgaWYgKCFwYWRkaW5nICYmIGNhcnJ5KVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYE5vbi16ZXJvIHBhZGRpbmc6ICR7Y2Fycnl9YCk7XG4gICAgaWYgKHBhZGRpbmcgJiYgcG9zID4gMClcbiAgICAgICAgcmVzLnB1c2goY2FycnkgPj4+IDApO1xuICAgIHJldHVybiByZXM7XG59XG5mdW5jdGlvbiByYWRpeChudW0pIHtcbiAgICBhc3NlcnROdW1iZXIobnVtKTtcbiAgICByZXR1cm4ge1xuICAgICAgICBlbmNvZGU6IChieXRlcykgPT4ge1xuICAgICAgICAgICAgaWYgKCEoYnl0ZXMgaW5zdGFuY2VvZiBVaW50OEFycmF5KSlcbiAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ3JhZGl4LmVuY29kZSBpbnB1dCBzaG91bGQgYmUgVWludDhBcnJheScpO1xuICAgICAgICAgICAgcmV0dXJuIGNvbnZlcnRSYWRpeChBcnJheS5mcm9tKGJ5dGVzKSwgMiAqKiA4LCBudW0pO1xuICAgICAgICB9LFxuICAgICAgICBkZWNvZGU6IChkaWdpdHMpID0+IHtcbiAgICAgICAgICAgIGlmICghQXJyYXkuaXNBcnJheShkaWdpdHMpIHx8IChkaWdpdHMubGVuZ3RoICYmIHR5cGVvZiBkaWdpdHNbMF0gIT09ICdudW1iZXInKSlcbiAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ3JhZGl4LmRlY29kZSBpbnB1dCBzaG91bGQgYmUgYXJyYXkgb2Ygc3RyaW5ncycpO1xuICAgICAgICAgICAgcmV0dXJuIFVpbnQ4QXJyYXkuZnJvbShjb252ZXJ0UmFkaXgoZGlnaXRzLCBudW0sIDIgKiogOCkpO1xuICAgICAgICB9LFxuICAgIH07XG59XG5mdW5jdGlvbiByYWRpeDIoYml0cywgcmV2UGFkZGluZyA9IGZhbHNlKSB7XG4gICAgYXNzZXJ0TnVtYmVyKGJpdHMpO1xuICAgIGlmIChiaXRzIDw9IDAgfHwgYml0cyA+IDMyKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ3JhZGl4MjogYml0cyBzaG91bGQgYmUgaW4gKDAuLjMyXScpO1xuICAgIGlmIChyYWRpeDJjYXJyeSg4LCBiaXRzKSA+IDMyIHx8IHJhZGl4MmNhcnJ5KGJpdHMsIDgpID4gMzIpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcigncmFkaXgyOiBjYXJyeSBvdmVyZmxvdycpO1xuICAgIHJldHVybiB7XG4gICAgICAgIGVuY29kZTogKGJ5dGVzKSA9PiB7XG4gICAgICAgICAgICBpZiAoIShieXRlcyBpbnN0YW5jZW9mIFVpbnQ4QXJyYXkpKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcigncmFkaXgyLmVuY29kZSBpbnB1dCBzaG91bGQgYmUgVWludDhBcnJheScpO1xuICAgICAgICAgICAgcmV0dXJuIGNvbnZlcnRSYWRpeDIoQXJyYXkuZnJvbShieXRlcyksIDgsIGJpdHMsICFyZXZQYWRkaW5nKTtcbiAgICAgICAgfSxcbiAgICAgICAgZGVjb2RlOiAoZGlnaXRzKSA9PiB7XG4gICAgICAgICAgICBpZiAoIUFycmF5LmlzQXJyYXkoZGlnaXRzKSB8fCAoZGlnaXRzLmxlbmd0aCAmJiB0eXBlb2YgZGlnaXRzWzBdICE9PSAnbnVtYmVyJykpXG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdyYWRpeDIuZGVjb2RlIGlucHV0IHNob3VsZCBiZSBhcnJheSBvZiBzdHJpbmdzJyk7XG4gICAgICAgICAgICByZXR1cm4gVWludDhBcnJheS5mcm9tKGNvbnZlcnRSYWRpeDIoZGlnaXRzLCBiaXRzLCA4LCByZXZQYWRkaW5nKSk7XG4gICAgICAgIH0sXG4gICAgfTtcbn1cbmZ1bmN0aW9uIHVuc2FmZVdyYXBwZXIoZm4pIHtcbiAgICBpZiAodHlwZW9mIGZuICE9PSAnZnVuY3Rpb24nKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ3Vuc2FmZVdyYXBwZXIgZm4gc2hvdWxkIGJlIGZ1bmN0aW9uJyk7XG4gICAgcmV0dXJuIGZ1bmN0aW9uICguLi5hcmdzKSB7XG4gICAgICAgIHRyeSB7XG4gICAgICAgICAgICByZXR1cm4gZm4uYXBwbHkobnVsbCwgYXJncyk7XG4gICAgICAgIH1cbiAgICAgICAgY2F0Y2ggKGUpIHsgfVxuICAgIH07XG59XG5mdW5jdGlvbiBjaGVja3N1bShsZW4sIGZuKSB7XG4gICAgYXNzZXJ0TnVtYmVyKGxlbik7XG4gICAgaWYgKHR5cGVvZiBmbiAhPT0gJ2Z1bmN0aW9uJylcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdjaGVja3N1bSBmbiBzaG91bGQgYmUgZnVuY3Rpb24nKTtcbiAgICByZXR1cm4ge1xuICAgICAgICBlbmNvZGUoZGF0YSkge1xuICAgICAgICAgICAgaWYgKCEoZGF0YSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXkpKVxuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignY2hlY2tzdW0uZW5jb2RlOiBpbnB1dCBzaG91bGQgYmUgVWludDhBcnJheScpO1xuICAgICAgICAgICAgY29uc3QgY2hlY2tzdW0gPSBmbihkYXRhKS5zbGljZSgwLCBsZW4pO1xuICAgICAgICAgICAgY29uc3QgcmVzID0gbmV3IFVpbnQ4QXJyYXkoZGF0YS5sZW5ndGggKyBsZW4pO1xuICAgICAgICAgICAgcmVzLnNldChkYXRhKTtcbiAgICAgICAgICAgIHJlcy5zZXQoY2hlY2tzdW0sIGRhdGEubGVuZ3RoKTtcbiAgICAgICAgICAgIHJldHVybiByZXM7XG4gICAgICAgIH0sXG4gICAgICAgIGRlY29kZShkYXRhKSB7XG4gICAgICAgICAgICBpZiAoIShkYXRhIGluc3RhbmNlb2YgVWludDhBcnJheSkpXG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdjaGVja3N1bS5kZWNvZGU6IGlucHV0IHNob3VsZCBiZSBVaW50OEFycmF5Jyk7XG4gICAgICAgICAgICBjb25zdCBwYXlsb2FkID0gZGF0YS5zbGljZSgwLCAtbGVuKTtcbiAgICAgICAgICAgIGNvbnN0IG5ld0NoZWNrc3VtID0gZm4ocGF5bG9hZCkuc2xpY2UoMCwgbGVuKTtcbiAgICAgICAgICAgIGNvbnN0IG9sZENoZWNrc3VtID0gZGF0YS5zbGljZSgtbGVuKTtcbiAgICAgICAgICAgIGZvciAobGV0IGkgPSAwOyBpIDwgbGVuOyBpKyspXG4gICAgICAgICAgICAgICAgaWYgKG5ld0NoZWNrc3VtW2ldICE9PSBvbGRDaGVja3N1bVtpXSlcbiAgICAgICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKCdJbnZhbGlkIGNoZWNrc3VtJyk7XG4gICAgICAgICAgICByZXR1cm4gcGF5bG9hZDtcbiAgICAgICAgfSxcbiAgICB9O1xufVxuZXhwb3J0IGNvbnN0IHV0aWxzID0geyBhbHBoYWJldCwgY2hhaW4sIGNoZWNrc3VtLCByYWRpeCwgcmFkaXgyLCBqb2luLCBwYWRkaW5nIH07XG5leHBvcnQgY29uc3QgYmFzZTE2ID0gY2hhaW4ocmFkaXgyKDQpLCBhbHBoYWJldCgnMDEyMzQ1Njc4OUFCQ0RFRicpLCBqb2luKCcnKSk7XG5leHBvcnQgY29uc3QgYmFzZTMyID0gY2hhaW4ocmFkaXgyKDUpLCBhbHBoYWJldCgnQUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVoyMzQ1NjcnKSwgcGFkZGluZyg1KSwgam9pbignJykpO1xuZXhwb3J0IGNvbnN0IGJhc2UzMmhleCA9IGNoYWluKHJhZGl4Mig1KSwgYWxwaGFiZXQoJzAxMjM0NTY3ODlBQkNERUZHSElKS0xNTk9QUVJTVFVWJyksIHBhZGRpbmcoNSksIGpvaW4oJycpKTtcbmV4cG9ydCBjb25zdCBiYXNlMzJjcm9ja2ZvcmQgPSBjaGFpbihyYWRpeDIoNSksIGFscGhhYmV0KCcwMTIzNDU2Nzg5QUJDREVGR0hKS01OUFFSU1RWV1hZWicpLCBqb2luKCcnKSwgbm9ybWFsaXplKChzKSA9PiBzLnRvVXBwZXJDYXNlKCkucmVwbGFjZSgvTy9nLCAnMCcpLnJlcGxhY2UoL1tJTF0vZywgJzEnKSkpO1xuZXhwb3J0IGNvbnN0IGJhc2U2NCA9IGNoYWluKHJhZGl4Mig2KSwgYWxwaGFiZXQoJ0FCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXowMTIzNDU2Nzg5Ky8nKSwgcGFkZGluZyg2KSwgam9pbignJykpO1xuZXhwb3J0IGNvbnN0IGJhc2U2NHVybCA9IGNoYWluKHJhZGl4Mig2KSwgYWxwaGFiZXQoJ0FCQ0RFRkdISUpLTE1OT1BRUlNUVVZXWFlaYWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXowMTIzNDU2Nzg5LV8nKSwgcGFkZGluZyg2KSwgam9pbignJykpO1xuY29uc3QgZ2VuQmFzZTU4ID0gKGFiYykgPT4gY2hhaW4ocmFkaXgoNTgpLCBhbHBoYWJldChhYmMpLCBqb2luKCcnKSk7XG5leHBvcnQgY29uc3QgYmFzZTU4ID0gZ2VuQmFzZTU4KCcxMjM0NTY3ODlBQkNERUZHSEpLTE1OUFFSU1RVVldYWVphYmNkZWZnaGlqa21ub3BxcnN0dXZ3eHl6Jyk7XG5leHBvcnQgY29uc3QgYmFzZTU4ZmxpY2tyID0gZ2VuQmFzZTU4KCcxMjM0NTY3ODlhYmNkZWZnaGlqa21ub3BxcnN0dXZ3eHl6QUJDREVGR0hKS0xNTlBRUlNUVVZXWFlaJyk7XG5leHBvcnQgY29uc3QgYmFzZTU4eHJwID0gZ2VuQmFzZTU4KCdycHNobmFmMzl3QlVETkVHSEpLTE00UFFSU1Q3VldYWVoyYmNkZUNnNjVqa204b0ZxaTF0dXZBeHl6Jyk7XG5jb25zdCBYTVJfQkxPQ0tfTEVOID0gWzAsIDIsIDMsIDUsIDYsIDcsIDksIDEwLCAxMV07XG5leHBvcnQgY29uc3QgYmFzZTU4eG1yID0ge1xuICAgIGVuY29kZShkYXRhKSB7XG4gICAgICAgIGxldCByZXMgPSAnJztcbiAgICAgICAgZm9yIChsZXQgaSA9IDA7IGkgPCBkYXRhLmxlbmd0aDsgaSArPSA4KSB7XG4gICAgICAgICAgICBjb25zdCBibG9jayA9IGRhdGEuc3ViYXJyYXkoaSwgaSArIDgpO1xuICAgICAgICAgICAgcmVzICs9IGJhc2U1OC5lbmNvZGUoYmxvY2spLnBhZFN0YXJ0KFhNUl9CTE9DS19MRU5bYmxvY2subGVuZ3RoXSwgJzEnKTtcbiAgICAgICAgfVxuICAgICAgICByZXR1cm4gcmVzO1xuICAgIH0sXG4gICAgZGVjb2RlKHN0cikge1xuICAgICAgICBsZXQgcmVzID0gW107XG4gICAgICAgIGZvciAobGV0IGkgPSAwOyBpIDwgc3RyLmxlbmd0aDsgaSArPSAxMSkge1xuICAgICAgICAgICAgY29uc3Qgc2xpY2UgPSBzdHIuc2xpY2UoaSwgaSArIDExKTtcbiAgICAgICAgICAgIGNvbnN0IGJsb2NrTGVuID0gWE1SX0JMT0NLX0xFTi5pbmRleE9mKHNsaWNlLmxlbmd0aCk7XG4gICAgICAgICAgICBjb25zdCBibG9jayA9IGJhc2U1OC5kZWNvZGUoc2xpY2UpO1xuICAgICAgICAgICAgZm9yIChsZXQgaiA9IDA7IGogPCBibG9jay5sZW5ndGggLSBibG9ja0xlbjsgaisrKSB7XG4gICAgICAgICAgICAgICAgaWYgKGJsb2NrW2pdICE9PSAwKVxuICAgICAgICAgICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoJ2Jhc2U1OHhtcjogd3JvbmcgcGFkZGluZycpO1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgcmVzID0gcmVzLmNvbmNhdChBcnJheS5mcm9tKGJsb2NrLnNsaWNlKGJsb2NrLmxlbmd0aCAtIGJsb2NrTGVuKSkpO1xuICAgICAgICB9XG4gICAgICAgIHJldHVybiBVaW50OEFycmF5LmZyb20ocmVzKTtcbiAgICB9LFxufTtcbmV4cG9ydCBjb25zdCBiYXNlNThjaGVjayA9IChzaGEyNTYpID0+IGNoYWluKGNoZWNrc3VtKDQsIChkYXRhKSA9PiBzaGEyNTYoc2hhMjU2KGRhdGEpKSksIGJhc2U1OCk7XG5jb25zdCBCRUNIX0FMUEhBQkVUID0gY2hhaW4oYWxwaGFiZXQoJ3FwenJ5OXg4Z2YydHZkdzBzM2puNTRraGNlNm11YTdsJyksIGpvaW4oJycpKTtcbmNvbnN0IFBPTFlNT0RfR0VORVJBVE9SUyA9IFsweDNiNmE1N2IyLCAweDI2NTA4ZTZkLCAweDFlYTExOWZhLCAweDNkNDIzM2RkLCAweDJhMTQ2MmIzXTtcbmZ1bmN0aW9uIGJlY2gzMlBvbHltb2QocHJlKSB7XG4gICAgY29uc3QgYiA9IHByZSA+PiAyNTtcbiAgICBsZXQgY2hrID0gKHByZSAmIDB4MWZmZmZmZikgPDwgNTtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IFBPTFlNT0RfR0VORVJBVE9SUy5sZW5ndGg7IGkrKykge1xuICAgICAgICBpZiAoKChiID4+IGkpICYgMSkgPT09IDEpXG4gICAgICAgICAgICBjaGsgXj0gUE9MWU1PRF9HRU5FUkFUT1JTW2ldO1xuICAgIH1cbiAgICByZXR1cm4gY2hrO1xufVxuZnVuY3Rpb24gYmVjaENoZWNrc3VtKHByZWZpeCwgd29yZHMsIGVuY29kaW5nQ29uc3QgPSAxKSB7XG4gICAgY29uc3QgbGVuID0gcHJlZml4Lmxlbmd0aDtcbiAgICBsZXQgY2hrID0gMTtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IGxlbjsgaSsrKSB7XG4gICAgICAgIGNvbnN0IGMgPSBwcmVmaXguY2hhckNvZGVBdChpKTtcbiAgICAgICAgaWYgKGMgPCAzMyB8fCBjID4gMTI2KVxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBJbnZhbGlkIHByZWZpeCAoJHtwcmVmaXh9KWApO1xuICAgICAgICBjaGsgPSBiZWNoMzJQb2x5bW9kKGNoaykgXiAoYyA+PiA1KTtcbiAgICB9XG4gICAgY2hrID0gYmVjaDMyUG9seW1vZChjaGspO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgbGVuOyBpKyspXG4gICAgICAgIGNoayA9IGJlY2gzMlBvbHltb2QoY2hrKSBeIChwcmVmaXguY2hhckNvZGVBdChpKSAmIDB4MWYpO1xuICAgIGZvciAobGV0IHYgb2Ygd29yZHMpXG4gICAgICAgIGNoayA9IGJlY2gzMlBvbHltb2QoY2hrKSBeIHY7XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCA2OyBpKyspXG4gICAgICAgIGNoayA9IGJlY2gzMlBvbHltb2QoY2hrKTtcbiAgICBjaGsgXj0gZW5jb2RpbmdDb25zdDtcbiAgICByZXR1cm4gQkVDSF9BTFBIQUJFVC5lbmNvZGUoY29udmVydFJhZGl4MihbY2hrICUgMiAqKiAzMF0sIDMwLCA1LCBmYWxzZSkpO1xufVxuZnVuY3Rpb24gZ2VuQmVjaDMyKGVuY29kaW5nKSB7XG4gICAgY29uc3QgRU5DT0RJTkdfQ09OU1QgPSBlbmNvZGluZyA9PT0gJ2JlY2gzMicgPyAxIDogMHgyYmM4MzBhMztcbiAgICBjb25zdCBfd29yZHMgPSByYWRpeDIoNSk7XG4gICAgY29uc3QgZnJvbVdvcmRzID0gX3dvcmRzLmRlY29kZTtcbiAgICBjb25zdCB0b1dvcmRzID0gX3dvcmRzLmVuY29kZTtcbiAgICBjb25zdCBmcm9tV29yZHNVbnNhZmUgPSB1bnNhZmVXcmFwcGVyKGZyb21Xb3Jkcyk7XG4gICAgZnVuY3Rpb24gZW5jb2RlKHByZWZpeCwgd29yZHMsIGxpbWl0ID0gOTApIHtcbiAgICAgICAgaWYgKHR5cGVvZiBwcmVmaXggIT09ICdzdHJpbmcnKVxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBiZWNoMzIuZW5jb2RlIHByZWZpeCBzaG91bGQgYmUgc3RyaW5nLCBub3QgJHt0eXBlb2YgcHJlZml4fWApO1xuICAgICAgICBpZiAoIUFycmF5LmlzQXJyYXkod29yZHMpIHx8ICh3b3Jkcy5sZW5ndGggJiYgdHlwZW9mIHdvcmRzWzBdICE9PSAnbnVtYmVyJykpXG4gICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYGJlY2gzMi5lbmNvZGUgd29yZHMgc2hvdWxkIGJlIGFycmF5IG9mIG51bWJlcnMsIG5vdCAke3R5cGVvZiB3b3Jkc31gKTtcbiAgICAgICAgY29uc3QgYWN0dWFsTGVuZ3RoID0gcHJlZml4Lmxlbmd0aCArIDcgKyB3b3Jkcy5sZW5ndGg7XG4gICAgICAgIGlmIChsaW1pdCAhPT0gZmFsc2UgJiYgYWN0dWFsTGVuZ3RoID4gbGltaXQpXG4gICAgICAgICAgICB0aHJvdyBuZXcgVHlwZUVycm9yKGBMZW5ndGggJHthY3R1YWxMZW5ndGh9IGV4Y2VlZHMgbGltaXQgJHtsaW1pdH1gKTtcbiAgICAgICAgcHJlZml4ID0gcHJlZml4LnRvTG93ZXJDYXNlKCk7XG4gICAgICAgIHJldHVybiBgJHtwcmVmaXh9MSR7QkVDSF9BTFBIQUJFVC5lbmNvZGUod29yZHMpfSR7YmVjaENoZWNrc3VtKHByZWZpeCwgd29yZHMsIEVOQ09ESU5HX0NPTlNUKX1gO1xuICAgIH1cbiAgICBmdW5jdGlvbiBkZWNvZGUoc3RyLCBsaW1pdCA9IDkwKSB7XG4gICAgICAgIGlmICh0eXBlb2Ygc3RyICE9PSAnc3RyaW5nJylcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgYmVjaDMyLmRlY29kZSBpbnB1dCBzaG91bGQgYmUgc3RyaW5nLCBub3QgJHt0eXBlb2Ygc3RyfWApO1xuICAgICAgICBpZiAoc3RyLmxlbmd0aCA8IDggfHwgKGxpbWl0ICE9PSBmYWxzZSAmJiBzdHIubGVuZ3RoID4gbGltaXQpKVxuICAgICAgICAgICAgdGhyb3cgbmV3IFR5cGVFcnJvcihgV3Jvbmcgc3RyaW5nIGxlbmd0aDogJHtzdHIubGVuZ3RofSAoJHtzdHJ9KS4gRXhwZWN0ZWQgKDguLiR7bGltaXR9KWApO1xuICAgICAgICBjb25zdCBsb3dlcmVkID0gc3RyLnRvTG93ZXJDYXNlKCk7XG4gICAgICAgIGlmIChzdHIgIT09IGxvd2VyZWQgJiYgc3RyICE9PSBzdHIudG9VcHBlckNhc2UoKSlcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgU3RyaW5nIG11c3QgYmUgbG93ZXJjYXNlIG9yIHVwcGVyY2FzZWApO1xuICAgICAgICBzdHIgPSBsb3dlcmVkO1xuICAgICAgICBjb25zdCBzZXBJbmRleCA9IHN0ci5sYXN0SW5kZXhPZignMScpO1xuICAgICAgICBpZiAoc2VwSW5kZXggPT09IDAgfHwgc2VwSW5kZXggPT09IC0xKVxuICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBMZXR0ZXIgXCIxXCIgbXVzdCBiZSBwcmVzZW50IGJldHdlZW4gcHJlZml4IGFuZCBkYXRhIG9ubHlgKTtcbiAgICAgICAgY29uc3QgcHJlZml4ID0gc3RyLnNsaWNlKDAsIHNlcEluZGV4KTtcbiAgICAgICAgY29uc3QgX3dvcmRzID0gc3RyLnNsaWNlKHNlcEluZGV4ICsgMSk7XG4gICAgICAgIGlmIChfd29yZHMubGVuZ3RoIDwgNilcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcignRGF0YSBtdXN0IGJlIGF0IGxlYXN0IDYgY2hhcmFjdGVycyBsb25nJyk7XG4gICAgICAgIGNvbnN0IHdvcmRzID0gQkVDSF9BTFBIQUJFVC5kZWNvZGUoX3dvcmRzKS5zbGljZSgwLCAtNik7XG4gICAgICAgIGNvbnN0IHN1bSA9IGJlY2hDaGVja3N1bShwcmVmaXgsIHdvcmRzLCBFTkNPRElOR19DT05TVCk7XG4gICAgICAgIGlmICghX3dvcmRzLmVuZHNXaXRoKHN1bSkpXG4gICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYEludmFsaWQgY2hlY2tzdW0gaW4gJHtzdHJ9OiBleHBlY3RlZCBcIiR7c3VtfVwiYCk7XG4gICAgICAgIHJldHVybiB7IHByZWZpeCwgd29yZHMgfTtcbiAgICB9XG4gICAgY29uc3QgZGVjb2RlVW5zYWZlID0gdW5zYWZlV3JhcHBlcihkZWNvZGUpO1xuICAgIGZ1bmN0aW9uIGRlY29kZVRvQnl0ZXMoc3RyKSB7XG4gICAgICAgIGNvbnN0IHsgcHJlZml4LCB3b3JkcyB9ID0gZGVjb2RlKHN0ciwgZmFsc2UpO1xuICAgICAgICByZXR1cm4geyBwcmVmaXgsIHdvcmRzLCBieXRlczogZnJvbVdvcmRzKHdvcmRzKSB9O1xuICAgIH1cbiAgICByZXR1cm4geyBlbmNvZGUsIGRlY29kZSwgZGVjb2RlVG9CeXRlcywgZGVjb2RlVW5zYWZlLCBmcm9tV29yZHMsIGZyb21Xb3Jkc1Vuc2FmZSwgdG9Xb3JkcyB9O1xufVxuZXhwb3J0IGNvbnN0IGJlY2gzMiA9IGdlbkJlY2gzMignYmVjaDMyJyk7XG5leHBvcnQgY29uc3QgYmVjaDMybSA9IGdlbkJlY2gzMignYmVjaDMybScpO1xuZXhwb3J0IGNvbnN0IHV0ZjggPSB7XG4gICAgZW5jb2RlOiAoZGF0YSkgPT4gbmV3IFRleHREZWNvZGVyKCkuZGVjb2RlKGRhdGEpLFxuICAgIGRlY29kZTogKHN0cikgPT4gbmV3IFRleHRFbmNvZGVyKCkuZW5jb2RlKHN0ciksXG59O1xuZXhwb3J0IGNvbnN0IGhleCA9IGNoYWluKHJhZGl4Mig0KSwgYWxwaGFiZXQoJzAxMjM0NTY3ODlhYmNkZWYnKSwgam9pbignJyksIG5vcm1hbGl6ZSgocykgPT4ge1xuICAgIGlmICh0eXBlb2YgcyAhPT0gJ3N0cmluZycgfHwgcy5sZW5ndGggJSAyKVxuICAgICAgICB0aHJvdyBuZXcgVHlwZUVycm9yKGBoZXguZGVjb2RlOiBleHBlY3RlZCBzdHJpbmcsIGdvdCAke3R5cGVvZiBzfSB3aXRoIGxlbmd0aCAke3MubGVuZ3RofWApO1xuICAgIHJldHVybiBzLnRvTG93ZXJDYXNlKCk7XG59KSk7XG5jb25zdCBDT0RFUlMgPSB7XG4gICAgdXRmOCwgaGV4LCBiYXNlMTYsIGJhc2UzMiwgYmFzZTY0LCBiYXNlNjR1cmwsIGJhc2U1OCwgYmFzZTU4eG1yXG59O1xuY29uc3QgY29kZXJUeXBlRXJyb3IgPSBgSW52YWxpZCBlbmNvZGluZyB0eXBlLiBBdmFpbGFibGUgdHlwZXM6ICR7T2JqZWN0LmtleXMoQ09ERVJTKS5qb2luKCcsICcpfWA7XG5leHBvcnQgY29uc3QgYnl0ZXNUb1N0cmluZyA9ICh0eXBlLCBieXRlcykgPT4ge1xuICAgIGlmICh0eXBlb2YgdHlwZSAhPT0gJ3N0cmluZycgfHwgIUNPREVSUy5oYXNPd25Qcm9wZXJ0eSh0eXBlKSlcbiAgICAgICAgdGhyb3cgbmV3IFR5cGVFcnJvcihjb2RlclR5cGVFcnJvcik7XG4gICAgaWYgKCEoYnl0ZXMgaW5zdGFuY2VvZiBVaW50OEFycmF5KSlcbiAgICAgICAgdGhyb3cgbmV3IFR5cGVFcnJvcignYnl0ZXNUb1N0cmluZygpIGV4cGVjdHMgVWludDhBcnJheScpO1xuICAgIHJldHVybiBDT0RFUlNbdHlwZV0uZW5jb2RlKGJ5dGVzKTtcbn07XG5leHBvcnQgY29uc3Qgc3RyID0gYnl0ZXNUb1N0cmluZztcbmV4cG9ydCBjb25zdCBzdHJpbmdUb0J5dGVzID0gKHR5cGUsIHN0cikgPT4ge1xuICAgIGlmICghQ09ERVJTLmhhc093blByb3BlcnR5KHR5cGUpKVxuICAgICAgICB0aHJvdyBuZXcgVHlwZUVycm9yKGNvZGVyVHlwZUVycm9yKTtcbiAgICBpZiAodHlwZW9mIHN0ciAhPT0gJ3N0cmluZycpXG4gICAgICAgIHRocm93IG5ldyBUeXBlRXJyb3IoJ3N0cmluZ1RvQnl0ZXMoKSBleHBlY3RzIHN0cmluZycpO1xuICAgIHJldHVybiBDT0RFUlNbdHlwZV0uZGVjb2RlKHN0cik7XG59O1xuZXhwb3J0IGNvbnN0IGJ5dGVzID0gc3RyaW5nVG9CeXRlcztcbiIsICJmdW5jdGlvbiBudW1iZXIobjogbnVtYmVyKSB7XG4gIGlmICghTnVtYmVyLmlzU2FmZUludGVnZXIobikgfHwgbiA8IDApIHRocm93IG5ldyBFcnJvcihgcG9zaXRpdmUgaW50ZWdlciBleHBlY3RlZCwgbm90ICR7bn1gKTtcbn1cblxuZnVuY3Rpb24gYm9vbChiOiBib29sZWFuKSB7XG4gIGlmICh0eXBlb2YgYiAhPT0gJ2Jvb2xlYW4nKSB0aHJvdyBuZXcgRXJyb3IoYGJvb2xlYW4gZXhwZWN0ZWQsIG5vdCAke2J9YCk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBpc0J5dGVzKGE6IHVua25vd24pOiBhIGlzIFVpbnQ4QXJyYXkge1xuICByZXR1cm4gKFxuICAgIGEgaW5zdGFuY2VvZiBVaW50OEFycmF5IHx8XG4gICAgKGEgIT0gbnVsbCAmJiB0eXBlb2YgYSA9PT0gJ29iamVjdCcgJiYgYS5jb25zdHJ1Y3Rvci5uYW1lID09PSAnVWludDhBcnJheScpXG4gICk7XG59XG5cbmZ1bmN0aW9uIGJ5dGVzKGI6IFVpbnQ4QXJyYXkgfCB1bmRlZmluZWQsIC4uLmxlbmd0aHM6IG51bWJlcltdKSB7XG4gIGlmICghaXNCeXRlcyhiKSkgdGhyb3cgbmV3IEVycm9yKCdVaW50OEFycmF5IGV4cGVjdGVkJyk7XG4gIGlmIChsZW5ndGhzLmxlbmd0aCA+IDAgJiYgIWxlbmd0aHMuaW5jbHVkZXMoYi5sZW5ndGgpKVxuICAgIHRocm93IG5ldyBFcnJvcihgVWludDhBcnJheSBleHBlY3RlZCBvZiBsZW5ndGggJHtsZW5ndGhzfSwgbm90IG9mIGxlbmd0aD0ke2IubGVuZ3RofWApO1xufVxuXG5leHBvcnQgdHlwZSBIYXNoID0ge1xuICAoZGF0YTogVWludDhBcnJheSk6IFVpbnQ4QXJyYXk7XG4gIGJsb2NrTGVuOiBudW1iZXI7XG4gIG91dHB1dExlbjogbnVtYmVyO1xuICBjcmVhdGU6IGFueTtcbn07XG5mdW5jdGlvbiBoYXNoKGhhc2g6IEhhc2gpIHtcbiAgaWYgKHR5cGVvZiBoYXNoICE9PSAnZnVuY3Rpb24nIHx8IHR5cGVvZiBoYXNoLmNyZWF0ZSAhPT0gJ2Z1bmN0aW9uJylcbiAgICB0aHJvdyBuZXcgRXJyb3IoJ2hhc2ggbXVzdCBiZSB3cmFwcGVkIGJ5IHV0aWxzLndyYXBDb25zdHJ1Y3RvcicpO1xuICBudW1iZXIoaGFzaC5vdXRwdXRMZW4pO1xuICBudW1iZXIoaGFzaC5ibG9ja0xlbik7XG59XG5cbmZ1bmN0aW9uIGV4aXN0cyhpbnN0YW5jZTogYW55LCBjaGVja0ZpbmlzaGVkID0gdHJ1ZSkge1xuICBpZiAoaW5zdGFuY2UuZGVzdHJveWVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2ggaW5zdGFuY2UgaGFzIGJlZW4gZGVzdHJveWVkJyk7XG4gIGlmIChjaGVja0ZpbmlzaGVkICYmIGluc3RhbmNlLmZpbmlzaGVkKSB0aHJvdyBuZXcgRXJyb3IoJ0hhc2gjZGlnZXN0KCkgaGFzIGFscmVhZHkgYmVlbiBjYWxsZWQnKTtcbn1cblxuZnVuY3Rpb24gb3V0cHV0KG91dDogYW55LCBpbnN0YW5jZTogYW55KSB7XG4gIGJ5dGVzKG91dCk7XG4gIGNvbnN0IG1pbiA9IGluc3RhbmNlLm91dHB1dExlbjtcbiAgaWYgKG91dC5sZW5ndGggPCBtaW4pIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoYGRpZ2VzdEludG8oKSBleHBlY3RzIG91dHB1dCBidWZmZXIgb2YgbGVuZ3RoIGF0IGxlYXN0ICR7bWlufWApO1xuICB9XG59XG5cbmV4cG9ydCB7IG51bWJlciwgYm9vbCwgYnl0ZXMsIGhhc2gsIGV4aXN0cywgb3V0cHV0IH07XG5jb25zdCBhc3NlcnQgPSB7IG51bWJlciwgYm9vbCwgYnl0ZXMsIGhhc2gsIGV4aXN0cywgb3V0cHV0IH07XG5leHBvcnQgZGVmYXVsdCBhc3NlcnQ7XG4iLCAiLyohIG5vYmxlLWNpcGhlcnMgLSBNSVQgTGljZW5zZSAoYykgMjAyMyBQYXVsIE1pbGxlciAocGF1bG1pbGxyLmNvbSkgKi9cbmltcG9ydCB7IGJ5dGVzIGFzIGFieXRlcywgaXNCeXRlcyB9IGZyb20gJy4vX2Fzc2VydC5qcyc7XG4vLyBwcmV0dGllci1pZ25vcmVcbmV4cG9ydCB0eXBlIFR5cGVkQXJyYXkgPSBJbnQ4QXJyYXkgfCBVaW50OENsYW1wZWRBcnJheSB8IFVpbnQ4QXJyYXkgfFxuICBVaW50MTZBcnJheSB8IEludDE2QXJyYXkgfCBVaW50MzJBcnJheSB8IEludDMyQXJyYXk7XG5cbi8vIENhc3QgYXJyYXkgdG8gZGlmZmVyZW50IHR5cGVcbmV4cG9ydCBjb25zdCB1OCA9IChhcnI6IFR5cGVkQXJyYXkpID0+IG5ldyBVaW50OEFycmF5KGFyci5idWZmZXIsIGFyci5ieXRlT2Zmc2V0LCBhcnIuYnl0ZUxlbmd0aCk7XG5leHBvcnQgY29uc3QgdTE2ID0gKGFycjogVHlwZWRBcnJheSkgPT5cbiAgbmV3IFVpbnQxNkFycmF5KGFyci5idWZmZXIsIGFyci5ieXRlT2Zmc2V0LCBNYXRoLmZsb29yKGFyci5ieXRlTGVuZ3RoIC8gMikpO1xuZXhwb3J0IGNvbnN0IHUzMiA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBVaW50MzJBcnJheShhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgTWF0aC5mbG9vcihhcnIuYnl0ZUxlbmd0aCAvIDQpKTtcblxuLy8gQ2FzdCBhcnJheSB0byB2aWV3XG5leHBvcnQgY29uc3QgY3JlYXRlVmlldyA9IChhcnI6IFR5cGVkQXJyYXkpID0+XG4gIG5ldyBEYXRhVmlldyhhcnIuYnVmZmVyLCBhcnIuYnl0ZU9mZnNldCwgYXJyLmJ5dGVMZW5ndGgpO1xuXG4vLyBiaWctZW5kaWFuIGhhcmR3YXJlIGlzIHJhcmUuIEp1c3QgaW4gY2FzZSBzb21lb25lIHN0aWxsIGRlY2lkZXMgdG8gcnVuIGNpcGhlcnM6XG4vLyBlYXJseS10aHJvdyBhbiBlcnJvciBiZWNhdXNlIHdlIGRvbid0IHN1cHBvcnQgQkUgeWV0LlxuZXhwb3J0IGNvbnN0IGlzTEUgPSBuZXcgVWludDhBcnJheShuZXcgVWludDMyQXJyYXkoWzB4MTEyMjMzNDRdKS5idWZmZXIpWzBdID09PSAweDQ0O1xuaWYgKCFpc0xFKSB0aHJvdyBuZXcgRXJyb3IoJ05vbiBsaXR0bGUtZW5kaWFuIGhhcmR3YXJlIGlzIG5vdCBzdXBwb3J0ZWQnKTtcblxuLy8gQXJyYXkgd2hlcmUgaW5kZXggMHhmMCAoMjQwKSBpcyBtYXBwZWQgdG8gc3RyaW5nICdmMCdcbmNvbnN0IGhleGVzID0gLyogQF9fUFVSRV9fICovIEFycmF5LmZyb20oeyBsZW5ndGg6IDI1NiB9LCAoXywgaSkgPT5cbiAgaS50b1N0cmluZygxNikucGFkU3RhcnQoMiwgJzAnKVxuKTtcbi8qKlxuICogQGV4YW1wbGUgYnl0ZXNUb0hleChVaW50OEFycmF5LmZyb20oWzB4Y2EsIDB4ZmUsIDB4MDEsIDB4MjNdKSkgLy8gJ2NhZmUwMTIzJ1xuICovXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb0hleChieXRlczogVWludDhBcnJheSk6IHN0cmluZyB7XG4gIGFieXRlcyhieXRlcyk7XG4gIC8vIHByZS1jYWNoaW5nIGltcHJvdmVzIHRoZSBzcGVlZCA2eFxuICBsZXQgaGV4ID0gJyc7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgYnl0ZXMubGVuZ3RoOyBpKyspIHtcbiAgICBoZXggKz0gaGV4ZXNbYnl0ZXNbaV1dO1xuICB9XG4gIHJldHVybiBoZXg7XG59XG5cbi8vIFdlIHVzZSBvcHRpbWl6ZWQgdGVjaG5pcXVlIHRvIGNvbnZlcnQgaGV4IHN0cmluZyB0byBieXRlIGFycmF5XG5jb25zdCBhc2NpaXMgPSB7IF8wOiA0OCwgXzk6IDU3LCBfQTogNjUsIF9GOiA3MCwgX2E6IDk3LCBfZjogMTAyIH0gYXMgY29uc3Q7XG5mdW5jdGlvbiBhc2NpaVRvQmFzZTE2KGNoYXI6IG51bWJlcik6IG51bWJlciB8IHVuZGVmaW5lZCB7XG4gIGlmIChjaGFyID49IGFzY2lpcy5fMCAmJiBjaGFyIDw9IGFzY2lpcy5fOSkgcmV0dXJuIGNoYXIgLSBhc2NpaXMuXzA7XG4gIGlmIChjaGFyID49IGFzY2lpcy5fQSAmJiBjaGFyIDw9IGFzY2lpcy5fRikgcmV0dXJuIGNoYXIgLSAoYXNjaWlzLl9BIC0gMTApO1xuICBpZiAoY2hhciA+PSBhc2NpaXMuX2EgJiYgY2hhciA8PSBhc2NpaXMuX2YpIHJldHVybiBjaGFyIC0gKGFzY2lpcy5fYSAtIDEwKTtcbiAgcmV0dXJuO1xufVxuXG4vKipcbiAqIEBleGFtcGxlIGhleFRvQnl0ZXMoJ2NhZmUwMTIzJykgLy8gVWludDhBcnJheS5mcm9tKFsweGNhLCAweGZlLCAweDAxLCAweDIzXSlcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIGhleFRvQnl0ZXMoaGV4OiBzdHJpbmcpOiBVaW50OEFycmF5IHtcbiAgaWYgKHR5cGVvZiBoZXggIT09ICdzdHJpbmcnKSB0aHJvdyBuZXcgRXJyb3IoJ2hleCBzdHJpbmcgZXhwZWN0ZWQsIGdvdCAnICsgdHlwZW9mIGhleCk7XG4gIGNvbnN0IGhsID0gaGV4Lmxlbmd0aDtcbiAgY29uc3QgYWwgPSBobCAvIDI7XG4gIGlmIChobCAlIDIpIHRocm93IG5ldyBFcnJvcigncGFkZGVkIGhleCBzdHJpbmcgZXhwZWN0ZWQsIGdvdCB1bnBhZGRlZCBoZXggb2YgbGVuZ3RoICcgKyBobCk7XG4gIGNvbnN0IGFycmF5ID0gbmV3IFVpbnQ4QXJyYXkoYWwpO1xuICBmb3IgKGxldCBhaSA9IDAsIGhpID0gMDsgYWkgPCBhbDsgYWkrKywgaGkgKz0gMikge1xuICAgIGNvbnN0IG4xID0gYXNjaWlUb0Jhc2UxNihoZXguY2hhckNvZGVBdChoaSkpO1xuICAgIGNvbnN0IG4yID0gYXNjaWlUb0Jhc2UxNihoZXguY2hhckNvZGVBdChoaSArIDEpKTtcbiAgICBpZiAobjEgPT09IHVuZGVmaW5lZCB8fCBuMiA9PT0gdW5kZWZpbmVkKSB7XG4gICAgICBjb25zdCBjaGFyID0gaGV4W2hpXSArIGhleFtoaSArIDFdO1xuICAgICAgdGhyb3cgbmV3IEVycm9yKCdoZXggc3RyaW5nIGV4cGVjdGVkLCBnb3Qgbm9uLWhleCBjaGFyYWN0ZXIgXCInICsgY2hhciArICdcIiBhdCBpbmRleCAnICsgaGkpO1xuICAgIH1cbiAgICBhcnJheVthaV0gPSBuMSAqIDE2ICsgbjI7XG4gIH1cbiAgcmV0dXJuIGFycmF5O1xufVxuXG5leHBvcnQgZnVuY3Rpb24gaGV4VG9OdW1iZXIoaGV4OiBzdHJpbmcpOiBiaWdpbnQge1xuICBpZiAodHlwZW9mIGhleCAhPT0gJ3N0cmluZycpIHRocm93IG5ldyBFcnJvcignaGV4IHN0cmluZyBleHBlY3RlZCwgZ290ICcgKyB0eXBlb2YgaGV4KTtcbiAgLy8gQmlnIEVuZGlhblxuICByZXR1cm4gQmlnSW50KGhleCA9PT0gJycgPyAnMCcgOiBgMHgke2hleH1gKTtcbn1cblxuLy8gQkU6IEJpZyBFbmRpYW4sIExFOiBMaXR0bGUgRW5kaWFuXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb051bWJlckJFKGJ5dGVzOiBVaW50OEFycmF5KTogYmlnaW50IHtcbiAgcmV0dXJuIGhleFRvTnVtYmVyKGJ5dGVzVG9IZXgoYnl0ZXMpKTtcbn1cblxuZXhwb3J0IGZ1bmN0aW9uIG51bWJlclRvQnl0ZXNCRShuOiBudW1iZXIgfCBiaWdpbnQsIGxlbjogbnVtYmVyKTogVWludDhBcnJheSB7XG4gIHJldHVybiBoZXhUb0J5dGVzKG4udG9TdHJpbmcoMTYpLnBhZFN0YXJ0KGxlbiAqIDIsICcwJykpO1xufVxuXG4vLyBUaGVyZSBpcyBubyBzZXRJbW1lZGlhdGUgaW4gYnJvd3NlciBhbmQgc2V0VGltZW91dCBpcyBzbG93LlxuLy8gY2FsbCBvZiBhc3luYyBmbiB3aWxsIHJldHVybiBQcm9taXNlLCB3aGljaCB3aWxsIGJlIGZ1bGxmaWxlZCBvbmx5IG9uXG4vLyBuZXh0IHNjaGVkdWxlciBxdWV1ZSBwcm9jZXNzaW5nIHN0ZXAgYW5kIHRoaXMgaXMgZXhhY3RseSB3aGF0IHdlIG5lZWQuXG5leHBvcnQgY29uc3QgbmV4dFRpY2sgPSBhc3luYyAoKSA9PiB7fTtcblxuLy8gUmV0dXJucyBjb250cm9sIHRvIHRocmVhZCBlYWNoICd0aWNrJyBtcyB0byBhdm9pZCBibG9ja2luZ1xuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGFzeW5jTG9vcChpdGVyczogbnVtYmVyLCB0aWNrOiBudW1iZXIsIGNiOiAoaTogbnVtYmVyKSA9PiB2b2lkKSB7XG4gIGxldCB0cyA9IERhdGUubm93KCk7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgaXRlcnM7IGkrKykge1xuICAgIGNiKGkpO1xuICAgIC8vIERhdGUubm93KCkgaXMgbm90IG1vbm90b25pYywgc28gaW4gY2FzZSBpZiBjbG9jayBnb2VzIGJhY2t3YXJkcyB3ZSByZXR1cm4gcmV0dXJuIGNvbnRyb2wgdG9vXG4gICAgY29uc3QgZGlmZiA9IERhdGUubm93KCkgLSB0cztcbiAgICBpZiAoZGlmZiA+PSAwICYmIGRpZmYgPCB0aWNrKSBjb250aW51ZTtcbiAgICBhd2FpdCBuZXh0VGljaygpO1xuICAgIHRzICs9IGRpZmY7XG4gIH1cbn1cblxuLy8gR2xvYmFsIHN5bWJvbHMgaW4gYm90aCBicm93c2VycyBhbmQgTm9kZS5qcyBzaW5jZSB2MTFcbi8vIFNlZSBodHRwczovL2dpdGh1Yi5jb20vbWljcm9zb2Z0L1R5cGVTY3JpcHQvaXNzdWVzLzMxNTM1XG5kZWNsYXJlIGNvbnN0IFRleHRFbmNvZGVyOiBhbnk7XG5kZWNsYXJlIGNvbnN0IFRleHREZWNvZGVyOiBhbnk7XG5cbi8qKlxuICogQGV4YW1wbGUgdXRmOFRvQnl0ZXMoJ2FiYycpIC8vIG5ldyBVaW50OEFycmF5KFs5NywgOTgsIDk5XSlcbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIHV0ZjhUb0J5dGVzKHN0cjogc3RyaW5nKTogVWludDhBcnJheSB7XG4gIGlmICh0eXBlb2Ygc3RyICE9PSAnc3RyaW5nJykgdGhyb3cgbmV3IEVycm9yKGBzdHJpbmcgZXhwZWN0ZWQsIGdvdCAke3R5cGVvZiBzdHJ9YCk7XG4gIHJldHVybiBuZXcgVWludDhBcnJheShuZXcgVGV4dEVuY29kZXIoKS5lbmNvZGUoc3RyKSk7IC8vIGh0dHBzOi8vYnVnemlsLmxhLzE2ODE4MDlcbn1cblxuLyoqXG4gKiBAZXhhbXBsZSBieXRlc1RvVXRmOChuZXcgVWludDhBcnJheShbOTcsIDk4LCA5OV0pKSAvLyAnYWJjJ1xuICovXG5leHBvcnQgZnVuY3Rpb24gYnl0ZXNUb1V0ZjgoYnl0ZXM6IFVpbnQ4QXJyYXkpOiBzdHJpbmcge1xuICByZXR1cm4gbmV3IFRleHREZWNvZGVyKCkuZGVjb2RlKGJ5dGVzKTtcbn1cblxuZXhwb3J0IHR5cGUgSW5wdXQgPSBVaW50OEFycmF5IHwgc3RyaW5nO1xuLyoqXG4gKiBOb3JtYWxpemVzIChub24taGV4KSBzdHJpbmcgb3IgVWludDhBcnJheSB0byBVaW50OEFycmF5LlxuICogV2FybmluZzogd2hlbiBVaW50OEFycmF5IGlzIHBhc3NlZCwgaXQgd291bGQgTk9UIGdldCBjb3BpZWQuXG4gKiBLZWVwIGluIG1pbmQgZm9yIGZ1dHVyZSBtdXRhYmxlIG9wZXJhdGlvbnMuXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiB0b0J5dGVzKGRhdGE6IElucHV0KTogVWludDhBcnJheSB7XG4gIGlmICh0eXBlb2YgZGF0YSA9PT0gJ3N0cmluZycpIGRhdGEgPSB1dGY4VG9CeXRlcyhkYXRhKTtcbiAgZWxzZSBpZiAoaXNCeXRlcyhkYXRhKSkgZGF0YSA9IGRhdGEuc2xpY2UoKTtcbiAgZWxzZSB0aHJvdyBuZXcgRXJyb3IoYFVpbnQ4QXJyYXkgZXhwZWN0ZWQsIGdvdCAke3R5cGVvZiBkYXRhfWApO1xuICByZXR1cm4gZGF0YTtcbn1cblxuLyoqXG4gKiBDb3BpZXMgc2V2ZXJhbCBVaW50OEFycmF5cyBpbnRvIG9uZS5cbiAqL1xuZXhwb3J0IGZ1bmN0aW9uIGNvbmNhdEJ5dGVzKC4uLmFycmF5czogVWludDhBcnJheVtdKTogVWludDhBcnJheSB7XG4gIGxldCBzdW0gPSAwO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGFycmF5cy5sZW5ndGg7IGkrKykge1xuICAgIGNvbnN0IGEgPSBhcnJheXNbaV07XG4gICAgYWJ5dGVzKGEpO1xuICAgIHN1bSArPSBhLmxlbmd0aDtcbiAgfVxuICBjb25zdCByZXMgPSBuZXcgVWludDhBcnJheShzdW0pO1xuICBmb3IgKGxldCBpID0gMCwgcGFkID0gMDsgaSA8IGFycmF5cy5sZW5ndGg7IGkrKykge1xuICAgIGNvbnN0IGEgPSBhcnJheXNbaV07XG4gICAgcmVzLnNldChhLCBwYWQpO1xuICAgIHBhZCArPSBhLmxlbmd0aDtcbiAgfVxuICByZXR1cm4gcmVzO1xufVxuXG50eXBlIEVtcHR5T2JqID0ge307XG5leHBvcnQgZnVuY3Rpb24gY2hlY2tPcHRzPFQxIGV4dGVuZHMgRW1wdHlPYmosIFQyIGV4dGVuZHMgRW1wdHlPYmo+KFxuICBkZWZhdWx0czogVDEsXG4gIG9wdHM6IFQyXG4pOiBUMSAmIFQyIHtcbiAgaWYgKG9wdHMgPT0gbnVsbCB8fCB0eXBlb2Ygb3B0cyAhPT0gJ29iamVjdCcpIHRocm93IG5ldyBFcnJvcignb3B0aW9ucyBtdXN0IGJlIGRlZmluZWQnKTtcbiAgY29uc3QgbWVyZ2VkID0gT2JqZWN0LmFzc2lnbihkZWZhdWx0cywgb3B0cyk7XG4gIHJldHVybiBtZXJnZWQgYXMgVDEgJiBUMjtcbn1cblxuLy8gQ29tcGFyZXMgMiB1OGEtcyBpbiBraW5kYSBjb25zdGFudCB0aW1lXG5leHBvcnQgZnVuY3Rpb24gZXF1YWxCeXRlcyhhOiBVaW50OEFycmF5LCBiOiBVaW50OEFycmF5KSB7XG4gIGlmIChhLmxlbmd0aCAhPT0gYi5sZW5ndGgpIHJldHVybiBmYWxzZTtcbiAgbGV0IGRpZmYgPSAwO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGEubGVuZ3RoOyBpKyspIGRpZmYgfD0gYVtpXSBeIGJbaV07XG4gIHJldHVybiBkaWZmID09PSAwO1xufVxuXG4vLyBGb3IgcnVudGltZSBjaGVjayBpZiBjbGFzcyBpbXBsZW1lbnRzIGludGVyZmFjZVxuZXhwb3J0IGFic3RyYWN0IGNsYXNzIEhhc2g8VCBleHRlbmRzIEhhc2g8VD4+IHtcbiAgYWJzdHJhY3QgYmxvY2tMZW46IG51bWJlcjsgLy8gQnl0ZXMgcGVyIGJsb2NrXG4gIGFic3RyYWN0IG91dHB1dExlbjogbnVtYmVyOyAvLyBCeXRlcyBpbiBvdXRwdXRcbiAgYWJzdHJhY3QgdXBkYXRlKGJ1ZjogSW5wdXQpOiB0aGlzO1xuICAvLyBXcml0ZXMgZGlnZXN0IGludG8gYnVmXG4gIGFic3RyYWN0IGRpZ2VzdEludG8oYnVmOiBVaW50OEFycmF5KTogdm9pZDtcbiAgYWJzdHJhY3QgZGlnZXN0KCk6IFVpbnQ4QXJyYXk7XG4gIC8qKlxuICAgKiBSZXNldHMgaW50ZXJuYWwgc3RhdGUuIE1ha2VzIEhhc2ggaW5zdGFuY2UgdW51c2FibGUuXG4gICAqIFJlc2V0IGlzIGltcG9zc2libGUgZm9yIGtleWVkIGhhc2hlcyBpZiBrZXkgaXMgY29uc3VtZWQgaW50byBzdGF0ZS4gSWYgZGlnZXN0IGlzIG5vdCBjb25zdW1lZFxuICAgKiBieSB1c2VyLCB0aGV5IHdpbGwgbmVlZCB0byBtYW51YWxseSBjYWxsIGBkZXN0cm95KClgIHdoZW4gemVyb2luZyBpcyBuZWNlc3NhcnkuXG4gICAqL1xuICBhYnN0cmFjdCBkZXN0cm95KCk6IHZvaWQ7XG59XG5cbi8vIFRoaXMgd2lsbCBhbGxvdyB0byByZS11c2Ugd2l0aCBjb21wb3NhYmxlIHRoaW5ncyBsaWtlIHBhY2tlZCAmIGJhc2UgZW5jb2RlcnNcbi8vIEFsc28sIHdlIHByb2JhYmx5IGNhbiBtYWtlIHRhZ3MgY29tcG9zYWJsZVxuZXhwb3J0IHR5cGUgQ2lwaGVyID0ge1xuICBlbmNyeXB0KHBsYWludGV4dDogVWludDhBcnJheSk6IFVpbnQ4QXJyYXk7XG4gIGRlY3J5cHQoY2lwaGVydGV4dDogVWludDhBcnJheSk6IFVpbnQ4QXJyYXk7XG59O1xuXG5leHBvcnQgdHlwZSBBc3luY0NpcGhlciA9IHtcbiAgZW5jcnlwdChwbGFpbnRleHQ6IFVpbnQ4QXJyYXkpOiBQcm9taXNlPFVpbnQ4QXJyYXk+O1xuICBkZWNyeXB0KGNpcGhlcnRleHQ6IFVpbnQ4QXJyYXkpOiBQcm9taXNlPFVpbnQ4QXJyYXk+O1xufTtcblxuZXhwb3J0IHR5cGUgQ2lwaGVyV2l0aE91dHB1dCA9IENpcGhlciAmIHtcbiAgZW5jcnlwdChwbGFpbnRleHQ6IFVpbnQ4QXJyYXksIG91dHB1dD86IFVpbnQ4QXJyYXkpOiBVaW50OEFycmF5O1xuICBkZWNyeXB0KGNpcGhlcnRleHQ6IFVpbnQ4QXJyYXksIG91dHB1dD86IFVpbnQ4QXJyYXkpOiBVaW50OEFycmF5O1xufTtcblxuLy8gUGFyYW1zIGlzIG91dHNpZGUgcmV0dXJuIHR5cGUsIHNvIGl0IGlzIGFjY2Vzc2libGUgYmVmb3JlIGNhbGxpbmcgY29uc3RydWN0b3Jcbi8vIElmIGZ1bmN0aW9uIHN1cHBvcnQgbXVsdGlwbGUgbm9uY2VMZW5ndGgncywgd2UgcmV0dXJuIGJlc3Qgb25lXG5leHBvcnQgdHlwZSBDaXBoZXJQYXJhbXMgPSB7IGJsb2NrU2l6ZTogbnVtYmVyOyBub25jZUxlbmd0aD86IG51bWJlcjsgdGFnTGVuZ3RoPzogbnVtYmVyIH07XG5leHBvcnQgdHlwZSBDaXBoZXJDb25zPFQgZXh0ZW5kcyBhbnlbXT4gPSAoa2V5OiBVaW50OEFycmF5LCAuLi5hcmdzOiBUKSA9PiBDaXBoZXI7XG4vKipcbiAqIEBfX05PX1NJREVfRUZGRUNUU19fXG4gKi9cbmV4cG9ydCBjb25zdCB3cmFwQ2lwaGVyID0gPEMgZXh0ZW5kcyBDaXBoZXJDb25zPGFueT4sIFAgZXh0ZW5kcyBDaXBoZXJQYXJhbXM+KFxuICBwYXJhbXM6IFAsXG4gIGM6IENcbik6IEMgJiBQID0+IHtcbiAgT2JqZWN0LmFzc2lnbihjLCBwYXJhbXMpO1xuICByZXR1cm4gYyBhcyBDICYgUDtcbn07XG5cbmV4cG9ydCB0eXBlIFhvclN0cmVhbSA9IChcbiAga2V5OiBVaW50OEFycmF5LFxuICBub25jZTogVWludDhBcnJheSxcbiAgZGF0YTogVWludDhBcnJheSxcbiAgb3V0cHV0PzogVWludDhBcnJheSxcbiAgY291bnRlcj86IG51bWJlclxuKSA9PiBVaW50OEFycmF5O1xuXG4vLyBQb2x5ZmlsbCBmb3IgU2FmYXJpIDE0XG5leHBvcnQgZnVuY3Rpb24gc2V0QmlnVWludDY0KFxuICB2aWV3OiBEYXRhVmlldyxcbiAgYnl0ZU9mZnNldDogbnVtYmVyLFxuICB2YWx1ZTogYmlnaW50LFxuICBpc0xFOiBib29sZWFuXG4pOiB2b2lkIHtcbiAgaWYgKHR5cGVvZiB2aWV3LnNldEJpZ1VpbnQ2NCA9PT0gJ2Z1bmN0aW9uJykgcmV0dXJuIHZpZXcuc2V0QmlnVWludDY0KGJ5dGVPZmZzZXQsIHZhbHVlLCBpc0xFKTtcbiAgY29uc3QgXzMybiA9IEJpZ0ludCgzMik7XG4gIGNvbnN0IF91MzJfbWF4ID0gQmlnSW50KDB4ZmZmZmZmZmYpO1xuICBjb25zdCB3aCA9IE51bWJlcigodmFsdWUgPj4gXzMybikgJiBfdTMyX21heCk7XG4gIGNvbnN0IHdsID0gTnVtYmVyKHZhbHVlICYgX3UzMl9tYXgpO1xuICBjb25zdCBoID0gaXNMRSA/IDQgOiAwO1xuICBjb25zdCBsID0gaXNMRSA/IDAgOiA0O1xuICB2aWV3LnNldFVpbnQzMihieXRlT2Zmc2V0ICsgaCwgd2gsIGlzTEUpO1xuICB2aWV3LnNldFVpbnQzMihieXRlT2Zmc2V0ICsgbCwgd2wsIGlzTEUpO1xufVxuXG5leHBvcnQgZnVuY3Rpb24gdTY0TGVuZ3RocyhjaXBoZXJ0ZXh0OiBVaW50OEFycmF5LCBBQUQ/OiBVaW50OEFycmF5KSB7XG4gIGNvbnN0IG51bSA9IG5ldyBVaW50OEFycmF5KDE2KTtcbiAgY29uc3QgdmlldyA9IGNyZWF0ZVZpZXcobnVtKTtcbiAgc2V0QmlnVWludDY0KHZpZXcsIDAsIEJpZ0ludChBQUQgPyBBQUQubGVuZ3RoIDogMCksIHRydWUpO1xuICBzZXRCaWdVaW50NjQodmlldywgOCwgQmlnSW50KGNpcGhlcnRleHQubGVuZ3RoKSwgdHJ1ZSk7XG4gIHJldHVybiBudW07XG59XG4iLCAiaW1wb3J0IHsgY3JlYXRlVmlldywgdG9CeXRlcywgSW5wdXQsIEhhc2gsIHUzMiB9IGZyb20gJy4vdXRpbHMuanMnO1xuaW1wb3J0IHsgYnl0ZXMgYXMgYWJ5dGVzLCBleGlzdHMgYXMgYWV4aXN0cywgb3V0cHV0IGFzIGFvdXRwdXQgfSBmcm9tICcuL19hc3NlcnQuanMnO1xuXG4vLyBHSGFzaCBmcm9tIEFFUy1HQ00gYW5kIGl0cyBsaXR0bGUtZW5kaWFuIFwibWlycm9yIGltYWdlXCIgUG9seXZhbCBmcm9tIEFFUy1TSVYuXG4vLyBJbXBsZW1lbnRlZCBpbiB0ZXJtcyBvZiBHSGFzaCB3aXRoIGNvbnZlcnNpb24gZnVuY3Rpb24gZm9yIGtleXNcbi8vIEdDTSBHSEFTSCBmcm9tIE5JU1QgU1A4MDAtMzhkLCBTSVYgZnJvbSBSRkMgODQ1Mi5cbi8vIGh0dHBzOi8vbnZscHVicy5uaXN0Lmdvdi9uaXN0cHVicy9MZWdhY3kvU1AvbmlzdHNwZWNpYWxwdWJsaWNhdGlvbjgwMC0zOGQucGRmXG5cbi8vIEdIQVNIICAgbW9kdWxvOiB4XjEyOCArIHheNyAgICsgeF4yICAgKyB4ICAgICArIDFcbi8vIFBPTFlWQUwgbW9kdWxvOiB4XjEyOCArIHheMTI3ICsgeF4xMjYgKyB4XjEyMSArIDFcblxuY29uc3QgQkxPQ0tfU0laRSA9IDE2O1xuLy8gVE9ETzogcmV3cml0ZVxuLy8gdGVtcG9yYXJ5IHBhZGRpbmcgYnVmZmVyXG5jb25zdCBaRVJPUzE2ID0gLyogQF9fUFVSRV9fICovIG5ldyBVaW50OEFycmF5KDE2KTtcbmNvbnN0IFpFUk9TMzIgPSB1MzIoWkVST1MxNik7XG5jb25zdCBQT0xZID0gMHhlMTsgLy8gdiA9IDIqdiAlIFBPTFlcblxuLy8gdiA9IDIqdiAlIFBPTFlcbi8vIE5PVEU6IGJlY2F1c2UgeCArIHggPSAwIChhZGQvc3ViIGlzIHNhbWUpLCBtdWwyKHgpICE9IHgreFxuLy8gV2UgY2FuIG11bHRpcGx5IGFueSBudW1iZXIgdXNpbmcgbW9udGdvbWVyeSBsYWRkZXIgYW5kIHRoaXMgZnVuY3Rpb24gKHdvcmtzIGFzIGRvdWJsZSwgYWRkIGlzIHNpbXBsZSB4b3IpXG5jb25zdCBtdWwyID0gKHMwOiBudW1iZXIsIHMxOiBudW1iZXIsIHMyOiBudW1iZXIsIHMzOiBudW1iZXIpID0+IHtcbiAgY29uc3QgaGlCaXQgPSBzMyAmIDE7XG4gIHJldHVybiB7XG4gICAgczM6IChzMiA8PCAzMSkgfCAoczMgPj4+IDEpLFxuICAgIHMyOiAoczEgPDwgMzEpIHwgKHMyID4+PiAxKSxcbiAgICBzMTogKHMwIDw8IDMxKSB8IChzMSA+Pj4gMSksXG4gICAgczA6IChzMCA+Pj4gMSkgXiAoKFBPTFkgPDwgMjQpICYgLShoaUJpdCAmIDEpKSwgLy8gcmVkdWNlICUgcG9seVxuICB9O1xufTtcblxuY29uc3Qgc3dhcExFID0gKG46IG51bWJlcikgPT5cbiAgKCgobiA+Pj4gMCkgJiAweGZmKSA8PCAyNCkgfFxuICAoKChuID4+PiA4KSAmIDB4ZmYpIDw8IDE2KSB8XG4gICgoKG4gPj4+IDE2KSAmIDB4ZmYpIDw8IDgpIHxcbiAgKChuID4+PiAyNCkgJiAweGZmKSB8XG4gIDA7XG5cbi8qKlxuICogYG11bFhfUE9MWVZBTChCeXRlUmV2ZXJzZShIKSlgIGZyb20gc3BlY1xuICogQHBhcmFtIGsgbXV0YXRlZCBpbiBwbGFjZVxuICovXG5leHBvcnQgZnVuY3Rpb24gX3RvR0hBU0hLZXkoazogVWludDhBcnJheSk6IFVpbnQ4QXJyYXkge1xuICBrLnJldmVyc2UoKTtcbiAgY29uc3QgaGlCaXQgPSBrWzE1XSAmIDE7XG4gIC8vIGsgPj49IDFcbiAgbGV0IGNhcnJ5ID0gMDtcbiAgZm9yIChsZXQgaSA9IDA7IGkgPCBrLmxlbmd0aDsgaSsrKSB7XG4gICAgY29uc3QgdCA9IGtbaV07XG4gICAga1tpXSA9ICh0ID4+PiAxKSB8IGNhcnJ5O1xuICAgIGNhcnJ5ID0gKHQgJiAxKSA8PCA3O1xuICB9XG4gIGtbMF0gXj0gLWhpQml0ICYgMHhlMTsgLy8gaWYgKGhpQml0KSBuIF49IDB4ZTEwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDA7XG4gIHJldHVybiBrO1xufVxuXG50eXBlIFZhbHVlID0geyBzMDogbnVtYmVyOyBzMTogbnVtYmVyOyBzMjogbnVtYmVyOyBzMzogbnVtYmVyIH07XG5cbmNvbnN0IGVzdGltYXRlV2luZG93ID0gKGJ5dGVzOiBudW1iZXIpID0+IHtcbiAgaWYgKGJ5dGVzID4gNjQgKiAxMDI0KSByZXR1cm4gODtcbiAgaWYgKGJ5dGVzID4gMTAyNCkgcmV0dXJuIDQ7XG4gIHJldHVybiAyO1xufTtcblxuY2xhc3MgR0hBU0ggaW1wbGVtZW50cyBIYXNoPEdIQVNIPiB7XG4gIHJlYWRvbmx5IGJsb2NrTGVuID0gQkxPQ0tfU0laRTtcbiAgcmVhZG9ubHkgb3V0cHV0TGVuID0gQkxPQ0tfU0laRTtcbiAgcHJvdGVjdGVkIHMwID0gMDtcbiAgcHJvdGVjdGVkIHMxID0gMDtcbiAgcHJvdGVjdGVkIHMyID0gMDtcbiAgcHJvdGVjdGVkIHMzID0gMDtcbiAgcHJvdGVjdGVkIGZpbmlzaGVkID0gZmFsc2U7XG4gIHByb3RlY3RlZCB0OiBWYWx1ZVtdO1xuICBwcml2YXRlIFc6IG51bWJlcjtcbiAgcHJpdmF0ZSB3aW5kb3dTaXplOiBudW1iZXI7XG4gIC8vIFdlIHNlbGVjdCBiaXRzIHBlciB3aW5kb3cgYWRhcHRpdmVseSBiYXNlZCBvbiBleHBlY3RlZExlbmd0aFxuICBjb25zdHJ1Y3RvcihrZXk6IElucHV0LCBleHBlY3RlZExlbmd0aD86IG51bWJlcikge1xuICAgIGtleSA9IHRvQnl0ZXMoa2V5KTtcbiAgICBhYnl0ZXMoa2V5LCAxNik7XG4gICAgY29uc3Qga1ZpZXcgPSBjcmVhdGVWaWV3KGtleSk7XG4gICAgbGV0IGswID0ga1ZpZXcuZ2V0VWludDMyKDAsIGZhbHNlKTtcbiAgICBsZXQgazEgPSBrVmlldy5nZXRVaW50MzIoNCwgZmFsc2UpO1xuICAgIGxldCBrMiA9IGtWaWV3LmdldFVpbnQzMig4LCBmYWxzZSk7XG4gICAgbGV0IGszID0ga1ZpZXcuZ2V0VWludDMyKDEyLCBmYWxzZSk7XG4gICAgLy8gZ2VuZXJhdGUgdGFibGUgb2YgZG91YmxlZCBrZXlzIChoYWxmIG9mIG1vbnRnb21lcnkgbGFkZGVyKVxuICAgIGNvbnN0IGRvdWJsZXM6IFZhbHVlW10gPSBbXTtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IDEyODsgaSsrKSB7XG4gICAgICBkb3VibGVzLnB1c2goeyBzMDogc3dhcExFKGswKSwgczE6IHN3YXBMRShrMSksIHMyOiBzd2FwTEUoazIpLCBzMzogc3dhcExFKGszKSB9KTtcbiAgICAgICh7IHMwOiBrMCwgczE6IGsxLCBzMjogazIsIHMzOiBrMyB9ID0gbXVsMihrMCwgazEsIGsyLCBrMykpO1xuICAgIH1cbiAgICBjb25zdCBXID0gZXN0aW1hdGVXaW5kb3coZXhwZWN0ZWRMZW5ndGggfHwgMTAyNCk7XG4gICAgaWYgKCFbMSwgMiwgNCwgOF0uaW5jbHVkZXMoVykpXG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYGdoYXNoOiB3cm9uZyB3aW5kb3cgc2l6ZT0ke1d9LCBzaG91bGQgYmUgMiwgNCBvciA4YCk7XG4gICAgdGhpcy5XID0gVztcbiAgICBjb25zdCBiaXRzID0gMTI4OyAvLyBhbHdheXMgMTI4IGJpdHM7XG4gICAgY29uc3Qgd2luZG93cyA9IGJpdHMgLyBXO1xuICAgIGNvbnN0IHdpbmRvd1NpemUgPSAodGhpcy53aW5kb3dTaXplID0gMiAqKiBXKTtcbiAgICBjb25zdCBpdGVtczogVmFsdWVbXSA9IFtdO1xuICAgIC8vIENyZWF0ZSBwcmVjb21wdXRlIHRhYmxlIGZvciB3aW5kb3cgb2YgVyBiaXRzXG4gICAgZm9yIChsZXQgdyA9IDA7IHcgPCB3aW5kb3dzOyB3KyspIHtcbiAgICAgIC8vIHRydXRoIHRhYmxlOiAwMCwgMDEsIDEwLCAxMVxuICAgICAgZm9yIChsZXQgYnl0ZSA9IDA7IGJ5dGUgPCB3aW5kb3dTaXplOyBieXRlKyspIHtcbiAgICAgICAgLy8gcHJldHRpZXItaWdub3JlXG4gICAgICAgIGxldCBzMCA9IDAsIHMxID0gMCwgczIgPSAwLCBzMyA9IDA7XG4gICAgICAgIGZvciAobGV0IGogPSAwOyBqIDwgVzsgaisrKSB7XG4gICAgICAgICAgY29uc3QgYml0ID0gKGJ5dGUgPj4+IChXIC0gaiAtIDEpKSAmIDE7XG4gICAgICAgICAgaWYgKCFiaXQpIGNvbnRpbnVlO1xuICAgICAgICAgIGNvbnN0IHsgczA6IGQwLCBzMTogZDEsIHMyOiBkMiwgczM6IGQzIH0gPSBkb3VibGVzW1cgKiB3ICsgal07XG4gICAgICAgICAgKHMwIF49IGQwKSwgKHMxIF49IGQxKSwgKHMyIF49IGQyKSwgKHMzIF49IGQzKTtcbiAgICAgICAgfVxuICAgICAgICBpdGVtcy5wdXNoKHsgczAsIHMxLCBzMiwgczMgfSk7XG4gICAgICB9XG4gICAgfVxuICAgIHRoaXMudCA9IGl0ZW1zO1xuICB9XG4gIHByb3RlY3RlZCBfdXBkYXRlQmxvY2soczA6IG51bWJlciwgczE6IG51bWJlciwgczI6IG51bWJlciwgczM6IG51bWJlcikge1xuICAgIChzMCBePSB0aGlzLnMwKSwgKHMxIF49IHRoaXMuczEpLCAoczIgXj0gdGhpcy5zMiksIChzMyBePSB0aGlzLnMzKTtcbiAgICBjb25zdCB7IFcsIHQsIHdpbmRvd1NpemUgfSA9IHRoaXM7XG4gICAgLy8gcHJldHRpZXItaWdub3JlXG4gICAgbGV0IG8wID0gMCwgbzEgPSAwLCBvMiA9IDAsIG8zID0gMDtcbiAgICBjb25zdCBtYXNrID0gKDEgPDwgVykgLSAxOyAvLyAyKipXIHdpbGwga2lsbCBwZXJmb3JtYW5jZS5cbiAgICBsZXQgdyA9IDA7XG4gICAgZm9yIChjb25zdCBudW0gb2YgW3MwLCBzMSwgczIsIHMzXSkge1xuICAgICAgZm9yIChsZXQgYnl0ZVBvcyA9IDA7IGJ5dGVQb3MgPCA0OyBieXRlUG9zKyspIHtcbiAgICAgICAgY29uc3QgYnl0ZSA9IChudW0gPj4+ICg4ICogYnl0ZVBvcykpICYgMHhmZjtcbiAgICAgICAgZm9yIChsZXQgYml0UG9zID0gOCAvIFcgLSAxOyBiaXRQb3MgPj0gMDsgYml0UG9zLS0pIHtcbiAgICAgICAgICBjb25zdCBiaXQgPSAoYnl0ZSA+Pj4gKFcgKiBiaXRQb3MpKSAmIG1hc2s7XG4gICAgICAgICAgY29uc3QgeyBzMDogZTAsIHMxOiBlMSwgczI6IGUyLCBzMzogZTMgfSA9IHRbdyAqIHdpbmRvd1NpemUgKyBiaXRdO1xuICAgICAgICAgIChvMCBePSBlMCksIChvMSBePSBlMSksIChvMiBePSBlMiksIChvMyBePSBlMyk7XG4gICAgICAgICAgdyArPSAxO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfVxuICAgIHRoaXMuczAgPSBvMDtcbiAgICB0aGlzLnMxID0gbzE7XG4gICAgdGhpcy5zMiA9IG8yO1xuICAgIHRoaXMuczMgPSBvMztcbiAgfVxuICB1cGRhdGUoZGF0YTogSW5wdXQpOiB0aGlzIHtcbiAgICBkYXRhID0gdG9CeXRlcyhkYXRhKTtcbiAgICBhZXhpc3RzKHRoaXMpO1xuICAgIGNvbnN0IGIzMiA9IHUzMihkYXRhKTtcbiAgICBjb25zdCBibG9ja3MgPSBNYXRoLmZsb29yKGRhdGEubGVuZ3RoIC8gQkxPQ0tfU0laRSk7XG4gICAgY29uc3QgbGVmdCA9IGRhdGEubGVuZ3RoICUgQkxPQ0tfU0laRTtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IGJsb2NrczsgaSsrKSB7XG4gICAgICB0aGlzLl91cGRhdGVCbG9jayhiMzJbaSAqIDQgKyAwXSwgYjMyW2kgKiA0ICsgMV0sIGIzMltpICogNCArIDJdLCBiMzJbaSAqIDQgKyAzXSk7XG4gICAgfVxuICAgIGlmIChsZWZ0KSB7XG4gICAgICBaRVJPUzE2LnNldChkYXRhLnN1YmFycmF5KGJsb2NrcyAqIEJMT0NLX1NJWkUpKTtcbiAgICAgIHRoaXMuX3VwZGF0ZUJsb2NrKFpFUk9TMzJbMF0sIFpFUk9TMzJbMV0sIFpFUk9TMzJbMl0sIFpFUk9TMzJbM10pO1xuICAgICAgWkVST1MzMi5maWxsKDApOyAvLyBjbGVhbiB0bXAgYnVmZmVyXG4gICAgfVxuICAgIHJldHVybiB0aGlzO1xuICB9XG4gIGRlc3Ryb3koKSB7XG4gICAgY29uc3QgeyB0IH0gPSB0aGlzO1xuICAgIC8vIGNsZWFuIHByZWNvbXB1dGUgdGFibGVcbiAgICBmb3IgKGNvbnN0IGVsbSBvZiB0KSB7XG4gICAgICAoZWxtLnMwID0gMCksIChlbG0uczEgPSAwKSwgKGVsbS5zMiA9IDApLCAoZWxtLnMzID0gMCk7XG4gICAgfVxuICB9XG4gIGRpZ2VzdEludG8ob3V0OiBVaW50OEFycmF5KSB7XG4gICAgYWV4aXN0cyh0aGlzKTtcbiAgICBhb3V0cHV0KG91dCwgdGhpcyk7XG4gICAgdGhpcy5maW5pc2hlZCA9IHRydWU7XG4gICAgY29uc3QgeyBzMCwgczEsIHMyLCBzMyB9ID0gdGhpcztcbiAgICBjb25zdCBvMzIgPSB1MzIob3V0KTtcbiAgICBvMzJbMF0gPSBzMDtcbiAgICBvMzJbMV0gPSBzMTtcbiAgICBvMzJbMl0gPSBzMjtcbiAgICBvMzJbM10gPSBzMztcbiAgICByZXR1cm4gb3V0O1xuICB9XG4gIGRpZ2VzdCgpOiBVaW50OEFycmF5IHtcbiAgICBjb25zdCByZXMgPSBuZXcgVWludDhBcnJheShCTE9DS19TSVpFKTtcbiAgICB0aGlzLmRpZ2VzdEludG8ocmVzKTtcbiAgICB0aGlzLmRlc3Ryb3koKTtcbiAgICByZXR1cm4gcmVzO1xuICB9XG59XG5cbmNsYXNzIFBvbHl2YWwgZXh0ZW5kcyBHSEFTSCB7XG4gIGNvbnN0cnVjdG9yKGtleTogSW5wdXQsIGV4cGVjdGVkTGVuZ3RoPzogbnVtYmVyKSB7XG4gICAga2V5ID0gdG9CeXRlcyhrZXkpO1xuICAgIGNvbnN0IGdoS2V5ID0gX3RvR0hBU0hLZXkoa2V5LnNsaWNlKCkpO1xuICAgIHN1cGVyKGdoS2V5LCBleHBlY3RlZExlbmd0aCk7XG4gICAgZ2hLZXkuZmlsbCgwKTtcbiAgfVxuICB1cGRhdGUoZGF0YTogSW5wdXQpOiB0aGlzIHtcbiAgICBkYXRhID0gdG9CeXRlcyhkYXRhKTtcbiAgICBhZXhpc3RzKHRoaXMpO1xuICAgIGNvbnN0IGIzMiA9IHUzMihkYXRhKTtcbiAgICBjb25zdCBsZWZ0ID0gZGF0YS5sZW5ndGggJSBCTE9DS19TSVpFO1xuICAgIGNvbnN0IGJsb2NrcyA9IE1hdGguZmxvb3IoZGF0YS5sZW5ndGggLyBCTE9DS19TSVpFKTtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IGJsb2NrczsgaSsrKSB7XG4gICAgICB0aGlzLl91cGRhdGVCbG9jayhcbiAgICAgICAgc3dhcExFKGIzMltpICogNCArIDNdKSxcbiAgICAgICAgc3dhcExFKGIzMltpICogNCArIDJdKSxcbiAgICAgICAgc3dhcExFKGIzMltpICogNCArIDFdKSxcbiAgICAgICAgc3dhcExFKGIzMltpICogNCArIDBdKVxuICAgICAgKTtcbiAgICB9XG4gICAgaWYgKGxlZnQpIHtcbiAgICAgIFpFUk9TMTYuc2V0KGRhdGEuc3ViYXJyYXkoYmxvY2tzICogQkxPQ0tfU0laRSkpO1xuICAgICAgdGhpcy5fdXBkYXRlQmxvY2soXG4gICAgICAgIHN3YXBMRShaRVJPUzMyWzNdKSxcbiAgICAgICAgc3dhcExFKFpFUk9TMzJbMl0pLFxuICAgICAgICBzd2FwTEUoWkVST1MzMlsxXSksXG4gICAgICAgIHN3YXBMRShaRVJPUzMyWzBdKVxuICAgICAgKTtcbiAgICAgIFpFUk9TMzIuZmlsbCgwKTsgLy8gY2xlYW4gdG1wIGJ1ZmZlclxuICAgIH1cbiAgICByZXR1cm4gdGhpcztcbiAgfVxuICBkaWdlc3RJbnRvKG91dDogVWludDhBcnJheSkge1xuICAgIGFleGlzdHModGhpcyk7XG4gICAgYW91dHB1dChvdXQsIHRoaXMpO1xuICAgIHRoaXMuZmluaXNoZWQgPSB0cnVlO1xuICAgIC8vIHRtcCB1Z2x5IGhhY2tcbiAgICBjb25zdCB7IHMwLCBzMSwgczIsIHMzIH0gPSB0aGlzO1xuICAgIGNvbnN0IG8zMiA9IHUzMihvdXQpO1xuICAgIG8zMlswXSA9IHMwO1xuICAgIG8zMlsxXSA9IHMxO1xuICAgIG8zMlsyXSA9IHMyO1xuICAgIG8zMlszXSA9IHMzO1xuICAgIHJldHVybiBvdXQucmV2ZXJzZSgpO1xuICB9XG59XG5cbmV4cG9ydCB0eXBlIENIYXNoID0gUmV0dXJuVHlwZTx0eXBlb2Ygd3JhcENvbnN0cnVjdG9yV2l0aEtleT47XG5mdW5jdGlvbiB3cmFwQ29uc3RydWN0b3JXaXRoS2V5PEggZXh0ZW5kcyBIYXNoPEg+PihcbiAgaGFzaENvbnM6IChrZXk6IElucHV0LCBleHBlY3RlZExlbmd0aD86IG51bWJlcikgPT4gSGFzaDxIPlxuKSB7XG4gIGNvbnN0IGhhc2hDID0gKG1zZzogSW5wdXQsIGtleTogSW5wdXQpOiBVaW50OEFycmF5ID0+XG4gICAgaGFzaENvbnMoa2V5LCBtc2cubGVuZ3RoKS51cGRhdGUodG9CeXRlcyhtc2cpKS5kaWdlc3QoKTtcbiAgY29uc3QgdG1wID0gaGFzaENvbnMobmV3IFVpbnQ4QXJyYXkoMTYpLCAwKTtcbiAgaGFzaEMub3V0cHV0TGVuID0gdG1wLm91dHB1dExlbjtcbiAgaGFzaEMuYmxvY2tMZW4gPSB0bXAuYmxvY2tMZW47XG4gIGhhc2hDLmNyZWF0ZSA9IChrZXk6IElucHV0LCBleHBlY3RlZExlbmd0aD86IG51bWJlcikgPT4gaGFzaENvbnMoa2V5LCBleHBlY3RlZExlbmd0aCk7XG4gIHJldHVybiBoYXNoQztcbn1cblxuZXhwb3J0IGNvbnN0IGdoYXNoID0gd3JhcENvbnN0cnVjdG9yV2l0aEtleShcbiAgKGtleSwgZXhwZWN0ZWRMZW5ndGgpID0+IG5ldyBHSEFTSChrZXksIGV4cGVjdGVkTGVuZ3RoKVxuKTtcbmV4cG9ydCBjb25zdCBwb2x5dmFsID0gd3JhcENvbnN0cnVjdG9yV2l0aEtleShcbiAgKGtleSwgZXhwZWN0ZWRMZW5ndGgpID0+IG5ldyBQb2x5dmFsKGtleSwgZXhwZWN0ZWRMZW5ndGgpXG4pO1xuIiwgIi8vIHByZXR0aWVyLWlnbm9yZVxuaW1wb3J0IHtcbiAgd3JhcENpcGhlciwgQ2lwaGVyLCBDaXBoZXJXaXRoT3V0cHV0LFxuICBjcmVhdGVWaWV3LCBzZXRCaWdVaW50NjQsIGVxdWFsQnl0ZXMsIHUzMiwgdTgsXG59IGZyb20gJy4vdXRpbHMuanMnO1xuaW1wb3J0IHsgZ2hhc2gsIHBvbHl2YWwgfSBmcm9tICcuL19wb2x5dmFsLmpzJztcbmltcG9ydCB7IGJ5dGVzIGFzIGFieXRlcyB9IGZyb20gJy4vX2Fzc2VydC5qcyc7XG5cbi8qXG5BRVMgKEFkdmFuY2VkIEVuY3J5cHRpb24gU3RhbmRhcmQpIGFrYSBSaWpuZGFlbCBibG9jayBjaXBoZXIuXG5cbkRhdGEgaXMgc3BsaXQgaW50byAxMjgtYml0IGJsb2Nrcy4gRW5jcnlwdGVkIGluIDEwLzEyLzE0IHJvdW5kcyAoMTI4LzE5Mi8yNTYgYml0cykuIEluIGV2ZXJ5IHJvdW5kOlxuMS4gKipTLWJveCoqLCB0YWJsZSBzdWJzdGl0dXRpb25cbjIuICoqU2hpZnQgcm93cyoqLCBjeWNsaWMgc2hpZnQgbGVmdCBvZiBhbGwgcm93cyBvZiBkYXRhIGFycmF5XG4zLiAqKk1peCBjb2x1bW5zKiosIG11bHRpcGx5aW5nIGV2ZXJ5IGNvbHVtbiBieSBmaXhlZCBwb2x5bm9taWFsXG40LiAqKkFkZCByb3VuZCBrZXkqKiwgcm91bmRfa2V5IHhvciBpLXRoIGNvbHVtbiBvZiBhcnJheVxuXG5SZXNvdXJjZXM6XG4tIEZJUFMtMTk3IGh0dHBzOi8vY3NyYy5uaXN0Lmdvdi9maWxlcy9wdWJzL2ZpcHMvMTk3L2ZpbmFsL2RvY3MvZmlwcy0xOTcucGRmXG4tIE9yaWdpbmFsIHByb3Bvc2FsOiBodHRwczovL2NzcmMubmlzdC5nb3YvY3NyYy9tZWRpYS9wcm9qZWN0cy9jcnlwdG9ncmFwaGljLXN0YW5kYXJkcy1hbmQtZ3VpZGVsaW5lcy9kb2N1bWVudHMvYWVzLWRldmVsb3BtZW50L3Jpam5kYWVsLWFtbWVuZGVkLnBkZlxuKi9cblxuY29uc3QgQkxPQ0tfU0laRSA9IDE2O1xuY29uc3QgQkxPQ0tfU0laRTMyID0gNDtcbmNvbnN0IEVNUFRZX0JMT0NLID0gbmV3IFVpbnQ4QXJyYXkoQkxPQ0tfU0laRSk7XG5jb25zdCBQT0xZID0gMHgxMWI7IC8vIDEgKyB4ICsgeCoqMyArIHgqKjQgKyB4Kio4XG5cbi8vIFRPRE86IHJlbW92ZSBtdWx0aXBsaWNhdGlvbiwgYmluYXJ5IG9wcyBvbmx5XG5mdW5jdGlvbiBtdWwyKG46IG51bWJlcikge1xuICByZXR1cm4gKG4gPDwgMSkgXiAoUE9MWSAmIC0obiA+PiA3KSk7XG59XG5cbmZ1bmN0aW9uIG11bChhOiBudW1iZXIsIGI6IG51bWJlcikge1xuICBsZXQgcmVzID0gMDtcbiAgZm9yICg7IGIgPiAwOyBiID4+PSAxKSB7XG4gICAgLy8gTW9udGdvbWVyeSBsYWRkZXJcbiAgICByZXMgXj0gYSAmIC0oYiAmIDEpOyAvLyBpZiAoYiYxKSByZXMgXj1hIChidXQgY29uc3QtdGltZSkuXG4gICAgYSA9IG11bDIoYSk7IC8vIGEgPSAyKmFcbiAgfVxuICByZXR1cm4gcmVzO1xufVxuXG4vLyBBRVMgUy1ib3ggaXMgZ2VuZXJhdGVkIHVzaW5nIGZpbml0ZSBmaWVsZCBpbnZlcnNpb24sXG4vLyBhbiBhZmZpbmUgdHJhbnNmb3JtLCBhbmQgeG9yIG9mIGEgY29uc3RhbnQgMHg2My5cbmNvbnN0IHNib3ggPSAvKiBAX19QVVJFX18gKi8gKCgpID0+IHtcbiAgbGV0IHQgPSBuZXcgVWludDhBcnJheSgyNTYpO1xuICBmb3IgKGxldCBpID0gMCwgeCA9IDE7IGkgPCAyNTY7IGkrKywgeCBePSBtdWwyKHgpKSB0W2ldID0geDtcbiAgY29uc3QgYm94ID0gbmV3IFVpbnQ4QXJyYXkoMjU2KTtcbiAgYm94WzBdID0gMHg2MzsgLy8gZmlyc3QgZWxtXG4gIGZvciAobGV0IGkgPSAwOyBpIDwgMjU1OyBpKyspIHtcbiAgICBsZXQgeCA9IHRbMjU1IC0gaV07XG4gICAgeCB8PSB4IDw8IDg7XG4gICAgYm94W3RbaV1dID0gKHggXiAoeCA+PiA0KSBeICh4ID4+IDUpIF4gKHggPj4gNikgXiAoeCA+PiA3KSBeIDB4NjMpICYgMHhmZjtcbiAgfVxuICByZXR1cm4gYm94O1xufSkoKTtcblxuLy8gSW52ZXJ0ZWQgUy1ib3hcbmNvbnN0IGludlNib3ggPSAvKiBAX19QVVJFX18gKi8gc2JveC5tYXAoKF8sIGopID0+IHNib3guaW5kZXhPZihqKSk7XG5cbi8vIFJvdGF0ZSB1MzIgYnkgOFxuY29uc3Qgcm90cjMyXzggPSAobjogbnVtYmVyKSA9PiAobiA8PCAyNCkgfCAobiA+Pj4gOCk7XG5jb25zdCByb3RsMzJfOCA9IChuOiBudW1iZXIpID0+IChuIDw8IDgpIHwgKG4gPj4+IDI0KTtcblxuLy8gVC10YWJsZSBpcyBvcHRpbWl6YXRpb24gc3VnZ2VzdGVkIGluIDUuMiBvZiBvcmlnaW5hbCBwcm9wb3NhbCAobWlzc2VkIGZyb20gRklQUy0xOTcpLiBDaGFuZ2VzOlxuLy8gLSBMRSBpbnN0ZWFkIG9mIEJFXG4vLyAtIGJpZ2dlciB0YWJsZXM6IFQwIGFuZCBUMSBhcmUgbWVyZ2VkIGludG8gVDAxIHRhYmxlIGFuZCBUMiAmIFQzIGludG8gVDIzO1xuLy8gICBzbyBpbmRleCBpcyB1MTYsIGluc3RlYWQgb2YgdTguIFRoaXMgc3BlZWRzIHVwIHRoaW5ncywgdW5leHBlY3RlZGx5XG5mdW5jdGlvbiBnZW5UdGFibGUoc2JveDogVWludDhBcnJheSwgZm46IChuOiBudW1iZXIpID0+IG51bWJlcikge1xuICBpZiAoc2JveC5sZW5ndGggIT09IDI1NikgdGhyb3cgbmV3IEVycm9yKCdXcm9uZyBzYm94IGxlbmd0aCcpO1xuICBjb25zdCBUMCA9IG5ldyBVaW50MzJBcnJheSgyNTYpLm1hcCgoXywgaikgPT4gZm4oc2JveFtqXSkpO1xuICBjb25zdCBUMSA9IFQwLm1hcChyb3RsMzJfOCk7XG4gIGNvbnN0IFQyID0gVDEubWFwKHJvdGwzMl84KTtcbiAgY29uc3QgVDMgPSBUMi5tYXAocm90bDMyXzgpO1xuICBjb25zdCBUMDEgPSBuZXcgVWludDMyQXJyYXkoMjU2ICogMjU2KTtcbiAgY29uc3QgVDIzID0gbmV3IFVpbnQzMkFycmF5KDI1NiAqIDI1Nik7XG4gIGNvbnN0IHNib3gyID0gbmV3IFVpbnQxNkFycmF5KDI1NiAqIDI1Nik7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgMjU2OyBpKyspIHtcbiAgICBmb3IgKGxldCBqID0gMDsgaiA8IDI1NjsgaisrKSB7XG4gICAgICBjb25zdCBpZHggPSBpICogMjU2ICsgajtcbiAgICAgIFQwMVtpZHhdID0gVDBbaV0gXiBUMVtqXTtcbiAgICAgIFQyM1tpZHhdID0gVDJbaV0gXiBUM1tqXTtcbiAgICAgIHNib3gyW2lkeF0gPSAoc2JveFtpXSA8PCA4KSB8IHNib3hbal07XG4gICAgfVxuICB9XG4gIHJldHVybiB7IHNib3gsIHNib3gyLCBUMCwgVDEsIFQyLCBUMywgVDAxLCBUMjMgfTtcbn1cblxuY29uc3QgdGFibGVFbmNvZGluZyA9IC8qIEBfX1BVUkVfXyAqLyBnZW5UdGFibGUoXG4gIHNib3gsXG4gIChzOiBudW1iZXIpID0+IChtdWwocywgMykgPDwgMjQpIHwgKHMgPDwgMTYpIHwgKHMgPDwgOCkgfCBtdWwocywgMilcbik7XG5jb25zdCB0YWJsZURlY29kaW5nID0gLyogQF9fUFVSRV9fICovIGdlblR0YWJsZShcbiAgaW52U2JveCxcbiAgKHMpID0+IChtdWwocywgMTEpIDw8IDI0KSB8IChtdWwocywgMTMpIDw8IDE2KSB8IChtdWwocywgOSkgPDwgOCkgfCBtdWwocywgMTQpXG4pO1xuXG5jb25zdCB4UG93ZXJzID0gLyogQF9fUFVSRV9fICovICgoKSA9PiB7XG4gIGNvbnN0IHAgPSBuZXcgVWludDhBcnJheSgxNik7XG4gIGZvciAobGV0IGkgPSAwLCB4ID0gMTsgaSA8IDE2OyBpKyssIHggPSBtdWwyKHgpKSBwW2ldID0geDtcbiAgcmV0dXJuIHA7XG59KSgpO1xuXG5leHBvcnQgZnVuY3Rpb24gZXhwYW5kS2V5TEUoa2V5OiBVaW50OEFycmF5KTogVWludDMyQXJyYXkge1xuICBhYnl0ZXMoa2V5KTtcbiAgY29uc3QgbGVuID0ga2V5Lmxlbmd0aDtcbiAgaWYgKCFbMTYsIDI0LCAzMl0uaW5jbHVkZXMobGVuKSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoYGFlczogd3Jvbmcga2V5IHNpemU6IHNob3VsZCBiZSAxNiwgMjQgb3IgMzIsIGdvdDogJHtsZW59YCk7XG4gIGNvbnN0IHsgc2JveDIgfSA9IHRhYmxlRW5jb2Rpbmc7XG4gIGNvbnN0IGszMiA9IHUzMihrZXkpO1xuICBjb25zdCBOayA9IGszMi5sZW5ndGg7XG4gIGNvbnN0IHN1YkJ5dGUgPSAobjogbnVtYmVyKSA9PiBhcHBseVNib3goc2JveDIsIG4sIG4sIG4sIG4pO1xuICBjb25zdCB4ayA9IG5ldyBVaW50MzJBcnJheShsZW4gKyAyOCk7IC8vIGV4cGFuZGVkIGtleVxuICB4ay5zZXQoazMyKTtcbiAgLy8gNC4zLjEgS2V5IGV4cGFuc2lvblxuICBmb3IgKGxldCBpID0gTms7IGkgPCB4ay5sZW5ndGg7IGkrKykge1xuICAgIGxldCB0ID0geGtbaSAtIDFdO1xuICAgIGlmIChpICUgTmsgPT09IDApIHQgPSBzdWJCeXRlKHJvdHIzMl84KHQpKSBeIHhQb3dlcnNbaSAvIE5rIC0gMV07XG4gICAgZWxzZSBpZiAoTmsgPiA2ICYmIGkgJSBOayA9PT0gNCkgdCA9IHN1YkJ5dGUodCk7XG4gICAgeGtbaV0gPSB4a1tpIC0gTmtdIF4gdDtcbiAgfVxuICByZXR1cm4geGs7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBleHBhbmRLZXlEZWNMRShrZXk6IFVpbnQ4QXJyYXkpOiBVaW50MzJBcnJheSB7XG4gIGNvbnN0IGVuY0tleSA9IGV4cGFuZEtleUxFKGtleSk7XG4gIGNvbnN0IHhrID0gZW5jS2V5LnNsaWNlKCk7XG4gIGNvbnN0IE5rID0gZW5jS2V5Lmxlbmd0aDtcbiAgY29uc3QgeyBzYm94MiB9ID0gdGFibGVFbmNvZGluZztcbiAgY29uc3QgeyBUMCwgVDEsIFQyLCBUMyB9ID0gdGFibGVEZWNvZGluZztcbiAgLy8gSW52ZXJzZSBrZXkgYnkgY2h1bmtzIG9mIDQgKHJvdW5kcylcbiAgZm9yIChsZXQgaSA9IDA7IGkgPCBOazsgaSArPSA0KSB7XG4gICAgZm9yIChsZXQgaiA9IDA7IGogPCA0OyBqKyspIHhrW2kgKyBqXSA9IGVuY0tleVtOayAtIGkgLSA0ICsgal07XG4gIH1cbiAgZW5jS2V5LmZpbGwoMCk7XG4gIC8vIGFwcGx5IEludk1peENvbHVtbiBleGNlcHQgZmlyc3QgJiBsYXN0IHJvdW5kXG4gIGZvciAobGV0IGkgPSA0OyBpIDwgTmsgLSA0OyBpKyspIHtcbiAgICBjb25zdCB4ID0geGtbaV07XG4gICAgY29uc3QgdyA9IGFwcGx5U2JveChzYm94MiwgeCwgeCwgeCwgeCk7XG4gICAgeGtbaV0gPSBUMFt3ICYgMHhmZl0gXiBUMVsodyA+Pj4gOCkgJiAweGZmXSBeIFQyWyh3ID4+PiAxNikgJiAweGZmXSBeIFQzW3cgPj4+IDI0XTtcbiAgfVxuICByZXR1cm4geGs7XG59XG5cbi8vIEFwcGx5IHRhYmxlc1xuZnVuY3Rpb24gYXBwbHkwMTIzKFxuICBUMDE6IFVpbnQzMkFycmF5LFxuICBUMjM6IFVpbnQzMkFycmF5LFxuICBzMDogbnVtYmVyLFxuICBzMTogbnVtYmVyLFxuICBzMjogbnVtYmVyLFxuICBzMzogbnVtYmVyXG4pIHtcbiAgcmV0dXJuIChcbiAgICBUMDFbKChzMCA8PCA4KSAmIDB4ZmYwMCkgfCAoKHMxID4+PiA4KSAmIDB4ZmYpXSBeXG4gICAgVDIzWygoczIgPj4+IDgpICYgMHhmZjAwKSB8ICgoczMgPj4+IDI0KSAmIDB4ZmYpXVxuICApO1xufVxuXG5mdW5jdGlvbiBhcHBseVNib3goc2JveDI6IFVpbnQxNkFycmF5LCBzMDogbnVtYmVyLCBzMTogbnVtYmVyLCBzMjogbnVtYmVyLCBzMzogbnVtYmVyKSB7XG4gIHJldHVybiAoXG4gICAgc2JveDJbKHMwICYgMHhmZikgfCAoczEgJiAweGZmMDApXSB8XG4gICAgKHNib3gyWygoczIgPj4+IDE2KSAmIDB4ZmYpIHwgKChzMyA+Pj4gMTYpICYgMHhmZjAwKV0gPDwgMTYpXG4gICk7XG59XG5cbmZ1bmN0aW9uIGVuY3J5cHQoeGs6IFVpbnQzMkFycmF5LCBzMDogbnVtYmVyLCBzMTogbnVtYmVyLCBzMjogbnVtYmVyLCBzMzogbnVtYmVyKSB7XG4gIGNvbnN0IHsgc2JveDIsIFQwMSwgVDIzIH0gPSB0YWJsZUVuY29kaW5nO1xuICBsZXQgayA9IDA7XG4gIChzMCBePSB4a1trKytdKSwgKHMxIF49IHhrW2srK10pLCAoczIgXj0geGtbaysrXSksIChzMyBePSB4a1trKytdKTtcbiAgY29uc3Qgcm91bmRzID0geGsubGVuZ3RoIC8gNCAtIDI7XG4gIGZvciAobGV0IGkgPSAwOyBpIDwgcm91bmRzOyBpKyspIHtcbiAgICBjb25zdCB0MCA9IHhrW2srK10gXiBhcHBseTAxMjMoVDAxLCBUMjMsIHMwLCBzMSwgczIsIHMzKTtcbiAgICBjb25zdCB0MSA9IHhrW2srK10gXiBhcHBseTAxMjMoVDAxLCBUMjMsIHMxLCBzMiwgczMsIHMwKTtcbiAgICBjb25zdCB0MiA9IHhrW2srK10gXiBhcHBseTAxMjMoVDAxLCBUMjMsIHMyLCBzMywgczAsIHMxKTtcbiAgICBjb25zdCB0MyA9IHhrW2srK10gXiBhcHBseTAxMjMoVDAxLCBUMjMsIHMzLCBzMCwgczEsIHMyKTtcbiAgICAoczAgPSB0MCksIChzMSA9IHQxKSwgKHMyID0gdDIpLCAoczMgPSB0Myk7XG4gIH1cbiAgLy8gbGFzdCByb3VuZCAod2l0aG91dCBtaXhjb2x1bW5zLCBzbyB1c2luZyBTQk9YMiB0YWJsZSlcbiAgY29uc3QgdDAgPSB4a1trKytdIF4gYXBwbHlTYm94KHNib3gyLCBzMCwgczEsIHMyLCBzMyk7XG4gIGNvbnN0IHQxID0geGtbaysrXSBeIGFwcGx5U2JveChzYm94MiwgczEsIHMyLCBzMywgczApO1xuICBjb25zdCB0MiA9IHhrW2srK10gXiBhcHBseVNib3goc2JveDIsIHMyLCBzMywgczAsIHMxKTtcbiAgY29uc3QgdDMgPSB4a1trKytdIF4gYXBwbHlTYm94KHNib3gyLCBzMywgczAsIHMxLCBzMik7XG4gIHJldHVybiB7IHMwOiB0MCwgczE6IHQxLCBzMjogdDIsIHMzOiB0MyB9O1xufVxuXG5mdW5jdGlvbiBkZWNyeXB0KHhrOiBVaW50MzJBcnJheSwgczA6IG51bWJlciwgczE6IG51bWJlciwgczI6IG51bWJlciwgczM6IG51bWJlcikge1xuICBjb25zdCB7IHNib3gyLCBUMDEsIFQyMyB9ID0gdGFibGVEZWNvZGluZztcbiAgbGV0IGsgPSAwO1xuICAoczAgXj0geGtbaysrXSksIChzMSBePSB4a1trKytdKSwgKHMyIF49IHhrW2srK10pLCAoczMgXj0geGtbaysrXSk7XG4gIGNvbnN0IHJvdW5kcyA9IHhrLmxlbmd0aCAvIDQgLSAyO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IHJvdW5kczsgaSsrKSB7XG4gICAgY29uc3QgdDAgPSB4a1trKytdIF4gYXBwbHkwMTIzKFQwMSwgVDIzLCBzMCwgczMsIHMyLCBzMSk7XG4gICAgY29uc3QgdDEgPSB4a1trKytdIF4gYXBwbHkwMTIzKFQwMSwgVDIzLCBzMSwgczAsIHMzLCBzMik7XG4gICAgY29uc3QgdDIgPSB4a1trKytdIF4gYXBwbHkwMTIzKFQwMSwgVDIzLCBzMiwgczEsIHMwLCBzMyk7XG4gICAgY29uc3QgdDMgPSB4a1trKytdIF4gYXBwbHkwMTIzKFQwMSwgVDIzLCBzMywgczIsIHMxLCBzMCk7XG4gICAgKHMwID0gdDApLCAoczEgPSB0MSksIChzMiA9IHQyKSwgKHMzID0gdDMpO1xuICB9XG4gIC8vIExhc3Qgcm91bmRcbiAgY29uc3QgdDAgPSB4a1trKytdIF4gYXBwbHlTYm94KHNib3gyLCBzMCwgczMsIHMyLCBzMSk7XG4gIGNvbnN0IHQxID0geGtbaysrXSBeIGFwcGx5U2JveChzYm94MiwgczEsIHMwLCBzMywgczIpO1xuICBjb25zdCB0MiA9IHhrW2srK10gXiBhcHBseVNib3goc2JveDIsIHMyLCBzMSwgczAsIHMzKTtcbiAgY29uc3QgdDMgPSB4a1trKytdIF4gYXBwbHlTYm94KHNib3gyLCBzMywgczIsIHMxLCBzMCk7XG4gIHJldHVybiB7IHMwOiB0MCwgczE6IHQxLCBzMjogdDIsIHMzOiB0MyB9O1xufVxuXG5mdW5jdGlvbiBnZXREc3QobGVuOiBudW1iZXIsIGRzdD86IFVpbnQ4QXJyYXkpIHtcbiAgaWYgKCFkc3QpIHJldHVybiBuZXcgVWludDhBcnJheShsZW4pO1xuICBhYnl0ZXMoZHN0KTtcbiAgaWYgKGRzdC5sZW5ndGggPCBsZW4pXG4gICAgdGhyb3cgbmV3IEVycm9yKGBhZXM6IHdyb25nIGRlc3RpbmF0aW9uIGxlbmd0aCwgZXhwZWN0ZWQgYXQgbGVhc3QgJHtsZW59LCBnb3Q6ICR7ZHN0Lmxlbmd0aH1gKTtcbiAgcmV0dXJuIGRzdDtcbn1cblxuLy8gVE9ETzogaW52ZXN0aWdhdGUgbWVyZ2luZyB3aXRoIGN0cjMyXG5mdW5jdGlvbiBjdHJDb3VudGVyKHhrOiBVaW50MzJBcnJheSwgbm9uY2U6IFVpbnQ4QXJyYXksIHNyYzogVWludDhBcnJheSwgZHN0PzogVWludDhBcnJheSkge1xuICBhYnl0ZXMobm9uY2UsIEJMT0NLX1NJWkUpO1xuICBhYnl0ZXMoc3JjKTtcbiAgY29uc3Qgc3JjTGVuID0gc3JjLmxlbmd0aDtcbiAgZHN0ID0gZ2V0RHN0KHNyY0xlbiwgZHN0KTtcbiAgY29uc3QgY3RyID0gbm9uY2U7XG4gIGNvbnN0IGMzMiA9IHUzMihjdHIpO1xuICAvLyBGaWxsIGJsb2NrIChlbXB0eSwgY3RyPTApXG4gIGxldCB7IHMwLCBzMSwgczIsIHMzIH0gPSBlbmNyeXB0KHhrLCBjMzJbMF0sIGMzMlsxXSwgYzMyWzJdLCBjMzJbM10pO1xuICBjb25zdCBzcmMzMiA9IHUzMihzcmMpO1xuICBjb25zdCBkc3QzMiA9IHUzMihkc3QpO1xuICAvLyBwcm9jZXNzIGJsb2Nrc1xuICBmb3IgKGxldCBpID0gMDsgaSArIDQgPD0gc3JjMzIubGVuZ3RoOyBpICs9IDQpIHtcbiAgICBkc3QzMltpICsgMF0gPSBzcmMzMltpICsgMF0gXiBzMDtcbiAgICBkc3QzMltpICsgMV0gPSBzcmMzMltpICsgMV0gXiBzMTtcbiAgICBkc3QzMltpICsgMl0gPSBzcmMzMltpICsgMl0gXiBzMjtcbiAgICBkc3QzMltpICsgM10gPSBzcmMzMltpICsgM10gXiBzMztcbiAgICAvLyBGdWxsIDEyOCBiaXQgY291bnRlciB3aXRoIHdyYXAgYXJvdW5kXG4gICAgbGV0IGNhcnJ5ID0gMTtcbiAgICBmb3IgKGxldCBpID0gY3RyLmxlbmd0aCAtIDE7IGkgPj0gMDsgaS0tKSB7XG4gICAgICBjYXJyeSA9IChjYXJyeSArIChjdHJbaV0gJiAweGZmKSkgfCAwO1xuICAgICAgY3RyW2ldID0gY2FycnkgJiAweGZmO1xuICAgICAgY2FycnkgPj4+PSA4O1xuICAgIH1cbiAgICAoeyBzMCwgczEsIHMyLCBzMyB9ID0gZW5jcnlwdCh4aywgYzMyWzBdLCBjMzJbMV0sIGMzMlsyXSwgYzMyWzNdKSk7XG4gIH1cbiAgLy8gbGVmdG92ZXJzIChsZXNzIHRoYW4gYmxvY2spXG4gIC8vIEl0J3MgcG9zc2libGUgdG8gaGFuZGxlID4gdTMyIGZhc3QsIGJ1dCBpcyBpdCB3b3J0aCBpdD9cbiAgY29uc3Qgc3RhcnQgPSBCTE9DS19TSVpFICogTWF0aC5mbG9vcihzcmMzMi5sZW5ndGggLyBCTE9DS19TSVpFMzIpO1xuICBpZiAoc3RhcnQgPCBzcmNMZW4pIHtcbiAgICBjb25zdCBiMzIgPSBuZXcgVWludDMyQXJyYXkoW3MwLCBzMSwgczIsIHMzXSk7XG4gICAgY29uc3QgYnVmID0gdTgoYjMyKTtcbiAgICBmb3IgKGxldCBpID0gc3RhcnQsIHBvcyA9IDA7IGkgPCBzcmNMZW47IGkrKywgcG9zKyspIGRzdFtpXSA9IHNyY1tpXSBeIGJ1Zltwb3NdO1xuICB9XG4gIHJldHVybiBkc3Q7XG59XG5cbi8vIEFFUyBDVFIgd2l0aCBvdmVyZmxvd2luZyAzMiBiaXQgY291bnRlclxuLy8gSXQncyBwb3NzaWJsZSB0byBkbyAzMmxlIHNpZ25pZmljYW50bHkgc2ltcGxlciAoYW5kIHByb2JhYmx5IGZhc3RlcikgYnkgdXNpbmcgdTMyLlxuLy8gQnV0LCB3ZSBuZWVkIGJvdGgsIGFuZCBwZXJmIGJvdHRsZW5lY2sgaXMgaW4gZ2hhc2ggYW55d2F5LlxuZnVuY3Rpb24gY3RyMzIoXG4gIHhrOiBVaW50MzJBcnJheSxcbiAgaXNMRTogYm9vbGVhbixcbiAgbm9uY2U6IFVpbnQ4QXJyYXksXG4gIHNyYzogVWludDhBcnJheSxcbiAgZHN0PzogVWludDhBcnJheVxuKSB7XG4gIGFieXRlcyhub25jZSwgQkxPQ0tfU0laRSk7XG4gIGFieXRlcyhzcmMpO1xuICBkc3QgPSBnZXREc3Qoc3JjLmxlbmd0aCwgZHN0KTtcbiAgY29uc3QgY3RyID0gbm9uY2U7IC8vIHdyaXRlIG5ldyB2YWx1ZSB0byBub25jZSwgc28gaXQgY2FuIGJlIHJlLXVzZWRcbiAgY29uc3QgYzMyID0gdTMyKGN0cik7XG4gIGNvbnN0IHZpZXcgPSBjcmVhdGVWaWV3KGN0cik7XG4gIGNvbnN0IHNyYzMyID0gdTMyKHNyYyk7XG4gIGNvbnN0IGRzdDMyID0gdTMyKGRzdCk7XG4gIGNvbnN0IGN0clBvcyA9IGlzTEUgPyAwIDogMTI7XG4gIGNvbnN0IHNyY0xlbiA9IHNyYy5sZW5ndGg7XG4gIC8vIEZpbGwgYmxvY2sgKGVtcHR5LCBjdHI9MClcbiAgbGV0IGN0ck51bSA9IHZpZXcuZ2V0VWludDMyKGN0clBvcywgaXNMRSk7IC8vIHJlYWQgY3VycmVudCBjb3VudGVyIHZhbHVlXG4gIGxldCB7IHMwLCBzMSwgczIsIHMzIH0gPSBlbmNyeXB0KHhrLCBjMzJbMF0sIGMzMlsxXSwgYzMyWzJdLCBjMzJbM10pO1xuICAvLyBwcm9jZXNzIGJsb2Nrc1xuICBmb3IgKGxldCBpID0gMDsgaSArIDQgPD0gc3JjMzIubGVuZ3RoOyBpICs9IDQpIHtcbiAgICBkc3QzMltpICsgMF0gPSBzcmMzMltpICsgMF0gXiBzMDtcbiAgICBkc3QzMltpICsgMV0gPSBzcmMzMltpICsgMV0gXiBzMTtcbiAgICBkc3QzMltpICsgMl0gPSBzcmMzMltpICsgMl0gXiBzMjtcbiAgICBkc3QzMltpICsgM10gPSBzcmMzMltpICsgM10gXiBzMztcbiAgICBjdHJOdW0gPSAoY3RyTnVtICsgMSkgPj4+IDA7IC8vIHUzMiB3cmFwXG4gICAgdmlldy5zZXRVaW50MzIoY3RyUG9zLCBjdHJOdW0sIGlzTEUpO1xuICAgICh7IHMwLCBzMSwgczIsIHMzIH0gPSBlbmNyeXB0KHhrLCBjMzJbMF0sIGMzMlsxXSwgYzMyWzJdLCBjMzJbM10pKTtcbiAgfVxuICAvLyBsZWZ0b3ZlcnMgKGxlc3MgdGhhbiBhIGJsb2NrKVxuICBjb25zdCBzdGFydCA9IEJMT0NLX1NJWkUgKiBNYXRoLmZsb29yKHNyYzMyLmxlbmd0aCAvIEJMT0NLX1NJWkUzMik7XG4gIGlmIChzdGFydCA8IHNyY0xlbikge1xuICAgIGNvbnN0IGIzMiA9IG5ldyBVaW50MzJBcnJheShbczAsIHMxLCBzMiwgczNdKTtcbiAgICBjb25zdCBidWYgPSB1OChiMzIpO1xuICAgIGZvciAobGV0IGkgPSBzdGFydCwgcG9zID0gMDsgaSA8IHNyY0xlbjsgaSsrLCBwb3MrKykgZHN0W2ldID0gc3JjW2ldIF4gYnVmW3Bvc107XG4gIH1cbiAgcmV0dXJuIGRzdDtcbn1cblxuLyoqXG4gKiBDVFI6IGNvdW50ZXIgbW9kZS4gQ3JlYXRlcyBzdHJlYW0gY2lwaGVyLlxuICogUmVxdWlyZXMgZ29vZCBJVi4gUGFyYWxsZWxpemFibGUuIE9LLCBidXQgbm8gTUFDLlxuICovXG5leHBvcnQgY29uc3QgY3RyID0gd3JhcENpcGhlcihcbiAgeyBibG9ja1NpemU6IDE2LCBub25jZUxlbmd0aDogMTYgfSxcbiAgZnVuY3Rpb24gY3RyKGtleTogVWludDhBcnJheSwgbm9uY2U6IFVpbnQ4QXJyYXkpOiBDaXBoZXJXaXRoT3V0cHV0IHtcbiAgICBhYnl0ZXMoa2V5KTtcbiAgICBhYnl0ZXMobm9uY2UsIEJMT0NLX1NJWkUpO1xuICAgIGZ1bmN0aW9uIHByb2Nlc3NDdHIoYnVmOiBVaW50OEFycmF5LCBkc3Q/OiBVaW50OEFycmF5KSB7XG4gICAgICBjb25zdCB4ayA9IGV4cGFuZEtleUxFKGtleSk7XG4gICAgICBjb25zdCBuID0gbm9uY2Uuc2xpY2UoKTtcbiAgICAgIGNvbnN0IG91dCA9IGN0ckNvdW50ZXIoeGssIG4sIGJ1ZiwgZHN0KTtcbiAgICAgIHhrLmZpbGwoMCk7XG4gICAgICBuLmZpbGwoMCk7XG4gICAgICByZXR1cm4gb3V0O1xuICAgIH1cbiAgICByZXR1cm4ge1xuICAgICAgZW5jcnlwdDogKHBsYWludGV4dDogVWludDhBcnJheSwgZHN0PzogVWludDhBcnJheSkgPT4gcHJvY2Vzc0N0cihwbGFpbnRleHQsIGRzdCksXG4gICAgICBkZWNyeXB0OiAoY2lwaGVydGV4dDogVWludDhBcnJheSwgZHN0PzogVWludDhBcnJheSkgPT4gcHJvY2Vzc0N0cihjaXBoZXJ0ZXh0LCBkc3QpLFxuICAgIH07XG4gIH1cbik7XG5cbmZ1bmN0aW9uIHZhbGlkYXRlQmxvY2tEZWNyeXB0KGRhdGE6IFVpbnQ4QXJyYXkpIHtcbiAgYWJ5dGVzKGRhdGEpO1xuICBpZiAoZGF0YS5sZW5ndGggJSBCTE9DS19TSVpFICE9PSAwKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKFxuICAgICAgYGFlcy8oY2JjLWVjYikuZGVjcnlwdCBjaXBoZXJ0ZXh0IHNob3VsZCBjb25zaXN0IG9mIGJsb2NrcyB3aXRoIHNpemUgJHtCTE9DS19TSVpFfWBcbiAgICApO1xuICB9XG59XG5cbmZ1bmN0aW9uIHZhbGlkYXRlQmxvY2tFbmNyeXB0KHBsYWludGV4dDogVWludDhBcnJheSwgcGNrczU6IGJvb2xlYW4sIGRzdD86IFVpbnQ4QXJyYXkpIHtcbiAgbGV0IG91dExlbiA9IHBsYWludGV4dC5sZW5ndGg7XG4gIGNvbnN0IHJlbWFpbmluZyA9IG91dExlbiAlIEJMT0NLX1NJWkU7XG4gIGlmICghcGNrczUgJiYgcmVtYWluaW5nICE9PSAwKVxuICAgIHRocm93IG5ldyBFcnJvcignYWVjLyhjYmMtZWNiKTogdW5wYWRkZWQgcGxhaW50ZXh0IHdpdGggZGlzYWJsZWQgcGFkZGluZycpO1xuICBjb25zdCBiID0gdTMyKHBsYWludGV4dCk7XG4gIGlmIChwY2tzNSkge1xuICAgIGxldCBsZWZ0ID0gQkxPQ0tfU0laRSAtIHJlbWFpbmluZztcbiAgICBpZiAoIWxlZnQpIGxlZnQgPSBCTE9DS19TSVpFOyAvLyBpZiBubyBieXRlcyBsZWZ0LCBjcmVhdGUgZW1wdHkgcGFkZGluZyBibG9ja1xuICAgIG91dExlbiA9IG91dExlbiArIGxlZnQ7XG4gIH1cbiAgY29uc3Qgb3V0ID0gZ2V0RHN0KG91dExlbiwgZHN0KTtcbiAgY29uc3QgbyA9IHUzMihvdXQpO1xuICByZXR1cm4geyBiLCBvLCBvdXQgfTtcbn1cblxuZnVuY3Rpb24gdmFsaWRhdGVQQ0tTKGRhdGE6IFVpbnQ4QXJyYXksIHBja3M1OiBib29sZWFuKSB7XG4gIGlmICghcGNrczUpIHJldHVybiBkYXRhO1xuICBjb25zdCBsZW4gPSBkYXRhLmxlbmd0aDtcbiAgaWYgKCFsZW4pIHRocm93IG5ldyBFcnJvcihgYWVzL3Bja3M1OiBlbXB0eSBjaXBoZXJ0ZXh0IG5vdCBhbGxvd2VkYCk7XG4gIGNvbnN0IGxhc3RCeXRlID0gZGF0YVtsZW4gLSAxXTtcbiAgaWYgKGxhc3RCeXRlIDw9IDAgfHwgbGFzdEJ5dGUgPiAxNikgdGhyb3cgbmV3IEVycm9yKGBhZXMvcGNrczU6IHdyb25nIHBhZGRpbmcgYnl0ZTogJHtsYXN0Qnl0ZX1gKTtcbiAgY29uc3Qgb3V0ID0gZGF0YS5zdWJhcnJheSgwLCAtbGFzdEJ5dGUpO1xuICBmb3IgKGxldCBpID0gMDsgaSA8IGxhc3RCeXRlOyBpKyspXG4gICAgaWYgKGRhdGFbbGVuIC0gaSAtIDFdICE9PSBsYXN0Qnl0ZSkgdGhyb3cgbmV3IEVycm9yKGBhZXMvcGNrczU6IHdyb25nIHBhZGRpbmdgKTtcbiAgcmV0dXJuIG91dDtcbn1cblxuZnVuY3Rpb24gcGFkUENLUyhsZWZ0OiBVaW50OEFycmF5KSB7XG4gIGNvbnN0IHRtcCA9IG5ldyBVaW50OEFycmF5KDE2KTtcbiAgY29uc3QgdG1wMzIgPSB1MzIodG1wKTtcbiAgdG1wLnNldChsZWZ0KTtcbiAgY29uc3QgcGFkZGluZ0J5dGUgPSBCTE9DS19TSVpFIC0gbGVmdC5sZW5ndGg7XG4gIGZvciAobGV0IGkgPSBCTE9DS19TSVpFIC0gcGFkZGluZ0J5dGU7IGkgPCBCTE9DS19TSVpFOyBpKyspIHRtcFtpXSA9IHBhZGRpbmdCeXRlO1xuICByZXR1cm4gdG1wMzI7XG59XG5cbmV4cG9ydCB0eXBlIEJsb2NrT3B0cyA9IHsgZGlzYWJsZVBhZGRpbmc/OiBib29sZWFuIH07XG5cbi8qKlxuICogRUNCOiBFbGVjdHJvbmljIENvZGVCb29rLiBTaW1wbGUgZGV0ZXJtaW5pc3RpYyByZXBsYWNlbWVudC5cbiAqIERhbmdlcm91czogYWx3YXlzIG1hcCB4IHRvIHkuIFNlZSBbQUVTIFBlbmd1aW5dKGh0dHBzOi8vd29yZHMuZmlsaXBwby5pby90aGUtZWNiLXBlbmd1aW4vKS5cbiAqL1xuZXhwb3J0IGNvbnN0IGVjYiA9IHdyYXBDaXBoZXIoXG4gIHsgYmxvY2tTaXplOiAxNiB9LFxuICBmdW5jdGlvbiBlY2Ioa2V5OiBVaW50OEFycmF5LCBvcHRzOiBCbG9ja09wdHMgPSB7fSk6IENpcGhlcldpdGhPdXRwdXQge1xuICAgIGFieXRlcyhrZXkpO1xuICAgIGNvbnN0IHBja3M1ID0gIW9wdHMuZGlzYWJsZVBhZGRpbmc7XG4gICAgcmV0dXJuIHtcbiAgICAgIGVuY3J5cHQ6IChwbGFpbnRleHQ6IFVpbnQ4QXJyYXksIGRzdD86IFVpbnQ4QXJyYXkpID0+IHtcbiAgICAgICAgYWJ5dGVzKHBsYWludGV4dCk7XG4gICAgICAgIGNvbnN0IHsgYiwgbywgb3V0OiBfb3V0IH0gPSB2YWxpZGF0ZUJsb2NrRW5jcnlwdChwbGFpbnRleHQsIHBja3M1LCBkc3QpO1xuICAgICAgICBjb25zdCB4ayA9IGV4cGFuZEtleUxFKGtleSk7XG4gICAgICAgIGxldCBpID0gMDtcbiAgICAgICAgZm9yICg7IGkgKyA0IDw9IGIubGVuZ3RoOyApIHtcbiAgICAgICAgICBjb25zdCB7IHMwLCBzMSwgczIsIHMzIH0gPSBlbmNyeXB0KHhrLCBiW2kgKyAwXSwgYltpICsgMV0sIGJbaSArIDJdLCBiW2kgKyAzXSk7XG4gICAgICAgICAgKG9baSsrXSA9IHMwKSwgKG9baSsrXSA9IHMxKSwgKG9baSsrXSA9IHMyKSwgKG9baSsrXSA9IHMzKTtcbiAgICAgICAgfVxuICAgICAgICBpZiAocGNrczUpIHtcbiAgICAgICAgICBjb25zdCB0bXAzMiA9IHBhZFBDS1MocGxhaW50ZXh0LnN1YmFycmF5KGkgKiA0KSk7XG4gICAgICAgICAgY29uc3QgeyBzMCwgczEsIHMyLCBzMyB9ID0gZW5jcnlwdCh4aywgdG1wMzJbMF0sIHRtcDMyWzFdLCB0bXAzMlsyXSwgdG1wMzJbM10pO1xuICAgICAgICAgIChvW2krK10gPSBzMCksIChvW2krK10gPSBzMSksIChvW2krK10gPSBzMiksIChvW2krK10gPSBzMyk7XG4gICAgICAgIH1cbiAgICAgICAgeGsuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIF9vdXQ7XG4gICAgICB9LFxuICAgICAgZGVjcnlwdDogKGNpcGhlcnRleHQ6IFVpbnQ4QXJyYXksIGRzdD86IFVpbnQ4QXJyYXkpID0+IHtcbiAgICAgICAgdmFsaWRhdGVCbG9ja0RlY3J5cHQoY2lwaGVydGV4dCk7XG4gICAgICAgIGNvbnN0IHhrID0gZXhwYW5kS2V5RGVjTEUoa2V5KTtcbiAgICAgICAgY29uc3Qgb3V0ID0gZ2V0RHN0KGNpcGhlcnRleHQubGVuZ3RoLCBkc3QpO1xuICAgICAgICBjb25zdCBiID0gdTMyKGNpcGhlcnRleHQpO1xuICAgICAgICBjb25zdCBvID0gdTMyKG91dCk7XG4gICAgICAgIGZvciAobGV0IGkgPSAwOyBpICsgNCA8PSBiLmxlbmd0aDsgKSB7XG4gICAgICAgICAgY29uc3QgeyBzMCwgczEsIHMyLCBzMyB9ID0gZGVjcnlwdCh4aywgYltpICsgMF0sIGJbaSArIDFdLCBiW2kgKyAyXSwgYltpICsgM10pO1xuICAgICAgICAgIChvW2krK10gPSBzMCksIChvW2krK10gPSBzMSksIChvW2krK10gPSBzMiksIChvW2krK10gPSBzMyk7XG4gICAgICAgIH1cbiAgICAgICAgeGsuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIHZhbGlkYXRlUENLUyhvdXQsIHBja3M1KTtcbiAgICAgIH0sXG4gICAgfTtcbiAgfVxuKTtcblxuLyoqXG4gKiBDQkM6IENpcGhlci1CbG9jay1DaGFpbmluZy4gS2V5IGlzIHByZXZpb3VzIHJvdW5kXHUyMDE5cyBibG9jay5cbiAqIEZyYWdpbGU6IG5lZWRzIHByb3BlciBwYWRkaW5nLiBVbmF1dGhlbnRpY2F0ZWQ6IG5lZWRzIE1BQy5cbiAqL1xuZXhwb3J0IGNvbnN0IGNiYyA9IHdyYXBDaXBoZXIoXG4gIHsgYmxvY2tTaXplOiAxNiwgbm9uY2VMZW5ndGg6IDE2IH0sXG4gIGZ1bmN0aW9uIGNiYyhrZXk6IFVpbnQ4QXJyYXksIGl2OiBVaW50OEFycmF5LCBvcHRzOiBCbG9ja09wdHMgPSB7fSk6IENpcGhlcldpdGhPdXRwdXQge1xuICAgIGFieXRlcyhrZXkpO1xuICAgIGFieXRlcyhpdiwgMTYpO1xuICAgIGNvbnN0IHBja3M1ID0gIW9wdHMuZGlzYWJsZVBhZGRpbmc7XG4gICAgcmV0dXJuIHtcbiAgICAgIGVuY3J5cHQ6IChwbGFpbnRleHQ6IFVpbnQ4QXJyYXksIGRzdD86IFVpbnQ4QXJyYXkpID0+IHtcbiAgICAgICAgY29uc3QgeGsgPSBleHBhbmRLZXlMRShrZXkpO1xuICAgICAgICBjb25zdCB7IGIsIG8sIG91dDogX291dCB9ID0gdmFsaWRhdGVCbG9ja0VuY3J5cHQocGxhaW50ZXh0LCBwY2tzNSwgZHN0KTtcbiAgICAgICAgY29uc3QgbjMyID0gdTMyKGl2KTtcbiAgICAgICAgLy8gcHJldHRpZXItaWdub3JlXG4gICAgICAgIGxldCBzMCA9IG4zMlswXSwgczEgPSBuMzJbMV0sIHMyID0gbjMyWzJdLCBzMyA9IG4zMlszXTtcbiAgICAgICAgbGV0IGkgPSAwO1xuICAgICAgICBmb3IgKDsgaSArIDQgPD0gYi5sZW5ndGg7ICkge1xuICAgICAgICAgIChzMCBePSBiW2kgKyAwXSksIChzMSBePSBiW2kgKyAxXSksIChzMiBePSBiW2kgKyAyXSksIChzMyBePSBiW2kgKyAzXSk7XG4gICAgICAgICAgKHsgczAsIHMxLCBzMiwgczMgfSA9IGVuY3J5cHQoeGssIHMwLCBzMSwgczIsIHMzKSk7XG4gICAgICAgICAgKG9baSsrXSA9IHMwKSwgKG9baSsrXSA9IHMxKSwgKG9baSsrXSA9IHMyKSwgKG9baSsrXSA9IHMzKTtcbiAgICAgICAgfVxuICAgICAgICBpZiAocGNrczUpIHtcbiAgICAgICAgICBjb25zdCB0bXAzMiA9IHBhZFBDS1MocGxhaW50ZXh0LnN1YmFycmF5KGkgKiA0KSk7XG4gICAgICAgICAgKHMwIF49IHRtcDMyWzBdKSwgKHMxIF49IHRtcDMyWzFdKSwgKHMyIF49IHRtcDMyWzJdKSwgKHMzIF49IHRtcDMyWzNdKTtcbiAgICAgICAgICAoeyBzMCwgczEsIHMyLCBzMyB9ID0gZW5jcnlwdCh4aywgczAsIHMxLCBzMiwgczMpKTtcbiAgICAgICAgICAob1tpKytdID0gczApLCAob1tpKytdID0gczEpLCAob1tpKytdID0gczIpLCAob1tpKytdID0gczMpO1xuICAgICAgICB9XG4gICAgICAgIHhrLmZpbGwoMCk7XG4gICAgICAgIHJldHVybiBfb3V0O1xuICAgICAgfSxcbiAgICAgIGRlY3J5cHQ6IChjaXBoZXJ0ZXh0OiBVaW50OEFycmF5LCBkc3Q/OiBVaW50OEFycmF5KSA9PiB7XG4gICAgICAgIHZhbGlkYXRlQmxvY2tEZWNyeXB0KGNpcGhlcnRleHQpO1xuICAgICAgICBjb25zdCB4ayA9IGV4cGFuZEtleURlY0xFKGtleSk7XG4gICAgICAgIGNvbnN0IG4zMiA9IHUzMihpdik7XG4gICAgICAgIGNvbnN0IG91dCA9IGdldERzdChjaXBoZXJ0ZXh0Lmxlbmd0aCwgZHN0KTtcbiAgICAgICAgY29uc3QgYiA9IHUzMihjaXBoZXJ0ZXh0KTtcbiAgICAgICAgY29uc3QgbyA9IHUzMihvdXQpO1xuICAgICAgICAvLyBwcmV0dGllci1pZ25vcmVcbiAgICAgICAgbGV0IHMwID0gbjMyWzBdLCBzMSA9IG4zMlsxXSwgczIgPSBuMzJbMl0sIHMzID0gbjMyWzNdO1xuICAgICAgICBmb3IgKGxldCBpID0gMDsgaSArIDQgPD0gYi5sZW5ndGg7ICkge1xuICAgICAgICAgIC8vIHByZXR0aWVyLWlnbm9yZVxuICAgICAgICAgIGNvbnN0IHBzMCA9IHMwLCBwczEgPSBzMSwgcHMyID0gczIsIHBzMyA9IHMzO1xuICAgICAgICAgIChzMCA9IGJbaSArIDBdKSwgKHMxID0gYltpICsgMV0pLCAoczIgPSBiW2kgKyAyXSksIChzMyA9IGJbaSArIDNdKTtcbiAgICAgICAgICBjb25zdCB7IHMwOiBvMCwgczE6IG8xLCBzMjogbzIsIHMzOiBvMyB9ID0gZGVjcnlwdCh4aywgczAsIHMxLCBzMiwgczMpO1xuICAgICAgICAgIChvW2krK10gPSBvMCBeIHBzMCksIChvW2krK10gPSBvMSBeIHBzMSksIChvW2krK10gPSBvMiBeIHBzMiksIChvW2krK10gPSBvMyBeIHBzMyk7XG4gICAgICAgIH1cbiAgICAgICAgeGsuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIHZhbGlkYXRlUENLUyhvdXQsIHBja3M1KTtcbiAgICAgIH0sXG4gICAgfTtcbiAgfVxuKTtcblxuLyoqXG4gKiBDRkI6IENpcGhlciBGZWVkYmFjayBNb2RlLiBUaGUgaW5wdXQgZm9yIHRoZSBibG9jayBjaXBoZXIgaXMgdGhlIHByZXZpb3VzIGNpcGhlciBvdXRwdXQuXG4gKiBVbmF1dGhlbnRpY2F0ZWQ6IG5lZWRzIE1BQy5cbiAqL1xuZXhwb3J0IGNvbnN0IGNmYiA9IHdyYXBDaXBoZXIoXG4gIHsgYmxvY2tTaXplOiAxNiwgbm9uY2VMZW5ndGg6IDE2IH0sXG4gIGZ1bmN0aW9uIGNmYihrZXk6IFVpbnQ4QXJyYXksIGl2OiBVaW50OEFycmF5KTogQ2lwaGVyV2l0aE91dHB1dCB7XG4gICAgYWJ5dGVzKGtleSk7XG4gICAgYWJ5dGVzKGl2LCAxNik7XG4gICAgZnVuY3Rpb24gcHJvY2Vzc0NmYihzcmM6IFVpbnQ4QXJyYXksIGlzRW5jcnlwdDogYm9vbGVhbiwgZHN0PzogVWludDhBcnJheSkge1xuICAgICAgY29uc3QgeGsgPSBleHBhbmRLZXlMRShrZXkpO1xuICAgICAgY29uc3Qgc3JjTGVuID0gc3JjLmxlbmd0aDtcbiAgICAgIGRzdCA9IGdldERzdChzcmNMZW4sIGRzdCk7XG4gICAgICBjb25zdCBzcmMzMiA9IHUzMihzcmMpO1xuICAgICAgY29uc3QgZHN0MzIgPSB1MzIoZHN0KTtcbiAgICAgIGNvbnN0IG5leHQzMiA9IGlzRW5jcnlwdCA/IGRzdDMyIDogc3JjMzI7XG4gICAgICBjb25zdCBuMzIgPSB1MzIoaXYpO1xuICAgICAgLy8gcHJldHRpZXItaWdub3JlXG4gICAgICBsZXQgczAgPSBuMzJbMF0sIHMxID0gbjMyWzFdLCBzMiA9IG4zMlsyXSwgczMgPSBuMzJbM107XG4gICAgICBmb3IgKGxldCBpID0gMDsgaSArIDQgPD0gc3JjMzIubGVuZ3RoOyApIHtcbiAgICAgICAgY29uc3QgeyBzMDogZTAsIHMxOiBlMSwgczI6IGUyLCBzMzogZTMgfSA9IGVuY3J5cHQoeGssIHMwLCBzMSwgczIsIHMzKTtcbiAgICAgICAgZHN0MzJbaSArIDBdID0gc3JjMzJbaSArIDBdIF4gZTA7XG4gICAgICAgIGRzdDMyW2kgKyAxXSA9IHNyYzMyW2kgKyAxXSBeIGUxO1xuICAgICAgICBkc3QzMltpICsgMl0gPSBzcmMzMltpICsgMl0gXiBlMjtcbiAgICAgICAgZHN0MzJbaSArIDNdID0gc3JjMzJbaSArIDNdIF4gZTM7XG4gICAgICAgIChzMCA9IG5leHQzMltpKytdKSwgKHMxID0gbmV4dDMyW2krK10pLCAoczIgPSBuZXh0MzJbaSsrXSksIChzMyA9IG5leHQzMltpKytdKTtcbiAgICAgIH1cbiAgICAgIC8vIGxlZnRvdmVycyAobGVzcyB0aGFuIGJsb2NrKVxuICAgICAgY29uc3Qgc3RhcnQgPSBCTE9DS19TSVpFICogTWF0aC5mbG9vcihzcmMzMi5sZW5ndGggLyBCTE9DS19TSVpFMzIpO1xuICAgICAgaWYgKHN0YXJ0IDwgc3JjTGVuKSB7XG4gICAgICAgICh7IHMwLCBzMSwgczIsIHMzIH0gPSBlbmNyeXB0KHhrLCBzMCwgczEsIHMyLCBzMykpO1xuICAgICAgICBjb25zdCBidWYgPSB1OChuZXcgVWludDMyQXJyYXkoW3MwLCBzMSwgczIsIHMzXSkpO1xuICAgICAgICBmb3IgKGxldCBpID0gc3RhcnQsIHBvcyA9IDA7IGkgPCBzcmNMZW47IGkrKywgcG9zKyspIGRzdFtpXSA9IHNyY1tpXSBeIGJ1Zltwb3NdO1xuICAgICAgICBidWYuZmlsbCgwKTtcbiAgICAgIH1cbiAgICAgIHhrLmZpbGwoMCk7XG4gICAgICByZXR1cm4gZHN0O1xuICAgIH1cbiAgICByZXR1cm4ge1xuICAgICAgZW5jcnlwdDogKHBsYWludGV4dDogVWludDhBcnJheSwgZHN0PzogVWludDhBcnJheSkgPT4gcHJvY2Vzc0NmYihwbGFpbnRleHQsIHRydWUsIGRzdCksXG4gICAgICBkZWNyeXB0OiAoY2lwaGVydGV4dDogVWludDhBcnJheSwgZHN0PzogVWludDhBcnJheSkgPT4gcHJvY2Vzc0NmYihjaXBoZXJ0ZXh0LCBmYWxzZSwgZHN0KSxcbiAgICB9O1xuICB9XG4pO1xuXG4vLyBUT0RPOiBtZXJnZSB3aXRoIGNoYWNoYSwgaG93ZXZlciBnY20gaGFzIGJpdExlbiB3aGlsZSBjaGFjaGEgaGFzIGJ5dGVMZW5cbmZ1bmN0aW9uIGNvbXB1dGVUYWcoXG4gIGZuOiB0eXBlb2YgZ2hhc2gsXG4gIGlzTEU6IGJvb2xlYW4sXG4gIGtleTogVWludDhBcnJheSxcbiAgZGF0YTogVWludDhBcnJheSxcbiAgQUFEPzogVWludDhBcnJheVxuKSB7XG4gIGNvbnN0IGggPSBmbi5jcmVhdGUoa2V5LCBkYXRhLmxlbmd0aCArIChBQUQ/Lmxlbmd0aCB8fCAwKSk7XG4gIGlmIChBQUQpIGgudXBkYXRlKEFBRCk7XG4gIGgudXBkYXRlKGRhdGEpO1xuICBjb25zdCBudW0gPSBuZXcgVWludDhBcnJheSgxNik7XG4gIGNvbnN0IHZpZXcgPSBjcmVhdGVWaWV3KG51bSk7XG4gIGlmIChBQUQpIHNldEJpZ1VpbnQ2NCh2aWV3LCAwLCBCaWdJbnQoQUFELmxlbmd0aCAqIDgpLCBpc0xFKTtcbiAgc2V0QmlnVWludDY0KHZpZXcsIDgsIEJpZ0ludChkYXRhLmxlbmd0aCAqIDgpLCBpc0xFKTtcbiAgaC51cGRhdGUobnVtKTtcbiAgcmV0dXJuIGguZGlnZXN0KCk7XG59XG5cbi8qKlxuICogR0NNOiBHYWxvaXMvQ291bnRlciBNb2RlLlxuICogR29vZCwgbW9kZXJuIHZlcnNpb24gb2YgQ1RSLCBwYXJhbGxlbCwgd2l0aCBNQUMuXG4gKiBCZSBjYXJlZnVsOiBNQUNzIGNhbiBiZSBmb3JnZWQuXG4gKi9cbmV4cG9ydCBjb25zdCBnY20gPSB3cmFwQ2lwaGVyKFxuICB7IGJsb2NrU2l6ZTogMTYsIG5vbmNlTGVuZ3RoOiAxMiwgdGFnTGVuZ3RoOiAxNiB9LFxuICBmdW5jdGlvbiBnY20oa2V5OiBVaW50OEFycmF5LCBub25jZTogVWludDhBcnJheSwgQUFEPzogVWludDhBcnJheSk6IENpcGhlciB7XG4gICAgYWJ5dGVzKG5vbmNlKTtcbiAgICAvLyBOb25jZSBjYW4gYmUgcHJldHR5IG11Y2ggYW55dGhpbmcgKGV2ZW4gMSBieXRlKS4gQnV0IHNtYWxsZXIgbm9uY2VzIGxlc3Mgc2VjdXJlLlxuICAgIGlmIChub25jZS5sZW5ndGggPT09IDApIHRocm93IG5ldyBFcnJvcignYWVzL2djbTogZW1wdHkgbm9uY2UnKTtcbiAgICBjb25zdCB0YWdMZW5ndGggPSAxNjtcbiAgICBmdW5jdGlvbiBfY29tcHV0ZVRhZyhhdXRoS2V5OiBVaW50OEFycmF5LCB0YWdNYXNrOiBVaW50OEFycmF5LCBkYXRhOiBVaW50OEFycmF5KSB7XG4gICAgICBjb25zdCB0YWcgPSBjb21wdXRlVGFnKGdoYXNoLCBmYWxzZSwgYXV0aEtleSwgZGF0YSwgQUFEKTtcbiAgICAgIGZvciAobGV0IGkgPSAwOyBpIDwgdGFnTWFzay5sZW5ndGg7IGkrKykgdGFnW2ldIF49IHRhZ01hc2tbaV07XG4gICAgICByZXR1cm4gdGFnO1xuICAgIH1cbiAgICBmdW5jdGlvbiBkZXJpdmVLZXlzKCkge1xuICAgICAgY29uc3QgeGsgPSBleHBhbmRLZXlMRShrZXkpO1xuICAgICAgY29uc3QgYXV0aEtleSA9IEVNUFRZX0JMT0NLLnNsaWNlKCk7XG4gICAgICBjb25zdCBjb3VudGVyID0gRU1QVFlfQkxPQ0suc2xpY2UoKTtcbiAgICAgIGN0cjMyKHhrLCBmYWxzZSwgY291bnRlciwgY291bnRlciwgYXV0aEtleSk7XG4gICAgICBpZiAobm9uY2UubGVuZ3RoID09PSAxMikge1xuICAgICAgICBjb3VudGVyLnNldChub25jZSk7XG4gICAgICB9IGVsc2Uge1xuICAgICAgICAvLyBTcGVjIChOSVNUIDgwMC0zOGQpIHN1cHBvcnRzIHZhcmlhYmxlIHNpemUgbm9uY2UuXG4gICAgICAgIC8vIE5vdCBzdXBwb3J0ZWQgZm9yIG5vdywgYnV0IGNhbiBiZSB1c2VmdWwuXG4gICAgICAgIGNvbnN0IG5vbmNlTGVuID0gRU1QVFlfQkxPQ0suc2xpY2UoKTtcbiAgICAgICAgY29uc3QgdmlldyA9IGNyZWF0ZVZpZXcobm9uY2VMZW4pO1xuICAgICAgICBzZXRCaWdVaW50NjQodmlldywgOCwgQmlnSW50KG5vbmNlLmxlbmd0aCAqIDgpLCBmYWxzZSk7XG4gICAgICAgIC8vIGdoYXNoKG5vbmNlIHx8IHU2NGJlKDApIHx8IHU2NGJlKG5vbmNlTGVuKjgpKVxuICAgICAgICBnaGFzaC5jcmVhdGUoYXV0aEtleSkudXBkYXRlKG5vbmNlKS51cGRhdGUobm9uY2VMZW4pLmRpZ2VzdEludG8oY291bnRlcik7XG4gICAgICB9XG4gICAgICBjb25zdCB0YWdNYXNrID0gY3RyMzIoeGssIGZhbHNlLCBjb3VudGVyLCBFTVBUWV9CTE9DSyk7XG4gICAgICByZXR1cm4geyB4aywgYXV0aEtleSwgY291bnRlciwgdGFnTWFzayB9O1xuICAgIH1cbiAgICByZXR1cm4ge1xuICAgICAgZW5jcnlwdDogKHBsYWludGV4dDogVWludDhBcnJheSkgPT4ge1xuICAgICAgICBhYnl0ZXMocGxhaW50ZXh0KTtcbiAgICAgICAgY29uc3QgeyB4aywgYXV0aEtleSwgY291bnRlciwgdGFnTWFzayB9ID0gZGVyaXZlS2V5cygpO1xuICAgICAgICBjb25zdCBvdXQgPSBuZXcgVWludDhBcnJheShwbGFpbnRleHQubGVuZ3RoICsgdGFnTGVuZ3RoKTtcbiAgICAgICAgY3RyMzIoeGssIGZhbHNlLCBjb3VudGVyLCBwbGFpbnRleHQsIG91dCk7XG4gICAgICAgIGNvbnN0IHRhZyA9IF9jb21wdXRlVGFnKGF1dGhLZXksIHRhZ01hc2ssIG91dC5zdWJhcnJheSgwLCBvdXQubGVuZ3RoIC0gdGFnTGVuZ3RoKSk7XG4gICAgICAgIG91dC5zZXQodGFnLCBwbGFpbnRleHQubGVuZ3RoKTtcbiAgICAgICAgeGsuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIG91dDtcbiAgICAgIH0sXG4gICAgICBkZWNyeXB0OiAoY2lwaGVydGV4dDogVWludDhBcnJheSkgPT4ge1xuICAgICAgICBhYnl0ZXMoY2lwaGVydGV4dCk7XG4gICAgICAgIGlmIChjaXBoZXJ0ZXh0Lmxlbmd0aCA8IHRhZ0xlbmd0aClcbiAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYGFlcy9nY206IGNpcGhlcnRleHQgbGVzcyB0aGFuIHRhZ0xlbiAoJHt0YWdMZW5ndGh9KWApO1xuICAgICAgICBjb25zdCB7IHhrLCBhdXRoS2V5LCBjb3VudGVyLCB0YWdNYXNrIH0gPSBkZXJpdmVLZXlzKCk7XG4gICAgICAgIGNvbnN0IGRhdGEgPSBjaXBoZXJ0ZXh0LnN1YmFycmF5KDAsIC10YWdMZW5ndGgpO1xuICAgICAgICBjb25zdCBwYXNzZWRUYWcgPSBjaXBoZXJ0ZXh0LnN1YmFycmF5KC10YWdMZW5ndGgpO1xuICAgICAgICBjb25zdCB0YWcgPSBfY29tcHV0ZVRhZyhhdXRoS2V5LCB0YWdNYXNrLCBkYXRhKTtcbiAgICAgICAgaWYgKCFlcXVhbEJ5dGVzKHRhZywgcGFzc2VkVGFnKSkgdGhyb3cgbmV3IEVycm9yKCdhZXMvZ2NtOiBpbnZhbGlkIGdoYXNoIHRhZycpO1xuICAgICAgICBjb25zdCBvdXQgPSBjdHIzMih4aywgZmFsc2UsIGNvdW50ZXIsIGRhdGEpO1xuICAgICAgICBhdXRoS2V5LmZpbGwoMCk7XG4gICAgICAgIHRhZ01hc2suZmlsbCgwKTtcbiAgICAgICAgeGsuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIG91dDtcbiAgICAgIH0sXG4gICAgfTtcbiAgfVxuKTtcblxuY29uc3QgbGltaXQgPSAobmFtZTogc3RyaW5nLCBtaW46IG51bWJlciwgbWF4OiBudW1iZXIpID0+ICh2YWx1ZTogbnVtYmVyKSA9PiB7XG4gIGlmICghTnVtYmVyLmlzU2FmZUludGVnZXIodmFsdWUpIHx8IG1pbiA+IHZhbHVlIHx8IHZhbHVlID4gbWF4KVxuICAgIHRocm93IG5ldyBFcnJvcihgJHtuYW1lfTogaW52YWxpZCB2YWx1ZT0ke3ZhbHVlfSwgbXVzdCBiZSBbJHttaW59Li4ke21heH1dYCk7XG59O1xuXG4vKipcbiAqIEFFUy1HQ00tU0lWOiBjbGFzc2ljIEFFUy1HQ00gd2l0aCBub25jZS1taXN1c2UgcmVzaXN0YW5jZS5cbiAqIEd1YXJhbnRlZXMgdGhhdCwgd2hlbiBhIG5vbmNlIGlzIHJlcGVhdGVkLCB0aGUgb25seSBzZWN1cml0eSBsb3NzIGlzIHRoYXQgaWRlbnRpY2FsXG4gKiBwbGFpbnRleHRzIHdpbGwgcHJvZHVjZSBpZGVudGljYWwgY2lwaGVydGV4dHMuXG4gKiBSRkMgODQ1MiwgaHR0cHM6Ly9kYXRhdHJhY2tlci5pZXRmLm9yZy9kb2MvaHRtbC9yZmM4NDUyXG4gKi9cbmV4cG9ydCBjb25zdCBzaXYgPSB3cmFwQ2lwaGVyKFxuICB7IGJsb2NrU2l6ZTogMTYsIG5vbmNlTGVuZ3RoOiAxMiwgdGFnTGVuZ3RoOiAxNiB9LFxuICBmdW5jdGlvbiBzaXYoa2V5OiBVaW50OEFycmF5LCBub25jZTogVWludDhBcnJheSwgQUFEPzogVWludDhBcnJheSk6IENpcGhlciB7XG4gICAgY29uc3QgdGFnTGVuZ3RoID0gMTY7XG4gICAgLy8gRnJvbSBSRkMgODQ1MjogU2VjdGlvbiA2XG4gICAgY29uc3QgQUFEX0xJTUlUID0gbGltaXQoJ0FBRCcsIDAsIDIgKiogMzYpO1xuICAgIGNvbnN0IFBMQUlOX0xJTUlUID0gbGltaXQoJ3BsYWludGV4dCcsIDAsIDIgKiogMzYpO1xuICAgIGNvbnN0IE5PTkNFX0xJTUlUID0gbGltaXQoJ25vbmNlJywgMTIsIDEyKTtcbiAgICBjb25zdCBDSVBIRVJfTElNSVQgPSBsaW1pdCgnY2lwaGVydGV4dCcsIDE2LCAyICoqIDM2ICsgMTYpO1xuICAgIGFieXRlcyhub25jZSk7XG4gICAgTk9OQ0VfTElNSVQobm9uY2UubGVuZ3RoKTtcbiAgICBpZiAoQUFEKSB7XG4gICAgICBhYnl0ZXMoQUFEKTtcbiAgICAgIEFBRF9MSU1JVChBQUQubGVuZ3RoKTtcbiAgICB9XG4gICAgZnVuY3Rpb24gZGVyaXZlS2V5cygpIHtcbiAgICAgIGNvbnN0IGxlbiA9IGtleS5sZW5ndGg7XG4gICAgICBpZiAobGVuICE9PSAxNiAmJiBsZW4gIT09IDI0ICYmIGxlbiAhPT0gMzIpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihga2V5IGxlbmd0aCBtdXN0IGJlIDE2LCAyNCBvciAzMiBieXRlcywgZ290OiAke2xlbn0gYnl0ZXNgKTtcbiAgICAgIGNvbnN0IHhrID0gZXhwYW5kS2V5TEUoa2V5KTtcbiAgICAgIGNvbnN0IGVuY0tleSA9IG5ldyBVaW50OEFycmF5KGxlbik7XG4gICAgICBjb25zdCBhdXRoS2V5ID0gbmV3IFVpbnQ4QXJyYXkoMTYpO1xuICAgICAgY29uc3QgbjMyID0gdTMyKG5vbmNlKTtcbiAgICAgIC8vIHByZXR0aWVyLWlnbm9yZVxuICAgICAgbGV0IHMwID0gMCwgczEgPSBuMzJbMF0sIHMyID0gbjMyWzFdLCBzMyA9IG4zMlsyXTtcbiAgICAgIGxldCBjb3VudGVyID0gMDtcbiAgICAgIGZvciAoY29uc3QgZGVyaXZlZEtleSBvZiBbYXV0aEtleSwgZW5jS2V5XS5tYXAodTMyKSkge1xuICAgICAgICBjb25zdCBkMzIgPSB1MzIoZGVyaXZlZEtleSk7XG4gICAgICAgIGZvciAobGV0IGkgPSAwOyBpIDwgZDMyLmxlbmd0aDsgaSArPSAyKSB7XG4gICAgICAgICAgLy8gYWVzKHUzMmxlKDApIHx8IG5vbmNlKVs6OF0gfHwgYWVzKHUzMmxlKDEpIHx8IG5vbmNlKVs6OF0gLi4uXG4gICAgICAgICAgY29uc3QgeyBzMDogbzAsIHMxOiBvMSB9ID0gZW5jcnlwdCh4aywgczAsIHMxLCBzMiwgczMpO1xuICAgICAgICAgIGQzMltpICsgMF0gPSBvMDtcbiAgICAgICAgICBkMzJbaSArIDFdID0gbzE7XG4gICAgICAgICAgczAgPSArK2NvdW50ZXI7IC8vIGluY3JlbWVudCBjb3VudGVyIGluc2lkZSBzdGF0ZVxuICAgICAgICB9XG4gICAgICB9XG4gICAgICB4ay5maWxsKDApO1xuICAgICAgcmV0dXJuIHsgYXV0aEtleSwgZW5jS2V5OiBleHBhbmRLZXlMRShlbmNLZXkpIH07XG4gICAgfVxuICAgIGZ1bmN0aW9uIF9jb21wdXRlVGFnKGVuY0tleTogVWludDMyQXJyYXksIGF1dGhLZXk6IFVpbnQ4QXJyYXksIGRhdGE6IFVpbnQ4QXJyYXkpIHtcbiAgICAgIGNvbnN0IHRhZyA9IGNvbXB1dGVUYWcocG9seXZhbCwgdHJ1ZSwgYXV0aEtleSwgZGF0YSwgQUFEKTtcbiAgICAgIC8vIENvbXB1dGUgdGhlIGV4cGVjdGVkIHRhZyBieSBYT1JpbmcgU19zIGFuZCB0aGUgbm9uY2UsIGNsZWFyaW5nIHRoZVxuICAgICAgLy8gbW9zdCBzaWduaWZpY2FudCBiaXQgb2YgdGhlIGxhc3QgYnl0ZSBhbmQgZW5jcnlwdGluZyB3aXRoIHRoZVxuICAgICAgLy8gbWVzc2FnZS1lbmNyeXB0aW9uIGtleS5cbiAgICAgIGZvciAobGV0IGkgPSAwOyBpIDwgMTI7IGkrKykgdGFnW2ldIF49IG5vbmNlW2ldO1xuICAgICAgdGFnWzE1XSAmPSAweDdmOyAvLyBDbGVhciB0aGUgaGlnaGVzdCBiaXRcbiAgICAgIC8vIGVuY3J5cHQgdGFnIGFzIGJsb2NrXG4gICAgICBjb25zdCB0MzIgPSB1MzIodGFnKTtcbiAgICAgIC8vIHByZXR0aWVyLWlnbm9yZVxuICAgICAgbGV0IHMwID0gdDMyWzBdLCBzMSA9IHQzMlsxXSwgczIgPSB0MzJbMl0sIHMzID0gdDMyWzNdO1xuICAgICAgKHsgczAsIHMxLCBzMiwgczMgfSA9IGVuY3J5cHQoZW5jS2V5LCBzMCwgczEsIHMyLCBzMykpO1xuICAgICAgKHQzMlswXSA9IHMwKSwgKHQzMlsxXSA9IHMxKSwgKHQzMlsyXSA9IHMyKSwgKHQzMlszXSA9IHMzKTtcbiAgICAgIHJldHVybiB0YWc7XG4gICAgfVxuICAgIC8vIGFjdHVhbCBkZWNyeXB0L2VuY3J5cHQgb2YgbWVzc2FnZS5cbiAgICBmdW5jdGlvbiBwcm9jZXNzU2l2KGVuY0tleTogVWludDMyQXJyYXksIHRhZzogVWludDhBcnJheSwgaW5wdXQ6IFVpbnQ4QXJyYXkpIHtcbiAgICAgIGxldCBibG9jayA9IHRhZy5zbGljZSgpO1xuICAgICAgYmxvY2tbMTVdIHw9IDB4ODA7IC8vIEZvcmNlIGhpZ2hlc3QgYml0XG4gICAgICByZXR1cm4gY3RyMzIoZW5jS2V5LCB0cnVlLCBibG9jaywgaW5wdXQpO1xuICAgIH1cbiAgICByZXR1cm4ge1xuICAgICAgZW5jcnlwdDogKHBsYWludGV4dDogVWludDhBcnJheSkgPT4ge1xuICAgICAgICBhYnl0ZXMocGxhaW50ZXh0KTtcbiAgICAgICAgUExBSU5fTElNSVQocGxhaW50ZXh0Lmxlbmd0aCk7XG4gICAgICAgIGNvbnN0IHsgZW5jS2V5LCBhdXRoS2V5IH0gPSBkZXJpdmVLZXlzKCk7XG4gICAgICAgIGNvbnN0IHRhZyA9IF9jb21wdXRlVGFnKGVuY0tleSwgYXV0aEtleSwgcGxhaW50ZXh0KTtcbiAgICAgICAgY29uc3Qgb3V0ID0gbmV3IFVpbnQ4QXJyYXkocGxhaW50ZXh0Lmxlbmd0aCArIHRhZ0xlbmd0aCk7XG4gICAgICAgIG91dC5zZXQodGFnLCBwbGFpbnRleHQubGVuZ3RoKTtcbiAgICAgICAgb3V0LnNldChwcm9jZXNzU2l2KGVuY0tleSwgdGFnLCBwbGFpbnRleHQpKTtcbiAgICAgICAgZW5jS2V5LmZpbGwoMCk7XG4gICAgICAgIGF1dGhLZXkuZmlsbCgwKTtcbiAgICAgICAgcmV0dXJuIG91dDtcbiAgICAgIH0sXG4gICAgICBkZWNyeXB0OiAoY2lwaGVydGV4dDogVWludDhBcnJheSkgPT4ge1xuICAgICAgICBhYnl0ZXMoY2lwaGVydGV4dCk7XG4gICAgICAgIENJUEhFUl9MSU1JVChjaXBoZXJ0ZXh0Lmxlbmd0aCk7XG4gICAgICAgIGNvbnN0IHRhZyA9IGNpcGhlcnRleHQuc3ViYXJyYXkoLXRhZ0xlbmd0aCk7XG4gICAgICAgIGNvbnN0IHsgZW5jS2V5LCBhdXRoS2V5IH0gPSBkZXJpdmVLZXlzKCk7XG4gICAgICAgIGNvbnN0IHBsYWludGV4dCA9IHByb2Nlc3NTaXYoZW5jS2V5LCB0YWcsIGNpcGhlcnRleHQuc3ViYXJyYXkoMCwgLXRhZ0xlbmd0aCkpO1xuICAgICAgICBjb25zdCBleHBlY3RlZFRhZyA9IF9jb21wdXRlVGFnKGVuY0tleSwgYXV0aEtleSwgcGxhaW50ZXh0KTtcbiAgICAgICAgZW5jS2V5LmZpbGwoMCk7XG4gICAgICAgIGF1dGhLZXkuZmlsbCgwKTtcbiAgICAgICAgaWYgKCFlcXVhbEJ5dGVzKHRhZywgZXhwZWN0ZWRUYWcpKSB0aHJvdyBuZXcgRXJyb3IoJ2ludmFsaWQgcG9seXZhbCB0YWcnKTtcbiAgICAgICAgcmV0dXJuIHBsYWludGV4dDtcbiAgICAgIH0sXG4gICAgfTtcbiAgfVxuKTtcblxuZnVuY3Rpb24gaXNCeXRlczMyKGE6IHVua25vd24pOiBhIGlzIFVpbnQ4QXJyYXkge1xuICByZXR1cm4gKFxuICAgIGEgIT0gbnVsbCAmJlxuICAgIHR5cGVvZiBhID09PSAnb2JqZWN0JyAmJlxuICAgIChhIGluc3RhbmNlb2YgVWludDMyQXJyYXkgfHwgYS5jb25zdHJ1Y3Rvci5uYW1lID09PSAnVWludDMyQXJyYXknKVxuICApO1xufVxuXG5mdW5jdGlvbiBlbmNyeXB0QmxvY2soeGs6IFVpbnQzMkFycmF5LCBibG9jazogVWludDhBcnJheSkge1xuICBhYnl0ZXMoYmxvY2ssIDE2KTtcbiAgaWYgKCFpc0J5dGVzMzIoeGspKSB0aHJvdyBuZXcgRXJyb3IoJ19lbmNyeXB0QmxvY2sgYWNjZXB0cyByZXN1bHQgb2YgZXhwYW5kS2V5TEUnKTtcbiAgY29uc3QgYjMyID0gdTMyKGJsb2NrKTtcbiAgbGV0IHsgczAsIHMxLCBzMiwgczMgfSA9IGVuY3J5cHQoeGssIGIzMlswXSwgYjMyWzFdLCBiMzJbMl0sIGIzMlszXSk7XG4gIChiMzJbMF0gPSBzMCksIChiMzJbMV0gPSBzMSksIChiMzJbMl0gPSBzMiksIChiMzJbM10gPSBzMyk7XG4gIHJldHVybiBibG9jaztcbn1cblxuZnVuY3Rpb24gZGVjcnlwdEJsb2NrKHhrOiBVaW50MzJBcnJheSwgYmxvY2s6IFVpbnQ4QXJyYXkpIHtcbiAgYWJ5dGVzKGJsb2NrLCAxNik7XG4gIGlmICghaXNCeXRlczMyKHhrKSkgdGhyb3cgbmV3IEVycm9yKCdfZGVjcnlwdEJsb2NrIGFjY2VwdHMgcmVzdWx0IG9mIGV4cGFuZEtleUxFJyk7XG4gIGNvbnN0IGIzMiA9IHUzMihibG9jayk7XG4gIGxldCB7IHMwLCBzMSwgczIsIHMzIH0gPSBkZWNyeXB0KHhrLCBiMzJbMF0sIGIzMlsxXSwgYjMyWzJdLCBiMzJbM10pO1xuICAoYjMyWzBdID0gczApLCAoYjMyWzFdID0gczEpLCAoYjMyWzJdID0gczIpLCAoYjMyWzNdID0gczMpO1xuICByZXR1cm4gYmxvY2s7XG59XG5cbi8vIEhpZ2hseSB1bnNhZmUgcHJpdmF0ZSBmdW5jdGlvbnMgZm9yIGltcGxlbWVudGluZyBuZXcgbW9kZXMgb3IgY2lwaGVycyBiYXNlZCBvbiBBRVNcbi8vIENhbiBjaGFuZ2UgYXQgYW55IHRpbWUsIG5vIEFQSSBndWFyYW50ZWVzXG5leHBvcnQgY29uc3QgdW5zYWZlID0ge1xuICBleHBhbmRLZXlMRSxcbiAgZXhwYW5kS2V5RGVjTEUsXG4gIGVuY3J5cHQsXG4gIGRlY3J5cHQsXG4gIGVuY3J5cHRCbG9jayxcbiAgZGVjcnlwdEJsb2NrLFxuICBjdHJDb3VudGVyLFxuICBjdHIzMixcbn07XG4iLCAiaW1wb3J0IHsgZXhpc3RzIGFzIGFleGlzdHMsIGJ5dGVzIGFzIGFieXRlcywgb3V0cHV0IGFzIGFvdXRwdXQgfSBmcm9tICcuL19hc3NlcnQuanMnO1xuaW1wb3J0IHsgSW5wdXQsIHRvQnl0ZXMsIEhhc2ggfSBmcm9tICcuL3V0aWxzLmpzJztcblxuLy8gUG9seTEzMDUgaXMgYSBmYXN0IGFuZCBwYXJhbGxlbCBzZWNyZXQta2V5IG1lc3NhZ2UtYXV0aGVudGljYXRpb24gY29kZS5cbi8vIGh0dHBzOi8vY3IueXAudG8vbWFjLmh0bWwsIGh0dHBzOi8vY3IueXAudG8vbWFjL3BvbHkxMzA1LTIwMDUwMzI5LnBkZlxuLy8gaHR0cHM6Ly9kYXRhdHJhY2tlci5pZXRmLm9yZy9kb2MvaHRtbC9yZmM4NDM5XG5cbi8vIEJhc2VkIG9uIFB1YmxpYyBEb21haW4gcG9seTEzMDUtZG9ubmEgaHR0cHM6Ly9naXRodWIuY29tL2Zsb29keWJlcnJ5L3BvbHkxMzA1LWRvbm5hXG5jb25zdCB1OHRvMTYgPSAoYTogVWludDhBcnJheSwgaTogbnVtYmVyKSA9PiAoYVtpKytdICYgMHhmZikgfCAoKGFbaSsrXSAmIDB4ZmYpIDw8IDgpO1xuY2xhc3MgUG9seTEzMDUgaW1wbGVtZW50cyBIYXNoPFBvbHkxMzA1PiB7XG4gIHJlYWRvbmx5IGJsb2NrTGVuID0gMTY7XG4gIHJlYWRvbmx5IG91dHB1dExlbiA9IDE2O1xuICBwcml2YXRlIGJ1ZmZlciA9IG5ldyBVaW50OEFycmF5KDE2KTtcbiAgcHJpdmF0ZSByID0gbmV3IFVpbnQxNkFycmF5KDEwKTtcbiAgcHJpdmF0ZSBoID0gbmV3IFVpbnQxNkFycmF5KDEwKTtcbiAgcHJpdmF0ZSBwYWQgPSBuZXcgVWludDE2QXJyYXkoOCk7XG4gIHByaXZhdGUgcG9zID0gMDtcbiAgcHJvdGVjdGVkIGZpbmlzaGVkID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3Ioa2V5OiBJbnB1dCkge1xuICAgIGtleSA9IHRvQnl0ZXMoa2V5KTtcbiAgICBhYnl0ZXMoa2V5LCAzMik7XG4gICAgY29uc3QgdDAgPSB1OHRvMTYoa2V5LCAwKTtcbiAgICBjb25zdCB0MSA9IHU4dG8xNihrZXksIDIpO1xuICAgIGNvbnN0IHQyID0gdTh0bzE2KGtleSwgNCk7XG4gICAgY29uc3QgdDMgPSB1OHRvMTYoa2V5LCA2KTtcbiAgICBjb25zdCB0NCA9IHU4dG8xNihrZXksIDgpO1xuICAgIGNvbnN0IHQ1ID0gdTh0bzE2KGtleSwgMTApO1xuICAgIGNvbnN0IHQ2ID0gdTh0bzE2KGtleSwgMTIpO1xuICAgIGNvbnN0IHQ3ID0gdTh0bzE2KGtleSwgMTQpO1xuXG4gICAgLy8gaHR0cHM6Ly9naXRodWIuY29tL2Zsb29keWJlcnJ5L3BvbHkxMzA1LWRvbm5hL2Jsb2IvZTZhZDZlMDkxZDMwZDdmNGVjMmQ0Zjk3OGJlMWZjZmNiY2U3Mjc4MS9wb2x5MTMwNS1kb25uYS0xNi5oI0w0N1xuICAgIHRoaXMuclswXSA9IHQwICYgMHgxZmZmO1xuICAgIHRoaXMuclsxXSA9ICgodDAgPj4+IDEzKSB8ICh0MSA8PCAzKSkgJiAweDFmZmY7XG4gICAgdGhpcy5yWzJdID0gKCh0MSA+Pj4gMTApIHwgKHQyIDw8IDYpKSAmIDB4MWYwMztcbiAgICB0aGlzLnJbM10gPSAoKHQyID4+PiA3KSB8ICh0MyA8PCA5KSkgJiAweDFmZmY7XG4gICAgdGhpcy5yWzRdID0gKCh0MyA+Pj4gNCkgfCAodDQgPDwgMTIpKSAmIDB4MDBmZjtcbiAgICB0aGlzLnJbNV0gPSAodDQgPj4+IDEpICYgMHgxZmZlO1xuICAgIHRoaXMucls2XSA9ICgodDQgPj4+IDE0KSB8ICh0NSA8PCAyKSkgJiAweDFmZmY7XG4gICAgdGhpcy5yWzddID0gKCh0NSA+Pj4gMTEpIHwgKHQ2IDw8IDUpKSAmIDB4MWY4MTtcbiAgICB0aGlzLnJbOF0gPSAoKHQ2ID4+PiA4KSB8ICh0NyA8PCA4KSkgJiAweDFmZmY7XG4gICAgdGhpcy5yWzldID0gKHQ3ID4+PiA1KSAmIDB4MDA3ZjtcbiAgICBmb3IgKGxldCBpID0gMDsgaSA8IDg7IGkrKykgdGhpcy5wYWRbaV0gPSB1OHRvMTYoa2V5LCAxNiArIDIgKiBpKTtcbiAgfVxuXG4gIHByaXZhdGUgcHJvY2VzcyhkYXRhOiBVaW50OEFycmF5LCBvZmZzZXQ6IG51bWJlciwgaXNMYXN0ID0gZmFsc2UpIHtcbiAgICBjb25zdCBoaWJpdCA9IGlzTGFzdCA/IDAgOiAxIDw8IDExO1xuICAgIGNvbnN0IHsgaCwgciB9ID0gdGhpcztcbiAgICBjb25zdCByMCA9IHJbMF07XG4gICAgY29uc3QgcjEgPSByWzFdO1xuICAgIGNvbnN0IHIyID0gclsyXTtcbiAgICBjb25zdCByMyA9IHJbM107XG4gICAgY29uc3QgcjQgPSByWzRdO1xuICAgIGNvbnN0IHI1ID0gcls1XTtcbiAgICBjb25zdCByNiA9IHJbNl07XG4gICAgY29uc3QgcjcgPSByWzddO1xuICAgIGNvbnN0IHI4ID0gcls4XTtcbiAgICBjb25zdCByOSA9IHJbOV07XG5cbiAgICBjb25zdCB0MCA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyAwKTtcbiAgICBjb25zdCB0MSA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyAyKTtcbiAgICBjb25zdCB0MiA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyA0KTtcbiAgICBjb25zdCB0MyA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyA2KTtcbiAgICBjb25zdCB0NCA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyA4KTtcbiAgICBjb25zdCB0NSA9IHU4dG8xNihkYXRhLCBvZmZzZXQgKyAxMCk7XG4gICAgY29uc3QgdDYgPSB1OHRvMTYoZGF0YSwgb2Zmc2V0ICsgMTIpO1xuICAgIGNvbnN0IHQ3ID0gdTh0bzE2KGRhdGEsIG9mZnNldCArIDE0KTtcblxuICAgIGxldCBoMCA9IGhbMF0gKyAodDAgJiAweDFmZmYpO1xuICAgIGxldCBoMSA9IGhbMV0gKyAoKCh0MCA+Pj4gMTMpIHwgKHQxIDw8IDMpKSAmIDB4MWZmZik7XG4gICAgbGV0IGgyID0gaFsyXSArICgoKHQxID4+PiAxMCkgfCAodDIgPDwgNikpICYgMHgxZmZmKTtcbiAgICBsZXQgaDMgPSBoWzNdICsgKCgodDIgPj4+IDcpIHwgKHQzIDw8IDkpKSAmIDB4MWZmZik7XG4gICAgbGV0IGg0ID0gaFs0XSArICgoKHQzID4+PiA0KSB8ICh0NCA8PCAxMikpICYgMHgxZmZmKTtcbiAgICBsZXQgaDUgPSBoWzVdICsgKCh0NCA+Pj4gMSkgJiAweDFmZmYpO1xuICAgIGxldCBoNiA9IGhbNl0gKyAoKCh0NCA+Pj4gMTQpIHwgKHQ1IDw8IDIpKSAmIDB4MWZmZik7XG4gICAgbGV0IGg3ID0gaFs3XSArICgoKHQ1ID4+PiAxMSkgfCAodDYgPDwgNSkpICYgMHgxZmZmKTtcbiAgICBsZXQgaDggPSBoWzhdICsgKCgodDYgPj4+IDgpIHwgKHQ3IDw8IDgpKSAmIDB4MWZmZik7XG4gICAgbGV0IGg5ID0gaFs5XSArICgodDcgPj4+IDUpIHwgaGliaXQpO1xuXG4gICAgbGV0IGMgPSAwO1xuXG4gICAgbGV0IGQwID0gYyArIGgwICogcjAgKyBoMSAqICg1ICogcjkpICsgaDIgKiAoNSAqIHI4KSArIGgzICogKDUgKiByNykgKyBoNCAqICg1ICogcjYpO1xuICAgIGMgPSBkMCA+Pj4gMTM7XG4gICAgZDAgJj0gMHgxZmZmO1xuICAgIGQwICs9IGg1ICogKDUgKiByNSkgKyBoNiAqICg1ICogcjQpICsgaDcgKiAoNSAqIHIzKSArIGg4ICogKDUgKiByMikgKyBoOSAqICg1ICogcjEpO1xuICAgIGMgKz0gZDAgPj4+IDEzO1xuICAgIGQwICY9IDB4MWZmZjtcblxuICAgIGxldCBkMSA9IGMgKyBoMCAqIHIxICsgaDEgKiByMCArIGgyICogKDUgKiByOSkgKyBoMyAqICg1ICogcjgpICsgaDQgKiAoNSAqIHI3KTtcbiAgICBjID0gZDEgPj4+IDEzO1xuICAgIGQxICY9IDB4MWZmZjtcbiAgICBkMSArPSBoNSAqICg1ICogcjYpICsgaDYgKiAoNSAqIHI1KSArIGg3ICogKDUgKiByNCkgKyBoOCAqICg1ICogcjMpICsgaDkgKiAoNSAqIHIyKTtcbiAgICBjICs9IGQxID4+PiAxMztcbiAgICBkMSAmPSAweDFmZmY7XG5cbiAgICBsZXQgZDIgPSBjICsgaDAgKiByMiArIGgxICogcjEgKyBoMiAqIHIwICsgaDMgKiAoNSAqIHI5KSArIGg0ICogKDUgKiByOCk7XG4gICAgYyA9IGQyID4+PiAxMztcbiAgICBkMiAmPSAweDFmZmY7XG4gICAgZDIgKz0gaDUgKiAoNSAqIHI3KSArIGg2ICogKDUgKiByNikgKyBoNyAqICg1ICogcjUpICsgaDggKiAoNSAqIHI0KSArIGg5ICogKDUgKiByMyk7XG4gICAgYyArPSBkMiA+Pj4gMTM7XG4gICAgZDIgJj0gMHgxZmZmO1xuXG4gICAgbGV0IGQzID0gYyArIGgwICogcjMgKyBoMSAqIHIyICsgaDIgKiByMSArIGgzICogcjAgKyBoNCAqICg1ICogcjkpO1xuICAgIGMgPSBkMyA+Pj4gMTM7XG4gICAgZDMgJj0gMHgxZmZmO1xuICAgIGQzICs9IGg1ICogKDUgKiByOCkgKyBoNiAqICg1ICogcjcpICsgaDcgKiAoNSAqIHI2KSArIGg4ICogKDUgKiByNSkgKyBoOSAqICg1ICogcjQpO1xuICAgIGMgKz0gZDMgPj4+IDEzO1xuICAgIGQzICY9IDB4MWZmZjtcblxuICAgIGxldCBkNCA9IGMgKyBoMCAqIHI0ICsgaDEgKiByMyArIGgyICogcjIgKyBoMyAqIHIxICsgaDQgKiByMDtcbiAgICBjID0gZDQgPj4+IDEzO1xuICAgIGQ0ICY9IDB4MWZmZjtcbiAgICBkNCArPSBoNSAqICg1ICogcjkpICsgaDYgKiAoNSAqIHI4KSArIGg3ICogKDUgKiByNykgKyBoOCAqICg1ICogcjYpICsgaDkgKiAoNSAqIHI1KTtcbiAgICBjICs9IGQ0ID4+PiAxMztcbiAgICBkNCAmPSAweDFmZmY7XG5cbiAgICBsZXQgZDUgPSBjICsgaDAgKiByNSArIGgxICogcjQgKyBoMiAqIHIzICsgaDMgKiByMiArIGg0ICogcjE7XG4gICAgYyA9IGQ1ID4+PiAxMztcbiAgICBkNSAmPSAweDFmZmY7XG4gICAgZDUgKz0gaDUgKiByMCArIGg2ICogKDUgKiByOSkgKyBoNyAqICg1ICogcjgpICsgaDggKiAoNSAqIHI3KSArIGg5ICogKDUgKiByNik7XG4gICAgYyArPSBkNSA+Pj4gMTM7XG4gICAgZDUgJj0gMHgxZmZmO1xuXG4gICAgbGV0IGQ2ID0gYyArIGgwICogcjYgKyBoMSAqIHI1ICsgaDIgKiByNCArIGgzICogcjMgKyBoNCAqIHIyO1xuICAgIGMgPSBkNiA+Pj4gMTM7XG4gICAgZDYgJj0gMHgxZmZmO1xuICAgIGQ2ICs9IGg1ICogcjEgKyBoNiAqIHIwICsgaDcgKiAoNSAqIHI5KSArIGg4ICogKDUgKiByOCkgKyBoOSAqICg1ICogcjcpO1xuICAgIGMgKz0gZDYgPj4+IDEzO1xuICAgIGQ2ICY9IDB4MWZmZjtcblxuICAgIGxldCBkNyA9IGMgKyBoMCAqIHI3ICsgaDEgKiByNiArIGgyICogcjUgKyBoMyAqIHI0ICsgaDQgKiByMztcbiAgICBjID0gZDcgPj4+IDEzO1xuICAgIGQ3ICY9IDB4MWZmZjtcbiAgICBkNyArPSBoNSAqIHIyICsgaDYgKiByMSArIGg3ICogcjAgKyBoOCAqICg1ICogcjkpICsgaDkgKiAoNSAqIHI4KTtcbiAgICBjICs9IGQ3ID4+PiAxMztcbiAgICBkNyAmPSAweDFmZmY7XG5cbiAgICBsZXQgZDggPSBjICsgaDAgKiByOCArIGgxICogcjcgKyBoMiAqIHI2ICsgaDMgKiByNSArIGg0ICogcjQ7XG4gICAgYyA9IGQ4ID4+PiAxMztcbiAgICBkOCAmPSAweDFmZmY7XG4gICAgZDggKz0gaDUgKiByMyArIGg2ICogcjIgKyBoNyAqIHIxICsgaDggKiByMCArIGg5ICogKDUgKiByOSk7XG4gICAgYyArPSBkOCA+Pj4gMTM7XG4gICAgZDggJj0gMHgxZmZmO1xuXG4gICAgbGV0IGQ5ID0gYyArIGgwICogcjkgKyBoMSAqIHI4ICsgaDIgKiByNyArIGgzICogcjYgKyBoNCAqIHI1O1xuICAgIGMgPSBkOSA+Pj4gMTM7XG4gICAgZDkgJj0gMHgxZmZmO1xuICAgIGQ5ICs9IGg1ICogcjQgKyBoNiAqIHIzICsgaDcgKiByMiArIGg4ICogcjEgKyBoOSAqIHIwO1xuICAgIGMgKz0gZDkgPj4+IDEzO1xuICAgIGQ5ICY9IDB4MWZmZjtcblxuICAgIGMgPSAoKGMgPDwgMikgKyBjKSB8IDA7XG4gICAgYyA9IChjICsgZDApIHwgMDtcbiAgICBkMCA9IGMgJiAweDFmZmY7XG4gICAgYyA9IGMgPj4+IDEzO1xuICAgIGQxICs9IGM7XG5cbiAgICBoWzBdID0gZDA7XG4gICAgaFsxXSA9IGQxO1xuICAgIGhbMl0gPSBkMjtcbiAgICBoWzNdID0gZDM7XG4gICAgaFs0XSA9IGQ0O1xuICAgIGhbNV0gPSBkNTtcbiAgICBoWzZdID0gZDY7XG4gICAgaFs3XSA9IGQ3O1xuICAgIGhbOF0gPSBkODtcbiAgICBoWzldID0gZDk7XG4gIH1cblxuICBwcml2YXRlIGZpbmFsaXplKCkge1xuICAgIGNvbnN0IHsgaCwgcGFkIH0gPSB0aGlzO1xuICAgIGNvbnN0IGcgPSBuZXcgVWludDE2QXJyYXkoMTApO1xuICAgIGxldCBjID0gaFsxXSA+Pj4gMTM7XG4gICAgaFsxXSAmPSAweDFmZmY7XG4gICAgZm9yIChsZXQgaSA9IDI7IGkgPCAxMDsgaSsrKSB7XG4gICAgICBoW2ldICs9IGM7XG4gICAgICBjID0gaFtpXSA+Pj4gMTM7XG4gICAgICBoW2ldICY9IDB4MWZmZjtcbiAgICB9XG4gICAgaFswXSArPSBjICogNTtcbiAgICBjID0gaFswXSA+Pj4gMTM7XG4gICAgaFswXSAmPSAweDFmZmY7XG4gICAgaFsxXSArPSBjO1xuICAgIGMgPSBoWzFdID4+PiAxMztcbiAgICBoWzFdICY9IDB4MWZmZjtcbiAgICBoWzJdICs9IGM7XG5cbiAgICBnWzBdID0gaFswXSArIDU7XG4gICAgYyA9IGdbMF0gPj4+IDEzO1xuICAgIGdbMF0gJj0gMHgxZmZmO1xuICAgIGZvciAobGV0IGkgPSAxOyBpIDwgMTA7IGkrKykge1xuICAgICAgZ1tpXSA9IGhbaV0gKyBjO1xuICAgICAgYyA9IGdbaV0gPj4+IDEzO1xuICAgICAgZ1tpXSAmPSAweDFmZmY7XG4gICAgfVxuICAgIGdbOV0gLT0gMSA8PCAxMztcblxuICAgIGxldCBtYXNrID0gKGMgXiAxKSAtIDE7XG4gICAgZm9yIChsZXQgaSA9IDA7IGkgPCAxMDsgaSsrKSBnW2ldICY9IG1hc2s7XG4gICAgbWFzayA9IH5tYXNrO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgMTA7IGkrKykgaFtpXSA9IChoW2ldICYgbWFzaykgfCBnW2ldO1xuICAgIGhbMF0gPSAoaFswXSB8IChoWzFdIDw8IDEzKSkgJiAweGZmZmY7XG4gICAgaFsxXSA9ICgoaFsxXSA+Pj4gMykgfCAoaFsyXSA8PCAxMCkpICYgMHhmZmZmO1xuICAgIGhbMl0gPSAoKGhbMl0gPj4+IDYpIHwgKGhbM10gPDwgNykpICYgMHhmZmZmO1xuICAgIGhbM10gPSAoKGhbM10gPj4+IDkpIHwgKGhbNF0gPDwgNCkpICYgMHhmZmZmO1xuICAgIGhbNF0gPSAoKGhbNF0gPj4+IDEyKSB8IChoWzVdIDw8IDEpIHwgKGhbNl0gPDwgMTQpKSAmIDB4ZmZmZjtcbiAgICBoWzVdID0gKChoWzZdID4+PiAyKSB8IChoWzddIDw8IDExKSkgJiAweGZmZmY7XG4gICAgaFs2XSA9ICgoaFs3XSA+Pj4gNSkgfCAoaFs4XSA8PCA4KSkgJiAweGZmZmY7XG4gICAgaFs3XSA9ICgoaFs4XSA+Pj4gOCkgfCAoaFs5XSA8PCA1KSkgJiAweGZmZmY7XG5cbiAgICBsZXQgZiA9IGhbMF0gKyBwYWRbMF07XG4gICAgaFswXSA9IGYgJiAweGZmZmY7XG4gICAgZm9yIChsZXQgaSA9IDE7IGkgPCA4OyBpKyspIHtcbiAgICAgIGYgPSAoKChoW2ldICsgcGFkW2ldKSB8IDApICsgKGYgPj4+IDE2KSkgfCAwO1xuICAgICAgaFtpXSA9IGYgJiAweGZmZmY7XG4gICAgfVxuICB9XG4gIHVwZGF0ZShkYXRhOiBJbnB1dCk6IHRoaXMge1xuICAgIGFleGlzdHModGhpcyk7XG4gICAgY29uc3QgeyBidWZmZXIsIGJsb2NrTGVuIH0gPSB0aGlzO1xuICAgIGRhdGEgPSB0b0J5dGVzKGRhdGEpO1xuICAgIGNvbnN0IGxlbiA9IGRhdGEubGVuZ3RoO1xuXG4gICAgZm9yIChsZXQgcG9zID0gMDsgcG9zIDwgbGVuOyApIHtcbiAgICAgIGNvbnN0IHRha2UgPSBNYXRoLm1pbihibG9ja0xlbiAtIHRoaXMucG9zLCBsZW4gLSBwb3MpO1xuICAgICAgLy8gRmFzdCBwYXRoOiB3ZSBoYXZlIGF0IGxlYXN0IG9uZSBibG9jayBpbiBpbnB1dFxuICAgICAgaWYgKHRha2UgPT09IGJsb2NrTGVuKSB7XG4gICAgICAgIGZvciAoOyBibG9ja0xlbiA8PSBsZW4gLSBwb3M7IHBvcyArPSBibG9ja0xlbikgdGhpcy5wcm9jZXNzKGRhdGEsIHBvcyk7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgYnVmZmVyLnNldChkYXRhLnN1YmFycmF5KHBvcywgcG9zICsgdGFrZSksIHRoaXMucG9zKTtcbiAgICAgIHRoaXMucG9zICs9IHRha2U7XG4gICAgICBwb3MgKz0gdGFrZTtcbiAgICAgIGlmICh0aGlzLnBvcyA9PT0gYmxvY2tMZW4pIHtcbiAgICAgICAgdGhpcy5wcm9jZXNzKGJ1ZmZlciwgMCwgZmFsc2UpO1xuICAgICAgICB0aGlzLnBvcyA9IDA7XG4gICAgICB9XG4gICAgfVxuICAgIHJldHVybiB0aGlzO1xuICB9XG4gIGRlc3Ryb3koKSB7XG4gICAgdGhpcy5oLmZpbGwoMCk7XG4gICAgdGhpcy5yLmZpbGwoMCk7XG4gICAgdGhpcy5idWZmZXIuZmlsbCgwKTtcbiAgICB0aGlzLnBhZC5maWxsKDApO1xuICB9XG4gIGRpZ2VzdEludG8ob3V0OiBVaW50OEFycmF5KSB7XG4gICAgYWV4aXN0cyh0aGlzKTtcbiAgICBhb3V0cHV0KG91dCwgdGhpcyk7XG4gICAgdGhpcy5maW5pc2hlZCA9IHRydWU7XG4gICAgY29uc3QgeyBidWZmZXIsIGggfSA9IHRoaXM7XG4gICAgbGV0IHsgcG9zIH0gPSB0aGlzO1xuICAgIGlmIChwb3MpIHtcbiAgICAgIGJ1ZmZlcltwb3MrK10gPSAxO1xuICAgICAgLy8gYnVmZmVyLnN1YmFycmF5KHBvcykuZmlsbCgwKTtcbiAgICAgIGZvciAoOyBwb3MgPCAxNjsgcG9zKyspIGJ1ZmZlcltwb3NdID0gMDtcbiAgICAgIHRoaXMucHJvY2VzcyhidWZmZXIsIDAsIHRydWUpO1xuICAgIH1cbiAgICB0aGlzLmZpbmFsaXplKCk7XG4gICAgbGV0IG9wb3MgPSAwO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgODsgaSsrKSB7XG4gICAgICBvdXRbb3BvcysrXSA9IGhbaV0gPj4+IDA7XG4gICAgICBvdXRbb3BvcysrXSA9IGhbaV0gPj4+IDg7XG4gICAgfVxuICAgIHJldHVybiBvdXQ7XG4gIH1cbiAgZGlnZXN0KCk6IFVpbnQ4QXJyYXkge1xuICAgIGNvbnN0IHsgYnVmZmVyLCBvdXRwdXRMZW4gfSA9IHRoaXM7XG4gICAgdGhpcy5kaWdlc3RJbnRvKGJ1ZmZlcik7XG4gICAgY29uc3QgcmVzID0gYnVmZmVyLnNsaWNlKDAsIG91dHB1dExlbik7XG4gICAgdGhpcy5kZXN0cm95KCk7XG4gICAgcmV0dXJuIHJlcztcbiAgfVxufVxuXG5leHBvcnQgdHlwZSBDSGFzaCA9IFJldHVyblR5cGU8dHlwZW9mIHdyYXBDb25zdHJ1Y3RvcldpdGhLZXk+O1xuZXhwb3J0IGZ1bmN0aW9uIHdyYXBDb25zdHJ1Y3RvcldpdGhLZXk8SCBleHRlbmRzIEhhc2g8SD4+KGhhc2hDb25zOiAoa2V5OiBJbnB1dCkgPT4gSGFzaDxIPikge1xuICBjb25zdCBoYXNoQyA9IChtc2c6IElucHV0LCBrZXk6IElucHV0KTogVWludDhBcnJheSA9PiBoYXNoQ29ucyhrZXkpLnVwZGF0ZSh0b0J5dGVzKG1zZykpLmRpZ2VzdCgpO1xuICBjb25zdCB0bXAgPSBoYXNoQ29ucyhuZXcgVWludDhBcnJheSgzMikpO1xuICBoYXNoQy5vdXRwdXRMZW4gPSB0bXAub3V0cHV0TGVuO1xuICBoYXNoQy5ibG9ja0xlbiA9IHRtcC5ibG9ja0xlbjtcbiAgaGFzaEMuY3JlYXRlID0gKGtleTogSW5wdXQpID0+IGhhc2hDb25zKGtleSk7XG4gIHJldHVybiBoYXNoQztcbn1cblxuZXhwb3J0IGNvbnN0IHBvbHkxMzA1ID0gd3JhcENvbnN0cnVjdG9yV2l0aEtleSgoa2V5KSA9PiBuZXcgUG9seTEzMDUoa2V5KSk7XG4iLCAiLy8gQmFzaWMgdXRpbHMgZm9yIEFSWCAoYWRkLXJvdGF0ZS14b3IpIHNhbHNhIGFuZCBjaGFjaGEgY2lwaGVycy5cbmltcG9ydCB7IG51bWJlciBhcyBhbnVtYmVyLCBieXRlcyBhcyBhYnl0ZXMsIGJvb2wgYXMgYWJvb2wgfSBmcm9tICcuL19hc3NlcnQuanMnO1xuaW1wb3J0IHsgWG9yU3RyZWFtLCBjaGVja09wdHMsIHUzMiB9IGZyb20gJy4vdXRpbHMuanMnO1xuXG4vKlxuUkZDODQzOSByZXF1aXJlcyBtdWx0aS1zdGVwIGNpcGhlciBzdHJlYW0sIHdoZXJlXG5hdXRoS2V5IHN0YXJ0cyB3aXRoIGNvdW50ZXI6IDAsIGFjdHVhbCBtc2cgd2l0aCBjb3VudGVyOiAxLlxuXG5Gb3IgdGhpcywgd2UgbmVlZCBhIHdheSB0byByZS11c2Ugbm9uY2UgLyBjb3VudGVyOlxuXG4gICAgY29uc3QgY291bnRlciA9IG5ldyBVaW50OEFycmF5KDQpO1xuICAgIGNoYWNoYSguLi4sIGNvdW50ZXIsIC4uLik7IC8vIGNvdW50ZXIgaXMgbm93IDFcbiAgICBjaGFjaGEoLi4uLCBjb3VudGVyLCAuLi4pOyAvLyBjb3VudGVyIGlzIG5vdyAyXG5cblRoaXMgaXMgY29tcGxpY2F0ZWQ6XG5cbi0gMzItYml0IGNvdW50ZXJzIGFyZSBlbm91Z2gsIG5vIG5lZWQgZm9yIDY0LWJpdDogbWF4IEFycmF5QnVmZmVyIHNpemUgaW4gSlMgaXMgNEdCXG4tIE9yaWdpbmFsIHBhcGVycyBkb24ndCBhbGxvdyBtdXRhdGluZyBjb3VudGVyc1xuLSBDb3VudGVyIG92ZXJmbG93IGlzIHVuZGVmaW5lZCBbXjFdXG4tIElkZWEgQTogYWxsb3cgcHJvdmlkaW5nIChub25jZSB8IGNvdW50ZXIpIGluc3RlYWQgb2YganVzdCBub25jZSwgcmUtdXNlIGl0XG4tIENhdmVhdDogQ2Fubm90IGJlIHJlLXVzZWQgdGhyb3VnaCBhbGwgY2FzZXM6XG4tICogY2hhY2hhIGhhcyAoY291bnRlciB8IG5vbmNlKVxuLSAqIHhjaGFjaGEgaGFzIChub25jZTE2IHwgY291bnRlciB8IG5vbmNlMTYpXG4tIElkZWEgQjogc2VwYXJhdGUgbm9uY2UgLyBjb3VudGVyIGFuZCBwcm92aWRlIHNlcGFyYXRlIEFQSSBmb3IgY291bnRlciByZS11c2Vcbi0gQ2F2ZWF0OiB0aGVyZSBhcmUgZGlmZmVyZW50IGNvdW50ZXIgc2l6ZXMgZGVwZW5kaW5nIG9uIGFuIGFsZ29yaXRobS5cbi0gc2Fsc2EgJiBjaGFjaGEgYWxzbyBkaWZmZXIgaW4gc3RydWN0dXJlcyBvZiBrZXkgJiBzaWdtYTpcbiAgc2Fsc2EyMDogICAgICBzWzBdIHwgayg0KSB8IHNbMV0gfCBub25jZSgyKSB8IGN0cigyKSB8IHNbMl0gfCBrKDQpIHwgc1szXVxuICBjaGFjaGE6ICAgICAgIHMoNCkgfCBrKDgpIHwgY3RyKDEpIHwgbm9uY2UoMylcbiAgY2hhY2hhMjBvcmlnOiBzKDQpIHwgayg4KSB8IGN0cigyKSB8IG5vbmNlKDIpXG4tIElkZWEgQzogaGVscGVyIG1ldGhvZCBzdWNoIGFzIGBzZXRTYWxzYVN0YXRlKGtleSwgbm9uY2UsIHNpZ21hLCBkYXRhKWBcbi0gQ2F2ZWF0OiB3ZSBjYW4ndCByZS11c2UgY291bnRlciBhcnJheVxuXG54Y2hhY2hhIFteMl0gdXNlcyB0aGUgc3Via2V5IGFuZCByZW1haW5pbmcgOCBieXRlIG5vbmNlIHdpdGggQ2hhQ2hhMjAgYXMgbm9ybWFsXG4ocHJlZml4ZWQgYnkgNCBOVUwgYnl0ZXMsIHNpbmNlIFtSRkM4NDM5XSBzcGVjaWZpZXMgYSAxMi1ieXRlIG5vbmNlKS5cblxuW14xXTogaHR0cHM6Ly9tYWlsYXJjaGl2ZS5pZXRmLm9yZy9hcmNoL21zZy9jZnJnL2dzT25USnpjYmdHNk9xRDhTYzBHTzVhUl90VS9cblteMl06IGh0dHBzOi8vZGF0YXRyYWNrZXIuaWV0Zi5vcmcvZG9jL2h0bWwvZHJhZnQtaXJ0Zi1jZnJnLXhjaGFjaGEjYXBwZW5kaXgtQS4yXG4qL1xuXG4vLyBXZSBjYW4ndCBtYWtlIHRvcC1sZXZlbCB2YXIgZGVwZW5kIG9uIHV0aWxzLnV0ZjhUb0J5dGVzXG4vLyBiZWNhdXNlIGl0J3Mgbm90IHByZXNlbnQgaW4gYWxsIGVudnMuIENyZWF0aW5nIGEgc2ltaWxhciBmbiBoZXJlXG5jb25zdCBfdXRmOFRvQnl0ZXMgPSAoc3RyOiBzdHJpbmcpID0+IFVpbnQ4QXJyYXkuZnJvbShzdHIuc3BsaXQoJycpLm1hcCgoYykgPT4gYy5jaGFyQ29kZUF0KDApKSk7XG5jb25zdCBzaWdtYTE2ID0gX3V0ZjhUb0J5dGVzKCdleHBhbmQgMTYtYnl0ZSBrJyk7XG5jb25zdCBzaWdtYTMyID0gX3V0ZjhUb0J5dGVzKCdleHBhbmQgMzItYnl0ZSBrJyk7XG5jb25zdCBzaWdtYTE2XzMyID0gdTMyKHNpZ21hMTYpO1xuY29uc3Qgc2lnbWEzMl8zMiA9IHUzMihzaWdtYTMyKTtcbmV4cG9ydCBjb25zdCBzaWdtYSA9IHNpZ21hMzJfMzIuc2xpY2UoKTtcblxuZXhwb3J0IGZ1bmN0aW9uIHJvdGwoYTogbnVtYmVyLCBiOiBudW1iZXIpOiBudW1iZXIge1xuICByZXR1cm4gKGEgPDwgYikgfCAoYSA+Pj4gKDMyIC0gYikpO1xufVxuXG5leHBvcnQgdHlwZSBDaXBoZXJDb3JlRm4gPSAoXG4gIHNpZ21hOiBVaW50MzJBcnJheSxcbiAga2V5OiBVaW50MzJBcnJheSxcbiAgbm9uY2U6IFVpbnQzMkFycmF5LFxuICBvdXRwdXQ6IFVpbnQzMkFycmF5LFxuICBjb3VudGVyOiBudW1iZXIsXG4gIHJvdW5kcz86IG51bWJlclxuKSA9PiB2b2lkO1xuXG5leHBvcnQgdHlwZSBFeHRlbmROb25jZUZuID0gKFxuICBzaWdtYTogVWludDMyQXJyYXksXG4gIGtleTogVWludDMyQXJyYXksXG4gIGlucHV0OiBVaW50MzJBcnJheSxcbiAgb3V0cHV0OiBVaW50MzJBcnJheVxuKSA9PiB2b2lkO1xuXG5leHBvcnQgdHlwZSBDaXBoZXJPcHRzID0ge1xuICBhbGxvd1Nob3J0S2V5cz86IGJvb2xlYW47IC8vIE9yaWdpbmFsIHNhbHNhIC8gY2hhY2hhIGFsbG93IDE2LWJ5dGUga2V5c1xuICBleHRlbmROb25jZUZuPzogRXh0ZW5kTm9uY2VGbjtcbiAgY291bnRlckxlbmd0aD86IG51bWJlcjtcbiAgY291bnRlclJpZ2h0PzogYm9vbGVhbjsgLy8gcmlnaHQ6IG5vbmNlfGNvdW50ZXI7IGxlZnQ6IGNvdW50ZXJ8bm9uY2VcbiAgcm91bmRzPzogbnVtYmVyO1xufTtcblxuLy8gSXMgYnl0ZSBhcnJheSBhbGlnbmVkIHRvIDQgYnl0ZSBvZmZzZXQgKHUzMik/XG5mdW5jdGlvbiBpc0FsaWduZWQzMihiOiBVaW50OEFycmF5KSB7XG4gIHJldHVybiBiLmJ5dGVPZmZzZXQgJSA0ID09PSAwO1xufVxuXG4vLyBTYWxzYSBhbmQgQ2hhY2hhIGJsb2NrIGxlbmd0aCBpcyBhbHdheXMgNTEyLWJpdFxuY29uc3QgQkxPQ0tfTEVOID0gNjQ7XG5jb25zdCBCTE9DS19MRU4zMiA9IDE2O1xuXG4vLyBuZXcgVWludDMyQXJyYXkoWzIqKjMyXSkgICAvLyA9PiBVaW50MzJBcnJheSgxKSBbIDAgXVxuLy8gbmV3IFVpbnQzMkFycmF5KFsyKiozMi0xXSkgLy8gPT4gVWludDMyQXJyYXkoMSkgWyA0Mjk0OTY3Mjk1IF1cbmNvbnN0IE1BWF9DT1VOVEVSID0gMiAqKiAzMiAtIDE7XG5cbmNvbnN0IFUzMl9FTVBUWSA9IG5ldyBVaW50MzJBcnJheSgpO1xuZnVuY3Rpb24gcnVuQ2lwaGVyKFxuICBjb3JlOiBDaXBoZXJDb3JlRm4sXG4gIHNpZ21hOiBVaW50MzJBcnJheSxcbiAga2V5OiBVaW50MzJBcnJheSxcbiAgbm9uY2U6IFVpbnQzMkFycmF5LFxuICBkYXRhOiBVaW50OEFycmF5LFxuICBvdXRwdXQ6IFVpbnQ4QXJyYXksXG4gIGNvdW50ZXI6IG51bWJlcixcbiAgcm91bmRzOiBudW1iZXJcbik6IHZvaWQge1xuICBjb25zdCBsZW4gPSBkYXRhLmxlbmd0aDtcbiAgY29uc3QgYmxvY2sgPSBuZXcgVWludDhBcnJheShCTE9DS19MRU4pO1xuICBjb25zdCBiMzIgPSB1MzIoYmxvY2spO1xuICAvLyBNYWtlIHN1cmUgdGhhdCBidWZmZXJzIGFsaWduZWQgdG8gNCBieXRlc1xuICBjb25zdCBpc0FsaWduZWQgPSBpc0FsaWduZWQzMihkYXRhKSAmJiBpc0FsaWduZWQzMihvdXRwdXQpO1xuICBjb25zdCBkMzIgPSBpc0FsaWduZWQgPyB1MzIoZGF0YSkgOiBVMzJfRU1QVFk7XG4gIGNvbnN0IG8zMiA9IGlzQWxpZ25lZCA/IHUzMihvdXRwdXQpIDogVTMyX0VNUFRZO1xuICBmb3IgKGxldCBwb3MgPSAwOyBwb3MgPCBsZW47IGNvdW50ZXIrKykge1xuICAgIGNvcmUoc2lnbWEsIGtleSwgbm9uY2UsIGIzMiwgY291bnRlciwgcm91bmRzKTtcbiAgICBpZiAoY291bnRlciA+PSBNQVhfQ09VTlRFUikgdGhyb3cgbmV3IEVycm9yKCdhcng6IGNvdW50ZXIgb3ZlcmZsb3cnKTtcbiAgICBjb25zdCB0YWtlID0gTWF0aC5taW4oQkxPQ0tfTEVOLCBsZW4gLSBwb3MpO1xuICAgIC8vIGFsaWduZWQgdG8gNCBieXRlc1xuICAgIGlmIChpc0FsaWduZWQgJiYgdGFrZSA9PT0gQkxPQ0tfTEVOKSB7XG4gICAgICBjb25zdCBwb3MzMiA9IHBvcyAvIDQ7XG4gICAgICBpZiAocG9zICUgNCAhPT0gMCkgdGhyb3cgbmV3IEVycm9yKCdhcng6IGludmFsaWQgYmxvY2sgcG9zaXRpb24nKTtcbiAgICAgIGZvciAobGV0IGogPSAwLCBwb3NqOiBudW1iZXI7IGogPCBCTE9DS19MRU4zMjsgaisrKSB7XG4gICAgICAgIHBvc2ogPSBwb3MzMiArIGo7XG4gICAgICAgIG8zMltwb3NqXSA9IGQzMltwb3NqXSBeIGIzMltqXTtcbiAgICAgIH1cbiAgICAgIHBvcyArPSBCTE9DS19MRU47XG4gICAgICBjb250aW51ZTtcbiAgICB9XG4gICAgZm9yIChsZXQgaiA9IDAsIHBvc2o7IGogPCB0YWtlOyBqKyspIHtcbiAgICAgIHBvc2ogPSBwb3MgKyBqO1xuICAgICAgb3V0cHV0W3Bvc2pdID0gZGF0YVtwb3NqXSBeIGJsb2NrW2pdO1xuICAgIH1cbiAgICBwb3MgKz0gdGFrZTtcbiAgfVxufVxuXG5leHBvcnQgZnVuY3Rpb24gY3JlYXRlQ2lwaGVyKGNvcmU6IENpcGhlckNvcmVGbiwgb3B0czogQ2lwaGVyT3B0cyk6IFhvclN0cmVhbSB7XG4gIGNvbnN0IHsgYWxsb3dTaG9ydEtleXMsIGV4dGVuZE5vbmNlRm4sIGNvdW50ZXJMZW5ndGgsIGNvdW50ZXJSaWdodCwgcm91bmRzIH0gPSBjaGVja09wdHMoXG4gICAgeyBhbGxvd1Nob3J0S2V5czogZmFsc2UsIGNvdW50ZXJMZW5ndGg6IDgsIGNvdW50ZXJSaWdodDogZmFsc2UsIHJvdW5kczogMjAgfSxcbiAgICBvcHRzXG4gICk7XG4gIGlmICh0eXBlb2YgY29yZSAhPT0gJ2Z1bmN0aW9uJykgdGhyb3cgbmV3IEVycm9yKCdjb3JlIG11c3QgYmUgYSBmdW5jdGlvbicpO1xuICBhbnVtYmVyKGNvdW50ZXJMZW5ndGgpO1xuICBhbnVtYmVyKHJvdW5kcyk7XG4gIGFib29sKGNvdW50ZXJSaWdodCk7XG4gIGFib29sKGFsbG93U2hvcnRLZXlzKTtcbiAgcmV0dXJuIChcbiAgICBrZXk6IFVpbnQ4QXJyYXksXG4gICAgbm9uY2U6IFVpbnQ4QXJyYXksXG4gICAgZGF0YTogVWludDhBcnJheSxcbiAgICBvdXRwdXQ/OiBVaW50OEFycmF5LFxuICAgIGNvdW50ZXIgPSAwXG4gICk6IFVpbnQ4QXJyYXkgPT4ge1xuICAgIGFieXRlcyhrZXkpO1xuICAgIGFieXRlcyhub25jZSk7XG4gICAgYWJ5dGVzKGRhdGEpO1xuICAgIGNvbnN0IGxlbiA9IGRhdGEubGVuZ3RoO1xuICAgIGlmICghb3V0cHV0KSBvdXRwdXQgPSBuZXcgVWludDhBcnJheShsZW4pO1xuICAgIGFieXRlcyhvdXRwdXQpO1xuICAgIGFudW1iZXIoY291bnRlcik7XG4gICAgaWYgKGNvdW50ZXIgPCAwIHx8IGNvdW50ZXIgPj0gTUFYX0NPVU5URVIpIHRocm93IG5ldyBFcnJvcignYXJ4OiBjb3VudGVyIG92ZXJmbG93Jyk7XG4gICAgaWYgKG91dHB1dC5sZW5ndGggPCBsZW4pXG4gICAgICB0aHJvdyBuZXcgRXJyb3IoYGFyeDogb3V0cHV0ICgke291dHB1dC5sZW5ndGh9KSBpcyBzaG9ydGVyIHRoYW4gZGF0YSAoJHtsZW59KWApO1xuICAgIGNvbnN0IHRvQ2xlYW4gPSBbXTtcblxuICAgIC8vIEtleSAmIHNpZ21hXG4gICAgLy8ga2V5PTE2IC0+IHNpZ21hMTYsIGs9a2V5fGtleVxuICAgIC8vIGtleT0zMiAtPiBzaWdtYTMyLCBrPWtleVxuICAgIGxldCBsID0ga2V5Lmxlbmd0aCxcbiAgICAgIGs6IFVpbnQ4QXJyYXksXG4gICAgICBzaWdtYTogVWludDMyQXJyYXk7XG4gICAgaWYgKGwgPT09IDMyKSB7XG4gICAgICBrID0ga2V5LnNsaWNlKCk7XG4gICAgICB0b0NsZWFuLnB1c2goayk7XG4gICAgICBzaWdtYSA9IHNpZ21hMzJfMzI7XG4gICAgfSBlbHNlIGlmIChsID09PSAxNiAmJiBhbGxvd1Nob3J0S2V5cykge1xuICAgICAgayA9IG5ldyBVaW50OEFycmF5KDMyKTtcbiAgICAgIGsuc2V0KGtleSk7XG4gICAgICBrLnNldChrZXksIDE2KTtcbiAgICAgIHNpZ21hID0gc2lnbWExNl8zMjtcbiAgICAgIHRvQ2xlYW4ucHVzaChrKTtcbiAgICB9IGVsc2Uge1xuICAgICAgdGhyb3cgbmV3IEVycm9yKGBhcng6IGludmFsaWQgMzItYnl0ZSBrZXksIGdvdCBsZW5ndGg9JHtsfWApO1xuICAgIH1cblxuICAgIC8vIE5vbmNlXG4gICAgLy8gc2Fsc2EyMDogICAgICA4ICAgKDgtYnl0ZSBjb3VudGVyKVxuICAgIC8vIGNoYWNoYTIwb3JpZzogOCAgICg4LWJ5dGUgY291bnRlcilcbiAgICAvLyBjaGFjaGEyMDogICAgIDEyICAoNC1ieXRlIGNvdW50ZXIpXG4gICAgLy8geHNhbHNhMjA6ICAgICAyNCAgKDE2IC0+IGhzYWxzYSwgIDggLT4gb2xkIG5vbmNlKVxuICAgIC8vIHhjaGFjaGEyMDogICAgMjQgICgxNiAtPiBoY2hhY2hhLCA4IC0+IG9sZCBub25jZSlcbiAgICAvLyBBbGlnbiBub25jZSB0byA0IGJ5dGVzXG4gICAgaWYgKCFpc0FsaWduZWQzMihub25jZSkpIHtcbiAgICAgIG5vbmNlID0gbm9uY2Uuc2xpY2UoKTtcbiAgICAgIHRvQ2xlYW4ucHVzaChub25jZSk7XG4gICAgfVxuXG4gICAgY29uc3QgazMyID0gdTMyKGspO1xuICAgIC8vIGhzYWxzYSAmIGhjaGFjaGE6IGhhbmRsZSBleHRlbmRlZCBub25jZVxuICAgIGlmIChleHRlbmROb25jZUZuKSB7XG4gICAgICBpZiAobm9uY2UubGVuZ3RoICE9PSAyNCkgdGhyb3cgbmV3IEVycm9yKGBhcng6IGV4dGVuZGVkIG5vbmNlIG11c3QgYmUgMjQgYnl0ZXNgKTtcbiAgICAgIGV4dGVuZE5vbmNlRm4oc2lnbWEsIGszMiwgdTMyKG5vbmNlLnN1YmFycmF5KDAsIDE2KSksIGszMik7XG4gICAgICBub25jZSA9IG5vbmNlLnN1YmFycmF5KDE2KTtcbiAgICB9XG5cbiAgICAvLyBIYW5kbGUgbm9uY2UgY291bnRlclxuICAgIGNvbnN0IG5vbmNlTmNMZW4gPSAxNiAtIGNvdW50ZXJMZW5ndGg7XG4gICAgaWYgKG5vbmNlTmNMZW4gIT09IG5vbmNlLmxlbmd0aClcbiAgICAgIHRocm93IG5ldyBFcnJvcihgYXJ4OiBub25jZSBtdXN0IGJlICR7bm9uY2VOY0xlbn0gb3IgMTYgYnl0ZXNgKTtcblxuICAgIC8vIFBhZCBjb3VudGVyIHdoZW4gbm9uY2UgaXMgNjQgYml0XG4gICAgaWYgKG5vbmNlTmNMZW4gIT09IDEyKSB7XG4gICAgICBjb25zdCBuYyA9IG5ldyBVaW50OEFycmF5KDEyKTtcbiAgICAgIG5jLnNldChub25jZSwgY291bnRlclJpZ2h0ID8gMCA6IDEyIC0gbm9uY2UubGVuZ3RoKTtcbiAgICAgIG5vbmNlID0gbmM7XG4gICAgICB0b0NsZWFuLnB1c2gobm9uY2UpO1xuICAgIH1cbiAgICBjb25zdCBuMzIgPSB1MzIobm9uY2UpO1xuICAgIHJ1bkNpcGhlcihjb3JlLCBzaWdtYSwgazMyLCBuMzIsIGRhdGEsIG91dHB1dCwgY291bnRlciwgcm91bmRzKTtcbiAgICB3aGlsZSAodG9DbGVhbi5sZW5ndGggPiAwKSB0b0NsZWFuLnBvcCgpIS5maWxsKDApO1xuICAgIHJldHVybiBvdXRwdXQ7XG4gIH07XG59XG4iLCAiLy8gcHJldHRpZXItaWdub3JlXG5pbXBvcnQge1xuICB3cmFwQ2lwaGVyLCBDaXBoZXJXaXRoT3V0cHV0LCBYb3JTdHJlYW0sIGNyZWF0ZVZpZXcsIGVxdWFsQnl0ZXMsIHNldEJpZ1VpbnQ2NCxcbn0gZnJvbSAnLi91dGlscy5qcyc7XG5pbXBvcnQgeyBwb2x5MTMwNSB9IGZyb20gJy4vX3BvbHkxMzA1LmpzJztcbmltcG9ydCB7IGNyZWF0ZUNpcGhlciwgcm90bCB9IGZyb20gJy4vX2FyeC5qcyc7XG5pbXBvcnQgeyBieXRlcyBhcyBhYnl0ZXMgfSBmcm9tICcuL19hc3NlcnQuanMnO1xuXG4vLyBDaGFDaGEyMCBzdHJlYW0gY2lwaGVyIHdhcyByZWxlYXNlZCBpbiAyMDA4LiBDaGFDaGEgYWltcyB0byBpbmNyZWFzZVxuLy8gdGhlIGRpZmZ1c2lvbiBwZXIgcm91bmQsIGJ1dCBoYWQgc2xpZ2h0bHkgbGVzcyBjcnlwdGFuYWx5c2lzLlxuLy8gaHR0cHM6Ly9jci55cC50by9jaGFjaGEuaHRtbCwgaHR0cDovL2NyLnlwLnRvL2NoYWNoYS9jaGFjaGEtMjAwODAxMjgucGRmXG5cbi8qKlxuICogQ2hhQ2hhIGNvcmUgZnVuY3Rpb24uXG4gKi9cbi8vIHByZXR0aWVyLWlnbm9yZVxuZnVuY3Rpb24gY2hhY2hhQ29yZShcbiAgczogVWludDMyQXJyYXksIGs6IFVpbnQzMkFycmF5LCBuOiBVaW50MzJBcnJheSwgb3V0OiBVaW50MzJBcnJheSwgY250OiBudW1iZXIsIHJvdW5kcyA9IDIwXG4pOiB2b2lkIHtcbiAgbGV0IHkwMCA9IHNbMF0sIHkwMSA9IHNbMV0sIHkwMiA9IHNbMl0sIHkwMyA9IHNbM10sIC8vIFwiZXhwYVwiICAgXCJuZCAzXCIgIFwiMi1ieVwiICBcInRlIGtcIlxuICAgICAgeTA0ID0ga1swXSwgeTA1ID0ga1sxXSwgeTA2ID0ga1syXSwgeTA3ID0ga1szXSwgLy8gS2V5ICAgICAgS2V5ICAgICBLZXkgICAgIEtleVxuICAgICAgeTA4ID0ga1s0XSwgeTA5ID0ga1s1XSwgeTEwID0ga1s2XSwgeTExID0ga1s3XSwgLy8gS2V5ICAgICAgS2V5ICAgICBLZXkgICAgIEtleVxuICAgICAgeTEyID0gY250LCAgeTEzID0gblswXSwgeTE0ID0gblsxXSwgeTE1ID0gblsyXTsgLy8gQ291bnRlciAgQ291bnRlclx0Tm9uY2UgICBOb25jZVxuICAvLyBTYXZlIHN0YXRlIHRvIHRlbXBvcmFyeSB2YXJpYWJsZXNcbiAgbGV0IHgwMCA9IHkwMCwgeDAxID0geTAxLCB4MDIgPSB5MDIsIHgwMyA9IHkwMyxcbiAgICAgIHgwNCA9IHkwNCwgeDA1ID0geTA1LCB4MDYgPSB5MDYsIHgwNyA9IHkwNyxcbiAgICAgIHgwOCA9IHkwOCwgeDA5ID0geTA5LCB4MTAgPSB5MTAsIHgxMSA9IHkxMSxcbiAgICAgIHgxMiA9IHkxMiwgeDEzID0geTEzLCB4MTQgPSB5MTQsIHgxNSA9IHkxNTtcbiAgZm9yIChsZXQgciA9IDA7IHIgPCByb3VuZHM7IHIgKz0gMikge1xuICAgIHgwMCA9ICh4MDAgKyB4MDQpIHwgMDsgeDEyID0gcm90bCh4MTIgXiB4MDAsIDE2KTtcbiAgICB4MDggPSAoeDA4ICsgeDEyKSB8IDA7IHgwNCA9IHJvdGwoeDA0IF4geDA4LCAxMik7XG4gICAgeDAwID0gKHgwMCArIHgwNCkgfCAwOyB4MTIgPSByb3RsKHgxMiBeIHgwMCwgOCk7XG4gICAgeDA4ID0gKHgwOCArIHgxMikgfCAwOyB4MDQgPSByb3RsKHgwNCBeIHgwOCwgNyk7XG5cbiAgICB4MDEgPSAoeDAxICsgeDA1KSB8IDA7IHgxMyA9IHJvdGwoeDEzIF4geDAxLCAxNik7XG4gICAgeDA5ID0gKHgwOSArIHgxMykgfCAwOyB4MDUgPSByb3RsKHgwNSBeIHgwOSwgMTIpO1xuICAgIHgwMSA9ICh4MDEgKyB4MDUpIHwgMDsgeDEzID0gcm90bCh4MTMgXiB4MDEsIDgpO1xuICAgIHgwOSA9ICh4MDkgKyB4MTMpIHwgMDsgeDA1ID0gcm90bCh4MDUgXiB4MDksIDcpO1xuXG4gICAgeDAyID0gKHgwMiArIHgwNikgfCAwOyB4MTQgPSByb3RsKHgxNCBeIHgwMiwgMTYpO1xuICAgIHgxMCA9ICh4MTAgKyB4MTQpIHwgMDsgeDA2ID0gcm90bCh4MDYgXiB4MTAsIDEyKTtcbiAgICB4MDIgPSAoeDAyICsgeDA2KSB8IDA7IHgxNCA9IHJvdGwoeDE0IF54MDIsIDgpO1xuICAgIHgxMCA9ICh4MTAgKyB4MTQpIHwgMDsgeDA2ID0gcm90bCh4MDYgXiB4MTAsIDcpO1xuXG4gICAgeDAzID0gKHgwMyArIHgwNykgfCAwOyB4MTUgPSByb3RsKHgxNSBeIHgwMywgMTYpO1xuICAgIHgxMSA9ICh4MTEgKyB4MTUpIHwgMDsgeDA3ID0gcm90bCh4MDcgXiB4MTEsIDEyKTtcbiAgICB4MDMgPSAoeDAzICsgeDA3KSB8IDA7IHgxNSA9IHJvdGwoeDE1IF4geDAzLCA4KVxuICAgIHgxMSA9ICh4MTEgKyB4MTUpIHwgMDsgeDA3ID0gcm90bCh4MDcgXiB4MTEsIDcpO1xuXG4gICAgeDAwID0gKHgwMCArIHgwNSkgfCAwOyB4MTUgPSByb3RsKHgxNSBeIHgwMCwgMTYpO1xuICAgIHgxMCA9ICh4MTAgKyB4MTUpIHwgMDsgeDA1ID0gcm90bCh4MDUgXiB4MTAsIDEyKTtcbiAgICB4MDAgPSAoeDAwICsgeDA1KSB8IDA7IHgxNSA9IHJvdGwoeDE1IF4geDAwLCA4KTtcbiAgICB4MTAgPSAoeDEwICsgeDE1KSB8IDA7IHgwNSA9IHJvdGwoeDA1IF4geDEwLCA3KTtcblxuICAgIHgwMSA9ICh4MDEgKyB4MDYpIHwgMDsgeDEyID0gcm90bCh4MTIgXiB4MDEsIDE2KTtcbiAgICB4MTEgPSAoeDExICsgeDEyKSB8IDA7IHgwNiA9IHJvdGwoeDA2IF4geDExLCAxMik7XG4gICAgeDAxID0gKHgwMSArIHgwNikgfCAwOyB4MTIgPSByb3RsKHgxMiBeIHgwMSwgOCk7XG4gICAgeDExID0gKHgxMSArIHgxMikgfCAwOyB4MDYgPSByb3RsKHgwNiBeIHgxMSwgNyk7XG5cbiAgICB4MDIgPSAoeDAyICsgeDA3KSB8IDA7IHgxMyA9IHJvdGwoeDEzIF4geDAyLCAxNik7XG4gICAgeDA4ID0gKHgwOCArIHgxMykgfCAwOyB4MDcgPSByb3RsKHgwNyBeIHgwOCwgMTIpO1xuICAgIHgwMiA9ICh4MDIgKyB4MDcpIHwgMDsgeDEzID0gcm90bCh4MTMgXiB4MDIsIDgpO1xuICAgIHgwOCA9ICh4MDggKyB4MTMpIHwgMDsgeDA3ID0gcm90bCh4MDcgXiB4MDgsIDcpO1xuXG4gICAgeDAzID0gKHgwMyArIHgwNCkgfCAwOyB4MTQgPSByb3RsKHgxNCBeIHgwMywgMTYpXG4gICAgeDA5ID0gKHgwOSArIHgxNCkgfCAwOyB4MDQgPSByb3RsKHgwNCBeIHgwOSwgMTIpO1xuICAgIHgwMyA9ICh4MDMgKyB4MDQpIHwgMDsgeDE0ID0gcm90bCh4MTQgXiB4MDMsIDgpO1xuICAgIHgwOSA9ICh4MDkgKyB4MTQpIHwgMDsgeDA0ID0gcm90bCh4MDQgXiB4MDksIDcpO1xuICB9XG4gIC8vIFdyaXRlIG91dHB1dFxuICBsZXQgb2kgPSAwO1xuICBvdXRbb2krK10gPSAoeTAwICsgeDAwKSB8IDA7IG91dFtvaSsrXSA9ICh5MDEgKyB4MDEpIHwgMDtcbiAgb3V0W29pKytdID0gKHkwMiArIHgwMikgfCAwOyBvdXRbb2krK10gPSAoeTAzICsgeDAzKSB8IDA7XG4gIG91dFtvaSsrXSA9ICh5MDQgKyB4MDQpIHwgMDsgb3V0W29pKytdID0gKHkwNSArIHgwNSkgfCAwO1xuICBvdXRbb2krK10gPSAoeTA2ICsgeDA2KSB8IDA7IG91dFtvaSsrXSA9ICh5MDcgKyB4MDcpIHwgMDtcbiAgb3V0W29pKytdID0gKHkwOCArIHgwOCkgfCAwOyBvdXRbb2krK10gPSAoeTA5ICsgeDA5KSB8IDA7XG4gIG91dFtvaSsrXSA9ICh5MTAgKyB4MTApIHwgMDsgb3V0W29pKytdID0gKHkxMSArIHgxMSkgfCAwO1xuICBvdXRbb2krK10gPSAoeTEyICsgeDEyKSB8IDA7IG91dFtvaSsrXSA9ICh5MTMgKyB4MTMpIHwgMDtcbiAgb3V0W29pKytdID0gKHkxNCArIHgxNCkgfCAwOyBvdXRbb2krK10gPSAoeTE1ICsgeDE1KSB8IDA7XG59XG4vKipcbiAqIGhjaGFjaGEgaGVscGVyIG1ldGhvZCwgdXNlZCBwcmltYXJpbHkgaW4geGNoYWNoYSwgdG8gaGFzaFxuICoga2V5IGFuZCBub25jZSBpbnRvIGtleScgYW5kIG5vbmNlJy5cbiAqIFNhbWUgYXMgY2hhY2hhQ29yZSwgYnV0IHRoZXJlIGRvZXNuJ3Qgc2VlbSB0byBiZSBhIHdheSB0byBtb3ZlIHRoZSBibG9ja1xuICogb3V0IHdpdGhvdXQgMjUlIHBlcmZvcm1hbmNlIGhpdC5cbiAqL1xuLy8gcHJldHRpZXItaWdub3JlXG5leHBvcnQgZnVuY3Rpb24gaGNoYWNoYShcbiAgczogVWludDMyQXJyYXksIGs6IFVpbnQzMkFycmF5LCBpOiBVaW50MzJBcnJheSwgbzMyOiBVaW50MzJBcnJheVxuKSB7XG4gIGxldCB4MDAgPSBzWzBdLCB4MDEgPSBzWzFdLCB4MDIgPSBzWzJdLCB4MDMgPSBzWzNdLFxuICAgICAgeDA0ID0ga1swXSwgeDA1ID0ga1sxXSwgeDA2ID0ga1syXSwgeDA3ID0ga1szXSxcbiAgICAgIHgwOCA9IGtbNF0sIHgwOSA9IGtbNV0sIHgxMCA9IGtbNl0sIHgxMSA9IGtbN10sXG4gICAgICB4MTIgPSBpWzBdLCB4MTMgPSBpWzFdLCB4MTQgPSBpWzJdLCB4MTUgPSBpWzNdO1xuICBmb3IgKGxldCByID0gMDsgciA8IDIwOyByICs9IDIpIHtcbiAgICB4MDAgPSAoeDAwICsgeDA0KSB8IDA7IHgxMiA9IHJvdGwoeDEyIF4geDAwLCAxNik7XG4gICAgeDA4ID0gKHgwOCArIHgxMikgfCAwOyB4MDQgPSByb3RsKHgwNCBeIHgwOCwgMTIpO1xuICAgIHgwMCA9ICh4MDAgKyB4MDQpIHwgMDsgeDEyID0gcm90bCh4MTIgXiB4MDAsIDgpO1xuICAgIHgwOCA9ICh4MDggKyB4MTIpIHwgMDsgeDA0ID0gcm90bCh4MDQgXiB4MDgsIDcpO1xuXG4gICAgeDAxID0gKHgwMSArIHgwNSkgfCAwOyB4MTMgPSByb3RsKHgxMyBeIHgwMSwgMTYpO1xuICAgIHgwOSA9ICh4MDkgKyB4MTMpIHwgMDsgeDA1ID0gcm90bCh4MDUgXiB4MDksIDEyKTtcbiAgICB4MDEgPSAoeDAxICsgeDA1KSB8IDA7IHgxMyA9IHJvdGwoeDEzIF4geDAxLCA4KTtcbiAgICB4MDkgPSAoeDA5ICsgeDEzKSB8IDA7IHgwNSA9IHJvdGwoeDA1IF4geDA5LCA3KTtcblxuICAgIHgwMiA9ICh4MDIgKyB4MDYpIHwgMDsgeDE0ID0gcm90bCh4MTQgXiB4MDIsIDE2KTtcbiAgICB4MTAgPSAoeDEwICsgeDE0KSB8IDA7IHgwNiA9IHJvdGwoeDA2IF4geDEwLCAxMik7XG4gICAgeDAyID0gKHgwMiArIHgwNikgfCAwOyB4MTQgPSByb3RsKHgxNCBeIHgwMiwgOCk7XG4gICAgeDEwID0gKHgxMCArIHgxNCkgfCAwOyB4MDYgPSByb3RsKHgwNiBeIHgxMCwgNyk7XG5cbiAgICB4MDMgPSAoeDAzICsgeDA3KSB8IDA7IHgxNSA9IHJvdGwoeDE1IF4geDAzLCAxNik7XG4gICAgeDExID0gKHgxMSArIHgxNSkgfCAwOyB4MDcgPSByb3RsKHgwNyBeIHgxMSwgMTIpO1xuICAgIHgwMyA9ICh4MDMgKyB4MDcpIHwgMDsgeDE1ID0gcm90bCh4MTUgXiB4MDMsIDgpXG4gICAgeDExID0gKHgxMSArIHgxNSkgfCAwOyB4MDcgPSByb3RsKHgwNyBeIHgxMSwgNyk7XG5cbiAgICB4MDAgPSAoeDAwICsgeDA1KSB8IDA7IHgxNSA9IHJvdGwoeDE1IF4geDAwLCAxNik7XG4gICAgeDEwID0gKHgxMCArIHgxNSkgfCAwOyB4MDUgPSByb3RsKHgwNSBeIHgxMCwgMTIpO1xuICAgIHgwMCA9ICh4MDAgKyB4MDUpIHwgMDsgeDE1ID0gcm90bCh4MTUgXiB4MDAsIDgpO1xuICAgIHgxMCA9ICh4MTAgKyB4MTUpIHwgMDsgeDA1ID0gcm90bCh4MDUgXiB4MTAsIDcpO1xuXG4gICAgeDAxID0gKHgwMSArIHgwNikgfCAwOyB4MTIgPSByb3RsKHgxMiBeIHgwMSwgMTYpO1xuICAgIHgxMSA9ICh4MTEgKyB4MTIpIHwgMDsgeDA2ID0gcm90bCh4MDYgXiB4MTEsIDEyKTtcbiAgICB4MDEgPSAoeDAxICsgeDA2KSB8IDA7IHgxMiA9IHJvdGwoeDEyIF4geDAxLCA4KTtcbiAgICB4MTEgPSAoeDExICsgeDEyKSB8IDA7IHgwNiA9IHJvdGwoeDA2IF4geDExLCA3KTtcblxuICAgIHgwMiA9ICh4MDIgKyB4MDcpIHwgMDsgeDEzID0gcm90bCh4MTMgXiB4MDIsIDE2KTtcbiAgICB4MDggPSAoeDA4ICsgeDEzKSB8IDA7IHgwNyA9IHJvdGwoeDA3IF4geDA4LCAxMik7XG4gICAgeDAyID0gKHgwMiArIHgwNykgfCAwOyB4MTMgPSByb3RsKHgxMyBeIHgwMiwgOCk7XG4gICAgeDA4ID0gKHgwOCArIHgxMykgfCAwOyB4MDcgPSByb3RsKHgwNyBeIHgwOCwgNyk7XG5cbiAgICB4MDMgPSAoeDAzICsgeDA0KSB8IDA7IHgxNCA9IHJvdGwoeDE0IF4geDAzLCAxNilcbiAgICB4MDkgPSAoeDA5ICsgeDE0KSB8IDA7IHgwNCA9IHJvdGwoeDA0IF4geDA5LCAxMik7XG4gICAgeDAzID0gKHgwMyArIHgwNCkgfCAwOyB4MTQgPSByb3RsKHgxNCBeIHgwMywgOCk7XG4gICAgeDA5ID0gKHgwOSArIHgxNCkgfCAwOyB4MDQgPSByb3RsKHgwNCBeIHgwOSwgNyk7XG4gIH1cbiAgbGV0IG9pID0gMDtcbiAgbzMyW29pKytdID0geDAwOyBvMzJbb2krK10gPSB4MDE7XG4gIG8zMltvaSsrXSA9IHgwMjsgbzMyW29pKytdID0geDAzO1xuICBvMzJbb2krK10gPSB4MTI7IG8zMltvaSsrXSA9IHgxMztcbiAgbzMyW29pKytdID0geDE0OyBvMzJbb2krK10gPSB4MTU7XG59XG4vKipcbiAqIE9yaWdpbmFsLCBub24tUkZDIGNoYWNoYTIwIGZyb20gREpCLiA4LWJ5dGUgbm9uY2UsIDgtYnl0ZSBjb3VudGVyLlxuICovXG5leHBvcnQgY29uc3QgY2hhY2hhMjBvcmlnID0gLyogQF9fUFVSRV9fICovIGNyZWF0ZUNpcGhlcihjaGFjaGFDb3JlLCB7XG4gIGNvdW50ZXJSaWdodDogZmFsc2UsXG4gIGNvdW50ZXJMZW5ndGg6IDgsXG4gIGFsbG93U2hvcnRLZXlzOiB0cnVlLFxufSk7XG4vKipcbiAqIENoYUNoYSBzdHJlYW0gY2lwaGVyLiBDb25mb3JtcyB0byBSRkMgODQzOSAoSUVURiwgVExTKS4gMTItYnl0ZSBub25jZSwgNC1ieXRlIGNvdW50ZXIuXG4gKiBXaXRoIDEyLWJ5dGUgbm9uY2UsIGl0J3Mgbm90IHNhZmUgdG8gdXNlIGZpbGwgaXQgd2l0aCByYW5kb20gKENTUFJORyksIGR1ZSB0byBjb2xsaXNpb24gY2hhbmNlLlxuICovXG5leHBvcnQgY29uc3QgY2hhY2hhMjAgPSAvKiBAX19QVVJFX18gKi8gY3JlYXRlQ2lwaGVyKGNoYWNoYUNvcmUsIHtcbiAgY291bnRlclJpZ2h0OiBmYWxzZSxcbiAgY291bnRlckxlbmd0aDogNCxcbiAgYWxsb3dTaG9ydEtleXM6IGZhbHNlLFxufSk7XG5cbi8qKlxuICogWENoYUNoYSBlWHRlbmRlZC1ub25jZSBDaGFDaGEuIDI0LWJ5dGUgbm9uY2UuXG4gKiBXaXRoIDI0LWJ5dGUgbm9uY2UsIGl0J3Mgc2FmZSB0byB1c2UgZmlsbCBpdCB3aXRoIHJhbmRvbSAoQ1NQUk5HKS5cbiAqIGh0dHBzOi8vZGF0YXRyYWNrZXIuaWV0Zi5vcmcvZG9jL2h0bWwvZHJhZnQtaXJ0Zi1jZnJnLXhjaGFjaGFcbiAqL1xuZXhwb3J0IGNvbnN0IHhjaGFjaGEyMCA9IC8qIEBfX1BVUkVfXyAqLyBjcmVhdGVDaXBoZXIoY2hhY2hhQ29yZSwge1xuICBjb3VudGVyUmlnaHQ6IGZhbHNlLFxuICBjb3VudGVyTGVuZ3RoOiA4LFxuICBleHRlbmROb25jZUZuOiBoY2hhY2hhLFxuICBhbGxvd1Nob3J0S2V5czogZmFsc2UsXG59KTtcblxuLyoqXG4gKiBSZWR1Y2VkIDgtcm91bmQgY2hhY2hhLCBkZXNjcmliZWQgaW4gb3JpZ2luYWwgcGFwZXIuXG4gKi9cbmV4cG9ydCBjb25zdCBjaGFjaGE4ID0gLyogQF9fUFVSRV9fICovIGNyZWF0ZUNpcGhlcihjaGFjaGFDb3JlLCB7XG4gIGNvdW50ZXJSaWdodDogZmFsc2UsXG4gIGNvdW50ZXJMZW5ndGg6IDQsXG4gIHJvdW5kczogOCxcbn0pO1xuXG4vKipcbiAqIFJlZHVjZWQgMTItcm91bmQgY2hhY2hhLCBkZXNjcmliZWQgaW4gb3JpZ2luYWwgcGFwZXIuXG4gKi9cbmV4cG9ydCBjb25zdCBjaGFjaGExMiA9IC8qIEBfX1BVUkVfXyAqLyBjcmVhdGVDaXBoZXIoY2hhY2hhQ29yZSwge1xuICBjb3VudGVyUmlnaHQ6IGZhbHNlLFxuICBjb3VudGVyTGVuZ3RoOiA0LFxuICByb3VuZHM6IDEyLFxufSk7XG5cbmNvbnN0IFpFUk9TMTYgPSAvKiBAX19QVVJFX18gKi8gbmV3IFVpbnQ4QXJyYXkoMTYpO1xuLy8gUGFkIHRvIGRpZ2VzdCBzaXplIHdpdGggemVyb3NcbmNvbnN0IHVwZGF0ZVBhZGRlZCA9IChoOiBSZXR1cm5UeXBlPHR5cGVvZiBwb2x5MTMwNS5jcmVhdGU+LCBtc2c6IFVpbnQ4QXJyYXkpID0+IHtcbiAgaC51cGRhdGUobXNnKTtcbiAgY29uc3QgbGVmdCA9IG1zZy5sZW5ndGggJSAxNjtcbiAgaWYgKGxlZnQpIGgudXBkYXRlKFpFUk9TMTYuc3ViYXJyYXkobGVmdCkpO1xufTtcblxuY29uc3QgWkVST1MzMiA9IC8qIEBfX1BVUkVfXyAqLyBuZXcgVWludDhBcnJheSgzMik7XG5mdW5jdGlvbiBjb21wdXRlVGFnKFxuICBmbjogWG9yU3RyZWFtLFxuICBrZXk6IFVpbnQ4QXJyYXksXG4gIG5vbmNlOiBVaW50OEFycmF5LFxuICBkYXRhOiBVaW50OEFycmF5LFxuICBBQUQ/OiBVaW50OEFycmF5XG4pOiBVaW50OEFycmF5IHtcbiAgY29uc3QgYXV0aEtleSA9IGZuKGtleSwgbm9uY2UsIFpFUk9TMzIpO1xuICBjb25zdCBoID0gcG9seTEzMDUuY3JlYXRlKGF1dGhLZXkpO1xuICBpZiAoQUFEKSB1cGRhdGVQYWRkZWQoaCwgQUFEKTtcbiAgdXBkYXRlUGFkZGVkKGgsIGRhdGEpO1xuICBjb25zdCBudW0gPSBuZXcgVWludDhBcnJheSgxNik7XG4gIGNvbnN0IHZpZXcgPSBjcmVhdGVWaWV3KG51bSk7XG4gIHNldEJpZ1VpbnQ2NCh2aWV3LCAwLCBCaWdJbnQoQUFEID8gQUFELmxlbmd0aCA6IDApLCB0cnVlKTtcbiAgc2V0QmlnVWludDY0KHZpZXcsIDgsIEJpZ0ludChkYXRhLmxlbmd0aCksIHRydWUpO1xuICBoLnVwZGF0ZShudW0pO1xuICBjb25zdCByZXMgPSBoLmRpZ2VzdCgpO1xuICBhdXRoS2V5LmZpbGwoMCk7XG4gIHJldHVybiByZXM7XG59XG5cbi8qKlxuICogQUVBRCBhbGdvcml0aG0gZnJvbSBSRkMgODQzOS5cbiAqIFNhbHNhMjAgYW5kIGNoYWNoYSAoUkZDIDg0MzkpIHVzZSBwb2x5MTMwNSBkaWZmZXJlbnRseS5cbiAqIFdlIGNvdWxkIGhhdmUgY29tcG9zZWQgdGhlbSBzaW1pbGFyIHRvOlxuICogaHR0cHM6Ly9naXRodWIuY29tL3BhdWxtaWxsci9zY3VyZS1iYXNlL2Jsb2IvYjI2NmM3M2RkZTk3N2IxZGQ3ZWY0MGVmN2EyM2NjMTVhYWI1MjZiMy9pbmRleC50cyNMMjUwXG4gKiBCdXQgaXQncyBoYXJkIGJlY2F1c2Ugb2YgYXV0aEtleTpcbiAqIEluIHNhbHNhMjAsIGF1dGhLZXkgY2hhbmdlcyBwb3NpdGlvbiBpbiBzYWxzYSBzdHJlYW0uXG4gKiBJbiBjaGFjaGEsIGF1dGhLZXkgY2FuJ3QgYmUgY29tcHV0ZWQgaW5zaWRlIGNvbXB1dGVUYWcsIGl0IG1vZGlmaWVzIHRoZSBjb3VudGVyLlxuICovXG5leHBvcnQgY29uc3QgX3BvbHkxMzA1X2FlYWQgPVxuICAoeG9yU3RyZWFtOiBYb3JTdHJlYW0pID0+XG4gIChrZXk6IFVpbnQ4QXJyYXksIG5vbmNlOiBVaW50OEFycmF5LCBBQUQ/OiBVaW50OEFycmF5KTogQ2lwaGVyV2l0aE91dHB1dCA9PiB7XG4gICAgY29uc3QgdGFnTGVuZ3RoID0gMTY7XG4gICAgYWJ5dGVzKGtleSwgMzIpO1xuICAgIGFieXRlcyhub25jZSk7XG4gICAgcmV0dXJuIHtcbiAgICAgIGVuY3J5cHQ6IChwbGFpbnRleHQ6IFVpbnQ4QXJyYXksIG91dHB1dD86IFVpbnQ4QXJyYXkpID0+IHtcbiAgICAgICAgY29uc3QgcGxlbmd0aCA9IHBsYWludGV4dC5sZW5ndGg7XG4gICAgICAgIGNvbnN0IGNsZW5ndGggPSBwbGVuZ3RoICsgdGFnTGVuZ3RoO1xuICAgICAgICBpZiAob3V0cHV0KSB7XG4gICAgICAgICAgYWJ5dGVzKG91dHB1dCwgY2xlbmd0aCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgb3V0cHV0ID0gbmV3IFVpbnQ4QXJyYXkoY2xlbmd0aCk7XG4gICAgICAgIH1cbiAgICAgICAgeG9yU3RyZWFtKGtleSwgbm9uY2UsIHBsYWludGV4dCwgb3V0cHV0LCAxKTtcbiAgICAgICAgY29uc3QgdGFnID0gY29tcHV0ZVRhZyh4b3JTdHJlYW0sIGtleSwgbm9uY2UsIG91dHB1dC5zdWJhcnJheSgwLCAtdGFnTGVuZ3RoKSwgQUFEKTtcbiAgICAgICAgb3V0cHV0LnNldCh0YWcsIHBsZW5ndGgpOyAvLyBhcHBlbmQgdGFnXG4gICAgICAgIHJldHVybiBvdXRwdXQ7XG4gICAgICB9LFxuICAgICAgZGVjcnlwdDogKGNpcGhlcnRleHQ6IFVpbnQ4QXJyYXksIG91dHB1dD86IFVpbnQ4QXJyYXkpID0+IHtcbiAgICAgICAgY29uc3QgY2xlbmd0aCA9IGNpcGhlcnRleHQubGVuZ3RoO1xuICAgICAgICBjb25zdCBwbGVuZ3RoID0gY2xlbmd0aCAtIHRhZ0xlbmd0aDtcbiAgICAgICAgaWYgKGNsZW5ndGggPCB0YWdMZW5ndGgpXG4gICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBlbmNyeXB0ZWQgZGF0YSBtdXN0IGJlIGF0IGxlYXN0ICR7dGFnTGVuZ3RofSBieXRlc2ApO1xuICAgICAgICBpZiAob3V0cHV0KSB7XG4gICAgICAgICAgYWJ5dGVzKG91dHB1dCwgcGxlbmd0aCk7XG4gICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgb3V0cHV0ID0gbmV3IFVpbnQ4QXJyYXkocGxlbmd0aCk7XG4gICAgICAgIH1cbiAgICAgICAgY29uc3QgZGF0YSA9IGNpcGhlcnRleHQuc3ViYXJyYXkoMCwgLXRhZ0xlbmd0aCk7XG4gICAgICAgIGNvbnN0IHBhc3NlZFRhZyA9IGNpcGhlcnRleHQuc3ViYXJyYXkoLXRhZ0xlbmd0aCk7XG4gICAgICAgIGNvbnN0IHRhZyA9IGNvbXB1dGVUYWcoeG9yU3RyZWFtLCBrZXksIG5vbmNlLCBkYXRhLCBBQUQpO1xuICAgICAgICBpZiAoIWVxdWFsQnl0ZXMocGFzc2VkVGFnLCB0YWcpKSB0aHJvdyBuZXcgRXJyb3IoJ2ludmFsaWQgdGFnJyk7XG4gICAgICAgIHhvclN0cmVhbShrZXksIG5vbmNlLCBkYXRhLCBvdXRwdXQsIDEpO1xuICAgICAgICByZXR1cm4gb3V0cHV0O1xuICAgICAgfSxcbiAgICB9O1xuICB9O1xuXG4vKipcbiAqIENoYUNoYTIwLVBvbHkxMzA1IGZyb20gUkZDIDg0MzkuXG4gKiBXaXRoIDEyLWJ5dGUgbm9uY2UsIGl0J3Mgbm90IHNhZmUgdG8gdXNlIGZpbGwgaXQgd2l0aCByYW5kb20gKENTUFJORyksIGR1ZSB0byBjb2xsaXNpb24gY2hhbmNlLlxuICovXG5leHBvcnQgY29uc3QgY2hhY2hhMjBwb2x5MTMwNSA9IC8qIEBfX1BVUkVfXyAqLyB3cmFwQ2lwaGVyKFxuICB7IGJsb2NrU2l6ZTogNjQsIG5vbmNlTGVuZ3RoOiAxMiwgdGFnTGVuZ3RoOiAxNiB9LFxuICBfcG9seTEzMDVfYWVhZChjaGFjaGEyMClcbik7XG4vKipcbiAqIFhDaGFDaGEyMC1Qb2x5MTMwNSBleHRlbmRlZC1ub25jZSBjaGFjaGEuXG4gKiBodHRwczovL2RhdGF0cmFja2VyLmlldGYub3JnL2RvYy9odG1sL2RyYWZ0LWlydGYtY2ZyZy14Y2hhY2hhXG4gKiBXaXRoIDI0LWJ5dGUgbm9uY2UsIGl0J3Mgc2FmZSB0byB1c2UgZmlsbCBpdCB3aXRoIHJhbmRvbSAoQ1NQUk5HKS5cbiAqL1xuZXhwb3J0IGNvbnN0IHhjaGFjaGEyMHBvbHkxMzA1ID0gLyogQF9fUFVSRV9fICovIHdyYXBDaXBoZXIoXG4gIHsgYmxvY2tTaXplOiA2NCwgbm9uY2VMZW5ndGg6IDI0LCB0YWdMZW5ndGg6IDE2IH0sXG4gIF9wb2x5MTMwNV9hZWFkKHhjaGFjaGEyMClcbik7XG4iLCAiaW1wb3J0IGFzc2VydCBmcm9tICcuL19hc3NlcnQuanMnO1xuaW1wb3J0IHsgSGFzaCwgQ0hhc2gsIElucHV0LCB0b0J5dGVzIH0gZnJvbSAnLi91dGlscy5qcyc7XG4vLyBITUFDIChSRkMgMjEwNClcbmV4cG9ydCBjbGFzcyBITUFDPFQgZXh0ZW5kcyBIYXNoPFQ+PiBleHRlbmRzIEhhc2g8SE1BQzxUPj4ge1xuICBvSGFzaDogVDtcbiAgaUhhc2g6IFQ7XG4gIGJsb2NrTGVuOiBudW1iZXI7XG4gIG91dHB1dExlbjogbnVtYmVyO1xuICBwcml2YXRlIGZpbmlzaGVkID0gZmFsc2U7XG4gIHByaXZhdGUgZGVzdHJveWVkID0gZmFsc2U7XG5cbiAgY29uc3RydWN0b3IoaGFzaDogQ0hhc2gsIF9rZXk6IElucHV0KSB7XG4gICAgc3VwZXIoKTtcbiAgICBhc3NlcnQuaGFzaChoYXNoKTtcbiAgICBjb25zdCBrZXkgPSB0b0J5dGVzKF9rZXkpO1xuICAgIHRoaXMuaUhhc2ggPSBoYXNoLmNyZWF0ZSgpIGFzIFQ7XG4gICAgaWYgKHR5cGVvZiB0aGlzLmlIYXNoLnVwZGF0ZSAhPT0gJ2Z1bmN0aW9uJylcbiAgICAgIHRocm93IG5ldyBFcnJvcignRXhwZWN0ZWQgaW5zdGFuY2Ugb2YgY2xhc3Mgd2hpY2ggZXh0ZW5kcyB1dGlscy5IYXNoJyk7XG4gICAgdGhpcy5ibG9ja0xlbiA9IHRoaXMuaUhhc2guYmxvY2tMZW47XG4gICAgdGhpcy5vdXRwdXRMZW4gPSB0aGlzLmlIYXNoLm91dHB1dExlbjtcbiAgICBjb25zdCBibG9ja0xlbiA9IHRoaXMuYmxvY2tMZW47XG4gICAgY29uc3QgcGFkID0gbmV3IFVpbnQ4QXJyYXkoYmxvY2tMZW4pO1xuICAgIC8vIGJsb2NrTGVuIGNhbiBiZSBiaWdnZXIgdGhhbiBvdXRwdXRMZW5cbiAgICBwYWQuc2V0KGtleS5sZW5ndGggPiBibG9ja0xlbiA/IGhhc2guY3JlYXRlKCkudXBkYXRlKGtleSkuZGlnZXN0KCkgOiBrZXkpO1xuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgcGFkLmxlbmd0aDsgaSsrKSBwYWRbaV0gXj0gMHgzNjtcbiAgICB0aGlzLmlIYXNoLnVwZGF0ZShwYWQpO1xuICAgIC8vIEJ5IGRvaW5nIHVwZGF0ZSAocHJvY2Vzc2luZyBvZiBmaXJzdCBibG9jaykgb2Ygb3V0ZXIgaGFzaCBoZXJlIHdlIGNhbiByZS11c2UgaXQgYmV0d2VlbiBtdWx0aXBsZSBjYWxscyB2aWEgY2xvbmVcbiAgICB0aGlzLm9IYXNoID0gaGFzaC5jcmVhdGUoKSBhcyBUO1xuICAgIC8vIFVuZG8gaW50ZXJuYWwgWE9SICYmIGFwcGx5IG91dGVyIFhPUlxuICAgIGZvciAobGV0IGkgPSAwOyBpIDwgcGFkLmxlbmd0aDsgaSsrKSBwYWRbaV0gXj0gMHgzNiBeIDB4NWM7XG4gICAgdGhpcy5vSGFzaC51cGRhdGUocGFkKTtcbiAgICBwYWQuZmlsbCgwKTtcbiAgfVxuICB1cGRhdGUoYnVmOiBJbnB1dCkge1xuICAgIGFzc2VydC5leGlzdHModGhpcyk7XG4gICAgdGhpcy5pSGFzaC51cGRhdGUoYnVmKTtcbiAgICByZXR1cm4gdGhpcztcbiAgfVxuICBkaWdlc3RJbnRvKG91dDogVWludDhBcnJheSkge1xuICAgIGFzc2VydC5leGlzdHModGhpcyk7XG4gICAgYXNzZXJ0LmJ5dGVzKG91dCwgdGhpcy5vdXRwdXRMZW4pO1xuICAgIHRoaXMuZmluaXNoZWQgPSB0cnVlO1xuICAgIHRoaXMuaUhhc2guZGlnZXN0SW50byhvdXQpO1xuICAgIHRoaXMub0hhc2gudXBkYXRlKG91dCk7XG4gICAgdGhpcy5vSGFzaC5kaWdlc3RJbnRvKG91dCk7XG4gICAgdGhpcy5kZXN0cm95KCk7XG4gIH1cbiAgZGlnZXN0KCkge1xuICAgIGNvbnN0IG91dCA9IG5ldyBVaW50OEFycmF5KHRoaXMub0hhc2gub3V0cHV0TGVuKTtcbiAgICB0aGlzLmRpZ2VzdEludG8ob3V0KTtcbiAgICByZXR1cm4gb3V0O1xuICB9XG4gIF9jbG9uZUludG8odG8/OiBITUFDPFQ+KTogSE1BQzxUPiB7XG4gICAgLy8gQ3JlYXRlIG5ldyBpbnN0YW5jZSB3aXRob3V0IGNhbGxpbmcgY29uc3RydWN0b3Igc2luY2Uga2V5IGFscmVhZHkgaW4gc3RhdGUgYW5kIHdlIGRvbid0IGtub3cgaXQuXG4gICAgdG8gfHw9IE9iamVjdC5jcmVhdGUoT2JqZWN0LmdldFByb3RvdHlwZU9mKHRoaXMpLCB7fSk7XG4gICAgY29uc3QgeyBvSGFzaCwgaUhhc2gsIGZpbmlzaGVkLCBkZXN0cm95ZWQsIGJsb2NrTGVuLCBvdXRwdXRMZW4gfSA9IHRoaXM7XG4gICAgdG8gPSB0byBhcyB0aGlzO1xuICAgIHRvLmZpbmlzaGVkID0gZmluaXNoZWQ7XG4gICAgdG8uZGVzdHJveWVkID0gZGVzdHJveWVkO1xuICAgIHRvLmJsb2NrTGVuID0gYmxvY2tMZW47XG4gICAgdG8ub3V0cHV0TGVuID0gb3V0cHV0TGVuO1xuICAgIHRvLm9IYXNoID0gb0hhc2guX2Nsb25lSW50byh0by5vSGFzaCk7XG4gICAgdG8uaUhhc2ggPSBpSGFzaC5fY2xvbmVJbnRvKHRvLmlIYXNoKTtcbiAgICByZXR1cm4gdG87XG4gIH1cbiAgZGVzdHJveSgpIHtcbiAgICB0aGlzLmRlc3Ryb3llZCA9IHRydWU7XG4gICAgdGhpcy5vSGFzaC5kZXN0cm95KCk7XG4gICAgdGhpcy5pSGFzaC5kZXN0cm95KCk7XG4gIH1cbn1cblxuLyoqXG4gKiBITUFDOiBSRkMyMTA0IG1lc3NhZ2UgYXV0aGVudGljYXRpb24gY29kZS5cbiAqIEBwYXJhbSBoYXNoIC0gZnVuY3Rpb24gdGhhdCB3b3VsZCBiZSB1c2VkIGUuZy4gc2hhMjU2XG4gKiBAcGFyYW0ga2V5IC0gbWVzc2FnZSBrZXlcbiAqIEBwYXJhbSBtZXNzYWdlIC0gbWVzc2FnZSBkYXRhXG4gKi9cbmV4cG9ydCBjb25zdCBobWFjID0gKGhhc2g6IENIYXNoLCBrZXk6IElucHV0LCBtZXNzYWdlOiBJbnB1dCk6IFVpbnQ4QXJyYXkgPT5cbiAgbmV3IEhNQUM8YW55PihoYXNoLCBrZXkpLnVwZGF0ZShtZXNzYWdlKS5kaWdlc3QoKTtcbmhtYWMuY3JlYXRlID0gKGhhc2g6IENIYXNoLCBrZXk6IElucHV0KSA9PiBuZXcgSE1BQzxhbnk+KGhhc2gsIGtleSk7XG4iLCAiaW1wb3J0IGFzc2VydCBmcm9tICcuL19hc3NlcnQuanMnO1xuaW1wb3J0IHsgQ0hhc2gsIElucHV0LCB0b0J5dGVzIH0gZnJvbSAnLi91dGlscy5qcyc7XG5pbXBvcnQgeyBobWFjIH0gZnJvbSAnLi9obWFjLmpzJztcblxuLy8gSEtERiAoUkZDIDU4NjkpXG4vLyBodHRwczovL3NvYXRvay5ibG9nLzIwMjEvMTEvMTcvdW5kZXJzdGFuZGluZy1oa2RmL1xuXG4vKipcbiAqIEhLREYtRXh0cmFjdChJS00sIHNhbHQpIC0+IFBSS1xuICogQXJndW1lbnRzIHBvc2l0aW9uIGRpZmZlcnMgZnJvbSBzcGVjIChJS00gaXMgZmlyc3Qgb25lLCBzaW5jZSBpdCBpcyBub3Qgb3B0aW9uYWwpXG4gKiBAcGFyYW0gaGFzaFxuICogQHBhcmFtIGlrbVxuICogQHBhcmFtIHNhbHRcbiAqIEByZXR1cm5zXG4gKi9cbmV4cG9ydCBmdW5jdGlvbiBleHRyYWN0KGhhc2g6IENIYXNoLCBpa206IElucHV0LCBzYWx0PzogSW5wdXQpIHtcbiAgYXNzZXJ0Lmhhc2goaGFzaCk7XG4gIC8vIE5PVEU6IHNvbWUgbGlicmFyaWVzIHRyZWF0IHplcm8tbGVuZ3RoIGFycmF5IGFzICdub3QgcHJvdmlkZWQnO1xuICAvLyB3ZSBkb24ndCwgc2luY2Ugd2UgaGF2ZSB1bmRlZmluZWQgYXMgJ25vdCBwcm92aWRlZCdcbiAgLy8gaHR0cHM6Ly9naXRodWIuY29tL1J1c3RDcnlwdG8vS0RGcy9pc3N1ZXMvMTVcbiAgaWYgKHNhbHQgPT09IHVuZGVmaW5lZCkgc2FsdCA9IG5ldyBVaW50OEFycmF5KGhhc2gub3V0cHV0TGVuKTsgLy8gaWYgbm90IHByb3ZpZGVkLCBpdCBpcyBzZXQgdG8gYSBzdHJpbmcgb2YgSGFzaExlbiB6ZXJvc1xuICByZXR1cm4gaG1hYyhoYXNoLCB0b0J5dGVzKHNhbHQpLCB0b0J5dGVzKGlrbSkpO1xufVxuXG4vLyBIS0RGLUV4cGFuZChQUkssIGluZm8sIEwpIC0+IE9LTVxuY29uc3QgSEtERl9DT1VOVEVSID0gbmV3IFVpbnQ4QXJyYXkoWzBdKTtcbmNvbnN0IEVNUFRZX0JVRkZFUiA9IG5ldyBVaW50OEFycmF5KCk7XG5cbi8qKlxuICogSEtERi1leHBhbmQgZnJvbSB0aGUgc3BlYy5cbiAqIEBwYXJhbSBwcmsgLSBhIHBzZXVkb3JhbmRvbSBrZXkgb2YgYXQgbGVhc3QgSGFzaExlbiBvY3RldHMgKHVzdWFsbHksIHRoZSBvdXRwdXQgZnJvbSB0aGUgZXh0cmFjdCBzdGVwKVxuICogQHBhcmFtIGluZm8gLSBvcHRpb25hbCBjb250ZXh0IGFuZCBhcHBsaWNhdGlvbiBzcGVjaWZpYyBpbmZvcm1hdGlvbiAoY2FuIGJlIGEgemVyby1sZW5ndGggc3RyaW5nKVxuICogQHBhcmFtIGxlbmd0aCAtIGxlbmd0aCBvZiBvdXRwdXQga2V5aW5nIG1hdGVyaWFsIGluIG9jdGV0c1xuICovXG5leHBvcnQgZnVuY3Rpb24gZXhwYW5kKGhhc2g6IENIYXNoLCBwcms6IElucHV0LCBpbmZvPzogSW5wdXQsIGxlbmd0aDogbnVtYmVyID0gMzIpIHtcbiAgYXNzZXJ0Lmhhc2goaGFzaCk7XG4gIGFzc2VydC5udW1iZXIobGVuZ3RoKTtcbiAgaWYgKGxlbmd0aCA+IDI1NSAqIGhhc2gub3V0cHV0TGVuKSB0aHJvdyBuZXcgRXJyb3IoJ0xlbmd0aCBzaG91bGQgYmUgPD0gMjU1Kkhhc2hMZW4nKTtcbiAgY29uc3QgYmxvY2tzID0gTWF0aC5jZWlsKGxlbmd0aCAvIGhhc2gub3V0cHV0TGVuKTtcbiAgaWYgKGluZm8gPT09IHVuZGVmaW5lZCkgaW5mbyA9IEVNUFRZX0JVRkZFUjtcbiAgLy8gZmlyc3QgTChlbmd0aCkgb2N0ZXRzIG9mIFRcbiAgY29uc3Qgb2ttID0gbmV3IFVpbnQ4QXJyYXkoYmxvY2tzICogaGFzaC5vdXRwdXRMZW4pO1xuICAvLyBSZS11c2UgSE1BQyBpbnN0YW5jZSBiZXR3ZWVuIGJsb2Nrc1xuICBjb25zdCBITUFDID0gaG1hYy5jcmVhdGUoaGFzaCwgcHJrKTtcbiAgY29uc3QgSE1BQ1RtcCA9IEhNQUMuX2Nsb25lSW50bygpO1xuICBjb25zdCBUID0gbmV3IFVpbnQ4QXJyYXkoSE1BQy5vdXRwdXRMZW4pO1xuICBmb3IgKGxldCBjb3VudGVyID0gMDsgY291bnRlciA8IGJsb2NrczsgY291bnRlcisrKSB7XG4gICAgSEtERl9DT1VOVEVSWzBdID0gY291bnRlciArIDE7XG4gICAgLy8gVCgwKSA9IGVtcHR5IHN0cmluZyAoemVybyBsZW5ndGgpXG4gICAgLy8gVChOKSA9IEhNQUMtSGFzaChQUkssIFQoTi0xKSB8IGluZm8gfCBOKVxuICAgIEhNQUNUbXAudXBkYXRlKGNvdW50ZXIgPT09IDAgPyBFTVBUWV9CVUZGRVIgOiBUKVxuICAgICAgLnVwZGF0ZShpbmZvKVxuICAgICAgLnVwZGF0ZShIS0RGX0NPVU5URVIpXG4gICAgICAuZGlnZXN0SW50byhUKTtcbiAgICBva20uc2V0KFQsIGhhc2gub3V0cHV0TGVuICogY291bnRlcik7XG4gICAgSE1BQy5fY2xvbmVJbnRvKEhNQUNUbXApO1xuICB9XG4gIEhNQUMuZGVzdHJveSgpO1xuICBITUFDVG1wLmRlc3Ryb3koKTtcbiAgVC5maWxsKDApO1xuICBIS0RGX0NPVU5URVIuZmlsbCgwKTtcbiAgcmV0dXJuIG9rbS5zbGljZSgwLCBsZW5ndGgpO1xufVxuXG4vKipcbiAqIEhLREYgKFJGQyA1ODY5KTogZXh0cmFjdCArIGV4cGFuZCBpbiBvbmUgc3RlcC5cbiAqIEBwYXJhbSBoYXNoIC0gaGFzaCBmdW5jdGlvbiB0aGF0IHdvdWxkIGJlIHVzZWQgKGUuZy4gc2hhMjU2KVxuICogQHBhcmFtIGlrbSAtIGlucHV0IGtleWluZyBtYXRlcmlhbCwgdGhlIGluaXRpYWwga2V5XG4gKiBAcGFyYW0gc2FsdCAtIG9wdGlvbmFsIHNhbHQgdmFsdWUgKGEgbm9uLXNlY3JldCByYW5kb20gdmFsdWUpXG4gKiBAcGFyYW0gaW5mbyAtIG9wdGlvbmFsIGNvbnRleHQgYW5kIGFwcGxpY2F0aW9uIHNwZWNpZmljIGluZm9ybWF0aW9uXG4gKiBAcGFyYW0gbGVuZ3RoIC0gbGVuZ3RoIG9mIG91dHB1dCBrZXlpbmcgbWF0ZXJpYWwgaW4gb2N0ZXRzXG4gKi9cbmV4cG9ydCBjb25zdCBoa2RmID0gKFxuICBoYXNoOiBDSGFzaCxcbiAgaWttOiBJbnB1dCxcbiAgc2FsdDogSW5wdXQgfCB1bmRlZmluZWQsXG4gIGluZm86IElucHV0IHwgdW5kZWZpbmVkLFxuICBsZW5ndGg6IG51bWJlclxuKSA9PiBleHBhbmQoaGFzaCwgZXh0cmFjdChoYXNoLCBpa20sIHNhbHQpLCBpbmZvLCBsZW5ndGgpO1xuIiwgInZhciBfX2RlZlByb3AgPSBPYmplY3QuZGVmaW5lUHJvcGVydHk7XG52YXIgX19leHBvcnQgPSAodGFyZ2V0LCBhbGwpID0+IHtcbiAgZm9yICh2YXIgbmFtZSBpbiBhbGwpXG4gICAgX19kZWZQcm9wKHRhcmdldCwgbmFtZSwgeyBnZXQ6IGFsbFtuYW1lXSwgZW51bWVyYWJsZTogdHJ1ZSB9KTtcbn07XG5cbi8vIHB1cmUudHNcbmltcG9ydCB7IHNjaG5vcnIgfSBmcm9tIFwiQG5vYmxlL2N1cnZlcy9zZWNwMjU2azFcIjtcbmltcG9ydCB7IGJ5dGVzVG9IZXggYXMgYnl0ZXNUb0hleDIgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy91dGlsc1wiO1xuXG4vLyBjb3JlLnRzXG52YXIgdmVyaWZpZWRTeW1ib2wgPSBTeW1ib2woXCJ2ZXJpZmllZFwiKTtcbnZhciBpc1JlY29yZCA9IChvYmopID0+IG9iaiBpbnN0YW5jZW9mIE9iamVjdDtcbmZ1bmN0aW9uIHZhbGlkYXRlRXZlbnQoZXZlbnQpIHtcbiAgaWYgKCFpc1JlY29yZChldmVudCkpXG4gICAgcmV0dXJuIGZhbHNlO1xuICBpZiAodHlwZW9mIGV2ZW50LmtpbmQgIT09IFwibnVtYmVyXCIpXG4gICAgcmV0dXJuIGZhbHNlO1xuICBpZiAodHlwZW9mIGV2ZW50LmNvbnRlbnQgIT09IFwic3RyaW5nXCIpXG4gICAgcmV0dXJuIGZhbHNlO1xuICBpZiAodHlwZW9mIGV2ZW50LmNyZWF0ZWRfYXQgIT09IFwibnVtYmVyXCIpXG4gICAgcmV0dXJuIGZhbHNlO1xuICBpZiAodHlwZW9mIGV2ZW50LnB1YmtleSAhPT0gXCJzdHJpbmdcIilcbiAgICByZXR1cm4gZmFsc2U7XG4gIGlmICghZXZlbnQucHVia2V5Lm1hdGNoKC9eW2EtZjAtOV17NjR9JC8pKVxuICAgIHJldHVybiBmYWxzZTtcbiAgaWYgKCFBcnJheS5pc0FycmF5KGV2ZW50LnRhZ3MpKVxuICAgIHJldHVybiBmYWxzZTtcbiAgZm9yIChsZXQgaTIgPSAwOyBpMiA8IGV2ZW50LnRhZ3MubGVuZ3RoOyBpMisrKSB7XG4gICAgbGV0IHRhZyA9IGV2ZW50LnRhZ3NbaTJdO1xuICAgIGlmICghQXJyYXkuaXNBcnJheSh0YWcpKVxuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIGZvciAobGV0IGogPSAwOyBqIDwgdGFnLmxlbmd0aDsgaisrKSB7XG4gICAgICBpZiAodHlwZW9mIHRhZ1tqXSAhPT0gXCJzdHJpbmdcIilcbiAgICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgfVxuICByZXR1cm4gdHJ1ZTtcbn1cbmZ1bmN0aW9uIHNvcnRFdmVudHMoZXZlbnRzKSB7XG4gIHJldHVybiBldmVudHMuc29ydCgoYSwgYikgPT4ge1xuICAgIGlmIChhLmNyZWF0ZWRfYXQgIT09IGIuY3JlYXRlZF9hdCkge1xuICAgICAgcmV0dXJuIGIuY3JlYXRlZF9hdCAtIGEuY3JlYXRlZF9hdDtcbiAgICB9XG4gICAgcmV0dXJuIGEuaWQubG9jYWxlQ29tcGFyZShiLmlkKTtcbiAgfSk7XG59XG5cbi8vIHB1cmUudHNcbmltcG9ydCB7IHNoYTI1NiB9IGZyb20gXCJAbm9ibGUvaGFzaGVzL3NoYTI1NlwiO1xuXG4vLyB1dGlscy50c1xudmFyIHV0aWxzX2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KHV0aWxzX2V4cG9ydHMsIHtcbiAgUXVldWU6ICgpID0+IFF1ZXVlLFxuICBRdWV1ZU5vZGU6ICgpID0+IFF1ZXVlTm9kZSxcbiAgYmluYXJ5U2VhcmNoOiAoKSA9PiBiaW5hcnlTZWFyY2gsXG4gIGJ5dGVzVG9IZXg6ICgpID0+IGJ5dGVzVG9IZXgsXG4gIGhleFRvQnl0ZXM6ICgpID0+IGhleFRvQnl0ZXMsXG4gIGluc2VydEV2ZW50SW50b0FzY2VuZGluZ0xpc3Q6ICgpID0+IGluc2VydEV2ZW50SW50b0FzY2VuZGluZ0xpc3QsXG4gIGluc2VydEV2ZW50SW50b0Rlc2NlbmRpbmdMaXN0OiAoKSA9PiBpbnNlcnRFdmVudEludG9EZXNjZW5kaW5nTGlzdCxcbiAgbm9ybWFsaXplVVJMOiAoKSA9PiBub3JtYWxpemVVUkwsXG4gIHV0ZjhEZWNvZGVyOiAoKSA9PiB1dGY4RGVjb2RlcixcbiAgdXRmOEVuY29kZXI6ICgpID0+IHV0ZjhFbmNvZGVyXG59KTtcbmltcG9ydCB7IGJ5dGVzVG9IZXgsIGhleFRvQnl0ZXMgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy91dGlsc1wiO1xudmFyIHV0ZjhEZWNvZGVyID0gbmV3IFRleHREZWNvZGVyKFwidXRmLThcIik7XG52YXIgdXRmOEVuY29kZXIgPSBuZXcgVGV4dEVuY29kZXIoKTtcbmZ1bmN0aW9uIG5vcm1hbGl6ZVVSTCh1cmwpIHtcbiAgdHJ5IHtcbiAgICBpZiAodXJsLmluZGV4T2YoXCI6Ly9cIikgPT09IC0xKVxuICAgICAgdXJsID0gXCJ3c3M6Ly9cIiArIHVybDtcbiAgICBsZXQgcCA9IG5ldyBVUkwodXJsKTtcbiAgICBwLnBhdGhuYW1lID0gcC5wYXRobmFtZS5yZXBsYWNlKC9cXC8rL2csIFwiL1wiKTtcbiAgICBpZiAocC5wYXRobmFtZS5lbmRzV2l0aChcIi9cIikpXG4gICAgICBwLnBhdGhuYW1lID0gcC5wYXRobmFtZS5zbGljZSgwLCAtMSk7XG4gICAgaWYgKHAucG9ydCA9PT0gXCI4MFwiICYmIHAucHJvdG9jb2wgPT09IFwid3M6XCIgfHwgcC5wb3J0ID09PSBcIjQ0M1wiICYmIHAucHJvdG9jb2wgPT09IFwid3NzOlwiKVxuICAgICAgcC5wb3J0ID0gXCJcIjtcbiAgICBwLnNlYXJjaFBhcmFtcy5zb3J0KCk7XG4gICAgcC5oYXNoID0gXCJcIjtcbiAgICByZXR1cm4gcC50b1N0cmluZygpO1xuICB9IGNhdGNoIChlKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKGBJbnZhbGlkIFVSTDogJHt1cmx9YCk7XG4gIH1cbn1cbmZ1bmN0aW9uIGluc2VydEV2ZW50SW50b0Rlc2NlbmRpbmdMaXN0KHNvcnRlZEFycmF5LCBldmVudCkge1xuICBjb25zdCBbaWR4LCBmb3VuZF0gPSBiaW5hcnlTZWFyY2goc29ydGVkQXJyYXksIChiKSA9PiB7XG4gICAgaWYgKGV2ZW50LmlkID09PSBiLmlkKVxuICAgICAgcmV0dXJuIDA7XG4gICAgaWYgKGV2ZW50LmNyZWF0ZWRfYXQgPT09IGIuY3JlYXRlZF9hdClcbiAgICAgIHJldHVybiAtMTtcbiAgICByZXR1cm4gYi5jcmVhdGVkX2F0IC0gZXZlbnQuY3JlYXRlZF9hdDtcbiAgfSk7XG4gIGlmICghZm91bmQpIHtcbiAgICBzb3J0ZWRBcnJheS5zcGxpY2UoaWR4LCAwLCBldmVudCk7XG4gIH1cbiAgcmV0dXJuIHNvcnRlZEFycmF5O1xufVxuZnVuY3Rpb24gaW5zZXJ0RXZlbnRJbnRvQXNjZW5kaW5nTGlzdChzb3J0ZWRBcnJheSwgZXZlbnQpIHtcbiAgY29uc3QgW2lkeCwgZm91bmRdID0gYmluYXJ5U2VhcmNoKHNvcnRlZEFycmF5LCAoYikgPT4ge1xuICAgIGlmIChldmVudC5pZCA9PT0gYi5pZClcbiAgICAgIHJldHVybiAwO1xuICAgIGlmIChldmVudC5jcmVhdGVkX2F0ID09PSBiLmNyZWF0ZWRfYXQpXG4gICAgICByZXR1cm4gLTE7XG4gICAgcmV0dXJuIGV2ZW50LmNyZWF0ZWRfYXQgLSBiLmNyZWF0ZWRfYXQ7XG4gIH0pO1xuICBpZiAoIWZvdW5kKSB7XG4gICAgc29ydGVkQXJyYXkuc3BsaWNlKGlkeCwgMCwgZXZlbnQpO1xuICB9XG4gIHJldHVybiBzb3J0ZWRBcnJheTtcbn1cbmZ1bmN0aW9uIGJpbmFyeVNlYXJjaChhcnIsIGNvbXBhcmUpIHtcbiAgbGV0IHN0YXJ0ID0gMDtcbiAgbGV0IGVuZCA9IGFyci5sZW5ndGggLSAxO1xuICB3aGlsZSAoc3RhcnQgPD0gZW5kKSB7XG4gICAgY29uc3QgbWlkID0gTWF0aC5mbG9vcigoc3RhcnQgKyBlbmQpIC8gMik7XG4gICAgY29uc3QgY21wID0gY29tcGFyZShhcnJbbWlkXSk7XG4gICAgaWYgKGNtcCA9PT0gMCkge1xuICAgICAgcmV0dXJuIFttaWQsIHRydWVdO1xuICAgIH1cbiAgICBpZiAoY21wIDwgMCkge1xuICAgICAgZW5kID0gbWlkIC0gMTtcbiAgICB9IGVsc2Uge1xuICAgICAgc3RhcnQgPSBtaWQgKyAxO1xuICAgIH1cbiAgfVxuICByZXR1cm4gW3N0YXJ0LCBmYWxzZV07XG59XG52YXIgUXVldWVOb2RlID0gY2xhc3Mge1xuICB2YWx1ZTtcbiAgbmV4dCA9IG51bGw7XG4gIHByZXYgPSBudWxsO1xuICBjb25zdHJ1Y3RvcihtZXNzYWdlKSB7XG4gICAgdGhpcy52YWx1ZSA9IG1lc3NhZ2U7XG4gIH1cbn07XG52YXIgUXVldWUgPSBjbGFzcyB7XG4gIGZpcnN0O1xuICBsYXN0O1xuICBjb25zdHJ1Y3RvcigpIHtcbiAgICB0aGlzLmZpcnN0ID0gbnVsbDtcbiAgICB0aGlzLmxhc3QgPSBudWxsO1xuICB9XG4gIGVucXVldWUodmFsdWUpIHtcbiAgICBjb25zdCBuZXdOb2RlID0gbmV3IFF1ZXVlTm9kZSh2YWx1ZSk7XG4gICAgaWYgKCF0aGlzLmxhc3QpIHtcbiAgICAgIHRoaXMuZmlyc3QgPSBuZXdOb2RlO1xuICAgICAgdGhpcy5sYXN0ID0gbmV3Tm9kZTtcbiAgICB9IGVsc2UgaWYgKHRoaXMubGFzdCA9PT0gdGhpcy5maXJzdCkge1xuICAgICAgdGhpcy5sYXN0ID0gbmV3Tm9kZTtcbiAgICAgIHRoaXMubGFzdC5wcmV2ID0gdGhpcy5maXJzdDtcbiAgICAgIHRoaXMuZmlyc3QubmV4dCA9IG5ld05vZGU7XG4gICAgfSBlbHNlIHtcbiAgICAgIG5ld05vZGUucHJldiA9IHRoaXMubGFzdDtcbiAgICAgIHRoaXMubGFzdC5uZXh0ID0gbmV3Tm9kZTtcbiAgICAgIHRoaXMubGFzdCA9IG5ld05vZGU7XG4gICAgfVxuICAgIHJldHVybiB0cnVlO1xuICB9XG4gIGRlcXVldWUoKSB7XG4gICAgaWYgKCF0aGlzLmZpcnN0KVxuICAgICAgcmV0dXJuIG51bGw7XG4gICAgaWYgKHRoaXMuZmlyc3QgPT09IHRoaXMubGFzdCkge1xuICAgICAgY29uc3QgdGFyZ2V0MiA9IHRoaXMuZmlyc3Q7XG4gICAgICB0aGlzLmZpcnN0ID0gbnVsbDtcbiAgICAgIHRoaXMubGFzdCA9IG51bGw7XG4gICAgICByZXR1cm4gdGFyZ2V0Mi52YWx1ZTtcbiAgICB9XG4gICAgY29uc3QgdGFyZ2V0ID0gdGhpcy5maXJzdDtcbiAgICB0aGlzLmZpcnN0ID0gdGFyZ2V0Lm5leHQ7XG4gICAgaWYgKHRoaXMuZmlyc3QpIHtcbiAgICAgIHRoaXMuZmlyc3QucHJldiA9IG51bGw7XG4gICAgfVxuICAgIHJldHVybiB0YXJnZXQudmFsdWU7XG4gIH1cbn07XG5cbi8vIHB1cmUudHNcbnZhciBKUyA9IGNsYXNzIHtcbiAgZ2VuZXJhdGVTZWNyZXRLZXkoKSB7XG4gICAgcmV0dXJuIHNjaG5vcnIudXRpbHMucmFuZG9tUHJpdmF0ZUtleSgpO1xuICB9XG4gIGdldFB1YmxpY0tleShzZWNyZXRLZXkpIHtcbiAgICByZXR1cm4gYnl0ZXNUb0hleDIoc2Nobm9yci5nZXRQdWJsaWNLZXkoc2VjcmV0S2V5KSk7XG4gIH1cbiAgZmluYWxpemVFdmVudCh0LCBzZWNyZXRLZXkpIHtcbiAgICBjb25zdCBldmVudCA9IHQ7XG4gICAgZXZlbnQucHVia2V5ID0gYnl0ZXNUb0hleDIoc2Nobm9yci5nZXRQdWJsaWNLZXkoc2VjcmV0S2V5KSk7XG4gICAgZXZlbnQuaWQgPSBnZXRFdmVudEhhc2goZXZlbnQpO1xuICAgIGV2ZW50LnNpZyA9IGJ5dGVzVG9IZXgyKHNjaG5vcnIuc2lnbihnZXRFdmVudEhhc2goZXZlbnQpLCBzZWNyZXRLZXkpKTtcbiAgICBldmVudFt2ZXJpZmllZFN5bWJvbF0gPSB0cnVlO1xuICAgIHJldHVybiBldmVudDtcbiAgfVxuICB2ZXJpZnlFdmVudChldmVudCkge1xuICAgIGlmICh0eXBlb2YgZXZlbnRbdmVyaWZpZWRTeW1ib2xdID09PSBcImJvb2xlYW5cIilcbiAgICAgIHJldHVybiBldmVudFt2ZXJpZmllZFN5bWJvbF07XG4gICAgY29uc3QgaGFzaCA9IGdldEV2ZW50SGFzaChldmVudCk7XG4gICAgaWYgKGhhc2ggIT09IGV2ZW50LmlkKSB7XG4gICAgICBldmVudFt2ZXJpZmllZFN5bWJvbF0gPSBmYWxzZTtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gICAgdHJ5IHtcbiAgICAgIGNvbnN0IHZhbGlkID0gc2Nobm9yci52ZXJpZnkoZXZlbnQuc2lnLCBoYXNoLCBldmVudC5wdWJrZXkpO1xuICAgICAgZXZlbnRbdmVyaWZpZWRTeW1ib2xdID0gdmFsaWQ7XG4gICAgICByZXR1cm4gdmFsaWQ7XG4gICAgfSBjYXRjaCAoZXJyKSB7XG4gICAgICBldmVudFt2ZXJpZmllZFN5bWJvbF0gPSBmYWxzZTtcbiAgICAgIHJldHVybiBmYWxzZTtcbiAgICB9XG4gIH1cbn07XG5mdW5jdGlvbiBzZXJpYWxpemVFdmVudChldnQpIHtcbiAgaWYgKCF2YWxpZGF0ZUV2ZW50KGV2dCkpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwiY2FuJ3Qgc2VyaWFsaXplIGV2ZW50IHdpdGggd3Jvbmcgb3IgbWlzc2luZyBwcm9wZXJ0aWVzXCIpO1xuICByZXR1cm4gSlNPTi5zdHJpbmdpZnkoWzAsIGV2dC5wdWJrZXksIGV2dC5jcmVhdGVkX2F0LCBldnQua2luZCwgZXZ0LnRhZ3MsIGV2dC5jb250ZW50XSk7XG59XG5mdW5jdGlvbiBnZXRFdmVudEhhc2goZXZlbnQpIHtcbiAgbGV0IGV2ZW50SGFzaCA9IHNoYTI1Nih1dGY4RW5jb2Rlci5lbmNvZGUoc2VyaWFsaXplRXZlbnQoZXZlbnQpKSk7XG4gIHJldHVybiBieXRlc1RvSGV4MihldmVudEhhc2gpO1xufVxudmFyIGkgPSBuZXcgSlMoKTtcbnZhciBnZW5lcmF0ZVNlY3JldEtleSA9IGkuZ2VuZXJhdGVTZWNyZXRLZXk7XG52YXIgZ2V0UHVibGljS2V5ID0gaS5nZXRQdWJsaWNLZXk7XG52YXIgZmluYWxpemVFdmVudCA9IGkuZmluYWxpemVFdmVudDtcbnZhciB2ZXJpZnlFdmVudCA9IGkudmVyaWZ5RXZlbnQ7XG5cbi8vIGtpbmRzLnRzXG52YXIga2luZHNfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQoa2luZHNfZXhwb3J0cywge1xuICBBcHBsaWNhdGlvbjogKCkgPT4gQXBwbGljYXRpb24sXG4gIEJhZGdlQXdhcmQ6ICgpID0+IEJhZGdlQXdhcmQsXG4gIEJhZGdlRGVmaW5pdGlvbjogKCkgPT4gQmFkZ2VEZWZpbml0aW9uLFxuICBCbG9ja2VkUmVsYXlzTGlzdDogKCkgPT4gQmxvY2tlZFJlbGF5c0xpc3QsXG4gIEJvb2ttYXJrTGlzdDogKCkgPT4gQm9va21hcmtMaXN0LFxuICBCb29rbWFya3NldHM6ICgpID0+IEJvb2ttYXJrc2V0cyxcbiAgQ2FsZW5kYXI6ICgpID0+IENhbGVuZGFyLFxuICBDYWxlbmRhckV2ZW50UlNWUDogKCkgPT4gQ2FsZW5kYXJFdmVudFJTVlAsXG4gIENoYW5uZWxDcmVhdGlvbjogKCkgPT4gQ2hhbm5lbENyZWF0aW9uLFxuICBDaGFubmVsSGlkZU1lc3NhZ2U6ICgpID0+IENoYW5uZWxIaWRlTWVzc2FnZSxcbiAgQ2hhbm5lbE1lc3NhZ2U6ICgpID0+IENoYW5uZWxNZXNzYWdlLFxuICBDaGFubmVsTWV0YWRhdGE6ICgpID0+IENoYW5uZWxNZXRhZGF0YSxcbiAgQ2hhbm5lbE11dGVVc2VyOiAoKSA9PiBDaGFubmVsTXV0ZVVzZXIsXG4gIENsYXNzaWZpZWRMaXN0aW5nOiAoKSA9PiBDbGFzc2lmaWVkTGlzdGluZyxcbiAgQ2xpZW50QXV0aDogKCkgPT4gQ2xpZW50QXV0aCxcbiAgQ29tbXVuaXRpZXNMaXN0OiAoKSA9PiBDb21tdW5pdGllc0xpc3QsXG4gIENvbW11bml0eURlZmluaXRpb246ICgpID0+IENvbW11bml0eURlZmluaXRpb24sXG4gIENvbW11bml0eVBvc3RBcHByb3ZhbDogKCkgPT4gQ29tbXVuaXR5UG9zdEFwcHJvdmFsLFxuICBDb250YWN0czogKCkgPT4gQ29udGFjdHMsXG4gIENyZWF0ZU9yVXBkYXRlUHJvZHVjdDogKCkgPT4gQ3JlYXRlT3JVcGRhdGVQcm9kdWN0LFxuICBDcmVhdGVPclVwZGF0ZVN0YWxsOiAoKSA9PiBDcmVhdGVPclVwZGF0ZVN0YWxsLFxuICBDdXJhdGlvbnNldHM6ICgpID0+IEN1cmF0aW9uc2V0cyxcbiAgRGF0ZTogKCkgPT4gRGF0ZTIsXG4gIERpcmVjdE1lc3NhZ2VSZWxheXNMaXN0OiAoKSA9PiBEaXJlY3RNZXNzYWdlUmVsYXlzTGlzdCxcbiAgRHJhZnRDbGFzc2lmaWVkTGlzdGluZzogKCkgPT4gRHJhZnRDbGFzc2lmaWVkTGlzdGluZyxcbiAgRHJhZnRMb25nOiAoKSA9PiBEcmFmdExvbmcsXG4gIEVtb2ppc2V0czogKCkgPT4gRW1vamlzZXRzLFxuICBFbmNyeXB0ZWREaXJlY3RNZXNzYWdlOiAoKSA9PiBFbmNyeXB0ZWREaXJlY3RNZXNzYWdlLFxuICBFdmVudERlbGV0aW9uOiAoKSA9PiBFdmVudERlbGV0aW9uLFxuICBGaWxlTWV0YWRhdGE6ICgpID0+IEZpbGVNZXRhZGF0YSxcbiAgRmlsZVNlcnZlclByZWZlcmVuY2U6ICgpID0+IEZpbGVTZXJ2ZXJQcmVmZXJlbmNlLFxuICBGb2xsb3dzZXRzOiAoKSA9PiBGb2xsb3dzZXRzLFxuICBHZW5lcmljUmVwb3N0OiAoKSA9PiBHZW5lcmljUmVwb3N0LFxuICBHZW5lcmljbGlzdHM6ICgpID0+IEdlbmVyaWNsaXN0cyxcbiAgR2lmdFdyYXA6ICgpID0+IEdpZnRXcmFwLFxuICBIVFRQQXV0aDogKCkgPT4gSFRUUEF1dGgsXG4gIEhhbmRsZXJpbmZvcm1hdGlvbjogKCkgPT4gSGFuZGxlcmluZm9ybWF0aW9uLFxuICBIYW5kbGVycmVjb21tZW5kYXRpb246ICgpID0+IEhhbmRsZXJyZWNvbW1lbmRhdGlvbixcbiAgSGlnaGxpZ2h0czogKCkgPT4gSGlnaGxpZ2h0cyxcbiAgSW50ZXJlc3RzTGlzdDogKCkgPT4gSW50ZXJlc3RzTGlzdCxcbiAgSW50ZXJlc3RzZXRzOiAoKSA9PiBJbnRlcmVzdHNldHMsXG4gIEpvYkZlZWRiYWNrOiAoKSA9PiBKb2JGZWVkYmFjayxcbiAgSm9iUmVxdWVzdDogKCkgPT4gSm9iUmVxdWVzdCxcbiAgSm9iUmVzdWx0OiAoKSA9PiBKb2JSZXN1bHQsXG4gIExhYmVsOiAoKSA9PiBMYWJlbCxcbiAgTGlnaHRuaW5nUHViUlBDOiAoKSA9PiBMaWdodG5pbmdQdWJSUEMsXG4gIExpdmVDaGF0TWVzc2FnZTogKCkgPT4gTGl2ZUNoYXRNZXNzYWdlLFxuICBMaXZlRXZlbnQ6ICgpID0+IExpdmVFdmVudCxcbiAgTG9uZ0Zvcm1BcnRpY2xlOiAoKSA9PiBMb25nRm9ybUFydGljbGUsXG4gIE1ldGFkYXRhOiAoKSA9PiBNZXRhZGF0YSxcbiAgTXV0ZWxpc3Q6ICgpID0+IE11dGVsaXN0LFxuICBOV0NXYWxsZXRJbmZvOiAoKSA9PiBOV0NXYWxsZXRJbmZvLFxuICBOV0NXYWxsZXRSZXF1ZXN0OiAoKSA9PiBOV0NXYWxsZXRSZXF1ZXN0LFxuICBOV0NXYWxsZXRSZXNwb25zZTogKCkgPT4gTldDV2FsbGV0UmVzcG9uc2UsXG4gIE5vc3RyQ29ubmVjdDogKCkgPT4gTm9zdHJDb25uZWN0LFxuICBPcGVuVGltZXN0YW1wczogKCkgPT4gT3BlblRpbWVzdGFtcHMsXG4gIFBpbmxpc3Q6ICgpID0+IFBpbmxpc3QsXG4gIFByaXZhdGVEaXJlY3RNZXNzYWdlOiAoKSA9PiBQcml2YXRlRGlyZWN0TWVzc2FnZSxcbiAgUHJvYmxlbVRyYWNrZXI6ICgpID0+IFByb2JsZW1UcmFja2VyLFxuICBQcm9maWxlQmFkZ2VzOiAoKSA9PiBQcm9maWxlQmFkZ2VzLFxuICBQdWJsaWNDaGF0c0xpc3Q6ICgpID0+IFB1YmxpY0NoYXRzTGlzdCxcbiAgUmVhY3Rpb246ICgpID0+IFJlYWN0aW9uLFxuICBSZWNvbW1lbmRSZWxheTogKCkgPT4gUmVjb21tZW5kUmVsYXksXG4gIFJlbGF5TGlzdDogKCkgPT4gUmVsYXlMaXN0LFxuICBSZWxheXNldHM6ICgpID0+IFJlbGF5c2V0cyxcbiAgUmVwb3J0OiAoKSA9PiBSZXBvcnQsXG4gIFJlcG9ydGluZzogKCkgPT4gUmVwb3J0aW5nLFxuICBSZXBvc3Q6ICgpID0+IFJlcG9zdCxcbiAgU2VhbDogKCkgPT4gU2VhbCxcbiAgU2VhcmNoUmVsYXlzTGlzdDogKCkgPT4gU2VhcmNoUmVsYXlzTGlzdCxcbiAgU2hvcnRUZXh0Tm90ZTogKCkgPT4gU2hvcnRUZXh0Tm90ZSxcbiAgVGltZTogKCkgPT4gVGltZSxcbiAgVXNlckVtb2ppTGlzdDogKCkgPT4gVXNlckVtb2ppTGlzdCxcbiAgVXNlclN0YXR1c2VzOiAoKSA9PiBVc2VyU3RhdHVzZXMsXG4gIFphcDogKCkgPT4gWmFwLFxuICBaYXBHb2FsOiAoKSA9PiBaYXBHb2FsLFxuICBaYXBSZXF1ZXN0OiAoKSA9PiBaYXBSZXF1ZXN0LFxuICBjbGFzc2lmeUtpbmQ6ICgpID0+IGNsYXNzaWZ5S2luZCxcbiAgaXNBZGRyZXNzYWJsZUtpbmQ6ICgpID0+IGlzQWRkcmVzc2FibGVLaW5kLFxuICBpc0VwaGVtZXJhbEtpbmQ6ICgpID0+IGlzRXBoZW1lcmFsS2luZCxcbiAgaXNLaW5kOiAoKSA9PiBpc0tpbmQsXG4gIGlzUGFyYW1ldGVyaXplZFJlcGxhY2VhYmxlS2luZDogKCkgPT4gaXNQYXJhbWV0ZXJpemVkUmVwbGFjZWFibGVLaW5kLFxuICBpc1JlZ3VsYXJLaW5kOiAoKSA9PiBpc1JlZ3VsYXJLaW5kLFxuICBpc1JlcGxhY2VhYmxlS2luZDogKCkgPT4gaXNSZXBsYWNlYWJsZUtpbmRcbn0pO1xuZnVuY3Rpb24gaXNSZWd1bGFyS2luZChraW5kKSB7XG4gIHJldHVybiAxZTMgPD0ga2luZCAmJiBraW5kIDwgMWU0IHx8IFsxLCAyLCA0LCA1LCA2LCA3LCA4LCAxNiwgNDAsIDQxLCA0MiwgNDMsIDQ0XS5pbmNsdWRlcyhraW5kKTtcbn1cbmZ1bmN0aW9uIGlzUmVwbGFjZWFibGVLaW5kKGtpbmQpIHtcbiAgcmV0dXJuIFswLCAzXS5pbmNsdWRlcyhraW5kKSB8fCAxZTQgPD0ga2luZCAmJiBraW5kIDwgMmU0O1xufVxuZnVuY3Rpb24gaXNFcGhlbWVyYWxLaW5kKGtpbmQpIHtcbiAgcmV0dXJuIDJlNCA8PSBraW5kICYmIGtpbmQgPCAzZTQ7XG59XG5mdW5jdGlvbiBpc0FkZHJlc3NhYmxlS2luZChraW5kKSB7XG4gIHJldHVybiAzZTQgPD0ga2luZCAmJiBraW5kIDwgNGU0O1xufVxudmFyIGlzUGFyYW1ldGVyaXplZFJlcGxhY2VhYmxlS2luZCA9IGlzQWRkcmVzc2FibGVLaW5kO1xuZnVuY3Rpb24gY2xhc3NpZnlLaW5kKGtpbmQpIHtcbiAgaWYgKGlzUmVndWxhcktpbmQoa2luZCkpXG4gICAgcmV0dXJuIFwicmVndWxhclwiO1xuICBpZiAoaXNSZXBsYWNlYWJsZUtpbmQoa2luZCkpXG4gICAgcmV0dXJuIFwicmVwbGFjZWFibGVcIjtcbiAgaWYgKGlzRXBoZW1lcmFsS2luZChraW5kKSlcbiAgICByZXR1cm4gXCJlcGhlbWVyYWxcIjtcbiAgaWYgKGlzQWRkcmVzc2FibGVLaW5kKGtpbmQpKVxuICAgIHJldHVybiBcInBhcmFtZXRlcml6ZWRcIjtcbiAgcmV0dXJuIFwidW5rbm93blwiO1xufVxuZnVuY3Rpb24gaXNLaW5kKGV2ZW50LCBraW5kKSB7XG4gIGNvbnN0IGtpbmRBc0FycmF5ID0ga2luZCBpbnN0YW5jZW9mIEFycmF5ID8ga2luZCA6IFtraW5kXTtcbiAgcmV0dXJuIHZhbGlkYXRlRXZlbnQoZXZlbnQpICYmIGtpbmRBc0FycmF5LmluY2x1ZGVzKGV2ZW50LmtpbmQpIHx8IGZhbHNlO1xufVxudmFyIE1ldGFkYXRhID0gMDtcbnZhciBTaG9ydFRleHROb3RlID0gMTtcbnZhciBSZWNvbW1lbmRSZWxheSA9IDI7XG52YXIgQ29udGFjdHMgPSAzO1xudmFyIEVuY3J5cHRlZERpcmVjdE1lc3NhZ2UgPSA0O1xudmFyIEV2ZW50RGVsZXRpb24gPSA1O1xudmFyIFJlcG9zdCA9IDY7XG52YXIgUmVhY3Rpb24gPSA3O1xudmFyIEJhZGdlQXdhcmQgPSA4O1xudmFyIFNlYWwgPSAxMztcbnZhciBQcml2YXRlRGlyZWN0TWVzc2FnZSA9IDE0O1xudmFyIEdlbmVyaWNSZXBvc3QgPSAxNjtcbnZhciBDaGFubmVsQ3JlYXRpb24gPSA0MDtcbnZhciBDaGFubmVsTWV0YWRhdGEgPSA0MTtcbnZhciBDaGFubmVsTWVzc2FnZSA9IDQyO1xudmFyIENoYW5uZWxIaWRlTWVzc2FnZSA9IDQzO1xudmFyIENoYW5uZWxNdXRlVXNlciA9IDQ0O1xudmFyIE9wZW5UaW1lc3RhbXBzID0gMTA0MDtcbnZhciBHaWZ0V3JhcCA9IDEwNTk7XG52YXIgRmlsZU1ldGFkYXRhID0gMTA2MztcbnZhciBMaXZlQ2hhdE1lc3NhZ2UgPSAxMzExO1xudmFyIFByb2JsZW1UcmFja2VyID0gMTk3MTtcbnZhciBSZXBvcnQgPSAxOTg0O1xudmFyIFJlcG9ydGluZyA9IDE5ODQ7XG52YXIgTGFiZWwgPSAxOTg1O1xudmFyIENvbW11bml0eVBvc3RBcHByb3ZhbCA9IDQ1NTA7XG52YXIgSm9iUmVxdWVzdCA9IDU5OTk7XG52YXIgSm9iUmVzdWx0ID0gNjk5OTtcbnZhciBKb2JGZWVkYmFjayA9IDdlMztcbnZhciBaYXBHb2FsID0gOTA0MTtcbnZhciBaYXBSZXF1ZXN0ID0gOTczNDtcbnZhciBaYXAgPSA5NzM1O1xudmFyIEhpZ2hsaWdodHMgPSA5ODAyO1xudmFyIE11dGVsaXN0ID0gMWU0O1xudmFyIFBpbmxpc3QgPSAxMDAwMTtcbnZhciBSZWxheUxpc3QgPSAxMDAwMjtcbnZhciBCb29rbWFya0xpc3QgPSAxMDAwMztcbnZhciBDb21tdW5pdGllc0xpc3QgPSAxMDAwNDtcbnZhciBQdWJsaWNDaGF0c0xpc3QgPSAxMDAwNTtcbnZhciBCbG9ja2VkUmVsYXlzTGlzdCA9IDEwMDA2O1xudmFyIFNlYXJjaFJlbGF5c0xpc3QgPSAxMDAwNztcbnZhciBJbnRlcmVzdHNMaXN0ID0gMTAwMTU7XG52YXIgVXNlckVtb2ppTGlzdCA9IDEwMDMwO1xudmFyIERpcmVjdE1lc3NhZ2VSZWxheXNMaXN0ID0gMTAwNTA7XG52YXIgRmlsZVNlcnZlclByZWZlcmVuY2UgPSAxMDA5NjtcbnZhciBOV0NXYWxsZXRJbmZvID0gMTMxOTQ7XG52YXIgTGlnaHRuaW5nUHViUlBDID0gMjFlMztcbnZhciBDbGllbnRBdXRoID0gMjIyNDI7XG52YXIgTldDV2FsbGV0UmVxdWVzdCA9IDIzMTk0O1xudmFyIE5XQ1dhbGxldFJlc3BvbnNlID0gMjMxOTU7XG52YXIgTm9zdHJDb25uZWN0ID0gMjQxMzM7XG52YXIgSFRUUEF1dGggPSAyNzIzNTtcbnZhciBGb2xsb3dzZXRzID0gM2U0O1xudmFyIEdlbmVyaWNsaXN0cyA9IDMwMDAxO1xudmFyIFJlbGF5c2V0cyA9IDMwMDAyO1xudmFyIEJvb2ttYXJrc2V0cyA9IDMwMDAzO1xudmFyIEN1cmF0aW9uc2V0cyA9IDMwMDA0O1xudmFyIFByb2ZpbGVCYWRnZXMgPSAzMDAwODtcbnZhciBCYWRnZURlZmluaXRpb24gPSAzMDAwOTtcbnZhciBJbnRlcmVzdHNldHMgPSAzMDAxNTtcbnZhciBDcmVhdGVPclVwZGF0ZVN0YWxsID0gMzAwMTc7XG52YXIgQ3JlYXRlT3JVcGRhdGVQcm9kdWN0ID0gMzAwMTg7XG52YXIgTG9uZ0Zvcm1BcnRpY2xlID0gMzAwMjM7XG52YXIgRHJhZnRMb25nID0gMzAwMjQ7XG52YXIgRW1vamlzZXRzID0gMzAwMzA7XG52YXIgQXBwbGljYXRpb24gPSAzMDA3ODtcbnZhciBMaXZlRXZlbnQgPSAzMDMxMTtcbnZhciBVc2VyU3RhdHVzZXMgPSAzMDMxNTtcbnZhciBDbGFzc2lmaWVkTGlzdGluZyA9IDMwNDAyO1xudmFyIERyYWZ0Q2xhc3NpZmllZExpc3RpbmcgPSAzMDQwMztcbnZhciBEYXRlMiA9IDMxOTIyO1xudmFyIFRpbWUgPSAzMTkyMztcbnZhciBDYWxlbmRhciA9IDMxOTI0O1xudmFyIENhbGVuZGFyRXZlbnRSU1ZQID0gMzE5MjU7XG52YXIgSGFuZGxlcnJlY29tbWVuZGF0aW9uID0gMzE5ODk7XG52YXIgSGFuZGxlcmluZm9ybWF0aW9uID0gMzE5OTA7XG52YXIgQ29tbXVuaXR5RGVmaW5pdGlvbiA9IDM0NTUwO1xuXG4vLyBmaWx0ZXIudHNcbmZ1bmN0aW9uIG1hdGNoRmlsdGVyKGZpbHRlciwgZXZlbnQpIHtcbiAgaWYgKGZpbHRlci5pZHMgJiYgZmlsdGVyLmlkcy5pbmRleE9mKGV2ZW50LmlkKSA9PT0gLTEpIHtcbiAgICByZXR1cm4gZmFsc2U7XG4gIH1cbiAgaWYgKGZpbHRlci5raW5kcyAmJiBmaWx0ZXIua2luZHMuaW5kZXhPZihldmVudC5raW5kKSA9PT0gLTEpIHtcbiAgICByZXR1cm4gZmFsc2U7XG4gIH1cbiAgaWYgKGZpbHRlci5hdXRob3JzICYmIGZpbHRlci5hdXRob3JzLmluZGV4T2YoZXZlbnQucHVia2V5KSA9PT0gLTEpIHtcbiAgICByZXR1cm4gZmFsc2U7XG4gIH1cbiAgZm9yIChsZXQgZiBpbiBmaWx0ZXIpIHtcbiAgICBpZiAoZlswXSA9PT0gXCIjXCIpIHtcbiAgICAgIGxldCB0YWdOYW1lID0gZi5zbGljZSgxKTtcbiAgICAgIGxldCB2YWx1ZXMgPSBmaWx0ZXJbYCMke3RhZ05hbWV9YF07XG4gICAgICBpZiAodmFsdWVzICYmICFldmVudC50YWdzLmZpbmQoKFt0LCB2XSkgPT4gdCA9PT0gZi5zbGljZSgxKSAmJiB2YWx1ZXMuaW5kZXhPZih2KSAhPT0gLTEpKVxuICAgICAgICByZXR1cm4gZmFsc2U7XG4gICAgfVxuICB9XG4gIGlmIChmaWx0ZXIuc2luY2UgJiYgZXZlbnQuY3JlYXRlZF9hdCA8IGZpbHRlci5zaW5jZSlcbiAgICByZXR1cm4gZmFsc2U7XG4gIGlmIChmaWx0ZXIudW50aWwgJiYgZXZlbnQuY3JlYXRlZF9hdCA+IGZpbHRlci51bnRpbClcbiAgICByZXR1cm4gZmFsc2U7XG4gIHJldHVybiB0cnVlO1xufVxuZnVuY3Rpb24gbWF0Y2hGaWx0ZXJzKGZpbHRlcnMsIGV2ZW50KSB7XG4gIGZvciAobGV0IGkyID0gMDsgaTIgPCBmaWx0ZXJzLmxlbmd0aDsgaTIrKykge1xuICAgIGlmIChtYXRjaEZpbHRlcihmaWx0ZXJzW2kyXSwgZXZlbnQpKSB7XG4gICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9XG4gIH1cbiAgcmV0dXJuIGZhbHNlO1xufVxuZnVuY3Rpb24gbWVyZ2VGaWx0ZXJzKC4uLmZpbHRlcnMpIHtcbiAgbGV0IHJlc3VsdCA9IHt9O1xuICBmb3IgKGxldCBpMiA9IDA7IGkyIDwgZmlsdGVycy5sZW5ndGg7IGkyKyspIHtcbiAgICBsZXQgZmlsdGVyID0gZmlsdGVyc1tpMl07XG4gICAgT2JqZWN0LmVudHJpZXMoZmlsdGVyKS5mb3JFYWNoKChbcHJvcGVydHksIHZhbHVlc10pID0+IHtcbiAgICAgIGlmIChwcm9wZXJ0eSA9PT0gXCJraW5kc1wiIHx8IHByb3BlcnR5ID09PSBcImlkc1wiIHx8IHByb3BlcnR5ID09PSBcImF1dGhvcnNcIiB8fCBwcm9wZXJ0eVswXSA9PT0gXCIjXCIpIHtcbiAgICAgICAgcmVzdWx0W3Byb3BlcnR5XSA9IHJlc3VsdFtwcm9wZXJ0eV0gfHwgW107XG4gICAgICAgIGZvciAobGV0IHYgPSAwOyB2IDwgdmFsdWVzLmxlbmd0aDsgdisrKSB7XG4gICAgICAgICAgbGV0IHZhbHVlID0gdmFsdWVzW3ZdO1xuICAgICAgICAgIGlmICghcmVzdWx0W3Byb3BlcnR5XS5pbmNsdWRlcyh2YWx1ZSkpXG4gICAgICAgICAgICByZXN1bHRbcHJvcGVydHldLnB1c2godmFsdWUpO1xuICAgICAgICB9XG4gICAgICB9XG4gICAgfSk7XG4gICAgaWYgKGZpbHRlci5saW1pdCAmJiAoIXJlc3VsdC5saW1pdCB8fCBmaWx0ZXIubGltaXQgPiByZXN1bHQubGltaXQpKVxuICAgICAgcmVzdWx0LmxpbWl0ID0gZmlsdGVyLmxpbWl0O1xuICAgIGlmIChmaWx0ZXIudW50aWwgJiYgKCFyZXN1bHQudW50aWwgfHwgZmlsdGVyLnVudGlsID4gcmVzdWx0LnVudGlsKSlcbiAgICAgIHJlc3VsdC51bnRpbCA9IGZpbHRlci51bnRpbDtcbiAgICBpZiAoZmlsdGVyLnNpbmNlICYmICghcmVzdWx0LnNpbmNlIHx8IGZpbHRlci5zaW5jZSA8IHJlc3VsdC5zaW5jZSkpXG4gICAgICByZXN1bHQuc2luY2UgPSBmaWx0ZXIuc2luY2U7XG4gIH1cbiAgcmV0dXJuIHJlc3VsdDtcbn1cbmZ1bmN0aW9uIGdldEZpbHRlckxpbWl0KGZpbHRlcikge1xuICBpZiAoZmlsdGVyLmlkcyAmJiAhZmlsdGVyLmlkcy5sZW5ndGgpXG4gICAgcmV0dXJuIDA7XG4gIGlmIChmaWx0ZXIua2luZHMgJiYgIWZpbHRlci5raW5kcy5sZW5ndGgpXG4gICAgcmV0dXJuIDA7XG4gIGlmIChmaWx0ZXIuYXV0aG9ycyAmJiAhZmlsdGVyLmF1dGhvcnMubGVuZ3RoKVxuICAgIHJldHVybiAwO1xuICBmb3IgKGNvbnN0IFtrZXksIHZhbHVlXSBvZiBPYmplY3QuZW50cmllcyhmaWx0ZXIpKSB7XG4gICAgaWYgKGtleVswXSA9PT0gXCIjXCIgJiYgQXJyYXkuaXNBcnJheSh2YWx1ZSkgJiYgIXZhbHVlLmxlbmd0aClcbiAgICAgIHJldHVybiAwO1xuICB9XG4gIHJldHVybiBNYXRoLm1pbihcbiAgICBNYXRoLm1heCgwLCBmaWx0ZXIubGltaXQgPz8gSW5maW5pdHkpLFxuICAgIGZpbHRlci5pZHM/Lmxlbmd0aCA/PyBJbmZpbml0eSxcbiAgICBmaWx0ZXIuYXV0aG9ycz8ubGVuZ3RoICYmIGZpbHRlci5raW5kcz8uZXZlcnkoKGtpbmQpID0+IGlzUmVwbGFjZWFibGVLaW5kKGtpbmQpKSA/IGZpbHRlci5hdXRob3JzLmxlbmd0aCAqIGZpbHRlci5raW5kcy5sZW5ndGggOiBJbmZpbml0eSxcbiAgICBmaWx0ZXIuYXV0aG9ycz8ubGVuZ3RoICYmIGZpbHRlci5raW5kcz8uZXZlcnkoKGtpbmQpID0+IGlzQWRkcmVzc2FibGVLaW5kKGtpbmQpKSAmJiBmaWx0ZXJbXCIjZFwiXT8ubGVuZ3RoID8gZmlsdGVyLmF1dGhvcnMubGVuZ3RoICogZmlsdGVyLmtpbmRzLmxlbmd0aCAqIGZpbHRlcltcIiNkXCJdLmxlbmd0aCA6IEluZmluaXR5XG4gICk7XG59XG5cbi8vIGZha2Vqc29uLnRzXG52YXIgZmFrZWpzb25fZXhwb3J0cyA9IHt9O1xuX19leHBvcnQoZmFrZWpzb25fZXhwb3J0cywge1xuICBnZXRIZXg2NDogKCkgPT4gZ2V0SGV4NjQsXG4gIGdldEludDogKCkgPT4gZ2V0SW50LFxuICBnZXRTdWJzY3JpcHRpb25JZDogKCkgPT4gZ2V0U3Vic2NyaXB0aW9uSWQsXG4gIG1hdGNoRXZlbnRJZDogKCkgPT4gbWF0Y2hFdmVudElkLFxuICBtYXRjaEV2ZW50S2luZDogKCkgPT4gbWF0Y2hFdmVudEtpbmQsXG4gIG1hdGNoRXZlbnRQdWJrZXk6ICgpID0+IG1hdGNoRXZlbnRQdWJrZXlcbn0pO1xuZnVuY3Rpb24gZ2V0SGV4NjQoanNvbiwgZmllbGQpIHtcbiAgbGV0IGxlbiA9IGZpZWxkLmxlbmd0aCArIDM7XG4gIGxldCBpZHggPSBqc29uLmluZGV4T2YoYFwiJHtmaWVsZH1cIjpgKSArIGxlbjtcbiAgbGV0IHMgPSBqc29uLnNsaWNlKGlkeCkuaW5kZXhPZihgXCJgKSArIGlkeCArIDE7XG4gIHJldHVybiBqc29uLnNsaWNlKHMsIHMgKyA2NCk7XG59XG5mdW5jdGlvbiBnZXRJbnQoanNvbiwgZmllbGQpIHtcbiAgbGV0IGxlbiA9IGZpZWxkLmxlbmd0aDtcbiAgbGV0IGlkeCA9IGpzb24uaW5kZXhPZihgXCIke2ZpZWxkfVwiOmApICsgbGVuICsgMztcbiAgbGV0IHNsaWNlZCA9IGpzb24uc2xpY2UoaWR4KTtcbiAgbGV0IGVuZCA9IE1hdGgubWluKHNsaWNlZC5pbmRleE9mKFwiLFwiKSwgc2xpY2VkLmluZGV4T2YoXCJ9XCIpKTtcbiAgcmV0dXJuIHBhcnNlSW50KHNsaWNlZC5zbGljZSgwLCBlbmQpLCAxMCk7XG59XG5mdW5jdGlvbiBnZXRTdWJzY3JpcHRpb25JZChqc29uKSB7XG4gIGxldCBpZHggPSBqc29uLnNsaWNlKDAsIDIyKS5pbmRleE9mKGBcIkVWRU5UXCJgKTtcbiAgaWYgKGlkeCA9PT0gLTEpXG4gICAgcmV0dXJuIG51bGw7XG4gIGxldCBwc3RhcnQgPSBqc29uLnNsaWNlKGlkeCArIDcgKyAxKS5pbmRleE9mKGBcImApO1xuICBpZiAocHN0YXJ0ID09PSAtMSlcbiAgICByZXR1cm4gbnVsbDtcbiAgbGV0IHN0YXJ0ID0gaWR4ICsgNyArIDEgKyBwc3RhcnQ7XG4gIGxldCBwZW5kID0ganNvbi5zbGljZShzdGFydCArIDEsIDgwKS5pbmRleE9mKGBcImApO1xuICBpZiAocGVuZCA9PT0gLTEpXG4gICAgcmV0dXJuIG51bGw7XG4gIGxldCBlbmQgPSBzdGFydCArIDEgKyBwZW5kO1xuICByZXR1cm4ganNvbi5zbGljZShzdGFydCArIDEsIGVuZCk7XG59XG5mdW5jdGlvbiBtYXRjaEV2ZW50SWQoanNvbiwgaWQpIHtcbiAgcmV0dXJuIGlkID09PSBnZXRIZXg2NChqc29uLCBcImlkXCIpO1xufVxuZnVuY3Rpb24gbWF0Y2hFdmVudFB1YmtleShqc29uLCBwdWJrZXkpIHtcbiAgcmV0dXJuIHB1YmtleSA9PT0gZ2V0SGV4NjQoanNvbiwgXCJwdWJrZXlcIik7XG59XG5mdW5jdGlvbiBtYXRjaEV2ZW50S2luZChqc29uLCBraW5kKSB7XG4gIHJldHVybiBraW5kID09PSBnZXRJbnQoanNvbiwgXCJraW5kXCIpO1xufVxuXG4vLyBuaXA0Mi50c1xudmFyIG5pcDQyX2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDQyX2V4cG9ydHMsIHtcbiAgbWFrZUF1dGhFdmVudDogKCkgPT4gbWFrZUF1dGhFdmVudFxufSk7XG5mdW5jdGlvbiBtYWtlQXV0aEV2ZW50KHJlbGF5VVJMLCBjaGFsbGVuZ2UpIHtcbiAgcmV0dXJuIHtcbiAgICBraW5kOiBDbGllbnRBdXRoLFxuICAgIGNyZWF0ZWRfYXQ6IE1hdGguZmxvb3IoRGF0ZS5ub3coKSAvIDFlMyksXG4gICAgdGFnczogW1xuICAgICAgW1wicmVsYXlcIiwgcmVsYXlVUkxdLFxuICAgICAgW1wiY2hhbGxlbmdlXCIsIGNoYWxsZW5nZV1cbiAgICBdLFxuICAgIGNvbnRlbnQ6IFwiXCJcbiAgfTtcbn1cblxuLy8gaGVscGVycy50c1xuYXN5bmMgZnVuY3Rpb24geWllbGRUaHJlYWQoKSB7XG4gIHJldHVybiBuZXcgUHJvbWlzZSgocmVzb2x2ZSkgPT4ge1xuICAgIGNvbnN0IGNoID0gbmV3IE1lc3NhZ2VDaGFubmVsKCk7XG4gICAgY29uc3QgaGFuZGxlciA9ICgpID0+IHtcbiAgICAgIGNoLnBvcnQxLnJlbW92ZUV2ZW50TGlzdGVuZXIoXCJtZXNzYWdlXCIsIGhhbmRsZXIpO1xuICAgICAgcmVzb2x2ZSgpO1xuICAgIH07XG4gICAgY2gucG9ydDEuYWRkRXZlbnRMaXN0ZW5lcihcIm1lc3NhZ2VcIiwgaGFuZGxlcik7XG4gICAgY2gucG9ydDIucG9zdE1lc3NhZ2UoMCk7XG4gICAgY2gucG9ydDEuc3RhcnQoKTtcbiAgfSk7XG59XG52YXIgYWx3YXlzVHJ1ZSA9ICh0KSA9PiB7XG4gIHRbdmVyaWZpZWRTeW1ib2xdID0gdHJ1ZTtcbiAgcmV0dXJuIHRydWU7XG59O1xuXG4vLyBhYnN0cmFjdC1yZWxheS50c1xudmFyIEFic3RyYWN0UmVsYXkgPSBjbGFzcyB7XG4gIHVybDtcbiAgX2Nvbm5lY3RlZCA9IGZhbHNlO1xuICBvbmNsb3NlID0gbnVsbDtcbiAgb25ub3RpY2UgPSAobXNnKSA9PiBjb25zb2xlLmRlYnVnKGBOT1RJQ0UgZnJvbSAke3RoaXMudXJsfTogJHttc2d9YCk7XG4gIF9vbmF1dGggPSBudWxsO1xuICBiYXNlRW9zZVRpbWVvdXQgPSA0NDAwO1xuICBjb25uZWN0aW9uVGltZW91dCA9IDQ0MDA7XG4gIHB1Ymxpc2hUaW1lb3V0ID0gNDQwMDtcbiAgb3BlblN1YnMgPSAvKiBAX19QVVJFX18gKi8gbmV3IE1hcCgpO1xuICBjb25uZWN0aW9uVGltZW91dEhhbmRsZTtcbiAgY29ubmVjdGlvblByb21pc2U7XG4gIG9wZW5Db3VudFJlcXVlc3RzID0gLyogQF9fUFVSRV9fICovIG5ldyBNYXAoKTtcbiAgb3BlbkV2ZW50UHVibGlzaGVzID0gLyogQF9fUFVSRV9fICovIG5ldyBNYXAoKTtcbiAgd3M7XG4gIGluY29taW5nTWVzc2FnZVF1ZXVlID0gbmV3IFF1ZXVlKCk7XG4gIHF1ZXVlUnVubmluZyA9IGZhbHNlO1xuICBjaGFsbGVuZ2U7XG4gIGF1dGhQcm9taXNlO1xuICBzZXJpYWwgPSAwO1xuICB2ZXJpZnlFdmVudDtcbiAgX1dlYlNvY2tldDtcbiAgY29uc3RydWN0b3IodXJsLCBvcHRzKSB7XG4gICAgdGhpcy51cmwgPSBub3JtYWxpemVVUkwodXJsKTtcbiAgICB0aGlzLnZlcmlmeUV2ZW50ID0gb3B0cy52ZXJpZnlFdmVudDtcbiAgICB0aGlzLl9XZWJTb2NrZXQgPSBvcHRzLndlYnNvY2tldEltcGxlbWVudGF0aW9uIHx8IFdlYlNvY2tldDtcbiAgfVxuICBzdGF0aWMgYXN5bmMgY29ubmVjdCh1cmwsIG9wdHMpIHtcbiAgICBjb25zdCByZWxheSA9IG5ldyBBYnN0cmFjdFJlbGF5KHVybCwgb3B0cyk7XG4gICAgYXdhaXQgcmVsYXkuY29ubmVjdCgpO1xuICAgIHJldHVybiByZWxheTtcbiAgfVxuICBjbG9zZUFsbFN1YnNjcmlwdGlvbnMocmVhc29uKSB7XG4gICAgZm9yIChsZXQgW18sIHN1Yl0gb2YgdGhpcy5vcGVuU3Vicykge1xuICAgICAgc3ViLmNsb3NlKHJlYXNvbik7XG4gICAgfVxuICAgIHRoaXMub3BlblN1YnMuY2xlYXIoKTtcbiAgICBmb3IgKGxldCBbXywgZXBdIG9mIHRoaXMub3BlbkV2ZW50UHVibGlzaGVzKSB7XG4gICAgICBlcC5yZWplY3QobmV3IEVycm9yKHJlYXNvbikpO1xuICAgIH1cbiAgICB0aGlzLm9wZW5FdmVudFB1Ymxpc2hlcy5jbGVhcigpO1xuICAgIGZvciAobGV0IFtfLCBjcl0gb2YgdGhpcy5vcGVuQ291bnRSZXF1ZXN0cykge1xuICAgICAgY3IucmVqZWN0KG5ldyBFcnJvcihyZWFzb24pKTtcbiAgICB9XG4gICAgdGhpcy5vcGVuQ291bnRSZXF1ZXN0cy5jbGVhcigpO1xuICB9XG4gIGdldCBjb25uZWN0ZWQoKSB7XG4gICAgcmV0dXJuIHRoaXMuX2Nvbm5lY3RlZDtcbiAgfVxuICBhc3luYyBjb25uZWN0KCkge1xuICAgIGlmICh0aGlzLmNvbm5lY3Rpb25Qcm9taXNlKVxuICAgICAgcmV0dXJuIHRoaXMuY29ubmVjdGlvblByb21pc2U7XG4gICAgdGhpcy5jaGFsbGVuZ2UgPSB2b2lkIDA7XG4gICAgdGhpcy5hdXRoUHJvbWlzZSA9IHZvaWQgMDtcbiAgICB0aGlzLmNvbm5lY3Rpb25Qcm9taXNlID0gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4ge1xuICAgICAgdGhpcy5jb25uZWN0aW9uVGltZW91dEhhbmRsZSA9IHNldFRpbWVvdXQoKCkgPT4ge1xuICAgICAgICByZWplY3QoXCJjb25uZWN0aW9uIHRpbWVkIG91dFwiKTtcbiAgICAgICAgdGhpcy5jb25uZWN0aW9uUHJvbWlzZSA9IHZvaWQgMDtcbiAgICAgICAgdGhpcy5vbmNsb3NlPy4oKTtcbiAgICAgICAgdGhpcy5jbG9zZUFsbFN1YnNjcmlwdGlvbnMoXCJyZWxheSBjb25uZWN0aW9uIHRpbWVkIG91dFwiKTtcbiAgICAgIH0sIHRoaXMuY29ubmVjdGlvblRpbWVvdXQpO1xuICAgICAgdHJ5IHtcbiAgICAgICAgdGhpcy53cyA9IG5ldyB0aGlzLl9XZWJTb2NrZXQodGhpcy51cmwpO1xuICAgICAgfSBjYXRjaCAoZXJyKSB7XG4gICAgICAgIGNsZWFyVGltZW91dCh0aGlzLmNvbm5lY3Rpb25UaW1lb3V0SGFuZGxlKTtcbiAgICAgICAgcmVqZWN0KGVycik7XG4gICAgICAgIHJldHVybjtcbiAgICAgIH1cbiAgICAgIHRoaXMud3Mub25vcGVuID0gKCkgPT4ge1xuICAgICAgICBjbGVhclRpbWVvdXQodGhpcy5jb25uZWN0aW9uVGltZW91dEhhbmRsZSk7XG4gICAgICAgIHRoaXMuX2Nvbm5lY3RlZCA9IHRydWU7XG4gICAgICAgIHJlc29sdmUoKTtcbiAgICAgIH07XG4gICAgICB0aGlzLndzLm9uZXJyb3IgPSAoZXYpID0+IHtcbiAgICAgICAgY2xlYXJUaW1lb3V0KHRoaXMuY29ubmVjdGlvblRpbWVvdXRIYW5kbGUpO1xuICAgICAgICByZWplY3QoZXYubWVzc2FnZSB8fCBcIndlYnNvY2tldCBlcnJvclwiKTtcbiAgICAgICAgaWYgKHRoaXMuX2Nvbm5lY3RlZCkge1xuICAgICAgICAgIHRoaXMuX2Nvbm5lY3RlZCA9IGZhbHNlO1xuICAgICAgICAgIHRoaXMuY29ubmVjdGlvblByb21pc2UgPSB2b2lkIDA7XG4gICAgICAgICAgdGhpcy5vbmNsb3NlPy4oKTtcbiAgICAgICAgICB0aGlzLmNsb3NlQWxsU3Vic2NyaXB0aW9ucyhcInJlbGF5IGNvbm5lY3Rpb24gZXJyb3JlZFwiKTtcbiAgICAgICAgfVxuICAgICAgfTtcbiAgICAgIHRoaXMud3Mub25jbG9zZSA9IGFzeW5jICgpID0+IHtcbiAgICAgICAgaWYgKHRoaXMuX2Nvbm5lY3RlZCkge1xuICAgICAgICAgIHRoaXMuX2Nvbm5lY3RlZCA9IGZhbHNlO1xuICAgICAgICAgIHRoaXMuY29ubmVjdGlvblByb21pc2UgPSB2b2lkIDA7XG4gICAgICAgICAgdGhpcy5vbmNsb3NlPy4oKTtcbiAgICAgICAgICB0aGlzLmNsb3NlQWxsU3Vic2NyaXB0aW9ucyhcInJlbGF5IGNvbm5lY3Rpb24gY2xvc2VkXCIpO1xuICAgICAgICB9XG4gICAgICB9O1xuICAgICAgdGhpcy53cy5vbm1lc3NhZ2UgPSB0aGlzLl9vbm1lc3NhZ2UuYmluZCh0aGlzKTtcbiAgICB9KTtcbiAgICByZXR1cm4gdGhpcy5jb25uZWN0aW9uUHJvbWlzZTtcbiAgfVxuICBhc3luYyBydW5RdWV1ZSgpIHtcbiAgICB0aGlzLnF1ZXVlUnVubmluZyA9IHRydWU7XG4gICAgd2hpbGUgKHRydWUpIHtcbiAgICAgIGlmIChmYWxzZSA9PT0gdGhpcy5oYW5kbGVOZXh0KCkpIHtcbiAgICAgICAgYnJlYWs7XG4gICAgICB9XG4gICAgICBhd2FpdCB5aWVsZFRocmVhZCgpO1xuICAgIH1cbiAgICB0aGlzLnF1ZXVlUnVubmluZyA9IGZhbHNlO1xuICB9XG4gIGhhbmRsZU5leHQoKSB7XG4gICAgY29uc3QganNvbiA9IHRoaXMuaW5jb21pbmdNZXNzYWdlUXVldWUuZGVxdWV1ZSgpO1xuICAgIGlmICghanNvbikge1xuICAgICAgcmV0dXJuIGZhbHNlO1xuICAgIH1cbiAgICBjb25zdCBzdWJpZCA9IGdldFN1YnNjcmlwdGlvbklkKGpzb24pO1xuICAgIGlmIChzdWJpZCkge1xuICAgICAgY29uc3Qgc28gPSB0aGlzLm9wZW5TdWJzLmdldChzdWJpZCk7XG4gICAgICBpZiAoIXNvKSB7XG4gICAgICAgIHJldHVybjtcbiAgICAgIH1cbiAgICAgIGNvbnN0IGlkID0gZ2V0SGV4NjQoanNvbiwgXCJpZFwiKTtcbiAgICAgIGNvbnN0IGFscmVhZHlIYXZlID0gc28uYWxyZWFkeUhhdmVFdmVudD8uKGlkKTtcbiAgICAgIHNvLnJlY2VpdmVkRXZlbnQ/Lih0aGlzLCBpZCk7XG4gICAgICBpZiAoYWxyZWFkeUhhdmUpIHtcbiAgICAgICAgcmV0dXJuO1xuICAgICAgfVxuICAgIH1cbiAgICB0cnkge1xuICAgICAgbGV0IGRhdGEgPSBKU09OLnBhcnNlKGpzb24pO1xuICAgICAgc3dpdGNoIChkYXRhWzBdKSB7XG4gICAgICAgIGNhc2UgXCJFVkVOVFwiOiB7XG4gICAgICAgICAgY29uc3Qgc28gPSB0aGlzLm9wZW5TdWJzLmdldChkYXRhWzFdKTtcbiAgICAgICAgICBjb25zdCBldmVudCA9IGRhdGFbMl07XG4gICAgICAgICAgaWYgKHRoaXMudmVyaWZ5RXZlbnQoZXZlbnQpICYmIG1hdGNoRmlsdGVycyhzby5maWx0ZXJzLCBldmVudCkpIHtcbiAgICAgICAgICAgIHNvLm9uZXZlbnQoZXZlbnQpO1xuICAgICAgICAgIH1cbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcIkNPVU5UXCI6IHtcbiAgICAgICAgICBjb25zdCBpZCA9IGRhdGFbMV07XG4gICAgICAgICAgY29uc3QgcGF5bG9hZCA9IGRhdGFbMl07XG4gICAgICAgICAgY29uc3QgY3IgPSB0aGlzLm9wZW5Db3VudFJlcXVlc3RzLmdldChpZCk7XG4gICAgICAgICAgaWYgKGNyKSB7XG4gICAgICAgICAgICBjci5yZXNvbHZlKHBheWxvYWQuY291bnQpO1xuICAgICAgICAgICAgdGhpcy5vcGVuQ291bnRSZXF1ZXN0cy5kZWxldGUoaWQpO1xuICAgICAgICAgIH1cbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcIkVPU0VcIjoge1xuICAgICAgICAgIGNvbnN0IHNvID0gdGhpcy5vcGVuU3Vicy5nZXQoZGF0YVsxXSk7XG4gICAgICAgICAgaWYgKCFzbylcbiAgICAgICAgICAgIHJldHVybjtcbiAgICAgICAgICBzby5yZWNlaXZlZEVvc2UoKTtcbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcIk9LXCI6IHtcbiAgICAgICAgICBjb25zdCBpZCA9IGRhdGFbMV07XG4gICAgICAgICAgY29uc3Qgb2sgPSBkYXRhWzJdO1xuICAgICAgICAgIGNvbnN0IHJlYXNvbiA9IGRhdGFbM107XG4gICAgICAgICAgY29uc3QgZXAgPSB0aGlzLm9wZW5FdmVudFB1Ymxpc2hlcy5nZXQoaWQpO1xuICAgICAgICAgIGlmIChlcCkge1xuICAgICAgICAgICAgY2xlYXJUaW1lb3V0KGVwLnRpbWVvdXQpO1xuICAgICAgICAgICAgaWYgKG9rKVxuICAgICAgICAgICAgICBlcC5yZXNvbHZlKHJlYXNvbik7XG4gICAgICAgICAgICBlbHNlXG4gICAgICAgICAgICAgIGVwLnJlamVjdChuZXcgRXJyb3IocmVhc29uKSk7XG4gICAgICAgICAgICB0aGlzLm9wZW5FdmVudFB1Ymxpc2hlcy5kZWxldGUoaWQpO1xuICAgICAgICAgIH1cbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcIkNMT1NFRFwiOiB7XG4gICAgICAgICAgY29uc3QgaWQgPSBkYXRhWzFdO1xuICAgICAgICAgIGNvbnN0IHNvID0gdGhpcy5vcGVuU3Vicy5nZXQoaWQpO1xuICAgICAgICAgIGlmICghc28pXG4gICAgICAgICAgICByZXR1cm47XG4gICAgICAgICAgc28uY2xvc2VkID0gdHJ1ZTtcbiAgICAgICAgICBzby5jbG9zZShkYXRhWzJdKTtcbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcIk5PVElDRVwiOlxuICAgICAgICAgIHRoaXMub25ub3RpY2UoZGF0YVsxXSk7XG4gICAgICAgICAgcmV0dXJuO1xuICAgICAgICBjYXNlIFwiQVVUSFwiOiB7XG4gICAgICAgICAgdGhpcy5jaGFsbGVuZ2UgPSBkYXRhWzFdO1xuICAgICAgICAgIHRoaXMuX29uYXV0aD8uKGRhdGFbMV0pO1xuICAgICAgICAgIHJldHVybjtcbiAgICAgICAgfVxuICAgICAgfVxuICAgIH0gY2F0Y2ggKGVycikge1xuICAgICAgcmV0dXJuO1xuICAgIH1cbiAgfVxuICBhc3luYyBzZW5kKG1lc3NhZ2UpIHtcbiAgICBpZiAoIXRoaXMuY29ubmVjdGlvblByb21pc2UpXG4gICAgICB0aHJvdyBuZXcgRXJyb3IoXCJzZW5kaW5nIG9uIGNsb3NlZCBjb25uZWN0aW9uXCIpO1xuICAgIHRoaXMuY29ubmVjdGlvblByb21pc2UudGhlbigoKSA9PiB7XG4gICAgICB0aGlzLndzPy5zZW5kKG1lc3NhZ2UpO1xuICAgIH0pO1xuICB9XG4gIGFzeW5jIGF1dGgoc2lnbkF1dGhFdmVudCkge1xuICAgIGNvbnN0IGNoYWxsZW5nZSA9IHRoaXMuY2hhbGxlbmdlO1xuICAgIGlmICghY2hhbGxlbmdlKVxuICAgICAgdGhyb3cgbmV3IEVycm9yKFwiY2FuJ3QgcGVyZm9ybSBhdXRoLCBubyBjaGFsbGVuZ2Ugd2FzIHJlY2VpdmVkXCIpO1xuICAgIGlmICh0aGlzLmF1dGhQcm9taXNlKVxuICAgICAgcmV0dXJuIHRoaXMuYXV0aFByb21pc2U7XG4gICAgdGhpcy5hdXRoUHJvbWlzZSA9IG5ldyBQcm9taXNlKGFzeW5jIChyZXNvbHZlLCByZWplY3QpID0+IHtcbiAgICAgIGNvbnN0IGV2dCA9IGF3YWl0IHNpZ25BdXRoRXZlbnQobWFrZUF1dGhFdmVudCh0aGlzLnVybCwgY2hhbGxlbmdlKSk7XG4gICAgICBjb25zdCB0aW1lb3V0ID0gc2V0VGltZW91dCgoKSA9PiB7XG4gICAgICAgIGNvbnN0IGVwID0gdGhpcy5vcGVuRXZlbnRQdWJsaXNoZXMuZ2V0KGV2dC5pZCk7XG4gICAgICAgIGlmIChlcCkge1xuICAgICAgICAgIGVwLnJlamVjdChuZXcgRXJyb3IoXCJhdXRoIHRpbWVkIG91dFwiKSk7XG4gICAgICAgICAgdGhpcy5vcGVuRXZlbnRQdWJsaXNoZXMuZGVsZXRlKGV2dC5pZCk7XG4gICAgICAgIH1cbiAgICAgIH0sIHRoaXMucHVibGlzaFRpbWVvdXQpO1xuICAgICAgdGhpcy5vcGVuRXZlbnRQdWJsaXNoZXMuc2V0KGV2dC5pZCwgeyByZXNvbHZlLCByZWplY3QsIHRpbWVvdXQgfSk7XG4gICAgICB0aGlzLnNlbmQoJ1tcIkFVVEhcIiwnICsgSlNPTi5zdHJpbmdpZnkoZXZ0KSArIFwiXVwiKTtcbiAgICB9KTtcbiAgICByZXR1cm4gdGhpcy5hdXRoUHJvbWlzZTtcbiAgfVxuICBhc3luYyBwdWJsaXNoKGV2ZW50KSB7XG4gICAgY29uc3QgcmV0ID0gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4ge1xuICAgICAgY29uc3QgdGltZW91dCA9IHNldFRpbWVvdXQoKCkgPT4ge1xuICAgICAgICBjb25zdCBlcCA9IHRoaXMub3BlbkV2ZW50UHVibGlzaGVzLmdldChldmVudC5pZCk7XG4gICAgICAgIGlmIChlcCkge1xuICAgICAgICAgIGVwLnJlamVjdChuZXcgRXJyb3IoXCJwdWJsaXNoIHRpbWVkIG91dFwiKSk7XG4gICAgICAgICAgdGhpcy5vcGVuRXZlbnRQdWJsaXNoZXMuZGVsZXRlKGV2ZW50LmlkKTtcbiAgICAgICAgfVxuICAgICAgfSwgdGhpcy5wdWJsaXNoVGltZW91dCk7XG4gICAgICB0aGlzLm9wZW5FdmVudFB1Ymxpc2hlcy5zZXQoZXZlbnQuaWQsIHsgcmVzb2x2ZSwgcmVqZWN0LCB0aW1lb3V0IH0pO1xuICAgIH0pO1xuICAgIHRoaXMuc2VuZCgnW1wiRVZFTlRcIiwnICsgSlNPTi5zdHJpbmdpZnkoZXZlbnQpICsgXCJdXCIpO1xuICAgIHJldHVybiByZXQ7XG4gIH1cbiAgYXN5bmMgY291bnQoZmlsdGVycywgcGFyYW1zKSB7XG4gICAgdGhpcy5zZXJpYWwrKztcbiAgICBjb25zdCBpZCA9IHBhcmFtcz8uaWQgfHwgXCJjb3VudDpcIiArIHRoaXMuc2VyaWFsO1xuICAgIGNvbnN0IHJldCA9IG5ldyBQcm9taXNlKChyZXNvbHZlLCByZWplY3QpID0+IHtcbiAgICAgIHRoaXMub3BlbkNvdW50UmVxdWVzdHMuc2V0KGlkLCB7IHJlc29sdmUsIHJlamVjdCB9KTtcbiAgICB9KTtcbiAgICB0aGlzLnNlbmQoJ1tcIkNPVU5UXCIsXCInICsgaWQgKyAnXCIsJyArIEpTT04uc3RyaW5naWZ5KGZpbHRlcnMpLnN1YnN0cmluZygxKSk7XG4gICAgcmV0dXJuIHJldDtcbiAgfVxuICBzdWJzY3JpYmUoZmlsdGVycywgcGFyYW1zKSB7XG4gICAgY29uc3Qgc3Vic2NyaXB0aW9uID0gdGhpcy5wcmVwYXJlU3Vic2NyaXB0aW9uKGZpbHRlcnMsIHBhcmFtcyk7XG4gICAgc3Vic2NyaXB0aW9uLmZpcmUoKTtcbiAgICByZXR1cm4gc3Vic2NyaXB0aW9uO1xuICB9XG4gIHByZXBhcmVTdWJzY3JpcHRpb24oZmlsdGVycywgcGFyYW1zKSB7XG4gICAgdGhpcy5zZXJpYWwrKztcbiAgICBjb25zdCBpZCA9IHBhcmFtcy5pZCB8fCAocGFyYW1zLmxhYmVsID8gcGFyYW1zLmxhYmVsICsgXCI6XCIgOiBcInN1YjpcIikgKyB0aGlzLnNlcmlhbDtcbiAgICBjb25zdCBzdWJzY3JpcHRpb24gPSBuZXcgU3Vic2NyaXB0aW9uKHRoaXMsIGlkLCBmaWx0ZXJzLCBwYXJhbXMpO1xuICAgIHRoaXMub3BlblN1YnMuc2V0KGlkLCBzdWJzY3JpcHRpb24pO1xuICAgIHJldHVybiBzdWJzY3JpcHRpb247XG4gIH1cbiAgY2xvc2UoKSB7XG4gICAgdGhpcy5jbG9zZUFsbFN1YnNjcmlwdGlvbnMoXCJyZWxheSBjb25uZWN0aW9uIGNsb3NlZCBieSB1c1wiKTtcbiAgICB0aGlzLl9jb25uZWN0ZWQgPSBmYWxzZTtcbiAgICB0aGlzLndzPy5jbG9zZSgpO1xuICB9XG4gIF9vbm1lc3NhZ2UoZXYpIHtcbiAgICB0aGlzLmluY29taW5nTWVzc2FnZVF1ZXVlLmVucXVldWUoZXYuZGF0YSk7XG4gICAgaWYgKCF0aGlzLnF1ZXVlUnVubmluZykge1xuICAgICAgdGhpcy5ydW5RdWV1ZSgpO1xuICAgIH1cbiAgfVxufTtcbnZhciBTdWJzY3JpcHRpb24gPSBjbGFzcyB7XG4gIHJlbGF5O1xuICBpZDtcbiAgY2xvc2VkID0gZmFsc2U7XG4gIGVvc2VkID0gZmFsc2U7XG4gIGZpbHRlcnM7XG4gIGFscmVhZHlIYXZlRXZlbnQ7XG4gIHJlY2VpdmVkRXZlbnQ7XG4gIG9uZXZlbnQ7XG4gIG9uZW9zZTtcbiAgb25jbG9zZTtcbiAgZW9zZVRpbWVvdXQ7XG4gIGVvc2VUaW1lb3V0SGFuZGxlO1xuICBjb25zdHJ1Y3RvcihyZWxheSwgaWQsIGZpbHRlcnMsIHBhcmFtcykge1xuICAgIHRoaXMucmVsYXkgPSByZWxheTtcbiAgICB0aGlzLmZpbHRlcnMgPSBmaWx0ZXJzO1xuICAgIHRoaXMuaWQgPSBpZDtcbiAgICB0aGlzLmFscmVhZHlIYXZlRXZlbnQgPSBwYXJhbXMuYWxyZWFkeUhhdmVFdmVudDtcbiAgICB0aGlzLnJlY2VpdmVkRXZlbnQgPSBwYXJhbXMucmVjZWl2ZWRFdmVudDtcbiAgICB0aGlzLmVvc2VUaW1lb3V0ID0gcGFyYW1zLmVvc2VUaW1lb3V0IHx8IHJlbGF5LmJhc2VFb3NlVGltZW91dDtcbiAgICB0aGlzLm9uZW9zZSA9IHBhcmFtcy5vbmVvc2U7XG4gICAgdGhpcy5vbmNsb3NlID0gcGFyYW1zLm9uY2xvc2U7XG4gICAgdGhpcy5vbmV2ZW50ID0gcGFyYW1zLm9uZXZlbnQgfHwgKChldmVudCkgPT4ge1xuICAgICAgY29uc29sZS53YXJuKFxuICAgICAgICBgb25ldmVudCgpIGNhbGxiYWNrIG5vdCBkZWZpbmVkIGZvciBzdWJzY3JpcHRpb24gJyR7dGhpcy5pZH0nIGluIHJlbGF5ICR7dGhpcy5yZWxheS51cmx9LiBldmVudCByZWNlaXZlZDpgLFxuICAgICAgICBldmVudFxuICAgICAgKTtcbiAgICB9KTtcbiAgfVxuICBmaXJlKCkge1xuICAgIHRoaXMucmVsYXkuc2VuZCgnW1wiUkVRXCIsXCInICsgdGhpcy5pZCArICdcIiwnICsgSlNPTi5zdHJpbmdpZnkodGhpcy5maWx0ZXJzKS5zdWJzdHJpbmcoMSkpO1xuICAgIHRoaXMuZW9zZVRpbWVvdXRIYW5kbGUgPSBzZXRUaW1lb3V0KHRoaXMucmVjZWl2ZWRFb3NlLmJpbmQodGhpcyksIHRoaXMuZW9zZVRpbWVvdXQpO1xuICB9XG4gIHJlY2VpdmVkRW9zZSgpIHtcbiAgICBpZiAodGhpcy5lb3NlZClcbiAgICAgIHJldHVybjtcbiAgICBjbGVhclRpbWVvdXQodGhpcy5lb3NlVGltZW91dEhhbmRsZSk7XG4gICAgdGhpcy5lb3NlZCA9IHRydWU7XG4gICAgdGhpcy5vbmVvc2U/LigpO1xuICB9XG4gIGNsb3NlKHJlYXNvbiA9IFwiY2xvc2VkIGJ5IGNhbGxlclwiKSB7XG4gICAgaWYgKCF0aGlzLmNsb3NlZCAmJiB0aGlzLnJlbGF5LmNvbm5lY3RlZCkge1xuICAgICAgdGhpcy5yZWxheS5zZW5kKCdbXCJDTE9TRVwiLCcgKyBKU09OLnN0cmluZ2lmeSh0aGlzLmlkKSArIFwiXVwiKTtcbiAgICAgIHRoaXMuY2xvc2VkID0gdHJ1ZTtcbiAgICB9XG4gICAgdGhpcy5yZWxheS5vcGVuU3Vicy5kZWxldGUodGhpcy5pZCk7XG4gICAgdGhpcy5vbmNsb3NlPy4ocmVhc29uKTtcbiAgfVxufTtcblxuLy8gcmVsYXkudHNcbnZhciBfV2ViU29ja2V0O1xudHJ5IHtcbiAgX1dlYlNvY2tldCA9IFdlYlNvY2tldDtcbn0gY2F0Y2gge1xufVxudmFyIFJlbGF5ID0gY2xhc3MgZXh0ZW5kcyBBYnN0cmFjdFJlbGF5IHtcbiAgY29uc3RydWN0b3IodXJsKSB7XG4gICAgc3VwZXIodXJsLCB7IHZlcmlmeUV2ZW50LCB3ZWJzb2NrZXRJbXBsZW1lbnRhdGlvbjogX1dlYlNvY2tldCB9KTtcbiAgfVxuICBzdGF0aWMgYXN5bmMgY29ubmVjdCh1cmwpIHtcbiAgICBjb25zdCByZWxheSA9IG5ldyBSZWxheSh1cmwpO1xuICAgIGF3YWl0IHJlbGF5LmNvbm5lY3QoKTtcbiAgICByZXR1cm4gcmVsYXk7XG4gIH1cbn07XG5cbi8vIGFic3RyYWN0LXBvb2wudHNcbnZhciBBYnN0cmFjdFNpbXBsZVBvb2wgPSBjbGFzcyB7XG4gIHJlbGF5cyA9IC8qIEBfX1BVUkVfXyAqLyBuZXcgTWFwKCk7XG4gIHNlZW5PbiA9IC8qIEBfX1BVUkVfXyAqLyBuZXcgTWFwKCk7XG4gIHRyYWNrUmVsYXlzID0gZmFsc2U7XG4gIHZlcmlmeUV2ZW50O1xuICB0cnVzdGVkUmVsYXlVUkxzID0gLyogQF9fUFVSRV9fICovIG5ldyBTZXQoKTtcbiAgX1dlYlNvY2tldDtcbiAgY29uc3RydWN0b3Iob3B0cykge1xuICAgIHRoaXMudmVyaWZ5RXZlbnQgPSBvcHRzLnZlcmlmeUV2ZW50O1xuICAgIHRoaXMuX1dlYlNvY2tldCA9IG9wdHMud2Vic29ja2V0SW1wbGVtZW50YXRpb247XG4gIH1cbiAgYXN5bmMgZW5zdXJlUmVsYXkodXJsLCBwYXJhbXMpIHtcbiAgICB1cmwgPSBub3JtYWxpemVVUkwodXJsKTtcbiAgICBsZXQgcmVsYXkgPSB0aGlzLnJlbGF5cy5nZXQodXJsKTtcbiAgICBpZiAoIXJlbGF5KSB7XG4gICAgICByZWxheSA9IG5ldyBBYnN0cmFjdFJlbGF5KHVybCwge1xuICAgICAgICB2ZXJpZnlFdmVudDogdGhpcy50cnVzdGVkUmVsYXlVUkxzLmhhcyh1cmwpID8gYWx3YXlzVHJ1ZSA6IHRoaXMudmVyaWZ5RXZlbnQsXG4gICAgICAgIHdlYnNvY2tldEltcGxlbWVudGF0aW9uOiB0aGlzLl9XZWJTb2NrZXRcbiAgICAgIH0pO1xuICAgICAgaWYgKHBhcmFtcz8uY29ubmVjdGlvblRpbWVvdXQpXG4gICAgICAgIHJlbGF5LmNvbm5lY3Rpb25UaW1lb3V0ID0gcGFyYW1zLmNvbm5lY3Rpb25UaW1lb3V0O1xuICAgICAgdGhpcy5yZWxheXMuc2V0KHVybCwgcmVsYXkpO1xuICAgIH1cbiAgICBhd2FpdCByZWxheS5jb25uZWN0KCk7XG4gICAgcmV0dXJuIHJlbGF5O1xuICB9XG4gIGNsb3NlKHJlbGF5cykge1xuICAgIHJlbGF5cy5tYXAobm9ybWFsaXplVVJMKS5mb3JFYWNoKCh1cmwpID0+IHtcbiAgICAgIHRoaXMucmVsYXlzLmdldCh1cmwpPy5jbG9zZSgpO1xuICAgIH0pO1xuICB9XG4gIHN1YnNjcmliZShyZWxheXMsIGZpbHRlciwgcGFyYW1zKSB7XG4gICAgcmV0dXJuIHRoaXMuc3Vic2NyaWJlTWFwKFxuICAgICAgcmVsYXlzLm1hcCgodXJsKSA9PiAoeyB1cmwsIGZpbHRlciB9KSksXG4gICAgICBwYXJhbXNcbiAgICApO1xuICB9XG4gIHN1YnNjcmliZU1hbnkocmVsYXlzLCBmaWx0ZXJzLCBwYXJhbXMpIHtcbiAgICByZXR1cm4gdGhpcy5zdWJzY3JpYmVNYXAoXG4gICAgICByZWxheXMuZmxhdE1hcCgodXJsKSA9PiBmaWx0ZXJzLm1hcCgoZmlsdGVyKSA9PiAoeyB1cmwsIGZpbHRlciB9KSkpLFxuICAgICAgcGFyYW1zXG4gICAgKTtcbiAgfVxuICBzdWJzY3JpYmVNYXAocmVxdWVzdHMsIHBhcmFtcykge1xuICAgIGlmICh0aGlzLnRyYWNrUmVsYXlzKSB7XG4gICAgICBwYXJhbXMucmVjZWl2ZWRFdmVudCA9IChyZWxheSwgaWQpID0+IHtcbiAgICAgICAgbGV0IHNldCA9IHRoaXMuc2Vlbk9uLmdldChpZCk7XG4gICAgICAgIGlmICghc2V0KSB7XG4gICAgICAgICAgc2V0ID0gLyogQF9fUFVSRV9fICovIG5ldyBTZXQoKTtcbiAgICAgICAgICB0aGlzLnNlZW5Pbi5zZXQoaWQsIHNldCk7XG4gICAgICAgIH1cbiAgICAgICAgc2V0LmFkZChyZWxheSk7XG4gICAgICB9O1xuICAgIH1cbiAgICBjb25zdCBfa25vd25JZHMgPSAvKiBAX19QVVJFX18gKi8gbmV3IFNldCgpO1xuICAgIGNvbnN0IHN1YnMgPSBbXTtcbiAgICBjb25zdCBlb3Nlc1JlY2VpdmVkID0gW107XG4gICAgbGV0IGhhbmRsZUVvc2UgPSAoaTIpID0+IHtcbiAgICAgIGlmIChlb3Nlc1JlY2VpdmVkW2kyXSlcbiAgICAgICAgcmV0dXJuO1xuICAgICAgZW9zZXNSZWNlaXZlZFtpMl0gPSB0cnVlO1xuICAgICAgaWYgKGVvc2VzUmVjZWl2ZWQuZmlsdGVyKChhKSA9PiBhKS5sZW5ndGggPT09IHJlcXVlc3RzLmxlbmd0aCkge1xuICAgICAgICBwYXJhbXMub25lb3NlPy4oKTtcbiAgICAgICAgaGFuZGxlRW9zZSA9ICgpID0+IHtcbiAgICAgICAgfTtcbiAgICAgIH1cbiAgICB9O1xuICAgIGNvbnN0IGNsb3Nlc1JlY2VpdmVkID0gW107XG4gICAgbGV0IGhhbmRsZUNsb3NlID0gKGkyLCByZWFzb24pID0+IHtcbiAgICAgIGlmIChjbG9zZXNSZWNlaXZlZFtpMl0pXG4gICAgICAgIHJldHVybjtcbiAgICAgIGhhbmRsZUVvc2UoaTIpO1xuICAgICAgY2xvc2VzUmVjZWl2ZWRbaTJdID0gcmVhc29uO1xuICAgICAgaWYgKGNsb3Nlc1JlY2VpdmVkLmZpbHRlcigoYSkgPT4gYSkubGVuZ3RoID09PSByZXF1ZXN0cy5sZW5ndGgpIHtcbiAgICAgICAgcGFyYW1zLm9uY2xvc2U/LihjbG9zZXNSZWNlaXZlZCk7XG4gICAgICAgIGhhbmRsZUNsb3NlID0gKCkgPT4ge1xuICAgICAgICB9O1xuICAgICAgfVxuICAgIH07XG4gICAgY29uc3QgbG9jYWxBbHJlYWR5SGF2ZUV2ZW50SGFuZGxlciA9IChpZCkgPT4ge1xuICAgICAgaWYgKHBhcmFtcy5hbHJlYWR5SGF2ZUV2ZW50Py4oaWQpKSB7XG4gICAgICAgIHJldHVybiB0cnVlO1xuICAgICAgfVxuICAgICAgY29uc3QgaGF2ZSA9IF9rbm93bklkcy5oYXMoaWQpO1xuICAgICAgX2tub3duSWRzLmFkZChpZCk7XG4gICAgICByZXR1cm4gaGF2ZTtcbiAgICB9O1xuICAgIGNvbnN0IGFsbE9wZW5lZCA9IFByb21pc2UuYWxsKFxuICAgICAgcmVxdWVzdHMubWFwKGFzeW5jICh7IHVybCwgZmlsdGVyIH0sIGkyKSA9PiB7XG4gICAgICAgIHVybCA9IG5vcm1hbGl6ZVVSTCh1cmwpO1xuICAgICAgICBsZXQgcmVsYXk7XG4gICAgICAgIHRyeSB7XG4gICAgICAgICAgcmVsYXkgPSBhd2FpdCB0aGlzLmVuc3VyZVJlbGF5KHVybCwge1xuICAgICAgICAgICAgY29ubmVjdGlvblRpbWVvdXQ6IHBhcmFtcy5tYXhXYWl0ID8gTWF0aC5tYXgocGFyYW1zLm1heFdhaXQgKiAwLjgsIHBhcmFtcy5tYXhXYWl0IC0gMWUzKSA6IHZvaWQgMFxuICAgICAgICAgIH0pO1xuICAgICAgICB9IGNhdGNoIChlcnIpIHtcbiAgICAgICAgICBoYW5kbGVDbG9zZShpMiwgZXJyPy5tZXNzYWdlIHx8IFN0cmluZyhlcnIpKTtcbiAgICAgICAgICByZXR1cm47XG4gICAgICAgIH1cbiAgICAgICAgbGV0IHN1YnNjcmlwdGlvbiA9IHJlbGF5LnN1YnNjcmliZShbZmlsdGVyXSwge1xuICAgICAgICAgIC4uLnBhcmFtcyxcbiAgICAgICAgICBvbmVvc2U6ICgpID0+IGhhbmRsZUVvc2UoaTIpLFxuICAgICAgICAgIG9uY2xvc2U6IChyZWFzb24pID0+IHtcbiAgICAgICAgICAgIGlmIChyZWFzb24uc3RhcnRzV2l0aChcImF1dGgtcmVxdWlyZWQ6XCIpICYmIHBhcmFtcy5kb2F1dGgpIHtcbiAgICAgICAgICAgICAgcmVsYXkuYXV0aChwYXJhbXMuZG9hdXRoKS50aGVuKCgpID0+IHtcbiAgICAgICAgICAgICAgICByZWxheS5zdWJzY3JpYmUoW2ZpbHRlcl0sIHtcbiAgICAgICAgICAgICAgICAgIC4uLnBhcmFtcyxcbiAgICAgICAgICAgICAgICAgIG9uZW9zZTogKCkgPT4gaGFuZGxlRW9zZShpMiksXG4gICAgICAgICAgICAgICAgICBvbmNsb3NlOiAocmVhc29uMikgPT4ge1xuICAgICAgICAgICAgICAgICAgICBoYW5kbGVDbG9zZShpMiwgcmVhc29uMik7XG4gICAgICAgICAgICAgICAgICB9LFxuICAgICAgICAgICAgICAgICAgYWxyZWFkeUhhdmVFdmVudDogbG9jYWxBbHJlYWR5SGF2ZUV2ZW50SGFuZGxlcixcbiAgICAgICAgICAgICAgICAgIGVvc2VUaW1lb3V0OiBwYXJhbXMubWF4V2FpdFxuICAgICAgICAgICAgICAgIH0pO1xuICAgICAgICAgICAgICB9KS5jYXRjaCgoZXJyKSA9PiB7XG4gICAgICAgICAgICAgICAgaGFuZGxlQ2xvc2UoaTIsIGBhdXRoIHdhcyByZXF1aXJlZCBhbmQgYXR0ZW1wdGVkLCBidXQgZmFpbGVkIHdpdGg6ICR7ZXJyfWApO1xuICAgICAgICAgICAgICB9KTtcbiAgICAgICAgICAgIH0gZWxzZSB7XG4gICAgICAgICAgICAgIGhhbmRsZUNsb3NlKGkyLCByZWFzb24pO1xuICAgICAgICAgICAgfVxuICAgICAgICAgIH0sXG4gICAgICAgICAgYWxyZWFkeUhhdmVFdmVudDogbG9jYWxBbHJlYWR5SGF2ZUV2ZW50SGFuZGxlcixcbiAgICAgICAgICBlb3NlVGltZW91dDogcGFyYW1zLm1heFdhaXRcbiAgICAgICAgfSk7XG4gICAgICAgIHN1YnMucHVzaChzdWJzY3JpcHRpb24pO1xuICAgICAgfSlcbiAgICApO1xuICAgIHJldHVybiB7XG4gICAgICBhc3luYyBjbG9zZSgpIHtcbiAgICAgICAgYXdhaXQgYWxsT3BlbmVkO1xuICAgICAgICBzdWJzLmZvckVhY2goKHN1YikgPT4ge1xuICAgICAgICAgIHN1Yi5jbG9zZSgpO1xuICAgICAgICB9KTtcbiAgICAgIH1cbiAgICB9O1xuICB9XG4gIHN1YnNjcmliZUVvc2UocmVsYXlzLCBmaWx0ZXIsIHBhcmFtcykge1xuICAgIGNvbnN0IHN1YmNsb3NlciA9IHRoaXMuc3Vic2NyaWJlKHJlbGF5cywgZmlsdGVyLCB7XG4gICAgICAuLi5wYXJhbXMsXG4gICAgICBvbmVvc2UoKSB7XG4gICAgICAgIHN1YmNsb3Nlci5jbG9zZSgpO1xuICAgICAgfVxuICAgIH0pO1xuICAgIHJldHVybiBzdWJjbG9zZXI7XG4gIH1cbiAgc3Vic2NyaWJlTWFueUVvc2UocmVsYXlzLCBmaWx0ZXJzLCBwYXJhbXMpIHtcbiAgICBjb25zdCBzdWJjbG9zZXIgPSB0aGlzLnN1YnNjcmliZU1hbnkocmVsYXlzLCBmaWx0ZXJzLCB7XG4gICAgICAuLi5wYXJhbXMsXG4gICAgICBvbmVvc2UoKSB7XG4gICAgICAgIHN1YmNsb3Nlci5jbG9zZSgpO1xuICAgICAgfVxuICAgIH0pO1xuICAgIHJldHVybiBzdWJjbG9zZXI7XG4gIH1cbiAgYXN5bmMgcXVlcnlTeW5jKHJlbGF5cywgZmlsdGVyLCBwYXJhbXMpIHtcbiAgICByZXR1cm4gbmV3IFByb21pc2UoYXN5bmMgKHJlc29sdmUpID0+IHtcbiAgICAgIGNvbnN0IGV2ZW50cyA9IFtdO1xuICAgICAgdGhpcy5zdWJzY3JpYmVFb3NlKHJlbGF5cywgZmlsdGVyLCB7XG4gICAgICAgIC4uLnBhcmFtcyxcbiAgICAgICAgb25ldmVudChldmVudCkge1xuICAgICAgICAgIGV2ZW50cy5wdXNoKGV2ZW50KTtcbiAgICAgICAgfSxcbiAgICAgICAgb25jbG9zZShfKSB7XG4gICAgICAgICAgcmVzb2x2ZShldmVudHMpO1xuICAgICAgICB9XG4gICAgICB9KTtcbiAgICB9KTtcbiAgfVxuICBhc3luYyBnZXQocmVsYXlzLCBmaWx0ZXIsIHBhcmFtcykge1xuICAgIGZpbHRlci5saW1pdCA9IDE7XG4gICAgY29uc3QgZXZlbnRzID0gYXdhaXQgdGhpcy5xdWVyeVN5bmMocmVsYXlzLCBmaWx0ZXIsIHBhcmFtcyk7XG4gICAgZXZlbnRzLnNvcnQoKGEsIGIpID0+IGIuY3JlYXRlZF9hdCAtIGEuY3JlYXRlZF9hdCk7XG4gICAgcmV0dXJuIGV2ZW50c1swXSB8fCBudWxsO1xuICB9XG4gIHB1Ymxpc2gocmVsYXlzLCBldmVudCkge1xuICAgIHJldHVybiByZWxheXMubWFwKG5vcm1hbGl6ZVVSTCkubWFwKGFzeW5jICh1cmwsIGkyLCBhcnIpID0+IHtcbiAgICAgIGlmIChhcnIuaW5kZXhPZih1cmwpICE9PSBpMikge1xuICAgICAgICByZXR1cm4gUHJvbWlzZS5yZWplY3QoXCJkdXBsaWNhdGUgdXJsXCIpO1xuICAgICAgfVxuICAgICAgbGV0IHIgPSBhd2FpdCB0aGlzLmVuc3VyZVJlbGF5KHVybCk7XG4gICAgICByZXR1cm4gci5wdWJsaXNoKGV2ZW50KS50aGVuKChyZWFzb24pID0+IHtcbiAgICAgICAgaWYgKHRoaXMudHJhY2tSZWxheXMpIHtcbiAgICAgICAgICBsZXQgc2V0ID0gdGhpcy5zZWVuT24uZ2V0KGV2ZW50LmlkKTtcbiAgICAgICAgICBpZiAoIXNldCkge1xuICAgICAgICAgICAgc2V0ID0gLyogQF9fUFVSRV9fICovIG5ldyBTZXQoKTtcbiAgICAgICAgICAgIHRoaXMuc2Vlbk9uLnNldChldmVudC5pZCwgc2V0KTtcbiAgICAgICAgICB9XG4gICAgICAgICAgc2V0LmFkZChyKTtcbiAgICAgICAgfVxuICAgICAgICByZXR1cm4gcmVhc29uO1xuICAgICAgfSk7XG4gICAgfSk7XG4gIH1cbiAgbGlzdENvbm5lY3Rpb25TdGF0dXMoKSB7XG4gICAgY29uc3QgbWFwID0gLyogQF9fUFVSRV9fICovIG5ldyBNYXAoKTtcbiAgICB0aGlzLnJlbGF5cy5mb3JFYWNoKChyZWxheSwgdXJsKSA9PiBtYXAuc2V0KHVybCwgcmVsYXkuY29ubmVjdGVkKSk7XG4gICAgcmV0dXJuIG1hcDtcbiAgfVxuICBkZXN0cm95KCkge1xuICAgIHRoaXMucmVsYXlzLmZvckVhY2goKGNvbm4pID0+IGNvbm4uY2xvc2UoKSk7XG4gICAgdGhpcy5yZWxheXMgPSAvKiBAX19QVVJFX18gKi8gbmV3IE1hcCgpO1xuICB9XG59O1xuXG4vLyBwb29sLnRzXG52YXIgX1dlYlNvY2tldDI7XG50cnkge1xuICBfV2ViU29ja2V0MiA9IFdlYlNvY2tldDtcbn0gY2F0Y2gge1xufVxudmFyIFNpbXBsZVBvb2wgPSBjbGFzcyBleHRlbmRzIEFic3RyYWN0U2ltcGxlUG9vbCB7XG4gIGNvbnN0cnVjdG9yKCkge1xuICAgIHN1cGVyKHsgdmVyaWZ5RXZlbnQsIHdlYnNvY2tldEltcGxlbWVudGF0aW9uOiBfV2ViU29ja2V0MiB9KTtcbiAgfVxufTtcblxuLy8gbmlwMTkudHNcbnZhciBuaXAxOV9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAxOV9leHBvcnRzLCB7XG4gIEJFQ0gzMl9SRUdFWDogKCkgPT4gQkVDSDMyX1JFR0VYLFxuICBCZWNoMzJNYXhTaXplOiAoKSA9PiBCZWNoMzJNYXhTaXplLFxuICBOb3N0clR5cGVHdWFyZDogKCkgPT4gTm9zdHJUeXBlR3VhcmQsXG4gIGRlY29kZTogKCkgPT4gZGVjb2RlLFxuICBkZWNvZGVOb3N0clVSSTogKCkgPT4gZGVjb2RlTm9zdHJVUkksXG4gIGVuY29kZUJ5dGVzOiAoKSA9PiBlbmNvZGVCeXRlcyxcbiAgbmFkZHJFbmNvZGU6ICgpID0+IG5hZGRyRW5jb2RlLFxuICBuZXZlbnRFbmNvZGU6ICgpID0+IG5ldmVudEVuY29kZSxcbiAgbm90ZUVuY29kZTogKCkgPT4gbm90ZUVuY29kZSxcbiAgbnByb2ZpbGVFbmNvZGU6ICgpID0+IG5wcm9maWxlRW5jb2RlLFxuICBucHViRW5jb2RlOiAoKSA9PiBucHViRW5jb2RlLFxuICBuc2VjRW5jb2RlOiAoKSA9PiBuc2VjRW5jb2RlXG59KTtcbmltcG9ydCB7IGJ5dGVzVG9IZXggYXMgYnl0ZXNUb0hleDMsIGNvbmNhdEJ5dGVzLCBoZXhUb0J5dGVzIGFzIGhleFRvQnl0ZXMyIH0gZnJvbSBcIkBub2JsZS9oYXNoZXMvdXRpbHNcIjtcbmltcG9ydCB7IGJlY2gzMiB9IGZyb20gXCJAc2N1cmUvYmFzZVwiO1xudmFyIE5vc3RyVHlwZUd1YXJkID0ge1xuICBpc05Qcm9maWxlOiAodmFsdWUpID0+IC9ebnByb2ZpbGUxW2EtelxcZF0rJC8udGVzdCh2YWx1ZSB8fCBcIlwiKSxcbiAgaXNORXZlbnQ6ICh2YWx1ZSkgPT4gL15uZXZlbnQxW2EtelxcZF0rJC8udGVzdCh2YWx1ZSB8fCBcIlwiKSxcbiAgaXNOQWRkcjogKHZhbHVlKSA9PiAvXm5hZGRyMVthLXpcXGRdKyQvLnRlc3QodmFsdWUgfHwgXCJcIiksXG4gIGlzTlNlYzogKHZhbHVlKSA9PiAvXm5zZWMxW2EtelxcZF17NTh9JC8udGVzdCh2YWx1ZSB8fCBcIlwiKSxcbiAgaXNOUHViOiAodmFsdWUpID0+IC9ebnB1YjFbYS16XFxkXXs1OH0kLy50ZXN0KHZhbHVlIHx8IFwiXCIpLFxuICBpc05vdGU6ICh2YWx1ZSkgPT4gL15ub3RlMVthLXpcXGRdKyQvLnRlc3QodmFsdWUgfHwgXCJcIiksXG4gIGlzTmNyeXB0c2VjOiAodmFsdWUpID0+IC9ebmNyeXB0c2VjMVthLXpcXGRdKyQvLnRlc3QodmFsdWUgfHwgXCJcIilcbn07XG52YXIgQmVjaDMyTWF4U2l6ZSA9IDVlMztcbnZhciBCRUNIMzJfUkVHRVggPSAvW1xceDIxLVxceDdFXXsxLDgzfTFbMDIzNDU2Nzg5YWNkZWZnaGprbG1ucHFyc3R1dnd4eXpdezYsfS87XG5mdW5jdGlvbiBpbnRlZ2VyVG9VaW50OEFycmF5KG51bWJlcikge1xuICBjb25zdCB1aW50OEFycmF5ID0gbmV3IFVpbnQ4QXJyYXkoNCk7XG4gIHVpbnQ4QXJyYXlbMF0gPSBudW1iZXIgPj4gMjQgJiAyNTU7XG4gIHVpbnQ4QXJyYXlbMV0gPSBudW1iZXIgPj4gMTYgJiAyNTU7XG4gIHVpbnQ4QXJyYXlbMl0gPSBudW1iZXIgPj4gOCAmIDI1NTtcbiAgdWludDhBcnJheVszXSA9IG51bWJlciAmIDI1NTtcbiAgcmV0dXJuIHVpbnQ4QXJyYXk7XG59XG5mdW5jdGlvbiBkZWNvZGVOb3N0clVSSShuaXAxOWNvZGUpIHtcbiAgdHJ5IHtcbiAgICBpZiAobmlwMTljb2RlLnN0YXJ0c1dpdGgoXCJub3N0cjpcIikpXG4gICAgICBuaXAxOWNvZGUgPSBuaXAxOWNvZGUuc3Vic3RyaW5nKDYpO1xuICAgIHJldHVybiBkZWNvZGUobmlwMTljb2RlKTtcbiAgfSBjYXRjaCAoX2Vycikge1xuICAgIHJldHVybiB7IHR5cGU6IFwiaW52YWxpZFwiLCBkYXRhOiBudWxsIH07XG4gIH1cbn1cbmZ1bmN0aW9uIGRlY29kZShjb2RlKSB7XG4gIGxldCB7IHByZWZpeCwgd29yZHMgfSA9IGJlY2gzMi5kZWNvZGUoY29kZSwgQmVjaDMyTWF4U2l6ZSk7XG4gIGxldCBkYXRhID0gbmV3IFVpbnQ4QXJyYXkoYmVjaDMyLmZyb21Xb3Jkcyh3b3JkcykpO1xuICBzd2l0Y2ggKHByZWZpeCkge1xuICAgIGNhc2UgXCJucHJvZmlsZVwiOiB7XG4gICAgICBsZXQgdGx2ID0gcGFyc2VUTFYoZGF0YSk7XG4gICAgICBpZiAoIXRsdlswXT8uWzBdKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJtaXNzaW5nIFRMViAwIGZvciBucHJvZmlsZVwiKTtcbiAgICAgIGlmICh0bHZbMF1bMF0ubGVuZ3RoICE9PSAzMilcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKFwiVExWIDAgc2hvdWxkIGJlIDMyIGJ5dGVzXCIpO1xuICAgICAgcmV0dXJuIHtcbiAgICAgICAgdHlwZTogXCJucHJvZmlsZVwiLFxuICAgICAgICBkYXRhOiB7XG4gICAgICAgICAgcHVia2V5OiBieXRlc1RvSGV4Myh0bHZbMF1bMF0pLFxuICAgICAgICAgIHJlbGF5czogdGx2WzFdID8gdGx2WzFdLm1hcCgoZCkgPT4gdXRmOERlY29kZXIuZGVjb2RlKGQpKSA6IFtdXG4gICAgICAgIH1cbiAgICAgIH07XG4gICAgfVxuICAgIGNhc2UgXCJuZXZlbnRcIjoge1xuICAgICAgbGV0IHRsdiA9IHBhcnNlVExWKGRhdGEpO1xuICAgICAgaWYgKCF0bHZbMF0/LlswXSlcbiAgICAgICAgdGhyb3cgbmV3IEVycm9yKFwibWlzc2luZyBUTFYgMCBmb3IgbmV2ZW50XCIpO1xuICAgICAgaWYgKHRsdlswXVswXS5sZW5ndGggIT09IDMyKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJUTFYgMCBzaG91bGQgYmUgMzIgYnl0ZXNcIik7XG4gICAgICBpZiAodGx2WzJdICYmIHRsdlsyXVswXS5sZW5ndGggIT09IDMyKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJUTFYgMiBzaG91bGQgYmUgMzIgYnl0ZXNcIik7XG4gICAgICBpZiAodGx2WzNdICYmIHRsdlszXVswXS5sZW5ndGggIT09IDQpXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihcIlRMViAzIHNob3VsZCBiZSA0IGJ5dGVzXCIpO1xuICAgICAgcmV0dXJuIHtcbiAgICAgICAgdHlwZTogXCJuZXZlbnRcIixcbiAgICAgICAgZGF0YToge1xuICAgICAgICAgIGlkOiBieXRlc1RvSGV4Myh0bHZbMF1bMF0pLFxuICAgICAgICAgIHJlbGF5czogdGx2WzFdID8gdGx2WzFdLm1hcCgoZCkgPT4gdXRmOERlY29kZXIuZGVjb2RlKGQpKSA6IFtdLFxuICAgICAgICAgIGF1dGhvcjogdGx2WzJdPy5bMF0gPyBieXRlc1RvSGV4Myh0bHZbMl1bMF0pIDogdm9pZCAwLFxuICAgICAgICAgIGtpbmQ6IHRsdlszXT8uWzBdID8gcGFyc2VJbnQoYnl0ZXNUb0hleDModGx2WzNdWzBdKSwgMTYpIDogdm9pZCAwXG4gICAgICAgIH1cbiAgICAgIH07XG4gICAgfVxuICAgIGNhc2UgXCJuYWRkclwiOiB7XG4gICAgICBsZXQgdGx2ID0gcGFyc2VUTFYoZGF0YSk7XG4gICAgICBpZiAoIXRsdlswXT8uWzBdKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJtaXNzaW5nIFRMViAwIGZvciBuYWRkclwiKTtcbiAgICAgIGlmICghdGx2WzJdPy5bMF0pXG4gICAgICAgIHRocm93IG5ldyBFcnJvcihcIm1pc3NpbmcgVExWIDIgZm9yIG5hZGRyXCIpO1xuICAgICAgaWYgKHRsdlsyXVswXS5sZW5ndGggIT09IDMyKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJUTFYgMiBzaG91bGQgYmUgMzIgYnl0ZXNcIik7XG4gICAgICBpZiAoIXRsdlszXT8uWzBdKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJtaXNzaW5nIFRMViAzIGZvciBuYWRkclwiKTtcbiAgICAgIGlmICh0bHZbM11bMF0ubGVuZ3RoICE9PSA0KVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJUTFYgMyBzaG91bGQgYmUgNCBieXRlc1wiKTtcbiAgICAgIHJldHVybiB7XG4gICAgICAgIHR5cGU6IFwibmFkZHJcIixcbiAgICAgICAgZGF0YToge1xuICAgICAgICAgIGlkZW50aWZpZXI6IHV0ZjhEZWNvZGVyLmRlY29kZSh0bHZbMF1bMF0pLFxuICAgICAgICAgIHB1YmtleTogYnl0ZXNUb0hleDModGx2WzJdWzBdKSxcbiAgICAgICAgICBraW5kOiBwYXJzZUludChieXRlc1RvSGV4Myh0bHZbM11bMF0pLCAxNiksXG4gICAgICAgICAgcmVsYXlzOiB0bHZbMV0gPyB0bHZbMV0ubWFwKChkKSA9PiB1dGY4RGVjb2Rlci5kZWNvZGUoZCkpIDogW11cbiAgICAgICAgfVxuICAgICAgfTtcbiAgICB9XG4gICAgY2FzZSBcIm5zZWNcIjpcbiAgICAgIHJldHVybiB7IHR5cGU6IHByZWZpeCwgZGF0YSB9O1xuICAgIGNhc2UgXCJucHViXCI6XG4gICAgY2FzZSBcIm5vdGVcIjpcbiAgICAgIHJldHVybiB7IHR5cGU6IHByZWZpeCwgZGF0YTogYnl0ZXNUb0hleDMoZGF0YSkgfTtcbiAgICBkZWZhdWx0OlxuICAgICAgdGhyb3cgbmV3IEVycm9yKGB1bmtub3duIHByZWZpeCAke3ByZWZpeH1gKTtcbiAgfVxufVxuZnVuY3Rpb24gcGFyc2VUTFYoZGF0YSkge1xuICBsZXQgcmVzdWx0ID0ge307XG4gIGxldCByZXN0ID0gZGF0YTtcbiAgd2hpbGUgKHJlc3QubGVuZ3RoID4gMCkge1xuICAgIGxldCB0ID0gcmVzdFswXTtcbiAgICBsZXQgbCA9IHJlc3RbMV07XG4gICAgbGV0IHYgPSByZXN0LnNsaWNlKDIsIDIgKyBsKTtcbiAgICByZXN0ID0gcmVzdC5zbGljZSgyICsgbCk7XG4gICAgaWYgKHYubGVuZ3RoIDwgbClcbiAgICAgIHRocm93IG5ldyBFcnJvcihgbm90IGVub3VnaCBkYXRhIHRvIHJlYWQgb24gVExWICR7dH1gKTtcbiAgICByZXN1bHRbdF0gPSByZXN1bHRbdF0gfHwgW107XG4gICAgcmVzdWx0W3RdLnB1c2godik7XG4gIH1cbiAgcmV0dXJuIHJlc3VsdDtcbn1cbmZ1bmN0aW9uIG5zZWNFbmNvZGUoa2V5KSB7XG4gIHJldHVybiBlbmNvZGVCeXRlcyhcIm5zZWNcIiwga2V5KTtcbn1cbmZ1bmN0aW9uIG5wdWJFbmNvZGUoaGV4KSB7XG4gIHJldHVybiBlbmNvZGVCeXRlcyhcIm5wdWJcIiwgaGV4VG9CeXRlczIoaGV4KSk7XG59XG5mdW5jdGlvbiBub3RlRW5jb2RlKGhleCkge1xuICByZXR1cm4gZW5jb2RlQnl0ZXMoXCJub3RlXCIsIGhleFRvQnl0ZXMyKGhleCkpO1xufVxuZnVuY3Rpb24gZW5jb2RlQmVjaDMyKHByZWZpeCwgZGF0YSkge1xuICBsZXQgd29yZHMgPSBiZWNoMzIudG9Xb3JkcyhkYXRhKTtcbiAgcmV0dXJuIGJlY2gzMi5lbmNvZGUocHJlZml4LCB3b3JkcywgQmVjaDMyTWF4U2l6ZSk7XG59XG5mdW5jdGlvbiBlbmNvZGVCeXRlcyhwcmVmaXgsIGJ5dGVzKSB7XG4gIHJldHVybiBlbmNvZGVCZWNoMzIocHJlZml4LCBieXRlcyk7XG59XG5mdW5jdGlvbiBucHJvZmlsZUVuY29kZShwcm9maWxlKSB7XG4gIGxldCBkYXRhID0gZW5jb2RlVExWKHtcbiAgICAwOiBbaGV4VG9CeXRlczIocHJvZmlsZS5wdWJrZXkpXSxcbiAgICAxOiAocHJvZmlsZS5yZWxheXMgfHwgW10pLm1hcCgodXJsKSA9PiB1dGY4RW5jb2Rlci5lbmNvZGUodXJsKSlcbiAgfSk7XG4gIHJldHVybiBlbmNvZGVCZWNoMzIoXCJucHJvZmlsZVwiLCBkYXRhKTtcbn1cbmZ1bmN0aW9uIG5ldmVudEVuY29kZShldmVudCkge1xuICBsZXQga2luZEFycmF5O1xuICBpZiAoZXZlbnQua2luZCAhPT0gdm9pZCAwKSB7XG4gICAga2luZEFycmF5ID0gaW50ZWdlclRvVWludDhBcnJheShldmVudC5raW5kKTtcbiAgfVxuICBsZXQgZGF0YSA9IGVuY29kZVRMVih7XG4gICAgMDogW2hleFRvQnl0ZXMyKGV2ZW50LmlkKV0sXG4gICAgMTogKGV2ZW50LnJlbGF5cyB8fCBbXSkubWFwKCh1cmwpID0+IHV0ZjhFbmNvZGVyLmVuY29kZSh1cmwpKSxcbiAgICAyOiBldmVudC5hdXRob3IgPyBbaGV4VG9CeXRlczIoZXZlbnQuYXV0aG9yKV0gOiBbXSxcbiAgICAzOiBraW5kQXJyYXkgPyBbbmV3IFVpbnQ4QXJyYXkoa2luZEFycmF5KV0gOiBbXVxuICB9KTtcbiAgcmV0dXJuIGVuY29kZUJlY2gzMihcIm5ldmVudFwiLCBkYXRhKTtcbn1cbmZ1bmN0aW9uIG5hZGRyRW5jb2RlKGFkZHIpIHtcbiAgbGV0IGtpbmQgPSBuZXcgQXJyYXlCdWZmZXIoNCk7XG4gIG5ldyBEYXRhVmlldyhraW5kKS5zZXRVaW50MzIoMCwgYWRkci5raW5kLCBmYWxzZSk7XG4gIGxldCBkYXRhID0gZW5jb2RlVExWKHtcbiAgICAwOiBbdXRmOEVuY29kZXIuZW5jb2RlKGFkZHIuaWRlbnRpZmllcildLFxuICAgIDE6IChhZGRyLnJlbGF5cyB8fCBbXSkubWFwKCh1cmwpID0+IHV0ZjhFbmNvZGVyLmVuY29kZSh1cmwpKSxcbiAgICAyOiBbaGV4VG9CeXRlczIoYWRkci5wdWJrZXkpXSxcbiAgICAzOiBbbmV3IFVpbnQ4QXJyYXkoa2luZCldXG4gIH0pO1xuICByZXR1cm4gZW5jb2RlQmVjaDMyKFwibmFkZHJcIiwgZGF0YSk7XG59XG5mdW5jdGlvbiBlbmNvZGVUTFYodGx2KSB7XG4gIGxldCBlbnRyaWVzID0gW107XG4gIE9iamVjdC5lbnRyaWVzKHRsdikucmV2ZXJzZSgpLmZvckVhY2goKFt0LCB2c10pID0+IHtcbiAgICB2cy5mb3JFYWNoKCh2KSA9PiB7XG4gICAgICBsZXQgZW50cnkgPSBuZXcgVWludDhBcnJheSh2Lmxlbmd0aCArIDIpO1xuICAgICAgZW50cnkuc2V0KFtwYXJzZUludCh0KV0sIDApO1xuICAgICAgZW50cnkuc2V0KFt2Lmxlbmd0aF0sIDEpO1xuICAgICAgZW50cnkuc2V0KHYsIDIpO1xuICAgICAgZW50cmllcy5wdXNoKGVudHJ5KTtcbiAgICB9KTtcbiAgfSk7XG4gIHJldHVybiBjb25jYXRCeXRlcyguLi5lbnRyaWVzKTtcbn1cblxuLy8gcmVmZXJlbmNlcy50c1xudmFyIG1lbnRpb25SZWdleCA9IC9cXGJub3N0cjooKG5vdGV8bnB1YnxuYWRkcnxuZXZlbnR8bnByb2ZpbGUpMVxcdyspXFxifCNcXFsoXFxkKylcXF0vZztcbmZ1bmN0aW9uIHBhcnNlUmVmZXJlbmNlcyhldnQpIHtcbiAgbGV0IHJlZmVyZW5jZXMgPSBbXTtcbiAgZm9yIChsZXQgcmVmIG9mIGV2dC5jb250ZW50Lm1hdGNoQWxsKG1lbnRpb25SZWdleCkpIHtcbiAgICBpZiAocmVmWzJdKSB7XG4gICAgICB0cnkge1xuICAgICAgICBsZXQgeyB0eXBlLCBkYXRhIH0gPSBkZWNvZGUocmVmWzFdKTtcbiAgICAgICAgc3dpdGNoICh0eXBlKSB7XG4gICAgICAgICAgY2FzZSBcIm5wdWJcIjoge1xuICAgICAgICAgICAgcmVmZXJlbmNlcy5wdXNoKHtcbiAgICAgICAgICAgICAgdGV4dDogcmVmWzBdLFxuICAgICAgICAgICAgICBwcm9maWxlOiB7IHB1YmtleTogZGF0YSwgcmVsYXlzOiBbXSB9XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgIH1cbiAgICAgICAgICBjYXNlIFwibnByb2ZpbGVcIjoge1xuICAgICAgICAgICAgcmVmZXJlbmNlcy5wdXNoKHtcbiAgICAgICAgICAgICAgdGV4dDogcmVmWzBdLFxuICAgICAgICAgICAgICBwcm9maWxlOiBkYXRhXG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgIH1cbiAgICAgICAgICBjYXNlIFwibm90ZVwiOiB7XG4gICAgICAgICAgICByZWZlcmVuY2VzLnB1c2goe1xuICAgICAgICAgICAgICB0ZXh0OiByZWZbMF0sXG4gICAgICAgICAgICAgIGV2ZW50OiB7IGlkOiBkYXRhLCByZWxheXM6IFtdIH1cbiAgICAgICAgICAgIH0pO1xuICAgICAgICAgICAgYnJlYWs7XG4gICAgICAgICAgfVxuICAgICAgICAgIGNhc2UgXCJuZXZlbnRcIjoge1xuICAgICAgICAgICAgcmVmZXJlbmNlcy5wdXNoKHtcbiAgICAgICAgICAgICAgdGV4dDogcmVmWzBdLFxuICAgICAgICAgICAgICBldmVudDogZGF0YVxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBicmVhaztcbiAgICAgICAgICB9XG4gICAgICAgICAgY2FzZSBcIm5hZGRyXCI6IHtcbiAgICAgICAgICAgIHJlZmVyZW5jZXMucHVzaCh7XG4gICAgICAgICAgICAgIHRleHQ6IHJlZlswXSxcbiAgICAgICAgICAgICAgYWRkcmVzczogZGF0YVxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBicmVhaztcbiAgICAgICAgICB9XG4gICAgICAgIH1cbiAgICAgIH0gY2F0Y2ggKGVycikge1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAocmVmWzNdKSB7XG4gICAgICBsZXQgaWR4ID0gcGFyc2VJbnQocmVmWzNdLCAxMCk7XG4gICAgICBsZXQgdGFnID0gZXZ0LnRhZ3NbaWR4XTtcbiAgICAgIGlmICghdGFnKVxuICAgICAgICBjb250aW51ZTtcbiAgICAgIHN3aXRjaCAodGFnWzBdKSB7XG4gICAgICAgIGNhc2UgXCJwXCI6IHtcbiAgICAgICAgICByZWZlcmVuY2VzLnB1c2goe1xuICAgICAgICAgICAgdGV4dDogcmVmWzBdLFxuICAgICAgICAgICAgcHJvZmlsZTogeyBwdWJrZXk6IHRhZ1sxXSwgcmVsYXlzOiB0YWdbMl0gPyBbdGFnWzJdXSA6IFtdIH1cbiAgICAgICAgICB9KTtcbiAgICAgICAgICBicmVhaztcbiAgICAgICAgfVxuICAgICAgICBjYXNlIFwiZVwiOiB7XG4gICAgICAgICAgcmVmZXJlbmNlcy5wdXNoKHtcbiAgICAgICAgICAgIHRleHQ6IHJlZlswXSxcbiAgICAgICAgICAgIGV2ZW50OiB7IGlkOiB0YWdbMV0sIHJlbGF5czogdGFnWzJdID8gW3RhZ1syXV0gOiBbXSB9XG4gICAgICAgICAgfSk7XG4gICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgICAgY2FzZSBcImFcIjoge1xuICAgICAgICAgIHRyeSB7XG4gICAgICAgICAgICBsZXQgW2tpbmQsIHB1YmtleSwgaWRlbnRpZmllcl0gPSB0YWdbMV0uc3BsaXQoXCI6XCIpO1xuICAgICAgICAgICAgcmVmZXJlbmNlcy5wdXNoKHtcbiAgICAgICAgICAgICAgdGV4dDogcmVmWzBdLFxuICAgICAgICAgICAgICBhZGRyZXNzOiB7XG4gICAgICAgICAgICAgICAgaWRlbnRpZmllcixcbiAgICAgICAgICAgICAgICBwdWJrZXksXG4gICAgICAgICAgICAgICAga2luZDogcGFyc2VJbnQoa2luZCwgMTApLFxuICAgICAgICAgICAgICAgIHJlbGF5czogdGFnWzJdID8gW3RhZ1syXV0gOiBbXVxuICAgICAgICAgICAgICB9XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgICB9IGNhdGNoIChlcnIpIHtcbiAgICAgICAgICB9XG4gICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgIH1cbiAgICB9XG4gIH1cbiAgcmV0dXJuIHJlZmVyZW5jZXM7XG59XG5cbi8vIG5pcDA0LnRzXG52YXIgbmlwMDRfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwMDRfZXhwb3J0cywge1xuICBkZWNyeXB0OiAoKSA9PiBkZWNyeXB0LFxuICBlbmNyeXB0OiAoKSA9PiBlbmNyeXB0XG59KTtcbmltcG9ydCB7IGJ5dGVzVG9IZXggYXMgYnl0ZXNUb0hleDQsIHJhbmRvbUJ5dGVzIH0gZnJvbSBcIkBub2JsZS9oYXNoZXMvdXRpbHNcIjtcbmltcG9ydCB7IHNlY3AyNTZrMSB9IGZyb20gXCJAbm9ibGUvY3VydmVzL3NlY3AyNTZrMVwiO1xuaW1wb3J0IHsgY2JjIH0gZnJvbSBcIkBub2JsZS9jaXBoZXJzL2Flc1wiO1xuaW1wb3J0IHsgYmFzZTY0IH0gZnJvbSBcIkBzY3VyZS9iYXNlXCI7XG5mdW5jdGlvbiBlbmNyeXB0KHNlY3JldEtleSwgcHVia2V5LCB0ZXh0KSB7XG4gIGNvbnN0IHByaXZrZXkgPSBzZWNyZXRLZXkgaW5zdGFuY2VvZiBVaW50OEFycmF5ID8gYnl0ZXNUb0hleDQoc2VjcmV0S2V5KSA6IHNlY3JldEtleTtcbiAgY29uc3Qga2V5ID0gc2VjcDI1NmsxLmdldFNoYXJlZFNlY3JldChwcml2a2V5LCBcIjAyXCIgKyBwdWJrZXkpO1xuICBjb25zdCBub3JtYWxpemVkS2V5ID0gZ2V0Tm9ybWFsaXplZFgoa2V5KTtcbiAgbGV0IGl2ID0gVWludDhBcnJheS5mcm9tKHJhbmRvbUJ5dGVzKDE2KSk7XG4gIGxldCBwbGFpbnRleHQgPSB1dGY4RW5jb2Rlci5lbmNvZGUodGV4dCk7XG4gIGxldCBjaXBoZXJ0ZXh0ID0gY2JjKG5vcm1hbGl6ZWRLZXksIGl2KS5lbmNyeXB0KHBsYWludGV4dCk7XG4gIGxldCBjdGI2NCA9IGJhc2U2NC5lbmNvZGUobmV3IFVpbnQ4QXJyYXkoY2lwaGVydGV4dCkpO1xuICBsZXQgaXZiNjQgPSBiYXNlNjQuZW5jb2RlKG5ldyBVaW50OEFycmF5KGl2LmJ1ZmZlcikpO1xuICByZXR1cm4gYCR7Y3RiNjR9P2l2PSR7aXZiNjR9YDtcbn1cbmZ1bmN0aW9uIGRlY3J5cHQoc2VjcmV0S2V5LCBwdWJrZXksIGRhdGEpIHtcbiAgY29uc3QgcHJpdmtleSA9IHNlY3JldEtleSBpbnN0YW5jZW9mIFVpbnQ4QXJyYXkgPyBieXRlc1RvSGV4NChzZWNyZXRLZXkpIDogc2VjcmV0S2V5O1xuICBsZXQgW2N0YjY0LCBpdmI2NF0gPSBkYXRhLnNwbGl0KFwiP2l2PVwiKTtcbiAgbGV0IGtleSA9IHNlY3AyNTZrMS5nZXRTaGFyZWRTZWNyZXQocHJpdmtleSwgXCIwMlwiICsgcHVia2V5KTtcbiAgbGV0IG5vcm1hbGl6ZWRLZXkgPSBnZXROb3JtYWxpemVkWChrZXkpO1xuICBsZXQgaXYgPSBiYXNlNjQuZGVjb2RlKGl2YjY0KTtcbiAgbGV0IGNpcGhlcnRleHQgPSBiYXNlNjQuZGVjb2RlKGN0YjY0KTtcbiAgbGV0IHBsYWludGV4dCA9IGNiYyhub3JtYWxpemVkS2V5LCBpdikuZGVjcnlwdChjaXBoZXJ0ZXh0KTtcbiAgcmV0dXJuIHV0ZjhEZWNvZGVyLmRlY29kZShwbGFpbnRleHQpO1xufVxuZnVuY3Rpb24gZ2V0Tm9ybWFsaXplZFgoa2V5KSB7XG4gIHJldHVybiBrZXkuc2xpY2UoMSwgMzMpO1xufVxuXG4vLyBuaXAwNS50c1xudmFyIG5pcDA1X2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDA1X2V4cG9ydHMsIHtcbiAgTklQMDVfUkVHRVg6ICgpID0+IE5JUDA1X1JFR0VYLFxuICBpc05pcDA1OiAoKSA9PiBpc05pcDA1LFxuICBpc1ZhbGlkOiAoKSA9PiBpc1ZhbGlkLFxuICBxdWVyeVByb2ZpbGU6ICgpID0+IHF1ZXJ5UHJvZmlsZSxcbiAgc2VhcmNoRG9tYWluOiAoKSA9PiBzZWFyY2hEb21haW4sXG4gIHVzZUZldGNoSW1wbGVtZW50YXRpb246ICgpID0+IHVzZUZldGNoSW1wbGVtZW50YXRpb25cbn0pO1xudmFyIE5JUDA1X1JFR0VYID0gL14oPzooW1xcdy4rLV0rKUApPyhbXFx3Xy1dKyhcXC5bXFx3Xy1dKykrKSQvO1xudmFyIGlzTmlwMDUgPSAodmFsdWUpID0+IE5JUDA1X1JFR0VYLnRlc3QodmFsdWUgfHwgXCJcIik7XG52YXIgX2ZldGNoO1xudHJ5IHtcbiAgX2ZldGNoID0gZmV0Y2g7XG59IGNhdGNoIChfKSB7XG4gIG51bGw7XG59XG5mdW5jdGlvbiB1c2VGZXRjaEltcGxlbWVudGF0aW9uKGZldGNoSW1wbGVtZW50YXRpb24pIHtcbiAgX2ZldGNoID0gZmV0Y2hJbXBsZW1lbnRhdGlvbjtcbn1cbmFzeW5jIGZ1bmN0aW9uIHNlYXJjaERvbWFpbihkb21haW4sIHF1ZXJ5ID0gXCJcIikge1xuICB0cnkge1xuICAgIGNvbnN0IHVybCA9IGBodHRwczovLyR7ZG9tYWlufS8ud2VsbC1rbm93bi9ub3N0ci5qc29uP25hbWU9JHtxdWVyeX1gO1xuICAgIGNvbnN0IHJlcyA9IGF3YWl0IF9mZXRjaCh1cmwsIHsgcmVkaXJlY3Q6IFwibWFudWFsXCIgfSk7XG4gICAgaWYgKHJlcy5zdGF0dXMgIT09IDIwMCkge1xuICAgICAgdGhyb3cgRXJyb3IoXCJXcm9uZyByZXNwb25zZSBjb2RlXCIpO1xuICAgIH1cbiAgICBjb25zdCBqc29uID0gYXdhaXQgcmVzLmpzb24oKTtcbiAgICByZXR1cm4ganNvbi5uYW1lcztcbiAgfSBjYXRjaCAoXykge1xuICAgIHJldHVybiB7fTtcbiAgfVxufVxuYXN5bmMgZnVuY3Rpb24gcXVlcnlQcm9maWxlKGZ1bGxuYW1lKSB7XG4gIGNvbnN0IG1hdGNoID0gZnVsbG5hbWUubWF0Y2goTklQMDVfUkVHRVgpO1xuICBpZiAoIW1hdGNoKVxuICAgIHJldHVybiBudWxsO1xuICBjb25zdCBbLCBuYW1lID0gXCJfXCIsIGRvbWFpbl0gPSBtYXRjaDtcbiAgdHJ5IHtcbiAgICBjb25zdCB1cmwgPSBgaHR0cHM6Ly8ke2RvbWFpbn0vLndlbGwta25vd24vbm9zdHIuanNvbj9uYW1lPSR7bmFtZX1gO1xuICAgIGNvbnN0IHJlcyA9IGF3YWl0IF9mZXRjaCh1cmwsIHsgcmVkaXJlY3Q6IFwibWFudWFsXCIgfSk7XG4gICAgaWYgKHJlcy5zdGF0dXMgIT09IDIwMCkge1xuICAgICAgdGhyb3cgRXJyb3IoXCJXcm9uZyByZXNwb25zZSBjb2RlXCIpO1xuICAgIH1cbiAgICBjb25zdCBqc29uID0gYXdhaXQgcmVzLmpzb24oKTtcbiAgICBjb25zdCBwdWJrZXkgPSBqc29uLm5hbWVzW25hbWVdO1xuICAgIHJldHVybiBwdWJrZXkgPyB7IHB1YmtleSwgcmVsYXlzOiBqc29uLnJlbGF5cz8uW3B1YmtleV0gfSA6IG51bGw7XG4gIH0gY2F0Y2ggKF9lKSB7XG4gICAgcmV0dXJuIG51bGw7XG4gIH1cbn1cbmFzeW5jIGZ1bmN0aW9uIGlzVmFsaWQocHVia2V5LCBuaXAwNSkge1xuICBjb25zdCByZXMgPSBhd2FpdCBxdWVyeVByb2ZpbGUobmlwMDUpO1xuICByZXR1cm4gcmVzID8gcmVzLnB1YmtleSA9PT0gcHVia2V5IDogZmFsc2U7XG59XG5cbi8vIG5pcDEwLnRzXG52YXIgbmlwMTBfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwMTBfZXhwb3J0cywge1xuICBwYXJzZTogKCkgPT4gcGFyc2Vcbn0pO1xuZnVuY3Rpb24gcGFyc2UoZXZlbnQpIHtcbiAgY29uc3QgcmVzdWx0ID0ge1xuICAgIHJlcGx5OiB2b2lkIDAsXG4gICAgcm9vdDogdm9pZCAwLFxuICAgIG1lbnRpb25zOiBbXSxcbiAgICBwcm9maWxlczogW10sXG4gICAgcXVvdGVzOiBbXVxuICB9O1xuICBsZXQgbWF5YmVQYXJlbnQ7XG4gIGxldCBtYXliZVJvb3Q7XG4gIGZvciAobGV0IGkyID0gZXZlbnQudGFncy5sZW5ndGggLSAxOyBpMiA+PSAwOyBpMi0tKSB7XG4gICAgY29uc3QgdGFnID0gZXZlbnQudGFnc1tpMl07XG4gICAgaWYgKHRhZ1swXSA9PT0gXCJlXCIgJiYgdGFnWzFdKSB7XG4gICAgICBjb25zdCBbXywgZVRhZ0V2ZW50SWQsIGVUYWdSZWxheVVybCwgZVRhZ01hcmtlciwgZVRhZ0F1dGhvcl0gPSB0YWc7XG4gICAgICBjb25zdCBldmVudFBvaW50ZXIgPSB7XG4gICAgICAgIGlkOiBlVGFnRXZlbnRJZCxcbiAgICAgICAgcmVsYXlzOiBlVGFnUmVsYXlVcmwgPyBbZVRhZ1JlbGF5VXJsXSA6IFtdLFxuICAgICAgICBhdXRob3I6IGVUYWdBdXRob3JcbiAgICAgIH07XG4gICAgICBpZiAoZVRhZ01hcmtlciA9PT0gXCJyb290XCIpIHtcbiAgICAgICAgcmVzdWx0LnJvb3QgPSBldmVudFBvaW50ZXI7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKGVUYWdNYXJrZXIgPT09IFwicmVwbHlcIikge1xuICAgICAgICByZXN1bHQucmVwbHkgPSBldmVudFBvaW50ZXI7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKGVUYWdNYXJrZXIgPT09IFwibWVudGlvblwiKSB7XG4gICAgICAgIHJlc3VsdC5tZW50aW9ucy5wdXNoKGV2ZW50UG9pbnRlcik7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgICAgaWYgKCFtYXliZVBhcmVudCkge1xuICAgICAgICBtYXliZVBhcmVudCA9IGV2ZW50UG9pbnRlcjtcbiAgICAgIH0gZWxzZSB7XG4gICAgICAgIG1heWJlUm9vdCA9IGV2ZW50UG9pbnRlcjtcbiAgICAgIH1cbiAgICAgIHJlc3VsdC5tZW50aW9ucy5wdXNoKGV2ZW50UG9pbnRlcik7XG4gICAgICBjb250aW51ZTtcbiAgICB9XG4gICAgaWYgKHRhZ1swXSA9PT0gXCJxXCIgJiYgdGFnWzFdKSB7XG4gICAgICBjb25zdCBbXywgZVRhZ0V2ZW50SWQsIGVUYWdSZWxheVVybF0gPSB0YWc7XG4gICAgICByZXN1bHQucXVvdGVzLnB1c2goe1xuICAgICAgICBpZDogZVRhZ0V2ZW50SWQsXG4gICAgICAgIHJlbGF5czogZVRhZ1JlbGF5VXJsID8gW2VUYWdSZWxheVVybF0gOiBbXVxuICAgICAgfSk7XG4gICAgfVxuICAgIGlmICh0YWdbMF0gPT09IFwicFwiICYmIHRhZ1sxXSkge1xuICAgICAgcmVzdWx0LnByb2ZpbGVzLnB1c2goe1xuICAgICAgICBwdWJrZXk6IHRhZ1sxXSxcbiAgICAgICAgcmVsYXlzOiB0YWdbMl0gPyBbdGFnWzJdXSA6IFtdXG4gICAgICB9KTtcbiAgICAgIGNvbnRpbnVlO1xuICAgIH1cbiAgfVxuICBpZiAoIXJlc3VsdC5yb290KSB7XG4gICAgcmVzdWx0LnJvb3QgPSBtYXliZVJvb3QgfHwgbWF5YmVQYXJlbnQgfHwgcmVzdWx0LnJlcGx5O1xuICB9XG4gIGlmICghcmVzdWx0LnJlcGx5KSB7XG4gICAgcmVzdWx0LnJlcGx5ID0gbWF5YmVQYXJlbnQgfHwgcmVzdWx0LnJvb3Q7XG4gIH1cbiAgO1xuICBbcmVzdWx0LnJlcGx5LCByZXN1bHQucm9vdF0uZm9yRWFjaCgocmVmKSA9PiB7XG4gICAgaWYgKCFyZWYpXG4gICAgICByZXR1cm47XG4gICAgbGV0IGlkeCA9IHJlc3VsdC5tZW50aW9ucy5pbmRleE9mKHJlZik7XG4gICAgaWYgKGlkeCAhPT0gLTEpIHtcbiAgICAgIHJlc3VsdC5tZW50aW9ucy5zcGxpY2UoaWR4LCAxKTtcbiAgICB9XG4gICAgaWYgKHJlZi5hdXRob3IpIHtcbiAgICAgIGxldCBhdXRob3IgPSByZXN1bHQucHJvZmlsZXMuZmluZCgocCkgPT4gcC5wdWJrZXkgPT09IHJlZi5hdXRob3IpO1xuICAgICAgaWYgKGF1dGhvciAmJiBhdXRob3IucmVsYXlzKSB7XG4gICAgICAgIGlmICghcmVmLnJlbGF5cykge1xuICAgICAgICAgIHJlZi5yZWxheXMgPSBbXTtcbiAgICAgICAgfVxuICAgICAgICBhdXRob3IucmVsYXlzLmZvckVhY2goKHVybCkgPT4ge1xuICAgICAgICAgIGlmIChyZWYucmVsYXlzPy5pbmRleE9mKHVybCkgPT09IC0xKVxuICAgICAgICAgICAgcmVmLnJlbGF5cy5wdXNoKHVybCk7XG4gICAgICAgIH0pO1xuICAgICAgICBhdXRob3IucmVsYXlzID0gcmVmLnJlbGF5cztcbiAgICAgIH1cbiAgICB9XG4gIH0pO1xuICByZXN1bHQubWVudGlvbnMuZm9yRWFjaCgocmVmKSA9PiB7XG4gICAgaWYgKHJlZi5hdXRob3IpIHtcbiAgICAgIGxldCBhdXRob3IgPSByZXN1bHQucHJvZmlsZXMuZmluZCgocCkgPT4gcC5wdWJrZXkgPT09IHJlZi5hdXRob3IpO1xuICAgICAgaWYgKGF1dGhvciAmJiBhdXRob3IucmVsYXlzKSB7XG4gICAgICAgIGlmICghcmVmLnJlbGF5cykge1xuICAgICAgICAgIHJlZi5yZWxheXMgPSBbXTtcbiAgICAgICAgfVxuICAgICAgICBhdXRob3IucmVsYXlzLmZvckVhY2goKHVybCkgPT4ge1xuICAgICAgICAgIGlmIChyZWYucmVsYXlzLmluZGV4T2YodXJsKSA9PT0gLTEpXG4gICAgICAgICAgICByZWYucmVsYXlzLnB1c2godXJsKTtcbiAgICAgICAgfSk7XG4gICAgICAgIGF1dGhvci5yZWxheXMgPSByZWYucmVsYXlzO1xuICAgICAgfVxuICAgIH1cbiAgfSk7XG4gIHJldHVybiByZXN1bHQ7XG59XG5cbi8vIG5pcDExLnRzXG52YXIgbmlwMTFfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwMTFfZXhwb3J0cywge1xuICBmZXRjaFJlbGF5SW5mb3JtYXRpb246ICgpID0+IGZldGNoUmVsYXlJbmZvcm1hdGlvbixcbiAgdXNlRmV0Y2hJbXBsZW1lbnRhdGlvbjogKCkgPT4gdXNlRmV0Y2hJbXBsZW1lbnRhdGlvbjJcbn0pO1xudmFyIF9mZXRjaDI7XG50cnkge1xuICBfZmV0Y2gyID0gZmV0Y2g7XG59IGNhdGNoIHtcbn1cbmZ1bmN0aW9uIHVzZUZldGNoSW1wbGVtZW50YXRpb24yKGZldGNoSW1wbGVtZW50YXRpb24pIHtcbiAgX2ZldGNoMiA9IGZldGNoSW1wbGVtZW50YXRpb247XG59XG5hc3luYyBmdW5jdGlvbiBmZXRjaFJlbGF5SW5mb3JtYXRpb24odXJsKSB7XG4gIHJldHVybiBhd2FpdCAoYXdhaXQgZmV0Y2godXJsLnJlcGxhY2UoXCJ3czovL1wiLCBcImh0dHA6Ly9cIikucmVwbGFjZShcIndzczovL1wiLCBcImh0dHBzOi8vXCIpLCB7XG4gICAgaGVhZGVyczogeyBBY2NlcHQ6IFwiYXBwbGljYXRpb24vbm9zdHIranNvblwiIH1cbiAgfSkpLmpzb24oKTtcbn1cblxuLy8gbmlwMTMudHNcbnZhciBuaXAxM19leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAxM19leHBvcnRzLCB7XG4gIGZhc3RFdmVudEhhc2g6ICgpID0+IGZhc3RFdmVudEhhc2gsXG4gIGdldFBvdzogKCkgPT4gZ2V0UG93LFxuICBtaW5lUG93OiAoKSA9PiBtaW5lUG93XG59KTtcbmltcG9ydCB7IGJ5dGVzVG9IZXggYXMgYnl0ZXNUb0hleDUgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy91dGlsc1wiO1xuaW1wb3J0IHsgc2hhMjU2IGFzIHNoYTI1NjIgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy9zaGEyNTZcIjtcbmZ1bmN0aW9uIGdldFBvdyhoZXgpIHtcbiAgbGV0IGNvdW50ID0gMDtcbiAgZm9yIChsZXQgaTIgPSAwOyBpMiA8IDY0OyBpMiArPSA4KSB7XG4gICAgY29uc3QgbmliYmxlID0gcGFyc2VJbnQoaGV4LnN1YnN0cmluZyhpMiwgaTIgKyA4KSwgMTYpO1xuICAgIGlmIChuaWJibGUgPT09IDApIHtcbiAgICAgIGNvdW50ICs9IDMyO1xuICAgIH0gZWxzZSB7XG4gICAgICBjb3VudCArPSBNYXRoLmNsejMyKG5pYmJsZSk7XG4gICAgICBicmVhaztcbiAgICB9XG4gIH1cbiAgcmV0dXJuIGNvdW50O1xufVxuZnVuY3Rpb24gbWluZVBvdyh1bnNpZ25lZCwgZGlmZmljdWx0eSkge1xuICBsZXQgY291bnQgPSAwO1xuICBjb25zdCBldmVudCA9IHVuc2lnbmVkO1xuICBjb25zdCB0YWcgPSBbXCJub25jZVwiLCBjb3VudC50b1N0cmluZygpLCBkaWZmaWN1bHR5LnRvU3RyaW5nKCldO1xuICBldmVudC50YWdzLnB1c2godGFnKTtcbiAgd2hpbGUgKHRydWUpIHtcbiAgICBjb25zdCBub3cyID0gTWF0aC5mbG9vcihuZXcgRGF0ZSgpLmdldFRpbWUoKSAvIDFlMyk7XG4gICAgaWYgKG5vdzIgIT09IGV2ZW50LmNyZWF0ZWRfYXQpIHtcbiAgICAgIGNvdW50ID0gMDtcbiAgICAgIGV2ZW50LmNyZWF0ZWRfYXQgPSBub3cyO1xuICAgIH1cbiAgICB0YWdbMV0gPSAoKytjb3VudCkudG9TdHJpbmcoKTtcbiAgICBldmVudC5pZCA9IGZhc3RFdmVudEhhc2goZXZlbnQpO1xuICAgIGlmIChnZXRQb3coZXZlbnQuaWQpID49IGRpZmZpY3VsdHkpIHtcbiAgICAgIGJyZWFrO1xuICAgIH1cbiAgfVxuICByZXR1cm4gZXZlbnQ7XG59XG5mdW5jdGlvbiBmYXN0RXZlbnRIYXNoKGV2dCkge1xuICByZXR1cm4gYnl0ZXNUb0hleDUoXG4gICAgc2hhMjU2Mih1dGY4RW5jb2Rlci5lbmNvZGUoSlNPTi5zdHJpbmdpZnkoWzAsIGV2dC5wdWJrZXksIGV2dC5jcmVhdGVkX2F0LCBldnQua2luZCwgZXZ0LnRhZ3MsIGV2dC5jb250ZW50XSkpKVxuICApO1xufVxuXG4vLyBuaXAxNy50c1xudmFyIG5pcDE3X2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDE3X2V4cG9ydHMsIHtcbiAgdW53cmFwRXZlbnQ6ICgpID0+IHVud3JhcEV2ZW50MixcbiAgdW53cmFwTWFueUV2ZW50czogKCkgPT4gdW53cmFwTWFueUV2ZW50czIsXG4gIHdyYXBFdmVudDogKCkgPT4gd3JhcEV2ZW50MixcbiAgd3JhcE1hbnlFdmVudHM6ICgpID0+IHdyYXBNYW55RXZlbnRzMlxufSk7XG5cbi8vIG5pcDU5LnRzXG52YXIgbmlwNTlfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwNTlfZXhwb3J0cywge1xuICBjcmVhdGVSdW1vcjogKCkgPT4gY3JlYXRlUnVtb3IsXG4gIGNyZWF0ZVNlYWw6ICgpID0+IGNyZWF0ZVNlYWwsXG4gIGNyZWF0ZVdyYXA6ICgpID0+IGNyZWF0ZVdyYXAsXG4gIHVud3JhcEV2ZW50OiAoKSA9PiB1bndyYXBFdmVudCxcbiAgdW53cmFwTWFueUV2ZW50czogKCkgPT4gdW53cmFwTWFueUV2ZW50cyxcbiAgd3JhcEV2ZW50OiAoKSA9PiB3cmFwRXZlbnQsXG4gIHdyYXBNYW55RXZlbnRzOiAoKSA9PiB3cmFwTWFueUV2ZW50c1xufSk7XG5cbi8vIG5pcDQ0LnRzXG52YXIgbmlwNDRfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwNDRfZXhwb3J0cywge1xuICBkZWNyeXB0OiAoKSA9PiBkZWNyeXB0MixcbiAgZW5jcnlwdDogKCkgPT4gZW5jcnlwdDIsXG4gIGdldENvbnZlcnNhdGlvbktleTogKCkgPT4gZ2V0Q29udmVyc2F0aW9uS2V5LFxuICB2MjogKCkgPT4gdjJcbn0pO1xuaW1wb3J0IHsgY2hhY2hhMjAgfSBmcm9tIFwiQG5vYmxlL2NpcGhlcnMvY2hhY2hhXCI7XG5pbXBvcnQgeyBlcXVhbEJ5dGVzIH0gZnJvbSBcIkBub2JsZS9jaXBoZXJzL3V0aWxzXCI7XG5pbXBvcnQgeyBzZWNwMjU2azEgYXMgc2VjcDI1NmsxMiB9IGZyb20gXCJAbm9ibGUvY3VydmVzL3NlY3AyNTZrMVwiO1xuaW1wb3J0IHsgZXh0cmFjdCBhcyBoa2RmX2V4dHJhY3QsIGV4cGFuZCBhcyBoa2RmX2V4cGFuZCB9IGZyb20gXCJAbm9ibGUvaGFzaGVzL2hrZGZcIjtcbmltcG9ydCB7IGhtYWMgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy9obWFjXCI7XG5pbXBvcnQgeyBzaGEyNTYgYXMgc2hhMjU2MyB9IGZyb20gXCJAbm9ibGUvaGFzaGVzL3NoYTI1NlwiO1xuaW1wb3J0IHsgY29uY2F0Qnl0ZXMgYXMgY29uY2F0Qnl0ZXMyLCByYW5kb21CeXRlcyBhcyByYW5kb21CeXRlczIgfSBmcm9tIFwiQG5vYmxlL2hhc2hlcy91dGlsc1wiO1xuaW1wb3J0IHsgYmFzZTY0IGFzIGJhc2U2NDIgfSBmcm9tIFwiQHNjdXJlL2Jhc2VcIjtcbnZhciBtaW5QbGFpbnRleHRTaXplID0gMTtcbnZhciBtYXhQbGFpbnRleHRTaXplID0gNjU1MzU7XG5mdW5jdGlvbiBnZXRDb252ZXJzYXRpb25LZXkocHJpdmtleUEsIHB1YmtleUIpIHtcbiAgY29uc3Qgc2hhcmVkWCA9IHNlY3AyNTZrMTIuZ2V0U2hhcmVkU2VjcmV0KHByaXZrZXlBLCBcIjAyXCIgKyBwdWJrZXlCKS5zdWJhcnJheSgxLCAzMyk7XG4gIHJldHVybiBoa2RmX2V4dHJhY3Qoc2hhMjU2Mywgc2hhcmVkWCwgXCJuaXA0NC12MlwiKTtcbn1cbmZ1bmN0aW9uIGdldE1lc3NhZ2VLZXlzKGNvbnZlcnNhdGlvbktleSwgbm9uY2UpIHtcbiAgY29uc3Qga2V5cyA9IGhrZGZfZXhwYW5kKHNoYTI1NjMsIGNvbnZlcnNhdGlvbktleSwgbm9uY2UsIDc2KTtcbiAgcmV0dXJuIHtcbiAgICBjaGFjaGFfa2V5OiBrZXlzLnN1YmFycmF5KDAsIDMyKSxcbiAgICBjaGFjaGFfbm9uY2U6IGtleXMuc3ViYXJyYXkoMzIsIDQ0KSxcbiAgICBobWFjX2tleToga2V5cy5zdWJhcnJheSg0NCwgNzYpXG4gIH07XG59XG5mdW5jdGlvbiBjYWxjUGFkZGVkTGVuKGxlbikge1xuICBpZiAoIU51bWJlci5pc1NhZmVJbnRlZ2VyKGxlbikgfHwgbGVuIDwgMSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJleHBlY3RlZCBwb3NpdGl2ZSBpbnRlZ2VyXCIpO1xuICBpZiAobGVuIDw9IDMyKVxuICAgIHJldHVybiAzMjtcbiAgY29uc3QgbmV4dFBvd2VyID0gMSA8PCBNYXRoLmZsb29yKE1hdGgubG9nMihsZW4gLSAxKSkgKyAxO1xuICBjb25zdCBjaHVuayA9IG5leHRQb3dlciA8PSAyNTYgPyAzMiA6IG5leHRQb3dlciAvIDg7XG4gIHJldHVybiBjaHVuayAqIChNYXRoLmZsb29yKChsZW4gLSAxKSAvIGNodW5rKSArIDEpO1xufVxuZnVuY3Rpb24gd3JpdGVVMTZCRShudW0pIHtcbiAgaWYgKCFOdW1iZXIuaXNTYWZlSW50ZWdlcihudW0pIHx8IG51bSA8IG1pblBsYWludGV4dFNpemUgfHwgbnVtID4gbWF4UGxhaW50ZXh0U2l6ZSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIHBsYWludGV4dCBzaXplOiBtdXN0IGJlIGJldHdlZW4gMSBhbmQgNjU1MzUgYnl0ZXNcIik7XG4gIGNvbnN0IGFyciA9IG5ldyBVaW50OEFycmF5KDIpO1xuICBuZXcgRGF0YVZpZXcoYXJyLmJ1ZmZlcikuc2V0VWludDE2KDAsIG51bSwgZmFsc2UpO1xuICByZXR1cm4gYXJyO1xufVxuZnVuY3Rpb24gcGFkKHBsYWludGV4dCkge1xuICBjb25zdCB1bnBhZGRlZCA9IHV0ZjhFbmNvZGVyLmVuY29kZShwbGFpbnRleHQpO1xuICBjb25zdCB1bnBhZGRlZExlbiA9IHVucGFkZGVkLmxlbmd0aDtcbiAgY29uc3QgcHJlZml4ID0gd3JpdGVVMTZCRSh1bnBhZGRlZExlbik7XG4gIGNvbnN0IHN1ZmZpeCA9IG5ldyBVaW50OEFycmF5KGNhbGNQYWRkZWRMZW4odW5wYWRkZWRMZW4pIC0gdW5wYWRkZWRMZW4pO1xuICByZXR1cm4gY29uY2F0Qnl0ZXMyKHByZWZpeCwgdW5wYWRkZWQsIHN1ZmZpeCk7XG59XG5mdW5jdGlvbiB1bnBhZChwYWRkZWQpIHtcbiAgY29uc3QgdW5wYWRkZWRMZW4gPSBuZXcgRGF0YVZpZXcocGFkZGVkLmJ1ZmZlcikuZ2V0VWludDE2KDApO1xuICBjb25zdCB1bnBhZGRlZCA9IHBhZGRlZC5zdWJhcnJheSgyLCAyICsgdW5wYWRkZWRMZW4pO1xuICBpZiAodW5wYWRkZWRMZW4gPCBtaW5QbGFpbnRleHRTaXplIHx8IHVucGFkZGVkTGVuID4gbWF4UGxhaW50ZXh0U2l6ZSB8fCB1bnBhZGRlZC5sZW5ndGggIT09IHVucGFkZGVkTGVuIHx8IHBhZGRlZC5sZW5ndGggIT09IDIgKyBjYWxjUGFkZGVkTGVuKHVucGFkZGVkTGVuKSlcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIHBhZGRpbmdcIik7XG4gIHJldHVybiB1dGY4RGVjb2Rlci5kZWNvZGUodW5wYWRkZWQpO1xufVxuZnVuY3Rpb24gaG1hY0FhZChrZXksIG1lc3NhZ2UsIGFhZCkge1xuICBpZiAoYWFkLmxlbmd0aCAhPT0gMzIpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwiQUFEIGFzc29jaWF0ZWQgZGF0YSBtdXN0IGJlIDMyIGJ5dGVzXCIpO1xuICBjb25zdCBjb21iaW5lZCA9IGNvbmNhdEJ5dGVzMihhYWQsIG1lc3NhZ2UpO1xuICByZXR1cm4gaG1hYyhzaGEyNTYzLCBrZXksIGNvbWJpbmVkKTtcbn1cbmZ1bmN0aW9uIGRlY29kZVBheWxvYWQocGF5bG9hZCkge1xuICBpZiAodHlwZW9mIHBheWxvYWQgIT09IFwic3RyaW5nXCIpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwicGF5bG9hZCBtdXN0IGJlIGEgdmFsaWQgc3RyaW5nXCIpO1xuICBjb25zdCBwbGVuID0gcGF5bG9hZC5sZW5ndGg7XG4gIGlmIChwbGVuIDwgMTMyIHx8IHBsZW4gPiA4NzQ3MilcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIHBheWxvYWQgbGVuZ3RoOiBcIiArIHBsZW4pO1xuICBpZiAocGF5bG9hZFswXSA9PT0gXCIjXCIpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwidW5rbm93biBlbmNyeXB0aW9uIHZlcnNpb25cIik7XG4gIGxldCBkYXRhO1xuICB0cnkge1xuICAgIGRhdGEgPSBiYXNlNjQyLmRlY29kZShwYXlsb2FkKTtcbiAgfSBjYXRjaCAoZXJyb3IpIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIGJhc2U2NDogXCIgKyBlcnJvci5tZXNzYWdlKTtcbiAgfVxuICBjb25zdCBkbGVuID0gZGF0YS5sZW5ndGg7XG4gIGlmIChkbGVuIDwgOTkgfHwgZGxlbiA+IDY1NjAzKVxuICAgIHRocm93IG5ldyBFcnJvcihcImludmFsaWQgZGF0YSBsZW5ndGg6IFwiICsgZGxlbik7XG4gIGNvbnN0IHZlcnMgPSBkYXRhWzBdO1xuICBpZiAodmVycyAhPT0gMilcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJ1bmtub3duIGVuY3J5cHRpb24gdmVyc2lvbiBcIiArIHZlcnMpO1xuICByZXR1cm4ge1xuICAgIG5vbmNlOiBkYXRhLnN1YmFycmF5KDEsIDMzKSxcbiAgICBjaXBoZXJ0ZXh0OiBkYXRhLnN1YmFycmF5KDMzLCAtMzIpLFxuICAgIG1hYzogZGF0YS5zdWJhcnJheSgtMzIpXG4gIH07XG59XG5mdW5jdGlvbiBlbmNyeXB0MihwbGFpbnRleHQsIGNvbnZlcnNhdGlvbktleSwgbm9uY2UgPSByYW5kb21CeXRlczIoMzIpKSB7XG4gIGNvbnN0IHsgY2hhY2hhX2tleSwgY2hhY2hhX25vbmNlLCBobWFjX2tleSB9ID0gZ2V0TWVzc2FnZUtleXMoY29udmVyc2F0aW9uS2V5LCBub25jZSk7XG4gIGNvbnN0IHBhZGRlZCA9IHBhZChwbGFpbnRleHQpO1xuICBjb25zdCBjaXBoZXJ0ZXh0ID0gY2hhY2hhMjAoY2hhY2hhX2tleSwgY2hhY2hhX25vbmNlLCBwYWRkZWQpO1xuICBjb25zdCBtYWMgPSBobWFjQWFkKGhtYWNfa2V5LCBjaXBoZXJ0ZXh0LCBub25jZSk7XG4gIHJldHVybiBiYXNlNjQyLmVuY29kZShjb25jYXRCeXRlczIobmV3IFVpbnQ4QXJyYXkoWzJdKSwgbm9uY2UsIGNpcGhlcnRleHQsIG1hYykpO1xufVxuZnVuY3Rpb24gZGVjcnlwdDIocGF5bG9hZCwgY29udmVyc2F0aW9uS2V5KSB7XG4gIGNvbnN0IHsgbm9uY2UsIGNpcGhlcnRleHQsIG1hYyB9ID0gZGVjb2RlUGF5bG9hZChwYXlsb2FkKTtcbiAgY29uc3QgeyBjaGFjaGFfa2V5LCBjaGFjaGFfbm9uY2UsIGhtYWNfa2V5IH0gPSBnZXRNZXNzYWdlS2V5cyhjb252ZXJzYXRpb25LZXksIG5vbmNlKTtcbiAgY29uc3QgY2FsY3VsYXRlZE1hYyA9IGhtYWNBYWQoaG1hY19rZXksIGNpcGhlcnRleHQsIG5vbmNlKTtcbiAgaWYgKCFlcXVhbEJ5dGVzKGNhbGN1bGF0ZWRNYWMsIG1hYykpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwiaW52YWxpZCBNQUNcIik7XG4gIGNvbnN0IHBhZGRlZCA9IGNoYWNoYTIwKGNoYWNoYV9rZXksIGNoYWNoYV9ub25jZSwgY2lwaGVydGV4dCk7XG4gIHJldHVybiB1bnBhZChwYWRkZWQpO1xufVxudmFyIHYyID0ge1xuICB1dGlsczoge1xuICAgIGdldENvbnZlcnNhdGlvbktleSxcbiAgICBjYWxjUGFkZGVkTGVuXG4gIH0sXG4gIGVuY3J5cHQ6IGVuY3J5cHQyLFxuICBkZWNyeXB0OiBkZWNyeXB0MlxufTtcblxuLy8gbmlwNTkudHNcbnZhciBUV09fREFZUyA9IDIgKiAyNCAqIDYwICogNjA7XG52YXIgbm93ID0gKCkgPT4gTWF0aC5yb3VuZChEYXRlLm5vdygpIC8gMWUzKTtcbnZhciByYW5kb21Ob3cgPSAoKSA9PiBNYXRoLnJvdW5kKG5vdygpIC0gTWF0aC5yYW5kb20oKSAqIFRXT19EQVlTKTtcbnZhciBuaXA0NENvbnZlcnNhdGlvbktleSA9IChwcml2YXRlS2V5LCBwdWJsaWNLZXkpID0+IGdldENvbnZlcnNhdGlvbktleShwcml2YXRlS2V5LCBwdWJsaWNLZXkpO1xudmFyIG5pcDQ0RW5jcnlwdCA9IChkYXRhLCBwcml2YXRlS2V5LCBwdWJsaWNLZXkpID0+IGVuY3J5cHQyKEpTT04uc3RyaW5naWZ5KGRhdGEpLCBuaXA0NENvbnZlcnNhdGlvbktleShwcml2YXRlS2V5LCBwdWJsaWNLZXkpKTtcbnZhciBuaXA0NERlY3J5cHQgPSAoZGF0YSwgcHJpdmF0ZUtleSkgPT4gSlNPTi5wYXJzZShkZWNyeXB0MihkYXRhLmNvbnRlbnQsIG5pcDQ0Q29udmVyc2F0aW9uS2V5KHByaXZhdGVLZXksIGRhdGEucHVia2V5KSkpO1xuZnVuY3Rpb24gY3JlYXRlUnVtb3IoZXZlbnQsIHByaXZhdGVLZXkpIHtcbiAgY29uc3QgcnVtb3IgPSB7XG4gICAgY3JlYXRlZF9hdDogbm93KCksXG4gICAgY29udGVudDogXCJcIixcbiAgICB0YWdzOiBbXSxcbiAgICAuLi5ldmVudCxcbiAgICBwdWJrZXk6IGdldFB1YmxpY0tleShwcml2YXRlS2V5KVxuICB9O1xuICBydW1vci5pZCA9IGdldEV2ZW50SGFzaChydW1vcik7XG4gIHJldHVybiBydW1vcjtcbn1cbmZ1bmN0aW9uIGNyZWF0ZVNlYWwocnVtb3IsIHByaXZhdGVLZXksIHJlY2lwaWVudFB1YmxpY0tleSkge1xuICByZXR1cm4gZmluYWxpemVFdmVudChcbiAgICB7XG4gICAgICBraW5kOiBTZWFsLFxuICAgICAgY29udGVudDogbmlwNDRFbmNyeXB0KHJ1bW9yLCBwcml2YXRlS2V5LCByZWNpcGllbnRQdWJsaWNLZXkpLFxuICAgICAgY3JlYXRlZF9hdDogcmFuZG9tTm93KCksXG4gICAgICB0YWdzOiBbXVxuICAgIH0sXG4gICAgcHJpdmF0ZUtleVxuICApO1xufVxuZnVuY3Rpb24gY3JlYXRlV3JhcChzZWFsLCByZWNpcGllbnRQdWJsaWNLZXkpIHtcbiAgY29uc3QgcmFuZG9tS2V5ID0gZ2VuZXJhdGVTZWNyZXRLZXkoKTtcbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoXG4gICAge1xuICAgICAga2luZDogR2lmdFdyYXAsXG4gICAgICBjb250ZW50OiBuaXA0NEVuY3J5cHQoc2VhbCwgcmFuZG9tS2V5LCByZWNpcGllbnRQdWJsaWNLZXkpLFxuICAgICAgY3JlYXRlZF9hdDogcmFuZG9tTm93KCksXG4gICAgICB0YWdzOiBbW1wicFwiLCByZWNpcGllbnRQdWJsaWNLZXldXVxuICAgIH0sXG4gICAgcmFuZG9tS2V5XG4gICk7XG59XG5mdW5jdGlvbiB3cmFwRXZlbnQoZXZlbnQsIHNlbmRlclByaXZhdGVLZXksIHJlY2lwaWVudFB1YmxpY0tleSkge1xuICBjb25zdCBydW1vciA9IGNyZWF0ZVJ1bW9yKGV2ZW50LCBzZW5kZXJQcml2YXRlS2V5KTtcbiAgY29uc3Qgc2VhbCA9IGNyZWF0ZVNlYWwocnVtb3IsIHNlbmRlclByaXZhdGVLZXksIHJlY2lwaWVudFB1YmxpY0tleSk7XG4gIHJldHVybiBjcmVhdGVXcmFwKHNlYWwsIHJlY2lwaWVudFB1YmxpY0tleSk7XG59XG5mdW5jdGlvbiB3cmFwTWFueUV2ZW50cyhldmVudCwgc2VuZGVyUHJpdmF0ZUtleSwgcmVjaXBpZW50c1B1YmxpY0tleXMpIHtcbiAgaWYgKCFyZWNpcGllbnRzUHVibGljS2V5cyB8fCByZWNpcGllbnRzUHVibGljS2V5cy5sZW5ndGggPT09IDApIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJBdCBsZWFzdCBvbmUgcmVjaXBpZW50IGlzIHJlcXVpcmVkLlwiKTtcbiAgfVxuICBjb25zdCBzZW5kZXJQdWJsaWNLZXkgPSBnZXRQdWJsaWNLZXkoc2VuZGVyUHJpdmF0ZUtleSk7XG4gIGNvbnN0IHdyYXBwZWRzID0gW3dyYXBFdmVudChldmVudCwgc2VuZGVyUHJpdmF0ZUtleSwgc2VuZGVyUHVibGljS2V5KV07XG4gIHJlY2lwaWVudHNQdWJsaWNLZXlzLmZvckVhY2goKHJlY2lwaWVudFB1YmxpY0tleSkgPT4ge1xuICAgIHdyYXBwZWRzLnB1c2god3JhcEV2ZW50KGV2ZW50LCBzZW5kZXJQcml2YXRlS2V5LCByZWNpcGllbnRQdWJsaWNLZXkpKTtcbiAgfSk7XG4gIHJldHVybiB3cmFwcGVkcztcbn1cbmZ1bmN0aW9uIHVud3JhcEV2ZW50KHdyYXAsIHJlY2lwaWVudFByaXZhdGVLZXkpIHtcbiAgY29uc3QgdW53cmFwcGVkU2VhbCA9IG5pcDQ0RGVjcnlwdCh3cmFwLCByZWNpcGllbnRQcml2YXRlS2V5KTtcbiAgcmV0dXJuIG5pcDQ0RGVjcnlwdCh1bndyYXBwZWRTZWFsLCByZWNpcGllbnRQcml2YXRlS2V5KTtcbn1cbmZ1bmN0aW9uIHVud3JhcE1hbnlFdmVudHMod3JhcHBlZEV2ZW50cywgcmVjaXBpZW50UHJpdmF0ZUtleSkge1xuICBsZXQgdW53cmFwcGVkRXZlbnRzID0gW107XG4gIHdyYXBwZWRFdmVudHMuZm9yRWFjaCgoZSkgPT4ge1xuICAgIHVud3JhcHBlZEV2ZW50cy5wdXNoKHVud3JhcEV2ZW50KGUsIHJlY2lwaWVudFByaXZhdGVLZXkpKTtcbiAgfSk7XG4gIHVud3JhcHBlZEV2ZW50cy5zb3J0KChhLCBiKSA9PiBhLmNyZWF0ZWRfYXQgLSBiLmNyZWF0ZWRfYXQpO1xuICByZXR1cm4gdW53cmFwcGVkRXZlbnRzO1xufVxuXG4vLyBuaXAxNy50c1xuZnVuY3Rpb24gY3JlYXRlRXZlbnQocmVjaXBpZW50cywgbWVzc2FnZSwgY29udmVyc2F0aW9uVGl0bGUsIHJlcGx5VG8pIHtcbiAgY29uc3QgYmFzZUV2ZW50ID0ge1xuICAgIGNyZWF0ZWRfYXQ6IE1hdGguY2VpbChEYXRlLm5vdygpIC8gMWUzKSxcbiAgICBraW5kOiBQcml2YXRlRGlyZWN0TWVzc2FnZSxcbiAgICB0YWdzOiBbXSxcbiAgICBjb250ZW50OiBtZXNzYWdlXG4gIH07XG4gIGNvbnN0IHJlY2lwaWVudHNBcnJheSA9IEFycmF5LmlzQXJyYXkocmVjaXBpZW50cykgPyByZWNpcGllbnRzIDogW3JlY2lwaWVudHNdO1xuICByZWNpcGllbnRzQXJyYXkuZm9yRWFjaCgoeyBwdWJsaWNLZXksIHJlbGF5VXJsIH0pID0+IHtcbiAgICBiYXNlRXZlbnQudGFncy5wdXNoKHJlbGF5VXJsID8gW1wicFwiLCBwdWJsaWNLZXksIHJlbGF5VXJsXSA6IFtcInBcIiwgcHVibGljS2V5XSk7XG4gIH0pO1xuICBpZiAocmVwbHlUbykge1xuICAgIGJhc2VFdmVudC50YWdzLnB1c2goW1wiZVwiLCByZXBseVRvLmV2ZW50SWQsIHJlcGx5VG8ucmVsYXlVcmwgfHwgXCJcIiwgXCJyZXBseVwiXSk7XG4gIH1cbiAgaWYgKGNvbnZlcnNhdGlvblRpdGxlKSB7XG4gICAgYmFzZUV2ZW50LnRhZ3MucHVzaChbXCJzdWJqZWN0XCIsIGNvbnZlcnNhdGlvblRpdGxlXSk7XG4gIH1cbiAgcmV0dXJuIGJhc2VFdmVudDtcbn1cbmZ1bmN0aW9uIHdyYXBFdmVudDIoc2VuZGVyUHJpdmF0ZUtleSwgcmVjaXBpZW50LCBtZXNzYWdlLCBjb252ZXJzYXRpb25UaXRsZSwgcmVwbHlUbykge1xuICBjb25zdCBldmVudCA9IGNyZWF0ZUV2ZW50KHJlY2lwaWVudCwgbWVzc2FnZSwgY29udmVyc2F0aW9uVGl0bGUsIHJlcGx5VG8pO1xuICByZXR1cm4gd3JhcEV2ZW50KGV2ZW50LCBzZW5kZXJQcml2YXRlS2V5LCByZWNpcGllbnQucHVibGljS2V5KTtcbn1cbmZ1bmN0aW9uIHdyYXBNYW55RXZlbnRzMihzZW5kZXJQcml2YXRlS2V5LCByZWNpcGllbnRzLCBtZXNzYWdlLCBjb252ZXJzYXRpb25UaXRsZSwgcmVwbHlUbykge1xuICBpZiAoIXJlY2lwaWVudHMgfHwgcmVjaXBpZW50cy5sZW5ndGggPT09IDApIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJBdCBsZWFzdCBvbmUgcmVjaXBpZW50IGlzIHJlcXVpcmVkLlwiKTtcbiAgfVxuICBjb25zdCBzZW5kZXJQdWJsaWNLZXkgPSBnZXRQdWJsaWNLZXkoc2VuZGVyUHJpdmF0ZUtleSk7XG4gIHJldHVybiBbeyBwdWJsaWNLZXk6IHNlbmRlclB1YmxpY0tleSB9LCAuLi5yZWNpcGllbnRzXS5tYXAoXG4gICAgKHJlY2lwaWVudCkgPT4gd3JhcEV2ZW50MihzZW5kZXJQcml2YXRlS2V5LCByZWNpcGllbnQsIG1lc3NhZ2UsIGNvbnZlcnNhdGlvblRpdGxlLCByZXBseVRvKVxuICApO1xufVxudmFyIHVud3JhcEV2ZW50MiA9IHVud3JhcEV2ZW50O1xudmFyIHVud3JhcE1hbnlFdmVudHMyID0gdW53cmFwTWFueUV2ZW50cztcblxuLy8gbmlwMTgudHNcbnZhciBuaXAxOF9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAxOF9leHBvcnRzLCB7XG4gIGZpbmlzaFJlcG9zdEV2ZW50OiAoKSA9PiBmaW5pc2hSZXBvc3RFdmVudCxcbiAgZ2V0UmVwb3N0ZWRFdmVudDogKCkgPT4gZ2V0UmVwb3N0ZWRFdmVudCxcbiAgZ2V0UmVwb3N0ZWRFdmVudFBvaW50ZXI6ICgpID0+IGdldFJlcG9zdGVkRXZlbnRQb2ludGVyXG59KTtcbmZ1bmN0aW9uIGZpbmlzaFJlcG9zdEV2ZW50KHQsIHJlcG9zdGVkLCByZWxheVVybCwgcHJpdmF0ZUtleSkge1xuICBsZXQga2luZDtcbiAgY29uc3QgdGFncyA9IFsuLi50LnRhZ3MgPz8gW10sIFtcImVcIiwgcmVwb3N0ZWQuaWQsIHJlbGF5VXJsXSwgW1wicFwiLCByZXBvc3RlZC5wdWJrZXldXTtcbiAgaWYgKHJlcG9zdGVkLmtpbmQgPT09IFNob3J0VGV4dE5vdGUpIHtcbiAgICBraW5kID0gUmVwb3N0O1xuICB9IGVsc2Uge1xuICAgIGtpbmQgPSBHZW5lcmljUmVwb3N0O1xuICAgIHRhZ3MucHVzaChbXCJrXCIsIFN0cmluZyhyZXBvc3RlZC5raW5kKV0pO1xuICB9XG4gIHJldHVybiBmaW5hbGl6ZUV2ZW50KFxuICAgIHtcbiAgICAgIGtpbmQsXG4gICAgICB0YWdzLFxuICAgICAgY29udGVudDogdC5jb250ZW50ID09PSBcIlwiIHx8IHJlcG9zdGVkLnRhZ3M/LmZpbmQoKHRhZykgPT4gdGFnWzBdID09PSBcIi1cIikgPyBcIlwiIDogSlNPTi5zdHJpbmdpZnkocmVwb3N0ZWQpLFxuICAgICAgY3JlYXRlZF9hdDogdC5jcmVhdGVkX2F0XG4gICAgfSxcbiAgICBwcml2YXRlS2V5XG4gICk7XG59XG5mdW5jdGlvbiBnZXRSZXBvc3RlZEV2ZW50UG9pbnRlcihldmVudCkge1xuICBpZiAoIVtSZXBvc3QsIEdlbmVyaWNSZXBvc3RdLmluY2x1ZGVzKGV2ZW50LmtpbmQpKSB7XG4gICAgcmV0dXJuIHZvaWQgMDtcbiAgfVxuICBsZXQgbGFzdEVUYWc7XG4gIGxldCBsYXN0UFRhZztcbiAgZm9yIChsZXQgaTIgPSBldmVudC50YWdzLmxlbmd0aCAtIDE7IGkyID49IDAgJiYgKGxhc3RFVGFnID09PSB2b2lkIDAgfHwgbGFzdFBUYWcgPT09IHZvaWQgMCk7IGkyLS0pIHtcbiAgICBjb25zdCB0YWcgPSBldmVudC50YWdzW2kyXTtcbiAgICBpZiAodGFnLmxlbmd0aCA+PSAyKSB7XG4gICAgICBpZiAodGFnWzBdID09PSBcImVcIiAmJiBsYXN0RVRhZyA9PT0gdm9pZCAwKSB7XG4gICAgICAgIGxhc3RFVGFnID0gdGFnO1xuICAgICAgfSBlbHNlIGlmICh0YWdbMF0gPT09IFwicFwiICYmIGxhc3RQVGFnID09PSB2b2lkIDApIHtcbiAgICAgICAgbGFzdFBUYWcgPSB0YWc7XG4gICAgICB9XG4gICAgfVxuICB9XG4gIGlmIChsYXN0RVRhZyA9PT0gdm9pZCAwKSB7XG4gICAgcmV0dXJuIHZvaWQgMDtcbiAgfVxuICByZXR1cm4ge1xuICAgIGlkOiBsYXN0RVRhZ1sxXSxcbiAgICByZWxheXM6IFtsYXN0RVRhZ1syXSwgbGFzdFBUYWc/LlsyXV0uZmlsdGVyKCh4KSA9PiB0eXBlb2YgeCA9PT0gXCJzdHJpbmdcIiksXG4gICAgYXV0aG9yOiBsYXN0UFRhZz8uWzFdXG4gIH07XG59XG5mdW5jdGlvbiBnZXRSZXBvc3RlZEV2ZW50KGV2ZW50LCB7IHNraXBWZXJpZmljYXRpb24gfSA9IHt9KSB7XG4gIGNvbnN0IHBvaW50ZXIgPSBnZXRSZXBvc3RlZEV2ZW50UG9pbnRlcihldmVudCk7XG4gIGlmIChwb2ludGVyID09PSB2b2lkIDAgfHwgZXZlbnQuY29udGVudCA9PT0gXCJcIikge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgbGV0IHJlcG9zdGVkRXZlbnQ7XG4gIHRyeSB7XG4gICAgcmVwb3N0ZWRFdmVudCA9IEpTT04ucGFyc2UoZXZlbnQuY29udGVudCk7XG4gIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgcmV0dXJuIHZvaWQgMDtcbiAgfVxuICBpZiAocmVwb3N0ZWRFdmVudC5pZCAhPT0gcG9pbnRlci5pZCkge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgaWYgKCFza2lwVmVyaWZpY2F0aW9uICYmICF2ZXJpZnlFdmVudChyZXBvc3RlZEV2ZW50KSkge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgcmV0dXJuIHJlcG9zdGVkRXZlbnQ7XG59XG5cbi8vIG5pcDIxLnRzXG52YXIgbmlwMjFfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwMjFfZXhwb3J0cywge1xuICBOT1NUUl9VUklfUkVHRVg6ICgpID0+IE5PU1RSX1VSSV9SRUdFWCxcbiAgcGFyc2U6ICgpID0+IHBhcnNlMixcbiAgdGVzdDogKCkgPT4gdGVzdFxufSk7XG52YXIgTk9TVFJfVVJJX1JFR0VYID0gbmV3IFJlZ0V4cChgbm9zdHI6KCR7QkVDSDMyX1JFR0VYLnNvdXJjZX0pYCk7XG5mdW5jdGlvbiB0ZXN0KHZhbHVlKSB7XG4gIHJldHVybiB0eXBlb2YgdmFsdWUgPT09IFwic3RyaW5nXCIgJiYgbmV3IFJlZ0V4cChgXiR7Tk9TVFJfVVJJX1JFR0VYLnNvdXJjZX0kYCkudGVzdCh2YWx1ZSk7XG59XG5mdW5jdGlvbiBwYXJzZTIodXJpKSB7XG4gIGNvbnN0IG1hdGNoID0gdXJpLm1hdGNoKG5ldyBSZWdFeHAoYF4ke05PU1RSX1VSSV9SRUdFWC5zb3VyY2V9JGApKTtcbiAgaWYgKCFtYXRjaClcbiAgICB0aHJvdyBuZXcgRXJyb3IoYEludmFsaWQgTm9zdHIgVVJJOiAke3VyaX1gKTtcbiAgcmV0dXJuIHtcbiAgICB1cmk6IG1hdGNoWzBdLFxuICAgIHZhbHVlOiBtYXRjaFsxXSxcbiAgICBkZWNvZGVkOiBkZWNvZGUobWF0Y2hbMV0pXG4gIH07XG59XG5cbi8vIG5pcDI1LnRzXG52YXIgbmlwMjVfZXhwb3J0cyA9IHt9O1xuX19leHBvcnQobmlwMjVfZXhwb3J0cywge1xuICBmaW5pc2hSZWFjdGlvbkV2ZW50OiAoKSA9PiBmaW5pc2hSZWFjdGlvbkV2ZW50LFxuICBnZXRSZWFjdGVkRXZlbnRQb2ludGVyOiAoKSA9PiBnZXRSZWFjdGVkRXZlbnRQb2ludGVyXG59KTtcbmZ1bmN0aW9uIGZpbmlzaFJlYWN0aW9uRXZlbnQodCwgcmVhY3RlZCwgcHJpdmF0ZUtleSkge1xuICBjb25zdCBpbmhlcml0ZWRUYWdzID0gcmVhY3RlZC50YWdzLmZpbHRlcigodGFnKSA9PiB0YWcubGVuZ3RoID49IDIgJiYgKHRhZ1swXSA9PT0gXCJlXCIgfHwgdGFnWzBdID09PSBcInBcIikpO1xuICByZXR1cm4gZmluYWxpemVFdmVudChcbiAgICB7XG4gICAgICAuLi50LFxuICAgICAga2luZDogUmVhY3Rpb24sXG4gICAgICB0YWdzOiBbLi4udC50YWdzID8/IFtdLCAuLi5pbmhlcml0ZWRUYWdzLCBbXCJlXCIsIHJlYWN0ZWQuaWRdLCBbXCJwXCIsIHJlYWN0ZWQucHVia2V5XV0sXG4gICAgICBjb250ZW50OiB0LmNvbnRlbnQgPz8gXCIrXCJcbiAgICB9LFxuICAgIHByaXZhdGVLZXlcbiAgKTtcbn1cbmZ1bmN0aW9uIGdldFJlYWN0ZWRFdmVudFBvaW50ZXIoZXZlbnQpIHtcbiAgaWYgKGV2ZW50LmtpbmQgIT09IFJlYWN0aW9uKSB7XG4gICAgcmV0dXJuIHZvaWQgMDtcbiAgfVxuICBsZXQgbGFzdEVUYWc7XG4gIGxldCBsYXN0UFRhZztcbiAgZm9yIChsZXQgaTIgPSBldmVudC50YWdzLmxlbmd0aCAtIDE7IGkyID49IDAgJiYgKGxhc3RFVGFnID09PSB2b2lkIDAgfHwgbGFzdFBUYWcgPT09IHZvaWQgMCk7IGkyLS0pIHtcbiAgICBjb25zdCB0YWcgPSBldmVudC50YWdzW2kyXTtcbiAgICBpZiAodGFnLmxlbmd0aCA+PSAyKSB7XG4gICAgICBpZiAodGFnWzBdID09PSBcImVcIiAmJiBsYXN0RVRhZyA9PT0gdm9pZCAwKSB7XG4gICAgICAgIGxhc3RFVGFnID0gdGFnO1xuICAgICAgfSBlbHNlIGlmICh0YWdbMF0gPT09IFwicFwiICYmIGxhc3RQVGFnID09PSB2b2lkIDApIHtcbiAgICAgICAgbGFzdFBUYWcgPSB0YWc7XG4gICAgICB9XG4gICAgfVxuICB9XG4gIGlmIChsYXN0RVRhZyA9PT0gdm9pZCAwIHx8IGxhc3RQVGFnID09PSB2b2lkIDApIHtcbiAgICByZXR1cm4gdm9pZCAwO1xuICB9XG4gIHJldHVybiB7XG4gICAgaWQ6IGxhc3RFVGFnWzFdLFxuICAgIHJlbGF5czogW2xhc3RFVGFnWzJdLCBsYXN0UFRhZ1syXV0uZmlsdGVyKCh4KSA9PiB4ICE9PSB2b2lkIDApLFxuICAgIGF1dGhvcjogbGFzdFBUYWdbMV1cbiAgfTtcbn1cblxuLy8gbmlwMjcudHNcbnZhciBuaXAyN19leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAyN19leHBvcnRzLCB7XG4gIHBhcnNlOiAoKSA9PiBwYXJzZTNcbn0pO1xudmFyIG5vQ2hhcmFjdGVyID0gL1xcVy9tO1xudmFyIG5vVVJMQ2hhcmFjdGVyID0gL1xcVyB8XFxXJHwkfCx8IC9tO1xuZnVuY3Rpb24qIHBhcnNlMyhjb250ZW50KSB7XG4gIGNvbnN0IG1heCA9IGNvbnRlbnQubGVuZ3RoO1xuICBsZXQgcHJldkluZGV4ID0gMDtcbiAgbGV0IGluZGV4ID0gMDtcbiAgd2hpbGUgKGluZGV4IDwgbWF4KSB7XG4gICAgbGV0IHUgPSBjb250ZW50LmluZGV4T2YoXCI6XCIsIGluZGV4KTtcbiAgICBpZiAodSA9PT0gLTEpIHtcbiAgICAgIGJyZWFrO1xuICAgIH1cbiAgICBpZiAoY29udGVudC5zdWJzdHJpbmcodSAtIDUsIHUpID09PSBcIm5vc3RyXCIpIHtcbiAgICAgIGNvbnN0IG0gPSBjb250ZW50LnN1YnN0cmluZyh1ICsgNjApLm1hdGNoKG5vQ2hhcmFjdGVyKTtcbiAgICAgIGNvbnN0IGVuZCA9IG0gPyB1ICsgNjAgKyBtLmluZGV4IDogbWF4O1xuICAgICAgdHJ5IHtcbiAgICAgICAgbGV0IHBvaW50ZXI7XG4gICAgICAgIGxldCB7IGRhdGEsIHR5cGUgfSA9IGRlY29kZShjb250ZW50LnN1YnN0cmluZyh1ICsgMSwgZW5kKSk7XG4gICAgICAgIHN3aXRjaCAodHlwZSkge1xuICAgICAgICAgIGNhc2UgXCJucHViXCI6XG4gICAgICAgICAgICBwb2ludGVyID0geyBwdWJrZXk6IGRhdGEgfTtcbiAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgIGNhc2UgXCJuc2VjXCI6XG4gICAgICAgICAgY2FzZSBcIm5vdGVcIjpcbiAgICAgICAgICAgIGluZGV4ID0gZW5kICsgMTtcbiAgICAgICAgICAgIGNvbnRpbnVlO1xuICAgICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgICBwb2ludGVyID0gZGF0YTtcbiAgICAgICAgfVxuICAgICAgICBpZiAocHJldkluZGV4ICE9PSB1IC0gNSkge1xuICAgICAgICAgIHlpZWxkIHsgdHlwZTogXCJ0ZXh0XCIsIHRleHQ6IGNvbnRlbnQuc3Vic3RyaW5nKHByZXZJbmRleCwgdSAtIDUpIH07XG4gICAgICAgIH1cbiAgICAgICAgeWllbGQgeyB0eXBlOiBcInJlZmVyZW5jZVwiLCBwb2ludGVyIH07XG4gICAgICAgIGluZGV4ID0gZW5kO1xuICAgICAgICBwcmV2SW5kZXggPSBpbmRleDtcbiAgICAgICAgY29udGludWU7XG4gICAgICB9IGNhdGNoIChfZXJyKSB7XG4gICAgICAgIGluZGV4ID0gdSArIDE7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoY29udGVudC5zdWJzdHJpbmcodSAtIDUsIHUpID09PSBcImh0dHBzXCIgfHwgY29udGVudC5zdWJzdHJpbmcodSAtIDQsIHUpID09PSBcImh0dHBcIikge1xuICAgICAgY29uc3QgbSA9IGNvbnRlbnQuc3Vic3RyaW5nKHUgKyA0KS5tYXRjaChub1VSTENoYXJhY3Rlcik7XG4gICAgICBjb25zdCBlbmQgPSBtID8gdSArIDQgKyBtLmluZGV4IDogbWF4O1xuICAgICAgY29uc3QgcHJlZml4TGVuID0gY29udGVudFt1IC0gMV0gPT09IFwic1wiID8gNSA6IDQ7XG4gICAgICB0cnkge1xuICAgICAgICBsZXQgdXJsID0gbmV3IFVSTChjb250ZW50LnN1YnN0cmluZyh1IC0gcHJlZml4TGVuLCBlbmQpKTtcbiAgICAgICAgaWYgKHVybC5ob3N0bmFtZS5pbmRleE9mKFwiLlwiKSA9PT0gLTEpIHtcbiAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJpbnZhbGlkIHVybFwiKTtcbiAgICAgICAgfVxuICAgICAgICBpZiAocHJldkluZGV4ICE9PSB1IC0gcHJlZml4TGVuKSB7XG4gICAgICAgICAgeWllbGQgeyB0eXBlOiBcInRleHRcIiwgdGV4dDogY29udGVudC5zdWJzdHJpbmcocHJldkluZGV4LCB1IC0gcHJlZml4TGVuKSB9O1xuICAgICAgICB9XG4gICAgICAgIGlmICh1cmwucGF0aG5hbWUuZW5kc1dpdGgoXCIucG5nXCIpIHx8IHVybC5wYXRobmFtZS5lbmRzV2l0aChcIi5qcGdcIikgfHwgdXJsLnBhdGhuYW1lLmVuZHNXaXRoKFwiLmpwZWdcIikgfHwgdXJsLnBhdGhuYW1lLmVuZHNXaXRoKFwiLmdpZlwiKSB8fCB1cmwucGF0aG5hbWUuZW5kc1dpdGgoXCIud2VicFwiKSkge1xuICAgICAgICAgIHlpZWxkIHsgdHlwZTogXCJpbWFnZVwiLCB1cmw6IHVybC50b1N0cmluZygpIH07XG4gICAgICAgICAgaW5kZXggPSBlbmQ7XG4gICAgICAgICAgcHJldkluZGV4ID0gaW5kZXg7XG4gICAgICAgICAgY29udGludWU7XG4gICAgICAgIH1cbiAgICAgICAgaWYgKHVybC5wYXRobmFtZS5lbmRzV2l0aChcIi5tcDRcIikgfHwgdXJsLnBhdGhuYW1lLmVuZHNXaXRoKFwiLmF2aVwiKSB8fCB1cmwucGF0aG5hbWUuZW5kc1dpdGgoXCIud2VibVwiKSB8fCB1cmwucGF0aG5hbWUuZW5kc1dpdGgoXCIubWt2XCIpKSB7XG4gICAgICAgICAgeWllbGQgeyB0eXBlOiBcInZpZGVvXCIsIHVybDogdXJsLnRvU3RyaW5nKCkgfTtcbiAgICAgICAgICBpbmRleCA9IGVuZDtcbiAgICAgICAgICBwcmV2SW5kZXggPSBpbmRleDtcbiAgICAgICAgICBjb250aW51ZTtcbiAgICAgICAgfVxuICAgICAgICBpZiAodXJsLnBhdGhuYW1lLmVuZHNXaXRoKFwiLm1wM1wiKSB8fCB1cmwucGF0aG5hbWUuZW5kc1dpdGgoXCIuYWFjXCIpIHx8IHVybC5wYXRobmFtZS5lbmRzV2l0aChcIi5vZ2dcIikgfHwgdXJsLnBhdGhuYW1lLmVuZHNXaXRoKFwiLm9wdXNcIikpIHtcbiAgICAgICAgICB5aWVsZCB7IHR5cGU6IFwiYXVkaW9cIiwgdXJsOiB1cmwudG9TdHJpbmcoKSB9O1xuICAgICAgICAgIGluZGV4ID0gZW5kO1xuICAgICAgICAgIHByZXZJbmRleCA9IGluZGV4O1xuICAgICAgICAgIGNvbnRpbnVlO1xuICAgICAgICB9XG4gICAgICAgIHlpZWxkIHsgdHlwZTogXCJ1cmxcIiwgdXJsOiB1cmwudG9TdHJpbmcoKSB9O1xuICAgICAgICBpbmRleCA9IGVuZDtcbiAgICAgICAgcHJldkluZGV4ID0gaW5kZXg7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfSBjYXRjaCAoX2Vycikge1xuICAgICAgICBpbmRleCA9IGVuZCArIDE7XG4gICAgICAgIGNvbnRpbnVlO1xuICAgICAgfVxuICAgIH0gZWxzZSBpZiAoY29udGVudC5zdWJzdHJpbmcodSAtIDMsIHUpID09PSBcIndzc1wiIHx8IGNvbnRlbnQuc3Vic3RyaW5nKHUgLSAyLCB1KSA9PT0gXCJ3c1wiKSB7XG4gICAgICBjb25zdCBtID0gY29udGVudC5zdWJzdHJpbmcodSArIDQpLm1hdGNoKG5vVVJMQ2hhcmFjdGVyKTtcbiAgICAgIGNvbnN0IGVuZCA9IG0gPyB1ICsgNCArIG0uaW5kZXggOiBtYXg7XG4gICAgICBjb25zdCBwcmVmaXhMZW4gPSBjb250ZW50W3UgLSAxXSA9PT0gXCJzXCIgPyAzIDogMjtcbiAgICAgIHRyeSB7XG4gICAgICAgIGxldCB1cmwgPSBuZXcgVVJMKGNvbnRlbnQuc3Vic3RyaW5nKHUgLSBwcmVmaXhMZW4sIGVuZCkpO1xuICAgICAgICBpZiAodXJsLmhvc3RuYW1lLmluZGV4T2YoXCIuXCIpID09PSAtMSkge1xuICAgICAgICAgIHRocm93IG5ldyBFcnJvcihcImludmFsaWQgd3MgdXJsXCIpO1xuICAgICAgICB9XG4gICAgICAgIGlmIChwcmV2SW5kZXggIT09IHUgLSBwcmVmaXhMZW4pIHtcbiAgICAgICAgICB5aWVsZCB7IHR5cGU6IFwidGV4dFwiLCB0ZXh0OiBjb250ZW50LnN1YnN0cmluZyhwcmV2SW5kZXgsIHUgLSBwcmVmaXhMZW4pIH07XG4gICAgICAgIH1cbiAgICAgICAgeWllbGQgeyB0eXBlOiBcInJlbGF5XCIsIHVybDogdXJsLnRvU3RyaW5nKCkgfTtcbiAgICAgICAgaW5kZXggPSBlbmQ7XG4gICAgICAgIHByZXZJbmRleCA9IGluZGV4O1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH0gY2F0Y2ggKF9lcnIpIHtcbiAgICAgICAgaW5kZXggPSBlbmQgKyAxO1xuICAgICAgICBjb250aW51ZTtcbiAgICAgIH1cbiAgICB9IGVsc2Uge1xuICAgICAgaW5kZXggPSB1ICsgMTtcbiAgICAgIGNvbnRpbnVlO1xuICAgIH1cbiAgfVxuICBpZiAocHJldkluZGV4ICE9PSBtYXgpIHtcbiAgICB5aWVsZCB7IHR5cGU6IFwidGV4dFwiLCB0ZXh0OiBjb250ZW50LnN1YnN0cmluZyhwcmV2SW5kZXgpIH07XG4gIH1cbn1cblxuLy8gbmlwMjgudHNcbnZhciBuaXAyOF9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAyOF9leHBvcnRzLCB7XG4gIGNoYW5uZWxDcmVhdGVFdmVudDogKCkgPT4gY2hhbm5lbENyZWF0ZUV2ZW50LFxuICBjaGFubmVsSGlkZU1lc3NhZ2VFdmVudDogKCkgPT4gY2hhbm5lbEhpZGVNZXNzYWdlRXZlbnQsXG4gIGNoYW5uZWxNZXNzYWdlRXZlbnQ6ICgpID0+IGNoYW5uZWxNZXNzYWdlRXZlbnQsXG4gIGNoYW5uZWxNZXRhZGF0YUV2ZW50OiAoKSA9PiBjaGFubmVsTWV0YWRhdGFFdmVudCxcbiAgY2hhbm5lbE11dGVVc2VyRXZlbnQ6ICgpID0+IGNoYW5uZWxNdXRlVXNlckV2ZW50XG59KTtcbnZhciBjaGFubmVsQ3JlYXRlRXZlbnQgPSAodCwgcHJpdmF0ZUtleSkgPT4ge1xuICBsZXQgY29udGVudDtcbiAgaWYgKHR5cGVvZiB0LmNvbnRlbnQgPT09IFwib2JqZWN0XCIpIHtcbiAgICBjb250ZW50ID0gSlNPTi5zdHJpbmdpZnkodC5jb250ZW50KTtcbiAgfSBlbHNlIGlmICh0eXBlb2YgdC5jb250ZW50ID09PSBcInN0cmluZ1wiKSB7XG4gICAgY29udGVudCA9IHQuY29udGVudDtcbiAgfSBlbHNlIHtcbiAgICByZXR1cm4gdm9pZCAwO1xuICB9XG4gIHJldHVybiBmaW5hbGl6ZUV2ZW50KFxuICAgIHtcbiAgICAgIGtpbmQ6IENoYW5uZWxDcmVhdGlvbixcbiAgICAgIHRhZ3M6IFsuLi50LnRhZ3MgPz8gW11dLFxuICAgICAgY29udGVudCxcbiAgICAgIGNyZWF0ZWRfYXQ6IHQuY3JlYXRlZF9hdFxuICAgIH0sXG4gICAgcHJpdmF0ZUtleVxuICApO1xufTtcbnZhciBjaGFubmVsTWV0YWRhdGFFdmVudCA9ICh0LCBwcml2YXRlS2V5KSA9PiB7XG4gIGxldCBjb250ZW50O1xuICBpZiAodHlwZW9mIHQuY29udGVudCA9PT0gXCJvYmplY3RcIikge1xuICAgIGNvbnRlbnQgPSBKU09OLnN0cmluZ2lmeSh0LmNvbnRlbnQpO1xuICB9IGVsc2UgaWYgKHR5cGVvZiB0LmNvbnRlbnQgPT09IFwic3RyaW5nXCIpIHtcbiAgICBjb250ZW50ID0gdC5jb250ZW50O1xuICB9IGVsc2Uge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoXG4gICAge1xuICAgICAga2luZDogQ2hhbm5lbE1ldGFkYXRhLFxuICAgICAgdGFnczogW1tcImVcIiwgdC5jaGFubmVsX2NyZWF0ZV9ldmVudF9pZF0sIC4uLnQudGFncyA/PyBbXV0sXG4gICAgICBjb250ZW50LFxuICAgICAgY3JlYXRlZF9hdDogdC5jcmVhdGVkX2F0XG4gICAgfSxcbiAgICBwcml2YXRlS2V5XG4gICk7XG59O1xudmFyIGNoYW5uZWxNZXNzYWdlRXZlbnQgPSAodCwgcHJpdmF0ZUtleSkgPT4ge1xuICBjb25zdCB0YWdzID0gW1tcImVcIiwgdC5jaGFubmVsX2NyZWF0ZV9ldmVudF9pZCwgdC5yZWxheV91cmwsIFwicm9vdFwiXV07XG4gIGlmICh0LnJlcGx5X3RvX2NoYW5uZWxfbWVzc2FnZV9ldmVudF9pZCkge1xuICAgIHRhZ3MucHVzaChbXCJlXCIsIHQucmVwbHlfdG9fY2hhbm5lbF9tZXNzYWdlX2V2ZW50X2lkLCB0LnJlbGF5X3VybCwgXCJyZXBseVwiXSk7XG4gIH1cbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoXG4gICAge1xuICAgICAga2luZDogQ2hhbm5lbE1lc3NhZ2UsXG4gICAgICB0YWdzOiBbLi4udGFncywgLi4udC50YWdzID8/IFtdXSxcbiAgICAgIGNvbnRlbnQ6IHQuY29udGVudCxcbiAgICAgIGNyZWF0ZWRfYXQ6IHQuY3JlYXRlZF9hdFxuICAgIH0sXG4gICAgcHJpdmF0ZUtleVxuICApO1xufTtcbnZhciBjaGFubmVsSGlkZU1lc3NhZ2VFdmVudCA9ICh0LCBwcml2YXRlS2V5KSA9PiB7XG4gIGxldCBjb250ZW50O1xuICBpZiAodHlwZW9mIHQuY29udGVudCA9PT0gXCJvYmplY3RcIikge1xuICAgIGNvbnRlbnQgPSBKU09OLnN0cmluZ2lmeSh0LmNvbnRlbnQpO1xuICB9IGVsc2UgaWYgKHR5cGVvZiB0LmNvbnRlbnQgPT09IFwic3RyaW5nXCIpIHtcbiAgICBjb250ZW50ID0gdC5jb250ZW50O1xuICB9IGVsc2Uge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoXG4gICAge1xuICAgICAga2luZDogQ2hhbm5lbEhpZGVNZXNzYWdlLFxuICAgICAgdGFnczogW1tcImVcIiwgdC5jaGFubmVsX21lc3NhZ2VfZXZlbnRfaWRdLCAuLi50LnRhZ3MgPz8gW11dLFxuICAgICAgY29udGVudCxcbiAgICAgIGNyZWF0ZWRfYXQ6IHQuY3JlYXRlZF9hdFxuICAgIH0sXG4gICAgcHJpdmF0ZUtleVxuICApO1xufTtcbnZhciBjaGFubmVsTXV0ZVVzZXJFdmVudCA9ICh0LCBwcml2YXRlS2V5KSA9PiB7XG4gIGxldCBjb250ZW50O1xuICBpZiAodHlwZW9mIHQuY29udGVudCA9PT0gXCJvYmplY3RcIikge1xuICAgIGNvbnRlbnQgPSBKU09OLnN0cmluZ2lmeSh0LmNvbnRlbnQpO1xuICB9IGVsc2UgaWYgKHR5cGVvZiB0LmNvbnRlbnQgPT09IFwic3RyaW5nXCIpIHtcbiAgICBjb250ZW50ID0gdC5jb250ZW50O1xuICB9IGVsc2Uge1xuICAgIHJldHVybiB2b2lkIDA7XG4gIH1cbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoXG4gICAge1xuICAgICAga2luZDogQ2hhbm5lbE11dGVVc2VyLFxuICAgICAgdGFnczogW1tcInBcIiwgdC5wdWJrZXlfdG9fbXV0ZV0sIC4uLnQudGFncyA/PyBbXV0sXG4gICAgICBjb250ZW50LFxuICAgICAgY3JlYXRlZF9hdDogdC5jcmVhdGVkX2F0XG4gICAgfSxcbiAgICBwcml2YXRlS2V5XG4gICk7XG59O1xuXG4vLyBuaXAzMC50c1xudmFyIG5pcDMwX2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDMwX2V4cG9ydHMsIHtcbiAgRU1PSklfU0hPUlRDT0RFX1JFR0VYOiAoKSA9PiBFTU9KSV9TSE9SVENPREVfUkVHRVgsXG4gIG1hdGNoQWxsOiAoKSA9PiBtYXRjaEFsbCxcbiAgcmVnZXg6ICgpID0+IHJlZ2V4LFxuICByZXBsYWNlQWxsOiAoKSA9PiByZXBsYWNlQWxsXG59KTtcbnZhciBFTU9KSV9TSE9SVENPREVfUkVHRVggPSAvOihcXHcrKTovO1xudmFyIHJlZ2V4ID0gKCkgPT4gbmV3IFJlZ0V4cChgXFxcXEIke0VNT0pJX1NIT1JUQ09ERV9SRUdFWC5zb3VyY2V9XFxcXEJgLCBcImdcIik7XG5mdW5jdGlvbiogbWF0Y2hBbGwoY29udGVudCkge1xuICBjb25zdCBtYXRjaGVzID0gY29udGVudC5tYXRjaEFsbChyZWdleCgpKTtcbiAgZm9yIChjb25zdCBtYXRjaCBvZiBtYXRjaGVzKSB7XG4gICAgdHJ5IHtcbiAgICAgIGNvbnN0IFtzaG9ydGNvZGUsIG5hbWVdID0gbWF0Y2g7XG4gICAgICB5aWVsZCB7XG4gICAgICAgIHNob3J0Y29kZSxcbiAgICAgICAgbmFtZSxcbiAgICAgICAgc3RhcnQ6IG1hdGNoLmluZGV4LFxuICAgICAgICBlbmQ6IG1hdGNoLmluZGV4ICsgc2hvcnRjb2RlLmxlbmd0aFxuICAgICAgfTtcbiAgICB9IGNhdGNoIChfZSkge1xuICAgIH1cbiAgfVxufVxuZnVuY3Rpb24gcmVwbGFjZUFsbChjb250ZW50LCByZXBsYWNlcikge1xuICByZXR1cm4gY29udGVudC5yZXBsYWNlQWxsKHJlZ2V4KCksIChzaG9ydGNvZGUsIG5hbWUpID0+IHtcbiAgICByZXR1cm4gcmVwbGFjZXIoe1xuICAgICAgc2hvcnRjb2RlLFxuICAgICAgbmFtZVxuICAgIH0pO1xuICB9KTtcbn1cblxuLy8gbmlwMzkudHNcbnZhciBuaXAzOV9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXAzOV9leHBvcnRzLCB7XG4gIHVzZUZldGNoSW1wbGVtZW50YXRpb246ICgpID0+IHVzZUZldGNoSW1wbGVtZW50YXRpb24zLFxuICB2YWxpZGF0ZUdpdGh1YjogKCkgPT4gdmFsaWRhdGVHaXRodWJcbn0pO1xudmFyIF9mZXRjaDM7XG50cnkge1xuICBfZmV0Y2gzID0gZmV0Y2g7XG59IGNhdGNoIHtcbn1cbmZ1bmN0aW9uIHVzZUZldGNoSW1wbGVtZW50YXRpb24zKGZldGNoSW1wbGVtZW50YXRpb24pIHtcbiAgX2ZldGNoMyA9IGZldGNoSW1wbGVtZW50YXRpb247XG59XG5hc3luYyBmdW5jdGlvbiB2YWxpZGF0ZUdpdGh1YihwdWJrZXksIHVzZXJuYW1lLCBwcm9vZikge1xuICB0cnkge1xuICAgIGxldCByZXMgPSBhd2FpdCAoYXdhaXQgX2ZldGNoMyhgaHR0cHM6Ly9naXN0LmdpdGh1Yi5jb20vJHt1c2VybmFtZX0vJHtwcm9vZn0vcmF3YCkpLnRleHQoKTtcbiAgICByZXR1cm4gcmVzID09PSBgVmVyaWZ5aW5nIHRoYXQgSSBjb250cm9sIHRoZSBmb2xsb3dpbmcgTm9zdHIgcHVibGljIGtleTogJHtwdWJrZXl9YDtcbiAgfSBjYXRjaCAoXykge1xuICAgIHJldHVybiBmYWxzZTtcbiAgfVxufVxuXG4vLyBuaXA0Ny50c1xudmFyIG5pcDQ3X2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDQ3X2V4cG9ydHMsIHtcbiAgbWFrZU53Y1JlcXVlc3RFdmVudDogKCkgPT4gbWFrZU53Y1JlcXVlc3RFdmVudCxcbiAgcGFyc2VDb25uZWN0aW9uU3RyaW5nOiAoKSA9PiBwYXJzZUNvbm5lY3Rpb25TdHJpbmdcbn0pO1xuZnVuY3Rpb24gcGFyc2VDb25uZWN0aW9uU3RyaW5nKGNvbm5lY3Rpb25TdHJpbmcpIHtcbiAgY29uc3QgeyBwYXRobmFtZSwgc2VhcmNoUGFyYW1zIH0gPSBuZXcgVVJMKGNvbm5lY3Rpb25TdHJpbmcpO1xuICBjb25zdCBwdWJrZXkgPSBwYXRobmFtZTtcbiAgY29uc3QgcmVsYXkgPSBzZWFyY2hQYXJhbXMuZ2V0KFwicmVsYXlcIik7XG4gIGNvbnN0IHNlY3JldCA9IHNlYXJjaFBhcmFtcy5nZXQoXCJzZWNyZXRcIik7XG4gIGlmICghcHVia2V5IHx8ICFyZWxheSB8fCAhc2VjcmV0KSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKFwiaW52YWxpZCBjb25uZWN0aW9uIHN0cmluZ1wiKTtcbiAgfVxuICByZXR1cm4geyBwdWJrZXksIHJlbGF5LCBzZWNyZXQgfTtcbn1cbmFzeW5jIGZ1bmN0aW9uIG1ha2VOd2NSZXF1ZXN0RXZlbnQocHVia2V5LCBzZWNyZXRLZXksIGludm9pY2UpIHtcbiAgY29uc3QgY29udGVudCA9IHtcbiAgICBtZXRob2Q6IFwicGF5X2ludm9pY2VcIixcbiAgICBwYXJhbXM6IHtcbiAgICAgIGludm9pY2VcbiAgICB9XG4gIH07XG4gIGNvbnN0IGVuY3J5cHRlZENvbnRlbnQgPSBlbmNyeXB0KHNlY3JldEtleSwgcHVia2V5LCBKU09OLnN0cmluZ2lmeShjb250ZW50KSk7XG4gIGNvbnN0IGV2ZW50VGVtcGxhdGUgPSB7XG4gICAga2luZDogTldDV2FsbGV0UmVxdWVzdCxcbiAgICBjcmVhdGVkX2F0OiBNYXRoLnJvdW5kKERhdGUubm93KCkgLyAxZTMpLFxuICAgIGNvbnRlbnQ6IGVuY3J5cHRlZENvbnRlbnQsXG4gICAgdGFnczogW1tcInBcIiwgcHVia2V5XV1cbiAgfTtcbiAgcmV0dXJuIGZpbmFsaXplRXZlbnQoZXZlbnRUZW1wbGF0ZSwgc2VjcmV0S2V5KTtcbn1cblxuLy8gbmlwNTQudHNcbnZhciBuaXA1NF9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXA1NF9leHBvcnRzLCB7XG4gIG5vcm1hbGl6ZUlkZW50aWZpZXI6ICgpID0+IG5vcm1hbGl6ZUlkZW50aWZpZXJcbn0pO1xuZnVuY3Rpb24gbm9ybWFsaXplSWRlbnRpZmllcihuYW1lKSB7XG4gIG5hbWUgPSBuYW1lLnRyaW0oKS50b0xvd2VyQ2FzZSgpO1xuICBuYW1lID0gbmFtZS5ub3JtYWxpemUoXCJORktDXCIpO1xuICByZXR1cm4gQXJyYXkuZnJvbShuYW1lKS5tYXAoKGNoYXIpID0+IHtcbiAgICBpZiAoL1xccHtMZXR0ZXJ9L3UudGVzdChjaGFyKSB8fCAvXFxwe051bWJlcn0vdS50ZXN0KGNoYXIpKSB7XG4gICAgICByZXR1cm4gY2hhcjtcbiAgICB9XG4gICAgcmV0dXJuIFwiLVwiO1xuICB9KS5qb2luKFwiXCIpO1xufVxuXG4vLyBuaXA1Ny50c1xudmFyIG5pcDU3X2V4cG9ydHMgPSB7fTtcbl9fZXhwb3J0KG5pcDU3X2V4cG9ydHMsIHtcbiAgZ2V0U2F0b3NoaXNBbW91bnRGcm9tQm9sdDExOiAoKSA9PiBnZXRTYXRvc2hpc0Ftb3VudEZyb21Cb2x0MTEsXG4gIGdldFphcEVuZHBvaW50OiAoKSA9PiBnZXRaYXBFbmRwb2ludCxcbiAgbWFrZVphcFJlY2VpcHQ6ICgpID0+IG1ha2VaYXBSZWNlaXB0LFxuICBtYWtlWmFwUmVxdWVzdDogKCkgPT4gbWFrZVphcFJlcXVlc3QsXG4gIHVzZUZldGNoSW1wbGVtZW50YXRpb246ICgpID0+IHVzZUZldGNoSW1wbGVtZW50YXRpb240LFxuICB2YWxpZGF0ZVphcFJlcXVlc3Q6ICgpID0+IHZhbGlkYXRlWmFwUmVxdWVzdFxufSk7XG5pbXBvcnQgeyBiZWNoMzIgYXMgYmVjaDMyMiB9IGZyb20gXCJAc2N1cmUvYmFzZVwiO1xudmFyIF9mZXRjaDQ7XG50cnkge1xuICBfZmV0Y2g0ID0gZmV0Y2g7XG59IGNhdGNoIHtcbn1cbmZ1bmN0aW9uIHVzZUZldGNoSW1wbGVtZW50YXRpb240KGZldGNoSW1wbGVtZW50YXRpb24pIHtcbiAgX2ZldGNoNCA9IGZldGNoSW1wbGVtZW50YXRpb247XG59XG5hc3luYyBmdW5jdGlvbiBnZXRaYXBFbmRwb2ludChtZXRhZGF0YSkge1xuICB0cnkge1xuICAgIGxldCBsbnVybCA9IFwiXCI7XG4gICAgbGV0IHsgbHVkMDYsIGx1ZDE2IH0gPSBKU09OLnBhcnNlKG1ldGFkYXRhLmNvbnRlbnQpO1xuICAgIGlmIChsdWQwNikge1xuICAgICAgbGV0IHsgd29yZHMgfSA9IGJlY2gzMjIuZGVjb2RlKGx1ZDA2LCAxZTMpO1xuICAgICAgbGV0IGRhdGEgPSBiZWNoMzIyLmZyb21Xb3Jkcyh3b3Jkcyk7XG4gICAgICBsbnVybCA9IHV0ZjhEZWNvZGVyLmRlY29kZShkYXRhKTtcbiAgICB9IGVsc2UgaWYgKGx1ZDE2KSB7XG4gICAgICBsZXQgW25hbWUsIGRvbWFpbl0gPSBsdWQxNi5zcGxpdChcIkBcIik7XG4gICAgICBsbnVybCA9IG5ldyBVUkwoYC8ud2VsbC1rbm93bi9sbnVybHAvJHtuYW1lfWAsIGBodHRwczovLyR7ZG9tYWlufWApLnRvU3RyaW5nKCk7XG4gICAgfSBlbHNlIHtcbiAgICAgIHJldHVybiBudWxsO1xuICAgIH1cbiAgICBsZXQgcmVzID0gYXdhaXQgX2ZldGNoNChsbnVybCk7XG4gICAgbGV0IGJvZHkgPSBhd2FpdCByZXMuanNvbigpO1xuICAgIGlmIChib2R5LmFsbG93c05vc3RyICYmIGJvZHkubm9zdHJQdWJrZXkpIHtcbiAgICAgIHJldHVybiBib2R5LmNhbGxiYWNrO1xuICAgIH1cbiAgfSBjYXRjaCAoZXJyKSB7XG4gIH1cbiAgcmV0dXJuIG51bGw7XG59XG5mdW5jdGlvbiBtYWtlWmFwUmVxdWVzdCh7XG4gIHByb2ZpbGUsXG4gIGV2ZW50LFxuICBhbW91bnQsXG4gIHJlbGF5cyxcbiAgY29tbWVudCA9IFwiXCJcbn0pIHtcbiAgaWYgKCFhbW91bnQpXG4gICAgdGhyb3cgbmV3IEVycm9yKFwiYW1vdW50IG5vdCBnaXZlblwiKTtcbiAgaWYgKCFwcm9maWxlKVxuICAgIHRocm93IG5ldyBFcnJvcihcInByb2ZpbGUgbm90IGdpdmVuXCIpO1xuICBsZXQgenIgPSB7XG4gICAga2luZDogOTczNCxcbiAgICBjcmVhdGVkX2F0OiBNYXRoLnJvdW5kKERhdGUubm93KCkgLyAxZTMpLFxuICAgIGNvbnRlbnQ6IGNvbW1lbnQsXG4gICAgdGFnczogW1xuICAgICAgW1wicFwiLCBwcm9maWxlXSxcbiAgICAgIFtcImFtb3VudFwiLCBhbW91bnQudG9TdHJpbmcoKV0sXG4gICAgICBbXCJyZWxheXNcIiwgLi4ucmVsYXlzXVxuICAgIF1cbiAgfTtcbiAgaWYgKGV2ZW50ICYmIHR5cGVvZiBldmVudCA9PT0gXCJzdHJpbmdcIikge1xuICAgIHpyLnRhZ3MucHVzaChbXCJlXCIsIGV2ZW50XSk7XG4gIH1cbiAgaWYgKGV2ZW50ICYmIHR5cGVvZiBldmVudCA9PT0gXCJvYmplY3RcIikge1xuICAgIGlmIChpc1JlcGxhY2VhYmxlS2luZChldmVudC5raW5kKSkge1xuICAgICAgY29uc3QgYSA9IFtcImFcIiwgYCR7ZXZlbnQua2luZH06JHtldmVudC5wdWJrZXl9OmBdO1xuICAgICAgenIudGFncy5wdXNoKGEpO1xuICAgIH0gZWxzZSBpZiAoaXNBZGRyZXNzYWJsZUtpbmQoZXZlbnQua2luZCkpIHtcbiAgICAgIGxldCBkID0gZXZlbnQudGFncy5maW5kKChbdCwgdl0pID0+IHQgPT09IFwiZFwiICYmIHYpO1xuICAgICAgaWYgKCFkKVxuICAgICAgICB0aHJvdyBuZXcgRXJyb3IoXCJkIHRhZyBub3QgZm91bmQgb3IgaXMgZW1wdHlcIik7XG4gICAgICBjb25zdCBhID0gW1wiYVwiLCBgJHtldmVudC5raW5kfToke2V2ZW50LnB1YmtleX06JHtkWzFdfWBdO1xuICAgICAgenIudGFncy5wdXNoKGEpO1xuICAgIH1cbiAgfVxuICByZXR1cm4genI7XG59XG5mdW5jdGlvbiB2YWxpZGF0ZVphcFJlcXVlc3QoemFwUmVxdWVzdFN0cmluZykge1xuICBsZXQgemFwUmVxdWVzdDtcbiAgdHJ5IHtcbiAgICB6YXBSZXF1ZXN0ID0gSlNPTi5wYXJzZSh6YXBSZXF1ZXN0U3RyaW5nKTtcbiAgfSBjYXRjaCAoZXJyKSB7XG4gICAgcmV0dXJuIFwiSW52YWxpZCB6YXAgcmVxdWVzdCBKU09OLlwiO1xuICB9XG4gIGlmICghdmFsaWRhdGVFdmVudCh6YXBSZXF1ZXN0KSlcbiAgICByZXR1cm4gXCJaYXAgcmVxdWVzdCBpcyBub3QgYSB2YWxpZCBOb3N0ciBldmVudC5cIjtcbiAgaWYgKCF2ZXJpZnlFdmVudCh6YXBSZXF1ZXN0KSlcbiAgICByZXR1cm4gXCJJbnZhbGlkIHNpZ25hdHVyZSBvbiB6YXAgcmVxdWVzdC5cIjtcbiAgbGV0IHAgPSB6YXBSZXF1ZXN0LnRhZ3MuZmluZCgoW3QsIHZdKSA9PiB0ID09PSBcInBcIiAmJiB2KTtcbiAgaWYgKCFwKVxuICAgIHJldHVybiBcIlphcCByZXF1ZXN0IGRvZXNuJ3QgaGF2ZSBhICdwJyB0YWcuXCI7XG4gIGlmICghcFsxXS5tYXRjaCgvXlthLWYwLTldezY0fSQvKSlcbiAgICByZXR1cm4gXCJaYXAgcmVxdWVzdCAncCcgdGFnIGlzIG5vdCB2YWxpZCBoZXguXCI7XG4gIGxldCBlID0gemFwUmVxdWVzdC50YWdzLmZpbmQoKFt0LCB2XSkgPT4gdCA9PT0gXCJlXCIgJiYgdik7XG4gIGlmIChlICYmICFlWzFdLm1hdGNoKC9eW2EtZjAtOV17NjR9JC8pKVxuICAgIHJldHVybiBcIlphcCByZXF1ZXN0ICdlJyB0YWcgaXMgbm90IHZhbGlkIGhleC5cIjtcbiAgbGV0IHJlbGF5cyA9IHphcFJlcXVlc3QudGFncy5maW5kKChbdCwgdl0pID0+IHQgPT09IFwicmVsYXlzXCIgJiYgdik7XG4gIGlmICghcmVsYXlzKVxuICAgIHJldHVybiBcIlphcCByZXF1ZXN0IGRvZXNuJ3QgaGF2ZSBhICdyZWxheXMnIHRhZy5cIjtcbiAgcmV0dXJuIG51bGw7XG59XG5mdW5jdGlvbiBtYWtlWmFwUmVjZWlwdCh7XG4gIHphcFJlcXVlc3QsXG4gIHByZWltYWdlLFxuICBib2x0MTEsXG4gIHBhaWRBdFxufSkge1xuICBsZXQgenIgPSBKU09OLnBhcnNlKHphcFJlcXVlc3QpO1xuICBsZXQgdGFnc0Zyb21aYXBSZXF1ZXN0ID0genIudGFncy5maWx0ZXIoKFt0XSkgPT4gdCA9PT0gXCJlXCIgfHwgdCA9PT0gXCJwXCIgfHwgdCA9PT0gXCJhXCIpO1xuICBsZXQgemFwID0ge1xuICAgIGtpbmQ6IDk3MzUsXG4gICAgY3JlYXRlZF9hdDogTWF0aC5yb3VuZChwYWlkQXQuZ2V0VGltZSgpIC8gMWUzKSxcbiAgICBjb250ZW50OiBcIlwiLFxuICAgIHRhZ3M6IFsuLi50YWdzRnJvbVphcFJlcXVlc3QsIFtcIlBcIiwgenIucHVia2V5XSwgW1wiYm9sdDExXCIsIGJvbHQxMV0sIFtcImRlc2NyaXB0aW9uXCIsIHphcFJlcXVlc3RdXVxuICB9O1xuICBpZiAocHJlaW1hZ2UpIHtcbiAgICB6YXAudGFncy5wdXNoKFtcInByZWltYWdlXCIsIHByZWltYWdlXSk7XG4gIH1cbiAgcmV0dXJuIHphcDtcbn1cbmZ1bmN0aW9uIGdldFNhdG9zaGlzQW1vdW50RnJvbUJvbHQxMShib2x0MTEpIHtcbiAgaWYgKGJvbHQxMS5sZW5ndGggPCA1MCkge1xuICAgIHJldHVybiAwO1xuICB9XG4gIGJvbHQxMSA9IGJvbHQxMS5zdWJzdHJpbmcoMCwgNTApO1xuICBjb25zdCBpZHggPSBib2x0MTEubGFzdEluZGV4T2YoXCIxXCIpO1xuICBpZiAoaWR4ID09PSAtMSkge1xuICAgIHJldHVybiAwO1xuICB9XG4gIGNvbnN0IGhycCA9IGJvbHQxMS5zdWJzdHJpbmcoMCwgaWR4KTtcbiAgaWYgKCFocnAuc3RhcnRzV2l0aChcImxuYmNcIikpIHtcbiAgICByZXR1cm4gMDtcbiAgfVxuICBjb25zdCBhbW91bnQgPSBocnAuc3Vic3RyaW5nKDQpO1xuICBpZiAoYW1vdW50Lmxlbmd0aCA8IDEpIHtcbiAgICByZXR1cm4gMDtcbiAgfVxuICBjb25zdCBjaGFyID0gYW1vdW50W2Ftb3VudC5sZW5ndGggLSAxXTtcbiAgY29uc3QgZGlnaXQgPSBjaGFyLmNoYXJDb2RlQXQoMCkgLSBcIjBcIi5jaGFyQ29kZUF0KDApO1xuICBjb25zdCBpc0RpZ2l0ID0gZGlnaXQgPj0gMCAmJiBkaWdpdCA8PSA5O1xuICBsZXQgY3V0UG9pbnQgPSBhbW91bnQubGVuZ3RoIC0gMTtcbiAgaWYgKGlzRGlnaXQpIHtcbiAgICBjdXRQb2ludCsrO1xuICB9XG4gIGlmIChjdXRQb2ludCA8IDEpIHtcbiAgICByZXR1cm4gMDtcbiAgfVxuICBjb25zdCBudW0gPSBwYXJzZUludChhbW91bnQuc3Vic3RyaW5nKDAsIGN1dFBvaW50KSk7XG4gIHN3aXRjaCAoY2hhcikge1xuICAgIGNhc2UgXCJtXCI6XG4gICAgICByZXR1cm4gbnVtICogMWU1O1xuICAgIGNhc2UgXCJ1XCI6XG4gICAgICByZXR1cm4gbnVtICogMTAwO1xuICAgIGNhc2UgXCJuXCI6XG4gICAgICByZXR1cm4gbnVtIC8gMTA7XG4gICAgY2FzZSBcInBcIjpcbiAgICAgIHJldHVybiBudW0gLyAxZTQ7XG4gICAgZGVmYXVsdDpcbiAgICAgIHJldHVybiBudW0gKiAxZTg7XG4gIH1cbn1cblxuLy8gbmlwOTgudHNcbnZhciBuaXA5OF9leHBvcnRzID0ge307XG5fX2V4cG9ydChuaXA5OF9leHBvcnRzLCB7XG4gIGdldFRva2VuOiAoKSA9PiBnZXRUb2tlbixcbiAgaGFzaFBheWxvYWQ6ICgpID0+IGhhc2hQYXlsb2FkLFxuICB1bnBhY2tFdmVudEZyb21Ub2tlbjogKCkgPT4gdW5wYWNrRXZlbnRGcm9tVG9rZW4sXG4gIHZhbGlkYXRlRXZlbnQ6ICgpID0+IHZhbGlkYXRlRXZlbnQyLFxuICB2YWxpZGF0ZUV2ZW50S2luZDogKCkgPT4gdmFsaWRhdGVFdmVudEtpbmQsXG4gIHZhbGlkYXRlRXZlbnRNZXRob2RUYWc6ICgpID0+IHZhbGlkYXRlRXZlbnRNZXRob2RUYWcsXG4gIHZhbGlkYXRlRXZlbnRQYXlsb2FkVGFnOiAoKSA9PiB2YWxpZGF0ZUV2ZW50UGF5bG9hZFRhZyxcbiAgdmFsaWRhdGVFdmVudFRpbWVzdGFtcDogKCkgPT4gdmFsaWRhdGVFdmVudFRpbWVzdGFtcCxcbiAgdmFsaWRhdGVFdmVudFVybFRhZzogKCkgPT4gdmFsaWRhdGVFdmVudFVybFRhZyxcbiAgdmFsaWRhdGVUb2tlbjogKCkgPT4gdmFsaWRhdGVUb2tlblxufSk7XG5pbXBvcnQgeyBzaGEyNTYgYXMgc2hhMjU2NCB9IGZyb20gXCJAbm9ibGUvaGFzaGVzL3NoYTI1NlwiO1xuaW1wb3J0IHsgYnl0ZXNUb0hleCBhcyBieXRlc1RvSGV4NiB9IGZyb20gXCJAbm9ibGUvaGFzaGVzL3V0aWxzXCI7XG5pbXBvcnQgeyBiYXNlNjQgYXMgYmFzZTY0MyB9IGZyb20gXCJAc2N1cmUvYmFzZVwiO1xudmFyIF9hdXRob3JpemF0aW9uU2NoZW1lID0gXCJOb3N0ciBcIjtcbmFzeW5jIGZ1bmN0aW9uIGdldFRva2VuKGxvZ2luVXJsLCBodHRwTWV0aG9kLCBzaWduLCBpbmNsdWRlQXV0aG9yaXphdGlvblNjaGVtZSA9IGZhbHNlLCBwYXlsb2FkKSB7XG4gIGNvbnN0IGV2ZW50ID0ge1xuICAgIGtpbmQ6IEhUVFBBdXRoLFxuICAgIHRhZ3M6IFtcbiAgICAgIFtcInVcIiwgbG9naW5VcmxdLFxuICAgICAgW1wibWV0aG9kXCIsIGh0dHBNZXRob2RdXG4gICAgXSxcbiAgICBjcmVhdGVkX2F0OiBNYXRoLnJvdW5kKG5ldyBEYXRlKCkuZ2V0VGltZSgpIC8gMWUzKSxcbiAgICBjb250ZW50OiBcIlwiXG4gIH07XG4gIGlmIChwYXlsb2FkKSB7XG4gICAgZXZlbnQudGFncy5wdXNoKFtcInBheWxvYWRcIiwgaGFzaFBheWxvYWQocGF5bG9hZCldKTtcbiAgfVxuICBjb25zdCBzaWduZWRFdmVudCA9IGF3YWl0IHNpZ24oZXZlbnQpO1xuICBjb25zdCBhdXRob3JpemF0aW9uU2NoZW1lID0gaW5jbHVkZUF1dGhvcml6YXRpb25TY2hlbWUgPyBfYXV0aG9yaXphdGlvblNjaGVtZSA6IFwiXCI7XG4gIHJldHVybiBhdXRob3JpemF0aW9uU2NoZW1lICsgYmFzZTY0My5lbmNvZGUodXRmOEVuY29kZXIuZW5jb2RlKEpTT04uc3RyaW5naWZ5KHNpZ25lZEV2ZW50KSkpO1xufVxuYXN5bmMgZnVuY3Rpb24gdmFsaWRhdGVUb2tlbih0b2tlbiwgdXJsLCBtZXRob2QpIHtcbiAgY29uc3QgZXZlbnQgPSBhd2FpdCB1bnBhY2tFdmVudEZyb21Ub2tlbih0b2tlbikuY2F0Y2goKGVycm9yKSA9PiB7XG4gICAgdGhyb3cgZXJyb3I7XG4gIH0pO1xuICBjb25zdCB2YWxpZCA9IGF3YWl0IHZhbGlkYXRlRXZlbnQyKGV2ZW50LCB1cmwsIG1ldGhvZCkuY2F0Y2goKGVycm9yKSA9PiB7XG4gICAgdGhyb3cgZXJyb3I7XG4gIH0pO1xuICByZXR1cm4gdmFsaWQ7XG59XG5hc3luYyBmdW5jdGlvbiB1bnBhY2tFdmVudEZyb21Ub2tlbih0b2tlbikge1xuICBpZiAoIXRva2VuKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKFwiTWlzc2luZyB0b2tlblwiKTtcbiAgfVxuICB0b2tlbiA9IHRva2VuLnJlcGxhY2UoX2F1dGhvcml6YXRpb25TY2hlbWUsIFwiXCIpO1xuICBjb25zdCBldmVudEI2NCA9IHV0ZjhEZWNvZGVyLmRlY29kZShiYXNlNjQzLmRlY29kZSh0b2tlbikpO1xuICBpZiAoIWV2ZW50QjY0IHx8IGV2ZW50QjY0Lmxlbmd0aCA9PT0gMCB8fCAhZXZlbnRCNjQuc3RhcnRzV2l0aChcIntcIikpIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJJbnZhbGlkIHRva2VuXCIpO1xuICB9XG4gIGNvbnN0IGV2ZW50ID0gSlNPTi5wYXJzZShldmVudEI2NCk7XG4gIHJldHVybiBldmVudDtcbn1cbmZ1bmN0aW9uIHZhbGlkYXRlRXZlbnRUaW1lc3RhbXAoZXZlbnQpIHtcbiAgaWYgKCFldmVudC5jcmVhdGVkX2F0KSB7XG4gICAgcmV0dXJuIGZhbHNlO1xuICB9XG4gIHJldHVybiBNYXRoLnJvdW5kKG5ldyBEYXRlKCkuZ2V0VGltZSgpIC8gMWUzKSAtIGV2ZW50LmNyZWF0ZWRfYXQgPCA2MDtcbn1cbmZ1bmN0aW9uIHZhbGlkYXRlRXZlbnRLaW5kKGV2ZW50KSB7XG4gIHJldHVybiBldmVudC5raW5kID09PSBIVFRQQXV0aDtcbn1cbmZ1bmN0aW9uIHZhbGlkYXRlRXZlbnRVcmxUYWcoZXZlbnQsIHVybCkge1xuICBjb25zdCB1cmxUYWcgPSBldmVudC50YWdzLmZpbmQoKHQpID0+IHRbMF0gPT09IFwidVwiKTtcbiAgaWYgKCF1cmxUYWcpIHtcbiAgICByZXR1cm4gZmFsc2U7XG4gIH1cbiAgcmV0dXJuIHVybFRhZy5sZW5ndGggPiAwICYmIHVybFRhZ1sxXSA9PT0gdXJsO1xufVxuZnVuY3Rpb24gdmFsaWRhdGVFdmVudE1ldGhvZFRhZyhldmVudCwgbWV0aG9kKSB7XG4gIGNvbnN0IG1ldGhvZFRhZyA9IGV2ZW50LnRhZ3MuZmluZCgodCkgPT4gdFswXSA9PT0gXCJtZXRob2RcIik7XG4gIGlmICghbWV0aG9kVGFnKSB7XG4gICAgcmV0dXJuIGZhbHNlO1xuICB9XG4gIHJldHVybiBtZXRob2RUYWcubGVuZ3RoID4gMCAmJiBtZXRob2RUYWdbMV0udG9Mb3dlckNhc2UoKSA9PT0gbWV0aG9kLnRvTG93ZXJDYXNlKCk7XG59XG5mdW5jdGlvbiBoYXNoUGF5bG9hZChwYXlsb2FkKSB7XG4gIGNvbnN0IGhhc2ggPSBzaGEyNTY0KHV0ZjhFbmNvZGVyLmVuY29kZShKU09OLnN0cmluZ2lmeShwYXlsb2FkKSkpO1xuICByZXR1cm4gYnl0ZXNUb0hleDYoaGFzaCk7XG59XG5mdW5jdGlvbiB2YWxpZGF0ZUV2ZW50UGF5bG9hZFRhZyhldmVudCwgcGF5bG9hZCkge1xuICBjb25zdCBwYXlsb2FkVGFnID0gZXZlbnQudGFncy5maW5kKCh0KSA9PiB0WzBdID09PSBcInBheWxvYWRcIik7XG4gIGlmICghcGF5bG9hZFRhZykge1xuICAgIHJldHVybiBmYWxzZTtcbiAgfVxuICBjb25zdCBwYXlsb2FkSGFzaCA9IGhhc2hQYXlsb2FkKHBheWxvYWQpO1xuICByZXR1cm4gcGF5bG9hZFRhZy5sZW5ndGggPiAwICYmIHBheWxvYWRUYWdbMV0gPT09IHBheWxvYWRIYXNoO1xufVxuYXN5bmMgZnVuY3Rpb24gdmFsaWRhdGVFdmVudDIoZXZlbnQsIHVybCwgbWV0aG9kLCBib2R5KSB7XG4gIGlmICghdmVyaWZ5RXZlbnQoZXZlbnQpKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKFwiSW52YWxpZCBub3N0ciBldmVudCwgc2lnbmF0dXJlIGludmFsaWRcIik7XG4gIH1cbiAgaWYgKCF2YWxpZGF0ZUV2ZW50S2luZChldmVudCkpIHtcbiAgICB0aHJvdyBuZXcgRXJyb3IoXCJJbnZhbGlkIG5vc3RyIGV2ZW50LCBraW5kIGludmFsaWRcIik7XG4gIH1cbiAgaWYgKCF2YWxpZGF0ZUV2ZW50VGltZXN0YW1wKGV2ZW50KSkge1xuICAgIHRocm93IG5ldyBFcnJvcihcIkludmFsaWQgbm9zdHIgZXZlbnQsIGNyZWF0ZWRfYXQgdGltZXN0YW1wIGludmFsaWRcIik7XG4gIH1cbiAgaWYgKCF2YWxpZGF0ZUV2ZW50VXJsVGFnKGV2ZW50LCB1cmwpKSB7XG4gICAgdGhyb3cgbmV3IEVycm9yKFwiSW52YWxpZCBub3N0ciBldmVudCwgdXJsIHRhZyBpbnZhbGlkXCIpO1xuICB9XG4gIGlmICghdmFsaWRhdGVFdmVudE1ldGhvZFRhZyhldmVudCwgbWV0aG9kKSkge1xuICAgIHRocm93IG5ldyBFcnJvcihcIkludmFsaWQgbm9zdHIgZXZlbnQsIG1ldGhvZCB0YWcgaW52YWxpZFwiKTtcbiAgfVxuICBpZiAoQm9vbGVhbihib2R5KSAmJiB0eXBlb2YgYm9keSA9PT0gXCJvYmplY3RcIiAmJiBPYmplY3Qua2V5cyhib2R5KS5sZW5ndGggPiAwKSB7XG4gICAgaWYgKCF2YWxpZGF0ZUV2ZW50UGF5bG9hZFRhZyhldmVudCwgYm9keSkpIHtcbiAgICAgIHRocm93IG5ldyBFcnJvcihcIkludmFsaWQgbm9zdHIgZXZlbnQsIHBheWxvYWQgdGFnIGRvZXMgbm90IG1hdGNoIHJlcXVlc3QgYm9keSBoYXNoXCIpO1xuICAgIH1cbiAgfVxuICByZXR1cm4gdHJ1ZTtcbn1cbmV4cG9ydCB7XG4gIFJlbGF5LFxuICBTaW1wbGVQb29sLFxuICBmaW5hbGl6ZUV2ZW50LFxuICBmYWtlanNvbl9leHBvcnRzIGFzIGZqLFxuICBnZW5lcmF0ZVNlY3JldEtleSxcbiAgZ2V0RXZlbnRIYXNoLFxuICBnZXRGaWx0ZXJMaW1pdCxcbiAgZ2V0UHVibGljS2V5LFxuICBraW5kc19leHBvcnRzIGFzIGtpbmRzLFxuICBtYXRjaEZpbHRlcixcbiAgbWF0Y2hGaWx0ZXJzLFxuICBtZXJnZUZpbHRlcnMsXG4gIG5pcDA0X2V4cG9ydHMgYXMgbmlwMDQsXG4gIG5pcDA1X2V4cG9ydHMgYXMgbmlwMDUsXG4gIG5pcDEwX2V4cG9ydHMgYXMgbmlwMTAsXG4gIG5pcDExX2V4cG9ydHMgYXMgbmlwMTEsXG4gIG5pcDEzX2V4cG9ydHMgYXMgbmlwMTMsXG4gIG5pcDE3X2V4cG9ydHMgYXMgbmlwMTcsXG4gIG5pcDE4X2V4cG9ydHMgYXMgbmlwMTgsXG4gIG5pcDE5X2V4cG9ydHMgYXMgbmlwMTksXG4gIG5pcDIxX2V4cG9ydHMgYXMgbmlwMjEsXG4gIG5pcDI1X2V4cG9ydHMgYXMgbmlwMjUsXG4gIG5pcDI3X2V4cG9ydHMgYXMgbmlwMjcsXG4gIG5pcDI4X2V4cG9ydHMgYXMgbmlwMjgsXG4gIG5pcDMwX2V4cG9ydHMgYXMgbmlwMzAsXG4gIG5pcDM5X2V4cG9ydHMgYXMgbmlwMzksXG4gIG5pcDQyX2V4cG9ydHMgYXMgbmlwNDIsXG4gIG5pcDQ0X2V4cG9ydHMgYXMgbmlwNDQsXG4gIG5pcDQ3X2V4cG9ydHMgYXMgbmlwNDcsXG4gIG5pcDU0X2V4cG9ydHMgYXMgbmlwNTQsXG4gIG5pcDU3X2V4cG9ydHMgYXMgbmlwNTcsXG4gIG5pcDU5X2V4cG9ydHMgYXMgbmlwNTksXG4gIG5pcDk4X2V4cG9ydHMgYXMgbmlwOTgsXG4gIHBhcnNlUmVmZXJlbmNlcyxcbiAgc2VyaWFsaXplRXZlbnQsXG4gIHNvcnRFdmVudHMsXG4gIHV0aWxzX2V4cG9ydHMgYXMgdXRpbHMsXG4gIHZhbGlkYXRlRXZlbnQsXG4gIHZlcmlmaWVkU3ltYm9sLFxuICB2ZXJpZnlFdmVudFxufTtcbiIsICJjb25zdCBFX1RJTUVPVVQgPSBuZXcgRXJyb3IoJ3RpbWVvdXQgd2hpbGUgd2FpdGluZyBmb3IgbXV0ZXggdG8gYmVjb21lIGF2YWlsYWJsZScpO1xuY29uc3QgRV9BTFJFQURZX0xPQ0tFRCA9IG5ldyBFcnJvcignbXV0ZXggYWxyZWFkeSBsb2NrZWQnKTtcbmNvbnN0IEVfQ0FOQ0VMRUQgPSBuZXcgRXJyb3IoJ3JlcXVlc3QgZm9yIGxvY2sgY2FuY2VsZWQnKTtcblxudmFyIF9fYXdhaXRlciQyID0gKHVuZGVmaW5lZCAmJiB1bmRlZmluZWQuX19hd2FpdGVyKSB8fCBmdW5jdGlvbiAodGhpc0FyZywgX2FyZ3VtZW50cywgUCwgZ2VuZXJhdG9yKSB7XG4gICAgZnVuY3Rpb24gYWRvcHQodmFsdWUpIHsgcmV0dXJuIHZhbHVlIGluc3RhbmNlb2YgUCA/IHZhbHVlIDogbmV3IFAoZnVuY3Rpb24gKHJlc29sdmUpIHsgcmVzb2x2ZSh2YWx1ZSk7IH0pOyB9XG4gICAgcmV0dXJuIG5ldyAoUCB8fCAoUCA9IFByb21pc2UpKShmdW5jdGlvbiAocmVzb2x2ZSwgcmVqZWN0KSB7XG4gICAgICAgIGZ1bmN0aW9uIGZ1bGZpbGxlZCh2YWx1ZSkgeyB0cnkgeyBzdGVwKGdlbmVyYXRvci5uZXh0KHZhbHVlKSk7IH0gY2F0Y2ggKGUpIHsgcmVqZWN0KGUpOyB9IH1cbiAgICAgICAgZnVuY3Rpb24gcmVqZWN0ZWQodmFsdWUpIHsgdHJ5IHsgc3RlcChnZW5lcmF0b3JbXCJ0aHJvd1wiXSh2YWx1ZSkpOyB9IGNhdGNoIChlKSB7IHJlamVjdChlKTsgfSB9XG4gICAgICAgIGZ1bmN0aW9uIHN0ZXAocmVzdWx0KSB7IHJlc3VsdC5kb25lID8gcmVzb2x2ZShyZXN1bHQudmFsdWUpIDogYWRvcHQocmVzdWx0LnZhbHVlKS50aGVuKGZ1bGZpbGxlZCwgcmVqZWN0ZWQpOyB9XG4gICAgICAgIHN0ZXAoKGdlbmVyYXRvciA9IGdlbmVyYXRvci5hcHBseSh0aGlzQXJnLCBfYXJndW1lbnRzIHx8IFtdKSkubmV4dCgpKTtcbiAgICB9KTtcbn07XG5jbGFzcyBTZW1hcGhvcmUge1xuICAgIGNvbnN0cnVjdG9yKF92YWx1ZSwgX2NhbmNlbEVycm9yID0gRV9DQU5DRUxFRCkge1xuICAgICAgICB0aGlzLl92YWx1ZSA9IF92YWx1ZTtcbiAgICAgICAgdGhpcy5fY2FuY2VsRXJyb3IgPSBfY2FuY2VsRXJyb3I7XG4gICAgICAgIHRoaXMuX3F1ZXVlID0gW107XG4gICAgICAgIHRoaXMuX3dlaWdodGVkV2FpdGVycyA9IFtdO1xuICAgIH1cbiAgICBhY3F1aXJlKHdlaWdodCA9IDEsIHByaW9yaXR5ID0gMCkge1xuICAgICAgICBpZiAod2VpZ2h0IDw9IDApXG4gICAgICAgICAgICB0aHJvdyBuZXcgRXJyb3IoYGludmFsaWQgd2VpZ2h0ICR7d2VpZ2h0fTogbXVzdCBiZSBwb3NpdGl2ZWApO1xuICAgICAgICByZXR1cm4gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4ge1xuICAgICAgICAgICAgY29uc3QgdGFzayA9IHsgcmVzb2x2ZSwgcmVqZWN0LCB3ZWlnaHQsIHByaW9yaXR5IH07XG4gICAgICAgICAgICBjb25zdCBpID0gZmluZEluZGV4RnJvbUVuZCh0aGlzLl9xdWV1ZSwgKG90aGVyKSA9PiBwcmlvcml0eSA8PSBvdGhlci5wcmlvcml0eSk7XG4gICAgICAgICAgICBpZiAoaSA9PT0gLTEgJiYgd2VpZ2h0IDw9IHRoaXMuX3ZhbHVlKSB7XG4gICAgICAgICAgICAgICAgLy8gTmVlZHMgaW1tZWRpYXRlIGRpc3BhdGNoLCBza2lwIHRoZSBxdWV1ZVxuICAgICAgICAgICAgICAgIHRoaXMuX2Rpc3BhdGNoSXRlbSh0YXNrKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGVsc2Uge1xuICAgICAgICAgICAgICAgIHRoaXMuX3F1ZXVlLnNwbGljZShpICsgMSwgMCwgdGFzayk7XG4gICAgICAgICAgICB9XG4gICAgICAgIH0pO1xuICAgIH1cbiAgICBydW5FeGNsdXNpdmUoY2FsbGJhY2tfMSkge1xuICAgICAgICByZXR1cm4gX19hd2FpdGVyJDIodGhpcywgYXJndW1lbnRzLCB2b2lkIDAsIGZ1bmN0aW9uKiAoY2FsbGJhY2ssIHdlaWdodCA9IDEsIHByaW9yaXR5ID0gMCkge1xuICAgICAgICAgICAgY29uc3QgW3ZhbHVlLCByZWxlYXNlXSA9IHlpZWxkIHRoaXMuYWNxdWlyZSh3ZWlnaHQsIHByaW9yaXR5KTtcbiAgICAgICAgICAgIHRyeSB7XG4gICAgICAgICAgICAgICAgcmV0dXJuIHlpZWxkIGNhbGxiYWNrKHZhbHVlKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGZpbmFsbHkge1xuICAgICAgICAgICAgICAgIHJlbGVhc2UoKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgfSk7XG4gICAgfVxuICAgIHdhaXRGb3JVbmxvY2sod2VpZ2h0ID0gMSwgcHJpb3JpdHkgPSAwKSB7XG4gICAgICAgIGlmICh3ZWlnaHQgPD0gMClcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgaW52YWxpZCB3ZWlnaHQgJHt3ZWlnaHR9OiBtdXN0IGJlIHBvc2l0aXZlYCk7XG4gICAgICAgIGlmICh0aGlzLl9jb3VsZExvY2tJbW1lZGlhdGVseSh3ZWlnaHQsIHByaW9yaXR5KSkge1xuICAgICAgICAgICAgcmV0dXJuIFByb21pc2UucmVzb2x2ZSgpO1xuICAgICAgICB9XG4gICAgICAgIGVsc2Uge1xuICAgICAgICAgICAgcmV0dXJuIG5ldyBQcm9taXNlKChyZXNvbHZlKSA9PiB7XG4gICAgICAgICAgICAgICAgaWYgKCF0aGlzLl93ZWlnaHRlZFdhaXRlcnNbd2VpZ2h0IC0gMV0pXG4gICAgICAgICAgICAgICAgICAgIHRoaXMuX3dlaWdodGVkV2FpdGVyc1t3ZWlnaHQgLSAxXSA9IFtdO1xuICAgICAgICAgICAgICAgIGluc2VydFNvcnRlZCh0aGlzLl93ZWlnaHRlZFdhaXRlcnNbd2VpZ2h0IC0gMV0sIHsgcmVzb2x2ZSwgcHJpb3JpdHkgfSk7XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgfVxuICAgIH1cbiAgICBpc0xvY2tlZCgpIHtcbiAgICAgICAgcmV0dXJuIHRoaXMuX3ZhbHVlIDw9IDA7XG4gICAgfVxuICAgIGdldFZhbHVlKCkge1xuICAgICAgICByZXR1cm4gdGhpcy5fdmFsdWU7XG4gICAgfVxuICAgIHNldFZhbHVlKHZhbHVlKSB7XG4gICAgICAgIHRoaXMuX3ZhbHVlID0gdmFsdWU7XG4gICAgICAgIHRoaXMuX2Rpc3BhdGNoUXVldWUoKTtcbiAgICB9XG4gICAgcmVsZWFzZSh3ZWlnaHQgPSAxKSB7XG4gICAgICAgIGlmICh3ZWlnaHQgPD0gMClcbiAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgaW52YWxpZCB3ZWlnaHQgJHt3ZWlnaHR9OiBtdXN0IGJlIHBvc2l0aXZlYCk7XG4gICAgICAgIHRoaXMuX3ZhbHVlICs9IHdlaWdodDtcbiAgICAgICAgdGhpcy5fZGlzcGF0Y2hRdWV1ZSgpO1xuICAgIH1cbiAgICBjYW5jZWwoKSB7XG4gICAgICAgIHRoaXMuX3F1ZXVlLmZvckVhY2goKGVudHJ5KSA9PiBlbnRyeS5yZWplY3QodGhpcy5fY2FuY2VsRXJyb3IpKTtcbiAgICAgICAgdGhpcy5fcXVldWUgPSBbXTtcbiAgICB9XG4gICAgX2Rpc3BhdGNoUXVldWUoKSB7XG4gICAgICAgIHRoaXMuX2RyYWluVW5sb2NrV2FpdGVycygpO1xuICAgICAgICB3aGlsZSAodGhpcy5fcXVldWUubGVuZ3RoID4gMCAmJiB0aGlzLl9xdWV1ZVswXS53ZWlnaHQgPD0gdGhpcy5fdmFsdWUpIHtcbiAgICAgICAgICAgIHRoaXMuX2Rpc3BhdGNoSXRlbSh0aGlzLl9xdWV1ZS5zaGlmdCgpKTtcbiAgICAgICAgICAgIHRoaXMuX2RyYWluVW5sb2NrV2FpdGVycygpO1xuICAgICAgICB9XG4gICAgfVxuICAgIF9kaXNwYXRjaEl0ZW0oaXRlbSkge1xuICAgICAgICBjb25zdCBwcmV2aW91c1ZhbHVlID0gdGhpcy5fdmFsdWU7XG4gICAgICAgIHRoaXMuX3ZhbHVlIC09IGl0ZW0ud2VpZ2h0O1xuICAgICAgICBpdGVtLnJlc29sdmUoW3ByZXZpb3VzVmFsdWUsIHRoaXMuX25ld1JlbGVhc2VyKGl0ZW0ud2VpZ2h0KV0pO1xuICAgIH1cbiAgICBfbmV3UmVsZWFzZXIod2VpZ2h0KSB7XG4gICAgICAgIGxldCBjYWxsZWQgPSBmYWxzZTtcbiAgICAgICAgcmV0dXJuICgpID0+IHtcbiAgICAgICAgICAgIGlmIChjYWxsZWQpXG4gICAgICAgICAgICAgICAgcmV0dXJuO1xuICAgICAgICAgICAgY2FsbGVkID0gdHJ1ZTtcbiAgICAgICAgICAgIHRoaXMucmVsZWFzZSh3ZWlnaHQpO1xuICAgICAgICB9O1xuICAgIH1cbiAgICBfZHJhaW5VbmxvY2tXYWl0ZXJzKCkge1xuICAgICAgICBpZiAodGhpcy5fcXVldWUubGVuZ3RoID09PSAwKSB7XG4gICAgICAgICAgICBmb3IgKGxldCB3ZWlnaHQgPSB0aGlzLl92YWx1ZTsgd2VpZ2h0ID4gMDsgd2VpZ2h0LS0pIHtcbiAgICAgICAgICAgICAgICBjb25zdCB3YWl0ZXJzID0gdGhpcy5fd2VpZ2h0ZWRXYWl0ZXJzW3dlaWdodCAtIDFdO1xuICAgICAgICAgICAgICAgIGlmICghd2FpdGVycylcbiAgICAgICAgICAgICAgICAgICAgY29udGludWU7XG4gICAgICAgICAgICAgICAgd2FpdGVycy5mb3JFYWNoKCh3YWl0ZXIpID0+IHdhaXRlci5yZXNvbHZlKCkpO1xuICAgICAgICAgICAgICAgIHRoaXMuX3dlaWdodGVkV2FpdGVyc1t3ZWlnaHQgLSAxXSA9IFtdO1xuICAgICAgICAgICAgfVxuICAgICAgICB9XG4gICAgICAgIGVsc2Uge1xuICAgICAgICAgICAgY29uc3QgcXVldWVkUHJpb3JpdHkgPSB0aGlzLl9xdWV1ZVswXS5wcmlvcml0eTtcbiAgICAgICAgICAgIGZvciAobGV0IHdlaWdodCA9IHRoaXMuX3ZhbHVlOyB3ZWlnaHQgPiAwOyB3ZWlnaHQtLSkge1xuICAgICAgICAgICAgICAgIGNvbnN0IHdhaXRlcnMgPSB0aGlzLl93ZWlnaHRlZFdhaXRlcnNbd2VpZ2h0IC0gMV07XG4gICAgICAgICAgICAgICAgaWYgKCF3YWl0ZXJzKVxuICAgICAgICAgICAgICAgICAgICBjb250aW51ZTtcbiAgICAgICAgICAgICAgICBjb25zdCBpID0gd2FpdGVycy5maW5kSW5kZXgoKHdhaXRlcikgPT4gd2FpdGVyLnByaW9yaXR5IDw9IHF1ZXVlZFByaW9yaXR5KTtcbiAgICAgICAgICAgICAgICAoaSA9PT0gLTEgPyB3YWl0ZXJzIDogd2FpdGVycy5zcGxpY2UoMCwgaSkpXG4gICAgICAgICAgICAgICAgICAgIC5mb3JFYWNoKCh3YWl0ZXIgPT4gd2FpdGVyLnJlc29sdmUoKSkpO1xuICAgICAgICAgICAgfVxuICAgICAgICB9XG4gICAgfVxuICAgIF9jb3VsZExvY2tJbW1lZGlhdGVseSh3ZWlnaHQsIHByaW9yaXR5KSB7XG4gICAgICAgIHJldHVybiAodGhpcy5fcXVldWUubGVuZ3RoID09PSAwIHx8IHRoaXMuX3F1ZXVlWzBdLnByaW9yaXR5IDwgcHJpb3JpdHkpICYmXG4gICAgICAgICAgICB3ZWlnaHQgPD0gdGhpcy5fdmFsdWU7XG4gICAgfVxufVxuZnVuY3Rpb24gaW5zZXJ0U29ydGVkKGEsIHYpIHtcbiAgICBjb25zdCBpID0gZmluZEluZGV4RnJvbUVuZChhLCAob3RoZXIpID0+IHYucHJpb3JpdHkgPD0gb3RoZXIucHJpb3JpdHkpO1xuICAgIGEuc3BsaWNlKGkgKyAxLCAwLCB2KTtcbn1cbmZ1bmN0aW9uIGZpbmRJbmRleEZyb21FbmQoYSwgcHJlZGljYXRlKSB7XG4gICAgZm9yIChsZXQgaSA9IGEubGVuZ3RoIC0gMTsgaSA+PSAwOyBpLS0pIHtcbiAgICAgICAgaWYgKHByZWRpY2F0ZShhW2ldKSkge1xuICAgICAgICAgICAgcmV0dXJuIGk7XG4gICAgICAgIH1cbiAgICB9XG4gICAgcmV0dXJuIC0xO1xufVxuXG52YXIgX19hd2FpdGVyJDEgPSAodW5kZWZpbmVkICYmIHVuZGVmaW5lZC5fX2F3YWl0ZXIpIHx8IGZ1bmN0aW9uICh0aGlzQXJnLCBfYXJndW1lbnRzLCBQLCBnZW5lcmF0b3IpIHtcbiAgICBmdW5jdGlvbiBhZG9wdCh2YWx1ZSkgeyByZXR1cm4gdmFsdWUgaW5zdGFuY2VvZiBQID8gdmFsdWUgOiBuZXcgUChmdW5jdGlvbiAocmVzb2x2ZSkgeyByZXNvbHZlKHZhbHVlKTsgfSk7IH1cbiAgICByZXR1cm4gbmV3IChQIHx8IChQID0gUHJvbWlzZSkpKGZ1bmN0aW9uIChyZXNvbHZlLCByZWplY3QpIHtcbiAgICAgICAgZnVuY3Rpb24gZnVsZmlsbGVkKHZhbHVlKSB7IHRyeSB7IHN0ZXAoZ2VuZXJhdG9yLm5leHQodmFsdWUpKTsgfSBjYXRjaCAoZSkgeyByZWplY3QoZSk7IH0gfVxuICAgICAgICBmdW5jdGlvbiByZWplY3RlZCh2YWx1ZSkgeyB0cnkgeyBzdGVwKGdlbmVyYXRvcltcInRocm93XCJdKHZhbHVlKSk7IH0gY2F0Y2ggKGUpIHsgcmVqZWN0KGUpOyB9IH1cbiAgICAgICAgZnVuY3Rpb24gc3RlcChyZXN1bHQpIHsgcmVzdWx0LmRvbmUgPyByZXNvbHZlKHJlc3VsdC52YWx1ZSkgOiBhZG9wdChyZXN1bHQudmFsdWUpLnRoZW4oZnVsZmlsbGVkLCByZWplY3RlZCk7IH1cbiAgICAgICAgc3RlcCgoZ2VuZXJhdG9yID0gZ2VuZXJhdG9yLmFwcGx5KHRoaXNBcmcsIF9hcmd1bWVudHMgfHwgW10pKS5uZXh0KCkpO1xuICAgIH0pO1xufTtcbmNsYXNzIE11dGV4IHtcbiAgICBjb25zdHJ1Y3RvcihjYW5jZWxFcnJvcikge1xuICAgICAgICB0aGlzLl9zZW1hcGhvcmUgPSBuZXcgU2VtYXBob3JlKDEsIGNhbmNlbEVycm9yKTtcbiAgICB9XG4gICAgYWNxdWlyZSgpIHtcbiAgICAgICAgcmV0dXJuIF9fYXdhaXRlciQxKHRoaXMsIGFyZ3VtZW50cywgdm9pZCAwLCBmdW5jdGlvbiogKHByaW9yaXR5ID0gMCkge1xuICAgICAgICAgICAgY29uc3QgWywgcmVsZWFzZXJdID0geWllbGQgdGhpcy5fc2VtYXBob3JlLmFjcXVpcmUoMSwgcHJpb3JpdHkpO1xuICAgICAgICAgICAgcmV0dXJuIHJlbGVhc2VyO1xuICAgICAgICB9KTtcbiAgICB9XG4gICAgcnVuRXhjbHVzaXZlKGNhbGxiYWNrLCBwcmlvcml0eSA9IDApIHtcbiAgICAgICAgcmV0dXJuIHRoaXMuX3NlbWFwaG9yZS5ydW5FeGNsdXNpdmUoKCkgPT4gY2FsbGJhY2soKSwgMSwgcHJpb3JpdHkpO1xuICAgIH1cbiAgICBpc0xvY2tlZCgpIHtcbiAgICAgICAgcmV0dXJuIHRoaXMuX3NlbWFwaG9yZS5pc0xvY2tlZCgpO1xuICAgIH1cbiAgICB3YWl0Rm9yVW5sb2NrKHByaW9yaXR5ID0gMCkge1xuICAgICAgICByZXR1cm4gdGhpcy5fc2VtYXBob3JlLndhaXRGb3JVbmxvY2soMSwgcHJpb3JpdHkpO1xuICAgIH1cbiAgICByZWxlYXNlKCkge1xuICAgICAgICBpZiAodGhpcy5fc2VtYXBob3JlLmlzTG9ja2VkKCkpXG4gICAgICAgICAgICB0aGlzLl9zZW1hcGhvcmUucmVsZWFzZSgpO1xuICAgIH1cbiAgICBjYW5jZWwoKSB7XG4gICAgICAgIHJldHVybiB0aGlzLl9zZW1hcGhvcmUuY2FuY2VsKCk7XG4gICAgfVxufVxuXG52YXIgX19hd2FpdGVyID0gKHVuZGVmaW5lZCAmJiB1bmRlZmluZWQuX19hd2FpdGVyKSB8fCBmdW5jdGlvbiAodGhpc0FyZywgX2FyZ3VtZW50cywgUCwgZ2VuZXJhdG9yKSB7XG4gICAgZnVuY3Rpb24gYWRvcHQodmFsdWUpIHsgcmV0dXJuIHZhbHVlIGluc3RhbmNlb2YgUCA/IHZhbHVlIDogbmV3IFAoZnVuY3Rpb24gKHJlc29sdmUpIHsgcmVzb2x2ZSh2YWx1ZSk7IH0pOyB9XG4gICAgcmV0dXJuIG5ldyAoUCB8fCAoUCA9IFByb21pc2UpKShmdW5jdGlvbiAocmVzb2x2ZSwgcmVqZWN0KSB7XG4gICAgICAgIGZ1bmN0aW9uIGZ1bGZpbGxlZCh2YWx1ZSkgeyB0cnkgeyBzdGVwKGdlbmVyYXRvci5uZXh0KHZhbHVlKSk7IH0gY2F0Y2ggKGUpIHsgcmVqZWN0KGUpOyB9IH1cbiAgICAgICAgZnVuY3Rpb24gcmVqZWN0ZWQodmFsdWUpIHsgdHJ5IHsgc3RlcChnZW5lcmF0b3JbXCJ0aHJvd1wiXSh2YWx1ZSkpOyB9IGNhdGNoIChlKSB7IHJlamVjdChlKTsgfSB9XG4gICAgICAgIGZ1bmN0aW9uIHN0ZXAocmVzdWx0KSB7IHJlc3VsdC5kb25lID8gcmVzb2x2ZShyZXN1bHQudmFsdWUpIDogYWRvcHQocmVzdWx0LnZhbHVlKS50aGVuKGZ1bGZpbGxlZCwgcmVqZWN0ZWQpOyB9XG4gICAgICAgIHN0ZXAoKGdlbmVyYXRvciA9IGdlbmVyYXRvci5hcHBseSh0aGlzQXJnLCBfYXJndW1lbnRzIHx8IFtdKSkubmV4dCgpKTtcbiAgICB9KTtcbn07XG5mdW5jdGlvbiB3aXRoVGltZW91dChzeW5jLCB0aW1lb3V0LCB0aW1lb3V0RXJyb3IgPSBFX1RJTUVPVVQpIHtcbiAgICByZXR1cm4ge1xuICAgICAgICBhY3F1aXJlOiAod2VpZ2h0T3JQcmlvcml0eSwgcHJpb3JpdHkpID0+IHtcbiAgICAgICAgICAgIGxldCB3ZWlnaHQ7XG4gICAgICAgICAgICBpZiAoaXNTZW1hcGhvcmUoc3luYykpIHtcbiAgICAgICAgICAgICAgICB3ZWlnaHQgPSB3ZWlnaHRPclByaW9yaXR5O1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgZWxzZSB7XG4gICAgICAgICAgICAgICAgd2VpZ2h0ID0gdW5kZWZpbmVkO1xuICAgICAgICAgICAgICAgIHByaW9yaXR5ID0gd2VpZ2h0T3JQcmlvcml0eTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIGlmICh3ZWlnaHQgIT09IHVuZGVmaW5lZCAmJiB3ZWlnaHQgPD0gMCkge1xuICAgICAgICAgICAgICAgIHRocm93IG5ldyBFcnJvcihgaW52YWxpZCB3ZWlnaHQgJHt3ZWlnaHR9OiBtdXN0IGJlIHBvc2l0aXZlYCk7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICByZXR1cm4gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4gX19hd2FpdGVyKHRoaXMsIHZvaWQgMCwgdm9pZCAwLCBmdW5jdGlvbiogKCkge1xuICAgICAgICAgICAgICAgIGxldCBpc1RpbWVvdXQgPSBmYWxzZTtcbiAgICAgICAgICAgICAgICBjb25zdCBoYW5kbGUgPSBzZXRUaW1lb3V0KCgpID0+IHtcbiAgICAgICAgICAgICAgICAgICAgaXNUaW1lb3V0ID0gdHJ1ZTtcbiAgICAgICAgICAgICAgICAgICAgcmVqZWN0KHRpbWVvdXRFcnJvcik7XG4gICAgICAgICAgICAgICAgfSwgdGltZW91dCk7XG4gICAgICAgICAgICAgICAgdHJ5IHtcbiAgICAgICAgICAgICAgICAgICAgY29uc3QgdGlja2V0ID0geWllbGQgKGlzU2VtYXBob3JlKHN5bmMpXG4gICAgICAgICAgICAgICAgICAgICAgICA/IHN5bmMuYWNxdWlyZSh3ZWlnaHQsIHByaW9yaXR5KVxuICAgICAgICAgICAgICAgICAgICAgICAgOiBzeW5jLmFjcXVpcmUocHJpb3JpdHkpKTtcbiAgICAgICAgICAgICAgICAgICAgaWYgKGlzVGltZW91dCkge1xuICAgICAgICAgICAgICAgICAgICAgICAgY29uc3QgcmVsZWFzZSA9IEFycmF5LmlzQXJyYXkodGlja2V0KSA/IHRpY2tldFsxXSA6IHRpY2tldDtcbiAgICAgICAgICAgICAgICAgICAgICAgIHJlbGVhc2UoKTtcbiAgICAgICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgICAgICBlbHNlIHtcbiAgICAgICAgICAgICAgICAgICAgICAgIGNsZWFyVGltZW91dChoYW5kbGUpO1xuICAgICAgICAgICAgICAgICAgICAgICAgcmVzb2x2ZSh0aWNrZXQpO1xuICAgICAgICAgICAgICAgICAgICB9XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgIGNhdGNoIChlKSB7XG4gICAgICAgICAgICAgICAgICAgIGlmICghaXNUaW1lb3V0KSB7XG4gICAgICAgICAgICAgICAgICAgICAgICBjbGVhclRpbWVvdXQoaGFuZGxlKTtcbiAgICAgICAgICAgICAgICAgICAgICAgIHJlamVjdChlKTtcbiAgICAgICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgIH1cbiAgICAgICAgICAgIH0pKTtcbiAgICAgICAgfSxcbiAgICAgICAgcnVuRXhjbHVzaXZlKGNhbGxiYWNrLCB3ZWlnaHQsIHByaW9yaXR5KSB7XG4gICAgICAgICAgICByZXR1cm4gX19hd2FpdGVyKHRoaXMsIHZvaWQgMCwgdm9pZCAwLCBmdW5jdGlvbiogKCkge1xuICAgICAgICAgICAgICAgIGxldCByZWxlYXNlID0gKCkgPT4gdW5kZWZpbmVkO1xuICAgICAgICAgICAgICAgIHRyeSB7XG4gICAgICAgICAgICAgICAgICAgIGNvbnN0IHRpY2tldCA9IHlpZWxkIHRoaXMuYWNxdWlyZSh3ZWlnaHQsIHByaW9yaXR5KTtcbiAgICAgICAgICAgICAgICAgICAgaWYgKEFycmF5LmlzQXJyYXkodGlja2V0KSkge1xuICAgICAgICAgICAgICAgICAgICAgICAgcmVsZWFzZSA9IHRpY2tldFsxXTtcbiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVybiB5aWVsZCBjYWxsYmFjayh0aWNrZXRbMF0pO1xuICAgICAgICAgICAgICAgICAgICB9XG4gICAgICAgICAgICAgICAgICAgIGVsc2Uge1xuICAgICAgICAgICAgICAgICAgICAgICAgcmVsZWFzZSA9IHRpY2tldDtcbiAgICAgICAgICAgICAgICAgICAgICAgIHJldHVybiB5aWVsZCBjYWxsYmFjaygpO1xuICAgICAgICAgICAgICAgICAgICB9XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgICAgIGZpbmFsbHkge1xuICAgICAgICAgICAgICAgICAgICByZWxlYXNlKCk7XG4gICAgICAgICAgICAgICAgfVxuICAgICAgICAgICAgfSk7XG4gICAgICAgIH0sXG4gICAgICAgIHJlbGVhc2Uod2VpZ2h0KSB7XG4gICAgICAgICAgICBzeW5jLnJlbGVhc2Uod2VpZ2h0KTtcbiAgICAgICAgfSxcbiAgICAgICAgY2FuY2VsKCkge1xuICAgICAgICAgICAgcmV0dXJuIHN5bmMuY2FuY2VsKCk7XG4gICAgICAgIH0sXG4gICAgICAgIHdhaXRGb3JVbmxvY2s6ICh3ZWlnaHRPclByaW9yaXR5LCBwcmlvcml0eSkgPT4ge1xuICAgICAgICAgICAgbGV0IHdlaWdodDtcbiAgICAgICAgICAgIGlmIChpc1NlbWFwaG9yZShzeW5jKSkge1xuICAgICAgICAgICAgICAgIHdlaWdodCA9IHdlaWdodE9yUHJpb3JpdHk7XG4gICAgICAgICAgICB9XG4gICAgICAgICAgICBlbHNlIHtcbiAgICAgICAgICAgICAgICB3ZWlnaHQgPSB1bmRlZmluZWQ7XG4gICAgICAgICAgICAgICAgcHJpb3JpdHkgPSB3ZWlnaHRPclByaW9yaXR5O1xuICAgICAgICAgICAgfVxuICAgICAgICAgICAgaWYgKHdlaWdodCAhPT0gdW5kZWZpbmVkICYmIHdlaWdodCA8PSAwKSB7XG4gICAgICAgICAgICAgICAgdGhyb3cgbmV3IEVycm9yKGBpbnZhbGlkIHdlaWdodCAke3dlaWdodH06IG11c3QgYmUgcG9zaXRpdmVgKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgICAgIHJldHVybiBuZXcgUHJvbWlzZSgocmVzb2x2ZSwgcmVqZWN0KSA9PiB7XG4gICAgICAgICAgICAgICAgY29uc3QgaGFuZGxlID0gc2V0VGltZW91dCgoKSA9PiByZWplY3QodGltZW91dEVycm9yKSwgdGltZW91dCk7XG4gICAgICAgICAgICAgICAgKGlzU2VtYXBob3JlKHN5bmMpXG4gICAgICAgICAgICAgICAgICAgID8gc3luYy53YWl0Rm9yVW5sb2NrKHdlaWdodCwgcHJpb3JpdHkpXG4gICAgICAgICAgICAgICAgICAgIDogc3luYy53YWl0Rm9yVW5sb2NrKHByaW9yaXR5KSkudGhlbigoKSA9PiB7XG4gICAgICAgICAgICAgICAgICAgIGNsZWFyVGltZW91dChoYW5kbGUpO1xuICAgICAgICAgICAgICAgICAgICByZXNvbHZlKCk7XG4gICAgICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICB9KTtcbiAgICAgICAgfSxcbiAgICAgICAgaXNMb2NrZWQ6ICgpID0+IHN5bmMuaXNMb2NrZWQoKSxcbiAgICAgICAgZ2V0VmFsdWU6ICgpID0+IHN5bmMuZ2V0VmFsdWUoKSxcbiAgICAgICAgc2V0VmFsdWU6ICh2YWx1ZSkgPT4gc3luYy5zZXRWYWx1ZSh2YWx1ZSksXG4gICAgfTtcbn1cbmZ1bmN0aW9uIGlzU2VtYXBob3JlKHN5bmMpIHtcbiAgICByZXR1cm4gc3luYy5nZXRWYWx1ZSAhPT0gdW5kZWZpbmVkO1xufVxuXG4vLyBlc2xpbnQtZGlzYWJsZS1uZXh0LWxpc25lIEB0eXBlc2NyaXB0LWVzbGludC9leHBsaWNpdC1tb2R1bGUtYm91bmRhcnktdHlwZXNcbmZ1bmN0aW9uIHRyeUFjcXVpcmUoc3luYywgYWxyZWFkeUFjcXVpcmVkRXJyb3IgPSBFX0FMUkVBRFlfTE9DS0VEKSB7XG4gICAgLy8gZXNsaW50LWRpc2FibGUtbmV4dC1saW5lIEB0eXBlc2NyaXB0LWVzbGludC9uby1leHBsaWNpdC1hbnlcbiAgICByZXR1cm4gd2l0aFRpbWVvdXQoc3luYywgMCwgYWxyZWFkeUFjcXVpcmVkRXJyb3IpO1xufVxuXG5leHBvcnQgeyBFX0FMUkVBRFlfTE9DS0VELCBFX0NBTkNFTEVELCBFX1RJTUVPVVQsIE11dGV4LCBTZW1hcGhvcmUsIHRyeUFjcXVpcmUsIHdpdGhUaW1lb3V0IH07XG4iLCAiY29uc3QgREJfVkVSU0lPTiA9IDM7XG5jb25zdCBzdG9yYWdlID0gYnJvd3Nlci5zdG9yYWdlLmxvY2FsO1xuZXhwb3J0IGNvbnN0IFJFQ09NTUVOREVEX1JFTEFZUyA9IFtcbiAgICBuZXcgVVJMKCd3c3M6Ly9yZWxheS5kYW11cy5pbycpLFxuICAgIG5ldyBVUkwoJ3dzczovL3JlbGF5LnNub3J0LnNvY2lhbCcpLFxuICAgIG5ldyBVUkwoJ3dzczovL25vcy5sb2wnKSxcbiAgICBuZXcgVVJMKCd3c3M6Ly9yZWxheS5wcmltYWwubmV0JyksXG4gICAgbmV3IFVSTCgnd3NzOi8vcmVsYXkubm9zdHIuYmFuZCcpLFxuICAgIG5ldyBVUkwoJ3dzczovL25vc3RyLm9yYW5nZXBpbGwuZGV2JyksXG5dO1xuLy8gcHJldHRpZXItaWdub3JlXG5leHBvcnQgY29uc3QgS0lORFMgPSBbXG4gICAgWzAsICdVc2VyIE1ldGFkYXRhJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzAxLm1kJ10sXG4gICAgWzEsICdTaG9ydCBUZXh0IE5vdGUnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTAubWQnXSxcbiAgICBbMiwgJ1JlY29tbWVuZCBSZWxheScsIG51bGxdLFxuICAgIFszLCAnRm9sbG93cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8wMi5tZCddLFxuICAgIFs0LCAnRW5jcnlwdGVkIERpcmVjdCBNZXNzYWdlcycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8wNC5tZCddLFxuICAgIFs1LCAnRXZlbnQgRGVsZXRpb24gUmVxdWVzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8wOS5tZCddLFxuICAgIFs2LCAnUmVwb3N0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzE4Lm1kJ10sXG4gICAgWzcsICdSZWFjdGlvbicsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yNS5tZCddLFxuICAgIFs4LCAnQmFkZ2UgQXdhcmQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTgubWQnXSxcbiAgICBbOSwgJ0NoYXQgTWVzc2FnZScsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci9DNy5tZCddLFxuICAgIFsxMCwgJ0dyb3VwIENoYXQgVGhyZWFkZWQgUmVwbHknLCBudWxsXSxcbiAgICBbMTEsICdUaHJlYWQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvN0QubWQnXSxcbiAgICBbMTIsICdHcm91cCBUaHJlYWQgUmVwbHknLCBudWxsXSxcbiAgICBbMTMsICdTZWFsJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzU5Lm1kJ10sXG4gICAgWzE0LCAnRGlyZWN0IE1lc3NhZ2UnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTcubWQnXSxcbiAgICBbMTUsICdGaWxlIE1lc3NhZ2UnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTcubWQnXSxcbiAgICBbMTYsICdHZW5lcmljIFJlcG9zdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8xOC5tZCddLFxuICAgIFsxNywgJ1JlYWN0aW9uIHRvIGEgd2Vic2l0ZScsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yNS5tZCddLFxuICAgIFsyMCwgJ1BpY3R1cmUnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNjgubWQnXSxcbiAgICBbMjEsICdWaWRlbyBFdmVudCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci83MS5tZCddLFxuICAgIFsyMiwgJ1Nob3J0LWZvcm0gUG9ydHJhaXQgVmlkZW8gRXZlbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNzEubWQnXSxcbiAgICBbMzAsICdpbnRlcm5hbCByZWZlcmVuY2UnLCAnaHR0cHM6Ly93aWtpc3RyLmNvbS9ua2JpcC0wMypmZDIwOGVlOGM4ZjI4Mzc4MGE5NTUyODk2ZTQ4MjNjYzlkYzZiZmQ0NDIwNjM4ODk1NzcxMDY5NDBmZDkyN2MxJ10sXG4gICAgWzMxLCAnZXh0ZXJuYWwgd2ViIHJlZmVyZW5jZScsICdodHRwczovL3dpa2lzdHIuY29tL25rYmlwLTAzKmZkMjA4ZWU4YzhmMjgzNzgwYTk1NTI4OTZlNDgyM2NjOWRjNmJmZDQ0MjA2Mzg4OTU3NzEwNjk0MGZkOTI3YzEnXSxcbiAgICBbMzIsICdoYXJkY29weSByZWZlcmVuY2UnLCAnaHR0cHM6Ly93aWtpc3RyLmNvbS9ua2JpcC0wMypmZDIwOGVlOGM4ZjI4Mzc4MGE5NTUyODk2ZTQ4MjNjYzlkYzZiZmQ0NDIwNjM4ODk1NzcxMDY5NDBmZDkyN2MxJ10sXG4gICAgWzMzLCAncHJvbXB0IHJlZmVyZW5jZScsICdodHRwczovL3dpa2lzdHIuY29tL25rYmlwLTAzKmZkMjA4ZWU4YzhmMjgzNzgwYTk1NTI4OTZlNDgyM2NjOWRjNmJmZDQ0MjA2Mzg4OTU3NzEwNjk0MGZkOTI3YzEnXSxcbiAgICBbNDAsICdDaGFubmVsIENyZWF0aW9uJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzI4Lm1kJ10sXG4gICAgWzQxLCAnQ2hhbm5lbCBNZXRhZGF0YScsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yOC5tZCddLFxuICAgIFs0MiwgJ0NoYW5uZWwgTWVzc2FnZScsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yOC5tZCddLFxuICAgIFs0MywgJ0NoYW5uZWwgSGlkZSBNZXNzYWdlJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzI4Lm1kJ10sXG4gICAgWzQ0LCAnQ2hhbm5lbCBNdXRlIFVzZXInLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMjgubWQnXSxcbiAgICBbNjIsICdSZXF1ZXN0IHRvIFZhbmlzaCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82Mi5tZCddLFxuICAgIFs2NCwgJ0NoZXNzIChQR04pJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzY0Lm1kJ10sXG4gICAgWzgxOCwgJ01lcmdlIFJlcXVlc3RzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzU0Lm1kJ10sXG4gICAgWzEwMTgsICdQb2xsIFJlc3BvbnNlJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzg4Lm1kJ10sXG4gICAgWzEwMjEsICdCaWQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTUubWQnXSxcbiAgICBbMTAyMiwgJ0JpZCBjb25maXJtYXRpb24nLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTUubWQnXSxcbiAgICBbMTA0MCwgJ09wZW5UaW1lc3RhbXBzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzAzLm1kJ10sXG4gICAgWzEwNTksICdHaWZ0IFdyYXAnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTkubWQnXSxcbiAgICBbMTA2MywgJ0ZpbGUgTWV0YWRhdGEnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvOTQubWQnXSxcbiAgICBbMTA2OCwgJ1BvbGwnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvODgubWQnXSxcbiAgICBbMTExMSwgJ0NvbW1lbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMjIubWQnXSxcbiAgICBbMTMxMSwgJ0xpdmUgQ2hhdCBNZXNzYWdlJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUzLm1kJ10sXG4gICAgWzEzMzcsICdDb2RlIFNuaXBwZXQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvQzAubWQnXSxcbiAgICBbMTYxNywgJ1BhdGNoZXMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMzQubWQnXSxcbiAgICBbMTYyMSwgJ0lzc3VlcycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8zNC5tZCddLFxuICAgIFsxNjIyLCAnR2l0IFJlcGxpZXMgKGRlcHJlY2F0ZWQpJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzM0Lm1kJ10sXG4gICAgWycxNjMwLTE2MzMnLCAnU3RhdHVzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzM0Lm1kJ10sXG4gICAgWzE5NzEsICdQcm9ibGVtIFRyYWNrZXInLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3Ryb2NrZXQvTklQUy9ibG9iL21haW4vUHJvYmxlbXMubWQnXSxcbiAgICBbMTk4NCwgJ1JlcG9ydGluZycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81Ni5tZCddLFxuICAgIFsxOTg1LCAnTGFiZWwnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMzIubWQnXSxcbiAgICBbMTk4NiwgJ1JlbGF5IHJldmlld3MnLCBudWxsXSxcbiAgICBbMTk4NywgJ0FJIEVtYmVkZGluZ3MgLyBWZWN0b3IgbGlzdHMnLCAnaHR0cHM6Ly93aWtpc3RyLmNvbS9ua2JpcC0wMipmZDIwOGVlOGM4ZjI4Mzc4MGE5NTUyODk2ZTQ4MjNjYzlkYzZiZmQ0NDIwNjM4ODk1NzcxMDY5NDBmZDkyN2MxJ10sXG4gICAgWzIwMDMsICdUb3JyZW50JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzM1Lm1kJ10sXG4gICAgWzIwMDQsICdUb3JyZW50IENvbW1lbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMzUubWQnXSxcbiAgICBbMjAyMiwgJ0NvaW5qb2luIFBvb2wnLCAnaHR0cHM6Ly9naXRsYWIuY29tLzE0NDAwMDBieXRlcy9qb2luc3RyLy0vYmxvYi9tYWluL05JUC5tZCddLFxuICAgIFs0NTUwLCAnQ29tbXVuaXR5IFBvc3QgQXBwcm92YWwnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNzIubWQnXSxcbiAgICBbJzUwMDAtNTk5OScsICdKb2IgUmVxdWVzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci85MC5tZCddLFxuICAgIFsnNjAwMC02OTk5JywgJ0pvYiBSZXN1bHQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvOTAubWQnXSxcbiAgICBbNzAwMCwgJ0pvYiBGZWVkYmFjaycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci85MC5tZCddLFxuICAgIFs3Mzc0LCAnUmVzZXJ2ZWQgQ2FzaHUgV2FsbGV0IFRva2VucycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82MC5tZCddLFxuICAgIFs3Mzc1LCAnQ2FzaHUgV2FsbGV0IFRva2VucycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82MC5tZCddLFxuICAgIFs3Mzc2LCAnQ2FzaHUgV2FsbGV0IEhpc3RvcnknLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNjAubWQnXSxcbiAgICBbJzkwMDAtOTAzMCcsICdHcm91cCBDb250cm9sIEV2ZW50cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yOS5tZCddLFxuICAgIFs5MDQxLCAnWmFwIEdvYWwnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNzUubWQnXSxcbiAgICBbOTMyMSwgJ051dHphcCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82MS5tZCddLFxuICAgIFs5NDY3LCAnVGlkYWwgbG9naW4nLCAnaHR0cHM6Ly93aWtpc3RyLmNvbS90aWRhbC1ub3N0ciddLFxuICAgIFs5NzM0LCAnWmFwIFJlcXVlc3QnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTcubWQnXSxcbiAgICBbOTczNSwgJ1phcCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81Ny5tZCddLFxuICAgIFs5ODAyLCAnSGlnaGxpZ2h0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci84NC5tZCddLFxuICAgIFsxMDAwMCwgJ011dGUgbGlzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFsxMDAwMSwgJ1BpbiBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDAyLCAnUmVsYXkgTGlzdCBNZXRhZGF0YScsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82NS5tZCddLFxuICAgIFsxMDAwMywgJ0Jvb2ttYXJrIGxpc3QnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMTAwMDQsICdDb21tdW5pdGllcyBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDA1LCAnUHVibGljIGNoYXRzIGxpc3QnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMTAwMDYsICdCbG9ja2VkIHJlbGF5cyBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDA3LCAnU2VhcmNoIHJlbGF5cyBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDA5LCAnVXNlciBncm91cHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMTAwMTIsICdGYXZvcml0ZSByZWxheXMgbGlzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFsxMDAxMywgJ1ByaXZhdGUgZXZlbnQgcmVsYXkgbGlzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8zNy5tZCddLFxuICAgIFsxMDAxNSwgJ0ludGVyZXN0cyBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDE5LCAnTnV0emFwIE1pbnQgUmVjb21tZW5kYXRpb24nLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNjEubWQnXSxcbiAgICBbMTAwMjAsICdNZWRpYSBmb2xsb3dzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDMwLCAnVXNlciBlbW9qaSBsaXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzEwMDUwLCAnUmVsYXkgbGlzdCB0byByZWNlaXZlIERNcycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFsxMDA2MywgJ1VzZXIgc2VydmVyIGxpc3QnLCAnaHR0cHM6Ly9naXRodWIuY29tL2h6cmQxNDkvYmxvc3NvbSddLFxuICAgIFsxMDA5NiwgJ0ZpbGUgc3RvcmFnZSBzZXJ2ZXIgbGlzdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci85Ni5tZCddLFxuICAgIFsxMDE2NiwgJ1JlbGF5IE1vbml0b3IgQW5ub3VuY2VtZW50JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzY2Lm1kJ10sXG4gICAgWzEzMTk0LCAnV2FsbGV0IEluZm8nLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNDcubWQnXSxcbiAgICBbMTczNzUsICdDYXNodSBXYWxsZXQgRXZlbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNjAubWQnXSxcbiAgICBbMjEwMDAsICdMaWdodG5pbmcgUHViIFJQQycsICdodHRwczovL2dpdGh1Yi5jb20vc2hvY2tuZXQvTGlnaHRuaW5nLlB1Yi9ibG9iL21hc3Rlci9wcm90by9hdXRvZ2VuZXJhdGVkL2NsaWVudC5tZCddLFxuICAgIFsyMjI0MiwgJ0NsaWVudCBBdXRoZW50aWNhdGlvbicsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci80Mi5tZCddLFxuICAgIFsyMzE5NCwgJ1dhbGxldCBSZXF1ZXN0JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzQ3Lm1kJ10sXG4gICAgWzIzMTk1LCAnV2FsbGV0IFJlc3BvbnNlJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzQ3Lm1kJ10sXG4gICAgWzI0MTMzLCAnTm9zdHIgQ29ubmVjdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci80Ni5tZCddLFxuICAgIFsyNDI0MiwgJ0Jsb2JzIHN0b3JlZCBvbiBtZWRpYXNlcnZlcnMnLCAnaHR0cHM6Ly9naXRodWIuY29tL2h6cmQxNDkvYmxvc3NvbSddLFxuICAgIFsyNzIzNSwgJ0hUVFAgQXV0aCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci85OC5tZCddLFxuICAgIFszMDAwMCwgJ0ZvbGxvdyBzZXRzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzMwMDAxLCAnR2VuZXJpYyBsaXN0cycsIG51bGxdLFxuICAgIFszMDAwMiwgJ1JlbGF5IHNldHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMzAwMDMsICdCb29rbWFyayBzZXRzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzMwMDA0LCAnQ3VyYXRpb24gc2V0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszMDAwNSwgJ1ZpZGVvIHNldHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMzAwMDcsICdLaW5kIG11dGUgc2V0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszMDAwOCwgJ1Byb2ZpbGUgQmFkZ2VzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzU4Lm1kJ10sXG4gICAgWzMwMDA5LCAnQmFkZ2UgRGVmaW5pdGlvbicsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81OC5tZCddLFxuICAgIFszMDAxNSwgJ0ludGVyZXN0IHNldHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTEubWQnXSxcbiAgICBbMzAwMTcsICdDcmVhdGUgb3IgdXBkYXRlIGEgc3RhbGwnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTUubWQnXSxcbiAgICBbMzAwMTgsICdDcmVhdGUgb3IgdXBkYXRlIGEgcHJvZHVjdCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8xNS5tZCddLFxuICAgIFszMDAxOSwgJ01hcmtldHBsYWNlIFVJL1VYJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzE1Lm1kJ10sXG4gICAgWzMwMDIwLCAnUHJvZHVjdCBzb2xkIGFzIGFuIGF1Y3Rpb24nLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMTUubWQnXSxcbiAgICBbMzAwMjMsICdMb25nLWZvcm0gQ29udGVudCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8yMy5tZCddLFxuICAgIFszMDAyNCwgJ0RyYWZ0IExvbmctZm9ybSBDb250ZW50JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzIzLm1kJ10sXG4gICAgWzMwMDMwLCAnRW1vamkgc2V0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszMDA0MCwgJ0N1cmF0ZWQgUHVibGljYXRpb24gSW5kZXgnLCAnaHR0cHM6Ly93aWtpc3RyLmNvbS9ua2JpcC0wMSpmZDIwOGVlOGM4ZjI4Mzc4MGE5NTUyODk2ZTQ4MjNjYzlkYzZiZmQ0NDIwNjM4ODk1NzcxMDY5NDBmZDkyN2MxJ10sXG4gICAgWzMwMDQxLCAnQ3VyYXRlZCBQdWJsaWNhdGlvbiBDb250ZW50JywgJ2h0dHBzOi8vd2lraXN0ci5jb20vbmtiaXAtMDEqZmQyMDhlZThjOGYyODM3ODBhOTU1Mjg5NmU0ODIzY2M5ZGM2YmZkNDQyMDYzODg5NTc3MTA2OTQwZmQ5MjdjMSddLFxuICAgIFszMDA2MywgJ1JlbGVhc2UgYXJ0aWZhY3Qgc2V0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszMDA3OCwgJ0FwcGxpY2F0aW9uLXNwZWNpZmljIERhdGEnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNzgubWQnXSxcbiAgICBbMzAxNjYsICdSZWxheSBEaXNjb3ZlcnknLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNjYubWQnXSxcbiAgICBbMzAyNjcsICdBcHAgY3VyYXRpb24gc2V0cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszMDMxMSwgJ0xpdmUgRXZlbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTMubWQnXSxcbiAgICBbMzAzMTUsICdVc2VyIFN0YXR1c2VzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzM4Lm1kJ10sXG4gICAgWzMwMzg4LCAnU2xpZGUgU2V0JywgJ2h0dHBzOi8vY29ybnljaGF0LmNvbS9kYXRhdHlwZXMja2luZDMwMzg4c2xpZGVzZXQnXSxcbiAgICBbMzA0MDIsICdDbGFzc2lmaWVkIExpc3RpbmcnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvOTkubWQnXSxcbiAgICBbMzA0MDMsICdEcmFmdCBDbGFzc2lmaWVkIExpc3RpbmcnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvOTkubWQnXSxcbiAgICBbMzA2MTcsICdSZXBvc2l0b3J5IGFubm91bmNlbWVudHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMzQubWQnXSxcbiAgICBbMzA2MTgsICdSZXBvc2l0b3J5IHN0YXRlIGFubm91bmNlbWVudHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMzQubWQnXSxcbiAgICBbMzA4MTgsICdXaWtpIGFydGljbGUnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTQubWQnXSxcbiAgICBbMzA4MTksICdSZWRpcmVjdHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTQubWQnXSxcbiAgICBbMzEyMzQsICdEcmFmdCBFdmVudCcsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci8zNy5tZCddLFxuICAgIFszMTM4OCwgJ0xpbmsgU2V0JywgJ2h0dHBzOi8vY29ybnljaGF0LmNvbS9kYXRhdHlwZXMja2luZDMxMzg4bGlua3NldCddLFxuICAgIFszMTg5MCwgJ0ZlZWQnLCAnaHR0cHM6Ly93aWtpZnJlZWRpYS54eXovY2lwLTAxLyddLFxuICAgIFszMTkyMiwgJ0RhdGUtQmFzZWQgQ2FsZW5kYXIgRXZlbnQnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTIubWQnXSxcbiAgICBbMzE5MjMsICdUaW1lLUJhc2VkIENhbGVuZGFyIEV2ZW50JywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUyLm1kJ10sXG4gICAgWzMxOTI0LCAnQ2FsZW5kYXInLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvNTIubWQnXSxcbiAgICBbMzE5MjUsICdDYWxlbmRhciBFdmVudCBSU1ZQJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUyLm1kJ10sXG4gICAgWzMxOTg5LCAnSGFuZGxlciByZWNvbW1lbmRhdGlvbicsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci84OS5tZCddLFxuICAgIFszMTk5MCwgJ0hhbmRsZXIgaW5mb3JtYXRpb24nLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvODkubWQnXSxcbiAgICBbMzIyNjcsICdTb2Z0d2FyZSBBcHBsaWNhdGlvbicsIG51bGxdLFxuICAgIFszNDU1MCwgJ0NvbW11bml0eSBEZWZpbml0aW9uJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzcyLm1kJ10sXG4gICAgWzM4MzgzLCAnUGVlci10by1wZWVyIE9yZGVyIGV2ZW50cycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci82OS5tZCddLFxuICAgIFsnMzkwMDAtOScsICdHcm91cCBtZXRhZGF0YSBldmVudHMnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvMjkubWQnXSxcbiAgICBbMzkwODksICdTdGFydGVyIHBhY2tzJywgJ2h0dHBzOi8vZ2l0aHViLmNvbS9ub3N0ci1wcm90b2NvbC9uaXBzL2Jsb2IvbWFzdGVyLzUxLm1kJ10sXG4gICAgWzM5MDkyLCAnTWVkaWEgc3RhcnRlciBwYWNrcycsICdodHRwczovL2dpdGh1Yi5jb20vbm9zdHItcHJvdG9jb2wvbmlwcy9ibG9iL21hc3Rlci81MS5tZCddLFxuICAgIFszOTcwMSwgJ1dlYiBib29rbWFya3MnLCAnaHR0cHM6Ly9naXRodWIuY29tL25vc3RyLXByb3RvY29sL25pcHMvYmxvYi9tYXN0ZXIvQjAubWQnXSxcbl07XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBpbml0aWFsaXplKCkge1xuICAgIGF3YWl0IGdldE9yU2V0RGVmYXVsdCgncHJvZmlsZUluZGV4JywgMCk7XG4gICAgYXdhaXQgZ2V0T3JTZXREZWZhdWx0KCdwcm9maWxlcycsIFthd2FpdCBnZW5lcmF0ZVByb2ZpbGUoKV0pO1xuICAgIGxldCB2ZXJzaW9uID0gKGF3YWl0IHN0b3JhZ2UuZ2V0KHsgdmVyc2lvbjogMCB9KSkudmVyc2lvbjtcbiAgICBjb25zb2xlLmxvZygnREIgdmVyc2lvbjogJywgdmVyc2lvbik7XG4gICAgd2hpbGUgKHZlcnNpb24gPCBEQl9WRVJTSU9OKSB7XG4gICAgICAgIHZlcnNpb24gPSBhd2FpdCBtaWdyYXRlKHZlcnNpb24sIERCX1ZFUlNJT04pO1xuICAgICAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHZlcnNpb24gfSk7XG4gICAgfVxufVxuXG5hc3luYyBmdW5jdGlvbiBtaWdyYXRlKHZlcnNpb24sIGdvYWwpIHtcbiAgICBpZiAodmVyc2lvbiA9PT0gMCkge1xuICAgICAgICBjb25zb2xlLmxvZygnTWlncmF0aW5nIHRvIHZlcnNpb24gMS4nKTtcbiAgICAgICAgbGV0IHByb2ZpbGVzID0gYXdhaXQgZ2V0UHJvZmlsZXMoKTtcbiAgICAgICAgcHJvZmlsZXMuZm9yRWFjaChwcm9maWxlID0+IChwcm9maWxlLmhvc3RzID0ge30pKTtcbiAgICAgICAgYXdhaXQgc3RvcmFnZS5zZXQoeyBwcm9maWxlcyB9KTtcbiAgICAgICAgcmV0dXJuIHZlcnNpb24gKyAxO1xuICAgIH1cblxuICAgIGlmICh2ZXJzaW9uID09PSAxKSB7XG4gICAgICAgIGNvbnNvbGUubG9nKCdtaWdyYXRpbmcgdG8gdmVyc2lvbiAyLicpO1xuICAgICAgICBsZXQgcHJvZmlsZXMgPSBhd2FpdCBnZXRQcm9maWxlcygpO1xuICAgICAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHByb2ZpbGVzIH0pO1xuICAgICAgICByZXR1cm4gdmVyc2lvbiArIDE7XG4gICAgfVxuXG4gICAgaWYgKHZlcnNpb24gPT09IDIpIHtcbiAgICAgICAgY29uc29sZS5sb2coJ01pZ3JhdGluZyB0byB2ZXJzaW9uIDMuJyk7XG4gICAgICAgIGxldCBwcm9maWxlcyA9IGF3YWl0IGdldFByb2ZpbGVzKCk7XG4gICAgICAgIHByb2ZpbGVzLmZvckVhY2gocHJvZmlsZSA9PiAocHJvZmlsZS5yZWxheVJlbWluZGVyID0gdHJ1ZSkpO1xuICAgICAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHByb2ZpbGVzIH0pO1xuICAgICAgICByZXR1cm4gdmVyc2lvbiArIDE7XG4gICAgfVxufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2V0UHJvZmlsZXMoKSB7XG4gICAgbGV0IHByb2ZpbGVzID0gYXdhaXQgc3RvcmFnZS5nZXQoeyBwcm9maWxlczogW10gfSk7XG4gICAgcmV0dXJuIHByb2ZpbGVzLnByb2ZpbGVzO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2V0UHJvZmlsZShpbmRleCkge1xuICAgIGxldCBwcm9maWxlcyA9IGF3YWl0IGdldFByb2ZpbGVzKCk7XG4gICAgcmV0dXJuIHByb2ZpbGVzW2luZGV4XTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdldFByb2ZpbGVOYW1lcygpIHtcbiAgICBsZXQgcHJvZmlsZXMgPSBhd2FpdCBnZXRQcm9maWxlcygpO1xuICAgIHJldHVybiBwcm9maWxlcy5tYXAocCA9PiBwLm5hbWUpO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2V0UHJvZmlsZUluZGV4KCkge1xuICAgIGNvbnN0IGluZGV4ID0gYXdhaXQgc3RvcmFnZS5nZXQoeyBwcm9maWxlSW5kZXg6IDAgfSk7XG4gICAgcmV0dXJuIGluZGV4LnByb2ZpbGVJbmRleDtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHNldFByb2ZpbGVJbmRleChwcm9maWxlSW5kZXgpIHtcbiAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHByb2ZpbGVJbmRleCB9KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGRlbGV0ZVByb2ZpbGUoaW5kZXgpIHtcbiAgICBsZXQgcHJvZmlsZXMgPSBhd2FpdCBnZXRQcm9maWxlcygpO1xuICAgIGxldCBwcm9maWxlSW5kZXggPSBhd2FpdCBnZXRQcm9maWxlSW5kZXgoKTtcbiAgICBwcm9maWxlcy5zcGxpY2UoaW5kZXgsIDEpO1xuICAgIGlmIChwcm9maWxlcy5sZW5ndGggPT0gMCkge1xuICAgICAgICBhd2FpdCBjbGVhckRhdGEoKTsgLy8gSWYgd2UgaGF2ZSBkZWxldGVkIGFsbCBvZiB0aGUgcHJvZmlsZXMsIGxldCdzIGp1c3Qgc3RhcnQgZnJlc2ggd2l0aCBhbGwgbmV3IGRhdGFcbiAgICAgICAgYXdhaXQgaW5pdGlhbGl6ZSgpO1xuICAgIH0gZWxzZSB7XG4gICAgICAgIC8vIElmIHRoZSBpbmRleCBkZWxldGVkIHdhcyB0aGUgYWN0aXZlIHByb2ZpbGUsIGNoYW5nZSB0aGUgYWN0aXZlIHByb2ZpbGUgdG8gdGhlIG5leHQgb25lXG4gICAgICAgIGxldCBuZXdJbmRleCA9XG4gICAgICAgICAgICBwcm9maWxlSW5kZXggPT09IGluZGV4ID8gTWF0aC5tYXgoaW5kZXggLSAxLCAwKSA6IHRoaXMucHJvZmlsZUluZGV4O1xuICAgICAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHByb2ZpbGVzLCBwcm9maWxlSW5kZXg6IG5ld0luZGV4IH0pO1xuICAgIH1cbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGNsZWFyRGF0YSgpIHtcbiAgICBsZXQgaWdub3JlSW5zdGFsbEhvb2sgPSBhd2FpdCBzdG9yYWdlLmdldCh7IGlnbm9yZUluc3RhbGxIb29rOiBmYWxzZSB9KTtcbiAgICBhd2FpdCBzdG9yYWdlLmNsZWFyKCk7XG4gICAgYXdhaXQgc3RvcmFnZS5zZXQoaWdub3JlSW5zdGFsbEhvb2spO1xufVxuXG5hc3luYyBmdW5jdGlvbiBnZW5lcmF0ZVByaXZhdGVLZXkoKSB7XG4gICAgcmV0dXJuIGF3YWl0IGJyb3dzZXIucnVudGltZS5zZW5kTWVzc2FnZSh7IGtpbmQ6ICdnZW5lcmF0ZVByaXZhdGVLZXknIH0pO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2VuZXJhdGVQcm9maWxlKG5hbWUgPSAnRGVmYXVsdCcpIHtcbiAgICByZXR1cm4ge1xuICAgICAgICBuYW1lLFxuICAgICAgICBwcml2S2V5OiBhd2FpdCBnZW5lcmF0ZVByaXZhdGVLZXkoKSxcbiAgICAgICAgaG9zdHM6IHt9LFxuICAgICAgICByZWxheXM6IFtdLFxuICAgICAgICByZWxheVJlbWluZGVyOiB0cnVlLFxuICAgIH07XG59XG5cbmFzeW5jIGZ1bmN0aW9uIGdldE9yU2V0RGVmYXVsdChrZXksIGRlZikge1xuICAgIGxldCB2YWwgPSAoYXdhaXQgc3RvcmFnZS5nZXQoa2V5KSlba2V5XTtcbiAgICBpZiAodmFsID09IG51bGwgfHwgdmFsID09IHVuZGVmaW5lZCkge1xuICAgICAgICBhd2FpdCBzdG9yYWdlLnNldCh7IFtrZXldOiBkZWYgfSk7XG4gICAgICAgIHJldHVybiBkZWY7XG4gICAgfVxuXG4gICAgcmV0dXJuIHZhbDtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHNhdmVQcm9maWxlTmFtZShpbmRleCwgcHJvZmlsZU5hbWUpIHtcbiAgICBsZXQgcHJvZmlsZXMgPSBhd2FpdCBnZXRQcm9maWxlcygpO1xuICAgIHByb2ZpbGVzW2luZGV4XS5uYW1lID0gcHJvZmlsZU5hbWU7XG4gICAgYXdhaXQgc3RvcmFnZS5zZXQoeyBwcm9maWxlcyB9KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHNhdmVQcml2YXRlS2V5KGluZGV4LCBwcml2YXRlS2V5KSB7XG4gICAgYXdhaXQgYnJvd3Nlci5ydW50aW1lLnNlbmRNZXNzYWdlKHtcbiAgICAgICAga2luZDogJ3NhdmVQcml2YXRlS2V5JyxcbiAgICAgICAgcGF5bG9hZDogW2luZGV4LCBwcml2YXRlS2V5XSxcbiAgICB9KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIG5ld1Byb2ZpbGUoKSB7XG4gICAgbGV0IHByb2ZpbGVzID0gYXdhaXQgZ2V0UHJvZmlsZXMoKTtcbiAgICBjb25zdCBuZXdQcm9maWxlID0gYXdhaXQgZ2VuZXJhdGVQcm9maWxlKCdOZXcgUHJvZmlsZScpO1xuICAgIHByb2ZpbGVzLnB1c2gobmV3UHJvZmlsZSk7XG4gICAgYXdhaXQgc3RvcmFnZS5zZXQoeyBwcm9maWxlcyB9KTtcbiAgICByZXR1cm4gcHJvZmlsZXMubGVuZ3RoIC0gMTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdldFJlbGF5cyhwcm9maWxlSW5kZXgpIHtcbiAgICBsZXQgcHJvZmlsZSA9IGF3YWl0IGdldFByb2ZpbGUocHJvZmlsZUluZGV4KTtcbiAgICByZXR1cm4gcHJvZmlsZS5yZWxheXMgfHwgW107XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBzYXZlUmVsYXlzKHByb2ZpbGVJbmRleCwgcmVsYXlzKSB7XG4gICAgLy8gSGF2aW5nIGFuIEFscGluZSBwcm94eSBvYmplY3QgYXMgYSBzdWItb2JqZWN0IGRvZXMgbm90IHNlcmlhbGl6ZSBjb3JyZWN0bHkgaW4gc3RvcmFnZSxcbiAgICAvLyBzbyB3ZSBhcmUgcHJlLXNlcmlhbGl6aW5nIGhlcmUgYmVmb3JlIGFzc2lnbmluZyBpdCB0byB0aGUgcHJvZmlsZSwgc28gdGhlIHByb3h5XG4gICAgLy8gb2JqIGRvZXNuJ3QgYnVnIG91dC5cbiAgICBsZXQgZml4ZWRSZWxheXMgPSBKU09OLnBhcnNlKEpTT04uc3RyaW5naWZ5KHJlbGF5cykpO1xuICAgIGxldCBwcm9maWxlcyA9IGF3YWl0IGdldFByb2ZpbGVzKCk7XG4gICAgbGV0IHByb2ZpbGUgPSBwcm9maWxlc1twcm9maWxlSW5kZXhdO1xuICAgIHByb2ZpbGUucmVsYXlzID0gZml4ZWRSZWxheXM7XG4gICAgYXdhaXQgc3RvcmFnZS5zZXQoeyBwcm9maWxlcyB9KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIGdldChpdGVtKSB7XG4gICAgcmV0dXJuIChhd2FpdCBzdG9yYWdlLmdldChpdGVtKSlbaXRlbV07XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZXRQZXJtaXNzaW9ucyhpbmRleCA9IG51bGwpIHtcbiAgICBpZiAoIWluZGV4KSB7XG4gICAgICAgIGluZGV4ID0gYXdhaXQgZ2V0UHJvZmlsZUluZGV4KCk7XG4gICAgfVxuICAgIGxldCBwcm9maWxlID0gYXdhaXQgZ2V0UHJvZmlsZShpbmRleCk7XG4gICAgbGV0IGhvc3RzID0gYXdhaXQgcHJvZmlsZS5ob3N0cztcbiAgICByZXR1cm4gaG9zdHM7XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZXRQZXJtaXNzaW9uKGhvc3QsIGFjdGlvbikge1xuICAgIGxldCBpbmRleCA9IGF3YWl0IGdldFByb2ZpbGVJbmRleCgpO1xuICAgIGxldCBwcm9maWxlID0gYXdhaXQgZ2V0UHJvZmlsZShpbmRleCk7XG4gICAgY29uc29sZS5sb2coaG9zdCwgYWN0aW9uKTtcbiAgICBjb25zb2xlLmxvZygncHJvZmlsZTogJywgcHJvZmlsZSk7XG4gICAgcmV0dXJuIHByb2ZpbGUuaG9zdHM/Lltob3N0XT8uW2FjdGlvbl0gfHwgJ2Fzayc7XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBzZXRQZXJtaXNzaW9uKGhvc3QsIGFjdGlvbiwgcGVybSwgaW5kZXggPSBudWxsKSB7XG4gICAgbGV0IHByb2ZpbGVzID0gYXdhaXQgZ2V0UHJvZmlsZXMoKTtcbiAgICBpZiAoIWluZGV4KSB7XG4gICAgICAgIGluZGV4ID0gYXdhaXQgZ2V0UHJvZmlsZUluZGV4KCk7XG4gICAgfVxuICAgIGxldCBwcm9maWxlID0gcHJvZmlsZXNbaW5kZXhdO1xuICAgIGxldCBuZXdQZXJtcyA9IHByb2ZpbGUuaG9zdHNbaG9zdF0gfHwge307XG4gICAgbmV3UGVybXMgPSB7IC4uLm5ld1Blcm1zLCBbYWN0aW9uXTogcGVybSB9O1xuICAgIHByb2ZpbGUuaG9zdHNbaG9zdF0gPSBuZXdQZXJtcztcbiAgICBwcm9maWxlc1tpbmRleF0gPSBwcm9maWxlO1xuICAgIGF3YWl0IHN0b3JhZ2Uuc2V0KHsgcHJvZmlsZXMgfSk7XG59XG5cbmV4cG9ydCBmdW5jdGlvbiBodW1hblBlcm1pc3Npb24ocCkge1xuICAgIC8vIEhhbmRsZSBzcGVjaWFsIGNhc2Ugd2hlcmUgZXZlbnQgc2lnbmluZyBpbmNsdWRlcyBhIGtpbmQgbnVtYmVyXG4gICAgaWYgKHAuc3RhcnRzV2l0aCgnc2lnbkV2ZW50OicpKSB7XG4gICAgICAgIGxldCBbZSwgbl0gPSBwLnNwbGl0KCc6Jyk7XG4gICAgICAgIG4gPSBwYXJzZUludChuKTtcbiAgICAgICAgbGV0IG5uYW1lID0gS0lORFMuZmluZChrID0+IGtbMF0gPT09IG4pPy5bMV0gfHwgYFVua25vd24gKEtpbmQgJHtufSlgO1xuICAgICAgICByZXR1cm4gYFNpZ24gZXZlbnQ6ICR7bm5hbWV9YDtcbiAgICB9XG5cbiAgICBzd2l0Y2ggKHApIHtcbiAgICAgICAgY2FzZSAnZ2V0UHViS2V5JzpcbiAgICAgICAgICAgIHJldHVybiAnUmVhZCBwdWJsaWMga2V5JztcbiAgICAgICAgY2FzZSAnc2lnbkV2ZW50JzpcbiAgICAgICAgICAgIHJldHVybiAnU2lnbiBldmVudCc7XG4gICAgICAgIGNhc2UgJ2dldFJlbGF5cyc6XG4gICAgICAgICAgICByZXR1cm4gJ1JlYWQgcmVsYXkgbGlzdCc7XG4gICAgICAgIGNhc2UgJ25pcDA0LmVuY3J5cHQnOlxuICAgICAgICAgICAgcmV0dXJuICdFbmNyeXB0IHByaXZhdGUgbWVzc2FnZSAoTklQLTA0KSc7XG4gICAgICAgIGNhc2UgJ25pcDA0LmRlY3J5cHQnOlxuICAgICAgICAgICAgcmV0dXJuICdEZWNyeXB0IHByaXZhdGUgbWVzc2FnZSAoTklQLTA0KSc7XG4gICAgICAgIGNhc2UgJ25pcDQ0LmVuY3J5cHQnOlxuICAgICAgICAgICAgcmV0dXJuICdFbmNyeXB0IHByaXZhdGUgbWVzc2FnZSAoTklQLTQ0KSc7XG4gICAgICAgIGNhc2UgJ25pcDQ0LmRlY3J5cHQnOlxuICAgICAgICAgICAgcmV0dXJuICdEZWNyeXB0IHByaXZhdGUgbWVzc2FnZSAoTklQLTQ0KSc7XG4gICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgICByZXR1cm4gJ1Vua25vd24nO1xuICAgIH1cbn1cblxuZXhwb3J0IGZ1bmN0aW9uIHZhbGlkYXRlS2V5KGtleSkge1xuICAgIGNvbnN0IGhleE1hdGNoID0gL15bXFxkYS1mXXs2NH0kL2kudGVzdChrZXkpO1xuICAgIGNvbnN0IGIzMk1hdGNoID0gL15uc2VjMVtxcHpyeTl4OGdmMnR2ZHcwczNqbjU0a2hjZTZtdWE3bF17NTh9JC8udGVzdChrZXkpO1xuXG4gICAgcmV0dXJuIGhleE1hdGNoIHx8IGIzMk1hdGNoO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZmVhdHVyZShuYW1lKSB7XG4gICAgbGV0IGZuYW1lID0gYGZlYXR1cmU6JHtuYW1lfWA7XG4gICAgbGV0IGYgPSBhd2FpdCBicm93c2VyLnN0b3JhZ2UubG9jYWwuZ2V0KHsgW2ZuYW1lXTogZmFsc2UgfSk7XG4gICAgcmV0dXJuIGZbZm5hbWVdO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gcmVsYXlSZW1pbmRlcigpIHtcbiAgICBsZXQgaW5kZXggPSBhd2FpdCBnZXRQcm9maWxlSW5kZXgoKTtcbiAgICBsZXQgcHJvZmlsZSA9IGF3YWl0IGdldFByb2ZpbGUoaW5kZXgpO1xuICAgIHJldHVybiBwcm9maWxlLnJlbGF5UmVtaW5kZXI7XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiB0b2dnbGVSZWxheVJlbWluZGVyKCkge1xuICAgIGxldCBpbmRleCA9IGF3YWl0IGdldFByb2ZpbGVJbmRleCgpO1xuICAgIGxldCBwcm9maWxlcyA9IGF3YWl0IGdldFByb2ZpbGVzKCk7XG4gICAgcHJvZmlsZXNbaW5kZXhdLnJlbGF5UmVtaW5kZXIgPSBmYWxzZTtcbiAgICBhd2FpdCBzdG9yYWdlLnNldCh7IHByb2ZpbGVzIH0pO1xufVxuXG5leHBvcnQgYXN5bmMgZnVuY3Rpb24gZ2V0TnB1YigpIHtcbiAgICBsZXQgaW5kZXggPSBhd2FpdCBnZXRQcm9maWxlSW5kZXgoKTtcbiAgICByZXR1cm4gYXdhaXQgYnJvd3Nlci5ydW50aW1lLnNlbmRNZXNzYWdlKHtcbiAgICAgICAga2luZDogJ2dldE5wdWInLFxuICAgICAgICBwYXlsb2FkOiBpbmRleCxcbiAgICB9KTtcbn1cbiIsICJjb25zdCBpbnN0YW5jZU9mQW55ID0gKG9iamVjdCwgY29uc3RydWN0b3JzKSA9PiBjb25zdHJ1Y3RvcnMuc29tZSgoYykgPT4gb2JqZWN0IGluc3RhbmNlb2YgYyk7XG5cbmxldCBpZGJQcm94eWFibGVUeXBlcztcbmxldCBjdXJzb3JBZHZhbmNlTWV0aG9kcztcbi8vIFRoaXMgaXMgYSBmdW5jdGlvbiB0byBwcmV2ZW50IGl0IHRocm93aW5nIHVwIGluIG5vZGUgZW52aXJvbm1lbnRzLlxuZnVuY3Rpb24gZ2V0SWRiUHJveHlhYmxlVHlwZXMoKSB7XG4gICAgcmV0dXJuIChpZGJQcm94eWFibGVUeXBlcyB8fFxuICAgICAgICAoaWRiUHJveHlhYmxlVHlwZXMgPSBbXG4gICAgICAgICAgICBJREJEYXRhYmFzZSxcbiAgICAgICAgICAgIElEQk9iamVjdFN0b3JlLFxuICAgICAgICAgICAgSURCSW5kZXgsXG4gICAgICAgICAgICBJREJDdXJzb3IsXG4gICAgICAgICAgICBJREJUcmFuc2FjdGlvbixcbiAgICAgICAgXSkpO1xufVxuLy8gVGhpcyBpcyBhIGZ1bmN0aW9uIHRvIHByZXZlbnQgaXQgdGhyb3dpbmcgdXAgaW4gbm9kZSBlbnZpcm9ubWVudHMuXG5mdW5jdGlvbiBnZXRDdXJzb3JBZHZhbmNlTWV0aG9kcygpIHtcbiAgICByZXR1cm4gKGN1cnNvckFkdmFuY2VNZXRob2RzIHx8XG4gICAgICAgIChjdXJzb3JBZHZhbmNlTWV0aG9kcyA9IFtcbiAgICAgICAgICAgIElEQkN1cnNvci5wcm90b3R5cGUuYWR2YW5jZSxcbiAgICAgICAgICAgIElEQkN1cnNvci5wcm90b3R5cGUuY29udGludWUsXG4gICAgICAgICAgICBJREJDdXJzb3IucHJvdG90eXBlLmNvbnRpbnVlUHJpbWFyeUtleSxcbiAgICAgICAgXSkpO1xufVxuY29uc3QgdHJhbnNhY3Rpb25Eb25lTWFwID0gbmV3IFdlYWtNYXAoKTtcbmNvbnN0IHRyYW5zZm9ybUNhY2hlID0gbmV3IFdlYWtNYXAoKTtcbmNvbnN0IHJldmVyc2VUcmFuc2Zvcm1DYWNoZSA9IG5ldyBXZWFrTWFwKCk7XG5mdW5jdGlvbiBwcm9taXNpZnlSZXF1ZXN0KHJlcXVlc3QpIHtcbiAgICBjb25zdCBwcm9taXNlID0gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4ge1xuICAgICAgICBjb25zdCB1bmxpc3RlbiA9ICgpID0+IHtcbiAgICAgICAgICAgIHJlcXVlc3QucmVtb3ZlRXZlbnRMaXN0ZW5lcignc3VjY2VzcycsIHN1Y2Nlc3MpO1xuICAgICAgICAgICAgcmVxdWVzdC5yZW1vdmVFdmVudExpc3RlbmVyKCdlcnJvcicsIGVycm9yKTtcbiAgICAgICAgfTtcbiAgICAgICAgY29uc3Qgc3VjY2VzcyA9ICgpID0+IHtcbiAgICAgICAgICAgIHJlc29sdmUod3JhcChyZXF1ZXN0LnJlc3VsdCkpO1xuICAgICAgICAgICAgdW5saXN0ZW4oKTtcbiAgICAgICAgfTtcbiAgICAgICAgY29uc3QgZXJyb3IgPSAoKSA9PiB7XG4gICAgICAgICAgICByZWplY3QocmVxdWVzdC5lcnJvcik7XG4gICAgICAgICAgICB1bmxpc3RlbigpO1xuICAgICAgICB9O1xuICAgICAgICByZXF1ZXN0LmFkZEV2ZW50TGlzdGVuZXIoJ3N1Y2Nlc3MnLCBzdWNjZXNzKTtcbiAgICAgICAgcmVxdWVzdC5hZGRFdmVudExpc3RlbmVyKCdlcnJvcicsIGVycm9yKTtcbiAgICB9KTtcbiAgICAvLyBUaGlzIG1hcHBpbmcgZXhpc3RzIGluIHJldmVyc2VUcmFuc2Zvcm1DYWNoZSBidXQgZG9lc24ndCBleGlzdCBpbiB0cmFuc2Zvcm1DYWNoZS4gVGhpc1xuICAgIC8vIGlzIGJlY2F1c2Ugd2UgY3JlYXRlIG1hbnkgcHJvbWlzZXMgZnJvbSBhIHNpbmdsZSBJREJSZXF1ZXN0LlxuICAgIHJldmVyc2VUcmFuc2Zvcm1DYWNoZS5zZXQocHJvbWlzZSwgcmVxdWVzdCk7XG4gICAgcmV0dXJuIHByb21pc2U7XG59XG5mdW5jdGlvbiBjYWNoZURvbmVQcm9taXNlRm9yVHJhbnNhY3Rpb24odHgpIHtcbiAgICAvLyBFYXJseSBiYWlsIGlmIHdlJ3ZlIGFscmVhZHkgY3JlYXRlZCBhIGRvbmUgcHJvbWlzZSBmb3IgdGhpcyB0cmFuc2FjdGlvbi5cbiAgICBpZiAodHJhbnNhY3Rpb25Eb25lTWFwLmhhcyh0eCkpXG4gICAgICAgIHJldHVybjtcbiAgICBjb25zdCBkb25lID0gbmV3IFByb21pc2UoKHJlc29sdmUsIHJlamVjdCkgPT4ge1xuICAgICAgICBjb25zdCB1bmxpc3RlbiA9ICgpID0+IHtcbiAgICAgICAgICAgIHR4LnJlbW92ZUV2ZW50TGlzdGVuZXIoJ2NvbXBsZXRlJywgY29tcGxldGUpO1xuICAgICAgICAgICAgdHgucmVtb3ZlRXZlbnRMaXN0ZW5lcignZXJyb3InLCBlcnJvcik7XG4gICAgICAgICAgICB0eC5yZW1vdmVFdmVudExpc3RlbmVyKCdhYm9ydCcsIGVycm9yKTtcbiAgICAgICAgfTtcbiAgICAgICAgY29uc3QgY29tcGxldGUgPSAoKSA9PiB7XG4gICAgICAgICAgICByZXNvbHZlKCk7XG4gICAgICAgICAgICB1bmxpc3RlbigpO1xuICAgICAgICB9O1xuICAgICAgICBjb25zdCBlcnJvciA9ICgpID0+IHtcbiAgICAgICAgICAgIHJlamVjdCh0eC5lcnJvciB8fCBuZXcgRE9NRXhjZXB0aW9uKCdBYm9ydEVycm9yJywgJ0Fib3J0RXJyb3InKSk7XG4gICAgICAgICAgICB1bmxpc3RlbigpO1xuICAgICAgICB9O1xuICAgICAgICB0eC5hZGRFdmVudExpc3RlbmVyKCdjb21wbGV0ZScsIGNvbXBsZXRlKTtcbiAgICAgICAgdHguYWRkRXZlbnRMaXN0ZW5lcignZXJyb3InLCBlcnJvcik7XG4gICAgICAgIHR4LmFkZEV2ZW50TGlzdGVuZXIoJ2Fib3J0JywgZXJyb3IpO1xuICAgIH0pO1xuICAgIC8vIENhY2hlIGl0IGZvciBsYXRlciByZXRyaWV2YWwuXG4gICAgdHJhbnNhY3Rpb25Eb25lTWFwLnNldCh0eCwgZG9uZSk7XG59XG5sZXQgaWRiUHJveHlUcmFwcyA9IHtcbiAgICBnZXQodGFyZ2V0LCBwcm9wLCByZWNlaXZlcikge1xuICAgICAgICBpZiAodGFyZ2V0IGluc3RhbmNlb2YgSURCVHJhbnNhY3Rpb24pIHtcbiAgICAgICAgICAgIC8vIFNwZWNpYWwgaGFuZGxpbmcgZm9yIHRyYW5zYWN0aW9uLmRvbmUuXG4gICAgICAgICAgICBpZiAocHJvcCA9PT0gJ2RvbmUnKVxuICAgICAgICAgICAgICAgIHJldHVybiB0cmFuc2FjdGlvbkRvbmVNYXAuZ2V0KHRhcmdldCk7XG4gICAgICAgICAgICAvLyBNYWtlIHR4LnN0b3JlIHJldHVybiB0aGUgb25seSBzdG9yZSBpbiB0aGUgdHJhbnNhY3Rpb24sIG9yIHVuZGVmaW5lZCBpZiB0aGVyZSBhcmUgbWFueS5cbiAgICAgICAgICAgIGlmIChwcm9wID09PSAnc3RvcmUnKSB7XG4gICAgICAgICAgICAgICAgcmV0dXJuIHJlY2VpdmVyLm9iamVjdFN0b3JlTmFtZXNbMV1cbiAgICAgICAgICAgICAgICAgICAgPyB1bmRlZmluZWRcbiAgICAgICAgICAgICAgICAgICAgOiByZWNlaXZlci5vYmplY3RTdG9yZShyZWNlaXZlci5vYmplY3RTdG9yZU5hbWVzWzBdKTtcbiAgICAgICAgICAgIH1cbiAgICAgICAgfVxuICAgICAgICAvLyBFbHNlIHRyYW5zZm9ybSB3aGF0ZXZlciB3ZSBnZXQgYmFjay5cbiAgICAgICAgcmV0dXJuIHdyYXAodGFyZ2V0W3Byb3BdKTtcbiAgICB9LFxuICAgIHNldCh0YXJnZXQsIHByb3AsIHZhbHVlKSB7XG4gICAgICAgIHRhcmdldFtwcm9wXSA9IHZhbHVlO1xuICAgICAgICByZXR1cm4gdHJ1ZTtcbiAgICB9LFxuICAgIGhhcyh0YXJnZXQsIHByb3ApIHtcbiAgICAgICAgaWYgKHRhcmdldCBpbnN0YW5jZW9mIElEQlRyYW5zYWN0aW9uICYmXG4gICAgICAgICAgICAocHJvcCA9PT0gJ2RvbmUnIHx8IHByb3AgPT09ICdzdG9yZScpKSB7XG4gICAgICAgICAgICByZXR1cm4gdHJ1ZTtcbiAgICAgICAgfVxuICAgICAgICByZXR1cm4gcHJvcCBpbiB0YXJnZXQ7XG4gICAgfSxcbn07XG5mdW5jdGlvbiByZXBsYWNlVHJhcHMoY2FsbGJhY2spIHtcbiAgICBpZGJQcm94eVRyYXBzID0gY2FsbGJhY2soaWRiUHJveHlUcmFwcyk7XG59XG5mdW5jdGlvbiB3cmFwRnVuY3Rpb24oZnVuYykge1xuICAgIC8vIER1ZSB0byBleHBlY3RlZCBvYmplY3QgZXF1YWxpdHkgKHdoaWNoIGlzIGVuZm9yY2VkIGJ5IHRoZSBjYWNoaW5nIGluIGB3cmFwYCksIHdlXG4gICAgLy8gb25seSBjcmVhdGUgb25lIG5ldyBmdW5jIHBlciBmdW5jLlxuICAgIC8vIEN1cnNvciBtZXRob2RzIGFyZSBzcGVjaWFsLCBhcyB0aGUgYmVoYXZpb3VyIGlzIGEgbGl0dGxlIG1vcmUgZGlmZmVyZW50IHRvIHN0YW5kYXJkIElEQi4gSW5cbiAgICAvLyBJREIsIHlvdSBhZHZhbmNlIHRoZSBjdXJzb3IgYW5kIHdhaXQgZm9yIGEgbmV3ICdzdWNjZXNzJyBvbiB0aGUgSURCUmVxdWVzdCB0aGF0IGdhdmUgeW91IHRoZVxuICAgIC8vIGN1cnNvci4gSXQncyBraW5kYSBsaWtlIGEgcHJvbWlzZSB0aGF0IGNhbiByZXNvbHZlIHdpdGggbWFueSB2YWx1ZXMuIFRoYXQgZG9lc24ndCBtYWtlIHNlbnNlXG4gICAgLy8gd2l0aCByZWFsIHByb21pc2VzLCBzbyBlYWNoIGFkdmFuY2UgbWV0aG9kcyByZXR1cm5zIGEgbmV3IHByb21pc2UgZm9yIHRoZSBjdXJzb3Igb2JqZWN0LCBvclxuICAgIC8vIHVuZGVmaW5lZCBpZiB0aGUgZW5kIG9mIHRoZSBjdXJzb3IgaGFzIGJlZW4gcmVhY2hlZC5cbiAgICBpZiAoZ2V0Q3Vyc29yQWR2YW5jZU1ldGhvZHMoKS5pbmNsdWRlcyhmdW5jKSkge1xuICAgICAgICByZXR1cm4gZnVuY3Rpb24gKC4uLmFyZ3MpIHtcbiAgICAgICAgICAgIC8vIENhbGxpbmcgdGhlIG9yaWdpbmFsIGZ1bmN0aW9uIHdpdGggdGhlIHByb3h5IGFzICd0aGlzJyBjYXVzZXMgSUxMRUdBTCBJTlZPQ0FUSU9OLCBzbyB3ZSB1c2VcbiAgICAgICAgICAgIC8vIHRoZSBvcmlnaW5hbCBvYmplY3QuXG4gICAgICAgICAgICBmdW5jLmFwcGx5KHVud3JhcCh0aGlzKSwgYXJncyk7XG4gICAgICAgICAgICByZXR1cm4gd3JhcCh0aGlzLnJlcXVlc3QpO1xuICAgICAgICB9O1xuICAgIH1cbiAgICByZXR1cm4gZnVuY3Rpb24gKC4uLmFyZ3MpIHtcbiAgICAgICAgLy8gQ2FsbGluZyB0aGUgb3JpZ2luYWwgZnVuY3Rpb24gd2l0aCB0aGUgcHJveHkgYXMgJ3RoaXMnIGNhdXNlcyBJTExFR0FMIElOVk9DQVRJT04sIHNvIHdlIHVzZVxuICAgICAgICAvLyB0aGUgb3JpZ2luYWwgb2JqZWN0LlxuICAgICAgICByZXR1cm4gd3JhcChmdW5jLmFwcGx5KHVud3JhcCh0aGlzKSwgYXJncykpO1xuICAgIH07XG59XG5mdW5jdGlvbiB0cmFuc2Zvcm1DYWNoYWJsZVZhbHVlKHZhbHVlKSB7XG4gICAgaWYgKHR5cGVvZiB2YWx1ZSA9PT0gJ2Z1bmN0aW9uJylcbiAgICAgICAgcmV0dXJuIHdyYXBGdW5jdGlvbih2YWx1ZSk7XG4gICAgLy8gVGhpcyBkb2Vzbid0IHJldHVybiwgaXQganVzdCBjcmVhdGVzIGEgJ2RvbmUnIHByb21pc2UgZm9yIHRoZSB0cmFuc2FjdGlvbixcbiAgICAvLyB3aGljaCBpcyBsYXRlciByZXR1cm5lZCBmb3IgdHJhbnNhY3Rpb24uZG9uZSAoc2VlIGlkYk9iamVjdEhhbmRsZXIpLlxuICAgIGlmICh2YWx1ZSBpbnN0YW5jZW9mIElEQlRyYW5zYWN0aW9uKVxuICAgICAgICBjYWNoZURvbmVQcm9taXNlRm9yVHJhbnNhY3Rpb24odmFsdWUpO1xuICAgIGlmIChpbnN0YW5jZU9mQW55KHZhbHVlLCBnZXRJZGJQcm94eWFibGVUeXBlcygpKSlcbiAgICAgICAgcmV0dXJuIG5ldyBQcm94eSh2YWx1ZSwgaWRiUHJveHlUcmFwcyk7XG4gICAgLy8gUmV0dXJuIHRoZSBzYW1lIHZhbHVlIGJhY2sgaWYgd2UncmUgbm90IGdvaW5nIHRvIHRyYW5zZm9ybSBpdC5cbiAgICByZXR1cm4gdmFsdWU7XG59XG5mdW5jdGlvbiB3cmFwKHZhbHVlKSB7XG4gICAgLy8gV2Ugc29tZXRpbWVzIGdlbmVyYXRlIG11bHRpcGxlIHByb21pc2VzIGZyb20gYSBzaW5nbGUgSURCUmVxdWVzdCAoZWcgd2hlbiBjdXJzb3JpbmcpLCBiZWNhdXNlXG4gICAgLy8gSURCIGlzIHdlaXJkIGFuZCBhIHNpbmdsZSBJREJSZXF1ZXN0IGNhbiB5aWVsZCBtYW55IHJlc3BvbnNlcywgc28gdGhlc2UgY2FuJ3QgYmUgY2FjaGVkLlxuICAgIGlmICh2YWx1ZSBpbnN0YW5jZW9mIElEQlJlcXVlc3QpXG4gICAgICAgIHJldHVybiBwcm9taXNpZnlSZXF1ZXN0KHZhbHVlKTtcbiAgICAvLyBJZiB3ZSd2ZSBhbHJlYWR5IHRyYW5zZm9ybWVkIHRoaXMgdmFsdWUgYmVmb3JlLCByZXVzZSB0aGUgdHJhbnNmb3JtZWQgdmFsdWUuXG4gICAgLy8gVGhpcyBpcyBmYXN0ZXIsIGJ1dCBpdCBhbHNvIHByb3ZpZGVzIG9iamVjdCBlcXVhbGl0eS5cbiAgICBpZiAodHJhbnNmb3JtQ2FjaGUuaGFzKHZhbHVlKSlcbiAgICAgICAgcmV0dXJuIHRyYW5zZm9ybUNhY2hlLmdldCh2YWx1ZSk7XG4gICAgY29uc3QgbmV3VmFsdWUgPSB0cmFuc2Zvcm1DYWNoYWJsZVZhbHVlKHZhbHVlKTtcbiAgICAvLyBOb3QgYWxsIHR5cGVzIGFyZSB0cmFuc2Zvcm1lZC5cbiAgICAvLyBUaGVzZSBtYXkgYmUgcHJpbWl0aXZlIHR5cGVzLCBzbyB0aGV5IGNhbid0IGJlIFdlYWtNYXAga2V5cy5cbiAgICBpZiAobmV3VmFsdWUgIT09IHZhbHVlKSB7XG4gICAgICAgIHRyYW5zZm9ybUNhY2hlLnNldCh2YWx1ZSwgbmV3VmFsdWUpO1xuICAgICAgICByZXZlcnNlVHJhbnNmb3JtQ2FjaGUuc2V0KG5ld1ZhbHVlLCB2YWx1ZSk7XG4gICAgfVxuICAgIHJldHVybiBuZXdWYWx1ZTtcbn1cbmNvbnN0IHVud3JhcCA9ICh2YWx1ZSkgPT4gcmV2ZXJzZVRyYW5zZm9ybUNhY2hlLmdldCh2YWx1ZSk7XG5cbi8qKlxuICogT3BlbiBhIGRhdGFiYXNlLlxuICpcbiAqIEBwYXJhbSBuYW1lIE5hbWUgb2YgdGhlIGRhdGFiYXNlLlxuICogQHBhcmFtIHZlcnNpb24gU2NoZW1hIHZlcnNpb24uXG4gKiBAcGFyYW0gY2FsbGJhY2tzIEFkZGl0aW9uYWwgY2FsbGJhY2tzLlxuICovXG5mdW5jdGlvbiBvcGVuREIobmFtZSwgdmVyc2lvbiwgeyBibG9ja2VkLCB1cGdyYWRlLCBibG9ja2luZywgdGVybWluYXRlZCB9ID0ge30pIHtcbiAgICBjb25zdCByZXF1ZXN0ID0gaW5kZXhlZERCLm9wZW4obmFtZSwgdmVyc2lvbik7XG4gICAgY29uc3Qgb3BlblByb21pc2UgPSB3cmFwKHJlcXVlc3QpO1xuICAgIGlmICh1cGdyYWRlKSB7XG4gICAgICAgIHJlcXVlc3QuYWRkRXZlbnRMaXN0ZW5lcigndXBncmFkZW5lZWRlZCcsIChldmVudCkgPT4ge1xuICAgICAgICAgICAgdXBncmFkZSh3cmFwKHJlcXVlc3QucmVzdWx0KSwgZXZlbnQub2xkVmVyc2lvbiwgZXZlbnQubmV3VmVyc2lvbiwgd3JhcChyZXF1ZXN0LnRyYW5zYWN0aW9uKSwgZXZlbnQpO1xuICAgICAgICB9KTtcbiAgICB9XG4gICAgaWYgKGJsb2NrZWQpIHtcbiAgICAgICAgcmVxdWVzdC5hZGRFdmVudExpc3RlbmVyKCdibG9ja2VkJywgKGV2ZW50KSA9PiBibG9ja2VkKFxuICAgICAgICAvLyBDYXN0aW5nIGR1ZSB0byBodHRwczovL2dpdGh1Yi5jb20vbWljcm9zb2Z0L1R5cGVTY3JpcHQtRE9NLWxpYi1nZW5lcmF0b3IvcHVsbC8xNDA1XG4gICAgICAgIGV2ZW50Lm9sZFZlcnNpb24sIGV2ZW50Lm5ld1ZlcnNpb24sIGV2ZW50KSk7XG4gICAgfVxuICAgIG9wZW5Qcm9taXNlXG4gICAgICAgIC50aGVuKChkYikgPT4ge1xuICAgICAgICBpZiAodGVybWluYXRlZClcbiAgICAgICAgICAgIGRiLmFkZEV2ZW50TGlzdGVuZXIoJ2Nsb3NlJywgKCkgPT4gdGVybWluYXRlZCgpKTtcbiAgICAgICAgaWYgKGJsb2NraW5nKSB7XG4gICAgICAgICAgICBkYi5hZGRFdmVudExpc3RlbmVyKCd2ZXJzaW9uY2hhbmdlJywgKGV2ZW50KSA9PiBibG9ja2luZyhldmVudC5vbGRWZXJzaW9uLCBldmVudC5uZXdWZXJzaW9uLCBldmVudCkpO1xuICAgICAgICB9XG4gICAgfSlcbiAgICAgICAgLmNhdGNoKCgpID0+IHsgfSk7XG4gICAgcmV0dXJuIG9wZW5Qcm9taXNlO1xufVxuLyoqXG4gKiBEZWxldGUgYSBkYXRhYmFzZS5cbiAqXG4gKiBAcGFyYW0gbmFtZSBOYW1lIG9mIHRoZSBkYXRhYmFzZS5cbiAqL1xuZnVuY3Rpb24gZGVsZXRlREIobmFtZSwgeyBibG9ja2VkIH0gPSB7fSkge1xuICAgIGNvbnN0IHJlcXVlc3QgPSBpbmRleGVkREIuZGVsZXRlRGF0YWJhc2UobmFtZSk7XG4gICAgaWYgKGJsb2NrZWQpIHtcbiAgICAgICAgcmVxdWVzdC5hZGRFdmVudExpc3RlbmVyKCdibG9ja2VkJywgKGV2ZW50KSA9PiBibG9ja2VkKFxuICAgICAgICAvLyBDYXN0aW5nIGR1ZSB0byBodHRwczovL2dpdGh1Yi5jb20vbWljcm9zb2Z0L1R5cGVTY3JpcHQtRE9NLWxpYi1nZW5lcmF0b3IvcHVsbC8xNDA1XG4gICAgICAgIGV2ZW50Lm9sZFZlcnNpb24sIGV2ZW50KSk7XG4gICAgfVxuICAgIHJldHVybiB3cmFwKHJlcXVlc3QpLnRoZW4oKCkgPT4gdW5kZWZpbmVkKTtcbn1cblxuY29uc3QgcmVhZE1ldGhvZHMgPSBbJ2dldCcsICdnZXRLZXknLCAnZ2V0QWxsJywgJ2dldEFsbEtleXMnLCAnY291bnQnXTtcbmNvbnN0IHdyaXRlTWV0aG9kcyA9IFsncHV0JywgJ2FkZCcsICdkZWxldGUnLCAnY2xlYXInXTtcbmNvbnN0IGNhY2hlZE1ldGhvZHMgPSBuZXcgTWFwKCk7XG5mdW5jdGlvbiBnZXRNZXRob2QodGFyZ2V0LCBwcm9wKSB7XG4gICAgaWYgKCEodGFyZ2V0IGluc3RhbmNlb2YgSURCRGF0YWJhc2UgJiZcbiAgICAgICAgIShwcm9wIGluIHRhcmdldCkgJiZcbiAgICAgICAgdHlwZW9mIHByb3AgPT09ICdzdHJpbmcnKSkge1xuICAgICAgICByZXR1cm47XG4gICAgfVxuICAgIGlmIChjYWNoZWRNZXRob2RzLmdldChwcm9wKSlcbiAgICAgICAgcmV0dXJuIGNhY2hlZE1ldGhvZHMuZ2V0KHByb3ApO1xuICAgIGNvbnN0IHRhcmdldEZ1bmNOYW1lID0gcHJvcC5yZXBsYWNlKC9Gcm9tSW5kZXgkLywgJycpO1xuICAgIGNvbnN0IHVzZUluZGV4ID0gcHJvcCAhPT0gdGFyZ2V0RnVuY05hbWU7XG4gICAgY29uc3QgaXNXcml0ZSA9IHdyaXRlTWV0aG9kcy5pbmNsdWRlcyh0YXJnZXRGdW5jTmFtZSk7XG4gICAgaWYgKFxuICAgIC8vIEJhaWwgaWYgdGhlIHRhcmdldCBkb2Vzbid0IGV4aXN0IG9uIHRoZSB0YXJnZXQuIEVnLCBnZXRBbGwgaXNuJ3QgaW4gRWRnZS5cbiAgICAhKHRhcmdldEZ1bmNOYW1lIGluICh1c2VJbmRleCA/IElEQkluZGV4IDogSURCT2JqZWN0U3RvcmUpLnByb3RvdHlwZSkgfHxcbiAgICAgICAgIShpc1dyaXRlIHx8IHJlYWRNZXRob2RzLmluY2x1ZGVzKHRhcmdldEZ1bmNOYW1lKSkpIHtcbiAgICAgICAgcmV0dXJuO1xuICAgIH1cbiAgICBjb25zdCBtZXRob2QgPSBhc3luYyBmdW5jdGlvbiAoc3RvcmVOYW1lLCAuLi5hcmdzKSB7XG4gICAgICAgIC8vIGlzV3JpdGUgPyAncmVhZHdyaXRlJyA6IHVuZGVmaW5lZCBnemlwcHMgYmV0dGVyLCBidXQgZmFpbHMgaW4gRWRnZSA6KFxuICAgICAgICBjb25zdCB0eCA9IHRoaXMudHJhbnNhY3Rpb24oc3RvcmVOYW1lLCBpc1dyaXRlID8gJ3JlYWR3cml0ZScgOiAncmVhZG9ubHknKTtcbiAgICAgICAgbGV0IHRhcmdldCA9IHR4LnN0b3JlO1xuICAgICAgICBpZiAodXNlSW5kZXgpXG4gICAgICAgICAgICB0YXJnZXQgPSB0YXJnZXQuaW5kZXgoYXJncy5zaGlmdCgpKTtcbiAgICAgICAgLy8gTXVzdCByZWplY3QgaWYgb3AgcmVqZWN0cy5cbiAgICAgICAgLy8gSWYgaXQncyBhIHdyaXRlIG9wZXJhdGlvbiwgbXVzdCByZWplY3QgaWYgdHguZG9uZSByZWplY3RzLlxuICAgICAgICAvLyBNdXN0IHJlamVjdCB3aXRoIG9wIHJlamVjdGlvbiBmaXJzdC5cbiAgICAgICAgLy8gTXVzdCByZXNvbHZlIHdpdGggb3AgdmFsdWUuXG4gICAgICAgIC8vIE11c3QgaGFuZGxlIGJvdGggcHJvbWlzZXMgKG5vIHVuaGFuZGxlZCByZWplY3Rpb25zKVxuICAgICAgICByZXR1cm4gKGF3YWl0IFByb21pc2UuYWxsKFtcbiAgICAgICAgICAgIHRhcmdldFt0YXJnZXRGdW5jTmFtZV0oLi4uYXJncyksXG4gICAgICAgICAgICBpc1dyaXRlICYmIHR4LmRvbmUsXG4gICAgICAgIF0pKVswXTtcbiAgICB9O1xuICAgIGNhY2hlZE1ldGhvZHMuc2V0KHByb3AsIG1ldGhvZCk7XG4gICAgcmV0dXJuIG1ldGhvZDtcbn1cbnJlcGxhY2VUcmFwcygob2xkVHJhcHMpID0+ICh7XG4gICAgLi4ub2xkVHJhcHMsXG4gICAgZ2V0OiAodGFyZ2V0LCBwcm9wLCByZWNlaXZlcikgPT4gZ2V0TWV0aG9kKHRhcmdldCwgcHJvcCkgfHwgb2xkVHJhcHMuZ2V0KHRhcmdldCwgcHJvcCwgcmVjZWl2ZXIpLFxuICAgIGhhczogKHRhcmdldCwgcHJvcCkgPT4gISFnZXRNZXRob2QodGFyZ2V0LCBwcm9wKSB8fCBvbGRUcmFwcy5oYXModGFyZ2V0LCBwcm9wKSxcbn0pKTtcblxuY29uc3QgYWR2YW5jZU1ldGhvZFByb3BzID0gWydjb250aW51ZScsICdjb250aW51ZVByaW1hcnlLZXknLCAnYWR2YW5jZSddO1xuY29uc3QgbWV0aG9kTWFwID0ge307XG5jb25zdCBhZHZhbmNlUmVzdWx0cyA9IG5ldyBXZWFrTWFwKCk7XG5jb25zdCBpdHRyUHJveGllZEN1cnNvclRvT3JpZ2luYWxQcm94eSA9IG5ldyBXZWFrTWFwKCk7XG5jb25zdCBjdXJzb3JJdGVyYXRvclRyYXBzID0ge1xuICAgIGdldCh0YXJnZXQsIHByb3ApIHtcbiAgICAgICAgaWYgKCFhZHZhbmNlTWV0aG9kUHJvcHMuaW5jbHVkZXMocHJvcCkpXG4gICAgICAgICAgICByZXR1cm4gdGFyZ2V0W3Byb3BdO1xuICAgICAgICBsZXQgY2FjaGVkRnVuYyA9IG1ldGhvZE1hcFtwcm9wXTtcbiAgICAgICAgaWYgKCFjYWNoZWRGdW5jKSB7XG4gICAgICAgICAgICBjYWNoZWRGdW5jID0gbWV0aG9kTWFwW3Byb3BdID0gZnVuY3Rpb24gKC4uLmFyZ3MpIHtcbiAgICAgICAgICAgICAgICBhZHZhbmNlUmVzdWx0cy5zZXQodGhpcywgaXR0clByb3hpZWRDdXJzb3JUb09yaWdpbmFsUHJveHkuZ2V0KHRoaXMpW3Byb3BdKC4uLmFyZ3MpKTtcbiAgICAgICAgICAgIH07XG4gICAgICAgIH1cbiAgICAgICAgcmV0dXJuIGNhY2hlZEZ1bmM7XG4gICAgfSxcbn07XG5hc3luYyBmdW5jdGlvbiogaXRlcmF0ZSguLi5hcmdzKSB7XG4gICAgLy8gdHNsaW50OmRpc2FibGUtbmV4dC1saW5lOm5vLXRoaXMtYXNzaWdubWVudFxuICAgIGxldCBjdXJzb3IgPSB0aGlzO1xuICAgIGlmICghKGN1cnNvciBpbnN0YW5jZW9mIElEQkN1cnNvcikpIHtcbiAgICAgICAgY3Vyc29yID0gYXdhaXQgY3Vyc29yLm9wZW5DdXJzb3IoLi4uYXJncyk7XG4gICAgfVxuICAgIGlmICghY3Vyc29yKVxuICAgICAgICByZXR1cm47XG4gICAgY3Vyc29yID0gY3Vyc29yO1xuICAgIGNvbnN0IHByb3hpZWRDdXJzb3IgPSBuZXcgUHJveHkoY3Vyc29yLCBjdXJzb3JJdGVyYXRvclRyYXBzKTtcbiAgICBpdHRyUHJveGllZEN1cnNvclRvT3JpZ2luYWxQcm94eS5zZXQocHJveGllZEN1cnNvciwgY3Vyc29yKTtcbiAgICAvLyBNYXAgdGhpcyBkb3VibGUtcHJveHkgYmFjayB0byB0aGUgb3JpZ2luYWwsIHNvIG90aGVyIGN1cnNvciBtZXRob2RzIHdvcmsuXG4gICAgcmV2ZXJzZVRyYW5zZm9ybUNhY2hlLnNldChwcm94aWVkQ3Vyc29yLCB1bndyYXAoY3Vyc29yKSk7XG4gICAgd2hpbGUgKGN1cnNvcikge1xuICAgICAgICB5aWVsZCBwcm94aWVkQ3Vyc29yO1xuICAgICAgICAvLyBJZiBvbmUgb2YgdGhlIGFkdmFuY2luZyBtZXRob2RzIHdhcyBub3QgY2FsbGVkLCBjYWxsIGNvbnRpbnVlKCkuXG4gICAgICAgIGN1cnNvciA9IGF3YWl0IChhZHZhbmNlUmVzdWx0cy5nZXQocHJveGllZEN1cnNvcikgfHwgY3Vyc29yLmNvbnRpbnVlKCkpO1xuICAgICAgICBhZHZhbmNlUmVzdWx0cy5kZWxldGUocHJveGllZEN1cnNvcik7XG4gICAgfVxufVxuZnVuY3Rpb24gaXNJdGVyYXRvclByb3AodGFyZ2V0LCBwcm9wKSB7XG4gICAgcmV0dXJuICgocHJvcCA9PT0gU3ltYm9sLmFzeW5jSXRlcmF0b3IgJiZcbiAgICAgICAgaW5zdGFuY2VPZkFueSh0YXJnZXQsIFtJREJJbmRleCwgSURCT2JqZWN0U3RvcmUsIElEQkN1cnNvcl0pKSB8fFxuICAgICAgICAocHJvcCA9PT0gJ2l0ZXJhdGUnICYmIGluc3RhbmNlT2ZBbnkodGFyZ2V0LCBbSURCSW5kZXgsIElEQk9iamVjdFN0b3JlXSkpKTtcbn1cbnJlcGxhY2VUcmFwcygob2xkVHJhcHMpID0+ICh7XG4gICAgLi4ub2xkVHJhcHMsXG4gICAgZ2V0KHRhcmdldCwgcHJvcCwgcmVjZWl2ZXIpIHtcbiAgICAgICAgaWYgKGlzSXRlcmF0b3JQcm9wKHRhcmdldCwgcHJvcCkpXG4gICAgICAgICAgICByZXR1cm4gaXRlcmF0ZTtcbiAgICAgICAgcmV0dXJuIG9sZFRyYXBzLmdldCh0YXJnZXQsIHByb3AsIHJlY2VpdmVyKTtcbiAgICB9LFxuICAgIGhhcyh0YXJnZXQsIHByb3ApIHtcbiAgICAgICAgcmV0dXJuIGlzSXRlcmF0b3JQcm9wKHRhcmdldCwgcHJvcCkgfHwgb2xkVHJhcHMuaGFzKHRhcmdldCwgcHJvcCk7XG4gICAgfSxcbn0pKTtcblxuZXhwb3J0IHsgZGVsZXRlREIsIG9wZW5EQiwgdW53cmFwLCB3cmFwIH07XG4iLCAiaW1wb3J0IHsgb3BlbkRCIH0gZnJvbSAnaWRiJztcblxuYXN5bmMgZnVuY3Rpb24gb3BlbkV2ZW50c0RiKCkge1xuICAgIHJldHVybiBhd2FpdCBvcGVuREIoJ2V2ZW50cycsIDEsIHtcbiAgICAgICAgdXBncmFkZShkYikge1xuICAgICAgICAgICAgY29uc3QgZXZlbnRzID0gZGIuY3JlYXRlT2JqZWN0U3RvcmUoJ2V2ZW50cycsIHtcbiAgICAgICAgICAgICAgICBrZXlQYXRoOiAnZXZlbnQuaWQnLFxuICAgICAgICAgICAgfSk7XG4gICAgICAgICAgICBldmVudHMuY3JlYXRlSW5kZXgoJ3B1YmtleScsICdldmVudC5wdWJrZXknKTtcbiAgICAgICAgICAgIGV2ZW50cy5jcmVhdGVJbmRleCgnY3JlYXRlZF9hdCcsICdldmVudC5jcmVhdGVkX2F0Jyk7XG4gICAgICAgICAgICBldmVudHMuY3JlYXRlSW5kZXgoJ2tpbmQnLCAnZXZlbnQua2luZCcpO1xuICAgICAgICAgICAgZXZlbnRzLmNyZWF0ZUluZGV4KCdob3N0JywgJ21ldGFkYXRhLmhvc3QnKTtcbiAgICAgICAgfSxcbiAgICB9KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHNhdmVFdmVudChldmVudCkge1xuICAgIGxldCBkYiA9IGF3YWl0IG9wZW5FdmVudHNEYigpO1xuICAgIHJldHVybiBkYi5wdXQoJ2V2ZW50cycsIGV2ZW50KTtcbn1cblxuZXhwb3J0IGFzeW5jIGZ1bmN0aW9uIHNvcnRCeUluZGV4KGluZGV4LCBxdWVyeSwgYXNjLCBtYXgpIHtcbiAgICBsZXQgZGIgPSBhd2FpdCBvcGVuRXZlbnRzRGIoKTtcbiAgICBsZXQgZXZlbnRzID0gW107XG4gICAgbGV0IGN1cnNvciA9IGF3YWl0IGRiXG4gICAgICAgIC50cmFuc2FjdGlvbignZXZlbnRzJylcbiAgICAgICAgLnN0b3JlLmluZGV4KGluZGV4KVxuICAgICAgICAub3BlbkN1cnNvcihxdWVyeSwgYXNjID8gJ25leHQnIDogJ3ByZXYnKTtcbiAgICB3aGlsZSAoY3Vyc29yKSB7XG4gICAgICAgIGV2ZW50cy5wdXNoKGN1cnNvci52YWx1ZSk7XG4gICAgICAgIGlmIChldmVudHMubGVuZ3RoID49IG1heCkge1xuICAgICAgICAgICAgYnJlYWs7XG4gICAgICAgIH1cbiAgICAgICAgY3Vyc29yID0gYXdhaXQgY3Vyc29yLmNvbnRpbnVlKCk7XG4gICAgfVxuICAgIHJldHVybiBldmVudHM7XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBnZXRIb3N0cygpIHtcbiAgICBsZXQgZGIgPSBhd2FpdCBvcGVuRXZlbnRzRGIoKTtcbiAgICBsZXQgaG9zdHMgPSBuZXcgU2V0KCk7XG4gICAgbGV0IGN1cnNvciA9IGF3YWl0IGRiLnRyYW5zYWN0aW9uKCdldmVudHMnKS5zdG9yZS5vcGVuQ3Vyc29yKCk7XG4gICAgd2hpbGUgKGN1cnNvcikge1xuICAgICAgICBob3N0cy5hZGQoY3Vyc29yLnZhbHVlLm1ldGFkYXRhLmhvc3QpO1xuICAgICAgICBjdXJzb3IgPSBhd2FpdCBjdXJzb3IuY29udGludWUoKTtcbiAgICB9XG4gICAgcmV0dXJuIFsuLi5ob3N0c107XG59XG5cbmV4cG9ydCBhc3luYyBmdW5jdGlvbiBkb3dubG9hZEFsbENvbnRlbnRzKCkge1xuICAgIGxldCBkYiA9IGF3YWl0IG9wZW5FdmVudHNEYigpO1xuICAgIGxldCBldmVudHMgPSBbXTtcbiAgICBsZXQgY3Vyc29yID0gYXdhaXQgZGIudHJhbnNhY3Rpb24oJ2V2ZW50cycpLnN0b3JlLm9wZW5DdXJzb3IoKTtcbiAgICB3aGlsZSAoY3Vyc29yKSB7XG4gICAgICAgIGV2ZW50cy5wdXNoKGN1cnNvci52YWx1ZS5ldmVudCk7XG4gICAgICAgIGN1cnNvciA9IGF3YWl0IGN1cnNvci5jb250aW51ZSgpO1xuICAgIH1cbiAgICBldmVudHMgPSBldmVudHMubWFwKGUgPT4gSlNPTi5zdHJpbmdpZnkoZSkpO1xuICAgIGV2ZW50cyA9IGV2ZW50cy5qb2luKCdcXG4nKTtcbiAgICBjb25zb2xlLmxvZyhldmVudHMpO1xuXG4gICAgY29uc3QgZmlsZSA9IG5ldyBGaWxlKFtldmVudHNdLCAnZXZlbnRzLmpzb25sJywge1xuICAgICAgICB0eXBlOiAnYXBwbGljYXRpb24vb2N0ZXQtc3RyZWFtJyxcbiAgICB9KTtcblxuICAgIGNvbnN0IGJsb2IgPSBuZXcgQmxvYihbZXZlbnRzXSwgeyB0eXBlOiAncGxhaW4vdGV4dCcgfSk7XG5cbiAgICByZXR1cm4gYmxvYjtcbn1cbiIsICJpbXBvcnQge1xuICAgIG5pcDA0LFxuICAgIG5pcDE5LFxuICAgIG5pcDQ0LFxuICAgIGdlbmVyYXRlU2VjcmV0S2V5LFxuICAgIGdldFB1YmxpY0tleSxcbiAgICBmaW5hbGl6ZUV2ZW50LFxufSBmcm9tICdub3N0ci10b29scyc7XG5pbXBvcnQgeyBieXRlc1RvSGV4LCBoZXhUb0J5dGVzIH0gZnJvbSAnQG5vYmxlL2hhc2hlcy91dGlscyc7XG5pbXBvcnQgeyBNdXRleCB9IGZyb20gJ2FzeW5jLW11dGV4JztcbmltcG9ydCB7XG4gICAgZ2V0UHJvZmlsZUluZGV4LFxuICAgIGdldCxcbiAgICBnZXRQcm9maWxlLFxuICAgIGdldFBlcm1pc3Npb24sXG4gICAgc2V0UGVybWlzc2lvbixcbn0gZnJvbSAnLi91dGlsaXRpZXMvdXRpbHMnO1xuaW1wb3J0IHsgc2F2ZUV2ZW50IH0gZnJvbSAnLi91dGlsaXRpZXMvZGInO1xuXG5jb25zdCBzdG9yYWdlID0gYnJvd3Nlci5zdG9yYWdlLmxvY2FsO1xuY29uc3QgbG9nID0gbXNnID0+IGNvbnNvbGUubG9nKCdCYWNrZ3JvdW5kOiAnLCBtc2cpO1xuY29uc3QgdmFsaWRhdGlvbnMgPSB7fTtcbmxldCBwcm9tcHQgPSB7IG11dGV4OiBuZXcgTXV0ZXgoKSwgcmVsZWFzZTogbnVsbCwgdGFiSWQ6IG51bGwgfTtcblxuYnJvd3Nlci5ydW50aW1lLm9uTWVzc2FnZS5hZGRMaXN0ZW5lcigobWVzc2FnZSwgX3NlbmRlciwgc2VuZFJlc3BvbnNlKSA9PiB7XG4gICAgbG9nKG1lc3NhZ2UpO1xuICAgIGxldCB1dWlkID0gY3J5cHRvLnJhbmRvbVVVSUQoKTtcbiAgICBsZXQgc3I7XG5cbiAgICBzd2l0Y2ggKG1lc3NhZ2Uua2luZCkge1xuICAgICAgICAvLyBHZW5lcmFsXG4gICAgICAgIGNhc2UgJ2Nsb3NlUHJvbXB0JzpcbiAgICAgICAgICAgIHByb21wdC5yZWxlYXNlPy4oKTtcbiAgICAgICAgICAgIHJldHVybiBQcm9taXNlLnJlc29sdmUodHJ1ZSk7XG4gICAgICAgIGNhc2UgJ2FsbG93ZWQnOlxuICAgICAgICAgICAgY29tcGxldGUobWVzc2FnZSk7XG4gICAgICAgICAgICByZXR1cm4gUHJvbWlzZS5yZXNvbHZlKHRydWUpO1xuICAgICAgICBjYXNlICdkZW5pZWQnOlxuICAgICAgICAgICAgZGVueShtZXNzYWdlKTtcbiAgICAgICAgICAgIHJldHVybiBQcm9taXNlLnJlc29sdmUodHJ1ZSk7XG4gICAgICAgIGNhc2UgJ2dlbmVyYXRlUHJpdmF0ZUtleSc6XG4gICAgICAgICAgICByZXR1cm4gUHJvbWlzZS5yZXNvbHZlKGdlbmVyYXRlUHJpdmF0ZUtleV8oKSk7XG4gICAgICAgIGNhc2UgJ3NhdmVQcml2YXRlS2V5JzpcbiAgICAgICAgICAgIHJldHVybiBzYXZlUHJpdmF0ZUtleShtZXNzYWdlLnBheWxvYWQpO1xuICAgICAgICBjYXNlICdnZXROcHViJzpcbiAgICAgICAgICAgIHJldHVybiBnZXROcHViKG1lc3NhZ2UucGF5bG9hZCk7XG4gICAgICAgIGNhc2UgJ2dldE5zZWMnOlxuICAgICAgICAgICAgcmV0dXJuIGdldE5zZWMobWVzc2FnZS5wYXlsb2FkKTtcbiAgICAgICAgY2FzZSAnY2FsY1B1YktleSc6XG4gICAgICAgICAgICByZXR1cm4gUHJvbWlzZS5yZXNvbHZlKGdldFB1YmxpY0tleShtZXNzYWdlLnBheWxvYWQpKTtcbiAgICAgICAgY2FzZSAnbnB1YkVuY29kZSc6XG4gICAgICAgICAgICByZXR1cm4gUHJvbWlzZS5yZXNvbHZlKG5pcDE5Lm5wdWJFbmNvZGUobWVzc2FnZS5wYXlsb2FkKSk7XG4gICAgICAgIGNhc2UgJ2NvcHknOlxuICAgICAgICAgICAgcmV0dXJuIG5hdmlnYXRvci5jbGlwYm9hcmQud3JpdGVUZXh0KG1lc3NhZ2UucGF5bG9hZCk7XG5cbiAgICAgICAgLy8gd2luZG93Lm5vc3RyXG4gICAgICAgIGNhc2UgJ2dldFB1YktleSc6XG4gICAgICAgIGNhc2UgJ3NpZ25FdmVudCc6XG4gICAgICAgIGNhc2UgJ25pcDA0LmVuY3J5cHQnOlxuICAgICAgICBjYXNlICduaXAwNC5kZWNyeXB0JzpcbiAgICAgICAgY2FzZSAnbmlwNDQuZW5jcnlwdCc6XG4gICAgICAgIGNhc2UgJ25pcDQ0LmRlY3J5cHQnOlxuICAgICAgICBjYXNlICdnZXRSZWxheXMnOlxuICAgICAgICAgICAgdmFsaWRhdGlvbnNbdXVpZF0gPSBzZW5kUmVzcG9uc2U7XG4gICAgICAgICAgICBhc2sodXVpZCwgbWVzc2FnZSk7XG4gICAgICAgICAgICBzZXRUaW1lb3V0KCgpID0+IHtcbiAgICAgICAgICAgICAgICBwcm9tcHQucmVsZWFzZT8uKCk7XG4gICAgICAgICAgICB9LCAxMF8wMDApO1xuICAgICAgICAgICAgcmV0dXJuIHRydWU7XG4gICAgICAgIGRlZmF1bHQ6XG4gICAgICAgICAgICByZXR1cm4gUHJvbWlzZS5yZXNvbHZlKCk7XG4gICAgfVxufSk7XG5cbmFzeW5jIGZ1bmN0aW9uIGZvcmNlUmVsZWFzZSgpIHtcbiAgICBpZiAocHJvbXB0LnRhYklkICE9PSBudWxsKSB7XG4gICAgICAgIHRyeSB7XG4gICAgICAgICAgICAvLyBJZiB0aGUgcHJldmlvdXMgcHJvbXB0IGlzIHN0aWxsIG9wZW4sIHRoZW4gdGhpcyB3b24ndCBkbyBhbnl0aGluZy5cbiAgICAgICAgICAgIC8vIElmIGl0J3Mgbm90IG9wZW4sIGl0IHdpbGwgdGhyb3cgYW4gZXJyb3IgYW5kIGdldCBjYXVnaHQuXG4gICAgICAgICAgICBhd2FpdCBicm93c2VyLnRhYnMuZ2V0KHByb21wdC50YWJJZCk7XG4gICAgICAgIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgICAgICAgICAvLyBJZiB0aGUgdGFiIGlzIGNsb3NlZCwgYnV0IHNvbWVob3cgZXNjYXBlZCBvdXIgZXZlbnQgaGFuZGxpbmcsIHdlIGNhbiBjbGVhbiBpdCB1cCBoZXJlXG4gICAgICAgICAgICAvLyBiZWZvcmUgYXR0ZW1wdGluZyB0byBvcGVuIHRoZSBuZXh0IHRhYi5cbiAgICAgICAgICAgIHByb21wdC5yZWxlYXNlPy4oKTtcbiAgICAgICAgICAgIHByb21wdC50YWJJZCA9IG51bGw7XG4gICAgICAgIH1cbiAgICB9XG59XG5cbmFzeW5jIGZ1bmN0aW9uIGdlbmVyYXRlUHJpdmF0ZUtleV8oKSB7XG4gICAgY29uc3Qgc2sgPSBnZW5lcmF0ZVNlY3JldEtleSgpO1xuICAgIHJldHVybiBieXRlc1RvSGV4KHNrKTtcbn1cblxuYXN5bmMgZnVuY3Rpb24gYXNrKHV1aWQsIHsga2luZCwgaG9zdCwgcGF5bG9hZCB9KSB7XG4gICAgYXdhaXQgZm9yY2VSZWxlYXNlKCk7IC8vIENsZWFuIHVwIHByZXZpb3VzIHRhYiBpZiBpdCBjbG9zZWQgd2l0aG91dCBjbGVhbmluZyBpdHNlbGYgdXBcbiAgICBwcm9tcHQucmVsZWFzZSA9IGF3YWl0IHByb21wdC5tdXRleC5hY3F1aXJlKCk7XG5cbiAgICBsZXQgbUtpbmQgPSBraW5kID09PSAnc2lnbkV2ZW50JyA/IGBzaWduRXZlbnQ6JHtwYXlsb2FkLmtpbmR9YCA6IGtpbmQ7XG4gICAgbGV0IHBlcm1pc3Npb24gPSBhd2FpdCBnZXRQZXJtaXNzaW9uKGhvc3QsIG1LaW5kKTtcbiAgICBpZiAocGVybWlzc2lvbiA9PT0gJ2FsbG93Jykge1xuICAgICAgICBjb21wbGV0ZSh7XG4gICAgICAgICAgICBwYXlsb2FkOiB1dWlkLFxuICAgICAgICAgICAgb3JpZ0tpbmQ6IGtpbmQsXG4gICAgICAgICAgICBldmVudDogcGF5bG9hZCxcbiAgICAgICAgICAgIHJlbWVtYmVyOiBmYWxzZSxcbiAgICAgICAgICAgIGhvc3QsXG4gICAgICAgIH0pO1xuICAgICAgICBwcm9tcHQucmVsZWFzZSgpO1xuICAgICAgICByZXR1cm47XG4gICAgfVxuXG4gICAgaWYgKHBlcm1pc3Npb24gPT09ICdkZW55Jykge1xuICAgICAgICBkZW55KHsgcGF5bG9hZDogdXVpZCwgb3JpZ0tpbmQ6IGtpbmQsIGhvc3QgfSk7XG4gICAgICAgIHByb21wdC5yZWxlYXNlKCk7XG4gICAgICAgIHJldHVybjtcbiAgICB9XG5cbiAgICBsZXQgcXMgPSBuZXcgVVJMU2VhcmNoUGFyYW1zKHtcbiAgICAgICAgdXVpZCxcbiAgICAgICAga2luZCxcbiAgICAgICAgaG9zdCxcbiAgICAgICAgcGF5bG9hZDogSlNPTi5zdHJpbmdpZnkocGF5bG9hZCB8fCBmYWxzZSksXG4gICAgfSk7XG4gICAgbGV0IHRhYiA9IGF3YWl0IGJyb3dzZXIudGFicy5nZXRDdXJyZW50KCk7XG4gICAgbGV0IHAgPSBhd2FpdCBicm93c2VyLnRhYnMuY3JlYXRlKHtcbiAgICAgICAgdXJsOiBgL3Blcm1pc3Npb24vcGVybWlzc2lvbi5odG1sPyR7cXMudG9TdHJpbmcoKX1gLFxuICAgICAgICBvcGVuZXJUYWJJZDogdGFiLmlkLFxuICAgIH0pO1xuICAgIHByb21wdC50YWJJZCA9IHAuaWQ7XG4gICAgcmV0dXJuIHRydWU7XG59XG5cbmZ1bmN0aW9uIGNvbXBsZXRlKHsgcGF5bG9hZCwgb3JpZ0tpbmQsIGV2ZW50LCByZW1lbWJlciwgaG9zdCB9KSB7XG4gICAgc2VuZFJlc3BvbnNlID0gdmFsaWRhdGlvbnNbcGF5bG9hZF07XG5cbiAgICBpZiAocmVtZW1iZXIpIHtcbiAgICAgICAgbGV0IG1LaW5kID1cbiAgICAgICAgICAgIG9yaWdLaW5kID09PSAnc2lnbkV2ZW50JyA/IGBzaWduRXZlbnQ6JHtldmVudC5raW5kfWAgOiBvcmlnS2luZDtcbiAgICAgICAgc2V0UGVybWlzc2lvbihob3N0LCBtS2luZCwgJ2FsbG93Jyk7XG4gICAgfVxuXG4gICAgaWYgKHNlbmRSZXNwb25zZSkge1xuICAgICAgICBzd2l0Y2ggKG9yaWdLaW5kKSB7XG4gICAgICAgICAgICBjYXNlICdnZXRQdWJLZXknOlxuICAgICAgICAgICAgICAgIGdldFB1YktleSgpLnRoZW4ocGsgPT4ge1xuICAgICAgICAgICAgICAgICAgICBzZW5kUmVzcG9uc2UocGspO1xuICAgICAgICAgICAgICAgIH0pO1xuICAgICAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgICAgY2FzZSAnc2lnbkV2ZW50JzpcbiAgICAgICAgICAgICAgICBzaWduRXZlbnRfKGV2ZW50LCBob3N0KS50aGVuKGUgPT4gc2VuZFJlc3BvbnNlKGUpKTtcbiAgICAgICAgICAgICAgICBicmVhaztcbiAgICAgICAgICAgIGNhc2UgJ25pcDA0LmVuY3J5cHQnOlxuICAgICAgICAgICAgICAgIG5pcDA0RW5jcnlwdChldmVudCkudGhlbihlID0+IHNlbmRSZXNwb25zZShlKSk7XG4gICAgICAgICAgICAgICAgYnJlYWs7XG4gICAgICAgICAgICBjYXNlICduaXAwNC5kZWNyeXB0JzpcbiAgICAgICAgICAgICAgICBuaXAwNERlY3J5cHQoZXZlbnQpLnRoZW4oZSA9PiBzZW5kUmVzcG9uc2UoZSkpO1xuICAgICAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICAgICAgY2FzZSAnbmlwNDQuZW5jcnlwdCc6XG4gICAgICAgICAgICAgICAgbmlwNDRFbmNyeXB0KGV2ZW50KS50aGVuKGUgPT4gc2VuZFJlc3BvbnNlKGUpKTtcbiAgICAgICAgICAgICAgICBicmVhaztcbiAgICAgICAgICAgIGNhc2UgJ25pcDQ0LmRlY3J5cHQnOlxuICAgICAgICAgICAgICAgIG5pcDQ0RGVjcnlwdChldmVudCkudGhlbihlID0+IHNlbmRSZXNwb25zZShlKSk7XG4gICAgICAgICAgICAgICAgYnJlYWs7XG4gICAgICAgICAgICBjYXNlICdnZXRSZWxheXMnOlxuICAgICAgICAgICAgICAgIGdldFJlbGF5cygpLnRoZW4oZSA9PiBzZW5kUmVzcG9uc2UoZSkpO1xuICAgICAgICAgICAgICAgIGJyZWFrO1xuICAgICAgICB9XG4gICAgfVxufVxuXG5mdW5jdGlvbiBkZW55KHsgb3JpZ0tpbmQsIGhvc3QsIHBheWxvYWQsIHJlbWVtYmVyLCBldmVudCB9KSB7XG4gICAgc2VuZFJlc3BvbnNlID0gdmFsaWRhdGlvbnNbcGF5bG9hZF07XG5cbiAgICBpZiAocmVtZW1iZXIpIHtcbiAgICAgICAgbGV0IG1LaW5kID1cbiAgICAgICAgICAgIG9yaWdLaW5kID09PSAnc2lnbkV2ZW50JyA/IGBzaWduRXZlbnQ6JHtldmVudC5raW5kfWAgOiBvcmlnS2luZDtcbiAgICAgICAgc2V0UGVybWlzc2lvbihob3N0LCBtS2luZCwgJ2RlbnknKTtcbiAgICB9XG5cbiAgICBzZW5kUmVzcG9uc2U/Lih1bmRlZmluZWQpO1xuICAgIHJldHVybiBmYWxzZTtcbn1cblxuLy8gT3B0aW9uc1xuYXN5bmMgZnVuY3Rpb24gc2F2ZVByaXZhdGVLZXkoW2luZGV4LCBwcml2S2V5XSkge1xuICAgIGlmIChwcml2S2V5LnN0YXJ0c1dpdGgoJ25zZWMnKSkge1xuICAgICAgICBwcml2S2V5ID0gbmlwMTkuZGVjb2RlKHByaXZLZXkpLmRhdGE7XG4gICAgfVxuICAgIGxldCBwcm9maWxlcyA9IGF3YWl0IGdldCgncHJvZmlsZXMnKTtcbiAgICBwcm9maWxlc1tpbmRleF0ucHJpdktleSA9IGJ5dGVzVG9IZXgocHJpdktleSk7XG4gICAgYXdhaXQgc3RvcmFnZS5zZXQoeyBwcm9maWxlcyB9KTtcbiAgICByZXR1cm4gdHJ1ZTtcbn1cblxuYXN5bmMgZnVuY3Rpb24gZ2V0TnNlYyhpbmRleCkge1xuICAgIGxldCBwcm9maWxlID0gYXdhaXQgZ2V0UHJvZmlsZShpbmRleCk7XG4gICAgbGV0IG5zZWMgPSBuaXAxOS5uc2VjRW5jb2RlKGhleFRvQnl0ZXMocHJvZmlsZS5wcml2S2V5KSk7XG4gICAgcmV0dXJuIG5zZWM7XG59XG5cbmFzeW5jIGZ1bmN0aW9uIGdldE5wdWIoaW5kZXgpIHtcbiAgICBsZXQgcHJvZmlsZSA9IGF3YWl0IGdldFByb2ZpbGUoaW5kZXgpO1xuICAgIGxldCBwdWJLZXkgPSBnZXRQdWJsaWNLZXkoaGV4VG9CeXRlcyhwcm9maWxlLnByaXZLZXkpKTtcbiAgICBsZXQgbnB1YiA9IG5pcDE5Lm5wdWJFbmNvZGUocHViS2V5KTtcbiAgICByZXR1cm4gbnB1Yjtcbn1cblxuYXN5bmMgZnVuY3Rpb24gZ2V0UHJpdktleSgpIHtcbiAgICBsZXQgcHJvZmlsZSA9IGF3YWl0IGN1cnJlbnRQcm9maWxlKCk7XG4gICAgcmV0dXJuIGhleFRvQnl0ZXMocHJvZmlsZS5wcml2S2V5KTtcbn1cblxuYXN5bmMgZnVuY3Rpb24gZ2V0UHViS2V5KCkge1xuICAgIGxldCBwaSA9IGF3YWl0IGdldFByb2ZpbGVJbmRleCgpO1xuICAgIGxldCBwcm9maWxlID0gYXdhaXQgZ2V0UHJvZmlsZShwaSk7XG4gICAgbGV0IHByaXZLZXkgPSBhd2FpdCBnZXRQcml2S2V5KCk7XG4gICAgbGV0IHB1YktleSA9IGdldFB1YmxpY0tleShwcml2S2V5KTtcbiAgICByZXR1cm4gcHViS2V5O1xufVxuXG5hc3luYyBmdW5jdGlvbiBjdXJyZW50UHJvZmlsZSgpIHtcbiAgICBsZXQgaW5kZXggPSBhd2FpdCBnZXRQcm9maWxlSW5kZXgoKTtcbiAgICBsZXQgcHJvZmlsZXMgPSBhd2FpdCBnZXQoJ3Byb2ZpbGVzJyk7XG4gICAgcmV0dXJuIHByb2ZpbGVzW2luZGV4XTtcbn1cblxuYXN5bmMgZnVuY3Rpb24gc2lnbkV2ZW50XyhldmVudCwgaG9zdCkge1xuICAgIGV2ZW50ID0gSlNPTi5wYXJzZShKU09OLnN0cmluZ2lmeShldmVudCkpO1xuICAgIGxldCBzayA9IGF3YWl0IGdldFByaXZLZXkoKTtcbiAgICBldmVudCA9IGZpbmFsaXplRXZlbnQoZXZlbnQsIHNrKTtcbiAgICBzYXZlRXZlbnQoe1xuICAgICAgICBldmVudCxcbiAgICAgICAgbWV0YWRhdGE6IHsgaG9zdCwgc2lnbmVkX2F0OiBNYXRoLnJvdW5kKERhdGUubm93KCkgLyAxMDAwKSB9LFxuICAgIH0pO1xuICAgIHJldHVybiBldmVudDtcbn1cblxuYXN5bmMgZnVuY3Rpb24gbmlwMDRFbmNyeXB0KHsgcHViS2V5LCBwbGFpblRleHQgfSkge1xuICAgIGxldCBwcml2S2V5ID0gYXdhaXQgZ2V0UHJpdktleSgpO1xuICAgIHJldHVybiBuaXAwNC5lbmNyeXB0KHByaXZLZXksIHB1YktleSwgcGxhaW5UZXh0KTtcbn1cblxuYXN5bmMgZnVuY3Rpb24gbmlwMDREZWNyeXB0KHsgcHViS2V5LCBjaXBoZXJUZXh0IH0pIHtcbiAgICBsZXQgcHJpdktleSA9IGF3YWl0IGdldFByaXZLZXkoKTtcbiAgICByZXR1cm4gbmlwMDQuZGVjcnlwdChwcml2S2V5LCBwdWJLZXksIGNpcGhlclRleHQpO1xufVxuXG5hc3luYyBmdW5jdGlvbiBuaXA0NEVuY3J5cHQoeyBwdWJLZXksIHBsYWluVGV4dCB9KSB7XG4gICAgbGV0IHByaXZLZXkgPSBhd2FpdCBnZXRQcml2S2V5KCk7XG4gICAgbGV0IGNvbnZlcnNhdGlvbktleSA9IG5pcDQ0LmdldENvbnZlcnNhdGlvbktleShwcml2S2V5LCBwdWJLZXkpXG4gICAgcmV0dXJuIG5pcDQ0LmVuY3J5cHQocGxhaW5UZXh0LCBjb252ZXJzYXRpb25LZXkpO1xufVxuXG5hc3luYyBmdW5jdGlvbiBuaXA0NERlY3J5cHQoeyBwdWJLZXksIGNpcGhlclRleHQgfSkge1xuICAgIGxldCBwcml2S2V5ID0gYXdhaXQgZ2V0UHJpdktleSgpO1xuICAgIGxldCBjb252ZXJzYXRpb25LZXkgPSBuaXA0NC5nZXRDb252ZXJzYXRpb25LZXkocHJpdktleSwgcHViS2V5KVxuICAgIHJldHVybiBuaXA0NC5kZWNyeXB0KGNpcGhlclRleHQsIGNvbnZlcnNhdGlvbktleSk7XG59XG5cbmFzeW5jIGZ1bmN0aW9uIGdldFJlbGF5cygpIHtcbiAgICBsZXQgcHJvZmlsZSA9IGF3YWl0IGN1cnJlbnRQcm9maWxlKCk7XG4gICAgbGV0IHJlbGF5cyA9IHByb2ZpbGUucmVsYXlzO1xuICAgIGxldCByZWxheU9iaiA9IHt9O1xuICAgIC8vIFRoZSBnZXRSZWxheXMgY2FsbCBleHBlY3RzIHRoaXMgdG8gYmUgcmV0dXJuZWQgYXMgYW4gb2JqZWN0LCBub3QgYXJyYXlcbiAgICByZWxheXMuZm9yRWFjaChyZWxheSA9PiB7XG4gICAgICAgIGxldCB7IHVybCwgcmVhZCwgd3JpdGUgfSA9IHJlbGF5O1xuICAgICAgICByZWxheU9ialt1cmxdID0geyByZWFkLCB3cml0ZSB9O1xuICAgIH0pO1xuICAgIHJldHVybiByZWxheU9iajtcbn1cbiJdLAogICJtYXBwaW5ncyI6ICI7Ozs7Ozs7O0FBQUEsV0FBUyxPQUFPLEdBQVM7QUFDdkIsUUFBSSxDQUFDLE9BQU8sY0FBYyxDQUFDLEtBQUssSUFBSTtBQUFHLFlBQU0sSUFBSSxNQUFNLDJCQUEyQixDQUFDLEVBQUU7RUFDdkY7QUFNQSxXQUFTLE1BQU0sTUFBOEIsU0FBaUI7QUFDNUQsUUFBSSxFQUFFLGFBQWE7QUFBYSxZQUFNLElBQUksTUFBTSxxQkFBcUI7QUFDckUsUUFBSSxRQUFRLFNBQVMsS0FBSyxDQUFDLFFBQVEsU0FBUyxFQUFFLE1BQU07QUFDbEQsWUFBTSxJQUFJLE1BQU0saUNBQWlDLE9BQU8sbUJBQW1CLEVBQUUsTUFBTSxFQUFFO0VBQ3pGO0FBUUEsV0FBUyxLQUFLQSxPQUFVO0FBQ3RCLFFBQUksT0FBT0EsVUFBUyxjQUFjLE9BQU9BLE1BQUssV0FBVztBQUN2RCxZQUFNLElBQUksTUFBTSxpREFBaUQ7QUFDbkUsV0FBT0EsTUFBSyxTQUFTO0FBQ3JCLFdBQU9BLE1BQUssUUFBUTtFQUN0QjtBQUVBLFdBQVMsT0FBTyxVQUFlLGdCQUFnQixNQUFJO0FBQ2pELFFBQUksU0FBUztBQUFXLFlBQU0sSUFBSSxNQUFNLGtDQUFrQztBQUMxRSxRQUFJLGlCQUFpQixTQUFTO0FBQVUsWUFBTSxJQUFJLE1BQU0sdUNBQXVDO0VBQ2pHO0FBQ0EsV0FBUyxPQUFPLEtBQVUsVUFBYTtBQUNyQyxVQUFNLEdBQUc7QUFDVCxVQUFNLE1BQU0sU0FBUztBQUNyQixRQUFJLElBQUksU0FBUyxLQUFLO0FBQ3BCLFlBQU0sSUFBSSxNQUFNLHlEQUF5RCxHQUFHLEVBQUU7O0VBRWxGOzs7QUNsQ08sTUFBTUMsVUFDWCxPQUFPLGVBQWUsWUFBWSxZQUFZLGFBQWEsV0FBVyxTQUFTOzs7QUNVakYsTUFBTSxNQUFNLENBQUMsTUFBNEIsYUFBYTtBQU8vQyxNQUFNLGFBQWEsQ0FBQyxRQUN6QixJQUFJLFNBQVMsSUFBSSxRQUFRLElBQUksWUFBWSxJQUFJLFVBQVU7QUFHbEQsTUFBTSxPQUFPLENBQUMsTUFBYyxVQUFtQixRQUFTLEtBQUssUUFBVyxTQUFTO0FBSWpGLE1BQU0sT0FBTyxJQUFJLFdBQVcsSUFBSSxZQUFZLENBQUMsU0FBVSxDQUFDLEVBQUUsTUFBTSxFQUFFLENBQUMsTUFBTTtBQUNoRixNQUFJLENBQUM7QUFBTSxVQUFNLElBQUksTUFBTSw2Q0FBNkM7QUE2RGxFLFdBQVUsWUFBWSxLQUFXO0FBQ3JDLFFBQUksT0FBTyxRQUFRO0FBQVUsWUFBTSxJQUFJLE1BQU0sb0NBQW9DLE9BQU8sR0FBRyxFQUFFO0FBQzdGLFdBQU8sSUFBSSxXQUFXLElBQUksWUFBVyxFQUFHLE9BQU8sR0FBRyxDQUFDO0VBQ3JEO0FBUU0sV0FBVSxRQUFRLE1BQVc7QUFDakMsUUFBSSxPQUFPLFNBQVM7QUFBVSxhQUFPLFlBQVksSUFBSTtBQUNyRCxRQUFJLENBQUMsSUFBSSxJQUFJO0FBQUcsWUFBTSxJQUFJLE1BQU0sNEJBQTRCLE9BQU8sSUFBSSxFQUFFO0FBQ3pFLFdBQU87RUFDVDtBQUtNLFdBQVUsZUFBZSxRQUFvQjtBQUNqRCxVQUFNLElBQUksSUFBSSxXQUFXLE9BQU8sT0FBTyxDQUFDLEtBQUssTUFBTSxNQUFNLEVBQUUsUUFBUSxDQUFDLENBQUM7QUFDckUsUUFBSUMsT0FBTTtBQUNWLFdBQU8sUUFBUSxDQUFDLE1BQUs7QUFDbkIsVUFBSSxDQUFDLElBQUksQ0FBQztBQUFHLGNBQU0sSUFBSSxNQUFNLHFCQUFxQjtBQUNsRCxRQUFFLElBQUksR0FBR0EsSUFBRztBQUNaLE1BQUFBLFFBQU8sRUFBRTtJQUNYLENBQUM7QUFDRCxXQUFPO0VBQ1Q7QUFHTSxNQUFnQixPQUFoQixNQUFvQjs7SUFzQnhCLFFBQUs7QUFDSCxhQUFPLEtBQUssV0FBVTtJQUN4Qjs7QUFjRixNQUFNLFFBQVEsQ0FBQSxFQUFHO0FBY1gsV0FBVSxnQkFBbUMsVUFBdUI7QUFDeEUsVUFBTSxRQUFRLENBQUMsUUFBMkIsU0FBUSxFQUFHLE9BQU8sUUFBUSxHQUFHLENBQUMsRUFBRSxPQUFNO0FBQ2hGLFVBQU0sTUFBTSxTQUFRO0FBQ3BCLFVBQU0sWUFBWSxJQUFJO0FBQ3RCLFVBQU0sV0FBVyxJQUFJO0FBQ3JCLFVBQU0sU0FBUyxNQUFNLFNBQVE7QUFDN0IsV0FBTztFQUNUO0FBMkJNLFdBQVUsWUFBWSxjQUFjLElBQUU7QUFDMUMsUUFBSUMsV0FBVSxPQUFPQSxRQUFPLG9CQUFvQixZQUFZO0FBQzFELGFBQU9BLFFBQU8sZ0JBQWdCLElBQUksV0FBVyxXQUFXLENBQUM7O0FBRTNELFVBQU0sSUFBSSxNQUFNLHdDQUF3QztFQUMxRDs7O0FDbE5BLFdBQVMsYUFBYSxNQUFnQixZQUFvQixPQUFlQyxPQUFhO0FBQ3BGLFFBQUksT0FBTyxLQUFLLGlCQUFpQjtBQUFZLGFBQU8sS0FBSyxhQUFhLFlBQVksT0FBT0EsS0FBSTtBQUM3RixVQUFNLE9BQU8sT0FBTyxFQUFFO0FBQ3RCLFVBQU0sV0FBVyxPQUFPLFVBQVU7QUFDbEMsVUFBTSxLQUFLLE9BQVEsU0FBUyxPQUFRLFFBQVE7QUFDNUMsVUFBTSxLQUFLLE9BQU8sUUFBUSxRQUFRO0FBQ2xDLFVBQU0sSUFBSUEsUUFBTyxJQUFJO0FBQ3JCLFVBQU0sSUFBSUEsUUFBTyxJQUFJO0FBQ3JCLFNBQUssVUFBVSxhQUFhLEdBQUcsSUFBSUEsS0FBSTtBQUN2QyxTQUFLLFVBQVUsYUFBYSxHQUFHLElBQUlBLEtBQUk7RUFDekM7QUFHTSxNQUFnQixPQUFoQixjQUFnRCxLQUFPO0lBYzNELFlBQ1csVUFDRixXQUNFLFdBQ0FBLE9BQWE7QUFFdEIsWUFBSztBQUxJLFdBQUEsV0FBQTtBQUNGLFdBQUEsWUFBQTtBQUNFLFdBQUEsWUFBQTtBQUNBLFdBQUEsT0FBQUE7QUFURCxXQUFBLFdBQVc7QUFDWCxXQUFBLFNBQVM7QUFDVCxXQUFBLE1BQU07QUFDTixXQUFBLFlBQVk7QUFTcEIsV0FBSyxTQUFTLElBQUksV0FBVyxRQUFRO0FBQ3JDLFdBQUssT0FBTyxXQUFXLEtBQUssTUFBTTtJQUNwQztJQUNBLE9BQU8sTUFBVztBQUNoQixhQUFPLElBQUk7QUFDWCxZQUFNLEVBQUUsTUFBTSxRQUFRLFNBQVEsSUFBSztBQUNuQyxhQUFPLFFBQVEsSUFBSTtBQUNuQixZQUFNLE1BQU0sS0FBSztBQUNqQixlQUFTLE1BQU0sR0FBRyxNQUFNLE9BQU87QUFDN0IsY0FBTSxPQUFPLEtBQUssSUFBSSxXQUFXLEtBQUssS0FBSyxNQUFNLEdBQUc7QUFFcEQsWUFBSSxTQUFTLFVBQVU7QUFDckIsZ0JBQU0sV0FBVyxXQUFXLElBQUk7QUFDaEMsaUJBQU8sWUFBWSxNQUFNLEtBQUssT0FBTztBQUFVLGlCQUFLLFFBQVEsVUFBVSxHQUFHO0FBQ3pFOztBQUVGLGVBQU8sSUFBSSxLQUFLLFNBQVMsS0FBSyxNQUFNLElBQUksR0FBRyxLQUFLLEdBQUc7QUFDbkQsYUFBSyxPQUFPO0FBQ1osZUFBTztBQUNQLFlBQUksS0FBSyxRQUFRLFVBQVU7QUFDekIsZUFBSyxRQUFRLE1BQU0sQ0FBQztBQUNwQixlQUFLLE1BQU07OztBQUdmLFdBQUssVUFBVSxLQUFLO0FBQ3BCLFdBQUssV0FBVTtBQUNmLGFBQU87SUFDVDtJQUNBLFdBQVcsS0FBZTtBQUN4QixhQUFPLElBQUk7QUFDWCxhQUFPLEtBQUssSUFBSTtBQUNoQixXQUFLLFdBQVc7QUFJaEIsWUFBTSxFQUFFLFFBQVEsTUFBTSxVQUFVLE1BQUFBLE1BQUksSUFBSztBQUN6QyxVQUFJLEVBQUUsSUFBRyxJQUFLO0FBRWQsYUFBTyxLQUFLLElBQUk7QUFDaEIsV0FBSyxPQUFPLFNBQVMsR0FBRyxFQUFFLEtBQUssQ0FBQztBQUVoQyxVQUFJLEtBQUssWUFBWSxXQUFXLEtBQUs7QUFDbkMsYUFBSyxRQUFRLE1BQU0sQ0FBQztBQUNwQixjQUFNOztBQUdSLGVBQVNDLEtBQUksS0FBS0EsS0FBSSxVQUFVQTtBQUFLLGVBQU9BLEVBQUMsSUFBSTtBQUlqRCxtQkFBYSxNQUFNLFdBQVcsR0FBRyxPQUFPLEtBQUssU0FBUyxDQUFDLEdBQUdELEtBQUk7QUFDOUQsV0FBSyxRQUFRLE1BQU0sQ0FBQztBQUNwQixZQUFNLFFBQVEsV0FBVyxHQUFHO0FBQzVCLFlBQU0sTUFBTSxLQUFLO0FBRWpCLFVBQUksTUFBTTtBQUFHLGNBQU0sSUFBSSxNQUFNLDZDQUE2QztBQUMxRSxZQUFNLFNBQVMsTUFBTTtBQUNyQixZQUFNLFFBQVEsS0FBSyxJQUFHO0FBQ3RCLFVBQUksU0FBUyxNQUFNO0FBQVEsY0FBTSxJQUFJLE1BQU0sb0NBQW9DO0FBQy9FLGVBQVNDLEtBQUksR0FBR0EsS0FBSSxRQUFRQTtBQUFLLGNBQU0sVUFBVSxJQUFJQSxJQUFHLE1BQU1BLEVBQUMsR0FBR0QsS0FBSTtJQUN4RTtJQUNBLFNBQU07QUFDSixZQUFNLEVBQUUsUUFBUSxVQUFTLElBQUs7QUFDOUIsV0FBSyxXQUFXLE1BQU07QUFDdEIsWUFBTSxNQUFNLE9BQU8sTUFBTSxHQUFHLFNBQVM7QUFDckMsV0FBSyxRQUFPO0FBQ1osYUFBTztJQUNUO0lBQ0EsV0FBVyxJQUFNO0FBQ2YsYUFBQSxLQUFPLElBQUssS0FBSyxZQUFtQjtBQUNwQyxTQUFHLElBQUksR0FBRyxLQUFLLElBQUcsQ0FBRTtBQUNwQixZQUFNLEVBQUUsVUFBVSxRQUFRLFFBQVEsVUFBVSxXQUFXLElBQUcsSUFBSztBQUMvRCxTQUFHLFNBQVM7QUFDWixTQUFHLE1BQU07QUFDVCxTQUFHLFdBQVc7QUFDZCxTQUFHLFlBQVk7QUFDZixVQUFJLFNBQVM7QUFBVSxXQUFHLE9BQU8sSUFBSSxNQUFNO0FBQzNDLGFBQU87SUFDVDs7OztBQzdHRixNQUFNLE1BQU0sQ0FBQyxHQUFXLEdBQVcsTUFBZSxJQUFJLElBQU0sQ0FBQyxJQUFJO0FBRWpFLE1BQU0sTUFBTSxDQUFDLEdBQVcsR0FBVyxNQUFlLElBQUksSUFBTSxJQUFJLElBQU0sSUFBSTtBQUsxRSxNQUFNLFdBQTBCLG9CQUFJLFlBQVk7SUFDOUM7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUNwRjtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQ3BGO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFDcEY7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUNwRjtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQ3BGO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFDcEY7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUNwRjtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0dBQ3JGO0FBSUQsTUFBTSxLQUFvQixvQkFBSSxZQUFZO0lBQ3hDO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7R0FDckY7QUFJRCxNQUFNLFdBQTJCLG9CQUFJLFlBQVksRUFBRTtBQUNuRCxNQUFNLFNBQU4sY0FBcUIsS0FBWTtJQVkvQixjQUFBO0FBQ0UsWUFBTSxJQUFJLElBQUksR0FBRyxLQUFLO0FBVnhCLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSSxHQUFHLENBQUMsSUFBSTtJQUlaO0lBQ1UsTUFBRztBQUNYLFlBQU0sRUFBRSxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEVBQUMsSUFBSztBQUNuQyxhQUFPLENBQUMsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxDQUFDO0lBQ2hDOztJQUVVLElBQ1IsR0FBVyxHQUFXLEdBQVcsR0FBVyxHQUFXLEdBQVcsR0FBVyxHQUFTO0FBRXRGLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7QUFDYixXQUFLLElBQUksSUFBSTtBQUNiLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7QUFDYixXQUFLLElBQUksSUFBSTtBQUNiLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7SUFDZjtJQUNVLFFBQVEsTUFBZ0IsUUFBYztBQUU5QyxlQUFTRSxLQUFJLEdBQUdBLEtBQUksSUFBSUEsTUFBSyxVQUFVO0FBQUcsaUJBQVNBLEVBQUMsSUFBSSxLQUFLLFVBQVUsUUFBUSxLQUFLO0FBQ3BGLGVBQVNBLEtBQUksSUFBSUEsS0FBSSxJQUFJQSxNQUFLO0FBQzVCLGNBQU0sTUFBTSxTQUFTQSxLQUFJLEVBQUU7QUFDM0IsY0FBTSxLQUFLLFNBQVNBLEtBQUksQ0FBQztBQUN6QixjQUFNLEtBQUssS0FBSyxLQUFLLENBQUMsSUFBSSxLQUFLLEtBQUssRUFBRSxJQUFLLFFBQVE7QUFDbkQsY0FBTSxLQUFLLEtBQUssSUFBSSxFQUFFLElBQUksS0FBSyxJQUFJLEVBQUUsSUFBSyxPQUFPO0FBQ2pELGlCQUFTQSxFQUFDLElBQUssS0FBSyxTQUFTQSxLQUFJLENBQUMsSUFBSSxLQUFLLFNBQVNBLEtBQUksRUFBRSxJQUFLOztBQUdqRSxVQUFJLEVBQUUsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxFQUFDLElBQUs7QUFDakMsZUFBU0EsS0FBSSxHQUFHQSxLQUFJLElBQUlBLE1BQUs7QUFDM0IsY0FBTSxTQUFTLEtBQUssR0FBRyxDQUFDLElBQUksS0FBSyxHQUFHLEVBQUUsSUFBSSxLQUFLLEdBQUcsRUFBRTtBQUNwRCxjQUFNLEtBQU0sSUFBSSxTQUFTLElBQUksR0FBRyxHQUFHLENBQUMsSUFBSSxTQUFTQSxFQUFDLElBQUksU0FBU0EsRUFBQyxJQUFLO0FBQ3JFLGNBQU0sU0FBUyxLQUFLLEdBQUcsQ0FBQyxJQUFJLEtBQUssR0FBRyxFQUFFLElBQUksS0FBSyxHQUFHLEVBQUU7QUFDcEQsY0FBTSxLQUFNLFNBQVMsSUFBSSxHQUFHLEdBQUcsQ0FBQyxJQUFLO0FBQ3JDLFlBQUk7QUFDSixZQUFJO0FBQ0osWUFBSTtBQUNKLFlBQUssSUFBSSxLQUFNO0FBQ2YsWUFBSTtBQUNKLFlBQUk7QUFDSixZQUFJO0FBQ0osWUFBSyxLQUFLLEtBQU07O0FBR2xCLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsVUFBSyxJQUFJLEtBQUssSUFBSztBQUNuQixVQUFLLElBQUksS0FBSyxJQUFLO0FBQ25CLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsVUFBSyxJQUFJLEtBQUssSUFBSztBQUNuQixVQUFLLElBQUksS0FBSyxJQUFLO0FBQ25CLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsVUFBSyxJQUFJLEtBQUssSUFBSztBQUNuQixXQUFLLElBQUksR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxDQUFDO0lBQ2pDO0lBQ1UsYUFBVTtBQUNsQixlQUFTLEtBQUssQ0FBQztJQUNqQjtJQUNBLFVBQU87QUFDTCxXQUFLLElBQUksR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxDQUFDO0FBQy9CLFdBQUssT0FBTyxLQUFLLENBQUM7SUFDcEI7O0FBc0JLLE1BQU0sU0FBeUIsZ0NBQWdCLE1BQU0sSUFBSSxPQUFNLENBQUU7OztBQ25JeEU7Ozs7Ozs7Ozt1QkFBQUM7SUFBQTs7Ozs7Ozs7O3VCQUFBQztJQUFBOztBQUtBLE1BQU0sTUFBTSxPQUFPLENBQUM7QUFDcEIsTUFBTSxNQUFNLE9BQU8sQ0FBQztBQUNwQixNQUFNLE1BQU0sT0FBTyxDQUFDO0FBQ3BCLE1BQU1DLE9BQU0sQ0FBQyxNQUE0QixhQUFhO0FBV3RELE1BQU0sUUFBd0Isc0JBQU0sS0FBSyxFQUFFLFFBQVEsSUFBRyxHQUFJLENBQUMsR0FBR0MsT0FDNURBLEdBQUUsU0FBUyxFQUFFLEVBQUUsU0FBUyxHQUFHLEdBQUcsQ0FBQztBQUszQixXQUFVLFdBQVdDLFFBQWlCO0FBQzFDLFFBQUksQ0FBQ0YsS0FBSUUsTUFBSztBQUFHLFlBQU0sSUFBSSxNQUFNLHFCQUFxQjtBQUV0RCxRQUFJQyxPQUFNO0FBQ1YsYUFBU0YsS0FBSSxHQUFHQSxLQUFJQyxPQUFNLFFBQVFELE1BQUs7QUFDckMsTUFBQUUsUUFBTyxNQUFNRCxPQUFNRCxFQUFDLENBQUM7O0FBRXZCLFdBQU9FO0VBQ1Q7QUFFTSxXQUFVLG9CQUFvQixLQUFvQjtBQUN0RCxVQUFNQSxPQUFNLElBQUksU0FBUyxFQUFFO0FBQzNCLFdBQU9BLEtBQUksU0FBUyxJQUFJLElBQUlBLElBQUcsS0FBS0E7RUFDdEM7QUFFTSxXQUFVLFlBQVlBLE1BQVc7QUFDckMsUUFBSSxPQUFPQSxTQUFRO0FBQVUsWUFBTSxJQUFJLE1BQU0sOEJBQThCLE9BQU9BLElBQUc7QUFFckYsV0FBTyxPQUFPQSxTQUFRLEtBQUssTUFBTSxLQUFLQSxJQUFHLEVBQUU7RUFDN0M7QUFLTSxXQUFVLFdBQVdBLE1BQVc7QUFDcEMsUUFBSSxPQUFPQSxTQUFRO0FBQVUsWUFBTSxJQUFJLE1BQU0sOEJBQThCLE9BQU9BLElBQUc7QUFDckYsVUFBTSxNQUFNQSxLQUFJO0FBQ2hCLFFBQUksTUFBTTtBQUFHLFlBQU0sSUFBSSxNQUFNLDREQUE0RCxHQUFHO0FBQzVGLFVBQU0sUUFBUSxJQUFJLFdBQVcsTUFBTSxDQUFDO0FBQ3BDLGFBQVNGLEtBQUksR0FBR0EsS0FBSSxNQUFNLFFBQVFBLE1BQUs7QUFDckMsWUFBTSxJQUFJQSxLQUFJO0FBQ2QsWUFBTSxVQUFVRSxLQUFJLE1BQU0sR0FBRyxJQUFJLENBQUM7QUFDbEMsWUFBTSxPQUFPLE9BQU8sU0FBUyxTQUFTLEVBQUU7QUFDeEMsVUFBSSxPQUFPLE1BQU0sSUFBSSxLQUFLLE9BQU87QUFBRyxjQUFNLElBQUksTUFBTSx1QkFBdUI7QUFDM0UsWUFBTUYsRUFBQyxJQUFJOztBQUViLFdBQU87RUFDVDtBQUdNLFdBQVUsZ0JBQWdCQyxRQUFpQjtBQUMvQyxXQUFPLFlBQVksV0FBV0EsTUFBSyxDQUFDO0VBQ3RDO0FBQ00sV0FBVSxnQkFBZ0JBLFFBQWlCO0FBQy9DLFFBQUksQ0FBQ0YsS0FBSUUsTUFBSztBQUFHLFlBQU0sSUFBSSxNQUFNLHFCQUFxQjtBQUN0RCxXQUFPLFlBQVksV0FBVyxXQUFXLEtBQUtBLE1BQUssRUFBRSxRQUFPLENBQUUsQ0FBQztFQUNqRTtBQUVNLFdBQVUsZ0JBQWdCLEdBQW9CLEtBQVc7QUFDN0QsV0FBTyxXQUFXLEVBQUUsU0FBUyxFQUFFLEVBQUUsU0FBUyxNQUFNLEdBQUcsR0FBRyxDQUFDO0VBQ3pEO0FBQ00sV0FBVSxnQkFBZ0IsR0FBb0IsS0FBVztBQUM3RCxXQUFPLGdCQUFnQixHQUFHLEdBQUcsRUFBRSxRQUFPO0VBQ3hDO0FBRU0sV0FBVSxtQkFBbUIsR0FBa0I7QUFDbkQsV0FBTyxXQUFXLG9CQUFvQixDQUFDLENBQUM7RUFDMUM7QUFXTSxXQUFVLFlBQVksT0FBZUMsTUFBVSxnQkFBdUI7QUFDMUUsUUFBSTtBQUNKLFFBQUksT0FBT0EsU0FBUSxVQUFVO0FBQzNCLFVBQUk7QUFDRixjQUFNLFdBQVdBLElBQUc7ZUFDYixHQUFHO0FBQ1YsY0FBTSxJQUFJLE1BQU0sR0FBRyxLQUFLLG1DQUFtQ0EsSUFBRyxhQUFhLENBQUMsRUFBRTs7ZUFFdkVILEtBQUlHLElBQUcsR0FBRztBQUduQixZQUFNLFdBQVcsS0FBS0EsSUFBRztXQUNwQjtBQUNMLFlBQU0sSUFBSSxNQUFNLEdBQUcsS0FBSyxtQ0FBbUM7O0FBRTdELFVBQU0sTUFBTSxJQUFJO0FBQ2hCLFFBQUksT0FBTyxtQkFBbUIsWUFBWSxRQUFRO0FBQ2hELFlBQU0sSUFBSSxNQUFNLEdBQUcsS0FBSyxhQUFhLGNBQWMsZUFBZSxHQUFHLEVBQUU7QUFDekUsV0FBTztFQUNUO0FBS00sV0FBVUwsZ0JBQWUsUUFBb0I7QUFDakQsVUFBTSxJQUFJLElBQUksV0FBVyxPQUFPLE9BQU8sQ0FBQyxLQUFLLE1BQU0sTUFBTSxFQUFFLFFBQVEsQ0FBQyxDQUFDO0FBQ3JFLFFBQUlNLE9BQU07QUFDVixXQUFPLFFBQVEsQ0FBQyxNQUFLO0FBQ25CLFVBQUksQ0FBQ0osS0FBSSxDQUFDO0FBQUcsY0FBTSxJQUFJLE1BQU0scUJBQXFCO0FBQ2xELFFBQUUsSUFBSSxHQUFHSSxJQUFHO0FBQ1osTUFBQUEsUUFBTyxFQUFFO0lBQ1gsQ0FBQztBQUNELFdBQU87RUFDVDtBQUVNLFdBQVUsV0FBVyxJQUFnQixJQUFjO0FBRXZELFFBQUksR0FBRyxXQUFXLEdBQUc7QUFBUSxhQUFPO0FBQ3BDLGFBQVNILEtBQUksR0FBR0EsS0FBSSxHQUFHLFFBQVFBO0FBQUssVUFBSSxHQUFHQSxFQUFDLE1BQU0sR0FBR0EsRUFBQztBQUFHLGVBQU87QUFDaEUsV0FBTztFQUNUO0FBU00sV0FBVUYsYUFBWSxLQUFXO0FBQ3JDLFFBQUksT0FBTyxRQUFRO0FBQVUsWUFBTSxJQUFJLE1BQU0sb0NBQW9DLE9BQU8sR0FBRyxFQUFFO0FBQzdGLFdBQU8sSUFBSSxXQUFXLElBQUksWUFBVyxFQUFHLE9BQU8sR0FBRyxDQUFDO0VBQ3JEO0FBUU0sV0FBVSxPQUFPLEdBQVM7QUFDOUIsUUFBSTtBQUNKLFNBQUssTUFBTSxHQUFHLElBQUksS0FBSyxNQUFNLEtBQUssT0FBTztBQUFFO0FBQzNDLFdBQU87RUFDVDtBQU9NLFdBQVUsT0FBTyxHQUFXLEtBQVc7QUFDM0MsV0FBUSxLQUFLLE9BQU8sR0FBRyxJQUFLO0VBQzlCO0FBS08sTUFBTSxTQUFTLENBQUMsR0FBVyxLQUFhLFVBQWtCO0FBQy9ELFdBQU8sS0FBTSxRQUFRLE1BQU0sUUFBUSxPQUFPLEdBQUc7RUFDL0M7QUFNTyxNQUFNLFVBQVUsQ0FBQyxPQUFlLE9BQU8sT0FBTyxJQUFJLENBQUMsS0FBSztBQUkvRCxNQUFNLE1BQU0sQ0FBQyxTQUFlLElBQUksV0FBVyxJQUFJO0FBQy9DLE1BQU0sT0FBTyxDQUFDLFFBQWEsV0FBVyxLQUFLLEdBQUc7QUFTeEMsV0FBVSxlQUNkLFNBQ0EsVUFDQSxRQUFrRTtBQUVsRSxRQUFJLE9BQU8sWUFBWSxZQUFZLFVBQVU7QUFBRyxZQUFNLElBQUksTUFBTSwwQkFBMEI7QUFDMUYsUUFBSSxPQUFPLGFBQWEsWUFBWSxXQUFXO0FBQUcsWUFBTSxJQUFJLE1BQU0sMkJBQTJCO0FBQzdGLFFBQUksT0FBTyxXQUFXO0FBQVksWUFBTSxJQUFJLE1BQU0sMkJBQTJCO0FBRTdFLFFBQUksSUFBSSxJQUFJLE9BQU87QUFDbkIsUUFBSSxJQUFJLElBQUksT0FBTztBQUNuQixRQUFJRSxLQUFJO0FBQ1IsVUFBTSxRQUFRLE1BQUs7QUFDakIsUUFBRSxLQUFLLENBQUM7QUFDUixRQUFFLEtBQUssQ0FBQztBQUNSLE1BQUFBLEtBQUk7SUFDTjtBQUNBLFVBQU0sSUFBSSxJQUFJLE1BQW9CLE9BQU8sR0FBRyxHQUFHLEdBQUcsQ0FBQztBQUNuRCxVQUFNLFNBQVMsQ0FBQyxPQUFPLElBQUcsTUFBTTtBQUU5QixVQUFJLEVBQUUsS0FBSyxDQUFDLENBQUksQ0FBQyxHQUFHLElBQUk7QUFDeEIsVUFBSSxFQUFDO0FBQ0wsVUFBSSxLQUFLLFdBQVc7QUFBRztBQUN2QixVQUFJLEVBQUUsS0FBSyxDQUFDLENBQUksQ0FBQyxHQUFHLElBQUk7QUFDeEIsVUFBSSxFQUFDO0lBQ1A7QUFDQSxVQUFNLE1BQU0sTUFBSztBQUVmLFVBQUlBLFFBQU87QUFBTSxjQUFNLElBQUksTUFBTSx5QkFBeUI7QUFDMUQsVUFBSSxNQUFNO0FBQ1YsWUFBTSxNQUFvQixDQUFBO0FBQzFCLGFBQU8sTUFBTSxVQUFVO0FBQ3JCLFlBQUksRUFBQztBQUNMLGNBQU0sS0FBSyxFQUFFLE1BQUs7QUFDbEIsWUFBSSxLQUFLLEVBQUU7QUFDWCxlQUFPLEVBQUU7O0FBRVgsYUFBT0gsYUFBWSxHQUFHLEdBQUc7SUFDM0I7QUFDQSxVQUFNLFdBQVcsQ0FBQyxNQUFrQixTQUFvQjtBQUN0RCxZQUFLO0FBQ0wsYUFBTyxJQUFJO0FBQ1gsVUFBSSxNQUFxQjtBQUN6QixhQUFPLEVBQUUsTUFBTSxLQUFLLElBQUcsQ0FBRTtBQUFJLGVBQU07QUFDbkMsWUFBSztBQUNMLGFBQU87SUFDVDtBQUNBLFdBQU87RUFDVDtBQUlBLE1BQU0sZUFBZTtJQUNuQixRQUFRLENBQUMsUUFBYSxPQUFPLFFBQVE7SUFDckMsVUFBVSxDQUFDLFFBQWEsT0FBTyxRQUFRO0lBQ3ZDLFNBQVMsQ0FBQyxRQUFhLE9BQU8sUUFBUTtJQUN0QyxRQUFRLENBQUMsUUFBYSxPQUFPLFFBQVE7SUFDckMsb0JBQW9CLENBQUMsUUFBYSxPQUFPLFFBQVEsWUFBWSxlQUFlO0lBQzVFLGVBQWUsQ0FBQyxRQUFhLE9BQU8sY0FBYyxHQUFHO0lBQ3JELE9BQU8sQ0FBQyxRQUFhLE1BQU0sUUFBUSxHQUFHO0lBQ3RDLE9BQU8sQ0FBQyxLQUFVLFdBQWlCLE9BQWUsR0FBRyxRQUFRLEdBQUc7SUFDaEUsTUFBTSxDQUFDLFFBQWEsT0FBTyxRQUFRLGNBQWMsT0FBTyxjQUFjLElBQUksU0FBUzs7QUFNL0UsV0FBVSxlQUNkLFFBQ0EsWUFDQSxnQkFBMkIsQ0FBQSxHQUFFO0FBRTdCLFVBQU0sYUFBYSxDQUFDLFdBQW9CLE1BQWlCLGVBQXVCO0FBQzlFLFlBQU0sV0FBVyxhQUFhLElBQUk7QUFDbEMsVUFBSSxPQUFPLGFBQWE7QUFDdEIsY0FBTSxJQUFJLE1BQU0sc0JBQXNCLElBQUksc0JBQXNCO0FBRWxFLFlBQU0sTUFBTSxPQUFPLFNBQWdDO0FBQ25ELFVBQUksY0FBYyxRQUFRO0FBQVc7QUFDckMsVUFBSSxDQUFDLFNBQVMsS0FBSyxNQUFNLEdBQUc7QUFDMUIsY0FBTSxJQUFJLE1BQ1IsaUJBQWlCLE9BQU8sU0FBUyxDQUFDLElBQUksR0FBRyxLQUFLLE9BQU8sR0FBRyxlQUFlLElBQUksRUFBRTs7SUFHbkY7QUFDQSxlQUFXLENBQUMsV0FBVyxJQUFJLEtBQUssT0FBTyxRQUFRLFVBQVU7QUFBRyxpQkFBVyxXQUFXLE1BQU8sS0FBSztBQUM5RixlQUFXLENBQUMsV0FBVyxJQUFJLEtBQUssT0FBTyxRQUFRLGFBQWE7QUFBRyxpQkFBVyxXQUFXLE1BQU8sSUFBSTtBQUNoRyxXQUFPO0VBQ1Q7OztBQzdRQSxNQUFNTyxPQUFNLE9BQU8sQ0FBQztBQUFwQixNQUF1QkMsT0FBTSxPQUFPLENBQUM7QUFBckMsTUFBd0NDLE9BQU0sT0FBTyxDQUFDO0FBQXRELE1BQXlELE1BQU0sT0FBTyxDQUFDO0FBRXZFLE1BQU0sTUFBTSxPQUFPLENBQUM7QUFBcEIsTUFBdUIsTUFBTSxPQUFPLENBQUM7QUFBckMsTUFBd0MsTUFBTSxPQUFPLENBQUM7QUFFdEQsTUFBTSxNQUFNLE9BQU8sQ0FBQztBQUFwQixNQUF1QixPQUFPLE9BQU8sRUFBRTtBQUdqQyxXQUFVLElBQUksR0FBVyxHQUFTO0FBQ3RDLFVBQU0sU0FBUyxJQUFJO0FBQ25CLFdBQU8sVUFBVUYsT0FBTSxTQUFTLElBQUk7RUFDdEM7QUFRTSxXQUFVLElBQUksS0FBYSxPQUFlLFFBQWM7QUFDNUQsUUFBSSxVQUFVQSxRQUFPLFFBQVFBO0FBQUssWUFBTSxJQUFJLE1BQU0sMkJBQTJCO0FBQzdFLFFBQUksV0FBV0M7QUFBSyxhQUFPRDtBQUMzQixRQUFJLE1BQU1DO0FBQ1YsV0FBTyxRQUFRRCxNQUFLO0FBQ2xCLFVBQUksUUFBUUM7QUFBSyxjQUFPLE1BQU0sTUFBTztBQUNyQyxZQUFPLE1BQU0sTUFBTztBQUNwQixnQkFBVUE7O0FBRVosV0FBTztFQUNUO0FBR00sV0FBVSxLQUFLLEdBQVcsT0FBZSxRQUFjO0FBQzNELFFBQUksTUFBTTtBQUNWLFdBQU8sVUFBVUQsTUFBSztBQUNwQixhQUFPO0FBQ1AsYUFBTzs7QUFFVCxXQUFPO0VBQ1Q7QUFHTSxXQUFVLE9BQU9HLFNBQWdCLFFBQWM7QUFDbkQsUUFBSUEsWUFBV0gsUUFBTyxVQUFVQSxNQUFLO0FBQ25DLFlBQU0sSUFBSSxNQUFNLDZDQUE2Q0csT0FBTSxRQUFRLE1BQU0sRUFBRTs7QUFJckYsUUFBSSxJQUFJLElBQUlBLFNBQVEsTUFBTTtBQUMxQixRQUFJLElBQUk7QUFFUixRQUFJLElBQUlILE1BQUssSUFBSUMsTUFBSyxJQUFJQSxNQUFLLElBQUlEO0FBQ25DLFdBQU8sTUFBTUEsTUFBSztBQUVoQixZQUFNLElBQUksSUFBSTtBQUNkLFlBQU0sSUFBSSxJQUFJO0FBQ2QsWUFBTSxJQUFJLElBQUksSUFBSTtBQUNsQixZQUFNLElBQUksSUFBSSxJQUFJO0FBRWxCLFVBQUksR0FBRyxJQUFJLEdBQUcsSUFBSSxHQUFHLElBQUksR0FBRyxJQUFJLEdBQUcsSUFBSTs7QUFFekMsVUFBTUksT0FBTTtBQUNaLFFBQUlBLFNBQVFIO0FBQUssWUFBTSxJQUFJLE1BQU0sd0JBQXdCO0FBQ3pELFdBQU8sSUFBSSxHQUFHLE1BQU07RUFDdEI7QUFVTSxXQUFVLGNBQWMsR0FBUztBQU1yQyxVQUFNLGFBQWEsSUFBSUEsUUFBT0M7QUFFOUIsUUFBSSxHQUFXLEdBQVc7QUFHMUIsU0FBSyxJQUFJLElBQUlELE1BQUssSUFBSSxHQUFHLElBQUlDLFNBQVFGLE1BQUssS0FBS0UsTUFBSztBQUFJO0FBR3hELFNBQUssSUFBSUEsTUFBSyxJQUFJLEtBQUssSUFBSSxHQUFHLFdBQVcsQ0FBQyxNQUFNLElBQUlELE1BQUs7QUFBSTtBQUc3RCxRQUFJLE1BQU0sR0FBRztBQUNYLFlBQU0sVUFBVSxJQUFJQSxRQUFPO0FBQzNCLGFBQU8sU0FBUyxZQUFlSSxLQUFlLEdBQUk7QUFDaEQsY0FBTSxPQUFPQSxJQUFHLElBQUksR0FBRyxNQUFNO0FBQzdCLFlBQUksQ0FBQ0EsSUFBRyxJQUFJQSxJQUFHLElBQUksSUFBSSxHQUFHLENBQUM7QUFBRyxnQkFBTSxJQUFJLE1BQU0seUJBQXlCO0FBQ3ZFLGVBQU87TUFDVDs7QUFJRixVQUFNLFVBQVUsSUFBSUosUUFBT0M7QUFDM0IsV0FBTyxTQUFTLFlBQWVHLEtBQWUsR0FBSTtBQUVoRCxVQUFJQSxJQUFHLElBQUksR0FBRyxTQUFTLE1BQU1BLElBQUcsSUFBSUEsSUFBRyxHQUFHO0FBQUcsY0FBTSxJQUFJLE1BQU0seUJBQXlCO0FBQ3RGLFVBQUksSUFBSTtBQUVSLFVBQUksSUFBSUEsSUFBRyxJQUFJQSxJQUFHLElBQUlBLElBQUcsS0FBSyxDQUFDLEdBQUcsQ0FBQztBQUNuQyxVQUFJLElBQUlBLElBQUcsSUFBSSxHQUFHLE1BQU07QUFDeEIsVUFBSSxJQUFJQSxJQUFHLElBQUksR0FBRyxDQUFDO0FBRW5CLGFBQU8sQ0FBQ0EsSUFBRyxJQUFJLEdBQUdBLElBQUcsR0FBRyxHQUFHO0FBQ3pCLFlBQUlBLElBQUcsSUFBSSxHQUFHQSxJQUFHLElBQUk7QUFBRyxpQkFBT0EsSUFBRztBQUVsQyxZQUFJLElBQUk7QUFDUixpQkFBUyxLQUFLQSxJQUFHLElBQUksQ0FBQyxHQUFHLElBQUksR0FBRyxLQUFLO0FBQ25DLGNBQUlBLElBQUcsSUFBSSxJQUFJQSxJQUFHLEdBQUc7QUFBRztBQUN4QixlQUFLQSxJQUFHLElBQUksRUFBRTs7QUFHaEIsY0FBTUMsTUFBS0QsSUFBRyxJQUFJLEdBQUdKLFFBQU8sT0FBTyxJQUFJLElBQUksQ0FBQyxDQUFDO0FBQzdDLFlBQUlJLElBQUcsSUFBSUMsR0FBRTtBQUNiLFlBQUlELElBQUcsSUFBSSxHQUFHQyxHQUFFO0FBQ2hCLFlBQUlELElBQUcsSUFBSSxHQUFHLENBQUM7QUFDZixZQUFJOztBQUVOLGFBQU87SUFDVDtFQUNGO0FBRU0sV0FBVSxPQUFPLEdBQVM7QUFNOUIsUUFBSSxJQUFJLFFBQVEsS0FBSztBQUtuQixZQUFNLFVBQVUsSUFBSUosUUFBTztBQUMzQixhQUFPLFNBQVMsVUFBYUksS0FBZSxHQUFJO0FBQzlDLGNBQU0sT0FBT0EsSUFBRyxJQUFJLEdBQUcsTUFBTTtBQUU3QixZQUFJLENBQUNBLElBQUcsSUFBSUEsSUFBRyxJQUFJLElBQUksR0FBRyxDQUFDO0FBQUcsZ0JBQU0sSUFBSSxNQUFNLHlCQUF5QjtBQUN2RSxlQUFPO01BQ1Q7O0FBSUYsUUFBSSxJQUFJLFFBQVEsS0FBSztBQUNuQixZQUFNLE1BQU0sSUFBSSxPQUFPO0FBQ3ZCLGFBQU8sU0FBUyxVQUFhQSxLQUFlLEdBQUk7QUFDOUMsY0FBTSxLQUFLQSxJQUFHLElBQUksR0FBR0gsSUFBRztBQUN4QixjQUFNLElBQUlHLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdkIsY0FBTSxLQUFLQSxJQUFHLElBQUksR0FBRyxDQUFDO0FBQ3RCLGNBQU1FLEtBQUlGLElBQUcsSUFBSUEsSUFBRyxJQUFJLElBQUlILElBQUcsR0FBRyxDQUFDO0FBQ25DLGNBQU0sT0FBT0csSUFBRyxJQUFJLElBQUlBLElBQUcsSUFBSUUsSUFBR0YsSUFBRyxHQUFHLENBQUM7QUFDekMsWUFBSSxDQUFDQSxJQUFHLElBQUlBLElBQUcsSUFBSSxJQUFJLEdBQUcsQ0FBQztBQUFHLGdCQUFNLElBQUksTUFBTSx5QkFBeUI7QUFDdkUsZUFBTztNQUNUOztBQUlGLFFBQUksSUFBSSxTQUFTLEtBQUs7O0FBdUJ0QixXQUFPLGNBQWMsQ0FBQztFQUN4QjtBQWdEQSxNQUFNLGVBQWU7SUFDbkI7SUFBVTtJQUFXO0lBQU87SUFBTztJQUFPO0lBQVE7SUFDbEQ7SUFBTztJQUFPO0lBQU87SUFBTztJQUFPO0lBQ25DO0lBQVE7SUFBUTtJQUFROztBQUVwQixXQUFVLGNBQWlCLE9BQWdCO0FBQy9DLFVBQU0sVUFBVTtNQUNkLE9BQU87TUFDUCxNQUFNO01BQ04sT0FBTztNQUNQLE1BQU07O0FBRVIsVUFBTSxPQUFPLGFBQWEsT0FBTyxDQUFDLEtBQUssUUFBZTtBQUNwRCxVQUFJLEdBQUcsSUFBSTtBQUNYLGFBQU87SUFDVCxHQUFHLE9BQU87QUFDVixXQUFPLGVBQWUsT0FBTyxJQUFJO0VBQ25DO0FBUU0sV0FBVSxNQUFTLEdBQWMsS0FBUSxPQUFhO0FBRzFELFFBQUksUUFBUUc7QUFBSyxZQUFNLElBQUksTUFBTSxvQkFBb0I7QUFDckQsUUFBSSxVQUFVQTtBQUFLLGFBQU8sRUFBRTtBQUM1QixRQUFJLFVBQVVDO0FBQUssYUFBTztBQUMxQixRQUFJLElBQUksRUFBRTtBQUNWLFFBQUksSUFBSTtBQUNSLFdBQU8sUUFBUUQsTUFBSztBQUNsQixVQUFJLFFBQVFDO0FBQUssWUFBSSxFQUFFLElBQUksR0FBRyxDQUFDO0FBQy9CLFVBQUksRUFBRSxJQUFJLENBQUM7QUFDWCxnQkFBVUE7O0FBRVosV0FBTztFQUNUO0FBTU0sV0FBVSxjQUFpQixHQUFjLE1BQVM7QUFDdEQsVUFBTSxNQUFNLElBQUksTUFBTSxLQUFLLE1BQU07QUFFakMsVUFBTSxpQkFBaUIsS0FBSyxPQUFPLENBQUMsS0FBSyxLQUFLQyxPQUFLO0FBQ2pELFVBQUksRUFBRSxJQUFJLEdBQUc7QUFBRyxlQUFPO0FBQ3ZCLFVBQUlBLEVBQUMsSUFBSTtBQUNULGFBQU8sRUFBRSxJQUFJLEtBQUssR0FBRztJQUN2QixHQUFHLEVBQUUsR0FBRztBQUVSLFVBQU0sV0FBVyxFQUFFLElBQUksY0FBYztBQUVyQyxTQUFLLFlBQVksQ0FBQyxLQUFLLEtBQUtBLE9BQUs7QUFDL0IsVUFBSSxFQUFFLElBQUksR0FBRztBQUFHLGVBQU87QUFDdkIsVUFBSUEsRUFBQyxJQUFJLEVBQUUsSUFBSSxLQUFLLElBQUlBLEVBQUMsQ0FBQztBQUMxQixhQUFPLEVBQUUsSUFBSSxLQUFLLEdBQUc7SUFDdkIsR0FBRyxRQUFRO0FBQ1gsV0FBTztFQUNUO0FBZ0JNLFdBQVUsUUFBUSxHQUFXLFlBQW1CO0FBRXBELFVBQU0sY0FBYyxlQUFlLFNBQVksYUFBYSxFQUFFLFNBQVMsQ0FBQyxFQUFFO0FBQzFFLFVBQU0sY0FBYyxLQUFLLEtBQUssY0FBYyxDQUFDO0FBQzdDLFdBQU8sRUFBRSxZQUFZLGFBQWEsWUFBVztFQUMvQztBQWVNLFdBQVUsTUFDZCxPQUNBQyxTQUNBQyxRQUFPLE9BQ1AsUUFBaUMsQ0FBQSxHQUFFO0FBRW5DLFFBQUksU0FBU0M7QUFBSyxZQUFNLElBQUksTUFBTSxpQ0FBaUMsS0FBSyxFQUFFO0FBQzFFLFVBQU0sRUFBRSxZQUFZLE1BQU0sYUFBYSxNQUFLLElBQUssUUFBUSxPQUFPRixPQUFNO0FBQ3RFLFFBQUksUUFBUTtBQUFNLFlBQU0sSUFBSSxNQUFNLGlEQUFpRDtBQUNuRixVQUFNLFFBQVEsT0FBTyxLQUFLO0FBQzFCLFVBQU0sSUFBdUIsT0FBTyxPQUFPO01BQ3pDO01BQ0E7TUFDQTtNQUNBLE1BQU0sUUFBUSxJQUFJO01BQ2xCLE1BQU1FO01BQ04sS0FBS0M7TUFDTCxRQUFRLENBQUMsUUFBUSxJQUFJLEtBQUssS0FBSztNQUMvQixTQUFTLENBQUMsUUFBTztBQUNmLFlBQUksT0FBTyxRQUFRO0FBQ2pCLGdCQUFNLElBQUksTUFBTSwrQ0FBK0MsT0FBTyxHQUFHLEVBQUU7QUFDN0UsZUFBT0QsUUFBTyxPQUFPLE1BQU07TUFDN0I7TUFDQSxLQUFLLENBQUMsUUFBUSxRQUFRQTtNQUN0QixPQUFPLENBQUMsU0FBUyxNQUFNQyxVQUFTQTtNQUNoQyxLQUFLLENBQUMsUUFBUSxJQUFJLENBQUMsS0FBSyxLQUFLO01BQzdCLEtBQUssQ0FBQyxLQUFLLFFBQVEsUUFBUTtNQUUzQixLQUFLLENBQUMsUUFBUSxJQUFJLE1BQU0sS0FBSyxLQUFLO01BQ2xDLEtBQUssQ0FBQyxLQUFLLFFBQVEsSUFBSSxNQUFNLEtBQUssS0FBSztNQUN2QyxLQUFLLENBQUMsS0FBSyxRQUFRLElBQUksTUFBTSxLQUFLLEtBQUs7TUFDdkMsS0FBSyxDQUFDLEtBQUssUUFBUSxJQUFJLE1BQU0sS0FBSyxLQUFLO01BQ3ZDLEtBQUssQ0FBQyxLQUFLLFVBQVUsTUFBTSxHQUFHLEtBQUssS0FBSztNQUN4QyxLQUFLLENBQUMsS0FBSyxRQUFRLElBQUksTUFBTSxPQUFPLEtBQUssS0FBSyxHQUFHLEtBQUs7O01BR3RELE1BQU0sQ0FBQyxRQUFRLE1BQU07TUFDckIsTUFBTSxDQUFDLEtBQUssUUFBUSxNQUFNO01BQzFCLE1BQU0sQ0FBQyxLQUFLLFFBQVEsTUFBTTtNQUMxQixNQUFNLENBQUMsS0FBSyxRQUFRLE1BQU07TUFFMUIsS0FBSyxDQUFDLFFBQVEsT0FBTyxLQUFLLEtBQUs7TUFDL0IsTUFBTSxNQUFNLFNBQVMsQ0FBQyxNQUFNLE1BQU0sR0FBRyxDQUFDO01BQ3RDLGFBQWEsQ0FBQyxRQUFRLGNBQWMsR0FBRyxHQUFHOzs7TUFHMUMsTUFBTSxDQUFDLEdBQUcsR0FBRyxNQUFPLElBQUksSUFBSTtNQUM1QixTQUFTLENBQUMsUUFBU0YsUUFBTyxnQkFBZ0IsS0FBSyxLQUFLLElBQUksZ0JBQWdCLEtBQUssS0FBSztNQUNsRixXQUFXLENBQUNHLFdBQVM7QUFDbkIsWUFBSUEsT0FBTSxXQUFXO0FBQ25CLGdCQUFNLElBQUksTUFBTSwwQkFBMEIsS0FBSyxTQUFTQSxPQUFNLE1BQU0sRUFBRTtBQUN4RSxlQUFPSCxRQUFPLGdCQUFnQkcsTUFBSyxJQUFJLGdCQUFnQkEsTUFBSztNQUM5RDtLQUNVO0FBQ1osV0FBTyxPQUFPLE9BQU8sQ0FBQztFQUN4QjtBQXdDTSxXQUFVLG9CQUFvQixZQUFrQjtBQUNwRCxRQUFJLE9BQU8sZUFBZTtBQUFVLFlBQU0sSUFBSSxNQUFNLDRCQUE0QjtBQUNoRixVQUFNLFlBQVksV0FBVyxTQUFTLENBQUMsRUFBRTtBQUN6QyxXQUFPLEtBQUssS0FBSyxZQUFZLENBQUM7RUFDaEM7QUFTTSxXQUFVLGlCQUFpQixZQUFrQjtBQUNqRCxVQUFNLFNBQVMsb0JBQW9CLFVBQVU7QUFDN0MsV0FBTyxTQUFTLEtBQUssS0FBSyxTQUFTLENBQUM7RUFDdEM7QUFlTSxXQUFVLGVBQWUsS0FBaUIsWUFBb0JDLFFBQU8sT0FBSztBQUM5RSxVQUFNLE1BQU0sSUFBSTtBQUNoQixVQUFNLFdBQVcsb0JBQW9CLFVBQVU7QUFDL0MsVUFBTSxTQUFTLGlCQUFpQixVQUFVO0FBRTFDLFFBQUksTUFBTSxNQUFNLE1BQU0sVUFBVSxNQUFNO0FBQ3BDLFlBQU0sSUFBSSxNQUFNLFlBQVksTUFBTSw2QkFBNkIsR0FBRyxFQUFFO0FBQ3RFLFVBQU0sTUFBTUEsUUFBTyxnQkFBZ0IsR0FBRyxJQUFJLGdCQUFnQixHQUFHO0FBRTdELFVBQU0sVUFBVSxJQUFJLEtBQUssYUFBYUMsSUFBRyxJQUFJQTtBQUM3QyxXQUFPRCxRQUFPLGdCQUFnQixTQUFTLFFBQVEsSUFBSSxnQkFBZ0IsU0FBUyxRQUFRO0VBQ3RGOzs7QUMvZEEsTUFBTUUsT0FBTSxPQUFPLENBQUM7QUFDcEIsTUFBTUMsT0FBTSxPQUFPLENBQUM7QUFpQ2QsV0FBVSxLQUF5QixHQUF3QixNQUFZO0FBQzNFLFVBQU0sa0JBQWtCLENBQUMsV0FBb0IsU0FBYztBQUN6RCxZQUFNLE1BQU0sS0FBSyxPQUFNO0FBQ3ZCLGFBQU8sWUFBWSxNQUFNO0lBQzNCO0FBQ0EsVUFBTSxPQUFPLENBQUMsTUFBYTtBQUN6QixZQUFNLFVBQVUsS0FBSyxLQUFLLE9BQU8sQ0FBQyxJQUFJO0FBQ3RDLFlBQU0sYUFBYSxNQUFNLElBQUk7QUFDN0IsYUFBTyxFQUFFLFNBQVMsV0FBVTtJQUM5QjtBQUNBLFdBQU87TUFDTDs7TUFFQSxhQUFhLEtBQVEsR0FBUztBQUM1QixZQUFJLElBQUksRUFBRTtBQUNWLFlBQUksSUFBTztBQUNYLGVBQU8sSUFBSUQsTUFBSztBQUNkLGNBQUksSUFBSUM7QUFBSyxnQkFBSSxFQUFFLElBQUksQ0FBQztBQUN4QixjQUFJLEVBQUUsT0FBTTtBQUNaLGdCQUFNQTs7QUFFUixlQUFPO01BQ1Q7Ozs7Ozs7Ozs7O01BWUEsaUJBQWlCLEtBQVEsR0FBUztBQUNoQyxjQUFNLEVBQUUsU0FBUyxXQUFVLElBQUssS0FBSyxDQUFDO0FBQ3RDLGNBQU0sU0FBYyxDQUFBO0FBQ3BCLFlBQUksSUFBTztBQUNYLFlBQUksT0FBTztBQUNYLGlCQUFTLFNBQVMsR0FBRyxTQUFTLFNBQVMsVUFBVTtBQUMvQyxpQkFBTztBQUNQLGlCQUFPLEtBQUssSUFBSTtBQUVoQixtQkFBU0MsS0FBSSxHQUFHQSxLQUFJLFlBQVlBLE1BQUs7QUFDbkMsbUJBQU8sS0FBSyxJQUFJLENBQUM7QUFDakIsbUJBQU8sS0FBSyxJQUFJOztBQUVsQixjQUFJLEtBQUssT0FBTTs7QUFFakIsZUFBTztNQUNUOzs7Ozs7OztNQVNBLEtBQUssR0FBVyxhQUFrQixHQUFTO0FBR3pDLGNBQU0sRUFBRSxTQUFTLFdBQVUsSUFBSyxLQUFLLENBQUM7QUFFdEMsWUFBSSxJQUFJLEVBQUU7QUFDVixZQUFJLElBQUksRUFBRTtBQUVWLGNBQU0sT0FBTyxPQUFPLEtBQUssSUFBSSxDQUFDO0FBQzlCLGNBQU0sWUFBWSxLQUFLO0FBQ3ZCLGNBQU0sVUFBVSxPQUFPLENBQUM7QUFFeEIsaUJBQVMsU0FBUyxHQUFHLFNBQVMsU0FBUyxVQUFVO0FBQy9DLGdCQUFNLFNBQVMsU0FBUztBQUV4QixjQUFJLFFBQVEsT0FBTyxJQUFJLElBQUk7QUFHM0IsZ0JBQU07QUFJTixjQUFJLFFBQVEsWUFBWTtBQUN0QixxQkFBUztBQUNULGlCQUFLRDs7QUFXUCxnQkFBTSxVQUFVO0FBQ2hCLGdCQUFNLFVBQVUsU0FBUyxLQUFLLElBQUksS0FBSyxJQUFJO0FBQzNDLGdCQUFNLFFBQVEsU0FBUyxNQUFNO0FBQzdCLGdCQUFNLFFBQVEsUUFBUTtBQUN0QixjQUFJLFVBQVUsR0FBRztBQUVmLGdCQUFJLEVBQUUsSUFBSSxnQkFBZ0IsT0FBTyxZQUFZLE9BQU8sQ0FBQyxDQUFDO2lCQUNqRDtBQUNMLGdCQUFJLEVBQUUsSUFBSSxnQkFBZ0IsT0FBTyxZQUFZLE9BQU8sQ0FBQyxDQUFDOzs7QUFRMUQsZUFBTyxFQUFFLEdBQUcsRUFBQztNQUNmO01BRUEsV0FBVyxHQUFNLGdCQUE2QixHQUFXLFdBQW9CO0FBRTNFLGNBQU0sSUFBWSxFQUFFLGdCQUFnQjtBQUVwQyxZQUFJLE9BQU8sZUFBZSxJQUFJLENBQUM7QUFDL0IsWUFBSSxDQUFDLE1BQU07QUFDVCxpQkFBTyxLQUFLLGlCQUFpQixHQUFHLENBQUM7QUFDakMsY0FBSSxNQUFNLEdBQUc7QUFDWCwyQkFBZSxJQUFJLEdBQUcsVUFBVSxJQUFJLENBQUM7OztBQUd6QyxlQUFPLEtBQUssS0FBSyxHQUFHLE1BQU0sQ0FBQztNQUM3Qjs7RUFFSjtBQWdCTSxXQUFVLGNBQXFCLE9BQXlCO0FBQzVELGtCQUFjLE1BQU0sRUFBRTtBQUN0QixtQkFDRSxPQUNBO01BQ0UsR0FBRztNQUNILEdBQUc7TUFDSCxJQUFJO01BQ0osSUFBSTtPQUVOO01BQ0UsWUFBWTtNQUNaLGFBQWE7S0FDZDtBQUdILFdBQU8sT0FBTyxPQUFPO01BQ25CLEdBQUcsUUFBUSxNQUFNLEdBQUcsTUFBTSxVQUFVO01BQ3BDLEdBQUc7TUFDSCxHQUFHLEVBQUUsR0FBRyxNQUFNLEdBQUcsTUFBSztLQUNkO0VBQ1o7OztBQ2hIQSxXQUFTLGtCQUFxQixPQUF5QjtBQUNyRCxVQUFNLE9BQU8sY0FBYyxLQUFLO0FBQ2hDLElBQUcsZUFDRCxNQUNBO01BQ0UsR0FBRztNQUNILEdBQUc7T0FFTDtNQUNFLDBCQUEwQjtNQUMxQixnQkFBZ0I7TUFDaEIsZUFBZTtNQUNmLGVBQWU7TUFDZixvQkFBb0I7TUFDcEIsV0FBVztNQUNYLFNBQVM7S0FDVjtBQUVILFVBQU0sRUFBRSxNQUFNLElBQUFFLEtBQUksRUFBQyxJQUFLO0FBQ3hCLFFBQUksTUFBTTtBQUNSLFVBQUksQ0FBQ0EsSUFBRyxJQUFJLEdBQUdBLElBQUcsSUFBSSxHQUFHO0FBQ3ZCLGNBQU0sSUFBSSxNQUFNLG1FQUFtRTs7QUFFckYsVUFDRSxPQUFPLFNBQVMsWUFDaEIsT0FBTyxLQUFLLFNBQVMsWUFDckIsT0FBTyxLQUFLLGdCQUFnQixZQUM1QjtBQUNBLGNBQU0sSUFBSSxNQUFNLG1FQUFtRTs7O0FBR3ZGLFdBQU8sT0FBTyxPQUFPLEVBQUUsR0FBRyxLQUFJLENBQVc7RUFDM0M7QUFVQSxNQUFNLEVBQUUsaUJBQWlCLEtBQUssWUFBWSxJQUFHLElBQUs7QUFDM0MsTUFBTSxNQUFNOztJQUVqQixLQUFLLE1BQU0sZUFBZSxNQUFLO01BQzdCLFlBQVksSUFBSSxJQUFFO0FBQ2hCLGNBQU0sQ0FBQztNQUNUOztJQUVGLFVBQVUsTUFBZ0I7QUFDeEIsWUFBTSxFQUFFLEtBQUssRUFBQyxJQUFLO0FBQ25CLFVBQUksS0FBSyxTQUFTLEtBQUssS0FBSyxDQUFDLE1BQU07QUFBTSxjQUFNLElBQUksRUFBRSwrQkFBK0I7QUFDcEYsWUFBTSxNQUFNLEtBQUssQ0FBQztBQUNsQixZQUFNLE1BQU0sS0FBSyxTQUFTLEdBQUcsTUFBTSxDQUFDO0FBQ3BDLFVBQUksQ0FBQyxPQUFPLElBQUksV0FBVztBQUFLLGNBQU0sSUFBSSxFQUFFLHlDQUF5QztBQUtyRixVQUFJLElBQUksQ0FBQyxJQUFJO0FBQVksY0FBTSxJQUFJLEVBQUUscUNBQXFDO0FBQzFFLFVBQUksSUFBSSxDQUFDLE1BQU0sS0FBUSxFQUFFLElBQUksQ0FBQyxJQUFJO0FBQ2hDLGNBQU0sSUFBSSxFQUFFLHFEQUFxRDtBQUNuRSxhQUFPLEVBQUUsR0FBRyxJQUFJLEdBQUcsR0FBRyxHQUFHLEtBQUssU0FBUyxNQUFNLENBQUMsRUFBQztJQUNqRDtJQUNBLE1BQU1DLE1BQXdCO0FBRTVCLFlBQU0sRUFBRSxLQUFLLEVBQUMsSUFBSztBQUNuQixZQUFNLE9BQU8sT0FBT0EsU0FBUSxXQUFXLElBQUlBLElBQUcsSUFBSUE7QUFDbEQsVUFBSSxFQUFFLGdCQUFnQjtBQUFhLGNBQU0sSUFBSSxNQUFNLGVBQWU7QUFDbEUsVUFBSSxJQUFJLEtBQUs7QUFDYixVQUFJLElBQUksS0FBSyxLQUFLLENBQUMsS0FBSztBQUFNLGNBQU0sSUFBSSxFQUFFLHVCQUF1QjtBQUNqRSxVQUFJLEtBQUssQ0FBQyxNQUFNLElBQUk7QUFBRyxjQUFNLElBQUksRUFBRSxxQ0FBcUM7QUFDeEUsWUFBTSxFQUFFLEdBQUcsR0FBRyxHQUFHLE9BQU0sSUFBSyxJQUFJLFVBQVUsS0FBSyxTQUFTLENBQUMsQ0FBQztBQUMxRCxZQUFNLEVBQUUsR0FBRyxHQUFHLEdBQUcsV0FBVSxJQUFLLElBQUksVUFBVSxNQUFNO0FBQ3BELFVBQUksV0FBVztBQUFRLGNBQU0sSUFBSSxFQUFFLDZDQUE2QztBQUNoRixhQUFPLEVBQUUsR0FBRyxFQUFDO0lBQ2Y7SUFDQSxXQUFXLEtBQTZCO0FBRXRDLFlBQU0sUUFBUSxDQUFDQyxPQUF1QixPQUFPLFNBQVNBLEdBQUUsQ0FBQyxHQUFHLEVBQUUsSUFBSSxJQUFTLE9BQU9BLEtBQUlBO0FBQ3RGLFlBQU0sSUFBSSxDQUFDLFFBQXdCO0FBQ2pDLGNBQU1ELE9BQU0sSUFBSSxTQUFTLEVBQUU7QUFDM0IsZUFBT0EsS0FBSSxTQUFTLElBQUksSUFBSUEsSUFBRyxLQUFLQTtNQUN0QztBQUNBLFlBQU0sSUFBSSxNQUFNLEVBQUUsSUFBSSxDQUFDLENBQUM7QUFDeEIsWUFBTSxJQUFJLE1BQU0sRUFBRSxJQUFJLENBQUMsQ0FBQztBQUN4QixZQUFNLE1BQU0sRUFBRSxTQUFTO0FBQ3ZCLFlBQU0sTUFBTSxFQUFFLFNBQVM7QUFDdkIsWUFBTSxLQUFLLEVBQUUsR0FBRztBQUNoQixZQUFNLEtBQUssRUFBRSxHQUFHO0FBQ2hCLGFBQU8sS0FBSyxFQUFFLE1BQU0sTUFBTSxDQUFDLENBQUMsS0FBSyxFQUFFLEdBQUcsQ0FBQyxLQUFLLEVBQUUsR0FBRyxDQUFDO0lBQ3BEOztBQUtGLE1BQU1FLE9BQU0sT0FBTyxDQUFDO0FBQXBCLE1BQXVCQyxPQUFNLE9BQU8sQ0FBQztBQUFyQyxNQUF3Q0MsT0FBTSxPQUFPLENBQUM7QUFBdEQsTUFBeURDLE9BQU0sT0FBTyxDQUFDO0FBQXZFLE1BQTBFQyxPQUFNLE9BQU8sQ0FBQztBQUVsRixXQUFVLGtCQUFxQixNQUF3QjtBQUMzRCxVQUFNLFFBQVEsa0JBQWtCLElBQUk7QUFDcEMsVUFBTSxFQUFFLElBQUFQLElBQUUsSUFBSztBQUVmLFVBQU1RLFdBQ0osTUFBTSxZQUNMLENBQUMsSUFBd0IsT0FBeUIsa0JBQTBCO0FBQzNFLFlBQU0sSUFBSSxNQUFNLFNBQVE7QUFDeEIsYUFBVUMsYUFBWSxXQUFXLEtBQUssQ0FBQyxDQUFJLENBQUMsR0FBR1QsSUFBRyxRQUFRLEVBQUUsQ0FBQyxHQUFHQSxJQUFHLFFBQVEsRUFBRSxDQUFDLENBQUM7SUFDakY7QUFDRixVQUFNLFlBQ0osTUFBTSxjQUNMLENBQUNVLFdBQXFCO0FBRXJCLFlBQU0sT0FBT0EsT0FBTSxTQUFTLENBQUM7QUFFN0IsWUFBTSxJQUFJVixJQUFHLFVBQVUsS0FBSyxTQUFTLEdBQUdBLElBQUcsS0FBSyxDQUFDO0FBQ2pELFlBQU0sSUFBSUEsSUFBRyxVQUFVLEtBQUssU0FBU0EsSUFBRyxPQUFPLElBQUlBLElBQUcsS0FBSyxDQUFDO0FBQzVELGFBQU8sRUFBRSxHQUFHLEVBQUM7SUFDZjtBQU1GLGFBQVMsb0JBQW9CLEdBQUk7QUFDL0IsWUFBTSxFQUFFLEdBQUcsRUFBQyxJQUFLO0FBQ2pCLFlBQU0sS0FBS0EsSUFBRyxJQUFJLENBQUM7QUFDbkIsWUFBTSxLQUFLQSxJQUFHLElBQUksSUFBSSxDQUFDO0FBQ3ZCLGFBQU9BLElBQUcsSUFBSUEsSUFBRyxJQUFJLElBQUlBLElBQUcsSUFBSSxHQUFHLENBQUMsQ0FBQyxHQUFHLENBQUM7SUFDM0M7QUFLQSxRQUFJLENBQUNBLElBQUcsSUFBSUEsSUFBRyxJQUFJLE1BQU0sRUFBRSxHQUFHLG9CQUFvQixNQUFNLEVBQUUsQ0FBQztBQUN6RCxZQUFNLElBQUksTUFBTSw2Q0FBNkM7QUFHL0QsYUFBUyxtQkFBbUIsS0FBVztBQUNyQyxhQUFPLE9BQU8sUUFBUSxZQUFZRyxPQUFNLE9BQU8sTUFBTSxNQUFNO0lBQzdEO0FBQ0EsYUFBUyxTQUFTLEtBQVc7QUFDM0IsVUFBSSxDQUFDLG1CQUFtQixHQUFHO0FBQUcsY0FBTSxJQUFJLE1BQU0sNkNBQTZDO0lBQzdGO0FBR0EsYUFBUyx1QkFBdUIsS0FBWTtBQUMxQyxZQUFNLEVBQUUsMEJBQTBCLFNBQVMsYUFBYSxnQkFBZ0IsRUFBQyxJQUFLO0FBQzlFLFVBQUksV0FBVyxPQUFPLFFBQVEsVUFBVTtBQUN0QyxZQUFJLGVBQWU7QUFBWSxnQkFBUyxXQUFXLEdBQUc7QUFFdEQsWUFBSSxPQUFPLFFBQVEsWUFBWSxDQUFDLFFBQVEsU0FBUyxJQUFJLE1BQU07QUFBRyxnQkFBTSxJQUFJLE1BQU0sYUFBYTtBQUMzRixjQUFNLElBQUksU0FBUyxjQUFjLEdBQUcsR0FBRzs7QUFFekMsVUFBSTtBQUNKLFVBQUk7QUFDRixjQUNFLE9BQU8sUUFBUSxXQUNYLE1BQ0csZ0JBQWdCLFlBQVksZUFBZSxLQUFLLFdBQVcsQ0FBQztlQUM5RCxPQUFPO0FBQ2QsY0FBTSxJQUFJLE1BQU0sdUJBQXVCLFdBQVcsOEJBQThCLE9BQU8sR0FBRyxFQUFFOztBQUU5RixVQUFJO0FBQWdCLGNBQVUsSUFBSSxLQUFLLENBQUM7QUFDeEMsZUFBUyxHQUFHO0FBQ1osYUFBTztJQUNUO0FBRUEsVUFBTSxtQkFBbUIsb0JBQUksSUFBRztBQUNoQyxhQUFTLGVBQWUsT0FBYztBQUNwQyxVQUFJLEVBQUUsaUJBQWlCUTtBQUFRLGNBQU0sSUFBSSxNQUFNLDBCQUEwQjtJQUMzRTtJQU1BLE1BQU1BLE9BQUs7TUFJVCxZQUFxQixJQUFnQixJQUFnQixJQUFLO0FBQXJDLGFBQUEsS0FBQTtBQUFnQixhQUFBLEtBQUE7QUFBZ0IsYUFBQSxLQUFBO0FBQ25ELFlBQUksTUFBTSxRQUFRLENBQUNYLElBQUcsUUFBUSxFQUFFO0FBQUcsZ0JBQU0sSUFBSSxNQUFNLFlBQVk7QUFDL0QsWUFBSSxNQUFNLFFBQVEsQ0FBQ0EsSUFBRyxRQUFRLEVBQUU7QUFBRyxnQkFBTSxJQUFJLE1BQU0sWUFBWTtBQUMvRCxZQUFJLE1BQU0sUUFBUSxDQUFDQSxJQUFHLFFBQVEsRUFBRTtBQUFHLGdCQUFNLElBQUksTUFBTSxZQUFZO01BQ2pFOzs7TUFJQSxPQUFPLFdBQVcsR0FBaUI7QUFDakMsY0FBTSxFQUFFLEdBQUcsRUFBQyxJQUFLLEtBQUssQ0FBQTtBQUN0QixZQUFJLENBQUMsS0FBSyxDQUFDQSxJQUFHLFFBQVEsQ0FBQyxLQUFLLENBQUNBLElBQUcsUUFBUSxDQUFDO0FBQUcsZ0JBQU0sSUFBSSxNQUFNLHNCQUFzQjtBQUNsRixZQUFJLGFBQWFXO0FBQU8sZ0JBQU0sSUFBSSxNQUFNLDhCQUE4QjtBQUN0RSxjQUFNLE1BQU0sQ0FBQ0MsT0FBU1osSUFBRyxJQUFJWSxJQUFHWixJQUFHLElBQUk7QUFFdkMsWUFBSSxJQUFJLENBQUMsS0FBSyxJQUFJLENBQUM7QUFBRyxpQkFBT1csT0FBTTtBQUNuQyxlQUFPLElBQUlBLE9BQU0sR0FBRyxHQUFHWCxJQUFHLEdBQUc7TUFDL0I7TUFFQSxJQUFJLElBQUM7QUFDSCxlQUFPLEtBQUssU0FBUSxFQUFHO01BQ3pCO01BQ0EsSUFBSSxJQUFDO0FBQ0gsZUFBTyxLQUFLLFNBQVEsRUFBRztNQUN6Qjs7Ozs7OztNQVFBLE9BQU8sV0FBVyxRQUFlO0FBQy9CLGNBQU0sUUFBUUEsSUFBRyxZQUFZLE9BQU8sSUFBSSxDQUFDLE1BQU0sRUFBRSxFQUFFLENBQUM7QUFDcEQsZUFBTyxPQUFPLElBQUksQ0FBQyxHQUFHWSxPQUFNLEVBQUUsU0FBUyxNQUFNQSxFQUFDLENBQUMsQ0FBQyxFQUFFLElBQUlELE9BQU0sVUFBVTtNQUN4RTs7Ozs7TUFNQSxPQUFPLFFBQVFWLE1BQVE7QUFDckIsY0FBTSxJQUFJVSxPQUFNLFdBQVcsVUFBVSxZQUFZLFlBQVlWLElBQUcsQ0FBQyxDQUFDO0FBQ2xFLFVBQUUsZUFBYztBQUNoQixlQUFPO01BQ1Q7O01BR0EsT0FBTyxlQUFlLFlBQW1CO0FBQ3ZDLGVBQU9VLE9BQU0sS0FBSyxTQUFTLHVCQUF1QixVQUFVLENBQUM7TUFDL0Q7O01BUUEsZUFBZSxZQUFrQjtBQUMvQixhQUFLLGVBQWU7QUFDcEIseUJBQWlCLE9BQU8sSUFBSTtNQUM5Qjs7TUFHQSxpQkFBYztBQUNaLFlBQUksS0FBSyxJQUFHLEdBQUk7QUFJZCxjQUFJLE1BQU0sc0JBQXNCLENBQUNYLElBQUcsSUFBSSxLQUFLLEVBQUU7QUFBRztBQUNsRCxnQkFBTSxJQUFJLE1BQU0saUJBQWlCOztBQUduQyxjQUFNLEVBQUUsR0FBRyxFQUFDLElBQUssS0FBSyxTQUFRO0FBRTlCLFlBQUksQ0FBQ0EsSUFBRyxRQUFRLENBQUMsS0FBSyxDQUFDQSxJQUFHLFFBQVEsQ0FBQztBQUFHLGdCQUFNLElBQUksTUFBTSwwQkFBMEI7QUFDaEYsY0FBTSxPQUFPQSxJQUFHLElBQUksQ0FBQztBQUNyQixjQUFNLFFBQVEsb0JBQW9CLENBQUM7QUFDbkMsWUFBSSxDQUFDQSxJQUFHLElBQUksTUFBTSxLQUFLO0FBQUcsZ0JBQU0sSUFBSSxNQUFNLG1DQUFtQztBQUM3RSxZQUFJLENBQUMsS0FBSyxjQUFhO0FBQUksZ0JBQU0sSUFBSSxNQUFNLHdDQUF3QztNQUNyRjtNQUNBLFdBQVE7QUFDTixjQUFNLEVBQUUsRUFBQyxJQUFLLEtBQUssU0FBUTtBQUMzQixZQUFJQSxJQUFHO0FBQU8saUJBQU8sQ0FBQ0EsSUFBRyxNQUFNLENBQUM7QUFDaEMsY0FBTSxJQUFJLE1BQU0sNkJBQTZCO01BQy9DOzs7O01BS0EsT0FBTyxPQUFZO0FBQ2pCLHVCQUFlLEtBQUs7QUFDcEIsY0FBTSxFQUFFLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUs7QUFDbkMsY0FBTSxFQUFFLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUs7QUFDbkMsY0FBTSxLQUFLQSxJQUFHLElBQUlBLElBQUcsSUFBSSxJQUFJLEVBQUUsR0FBR0EsSUFBRyxJQUFJLElBQUksRUFBRSxDQUFDO0FBQ2hELGNBQU0sS0FBS0EsSUFBRyxJQUFJQSxJQUFHLElBQUksSUFBSSxFQUFFLEdBQUdBLElBQUcsSUFBSSxJQUFJLEVBQUUsQ0FBQztBQUNoRCxlQUFPLE1BQU07TUFDZjs7OztNQUtBLFNBQU07QUFDSixlQUFPLElBQUlXLE9BQU0sS0FBSyxJQUFJWCxJQUFHLElBQUksS0FBSyxFQUFFLEdBQUcsS0FBSyxFQUFFO01BQ3BEOzs7OztNQU1BLFNBQU07QUFDSixjQUFNLEVBQUUsR0FBRyxFQUFDLElBQUs7QUFDakIsY0FBTSxLQUFLQSxJQUFHLElBQUksR0FBR00sSUFBRztBQUN4QixjQUFNLEVBQUUsSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSztBQUNuQyxZQUFJLEtBQUtOLElBQUcsTUFBTSxLQUFLQSxJQUFHLE1BQU0sS0FBS0EsSUFBRztBQUN4QyxZQUFJLEtBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdEIsWUFBSSxLQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ3RCLFlBQUksS0FBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUN0QixZQUFJLEtBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLEdBQUcsRUFBRTtBQUNqQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxHQUFHLEVBQUU7QUFDakIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksR0FBRyxFQUFFO0FBQ2pCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsZUFBTyxJQUFJVyxPQUFNLElBQUksSUFBSSxFQUFFO01BQzdCOzs7OztNQU1BLElBQUksT0FBWTtBQUNkLHVCQUFlLEtBQUs7QUFDcEIsY0FBTSxFQUFFLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUs7QUFDbkMsY0FBTSxFQUFFLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUs7QUFDbkMsWUFBSSxLQUFLWCxJQUFHLE1BQU0sS0FBS0EsSUFBRyxNQUFNLEtBQUtBLElBQUc7QUFDeEMsY0FBTSxJQUFJLE1BQU07QUFDaEIsY0FBTSxLQUFLQSxJQUFHLElBQUksTUFBTSxHQUFHTSxJQUFHO0FBQzlCLFlBQUksS0FBS04sSUFBRyxJQUFJLElBQUksRUFBRTtBQUN0QixZQUFJLEtBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdEIsWUFBSSxLQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ3RCLFlBQUksS0FBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUN0QixZQUFJLEtBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixZQUFJLEtBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDdEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxHQUFHLEVBQUU7QUFDakIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksR0FBRyxFQUFFO0FBQ2pCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxHQUFHLEVBQUU7QUFDakIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsYUFBS0EsSUFBRyxJQUFJLElBQUksRUFBRTtBQUNsQixhQUFLQSxJQUFHLElBQUksSUFBSSxFQUFFO0FBQ2xCLGFBQUtBLElBQUcsSUFBSSxJQUFJLEVBQUU7QUFDbEIsZUFBTyxJQUFJVyxPQUFNLElBQUksSUFBSSxFQUFFO01BQzdCO01BRUEsU0FBUyxPQUFZO0FBQ25CLGVBQU8sS0FBSyxJQUFJLE1BQU0sT0FBTSxDQUFFO01BQ2hDO01BRVEsTUFBRztBQUNULGVBQU8sS0FBSyxPQUFPQSxPQUFNLElBQUk7TUFDL0I7TUFDUSxLQUFLLEdBQVM7QUFDcEIsZUFBTyxLQUFLLFdBQVcsTUFBTSxrQkFBa0IsR0FBRyxDQUFDLFNBQWlCO0FBQ2xFLGdCQUFNLFFBQVFYLElBQUcsWUFBWSxLQUFLLElBQUksQ0FBQyxNQUFNLEVBQUUsRUFBRSxDQUFDO0FBQ2xELGlCQUFPLEtBQUssSUFBSSxDQUFDLEdBQUdZLE9BQU0sRUFBRSxTQUFTLE1BQU1BLEVBQUMsQ0FBQyxDQUFDLEVBQUUsSUFBSUQsT0FBTSxVQUFVO1FBQ3RFLENBQUM7TUFDSDs7Ozs7O01BT0EsZUFBZSxHQUFTO0FBQ3RCLGNBQU0sSUFBSUEsT0FBTTtBQUNoQixZQUFJLE1BQU1SO0FBQUssaUJBQU87QUFDdEIsaUJBQVMsQ0FBQztBQUNWLFlBQUksTUFBTUM7QUFBSyxpQkFBTztBQUN0QixjQUFNLEVBQUUsS0FBSSxJQUFLO0FBQ2pCLFlBQUksQ0FBQztBQUFNLGlCQUFPLEtBQUssYUFBYSxNQUFNLENBQUM7QUFHM0MsWUFBSSxFQUFFLE9BQU8sSUFBSSxPQUFPLEdBQUUsSUFBSyxLQUFLLFlBQVksQ0FBQztBQUNqRCxZQUFJLE1BQU07QUFDVixZQUFJLE1BQU07QUFDVixZQUFJLElBQVc7QUFDZixlQUFPLEtBQUtELFFBQU8sS0FBS0EsTUFBSztBQUMzQixjQUFJLEtBQUtDO0FBQUssa0JBQU0sSUFBSSxJQUFJLENBQUM7QUFDN0IsY0FBSSxLQUFLQTtBQUFLLGtCQUFNLElBQUksSUFBSSxDQUFDO0FBQzdCLGNBQUksRUFBRSxPQUFNO0FBQ1osaUJBQU9BO0FBQ1AsaUJBQU9BOztBQUVULFlBQUk7QUFBTyxnQkFBTSxJQUFJLE9BQU07QUFDM0IsWUFBSTtBQUFPLGdCQUFNLElBQUksT0FBTTtBQUMzQixjQUFNLElBQUlPLE9BQU1YLElBQUcsSUFBSSxJQUFJLElBQUksS0FBSyxJQUFJLEdBQUcsSUFBSSxJQUFJLElBQUksRUFBRTtBQUN6RCxlQUFPLElBQUksSUFBSSxHQUFHO01BQ3BCOzs7Ozs7Ozs7O01BV0EsU0FBUyxRQUFjO0FBQ3JCLGlCQUFTLE1BQU07QUFDZixZQUFJLElBQUk7QUFDUixZQUFJLE9BQWM7QUFDbEIsY0FBTSxFQUFFLEtBQUksSUFBSztBQUNqQixZQUFJLE1BQU07QUFDUixnQkFBTSxFQUFFLE9BQU8sSUFBSSxPQUFPLEdBQUUsSUFBSyxLQUFLLFlBQVksQ0FBQztBQUNuRCxjQUFJLEVBQUUsR0FBRyxLQUFLLEdBQUcsSUFBRyxJQUFLLEtBQUssS0FBSyxFQUFFO0FBQ3JDLGNBQUksRUFBRSxHQUFHLEtBQUssR0FBRyxJQUFHLElBQUssS0FBSyxLQUFLLEVBQUU7QUFDckMsZ0JBQU0sS0FBSyxnQkFBZ0IsT0FBTyxHQUFHO0FBQ3JDLGdCQUFNLEtBQUssZ0JBQWdCLE9BQU8sR0FBRztBQUNyQyxnQkFBTSxJQUFJVyxPQUFNWCxJQUFHLElBQUksSUFBSSxJQUFJLEtBQUssSUFBSSxHQUFHLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDekQsa0JBQVEsSUFBSSxJQUFJLEdBQUc7QUFDbkIsaUJBQU8sSUFBSSxJQUFJLEdBQUc7ZUFDYjtBQUNMLGdCQUFNLEVBQUUsR0FBRyxFQUFDLElBQUssS0FBSyxLQUFLLENBQUM7QUFDNUIsa0JBQVE7QUFDUixpQkFBTzs7QUFHVCxlQUFPVyxPQUFNLFdBQVcsQ0FBQyxPQUFPLElBQUksQ0FBQyxFQUFFLENBQUM7TUFDMUM7Ozs7Ozs7TUFRQSxxQkFBcUIsR0FBVSxHQUFXLEdBQVM7QUFDakQsY0FBTSxJQUFJQSxPQUFNO0FBQ2hCLGNBQU1FLE9BQU0sQ0FDVixHQUNBQyxPQUNJQSxPQUFNWCxRQUFPVyxPQUFNVixRQUFPLENBQUMsRUFBRSxPQUFPLENBQUMsSUFBSSxFQUFFLGVBQWVVLEVBQUMsSUFBSSxFQUFFLFNBQVNBLEVBQUM7QUFDakYsY0FBTSxNQUFNRCxLQUFJLE1BQU0sQ0FBQyxFQUFFLElBQUlBLEtBQUksR0FBRyxDQUFDLENBQUM7QUFDdEMsZUFBTyxJQUFJLElBQUcsSUFBSyxTQUFZO01BQ2pDOzs7O01BS0EsU0FBUyxJQUFNO0FBQ2IsY0FBTSxFQUFFLElBQUksR0FBRyxJQUFJLEdBQUcsSUFBSSxFQUFDLElBQUs7QUFDaEMsY0FBTSxNQUFNLEtBQUssSUFBRztBQUdwQixZQUFJLE1BQU07QUFBTSxlQUFLLE1BQU1iLElBQUcsTUFBTUEsSUFBRyxJQUFJLENBQUM7QUFDNUMsY0FBTSxLQUFLQSxJQUFHLElBQUksR0FBRyxFQUFFO0FBQ3ZCLGNBQU0sS0FBS0EsSUFBRyxJQUFJLEdBQUcsRUFBRTtBQUN2QixjQUFNLEtBQUtBLElBQUcsSUFBSSxHQUFHLEVBQUU7QUFDdkIsWUFBSTtBQUFLLGlCQUFPLEVBQUUsR0FBR0EsSUFBRyxNQUFNLEdBQUdBLElBQUcsS0FBSTtBQUN4QyxZQUFJLENBQUNBLElBQUcsSUFBSSxJQUFJQSxJQUFHLEdBQUc7QUFBRyxnQkFBTSxJQUFJLE1BQU0sa0JBQWtCO0FBQzNELGVBQU8sRUFBRSxHQUFHLElBQUksR0FBRyxHQUFFO01BQ3ZCO01BQ0EsZ0JBQWE7QUFDWCxjQUFNLEVBQUUsR0FBRyxVQUFVLGNBQWEsSUFBSztBQUN2QyxZQUFJLGFBQWFJO0FBQUssaUJBQU87QUFDN0IsWUFBSTtBQUFlLGlCQUFPLGNBQWNPLFFBQU8sSUFBSTtBQUNuRCxjQUFNLElBQUksTUFBTSw4REFBOEQ7TUFDaEY7TUFDQSxnQkFBYTtBQUNYLGNBQU0sRUFBRSxHQUFHLFVBQVUsY0FBYSxJQUFLO0FBQ3ZDLFlBQUksYUFBYVA7QUFBSyxpQkFBTztBQUM3QixZQUFJO0FBQWUsaUJBQU8sY0FBY08sUUFBTyxJQUFJO0FBQ25ELGVBQU8sS0FBSyxlQUFlLE1BQU0sQ0FBQztNQUNwQztNQUVBLFdBQVcsZUFBZSxNQUFJO0FBQzVCLGFBQUssZUFBYztBQUNuQixlQUFPSCxTQUFRRyxRQUFPLE1BQU0sWUFBWTtNQUMxQztNQUVBLE1BQU0sZUFBZSxNQUFJO0FBQ3ZCLGVBQVUsV0FBVyxLQUFLLFdBQVcsWUFBWSxDQUFDO01BQ3BEOztBQTlVZ0IsSUFBQUEsT0FBQSxPQUFPLElBQUlBLE9BQU0sTUFBTSxJQUFJLE1BQU0sSUFBSVgsSUFBRyxHQUFHO0FBQzNDLElBQUFXLE9BQUEsT0FBTyxJQUFJQSxPQUFNWCxJQUFHLE1BQU1BLElBQUcsS0FBS0EsSUFBRyxJQUFJO0FBK1UzRCxVQUFNLFFBQVEsTUFBTTtBQUNwQixVQUFNLE9BQU8sS0FBS1csUUFBTyxNQUFNLE9BQU8sS0FBSyxLQUFLLFFBQVEsQ0FBQyxJQUFJLEtBQUs7QUFFbEUsV0FBTztNQUNMO01BQ0EsaUJBQWlCQTtNQUNqQjtNQUNBO01BQ0E7O0VBRUo7QUF3Q0EsV0FBUyxhQUFhLE9BQWdCO0FBQ3BDLFVBQU0sT0FBTyxjQUFjLEtBQUs7QUFDaEMsSUFBRyxlQUNELE1BQ0E7TUFDRSxNQUFNO01BQ04sTUFBTTtNQUNOLGFBQWE7T0FFZjtNQUNFLFVBQVU7TUFDVixlQUFlO01BQ2YsTUFBTTtLQUNQO0FBRUgsV0FBTyxPQUFPLE9BQU8sRUFBRSxNQUFNLE1BQU0sR0FBRyxLQUFJLENBQVc7RUFDdkQ7QUFrQk0sV0FBVSxZQUFZLFVBQW1CO0FBQzdDLFVBQU0sUUFBUSxhQUFhLFFBQVE7QUFDbkMsVUFBTSxFQUFFLElBQUFYLEtBQUksR0FBRyxZQUFXLElBQUs7QUFDL0IsVUFBTSxnQkFBZ0JBLElBQUcsUUFBUTtBQUNqQyxVQUFNLGtCQUFrQixJQUFJQSxJQUFHLFFBQVE7QUFFdkMsYUFBUyxvQkFBb0IsS0FBVztBQUN0QyxhQUFPRyxPQUFNLE9BQU8sTUFBTUgsSUFBRztJQUMvQjtBQUNBLGFBQVNlLE1BQUssR0FBUztBQUNyQixhQUFXLElBQUksR0FBRyxXQUFXO0lBQy9CO0FBQ0EsYUFBUyxLQUFLLEdBQVM7QUFDckIsYUFBVyxPQUFPLEdBQUcsV0FBVztJQUNsQztBQUVBLFVBQU0sRUFDSixpQkFBaUJKLFFBQ2pCLHdCQUNBLHFCQUNBLG1CQUFrQixJQUNoQixrQkFBa0I7TUFDcEIsR0FBRztNQUNILFFBQVEsSUFBSSxPQUFPLGNBQXFCO0FBQ3RDLGNBQU0sSUFBSSxNQUFNLFNBQVE7QUFDeEIsY0FBTSxJQUFJWCxJQUFHLFFBQVEsRUFBRSxDQUFDO0FBQ3hCLGNBQU0sTUFBU1M7QUFDZixZQUFJLGNBQWM7QUFDaEIsaUJBQU8sSUFBSSxXQUFXLEtBQUssQ0FBQyxNQUFNLFNBQVEsSUFBSyxJQUFPLENBQUksQ0FBQyxHQUFHLENBQUM7ZUFDMUQ7QUFDTCxpQkFBTyxJQUFJLFdBQVcsS0FBSyxDQUFDLENBQUksQ0FBQyxHQUFHLEdBQUdULElBQUcsUUFBUSxFQUFFLENBQUMsQ0FBQzs7TUFFMUQ7TUFDQSxVQUFVVSxRQUFpQjtBQUN6QixjQUFNLE1BQU1BLE9BQU07QUFDbEIsY0FBTSxPQUFPQSxPQUFNLENBQUM7QUFDcEIsY0FBTSxPQUFPQSxPQUFNLFNBQVMsQ0FBQztBQUU3QixZQUFJLFFBQVEsa0JBQWtCLFNBQVMsS0FBUSxTQUFTLElBQU87QUFDN0QsZ0JBQU0sSUFBTyxnQkFBZ0IsSUFBSTtBQUNqQyxjQUFJLENBQUMsb0JBQW9CLENBQUM7QUFBRyxrQkFBTSxJQUFJLE1BQU0sdUJBQXVCO0FBQ3BFLGdCQUFNLEtBQUssb0JBQW9CLENBQUM7QUFDaEMsY0FBSSxJQUFJVixJQUFHLEtBQUssRUFBRTtBQUNsQixnQkFBTSxVQUFVLElBQUlJLFVBQVNBO0FBRTdCLGdCQUFNLGFBQWEsT0FBTyxPQUFPO0FBQ2pDLGNBQUksY0FBYztBQUFRLGdCQUFJSixJQUFHLElBQUksQ0FBQztBQUN0QyxpQkFBTyxFQUFFLEdBQUcsRUFBQzttQkFDSixRQUFRLG1CQUFtQixTQUFTLEdBQU07QUFDbkQsZ0JBQU0sSUFBSUEsSUFBRyxVQUFVLEtBQUssU0FBUyxHQUFHQSxJQUFHLEtBQUssQ0FBQztBQUNqRCxnQkFBTSxJQUFJQSxJQUFHLFVBQVUsS0FBSyxTQUFTQSxJQUFHLE9BQU8sSUFBSUEsSUFBRyxLQUFLLENBQUM7QUFDNUQsaUJBQU8sRUFBRSxHQUFHLEVBQUM7ZUFDUjtBQUNMLGdCQUFNLElBQUksTUFDUixtQkFBbUIsR0FBRywwQkFBMEIsYUFBYSx3QkFBd0IsZUFBZSxxQkFBcUI7O01BRy9IO0tBQ0Q7QUFDRCxVQUFNLGdCQUFnQixDQUFDLFFBQ2xCLFdBQWMsZ0JBQWdCLEtBQUssTUFBTSxXQUFXLENBQUM7QUFFMUQsYUFBUyxzQkFBc0JnQixTQUFjO0FBQzNDLFlBQU0sT0FBTyxlQUFlWjtBQUM1QixhQUFPWSxVQUFTO0lBQ2xCO0FBRUEsYUFBUyxXQUFXLEdBQVM7QUFDM0IsYUFBTyxzQkFBc0IsQ0FBQyxJQUFJRCxNQUFLLENBQUMsQ0FBQyxJQUFJO0lBQy9DO0FBRUEsVUFBTSxTQUFTLENBQUMsR0FBZSxNQUFjLE9BQWtCLGdCQUFnQixFQUFFLE1BQU0sTUFBTSxFQUFFLENBQUM7SUFLaEcsTUFBTSxVQUFTO01BQ2IsWUFBcUIsR0FBb0IsR0FBb0IsVUFBaUI7QUFBekQsYUFBQSxJQUFBO0FBQW9CLGFBQUEsSUFBQTtBQUFvQixhQUFBLFdBQUE7QUFDM0QsYUFBSyxlQUFjO01BQ3JCOztNQUdBLE9BQU8sWUFBWWQsTUFBUTtBQUN6QixjQUFNLElBQUksTUFBTTtBQUNoQixRQUFBQSxPQUFNLFlBQVksb0JBQW9CQSxNQUFLLElBQUksQ0FBQztBQUNoRCxlQUFPLElBQUksVUFBVSxPQUFPQSxNQUFLLEdBQUcsQ0FBQyxHQUFHLE9BQU9BLE1BQUssR0FBRyxJQUFJLENBQUMsQ0FBQztNQUMvRDs7O01BSUEsT0FBTyxRQUFRQSxNQUFRO0FBQ3JCLGNBQU0sRUFBRSxHQUFHLEVBQUMsSUFBSyxJQUFJLE1BQU0sWUFBWSxPQUFPQSxJQUFHLENBQUM7QUFDbEQsZUFBTyxJQUFJLFVBQVUsR0FBRyxDQUFDO01BQzNCO01BRUEsaUJBQWM7QUFFWixZQUFJLENBQUMsbUJBQW1CLEtBQUssQ0FBQztBQUFHLGdCQUFNLElBQUksTUFBTSwyQkFBMkI7QUFDNUUsWUFBSSxDQUFDLG1CQUFtQixLQUFLLENBQUM7QUFBRyxnQkFBTSxJQUFJLE1BQU0sMkJBQTJCO01BQzlFO01BRUEsZUFBZSxVQUFnQjtBQUM3QixlQUFPLElBQUksVUFBVSxLQUFLLEdBQUcsS0FBSyxHQUFHLFFBQVE7TUFDL0M7TUFFQSxpQkFBaUIsU0FBWTtBQUMzQixjQUFNLEVBQUUsR0FBRyxHQUFHLFVBQVUsSUFBRyxJQUFLO0FBQ2hDLGNBQU0sSUFBSSxjQUFjLFlBQVksV0FBVyxPQUFPLENBQUM7QUFDdkQsWUFBSSxPQUFPLFFBQVEsQ0FBQyxDQUFDLEdBQUcsR0FBRyxHQUFHLENBQUMsRUFBRSxTQUFTLEdBQUc7QUFBRyxnQkFBTSxJQUFJLE1BQU0scUJBQXFCO0FBQ3JGLGNBQU0sT0FBTyxRQUFRLEtBQUssUUFBUSxJQUFJLElBQUksTUFBTSxJQUFJO0FBQ3BELFlBQUksUUFBUUQsSUFBRztBQUFPLGdCQUFNLElBQUksTUFBTSw0QkFBNEI7QUFDbEUsY0FBTSxVQUFVLE1BQU0sT0FBTyxJQUFJLE9BQU87QUFDeEMsY0FBTSxJQUFJVyxPQUFNLFFBQVEsU0FBUyxjQUFjLElBQUksQ0FBQztBQUNwRCxjQUFNLEtBQUssS0FBSyxJQUFJO0FBQ3BCLGNBQU0sS0FBS0ksTUFBSyxDQUFDLElBQUksRUFBRTtBQUN2QixjQUFNLEtBQUtBLE1BQUssSUFBSSxFQUFFO0FBQ3RCLGNBQU0sSUFBSUosT0FBTSxLQUFLLHFCQUFxQixHQUFHLElBQUksRUFBRTtBQUNuRCxZQUFJLENBQUM7QUFBRyxnQkFBTSxJQUFJLE1BQU0sbUJBQW1CO0FBQzNDLFVBQUUsZUFBYztBQUNoQixlQUFPO01BQ1Q7O01BR0EsV0FBUTtBQUNOLGVBQU8sc0JBQXNCLEtBQUssQ0FBQztNQUNyQztNQUVBLGFBQVU7QUFDUixlQUFPLEtBQUssU0FBUSxJQUFLLElBQUksVUFBVSxLQUFLLEdBQUdJLE1BQUssQ0FBQyxLQUFLLENBQUMsR0FBRyxLQUFLLFFBQVEsSUFBSTtNQUNqRjs7TUFHQSxnQkFBYTtBQUNYLGVBQVUsV0FBVyxLQUFLLFNBQVEsQ0FBRTtNQUN0QztNQUNBLFdBQVE7QUFDTixlQUFPLElBQUksV0FBVyxFQUFFLEdBQUcsS0FBSyxHQUFHLEdBQUcsS0FBSyxFQUFDLENBQUU7TUFDaEQ7O01BR0Esb0JBQWlCO0FBQ2YsZUFBVSxXQUFXLEtBQUssYUFBWSxDQUFFO01BQzFDO01BQ0EsZUFBWTtBQUNWLGVBQU8sY0FBYyxLQUFLLENBQUMsSUFBSSxjQUFjLEtBQUssQ0FBQztNQUNyRDs7QUFJRixVQUFNLFFBQVE7TUFDWixrQkFBa0IsWUFBbUI7QUFDbkMsWUFBSTtBQUNGLGlDQUF1QixVQUFVO0FBQ2pDLGlCQUFPO2lCQUNBLE9BQU87QUFDZCxpQkFBTzs7TUFFWDtNQUNBOzs7OztNQU1BLGtCQUFrQixNQUFpQjtBQUNqQyxjQUFNLFNBQWEsaUJBQWlCLE1BQU0sQ0FBQztBQUMzQyxlQUFXLGVBQWUsTUFBTSxZQUFZLE1BQU0sR0FBRyxNQUFNLENBQUM7TUFDOUQ7Ozs7Ozs7OztNQVVBLFdBQVcsYUFBYSxHQUFHLFFBQVFKLE9BQU0sTUFBSTtBQUMzQyxjQUFNLGVBQWUsVUFBVTtBQUMvQixjQUFNLFNBQVMsT0FBTyxDQUFDLENBQUM7QUFDeEIsZUFBTztNQUNUOztBQVNGLGFBQVNNLGNBQWEsWUFBcUIsZUFBZSxNQUFJO0FBQzVELGFBQU9OLE9BQU0sZUFBZSxVQUFVLEVBQUUsV0FBVyxZQUFZO0lBQ2pFO0FBS0EsYUFBUyxVQUFVLE1BQXNCO0FBQ3ZDLFlBQU0sTUFBTSxnQkFBZ0I7QUFDNUIsWUFBTSxNQUFNLE9BQU8sU0FBUztBQUM1QixZQUFNLE9BQU8sT0FBTyxRQUFTLEtBQWE7QUFDMUMsVUFBSTtBQUFLLGVBQU8sUUFBUSxpQkFBaUIsUUFBUTtBQUNqRCxVQUFJO0FBQUssZUFBTyxRQUFRLElBQUksaUJBQWlCLFFBQVEsSUFBSTtBQUN6RCxVQUFJLGdCQUFnQkE7QUFBTyxlQUFPO0FBQ2xDLGFBQU87SUFDVDtBQVlBLGFBQVMsZ0JBQWdCLFVBQW1CLFNBQWMsZUFBZSxNQUFJO0FBQzNFLFVBQUksVUFBVSxRQUFRO0FBQUcsY0FBTSxJQUFJLE1BQU0sK0JBQStCO0FBQ3hFLFVBQUksQ0FBQyxVQUFVLE9BQU87QUFBRyxjQUFNLElBQUksTUFBTSwrQkFBK0I7QUFDeEUsWUFBTSxJQUFJQSxPQUFNLFFBQVEsT0FBTztBQUMvQixhQUFPLEVBQUUsU0FBUyx1QkFBdUIsUUFBUSxDQUFDLEVBQUUsV0FBVyxZQUFZO0lBQzdFO0FBTUEsVUFBTSxXQUNKLE1BQU0sWUFDTixTQUFVRCxRQUFpQjtBQUd6QixZQUFNLE1BQVMsZ0JBQWdCQSxNQUFLO0FBQ3BDLFlBQU0sUUFBUUEsT0FBTSxTQUFTLElBQUksTUFBTTtBQUN2QyxhQUFPLFFBQVEsSUFBSSxPQUFPLE9BQU8sS0FBSyxJQUFJO0lBQzVDO0FBQ0YsVUFBTSxnQkFDSixNQUFNLGlCQUNOLFNBQVVBLFFBQWlCO0FBQ3pCLGFBQU9LLE1BQUssU0FBU0wsTUFBSyxDQUFDO0lBQzdCO0FBRUYsVUFBTSxhQUFnQixRQUFRLE1BQU0sVUFBVTtBQUk5QyxhQUFTLFdBQVcsS0FBVztBQUM3QixVQUFJLE9BQU8sUUFBUTtBQUFVLGNBQU0sSUFBSSxNQUFNLGlCQUFpQjtBQUM5RCxVQUFJLEVBQUVQLFFBQU8sT0FBTyxNQUFNO0FBQ3hCLGNBQU0sSUFBSSxNQUFNLHVCQUF1QixNQUFNLFVBQVUsRUFBRTtBQUUzRCxhQUFVLGdCQUFnQixLQUFLLE1BQU0sV0FBVztJQUNsRDtBQU9BLGFBQVMsUUFBUSxTQUFjLFlBQXFCLE9BQU8sZ0JBQWM7QUFDdkUsVUFBSSxDQUFDLGFBQWEsV0FBVyxFQUFFLEtBQUssQ0FBQyxNQUFNLEtBQUssSUFBSTtBQUNsRCxjQUFNLElBQUksTUFBTSxxQ0FBcUM7QUFDdkQsWUFBTSxFQUFFLE1BQUFlLE9BQU0sYUFBQUMsYUFBVyxJQUFLO0FBQzlCLFVBQUksRUFBRSxNQUFNLFNBQVMsY0FBYyxJQUFHLElBQUs7QUFDM0MsVUFBSSxRQUFRO0FBQU0sZUFBTztBQUN6QixnQkFBVSxZQUFZLFdBQVcsT0FBTztBQUN4QyxVQUFJO0FBQVMsa0JBQVUsWUFBWSxxQkFBcUJELE1BQUssT0FBTyxDQUFDO0FBS3JFLFlBQU0sUUFBUSxjQUFjLE9BQU87QUFDbkMsWUFBTSxJQUFJLHVCQUF1QixVQUFVO0FBQzNDLFlBQU0sV0FBVyxDQUFDLFdBQVcsQ0FBQyxHQUFHLFdBQVcsS0FBSyxDQUFDO0FBRWxELFVBQUksT0FBTyxNQUFNO0FBRWYsY0FBTSxJQUFJLFFBQVEsT0FBT0MsYUFBWW5CLElBQUcsS0FBSyxJQUFJO0FBQ2pELGlCQUFTLEtBQUssWUFBWSxnQkFBZ0IsQ0FBQyxDQUFDOztBQUU5QyxZQUFNLE9BQVVTLGFBQVksR0FBRyxRQUFRO0FBQ3ZDLFlBQU0sSUFBSTtBQUVWLGVBQVMsTUFBTSxRQUFrQjtBQUUvQixjQUFNLElBQUksU0FBUyxNQUFNO0FBQ3pCLFlBQUksQ0FBQyxtQkFBbUIsQ0FBQztBQUFHO0FBQzVCLGNBQU0sS0FBSyxLQUFLLENBQUM7QUFDakIsY0FBTSxJQUFJRSxPQUFNLEtBQUssU0FBUyxDQUFDLEVBQUUsU0FBUTtBQUN6QyxjQUFNLElBQUlJLE1BQUssRUFBRSxDQUFDO0FBQ2xCLFlBQUksTUFBTVo7QUFBSztBQUlmLGNBQU0sSUFBSVksTUFBSyxLQUFLQSxNQUFLLElBQUksSUFBSSxDQUFDLENBQUM7QUFDbkMsWUFBSSxNQUFNWjtBQUFLO0FBQ2YsWUFBSSxZQUFZLEVBQUUsTUFBTSxJQUFJLElBQUksS0FBSyxPQUFPLEVBQUUsSUFBSUMsSUFBRztBQUNyRCxZQUFJLFFBQVE7QUFDWixZQUFJLFFBQVEsc0JBQXNCLENBQUMsR0FBRztBQUNwQyxrQkFBUSxXQUFXLENBQUM7QUFDcEIsc0JBQVk7O0FBRWQsZUFBTyxJQUFJLFVBQVUsR0FBRyxPQUFPLFFBQVE7TUFDekM7QUFDQSxhQUFPLEVBQUUsTUFBTSxNQUFLO0lBQ3RCO0FBQ0EsVUFBTSxpQkFBMkIsRUFBRSxNQUFNLE1BQU0sTUFBTSxTQUFTLE1BQUs7QUFDbkUsVUFBTSxpQkFBMEIsRUFBRSxNQUFNLE1BQU0sTUFBTSxTQUFTLE1BQUs7QUFlbEUsYUFBUyxLQUFLLFNBQWMsU0FBa0IsT0FBTyxnQkFBYztBQUNqRSxZQUFNLEVBQUUsTUFBTSxNQUFLLElBQUssUUFBUSxTQUFTLFNBQVMsSUFBSTtBQUN0RCxZQUFNLElBQUk7QUFDVixZQUFNLE9BQVUsZUFBbUMsRUFBRSxLQUFLLFdBQVcsRUFBRSxhQUFhLEVBQUUsSUFBSTtBQUMxRixhQUFPLEtBQUssTUFBTSxLQUFLO0lBQ3pCO0FBR0EsSUFBQU8sT0FBTSxLQUFLLGVBQWUsQ0FBQztBQWdCM0IsYUFBUyxPQUNQLFdBQ0EsU0FDQSxXQUNBLE9BQU8sZ0JBQWM7QUFFckIsWUFBTSxLQUFLO0FBQ1gsZ0JBQVUsWUFBWSxXQUFXLE9BQU87QUFDeEMsa0JBQVksWUFBWSxhQUFhLFNBQVM7QUFDOUMsVUFBSSxZQUFZO0FBQU0sY0FBTSxJQUFJLE1BQU0sb0NBQW9DO0FBQzFFLFlBQU0sRUFBRSxNQUFNLFFBQU8sSUFBSztBQUUxQixVQUFJLE9BQThCO0FBQ2xDLFVBQUk7QUFDSixVQUFJO0FBQ0YsWUFBSSxPQUFPLE9BQU8sWUFBWSxjQUFjLFlBQVk7QUFHdEQsY0FBSTtBQUNGLG1CQUFPLFVBQVUsUUFBUSxFQUFFO21CQUNwQixVQUFVO0FBQ2pCLGdCQUFJLEVBQUUsb0JBQW9CLElBQUk7QUFBTSxvQkFBTTtBQUMxQyxtQkFBTyxVQUFVLFlBQVksRUFBRTs7bUJBRXhCLE9BQU8sT0FBTyxZQUFZLE9BQU8sR0FBRyxNQUFNLFlBQVksT0FBTyxHQUFHLE1BQU0sVUFBVTtBQUN6RixnQkFBTSxFQUFFLEdBQUFTLElBQUcsR0FBQWxCLEdBQUMsSUFBSztBQUNqQixpQkFBTyxJQUFJLFVBQVVrQixJQUFHbEIsRUFBQztlQUNwQjtBQUNMLGdCQUFNLElBQUksTUFBTSxPQUFPOztBQUV6QixZQUFJUyxPQUFNLFFBQVEsU0FBUztlQUNwQixPQUFPO0FBQ2QsWUFBSyxNQUFnQixZQUFZO0FBQy9CLGdCQUFNLElBQUksTUFBTSxnRUFBZ0U7QUFDbEYsZUFBTzs7QUFFVCxVQUFJLFFBQVEsS0FBSyxTQUFRO0FBQUksZUFBTztBQUNwQyxVQUFJO0FBQVMsa0JBQVUsTUFBTSxLQUFLLE9BQU87QUFDekMsWUFBTSxFQUFFLEdBQUcsRUFBQyxJQUFLO0FBQ2pCLFlBQU0sSUFBSSxjQUFjLE9BQU87QUFDL0IsWUFBTSxLQUFLLEtBQUssQ0FBQztBQUNqQixZQUFNLEtBQUtJLE1BQUssSUFBSSxFQUFFO0FBQ3RCLFlBQU0sS0FBS0EsTUFBSyxJQUFJLEVBQUU7QUFDdEIsWUFBTSxJQUFJSixPQUFNLEtBQUsscUJBQXFCLEdBQUcsSUFBSSxFQUFFLEdBQUcsU0FBUTtBQUM5RCxVQUFJLENBQUM7QUFBRyxlQUFPO0FBQ2YsWUFBTSxJQUFJSSxNQUFLLEVBQUUsQ0FBQztBQUNsQixhQUFPLE1BQU07SUFDZjtBQUNBLFdBQU87TUFDTDtNQUNBLGNBQUFFO01BQ0E7TUFDQTtNQUNBO01BQ0EsaUJBQWlCTjtNQUNqQjtNQUNBOztFQUVKOzs7QUNsa0NNLE1BQU8sT0FBUCxjQUF1QyxLQUFhO0lBUXhELFlBQVlVLE9BQWEsTUFBVztBQUNsQyxZQUFLO0FBSkMsV0FBQSxXQUFXO0FBQ1gsV0FBQSxZQUFZO0FBSWxCLFdBQVdBLEtBQUk7QUFDZixZQUFNLE1BQU0sUUFBUSxJQUFJO0FBQ3hCLFdBQUssUUFBUUEsTUFBSyxPQUFNO0FBQ3hCLFVBQUksT0FBTyxLQUFLLE1BQU0sV0FBVztBQUMvQixjQUFNLElBQUksTUFBTSxxREFBcUQ7QUFDdkUsV0FBSyxXQUFXLEtBQUssTUFBTTtBQUMzQixXQUFLLFlBQVksS0FBSyxNQUFNO0FBQzVCLFlBQU0sV0FBVyxLQUFLO0FBQ3RCLFlBQU1DLE9BQU0sSUFBSSxXQUFXLFFBQVE7QUFFbkMsTUFBQUEsS0FBSSxJQUFJLElBQUksU0FBUyxXQUFXRCxNQUFLLE9BQU0sRUFBRyxPQUFPLEdBQUcsRUFBRSxPQUFNLElBQUssR0FBRztBQUN4RSxlQUFTRSxLQUFJLEdBQUdBLEtBQUlELEtBQUksUUFBUUM7QUFBSyxRQUFBRCxLQUFJQyxFQUFDLEtBQUs7QUFDL0MsV0FBSyxNQUFNLE9BQU9ELElBQUc7QUFFckIsV0FBSyxRQUFRRCxNQUFLLE9BQU07QUFFeEIsZUFBU0UsS0FBSSxHQUFHQSxLQUFJRCxLQUFJLFFBQVFDO0FBQUssUUFBQUQsS0FBSUMsRUFBQyxLQUFLLEtBQU87QUFDdEQsV0FBSyxNQUFNLE9BQU9ELElBQUc7QUFDckIsTUFBQUEsS0FBSSxLQUFLLENBQUM7SUFDWjtJQUNBLE9BQU8sS0FBVTtBQUNmLGFBQWEsSUFBSTtBQUNqQixXQUFLLE1BQU0sT0FBTyxHQUFHO0FBQ3JCLGFBQU87SUFDVDtJQUNBLFdBQVcsS0FBZTtBQUN4QixhQUFhLElBQUk7QUFDakIsWUFBWSxLQUFLLEtBQUssU0FBUztBQUMvQixXQUFLLFdBQVc7QUFDaEIsV0FBSyxNQUFNLFdBQVcsR0FBRztBQUN6QixXQUFLLE1BQU0sT0FBTyxHQUFHO0FBQ3JCLFdBQUssTUFBTSxXQUFXLEdBQUc7QUFDekIsV0FBSyxRQUFPO0lBQ2Q7SUFDQSxTQUFNO0FBQ0osWUFBTSxNQUFNLElBQUksV0FBVyxLQUFLLE1BQU0sU0FBUztBQUMvQyxXQUFLLFdBQVcsR0FBRztBQUNuQixhQUFPO0lBQ1Q7SUFDQSxXQUFXLElBQVk7QUFFckIsYUFBQSxLQUFPLE9BQU8sT0FBTyxPQUFPLGVBQWUsSUFBSSxHQUFHLENBQUEsQ0FBRTtBQUNwRCxZQUFNLEVBQUUsT0FBTyxPQUFPLFVBQVUsV0FBVyxVQUFVLFVBQVMsSUFBSztBQUNuRSxXQUFLO0FBQ0wsU0FBRyxXQUFXO0FBQ2QsU0FBRyxZQUFZO0FBQ2YsU0FBRyxXQUFXO0FBQ2QsU0FBRyxZQUFZO0FBQ2YsU0FBRyxRQUFRLE1BQU0sV0FBVyxHQUFHLEtBQUs7QUFDcEMsU0FBRyxRQUFRLE1BQU0sV0FBVyxHQUFHLEtBQUs7QUFDcEMsYUFBTztJQUNUO0lBQ0EsVUFBTztBQUNMLFdBQUssWUFBWTtBQUNqQixXQUFLLE1BQU0sUUFBTztBQUNsQixXQUFLLE1BQU0sUUFBTztJQUNwQjs7QUFTSyxNQUFNLE9BQU8sQ0FBQ0QsT0FBYSxLQUFZLFlBQzVDLElBQUksS0FBVUEsT0FBTSxHQUFHLEVBQUUsT0FBTyxPQUFPLEVBQUUsT0FBTTtBQUNqRCxPQUFLLFNBQVMsQ0FBQ0EsT0FBYSxRQUFlLElBQUksS0FBVUEsT0FBTSxHQUFHOzs7QUN6RTVELFdBQVUsUUFBUUcsT0FBVztBQUNqQyxXQUFPO01BQ0wsTUFBQUE7TUFDQSxNQUFNLENBQUMsUUFBb0IsU0FBdUIsS0FBS0EsT0FBTSxLQUFLLFlBQVksR0FBRyxJQUFJLENBQUM7TUFDdEY7O0VBRUo7QUFHTSxXQUFVLFlBQVksVUFBb0IsU0FBYztBQUM1RCxVQUFNLFNBQVMsQ0FBQ0EsVUFBZ0IsWUFBWSxFQUFFLEdBQUcsVUFBVSxHQUFHLFFBQVFBLEtBQUksRUFBQyxDQUFFO0FBQzdFLFdBQU8sT0FBTyxPQUFPLEVBQUUsR0FBRyxPQUFPLE9BQU8sR0FBRyxPQUFNLENBQUU7RUFDckQ7OztBQ1RBLE1BQU0sYUFBYSxPQUFPLG9FQUFvRTtBQUM5RixNQUFNLGFBQWEsT0FBTyxvRUFBb0U7QUFDOUYsTUFBTUMsT0FBTSxPQUFPLENBQUM7QUFDcEIsTUFBTUMsT0FBTSxPQUFPLENBQUM7QUFDcEIsTUFBTSxhQUFhLENBQUMsR0FBVyxPQUFlLElBQUksSUFBSUEsUUFBTztBQU03RCxXQUFTLFFBQVEsR0FBUztBQUN4QixVQUFNLElBQUk7QUFFVixVQUFNQyxPQUFNLE9BQU8sQ0FBQyxHQUFHLE1BQU0sT0FBTyxDQUFDLEdBQUcsT0FBTyxPQUFPLEVBQUUsR0FBRyxPQUFPLE9BQU8sRUFBRTtBQUUzRSxVQUFNLE9BQU8sT0FBTyxFQUFFLEdBQUcsT0FBTyxPQUFPLEVBQUUsR0FBRyxPQUFPLE9BQU8sRUFBRTtBQUM1RCxVQUFNLEtBQU0sSUFBSSxJQUFJLElBQUs7QUFDekIsVUFBTSxLQUFNLEtBQUssS0FBSyxJQUFLO0FBQzNCLFVBQU0sS0FBTSxLQUFLLElBQUlBLE1BQUssQ0FBQyxJQUFJLEtBQU07QUFDckMsVUFBTSxLQUFNLEtBQUssSUFBSUEsTUFBSyxDQUFDLElBQUksS0FBTTtBQUNyQyxVQUFNLE1BQU8sS0FBSyxJQUFJRCxNQUFLLENBQUMsSUFBSSxLQUFNO0FBQ3RDLFVBQU0sTUFBTyxLQUFLLEtBQUssTUFBTSxDQUFDLElBQUksTUFBTztBQUN6QyxVQUFNLE1BQU8sS0FBSyxLQUFLLE1BQU0sQ0FBQyxJQUFJLE1BQU87QUFDekMsVUFBTSxNQUFPLEtBQUssS0FBSyxNQUFNLENBQUMsSUFBSSxNQUFPO0FBQ3pDLFVBQU0sT0FBUSxLQUFLLEtBQUssTUFBTSxDQUFDLElBQUksTUFBTztBQUMxQyxVQUFNLE9BQVEsS0FBSyxNQUFNLE1BQU0sQ0FBQyxJQUFJLE1BQU87QUFDM0MsVUFBTSxPQUFRLEtBQUssTUFBTUMsTUFBSyxDQUFDLElBQUksS0FBTTtBQUN6QyxVQUFNLEtBQU0sS0FBSyxNQUFNLE1BQU0sQ0FBQyxJQUFJLE1BQU87QUFDekMsVUFBTSxLQUFNLEtBQUssSUFBSSxLQUFLLENBQUMsSUFBSSxLQUFNO0FBQ3JDLFVBQU0sT0FBTyxLQUFLLElBQUlELE1BQUssQ0FBQztBQUM1QixRQUFJLENBQUMsR0FBRyxJQUFJLEdBQUcsSUFBSSxJQUFJLEdBQUcsQ0FBQztBQUFHLFlBQU0sSUFBSSxNQUFNLHlCQUF5QjtBQUN2RSxXQUFPO0VBQ1Q7QUFFQSxNQUFNLEtBQUssTUFBTSxZQUFZLFFBQVcsUUFBVyxFQUFFLE1BQU0sUUFBTyxDQUFFO0FBRTdELE1BQU0sWUFBWSxZQUN2QjtJQUNFLEdBQUcsT0FBTyxDQUFDO0lBQ1gsR0FBRyxPQUFPLENBQUM7SUFDWDtJQUNBLEdBQUc7O0lBRUgsSUFBSSxPQUFPLCtFQUErRTtJQUMxRixJQUFJLE9BQU8sK0VBQStFO0lBQzFGLEdBQUcsT0FBTyxDQUFDO0lBQ1gsTUFBTTs7Ozs7OztJQU9OLE1BQU07TUFDSixNQUFNLE9BQU8sb0VBQW9FO01BQ2pGLGFBQWEsQ0FBQyxNQUFhO0FBQ3pCLGNBQU0sSUFBSTtBQUNWLGNBQU0sS0FBSyxPQUFPLG9DQUFvQztBQUN0RCxjQUFNLEtBQUssQ0FBQ0QsT0FBTSxPQUFPLG9DQUFvQztBQUM3RCxjQUFNLEtBQUssT0FBTyxxQ0FBcUM7QUFDdkQsY0FBTSxLQUFLO0FBQ1gsY0FBTSxZQUFZLE9BQU8scUNBQXFDO0FBRTlELGNBQU0sS0FBSyxXQUFXLEtBQUssR0FBRyxDQUFDO0FBQy9CLGNBQU0sS0FBSyxXQUFXLENBQUMsS0FBSyxHQUFHLENBQUM7QUFDaEMsWUFBSSxLQUFLLElBQUksSUFBSSxLQUFLLEtBQUssS0FBSyxJQUFJLENBQUM7QUFDckMsWUFBSSxLQUFLLElBQUksQ0FBQyxLQUFLLEtBQUssS0FBSyxJQUFJLENBQUM7QUFDbEMsY0FBTSxRQUFRLEtBQUs7QUFDbkIsY0FBTSxRQUFRLEtBQUs7QUFDbkIsWUFBSTtBQUFPLGVBQUssSUFBSTtBQUNwQixZQUFJO0FBQU8sZUFBSyxJQUFJO0FBQ3BCLFlBQUksS0FBSyxhQUFhLEtBQUssV0FBVztBQUNwQyxnQkFBTSxJQUFJLE1BQU0seUNBQXlDLENBQUM7O0FBRTVELGVBQU8sRUFBRSxPQUFPLElBQUksT0FBTyxHQUFFO01BQy9COztLQUdKLE1BQU07QUFLUixNQUFNRyxPQUFNLE9BQU8sQ0FBQztBQUNwQixNQUFNLEtBQUssQ0FBQyxNQUFjLE9BQU8sTUFBTSxZQUFZQSxPQUFNLEtBQUssSUFBSTtBQUNsRSxNQUFNLEtBQUssQ0FBQyxNQUFjLE9BQU8sTUFBTSxZQUFZQSxPQUFNLEtBQUssSUFBSTtBQUVsRSxNQUFNLHVCQUFzRCxDQUFBO0FBQzVELFdBQVMsV0FBVyxRQUFnQixVQUFzQjtBQUN4RCxRQUFJLE9BQU8scUJBQXFCLEdBQUc7QUFDbkMsUUFBSSxTQUFTLFFBQVc7QUFDdEIsWUFBTSxPQUFPLE9BQU8sV0FBVyxLQUFLLEtBQUssQ0FBQyxNQUFNLEVBQUUsV0FBVyxDQUFDLENBQUMsQ0FBQztBQUNoRSxhQUFPQyxhQUFZLE1BQU0sSUFBSTtBQUM3QiwyQkFBcUIsR0FBRyxJQUFJOztBQUU5QixXQUFPLE9BQU9BLGFBQVksTUFBTSxHQUFHLFFBQVEsQ0FBQztFQUM5QztBQUdBLE1BQU0sZUFBZSxDQUFDLFVBQTZCLE1BQU0sV0FBVyxJQUFJLEVBQUUsTUFBTSxDQUFDO0FBQ2pGLE1BQU0sV0FBVyxDQUFDLE1BQWMsZ0JBQWdCLEdBQUcsRUFBRTtBQUNyRCxNQUFNLE9BQU8sQ0FBQyxNQUFjLElBQUksR0FBRyxVQUFVO0FBQzdDLE1BQU0sT0FBTyxDQUFDLE1BQWMsSUFBSSxHQUFHLFVBQVU7QUFDN0MsTUFBTSxRQUFRLFVBQVU7QUFDeEIsTUFBTSxVQUFVLENBQUMsR0FBc0IsR0FBVyxNQUNoRCxNQUFNLEtBQUsscUJBQXFCLEdBQUcsR0FBRyxDQUFDO0FBR3pDLFdBQVMsb0JBQW9CLE1BQWE7QUFDeEMsUUFBSSxLQUFLLFVBQVUsTUFBTSx1QkFBdUIsSUFBSTtBQUNwRCxRQUFJLElBQUksTUFBTSxlQUFlLEVBQUU7QUFDL0IsVUFBTSxTQUFTLEVBQUUsU0FBUSxJQUFLLEtBQUssS0FBSyxDQUFDLEVBQUU7QUFDM0MsV0FBTyxFQUFFLFFBQWdCLE9BQU8sYUFBYSxDQUFDLEVBQUM7RUFDakQ7QUFLQSxXQUFTLE9BQU8sR0FBUztBQUN2QixRQUFJLENBQUMsR0FBRyxDQUFDO0FBQUcsWUFBTSxJQUFJLE1BQU0sdUJBQXVCO0FBQ25ELFVBQU0sS0FBSyxLQUFLLElBQUksQ0FBQztBQUNyQixVQUFNLElBQUksS0FBSyxLQUFLLElBQUksT0FBTyxDQUFDLENBQUM7QUFDakMsUUFBSSxJQUFJLFFBQVEsQ0FBQztBQUNqQixRQUFJLElBQUlILFNBQVFFO0FBQUssVUFBSSxLQUFLLENBQUMsQ0FBQztBQUNoQyxVQUFNLElBQUksSUFBSSxNQUFNLEdBQUcsR0FBR0gsSUFBRztBQUM3QixNQUFFLGVBQWM7QUFDaEIsV0FBTztFQUNUO0FBSUEsV0FBUyxhQUFhLE1BQWtCO0FBQ3RDLFdBQU8sS0FBSyxnQkFBZ0IsV0FBVyxxQkFBcUIsR0FBRyxJQUFJLENBQUMsQ0FBQztFQUN2RTtBQUtBLFdBQVMsb0JBQW9CLFlBQWU7QUFDMUMsV0FBTyxvQkFBb0IsVUFBVSxFQUFFO0VBQ3pDO0FBTUEsV0FBUyxZQUNQLFNBQ0EsWUFDQSxVQUFlLFlBQVksRUFBRSxHQUFDO0FBRTlCLFVBQU0sSUFBSSxZQUFZLFdBQVcsT0FBTztBQUN4QyxVQUFNLEVBQUUsT0FBTyxJQUFJLFFBQVEsRUFBQyxJQUFLLG9CQUFvQixVQUFVO0FBQy9ELFVBQU0sSUFBSSxZQUFZLFdBQVcsU0FBUyxFQUFFO0FBQzVDLFVBQU0sSUFBSSxTQUFTLElBQUksZ0JBQWdCLFdBQVcsZUFBZSxDQUFDLENBQUMsQ0FBQztBQUNwRSxVQUFNLE9BQU8sV0FBVyxpQkFBaUIsR0FBRyxJQUFJLENBQUM7QUFDakQsVUFBTSxLQUFLLEtBQUssZ0JBQWdCLElBQUksQ0FBQztBQUNyQyxRQUFJLE9BQU9HO0FBQUssWUFBTSxJQUFJLE1BQU0sd0JBQXdCO0FBQ3hELFVBQU0sRUFBRSxPQUFPLElBQUksUUFBUSxFQUFDLElBQUssb0JBQW9CLEVBQUU7QUFDdkQsVUFBTSxJQUFJLFVBQVUsSUFBSSxJQUFJLENBQUM7QUFDN0IsVUFBTSxNQUFNLElBQUksV0FBVyxFQUFFO0FBQzdCLFFBQUksSUFBSSxJQUFJLENBQUM7QUFDYixRQUFJLElBQUksU0FBUyxLQUFLLElBQUksSUFBSSxDQUFDLENBQUMsR0FBRyxFQUFFO0FBRXJDLFFBQUksQ0FBQyxjQUFjLEtBQUssR0FBRyxFQUFFO0FBQUcsWUFBTSxJQUFJLE1BQU0sa0NBQWtDO0FBQ2xGLFdBQU87RUFDVDtBQU1BLFdBQVMsY0FBYyxXQUFnQixTQUFjLFdBQWM7QUFDakUsVUFBTSxNQUFNLFlBQVksYUFBYSxXQUFXLEVBQUU7QUFDbEQsVUFBTSxJQUFJLFlBQVksV0FBVyxPQUFPO0FBQ3hDLFVBQU0sTUFBTSxZQUFZLGFBQWEsV0FBVyxFQUFFO0FBQ2xELFFBQUk7QUFDRixZQUFNLElBQUksT0FBTyxnQkFBZ0IsR0FBRyxDQUFDO0FBQ3JDLFlBQU0sSUFBSSxnQkFBZ0IsSUFBSSxTQUFTLEdBQUcsRUFBRSxDQUFDO0FBQzdDLFVBQUksQ0FBQyxHQUFHLENBQUM7QUFBRyxlQUFPO0FBQ25CLFlBQU0sSUFBSSxnQkFBZ0IsSUFBSSxTQUFTLElBQUksRUFBRSxDQUFDO0FBQzlDLFVBQUksQ0FBQyxHQUFHLENBQUM7QUFBRyxlQUFPO0FBQ25CLFlBQU0sSUFBSSxVQUFVLFNBQVMsQ0FBQyxHQUFHLGFBQWEsQ0FBQyxHQUFHLENBQUM7QUFDbkQsWUFBTSxJQUFJLFFBQVEsR0FBRyxHQUFHLEtBQUssQ0FBQyxDQUFDLENBQUM7QUFDaEMsVUFBSSxDQUFDLEtBQUssQ0FBQyxFQUFFLFNBQVEsS0FBTSxFQUFFLFNBQVEsRUFBRyxNQUFNO0FBQUcsZUFBTztBQUN4RCxhQUFPO2FBQ0EsT0FBTztBQUNkLGFBQU87O0VBRVg7QUFFTyxNQUFNLFVBQTJCLHdCQUFPO0lBQzdDLGNBQWM7SUFDZCxNQUFNO0lBQ04sUUFBUTtJQUNSLE9BQU87TUFDTCxrQkFBa0IsVUFBVSxNQUFNO01BQ2xDO01BQ0E7TUFDQTtNQUNBO01BQ0E7TUFDQTs7TUFFRDs7O0FDbk5JLE1BQU1FLFVBQ1gsT0FBTyxlQUFlLFlBQVksWUFBWSxhQUFhLFdBQVcsU0FBUzs7O0FDVWpGLE1BQU1DLE9BQU0sQ0FBQyxNQUE0QixhQUFhO0FBTy9DLE1BQU1DLGNBQWEsQ0FBQyxRQUN6QixJQUFJLFNBQVMsSUFBSSxRQUFRLElBQUksWUFBWSxJQUFJLFVBQVU7QUFHbEQsTUFBTUMsUUFBTyxDQUFDLE1BQWMsVUFBbUIsUUFBUyxLQUFLLFFBQVcsU0FBUztBQUlqRixNQUFNQyxRQUFPLElBQUksV0FBVyxJQUFJLFlBQVksQ0FBQyxTQUFVLENBQUMsRUFBRSxNQUFNLEVBQUUsQ0FBQyxNQUFNO0FBQ2hGLE1BQUksQ0FBQ0E7QUFBTSxVQUFNLElBQUksTUFBTSw2Q0FBNkM7QUFFeEUsTUFBTUMsU0FBUSxNQUFNLEtBQUssRUFBRSxRQUFRLElBQUcsR0FBSSxDQUFDLEdBQUdDLE9BQU1BLEdBQUUsU0FBUyxFQUFFLEVBQUUsU0FBUyxHQUFHLEdBQUcsQ0FBQztBQUk3RSxXQUFVQyxZQUFXQyxRQUFpQjtBQUMxQyxRQUFJLENBQUNDLEtBQUlELE1BQUs7QUFBRyxZQUFNLElBQUksTUFBTSxxQkFBcUI7QUFFdEQsUUFBSUUsT0FBTTtBQUNWLGFBQVNKLEtBQUksR0FBR0EsS0FBSUUsT0FBTSxRQUFRRixNQUFLO0FBQ3JDLE1BQUFJLFFBQU9MLE9BQU1HLE9BQU1GLEVBQUMsQ0FBQzs7QUFFdkIsV0FBT0k7RUFDVDtBQUtNLFdBQVVDLFlBQVdELE1BQVc7QUFDcEMsUUFBSSxPQUFPQSxTQUFRO0FBQVUsWUFBTSxJQUFJLE1BQU0sOEJBQThCLE9BQU9BLElBQUc7QUFDckYsVUFBTSxNQUFNQSxLQUFJO0FBQ2hCLFFBQUksTUFBTTtBQUFHLFlBQU0sSUFBSSxNQUFNLDREQUE0RCxHQUFHO0FBQzVGLFVBQU0sUUFBUSxJQUFJLFdBQVcsTUFBTSxDQUFDO0FBQ3BDLGFBQVNKLEtBQUksR0FBR0EsS0FBSSxNQUFNLFFBQVFBLE1BQUs7QUFDckMsWUFBTSxJQUFJQSxLQUFJO0FBQ2QsWUFBTSxVQUFVSSxLQUFJLE1BQU0sR0FBRyxJQUFJLENBQUM7QUFDbEMsWUFBTSxPQUFPLE9BQU8sU0FBUyxTQUFTLEVBQUU7QUFDeEMsVUFBSSxPQUFPLE1BQU0sSUFBSSxLQUFLLE9BQU87QUFBRyxjQUFNLElBQUksTUFBTSx1QkFBdUI7QUFDM0UsWUFBTUosRUFBQyxJQUFJOztBQUViLFdBQU87RUFDVDtBQTJCTSxXQUFVTSxhQUFZLEtBQVc7QUFDckMsUUFBSSxPQUFPLFFBQVE7QUFBVSxZQUFNLElBQUksTUFBTSxvQ0FBb0MsT0FBTyxHQUFHLEVBQUU7QUFDN0YsV0FBTyxJQUFJLFdBQVcsSUFBSSxZQUFXLEVBQUcsT0FBTyxHQUFHLENBQUM7RUFDckQ7QUFRTSxXQUFVQyxTQUFRLE1BQVc7QUFDakMsUUFBSSxPQUFPLFNBQVM7QUFBVSxhQUFPRCxhQUFZLElBQUk7QUFDckQsUUFBSSxDQUFDRSxLQUFJLElBQUk7QUFBRyxZQUFNLElBQUksTUFBTSw0QkFBNEIsT0FBTyxJQUFJLEVBQUU7QUFDekUsV0FBTztFQUNUO0FBS00sV0FBVUMsZ0JBQWUsUUFBb0I7QUFDakQsVUFBTSxJQUFJLElBQUksV0FBVyxPQUFPLE9BQU8sQ0FBQyxLQUFLLE1BQU0sTUFBTSxFQUFFLFFBQVEsQ0FBQyxDQUFDO0FBQ3JFLFFBQUlDLE9BQU07QUFDVixXQUFPLFFBQVEsQ0FBQyxNQUFLO0FBQ25CLFVBQUksQ0FBQ0YsS0FBSSxDQUFDO0FBQUcsY0FBTSxJQUFJLE1BQU0scUJBQXFCO0FBQ2xELFFBQUUsSUFBSSxHQUFHRSxJQUFHO0FBQ1osTUFBQUEsUUFBTyxFQUFFO0lBQ1gsQ0FBQztBQUNELFdBQU87RUFDVDtBQUdNLE1BQWdCQyxRQUFoQixNQUFvQjs7SUFzQnhCLFFBQUs7QUFDSCxhQUFPLEtBQUssV0FBVTtJQUN4Qjs7QUErQkksV0FBVUMsaUJBQW1DLFVBQXVCO0FBQ3hFLFVBQU0sUUFBUSxDQUFDLFFBQTJCLFNBQVEsRUFBRyxPQUFPQyxTQUFRLEdBQUcsQ0FBQyxFQUFFLE9BQU07QUFDaEYsVUFBTSxNQUFNLFNBQVE7QUFDcEIsVUFBTSxZQUFZLElBQUk7QUFDdEIsVUFBTSxXQUFXLElBQUk7QUFDckIsVUFBTSxTQUFTLE1BQU0sU0FBUTtBQUM3QixXQUFPO0VBQ1Q7QUEyQk0sV0FBVUMsYUFBWSxjQUFjLElBQUU7QUFDMUMsUUFBSUMsV0FBVSxPQUFPQSxRQUFPLG9CQUFvQixZQUFZO0FBQzFELGFBQU9BLFFBQU8sZ0JBQWdCLElBQUksV0FBVyxXQUFXLENBQUM7O0FBRTNELFVBQU0sSUFBSSxNQUFNLHdDQUF3QztFQUMxRDs7O0FDdk5NLFdBQVVDLFFBQU8sR0FBUztBQUM5QixRQUFJLENBQUMsT0FBTyxjQUFjLENBQUMsS0FBSyxJQUFJO0FBQUcsWUFBTSxJQUFJLE1BQU0sMkJBQTJCLENBQUMsRUFBRTtFQUN2RjtBQUVNLFdBQVUsS0FBSyxHQUFVO0FBQzdCLFFBQUksT0FBTyxNQUFNO0FBQVcsWUFBTSxJQUFJLE1BQU0seUJBQXlCLENBQUMsRUFBRTtFQUMxRTtBQUVNLFdBQVVDLE9BQU0sTUFBOEIsU0FBaUI7QUFDbkUsUUFBSSxFQUFFLGFBQWE7QUFBYSxZQUFNLElBQUksTUFBTSxxQkFBcUI7QUFDckUsUUFBSSxRQUFRLFNBQVMsS0FBSyxDQUFDLFFBQVEsU0FBUyxFQUFFLE1BQU07QUFDbEQsWUFBTSxJQUFJLE1BQU0saUNBQWlDLE9BQU8sbUJBQW1CLEVBQUUsTUFBTSxFQUFFO0VBQ3pGO0FBUU0sV0FBVUMsTUFBS0EsT0FBVTtBQUM3QixRQUFJLE9BQU9BLFVBQVMsY0FBYyxPQUFPQSxNQUFLLFdBQVc7QUFDdkQsWUFBTSxJQUFJLE1BQU0saURBQWlEO0FBQ25FLElBQUFGLFFBQU9FLE1BQUssU0FBUztBQUNyQixJQUFBRixRQUFPRSxNQUFLLFFBQVE7RUFDdEI7QUFFTSxXQUFVQyxRQUFPLFVBQWUsZ0JBQWdCLE1BQUk7QUFDeEQsUUFBSSxTQUFTO0FBQVcsWUFBTSxJQUFJLE1BQU0sa0NBQWtDO0FBQzFFLFFBQUksaUJBQWlCLFNBQVM7QUFBVSxZQUFNLElBQUksTUFBTSx1Q0FBdUM7RUFDakc7QUFDTSxXQUFVQyxRQUFPLEtBQVUsVUFBYTtBQUM1QyxJQUFBSCxPQUFNLEdBQUc7QUFDVCxVQUFNLE1BQU0sU0FBUztBQUNyQixRQUFJLElBQUksU0FBUyxLQUFLO0FBQ3BCLFlBQU0sSUFBSSxNQUFNLHlEQUF5RCxHQUFHLEVBQUU7O0VBRWxGO0FBRUEsTUFBTSxTQUFTO0lBQ2IsUUFBQUQ7SUFDQTtJQUNBLE9BQUFDO0lBQ0EsTUFBQUM7SUFDQSxRQUFBQztJQUNBLFFBQUFDOztBQUdGLE1BQUEsaUJBQWU7OztBQzVDZixXQUFTQyxjQUFhLE1BQWdCLFlBQW9CLE9BQWVDLE9BQWE7QUFDcEYsUUFBSSxPQUFPLEtBQUssaUJBQWlCO0FBQVksYUFBTyxLQUFLLGFBQWEsWUFBWSxPQUFPQSxLQUFJO0FBQzdGLFVBQU0sT0FBTyxPQUFPLEVBQUU7QUFDdEIsVUFBTSxXQUFXLE9BQU8sVUFBVTtBQUNsQyxVQUFNLEtBQUssT0FBUSxTQUFTLE9BQVEsUUFBUTtBQUM1QyxVQUFNLEtBQUssT0FBTyxRQUFRLFFBQVE7QUFDbEMsVUFBTSxJQUFJQSxRQUFPLElBQUk7QUFDckIsVUFBTSxJQUFJQSxRQUFPLElBQUk7QUFDckIsU0FBSyxVQUFVLGFBQWEsR0FBRyxJQUFJQSxLQUFJO0FBQ3ZDLFNBQUssVUFBVSxhQUFhLEdBQUcsSUFBSUEsS0FBSTtFQUN6QztBQUdNLE1BQWdCQyxRQUFoQixjQUFnREMsTUFBTztJQWMzRCxZQUNXLFVBQ0YsV0FDRSxXQUNBRixPQUFhO0FBRXRCLFlBQUs7QUFMSSxXQUFBLFdBQUE7QUFDRixXQUFBLFlBQUE7QUFDRSxXQUFBLFlBQUE7QUFDQSxXQUFBLE9BQUFBO0FBVEQsV0FBQSxXQUFXO0FBQ1gsV0FBQSxTQUFTO0FBQ1QsV0FBQSxNQUFNO0FBQ04sV0FBQSxZQUFZO0FBU3BCLFdBQUssU0FBUyxJQUFJLFdBQVcsUUFBUTtBQUNyQyxXQUFLLE9BQU9HLFlBQVcsS0FBSyxNQUFNO0lBQ3BDO0lBQ0EsT0FBTyxNQUFXO0FBQ2hCLHFCQUFPLE9BQU8sSUFBSTtBQUNsQixZQUFNLEVBQUUsTUFBTSxRQUFRLFNBQVEsSUFBSztBQUNuQyxhQUFPQyxTQUFRLElBQUk7QUFDbkIsWUFBTSxNQUFNLEtBQUs7QUFDakIsZUFBUyxNQUFNLEdBQUcsTUFBTSxPQUFPO0FBQzdCLGNBQU0sT0FBTyxLQUFLLElBQUksV0FBVyxLQUFLLEtBQUssTUFBTSxHQUFHO0FBRXBELFlBQUksU0FBUyxVQUFVO0FBQ3JCLGdCQUFNLFdBQVdELFlBQVcsSUFBSTtBQUNoQyxpQkFBTyxZQUFZLE1BQU0sS0FBSyxPQUFPO0FBQVUsaUJBQUssUUFBUSxVQUFVLEdBQUc7QUFDekU7O0FBRUYsZUFBTyxJQUFJLEtBQUssU0FBUyxLQUFLLE1BQU0sSUFBSSxHQUFHLEtBQUssR0FBRztBQUNuRCxhQUFLLE9BQU87QUFDWixlQUFPO0FBQ1AsWUFBSSxLQUFLLFFBQVEsVUFBVTtBQUN6QixlQUFLLFFBQVEsTUFBTSxDQUFDO0FBQ3BCLGVBQUssTUFBTTs7O0FBR2YsV0FBSyxVQUFVLEtBQUs7QUFDcEIsV0FBSyxXQUFVO0FBQ2YsYUFBTztJQUNUO0lBQ0EsV0FBVyxLQUFlO0FBQ3hCLHFCQUFPLE9BQU8sSUFBSTtBQUNsQixxQkFBTyxPQUFPLEtBQUssSUFBSTtBQUN2QixXQUFLLFdBQVc7QUFJaEIsWUFBTSxFQUFFLFFBQVEsTUFBTSxVQUFVLE1BQUFILE1BQUksSUFBSztBQUN6QyxVQUFJLEVBQUUsSUFBRyxJQUFLO0FBRWQsYUFBTyxLQUFLLElBQUk7QUFDaEIsV0FBSyxPQUFPLFNBQVMsR0FBRyxFQUFFLEtBQUssQ0FBQztBQUVoQyxVQUFJLEtBQUssWUFBWSxXQUFXLEtBQUs7QUFDbkMsYUFBSyxRQUFRLE1BQU0sQ0FBQztBQUNwQixjQUFNOztBQUdSLGVBQVNLLEtBQUksS0FBS0EsS0FBSSxVQUFVQTtBQUFLLGVBQU9BLEVBQUMsSUFBSTtBQUlqRCxNQUFBTixjQUFhLE1BQU0sV0FBVyxHQUFHLE9BQU8sS0FBSyxTQUFTLENBQUMsR0FBR0MsS0FBSTtBQUM5RCxXQUFLLFFBQVEsTUFBTSxDQUFDO0FBQ3BCLFlBQU0sUUFBUUcsWUFBVyxHQUFHO0FBQzVCLFlBQU0sTUFBTSxLQUFLO0FBRWpCLFVBQUksTUFBTTtBQUFHLGNBQU0sSUFBSSxNQUFNLDZDQUE2QztBQUMxRSxZQUFNLFNBQVMsTUFBTTtBQUNyQixZQUFNLFFBQVEsS0FBSyxJQUFHO0FBQ3RCLFVBQUksU0FBUyxNQUFNO0FBQVEsY0FBTSxJQUFJLE1BQU0sb0NBQW9DO0FBQy9FLGVBQVNFLEtBQUksR0FBR0EsS0FBSSxRQUFRQTtBQUFLLGNBQU0sVUFBVSxJQUFJQSxJQUFHLE1BQU1BLEVBQUMsR0FBR0wsS0FBSTtJQUN4RTtJQUNBLFNBQU07QUFDSixZQUFNLEVBQUUsUUFBUSxVQUFTLElBQUs7QUFDOUIsV0FBSyxXQUFXLE1BQU07QUFDdEIsWUFBTSxNQUFNLE9BQU8sTUFBTSxHQUFHLFNBQVM7QUFDckMsV0FBSyxRQUFPO0FBQ1osYUFBTztJQUNUO0lBQ0EsV0FBVyxJQUFNO0FBQ2YsYUFBQSxLQUFPLElBQUssS0FBSyxZQUFtQjtBQUNwQyxTQUFHLElBQUksR0FBRyxLQUFLLElBQUcsQ0FBRTtBQUNwQixZQUFNLEVBQUUsVUFBVSxRQUFRLFFBQVEsVUFBVSxXQUFXLElBQUcsSUFBSztBQUMvRCxTQUFHLFNBQVM7QUFDWixTQUFHLE1BQU07QUFDVCxTQUFHLFdBQVc7QUFDZCxTQUFHLFlBQVk7QUFDZixVQUFJLFNBQVM7QUFBVSxXQUFHLE9BQU8sSUFBSSxNQUFNO0FBQzNDLGFBQU87SUFDVDs7OztBQ2hIRixNQUFNTSxPQUFNLENBQUMsR0FBVyxHQUFXLE1BQWUsSUFBSSxJQUFNLENBQUMsSUFBSTtBQUVqRSxNQUFNQyxPQUFNLENBQUMsR0FBVyxHQUFXLE1BQWUsSUFBSSxJQUFNLElBQUksSUFBTSxJQUFJO0FBSzFFLE1BQU1DLFlBQVcsSUFBSSxZQUFZO0lBQy9CO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFDcEY7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUNwRjtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQ3BGO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFDcEY7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUNwRjtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQ3BGO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFDcEY7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtHQUNyRjtBQUlELE1BQU1DLE1BQUssSUFBSSxZQUFZO0lBQ3pCO0lBQVk7SUFBWTtJQUFZO0lBQVk7SUFBWTtJQUFZO0lBQVk7R0FDckY7QUFJRCxNQUFNQyxZQUFXLElBQUksWUFBWSxFQUFFO0FBQ25DLE1BQU1DLFVBQU4sY0FBcUJDLE1BQVk7SUFZL0IsY0FBQTtBQUNFLFlBQU0sSUFBSSxJQUFJLEdBQUcsS0FBSztBQVZ4QixXQUFBLElBQUlILElBQUcsQ0FBQyxJQUFJO0FBQ1osV0FBQSxJQUFJQSxJQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSUEsSUFBRyxDQUFDLElBQUk7QUFDWixXQUFBLElBQUlBLElBQUcsQ0FBQyxJQUFJO0FBQ1osV0FBQSxJQUFJQSxJQUFHLENBQUMsSUFBSTtBQUNaLFdBQUEsSUFBSUEsSUFBRyxDQUFDLElBQUk7QUFDWixXQUFBLElBQUlBLElBQUcsQ0FBQyxJQUFJO0FBQ1osV0FBQSxJQUFJQSxJQUFHLENBQUMsSUFBSTtJQUlaO0lBQ1UsTUFBRztBQUNYLFlBQU0sRUFBRSxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEVBQUMsSUFBSztBQUNuQyxhQUFPLENBQUMsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxDQUFDO0lBQ2hDOztJQUVVLElBQ1IsR0FBVyxHQUFXLEdBQVcsR0FBVyxHQUFXLEdBQVcsR0FBVyxHQUFTO0FBRXRGLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7QUFDYixXQUFLLElBQUksSUFBSTtBQUNiLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7QUFDYixXQUFLLElBQUksSUFBSTtBQUNiLFdBQUssSUFBSSxJQUFJO0FBQ2IsV0FBSyxJQUFJLElBQUk7SUFDZjtJQUNVLFFBQVEsTUFBZ0IsUUFBYztBQUU5QyxlQUFTSSxLQUFJLEdBQUdBLEtBQUksSUFBSUEsTUFBSyxVQUFVO0FBQUcsUUFBQUgsVUFBU0csRUFBQyxJQUFJLEtBQUssVUFBVSxRQUFRLEtBQUs7QUFDcEYsZUFBU0EsS0FBSSxJQUFJQSxLQUFJLElBQUlBLE1BQUs7QUFDNUIsY0FBTSxNQUFNSCxVQUFTRyxLQUFJLEVBQUU7QUFDM0IsY0FBTSxLQUFLSCxVQUFTRyxLQUFJLENBQUM7QUFDekIsY0FBTSxLQUFLQyxNQUFLLEtBQUssQ0FBQyxJQUFJQSxNQUFLLEtBQUssRUFBRSxJQUFLLFFBQVE7QUFDbkQsY0FBTSxLQUFLQSxNQUFLLElBQUksRUFBRSxJQUFJQSxNQUFLLElBQUksRUFBRSxJQUFLLE9BQU87QUFDakQsUUFBQUosVUFBU0csRUFBQyxJQUFLLEtBQUtILFVBQVNHLEtBQUksQ0FBQyxJQUFJLEtBQUtILFVBQVNHLEtBQUksRUFBRSxJQUFLOztBQUdqRSxVQUFJLEVBQUUsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxFQUFDLElBQUs7QUFDakMsZUFBU0EsS0FBSSxHQUFHQSxLQUFJLElBQUlBLE1BQUs7QUFDM0IsY0FBTSxTQUFTQyxNQUFLLEdBQUcsQ0FBQyxJQUFJQSxNQUFLLEdBQUcsRUFBRSxJQUFJQSxNQUFLLEdBQUcsRUFBRTtBQUNwRCxjQUFNLEtBQU0sSUFBSSxTQUFTUixLQUFJLEdBQUcsR0FBRyxDQUFDLElBQUlFLFVBQVNLLEVBQUMsSUFBSUgsVUFBU0csRUFBQyxJQUFLO0FBQ3JFLGNBQU0sU0FBU0MsTUFBSyxHQUFHLENBQUMsSUFBSUEsTUFBSyxHQUFHLEVBQUUsSUFBSUEsTUFBSyxHQUFHLEVBQUU7QUFDcEQsY0FBTSxLQUFNLFNBQVNQLEtBQUksR0FBRyxHQUFHLENBQUMsSUFBSztBQUNyQyxZQUFJO0FBQ0osWUFBSTtBQUNKLFlBQUk7QUFDSixZQUFLLElBQUksS0FBTTtBQUNmLFlBQUk7QUFDSixZQUFJO0FBQ0osWUFBSTtBQUNKLFlBQUssS0FBSyxLQUFNOztBQUdsQixVQUFLLElBQUksS0FBSyxJQUFLO0FBQ25CLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsVUFBSyxJQUFJLEtBQUssSUFBSztBQUNuQixVQUFLLElBQUksS0FBSyxJQUFLO0FBQ25CLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsVUFBSyxJQUFJLEtBQUssSUFBSztBQUNuQixVQUFLLElBQUksS0FBSyxJQUFLO0FBQ25CLFVBQUssSUFBSSxLQUFLLElBQUs7QUFDbkIsV0FBSyxJQUFJLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsQ0FBQztJQUNqQztJQUNVLGFBQVU7QUFDbEIsTUFBQUcsVUFBUyxLQUFLLENBQUM7SUFDakI7SUFDQSxVQUFPO0FBQ0wsV0FBSyxJQUFJLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsQ0FBQztBQUMvQixXQUFLLE9BQU8sS0FBSyxDQUFDO0lBQ3BCOztBQUdGLE1BQU0sU0FBTixjQUFxQkMsUUFBTTtJQVN6QixjQUFBO0FBQ0UsWUFBSztBQVRQLFdBQUEsSUFBSSxhQUFhO0FBQ2pCLFdBQUEsSUFBSSxZQUFhO0FBQ2pCLFdBQUEsSUFBSSxZQUFhO0FBQ2pCLFdBQUEsSUFBSSxhQUFhO0FBQ2pCLFdBQUEsSUFBSSxhQUFhO0FBQ2pCLFdBQUEsSUFBSSxhQUFhO0FBQ2pCLFdBQUEsSUFBSSxhQUFhO0FBQ2pCLFdBQUEsSUFBSSxhQUFhO0FBR2YsV0FBSyxZQUFZO0lBQ25COztBQU9LLE1BQU1JLFVBQVNDLGlCQUFnQixNQUFNLElBQUlMLFFBQU0sQ0FBRTtBQUNqRCxNQUFNLFNBQVNLLGlCQUFnQixNQUFNLElBQUksT0FBTSxDQUFFOzs7QUNoSWpELFdBQVMsYUFBYSxHQUFHO0FBQzVCLFFBQUksQ0FBQyxPQUFPLGNBQWMsQ0FBQztBQUN2QixZQUFNLElBQUksTUFBTSxrQkFBa0IsQ0FBQyxFQUFFO0FBQUEsRUFDN0M7QUFDQSxXQUFTLFNBQVMsTUFBTTtBQUNwQixVQUFNQyxRQUFPLENBQUMsR0FBRyxNQUFNLENBQUMsTUFBTSxFQUFFLEVBQUUsQ0FBQyxDQUFDO0FBQ3BDLFVBQU0sU0FBUyxNQUFNLEtBQUssSUFBSSxFQUN6QixRQUFRLEVBQ1IsT0FBTyxDQUFDLEtBQUtDLE9BQU8sTUFBTUQsTUFBSyxLQUFLQyxHQUFFLE1BQU0sSUFBSUEsR0FBRSxRQUFTLE1BQVM7QUFDekUsVUFBTUMsVUFBUyxLQUFLLE9BQU8sQ0FBQyxLQUFLRCxPQUFPLE1BQU1ELE1BQUssS0FBS0MsR0FBRSxNQUFNLElBQUlBLEdBQUUsUUFBUyxNQUFTO0FBQ3hGLFdBQU8sRUFBRSxRQUFRLFFBQUFDLFFBQU87QUFBQSxFQUM1QjtBQUNBLFdBQVMsU0FBU0MsV0FBVTtBQUN4QixXQUFPO0FBQUEsTUFDSCxRQUFRLENBQUMsV0FBVztBQUNoQixZQUFJLENBQUMsTUFBTSxRQUFRLE1BQU0sS0FBTSxPQUFPLFVBQVUsT0FBTyxPQUFPLENBQUMsTUFBTTtBQUNqRSxnQkFBTSxJQUFJLE1BQU0scURBQXFEO0FBQ3pFLGVBQU8sT0FBTyxJQUFJLENBQUNGLE9BQU07QUFDckIsdUJBQWFBLEVBQUM7QUFDZCxjQUFJQSxLQUFJLEtBQUtBLE1BQUtFLFVBQVM7QUFDdkIsa0JBQU0sSUFBSSxNQUFNLGlDQUFpQ0YsRUFBQyxlQUFlRSxVQUFTLE1BQU0sR0FBRztBQUN2RixpQkFBT0EsVUFBU0YsRUFBQztBQUFBLFFBQ3JCLENBQUM7QUFBQSxNQUNMO0FBQUEsTUFDQSxRQUFRLENBQUMsVUFBVTtBQUNmLFlBQUksQ0FBQyxNQUFNLFFBQVEsS0FBSyxLQUFNLE1BQU0sVUFBVSxPQUFPLE1BQU0sQ0FBQyxNQUFNO0FBQzlELGdCQUFNLElBQUksTUFBTSxrREFBa0Q7QUFDdEUsZUFBTyxNQUFNLElBQUksQ0FBQyxXQUFXO0FBQ3pCLGNBQUksT0FBTyxXQUFXO0FBQ2xCLGtCQUFNLElBQUksTUFBTSx1Q0FBdUMsTUFBTSxFQUFFO0FBQ25FLGdCQUFNLFFBQVFFLFVBQVMsUUFBUSxNQUFNO0FBQ3JDLGNBQUksVUFBVTtBQUNWLGtCQUFNLElBQUksTUFBTSxvQkFBb0IsTUFBTSxlQUFlQSxTQUFRLEVBQUU7QUFDdkUsaUJBQU87QUFBQSxRQUNYLENBQUM7QUFBQSxNQUNMO0FBQUEsSUFDSjtBQUFBLEVBQ0o7QUFDQSxXQUFTLEtBQUssWUFBWSxJQUFJO0FBQzFCLFFBQUksT0FBTyxjQUFjO0FBQ3JCLFlBQU0sSUFBSSxNQUFNLGlDQUFpQztBQUNyRCxXQUFPO0FBQUEsTUFDSCxRQUFRLENBQUMsU0FBUztBQUNkLFlBQUksQ0FBQyxNQUFNLFFBQVEsSUFBSSxLQUFNLEtBQUssVUFBVSxPQUFPLEtBQUssQ0FBQyxNQUFNO0FBQzNELGdCQUFNLElBQUksTUFBTSw4Q0FBOEM7QUFDbEUsaUJBQVNGLE1BQUs7QUFDVixjQUFJLE9BQU9BLE9BQU07QUFDYixrQkFBTSxJQUFJLE1BQU0saUNBQWlDQSxFQUFDLEVBQUU7QUFDNUQsZUFBTyxLQUFLLEtBQUssU0FBUztBQUFBLE1BQzlCO0FBQUEsTUFDQSxRQUFRLENBQUMsT0FBTztBQUNaLFlBQUksT0FBTyxPQUFPO0FBQ2QsZ0JBQU0sSUFBSSxNQUFNLG9DQUFvQztBQUN4RCxlQUFPLEdBQUcsTUFBTSxTQUFTO0FBQUEsTUFDN0I7QUFBQSxJQUNKO0FBQUEsRUFDSjtBQUNBLFdBQVMsUUFBUSxNQUFNLE1BQU0sS0FBSztBQUM5QixpQkFBYSxJQUFJO0FBQ2pCLFFBQUksT0FBTyxRQUFRO0FBQ2YsWUFBTSxJQUFJLE1BQU0sOEJBQThCO0FBQ2xELFdBQU87QUFBQSxNQUNILE9BQU8sTUFBTTtBQUNULFlBQUksQ0FBQyxNQUFNLFFBQVEsSUFBSSxLQUFNLEtBQUssVUFBVSxPQUFPLEtBQUssQ0FBQyxNQUFNO0FBQzNELGdCQUFNLElBQUksTUFBTSxpREFBaUQ7QUFDckUsaUJBQVNBLE1BQUs7QUFDVixjQUFJLE9BQU9BLE9BQU07QUFDYixrQkFBTSxJQUFJLE1BQU0sb0NBQW9DQSxFQUFDLEVBQUU7QUFDL0QsZUFBUSxLQUFLLFNBQVMsT0FBUTtBQUMxQixlQUFLLEtBQUssR0FBRztBQUNqQixlQUFPO0FBQUEsTUFDWDtBQUFBLE1BQ0EsT0FBTyxPQUFPO0FBQ1YsWUFBSSxDQUFDLE1BQU0sUUFBUSxLQUFLLEtBQU0sTUFBTSxVQUFVLE9BQU8sTUFBTSxDQUFDLE1BQU07QUFDOUQsZ0JBQU0sSUFBSSxNQUFNLGlEQUFpRDtBQUNyRSxpQkFBU0EsTUFBSztBQUNWLGNBQUksT0FBT0EsT0FBTTtBQUNiLGtCQUFNLElBQUksTUFBTSxvQ0FBb0NBLEVBQUMsRUFBRTtBQUMvRCxZQUFJLE1BQU0sTUFBTTtBQUNoQixZQUFLLE1BQU0sT0FBUTtBQUNmLGdCQUFNLElBQUksTUFBTSwyREFBMkQ7QUFDL0UsZUFBTyxNQUFNLEtBQUssTUFBTSxNQUFNLENBQUMsTUFBTSxLQUFLLE9BQU87QUFDN0MsY0FBSSxHQUFJLE1BQU0sS0FBSyxPQUFRO0FBQ3ZCLGtCQUFNLElBQUksTUFBTSw4Q0FBOEM7QUFBQSxRQUN0RTtBQUNBLGVBQU8sTUFBTSxNQUFNLEdBQUcsR0FBRztBQUFBLE1BQzdCO0FBQUEsSUFDSjtBQUFBLEVBQ0o7QUFDQSxXQUFTLFVBQVUsSUFBSTtBQUNuQixRQUFJLE9BQU8sT0FBTztBQUNkLFlBQU0sSUFBSSxNQUFNLGlDQUFpQztBQUNyRCxXQUFPLEVBQUUsUUFBUSxDQUFDLFNBQVMsTUFBTSxRQUFRLENBQUMsT0FBTyxHQUFHLEVBQUUsRUFBRTtBQUFBLEVBQzVEO0FBQ0EsV0FBUyxhQUFhLE1BQU0sTUFBTSxJQUFJO0FBQ2xDLFFBQUksT0FBTztBQUNQLFlBQU0sSUFBSSxNQUFNLDRCQUE0QixJQUFJLDhCQUE4QjtBQUNsRixRQUFJLEtBQUs7QUFDTCxZQUFNLElBQUksTUFBTSwwQkFBMEIsRUFBRSw4QkFBOEI7QUFDOUUsUUFBSSxDQUFDLE1BQU0sUUFBUSxJQUFJO0FBQ25CLFlBQU0sSUFBSSxNQUFNLG9DQUFvQztBQUN4RCxRQUFJLENBQUMsS0FBSztBQUNOLGFBQU8sQ0FBQztBQUNaLFFBQUksTUFBTTtBQUNWLFVBQU0sTUFBTSxDQUFDO0FBQ2IsVUFBTSxTQUFTLE1BQU0sS0FBSyxJQUFJO0FBQzlCLFdBQU8sUUFBUSxDQUFDLE1BQU07QUFDbEIsbUJBQWEsQ0FBQztBQUNkLFVBQUksSUFBSSxLQUFLLEtBQUs7QUFDZCxjQUFNLElBQUksTUFBTSxrQkFBa0IsQ0FBQyxFQUFFO0FBQUEsSUFDN0MsQ0FBQztBQUNELFdBQU8sTUFBTTtBQUNULFVBQUksUUFBUTtBQUNaLFVBQUksT0FBTztBQUNYLGVBQVNBLEtBQUksS0FBS0EsS0FBSSxPQUFPLFFBQVFBLE1BQUs7QUFDdEMsY0FBTSxRQUFRLE9BQU9BLEVBQUM7QUFDdEIsY0FBTSxZQUFZLE9BQU8sUUFBUTtBQUNqQyxZQUFJLENBQUMsT0FBTyxjQUFjLFNBQVMsS0FDOUIsT0FBTyxRQUFTLFNBQVMsU0FDMUIsWUFBWSxVQUFVLE9BQU8sT0FBTztBQUNwQyxnQkFBTSxJQUFJLE1BQU0sOEJBQThCO0FBQUEsUUFDbEQ7QUFDQSxnQkFBUSxZQUFZO0FBQ3BCLGVBQU9BLEVBQUMsSUFBSSxLQUFLLE1BQU0sWUFBWSxFQUFFO0FBQ3JDLFlBQUksQ0FBQyxPQUFPLGNBQWMsT0FBT0EsRUFBQyxDQUFDLEtBQUssT0FBT0EsRUFBQyxJQUFJLEtBQUssVUFBVTtBQUMvRCxnQkFBTSxJQUFJLE1BQU0sOEJBQThCO0FBQ2xELFlBQUksQ0FBQztBQUNEO0FBQUEsaUJBQ0ssQ0FBQyxPQUFPQSxFQUFDO0FBQ2QsZ0JBQU1BO0FBQUE7QUFFTixpQkFBTztBQUFBLE1BQ2Y7QUFDQSxVQUFJLEtBQUssS0FBSztBQUNkLFVBQUk7QUFDQTtBQUFBLElBQ1I7QUFDQSxhQUFTQSxLQUFJLEdBQUdBLEtBQUksS0FBSyxTQUFTLEtBQUssS0FBS0EsRUFBQyxNQUFNLEdBQUdBO0FBQ2xELFVBQUksS0FBSyxDQUFDO0FBQ2QsV0FBTyxJQUFJLFFBQVE7QUFBQSxFQUN2QjtBQUNBLE1BQU0sTUFBTSxDQUFDLEdBQUcsTUFBTyxDQUFDLElBQUksSUFBSSxJQUFJLEdBQUcsSUFBSSxDQUFDO0FBQzVDLE1BQU0sY0FBYyxDQUFDLE1BQU0sT0FBTyxRQUFRLEtBQUssSUFBSSxNQUFNLEVBQUU7QUFDM0QsV0FBUyxjQUFjLE1BQU0sTUFBTSxJQUFJRyxVQUFTO0FBQzVDLFFBQUksQ0FBQyxNQUFNLFFBQVEsSUFBSTtBQUNuQixZQUFNLElBQUksTUFBTSxxQ0FBcUM7QUFDekQsUUFBSSxRQUFRLEtBQUssT0FBTztBQUNwQixZQUFNLElBQUksTUFBTSw2QkFBNkIsSUFBSSxFQUFFO0FBQ3ZELFFBQUksTUFBTSxLQUFLLEtBQUs7QUFDaEIsWUFBTSxJQUFJLE1BQU0sMkJBQTJCLEVBQUUsRUFBRTtBQUNuRCxRQUFJLFlBQVksTUFBTSxFQUFFLElBQUksSUFBSTtBQUM1QixZQUFNLElBQUksTUFBTSxzQ0FBc0MsSUFBSSxPQUFPLEVBQUUsY0FBYyxZQUFZLE1BQU0sRUFBRSxDQUFDLEVBQUU7QUFBQSxJQUM1RztBQUNBLFFBQUksUUFBUTtBQUNaLFFBQUksTUFBTTtBQUNWLFVBQU0sT0FBTyxLQUFLLEtBQUs7QUFDdkIsVUFBTSxNQUFNLENBQUM7QUFDYixlQUFXLEtBQUssTUFBTTtBQUNsQixtQkFBYSxDQUFDO0FBQ2QsVUFBSSxLQUFLLEtBQUs7QUFDVixjQUFNLElBQUksTUFBTSxvQ0FBb0MsQ0FBQyxTQUFTLElBQUksRUFBRTtBQUN4RSxjQUFTLFNBQVMsT0FBUTtBQUMxQixVQUFJLE1BQU0sT0FBTztBQUNiLGNBQU0sSUFBSSxNQUFNLHFDQUFxQyxHQUFHLFNBQVMsSUFBSSxFQUFFO0FBQzNFLGFBQU87QUFDUCxhQUFPLE9BQU8sSUFBSSxPQUFPO0FBQ3JCLFlBQUksTUFBTyxTQUFVLE1BQU0sS0FBTyxVQUFVLENBQUM7QUFDakQsZUFBUyxLQUFLLE1BQU07QUFBQSxJQUN4QjtBQUNBLFlBQVMsU0FBVSxLQUFLLE1BQVE7QUFDaEMsUUFBSSxDQUFDQSxZQUFXLE9BQU87QUFDbkIsWUFBTSxJQUFJLE1BQU0sZ0JBQWdCO0FBQ3BDLFFBQUksQ0FBQ0EsWUFBVztBQUNaLFlBQU0sSUFBSSxNQUFNLHFCQUFxQixLQUFLLEVBQUU7QUFDaEQsUUFBSUEsWUFBVyxNQUFNO0FBQ2pCLFVBQUksS0FBSyxVQUFVLENBQUM7QUFDeEIsV0FBTztBQUFBLEVBQ1g7QUFDQSxXQUFTLE1BQU0sS0FBSztBQUNoQixpQkFBYSxHQUFHO0FBQ2hCLFdBQU87QUFBQSxNQUNILFFBQVEsQ0FBQ0MsV0FBVTtBQUNmLFlBQUksRUFBRUEsa0JBQWlCO0FBQ25CLGdCQUFNLElBQUksTUFBTSx5Q0FBeUM7QUFDN0QsZUFBTyxhQUFhLE1BQU0sS0FBS0EsTUFBSyxHQUFHLEtBQUssR0FBRyxHQUFHO0FBQUEsTUFDdEQ7QUFBQSxNQUNBLFFBQVEsQ0FBQyxXQUFXO0FBQ2hCLFlBQUksQ0FBQyxNQUFNLFFBQVEsTUFBTSxLQUFNLE9BQU8sVUFBVSxPQUFPLE9BQU8sQ0FBQyxNQUFNO0FBQ2pFLGdCQUFNLElBQUksTUFBTSwrQ0FBK0M7QUFDbkUsZUFBTyxXQUFXLEtBQUssYUFBYSxRQUFRLEtBQUssS0FBSyxDQUFDLENBQUM7QUFBQSxNQUM1RDtBQUFBLElBQ0o7QUFBQSxFQUNKO0FBQ0EsV0FBUyxPQUFPLE1BQU0sYUFBYSxPQUFPO0FBQ3RDLGlCQUFhLElBQUk7QUFDakIsUUFBSSxRQUFRLEtBQUssT0FBTztBQUNwQixZQUFNLElBQUksTUFBTSxtQ0FBbUM7QUFDdkQsUUFBSSxZQUFZLEdBQUcsSUFBSSxJQUFJLE1BQU0sWUFBWSxNQUFNLENBQUMsSUFBSTtBQUNwRCxZQUFNLElBQUksTUFBTSx3QkFBd0I7QUFDNUMsV0FBTztBQUFBLE1BQ0gsUUFBUSxDQUFDQSxXQUFVO0FBQ2YsWUFBSSxFQUFFQSxrQkFBaUI7QUFDbkIsZ0JBQU0sSUFBSSxNQUFNLDBDQUEwQztBQUM5RCxlQUFPLGNBQWMsTUFBTSxLQUFLQSxNQUFLLEdBQUcsR0FBRyxNQUFNLENBQUMsVUFBVTtBQUFBLE1BQ2hFO0FBQUEsTUFDQSxRQUFRLENBQUMsV0FBVztBQUNoQixZQUFJLENBQUMsTUFBTSxRQUFRLE1BQU0sS0FBTSxPQUFPLFVBQVUsT0FBTyxPQUFPLENBQUMsTUFBTTtBQUNqRSxnQkFBTSxJQUFJLE1BQU0sZ0RBQWdEO0FBQ3BFLGVBQU8sV0FBVyxLQUFLLGNBQWMsUUFBUSxNQUFNLEdBQUcsVUFBVSxDQUFDO0FBQUEsTUFDckU7QUFBQSxJQUNKO0FBQUEsRUFDSjtBQUNBLFdBQVMsY0FBYyxJQUFJO0FBQ3ZCLFFBQUksT0FBTyxPQUFPO0FBQ2QsWUFBTSxJQUFJLE1BQU0scUNBQXFDO0FBQ3pELFdBQU8sWUFBYSxNQUFNO0FBQ3RCLFVBQUk7QUFDQSxlQUFPLEdBQUcsTUFBTSxNQUFNLElBQUk7QUFBQSxNQUM5QixTQUNPLEdBQUc7QUFBQSxNQUFFO0FBQUEsSUFDaEI7QUFBQSxFQUNKO0FBNkJPLE1BQU0sU0FBUyxNQUFNLE9BQU8sQ0FBQyxHQUFHLFNBQVMsa0JBQWtCLEdBQUcsS0FBSyxFQUFFLENBQUM7QUFDdEUsTUFBTSxTQUFTLE1BQU0sT0FBTyxDQUFDLEdBQUcsU0FBUyxrQ0FBa0MsR0FBRyxRQUFRLENBQUMsR0FBRyxLQUFLLEVBQUUsQ0FBQztBQUNsRyxNQUFNLFlBQVksTUFBTSxPQUFPLENBQUMsR0FBRyxTQUFTLGtDQUFrQyxHQUFHLFFBQVEsQ0FBQyxHQUFHLEtBQUssRUFBRSxDQUFDO0FBQ3JHLE1BQU0sa0JBQWtCLE1BQU0sT0FBTyxDQUFDLEdBQUcsU0FBUyxrQ0FBa0MsR0FBRyxLQUFLLEVBQUUsR0FBRyxVQUFVLENBQUMsTUFBTSxFQUFFLFlBQVksRUFBRSxRQUFRLE1BQU0sR0FBRyxFQUFFLFFBQVEsU0FBUyxHQUFHLENBQUMsQ0FBQztBQUMzSyxNQUFNLFNBQVMsTUFBTSxPQUFPLENBQUMsR0FBRyxTQUFTLGtFQUFrRSxHQUFHLFFBQVEsQ0FBQyxHQUFHLEtBQUssRUFBRSxDQUFDO0FBQ2xJLE1BQU0sWUFBWSxNQUFNLE9BQU8sQ0FBQyxHQUFHLFNBQVMsa0VBQWtFLEdBQUcsUUFBUSxDQUFDLEdBQUcsS0FBSyxFQUFFLENBQUM7QUFDNUksTUFBTSxZQUFZLENBQUMsUUFBUSxNQUFNLE1BQU0sRUFBRSxHQUFHLFNBQVMsR0FBRyxHQUFHLEtBQUssRUFBRSxDQUFDO0FBQzVELE1BQU0sU0FBUyxVQUFVLDREQUE0RDtBQUNyRixNQUFNLGVBQWUsVUFBVSw0REFBNEQ7QUFDM0YsTUFBTSxZQUFZLFVBQVUsNERBQTREO0FBQy9GLE1BQU0sZ0JBQWdCLENBQUMsR0FBRyxHQUFHLEdBQUcsR0FBRyxHQUFHLEdBQUcsR0FBRyxJQUFJLEVBQUU7QUFDM0MsTUFBTSxZQUFZO0FBQUEsSUFDckIsT0FBTyxNQUFNO0FBQ1QsVUFBSSxNQUFNO0FBQ1YsZUFBU0MsS0FBSSxHQUFHQSxLQUFJLEtBQUssUUFBUUEsTUFBSyxHQUFHO0FBQ3JDLGNBQU0sUUFBUSxLQUFLLFNBQVNBLElBQUdBLEtBQUksQ0FBQztBQUNwQyxlQUFPLE9BQU8sT0FBTyxLQUFLLEVBQUUsU0FBUyxjQUFjLE1BQU0sTUFBTSxHQUFHLEdBQUc7QUFBQSxNQUN6RTtBQUNBLGFBQU87QUFBQSxJQUNYO0FBQUEsSUFDQSxPQUFPLEtBQUs7QUFDUixVQUFJLE1BQU0sQ0FBQztBQUNYLGVBQVNBLEtBQUksR0FBR0EsS0FBSSxJQUFJLFFBQVFBLE1BQUssSUFBSTtBQUNyQyxjQUFNLFFBQVEsSUFBSSxNQUFNQSxJQUFHQSxLQUFJLEVBQUU7QUFDakMsY0FBTSxXQUFXLGNBQWMsUUFBUSxNQUFNLE1BQU07QUFDbkQsY0FBTSxRQUFRLE9BQU8sT0FBTyxLQUFLO0FBQ2pDLGlCQUFTLElBQUksR0FBRyxJQUFJLE1BQU0sU0FBUyxVQUFVLEtBQUs7QUFDOUMsY0FBSSxNQUFNLENBQUMsTUFBTTtBQUNiLGtCQUFNLElBQUksTUFBTSwwQkFBMEI7QUFBQSxRQUNsRDtBQUNBLGNBQU0sSUFBSSxPQUFPLE1BQU0sS0FBSyxNQUFNLE1BQU0sTUFBTSxTQUFTLFFBQVEsQ0FBQyxDQUFDO0FBQUEsTUFDckU7QUFDQSxhQUFPLFdBQVcsS0FBSyxHQUFHO0FBQUEsSUFDOUI7QUFBQSxFQUNKO0FBRUEsTUFBTSxnQkFBZ0IsTUFBTSxTQUFTLGtDQUFrQyxHQUFHLEtBQUssRUFBRSxDQUFDO0FBQ2xGLE1BQU0scUJBQXFCLENBQUMsV0FBWSxXQUFZLFdBQVksWUFBWSxTQUFVO0FBQ3RGLFdBQVMsY0FBYyxLQUFLO0FBQ3hCLFVBQU0sSUFBSSxPQUFPO0FBQ2pCLFFBQUksT0FBTyxNQUFNLGFBQWM7QUFDL0IsYUFBU0MsS0FBSSxHQUFHQSxLQUFJLG1CQUFtQixRQUFRQSxNQUFLO0FBQ2hELFdBQU0sS0FBS0EsS0FBSyxPQUFPO0FBQ25CLGVBQU8sbUJBQW1CQSxFQUFDO0FBQUEsSUFDbkM7QUFDQSxXQUFPO0FBQUEsRUFDWDtBQUNBLFdBQVMsYUFBYSxRQUFRLE9BQU8sZ0JBQWdCLEdBQUc7QUFDcEQsVUFBTSxNQUFNLE9BQU87QUFDbkIsUUFBSSxNQUFNO0FBQ1YsYUFBU0EsS0FBSSxHQUFHQSxLQUFJLEtBQUtBLE1BQUs7QUFDMUIsWUFBTSxJQUFJLE9BQU8sV0FBV0EsRUFBQztBQUM3QixVQUFJLElBQUksTUFBTSxJQUFJO0FBQ2QsY0FBTSxJQUFJLE1BQU0sbUJBQW1CLE1BQU0sR0FBRztBQUNoRCxZQUFNLGNBQWMsR0FBRyxJQUFLLEtBQUs7QUFBQSxJQUNyQztBQUNBLFVBQU0sY0FBYyxHQUFHO0FBQ3ZCLGFBQVNBLEtBQUksR0FBR0EsS0FBSSxLQUFLQTtBQUNyQixZQUFNLGNBQWMsR0FBRyxJQUFLLE9BQU8sV0FBV0EsRUFBQyxJQUFJO0FBQ3ZELGFBQVMsS0FBSztBQUNWLFlBQU0sY0FBYyxHQUFHLElBQUk7QUFDL0IsYUFBU0EsS0FBSSxHQUFHQSxLQUFJLEdBQUdBO0FBQ25CLFlBQU0sY0FBYyxHQUFHO0FBQzNCLFdBQU87QUFDUCxXQUFPLGNBQWMsT0FBTyxjQUFjLENBQUMsTUFBTSxLQUFLLEVBQUUsR0FBRyxJQUFJLEdBQUcsS0FBSyxDQUFDO0FBQUEsRUFDNUU7QUFDQSxXQUFTLFVBQVUsVUFBVTtBQUN6QixVQUFNLGlCQUFpQixhQUFhLFdBQVcsSUFBSTtBQUNuRCxVQUFNLFNBQVMsT0FBTyxDQUFDO0FBQ3ZCLFVBQU0sWUFBWSxPQUFPO0FBQ3pCLFVBQU0sVUFBVSxPQUFPO0FBQ3ZCLFVBQU0sa0JBQWtCLGNBQWMsU0FBUztBQUMvQyxhQUFTLE9BQU8sUUFBUSxPQUFPQyxTQUFRLElBQUk7QUFDdkMsVUFBSSxPQUFPLFdBQVc7QUFDbEIsY0FBTSxJQUFJLE1BQU0sOENBQThDLE9BQU8sTUFBTSxFQUFFO0FBQ2pGLFVBQUksQ0FBQyxNQUFNLFFBQVEsS0FBSyxLQUFNLE1BQU0sVUFBVSxPQUFPLE1BQU0sQ0FBQyxNQUFNO0FBQzlELGNBQU0sSUFBSSxNQUFNLHVEQUF1RCxPQUFPLEtBQUssRUFBRTtBQUN6RixZQUFNLGVBQWUsT0FBTyxTQUFTLElBQUksTUFBTTtBQUMvQyxVQUFJQSxXQUFVLFNBQVMsZUFBZUE7QUFDbEMsY0FBTSxJQUFJLFVBQVUsVUFBVSxZQUFZLGtCQUFrQkEsTUFBSyxFQUFFO0FBQ3ZFLGVBQVMsT0FBTyxZQUFZO0FBQzVCLGFBQU8sR0FBRyxNQUFNLElBQUksY0FBYyxPQUFPLEtBQUssQ0FBQyxHQUFHLGFBQWEsUUFBUSxPQUFPLGNBQWMsQ0FBQztBQUFBLElBQ2pHO0FBQ0EsYUFBU0MsUUFBTyxLQUFLRCxTQUFRLElBQUk7QUFDN0IsVUFBSSxPQUFPLFFBQVE7QUFDZixjQUFNLElBQUksTUFBTSw2Q0FBNkMsT0FBTyxHQUFHLEVBQUU7QUFDN0UsVUFBSSxJQUFJLFNBQVMsS0FBTUEsV0FBVSxTQUFTLElBQUksU0FBU0E7QUFDbkQsY0FBTSxJQUFJLFVBQVUsd0JBQXdCLElBQUksTUFBTSxLQUFLLEdBQUcsbUJBQW1CQSxNQUFLLEdBQUc7QUFDN0YsWUFBTSxVQUFVLElBQUksWUFBWTtBQUNoQyxVQUFJLFFBQVEsV0FBVyxRQUFRLElBQUksWUFBWTtBQUMzQyxjQUFNLElBQUksTUFBTSx1Q0FBdUM7QUFDM0QsWUFBTTtBQUNOLFlBQU0sV0FBVyxJQUFJLFlBQVksR0FBRztBQUNwQyxVQUFJLGFBQWEsS0FBSyxhQUFhO0FBQy9CLGNBQU0sSUFBSSxNQUFNLHlEQUF5RDtBQUM3RSxZQUFNLFNBQVMsSUFBSSxNQUFNLEdBQUcsUUFBUTtBQUNwQyxZQUFNRSxVQUFTLElBQUksTUFBTSxXQUFXLENBQUM7QUFDckMsVUFBSUEsUUFBTyxTQUFTO0FBQ2hCLGNBQU0sSUFBSSxNQUFNLHlDQUF5QztBQUM3RCxZQUFNLFFBQVEsY0FBYyxPQUFPQSxPQUFNLEVBQUUsTUFBTSxHQUFHLEVBQUU7QUFDdEQsWUFBTSxNQUFNLGFBQWEsUUFBUSxPQUFPLGNBQWM7QUFDdEQsVUFBSSxDQUFDQSxRQUFPLFNBQVMsR0FBRztBQUNwQixjQUFNLElBQUksTUFBTSx1QkFBdUIsR0FBRyxlQUFlLEdBQUcsR0FBRztBQUNuRSxhQUFPLEVBQUUsUUFBUSxNQUFNO0FBQUEsSUFDM0I7QUFDQSxVQUFNLGVBQWUsY0FBY0QsT0FBTTtBQUN6QyxhQUFTLGNBQWMsS0FBSztBQUN4QixZQUFNLEVBQUUsUUFBUSxNQUFNLElBQUlBLFFBQU8sS0FBSyxLQUFLO0FBQzNDLGFBQU8sRUFBRSxRQUFRLE9BQU8sT0FBTyxVQUFVLEtBQUssRUFBRTtBQUFBLElBQ3BEO0FBQ0EsV0FBTyxFQUFFLFFBQVEsUUFBQUEsU0FBUSxlQUFlLGNBQWMsV0FBVyxpQkFBaUIsUUFBUTtBQUFBLEVBQzlGO0FBQ08sTUFBTSxTQUFTLFVBQVUsUUFBUTtBQUNqQyxNQUFNLFVBQVUsVUFBVSxTQUFTO0FBQ25DLE1BQU0sT0FBTztBQUFBLElBQ2hCLFFBQVEsQ0FBQyxTQUFTLElBQUksWUFBWSxFQUFFLE9BQU8sSUFBSTtBQUFBLElBQy9DLFFBQVEsQ0FBQyxRQUFRLElBQUksWUFBWSxFQUFFLE9BQU8sR0FBRztBQUFBLEVBQ2pEO0FBQ08sTUFBTSxNQUFNLE1BQU0sT0FBTyxDQUFDLEdBQUcsU0FBUyxrQkFBa0IsR0FBRyxLQUFLLEVBQUUsR0FBRyxVQUFVLENBQUMsTUFBTTtBQUN6RixRQUFJLE9BQU8sTUFBTSxZQUFZLEVBQUUsU0FBUztBQUNwQyxZQUFNLElBQUksVUFBVSxvQ0FBb0MsT0FBTyxDQUFDLGdCQUFnQixFQUFFLE1BQU0sRUFBRTtBQUM5RixXQUFPLEVBQUUsWUFBWTtBQUFBLEVBQ3pCLENBQUMsQ0FBQztBQUNGLE1BQU0sU0FBUztBQUFBLElBQ1g7QUFBQSxJQUFNO0FBQUEsSUFBSztBQUFBLElBQVE7QUFBQSxJQUFRO0FBQUEsSUFBUTtBQUFBLElBQVc7QUFBQSxJQUFRO0FBQUEsRUFDMUQ7QUFDQSxNQUFNLGlCQUFpQiwyQ0FBMkMsT0FBTyxLQUFLLE1BQU0sRUFBRSxLQUFLLElBQUksQ0FBQzs7O0FDelhoRyxXQUFTRSxRQUFPLEdBQVM7QUFDdkIsUUFBSSxDQUFDLE9BQU8sY0FBYyxDQUFDLEtBQUssSUFBSTtBQUFHLFlBQU0sSUFBSSxNQUFNLGtDQUFrQyxDQUFDLEVBQUU7RUFDOUY7QUFFQSxXQUFTQyxNQUFLLEdBQVU7QUFDdEIsUUFBSSxPQUFPLE1BQU07QUFBVyxZQUFNLElBQUksTUFBTSx5QkFBeUIsQ0FBQyxFQUFFO0VBQzFFO0FBRU0sV0FBVSxRQUFRLEdBQVU7QUFDaEMsV0FDRSxhQUFhLGNBQ1osS0FBSyxRQUFRLE9BQU8sTUFBTSxZQUFZLEVBQUUsWUFBWSxTQUFTO0VBRWxFO0FBRUEsV0FBU0MsT0FBTSxNQUE4QixTQUFpQjtBQUM1RCxRQUFJLENBQUMsUUFBUSxDQUFDO0FBQUcsWUFBTSxJQUFJLE1BQU0scUJBQXFCO0FBQ3RELFFBQUksUUFBUSxTQUFTLEtBQUssQ0FBQyxRQUFRLFNBQVMsRUFBRSxNQUFNO0FBQ2xELFlBQU0sSUFBSSxNQUFNLGlDQUFpQyxPQUFPLG1CQUFtQixFQUFFLE1BQU0sRUFBRTtFQUN6RjtBQWVBLFdBQVNDLFFBQU8sVUFBZSxnQkFBZ0IsTUFBSTtBQUNqRCxRQUFJLFNBQVM7QUFBVyxZQUFNLElBQUksTUFBTSxrQ0FBa0M7QUFDMUUsUUFBSSxpQkFBaUIsU0FBUztBQUFVLFlBQU0sSUFBSSxNQUFNLHVDQUF1QztFQUNqRztBQUVBLFdBQVNDLFFBQU8sS0FBVSxVQUFhO0FBQ3JDLElBQUFDLE9BQU0sR0FBRztBQUNULFVBQU0sTUFBTSxTQUFTO0FBQ3JCLFFBQUksSUFBSSxTQUFTLEtBQUs7QUFDcEIsWUFBTSxJQUFJLE1BQU0seURBQXlELEdBQUcsRUFBRTtJQUNoRjtFQUNGOzs7QUN0Q08sTUFBTSxLQUFLLENBQUMsUUFBb0IsSUFBSSxXQUFXLElBQUksUUFBUSxJQUFJLFlBQVksSUFBSSxVQUFVO0FBR3pGLE1BQU0sTUFBTSxDQUFDLFFBQ2xCLElBQUksWUFBWSxJQUFJLFFBQVEsSUFBSSxZQUFZLEtBQUssTUFBTSxJQUFJLGFBQWEsQ0FBQyxDQUFDO0FBR3JFLE1BQU1DLGNBQWEsQ0FBQyxRQUN6QixJQUFJLFNBQVMsSUFBSSxRQUFRLElBQUksWUFBWSxJQUFJLFVBQVU7QUFJbEQsTUFBTUMsUUFBTyxJQUFJLFdBQVcsSUFBSSxZQUFZLENBQUMsU0FBVSxDQUFDLEVBQUUsTUFBTSxFQUFFLENBQUMsTUFBTTtBQUNoRixNQUFJLENBQUNBO0FBQU0sVUFBTSxJQUFJLE1BQU0sNkNBQTZDO0FBMEZsRSxXQUFVQyxhQUFZLEtBQVc7QUFDckMsUUFBSSxPQUFPLFFBQVE7QUFBVSxZQUFNLElBQUksTUFBTSx3QkFBd0IsT0FBTyxHQUFHLEVBQUU7QUFDakYsV0FBTyxJQUFJLFdBQVcsSUFBSSxZQUFXLEVBQUcsT0FBTyxHQUFHLENBQUM7RUFDckQ7QUFlTSxXQUFVQyxTQUFRLE1BQVc7QUFDakMsUUFBSSxPQUFPLFNBQVM7QUFBVSxhQUFPQyxhQUFZLElBQUk7YUFDNUMsUUFBUSxJQUFJO0FBQUcsYUFBTyxLQUFLLE1BQUs7O0FBQ3BDLFlBQU0sSUFBSSxNQUFNLDRCQUE0QixPQUFPLElBQUksRUFBRTtBQUM5RCxXQUFPO0VBQ1Q7QUFzQk0sV0FBVSxVQUNkLFVBQ0EsTUFBUTtBQUVSLFFBQUksUUFBUSxRQUFRLE9BQU8sU0FBUztBQUFVLFlBQU0sSUFBSSxNQUFNLHlCQUF5QjtBQUN2RixVQUFNLFNBQVMsT0FBTyxPQUFPLFVBQVUsSUFBSTtBQUMzQyxXQUFPO0VBQ1Q7QUFHTSxXQUFVQyxZQUFXLEdBQWUsR0FBYTtBQUNyRCxRQUFJLEVBQUUsV0FBVyxFQUFFO0FBQVEsYUFBTztBQUNsQyxRQUFJLE9BQU87QUFDWCxhQUFTQyxLQUFJLEdBQUdBLEtBQUksRUFBRSxRQUFRQTtBQUFLLGNBQVEsRUFBRUEsRUFBQyxJQUFJLEVBQUVBLEVBQUM7QUFDckQsV0FBTyxTQUFTO0VBQ2xCO0FBMENPLE1BQU0sd0NBQWEsQ0FDeEIsUUFDQSxNQUNTO0FBQ1QsV0FBTyxPQUFPLEdBQUcsTUFBTTtBQUN2QixXQUFPO0VBQ1Q7QUFXTSxXQUFVQyxjQUNkLE1BQ0EsWUFDQSxPQUNBQyxPQUFhO0FBRWIsUUFBSSxPQUFPLEtBQUssaUJBQWlCO0FBQVksYUFBTyxLQUFLLGFBQWEsWUFBWSxPQUFPQSxLQUFJO0FBQzdGLFVBQU0sT0FBTyxPQUFPLEVBQUU7QUFDdEIsVUFBTSxXQUFXLE9BQU8sVUFBVTtBQUNsQyxVQUFNLEtBQUssT0FBUSxTQUFTLE9BQVEsUUFBUTtBQUM1QyxVQUFNLEtBQUssT0FBTyxRQUFRLFFBQVE7QUFDbEMsVUFBTSxJQUFJQSxRQUFPLElBQUk7QUFDckIsVUFBTSxJQUFJQSxRQUFPLElBQUk7QUFDckIsU0FBSyxVQUFVLGFBQWEsR0FBRyxJQUFJQSxLQUFJO0FBQ3ZDLFNBQUssVUFBVSxhQUFhLEdBQUcsSUFBSUEsS0FBSTtFQUN6Qzs7O0FDek9BLE1BQU0sYUFBYTtBQUduQixNQUFNLFVBQTBCLG9CQUFJLFdBQVcsRUFBRTtBQUNqRCxNQUFNLFVBQVUsSUFBSSxPQUFPO0FBQzNCLE1BQU0sT0FBTztBQUtiLE1BQU0sT0FBTyxDQUFDLElBQVksSUFBWSxJQUFZLE9BQWM7QUFDOUQsVUFBTSxRQUFRLEtBQUs7QUFDbkIsV0FBTztNQUNMLElBQUssTUFBTSxLQUFPLE9BQU87TUFDekIsSUFBSyxNQUFNLEtBQU8sT0FBTztNQUN6QixJQUFLLE1BQU0sS0FBTyxPQUFPO01BQ3pCLElBQUssT0FBTyxJQUFPLFFBQVEsS0FBTSxFQUFFLFFBQVE7OztFQUUvQztBQUVBLE1BQU0sU0FBUyxDQUFDLE9BQ1gsTUFBTSxJQUFLLFFBQVMsTUFDcEIsTUFBTSxJQUFLLFFBQVMsTUFDcEIsTUFBTSxLQUFNLFFBQVMsSUFDdEIsTUFBTSxLQUFNLE1BQ2Q7QUFNSSxXQUFVLFlBQVksR0FBYTtBQUN2QyxNQUFFLFFBQU87QUFDVCxVQUFNLFFBQVEsRUFBRSxFQUFFLElBQUk7QUFFdEIsUUFBSSxRQUFRO0FBQ1osYUFBU0MsS0FBSSxHQUFHQSxLQUFJLEVBQUUsUUFBUUEsTUFBSztBQUNqQyxZQUFNLElBQUksRUFBRUEsRUFBQztBQUNiLFFBQUVBLEVBQUMsSUFBSyxNQUFNLElBQUs7QUFDbkIsZUFBUyxJQUFJLE1BQU07SUFDckI7QUFDQSxNQUFFLENBQUMsS0FBSyxDQUFDLFFBQVE7QUFDakIsV0FBTztFQUNUO0FBSUEsTUFBTSxpQkFBaUIsQ0FBQ0MsV0FBaUI7QUFDdkMsUUFBSUEsU0FBUSxLQUFLO0FBQU0sYUFBTztBQUM5QixRQUFJQSxTQUFRO0FBQU0sYUFBTztBQUN6QixXQUFPO0VBQ1Q7QUFFQSxNQUFNLFFBQU4sTUFBVzs7SUFZVCxZQUFZLEtBQVksZ0JBQXVCO0FBWHRDLFdBQUEsV0FBVztBQUNYLFdBQUEsWUFBWTtBQUNYLFdBQUEsS0FBSztBQUNMLFdBQUEsS0FBSztBQUNMLFdBQUEsS0FBSztBQUNMLFdBQUEsS0FBSztBQUNMLFdBQUEsV0FBVztBQU1uQixZQUFNQyxTQUFRLEdBQUc7QUFDakIsTUFBQUQsT0FBTyxLQUFLLEVBQUU7QUFDZCxZQUFNLFFBQVFFLFlBQVcsR0FBRztBQUM1QixVQUFJLEtBQUssTUFBTSxVQUFVLEdBQUcsS0FBSztBQUNqQyxVQUFJLEtBQUssTUFBTSxVQUFVLEdBQUcsS0FBSztBQUNqQyxVQUFJLEtBQUssTUFBTSxVQUFVLEdBQUcsS0FBSztBQUNqQyxVQUFJLEtBQUssTUFBTSxVQUFVLElBQUksS0FBSztBQUVsQyxZQUFNLFVBQW1CLENBQUE7QUFDekIsZUFBU0gsS0FBSSxHQUFHQSxLQUFJLEtBQUtBLE1BQUs7QUFDNUIsZ0JBQVEsS0FBSyxFQUFFLElBQUksT0FBTyxFQUFFLEdBQUcsSUFBSSxPQUFPLEVBQUUsR0FBRyxJQUFJLE9BQU8sRUFBRSxHQUFHLElBQUksT0FBTyxFQUFFLEVBQUMsQ0FBRTtBQUMvRSxTQUFDLEVBQUUsSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssS0FBSyxJQUFJLElBQUksSUFBSSxFQUFFO01BQzNEO0FBQ0EsWUFBTSxJQUFJLGVBQWUsa0JBQWtCLElBQUk7QUFDL0MsVUFBSSxDQUFDLENBQUMsR0FBRyxHQUFHLEdBQUcsQ0FBQyxFQUFFLFNBQVMsQ0FBQztBQUMxQixjQUFNLElBQUksTUFBTSw0QkFBNEIsQ0FBQyx1QkFBdUI7QUFDdEUsV0FBSyxJQUFJO0FBQ1QsWUFBTSxPQUFPO0FBQ2IsWUFBTSxVQUFVLE9BQU87QUFDdkIsWUFBTSxhQUFjLEtBQUssYUFBYSxLQUFLO0FBQzNDLFlBQU0sUUFBaUIsQ0FBQTtBQUV2QixlQUFTLElBQUksR0FBRyxJQUFJLFNBQVMsS0FBSztBQUVoQyxpQkFBUyxPQUFPLEdBQUcsT0FBTyxZQUFZLFFBQVE7QUFFNUMsY0FBSSxLQUFLLEdBQUcsS0FBSyxHQUFHLEtBQUssR0FBRyxLQUFLO0FBQ2pDLG1CQUFTLElBQUksR0FBRyxJQUFJLEdBQUcsS0FBSztBQUMxQixrQkFBTSxNQUFPLFNBQVUsSUFBSSxJQUFJLElBQU07QUFDckMsZ0JBQUksQ0FBQztBQUFLO0FBQ1Ysa0JBQU0sRUFBRSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksSUFBSSxDQUFDO0FBQzVELFlBQUMsTUFBTSxJQUFNLE1BQU0sSUFBTSxNQUFNLElBQU0sTUFBTTtVQUM3QztBQUNBLGdCQUFNLEtBQUssRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLENBQUU7UUFDL0I7TUFDRjtBQUNBLFdBQUssSUFBSTtJQUNYO0lBQ1UsYUFBYSxJQUFZLElBQVksSUFBWSxJQUFVO0FBQ25FLE1BQUMsTUFBTSxLQUFLLElBQU0sTUFBTSxLQUFLLElBQU0sTUFBTSxLQUFLLElBQU0sTUFBTSxLQUFLO0FBQy9ELFlBQU0sRUFBRSxHQUFHLEdBQUcsV0FBVSxJQUFLO0FBRTdCLFVBQUksS0FBSyxHQUFHLEtBQUssR0FBRyxLQUFLLEdBQUcsS0FBSztBQUNqQyxZQUFNLFFBQVEsS0FBSyxLQUFLO0FBQ3hCLFVBQUksSUFBSTtBQUNSLGlCQUFXLE9BQU8sQ0FBQyxJQUFJLElBQUksSUFBSSxFQUFFLEdBQUc7QUFDbEMsaUJBQVMsVUFBVSxHQUFHLFVBQVUsR0FBRyxXQUFXO0FBQzVDLGdCQUFNLE9BQVEsUUFBUyxJQUFJLFVBQVk7QUFDdkMsbUJBQVMsU0FBUyxJQUFJLElBQUksR0FBRyxVQUFVLEdBQUcsVUFBVTtBQUNsRCxrQkFBTSxNQUFPLFNBQVUsSUFBSSxTQUFXO0FBQ3RDLGtCQUFNLEVBQUUsSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssRUFBRSxJQUFJLGFBQWEsR0FBRztBQUNqRSxZQUFDLE1BQU0sSUFBTSxNQUFNLElBQU0sTUFBTSxJQUFNLE1BQU07QUFDM0MsaUJBQUs7VUFDUDtRQUNGO01BQ0Y7QUFDQSxXQUFLLEtBQUs7QUFDVixXQUFLLEtBQUs7QUFDVixXQUFLLEtBQUs7QUFDVixXQUFLLEtBQUs7SUFDWjtJQUNBLE9BQU8sTUFBVztBQUNoQixhQUFPRSxTQUFRLElBQUk7QUFDbkIsTUFBQUUsUUFBUSxJQUFJO0FBQ1osWUFBTSxNQUFNLElBQUksSUFBSTtBQUNwQixZQUFNLFNBQVMsS0FBSyxNQUFNLEtBQUssU0FBUyxVQUFVO0FBQ2xELFlBQU0sT0FBTyxLQUFLLFNBQVM7QUFDM0IsZUFBU0osS0FBSSxHQUFHQSxLQUFJLFFBQVFBLE1BQUs7QUFDL0IsYUFBSyxhQUFhLElBQUlBLEtBQUksSUFBSSxDQUFDLEdBQUcsSUFBSUEsS0FBSSxJQUFJLENBQUMsR0FBRyxJQUFJQSxLQUFJLElBQUksQ0FBQyxHQUFHLElBQUlBLEtBQUksSUFBSSxDQUFDLENBQUM7TUFDbEY7QUFDQSxVQUFJLE1BQU07QUFDUixnQkFBUSxJQUFJLEtBQUssU0FBUyxTQUFTLFVBQVUsQ0FBQztBQUM5QyxhQUFLLGFBQWEsUUFBUSxDQUFDLEdBQUcsUUFBUSxDQUFDLEdBQUcsUUFBUSxDQUFDLEdBQUcsUUFBUSxDQUFDLENBQUM7QUFDaEUsZ0JBQVEsS0FBSyxDQUFDO01BQ2hCO0FBQ0EsYUFBTztJQUNUO0lBQ0EsVUFBTztBQUNMLFlBQU0sRUFBRSxFQUFDLElBQUs7QUFFZCxpQkFBVyxPQUFPLEdBQUc7QUFDbkIsUUFBQyxJQUFJLEtBQUssR0FBSyxJQUFJLEtBQUssR0FBSyxJQUFJLEtBQUssR0FBSyxJQUFJLEtBQUs7TUFDdEQ7SUFDRjtJQUNBLFdBQVcsS0FBZTtBQUN4QixNQUFBSSxRQUFRLElBQUk7QUFDWixNQUFBQyxRQUFRLEtBQUssSUFBSTtBQUNqQixXQUFLLFdBQVc7QUFDaEIsWUFBTSxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSztBQUMzQixZQUFNLE1BQU0sSUFBSSxHQUFHO0FBQ25CLFVBQUksQ0FBQyxJQUFJO0FBQ1QsVUFBSSxDQUFDLElBQUk7QUFDVCxVQUFJLENBQUMsSUFBSTtBQUNULFVBQUksQ0FBQyxJQUFJO0FBQ1QsYUFBTztJQUNUO0lBQ0EsU0FBTTtBQUNKLFlBQU0sTUFBTSxJQUFJLFdBQVcsVUFBVTtBQUNyQyxXQUFLLFdBQVcsR0FBRztBQUNuQixXQUFLLFFBQU87QUFDWixhQUFPO0lBQ1Q7O0FBR0YsTUFBTSxVQUFOLGNBQXNCLE1BQUs7SUFDekIsWUFBWSxLQUFZLGdCQUF1QjtBQUM3QyxZQUFNSCxTQUFRLEdBQUc7QUFDakIsWUFBTSxRQUFRLFlBQVksSUFBSSxNQUFLLENBQUU7QUFDckMsWUFBTSxPQUFPLGNBQWM7QUFDM0IsWUFBTSxLQUFLLENBQUM7SUFDZDtJQUNBLE9BQU8sTUFBVztBQUNoQixhQUFPQSxTQUFRLElBQUk7QUFDbkIsTUFBQUUsUUFBUSxJQUFJO0FBQ1osWUFBTSxNQUFNLElBQUksSUFBSTtBQUNwQixZQUFNLE9BQU8sS0FBSyxTQUFTO0FBQzNCLFlBQU0sU0FBUyxLQUFLLE1BQU0sS0FBSyxTQUFTLFVBQVU7QUFDbEQsZUFBU0osS0FBSSxHQUFHQSxLQUFJLFFBQVFBLE1BQUs7QUFDL0IsYUFBSyxhQUNILE9BQU8sSUFBSUEsS0FBSSxJQUFJLENBQUMsQ0FBQyxHQUNyQixPQUFPLElBQUlBLEtBQUksSUFBSSxDQUFDLENBQUMsR0FDckIsT0FBTyxJQUFJQSxLQUFJLElBQUksQ0FBQyxDQUFDLEdBQ3JCLE9BQU8sSUFBSUEsS0FBSSxJQUFJLENBQUMsQ0FBQyxDQUFDO01BRTFCO0FBQ0EsVUFBSSxNQUFNO0FBQ1IsZ0JBQVEsSUFBSSxLQUFLLFNBQVMsU0FBUyxVQUFVLENBQUM7QUFDOUMsYUFBSyxhQUNILE9BQU8sUUFBUSxDQUFDLENBQUMsR0FDakIsT0FBTyxRQUFRLENBQUMsQ0FBQyxHQUNqQixPQUFPLFFBQVEsQ0FBQyxDQUFDLEdBQ2pCLE9BQU8sUUFBUSxDQUFDLENBQUMsQ0FBQztBQUVwQixnQkFBUSxLQUFLLENBQUM7TUFDaEI7QUFDQSxhQUFPO0lBQ1Q7SUFDQSxXQUFXLEtBQWU7QUFDeEIsTUFBQUksUUFBUSxJQUFJO0FBQ1osTUFBQUMsUUFBUSxLQUFLLElBQUk7QUFDakIsV0FBSyxXQUFXO0FBRWhCLFlBQU0sRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUs7QUFDM0IsWUFBTSxNQUFNLElBQUksR0FBRztBQUNuQixVQUFJLENBQUMsSUFBSTtBQUNULFVBQUksQ0FBQyxJQUFJO0FBQ1QsVUFBSSxDQUFDLElBQUk7QUFDVCxVQUFJLENBQUMsSUFBSTtBQUNULGFBQU8sSUFBSSxRQUFPO0lBQ3BCOztBQUlGLFdBQVMsdUJBQ1AsVUFBMEQ7QUFFMUQsVUFBTSxRQUFRLENBQUMsS0FBWSxRQUN6QixTQUFTLEtBQUssSUFBSSxNQUFNLEVBQUUsT0FBT0gsU0FBUSxHQUFHLENBQUMsRUFBRSxPQUFNO0FBQ3ZELFVBQU0sTUFBTSxTQUFTLElBQUksV0FBVyxFQUFFLEdBQUcsQ0FBQztBQUMxQyxVQUFNLFlBQVksSUFBSTtBQUN0QixVQUFNLFdBQVcsSUFBSTtBQUNyQixVQUFNLFNBQVMsQ0FBQyxLQUFZLG1CQUE0QixTQUFTLEtBQUssY0FBYztBQUNwRixXQUFPO0VBQ1Q7QUFFTyxNQUFNLFFBQVEsdUJBQ25CLENBQUMsS0FBSyxtQkFBbUIsSUFBSSxNQUFNLEtBQUssY0FBYyxDQUFDO0FBRWxELE1BQU0sVUFBVSx1QkFDckIsQ0FBQyxLQUFLLG1CQUFtQixJQUFJLFFBQVEsS0FBSyxjQUFjLENBQUM7OztBQ2hPM0QsTUFBTUksY0FBYTtBQUNuQixNQUFNLGVBQWU7QUFDckIsTUFBTSxjQUFjLElBQUksV0FBV0EsV0FBVTtBQUM3QyxNQUFNQyxRQUFPO0FBR2IsV0FBU0MsTUFBSyxHQUFTO0FBQ3JCLFdBQVEsS0FBSyxJQUFNRCxRQUFPLEVBQUUsS0FBSztFQUNuQztBQUVBLFdBQVMsSUFBSSxHQUFXLEdBQVM7QUFDL0IsUUFBSSxNQUFNO0FBQ1YsV0FBTyxJQUFJLEdBQUcsTUFBTSxHQUFHO0FBRXJCLGFBQU8sSUFBSSxFQUFFLElBQUk7QUFDakIsVUFBSUMsTUFBSyxDQUFDO0lBQ1o7QUFDQSxXQUFPO0VBQ1Q7QUFJQSxNQUFNLE9BQXdCLHVCQUFLO0FBQ2pDLFFBQUksSUFBSSxJQUFJLFdBQVcsR0FBRztBQUMxQixhQUFTQyxLQUFJLEdBQUcsSUFBSSxHQUFHQSxLQUFJLEtBQUtBLE1BQUssS0FBS0QsTUFBSyxDQUFDO0FBQUcsUUFBRUMsRUFBQyxJQUFJO0FBQzFELFVBQU0sTUFBTSxJQUFJLFdBQVcsR0FBRztBQUM5QixRQUFJLENBQUMsSUFBSTtBQUNULGFBQVNBLEtBQUksR0FBR0EsS0FBSSxLQUFLQSxNQUFLO0FBQzVCLFVBQUksSUFBSSxFQUFFLE1BQU1BLEVBQUM7QUFDakIsV0FBSyxLQUFLO0FBQ1YsVUFBSSxFQUFFQSxFQUFDLENBQUMsS0FBSyxJQUFLLEtBQUssSUFBTSxLQUFLLElBQU0sS0FBSyxJQUFNLEtBQUssSUFBSyxNQUFRO0lBQ3ZFO0FBQ0EsV0FBTztFQUNULEdBQUU7QUFHRixNQUFNLFVBQTBCLHFCQUFLLElBQUksQ0FBQyxHQUFHLE1BQU0sS0FBSyxRQUFRLENBQUMsQ0FBQztBQUdsRSxNQUFNLFdBQVcsQ0FBQyxNQUFlLEtBQUssS0FBTyxNQUFNO0FBQ25ELE1BQU0sV0FBVyxDQUFDLE1BQWUsS0FBSyxJQUFNLE1BQU07QUFNbEQsV0FBUyxVQUFVQyxPQUFrQixJQUF5QjtBQUM1RCxRQUFJQSxNQUFLLFdBQVc7QUFBSyxZQUFNLElBQUksTUFBTSxtQkFBbUI7QUFDNUQsVUFBTSxLQUFLLElBQUksWUFBWSxHQUFHLEVBQUUsSUFBSSxDQUFDLEdBQUcsTUFBTSxHQUFHQSxNQUFLLENBQUMsQ0FBQyxDQUFDO0FBQ3pELFVBQU0sS0FBSyxHQUFHLElBQUksUUFBUTtBQUMxQixVQUFNLEtBQUssR0FBRyxJQUFJLFFBQVE7QUFDMUIsVUFBTSxLQUFLLEdBQUcsSUFBSSxRQUFRO0FBQzFCLFVBQU0sTUFBTSxJQUFJLFlBQVksTUFBTSxHQUFHO0FBQ3JDLFVBQU0sTUFBTSxJQUFJLFlBQVksTUFBTSxHQUFHO0FBQ3JDLFVBQU1DLFNBQVEsSUFBSSxZQUFZLE1BQU0sR0FBRztBQUN2QyxhQUFTRixLQUFJLEdBQUdBLEtBQUksS0FBS0EsTUFBSztBQUM1QixlQUFTLElBQUksR0FBRyxJQUFJLEtBQUssS0FBSztBQUM1QixjQUFNLE1BQU1BLEtBQUksTUFBTTtBQUN0QixZQUFJLEdBQUcsSUFBSSxHQUFHQSxFQUFDLElBQUksR0FBRyxDQUFDO0FBQ3ZCLFlBQUksR0FBRyxJQUFJLEdBQUdBLEVBQUMsSUFBSSxHQUFHLENBQUM7QUFDdkIsUUFBQUUsT0FBTSxHQUFHLElBQUtELE1BQUtELEVBQUMsS0FBSyxJQUFLQyxNQUFLLENBQUM7TUFDdEM7SUFDRjtBQUNBLFdBQU8sRUFBRSxNQUFBQSxPQUFNLE9BQUFDLFFBQU8sSUFBSSxJQUFJLElBQUksSUFBSSxLQUFLLElBQUc7RUFDaEQ7QUFFQSxNQUFNLGdCQUFnQywwQkFDcEMsTUFDQSxDQUFDLE1BQWUsSUFBSSxHQUFHLENBQUMsS0FBSyxLQUFPLEtBQUssS0FBTyxLQUFLLElBQUssSUFBSSxHQUFHLENBQUMsQ0FBQztBQUVyRSxNQUFNLGdCQUFnQywwQkFDcEMsU0FDQSxDQUFDLE1BQU8sSUFBSSxHQUFHLEVBQUUsS0FBSyxLQUFPLElBQUksR0FBRyxFQUFFLEtBQUssS0FBTyxJQUFJLEdBQUcsQ0FBQyxLQUFLLElBQUssSUFBSSxHQUFHLEVBQUUsQ0FBQztBQUdoRixNQUFNLFVBQTJCLHVCQUFLO0FBQ3BDLFVBQU0sSUFBSSxJQUFJLFdBQVcsRUFBRTtBQUMzQixhQUFTRixLQUFJLEdBQUcsSUFBSSxHQUFHQSxLQUFJLElBQUlBLE1BQUssSUFBSUQsTUFBSyxDQUFDO0FBQUcsUUFBRUMsRUFBQyxJQUFJO0FBQ3hELFdBQU87RUFDVCxHQUFFO0FBRUksV0FBVSxZQUFZLEtBQWU7QUFDekMsSUFBQUcsT0FBTyxHQUFHO0FBQ1YsVUFBTSxNQUFNLElBQUk7QUFDaEIsUUFBSSxDQUFDLENBQUMsSUFBSSxJQUFJLEVBQUUsRUFBRSxTQUFTLEdBQUc7QUFDNUIsWUFBTSxJQUFJLE1BQU0scURBQXFELEdBQUcsRUFBRTtBQUM1RSxVQUFNLEVBQUUsTUFBSyxJQUFLO0FBQ2xCLFVBQU0sTUFBTSxJQUFJLEdBQUc7QUFDbkIsVUFBTSxLQUFLLElBQUk7QUFDZixVQUFNLFVBQVUsQ0FBQyxNQUFjLFVBQVUsT0FBTyxHQUFHLEdBQUcsR0FBRyxDQUFDO0FBQzFELFVBQU0sS0FBSyxJQUFJLFlBQVksTUFBTSxFQUFFO0FBQ25DLE9BQUcsSUFBSSxHQUFHO0FBRVYsYUFBU0gsS0FBSSxJQUFJQSxLQUFJLEdBQUcsUUFBUUEsTUFBSztBQUNuQyxVQUFJLElBQUksR0FBR0EsS0FBSSxDQUFDO0FBQ2hCLFVBQUlBLEtBQUksT0FBTztBQUFHLFlBQUksUUFBUSxTQUFTLENBQUMsQ0FBQyxJQUFJLFFBQVFBLEtBQUksS0FBSyxDQUFDO2VBQ3RELEtBQUssS0FBS0EsS0FBSSxPQUFPO0FBQUcsWUFBSSxRQUFRLENBQUM7QUFDOUMsU0FBR0EsRUFBQyxJQUFJLEdBQUdBLEtBQUksRUFBRSxJQUFJO0lBQ3ZCO0FBQ0EsV0FBTztFQUNUO0FBRU0sV0FBVSxlQUFlLEtBQWU7QUFDNUMsVUFBTSxTQUFTLFlBQVksR0FBRztBQUM5QixVQUFNLEtBQUssT0FBTyxNQUFLO0FBQ3ZCLFVBQU0sS0FBSyxPQUFPO0FBQ2xCLFVBQU0sRUFBRSxNQUFLLElBQUs7QUFDbEIsVUFBTSxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSztBQUUzQixhQUFTQSxLQUFJLEdBQUdBLEtBQUksSUFBSUEsTUFBSyxHQUFHO0FBQzlCLGVBQVMsSUFBSSxHQUFHLElBQUksR0FBRztBQUFLLFdBQUdBLEtBQUksQ0FBQyxJQUFJLE9BQU8sS0FBS0EsS0FBSSxJQUFJLENBQUM7SUFDL0Q7QUFDQSxXQUFPLEtBQUssQ0FBQztBQUViLGFBQVNBLEtBQUksR0FBR0EsS0FBSSxLQUFLLEdBQUdBLE1BQUs7QUFDL0IsWUFBTSxJQUFJLEdBQUdBLEVBQUM7QUFDZCxZQUFNLElBQUksVUFBVSxPQUFPLEdBQUcsR0FBRyxHQUFHLENBQUM7QUFDckMsU0FBR0EsRUFBQyxJQUFJLEdBQUcsSUFBSSxHQUFJLElBQUksR0FBSSxNQUFNLElBQUssR0FBSSxJQUFJLEdBQUksTUFBTSxLQUFNLEdBQUksSUFBSSxHQUFHLE1BQU0sRUFBRTtJQUNuRjtBQUNBLFdBQU87RUFDVDtBQUdBLFdBQVMsVUFDUCxLQUNBLEtBQ0EsSUFDQSxJQUNBLElBQ0EsSUFBVTtBQUVWLFdBQ0UsSUFBTSxNQUFNLElBQUssUUFBWSxPQUFPLElBQUssR0FBSyxJQUM5QyxJQUFNLE9BQU8sSUFBSyxRQUFZLE9BQU8sS0FBTSxHQUFLO0VBRXBEO0FBRUEsV0FBUyxVQUFVLE9BQW9CLElBQVksSUFBWSxJQUFZLElBQVU7QUFDbkYsV0FDRSxNQUFPLEtBQUssTUFBUyxLQUFLLEtBQU8sSUFDaEMsTUFBUSxPQUFPLEtBQU0sTUFBVSxPQUFPLEtBQU0sS0FBTyxLQUFLO0VBRTdEO0FBRUEsV0FBUyxRQUFRLElBQWlCLElBQVksSUFBWSxJQUFZLElBQVU7QUFDOUUsVUFBTSxFQUFFLE9BQU8sS0FBSyxJQUFHLElBQUs7QUFDNUIsUUFBSSxJQUFJO0FBQ1IsSUFBQyxNQUFNLEdBQUcsR0FBRyxHQUFLLE1BQU0sR0FBRyxHQUFHLEdBQUssTUFBTSxHQUFHLEdBQUcsR0FBSyxNQUFNLEdBQUcsR0FBRztBQUNoRSxVQUFNLFNBQVMsR0FBRyxTQUFTLElBQUk7QUFDL0IsYUFBU0EsS0FBSSxHQUFHQSxLQUFJLFFBQVFBLE1BQUs7QUFDL0IsWUFBTUksTUFBSyxHQUFHLEdBQUcsSUFBSSxVQUFVLEtBQUssS0FBSyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3ZELFlBQU1DLE1BQUssR0FBRyxHQUFHLElBQUksVUFBVSxLQUFLLEtBQUssSUFBSSxJQUFJLElBQUksRUFBRTtBQUN2RCxZQUFNQyxNQUFLLEdBQUcsR0FBRyxJQUFJLFVBQVUsS0FBSyxLQUFLLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDdkQsWUFBTUMsTUFBSyxHQUFHLEdBQUcsSUFBSSxVQUFVLEtBQUssS0FBSyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3ZELE1BQUMsS0FBS0gsS0FBTSxLQUFLQyxLQUFNLEtBQUtDLEtBQU0sS0FBS0M7SUFDekM7QUFFQSxVQUFNLEtBQUssR0FBRyxHQUFHLElBQUksVUFBVSxPQUFPLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDcEQsVUFBTSxLQUFLLEdBQUcsR0FBRyxJQUFJLFVBQVUsT0FBTyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3BELFVBQU0sS0FBSyxHQUFHLEdBQUcsSUFBSSxVQUFVLE9BQU8sSUFBSSxJQUFJLElBQUksRUFBRTtBQUNwRCxVQUFNLEtBQUssR0FBRyxHQUFHLElBQUksVUFBVSxPQUFPLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDcEQsV0FBTyxFQUFFLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksR0FBRTtFQUN6QztBQUVBLFdBQVMsUUFBUSxJQUFpQixJQUFZLElBQVksSUFBWSxJQUFVO0FBQzlFLFVBQU0sRUFBRSxPQUFPLEtBQUssSUFBRyxJQUFLO0FBQzVCLFFBQUksSUFBSTtBQUNSLElBQUMsTUFBTSxHQUFHLEdBQUcsR0FBSyxNQUFNLEdBQUcsR0FBRyxHQUFLLE1BQU0sR0FBRyxHQUFHLEdBQUssTUFBTSxHQUFHLEdBQUc7QUFDaEUsVUFBTSxTQUFTLEdBQUcsU0FBUyxJQUFJO0FBQy9CLGFBQVNQLEtBQUksR0FBR0EsS0FBSSxRQUFRQSxNQUFLO0FBQy9CLFlBQU1JLE1BQUssR0FBRyxHQUFHLElBQUksVUFBVSxLQUFLLEtBQUssSUFBSSxJQUFJLElBQUksRUFBRTtBQUN2RCxZQUFNQyxNQUFLLEdBQUcsR0FBRyxJQUFJLFVBQVUsS0FBSyxLQUFLLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDdkQsWUFBTUMsTUFBSyxHQUFHLEdBQUcsSUFBSSxVQUFVLEtBQUssS0FBSyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3ZELFlBQU1DLE1BQUssR0FBRyxHQUFHLElBQUksVUFBVSxLQUFLLEtBQUssSUFBSSxJQUFJLElBQUksRUFBRTtBQUN2RCxNQUFDLEtBQUtILEtBQU0sS0FBS0MsS0FBTSxLQUFLQyxLQUFNLEtBQUtDO0lBQ3pDO0FBRUEsVUFBTSxLQUFLLEdBQUcsR0FBRyxJQUFJLFVBQVUsT0FBTyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3BELFVBQU0sS0FBSyxHQUFHLEdBQUcsSUFBSSxVQUFVLE9BQU8sSUFBSSxJQUFJLElBQUksRUFBRTtBQUNwRCxVQUFNLEtBQUssR0FBRyxHQUFHLElBQUksVUFBVSxPQUFPLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDcEQsVUFBTSxLQUFLLEdBQUcsR0FBRyxJQUFJLFVBQVUsT0FBTyxJQUFJLElBQUksSUFBSSxFQUFFO0FBQ3BELFdBQU8sRUFBRSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLEdBQUU7RUFDekM7QUFFQSxXQUFTLE9BQU8sS0FBYSxLQUFnQjtBQUMzQyxRQUFJLENBQUM7QUFBSyxhQUFPLElBQUksV0FBVyxHQUFHO0FBQ25DLElBQUFKLE9BQU8sR0FBRztBQUNWLFFBQUksSUFBSSxTQUFTO0FBQ2YsWUFBTSxJQUFJLE1BQU0sb0RBQW9ELEdBQUcsVUFBVSxJQUFJLE1BQU0sRUFBRTtBQUMvRixXQUFPO0VBQ1Q7QUFHQSxXQUFTLFdBQVcsSUFBaUIsT0FBbUIsS0FBaUIsS0FBZ0I7QUFDdkYsSUFBQUEsT0FBTyxPQUFPTixXQUFVO0FBQ3hCLElBQUFNLE9BQU8sR0FBRztBQUNWLFVBQU0sU0FBUyxJQUFJO0FBQ25CLFVBQU0sT0FBTyxRQUFRLEdBQUc7QUFDeEIsVUFBTUssT0FBTTtBQUNaLFVBQU0sTUFBTSxJQUFJQSxJQUFHO0FBRW5CLFFBQUksRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxDQUFDO0FBQ25FLFVBQU0sUUFBUSxJQUFJLEdBQUc7QUFDckIsVUFBTSxRQUFRLElBQUksR0FBRztBQUVyQixhQUFTUixLQUFJLEdBQUdBLEtBQUksS0FBSyxNQUFNLFFBQVFBLE1BQUssR0FBRztBQUM3QyxZQUFNQSxLQUFJLENBQUMsSUFBSSxNQUFNQSxLQUFJLENBQUMsSUFBSTtBQUM5QixZQUFNQSxLQUFJLENBQUMsSUFBSSxNQUFNQSxLQUFJLENBQUMsSUFBSTtBQUM5QixZQUFNQSxLQUFJLENBQUMsSUFBSSxNQUFNQSxLQUFJLENBQUMsSUFBSTtBQUM5QixZQUFNQSxLQUFJLENBQUMsSUFBSSxNQUFNQSxLQUFJLENBQUMsSUFBSTtBQUU5QixVQUFJLFFBQVE7QUFDWixlQUFTQSxLQUFJUSxLQUFJLFNBQVMsR0FBR1IsTUFBSyxHQUFHQSxNQUFLO0FBQ3hDLGdCQUFTLFNBQVNRLEtBQUlSLEVBQUMsSUFBSSxPQUFTO0FBQ3BDLFFBQUFRLEtBQUlSLEVBQUMsSUFBSSxRQUFRO0FBQ2pCLG1CQUFXO01BQ2I7QUFDQSxPQUFDLEVBQUUsSUFBSSxJQUFJLElBQUksR0FBRSxJQUFLLFFBQVEsSUFBSSxJQUFJLENBQUMsR0FBRyxJQUFJLENBQUMsR0FBRyxJQUFJLENBQUMsR0FBRyxJQUFJLENBQUMsQ0FBQztJQUNsRTtBQUdBLFVBQU0sUUFBUUgsY0FBYSxLQUFLLE1BQU0sTUFBTSxTQUFTLFlBQVk7QUFDakUsUUFBSSxRQUFRLFFBQVE7QUFDbEIsWUFBTSxNQUFNLElBQUksWUFBWSxDQUFDLElBQUksSUFBSSxJQUFJLEVBQUUsQ0FBQztBQUM1QyxZQUFNLE1BQU0sR0FBRyxHQUFHO0FBQ2xCLGVBQVNHLEtBQUksT0FBTyxNQUFNLEdBQUdBLEtBQUksUUFBUUEsTUFBSztBQUFPLFlBQUlBLEVBQUMsSUFBSSxJQUFJQSxFQUFDLElBQUksSUFBSSxHQUFHO0lBQ2hGO0FBQ0EsV0FBTztFQUNUO0FBS0EsV0FBUyxNQUNQLElBQ0FTLE9BQ0EsT0FDQSxLQUNBLEtBQWdCO0FBRWhCLElBQUFOLE9BQU8sT0FBT04sV0FBVTtBQUN4QixJQUFBTSxPQUFPLEdBQUc7QUFDVixVQUFNLE9BQU8sSUFBSSxRQUFRLEdBQUc7QUFDNUIsVUFBTUssT0FBTTtBQUNaLFVBQU0sTUFBTSxJQUFJQSxJQUFHO0FBQ25CLFVBQU0sT0FBT0UsWUFBV0YsSUFBRztBQUMzQixVQUFNLFFBQVEsSUFBSSxHQUFHO0FBQ3JCLFVBQU0sUUFBUSxJQUFJLEdBQUc7QUFDckIsVUFBTSxTQUFTQyxRQUFPLElBQUk7QUFDMUIsVUFBTSxTQUFTLElBQUk7QUFFbkIsUUFBSSxTQUFTLEtBQUssVUFBVSxRQUFRQSxLQUFJO0FBQ3hDLFFBQUksRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxHQUFHLElBQUksQ0FBQyxDQUFDO0FBRW5FLGFBQVNULEtBQUksR0FBR0EsS0FBSSxLQUFLLE1BQU0sUUFBUUEsTUFBSyxHQUFHO0FBQzdDLFlBQU1BLEtBQUksQ0FBQyxJQUFJLE1BQU1BLEtBQUksQ0FBQyxJQUFJO0FBQzlCLFlBQU1BLEtBQUksQ0FBQyxJQUFJLE1BQU1BLEtBQUksQ0FBQyxJQUFJO0FBQzlCLFlBQU1BLEtBQUksQ0FBQyxJQUFJLE1BQU1BLEtBQUksQ0FBQyxJQUFJO0FBQzlCLFlBQU1BLEtBQUksQ0FBQyxJQUFJLE1BQU1BLEtBQUksQ0FBQyxJQUFJO0FBQzlCLGVBQVUsU0FBUyxNQUFPO0FBQzFCLFdBQUssVUFBVSxRQUFRLFFBQVFTLEtBQUk7QUFDbkMsT0FBQyxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksSUFBSSxDQUFDLEdBQUcsSUFBSSxDQUFDLEdBQUcsSUFBSSxDQUFDLEdBQUcsSUFBSSxDQUFDLENBQUM7SUFDbEU7QUFFQSxVQUFNLFFBQVFaLGNBQWEsS0FBSyxNQUFNLE1BQU0sU0FBUyxZQUFZO0FBQ2pFLFFBQUksUUFBUSxRQUFRO0FBQ2xCLFlBQU0sTUFBTSxJQUFJLFlBQVksQ0FBQyxJQUFJLElBQUksSUFBSSxFQUFFLENBQUM7QUFDNUMsWUFBTSxNQUFNLEdBQUcsR0FBRztBQUNsQixlQUFTRyxLQUFJLE9BQU8sTUFBTSxHQUFHQSxLQUFJLFFBQVFBLE1BQUs7QUFBTyxZQUFJQSxFQUFDLElBQUksSUFBSUEsRUFBQyxJQUFJLElBQUksR0FBRztJQUNoRjtBQUNBLFdBQU87RUFDVDtBQU1PLE1BQU0sTUFBTSxXQUNqQixFQUFFLFdBQVcsSUFBSSxhQUFhLEdBQUUsR0FDaEMsU0FBU1EsS0FBSSxLQUFpQixPQUFpQjtBQUM3QyxJQUFBTCxPQUFPLEdBQUc7QUFDVixJQUFBQSxPQUFPLE9BQU9OLFdBQVU7QUFDeEIsYUFBUyxXQUFXLEtBQWlCLEtBQWdCO0FBQ25ELFlBQU0sS0FBSyxZQUFZLEdBQUc7QUFDMUIsWUFBTSxJQUFJLE1BQU0sTUFBSztBQUNyQixZQUFNLE1BQU0sV0FBVyxJQUFJLEdBQUcsS0FBSyxHQUFHO0FBQ3RDLFNBQUcsS0FBSyxDQUFDO0FBQ1QsUUFBRSxLQUFLLENBQUM7QUFDUixhQUFPO0lBQ1Q7QUFDQSxXQUFPO01BQ0wsU0FBUyxDQUFDLFdBQXVCLFFBQXFCLFdBQVcsV0FBVyxHQUFHO01BQy9FLFNBQVMsQ0FBQyxZQUF3QixRQUFxQixXQUFXLFlBQVksR0FBRzs7RUFFckYsQ0FBQztBQUdILFdBQVMscUJBQXFCLE1BQWdCO0FBQzVDLElBQUFNLE9BQU8sSUFBSTtBQUNYLFFBQUksS0FBSyxTQUFTTixnQkFBZSxHQUFHO0FBQ2xDLFlBQU0sSUFBSSxNQUNSLHVFQUF1RUEsV0FBVSxFQUFFO0lBRXZGO0VBQ0Y7QUFFQSxXQUFTLHFCQUFxQixXQUF1QixPQUFnQixLQUFnQjtBQUNuRixRQUFJLFNBQVMsVUFBVTtBQUN2QixVQUFNLFlBQVksU0FBU0E7QUFDM0IsUUFBSSxDQUFDLFNBQVMsY0FBYztBQUMxQixZQUFNLElBQUksTUFBTSx5REFBeUQ7QUFDM0UsVUFBTSxJQUFJLElBQUksU0FBUztBQUN2QixRQUFJLE9BQU87QUFDVCxVQUFJLE9BQU9BLGNBQWE7QUFDeEIsVUFBSSxDQUFDO0FBQU0sZUFBT0E7QUFDbEIsZUFBUyxTQUFTO0lBQ3BCO0FBQ0EsVUFBTSxNQUFNLE9BQU8sUUFBUSxHQUFHO0FBQzlCLFVBQU0sSUFBSSxJQUFJLEdBQUc7QUFDakIsV0FBTyxFQUFFLEdBQUcsR0FBRyxJQUFHO0VBQ3BCO0FBRUEsV0FBUyxhQUFhLE1BQWtCLE9BQWM7QUFDcEQsUUFBSSxDQUFDO0FBQU8sYUFBTztBQUNuQixVQUFNLE1BQU0sS0FBSztBQUNqQixRQUFJLENBQUM7QUFBSyxZQUFNLElBQUksTUFBTSx5Q0FBeUM7QUFDbkUsVUFBTSxXQUFXLEtBQUssTUFBTSxDQUFDO0FBQzdCLFFBQUksWUFBWSxLQUFLLFdBQVc7QUFBSSxZQUFNLElBQUksTUFBTSxrQ0FBa0MsUUFBUSxFQUFFO0FBQ2hHLFVBQU0sTUFBTSxLQUFLLFNBQVMsR0FBRyxDQUFDLFFBQVE7QUFDdEMsYUFBU0csS0FBSSxHQUFHQSxLQUFJLFVBQVVBO0FBQzVCLFVBQUksS0FBSyxNQUFNQSxLQUFJLENBQUMsTUFBTTtBQUFVLGNBQU0sSUFBSSxNQUFNLDBCQUEwQjtBQUNoRixXQUFPO0VBQ1Q7QUFFQSxXQUFTLFFBQVEsTUFBZ0I7QUFDL0IsVUFBTSxNQUFNLElBQUksV0FBVyxFQUFFO0FBQzdCLFVBQU0sUUFBUSxJQUFJLEdBQUc7QUFDckIsUUFBSSxJQUFJLElBQUk7QUFDWixVQUFNLGNBQWNILGNBQWEsS0FBSztBQUN0QyxhQUFTRyxLQUFJSCxjQUFhLGFBQWFHLEtBQUlILGFBQVlHO0FBQUssVUFBSUEsRUFBQyxJQUFJO0FBQ3JFLFdBQU87RUFDVDtBQVFPLE1BQU0sTUFBTSxXQUNqQixFQUFFLFdBQVcsR0FBRSxHQUNmLFNBQVNXLEtBQUksS0FBaUIsT0FBa0IsQ0FBQSxHQUFFO0FBQ2hELElBQUFSLE9BQU8sR0FBRztBQUNWLFVBQU0sUUFBUSxDQUFDLEtBQUs7QUFDcEIsV0FBTztNQUNMLFNBQVMsQ0FBQyxXQUF1QixRQUFvQjtBQUNuRCxRQUFBQSxPQUFPLFNBQVM7QUFDaEIsY0FBTSxFQUFFLEdBQUcsR0FBRyxLQUFLLEtBQUksSUFBSyxxQkFBcUIsV0FBVyxPQUFPLEdBQUc7QUFDdEUsY0FBTSxLQUFLLFlBQVksR0FBRztBQUMxQixZQUFJSCxLQUFJO0FBQ1IsZUFBT0EsS0FBSSxLQUFLLEVBQUUsVUFBVTtBQUMxQixnQkFBTSxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksRUFBRUEsS0FBSSxDQUFDLEdBQUcsRUFBRUEsS0FBSSxDQUFDLEdBQUcsRUFBRUEsS0FBSSxDQUFDLEdBQUcsRUFBRUEsS0FBSSxDQUFDLENBQUM7QUFDN0UsVUFBQyxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUk7UUFDekQ7QUFDQSxZQUFJLE9BQU87QUFDVCxnQkFBTSxRQUFRLFFBQVEsVUFBVSxTQUFTQSxLQUFJLENBQUMsQ0FBQztBQUMvQyxnQkFBTSxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksTUFBTSxDQUFDLEdBQUcsTUFBTSxDQUFDLEdBQUcsTUFBTSxDQUFDLEdBQUcsTUFBTSxDQUFDLENBQUM7QUFDN0UsVUFBQyxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUk7UUFDekQ7QUFDQSxXQUFHLEtBQUssQ0FBQztBQUNULGVBQU87TUFDVDtNQUNBLFNBQVMsQ0FBQyxZQUF3QixRQUFvQjtBQUNwRCw2QkFBcUIsVUFBVTtBQUMvQixjQUFNLEtBQUssZUFBZSxHQUFHO0FBQzdCLGNBQU0sTUFBTSxPQUFPLFdBQVcsUUFBUSxHQUFHO0FBQ3pDLGNBQU0sSUFBSSxJQUFJLFVBQVU7QUFDeEIsY0FBTSxJQUFJLElBQUksR0FBRztBQUNqQixpQkFBU0EsS0FBSSxHQUFHQSxLQUFJLEtBQUssRUFBRSxVQUFVO0FBQ25DLGdCQUFNLEVBQUUsSUFBSSxJQUFJLElBQUksR0FBRSxJQUFLLFFBQVEsSUFBSSxFQUFFQSxLQUFJLENBQUMsR0FBRyxFQUFFQSxLQUFJLENBQUMsR0FBRyxFQUFFQSxLQUFJLENBQUMsR0FBRyxFQUFFQSxLQUFJLENBQUMsQ0FBQztBQUM3RSxVQUFDLEVBQUVBLElBQUcsSUFBSSxJQUFNLEVBQUVBLElBQUcsSUFBSSxJQUFNLEVBQUVBLElBQUcsSUFBSSxJQUFNLEVBQUVBLElBQUcsSUFBSTtRQUN6RDtBQUNBLFdBQUcsS0FBSyxDQUFDO0FBQ1QsZUFBTyxhQUFhLEtBQUssS0FBSztNQUNoQzs7RUFFSixDQUFDO0FBT0ksTUFBTSxNQUFNLFdBQ2pCLEVBQUUsV0FBVyxJQUFJLGFBQWEsR0FBRSxHQUNoQyxTQUFTWSxLQUFJLEtBQWlCLElBQWdCLE9BQWtCLENBQUEsR0FBRTtBQUNoRSxJQUFBVCxPQUFPLEdBQUc7QUFDVixJQUFBQSxPQUFPLElBQUksRUFBRTtBQUNiLFVBQU0sUUFBUSxDQUFDLEtBQUs7QUFDcEIsV0FBTztNQUNMLFNBQVMsQ0FBQyxXQUF1QixRQUFvQjtBQUNuRCxjQUFNLEtBQUssWUFBWSxHQUFHO0FBQzFCLGNBQU0sRUFBRSxHQUFHLEdBQUcsS0FBSyxLQUFJLElBQUsscUJBQXFCLFdBQVcsT0FBTyxHQUFHO0FBQ3RFLGNBQU0sTUFBTSxJQUFJLEVBQUU7QUFFbEIsWUFBSSxLQUFLLElBQUksQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQztBQUNyRCxZQUFJSCxLQUFJO0FBQ1IsZUFBT0EsS0FBSSxLQUFLLEVBQUUsVUFBVTtBQUMxQixVQUFDLE1BQU0sRUFBRUEsS0FBSSxDQUFDLEdBQUssTUFBTSxFQUFFQSxLQUFJLENBQUMsR0FBSyxNQUFNLEVBQUVBLEtBQUksQ0FBQyxHQUFLLE1BQU0sRUFBRUEsS0FBSSxDQUFDO0FBQ3BFLFdBQUMsRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDaEQsVUFBQyxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUk7UUFDekQ7QUFDQSxZQUFJLE9BQU87QUFDVCxnQkFBTSxRQUFRLFFBQVEsVUFBVSxTQUFTQSxLQUFJLENBQUMsQ0FBQztBQUMvQyxVQUFDLE1BQU0sTUFBTSxDQUFDLEdBQUssTUFBTSxNQUFNLENBQUMsR0FBSyxNQUFNLE1BQU0sQ0FBQyxHQUFLLE1BQU0sTUFBTSxDQUFDO0FBQ3BFLFdBQUMsRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDaEQsVUFBQyxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUksSUFBTSxFQUFFQSxJQUFHLElBQUk7UUFDekQ7QUFDQSxXQUFHLEtBQUssQ0FBQztBQUNULGVBQU87TUFDVDtNQUNBLFNBQVMsQ0FBQyxZQUF3QixRQUFvQjtBQUNwRCw2QkFBcUIsVUFBVTtBQUMvQixjQUFNLEtBQUssZUFBZSxHQUFHO0FBQzdCLGNBQU0sTUFBTSxJQUFJLEVBQUU7QUFDbEIsY0FBTSxNQUFNLE9BQU8sV0FBVyxRQUFRLEdBQUc7QUFDekMsY0FBTSxJQUFJLElBQUksVUFBVTtBQUN4QixjQUFNLElBQUksSUFBSSxHQUFHO0FBRWpCLFlBQUksS0FBSyxJQUFJLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUM7QUFDckQsaUJBQVNBLEtBQUksR0FBR0EsS0FBSSxLQUFLLEVBQUUsVUFBVTtBQUVuQyxnQkFBTSxNQUFNLElBQUksTUFBTSxJQUFJLE1BQU0sSUFBSSxNQUFNO0FBQzFDLFVBQUMsS0FBSyxFQUFFQSxLQUFJLENBQUMsR0FBSyxLQUFLLEVBQUVBLEtBQUksQ0FBQyxHQUFLLEtBQUssRUFBRUEsS0FBSSxDQUFDLEdBQUssS0FBSyxFQUFFQSxLQUFJLENBQUM7QUFDaEUsZ0JBQU0sRUFBRSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksSUFBSSxJQUFJLElBQUksRUFBRTtBQUNyRSxVQUFDLEVBQUVBLElBQUcsSUFBSSxLQUFLLEtBQU8sRUFBRUEsSUFBRyxJQUFJLEtBQUssS0FBTyxFQUFFQSxJQUFHLElBQUksS0FBSyxLQUFPLEVBQUVBLElBQUcsSUFBSSxLQUFLO1FBQ2hGO0FBQ0EsV0FBRyxLQUFLLENBQUM7QUFDVCxlQUFPLGFBQWEsS0FBSyxLQUFLO01BQ2hDOztFQUVKLENBQUM7QUFPSSxNQUFNLE1BQU0sV0FDakIsRUFBRSxXQUFXLElBQUksYUFBYSxHQUFFLEdBQ2hDLFNBQVNhLEtBQUksS0FBaUIsSUFBYztBQUMxQyxJQUFBVixPQUFPLEdBQUc7QUFDVixJQUFBQSxPQUFPLElBQUksRUFBRTtBQUNiLGFBQVMsV0FBVyxLQUFpQixXQUFvQixLQUFnQjtBQUN2RSxZQUFNLEtBQUssWUFBWSxHQUFHO0FBQzFCLFlBQU0sU0FBUyxJQUFJO0FBQ25CLFlBQU0sT0FBTyxRQUFRLEdBQUc7QUFDeEIsWUFBTSxRQUFRLElBQUksR0FBRztBQUNyQixZQUFNLFFBQVEsSUFBSSxHQUFHO0FBQ3JCLFlBQU0sU0FBUyxZQUFZLFFBQVE7QUFDbkMsWUFBTSxNQUFNLElBQUksRUFBRTtBQUVsQixVQUFJLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDO0FBQ3JELGVBQVNILEtBQUksR0FBR0EsS0FBSSxLQUFLLE1BQU0sVUFBVTtBQUN2QyxjQUFNLEVBQUUsSUFBSSxJQUFJLElBQUksSUFBSSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDckUsY0FBTUEsS0FBSSxDQUFDLElBQUksTUFBTUEsS0FBSSxDQUFDLElBQUk7QUFDOUIsY0FBTUEsS0FBSSxDQUFDLElBQUksTUFBTUEsS0FBSSxDQUFDLElBQUk7QUFDOUIsY0FBTUEsS0FBSSxDQUFDLElBQUksTUFBTUEsS0FBSSxDQUFDLElBQUk7QUFDOUIsY0FBTUEsS0FBSSxDQUFDLElBQUksTUFBTUEsS0FBSSxDQUFDLElBQUk7QUFDOUIsUUFBQyxLQUFLLE9BQU9BLElBQUcsR0FBSyxLQUFLLE9BQU9BLElBQUcsR0FBSyxLQUFLLE9BQU9BLElBQUcsR0FBSyxLQUFLLE9BQU9BLElBQUc7TUFDOUU7QUFFQSxZQUFNLFFBQVFILGNBQWEsS0FBSyxNQUFNLE1BQU0sU0FBUyxZQUFZO0FBQ2pFLFVBQUksUUFBUSxRQUFRO0FBQ2xCLFNBQUMsRUFBRSxJQUFJLElBQUksSUFBSSxHQUFFLElBQUssUUFBUSxJQUFJLElBQUksSUFBSSxJQUFJLEVBQUU7QUFDaEQsY0FBTSxNQUFNLEdBQUcsSUFBSSxZQUFZLENBQUMsSUFBSSxJQUFJLElBQUksRUFBRSxDQUFDLENBQUM7QUFDaEQsaUJBQVNHLEtBQUksT0FBTyxNQUFNLEdBQUdBLEtBQUksUUFBUUEsTUFBSztBQUFPLGNBQUlBLEVBQUMsSUFBSSxJQUFJQSxFQUFDLElBQUksSUFBSSxHQUFHO0FBQzlFLFlBQUksS0FBSyxDQUFDO01BQ1o7QUFDQSxTQUFHLEtBQUssQ0FBQztBQUNULGFBQU87SUFDVDtBQUNBLFdBQU87TUFDTCxTQUFTLENBQUMsV0FBdUIsUUFBcUIsV0FBVyxXQUFXLE1BQU0sR0FBRztNQUNyRixTQUFTLENBQUMsWUFBd0IsUUFBcUIsV0FBVyxZQUFZLE9BQU8sR0FBRzs7RUFFNUYsQ0FBQztBQUlILFdBQVMsV0FDUCxJQUNBUyxPQUNBLEtBQ0EsTUFDQSxLQUFnQjtBQUVoQixVQUFNLElBQUksR0FBRyxPQUFPLEtBQUssS0FBSyxVQUFVLEtBQUssVUFBVSxFQUFFO0FBQ3pELFFBQUk7QUFBSyxRQUFFLE9BQU8sR0FBRztBQUNyQixNQUFFLE9BQU8sSUFBSTtBQUNiLFVBQU0sTUFBTSxJQUFJLFdBQVcsRUFBRTtBQUM3QixVQUFNLE9BQU9DLFlBQVcsR0FBRztBQUMzQixRQUFJO0FBQUssTUFBQUksY0FBYSxNQUFNLEdBQUcsT0FBTyxJQUFJLFNBQVMsQ0FBQyxHQUFHTCxLQUFJO0FBQzNELElBQUFLLGNBQWEsTUFBTSxHQUFHLE9BQU8sS0FBSyxTQUFTLENBQUMsR0FBR0wsS0FBSTtBQUNuRCxNQUFFLE9BQU8sR0FBRztBQUNaLFdBQU8sRUFBRSxPQUFNO0VBQ2pCO0FBT08sTUFBTSxNQUFNLFdBQ2pCLEVBQUUsV0FBVyxJQUFJLGFBQWEsSUFBSSxXQUFXLEdBQUUsR0FDL0MsU0FBU00sS0FBSSxLQUFpQixPQUFtQixLQUFnQjtBQUMvRCxJQUFBWixPQUFPLEtBQUs7QUFFWixRQUFJLE1BQU0sV0FBVztBQUFHLFlBQU0sSUFBSSxNQUFNLHNCQUFzQjtBQUM5RCxVQUFNLFlBQVk7QUFDbEIsYUFBUyxZQUFZLFNBQXFCLFNBQXFCLE1BQWdCO0FBQzdFLFlBQU0sTUFBTSxXQUFXLE9BQU8sT0FBTyxTQUFTLE1BQU0sR0FBRztBQUN2RCxlQUFTSCxLQUFJLEdBQUdBLEtBQUksUUFBUSxRQUFRQTtBQUFLLFlBQUlBLEVBQUMsS0FBSyxRQUFRQSxFQUFDO0FBQzVELGFBQU87SUFDVDtBQUNBLGFBQVMsYUFBVTtBQUNqQixZQUFNLEtBQUssWUFBWSxHQUFHO0FBQzFCLFlBQU0sVUFBVSxZQUFZLE1BQUs7QUFDakMsWUFBTSxVQUFVLFlBQVksTUFBSztBQUNqQyxZQUFNLElBQUksT0FBTyxTQUFTLFNBQVMsT0FBTztBQUMxQyxVQUFJLE1BQU0sV0FBVyxJQUFJO0FBQ3ZCLGdCQUFRLElBQUksS0FBSztNQUNuQixPQUFPO0FBR0wsY0FBTSxXQUFXLFlBQVksTUFBSztBQUNsQyxjQUFNLE9BQU9VLFlBQVcsUUFBUTtBQUNoQyxRQUFBSSxjQUFhLE1BQU0sR0FBRyxPQUFPLE1BQU0sU0FBUyxDQUFDLEdBQUcsS0FBSztBQUVyRCxjQUFNLE9BQU8sT0FBTyxFQUFFLE9BQU8sS0FBSyxFQUFFLE9BQU8sUUFBUSxFQUFFLFdBQVcsT0FBTztNQUN6RTtBQUNBLFlBQU0sVUFBVSxNQUFNLElBQUksT0FBTyxTQUFTLFdBQVc7QUFDckQsYUFBTyxFQUFFLElBQUksU0FBUyxTQUFTLFFBQU87SUFDeEM7QUFDQSxXQUFPO01BQ0wsU0FBUyxDQUFDLGNBQXlCO0FBQ2pDLFFBQUFYLE9BQU8sU0FBUztBQUNoQixjQUFNLEVBQUUsSUFBSSxTQUFTLFNBQVMsUUFBTyxJQUFLLFdBQVU7QUFDcEQsY0FBTSxNQUFNLElBQUksV0FBVyxVQUFVLFNBQVMsU0FBUztBQUN2RCxjQUFNLElBQUksT0FBTyxTQUFTLFdBQVcsR0FBRztBQUN4QyxjQUFNLE1BQU0sWUFBWSxTQUFTLFNBQVMsSUFBSSxTQUFTLEdBQUcsSUFBSSxTQUFTLFNBQVMsQ0FBQztBQUNqRixZQUFJLElBQUksS0FBSyxVQUFVLE1BQU07QUFDN0IsV0FBRyxLQUFLLENBQUM7QUFDVCxlQUFPO01BQ1Q7TUFDQSxTQUFTLENBQUMsZUFBMEI7QUFDbEMsUUFBQUEsT0FBTyxVQUFVO0FBQ2pCLFlBQUksV0FBVyxTQUFTO0FBQ3RCLGdCQUFNLElBQUksTUFBTSx5Q0FBeUMsU0FBUyxHQUFHO0FBQ3ZFLGNBQU0sRUFBRSxJQUFJLFNBQVMsU0FBUyxRQUFPLElBQUssV0FBVTtBQUNwRCxjQUFNLE9BQU8sV0FBVyxTQUFTLEdBQUcsQ0FBQyxTQUFTO0FBQzlDLGNBQU0sWUFBWSxXQUFXLFNBQVMsQ0FBQyxTQUFTO0FBQ2hELGNBQU0sTUFBTSxZQUFZLFNBQVMsU0FBUyxJQUFJO0FBQzlDLFlBQUksQ0FBQ2EsWUFBVyxLQUFLLFNBQVM7QUFBRyxnQkFBTSxJQUFJLE1BQU0sNEJBQTRCO0FBQzdFLGNBQU0sTUFBTSxNQUFNLElBQUksT0FBTyxTQUFTLElBQUk7QUFDMUMsZ0JBQVEsS0FBSyxDQUFDO0FBQ2QsZ0JBQVEsS0FBSyxDQUFDO0FBQ2QsV0FBRyxLQUFLLENBQUM7QUFDVCxlQUFPO01BQ1Q7O0VBRUosQ0FBQztBQUdILE1BQU0sUUFBUSxDQUFDLE1BQWMsS0FBYSxRQUFnQixDQUFDLFVBQWlCO0FBQzFFLFFBQUksQ0FBQyxPQUFPLGNBQWMsS0FBSyxLQUFLLE1BQU0sU0FBUyxRQUFRO0FBQ3pELFlBQU0sSUFBSSxNQUFNLEdBQUcsSUFBSSxtQkFBbUIsS0FBSyxjQUFjLEdBQUcsS0FBSyxHQUFHLEdBQUc7RUFDL0U7QUFRTyxNQUFNLE1BQU0sV0FDakIsRUFBRSxXQUFXLElBQUksYUFBYSxJQUFJLFdBQVcsR0FBRSxHQUMvQyxTQUFTQyxLQUFJLEtBQWlCLE9BQW1CLEtBQWdCO0FBQy9ELFVBQU0sWUFBWTtBQUVsQixVQUFNLFlBQVksTUFBTSxPQUFPLEdBQUcsS0FBSyxFQUFFO0FBQ3pDLFVBQU0sY0FBYyxNQUFNLGFBQWEsR0FBRyxLQUFLLEVBQUU7QUFDakQsVUFBTSxjQUFjLE1BQU0sU0FBUyxJQUFJLEVBQUU7QUFDekMsVUFBTSxlQUFlLE1BQU0sY0FBYyxJQUFJLEtBQUssS0FBSyxFQUFFO0FBQ3pELElBQUFkLE9BQU8sS0FBSztBQUNaLGdCQUFZLE1BQU0sTUFBTTtBQUN4QixRQUFJLEtBQUs7QUFDUCxNQUFBQSxPQUFPLEdBQUc7QUFDVixnQkFBVSxJQUFJLE1BQU07SUFDdEI7QUFDQSxhQUFTLGFBQVU7QUFDakIsWUFBTSxNQUFNLElBQUk7QUFDaEIsVUFBSSxRQUFRLE1BQU0sUUFBUSxNQUFNLFFBQVE7QUFDdEMsY0FBTSxJQUFJLE1BQU0sK0NBQStDLEdBQUcsUUFBUTtBQUM1RSxZQUFNLEtBQUssWUFBWSxHQUFHO0FBQzFCLFlBQU0sU0FBUyxJQUFJLFdBQVcsR0FBRztBQUNqQyxZQUFNLFVBQVUsSUFBSSxXQUFXLEVBQUU7QUFDakMsWUFBTSxNQUFNLElBQUksS0FBSztBQUVyQixVQUFJLEtBQUssR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUM7QUFDaEQsVUFBSSxVQUFVO0FBQ2QsaUJBQVcsY0FBYyxDQUFDLFNBQVMsTUFBTSxFQUFFLElBQUksR0FBRyxHQUFHO0FBQ25ELGNBQU0sTUFBTSxJQUFJLFVBQVU7QUFDMUIsaUJBQVNILEtBQUksR0FBR0EsS0FBSSxJQUFJLFFBQVFBLE1BQUssR0FBRztBQUV0QyxnQkFBTSxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLElBQUksSUFBSSxJQUFJLElBQUksRUFBRTtBQUNyRCxjQUFJQSxLQUFJLENBQUMsSUFBSTtBQUNiLGNBQUlBLEtBQUksQ0FBQyxJQUFJO0FBQ2IsZUFBSyxFQUFFO1FBQ1Q7TUFDRjtBQUNBLFNBQUcsS0FBSyxDQUFDO0FBQ1QsYUFBTyxFQUFFLFNBQVMsUUFBUSxZQUFZLE1BQU0sRUFBQztJQUMvQztBQUNBLGFBQVMsWUFBWSxRQUFxQixTQUFxQixNQUFnQjtBQUM3RSxZQUFNLE1BQU0sV0FBVyxTQUFTLE1BQU0sU0FBUyxNQUFNLEdBQUc7QUFJeEQsZUFBU0EsS0FBSSxHQUFHQSxLQUFJLElBQUlBO0FBQUssWUFBSUEsRUFBQyxLQUFLLE1BQU1BLEVBQUM7QUFDOUMsVUFBSSxFQUFFLEtBQUs7QUFFWCxZQUFNLE1BQU0sSUFBSSxHQUFHO0FBRW5CLFVBQUksS0FBSyxJQUFJLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUM7QUFDckQsT0FBQyxFQUFFLElBQUksSUFBSSxJQUFJLEdBQUUsSUFBSyxRQUFRLFFBQVEsSUFBSSxJQUFJLElBQUksRUFBRTtBQUNwRCxNQUFDLElBQUksQ0FBQyxJQUFJLElBQU0sSUFBSSxDQUFDLElBQUksSUFBTSxJQUFJLENBQUMsSUFBSSxJQUFNLElBQUksQ0FBQyxJQUFJO0FBQ3ZELGFBQU87SUFDVDtBQUVBLGFBQVMsV0FBVyxRQUFxQixLQUFpQixPQUFpQjtBQUN6RSxVQUFJLFFBQVEsSUFBSSxNQUFLO0FBQ3JCLFlBQU0sRUFBRSxLQUFLO0FBQ2IsYUFBTyxNQUFNLFFBQVEsTUFBTSxPQUFPLEtBQUs7SUFDekM7QUFDQSxXQUFPO01BQ0wsU0FBUyxDQUFDLGNBQXlCO0FBQ2pDLFFBQUFHLE9BQU8sU0FBUztBQUNoQixvQkFBWSxVQUFVLE1BQU07QUFDNUIsY0FBTSxFQUFFLFFBQVEsUUFBTyxJQUFLLFdBQVU7QUFDdEMsY0FBTSxNQUFNLFlBQVksUUFBUSxTQUFTLFNBQVM7QUFDbEQsY0FBTSxNQUFNLElBQUksV0FBVyxVQUFVLFNBQVMsU0FBUztBQUN2RCxZQUFJLElBQUksS0FBSyxVQUFVLE1BQU07QUFDN0IsWUFBSSxJQUFJLFdBQVcsUUFBUSxLQUFLLFNBQVMsQ0FBQztBQUMxQyxlQUFPLEtBQUssQ0FBQztBQUNiLGdCQUFRLEtBQUssQ0FBQztBQUNkLGVBQU87TUFDVDtNQUNBLFNBQVMsQ0FBQyxlQUEwQjtBQUNsQyxRQUFBQSxPQUFPLFVBQVU7QUFDakIscUJBQWEsV0FBVyxNQUFNO0FBQzlCLGNBQU0sTUFBTSxXQUFXLFNBQVMsQ0FBQyxTQUFTO0FBQzFDLGNBQU0sRUFBRSxRQUFRLFFBQU8sSUFBSyxXQUFVO0FBQ3RDLGNBQU0sWUFBWSxXQUFXLFFBQVEsS0FBSyxXQUFXLFNBQVMsR0FBRyxDQUFDLFNBQVMsQ0FBQztBQUM1RSxjQUFNLGNBQWMsWUFBWSxRQUFRLFNBQVMsU0FBUztBQUMxRCxlQUFPLEtBQUssQ0FBQztBQUNiLGdCQUFRLEtBQUssQ0FBQztBQUNkLFlBQUksQ0FBQ2EsWUFBVyxLQUFLLFdBQVc7QUFBRyxnQkFBTSxJQUFJLE1BQU0scUJBQXFCO0FBQ3hFLGVBQU87TUFDVDs7RUFFSixDQUFDOzs7QUM3cUJILE1BQU0sU0FBUyxDQUFDLEdBQWVFLE9BQWUsRUFBRUEsSUFBRyxJQUFJLE9BQVUsRUFBRUEsSUFBRyxJQUFJLFFBQVM7QUFDbkYsTUFBTSxXQUFOLE1BQWM7SUFVWixZQUFZLEtBQVU7QUFUYixXQUFBLFdBQVc7QUFDWCxXQUFBLFlBQVk7QUFDYixXQUFBLFNBQVMsSUFBSSxXQUFXLEVBQUU7QUFDMUIsV0FBQSxJQUFJLElBQUksWUFBWSxFQUFFO0FBQ3RCLFdBQUEsSUFBSSxJQUFJLFlBQVksRUFBRTtBQUN0QixXQUFBLE1BQU0sSUFBSSxZQUFZLENBQUM7QUFDdkIsV0FBQSxNQUFNO0FBQ0osV0FBQSxXQUFXO0FBR25CLFlBQU1DLFNBQVEsR0FBRztBQUNqQixNQUFBQyxPQUFPLEtBQUssRUFBRTtBQUNkLFlBQU0sS0FBSyxPQUFPLEtBQUssQ0FBQztBQUN4QixZQUFNLEtBQUssT0FBTyxLQUFLLENBQUM7QUFDeEIsWUFBTSxLQUFLLE9BQU8sS0FBSyxDQUFDO0FBQ3hCLFlBQU0sS0FBSyxPQUFPLEtBQUssQ0FBQztBQUN4QixZQUFNLEtBQUssT0FBTyxLQUFLLENBQUM7QUFDeEIsWUFBTSxLQUFLLE9BQU8sS0FBSyxFQUFFO0FBQ3pCLFlBQU0sS0FBSyxPQUFPLEtBQUssRUFBRTtBQUN6QixZQUFNLEtBQUssT0FBTyxLQUFLLEVBQUU7QUFHekIsV0FBSyxFQUFFLENBQUMsSUFBSSxLQUFLO0FBQ2pCLFdBQUssRUFBRSxDQUFDLEtBQU0sT0FBTyxLQUFPLE1BQU0sS0FBTTtBQUN4QyxXQUFLLEVBQUUsQ0FBQyxLQUFNLE9BQU8sS0FBTyxNQUFNLEtBQU07QUFDeEMsV0FBSyxFQUFFLENBQUMsS0FBTSxPQUFPLElBQU0sTUFBTSxLQUFNO0FBQ3ZDLFdBQUssRUFBRSxDQUFDLEtBQU0sT0FBTyxJQUFNLE1BQU0sTUFBTztBQUN4QyxXQUFLLEVBQUUsQ0FBQyxJQUFLLE9BQU8sSUFBSztBQUN6QixXQUFLLEVBQUUsQ0FBQyxLQUFNLE9BQU8sS0FBTyxNQUFNLEtBQU07QUFDeEMsV0FBSyxFQUFFLENBQUMsS0FBTSxPQUFPLEtBQU8sTUFBTSxLQUFNO0FBQ3hDLFdBQUssRUFBRSxDQUFDLEtBQU0sT0FBTyxJQUFNLE1BQU0sS0FBTTtBQUN2QyxXQUFLLEVBQUUsQ0FBQyxJQUFLLE9BQU8sSUFBSztBQUN6QixlQUFTRixLQUFJLEdBQUdBLEtBQUksR0FBR0E7QUFBSyxhQUFLLElBQUlBLEVBQUMsSUFBSSxPQUFPLEtBQUssS0FBSyxJQUFJQSxFQUFDO0lBQ2xFO0lBRVEsUUFBUSxNQUFrQixRQUFnQixTQUFTLE9BQUs7QUFDOUQsWUFBTSxRQUFRLFNBQVMsSUFBSSxLQUFLO0FBQ2hDLFlBQU0sRUFBRSxHQUFHLEVBQUMsSUFBSztBQUNqQixZQUFNLEtBQUssRUFBRSxDQUFDO0FBQ2QsWUFBTSxLQUFLLEVBQUUsQ0FBQztBQUNkLFlBQU0sS0FBSyxFQUFFLENBQUM7QUFDZCxZQUFNLEtBQUssRUFBRSxDQUFDO0FBQ2QsWUFBTSxLQUFLLEVBQUUsQ0FBQztBQUNkLFlBQU0sS0FBSyxFQUFFLENBQUM7QUFDZCxZQUFNLEtBQUssRUFBRSxDQUFDO0FBQ2QsWUFBTSxLQUFLLEVBQUUsQ0FBQztBQUNkLFlBQU0sS0FBSyxFQUFFLENBQUM7QUFDZCxZQUFNLEtBQUssRUFBRSxDQUFDO0FBRWQsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFDbEMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFDbEMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFDbEMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFDbEMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFDbEMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLEVBQUU7QUFDbkMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLEVBQUU7QUFDbkMsWUFBTSxLQUFLLE9BQU8sTUFBTSxTQUFTLEVBQUU7QUFFbkMsVUFBSSxLQUFLLEVBQUUsQ0FBQyxLQUFLLEtBQUs7QUFDdEIsVUFBSSxLQUFLLEVBQUUsQ0FBQyxNQUFPLE9BQU8sS0FBTyxNQUFNLEtBQU07QUFDN0MsVUFBSSxLQUFLLEVBQUUsQ0FBQyxNQUFPLE9BQU8sS0FBTyxNQUFNLEtBQU07QUFDN0MsVUFBSSxLQUFLLEVBQUUsQ0FBQyxNQUFPLE9BQU8sSUFBTSxNQUFNLEtBQU07QUFDNUMsVUFBSSxLQUFLLEVBQUUsQ0FBQyxNQUFPLE9BQU8sSUFBTSxNQUFNLE1BQU87QUFDN0MsVUFBSSxLQUFLLEVBQUUsQ0FBQyxLQUFNLE9BQU8sSUFBSztBQUM5QixVQUFJLEtBQUssRUFBRSxDQUFDLE1BQU8sT0FBTyxLQUFPLE1BQU0sS0FBTTtBQUM3QyxVQUFJLEtBQUssRUFBRSxDQUFDLE1BQU8sT0FBTyxLQUFPLE1BQU0sS0FBTTtBQUM3QyxVQUFJLEtBQUssRUFBRSxDQUFDLE1BQU8sT0FBTyxJQUFNLE1BQU0sS0FBTTtBQUM1QyxVQUFJLEtBQUssRUFBRSxDQUFDLEtBQU0sT0FBTyxJQUFLO0FBRTlCLFVBQUksSUFBSTtBQUVSLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDakYsVUFBSSxPQUFPO0FBQ1gsWUFBTTtBQUNOLFlBQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJO0FBQ2hGLFdBQUssT0FBTztBQUNaLFlBQU07QUFFTixVQUFJLEtBQUssSUFBSSxLQUFLLEtBQUssS0FBSyxLQUFLLE1BQU0sSUFBSSxNQUFNLE1BQU0sSUFBSSxNQUFNLE1BQU0sSUFBSTtBQUMzRSxVQUFJLE9BQU87QUFDWCxZQUFNO0FBQ04sWUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDaEYsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLE1BQU0sSUFBSSxNQUFNLE1BQU0sSUFBSTtBQUNyRSxVQUFJLE9BQU87QUFDWCxZQUFNO0FBQ04sWUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDaEYsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxNQUFNLElBQUk7QUFDL0QsVUFBSSxPQUFPO0FBQ1gsWUFBTTtBQUNOLFlBQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJO0FBQ2hGLFdBQUssT0FBTztBQUNaLFlBQU07QUFFTixVQUFJLEtBQUssSUFBSSxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSztBQUMxRCxVQUFJLE9BQU87QUFDWCxZQUFNO0FBQ04sWUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDaEYsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLO0FBQzFELFVBQUksT0FBTztBQUNYLFlBQU07QUFDTixZQUFNLEtBQUssS0FBSyxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDMUUsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLO0FBQzFELFVBQUksT0FBTztBQUNYLFlBQU07QUFDTixZQUFNLEtBQUssS0FBSyxLQUFLLEtBQUssTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJLE1BQU0sTUFBTSxJQUFJO0FBQ3BFLFdBQUssT0FBTztBQUNaLFlBQU07QUFFTixVQUFJLEtBQUssSUFBSSxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSztBQUMxRCxVQUFJLE9BQU87QUFDWCxZQUFNO0FBQ04sWUFBTSxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxNQUFNLElBQUksTUFBTSxNQUFNLElBQUk7QUFDOUQsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLO0FBQzFELFVBQUksT0FBTztBQUNYLFlBQU07QUFDTixZQUFNLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxNQUFNLElBQUk7QUFDeEQsV0FBSyxPQUFPO0FBQ1osWUFBTTtBQUVOLFVBQUksS0FBSyxJQUFJLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLO0FBQzFELFVBQUksT0FBTztBQUNYLFlBQU07QUFDTixZQUFNLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLLEtBQUssS0FBSyxLQUFLO0FBQ25ELFdBQUssT0FBTztBQUNaLFlBQU07QUFFTixXQUFNLEtBQUssS0FBSyxJQUFLO0FBQ3JCLFVBQUssSUFBSSxLQUFNO0FBQ2YsV0FBSyxJQUFJO0FBQ1QsVUFBSSxNQUFNO0FBQ1YsWUFBTTtBQUVOLFFBQUUsQ0FBQyxJQUFJO0FBQ1AsUUFBRSxDQUFDLElBQUk7QUFDUCxRQUFFLENBQUMsSUFBSTtBQUNQLFFBQUUsQ0FBQyxJQUFJO0FBQ1AsUUFBRSxDQUFDLElBQUk7QUFDUCxRQUFFLENBQUMsSUFBSTtBQUNQLFFBQUUsQ0FBQyxJQUFJO0FBQ1AsUUFBRSxDQUFDLElBQUk7QUFDUCxRQUFFLENBQUMsSUFBSTtBQUNQLFFBQUUsQ0FBQyxJQUFJO0lBQ1Q7SUFFUSxXQUFRO0FBQ2QsWUFBTSxFQUFFLEdBQUcsS0FBQUcsS0FBRyxJQUFLO0FBQ25CLFlBQU0sSUFBSSxJQUFJLFlBQVksRUFBRTtBQUM1QixVQUFJLElBQUksRUFBRSxDQUFDLE1BQU07QUFDakIsUUFBRSxDQUFDLEtBQUs7QUFDUixlQUFTSCxLQUFJLEdBQUdBLEtBQUksSUFBSUEsTUFBSztBQUMzQixVQUFFQSxFQUFDLEtBQUs7QUFDUixZQUFJLEVBQUVBLEVBQUMsTUFBTTtBQUNiLFVBQUVBLEVBQUMsS0FBSztNQUNWO0FBQ0EsUUFBRSxDQUFDLEtBQUssSUFBSTtBQUNaLFVBQUksRUFBRSxDQUFDLE1BQU07QUFDYixRQUFFLENBQUMsS0FBSztBQUNSLFFBQUUsQ0FBQyxLQUFLO0FBQ1IsVUFBSSxFQUFFLENBQUMsTUFBTTtBQUNiLFFBQUUsQ0FBQyxLQUFLO0FBQ1IsUUFBRSxDQUFDLEtBQUs7QUFFUixRQUFFLENBQUMsSUFBSSxFQUFFLENBQUMsSUFBSTtBQUNkLFVBQUksRUFBRSxDQUFDLE1BQU07QUFDYixRQUFFLENBQUMsS0FBSztBQUNSLGVBQVNBLEtBQUksR0FBR0EsS0FBSSxJQUFJQSxNQUFLO0FBQzNCLFVBQUVBLEVBQUMsSUFBSSxFQUFFQSxFQUFDLElBQUk7QUFDZCxZQUFJLEVBQUVBLEVBQUMsTUFBTTtBQUNiLFVBQUVBLEVBQUMsS0FBSztNQUNWO0FBQ0EsUUFBRSxDQUFDLEtBQUssS0FBSztBQUViLFVBQUksUUFBUSxJQUFJLEtBQUs7QUFDckIsZUFBU0EsS0FBSSxHQUFHQSxLQUFJLElBQUlBO0FBQUssVUFBRUEsRUFBQyxLQUFLO0FBQ3JDLGFBQU8sQ0FBQztBQUNSLGVBQVNBLEtBQUksR0FBR0EsS0FBSSxJQUFJQTtBQUFLLFVBQUVBLEVBQUMsSUFBSyxFQUFFQSxFQUFDLElBQUksT0FBUSxFQUFFQSxFQUFDO0FBQ3ZELFFBQUUsQ0FBQyxLQUFLLEVBQUUsQ0FBQyxJQUFLLEVBQUUsQ0FBQyxLQUFLLE1BQU87QUFDL0IsUUFBRSxDQUFDLEtBQU0sRUFBRSxDQUFDLE1BQU0sSUFBTSxFQUFFLENBQUMsS0FBSyxNQUFPO0FBQ3ZDLFFBQUUsQ0FBQyxLQUFNLEVBQUUsQ0FBQyxNQUFNLElBQU0sRUFBRSxDQUFDLEtBQUssS0FBTTtBQUN0QyxRQUFFLENBQUMsS0FBTSxFQUFFLENBQUMsTUFBTSxJQUFNLEVBQUUsQ0FBQyxLQUFLLEtBQU07QUFDdEMsUUFBRSxDQUFDLEtBQU0sRUFBRSxDQUFDLE1BQU0sS0FBTyxFQUFFLENBQUMsS0FBSyxJQUFNLEVBQUUsQ0FBQyxLQUFLLE1BQU87QUFDdEQsUUFBRSxDQUFDLEtBQU0s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Qy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQUssS0FBSyxDQUFDO0FBQzdDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBRTlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxFQUFFO0FBQy9DLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0FBQzlDLFlBQU8sTUFBTSxNQUFPO0FBQUcsWUFBTSxLQUFLLE1BQU0sS0FBSyxDQUFDO0lBQ2hEO0FBRUEsUUFBSSxLQUFLO0FBQ1QsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQUcsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQ3ZELFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztBQUFHLFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztBQUN2RCxRQUFJLElBQUksSUFBSyxNQUFNLE1BQU87QUFBRyxRQUFJLElBQUksSUFBSyxNQUFNLE1BQU87QUFDdkQsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQUcsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQ3ZELFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztBQUFHLFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztBQUN2RCxRQUFJLElBQUksSUFBSyxNQUFNLE1BQU87QUFBRyxRQUFJLElBQUksSUFBSyxNQUFNLE1BQU87QUFDdkQsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQUcsUUFBSSxJQUFJLElBQUssTUFBTSxNQUFPO0FBQ3ZELFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztBQUFHLFFBQUksSUFBSSxJQUFLLE1BQU0sTUFBTztFQUN6RDtBQVFNLFdBQVUsUUFDZCxHQUFnQixHQUFnQkksSUFBZ0IsS0FBZ0I7QUFFaEUsUUFBSSxNQUFNLEVBQUUsQ0FBQyxHQUFHLE1BQU0sRUFBRSxDQUFDLEdBQUcsTUFBTSxFQUFFLENBQUMsR0FBRyxNQUFNLEVBQUUsQ0FBQyxHQUM3QyxNQUFNLEVBQUUsQ0FBQyxHQUFHLE1BQU0sRUFBRSxDQUFDLEdBQUcsTUFBTSxFQUFFLENBQUMsR0FBRyxNQUFNLEVBQUUsQ0FBQyxHQUM3QyxNQUFNLEVBQUUsQ0FBQyxHQUFHLE1BQU0sRUFBRSxDQUFDLEdBQUcsTUFBTSxFQUFFLENBQUMsR0FBRyxNQUFNLEVBQUUsQ0FBQyxHQUM3QyxNQUFNQSxHQUFFLENBQUMsR0FBRyxNQUFNQSxHQUFFLENBQUMsR0FBRyxNQUFNQSxHQUFFLENBQUMsR0FBRyxNQUFNQSxHQUFFLENBQUM7QUFDakQsYUFBUyxJQUFJLEdBQUcsSUFBSSxJQUFJLEtBQUssR0FBRztBQUM5QixZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUM5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssQ0FBQztBQUU5QyxZQUFPLE1BQU0sTUFBTztBQUFHLFlBQU0sS0FBSyxNQUFNLEtBQUssRUFBRTtBQUMvQyxZQUFPLE1BQU0sTUFBTztB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QUFDOUQsV0FBTyxjQUFNLFFBQVEsWUFBWSxlQUFlO0FBQUEsRUFDcEQ7QUFFQSxpQkFBZSxZQUFZO0FBQ3ZCLFFBQUksVUFBVSxNQUFNLGVBQWU7QUFDbkMsUUFBSSxTQUFTLFFBQVE7QUFDckIsUUFBSSxXQUFXLENBQUM7QUFFaEIsV0FBTyxRQUFRLFdBQVM7QUFDcEIsVUFBSSxFQUFFLEtBQUssTUFBTSxNQUFNLElBQUk7QUFDM0IsZUFBUyxHQUFHLElBQUksRUFBRSxNQUFNLE1BQU07QUFBQSxJQUNsQyxDQUFDO0FBQ0QsV0FBTztBQUFBLEVBQ1g7IiwKICAibmFtZXMiOiBbImhhc2giLCAiY3J5cHRvIiwgInBhZCIsICJjcnlwdG8iLCAiaXNMRSIsICJpIiwgImkiLCAiY29uY2F0Qnl0ZXMiLCAidXRmOFRvQnl0ZXMiLCAidThhIiwgImkiLCAiYnl0ZXMiLCAiaGV4IiwgInBhZCIsICJfMG4iLCAiXzFuIiwgIl8ybiIsICJudW1iZXIiLCAiZ2NkIiwgIkZwIiwgImdlIiwgImkiLCAiXzBuIiwgIl8xbiIsICJpIiwgImJpdExlbiIsICJpc0xFIiwgIl8wbiIsICJfMW4iLCAiYnl0ZXMiLCAiaXNMRSIsICJfMW4iLCAiXzBuIiwgIl8xbiIsICJpIiwgIkZwIiwgImhleCIsICJzIiwgIl8wbiIsICJfMW4iLCAiXzJuIiwgIl8zbiIsICJfNG4iLCAidG9CeXRlcyIsICJjb25jYXRCeXRlcyIsICJieXRlcyIsICJQb2ludCIsICJpIiwgIm11bCIsICJhIiwgIm1vZE4iLCAibnVtYmVyIiwgImdldFB1YmxpY0tleSIsICJoYXNoIiwgInJhbmRvbUJ5dGVzIiwgInIiLCAiaGFzaCIsICJwYWQiLCAiaSIsICJoYXNoIiwgIl8xbiIsICJfMm4iLCAiXzNuIiwgIl8wbiIsICJjb25jYXRCeXRlcyIsICJjcnlwdG8iLCAidThhIiwgImNyZWF0ZVZpZXciLCAicm90ciIsICJpc0xFIiwgImhleGVzIiwgImkiLCAiYnl0ZXNUb0hleCIsICJieXRlcyIsICJ1OGEiLCAiaGV4IiwgImhleFRvQnl0ZXMiLCAidXRmOFRvQnl0ZXMiLCAidG9CeXRlcyIsICJ1OGEiLCAiY29uY2F0Qnl0ZXMiLCAicGFkIiwgIkhhc2giLCAid3JhcENvbnN0cnVjdG9yIiwgInRvQnl0ZXMiLCAicmFuZG9tQnl0ZXMiLCAiY3J5cHRvIiwgIm51bWJlciIsICJieXRlcyIsICJoYXNoIiwgImV4aXN0cyIsICJvdXRwdXQiLCAic2V0QmlnVWludDY0IiwgImlzTEUiLCAiU0hBMiIsICJIYXNoIiwgImNyZWF0ZVZpZXciLCAidG9CeXRlcyIsICJpIiwgIkNoaSIsICJNYWoiLCAiU0hBMjU2X0siLCAiSVYiLCAiU0hBMjU2X1ciLCAiU0hBMjU2IiwgIlNIQTIiLCAiaSIsICJyb3RyIiwgInNoYTI1NiIsICJ3cmFwQ29uc3RydWN0b3IiLCAid3JhcCIsICJpIiwgImRlY29kZSIsICJhbHBoYWJldCIsICJwYWRkaW5nIiwgImJ5dGVzIiwgImkiLCAiaSIsICJsaW1pdCIsICJkZWNvZGUiLCAiX3dvcmRzIiwgIm51bWJlciIsICJib29sIiwgImJ5dGVzIiwgImV4aXN0cyIsICJvdXRwdXQiLCAiYnl0ZXMiLCAiY3JlYXRlVmlldyIsICJpc0xFIiwgInV0ZjhUb0J5dGVzIiwgInRvQnl0ZXMiLCAidXRmOFRvQnl0ZXMiLCAiZXF1YWxCeXRlcyIsICJpIiwgInNldEJpZ1VpbnQ2NCIsICJpc0xFIiwgImkiLCAiYnl0ZXMiLCAidG9CeXRlcyIsICJjcmVhdGVWaWV3IiwgImV4aXN0cyIsICJvdXRwdXQiLCAiQkxPQ0tfU0laRSIsICJQT0xZIiwgIm11bDIiLCAiaSIsICJzYm94IiwgInNib3gyIiwgImJ5dGVzIiwgInQwIiwgInQxIiwgInQyIiwgInQzIiwgImN0ciIsICJpc0xFIiwgImNyZWF0ZVZpZXciLCAiZWNiIiwgImNiYyIsICJjZmIiLCAic2V0QmlnVWludDY0IiwgImdjbSIsICJlcXVhbEJ5dGVzIiwgInNpdiIsICJpIiwgInRvQnl0ZXMiLCAiYnl0ZXMiLCAicGFkIiwgImV4aXN0cyIsICJvdXRwdXQiLCAid3JhcENvbnN0cnVjdG9yV2l0aEtleSIsICJzaWdtYSIsICJvdXRwdXQiLCAibnVtYmVyIiwgImJvb2wiLCAiYnl0ZXMiLCAiaSIsICJaRVJPUzE2IiwgIlpFUk9TMzIiLCAiY29tcHV0ZVRhZyIsICJjcmVhdGVWaWV3IiwgInNldEJpZ1VpbnQ2NCIsICJieXRlcyIsICJvdXRwdXQiLCAiZXF1YWxCeXRlcyIsICJITUFDIiwgIkhhc2giLCAiaGFzaCIsICJ0b0J5dGVzIiwgInBhZCIsICJpIiwgImhtYWMiLCAiaGFzaCIsICJobWFjIiwgInRvQnl0ZXMiLCAiSE1BQyIsICJfX2RlZlByb3AiLCAiX19leHBvcnQiLCAidXRpbHNfZXhwb3J0cyIsICJfX2V4cG9ydCIsICJieXRlc1RvSGV4IiwgImhleFRvQnl0ZXMiLCAiaGFzaCIsICJzaGEyNTYiLCAiX19leHBvcnQiLCAiY2hhbGxlbmdlIiwgIl9fZXhwb3J0IiwgIm51bWJlciIsICJieXRlc1RvSGV4IiwgImhleCIsICJoZXhUb0J5dGVzIiwgImJ5dGVzIiwgImNvbmNhdEJ5dGVzIiwgIl9fZXhwb3J0IiwgImRlY3J5cHQiLCAiZW5jcnlwdCIsICJieXRlc1RvSGV4IiwgInJhbmRvbUJ5dGVzIiwgImhleCIsICJzaGEyNTYiLCAiZGVjcnlwdDIiLCAiZW5jcnlwdDIiLCAiY29uY2F0Qnl0ZXMiLCAiaG1hYyIsICJlcXVhbEJ5dGVzIiwgIndyYXAiLCAiaGFzaCIsICJpIiwgImNvbXBsZXRlIiwgInRhcmdldCIsICJzdG9yYWdlIiwgInNlbmRSZXNwb25zZSIsICJieXRlc1RvSGV4IiwgIm5pcDQ0RW5jcnlwdCIsICJuaXA0NERlY3J5cHQiLCAiaGV4VG9CeXRlcyJdCn0K
