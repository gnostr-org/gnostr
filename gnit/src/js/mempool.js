;(function (f) {
  if (typeof exports === "object" && typeof module !== "undefined") {
    module.exports = f()
  } else if (typeof define === "function" && define.amd) {
    define([], f)
  } else {
    var g
    if (typeof window !== "undefined") {
      g = window
    } else if (typeof global !== "undefined") {
      g = global
    } else if (typeof self !== "undefined") {
      g = self
    } else {
      g = this
    }
    g.mempoolJS = f()
  }
})(function () {
  var define, module, exports
  return (function () {
    function r(e, n, t) {
      function o(i, f) {
        if (!n[i]) {
          if (!e[i]) {
            var c = "function" == typeof require && require
            if (!f && c) return c(i, !0)
            if (u) return u(i, !0)
            var a = new Error("Cannot find module '" + i + "'")
            throw ((a.code = "MODULE_NOT_FOUND"), a)
          }
          var p = (n[i] = { exports: {} })
          e[i][0].call(
            p.exports,
            function (r) {
              var n = e[i][1][r]
              return o(n || r)
            },
            p,
            p.exports,
            r,
            e,
            n,
            t,
          )
        }
        return n[i].exports
      }
      for (var u = "function" == typeof require && require, i = 0; i < t.length; i++) o(t[i])
      return o
    }
    return r
  })()(
    {
      1: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useAddresses = void 0
          var useAddresses = function (api) {
            var getAddress = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/address/".concat(params.address))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAddressTxs = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data_1, data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      if (!params.after_txid) return [3 /*break*/, 2]
                      return [
                        4 /*yield*/,
                        api.get("/address/".concat(params.address, "/txs?after_txid=").concat(params.after_txid)),
                      ]
                    case 1:
                      data_1 = _a.sent().data
                      return [2 /*return*/, data_1]
                    case 2:
                      return [4 /*yield*/, api.get("/address/".concat(params.address, "/txs"))]
                    case 3:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAddressTxsChain = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/address/".concat(params.address, "/txs/chain"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAddressTxsMempool = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/address/".concat(params.address, "/txs/mempool"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAddressTxsUtxo = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/address/".concat(params.address, "/utxo"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getAddress: getAddress,
              getAddressTxs: getAddressTxs,
              getAddressTxsChain: getAddressTxsChain,
              getAddressTxsMempool: getAddressTxsMempool,
              getAddressTxsUtxo: getAddressTxsUtxo,
            }
          }
          exports.useAddresses = useAddresses
        },
        {},
      ],
      2: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useBlocks = void 0
          var useBlocks = function (api) {
            var getBlock = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/block/".concat(params.hash))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockStatus = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/status"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxs = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txs/").concat(params.start_index))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxids = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txids"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxid = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txid/").concat(params.index))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockRaw = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/raw"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockHeader = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/header"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockHeight = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block-height/".concat(params.height))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocks = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/blocks/".concat(params.start_height))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocksTipHeight = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/blocks/tip/height")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocksTipHash = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/blocks/tip/hash")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getBlock: getBlock,
              getBlocks: getBlocks,
              getBlockStatus: getBlockStatus,
              getBlockTxs: getBlockTxs,
              getBlockTxid: getBlockTxid,
              getBlockTxids: getBlockTxids,
              getBlockRaw: getBlockRaw,
              getBlockHeader: getBlockHeader,
              getBlockHeight: getBlockHeight,
              getBlocksTipHash: getBlocksTipHash,
              getBlocksTipHeight: getBlocksTipHeight,
            }
          }
          exports.useBlocks = useBlocks
        },
        {},
      ],
      3: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useDifficulty = void 0
          var useDifficulty = function (api) {
            var getDifficultyAdjustment = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/difficulty-adjustment")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getHashrate = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/mining/hashrate/".concat(params.interval))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getDifficultyAdjustment: getDifficultyAdjustment,
              getHashrate: getHashrate,
            }
          }
          exports.useDifficulty = useDifficulty
        },
        {},
      ],
      4: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useFees = void 0
          var useFees = function (api) {
            var getFeesRecommended = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/fees/recommended")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getFeesMempoolBlocks = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/fees/mempool-blocks")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getCPFP = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/cpfp/".concat(params.txid))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getFeesRecommended: getFeesRecommended,
              getFeesMempoolBlocks: getFeesMempoolBlocks,
              getCPFP: getCPFP,
            }
          }
          exports.useFees = useFees
        },
        {},
      ],
      5: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useLightning = void 0
          var useLightning = function (api) {
            var getNetworkStats = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/statistics/latest")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getNodesInCountry = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/country/".concat(params.country))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getNodesStatsPerCountry = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/countries")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getNodesHostedByISP = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/isp/".concat(params.isp))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getISPRanking = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/isp-ranking")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getLiquidityRanking = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/rankings/liquidity")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getConnectivityRanking = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/rankings/connectivity")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getOldestNodes = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/rankings/age")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getNodeStats = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/".concat(params.public_key))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getHistoricalNodeStats = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/nodes/".concat(params.public_key, "/statistics"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getChannel = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/channels/".concat(params.id))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getChannelsFromTxIds = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [
                        4 /*yield*/,
                        api.get("/v1/lightning/channels/txids?txId[]=".concat(params.txId.join("&txId[]="))),
                      ]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getChannelsFromNodePublicKey = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [
                        4 /*yield*/,
                        api.get(
                          "/v1/lightning/channels?public_key="
                            .concat(params.public_key, "&status=")
                            .concat(params.status, "&index=")
                            .concat(params === null || params === void 0 ? void 0 : params.index),
                        ),
                      ]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getChannelsGeodata = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/channels-geo")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getChannelsGeodataByPublicKey = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/lightning/channels-geo/".concat(params.public_key))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getNetworkStats: getNetworkStats,
              getNodesInCountry: getNodesInCountry,
              getNodesStatsPerCountry: getNodesStatsPerCountry,
              getNodesHostedByISP: getNodesHostedByISP,
              getISPRanking: getISPRanking,
              getLiquidityRanking: getLiquidityRanking,
              getConnectivityRanking: getConnectivityRanking,
              getOldestNodes: getOldestNodes,
              getNodeStats: getNodeStats,
              getHistoricalNodeStats: getHistoricalNodeStats,
              getChannel: getChannel,
              getChannelsFromTxIds: getChannelsFromTxIds,
              getChannelsFromNodePublicKey: getChannelsFromNodePublicKey,
              getChannelsGeodata: getChannelsGeodata,
              getChannelsGeodataByPublicKey: getChannelsGeodataByPublicKey,
            }
          }
          exports.useLightning = useLightning
        },
        {},
      ],
      6: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useMempool = void 0
          var useMempool = function (api) {
            var getMempool = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/mempool")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getMempoolTxids = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/mempool/txids")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getMempoolRecent = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/mempool/recent")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getMempool: getMempool,
              getMempoolTxids: getMempoolTxids,
              getMempoolRecent: getMempoolRecent,
            }
          }
          exports.useMempool = useMempool
        },
        {},
      ],
      7: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useTransactions = void 0
          var useTransactions = function (api) {
            var getTx = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxStatus = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/status"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxHex = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/hex"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxRaw = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/raw"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxMerkleBlockProof = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/merkleblock-proof"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxMerkleProof = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/merkle-proof"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxOutspend = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/outspend/").concat(params.vout))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxOutspends = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/outspends"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var postTx = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.post("/tx", params.txhex)]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getTx: getTx,
              getTxStatus: getTxStatus,
              getTxHex: getTxHex,
              getTxRaw: getTxRaw,
              getTxMerkleBlockProof: getTxMerkleBlockProof,
              getTxMerkleProof: getTxMerkleProof,
              getTxOutspend: getTxOutspend,
              getTxOutspends: getTxOutspends,
              postTx: postTx,
            }
          }
          exports.useTransactions = useTransactions
        },
        {},
      ],
      8: [
        function (require, module, exports) {
          "use strict"
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useWebsocket = void 0
          var ws_client_browser_1 = require("../../services/ws/ws-client-browser")
          var ws_client_node_1 = require("../../services/ws/ws-client-node")
          var useWebsocket = function (hostname, network, protocol) {
            if (!protocol) {
              ;(hostname === null || hostname === void 0 ? void 0 : hostname.includes("localhost"))
                ? (protocol = "ws")
                : (protocol = "wss")
            } else if (protocol === "http" || protocol === "ws") {
              protocol = "ws"
            } else {
              protocol = "wss"
            }
            if (network && ["testnet", "testnet4", "signet"].includes(network)) {
              network = "/".concat(network)
            } else {
              network = ""
            }
            var wsEndpoint = "".concat(protocol, "://").concat(hostname).concat(network, "/api/v1/ws")
            return {
              wsInit: function () {
                return (0, ws_client_node_1.wsInit)(wsEndpoint)
              },
              wsInitBrowser: function () {
                return (0, ws_client_browser_1.wsInit)(wsEndpoint)
              },
              wsWantData: function (ws, options) {
                return (0, ws_client_node_1.wsWantData)(ws, options)
              },
              wsWantDataBrowser: function (ws, options) {
                return (0, ws_client_browser_1.wsWantData)(ws, options)
              },
              wsStopData: function (ws) {
                return (0, ws_client_node_1.wsStopData)(ws)
              },
              wsStopDataBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopData)(ws)
              },
              wsTrackAddress: function (ws, address) {
                return (0, ws_client_node_1.wsTrackAddress)(ws, address)
              },
              wsTrackAddressBrowser: function (ws, address) {
                return (0, ws_client_browser_1.wsTrackAddress)(ws, address)
              },
              wsStopTrackingAddress: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingAddress)(ws)
              },
              wsStopTrackingAddressBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingAddress)(ws)
              },
              wsTrackAddresses: function (ws, addresses) {
                return (0, ws_client_node_1.wsTrackAddresses)(ws, addresses)
              },
              wsTrackAddressesBrowser: function (ws, addresses) {
                return (0, ws_client_browser_1.wsTrackAddresses)(ws, addresses)
              },
              wsStopTrackingAddresses: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingAddresses)(ws)
              },
              wsStopTrackingAddressesBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingAddresses)(ws)
              },
              wsTrackTransaction: function (ws, txid) {
                return (0, ws_client_node_1.wsTrackTransaction)(ws, txid)
              },
              wsTrackTransactionBrowser: function (ws, txid) {
                return (0, ws_client_browser_1.wsTrackTransaction)(ws, txid)
              },
              wsStopTrackingTransaction: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingTransaction)(ws)
              },
              wsStopTrackingTransactionBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingTransaction)(ws)
              },
              wsTrackRbfSummary: function (ws) {
                return (0, ws_client_node_1.wsTrackRbfSummary)(ws)
              },
              wsTrackRbfSummaryBrowser: function (ws) {
                return (0, ws_client_browser_1.wsTrackRbfSummary)(ws)
              },
              wsStopTrackingRbfSummary: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingRbfSummary)(ws)
              },
              wsStopTrackingRbfSummaryBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingRbfSummary)(ws)
              },
              wsTrackRbf: function (ws, fullRbf) {
                return (0, ws_client_node_1.wsTrackRbf)(ws, fullRbf)
              },
              wsTrackRbfBrowser: function (ws, fullRbf) {
                return (0, ws_client_browser_1.wsTrackRbf)(ws, fullRbf)
              },
              wsStopTrackingRbf: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingRbf)(ws)
              },
              wsStopTrackingRbfBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingRbf)(ws)
              },
              wsTrackMempoolBlock: function (ws, index) {
                return (0, ws_client_node_1.wsTrackMempoolBlock)(ws, index)
              },
              wsTrackMempoolBlockBrowser: function (ws, index) {
                return (0, ws_client_browser_1.wsTrackMempoolBlock)(ws, index)
              },
              wsStopTrackingMempoolBlock: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingMempoolBlock)(ws)
              },
              wsStopTrackingMempoolBlockBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingMempoolBlock)(ws)
              },
            }
          }
          exports.useWebsocket = useWebsocket
        },
        { "../../services/ws/ws-client-browser": 18, "../../services/ws/ws-client-node": 19 },
      ],
      9: [
        function (require, module, exports) {
          arguments[4][1][0].apply(exports, arguments)
        },
        { dup: 1 },
      ],
      10: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useAssets = void 0
          var useAssets = function (api) {
            var getAsset = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/asset/".concat(params.asset_id))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAssetIcon = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/asset/".concat(params.asset_id, "/icon"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAssetTxs = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var paramsMempools, data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      paramsMempools = params.is_mempool === true ? "/mempool" : "/chain"
                      return [4 /*yield*/, api.get("/asset/".concat(params.asset_id, "/txs").concat(paramsMempools))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAssetSupply = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var paramDecimal, data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      paramDecimal = params.decimal === true ? "/decimal" : ""
                      return [4 /*yield*/, api.get("/asset/".concat(params.asset_id, "/supply").concat(paramDecimal))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getAssetsIcons = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/assets/icons")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getAsset: getAsset,
              getAssetIcon: getAssetIcon,
              getAssetTxs: getAssetTxs,
              getAssetSupply: getAssetSupply,
              getAssetsIcons: getAssetsIcons,
            }
          }
          exports.useAssets = useAssets
        },
        {},
      ],
      11: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useBlocks = void 0
          var useBlocks = function (api) {
            var getBlock = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/block/".concat(params.hash))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockStatus = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/status"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxs = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txs/").concat(params.start_index))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxids = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txids"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockTxid = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/txid/").concat(params.index))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockRaw = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block/".concat(params.hash, "/raw"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlockHeight = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/block-height/".concat(params.height))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocks = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/v1/blocks/".concat(params.start_height))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocksTipHeight = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/blocks/tip/height")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getBlocksTipHash = function () {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/blocks/tip/hash")]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getBlock: getBlock,
              getBlocks: getBlocks,
              getBlockStatus: getBlockStatus,
              getBlockTxs: getBlockTxs,
              getBlockTxid: getBlockTxid,
              getBlockTxids: getBlockTxids,
              getBlockRaw: getBlockRaw,
              getBlockHeight: getBlockHeight,
              getBlocksTipHash: getBlocksTipHash,
              getBlocksTipHeight: getBlocksTipHeight,
            }
          }
          exports.useBlocks = useBlocks
        },
        {},
      ],
      12: [
        function (require, module, exports) {
          arguments[4][4][0].apply(exports, arguments)
        },
        { dup: 4 },
      ],
      13: [
        function (require, module, exports) {
          arguments[4][6][0].apply(exports, arguments)
        },
        { dup: 6 },
      ],
      14: [
        function (require, module, exports) {
          "use strict"
          var __awaiter =
            (this && this.__awaiter) ||
            function (thisArg, _arguments, P, generator) {
              function adopt(value) {
                return value instanceof P
                  ? value
                  : new P(function (resolve) {
                      resolve(value)
                    })
              }
              return new (P || (P = Promise))(function (resolve, reject) {
                function fulfilled(value) {
                  try {
                    step(generator.next(value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function rejected(value) {
                  try {
                    step(generator["throw"](value))
                  } catch (e) {
                    reject(e)
                  }
                }
                function step(result) {
                  result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected)
                }
                step((generator = generator.apply(thisArg, _arguments || [])).next())
              })
            }
          var __generator =
            (this && this.__generator) ||
            function (thisArg, body) {
              var _ = {
                  label: 0,
                  sent: function () {
                    if (t[0] & 1) throw t[1]
                    return t[1]
                  },
                  trys: [],
                  ops: [],
                },
                f,
                y,
                t,
                g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype)
              return (
                (g.next = verb(0)),
                (g["throw"] = verb(1)),
                (g["return"] = verb(2)),
                typeof Symbol === "function" &&
                  (g[Symbol.iterator] = function () {
                    return this
                  }),
                g
              )
              function verb(n) {
                return function (v) {
                  return step([n, v])
                }
              }
              function step(op) {
                if (f) throw new TypeError("Generator is already executing.")
                while ((g && ((g = 0), op[0] && (_ = 0)), _))
                  try {
                    if (
                      ((f = 1),
                      y &&
                        (t =
                          op[0] & 2
                            ? y["return"]
                            : op[0]
                              ? y["throw"] || ((t = y["return"]) && t.call(y), 0)
                              : y.next) &&
                        !(t = t.call(y, op[1])).done)
                    )
                      return t
                    if (((y = 0), t)) op = [op[0] & 2, t.value]
                    switch (op[0]) {
                      case 0:
                      case 1:
                        t = op
                        break
                      case 4:
                        _.label++
                        return { value: op[1], done: false }
                      case 5:
                        _.label++
                        y = op[1]
                        op = [0]
                        continue
                      case 7:
                        op = _.ops.pop()
                        _.trys.pop()
                        continue
                      default:
                        if (!((t = _.trys), (t = t.length > 0 && t[t.length - 1])) && (op[0] === 6 || op[0] === 2)) {
                          _ = 0
                          continue
                        }
                        if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) {
                          _.label = op[1]
                          break
                        }
                        if (op[0] === 6 && _.label < t[1]) {
                          _.label = t[1]
                          t = op
                          break
                        }
                        if (t && _.label < t[2]) {
                          _.label = t[2]
                          _.ops.push(op)
                          break
                        }
                        if (t[2]) _.ops.pop()
                        _.trys.pop()
                        continue
                    }
                    op = body.call(thisArg, _)
                  } catch (e) {
                    op = [6, e]
                    y = 0
                  } finally {
                    f = t = 0
                  }
                if (op[0] & 5) throw op[1]
                return { value: op[0] ? op[1] : void 0, done: true }
              }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useTransactions = void 0
          var useTransactions = function (api) {
            var getTx = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxStatus = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/status"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxHex = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/hex"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxRaw = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/raw"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxMerkleProof = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/merkle-proof"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxOutspend = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/outspend/").concat(params.vout))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var getTxOutspends = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.get("/tx/".concat(params.txid, "/outspends"))]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            var postTx = function (params) {
              return __awaiter(void 0, void 0, void 0, function () {
                var data
                return __generator(this, function (_a) {
                  switch (_a.label) {
                    case 0:
                      return [4 /*yield*/, api.post("/tx", params.txhex)]
                    case 1:
                      data = _a.sent().data
                      return [2 /*return*/, data]
                  }
                })
              })
            }
            return {
              getTx: getTx,
              getTxStatus: getTxStatus,
              getTxHex: getTxHex,
              getTxRaw: getTxRaw,
              getTxMerkleProof: getTxMerkleProof,
              getTxOutspend: getTxOutspend,
              getTxOutspends: getTxOutspends,
              postTx: postTx,
            }
          }
          exports.useTransactions = useTransactions
        },
        {},
      ],
      15: [
        function (require, module, exports) {
          "use strict"
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.useWebsocket = void 0
          var ws_client_browser_1 = require("../../services/ws/ws-client-browser")
          var ws_client_node_1 = require("../../services/ws/ws-client-node")
          var useWebsocket = function (hostname, network, protocol) {
            if (!protocol) {
              ;(hostname === null || hostname === void 0 ? void 0 : hostname.includes("localhost"))
                ? (protocol = "ws")
                : (protocol = "wss")
            } else if (protocol === "http" || protocol === "ws") {
              protocol = "ws"
            } else {
              protocol = "wss"
            }
            if (network && ["testnet", "liquidtestnet"].includes(network)) {
              network = "/liquidtestnet"
            } else {
              network = "/liquid"
            }
            var wsEndpoint = "".concat(protocol, "://").concat(hostname).concat(network, "/api/v1/ws")
            return {
              wsInit: function () {
                return (0, ws_client_node_1.wsInit)(wsEndpoint)
              },
              wsInitBrowser: function () {
                return (0, ws_client_browser_1.wsInit)(wsEndpoint)
              },
              wsWantData: function (ws, options) {
                return (0, ws_client_node_1.wsWantData)(ws, options)
              },
              wsWantDataBrowser: function (ws, options) {
                return (0, ws_client_browser_1.wsWantData)(ws, options)
              },
              wsStopData: function (ws) {
                return (0, ws_client_node_1.wsStopData)(ws)
              },
              wsStopDataBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopData)(ws)
              },
              wsTrackAddress: function (ws, address) {
                return (0, ws_client_node_1.wsTrackAddress)(ws, address)
              },
              wsTrackAddressBrowser: function (ws, address) {
                return (0, ws_client_browser_1.wsTrackAddress)(ws, address)
              },
              wsStopTrackingAddress: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingAddress)(ws)
              },
              wsStopTrackingAddressBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingAddress)(ws)
              },
              wsTrackAddresses: function (ws, addresses) {
                return (0, ws_client_node_1.wsTrackAddresses)(ws, addresses)
              },
              wsTrackAddressesBrowser: function (ws, addresses) {
                return (0, ws_client_browser_1.wsTrackAddresses)(ws, addresses)
              },
              wsStopTrackingAddresses: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingAddresses)(ws)
              },
              wsStopTrackingAddressesBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingAddresses)(ws)
              },
              wsTrackTransaction: function (ws, txid) {
                return (0, ws_client_node_1.wsTrackTransaction)(ws, txid)
              },
              wsTrackTransactionBrowser: function (ws, txid) {
                return (0, ws_client_browser_1.wsTrackTransaction)(ws, txid)
              },
              wsStopTrackingTransaction: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingTransaction)(ws)
              },
              wsStopTrackingTransactionBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingTransaction)(ws)
              },
              wsTrackMempoolBlock: function (ws, index) {
                return (0, ws_client_node_1.wsTrackMempoolBlock)(ws, index)
              },
              wsTrackMempoolBlockBrowser: function (ws, index) {
                return (0, ws_client_browser_1.wsTrackMempoolBlock)(ws, index)
              },
              wsStopTrackingMempoolBlock: function (ws) {
                return (0, ws_client_node_1.wsStopTrackingMempoolBlock)(ws)
              },
              wsStopTrackingMempoolBlockBrowser: function (ws) {
                return (0, ws_client_browser_1.wsStopTrackingMempoolBlock)(ws)
              },
            }
          }
          exports.useWebsocket = useWebsocket
        },
        { "../../services/ws/ws-client-browser": 18, "../../services/ws/ws-client-node": 19 },
      ],
      16: [
        function (require, module, exports) {
          "use strict"
          var index_1 = require("./services/api/index")
          var addresses_1 = require("./app/bitcoin/addresses")
          var blocks_1 = require("./app/bitcoin/blocks")
          var difficulty_1 = require("./app/bitcoin/difficulty")
          var fees_1 = require("./app/bitcoin/fees")
          var lightning_1 = require("./app/bitcoin/lightning")
          var mempool_1 = require("./app/bitcoin/mempool")
          var transactions_1 = require("./app/bitcoin/transactions")
          var websocket_1 = require("./app/bitcoin/websocket")
          var assets_1 = require("./app/liquid/assets")
          var addresses_2 = require("./app/liquid/addresses")
          var blocks_2 = require("./app/liquid/blocks")
          var fees_2 = require("./app/liquid/fees")
          var mempool_2 = require("./app/liquid/mempool")
          var transactions_2 = require("./app/liquid/transactions")
          var websocket_2 = require("./app/liquid/websocket")
          var hostnameEndpointDefault = "mempool.space"
          var networkEndpointDefault = "main"
          var mempool = function (_a) {
            var _b =
                _a === void 0
                  ? {
                      hostname: hostnameEndpointDefault,
                      network: networkEndpointDefault,
                    }
                  : _a,
              hostname = _b.hostname,
              network = _b.network,
              protocol = _b.protocol,
              config = _b.config
            if (!hostname) hostname = hostnameEndpointDefault
            if (!network) network = networkEndpointDefault
            var apiBitcoin = (0, index_1.makeBitcoinAPI)({
              hostname: hostname,
              network: network,
              protocol: protocol,
              config: config,
            }).api
            var apiLiquid = (0, index_1.makeLiquidAPI)({
              hostname: hostname,
              network: network,
              protocol: protocol,
              config: config,
            }).api
            return {
              bitcoin: {
                addresses: (0, addresses_1.useAddresses)(apiBitcoin),
                blocks: (0, blocks_1.useBlocks)(apiBitcoin),
                difficulty: (0, difficulty_1.useDifficulty)(apiBitcoin),
                fees: (0, fees_1.useFees)(apiBitcoin),
                lightning: (0, lightning_1.useLightning)(apiBitcoin),
                mempool: (0, mempool_1.useMempool)(apiBitcoin),
                transactions: (0, transactions_1.useTransactions)(apiBitcoin),
                websocket: (0, websocket_1.useWebsocket)(hostname, network, protocol),
              },
              liquid: {
                addresses: (0, addresses_2.useAddresses)(apiLiquid),
                assets: (0, assets_1.useAssets)(apiLiquid),
                blocks: (0, blocks_2.useBlocks)(apiLiquid),
                fees: (0, fees_2.useFees)(apiLiquid),
                mempool: (0, mempool_2.useMempool)(apiLiquid),
                transactions: (0, transactions_2.useTransactions)(apiLiquid),
                websocket: (0, websocket_2.useWebsocket)(hostname, network, protocol),
              },
            }
          }
          mempool.default = mempool
          module.exports = mempool
        },
        {
          "./app/bitcoin/addresses": 1,
          "./app/bitcoin/blocks": 2,
          "./app/bitcoin/difficulty": 3,
          "./app/bitcoin/fees": 4,
          "./app/bitcoin/lightning": 5,
          "./app/bitcoin/mempool": 6,
          "./app/bitcoin/transactions": 7,
          "./app/bitcoin/websocket": 8,
          "./app/liquid/addresses": 9,
          "./app/liquid/assets": 10,
          "./app/liquid/blocks": 11,
          "./app/liquid/fees": 12,
          "./app/liquid/mempool": 13,
          "./app/liquid/transactions": 14,
          "./app/liquid/websocket": 15,
          "./services/api/index": 17,
        },
      ],
      17: [
        function (require, module, exports) {
          "use strict"
          var __assign =
            (this && this.__assign) ||
            function () {
              __assign =
                Object.assign ||
                function (t) {
                  for (var s, i = 1, n = arguments.length; i < n; i++) {
                    s = arguments[i]
                    for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p)) t[p] = s[p]
                  }
                  return t
                }
              return __assign.apply(this, arguments)
            }
          var __importDefault =
            (this && this.__importDefault) ||
            function (mod) {
              return mod && mod.__esModule ? mod : { default: mod }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.makeLiquidAPI = exports.makeBitcoinAPI = void 0
          var axios_1 = __importDefault(require("axios"))
          var makeBitcoinAPI = function (_a) {
            var hostname = _a.hostname,
              network = _a.network,
              protocol = _a.protocol,
              config = _a.config
            if (!protocol) {
              ;(hostname === null || hostname === void 0 ? void 0 : hostname.includes("localhost"))
                ? (protocol = "http")
                : (protocol = "https")
            }
            if (network && ["testnet", "testnet4", "signet"].includes(network)) {
              network = "/".concat(network)
            } else {
              network = ""
            }
            var api = axios_1.default.create(
              __assign({ baseURL: "".concat(protocol, "://").concat(hostname).concat(network, "/api/") }, config),
            )
            return {
              api: api,
            }
          }
          exports.makeBitcoinAPI = makeBitcoinAPI
          var makeLiquidAPI = function (_a) {
            var hostname = _a.hostname,
              network = _a.network,
              protocol = _a.protocol,
              config = _a.config
            if (!protocol) {
              ;(hostname === null || hostname === void 0 ? void 0 : hostname.includes("localhost"))
                ? (protocol = "http")
                : (protocol = "https")
            }
            if (network && ["testnet", "liquidtestnet"].includes(network)) {
              network = "/liquidtestnet"
            } else {
              network = "/liquid"
            }
            var api = axios_1.default.create(
              __assign({ baseURL: "".concat(protocol, "://").concat(hostname).concat(network, "/api/") }, config),
            )
            return {
              api: api,
            }
          }
          exports.makeLiquidAPI = makeLiquidAPI
          exports.default = {
            makeBitcoinAPI: exports.makeBitcoinAPI,
            makeLiquidAPI: exports.makeLiquidAPI,
          }
        },
        { axios: 20 },
      ],
      18: [
        function (require, module, exports) {
          "use strict"
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.wsStopTrackingMempoolBlock =
            exports.wsTrackMempoolBlock =
            exports.wsStopTrackingRbf =
            exports.wsTrackRbf =
            exports.wsStopTrackingRbfSummary =
            exports.wsTrackRbfSummary =
            exports.wsStopTrackingTransaction =
            exports.wsTrackTransaction =
            exports.wsStopTrackingAddresses =
            exports.wsTrackAddresses =
            exports.wsStopTrackingAddress =
            exports.wsTrackAddress =
            exports.wsStopData =
            exports.wsWantData =
            exports.wsInit =
              void 0
          var TIMEOUT_DURATION = 5000
          var MAX_RETRY_COUNT = 3
          var wsActionWrapper = function (ws, action) {
            var retryCount = 0
            var sendData = function () {
              if (ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify(action))
              } else if (retryCount < MAX_RETRY_COUNT) {
                retryCount++
                console.log("WebSocket connection is not open. Retrying...")
                setTimeout(sendData, TIMEOUT_DURATION)
              } else {
                console.log("WebSocket connection is not open. Max retry count reached.")
              }
            }
            sendData()
          }
          var wsInit = function (endpoint) {
            var ws = new WebSocket(endpoint)
            ws.addEventListener("open", function open() {
              ws.send(JSON.stringify({ action: "init" }))
            })
            return ws
          }
          exports.wsInit = wsInit
          var wsWantData = function (ws, options) {
            wsActionWrapper(ws, { action: "want", data: options })
          }
          exports.wsWantData = wsWantData
          var wsStopData = function (ws) {
            wsActionWrapper(ws, { action: "want", data: [] })
          }
          exports.wsStopData = wsStopData
          var wsTrackAddress = function (ws, address) {
            wsActionWrapper(ws, { "track-address": address })
          }
          exports.wsTrackAddress = wsTrackAddress
          var wsStopTrackingAddress = function (ws) {
            wsActionWrapper(ws, { "track-address": "stop" })
          }
          exports.wsStopTrackingAddress = wsStopTrackingAddress
          var wsTrackAddresses = function (ws, addresses) {
            wsActionWrapper(ws, { "track-addresses": addresses })
          }
          exports.wsTrackAddresses = wsTrackAddresses
          var wsStopTrackingAddresses = function (ws) {
            wsActionWrapper(ws, { "track-addresses": [] })
          }
          exports.wsStopTrackingAddresses = wsStopTrackingAddresses
          var wsTrackTransaction = function (ws, txid) {
            wsActionWrapper(ws, { "track-tx": txid })
          }
          exports.wsTrackTransaction = wsTrackTransaction
          var wsStopTrackingTransaction = function (ws) {
            wsActionWrapper(ws, { "track-tx": "stop" })
          }
          exports.wsStopTrackingTransaction = wsStopTrackingTransaction
          var wsTrackRbfSummary = function (ws) {
            wsActionWrapper(ws, { "track-rbf-summary": true })
          }
          exports.wsTrackRbfSummary = wsTrackRbfSummary
          var wsStopTrackingRbfSummary = function (ws) {
            wsActionWrapper(ws, { "track-rbf-summary": false })
          }
          exports.wsStopTrackingRbfSummary = wsStopTrackingRbfSummary
          var wsTrackRbf = function (ws, fullRbf) {
            if (!fullRbf) {
              wsActionWrapper(ws, { "track-rbf": "all" })
            } else {
              wsActionWrapper(ws, { "track-rbf": "fullRbf" })
            }
          }
          exports.wsTrackRbf = wsTrackRbf
          var wsStopTrackingRbf = function (ws) {
            wsActionWrapper(ws, { "track-rbf": "stop" })
          }
          exports.wsStopTrackingRbf = wsStopTrackingRbf
          var wsTrackMempoolBlock = function (ws, index) {
            wsActionWrapper(ws, { "track-mempool-block": index })
          }
          exports.wsTrackMempoolBlock = wsTrackMempoolBlock
          var wsStopTrackingMempoolBlock = function (ws) {
            wsActionWrapper(ws, { "track-mempool-block": -1 })
          }
          exports.wsStopTrackingMempoolBlock = wsStopTrackingMempoolBlock
        },
        {},
      ],
      19: [
        function (require, module, exports) {
          "use strict"
          var __importDefault =
            (this && this.__importDefault) ||
            function (mod) {
              return mod && mod.__esModule ? mod : { default: mod }
            }
          Object.defineProperty(exports, "__esModule", { value: true })
          exports.wsStopTrackingMempoolBlock =
            exports.wsTrackMempoolBlock =
            exports.wsStopTrackingRbf =
            exports.wsTrackRbf =
            exports.wsStopTrackingRbfSummary =
            exports.wsTrackRbfSummary =
            exports.wsStopTrackingTransaction =
            exports.wsTrackTransaction =
            exports.wsStopTrackingAddresses =
            exports.wsTrackAddresses =
            exports.wsStopTrackingAddress =
            exports.wsTrackAddress =
            exports.wsStopData =
            exports.wsWantData =
            exports.wsInit =
              void 0
          var ws_1 = __importDefault(require("ws"))
          var TIMEOUT_DURATION = 5000
          var MAX_RETRY_COUNT = 3
          var wsActionWrapper = function (ws, action) {
            var retryCount = 0
            var sendData = function () {
              if (ws.readyState === ws_1.default.OPEN) {
                ws.send(JSON.stringify(action))
              } else if (retryCount < MAX_RETRY_COUNT) {
                retryCount++
                console.log("WebSocket connection is not open. Retrying...")
                setTimeout(sendData, TIMEOUT_DURATION)
              } else {
                console.log("WebSocket connection is not open. Max retry count reached.")
              }
            }
            sendData()
          }
          var wsInit = function (endpoint) {
            var ws = new ws_1.default(endpoint)
            ws.addEventListener("open", function open() {
              ws.send(JSON.stringify({ action: "init" }))
            })
            return ws
          }
          exports.wsInit = wsInit
          var wsWantData = function (ws, options) {
            wsActionWrapper(ws, { action: "want", data: options })
          }
          exports.wsWantData = wsWantData
          var wsStopData = function (ws) {
            wsActionWrapper(ws, { action: "want", data: [] })
          }
          exports.wsStopData = wsStopData
          var wsTrackAddress = function (ws, address) {
            wsActionWrapper(ws, { "track-address": address })
          }
          exports.wsTrackAddress = wsTrackAddress
          var wsStopTrackingAddress = function (ws) {
            wsActionWrapper(ws, { "track-address": "stop" })
          }
          exports.wsStopTrackingAddress = wsStopTrackingAddress
          var wsTrackAddresses = function (ws, addresses) {
            wsActionWrapper(ws, { "track-addresses": addresses })
          }
          exports.wsTrackAddresses = wsTrackAddresses
          var wsStopTrackingAddresses = function (ws) {
            wsActionWrapper(ws, { "track-addresses": [] })
          }
          exports.wsStopTrackingAddresses = wsStopTrackingAddresses
          var wsTrackTransaction = function (ws, txid) {
            wsActionWrapper(ws, { "track-tx": txid })
          }
          exports.wsTrackTransaction = wsTrackTransaction
          var wsStopTrackingTransaction = function (ws) {
            wsActionWrapper(ws, { "track-tx": "stop" })
          }
          exports.wsStopTrackingTransaction = wsStopTrackingTransaction
          var wsTrackRbfSummary = function (ws) {
            wsActionWrapper(ws, { "track-rbf-summary": true })
          }
          exports.wsTrackRbfSummary = wsTrackRbfSummary
          var wsStopTrackingRbfSummary = function (ws) {
            wsActionWrapper(ws, { "track-rbf-summary": false })
          }
          exports.wsStopTrackingRbfSummary = wsStopTrackingRbfSummary
          var wsTrackRbf = function (ws, fullRbf) {
            if (!fullRbf) {
              wsActionWrapper(ws, { "track-rbf": "all" })
            } else {
              wsActionWrapper(ws, { "track-rbf": "fullRbf" })
            }
          }
          exports.wsTrackRbf = wsTrackRbf
          var wsStopTrackingRbf = function (ws) {
            wsActionWrapper(ws, { "track-rbf": "stop" })
          }
          exports.wsStopTrackingRbf = wsStopTrackingRbf
          var wsTrackMempoolBlock = function (ws, index) {
            wsActionWrapper(ws, { "track-mempool-block": index })
          }
          exports.wsTrackMempoolBlock = wsTrackMempoolBlock
          var wsStopTrackingMempoolBlock = function (ws) {
            wsActionWrapper(ws, { "track-mempool-block": -1 })
          }
          exports.wsStopTrackingMempoolBlock = wsStopTrackingMempoolBlock
        },
        { ws: 75 },
      ],
      20: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.all =
            exports.VERSION =
            exports.HttpStatusCode =
            exports.CanceledError =
            exports.CancelToken =
            exports.Cancel =
            exports.AxiosHeaders =
            exports.AxiosError =
            exports.Axios =
              void 0
          Object.defineProperty(exports, "default", {
            enumerable: true,
            get: function () {
              return _axios.default
            },
          })
          exports.toFormData =
            exports.spread =
            exports.mergeConfig =
            exports.isCancel =
            exports.isAxiosError =
            exports.getAdapter =
            exports.formToJSON =
              void 0
          var _axios = _interopRequireDefault(require("./lib/axios.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          // This module is intended to unwrap Axios default export as named.
          // Keep top-level export same with static properties
          // so that it can keep same with es module or cjs
          const {
            Axios,
            AxiosError,
            CanceledError,
            isCancel,
            CancelToken,
            VERSION,
            all,
            Cancel,
            isAxiosError,
            spread,
            toFormData,
            AxiosHeaders,
            HttpStatusCode,
            formToJSON,
            getAdapter,
            mergeConfig,
          } = _axios.default
          exports.mergeConfig = mergeConfig
          exports.getAdapter = getAdapter
          exports.formToJSON = formToJSON
          exports.HttpStatusCode = HttpStatusCode
          exports.AxiosHeaders = AxiosHeaders
          exports.toFormData = toFormData
          exports.spread = spread
          exports.isAxiosError = isAxiosError
          exports.Cancel = Cancel
          exports.all = all
          exports.VERSION = VERSION
          exports.CancelToken = CancelToken
          exports.isCancel = isCancel
          exports.CanceledError = CanceledError
          exports.AxiosError = AxiosError
          exports.Axios = Axios
        },
        { "./lib/axios.js": 24 },
      ],
      21: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _http = _interopRequireDefault(require("./http.js"))
          var _xhr = _interopRequireDefault(require("./xhr.js"))
          var fetchAdapter = _interopRequireWildcard(require("./fetch.js"))
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          function _interopRequireWildcard(e, t) {
            if ("function" == typeof WeakMap)
              var r = new WeakMap(),
                n = new WeakMap()
            return (_interopRequireWildcard = function (e, t) {
              if (!t && e && e.__esModule) return e
              var o,
                i,
                f = { __proto__: null, default: e }
              if (null === e || ("object" != typeof e && "function" != typeof e)) return f
              if ((o = t ? n : r)) {
                if (o.has(e)) return o.get(e)
                o.set(e, f)
              }
              for (const t in e)
                "default" !== t &&
                  {}.hasOwnProperty.call(e, t) &&
                  ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e, t)) && (i.get || i.set)
                    ? o(f, t, i)
                    : (f[t] = e[t]))
              return f
            })(e, t)
          }
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Known adapters mapping.
           * Provides environment-specific adapters for Axios:
           * - `http` for Node.js
           * - `xhr` for browsers
           * - `fetch` for fetch API-based requests
           *
           * @type {Object<string, Function|Object>}
           */
          const knownAdapters = {
            http: _http.default,
            xhr: _xhr.default,
            fetch: {
              get: fetchAdapter.getFetch,
            },
          }

          // Assign adapter names for easier debugging and identification
          _utils.default.forEach(knownAdapters, (fn, value) => {
            if (fn) {
              try {
                Object.defineProperty(fn, "name", {
                  value,
                })
              } catch (e) {
                // eslint-disable-next-line no-empty
              }
              Object.defineProperty(fn, "adapterName", {
                value,
              })
            }
          })

          /**
           * Render a rejection reason string for unknown or unsupported adapters
           *
           * @param {string} reason
           * @returns {string}
           */
          const renderReason = (reason) => `- ${reason}`

          /**
           * Check if the adapter is resolved (function, null, or false)
           *
           * @param {Function|null|false} adapter
           * @returns {boolean}
           */
          const isResolvedHandle = (adapter) =>
            _utils.default.isFunction(adapter) || adapter === null || adapter === false

          /**
           * Get the first suitable adapter from the provided list.
           * Tries each adapter in order until a supported one is found.
           * Throws an AxiosError if no adapter is suitable.
           *
           * @param {Array<string|Function>|string|Function} adapters - Adapter(s) by name or function.
           * @param {Object} config - Axios request configuration
           * @throws {AxiosError} If no suitable adapter is available
           * @returns {Function} The resolved adapter function
           */
          function getAdapter(adapters, config) {
            adapters = _utils.default.isArray(adapters) ? adapters : [adapters]
            const { length } = adapters
            let nameOrAdapter
            let adapter
            const rejectedReasons = {}
            for (let i = 0; i < length; i++) {
              nameOrAdapter = adapters[i]
              let id
              adapter = nameOrAdapter
              if (!isResolvedHandle(nameOrAdapter)) {
                adapter = knownAdapters[(id = String(nameOrAdapter)).toLowerCase()]
                if (adapter === undefined) {
                  throw new _AxiosError.default(`Unknown adapter '${id}'`)
                }
              }
              if (adapter && (_utils.default.isFunction(adapter) || (adapter = adapter.get(config)))) {
                break
              }
              rejectedReasons[id || "#" + i] = adapter
            }
            if (!adapter) {
              const reasons = Object.entries(rejectedReasons).map(
                ([id, state]) =>
                  `adapter ${id} ` +
                  (state === false ? "is not supported by the environment" : "is not available in the build"),
              )
              let s = length
                ? reasons.length > 1
                  ? "since :\n" + reasons.map(renderReason).join("\n")
                  : " " + renderReason(reasons[0])
                : "as no adapter specified"
              throw new _AxiosError.default(
                `There is no suitable adapter to dispatch the request ` + s,
                "ERR_NOT_SUPPORT",
              )
            }
            return adapter
          }

          /**
           * Exports Axios adapters and utility to resolve an adapter
           */
          var _default = (exports.default = {
            /**
             * Resolve an adapter from a list of adapter names or functions.
             * @type {Function}
             */
            getAdapter,
            /**
             * Exposes all known adapters
             * @type {Object<string, Function|Object>}
             */
            adapters: knownAdapters,
          })
        },
        { "../core/AxiosError.js": 29, "../utils.js": 69, "./fetch.js": 22, "./http.js": 51, "./xhr.js": 23 },
      ],
      22: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.getFetch = exports.default = void 0
          var _index = _interopRequireDefault(require("../platform/index.js"))
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          var _composeSignals = _interopRequireDefault(require("../helpers/composeSignals.js"))
          var _trackStream = require("../helpers/trackStream.js")
          var _AxiosHeaders = _interopRequireDefault(require("../core/AxiosHeaders.js"))
          var _progressEventReducer = require("../helpers/progressEventReducer.js")
          var _resolveConfig = _interopRequireDefault(require("../helpers/resolveConfig.js"))
          var _settle = _interopRequireDefault(require("../core/settle.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const DEFAULT_CHUNK_SIZE = 64 * 1024
          const { isFunction } = _utils.default
          const globalFetchAPI = (({ Request, Response }) => ({
            Request,
            Response,
          }))(_utils.default.global)
          const { ReadableStream, TextEncoder } = _utils.default.global
          const test = (fn, ...args) => {
            try {
              return !!fn(...args)
            } catch (e) {
              return false
            }
          }
          const factory = (env) => {
            env = _utils.default.merge.call(
              {
                skipUndefined: true,
              },
              globalFetchAPI,
              env,
            )
            const { fetch: envFetch, Request, Response } = env
            const isFetchSupported = envFetch ? isFunction(envFetch) : typeof fetch === "function"
            const isRequestSupported = isFunction(Request)
            const isResponseSupported = isFunction(Response)
            if (!isFetchSupported) {
              return false
            }
            const isReadableStreamSupported = isFetchSupported && isFunction(ReadableStream)
            const encodeText =
              isFetchSupported &&
              (typeof TextEncoder === "function"
                ? (
                    (encoder) => (str) =>
                      encoder.encode(str)
                  )(new TextEncoder())
                : async (str) => new Uint8Array(await new Request(str).arrayBuffer()))
            const supportsRequestStream =
              isRequestSupported &&
              isReadableStreamSupported &&
              test(() => {
                let duplexAccessed = false
                const hasContentType = new Request(_index.default.origin, {
                  body: new ReadableStream(),
                  method: "POST",
                  get duplex() {
                    duplexAccessed = true
                    return "half"
                  },
                }).headers.has("Content-Type")
                return duplexAccessed && !hasContentType
              })
            const supportsResponseStream =
              isResponseSupported &&
              isReadableStreamSupported &&
              test(() => _utils.default.isReadableStream(new Response("").body))
            const resolvers = {
              stream: supportsResponseStream && ((res) => res.body),
            }
            isFetchSupported &&
              (() => {
                ;["text", "arrayBuffer", "blob", "formData", "stream"].forEach((type) => {
                  !resolvers[type] &&
                    (resolvers[type] = (res, config) => {
                      let method = res && res[type]
                      if (method) {
                        return method.call(res)
                      }
                      throw new _AxiosError.default(
                        `Response type '${type}' is not supported`,
                        _AxiosError.default.ERR_NOT_SUPPORT,
                        config,
                      )
                    })
                })
              })()
            const getBodyLength = async (body) => {
              if (body == null) {
                return 0
              }
              if (_utils.default.isBlob(body)) {
                return body.size
              }
              if (_utils.default.isSpecCompliantForm(body)) {
                const _request = new Request(_index.default.origin, {
                  method: "POST",
                  body,
                })
                return (await _request.arrayBuffer()).byteLength
              }
              if (_utils.default.isArrayBufferView(body) || _utils.default.isArrayBuffer(body)) {
                return body.byteLength
              }
              if (_utils.default.isURLSearchParams(body)) {
                body = body + ""
              }
              if (_utils.default.isString(body)) {
                return (await encodeText(body)).byteLength
              }
            }
            const resolveBodyLength = async (headers, body) => {
              const length = _utils.default.toFiniteNumber(headers.getContentLength())
              return length == null ? getBodyLength(body) : length
            }
            return async (config) => {
              let {
                url,
                method,
                data,
                signal,
                cancelToken,
                timeout,
                onDownloadProgress,
                onUploadProgress,
                responseType,
                headers,
                withCredentials = "same-origin",
                fetchOptions,
              } = (0, _resolveConfig.default)(config)
              let _fetch = envFetch || fetch
              responseType = responseType ? (responseType + "").toLowerCase() : "text"
              let composedSignal = (0, _composeSignals.default)(
                [signal, cancelToken && cancelToken.toAbortSignal()],
                timeout,
              )
              let request = null
              const unsubscribe =
                composedSignal &&
                composedSignal.unsubscribe &&
                (() => {
                  composedSignal.unsubscribe()
                })
              let requestContentLength
              try {
                if (
                  onUploadProgress &&
                  supportsRequestStream &&
                  method !== "get" &&
                  method !== "head" &&
                  (requestContentLength = await resolveBodyLength(headers, data)) !== 0
                ) {
                  let _request = new Request(url, {
                    method: "POST",
                    body: data,
                    duplex: "half",
                  })
                  let contentTypeHeader
                  if (_utils.default.isFormData(data) && (contentTypeHeader = _request.headers.get("content-type"))) {
                    headers.setContentType(contentTypeHeader)
                  }
                  if (_request.body) {
                    const [onProgress, flush] = (0, _progressEventReducer.progressEventDecorator)(
                      requestContentLength,
                      (0, _progressEventReducer.progressEventReducer)(
                        (0, _progressEventReducer.asyncDecorator)(onUploadProgress),
                      ),
                    )
                    data = (0, _trackStream.trackStream)(_request.body, DEFAULT_CHUNK_SIZE, onProgress, flush)
                  }
                }
                if (!_utils.default.isString(withCredentials)) {
                  withCredentials = withCredentials ? "include" : "omit"
                }

                // Cloudflare Workers throws when credentials are defined
                // see https://github.com/cloudflare/workerd/issues/902
                const isCredentialsSupported = isRequestSupported && "credentials" in Request.prototype
                const resolvedOptions = {
                  ...fetchOptions,
                  signal: composedSignal,
                  method: method.toUpperCase(),
                  headers: headers.normalize().toJSON(),
                  body: data,
                  duplex: "half",
                  credentials: isCredentialsSupported ? withCredentials : undefined,
                }
                request = isRequestSupported && new Request(url, resolvedOptions)
                let response = await (isRequestSupported ? _fetch(request, fetchOptions) : _fetch(url, resolvedOptions))
                const isStreamResponse =
                  supportsResponseStream && (responseType === "stream" || responseType === "response")
                if (supportsResponseStream && (onDownloadProgress || (isStreamResponse && unsubscribe))) {
                  const options = {}
                  ;["status", "statusText", "headers"].forEach((prop) => {
                    options[prop] = response[prop]
                  })
                  const responseContentLength = _utils.default.toFiniteNumber(response.headers.get("content-length"))
                  const [onProgress, flush] =
                    (onDownloadProgress &&
                      (0, _progressEventReducer.progressEventDecorator)(
                        responseContentLength,
                        (0, _progressEventReducer.progressEventReducer)(
                          (0, _progressEventReducer.asyncDecorator)(onDownloadProgress),
                          true,
                        ),
                      )) ||
                    []
                  response = new Response(
                    (0, _trackStream.trackStream)(response.body, DEFAULT_CHUNK_SIZE, onProgress, () => {
                      flush && flush()
                      unsubscribe && unsubscribe()
                    }),
                    options,
                  )
                }
                responseType = responseType || "text"
                let responseData = await resolvers[_utils.default.findKey(resolvers, responseType) || "text"](
                  response,
                  config,
                )
                !isStreamResponse && unsubscribe && unsubscribe()
                return await new Promise((resolve, reject) => {
                  ;(0, _settle.default)(resolve, reject, {
                    data: responseData,
                    headers: _AxiosHeaders.default.from(response.headers),
                    status: response.status,
                    statusText: response.statusText,
                    config,
                    request,
                  })
                })
              } catch (err) {
                unsubscribe && unsubscribe()
                if (err && err.name === "TypeError" && /Load failed|fetch/i.test(err.message)) {
                  throw Object.assign(
                    new _AxiosError.default("Network Error", _AxiosError.default.ERR_NETWORK, config, request),
                    {
                      cause: err.cause || err,
                    },
                  )
                }
                throw _AxiosError.default.from(err, err && err.code, config, request)
              }
            }
          }
          const seedCache = new Map()
          const getFetch = (config) => {
            let env = (config && config.env) || {}
            const { fetch, Request, Response } = env
            const seeds = [Request, Response, fetch]
            let len = seeds.length,
              i = len,
              seed,
              target,
              map = seedCache
            while (i--) {
              seed = seeds[i]
              target = map.get(seed)
              target === undefined && map.set(seed, (target = i ? new Map() : factory(env)))
              map = target
            }
            return target
          }
          exports.getFetch = getFetch
          const adapter = getFetch()
          var _default = (exports.default = adapter)
        },
        {
          "../core/AxiosError.js": 29,
          "../core/AxiosHeaders.js": 30,
          "../core/settle.js": 35,
          "../helpers/composeSignals.js": 45,
          "../helpers/progressEventReducer.js": 54,
          "../helpers/resolveConfig.js": 55,
          "../helpers/trackStream.js": 61,
          "../platform/index.js": 68,
          "../utils.js": 69,
        },
      ],
      23: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./../utils.js"))
          var _settle = _interopRequireDefault(require("./../core/settle.js"))
          var _transitional = _interopRequireDefault(require("../defaults/transitional.js"))
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          var _CanceledError = _interopRequireDefault(require("../cancel/CanceledError.js"))
          var _parseProtocol = _interopRequireDefault(require("../helpers/parseProtocol.js"))
          var _index = _interopRequireDefault(require("../platform/index.js"))
          var _AxiosHeaders = _interopRequireDefault(require("../core/AxiosHeaders.js"))
          var _progressEventReducer = require("../helpers/progressEventReducer.js")
          var _resolveConfig = _interopRequireDefault(require("../helpers/resolveConfig.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const isXHRAdapterSupported = typeof XMLHttpRequest !== "undefined"
          var _default = (exports.default =
            isXHRAdapterSupported &&
            function (config) {
              return new Promise(function dispatchXhrRequest(resolve, reject) {
                const _config = (0, _resolveConfig.default)(config)
                let requestData = _config.data
                const requestHeaders = _AxiosHeaders.default.from(_config.headers).normalize()
                let { responseType, onUploadProgress, onDownloadProgress } = _config
                let onCanceled
                let uploadThrottled, downloadThrottled
                let flushUpload, flushDownload
                function done() {
                  flushUpload && flushUpload() // flush events
                  flushDownload && flushDownload() // flush events

                  _config.cancelToken && _config.cancelToken.unsubscribe(onCanceled)
                  _config.signal && _config.signal.removeEventListener("abort", onCanceled)
                }
                let request = new XMLHttpRequest()
                request.open(_config.method.toUpperCase(), _config.url, true)

                // Set the request timeout in MS
                request.timeout = _config.timeout
                function onloadend() {
                  if (!request) {
                    return
                  }
                  // Prepare the response
                  const responseHeaders = _AxiosHeaders.default.from(
                    "getAllResponseHeaders" in request && request.getAllResponseHeaders(),
                  )
                  const responseData =
                    !responseType || responseType === "text" || responseType === "json"
                      ? request.responseText
                      : request.response
                  const response = {
                    data: responseData,
                    status: request.status,
                    statusText: request.statusText,
                    headers: responseHeaders,
                    config,
                    request,
                  }
                  ;(0, _settle.default)(
                    function _resolve(value) {
                      resolve(value)
                      done()
                    },
                    function _reject(err) {
                      reject(err)
                      done()
                    },
                    response,
                  )

                  // Clean up request
                  request = null
                }
                if ("onloadend" in request) {
                  // Use onloadend if available
                  request.onloadend = onloadend
                } else {
                  // Listen for ready state to emulate onloadend
                  request.onreadystatechange = function handleLoad() {
                    if (!request || request.readyState !== 4) {
                      return
                    }

                    // The request errored out and we didn't get a response, this will be
                    // handled by onerror instead
                    // With one exception: request that using file: protocol, most browsers
                    // will return status as 0 even though it's a successful request
                    if (request.status === 0 && !(request.responseURL && request.responseURL.indexOf("file:") === 0)) {
                      return
                    }
                    // readystate handler is calling before onerror or ontimeout handlers,
                    // so we should call onloadend on the next 'tick'
                    setTimeout(onloadend)
                  }
                }

                // Handle browser request cancellation (as opposed to a manual cancellation)
                request.onabort = function handleAbort() {
                  if (!request) {
                    return
                  }
                  reject(new _AxiosError.default("Request aborted", _AxiosError.default.ECONNABORTED, config, request))

                  // Clean up request
                  request = null
                }

                // Handle low level network errors
                request.onerror = function handleError(event) {
                  // Browsers deliver a ProgressEvent in XHR onerror
                  // (message may be empty; when present, surface it)
                  // See https://developer.mozilla.org/docs/Web/API/XMLHttpRequest/error_event
                  const msg = event && event.message ? event.message : "Network Error"
                  const err = new _AxiosError.default(msg, _AxiosError.default.ERR_NETWORK, config, request)
                  // attach the underlying event for consumers who want details
                  err.event = event || null
                  reject(err)
                  request = null
                }

                // Handle timeout
                request.ontimeout = function handleTimeout() {
                  let timeoutErrorMessage = _config.timeout
                    ? "timeout of " + _config.timeout + "ms exceeded"
                    : "timeout exceeded"
                  const transitional = _config.transitional || _transitional.default
                  if (_config.timeoutErrorMessage) {
                    timeoutErrorMessage = _config.timeoutErrorMessage
                  }
                  reject(
                    new _AxiosError.default(
                      timeoutErrorMessage,
                      transitional.clarifyTimeoutError
                        ? _AxiosError.default.ETIMEDOUT
                        : _AxiosError.default.ECONNABORTED,
                      config,
                      request,
                    ),
                  )

                  // Clean up request
                  request = null
                }

                // Remove Content-Type if data is undefined
                requestData === undefined && requestHeaders.setContentType(null)

                // Add headers to the request
                if ("setRequestHeader" in request) {
                  _utils.default.forEach(requestHeaders.toJSON(), function setRequestHeader(val, key) {
                    request.setRequestHeader(key, val)
                  })
                }

                // Add withCredentials to request if needed
                if (!_utils.default.isUndefined(_config.withCredentials)) {
                  request.withCredentials = !!_config.withCredentials
                }

                // Add responseType to request if needed
                if (responseType && responseType !== "json") {
                  request.responseType = _config.responseType
                }

                // Handle progress if needed
                if (onDownloadProgress) {
                  ;[downloadThrottled, flushDownload] = (0, _progressEventReducer.progressEventReducer)(
                    onDownloadProgress,
                    true,
                  )
                  request.addEventListener("progress", downloadThrottled)
                }

                // Not all browsers support upload events
                if (onUploadProgress && request.upload) {
                  ;[uploadThrottled, flushUpload] = (0, _progressEventReducer.progressEventReducer)(onUploadProgress)
                  request.upload.addEventListener("progress", uploadThrottled)
                  request.upload.addEventListener("loadend", flushUpload)
                }
                if (_config.cancelToken || _config.signal) {
                  // Handle cancellation
                  // eslint-disable-next-line func-names
                  onCanceled = (cancel) => {
                    if (!request) {
                      return
                    }
                    reject(!cancel || cancel.type ? new _CanceledError.default(null, config, request) : cancel)
                    request.abort()
                    request = null
                  }
                  _config.cancelToken && _config.cancelToken.subscribe(onCanceled)
                  if (_config.signal) {
                    _config.signal.aborted ? onCanceled() : _config.signal.addEventListener("abort", onCanceled)
                  }
                }
                const protocol = (0, _parseProtocol.default)(_config.url)
                if (protocol && _index.default.protocols.indexOf(protocol) === -1) {
                  reject(
                    new _AxiosError.default(
                      "Unsupported protocol " + protocol + ":",
                      _AxiosError.default.ERR_BAD_REQUEST,
                      config,
                    ),
                  )
                  return
                }

                // Send the request
                request.send(requestData || null)
              })
            })
        },
        {
          "../cancel/CanceledError.js": 26,
          "../core/AxiosError.js": 29,
          "../core/AxiosHeaders.js": 30,
          "../defaults/transitional.js": 38,
          "../helpers/parseProtocol.js": 53,
          "../helpers/progressEventReducer.js": 54,
          "../helpers/resolveConfig.js": 55,
          "../platform/index.js": 68,
          "./../core/settle.js": 35,
          "./../utils.js": 69,
        },
      ],
      24: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./utils.js"))
          var _bind = _interopRequireDefault(require("./helpers/bind.js"))
          var _Axios = _interopRequireDefault(require("./core/Axios.js"))
          var _mergeConfig = _interopRequireDefault(require("./core/mergeConfig.js"))
          var _index = _interopRequireDefault(require("./defaults/index.js"))
          var _formDataToJSON = _interopRequireDefault(require("./helpers/formDataToJSON.js"))
          var _CanceledError = _interopRequireDefault(require("./cancel/CanceledError.js"))
          var _CancelToken = _interopRequireDefault(require("./cancel/CancelToken.js"))
          var _isCancel = _interopRequireDefault(require("./cancel/isCancel.js"))
          var _data = require("./env/data.js")
          var _toFormData = _interopRequireDefault(require("./helpers/toFormData.js"))
          var _AxiosError = _interopRequireDefault(require("./core/AxiosError.js"))
          var _spread = _interopRequireDefault(require("./helpers/spread.js"))
          var _isAxiosError = _interopRequireDefault(require("./helpers/isAxiosError.js"))
          var _AxiosHeaders = _interopRequireDefault(require("./core/AxiosHeaders.js"))
          var _adapters = _interopRequireDefault(require("./adapters/adapters.js"))
          var _HttpStatusCode = _interopRequireDefault(require("./helpers/HttpStatusCode.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Create an instance of Axios
           *
           * @param {Object} defaultConfig The default config for the instance
           *
           * @returns {Axios} A new instance of Axios
           */
          function createInstance(defaultConfig) {
            const context = new _Axios.default(defaultConfig)
            const instance = (0, _bind.default)(_Axios.default.prototype.request, context)

            // Copy axios.prototype to instance
            _utils.default.extend(instance, _Axios.default.prototype, context, {
              allOwnKeys: true,
            })

            // Copy context to instance
            _utils.default.extend(instance, context, null, {
              allOwnKeys: true,
            })

            // Factory for creating new instances
            instance.create = function create(instanceConfig) {
              return createInstance((0, _mergeConfig.default)(defaultConfig, instanceConfig))
            }
            return instance
          }

          // Create the default instance to be exported
          const axios = createInstance(_index.default)

          // Expose Axios class to allow class inheritance
          axios.Axios = _Axios.default

          // Expose Cancel & CancelToken
          axios.CanceledError = _CanceledError.default
          axios.CancelToken = _CancelToken.default
          axios.isCancel = _isCancel.default
          axios.VERSION = _data.VERSION
          axios.toFormData = _toFormData.default

          // Expose AxiosError class
          axios.AxiosError = _AxiosError.default

          // alias for CanceledError for backward compatibility
          axios.Cancel = axios.CanceledError

          // Expose all/spread
          axios.all = function all(promises) {
            return Promise.all(promises)
          }
          axios.spread = _spread.default

          // Expose isAxiosError
          axios.isAxiosError = _isAxiosError.default

          // Expose mergeConfig
          axios.mergeConfig = _mergeConfig.default
          axios.AxiosHeaders = _AxiosHeaders.default
          axios.formToJSON = (thing) =>
            (0, _formDataToJSON.default)(_utils.default.isHTMLForm(thing) ? new FormData(thing) : thing)
          axios.getAdapter = _adapters.default.getAdapter
          axios.HttpStatusCode = _HttpStatusCode.default
          axios.default = axios

          // this module should only have a default export
          var _default = (exports.default = axios)
        },
        {
          "./adapters/adapters.js": 21,
          "./cancel/CancelToken.js": 25,
          "./cancel/CanceledError.js": 26,
          "./cancel/isCancel.js": 27,
          "./core/Axios.js": 28,
          "./core/AxiosError.js": 29,
          "./core/AxiosHeaders.js": 30,
          "./core/mergeConfig.js": 34,
          "./defaults/index.js": 37,
          "./env/data.js": 39,
          "./helpers/HttpStatusCode.js": 41,
          "./helpers/bind.js": 42,
          "./helpers/formDataToJSON.js": 47,
          "./helpers/isAxiosError.js": 49,
          "./helpers/spread.js": 57,
          "./helpers/toFormData.js": 59,
          "./utils.js": 69,
        },
      ],
      25: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _CanceledError = _interopRequireDefault(require("./CanceledError.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * A `CancelToken` is an object that can be used to request cancellation of an operation.
           *
           * @param {Function} executor The executor function.
           *
           * @returns {CancelToken}
           */
          class CancelToken {
            constructor(executor) {
              if (typeof executor !== "function") {
                throw new TypeError("executor must be a function.")
              }
              let resolvePromise
              this.promise = new Promise(function promiseExecutor(resolve) {
                resolvePromise = resolve
              })
              const token = this

              // eslint-disable-next-line func-names
              this.promise.then((cancel) => {
                if (!token._listeners) return
                let i = token._listeners.length
                while (i-- > 0) {
                  token._listeners[i](cancel)
                }
                token._listeners = null
              })

              // eslint-disable-next-line func-names
              this.promise.then = (onfulfilled) => {
                let _resolve
                // eslint-disable-next-line func-names
                const promise = new Promise((resolve) => {
                  token.subscribe(resolve)
                  _resolve = resolve
                }).then(onfulfilled)
                promise.cancel = function reject() {
                  token.unsubscribe(_resolve)
                }
                return promise
              }
              executor(function cancel(message, config, request) {
                if (token.reason) {
                  // Cancellation has already been requested
                  return
                }
                token.reason = new _CanceledError.default(message, config, request)
                resolvePromise(token.reason)
              })
            }

            /**
             * Throws a `CanceledError` if cancellation has been requested.
             */
            throwIfRequested() {
              if (this.reason) {
                throw this.reason
              }
            }

            /**
             * Subscribe to the cancel signal
             */

            subscribe(listener) {
              if (this.reason) {
                listener(this.reason)
                return
              }
              if (this._listeners) {
                this._listeners.push(listener)
              } else {
                this._listeners = [listener]
              }
            }

            /**
             * Unsubscribe from the cancel signal
             */

            unsubscribe(listener) {
              if (!this._listeners) {
                return
              }
              const index = this._listeners.indexOf(listener)
              if (index !== -1) {
                this._listeners.splice(index, 1)
              }
            }
            toAbortSignal() {
              const controller = new AbortController()
              const abort = (err) => {
                controller.abort(err)
              }
              this.subscribe(abort)
              controller.signal.unsubscribe = () => this.unsubscribe(abort)
              return controller.signal
            }

            /**
             * Returns an object that contains a new `CancelToken` and a function that, when called,
             * cancels the `CancelToken`.
             */
            static source() {
              let cancel
              const token = new CancelToken(function executor(c) {
                cancel = c
              })
              return {
                token,
                cancel,
              }
            }
          }
          var _default = (exports.default = CancelToken)
        },
        { "./CanceledError.js": 26 },
      ],
      26: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          var _utils = _interopRequireDefault(require("../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * A `CanceledError` is an object that is thrown when an operation is canceled.
           *
           * @param {string=} message The message.
           * @param {Object=} config The config.
           * @param {Object=} request The request.
           *
           * @returns {CanceledError} The created error.
           */
          function CanceledError(message, config, request) {
            // eslint-disable-next-line no-eq-null,eqeqeq
            _AxiosError.default.call(
              this,
              message == null ? "canceled" : message,
              _AxiosError.default.ERR_CANCELED,
              config,
              request,
            )
            this.name = "CanceledError"
          }
          _utils.default.inherits(CanceledError, _AxiosError.default, {
            __CANCEL__: true,
          })
          var _default = (exports.default = CanceledError)
        },
        { "../core/AxiosError.js": 29, "../utils.js": 69 },
      ],
      27: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = isCancel
          function isCancel(value) {
            return !!(value && value.__CANCEL__)
          }
        },
        {},
      ],
      28: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./../utils.js"))
          var _buildURL = _interopRequireDefault(require("../helpers/buildURL.js"))
          var _InterceptorManager = _interopRequireDefault(require("./InterceptorManager.js"))
          var _dispatchRequest = _interopRequireDefault(require("./dispatchRequest.js"))
          var _mergeConfig = _interopRequireDefault(require("./mergeConfig.js"))
          var _buildFullPath = _interopRequireDefault(require("./buildFullPath.js"))
          var _validator = _interopRequireDefault(require("../helpers/validator.js"))
          var _AxiosHeaders = _interopRequireDefault(require("./AxiosHeaders.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const validators = _validator.default.validators

          /**
           * Create a new instance of Axios
           *
           * @param {Object} instanceConfig The default config for the instance
           *
           * @return {Axios} A new instance of Axios
           */
          class Axios {
            constructor(instanceConfig) {
              this.defaults = instanceConfig || {}
              this.interceptors = {
                request: new _InterceptorManager.default(),
                response: new _InterceptorManager.default(),
              }
            }

            /**
             * Dispatch a request
             *
             * @param {String|Object} configOrUrl The config specific for this request (merged with this.defaults)
             * @param {?Object} config
             *
             * @returns {Promise} The Promise to be fulfilled
             */
            async request(configOrUrl, config) {
              try {
                return await this._request(configOrUrl, config)
              } catch (err) {
                if (err instanceof Error) {
                  let dummy = {}
                  Error.captureStackTrace ? Error.captureStackTrace(dummy) : (dummy = new Error())

                  // slice off the Error: ... line
                  const stack = dummy.stack ? dummy.stack.replace(/^.+\n/, "") : ""
                  try {
                    if (!err.stack) {
                      err.stack = stack
                      // match without the 2 top stack lines
                    } else if (stack && !String(err.stack).endsWith(stack.replace(/^.+\n.+\n/, ""))) {
                      err.stack += "\n" + stack
                    }
                  } catch (e) {
                    // ignore the case where "stack" is an un-writable property
                  }
                }
                throw err
              }
            }
            _request(configOrUrl, config) {
              /*eslint no-param-reassign:0*/
              // Allow for axios('example/url'[, config]) a la fetch API
              if (typeof configOrUrl === "string") {
                config = config || {}
                config.url = configOrUrl
              } else {
                config = configOrUrl || {}
              }
              config = (0, _mergeConfig.default)(this.defaults, config)
              const { transitional, paramsSerializer, headers } = config
              if (transitional !== undefined) {
                _validator.default.assertOptions(
                  transitional,
                  {
                    silentJSONParsing: validators.transitional(validators.boolean),
                    forcedJSONParsing: validators.transitional(validators.boolean),
                    clarifyTimeoutError: validators.transitional(validators.boolean),
                  },
                  false,
                )
              }
              if (paramsSerializer != null) {
                if (_utils.default.isFunction(paramsSerializer)) {
                  config.paramsSerializer = {
                    serialize: paramsSerializer,
                  }
                } else {
                  _validator.default.assertOptions(
                    paramsSerializer,
                    {
                      encode: validators.function,
                      serialize: validators.function,
                    },
                    true,
                  )
                }
              }

              // Set config.allowAbsoluteUrls
              if (config.allowAbsoluteUrls !== undefined) {
                // do nothing
              } else if (this.defaults.allowAbsoluteUrls !== undefined) {
                config.allowAbsoluteUrls = this.defaults.allowAbsoluteUrls
              } else {
                config.allowAbsoluteUrls = true
              }
              _validator.default.assertOptions(
                config,
                {
                  baseUrl: validators.spelling("baseURL"),
                  withXsrfToken: validators.spelling("withXSRFToken"),
                },
                true,
              )

              // Set config.method
              config.method = (config.method || this.defaults.method || "get").toLowerCase()

              // Flatten headers
              let contextHeaders = headers && _utils.default.merge(headers.common, headers[config.method])
              headers &&
                _utils.default.forEach(["delete", "get", "head", "post", "put", "patch", "common"], (method) => {
                  delete headers[method]
                })
              config.headers = _AxiosHeaders.default.concat(contextHeaders, headers)

              // filter out skipped interceptors
              const requestInterceptorChain = []
              let synchronousRequestInterceptors = true
              this.interceptors.request.forEach(function unshiftRequestInterceptors(interceptor) {
                if (typeof interceptor.runWhen === "function" && interceptor.runWhen(config) === false) {
                  return
                }
                synchronousRequestInterceptors = synchronousRequestInterceptors && interceptor.synchronous
                requestInterceptorChain.unshift(interceptor.fulfilled, interceptor.rejected)
              })
              const responseInterceptorChain = []
              this.interceptors.response.forEach(function pushResponseInterceptors(interceptor) {
                responseInterceptorChain.push(interceptor.fulfilled, interceptor.rejected)
              })
              let promise
              let i = 0
              let len
              if (!synchronousRequestInterceptors) {
                const chain = [_dispatchRequest.default.bind(this), undefined]
                chain.unshift(...requestInterceptorChain)
                chain.push(...responseInterceptorChain)
                len = chain.length
                promise = Promise.resolve(config)
                while (i < len) {
                  promise = promise.then(chain[i++], chain[i++])
                }
                return promise
              }
              len = requestInterceptorChain.length
              let newConfig = config
              while (i < len) {
                const onFulfilled = requestInterceptorChain[i++]
                const onRejected = requestInterceptorChain[i++]
                try {
                  newConfig = onFulfilled(newConfig)
                } catch (error) {
                  onRejected.call(this, error)
                  break
                }
              }
              try {
                promise = _dispatchRequest.default.call(this, newConfig)
              } catch (error) {
                return Promise.reject(error)
              }
              i = 0
              len = responseInterceptorChain.length
              while (i < len) {
                promise = promise.then(responseInterceptorChain[i++], responseInterceptorChain[i++])
              }
              return promise
            }
            getUri(config) {
              config = (0, _mergeConfig.default)(this.defaults, config)
              const fullPath = (0, _buildFullPath.default)(config.baseURL, config.url, config.allowAbsoluteUrls)
              return (0, _buildURL.default)(fullPath, config.params, config.paramsSerializer)
            }
          }

          // Provide aliases for supported request methods
          _utils.default.forEach(["delete", "get", "head", "options"], function forEachMethodNoData(method) {
            /*eslint func-names:0*/
            Axios.prototype[method] = function (url, config) {
              return this.request(
                (0, _mergeConfig.default)(config || {}, {
                  method,
                  url,
                  data: (config || {}).data,
                }),
              )
            }
          })
          _utils.default.forEach(["post", "put", "patch"], function forEachMethodWithData(method) {
            /*eslint func-names:0*/

            function generateHTTPMethod(isForm) {
              return function httpMethod(url, data, config) {
                return this.request(
                  (0, _mergeConfig.default)(config || {}, {
                    method,
                    headers: isForm
                      ? {
                          "Content-Type": "multipart/form-data",
                        }
                      : {},
                    url,
                    data,
                  }),
                )
              }
            }
            Axios.prototype[method] = generateHTTPMethod()
            Axios.prototype[method + "Form"] = generateHTTPMethod(true)
          })
          var _default = (exports.default = Axios)
        },
        {
          "../helpers/buildURL.js": 43,
          "../helpers/validator.js": 62,
          "./../utils.js": 69,
          "./AxiosHeaders.js": 30,
          "./InterceptorManager.js": 31,
          "./buildFullPath.js": 32,
          "./dispatchRequest.js": 33,
          "./mergeConfig.js": 34,
        },
      ],
      29: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Create an Error with the specified message, config, error code, request and response.
           *
           * @param {string} message The error message.
           * @param {string} [code] The error code (for example, 'ECONNABORTED').
           * @param {Object} [config] The config.
           * @param {Object} [request] The request.
           * @param {Object} [response] The response.
           *
           * @returns {Error} The created error.
           */
          function AxiosError(message, code, config, request, response) {
            Error.call(this)
            if (Error.captureStackTrace) {
              Error.captureStackTrace(this, this.constructor)
            } else {
              this.stack = new Error().stack
            }
            this.message = message
            this.name = "AxiosError"
            code && (this.code = code)
            config && (this.config = config)
            request && (this.request = request)
            if (response) {
              this.response = response
              this.status = response.status ? response.status : null
            }
          }
          _utils.default.inherits(AxiosError, Error, {
            toJSON: function toJSON() {
              return {
                // Standard
                message: this.message,
                name: this.name,
                // Microsoft
                description: this.description,
                number: this.number,
                // Mozilla
                fileName: this.fileName,
                lineNumber: this.lineNumber,
                columnNumber: this.columnNumber,
                stack: this.stack,
                // Axios
                config: _utils.default.toJSONObject(this.config),
                code: this.code,
                status: this.status,
              }
            },
          })
          const prototype = AxiosError.prototype
          const descriptors = {}
          ;[
            "ERR_BAD_OPTION_VALUE",
            "ERR_BAD_OPTION",
            "ECONNABORTED",
            "ETIMEDOUT",
            "ERR_NETWORK",
            "ERR_FR_TOO_MANY_REDIRECTS",
            "ERR_DEPRECATED",
            "ERR_BAD_RESPONSE",
            "ERR_BAD_REQUEST",
            "ERR_CANCELED",
            "ERR_NOT_SUPPORT",
            "ERR_INVALID_URL",
            // eslint-disable-next-line func-names
          ].forEach((code) => {
            descriptors[code] = {
              value: code,
            }
          })
          Object.defineProperties(AxiosError, descriptors)
          Object.defineProperty(prototype, "isAxiosError", {
            value: true,
          })

          // eslint-disable-next-line func-names
          AxiosError.from = (error, code, config, request, response, customProps) => {
            const axiosError = Object.create(prototype)
            _utils.default.toFlatObject(
              error,
              axiosError,
              function filter(obj) {
                return obj !== Error.prototype
              },
              (prop) => {
                return prop !== "isAxiosError"
              },
            )
            const msg = error && error.message ? error.message : "Error"

            // Prefer explicit code; otherwise copy the low-level error's code (e.g. ECONNREFUSED)
            const errCode = code == null && error ? error.code : code
            AxiosError.call(axiosError, msg, errCode, config, request, response)

            // Chain the original error on the standard field; non-enumerable to avoid JSON noise
            if (error && axiosError.cause == null) {
              Object.defineProperty(axiosError, "cause", {
                value: error,
                configurable: true,
              })
            }
            axiosError.name = (error && error.name) || "Error"
            customProps && Object.assign(axiosError, customProps)
            return axiosError
          }
          var _default = (exports.default = AxiosError)
        },
        { "../utils.js": 69 },
      ],
      30: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _parseHeaders = _interopRequireDefault(require("../helpers/parseHeaders.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const $internals = Symbol("internals")
          function normalizeHeader(header) {
            return header && String(header).trim().toLowerCase()
          }
          function normalizeValue(value) {
            if (value === false || value == null) {
              return value
            }
            return _utils.default.isArray(value) ? value.map(normalizeValue) : String(value)
          }
          function parseTokens(str) {
            const tokens = Object.create(null)
            const tokensRE = /([^\s,;=]+)\s*(?:=\s*([^,;]+))?/g
            let match
            while ((match = tokensRE.exec(str))) {
              tokens[match[1]] = match[2]
            }
            return tokens
          }
          const isValidHeaderName = (str) => /^[-_a-zA-Z0-9^`|~,!#$%&'*+.]+$/.test(str.trim())
          function matchHeaderValue(context, value, header, filter, isHeaderNameFilter) {
            if (_utils.default.isFunction(filter)) {
              return filter.call(this, value, header)
            }
            if (isHeaderNameFilter) {
              value = header
            }
            if (!_utils.default.isString(value)) return
            if (_utils.default.isString(filter)) {
              return value.indexOf(filter) !== -1
            }
            if (_utils.default.isRegExp(filter)) {
              return filter.test(value)
            }
          }
          function formatHeader(header) {
            return header
              .trim()
              .toLowerCase()
              .replace(/([a-z\d])(\w*)/g, (w, char, str) => {
                return char.toUpperCase() + str
              })
          }
          function buildAccessors(obj, header) {
            const accessorName = _utils.default.toCamelCase(" " + header)
            ;["get", "set", "has"].forEach((methodName) => {
              Object.defineProperty(obj, methodName + accessorName, {
                value: function (arg1, arg2, arg3) {
                  return this[methodName].call(this, header, arg1, arg2, arg3)
                },
                configurable: true,
              })
            })
          }
          class AxiosHeaders {
            constructor(headers) {
              headers && this.set(headers)
            }
            set(header, valueOrRewrite, rewrite) {
              const self = this
              function setHeader(_value, _header, _rewrite) {
                const lHeader = normalizeHeader(_header)
                if (!lHeader) {
                  throw new Error("header name must be a non-empty string")
                }
                const key = _utils.default.findKey(self, lHeader)
                if (
                  !key ||
                  self[key] === undefined ||
                  _rewrite === true ||
                  (_rewrite === undefined && self[key] !== false)
                ) {
                  self[key || _header] = normalizeValue(_value)
                }
              }
              const setHeaders = (headers, _rewrite) =>
                _utils.default.forEach(headers, (_value, _header) => setHeader(_value, _header, _rewrite))
              if (_utils.default.isPlainObject(header) || header instanceof this.constructor) {
                setHeaders(header, valueOrRewrite)
              } else if (_utils.default.isString(header) && (header = header.trim()) && !isValidHeaderName(header)) {
                setHeaders((0, _parseHeaders.default)(header), valueOrRewrite)
              } else if (_utils.default.isObject(header) && _utils.default.isIterable(header)) {
                let obj = {},
                  dest,
                  key
                for (const entry of header) {
                  if (!_utils.default.isArray(entry)) {
                    throw TypeError("Object iterator must return a key-value pair")
                  }
                  obj[(key = entry[0])] = (dest = obj[key])
                    ? _utils.default.isArray(dest)
                      ? [...dest, entry[1]]
                      : [dest, entry[1]]
                    : entry[1]
                }
                setHeaders(obj, valueOrRewrite)
              } else {
                header != null && setHeader(valueOrRewrite, header, rewrite)
              }
              return this
            }
            get(header, parser) {
              header = normalizeHeader(header)
              if (header) {
                const key = _utils.default.findKey(this, header)
                if (key) {
                  const value = this[key]
                  if (!parser) {
                    return value
                  }
                  if (parser === true) {
                    return parseTokens(value)
                  }
                  if (_utils.default.isFunction(parser)) {
                    return parser.call(this, value, key)
                  }
                  if (_utils.default.isRegExp(parser)) {
                    return parser.exec(value)
                  }
                  throw new TypeError("parser must be boolean|regexp|function")
                }
              }
            }
            has(header, matcher) {
              header = normalizeHeader(header)
              if (header) {
                const key = _utils.default.findKey(this, header)
                return !!(
                  key &&
                  this[key] !== undefined &&
                  (!matcher || matchHeaderValue(this, this[key], key, matcher))
                )
              }
              return false
            }
            delete(header, matcher) {
              const self = this
              let deleted = false
              function deleteHeader(_header) {
                _header = normalizeHeader(_header)
                if (_header) {
                  const key = _utils.default.findKey(self, _header)
                  if (key && (!matcher || matchHeaderValue(self, self[key], key, matcher))) {
                    delete self[key]
                    deleted = true
                  }
                }
              }
              if (_utils.default.isArray(header)) {
                header.forEach(deleteHeader)
              } else {
                deleteHeader(header)
              }
              return deleted
            }
            clear(matcher) {
              const keys = Object.keys(this)
              let i = keys.length
              let deleted = false
              while (i--) {
                const key = keys[i]
                if (!matcher || matchHeaderValue(this, this[key], key, matcher, true)) {
                  delete this[key]
                  deleted = true
                }
              }
              return deleted
            }
            normalize(format) {
              const self = this
              const headers = {}
              _utils.default.forEach(this, (value, header) => {
                const key = _utils.default.findKey(headers, header)
                if (key) {
                  self[key] = normalizeValue(value)
                  delete self[header]
                  return
                }
                const normalized = format ? formatHeader(header) : String(header).trim()
                if (normalized !== header) {
                  delete self[header]
                }
                self[normalized] = normalizeValue(value)
                headers[normalized] = true
              })
              return this
            }
            concat(...targets) {
              return this.constructor.concat(this, ...targets)
            }
            toJSON(asStrings) {
              const obj = Object.create(null)
              _utils.default.forEach(this, (value, header) => {
                value != null &&
                  value !== false &&
                  (obj[header] = asStrings && _utils.default.isArray(value) ? value.join(", ") : value)
              })
              return obj
            }
            [Symbol.iterator]() {
              return Object.entries(this.toJSON())[Symbol.iterator]()
            }
            toString() {
              return Object.entries(this.toJSON())
                .map(([header, value]) => header + ": " + value)
                .join("\n")
            }
            getSetCookie() {
              return this.get("set-cookie") || []
            }
            get [Symbol.toStringTag]() {
              return "AxiosHeaders"
            }
            static from(thing) {
              return thing instanceof this ? thing : new this(thing)
            }
            static concat(first, ...targets) {
              const computed = new this(first)
              targets.forEach((target) => computed.set(target))
              return computed
            }
            static accessor(header) {
              const internals =
                (this[$internals] =
                this[$internals] =
                  {
                    accessors: {},
                  })
              const accessors = internals.accessors
              const prototype = this.prototype
              function defineAccessor(_header) {
                const lHeader = normalizeHeader(_header)
                if (!accessors[lHeader]) {
                  buildAccessors(prototype, _header)
                  accessors[lHeader] = true
                }
              }
              _utils.default.isArray(header) ? header.forEach(defineAccessor) : defineAccessor(header)
              return this
            }
          }
          AxiosHeaders.accessor([
            "Content-Type",
            "Content-Length",
            "Accept",
            "Accept-Encoding",
            "User-Agent",
            "Authorization",
          ])

          // reserved names hotfix
          _utils.default.reduceDescriptors(AxiosHeaders.prototype, ({ value }, key) => {
            let mapped = key[0].toUpperCase() + key.slice(1) // map `set` => `Set`
            return {
              get: () => value,
              set(headerValue) {
                this[mapped] = headerValue
              },
            }
          })
          _utils.default.freezeMethods(AxiosHeaders)
          var _default = (exports.default = AxiosHeaders)
        },
        { "../helpers/parseHeaders.js": 52, "../utils.js": 69 },
      ],
      31: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          class InterceptorManager {
            constructor() {
              this.handlers = []
            }

            /**
             * Add a new interceptor to the stack
             *
             * @param {Function} fulfilled The function to handle `then` for a `Promise`
             * @param {Function} rejected The function to handle `reject` for a `Promise`
             *
             * @return {Number} An ID used to remove interceptor later
             */
            use(fulfilled, rejected, options) {
              this.handlers.push({
                fulfilled,
                rejected,
                synchronous: options ? options.synchronous : false,
                runWhen: options ? options.runWhen : null,
              })
              return this.handlers.length - 1
            }

            /**
             * Remove an interceptor from the stack
             *
             * @param {Number} id The ID that was returned by `use`
             *
             * @returns {void}
             */
            eject(id) {
              if (this.handlers[id]) {
                this.handlers[id] = null
              }
            }

            /**
             * Clear all interceptors from the stack
             *
             * @returns {void}
             */
            clear() {
              if (this.handlers) {
                this.handlers = []
              }
            }

            /**
             * Iterate over all the registered interceptors
             *
             * This method is particularly useful for skipping over any
             * interceptors that may have become `null` calling `eject`.
             *
             * @param {Function} fn The function to call for each interceptor
             *
             * @returns {void}
             */
            forEach(fn) {
              _utils.default.forEach(this.handlers, function forEachHandler(h) {
                if (h !== null) {
                  fn(h)
                }
              })
            }
          }
          var _default = (exports.default = InterceptorManager)
        },
        { "./../utils.js": 69 },
      ],
      32: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = buildFullPath
          var _isAbsoluteURL = _interopRequireDefault(require("../helpers/isAbsoluteURL.js"))
          var _combineURLs = _interopRequireDefault(require("../helpers/combineURLs.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Creates a new URL by combining the baseURL with the requestedURL,
           * only when the requestedURL is not already an absolute URL.
           * If the requestURL is absolute, this function returns the requestedURL untouched.
           *
           * @param {string} baseURL The base URL
           * @param {string} requestedURL Absolute or relative URL to combine
           *
           * @returns {string} The combined full path
           */
          function buildFullPath(baseURL, requestedURL, allowAbsoluteUrls) {
            let isRelativeUrl = !(0, _isAbsoluteURL.default)(requestedURL)
            if (baseURL && (isRelativeUrl || allowAbsoluteUrls == false)) {
              return (0, _combineURLs.default)(baseURL, requestedURL)
            }
            return requestedURL
          }
        },
        { "../helpers/combineURLs.js": 44, "../helpers/isAbsoluteURL.js": 48 },
      ],
      33: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = dispatchRequest
          var _transformData = _interopRequireDefault(require("./transformData.js"))
          var _isCancel = _interopRequireDefault(require("../cancel/isCancel.js"))
          var _index = _interopRequireDefault(require("../defaults/index.js"))
          var _CanceledError = _interopRequireDefault(require("../cancel/CanceledError.js"))
          var _AxiosHeaders = _interopRequireDefault(require("../core/AxiosHeaders.js"))
          var _adapters = _interopRequireDefault(require("../adapters/adapters.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Throws a `CanceledError` if cancellation has been requested.
           *
           * @param {Object} config The config that is to be used for the request
           *
           * @returns {void}
           */
          function throwIfCancellationRequested(config) {
            if (config.cancelToken) {
              config.cancelToken.throwIfRequested()
            }
            if (config.signal && config.signal.aborted) {
              throw new _CanceledError.default(null, config)
            }
          }

          /**
           * Dispatch a request to the server using the configured adapter.
           *
           * @param {object} config The config that is to be used for the request
           *
           * @returns {Promise} The Promise to be fulfilled
           */
          function dispatchRequest(config) {
            throwIfCancellationRequested(config)
            config.headers = _AxiosHeaders.default.from(config.headers)

            // Transform request data
            config.data = _transformData.default.call(config, config.transformRequest)
            if (["post", "put", "patch"].indexOf(config.method) !== -1) {
              config.headers.setContentType("application/x-www-form-urlencoded", false)
            }
            const adapter = _adapters.default.getAdapter(config.adapter || _index.default.adapter, config)
            return adapter(config).then(
              function onAdapterResolution(response) {
                throwIfCancellationRequested(config)

                // Transform response data
                response.data = _transformData.default.call(config, config.transformResponse, response)
                response.headers = _AxiosHeaders.default.from(response.headers)
                return response
              },
              function onAdapterRejection(reason) {
                if (!(0, _isCancel.default)(reason)) {
                  throwIfCancellationRequested(config)

                  // Transform response data
                  if (reason && reason.response) {
                    reason.response.data = _transformData.default.call(
                      config,
                      config.transformResponse,
                      reason.response,
                    )
                    reason.response.headers = _AxiosHeaders.default.from(reason.response.headers)
                  }
                }
                return Promise.reject(reason)
              },
            )
          }
        },
        {
          "../adapters/adapters.js": 21,
          "../cancel/CanceledError.js": 26,
          "../cancel/isCancel.js": 27,
          "../core/AxiosHeaders.js": 30,
          "../defaults/index.js": 37,
          "./transformData.js": 36,
        },
      ],
      34: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = mergeConfig
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _AxiosHeaders = _interopRequireDefault(require("./AxiosHeaders.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const headersToObject = (thing) =>
            thing instanceof _AxiosHeaders.default
              ? {
                  ...thing,
                }
              : thing

          /**
           * Config-specific merge-function which creates a new config-object
           * by merging two configuration objects together.
           *
           * @param {Object} config1
           * @param {Object} config2
           *
           * @returns {Object} New object resulting from merging config2 to config1
           */
          function mergeConfig(config1, config2) {
            // eslint-disable-next-line no-param-reassign
            config2 = config2 || {}
            const config = {}
            function getMergedValue(target, source, prop, caseless) {
              if (_utils.default.isPlainObject(target) && _utils.default.isPlainObject(source)) {
                return _utils.default.merge.call(
                  {
                    caseless,
                  },
                  target,
                  source,
                )
              } else if (_utils.default.isPlainObject(source)) {
                return _utils.default.merge({}, source)
              } else if (_utils.default.isArray(source)) {
                return source.slice()
              }
              return source
            }

            // eslint-disable-next-line consistent-return
            function mergeDeepProperties(a, b, prop, caseless) {
              if (!_utils.default.isUndefined(b)) {
                return getMergedValue(a, b, prop, caseless)
              } else if (!_utils.default.isUndefined(a)) {
                return getMergedValue(undefined, a, prop, caseless)
              }
            }

            // eslint-disable-next-line consistent-return
            function valueFromConfig2(a, b) {
              if (!_utils.default.isUndefined(b)) {
                return getMergedValue(undefined, b)
              }
            }

            // eslint-disable-next-line consistent-return
            function defaultToConfig2(a, b) {
              if (!_utils.default.isUndefined(b)) {
                return getMergedValue(undefined, b)
              } else if (!_utils.default.isUndefined(a)) {
                return getMergedValue(undefined, a)
              }
            }

            // eslint-disable-next-line consistent-return
            function mergeDirectKeys(a, b, prop) {
              if (prop in config2) {
                return getMergedValue(a, b)
              } else if (prop in config1) {
                return getMergedValue(undefined, a)
              }
            }
            const mergeMap = {
              url: valueFromConfig2,
              method: valueFromConfig2,
              data: valueFromConfig2,
              baseURL: defaultToConfig2,
              transformRequest: defaultToConfig2,
              transformResponse: defaultToConfig2,
              paramsSerializer: defaultToConfig2,
              timeout: defaultToConfig2,
              timeoutMessage: defaultToConfig2,
              withCredentials: defaultToConfig2,
              withXSRFToken: defaultToConfig2,
              adapter: defaultToConfig2,
              responseType: defaultToConfig2,
              xsrfCookieName: defaultToConfig2,
              xsrfHeaderName: defaultToConfig2,
              onUploadProgress: defaultToConfig2,
              onDownloadProgress: defaultToConfig2,
              decompress: defaultToConfig2,
              maxContentLength: defaultToConfig2,
              maxBodyLength: defaultToConfig2,
              beforeRedirect: defaultToConfig2,
              transport: defaultToConfig2,
              httpAgent: defaultToConfig2,
              httpsAgent: defaultToConfig2,
              cancelToken: defaultToConfig2,
              socketPath: defaultToConfig2,
              responseEncoding: defaultToConfig2,
              validateStatus: mergeDirectKeys,
              headers: (a, b, prop) => mergeDeepProperties(headersToObject(a), headersToObject(b), prop, true),
            }
            _utils.default.forEach(
              Object.keys({
                ...config1,
                ...config2,
              }),
              function computeConfigValue(prop) {
                const merge = mergeMap[prop] || mergeDeepProperties
                const configValue = merge(config1[prop], config2[prop], prop)
                ;(_utils.default.isUndefined(configValue) && merge !== mergeDirectKeys) || (config[prop] = configValue)
              },
            )
            return config
          }
        },
        { "../utils.js": 69, "./AxiosHeaders.js": 30 },
      ],
      35: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = settle
          var _AxiosError = _interopRequireDefault(require("./AxiosError.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Resolve or reject a Promise based on response status.
           *
           * @param {Function} resolve A function that resolves the promise.
           * @param {Function} reject A function that rejects the promise.
           * @param {object} response The response.
           *
           * @returns {object} The response.
           */
          function settle(resolve, reject, response) {
            const validateStatus = response.config.validateStatus
            if (!response.status || !validateStatus || validateStatus(response.status)) {
              resolve(response)
            } else {
              reject(
                new _AxiosError.default(
                  "Request failed with status code " + response.status,
                  [_AxiosError.default.ERR_BAD_REQUEST, _AxiosError.default.ERR_BAD_RESPONSE][
                    Math.floor(response.status / 100) - 4
                  ],
                  response.config,
                  response.request,
                  response,
                ),
              )
            }
          }
        },
        { "./AxiosError.js": 29 },
      ],
      36: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = transformData
          var _utils = _interopRequireDefault(require("./../utils.js"))
          var _index = _interopRequireDefault(require("../defaults/index.js"))
          var _AxiosHeaders = _interopRequireDefault(require("../core/AxiosHeaders.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Transform the data for a request or a response
           *
           * @param {Array|Function} fns A single function or Array of functions
           * @param {?Object} response The response object
           *
           * @returns {*} The resulting transformed data
           */
          function transformData(fns, response) {
            const config = this || _index.default
            const context = response || config
            const headers = _AxiosHeaders.default.from(context.headers)
            let data = context.data
            _utils.default.forEach(fns, function transform(fn) {
              data = fn.call(config, data, headers.normalize(), response ? response.status : undefined)
            })
            headers.normalize()
            return data
          }
        },
        { "../core/AxiosHeaders.js": 30, "../defaults/index.js": 37, "./../utils.js": 69 },
      ],
      37: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          var _transitional = _interopRequireDefault(require("./transitional.js"))
          var _toFormData = _interopRequireDefault(require("../helpers/toFormData.js"))
          var _toURLEncodedForm = _interopRequireDefault(require("../helpers/toURLEncodedForm.js"))
          var _index = _interopRequireDefault(require("../platform/index.js"))
          var _formDataToJSON = _interopRequireDefault(require("../helpers/formDataToJSON.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * It takes a string, tries to parse it, and if it fails, it returns the stringified version
           * of the input
           *
           * @param {any} rawValue - The value to be stringified.
           * @param {Function} parser - A function that parses a string into a JavaScript object.
           * @param {Function} encoder - A function that takes a value and returns a string.
           *
           * @returns {string} A stringified version of the rawValue.
           */
          function stringifySafely(rawValue, parser, encoder) {
            if (_utils.default.isString(rawValue)) {
              try {
                ;(parser || JSON.parse)(rawValue)
                return _utils.default.trim(rawValue)
              } catch (e) {
                if (e.name !== "SyntaxError") {
                  throw e
                }
              }
            }
            return (encoder || JSON.stringify)(rawValue)
          }
          const defaults = {
            transitional: _transitional.default,
            adapter: ["xhr", "http", "fetch"],
            transformRequest: [
              function transformRequest(data, headers) {
                const contentType = headers.getContentType() || ""
                const hasJSONContentType = contentType.indexOf("application/json") > -1
                const isObjectPayload = _utils.default.isObject(data)
                if (isObjectPayload && _utils.default.isHTMLForm(data)) {
                  data = new FormData(data)
                }
                const isFormData = _utils.default.isFormData(data)
                if (isFormData) {
                  return hasJSONContentType ? JSON.stringify((0, _formDataToJSON.default)(data)) : data
                }
                if (
                  _utils.default.isArrayBuffer(data) ||
                  _utils.default.isBuffer(data) ||
                  _utils.default.isStream(data) ||
                  _utils.default.isFile(data) ||
                  _utils.default.isBlob(data) ||
                  _utils.default.isReadableStream(data)
                ) {
                  return data
                }
                if (_utils.default.isArrayBufferView(data)) {
                  return data.buffer
                }
                if (_utils.default.isURLSearchParams(data)) {
                  headers.setContentType("application/x-www-form-urlencoded;charset=utf-8", false)
                  return data.toString()
                }
                let isFileList
                if (isObjectPayload) {
                  if (contentType.indexOf("application/x-www-form-urlencoded") > -1) {
                    return (0, _toURLEncodedForm.default)(data, this.formSerializer).toString()
                  }
                  if (
                    (isFileList = _utils.default.isFileList(data)) ||
                    contentType.indexOf("multipart/form-data") > -1
                  ) {
                    const _FormData = this.env && this.env.FormData
                    return (0, _toFormData.default)(
                      isFileList
                        ? {
                            "files[]": data,
                          }
                        : data,
                      _FormData && new _FormData(),
                      this.formSerializer,
                    )
                  }
                }
                if (isObjectPayload || hasJSONContentType) {
                  headers.setContentType("application/json", false)
                  return stringifySafely(data)
                }
                return data
              },
            ],
            transformResponse: [
              function transformResponse(data) {
                const transitional = this.transitional || defaults.transitional
                const forcedJSONParsing = transitional && transitional.forcedJSONParsing
                const JSONRequested = this.responseType === "json"
                if (_utils.default.isResponse(data) || _utils.default.isReadableStream(data)) {
                  return data
                }
                if (
                  data &&
                  _utils.default.isString(data) &&
                  ((forcedJSONParsing && !this.responseType) || JSONRequested)
                ) {
                  const silentJSONParsing = transitional && transitional.silentJSONParsing
                  const strictJSONParsing = !silentJSONParsing && JSONRequested
                  try {
                    return JSON.parse(data, this.parseReviver)
                  } catch (e) {
                    if (strictJSONParsing) {
                      if (e.name === "SyntaxError") {
                        throw _AxiosError.default.from(
                          e,
                          _AxiosError.default.ERR_BAD_RESPONSE,
                          this,
                          null,
                          this.response,
                        )
                      }
                      throw e
                    }
                  }
                }
                return data
              },
            ],
            /**
             * A timeout in milliseconds to abort a request. If set to 0 (default) a
             * timeout is not created.
             */
            timeout: 0,
            xsrfCookieName: "XSRF-TOKEN",
            xsrfHeaderName: "X-XSRF-TOKEN",
            maxContentLength: -1,
            maxBodyLength: -1,
            env: {
              FormData: _index.default.classes.FormData,
              Blob: _index.default.classes.Blob,
            },
            validateStatus: function validateStatus(status) {
              return status >= 200 && status < 300
            },
            headers: {
              common: {
                Accept: "application/json, text/plain, */*",
                "Content-Type": undefined,
              },
            },
          }
          _utils.default.forEach(["delete", "get", "head", "post", "put", "patch"], (method) => {
            defaults.headers[method] = {}
          })
          var _default = (exports.default = defaults)
        },
        {
          "../core/AxiosError.js": 29,
          "../helpers/formDataToJSON.js": 47,
          "../helpers/toFormData.js": 59,
          "../helpers/toURLEncodedForm.js": 60,
          "../platform/index.js": 68,
          "../utils.js": 69,
          "./transitional.js": 38,
        },
      ],
      38: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _default = (exports.default = {
            silentJSONParsing: true,
            forcedJSONParsing: true,
            clarifyTimeoutError: false,
          })
        },
        {},
      ],
      39: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.VERSION = void 0
          const VERSION = (exports.VERSION = "1.13.2")
        },
        {},
      ],
      40: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _toFormData = _interopRequireDefault(require("./toFormData.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * It encodes a string by replacing all characters that are not in the unreserved set with
           * their percent-encoded equivalents
           *
           * @param {string} str - The string to encode.
           *
           * @returns {string} The encoded string.
           */
          function encode(str) {
            const charMap = {
              "!": "%21",
              "'": "%27",
              "(": "%28",
              ")": "%29",
              "~": "%7E",
              "%20": "+",
              "%00": "\x00",
            }
            return encodeURIComponent(str).replace(/[!'()~]|%20|%00/g, function replacer(match) {
              return charMap[match]
            })
          }

          /**
           * It takes a params object and converts it to a FormData object
           *
           * @param {Object<string, any>} params - The parameters to be converted to a FormData object.
           * @param {Object<string, any>} options - The options object passed to the Axios constructor.
           *
           * @returns {void}
           */
          function AxiosURLSearchParams(params, options) {
            this._pairs = []
            params && (0, _toFormData.default)(params, this, options)
          }
          const prototype = AxiosURLSearchParams.prototype
          prototype.append = function append(name, value) {
            this._pairs.push([name, value])
          }
          prototype.toString = function toString(encoder) {
            const _encode = encoder
              ? function (value) {
                  return encoder.call(this, value, encode)
                }
              : encode
            return this._pairs
              .map(function each(pair) {
                return _encode(pair[0]) + "=" + _encode(pair[1])
              }, "")
              .join("&")
          }
          var _default = (exports.default = AxiosURLSearchParams)
        },
        { "./toFormData.js": 59 },
      ],
      41: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          const HttpStatusCode = {
            Continue: 100,
            SwitchingProtocols: 101,
            Processing: 102,
            EarlyHints: 103,
            Ok: 200,
            Created: 201,
            Accepted: 202,
            NonAuthoritativeInformation: 203,
            NoContent: 204,
            ResetContent: 205,
            PartialContent: 206,
            MultiStatus: 207,
            AlreadyReported: 208,
            ImUsed: 226,
            MultipleChoices: 300,
            MovedPermanently: 301,
            Found: 302,
            SeeOther: 303,
            NotModified: 304,
            UseProxy: 305,
            Unused: 306,
            TemporaryRedirect: 307,
            PermanentRedirect: 308,
            BadRequest: 400,
            Unauthorized: 401,
            PaymentRequired: 402,
            Forbidden: 403,
            NotFound: 404,
            MethodNotAllowed: 405,
            NotAcceptable: 406,
            ProxyAuthenticationRequired: 407,
            RequestTimeout: 408,
            Conflict: 409,
            Gone: 410,
            LengthRequired: 411,
            PreconditionFailed: 412,
            PayloadTooLarge: 413,
            UriTooLong: 414,
            UnsupportedMediaType: 415,
            RangeNotSatisfiable: 416,
            ExpectationFailed: 417,
            ImATeapot: 418,
            MisdirectedRequest: 421,
            UnprocessableEntity: 422,
            Locked: 423,
            FailedDependency: 424,
            TooEarly: 425,
            UpgradeRequired: 426,
            PreconditionRequired: 428,
            TooManyRequests: 429,
            RequestHeaderFieldsTooLarge: 431,
            UnavailableForLegalReasons: 451,
            InternalServerError: 500,
            NotImplemented: 501,
            BadGateway: 502,
            ServiceUnavailable: 503,
            GatewayTimeout: 504,
            HttpVersionNotSupported: 505,
            VariantAlsoNegotiates: 506,
            InsufficientStorage: 507,
            LoopDetected: 508,
            NotExtended: 510,
            NetworkAuthenticationRequired: 511,
            WebServerIsDown: 521,
            ConnectionTimedOut: 522,
            OriginIsUnreachable: 523,
            TimeoutOccurred: 524,
            SslHandshakeFailed: 525,
            InvalidSslCertificate: 526,
          }
          Object.entries(HttpStatusCode).forEach(([key, value]) => {
            HttpStatusCode[value] = key
          })
          var _default = (exports.default = HttpStatusCode)
        },
        {},
      ],
      42: [
        function (require, module, exports) {
          "use strict"

          /**
           * Create a bound version of a function with a specified `this` context
           *
           * @param {Function} fn - The function to bind
           * @param {*} thisArg - The value to be passed as the `this` parameter
           * @returns {Function} A new function that will call the original function with the specified `this` context
           */
          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = bind
          function bind(fn, thisArg) {
            return function wrap() {
              return fn.apply(thisArg, arguments)
            }
          }
        },
        {},
      ],
      43: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = buildURL
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _AxiosURLSearchParams = _interopRequireDefault(require("../helpers/AxiosURLSearchParams.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * It replaces all instances of the characters `:`, `$`, `,`, `+`, `[`, and `]` with their
           * URI encoded counterparts
           *
           * @param {string} val The value to be encoded.
           *
           * @returns {string} The encoded value.
           */
          function encode(val) {
            return encodeURIComponent(val)
              .replace(/%3A/gi, ":")
              .replace(/%24/g, "$")
              .replace(/%2C/gi, ",")
              .replace(/%20/g, "+")
          }

          /**
           * Build a URL by appending params to the end
           *
           * @param {string} url The base of the url (e.g., http://www.google.com)
           * @param {object} [params] The params to be appended
           * @param {?(object|Function)} options
           *
           * @returns {string} The formatted url
           */
          function buildURL(url, params, options) {
            /*eslint no-param-reassign:0*/
            if (!params) {
              return url
            }
            const _encode = (options && options.encode) || encode
            if (_utils.default.isFunction(options)) {
              options = {
                serialize: options,
              }
            }
            const serializeFn = options && options.serialize
            let serializedParams
            if (serializeFn) {
              serializedParams = serializeFn(params, options)
            } else {
              serializedParams = _utils.default.isURLSearchParams(params)
                ? params.toString()
                : new _AxiosURLSearchParams.default(params, options).toString(_encode)
            }
            if (serializedParams) {
              const hashmarkIndex = url.indexOf("#")
              if (hashmarkIndex !== -1) {
                url = url.slice(0, hashmarkIndex)
              }
              url += (url.indexOf("?") === -1 ? "?" : "&") + serializedParams
            }
            return url
          }
        },
        { "../helpers/AxiosURLSearchParams.js": 40, "../utils.js": 69 },
      ],
      44: [
        function (require, module, exports) {
          "use strict"

          /**
           * Creates a new URL by combining the specified URLs
           *
           * @param {string} baseURL The base URL
           * @param {string} relativeURL The relative URL
           *
           * @returns {string} The combined URL
           */
          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = combineURLs
          function combineURLs(baseURL, relativeURL) {
            return relativeURL ? baseURL.replace(/\/?\/$/, "") + "/" + relativeURL.replace(/^\/+/, "") : baseURL
          }
        },
        {},
      ],
      45: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _CanceledError = _interopRequireDefault(require("../cancel/CanceledError.js"))
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          var _utils = _interopRequireDefault(require("../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const composeSignals = (signals, timeout) => {
            const { length } = (signals = signals ? signals.filter(Boolean) : [])
            if (timeout || length) {
              let controller = new AbortController()
              let aborted
              const onabort = function (reason) {
                if (!aborted) {
                  aborted = true
                  unsubscribe()
                  const err = reason instanceof Error ? reason : this.reason
                  controller.abort(
                    err instanceof _AxiosError.default
                      ? err
                      : new _CanceledError.default(err instanceof Error ? err.message : err),
                  )
                }
              }
              let timer =
                timeout &&
                setTimeout(() => {
                  timer = null
                  onabort(new _AxiosError.default(`timeout ${timeout} of ms exceeded`, _AxiosError.default.ETIMEDOUT))
                }, timeout)
              const unsubscribe = () => {
                if (signals) {
                  timer && clearTimeout(timer)
                  timer = null
                  signals.forEach((signal) => {
                    signal.unsubscribe ? signal.unsubscribe(onabort) : signal.removeEventListener("abort", onabort)
                  })
                  signals = null
                }
              }
              signals.forEach((signal) => signal.addEventListener("abort", onabort))
              const { signal } = controller
              signal.unsubscribe = () => _utils.default.asap(unsubscribe)
              return signal
            }
          }
          var _default = (exports.default = composeSignals)
        },
        { "../cancel/CanceledError.js": 26, "../core/AxiosError.js": 29, "../utils.js": 69 },
      ],
      46: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./../utils.js"))
          var _index = _interopRequireDefault(require("../platform/index.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (exports.default = _index.default.hasStandardBrowserEnv
            ? // Standard browser envs support document.cookie
              {
                write(name, value, expires, path, domain, secure, sameSite) {
                  if (typeof document === "undefined") return
                  const cookie = [`${name}=${encodeURIComponent(value)}`]
                  if (_utils.default.isNumber(expires)) {
                    cookie.push(`expires=${new Date(expires).toUTCString()}`)
                  }
                  if (_utils.default.isString(path)) {
                    cookie.push(`path=${path}`)
                  }
                  if (_utils.default.isString(domain)) {
                    cookie.push(`domain=${domain}`)
                  }
                  if (secure === true) {
                    cookie.push("secure")
                  }
                  if (_utils.default.isString(sameSite)) {
                    cookie.push(`SameSite=${sameSite}`)
                  }
                  document.cookie = cookie.join("; ")
                },
                read(name) {
                  if (typeof document === "undefined") return null
                  const match = document.cookie.match(new RegExp("(?:^|; )" + name + "=([^;]*)"))
                  return match ? decodeURIComponent(match[1]) : null
                },
                remove(name) {
                  this.write(name, "", Date.now() - 86400000, "/")
                },
              }
            : // Non-standard browser env (web workers, react-native) lack needed support.
              {
                write() {},
                read() {
                  return null
                },
                remove() {},
              })
        },
        { "../platform/index.js": 68, "./../utils.js": 69 },
      ],
      47: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * It takes a string like `foo[x][y][z]` and returns an array like `['foo', 'x', 'y', 'z']
           *
           * @param {string} name - The name of the property to get.
           *
           * @returns An array of strings.
           */
          function parsePropPath(name) {
            // foo[x][y][z]
            // foo.x.y.z
            // foo-x-y-z
            // foo x y z
            return _utils.default.matchAll(/\w+|\[(\w*)]/g, name).map((match) => {
              return match[0] === "[]" ? "" : match[1] || match[0]
            })
          }

          /**
           * Convert an array to an object.
           *
           * @param {Array<any>} arr - The array to convert to an object.
           *
           * @returns An object with the same keys and values as the array.
           */
          function arrayToObject(arr) {
            const obj = {}
            const keys = Object.keys(arr)
            let i
            const len = keys.length
            let key
            for (i = 0; i < len; i++) {
              key = keys[i]
              obj[key] = arr[key]
            }
            return obj
          }

          /**
           * It takes a FormData object and returns a JavaScript object
           *
           * @param {string} formData The FormData object to convert to JSON.
           *
           * @returns {Object<string, any> | null} The converted object.
           */
          function formDataToJSON(formData) {
            function buildPath(path, value, target, index) {
              let name = path[index++]
              if (name === "__proto__") return true
              const isNumericKey = Number.isFinite(+name)
              const isLast = index >= path.length
              name = !name && _utils.default.isArray(target) ? target.length : name
              if (isLast) {
                if (_utils.default.hasOwnProp(target, name)) {
                  target[name] = [target[name], value]
                } else {
                  target[name] = value
                }
                return !isNumericKey
              }
              if (!target[name] || !_utils.default.isObject(target[name])) {
                target[name] = []
              }
              const result = buildPath(path, value, target[name], index)
              if (result && _utils.default.isArray(target[name])) {
                target[name] = arrayToObject(target[name])
              }
              return !isNumericKey
            }
            if (_utils.default.isFormData(formData) && _utils.default.isFunction(formData.entries)) {
              const obj = {}
              _utils.default.forEachEntry(formData, (name, value) => {
                buildPath(parsePropPath(name), value, obj, 0)
              })
              return obj
            }
            return null
          }
          var _default = (exports.default = formDataToJSON)
        },
        { "../utils.js": 69 },
      ],
      48: [
        function (require, module, exports) {
          "use strict"

          /**
           * Determines whether the specified URL is absolute
           *
           * @param {string} url The URL to test
           *
           * @returns {boolean} True if the specified URL is absolute, otherwise false
           */
          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = isAbsoluteURL
          function isAbsoluteURL(url) {
            // A URL is considered absolute if it begins with "<scheme>://" or "//" (protocol-relative URL).
            // RFC 3986 defines scheme name as a sequence of characters beginning with a letter and followed
            // by any combination of letters, digits, plus, period, or hyphen.
            return /^([a-z][a-z\d+\-.]*:)?\/\//i.test(url)
          }
        },
        {},
      ],
      49: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = isAxiosError
          var _utils = _interopRequireDefault(require("./../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          /**
           * Determines whether the payload is an error thrown by Axios
           *
           * @param {*} payload The value to test
           *
           * @returns {boolean} True if the payload is an error thrown by Axios, otherwise false
           */
          function isAxiosError(payload) {
            return _utils.default.isObject(payload) && payload.isAxiosError === true
          }
        },
        { "./../utils.js": 69 },
      ],
      50: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _index = _interopRequireDefault(require("../platform/index.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (exports.default = _index.default.hasStandardBrowserEnv
            ? ((origin, isMSIE) => (url) => {
                url = new URL(url, _index.default.origin)
                return (
                  origin.protocol === url.protocol && origin.host === url.host && (isMSIE || origin.port === url.port)
                )
              })(
                new URL(_index.default.origin),
                _index.default.navigator && /(msie|trident)/i.test(_index.default.navigator.userAgent),
              )
            : () => true)
        },
        { "../platform/index.js": 68 },
      ],
      51: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          // eslint-disable-next-line strict
          var _default = (exports.default = null)
        },
        {},
      ],
      52: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _utils = _interopRequireDefault(require("./../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          // RawAxiosHeaders whose duplicates are ignored by node
          // c.f. https://nodejs.org/api/http.html#http_message_headers
          const ignoreDuplicateOf = _utils.default.toObjectSet([
            "age",
            "authorization",
            "content-length",
            "content-type",
            "etag",
            "expires",
            "from",
            "host",
            "if-modified-since",
            "if-unmodified-since",
            "last-modified",
            "location",
            "max-forwards",
            "proxy-authorization",
            "referer",
            "retry-after",
            "user-agent",
          ])

          /**
           * Parse headers into an object
           *
           * ```
           * Date: Wed, 27 Aug 2014 08:58:49 GMT
           * Content-Type: application/json
           * Connection: keep-alive
           * Transfer-Encoding: chunked
           * ```
           *
           * @param {String} rawHeaders Headers needing to be parsed
           *
           * @returns {Object} Headers parsed into an object
           */
          var _default = (rawHeaders) => {
            const parsed = {}
            let key
            let val
            let i
            rawHeaders &&
              rawHeaders.split("\n").forEach(function parser(line) {
                i = line.indexOf(":")
                key = line.substring(0, i).trim().toLowerCase()
                val = line.substring(i + 1).trim()
                if (!key || (parsed[key] && ignoreDuplicateOf[key])) {
                  return
                }
                if (key === "set-cookie") {
                  if (parsed[key]) {
                    parsed[key].push(val)
                  } else {
                    parsed[key] = [val]
                  }
                } else {
                  parsed[key] = parsed[key] ? parsed[key] + ", " + val : val
                }
              })
            return parsed
          }
          exports.default = _default
        },
        { "./../utils.js": 69 },
      ],
      53: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = parseProtocol
          function parseProtocol(url) {
            const match = /^([-+\w]{1,25})(:?\/\/|:)/.exec(url)
            return (match && match[1]) || ""
          }
        },
        {},
      ],
      54: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.progressEventReducer = exports.progressEventDecorator = exports.asyncDecorator = void 0
          var _speedometer2 = _interopRequireDefault(require("./speedometer.js"))
          var _throttle = _interopRequireDefault(require("./throttle.js"))
          var _utils = _interopRequireDefault(require("../utils.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const progressEventReducer = (listener, isDownloadStream, freq = 3) => {
            let bytesNotified = 0
            const _speedometer = (0, _speedometer2.default)(50, 250)
            return (0, _throttle.default)((e) => {
              const loaded = e.loaded
              const total = e.lengthComputable ? e.total : undefined
              const progressBytes = loaded - bytesNotified
              const rate = _speedometer(progressBytes)
              const inRange = loaded <= total
              bytesNotified = loaded
              const data = {
                loaded,
                total,
                progress: total ? loaded / total : undefined,
                bytes: progressBytes,
                rate: rate ? rate : undefined,
                estimated: rate && total && inRange ? (total - loaded) / rate : undefined,
                event: e,
                lengthComputable: total != null,
                [isDownloadStream ? "download" : "upload"]: true,
              }
              listener(data)
            }, freq)
          }
          exports.progressEventReducer = progressEventReducer
          const progressEventDecorator = (total, throttled) => {
            const lengthComputable = total != null
            return [
              (loaded) =>
                throttled[0]({
                  lengthComputable,
                  total,
                  loaded,
                }),
              throttled[1],
            ]
          }
          exports.progressEventDecorator = progressEventDecorator
          const asyncDecorator =
            (fn) =>
            (...args) =>
              _utils.default.asap(() => fn(...args))
          exports.asyncDecorator = asyncDecorator
        },
        { "../utils.js": 69, "./speedometer.js": 56, "./throttle.js": 58 },
      ],
      55: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _index = _interopRequireDefault(require("../platform/index.js"))
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _isURLSameOrigin = _interopRequireDefault(require("./isURLSameOrigin.js"))
          var _cookies = _interopRequireDefault(require("./cookies.js"))
          var _buildFullPath = _interopRequireDefault(require("../core/buildFullPath.js"))
          var _mergeConfig = _interopRequireDefault(require("../core/mergeConfig.js"))
          var _AxiosHeaders = _interopRequireDefault(require("../core/AxiosHeaders.js"))
          var _buildURL = _interopRequireDefault(require("./buildURL.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (config) => {
            const newConfig = (0, _mergeConfig.default)({}, config)
            let { data, withXSRFToken, xsrfHeaderName, xsrfCookieName, headers, auth } = newConfig
            newConfig.headers = headers = _AxiosHeaders.default.from(headers)
            newConfig.url = (0, _buildURL.default)(
              (0, _buildFullPath.default)(newConfig.baseURL, newConfig.url, newConfig.allowAbsoluteUrls),
              config.params,
              config.paramsSerializer,
            )

            // HTTP basic authentication
            if (auth) {
              headers.set(
                "Authorization",
                "Basic " +
                  btoa(
                    (auth.username || "") + ":" + (auth.password ? unescape(encodeURIComponent(auth.password)) : ""),
                  ),
              )
            }
            if (_utils.default.isFormData(data)) {
              if (_index.default.hasStandardBrowserEnv || _index.default.hasStandardBrowserWebWorkerEnv) {
                headers.setContentType(undefined) // browser handles it
              } else if (_utils.default.isFunction(data.getHeaders)) {
                // Node.js FormData (like form-data package)
                const formHeaders = data.getHeaders()
                // Only set safe headers to avoid overwriting security headers
                const allowedHeaders = ["content-type", "content-length"]
                Object.entries(formHeaders).forEach(([key, val]) => {
                  if (allowedHeaders.includes(key.toLowerCase())) {
                    headers.set(key, val)
                  }
                })
              }
            }

            // Add xsrf header
            // This is only done if running in a standard browser environment.
            // Specifically not if we're in a web worker, or react-native.

            if (_index.default.hasStandardBrowserEnv) {
              withXSRFToken && _utils.default.isFunction(withXSRFToken) && (withXSRFToken = withXSRFToken(newConfig))
              if (withXSRFToken || (withXSRFToken !== false && (0, _isURLSameOrigin.default)(newConfig.url))) {
                // Add xsrf header
                const xsrfValue = xsrfHeaderName && xsrfCookieName && _cookies.default.read(xsrfCookieName)
                if (xsrfValue) {
                  headers.set(xsrfHeaderName, xsrfValue)
                }
              }
            }
            return newConfig
          }
          exports.default = _default
        },
        {
          "../core/AxiosHeaders.js": 30,
          "../core/buildFullPath.js": 32,
          "../core/mergeConfig.js": 34,
          "../platform/index.js": 68,
          "../utils.js": 69,
          "./buildURL.js": 43,
          "./cookies.js": 46,
          "./isURLSameOrigin.js": 50,
        },
      ],
      56: [
        function (require, module, exports) {
          "use strict"

          /**
           * Calculate data maxRate
           * @param {Number} [samplesCount= 10]
           * @param {Number} [min= 1000]
           * @returns {Function}
           */
          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          function speedometer(samplesCount, min) {
            samplesCount = samplesCount || 10
            const bytes = new Array(samplesCount)
            const timestamps = new Array(samplesCount)
            let head = 0
            let tail = 0
            let firstSampleTS
            min = min !== undefined ? min : 1000
            return function push(chunkLength) {
              const now = Date.now()
              const startedAt = timestamps[tail]
              if (!firstSampleTS) {
                firstSampleTS = now
              }
              bytes[head] = chunkLength
              timestamps[head] = now
              let i = tail
              let bytesCount = 0
              while (i !== head) {
                bytesCount += bytes[i++]
                i = i % samplesCount
              }
              head = (head + 1) % samplesCount
              if (head === tail) {
                tail = (tail + 1) % samplesCount
              }
              if (now - firstSampleTS < min) {
                return
              }
              const passed = startedAt && now - startedAt
              return passed ? Math.round((bytesCount * 1000) / passed) : undefined
            }
          }
          var _default = (exports.default = speedometer)
        },
        {},
      ],
      57: [
        function (require, module, exports) {
          "use strict"

          /**
           * Syntactic sugar for invoking a function and expanding an array for arguments.
           *
           * Common use case would be to use `Function.prototype.apply`.
           *
           *  ```js
           *  function f(x, y, z) {}
           *  var args = [1, 2, 3];
           *  f.apply(null, args);
           *  ```
           *
           * With `spread` this example can be re-written.
           *
           *  ```js
           *  spread(function(x, y, z) {})([1, 2, 3]);
           *  ```
           *
           * @param {Function} callback
           *
           * @returns {Function}
           */
          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = spread
          function spread(callback) {
            return function wrap(arr) {
              return callback.apply(null, arr)
            }
          }
        },
        {},
      ],
      58: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          /**
           * Throttle decorator
           * @param {Function} fn
           * @param {Number} freq
           * @return {Function}
           */
          function throttle(fn, freq) {
            let timestamp = 0
            let threshold = 1000 / freq
            let lastArgs
            let timer
            const invoke = (args, now = Date.now()) => {
              timestamp = now
              lastArgs = null
              if (timer) {
                clearTimeout(timer)
                timer = null
              }
              fn(...args)
            }
            const throttled = (...args) => {
              const now = Date.now()
              const passed = now - timestamp
              if (passed >= threshold) {
                invoke(args, now)
              } else {
                lastArgs = args
                if (!timer) {
                  timer = setTimeout(() => {
                    timer = null
                    invoke(lastArgs)
                  }, threshold - passed)
                }
              }
            }
            const flush = () => lastArgs && invoke(lastArgs)
            return [throttled, flush]
          }
          var _default = (exports.default = throttle)
        },
        {},
      ],
      59: [
        function (require, module, exports) {
          ;(function (Buffer) {
            ;(function () {
              "use strict"

              Object.defineProperty(exports, "__esModule", {
                value: true,
              })
              exports.default = void 0
              var _utils = _interopRequireDefault(require("../utils.js"))
              var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
              var _FormData = _interopRequireDefault(require("../platform/node/classes/FormData.js"))
              function _interopRequireDefault(e) {
                return e && e.__esModule ? e : { default: e }
              }
              // temporary hotfix to avoid circular references until AxiosURLSearchParams is refactored

              /**
               * Determines if the given thing is a array or js object.
               *
               * @param {string} thing - The object or array to be visited.
               *
               * @returns {boolean}
               */
              function isVisitable(thing) {
                return _utils.default.isPlainObject(thing) || _utils.default.isArray(thing)
              }

              /**
               * It removes the brackets from the end of a string
               *
               * @param {string} key - The key of the parameter.
               *
               * @returns {string} the key without the brackets.
               */
              function removeBrackets(key) {
                return _utils.default.endsWith(key, "[]") ? key.slice(0, -2) : key
              }

              /**
               * It takes a path, a key, and a boolean, and returns a string
               *
               * @param {string} path - The path to the current key.
               * @param {string} key - The key of the current object being iterated over.
               * @param {string} dots - If true, the key will be rendered with dots instead of brackets.
               *
               * @returns {string} The path to the current key.
               */
              function renderKey(path, key, dots) {
                if (!path) return key
                return path
                  .concat(key)
                  .map(function each(token, i) {
                    // eslint-disable-next-line no-param-reassign
                    token = removeBrackets(token)
                    return !dots && i ? "[" + token + "]" : token
                  })
                  .join(dots ? "." : "")
              }

              /**
               * If the array is an array and none of its elements are visitable, then it's a flat array.
               *
               * @param {Array<any>} arr - The array to check
               *
               * @returns {boolean}
               */
              function isFlatArray(arr) {
                return _utils.default.isArray(arr) && !arr.some(isVisitable)
              }
              const predicates = _utils.default.toFlatObject(_utils.default, {}, null, function filter(prop) {
                return /^is[A-Z]/.test(prop)
              })

              /**
               * Convert a data object to FormData
               *
               * @param {Object} obj
               * @param {?Object} [formData]
               * @param {?Object} [options]
               * @param {Function} [options.visitor]
               * @param {Boolean} [options.metaTokens = true]
               * @param {Boolean} [options.dots = false]
               * @param {?Boolean} [options.indexes = false]
               *
               * @returns {Object}
               **/

              /**
               * It converts an object into a FormData object
               *
               * @param {Object<any, any>} obj - The object to convert to form data.
               * @param {string} formData - The FormData object to append to.
               * @param {Object<string, any>} options
               *
               * @returns
               */
              function toFormData(obj, formData, options) {
                if (!_utils.default.isObject(obj)) {
                  throw new TypeError("target must be an object")
                }

                // eslint-disable-next-line no-param-reassign
                formData = formData || new (_FormData.default || FormData)()

                // eslint-disable-next-line no-param-reassign
                options = _utils.default.toFlatObject(
                  options,
                  {
                    metaTokens: true,
                    dots: false,
                    indexes: false,
                  },
                  false,
                  function defined(option, source) {
                    // eslint-disable-next-line no-eq-null,eqeqeq
                    return !_utils.default.isUndefined(source[option])
                  },
                )
                const metaTokens = options.metaTokens
                // eslint-disable-next-line no-use-before-define
                const visitor = options.visitor || defaultVisitor
                const dots = options.dots
                const indexes = options.indexes
                const _Blob = options.Blob || (typeof Blob !== "undefined" && Blob)
                const useBlob = _Blob && _utils.default.isSpecCompliantForm(formData)
                if (!_utils.default.isFunction(visitor)) {
                  throw new TypeError("visitor must be a function")
                }
                function convertValue(value) {
                  if (value === null) return ""
                  if (_utils.default.isDate(value)) {
                    return value.toISOString()
                  }
                  if (_utils.default.isBoolean(value)) {
                    return value.toString()
                  }
                  if (!useBlob && _utils.default.isBlob(value)) {
                    throw new _AxiosError.default("Blob is not supported. Use a Buffer instead.")
                  }
                  if (_utils.default.isArrayBuffer(value) || _utils.default.isTypedArray(value)) {
                    return useBlob && typeof Blob === "function" ? new Blob([value]) : Buffer.from(value)
                  }
                  return value
                }

                /**
                 * Default visitor.
                 *
                 * @param {*} value
                 * @param {String|Number} key
                 * @param {Array<String|Number>} path
                 * @this {FormData}
                 *
                 * @returns {boolean} return true to visit the each prop of the value recursively
                 */
                function defaultVisitor(value, key, path) {
                  let arr = value
                  if (value && !path && typeof value === "object") {
                    if (_utils.default.endsWith(key, "{}")) {
                      // eslint-disable-next-line no-param-reassign
                      key = metaTokens ? key : key.slice(0, -2)
                      // eslint-disable-next-line no-param-reassign
                      value = JSON.stringify(value)
                    } else if (
                      (_utils.default.isArray(value) && isFlatArray(value)) ||
                      ((_utils.default.isFileList(value) || _utils.default.endsWith(key, "[]")) &&
                        (arr = _utils.default.toArray(value)))
                    ) {
                      // eslint-disable-next-line no-param-reassign
                      key = removeBrackets(key)
                      arr.forEach(function each(el, index) {
                        !(_utils.default.isUndefined(el) || el === null) &&
                          formData.append(
                            // eslint-disable-next-line no-nested-ternary
                            indexes === true ? renderKey([key], index, dots) : indexes === null ? key : key + "[]",
                            convertValue(el),
                          )
                      })
                      return false
                    }
                  }
                  if (isVisitable(value)) {
                    return true
                  }
                  formData.append(renderKey(path, key, dots), convertValue(value))
                  return false
                }
                const stack = []
                const exposedHelpers = Object.assign(predicates, {
                  defaultVisitor,
                  convertValue,
                  isVisitable,
                })
                function build(value, path) {
                  if (_utils.default.isUndefined(value)) return
                  if (stack.indexOf(value) !== -1) {
                    throw Error("Circular reference detected in " + path.join("."))
                  }
                  stack.push(value)
                  _utils.default.forEach(value, function each(el, key) {
                    const result =
                      !(_utils.default.isUndefined(el) || el === null) &&
                      visitor.call(formData, el, _utils.default.isString(key) ? key.trim() : key, path, exposedHelpers)
                    if (result === true) {
                      build(el, path ? path.concat(key) : [key])
                    }
                  })
                  stack.pop()
                }
                if (!_utils.default.isObject(obj)) {
                  throw new TypeError("data must be an object")
                }
                build(obj)
                return formData
              }
              var _default = (exports.default = toFormData)
            }).call(this)
          }).call(this, require("buffer").Buffer)
        },
        { "../core/AxiosError.js": 29, "../platform/node/classes/FormData.js": 51, "../utils.js": 69, buffer: 71 },
      ],
      60: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = toURLEncodedForm
          var _utils = _interopRequireDefault(require("../utils.js"))
          var _toFormData = _interopRequireDefault(require("./toFormData.js"))
          var _index = _interopRequireDefault(require("../platform/index.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          function toURLEncodedForm(data, options) {
            return (0, _toFormData.default)(data, new _index.default.classes.URLSearchParams(), {
              visitor: function (value, key, path, helpers) {
                if (_index.default.isNode && _utils.default.isBuffer(value)) {
                  this.append(key, value.toString("base64"))
                  return false
                }
                return helpers.defaultVisitor.apply(this, arguments)
              },
              ...options,
            })
          }
        },
        { "../platform/index.js": 68, "../utils.js": 69, "./toFormData.js": 59 },
      ],
      61: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.trackStream = exports.streamChunk = exports.readBytes = void 0
          const streamChunk = function* (chunk, chunkSize) {
            let len = chunk.byteLength
            if (!chunkSize || len < chunkSize) {
              yield chunk
              return
            }
            let pos = 0
            let end
            while (pos < len) {
              end = pos + chunkSize
              yield chunk.slice(pos, end)
              pos = end
            }
          }
          exports.streamChunk = streamChunk
          const readBytes = async function* (iterable, chunkSize) {
            for await (const chunk of readStream(iterable)) {
              yield* streamChunk(chunk, chunkSize)
            }
          }
          exports.readBytes = readBytes
          const readStream = async function* (stream) {
            if (stream[Symbol.asyncIterator]) {
              yield* stream
              return
            }
            const reader = stream.getReader()
            try {
              for (;;) {
                const { done, value } = await reader.read()
                if (done) {
                  break
                }
                yield value
              }
            } finally {
              await reader.cancel()
            }
          }
          const trackStream = (stream, chunkSize, onProgress, onFinish) => {
            const iterator = readBytes(stream, chunkSize)
            let bytes = 0
            let done
            let _onFinish = (e) => {
              if (!done) {
                done = true
                onFinish && onFinish(e)
              }
            }
            return new ReadableStream(
              {
                async pull(controller) {
                  try {
                    const { done, value } = await iterator.next()
                    if (done) {
                      _onFinish()
                      controller.close()
                      return
                    }
                    let len = value.byteLength
                    if (onProgress) {
                      let loadedBytes = (bytes += len)
                      onProgress(loadedBytes)
                    }
                    controller.enqueue(new Uint8Array(value))
                  } catch (err) {
                    _onFinish(err)
                    throw err
                  }
                },
                cancel(reason) {
                  _onFinish(reason)
                  return iterator.return()
                },
              },
              {
                highWaterMark: 2,
              },
            )
          }
          exports.trackStream = trackStream
        },
        {},
      ],
      62: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _data = require("../env/data.js")
          var _AxiosError = _interopRequireDefault(require("../core/AxiosError.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          const validators = {}

          // eslint-disable-next-line func-names
          ;["object", "boolean", "number", "function", "string", "symbol"].forEach((type, i) => {
            validators[type] = function validator(thing) {
              return typeof thing === type || "a" + (i < 1 ? "n " : " ") + type
            }
          })
          const deprecatedWarnings = {}

          /**
           * Transitional option validator
           *
           * @param {function|boolean?} validator - set to false if the transitional option has been removed
           * @param {string?} version - deprecated version / removed since version
           * @param {string?} message - some message with additional info
           *
           * @returns {function}
           */
          validators.transitional = function transitional(validator, version, message) {
            function formatMessage(opt, desc) {
              return (
                "[Axios v" +
                _data.VERSION +
                "] Transitional option '" +
                opt +
                "'" +
                desc +
                (message ? ". " + message : "")
              )
            }

            // eslint-disable-next-line func-names
            return (value, opt, opts) => {
              if (validator === false) {
                throw new _AxiosError.default(
                  formatMessage(opt, " has been removed" + (version ? " in " + version : "")),
                  _AxiosError.default.ERR_DEPRECATED,
                )
              }
              if (version && !deprecatedWarnings[opt]) {
                deprecatedWarnings[opt] = true
                // eslint-disable-next-line no-console
                console.warn(
                  formatMessage(
                    opt,
                    " has been deprecated since v" + version + " and will be removed in the near future",
                  ),
                )
              }
              return validator ? validator(value, opt, opts) : true
            }
          }
          validators.spelling = function spelling(correctSpelling) {
            return (value, opt) => {
              // eslint-disable-next-line no-console
              console.warn(`${opt} is likely a misspelling of ${correctSpelling}`)
              return true
            }
          }

          /**
           * Assert object's properties type
           *
           * @param {object} options
           * @param {object} schema
           * @param {boolean?} allowUnknown
           *
           * @returns {object}
           */

          function assertOptions(options, schema, allowUnknown) {
            if (typeof options !== "object") {
              throw new _AxiosError.default("options must be an object", _AxiosError.default.ERR_BAD_OPTION_VALUE)
            }
            const keys = Object.keys(options)
            let i = keys.length
            while (i-- > 0) {
              const opt = keys[i]
              const validator = schema[opt]
              if (validator) {
                const value = options[opt]
                const result = value === undefined || validator(value, opt, options)
                if (result !== true) {
                  throw new _AxiosError.default(
                    "option " + opt + " must be " + result,
                    _AxiosError.default.ERR_BAD_OPTION_VALUE,
                  )
                }
                continue
              }
              if (allowUnknown !== true) {
                throw new _AxiosError.default("Unknown option " + opt, _AxiosError.default.ERR_BAD_OPTION)
              }
            }
          }
          var _default = (exports.default = {
            assertOptions,
            validators,
          })
        },
        { "../core/AxiosError.js": 29, "../env/data.js": 39 },
      ],
      63: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _default = (exports.default = typeof Blob !== "undefined" ? Blob : null)
        },
        {},
      ],
      64: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _default = (exports.default = typeof FormData !== "undefined" ? FormData : null)
        },
        {},
      ],
      65: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _AxiosURLSearchParams = _interopRequireDefault(require("../../../helpers/AxiosURLSearchParams.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (exports.default =
            typeof URLSearchParams !== "undefined" ? URLSearchParams : _AxiosURLSearchParams.default)
        },
        { "../../../helpers/AxiosURLSearchParams.js": 40 },
      ],
      66: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _URLSearchParams = _interopRequireDefault(require("./classes/URLSearchParams.js"))
          var _FormData = _interopRequireDefault(require("./classes/FormData.js"))
          var _Blob = _interopRequireDefault(require("./classes/Blob.js"))
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (exports.default = {
            isBrowser: true,
            classes: {
              URLSearchParams: _URLSearchParams.default,
              FormData: _FormData.default,
              Blob: _Blob.default,
            },
            protocols: ["http", "https", "file", "blob", "url", "data"],
          })
        },
        { "./classes/Blob.js": 63, "./classes/FormData.js": 64, "./classes/URLSearchParams.js": 65 },
      ],
      67: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.origin =
            exports.navigator =
            exports.hasStandardBrowserWebWorkerEnv =
            exports.hasStandardBrowserEnv =
            exports.hasBrowserEnv =
              void 0
          const hasBrowserEnv = (exports.hasBrowserEnv =
            typeof window !== "undefined" && typeof document !== "undefined")
          const _navigator = (exports.navigator = (typeof navigator === "object" && navigator) || undefined)

          /**
           * Determine if we're running in a standard browser environment
           *
           * This allows axios to run in a web worker, and react-native.
           * Both environments support XMLHttpRequest, but not fully standard globals.
           *
           * web workers:
           *  typeof window -> undefined
           *  typeof document -> undefined
           *
           * react-native:
           *  navigator.product -> 'ReactNative'
           * nativescript
           *  navigator.product -> 'NativeScript' or 'NS'
           *
           * @returns {boolean}
           */
          const hasStandardBrowserEnv = (exports.hasStandardBrowserEnv =
            hasBrowserEnv && (!_navigator || ["ReactNative", "NativeScript", "NS"].indexOf(_navigator.product) < 0))

          /**
           * Determine if we're running in a standard browser webWorker environment
           *
           * Although the `isStandardBrowserEnv` method indicates that
           * `allows axios to run in a web worker`, the WebWorker will still be
           * filtered out due to its judgment standard
           * `typeof window !== 'undefined' && typeof document !== 'undefined'`.
           * This leads to a problem when axios post `FormData` in webWorker
           */
          const hasStandardBrowserWebWorkerEnv = (exports.hasStandardBrowserWebWorkerEnv = (() => {
            return (
              typeof WorkerGlobalScope !== "undefined" &&
              // eslint-disable-next-line no-undef
              self instanceof WorkerGlobalScope &&
              typeof self.importScripts === "function"
            )
          })())
          const origin = (exports.origin = (hasBrowserEnv && window.location.href) || "http://localhost")
        },
        {},
      ],
      68: [
        function (require, module, exports) {
          "use strict"

          Object.defineProperty(exports, "__esModule", {
            value: true,
          })
          exports.default = void 0
          var _index = _interopRequireDefault(require("./node/index.js"))
          var utils = _interopRequireWildcard(require("./common/utils.js"))
          function _interopRequireWildcard(e, t) {
            if ("function" == typeof WeakMap)
              var r = new WeakMap(),
                n = new WeakMap()
            return (_interopRequireWildcard = function (e, t) {
              if (!t && e && e.__esModule) return e
              var o,
                i,
                f = { __proto__: null, default: e }
              if (null === e || ("object" != typeof e && "function" != typeof e)) return f
              if ((o = t ? n : r)) {
                if (o.has(e)) return o.get(e)
                o.set(e, f)
              }
              for (const t in e)
                "default" !== t &&
                  {}.hasOwnProperty.call(e, t) &&
                  ((i = (o = Object.defineProperty) && Object.getOwnPropertyDescriptor(e, t)) && (i.get || i.set)
                    ? o(f, t, i)
                    : (f[t] = e[t]))
              return f
            })(e, t)
          }
          function _interopRequireDefault(e) {
            return e && e.__esModule ? e : { default: e }
          }
          var _default = (exports.default = {
            ...utils,
            ..._index.default,
          })
        },
        { "./common/utils.js": 67, "./node/index.js": 66 },
      ],
      69: [
        function (require, module, exports) {
          ;(function (process, global, setImmediate) {
            ;(function () {
              "use strict"

              Object.defineProperty(exports, "__esModule", {
                value: true,
              })
              exports.default = void 0
              var _bind = _interopRequireDefault(require("./helpers/bind.js"))
              function _interopRequireDefault(e) {
                return e && e.__esModule ? e : { default: e }
              }
              // utils is a library of generic helper functions non-specific to axios

              const { toString } = Object.prototype
              const { getPrototypeOf } = Object
              const { iterator, toStringTag } = Symbol
              const kindOf = ((cache) => (thing) => {
                const str = toString.call(thing)
                return cache[str] || (cache[str] = str.slice(8, -1).toLowerCase())
              })(Object.create(null))
              const kindOfTest = (type) => {
                type = type.toLowerCase()
                return (thing) => kindOf(thing) === type
              }
              const typeOfTest = (type) => (thing) => typeof thing === type

              /**
               * Determine if a value is an Array
               *
               * @param {Object} val The value to test
               *
               * @returns {boolean} True if value is an Array, otherwise false
               */
              const { isArray } = Array

              /**
               * Determine if a value is undefined
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if the value is undefined, otherwise false
               */
              const isUndefined = typeOfTest("undefined")

              /**
               * Determine if a value is a Buffer
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a Buffer, otherwise false
               */
              function isBuffer(val) {
                return (
                  val !== null &&
                  !isUndefined(val) &&
                  val.constructor !== null &&
                  !isUndefined(val.constructor) &&
                  isFunction(val.constructor.isBuffer) &&
                  val.constructor.isBuffer(val)
                )
              }

              /**
               * Determine if a value is an ArrayBuffer
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is an ArrayBuffer, otherwise false
               */
              const isArrayBuffer = kindOfTest("ArrayBuffer")

              /**
               * Determine if a value is a view on an ArrayBuffer
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a view on an ArrayBuffer, otherwise false
               */
              function isArrayBufferView(val) {
                let result
                if (typeof ArrayBuffer !== "undefined" && ArrayBuffer.isView) {
                  result = ArrayBuffer.isView(val)
                } else {
                  result = val && val.buffer && isArrayBuffer(val.buffer)
                }
                return result
              }

              /**
               * Determine if a value is a String
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a String, otherwise false
               */
              const isString = typeOfTest("string")

              /**
               * Determine if a value is a Function
               *
               * @param {*} val The value to test
               * @returns {boolean} True if value is a Function, otherwise false
               */
              const isFunction = typeOfTest("function")

              /**
               * Determine if a value is a Number
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a Number, otherwise false
               */
              const isNumber = typeOfTest("number")

              /**
               * Determine if a value is an Object
               *
               * @param {*} thing The value to test
               *
               * @returns {boolean} True if value is an Object, otherwise false
               */
              const isObject = (thing) => thing !== null && typeof thing === "object"

              /**
               * Determine if a value is a Boolean
               *
               * @param {*} thing The value to test
               * @returns {boolean} True if value is a Boolean, otherwise false
               */
              const isBoolean = (thing) => thing === true || thing === false

              /**
               * Determine if a value is a plain Object
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a plain Object, otherwise false
               */
              const isPlainObject = (val) => {
                if (kindOf(val) !== "object") {
                  return false
                }
                const prototype = getPrototypeOf(val)
                return (
                  (prototype === null || prototype === Object.prototype || Object.getPrototypeOf(prototype) === null) &&
                  !(toStringTag in val) &&
                  !(iterator in val)
                )
              }

              /**
               * Determine if a value is an empty object (safely handles Buffers)
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is an empty object, otherwise false
               */
              const isEmptyObject = (val) => {
                // Early return for non-objects or Buffers to prevent RangeError
                if (!isObject(val) || isBuffer(val)) {
                  return false
                }
                try {
                  return Object.keys(val).length === 0 && Object.getPrototypeOf(val) === Object.prototype
                } catch (e) {
                  // Fallback for any other objects that might cause RangeError with Object.keys()
                  return false
                }
              }

              /**
               * Determine if a value is a Date
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a Date, otherwise false
               */
              const isDate = kindOfTest("Date")

              /**
               * Determine if a value is a File
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a File, otherwise false
               */
              const isFile = kindOfTest("File")

              /**
               * Determine if a value is a Blob
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a Blob, otherwise false
               */
              const isBlob = kindOfTest("Blob")

              /**
               * Determine if a value is a FileList
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a File, otherwise false
               */
              const isFileList = kindOfTest("FileList")

              /**
               * Determine if a value is a Stream
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a Stream, otherwise false
               */
              const isStream = (val) => isObject(val) && isFunction(val.pipe)

              /**
               * Determine if a value is a FormData
               *
               * @param {*} thing The value to test
               *
               * @returns {boolean} True if value is an FormData, otherwise false
               */
              const isFormData = (thing) => {
                let kind
                return (
                  thing &&
                  ((typeof FormData === "function" && thing instanceof FormData) ||
                    (isFunction(thing.append) &&
                      ((kind = kindOf(thing)) === "formdata" ||
                        // detect form-data instance
                        (kind === "object" && isFunction(thing.toString) && thing.toString() === "[object FormData]"))))
                )
              }

              /**
               * Determine if a value is a URLSearchParams object
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a URLSearchParams object, otherwise false
               */
              const isURLSearchParams = kindOfTest("URLSearchParams")
              const [isReadableStream, isRequest, isResponse, isHeaders] = [
                "ReadableStream",
                "Request",
                "Response",
                "Headers",
              ].map(kindOfTest)

              /**
               * Trim excess whitespace off the beginning and end of a string
               *
               * @param {String} str The String to trim
               *
               * @returns {String} The String freed of excess whitespace
               */
              const trim = (str) => (str.trim ? str.trim() : str.replace(/^[\s\uFEFF\xA0]+|[\s\uFEFF\xA0]+$/g, ""))

              /**
               * Iterate over an Array or an Object invoking a function for each item.
               *
               * If `obj` is an Array callback will be called passing
               * the value, index, and complete array for each item.
               *
               * If 'obj' is an Object callback will be called passing
               * the value, key, and complete object for each property.
               *
               * @param {Object|Array} obj The object to iterate
               * @param {Function} fn The callback to invoke for each item
               *
               * @param {Boolean} [allOwnKeys = false]
               * @returns {any}
               */
              function forEach(obj, fn, { allOwnKeys = false } = {}) {
                // Don't bother if no value provided
                if (obj === null || typeof obj === "undefined") {
                  return
                }
                let i
                let l

                // Force an array if not already something iterable
                if (typeof obj !== "object") {
                  /*eslint no-param-reassign:0*/
                  obj = [obj]
                }
                if (isArray(obj)) {
                  // Iterate over array values
                  for (i = 0, l = obj.length; i < l; i++) {
                    fn.call(null, obj[i], i, obj)
                  }
                } else {
                  // Buffer check
                  if (isBuffer(obj)) {
                    return
                  }

                  // Iterate over object keys
                  const keys = allOwnKeys ? Object.getOwnPropertyNames(obj) : Object.keys(obj)
                  const len = keys.length
                  let key
                  for (i = 0; i < len; i++) {
                    key = keys[i]
                    fn.call(null, obj[key], key, obj)
                  }
                }
              }
              function findKey(obj, key) {
                if (isBuffer(obj)) {
                  return null
                }
                key = key.toLowerCase()
                const keys = Object.keys(obj)
                let i = keys.length
                let _key
                while (i-- > 0) {
                  _key = keys[i]
                  if (key === _key.toLowerCase()) {
                    return _key
                  }
                }
                return null
              }
              const _global = (() => {
                /*eslint no-undef:0*/
                if (typeof globalThis !== "undefined") return globalThis
                return typeof self !== "undefined" ? self : typeof window !== "undefined" ? window : global
              })()
              const isContextDefined = (context) => !isUndefined(context) && context !== _global

              /**
               * Accepts varargs expecting each argument to be an object, then
               * immutably merges the properties of each object and returns result.
               *
               * When multiple objects contain the same key the later object in
               * the arguments list will take precedence.
               *
               * Example:
               *
               * ```js
               * var result = merge({foo: 123}, {foo: 456});
               * console.log(result.foo); // outputs 456
               * ```
               *
               * @param {Object} obj1 Object to merge
               *
               * @returns {Object} Result of all merge properties
               */
              function merge /* obj1, obj2, obj3, ... */() {
                const { caseless, skipUndefined } = (isContextDefined(this) && this) || {}
                const result = {}
                const assignValue = (val, key) => {
                  const targetKey = (caseless && findKey(result, key)) || key
                  if (isPlainObject(result[targetKey]) && isPlainObject(val)) {
                    result[targetKey] = merge(result[targetKey], val)
                  } else if (isPlainObject(val)) {
                    result[targetKey] = merge({}, val)
                  } else if (isArray(val)) {
                    result[targetKey] = val.slice()
                  } else if (!skipUndefined || !isUndefined(val)) {
                    result[targetKey] = val
                  }
                }
                for (let i = 0, l = arguments.length; i < l; i++) {
                  arguments[i] && forEach(arguments[i], assignValue)
                }
                return result
              }

              /**
               * Extends object a by mutably adding to it the properties of object b.
               *
               * @param {Object} a The object to be extended
               * @param {Object} b The object to copy properties from
               * @param {Object} thisArg The object to bind function to
               *
               * @param {Boolean} [allOwnKeys]
               * @returns {Object} The resulting value of object a
               */
              const extend = (a, b, thisArg, { allOwnKeys } = {}) => {
                forEach(
                  b,
                  (val, key) => {
                    if (thisArg && isFunction(val)) {
                      a[key] = (0, _bind.default)(val, thisArg)
                    } else {
                      a[key] = val
                    }
                  },
                  {
                    allOwnKeys,
                  },
                )
                return a
              }

              /**
               * Remove byte order marker. This catches EF BB BF (the UTF-8 BOM)
               *
               * @param {string} content with BOM
               *
               * @returns {string} content value without BOM
               */
              const stripBOM = (content) => {
                if (content.charCodeAt(0) === 0xfeff) {
                  content = content.slice(1)
                }
                return content
              }

              /**
               * Inherit the prototype methods from one constructor into another
               * @param {function} constructor
               * @param {function} superConstructor
               * @param {object} [props]
               * @param {object} [descriptors]
               *
               * @returns {void}
               */
              const inherits = (constructor, superConstructor, props, descriptors) => {
                constructor.prototype = Object.create(superConstructor.prototype, descriptors)
                constructor.prototype.constructor = constructor
                Object.defineProperty(constructor, "super", {
                  value: superConstructor.prototype,
                })
                props && Object.assign(constructor.prototype, props)
              }

              /**
               * Resolve object with deep prototype chain to a flat object
               * @param {Object} sourceObj source object
               * @param {Object} [destObj]
               * @param {Function|Boolean} [filter]
               * @param {Function} [propFilter]
               *
               * @returns {Object}
               */
              const toFlatObject = (sourceObj, destObj, filter, propFilter) => {
                let props
                let i
                let prop
                const merged = {}
                destObj = destObj || {}
                // eslint-disable-next-line no-eq-null,eqeqeq
                if (sourceObj == null) return destObj
                do {
                  props = Object.getOwnPropertyNames(sourceObj)
                  i = props.length
                  while (i-- > 0) {
                    prop = props[i]
                    if ((!propFilter || propFilter(prop, sourceObj, destObj)) && !merged[prop]) {
                      destObj[prop] = sourceObj[prop]
                      merged[prop] = true
                    }
                  }
                  sourceObj = filter !== false && getPrototypeOf(sourceObj)
                } while (sourceObj && (!filter || filter(sourceObj, destObj)) && sourceObj !== Object.prototype)
                return destObj
              }

              /**
               * Determines whether a string ends with the characters of a specified string
               *
               * @param {String} str
               * @param {String} searchString
               * @param {Number} [position= 0]
               *
               * @returns {boolean}
               */
              const endsWith = (str, searchString, position) => {
                str = String(str)
                if (position === undefined || position > str.length) {
                  position = str.length
                }
                position -= searchString.length
                const lastIndex = str.indexOf(searchString, position)
                return lastIndex !== -1 && lastIndex === position
              }

              /**
               * Returns new array from array like object or null if failed
               *
               * @param {*} [thing]
               *
               * @returns {?Array}
               */
              const toArray = (thing) => {
                if (!thing) return null
                if (isArray(thing)) return thing
                let i = thing.length
                if (!isNumber(i)) return null
                const arr = new Array(i)
                while (i-- > 0) {
                  arr[i] = thing[i]
                }
                return arr
              }

              /**
               * Checking if the Uint8Array exists and if it does, it returns a function that checks if the
               * thing passed in is an instance of Uint8Array
               *
               * @param {TypedArray}
               *
               * @returns {Array}
               */
              // eslint-disable-next-line func-names
              const isTypedArray = ((TypedArray) => {
                // eslint-disable-next-line func-names
                return (thing) => {
                  return TypedArray && thing instanceof TypedArray
                }
              })(typeof Uint8Array !== "undefined" && getPrototypeOf(Uint8Array))

              /**
               * For each entry in the object, call the function with the key and value.
               *
               * @param {Object<any, any>} obj - The object to iterate over.
               * @param {Function} fn - The function to call for each entry.
               *
               * @returns {void}
               */
              const forEachEntry = (obj, fn) => {
                const generator = obj && obj[iterator]
                const _iterator = generator.call(obj)
                let result
                while ((result = _iterator.next()) && !result.done) {
                  const pair = result.value
                  fn.call(obj, pair[0], pair[1])
                }
              }

              /**
               * It takes a regular expression and a string, and returns an array of all the matches
               *
               * @param {string} regExp - The regular expression to match against.
               * @param {string} str - The string to search.
               *
               * @returns {Array<boolean>}
               */
              const matchAll = (regExp, str) => {
                let matches
                const arr = []
                while ((matches = regExp.exec(str)) !== null) {
                  arr.push(matches)
                }
                return arr
              }

              /* Checking if the kindOfTest function returns true when passed an HTMLFormElement. */
              const isHTMLForm = kindOfTest("HTMLFormElement")
              const toCamelCase = (str) => {
                return str.toLowerCase().replace(/[-_\s]([a-z\d])(\w*)/g, function replacer(m, p1, p2) {
                  return p1.toUpperCase() + p2
                })
              }

              /* Creating a function that will check if an object has a property. */
              const hasOwnProperty = (
                ({ hasOwnProperty }) =>
                (obj, prop) =>
                  hasOwnProperty.call(obj, prop)
              )(Object.prototype)

              /**
               * Determine if a value is a RegExp object
               *
               * @param {*} val The value to test
               *
               * @returns {boolean} True if value is a RegExp object, otherwise false
               */
              const isRegExp = kindOfTest("RegExp")
              const reduceDescriptors = (obj, reducer) => {
                const descriptors = Object.getOwnPropertyDescriptors(obj)
                const reducedDescriptors = {}
                forEach(descriptors, (descriptor, name) => {
                  let ret
                  if ((ret = reducer(descriptor, name, obj)) !== false) {
                    reducedDescriptors[name] = ret || descriptor
                  }
                })
                Object.defineProperties(obj, reducedDescriptors)
              }

              /**
               * Makes all methods read-only
               * @param {Object} obj
               */

              const freezeMethods = (obj) => {
                reduceDescriptors(obj, (descriptor, name) => {
                  // skip restricted props in strict mode
                  if (isFunction(obj) && ["arguments", "caller", "callee"].indexOf(name) !== -1) {
                    return false
                  }
                  const value = obj[name]
                  if (!isFunction(value)) return
                  descriptor.enumerable = false
                  if ("writable" in descriptor) {
                    descriptor.writable = false
                    return
                  }
                  if (!descriptor.set) {
                    descriptor.set = () => {
                      throw Error("Can not rewrite read-only method '" + name + "'")
                    }
                  }
                })
              }
              const toObjectSet = (arrayOrString, delimiter) => {
                const obj = {}
                const define = (arr) => {
                  arr.forEach((value) => {
                    obj[value] = true
                  })
                }
                isArray(arrayOrString) ? define(arrayOrString) : define(String(arrayOrString).split(delimiter))
                return obj
              }
              const noop = () => {}
              const toFiniteNumber = (value, defaultValue) => {
                return value != null && Number.isFinite((value = +value)) ? value : defaultValue
              }

              /**
               * If the thing is a FormData object, return true, otherwise return false.
               *
               * @param {unknown} thing - The thing to check.
               *
               * @returns {boolean}
               */
              function isSpecCompliantForm(thing) {
                return !!(thing && isFunction(thing.append) && thing[toStringTag] === "FormData" && thing[iterator])
              }
              const toJSONObject = (obj) => {
                const stack = new Array(10)
                const visit = (source, i) => {
                  if (isObject(source)) {
                    if (stack.indexOf(source) >= 0) {
                      return
                    }

                    //Buffer check
                    if (isBuffer(source)) {
                      return source
                    }
                    if (!("toJSON" in source)) {
                      stack[i] = source
                      const target = isArray(source) ? [] : {}
                      forEach(source, (value, key) => {
                        const reducedValue = visit(value, i + 1)
                        !isUndefined(reducedValue) && (target[key] = reducedValue)
                      })
                      stack[i] = undefined
                      return target
                    }
                  }
                  return source
                }
                return visit(obj, 0)
              }
              const isAsyncFn = kindOfTest("AsyncFunction")
              const isThenable = (thing) =>
                thing && (isObject(thing) || isFunction(thing)) && isFunction(thing.then) && isFunction(thing.catch)

              // original code
              // https://github.com/DigitalBrainJS/AxiosPromise/blob/16deab13710ec09779922131f3fa5954320f83ab/lib/utils.js#L11-L34

              const _setImmediate = ((setImmediateSupported, postMessageSupported) => {
                if (setImmediateSupported) {
                  return setImmediate
                }
                return postMessageSupported
                  ? ((token, callbacks) => {
                      _global.addEventListener(
                        "message",
                        ({ source, data }) => {
                          if (source === _global && data === token) {
                            callbacks.length && callbacks.shift()()
                          }
                        },
                        false,
                      )
                      return (cb) => {
                        callbacks.push(cb)
                        _global.postMessage(token, "*")
                      }
                    })(`axios@${Math.random()}`, [])
                  : (cb) => setTimeout(cb)
              })(typeof setImmediate === "function", isFunction(_global.postMessage))
              const asap =
                typeof queueMicrotask !== "undefined"
                  ? queueMicrotask.bind(_global)
                  : (typeof process !== "undefined" && process.nextTick) || _setImmediate

              // *********************

              const isIterable = (thing) => thing != null && isFunction(thing[iterator])
              var _default = (exports.default = {
                isArray,
                isArrayBuffer,
                isBuffer,
                isFormData,
                isArrayBufferView,
                isString,
                isNumber,
                isBoolean,
                isObject,
                isPlainObject,
                isEmptyObject,
                isReadableStream,
                isRequest,
                isResponse,
                isHeaders,
                isUndefined,
                isDate,
                isFile,
                isBlob,
                isRegExp,
                isFunction,
                isStream,
                isURLSearchParams,
                isTypedArray,
                isFileList,
                forEach,
                merge,
                extend,
                trim,
                stripBOM,
                inherits,
                toFlatObject,
                kindOf,
                kindOfTest,
                endsWith,
                toArray,
                forEachEntry,
                matchAll,
                isHTMLForm,
                hasOwnProperty,
                hasOwnProp: hasOwnProperty,
                // an alias to avoid ESLint no-prototype-builtins detection
                reduceDescriptors,
                freezeMethods,
                toObjectSet,
                toCamelCase,
                noop,
                toFiniteNumber,
                findKey,
                global: _global,
                isContextDefined,
                isSpecCompliantForm,
                toJSONObject,
                isAsyncFn,
                isThenable,
                setImmediate: _setImmediate,
                asap,
                isIterable,
              })
            }).call(this)
          }).call(
            this,
            require("_process"),
            typeof global !== "undefined"
              ? global
              : typeof self !== "undefined"
                ? self
                : typeof window !== "undefined"
                  ? window
                  : {},
            require("timers").setImmediate,
          )
        },
        { "./helpers/bind.js": 42, _process: 73, timers: 74 },
      ],
      70: [
        function (require, module, exports) {
          "use strict"

          exports.byteLength = byteLength
          exports.toByteArray = toByteArray
          exports.fromByteArray = fromByteArray

          var lookup = []
          var revLookup = []
          var Arr = typeof Uint8Array !== "undefined" ? Uint8Array : Array

          var code = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
          for (var i = 0, len = code.length; i < len; ++i) {
            lookup[i] = code[i]
            revLookup[code.charCodeAt(i)] = i
          }

          // Support decoding URL-safe base64 strings, as Node.js does.
          // See: https://en.wikipedia.org/wiki/Base64#URL_applications
          revLookup["-".charCodeAt(0)] = 62
          revLookup["_".charCodeAt(0)] = 63

          function getLens(b64) {
            var len = b64.length

            if (len % 4 > 0) {
              throw new Error("Invalid string. Length must be a multiple of 4")
            }

            // Trim off extra bytes after placeholder bytes are found
            // See: https://github.com/beatgammit/base64-js/issues/42
            var validLen = b64.indexOf("=")
            if (validLen === -1) validLen = len

            var placeHoldersLen = validLen === len ? 0 : 4 - (validLen % 4)

            return [validLen, placeHoldersLen]
          }

          // base64 is 4/3 + up to two characters of the original data
          function byteLength(b64) {
            var lens = getLens(b64)
            var validLen = lens[0]
            var placeHoldersLen = lens[1]
            return ((validLen + placeHoldersLen) * 3) / 4 - placeHoldersLen
          }

          function _byteLength(b64, validLen, placeHoldersLen) {
            return ((validLen + placeHoldersLen) * 3) / 4 - placeHoldersLen
          }

          function toByteArray(b64) {
            var tmp
            var lens = getLens(b64)
            var validLen = lens[0]
            var placeHoldersLen = lens[1]

            var arr = new Arr(_byteLength(b64, validLen, placeHoldersLen))

            var curByte = 0

            // if there are placeholders, only get up to the last complete 4 chars
            var len = placeHoldersLen > 0 ? validLen - 4 : validLen

            var i
            for (i = 0; i < len; i += 4) {
              tmp =
                (revLookup[b64.charCodeAt(i)] << 18) |
                (revLookup[b64.charCodeAt(i + 1)] << 12) |
                (revLookup[b64.charCodeAt(i + 2)] << 6) |
                revLookup[b64.charCodeAt(i + 3)]
              arr[curByte++] = (tmp >> 16) & 0xff
              arr[curByte++] = (tmp >> 8) & 0xff
              arr[curByte++] = tmp & 0xff
            }

            if (placeHoldersLen === 2) {
              tmp = (revLookup[b64.charCodeAt(i)] << 2) | (revLookup[b64.charCodeAt(i + 1)] >> 4)
              arr[curByte++] = tmp & 0xff
            }

            if (placeHoldersLen === 1) {
              tmp =
                (revLookup[b64.charCodeAt(i)] << 10) |
                (revLookup[b64.charCodeAt(i + 1)] << 4) |
                (revLookup[b64.charCodeAt(i + 2)] >> 2)
              arr[curByte++] = (tmp >> 8) & 0xff
              arr[curByte++] = tmp & 0xff
            }

            return arr
          }

          function tripletToBase64(num) {
            return (
              lookup[(num >> 18) & 0x3f] + lookup[(num >> 12) & 0x3f] + lookup[(num >> 6) & 0x3f] + lookup[num & 0x3f]
            )
          }

          function encodeChunk(uint8, start, end) {
            var tmp
            var output = []
            for (var i = start; i < end; i += 3) {
              tmp = ((uint8[i] << 16) & 0xff0000) + ((uint8[i + 1] << 8) & 0xff00) + (uint8[i + 2] & 0xff)
              output.push(tripletToBase64(tmp))
            }
            return output.join("")
          }

          function fromByteArray(uint8) {
            var tmp
            var len = uint8.length
            var extraBytes = len % 3 // if we have 1 byte left, pad 2 bytes
            var parts = []
            var maxChunkLength = 16383 // must be multiple of 3

            // go through the array every three bytes, we'll deal with trailing stuff later
            for (var i = 0, len2 = len - extraBytes; i < len2; i += maxChunkLength) {
              parts.push(encodeChunk(uint8, i, i + maxChunkLength > len2 ? len2 : i + maxChunkLength))
            }

            // pad the end with zeros, but make sure to not forget the extra bytes
            if (extraBytes === 1) {
              tmp = uint8[len - 1]
              parts.push(lookup[tmp >> 2] + lookup[(tmp << 4) & 0x3f] + "==")
            } else if (extraBytes === 2) {
              tmp = (uint8[len - 2] << 8) + uint8[len - 1]
              parts.push(lookup[tmp >> 10] + lookup[(tmp >> 4) & 0x3f] + lookup[(tmp << 2) & 0x3f] + "=")
            }

            return parts.join("")
          }
        },
        {},
      ],
      71: [
        function (require, module, exports) {
          ;(function (Buffer) {
            ;(function () {
              /*!
               * The buffer module from node.js, for the browser.
               *
               * @author   Feross Aboukhadijeh <https://feross.org>
               * @license  MIT
               */
              /* eslint-disable no-proto */

              "use strict"

              var base64 = require("base64-js")
              var ieee754 = require("ieee754")

              exports.Buffer = Buffer
              exports.SlowBuffer = SlowBuffer
              exports.INSPECT_MAX_BYTES = 50

              var K_MAX_LENGTH = 0x7fffffff
              exports.kMaxLength = K_MAX_LENGTH

              /**
               * If `Buffer.TYPED_ARRAY_SUPPORT`:
               *   === true    Use Uint8Array implementation (fastest)
               *   === false   Print warning and recommend using `buffer` v4.x which has an Object
               *               implementation (most compatible, even IE6)
               *
               * Browsers that support typed arrays are IE 10+, Firefox 4+, Chrome 7+, Safari 5.1+,
               * Opera 11.6+, iOS 4.2+.
               *
               * We report that the browser does not support typed arrays if the are not subclassable
               * using __proto__. Firefox 4-29 lacks support for adding new properties to `Uint8Array`
               * (See: https://bugzilla.mozilla.org/show_bug.cgi?id=695438). IE 10 lacks support
               * for __proto__ and has a buggy typed array implementation.
               */
              Buffer.TYPED_ARRAY_SUPPORT = typedArraySupport()

              if (
                !Buffer.TYPED_ARRAY_SUPPORT &&
                typeof console !== "undefined" &&
                typeof console.error === "function"
              ) {
                console.error(
                  "This browser lacks typed array (Uint8Array) support which is required by " +
                    "`buffer` v5.x. Use `buffer` v4.x if you require old browser support.",
                )
              }

              function typedArraySupport() {
                // Can typed array instances can be augmented?
                try {
                  var arr = new Uint8Array(1)
                  arr.__proto__ = {
                    __proto__: Uint8Array.prototype,
                    foo: function () {
                      return 42
                    },
                  }
                  return arr.foo() === 42
                } catch (e) {
                  return false
                }
              }

              Object.defineProperty(Buffer.prototype, "parent", {
                enumerable: true,
                get: function () {
                  if (!Buffer.isBuffer(this)) return undefined
                  return this.buffer
                },
              })

              Object.defineProperty(Buffer.prototype, "offset", {
                enumerable: true,
                get: function () {
                  if (!Buffer.isBuffer(this)) return undefined
                  return this.byteOffset
                },
              })

              function createBuffer(length) {
                if (length > K_MAX_LENGTH) {
                  throw new RangeError('The value "' + length + '" is invalid for option "size"')
                }
                // Return an augmented `Uint8Array` instance
                var buf = new Uint8Array(length)
                buf.__proto__ = Buffer.prototype
                return buf
              }

              /**
               * The Buffer constructor returns instances of `Uint8Array` that have their
               * prototype changed to `Buffer.prototype`. Furthermore, `Buffer` is a subclass of
               * `Uint8Array`, so the returned instances will have all the node `Buffer` methods
               * and the `Uint8Array` methods. Square bracket notation works as expected -- it
               * returns a single octet.
               *
               * The `Uint8Array` prototype remains unmodified.
               */

              function Buffer(arg, encodingOrOffset, length) {
                // Common case.
                if (typeof arg === "number") {
                  if (typeof encodingOrOffset === "string") {
                    throw new TypeError('The "string" argument must be of type string. Received type number')
                  }
                  return allocUnsafe(arg)
                }
                return from(arg, encodingOrOffset, length)
              }

              // Fix subarray() in ES2016. See: https://github.com/feross/buffer/pull/97
              if (typeof Symbol !== "undefined" && Symbol.species != null && Buffer[Symbol.species] === Buffer) {
                Object.defineProperty(Buffer, Symbol.species, {
                  value: null,
                  configurable: true,
                  enumerable: false,
                  writable: false,
                })
              }

              Buffer.poolSize = 8192 // not used by this implementation

              function from(value, encodingOrOffset, length) {
                if (typeof value === "string") {
                  return fromString(value, encodingOrOffset)
                }

                if (ArrayBuffer.isView(value)) {
                  return fromArrayLike(value)
                }

                if (value == null) {
                  throw TypeError(
                    "The first argument must be one of type string, Buffer, ArrayBuffer, Array, " +
                      "or Array-like Object. Received type " +
                      typeof value,
                  )
                }

                if (isInstance(value, ArrayBuffer) || (value && isInstance(value.buffer, ArrayBuffer))) {
                  return fromArrayBuffer(value, encodingOrOffset, length)
                }

                if (typeof value === "number") {
                  throw new TypeError('The "value" argument must not be of type number. Received type number')
                }

                var valueOf = value.valueOf && value.valueOf()
                if (valueOf != null && valueOf !== value) {
                  return Buffer.from(valueOf, encodingOrOffset, length)
                }

                var b = fromObject(value)
                if (b) return b

                if (
                  typeof Symbol !== "undefined" &&
                  Symbol.toPrimitive != null &&
                  typeof value[Symbol.toPrimitive] === "function"
                ) {
                  return Buffer.from(value[Symbol.toPrimitive]("string"), encodingOrOffset, length)
                }

                throw new TypeError(
                  "The first argument must be one of type string, Buffer, ArrayBuffer, Array, " +
                    "or Array-like Object. Received type " +
                    typeof value,
                )
              }

              /**
               * Functionally equivalent to Buffer(arg, encoding) but throws a TypeError
               * if value is a number.
               * Buffer.from(str[, encoding])
               * Buffer.from(array)
               * Buffer.from(buffer)
               * Buffer.from(arrayBuffer[, byteOffset[, length]])
               **/
              Buffer.from = function (value, encodingOrOffset, length) {
                return from(value, encodingOrOffset, length)
              }

              // Note: Change prototype *after* Buffer.from is defined to workaround Chrome bug:
              // https://github.com/feross/buffer/pull/148
              Buffer.prototype.__proto__ = Uint8Array.prototype
              Buffer.__proto__ = Uint8Array

              function assertSize(size) {
                if (typeof size !== "number") {
                  throw new TypeError('"size" argument must be of type number')
                } else if (size < 0) {
                  throw new RangeError('The value "' + size + '" is invalid for option "size"')
                }
              }

              function alloc(size, fill, encoding) {
                assertSize(size)
                if (size <= 0) {
                  return createBuffer(size)
                }
                if (fill !== undefined) {
                  // Only pay attention to encoding if it's a string. This
                  // prevents accidentally sending in a number that would
                  // be interpretted as a start offset.
                  return typeof encoding === "string"
                    ? createBuffer(size).fill(fill, encoding)
                    : createBuffer(size).fill(fill)
                }
                return createBuffer(size)
              }

              /**
               * Creates a new filled Buffer instance.
               * alloc(size[, fill[, encoding]])
               **/
              Buffer.alloc = function (size, fill, encoding) {
                return alloc(size, fill, encoding)
              }

              function allocUnsafe(size) {
                assertSize(size)
                return createBuffer(size < 0 ? 0 : checked(size) | 0)
              }

              /**
               * Equivalent to Buffer(num), by default creates a non-zero-filled Buffer instance.
               * */
              Buffer.allocUnsafe = function (size) {
                return allocUnsafe(size)
              }
              /**
               * Equivalent to SlowBuffer(num), by default creates a non-zero-filled Buffer instance.
               */
              Buffer.allocUnsafeSlow = function (size) {
                return allocUnsafe(size)
              }

              function fromString(string, encoding) {
                if (typeof encoding !== "string" || encoding === "") {
                  encoding = "utf8"
                }

                if (!Buffer.isEncoding(encoding)) {
                  throw new TypeError("Unknown encoding: " + encoding)
                }

                var length = byteLength(string, encoding) | 0
                var buf = createBuffer(length)

                var actual = buf.write(string, encoding)

                if (actual !== length) {
                  // Writing a hex string, for example, that contains invalid characters will
                  // cause everything after the first invalid character to be ignored. (e.g.
                  // 'abxxcd' will be treated as 'ab')
                  buf = buf.slice(0, actual)
                }

                return buf
              }

              function fromArrayLike(array) {
                var length = array.length < 0 ? 0 : checked(array.length) | 0
                var buf = createBuffer(length)
                for (var i = 0; i < length; i += 1) {
                  buf[i] = array[i] & 255
                }
                return buf
              }

              function fromArrayBuffer(array, byteOffset, length) {
                if (byteOffset < 0 || array.byteLength < byteOffset) {
                  throw new RangeError('"offset" is outside of buffer bounds')
                }

                if (array.byteLength < byteOffset + (length || 0)) {
                  throw new RangeError('"length" is outside of buffer bounds')
                }

                var buf
                if (byteOffset === undefined && length === undefined) {
                  buf = new Uint8Array(array)
                } else if (length === undefined) {
                  buf = new Uint8Array(array, byteOffset)
                } else {
                  buf = new Uint8Array(array, byteOffset, length)
                }

                // Return an augmented `Uint8Array` instance
                buf.__proto__ = Buffer.prototype
                return buf
              }

              function fromObject(obj) {
                if (Buffer.isBuffer(obj)) {
                  var len = checked(obj.length) | 0
                  var buf = createBuffer(len)

                  if (buf.length === 0) {
                    return buf
                  }

                  obj.copy(buf, 0, 0, len)
                  return buf
                }

                if (obj.length !== undefined) {
                  if (typeof obj.length !== "number" || numberIsNaN(obj.length)) {
                    return createBuffer(0)
                  }
                  return fromArrayLike(obj)
                }

                if (obj.type === "Buffer" && Array.isArray(obj.data)) {
                  return fromArrayLike(obj.data)
                }
              }

              function checked(length) {
                // Note: cannot use `length < K_MAX_LENGTH` here because that fails when
                // length is NaN (which is otherwise coerced to zero.)
                if (length >= K_MAX_LENGTH) {
                  throw new RangeError(
                    "Attempt to allocate Buffer larger than maximum " +
                      "size: 0x" +
                      K_MAX_LENGTH.toString(16) +
                      " bytes",
                  )
                }
                return length | 0
              }

              function SlowBuffer(length) {
                if (+length != length) {
                  // eslint-disable-line eqeqeq
                  length = 0
                }
                return Buffer.alloc(+length)
              }

              Buffer.isBuffer = function isBuffer(b) {
                return b != null && b._isBuffer === true && b !== Buffer.prototype // so Buffer.isBuffer(Buffer.prototype) will be false
              }

              Buffer.compare = function compare(a, b) {
                if (isInstance(a, Uint8Array)) a = Buffer.from(a, a.offset, a.byteLength)
                if (isInstance(b, Uint8Array)) b = Buffer.from(b, b.offset, b.byteLength)
                if (!Buffer.isBuffer(a) || !Buffer.isBuffer(b)) {
                  throw new TypeError('The "buf1", "buf2" arguments must be one of type Buffer or Uint8Array')
                }

                if (a === b) return 0

                var x = a.length
                var y = b.length

                for (var i = 0, len = Math.min(x, y); i < len; ++i) {
                  if (a[i] !== b[i]) {
                    x = a[i]
                    y = b[i]
                    break
                  }
                }

                if (x < y) return -1
                if (y < x) return 1
                return 0
              }

              Buffer.isEncoding = function isEncoding(encoding) {
                switch (String(encoding).toLowerCase()) {
                  case "hex":
                  case "utf8":
                  case "utf-8":
                  case "ascii":
                  case "latin1":
                  case "binary":
                  case "base64":
                  case "ucs2":
                  case "ucs-2":
                  case "utf16le":
                  case "utf-16le":
                    return true
                  default:
                    return false
                }
              }

              Buffer.concat = function concat(list, length) {
                if (!Array.isArray(list)) {
                  throw new TypeError('"list" argument must be an Array of Buffers')
                }

                if (list.length === 0) {
                  return Buffer.alloc(0)
                }

                var i
                if (length === undefined) {
                  length = 0
                  for (i = 0; i < list.length; ++i) {
                    length += list[i].length
                  }
                }

                var buffer = Buffer.allocUnsafe(length)
                var pos = 0
                for (i = 0; i < list.length; ++i) {
                  var buf = list[i]
                  if (isInstance(buf, Uint8Array)) {
                    buf = Buffer.from(buf)
                  }
                  if (!Buffer.isBuffer(buf)) {
                    throw new TypeError('"list" argument must be an Array of Buffers')
                  }
                  buf.copy(buffer, pos)
                  pos += buf.length
                }
                return buffer
              }

              function byteLength(string, encoding) {
                if (Buffer.isBuffer(string)) {
                  return string.length
                }
                if (ArrayBuffer.isView(string) || isInstance(string, ArrayBuffer)) {
                  return string.byteLength
                }
                if (typeof string !== "string") {
                  throw new TypeError(
                    'The "string" argument must be one of type string, Buffer, or ArrayBuffer. ' +
                      "Received type " +
                      typeof string,
                  )
                }

                var len = string.length
                var mustMatch = arguments.length > 2 && arguments[2] === true
                if (!mustMatch && len === 0) return 0

                // Use a for loop to avoid recursion
                var loweredCase = false
                for (;;) {
                  switch (encoding) {
                    case "ascii":
                    case "latin1":
                    case "binary":
                      return len
                    case "utf8":
                    case "utf-8":
                      return utf8ToBytes(string).length
                    case "ucs2":
                    case "ucs-2":
                    case "utf16le":
                    case "utf-16le":
                      return len * 2
                    case "hex":
                      return len >>> 1
                    case "base64":
                      return base64ToBytes(string).length
                    default:
                      if (loweredCase) {
                        return mustMatch ? -1 : utf8ToBytes(string).length // assume utf8
                      }
                      encoding = ("" + encoding).toLowerCase()
                      loweredCase = true
                  }
                }
              }
              Buffer.byteLength = byteLength

              function slowToString(encoding, start, end) {
                var loweredCase = false

                // No need to verify that "this.length <= MAX_UINT32" since it's a read-only
                // property of a typed array.

                // This behaves neither like String nor Uint8Array in that we set start/end
                // to their upper/lower bounds if the value passed is out of range.
                // undefined is handled specially as per ECMA-262 6th Edition,
                // Section 13.3.3.7 Runtime Semantics: KeyedBindingInitialization.
                if (start === undefined || start < 0) {
                  start = 0
                }
                // Return early if start > this.length. Done here to prevent potential uint32
                // coercion fail below.
                if (start > this.length) {
                  return ""
                }

                if (end === undefined || end > this.length) {
                  end = this.length
                }

                if (end <= 0) {
                  return ""
                }

                // Force coersion to uint32. This will also coerce falsey/NaN values to 0.
                end >>>= 0
                start >>>= 0

                if (end <= start) {
                  return ""
                }

                if (!encoding) encoding = "utf8"

                while (true) {
                  switch (encoding) {
                    case "hex":
                      return hexSlice(this, start, end)

                    case "utf8":
                    case "utf-8":
                      return utf8Slice(this, start, end)

                    case "ascii":
                      return asciiSlice(this, start, end)

                    case "latin1":
                    case "binary":
                      return latin1Slice(this, start, end)

                    case "base64":
                      return base64Slice(this, start, end)

                    case "ucs2":
                    case "ucs-2":
                    case "utf16le":
                    case "utf-16le":
                      return utf16leSlice(this, start, end)

                    default:
                      if (loweredCase) throw new TypeError("Unknown encoding: " + encoding)
                      encoding = (encoding + "").toLowerCase()
                      loweredCase = true
                  }
                }
              }

              // This property is used by `Buffer.isBuffer` (and the `is-buffer` npm package)
              // to detect a Buffer instance. It's not possible to use `instanceof Buffer`
              // reliably in a browserify context because there could be multiple different
              // copies of the 'buffer' package in use. This method works even for Buffer
              // instances that were created from another copy of the `buffer` package.
              // See: https://github.com/feross/buffer/issues/154
              Buffer.prototype._isBuffer = true

              function swap(b, n, m) {
                var i = b[n]
                b[n] = b[m]
                b[m] = i
              }

              Buffer.prototype.swap16 = function swap16() {
                var len = this.length
                if (len % 2 !== 0) {
                  throw new RangeError("Buffer size must be a multiple of 16-bits")
                }
                for (var i = 0; i < len; i += 2) {
                  swap(this, i, i + 1)
                }
                return this
              }

              Buffer.prototype.swap32 = function swap32() {
                var len = this.length
                if (len % 4 !== 0) {
                  throw new RangeError("Buffer size must be a multiple of 32-bits")
                }
                for (var i = 0; i < len; i += 4) {
                  swap(this, i, i + 3)
                  swap(this, i + 1, i + 2)
                }
                return this
              }

              Buffer.prototype.swap64 = function swap64() {
                var len = this.length
                if (len % 8 !== 0) {
                  throw new RangeError("Buffer size must be a multiple of 64-bits")
                }
                for (var i = 0; i < len; i += 8) {
                  swap(this, i, i + 7)
                  swap(this, i + 1, i + 6)
                  swap(this, i + 2, i + 5)
                  swap(this, i + 3, i + 4)
                }
                return this
              }

              Buffer.prototype.toString = function toString() {
                var length = this.length
                if (length === 0) return ""
                if (arguments.length === 0) return utf8Slice(this, 0, length)
                return slowToString.apply(this, arguments)
              }

              Buffer.prototype.toLocaleString = Buffer.prototype.toString

              Buffer.prototype.equals = function equals(b) {
                if (!Buffer.isBuffer(b)) throw new TypeError("Argument must be a Buffer")
                if (this === b) return true
                return Buffer.compare(this, b) === 0
              }

              Buffer.prototype.inspect = function inspect() {
                var str = ""
                var max = exports.INSPECT_MAX_BYTES
                str = this.toString("hex", 0, max)
                  .replace(/(.{2})/g, "$1 ")
                  .trim()
                if (this.length > max) str += " ... "
                return "<Buffer " + str + ">"
              }

              Buffer.prototype.compare = function compare(target, start, end, thisStart, thisEnd) {
                if (isInstance(target, Uint8Array)) {
                  target = Buffer.from(target, target.offset, target.byteLength)
                }
                if (!Buffer.isBuffer(target)) {
                  throw new TypeError(
                    'The "target" argument must be one of type Buffer or Uint8Array. ' +
                      "Received type " +
                      typeof target,
                  )
                }

                if (start === undefined) {
                  start = 0
                }
                if (end === undefined) {
                  end = target ? target.length : 0
                }
                if (thisStart === undefined) {
                  thisStart = 0
                }
                if (thisEnd === undefined) {
                  thisEnd = this.length
                }

                if (start < 0 || end > target.length || thisStart < 0 || thisEnd > this.length) {
                  throw new RangeError("out of range index")
                }

                if (thisStart >= thisEnd && start >= end) {
                  return 0
                }
                if (thisStart >= thisEnd) {
                  return -1
                }
                if (start >= end) {
                  return 1
                }

                start >>>= 0
                end >>>= 0
                thisStart >>>= 0
                thisEnd >>>= 0

                if (this === target) return 0

                var x = thisEnd - thisStart
                var y = end - start
                var len = Math.min(x, y)

                var thisCopy = this.slice(thisStart, thisEnd)
                var targetCopy = target.slice(start, end)

                for (var i = 0; i < len; ++i) {
                  if (thisCopy[i] !== targetCopy[i]) {
                    x = thisCopy[i]
                    y = targetCopy[i]
                    break
                  }
                }

                if (x < y) return -1
                if (y < x) return 1
                return 0
              }

              // Finds either the first index of `val` in `buffer` at offset >= `byteOffset`,
              // OR the last index of `val` in `buffer` at offset <= `byteOffset`.
              //
              // Arguments:
              // - buffer - a Buffer to search
              // - val - a string, Buffer, or number
              // - byteOffset - an index into `buffer`; will be clamped to an int32
              // - encoding - an optional encoding, relevant is val is a string
              // - dir - true for indexOf, false for lastIndexOf
              function bidirectionalIndexOf(buffer, val, byteOffset, encoding, dir) {
                // Empty buffer means no match
                if (buffer.length === 0) return -1

                // Normalize byteOffset
                if (typeof byteOffset === "string") {
                  encoding = byteOffset
                  byteOffset = 0
                } else if (byteOffset > 0x7fffffff) {
                  byteOffset = 0x7fffffff
                } else if (byteOffset < -0x80000000) {
                  byteOffset = -0x80000000
                }
                byteOffset = +byteOffset // Coerce to Number.
                if (numberIsNaN(byteOffset)) {
                  // byteOffset: it it's undefined, null, NaN, "foo", etc, search whole buffer
                  byteOffset = dir ? 0 : buffer.length - 1
                }

                // Normalize byteOffset: negative offsets start from the end of the buffer
                if (byteOffset < 0) byteOffset = buffer.length + byteOffset
                if (byteOffset >= buffer.length) {
                  if (dir) return -1
                  else byteOffset = buffer.length - 1
                } else if (byteOffset < 0) {
                  if (dir) byteOffset = 0
                  else return -1
                }

                // Normalize val
                if (typeof val === "string") {
                  val = Buffer.from(val, encoding)
                }

                // Finally, search either indexOf (if dir is true) or lastIndexOf
                if (Buffer.isBuffer(val)) {
                  // Special case: looking for empty string/buffer always fails
                  if (val.length === 0) {
                    return -1
                  }
                  return arrayIndexOf(buffer, val, byteOffset, encoding, dir)
                } else if (typeof val === "number") {
                  val = val & 0xff // Search for a byte value [0-255]
                  if (typeof Uint8Array.prototype.indexOf === "function") {
                    if (dir) {
                      return Uint8Array.prototype.indexOf.call(buffer, val, byteOffset)
                    } else {
                      return Uint8Array.prototype.lastIndexOf.call(buffer, val, byteOffset)
                    }
                  }
                  return arrayIndexOf(buffer, [val], byteOffset, encoding, dir)
                }

                throw new TypeError("val must be string, number or Buffer")
              }

              function arrayIndexOf(arr, val, byteOffset, encoding, dir) {
                var indexSize = 1
                var arrLength = arr.length
                var valLength = val.length

                if (encoding !== undefined) {
                  encoding = String(encoding).toLowerCase()
                  if (
                    encoding === "ucs2" ||
                    encoding === "ucs-2" ||
                    encoding === "utf16le" ||
                    encoding === "utf-16le"
                  ) {
                    if (arr.length < 2 || val.length < 2) {
                      return -1
                    }
                    indexSize = 2
                    arrLength /= 2
                    valLength /= 2
                    byteOffset /= 2
                  }
                }

                function read(buf, i) {
                  if (indexSize === 1) {
                    return buf[i]
                  } else {
                    return buf.readUInt16BE(i * indexSize)
                  }
                }

                var i
                if (dir) {
                  var foundIndex = -1
                  for (i = byteOffset; i < arrLength; i++) {
                    if (read(arr, i) === read(val, foundIndex === -1 ? 0 : i - foundIndex)) {
                      if (foundIndex === -1) foundIndex = i
                      if (i - foundIndex + 1 === valLength) return foundIndex * indexSize
                    } else {
                      if (foundIndex !== -1) i -= i - foundIndex
                      foundIndex = -1
                    }
                  }
                } else {
                  if (byteOffset + valLength > arrLength) byteOffset = arrLength - valLength
                  for (i = byteOffset; i >= 0; i--) {
                    var found = true
                    for (var j = 0; j < valLength; j++) {
                      if (read(arr, i + j) !== read(val, j)) {
                        found = false
                        break
                      }
                    }
                    if (found) return i
                  }
                }

                return -1
              }

              Buffer.prototype.includes = function includes(val, byteOffset, encoding) {
                return this.indexOf(val, byteOffset, encoding) !== -1
              }

              Buffer.prototype.indexOf = function indexOf(val, byteOffset, encoding) {
                return bidirectionalIndexOf(this, val, byteOffset, encoding, true)
              }

              Buffer.prototype.lastIndexOf = function lastIndexOf(val, byteOffset, encoding) {
                return bidirectionalIndexOf(this, val, byteOffset, encoding, false)
              }

              function hexWrite(buf, string, offset, length) {
                offset = Number(offset) || 0
                var remaining = buf.length - offset
                if (!length) {
                  length = remaining
                } else {
                  length = Number(length)
                  if (length > remaining) {
                    length = remaining
                  }
                }

                var strLen = string.length

                if (length > strLen / 2) {
                  length = strLen / 2
                }
                for (var i = 0; i < length; ++i) {
                  var parsed = parseInt(string.substr(i * 2, 2), 16)
                  if (numberIsNaN(parsed)) return i
                  buf[offset + i] = parsed
                }
                return i
              }

              function utf8Write(buf, string, offset, length) {
                return blitBuffer(utf8ToBytes(string, buf.length - offset), buf, offset, length)
              }

              function asciiWrite(buf, string, offset, length) {
                return blitBuffer(asciiToBytes(string), buf, offset, length)
              }

              function latin1Write(buf, string, offset, length) {
                return asciiWrite(buf, string, offset, length)
              }

              function base64Write(buf, string, offset, length) {
                return blitBuffer(base64ToBytes(string), buf, offset, length)
              }

              function ucs2Write(buf, string, offset, length) {
                return blitBuffer(utf16leToBytes(string, buf.length - offset), buf, offset, length)
              }

              Buffer.prototype.write = function write(string, offset, length, encoding) {
                // Buffer#write(string)
                if (offset === undefined) {
                  encoding = "utf8"
                  length = this.length
                  offset = 0
                  // Buffer#write(string, encoding)
                } else if (length === undefined && typeof offset === "string") {
                  encoding = offset
                  length = this.length
                  offset = 0
                  // Buffer#write(string, offset[, length][, encoding])
                } else if (isFinite(offset)) {
                  offset = offset >>> 0
                  if (isFinite(length)) {
                    length = length >>> 0
                    if (encoding === undefined) encoding = "utf8"
                  } else {
                    encoding = length
                    length = undefined
                  }
                } else {
                  throw new Error("Buffer.write(string, encoding, offset[, length]) is no longer supported")
                }

                var remaining = this.length - offset
                if (length === undefined || length > remaining) length = remaining

                if ((string.length > 0 && (length < 0 || offset < 0)) || offset > this.length) {
                  throw new RangeError("Attempt to write outside buffer bounds")
                }

                if (!encoding) encoding = "utf8"

                var loweredCase = false
                for (;;) {
                  switch (encoding) {
                    case "hex":
                      return hexWrite(this, string, offset, length)

                    case "utf8":
                    case "utf-8":
                      return utf8Write(this, string, offset, length)

                    case "ascii":
                      return asciiWrite(this, string, offset, length)

                    case "latin1":
                    case "binary":
                      return latin1Write(this, string, offset, length)

                    case "base64":
                      // Warning: maxLength not taken into account in base64Write
                      return base64Write(this, string, offset, length)

                    case "ucs2":
                    case "ucs-2":
                    case "utf16le":
                    case "utf-16le":
                      return ucs2Write(this, string, offset, length)

                    default:
                      if (loweredCase) throw new TypeError("Unknown encoding: " + encoding)
                      encoding = ("" + encoding).toLowerCase()
                      loweredCase = true
                  }
                }
              }

              Buffer.prototype.toJSON = function toJSON() {
                return {
                  type: "Buffer",
                  data: Array.prototype.slice.call(this._arr || this, 0),
                }
              }

              function base64Slice(buf, start, end) {
                if (start === 0 && end === buf.length) {
                  return base64.fromByteArray(buf)
                } else {
                  return base64.fromByteArray(buf.slice(start, end))
                }
              }

              function utf8Slice(buf, start, end) {
                end = Math.min(buf.length, end)
                var res = []

                var i = start
                while (i < end) {
                  var firstByte = buf[i]
                  var codePoint = null
                  var bytesPerSequence = firstByte > 0xef ? 4 : firstByte > 0xdf ? 3 : firstByte > 0xbf ? 2 : 1

                  if (i + bytesPerSequence <= end) {
                    var secondByte, thirdByte, fourthByte, tempCodePoint

                    switch (bytesPerSequence) {
                      case 1:
                        if (firstByte < 0x80) {
                          codePoint = firstByte
                        }
                        break
                      case 2:
                        secondByte = buf[i + 1]
                        if ((secondByte & 0xc0) === 0x80) {
                          tempCodePoint = ((firstByte & 0x1f) << 0x6) | (secondByte & 0x3f)
                          if (tempCodePoint > 0x7f) {
                            codePoint = tempCodePoint
                          }
                        }
                        break
                      case 3:
                        secondByte = buf[i + 1]
                        thirdByte = buf[i + 2]
                        if ((secondByte & 0xc0) === 0x80 && (thirdByte & 0xc0) === 0x80) {
                          tempCodePoint = ((firstByte & 0xf) << 0xc) | ((secondByte & 0x3f) << 0x6) | (thirdByte & 0x3f)
                          if (tempCodePoint > 0x7ff && (tempCodePoint < 0xd800 || tempCodePoint > 0xdfff)) {
                            codePoint = tempCodePoint
                          }
                        }
                        break
                      case 4:
                        secondByte = buf[i + 1]
                        thirdByte = buf[i + 2]
                        fourthByte = buf[i + 3]
                        if (
                          (secondByte & 0xc0) === 0x80 &&
                          (thirdByte & 0xc0) === 0x80 &&
                          (fourthByte & 0xc0) === 0x80
                        ) {
                          tempCodePoint =
                            ((firstByte & 0xf) << 0x12) |
                            ((secondByte & 0x3f) << 0xc) |
                            ((thirdByte & 0x3f) << 0x6) |
                            (fourthByte & 0x3f)
                          if (tempCodePoint > 0xffff && tempCodePoint < 0x110000) {
                            codePoint = tempCodePoint
                          }
                        }
                    }
                  }

                  if (codePoint === null) {
                    // we did not generate a valid codePoint so insert a
                    // replacement char (U+FFFD) and advance only 1 byte
                    codePoint = 0xfffd
                    bytesPerSequence = 1
                  } else if (codePoint > 0xffff) {
                    // encode to utf16 (surrogate pair dance)
                    codePoint -= 0x10000
                    res.push(((codePoint >>> 10) & 0x3ff) | 0xd800)
                    codePoint = 0xdc00 | (codePoint & 0x3ff)
                  }

                  res.push(codePoint)
                  i += bytesPerSequence
                }

                return decodeCodePointsArray(res)
              }

              // Based on http://stackoverflow.com/a/22747272/680742, the browser with
              // the lowest limit is Chrome, with 0x10000 args.
              // We go 1 magnitude less, for safety
              var MAX_ARGUMENTS_LENGTH = 0x1000

              function decodeCodePointsArray(codePoints) {
                var len = codePoints.length
                if (len <= MAX_ARGUMENTS_LENGTH) {
                  return String.fromCharCode.apply(String, codePoints) // avoid extra slice()
                }

                // Decode in chunks to avoid "call stack size exceeded".
                var res = ""
                var i = 0
                while (i < len) {
                  res += String.fromCharCode.apply(String, codePoints.slice(i, (i += MAX_ARGUMENTS_LENGTH)))
                }
                return res
              }

              function asciiSlice(buf, start, end) {
                var ret = ""
                end = Math.min(buf.length, end)

                for (var i = start; i < end; ++i) {
                  ret += String.fromCharCode(buf[i] & 0x7f)
                }
                return ret
              }

              function latin1Slice(buf, start, end) {
                var ret = ""
                end = Math.min(buf.length, end)

                for (var i = start; i < end; ++i) {
                  ret += String.fromCharCode(buf[i])
                }
                return ret
              }

              function hexSlice(buf, start, end) {
                var len = buf.length

                if (!start || start < 0) start = 0
                if (!end || end < 0 || end > len) end = len

                var out = ""
                for (var i = start; i < end; ++i) {
                  out += toHex(buf[i])
                }
                return out
              }

              function utf16leSlice(buf, start, end) {
                var bytes = buf.slice(start, end)
                var res = ""
                for (var i = 0; i < bytes.length; i += 2) {
                  res += String.fromCharCode(bytes[i] + bytes[i + 1] * 256)
                }
                return res
              }

              Buffer.prototype.slice = function slice(start, end) {
                var len = this.length
                start = ~~start
                end = end === undefined ? len : ~~end

                if (start < 0) {
                  start += len
                  if (start < 0) start = 0
                } else if (start > len) {
                  start = len
                }

                if (end < 0) {
                  end += len
                  if (end < 0) end = 0
                } else if (end > len) {
                  end = len
                }

                if (end < start) end = start

                var newBuf = this.subarray(start, end)
                // Return an augmented `Uint8Array` instance
                newBuf.__proto__ = Buffer.prototype
                return newBuf
              }

              /*
               * Need to make sure that buffer isn't trying to write out of bounds.
               */
              function checkOffset(offset, ext, length) {
                if (offset % 1 !== 0 || offset < 0) throw new RangeError("offset is not uint")
                if (offset + ext > length) throw new RangeError("Trying to access beyond buffer length")
              }

              Buffer.prototype.readUIntLE = function readUIntLE(offset, byteLength, noAssert) {
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) checkOffset(offset, byteLength, this.length)

                var val = this[offset]
                var mul = 1
                var i = 0
                while (++i < byteLength && (mul *= 0x100)) {
                  val += this[offset + i] * mul
                }

                return val
              }

              Buffer.prototype.readUIntBE = function readUIntBE(offset, byteLength, noAssert) {
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) {
                  checkOffset(offset, byteLength, this.length)
                }

                var val = this[offset + --byteLength]
                var mul = 1
                while (byteLength > 0 && (mul *= 0x100)) {
                  val += this[offset + --byteLength] * mul
                }

                return val
              }

              Buffer.prototype.readUInt8 = function readUInt8(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 1, this.length)
                return this[offset]
              }

              Buffer.prototype.readUInt16LE = function readUInt16LE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 2, this.length)
                return this[offset] | (this[offset + 1] << 8)
              }

              Buffer.prototype.readUInt16BE = function readUInt16BE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 2, this.length)
                return (this[offset] << 8) | this[offset + 1]
              }

              Buffer.prototype.readUInt32LE = function readUInt32LE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)

                return (
                  (this[offset] | (this[offset + 1] << 8) | (this[offset + 2] << 16)) + this[offset + 3] * 0x1000000
                )
              }

              Buffer.prototype.readUInt32BE = function readUInt32BE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)

                return (
                  this[offset] * 0x1000000 + ((this[offset + 1] << 16) | (this[offset + 2] << 8) | this[offset + 3])
                )
              }

              Buffer.prototype.readIntLE = function readIntLE(offset, byteLength, noAssert) {
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) checkOffset(offset, byteLength, this.length)

                var val = this[offset]
                var mul = 1
                var i = 0
                while (++i < byteLength && (mul *= 0x100)) {
                  val += this[offset + i] * mul
                }
                mul *= 0x80

                if (val >= mul) val -= Math.pow(2, 8 * byteLength)

                return val
              }

              Buffer.prototype.readIntBE = function readIntBE(offset, byteLength, noAssert) {
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) checkOffset(offset, byteLength, this.length)

                var i = byteLength
                var mul = 1
                var val = this[offset + --i]
                while (i > 0 && (mul *= 0x100)) {
                  val += this[offset + --i] * mul
                }
                mul *= 0x80

                if (val >= mul) val -= Math.pow(2, 8 * byteLength)

                return val
              }

              Buffer.prototype.readInt8 = function readInt8(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 1, this.length)
                if (!(this[offset] & 0x80)) return this[offset]
                return (0xff - this[offset] + 1) * -1
              }

              Buffer.prototype.readInt16LE = function readInt16LE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 2, this.length)
                var val = this[offset] | (this[offset + 1] << 8)
                return val & 0x8000 ? val | 0xffff0000 : val
              }

              Buffer.prototype.readInt16BE = function readInt16BE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 2, this.length)
                var val = this[offset + 1] | (this[offset] << 8)
                return val & 0x8000 ? val | 0xffff0000 : val
              }

              Buffer.prototype.readInt32LE = function readInt32LE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)

                return this[offset] | (this[offset + 1] << 8) | (this[offset + 2] << 16) | (this[offset + 3] << 24)
              }

              Buffer.prototype.readInt32BE = function readInt32BE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)

                return (this[offset] << 24) | (this[offset + 1] << 16) | (this[offset + 2] << 8) | this[offset + 3]
              }

              Buffer.prototype.readFloatLE = function readFloatLE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)
                return ieee754.read(this, offset, true, 23, 4)
              }

              Buffer.prototype.readFloatBE = function readFloatBE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 4, this.length)
                return ieee754.read(this, offset, false, 23, 4)
              }

              Buffer.prototype.readDoubleLE = function readDoubleLE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 8, this.length)
                return ieee754.read(this, offset, true, 52, 8)
              }

              Buffer.prototype.readDoubleBE = function readDoubleBE(offset, noAssert) {
                offset = offset >>> 0
                if (!noAssert) checkOffset(offset, 8, this.length)
                return ieee754.read(this, offset, false, 52, 8)
              }

              function checkInt(buf, value, offset, ext, max, min) {
                if (!Buffer.isBuffer(buf)) throw new TypeError('"buffer" argument must be a Buffer instance')
                if (value > max || value < min) throw new RangeError('"value" argument is out of bounds')
                if (offset + ext > buf.length) throw new RangeError("Index out of range")
              }

              Buffer.prototype.writeUIntLE = function writeUIntLE(value, offset, byteLength, noAssert) {
                value = +value
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) {
                  var maxBytes = Math.pow(2, 8 * byteLength) - 1
                  checkInt(this, value, offset, byteLength, maxBytes, 0)
                }

                var mul = 1
                var i = 0
                this[offset] = value & 0xff
                while (++i < byteLength && (mul *= 0x100)) {
                  this[offset + i] = (value / mul) & 0xff
                }

                return offset + byteLength
              }

              Buffer.prototype.writeUIntBE = function writeUIntBE(value, offset, byteLength, noAssert) {
                value = +value
                offset = offset >>> 0
                byteLength = byteLength >>> 0
                if (!noAssert) {
                  var maxBytes = Math.pow(2, 8 * byteLength) - 1
                  checkInt(this, value, offset, byteLength, maxBytes, 0)
                }

                var i = byteLength - 1
                var mul = 1
                this[offset + i] = value & 0xff
                while (--i >= 0 && (mul *= 0x100)) {
                  this[offset + i] = (value / mul) & 0xff
                }

                return offset + byteLength
              }

              Buffer.prototype.writeUInt8 = function writeUInt8(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 1, 0xff, 0)
                this[offset] = value & 0xff
                return offset + 1
              }

              Buffer.prototype.writeUInt16LE = function writeUInt16LE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 2, 0xffff, 0)
                this[offset] = value & 0xff
                this[offset + 1] = value >>> 8
                return offset + 2
              }

              Buffer.prototype.writeUInt16BE = function writeUInt16BE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 2, 0xffff, 0)
                this[offset] = value >>> 8
                this[offset + 1] = value & 0xff
                return offset + 2
              }

              Buffer.prototype.writeUInt32LE = function writeUInt32LE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 4, 0xffffffff, 0)
                this[offset + 3] = value >>> 24
                this[offset + 2] = value >>> 16
                this[offset + 1] = value >>> 8
                this[offset] = value & 0xff
                return offset + 4
              }

              Buffer.prototype.writeUInt32BE = function writeUInt32BE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 4, 0xffffffff, 0)
                this[offset] = value >>> 24
                this[offset + 1] = value >>> 16
                this[offset + 2] = value >>> 8
                this[offset + 3] = value & 0xff
                return offset + 4
              }

              Buffer.prototype.writeIntLE = function writeIntLE(value, offset, byteLength, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) {
                  var limit = Math.pow(2, 8 * byteLength - 1)

                  checkInt(this, value, offset, byteLength, limit - 1, -limit)
                }

                var i = 0
                var mul = 1
                var sub = 0
                this[offset] = value & 0xff
                while (++i < byteLength && (mul *= 0x100)) {
                  if (value < 0 && sub === 0 && this[offset + i - 1] !== 0) {
                    sub = 1
                  }
                  this[offset + i] = (((value / mul) >> 0) - sub) & 0xff
                }

                return offset + byteLength
              }

              Buffer.prototype.writeIntBE = function writeIntBE(value, offset, byteLength, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) {
                  var limit = Math.pow(2, 8 * byteLength - 1)

                  checkInt(this, value, offset, byteLength, limit - 1, -limit)
                }

                var i = byteLength - 1
                var mul = 1
                var sub = 0
                this[offset + i] = value & 0xff
                while (--i >= 0 && (mul *= 0x100)) {
                  if (value < 0 && sub === 0 && this[offset + i + 1] !== 0) {
                    sub = 1
                  }
                  this[offset + i] = (((value / mul) >> 0) - sub) & 0xff
                }

                return offset + byteLength
              }

              Buffer.prototype.writeInt8 = function writeInt8(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 1, 0x7f, -0x80)
                if (value < 0) value = 0xff + value + 1
                this[offset] = value & 0xff
                return offset + 1
              }

              Buffer.prototype.writeInt16LE = function writeInt16LE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 2, 0x7fff, -0x8000)
                this[offset] = value & 0xff
                this[offset + 1] = value >>> 8
                return offset + 2
              }

              Buffer.prototype.writeInt16BE = function writeInt16BE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 2, 0x7fff, -0x8000)
                this[offset] = value >>> 8
                this[offset + 1] = value & 0xff
                return offset + 2
              }

              Buffer.prototype.writeInt32LE = function writeInt32LE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 4, 0x7fffffff, -0x80000000)
                this[offset] = value & 0xff
                this[offset + 1] = value >>> 8
                this[offset + 2] = value >>> 16
                this[offset + 3] = value >>> 24
                return offset + 4
              }

              Buffer.prototype.writeInt32BE = function writeInt32BE(value, offset, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) checkInt(this, value, offset, 4, 0x7fffffff, -0x80000000)
                if (value < 0) value = 0xffffffff + value + 1
                this[offset] = value >>> 24
                this[offset + 1] = value >>> 16
                this[offset + 2] = value >>> 8
                this[offset + 3] = value & 0xff
                return offset + 4
              }

              function checkIEEE754(buf, value, offset, ext, max, min) {
                if (offset + ext > buf.length) throw new RangeError("Index out of range")
                if (offset < 0) throw new RangeError("Index out of range")
              }

              function writeFloat(buf, value, offset, littleEndian, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) {
                  checkIEEE754(buf, value, offset, 4, 3.4028234663852886e38, -3.4028234663852886e38)
                }
                ieee754.write(buf, value, offset, littleEndian, 23, 4)
                return offset + 4
              }

              Buffer.prototype.writeFloatLE = function writeFloatLE(value, offset, noAssert) {
                return writeFloat(this, value, offset, true, noAssert)
              }

              Buffer.prototype.writeFloatBE = function writeFloatBE(value, offset, noAssert) {
                return writeFloat(this, value, offset, false, noAssert)
              }

              function writeDouble(buf, value, offset, littleEndian, noAssert) {
                value = +value
                offset = offset >>> 0
                if (!noAssert) {
                  checkIEEE754(buf, value, offset, 8, 1.7976931348623157e308, -1.7976931348623157e308)
                }
                ieee754.write(buf, value, offset, littleEndian, 52, 8)
                return offset + 8
              }

              Buffer.prototype.writeDoubleLE = function writeDoubleLE(value, offset, noAssert) {
                return writeDouble(this, value, offset, true, noAssert)
              }

              Buffer.prototype.writeDoubleBE = function writeDoubleBE(value, offset, noAssert) {
                return writeDouble(this, value, offset, false, noAssert)
              }

              // copy(targetBuffer, targetStart=0, sourceStart=0, sourceEnd=buffer.length)
              Buffer.prototype.copy = function copy(target, targetStart, start, end) {
                if (!Buffer.isBuffer(target)) throw new TypeError("argument should be a Buffer")
                if (!start) start = 0
                if (!end && end !== 0) end = this.length
                if (targetStart >= target.length) targetStart = target.length
                if (!targetStart) targetStart = 0
                if (end > 0 && end < start) end = start

                // Copy 0 bytes; we're done
                if (end === start) return 0
                if (target.length === 0 || this.length === 0) return 0

                // Fatal error conditions
                if (targetStart < 0) {
                  throw new RangeError("targetStart out of bounds")
                }
                if (start < 0 || start >= this.length) throw new RangeError("Index out of range")
                if (end < 0) throw new RangeError("sourceEnd out of bounds")

                // Are we oob?
                if (end > this.length) end = this.length
                if (target.length - targetStart < end - start) {
                  end = target.length - targetStart + start
                }

                var len = end - start

                if (this === target && typeof Uint8Array.prototype.copyWithin === "function") {
                  // Use built-in when available, missing from IE11
                  this.copyWithin(targetStart, start, end)
                } else if (this === target && start < targetStart && targetStart < end) {
                  // descending copy from end
                  for (var i = len - 1; i >= 0; --i) {
                    target[i + targetStart] = this[i + start]
                  }
                } else {
                  Uint8Array.prototype.set.call(target, this.subarray(start, end), targetStart)
                }

                return len
              }

              // Usage:
              //    buffer.fill(number[, offset[, end]])
              //    buffer.fill(buffer[, offset[, end]])
              //    buffer.fill(string[, offset[, end]][, encoding])
              Buffer.prototype.fill = function fill(val, start, end, encoding) {
                // Handle string cases:
                if (typeof val === "string") {
                  if (typeof start === "string") {
                    encoding = start
                    start = 0
                    end = this.length
                  } else if (typeof end === "string") {
                    encoding = end
                    end = this.length
                  }
                  if (encoding !== undefined && typeof encoding !== "string") {
                    throw new TypeError("encoding must be a string")
                  }
                  if (typeof encoding === "string" && !Buffer.isEncoding(encoding)) {
                    throw new TypeError("Unknown encoding: " + encoding)
                  }
                  if (val.length === 1) {
                    var code = val.charCodeAt(0)
                    if ((encoding === "utf8" && code < 128) || encoding === "latin1") {
                      // Fast path: If `val` fits into a single byte, use that numeric value.
                      val = code
                    }
                  }
                } else if (typeof val === "number") {
                  val = val & 255
                }

                // Invalid ranges are not set to a default, so can range check early.
                if (start < 0 || this.length < start || this.length < end) {
                  throw new RangeError("Out of range index")
                }

                if (end <= start) {
                  return this
                }

                start = start >>> 0
                end = end === undefined ? this.length : end >>> 0

                if (!val) val = 0

                var i
                if (typeof val === "number") {
                  for (i = start; i < end; ++i) {
                    this[i] = val
                  }
                } else {
                  var bytes = Buffer.isBuffer(val) ? val : Buffer.from(val, encoding)
                  var len = bytes.length
                  if (len === 0) {
                    throw new TypeError('The value "' + val + '" is invalid for argument "value"')
                  }
                  for (i = 0; i < end - start; ++i) {
                    this[i + start] = bytes[i % len]
                  }
                }

                return this
              }

              // HELPER FUNCTIONS
              // ================

              var INVALID_BASE64_RE = /[^+/0-9A-Za-z-_]/g

              function base64clean(str) {
                // Node takes equal signs as end of the Base64 encoding
                str = str.split("=")[0]
                // Node strips out invalid characters like \n and \t from the string, base64-js does not
                str = str.trim().replace(INVALID_BASE64_RE, "")
                // Node converts strings with length < 2 to ''
                if (str.length < 2) return ""
                // Node allows for non-padded base64 strings (missing trailing ===), base64-js does not
                while (str.length % 4 !== 0) {
                  str = str + "="
                }
                return str
              }

              function toHex(n) {
                if (n < 16) return "0" + n.toString(16)
                return n.toString(16)
              }

              function utf8ToBytes(string, units) {
                units = units || Infinity
                var codePoint
                var length = string.length
                var leadSurrogate = null
                var bytes = []

                for (var i = 0; i < length; ++i) {
                  codePoint = string.charCodeAt(i)

                  // is surrogate component
                  if (codePoint > 0xd7ff && codePoint < 0xe000) {
                    // last char was a lead
                    if (!leadSurrogate) {
                      // no lead yet
                      if (codePoint > 0xdbff) {
                        // unexpected trail
                        if ((units -= 3) > -1) bytes.push(0xef, 0xbf, 0xbd)
                        continue
                      } else if (i + 1 === length) {
                        // unpaired lead
                        if ((units -= 3) > -1) bytes.push(0xef, 0xbf, 0xbd)
                        continue
                      }

                      // valid lead
                      leadSurrogate = codePoint

                      continue
                    }

                    // 2 leads in a row
                    if (codePoint < 0xdc00) {
                      if ((units -= 3) > -1) bytes.push(0xef, 0xbf, 0xbd)
                      leadSurrogate = codePoint
                      continue
                    }

                    // valid surrogate pair
                    codePoint = (((leadSurrogate - 0xd800) << 10) | (codePoint - 0xdc00)) + 0x10000
                  } else if (leadSurrogate) {
                    // valid bmp char, but last char was a lead
                    if ((units -= 3) > -1) bytes.push(0xef, 0xbf, 0xbd)
                  }

                  leadSurrogate = null

                  // encode utf8
                  if (codePoint < 0x80) {
                    if ((units -= 1) < 0) break
                    bytes.push(codePoint)
                  } else if (codePoint < 0x800) {
                    if ((units -= 2) < 0) break
                    bytes.push((codePoint >> 0x6) | 0xc0, (codePoint & 0x3f) | 0x80)
                  } else if (codePoint < 0x10000) {
                    if ((units -= 3) < 0) break
                    bytes.push((codePoint >> 0xc) | 0xe0, ((codePoint >> 0x6) & 0x3f) | 0x80, (codePoint & 0x3f) | 0x80)
                  } else if (codePoint < 0x110000) {
                    if ((units -= 4) < 0) break
                    bytes.push(
                      (codePoint >> 0x12) | 0xf0,
                      ((codePoint >> 0xc) & 0x3f) | 0x80,
                      ((codePoint >> 0x6) & 0x3f) | 0x80,
                      (codePoint & 0x3f) | 0x80,
                    )
                  } else {
                    throw new Error("Invalid code point")
                  }
                }

                return bytes
              }

              function asciiToBytes(str) {
                var byteArray = []
                for (var i = 0; i < str.length; ++i) {
                  // Node's code seems to be doing this and not & 0x7F..
                  byteArray.push(str.charCodeAt(i) & 0xff)
                }
                return byteArray
              }

              function utf16leToBytes(str, units) {
                var c, hi, lo
                var byteArray = []
                for (var i = 0; i < str.length; ++i) {
                  if ((units -= 2) < 0) break

                  c = str.charCodeAt(i)
                  hi = c >> 8
                  lo = c % 256
                  byteArray.push(lo)
                  byteArray.push(hi)
                }

                return byteArray
              }

              function base64ToBytes(str) {
                return base64.toByteArray(base64clean(str))
              }

              function blitBuffer(src, dst, offset, length) {
                for (var i = 0; i < length; ++i) {
                  if (i + offset >= dst.length || i >= src.length) break
                  dst[i + offset] = src[i]
                }
                return i
              }

              // ArrayBuffer or Uint8Array objects from other contexts (i.e. iframes) do not pass
              // the `instanceof` check but they should be treated as of that type.
              // See: https://github.com/feross/buffer/issues/166
              function isInstance(obj, type) {
                return (
                  obj instanceof type ||
                  (obj != null &&
                    obj.constructor != null &&
                    obj.constructor.name != null &&
                    obj.constructor.name === type.name)
                )
              }
              function numberIsNaN(obj) {
                // For IE11 support
                return obj !== obj // eslint-disable-line no-self-compare
              }
            }).call(this)
          }).call(this, require("buffer").Buffer)
        },
        { "base64-js": 70, buffer: 71, ieee754: 72 },
      ],
      72: [
        function (require, module, exports) {
          /*! ieee754. BSD-3-Clause License. Feross Aboukhadijeh <https://feross.org/opensource> */
          exports.read = function (buffer, offset, isLE, mLen, nBytes) {
            var e, m
            var eLen = nBytes * 8 - mLen - 1
            var eMax = (1 << eLen) - 1
            var eBias = eMax >> 1
            var nBits = -7
            var i = isLE ? nBytes - 1 : 0
            var d = isLE ? -1 : 1
            var s = buffer[offset + i]

            i += d

            e = s & ((1 << -nBits) - 1)
            s >>= -nBits
            nBits += eLen
            for (; nBits > 0; e = e * 256 + buffer[offset + i], i += d, nBits -= 8) {}

            m = e & ((1 << -nBits) - 1)
            e >>= -nBits
            nBits += mLen
            for (; nBits > 0; m = m * 256 + buffer[offset + i], i += d, nBits -= 8) {}

            if (e === 0) {
              e = 1 - eBias
            } else if (e === eMax) {
              return m ? NaN : (s ? -1 : 1) * Infinity
            } else {
              m = m + Math.pow(2, mLen)
              e = e - eBias
            }
            return (s ? -1 : 1) * m * Math.pow(2, e - mLen)
          }

          exports.write = function (buffer, value, offset, isLE, mLen, nBytes) {
            var e, m, c
            var eLen = nBytes * 8 - mLen - 1
            var eMax = (1 << eLen) - 1
            var eBias = eMax >> 1
            var rt = mLen === 23 ? Math.pow(2, -24) - Math.pow(2, -77) : 0
            var i = isLE ? 0 : nBytes - 1
            var d = isLE ? 1 : -1
            var s = value < 0 || (value === 0 && 1 / value < 0) ? 1 : 0

            value = Math.abs(value)

            if (isNaN(value) || value === Infinity) {
              m = isNaN(value) ? 1 : 0
              e = eMax
            } else {
              e = Math.floor(Math.log(value) / Math.LN2)
              if (value * (c = Math.pow(2, -e)) < 1) {
                e--
                c *= 2
              }
              if (e + eBias >= 1) {
                value += rt / c
              } else {
                value += rt * Math.pow(2, 1 - eBias)
              }
              if (value * c >= 2) {
                e++
                c /= 2
              }

              if (e + eBias >= eMax) {
                m = 0
                e = eMax
              } else if (e + eBias >= 1) {
                m = (value * c - 1) * Math.pow(2, mLen)
                e = e + eBias
              } else {
                m = value * Math.pow(2, eBias - 1) * Math.pow(2, mLen)
                e = 0
              }
            }

            for (; mLen >= 8; buffer[offset + i] = m & 0xff, i += d, m /= 256, mLen -= 8) {}

            e = (e << mLen) | m
            eLen += mLen
            for (; eLen > 0; buffer[offset + i] = e & 0xff, i += d, e /= 256, eLen -= 8) {}

            buffer[offset + i - d] |= s * 128
          }
        },
        {},
      ],
      73: [
        function (require, module, exports) {
          // shim for using process in browser
          var process = (module.exports = {})

          // cached from whatever global is present so that test runners that stub it
          // don't break things.  But we need to wrap it in a try catch in case it is
          // wrapped in strict mode code which doesn't define any globals.  It's inside a
          // function because try/catches deoptimize in certain engines.

          var cachedSetTimeout
          var cachedClearTimeout

          function defaultSetTimout() {
            throw new Error("setTimeout has not been defined")
          }
          function defaultClearTimeout() {
            throw new Error("clearTimeout has not been defined")
          }
          ;(function () {
            try {
              if (typeof setTimeout === "function") {
                cachedSetTimeout = setTimeout
              } else {
                cachedSetTimeout = defaultSetTimout
              }
            } catch (e) {
              cachedSetTimeout = defaultSetTimout
            }
            try {
              if (typeof clearTimeout === "function") {
                cachedClearTimeout = clearTimeout
              } else {
                cachedClearTimeout = defaultClearTimeout
              }
            } catch (e) {
              cachedClearTimeout = defaultClearTimeout
            }
          })()
          function runTimeout(fun) {
            if (cachedSetTimeout === setTimeout) {
              //normal enviroments in sane situations
              return setTimeout(fun, 0)
            }
            // if setTimeout wasn't available but was latter defined
            if ((cachedSetTimeout === defaultSetTimout || !cachedSetTimeout) && setTimeout) {
              cachedSetTimeout = setTimeout
              return setTimeout(fun, 0)
            }
            try {
              // when when somebody has screwed with setTimeout but no I.E. maddness
              return cachedSetTimeout(fun, 0)
            } catch (e) {
              try {
                // When we are in I.E. but the script has been evaled so I.E. doesn't trust the global object when called normally
                return cachedSetTimeout.call(null, fun, 0)
              } catch (e) {
                // same as above but when it's a version of I.E. that must have the global object for 'this', hopfully our context correct otherwise it will throw a global error
                return cachedSetTimeout.call(this, fun, 0)
              }
            }
          }
          function runClearTimeout(marker) {
            if (cachedClearTimeout === clearTimeout) {
              //normal enviroments in sane situations
              return clearTimeout(marker)
            }
            // if clearTimeout wasn't available but was latter defined
            if ((cachedClearTimeout === defaultClearTimeout || !cachedClearTimeout) && clearTimeout) {
              cachedClearTimeout = clearTimeout
              return clearTimeout(marker)
            }
            try {
              // when when somebody has screwed with setTimeout but no I.E. maddness
              return cachedClearTimeout(marker)
            } catch (e) {
              try {
                // When we are in I.E. but the script has been evaled so I.E. doesn't  trust the global object when called normally
                return cachedClearTimeout.call(null, marker)
              } catch (e) {
                // same as above but when it's a version of I.E. that must have the global object for 'this', hopfully our context correct otherwise it will throw a global error.
                // Some versions of I.E. have different rules for clearTimeout vs setTimeout
                return cachedClearTimeout.call(this, marker)
              }
            }
          }
          var queue = []
          var draining = false
          var currentQueue
          var queueIndex = -1

          function cleanUpNextTick() {
            if (!draining || !currentQueue) {
              return
            }
            draining = false
            if (currentQueue.length) {
              queue = currentQueue.concat(queue)
            } else {
              queueIndex = -1
            }
            if (queue.length) {
              drainQueue()
            }
          }

          function drainQueue() {
            if (draining) {
              return
            }
            var timeout = runTimeout(cleanUpNextTick)
            draining = true

            var len = queue.length
            while (len) {
              currentQueue = queue
              queue = []
              while (++queueIndex < len) {
                if (currentQueue) {
                  currentQueue[queueIndex].run()
                }
              }
              queueIndex = -1
              len = queue.length
            }
            currentQueue = null
            draining = false
            runClearTimeout(timeout)
          }

          process.nextTick = function (fun) {
            var args = new Array(arguments.length - 1)
            if (arguments.length > 1) {
              for (var i = 1; i < arguments.length; i++) {
                args[i - 1] = arguments[i]
              }
            }
            queue.push(new Item(fun, args))
            if (queue.length === 1 && !draining) {
              runTimeout(drainQueue)
            }
          }

          // v8 likes predictible objects
          function Item(fun, array) {
            this.fun = fun
            this.array = array
          }
          Item.prototype.run = function () {
            this.fun.apply(null, this.array)
          }
          process.title = "browser"
          process.browser = true
          process.env = {}
          process.argv = []
          process.version = "" // empty string to avoid regexp issues
          process.versions = {}

          function noop() {}

          process.on = noop
          process.addListener = noop
          process.once = noop
          process.off = noop
          process.removeListener = noop
          process.removeAllListeners = noop
          process.emit = noop
          process.prependListener = noop
          process.prependOnceListener = noop

          process.listeners = function (name) {
            return []
          }

          process.binding = function (name) {
            throw new Error("process.binding is not supported")
          }

          process.cwd = function () {
            return "/"
          }
          process.chdir = function (dir) {
            throw new Error("process.chdir is not supported")
          }
          process.umask = function () {
            return 0
          }
        },
        {},
      ],
      74: [
        function (require, module, exports) {
          ;(function (setImmediate, clearImmediate) {
            ;(function () {
              var nextTick = require("process/browser.js").nextTick
              var apply = Function.prototype.apply
              var slice = Array.prototype.slice
              var immediateIds = {}
              var nextImmediateId = 0

              // DOM APIs, for completeness

              exports.setTimeout = function () {
                return new Timeout(apply.call(setTimeout, window, arguments), clearTimeout)
              }
              exports.setInterval = function () {
                return new Timeout(apply.call(setInterval, window, arguments), clearInterval)
              }
              exports.clearTimeout = exports.clearInterval = function (timeout) {
                timeout.close()
              }

              function Timeout(id, clearFn) {
                this._id = id
                this._clearFn = clearFn
              }
              Timeout.prototype.unref = Timeout.prototype.ref = function () {}
              Timeout.prototype.close = function () {
                this._clearFn.call(window, this._id)
              }

              // Does not start the time, just sets up the members needed.
              exports.enroll = function (item, msecs) {
                clearTimeout(item._idleTimeoutId)
                item._idleTimeout = msecs
              }

              exports.unenroll = function (item) {
                clearTimeout(item._idleTimeoutId)
                item._idleTimeout = -1
              }

              exports._unrefActive = exports.active = function (item) {
                clearTimeout(item._idleTimeoutId)

                var msecs = item._idleTimeout
                if (msecs >= 0) {
                  item._idleTimeoutId = setTimeout(function onTimeout() {
                    if (item._onTimeout) item._onTimeout()
                  }, msecs)
                }
              }

              // That's not how node.js implements it but the exposed api is the same.
              exports.setImmediate =
                typeof setImmediate === "function"
                  ? setImmediate
                  : function (fn) {
                      var id = nextImmediateId++
                      var args = arguments.length < 2 ? false : slice.call(arguments, 1)

                      immediateIds[id] = true

                      nextTick(function onNextTick() {
                        if (immediateIds[id]) {
                          // fn.call() is faster so we optimize for the common use-case
                          // @see http://jsperf.com/call-apply-segu
                          if (args) {
                            fn.apply(null, args)
                          } else {
                            fn.call(null)
                          }
                          // Prevent ids from leaking
                          exports.clearImmediate(id)
                        }
                      })

                      return id
                    }

              exports.clearImmediate =
                typeof clearImmediate === "function"
                  ? clearImmediate
                  : function (id) {
                      delete immediateIds[id]
                    }
            }).call(this)
          }).call(this, require("timers").setImmediate, require("timers").clearImmediate)
        },
        { "process/browser.js": 73, timers: 74 },
      ],
      75: [
        function (require, module, exports) {
          "use strict"

          module.exports = function () {
            throw new Error(
              "ws does not work in the browser. Browser clients must use the native " + "WebSocket object",
            )
          }
        },
        {},
      ],
    },
    {},
    [16],
  )(16)
})
