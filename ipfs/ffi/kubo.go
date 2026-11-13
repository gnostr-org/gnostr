package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"unsafe"

	logging "github.com/ipfs/go-log/v2"
	"go.uber.org/zap/zapcore"

	"github.com/fsnotify/fsnotify"
	"github.com/ipfs/boxo/files"
	"github.com/ipfs/boxo/path"
	"github.com/ipfs/go-cid"
	ipfs "github.com/ipfs/kubo"
	"github.com/ipfs/kubo/commands"
	"github.com/ipfs/kubo/config"
	"github.com/ipfs/kubo/core"
	"github.com/ipfs/kubo/core/coreapi"
	"github.com/ipfs/kubo/core/corehttp"
	coreiface "github.com/ipfs/kubo/core/coreiface"
	"github.com/ipfs/kubo/core/coreiface/options"
	"github.com/ipfs/kubo/core/node/libp2p"
	"github.com/ipfs/kubo/plugin/loader"
	"github.com/ipfs/kubo/repo/fsrepo"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	manet "github.com/multiformats/go-multiaddr/net"
)

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

//export kubo_version
func kubo_version() *C.char {
	return C.CString(ipfs.CurrentVersionNumber)
}

// ---------------------------------------------------------------------------
// Log capture
// ---------------------------------------------------------------------------

var (
	logBufMu sync.Mutex
	logBuf   strings.Builder
)

type logWriter struct{}

func (logWriter) Write(p []byte) (int, error) {
	logBufMu.Lock()
	defer logBufMu.Unlock()
	logBuf.Write(p)
	return len(p), nil
}

func init() {
	encCfg := zapcore.EncoderConfig{
		TimeKey:        "t",
		LevelKey:       "level",
		NameKey:        "logger",
		CallerKey:      "caller",
		MessageKey:     "msg",
		LineEnding:     zapcore.DefaultLineEnding,
		EncodeLevel:    zapcore.CapitalLevelEncoder,
		EncodeTime:     zapcore.ISO8601TimeEncoder,
		EncodeDuration: zapcore.StringDurationEncoder,
		EncodeCaller:   zapcore.ShortCallerEncoder,
	}
	core := zapcore.NewCore(
		zapcore.NewConsoleEncoder(encCfg),
		zapcore.AddSync(logWriter{}),
		zapcore.DebugLevel,
	)
	logging.SetPrimaryCore(core)
}

//export kubo_get_logs
func kubo_get_logs() *C.char {
	logBufMu.Lock()
	defer logBufMu.Unlock()
	s := logBuf.String()
	logBuf.Reset()
	return C.CString(s)
}

// ---------------------------------------------------------------------------
// Plugin loading (once)
// ---------------------------------------------------------------------------

var (
	pluginsOnce   sync.Once
	pluginsErr    error
	globalPlugins *loader.PluginLoader
)

func ensurePlugins() error {
	pluginsOnce.Do(func() {
		plugins, err := loader.NewPluginLoader("")
		if err != nil {
			pluginsErr = fmt.Errorf("error loading plugins: %w", err)
			return
		}
		if err := plugins.Initialize(); err != nil {
			pluginsErr = fmt.Errorf("error initializing plugins: %w", err)
			return
		}
		if err := plugins.Inject(); err != nil {
			pluginsErr = fmt.Errorf("error injecting plugins: %w", err)
			return
		}
		globalPlugins = plugins
	})
	return pluginsErr
}

// GlobalPlugins returns the process-wide plugin loader (may be nil before ensurePlugins).
func GlobalPlugins() *loader.PluginLoader {
	return globalPlugins
}

// ---------------------------------------------------------------------------
// Repo initialization
// ---------------------------------------------------------------------------

//export kubo_init_repo
func kubo_init_repo(repoPath *C.char) int64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return -1
	}

	path := C.GoString(repoPath)

	identity, err := config.CreateIdentity(io.Discard, []options.KeyGenerateOption{
		options.Key.Type(options.Ed25519Key),
	})
	if err != nil {
		setError(fmt.Errorf("create identity: %w", err))
		return -1
	}

	cfg, err := config.InitWithIdentity(identity)
	if err != nil {
		setError(fmt.Errorf("init config: %w", err))
		return -1
	}

	cfg.Addresses.Swarm = []string{
		"/ip4/127.0.0.1/tcp/0",
	}
	cfg.Swarm.Transports.Network.QUIC = config.False
	cfg.Swarm.Transports.Network.Relay = config.False
	cfg.Swarm.Transports.Network.WebTransport = config.False
	cfg.Swarm.Transports.Network.WebRTCDirect = config.False
	cfg.Swarm.Transports.Network.Websocket = config.False
	cfg.AutoTLS.Enabled = config.False
	cfg.Routing.Type = config.NewOptionalString("none")
	cfg.Bootstrap = []string{}
	cfg.Discovery.MDNS.Enabled = false

	if err := fsrepo.Init(path, cfg); err != nil {
		setError(fmt.Errorf("init repo: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// kubo_init_repo_networked creates a repo configured for real network
// participation: listens on all interfaces, enables mDNS + QUIC + relay +
// AutoTLS, and uses Kubo's default bootstrap peers so the node joins the
// public IPFS swarm. Use this for registry nodes that must be reachable.
//
//export kubo_init_repo_networked
func kubo_init_repo_networked(repoPath *C.char) int64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return -1
	}

	path := C.GoString(repoPath)

	identity, err := config.CreateIdentity(io.Discard, []options.KeyGenerateOption{
		options.Key.Type(options.Ed25519Key),
	})
	if err != nil {
		setError(fmt.Errorf("create identity: %w", err))
		return -1
	}

	cfg, err := config.InitWithIdentity(identity)
	if err != nil {
		setError(fmt.Errorf("init config: %w", err))
		return -1
	}

	cfg.Addresses.Swarm = []string{
		"/ip4/0.0.0.0/tcp/0",
		"/ip6/::/tcp/0",
		"/ip4/0.0.0.0/udp/0/quic-v1",
		"/ip6/::/udp/0/quic-v1",
	}
	cfg.Swarm.Transports.Network.QUIC = config.True
	cfg.Swarm.Transports.Network.Relay = config.True
	cfg.Swarm.Transports.Network.WebTransport = config.True
	cfg.Swarm.Transports.Network.WebRTCDirect = config.True
	cfg.Swarm.Transports.Network.Websocket = config.True
	cfg.AutoTLS.Enabled = config.True
	cfg.Routing.Type = config.NewOptionalString("auto")
	// Keep Kubo's default bootstrap peers (autoconf fills them in).
	cfg.Discovery.MDNS.Enabled = true

	if err := fsrepo.Init(path, cfg); err != nil {
		setError(fmt.Errorf("init repo: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// kubo_init_repo_with_config initialises a repo from a caller-supplied Kubo
// config JSON blob, allowing full control over addresses, bootstrap, routing,
// transports, etc. repoPath must not already contain a repo.
//
//export kubo_init_repo_with_config
func kubo_init_repo_with_config(repoPath *C.char, configJSON *C.char) int64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return -1
	}

	path := C.GoString(repoPath)
	jsonStr := C.GoString(configJSON)

	identity, err := config.CreateIdentity(io.Discard, []options.KeyGenerateOption{
		options.Key.Type(options.Ed25519Key),
	})
	if err != nil {
		setError(fmt.Errorf("create identity: %w", err))
		return -1
	}

	cfg, err := config.InitWithIdentity(identity)
	if err != nil {
		setError(fmt.Errorf("init config: %w", err))
		return -1
	}

	if err := json.Unmarshal([]byte(jsonStr), cfg); err != nil {
		setError(fmt.Errorf("parse config json: %w", err))
		return -1
	}

	if err := fsrepo.Init(path, cfg); err != nil {
		setError(fmt.Errorf("init repo: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Node registry
// ---------------------------------------------------------------------------

type nodeHandle struct {
	ctx            context.Context
	cancel         context.CancelFunc
	node           *core.IpfsNode
	api            coreiface.CoreAPI
	apiListener    manet.Listener
	apiNetListener net.Listener
	apiServer      *http.Server
	staticDirs     map[string]string
	authToken      string
}

var (
	nodesMu    sync.RWMutex
	nodes             = make(map[uint64]*nodeHandle)
	nextHandle uint64 = 1
)

// startNodeInternal is the string-based implementation used by tests and exports.
func startNodeInternal(repoPath string, online bool) (uint64, error) {
	if err := ensurePlugins(); err != nil {
		return 0, err
	}

	repo, err := fsrepo.Open(repoPath)
	if err != nil {
		return 0, fmt.Errorf("open repo: %w", err)
	}

	ctx, cancel := context.WithCancel(context.Background())

	cfg := &core.BuildCfg{
		Online: online,
		Repo:   repo,
	}
	if !cfg.Online {
		cfg.Routing = libp2p.NilRouterOption
	}

	n, err := core.NewNode(ctx, cfg)
	if err != nil {
		cancel()
		repo.Close()
		return 0, fmt.Errorf("new node: %w", err)
	}

	api, err := coreapi.NewCoreAPI(n)
	if err != nil {
		n.Close()
		cancel()
		repo.Close()
		return 0, fmt.Errorf("core api: %w", err)
	}

	h := &nodeHandle{
		ctx:    ctx,
		cancel: cancel,
		node:   n,
		api:    api,
	}

	nodesMu.Lock()
	handle := nextHandle
	nextHandle++
	nodes[handle] = h
	nodesMu.Unlock()

	return handle, nil
}

//export kubo_node_start
func kubo_node_start(repoPath *C.char, online C.uint8_t) uint64 {
	h, err := startNodeInternal(C.GoString(repoPath), online != 0)
	if err != nil {
		setError(err)
		return 0
	}
	setError(nil)
	return h
}

//export kubo_node_stop
func kubo_node_stop(handle uint64) int64 {
	nodesMu.Lock()
	h, ok := nodes[handle]
	if ok {
		delete(nodes, handle)
	}
	nodesMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	if h.apiListener != nil {
		h.apiListener.Close()
	}
	h.cancel()
	if err := h.node.Close(); err != nil {
		setError(fmt.Errorf("close node: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_node_add_static
func kubo_node_add_static(handle uint64, prefix *C.char, dir *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	if h.staticDirs == nil {
		h.staticDirs = make(map[string]string)
	}
	p := C.GoString(prefix)
	if p == "" {
		p = "/"
	}
	h.staticDirs[p] = C.GoString(dir)
	setError(nil)
	return 0
}

//export kubo_node_start_api
func kubo_node_start_api(handle uint64, multiaddr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrStr := C.GoString(multiaddr)
	addr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return nil
	}

	listener, err := manet.Listen(addr)
	if err != nil {
		setError(fmt.Errorf("listen: %w", err))
		return nil
	}

	cctx := commands.Context{
		ConstructNode: func() (*core.IpfsNode, error) {
			return h.node, nil
		},
		ReqLog: &commands.ReqLog{},
	}

	opts := []corehttp.ServeOption{
		corehttp.CheckVersionOption(),
		corehttp.CommandsOption(cctx),
		corehttp.WebUIOption,
		corehttp.GatewayOption("/ipfs", "/ipns"),
		corehttp.VersionOption(),
	}

	// Serve any registered static directories.
	if len(h.staticDirs) > 0 {
		dirs := h.staticDirs
		opts = append(opts, func(n *core.IpfsNode, _ net.Listener, mux *http.ServeMux) (*http.ServeMux, error) {
			for prefix, dir := range dirs {
				fs := http.FileServer(http.Dir(dir))
				mux.Handle(prefix, http.StripPrefix(prefix, fs))
			}
			return mux, nil
		})
	}

	h.apiListener = listener
	h.apiNetListener = manet.NetListener(listener)
	h.apiServer = &http.Server{}

	go func() {
		handler, err := corehttp.MakeHandler(h.node, h.apiNetListener, opts...)
		if err != nil {
			setError(fmt.Errorf("make handler: %w", err))
			return
		}
		h.apiServer.Handler = handler
		h.apiServer.Serve(h.apiNetListener)
	}()

	setError(nil)
	return C.CString(listener.Multiaddr().String())
}

// kubo_node_start_gateway serves ONLY the IPFS gateway (/ipfs, /ipns) and any
// registered static directories. It does NOT expose the writable CoreAPI
// (/api/v0/*) or the WebUI, making it safe to bind to a public address for a
// registry server. Call kubo_node_add_static BEFORE this.
//
//export kubo_node_start_gateway
func kubo_node_start_gateway(handle uint64, multiaddr *C.char) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrStr := C.GoString(multiaddr)
	addr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return nil
	}

	listener, err := manet.Listen(addr)
	if err != nil {
		setError(fmt.Errorf("listen: %w", err))
		return nil
	}

	opts := []corehttp.ServeOption{
		corehttp.GatewayOption("/ipfs", "/ipns"),
		corehttp.VersionOption(),
	}

	if len(h.staticDirs) > 0 {
		dirs := h.staticDirs
		opts = append(opts, func(n *core.IpfsNode, _ net.Listener, mux *http.ServeMux) (*http.ServeMux, error) {
			// Crate redirect handler (if registered via kubo_node_add_crate_redirect).
			if regDir, ok := dirs["__crate_redirect__"]; ok {
				mux.HandleFunc("/crates/", func(w http.ResponseWriter, r *http.Request) {
					trimmed := strings.TrimPrefix(r.URL.Path, "/crates/")
					parts := strings.Split(trimmed, "/")
					if len(parts) < 3 {
						http.NotFound(w, r)
						return
					}
					crateName := parts[1]
					version := strings.TrimSuffix(parts[2], ".crate")

					// Compute the index file path using Margo's prefix logic.
					var indexParts []string
					n := len(crateName)
					if n == 1 {
						indexParts = []string{"1"}
					} else if n == 2 {
						indexParts = []string{"2"}
					} else if n == 3 {
						indexParts = []string{"3", crateName[0:1]}
					} else if n >= 4 {
						indexParts = []string{crateName[0:2], crateName[2:4]}
					}
					indexPath := filepath.Join(append([]string{regDir}, indexParts...)...)
					indexPath = filepath.Join(indexPath, crateName)
					data, err := os.ReadFile(indexPath)
					if err != nil {
						http.NotFound(w, r)
						return
					}

					var cid string
					for _, line := range strings.Split(string(data), "\n") {
						line = strings.TrimSpace(line)
						if line == "" {
							continue
						}
						var entry map[string]interface{}
						if err := json.Unmarshal([]byte(line), &entry); err != nil {
							continue
						}
						if v, ok := entry["vers"].(string); ok && v == version {
							if c, ok := entry["ipfs_cid"].(string); ok && c != "" {
								cid = c
							}
							break
						}
					}

					if cid != "" {
						// Fetch-on-miss: if the crate file is not on disk, fetch it
						// from IPFS and cache it for future requests.
						filePath := filepath.Join(append([]string{regDir, "crates"}, indexParts...)...)
						filePath = filepath.Join(filePath, crateName, version+".crate")
						if _, err := os.Stat(filePath); os.IsNotExist(err) {
							// Fetch from IPFS via the local node.
							ipfsPath, _ := path.NewPath("/ipfs/" + cid)
							node, err := h.api.Unixfs().Get(h.ctx, ipfsPath)
							if err == nil {
								if file, ok := node.(files.File); ok {
									buf, err := io.ReadAll(file)
									node.Close()
									if err == nil && len(buf) > 0 {
										// Verify against cksum if present.
										var expectedCksum string
										for _, line := range strings.Split(string(data), "\n") {
											line = strings.TrimSpace(line)
											if line == "" {
												continue
											}
											var e map[string]interface{}
											if err := json.Unmarshal([]byte(line), &e); err != nil {
												continue
											}
											if v, ok := e["vers"].(string); ok && v == version {
												if c, ok := e["cksum"].(string); ok {
													expectedCksum = c
												}
												break
											}
										}
										verified := true
										if expectedCksum != "" {
											h := sha256.Sum256(buf)
											actual := hex.EncodeToString(h[:])
											if actual != expectedCksum {
												verified = false
											}
										}
										if verified {
											// Ensure parent dir exists.
											os.MkdirAll(filepath.Dir(filePath), 0755)
											os.WriteFile(filePath, buf, 0644)
										}
									}
								} else {
									node.Close()
								}
							}
						}

						http.Redirect(w, r, "/ipfs/"+cid, http.StatusFound)
						return
					}

					// Fall back to serving the file from disk.
					filePath := filepath.Join(append([]string{regDir, "crates"}, indexParts...)...)
					filePath = filepath.Join(filePath, crateName, version+".crate")
					f, err := os.Open(filePath)
					if err != nil {
						http.NotFound(w, r)
						return
					}
					defer f.Close()
					fi, err := f.Stat()
					if err != nil {
						http.NotFound(w, r)
						return
					}
					http.ServeContent(w, r, fi.Name(), fi.ModTime(), f)
				})
			}

			for prefix, dir := range dirs {
				if prefix == "__crate_redirect__" {
					continue
				}
				fs := http.FileServer(http.Dir(dir))
				mux.Handle(prefix, http.StripPrefix(prefix, fs))
			}
			return mux, nil
		})
	}

	h.apiListener = listener
	h.apiNetListener = manet.NetListener(listener)
	h.apiServer = &http.Server{}

	go func() {
		handler, err := corehttp.MakeHandler(h.node, h.apiNetListener, opts...)
		if err != nil {
			setError(fmt.Errorf("make handler: %w", err))
			return
		}
		h.apiServer.Handler = handler
		if h.authToken != "" {
			h.apiServer.Handler = authMiddleware(handler, h.authToken)
		}
		h.apiServer.Serve(h.apiNetListener)
	}()

	setError(nil)
	return C.CString(listener.Multiaddr().String())
}

// kubo_node_set_auth_token sets a Bearer token for gateway authentication.
// Must be called before kubo_node_start_gateway.
//
//export kubo_node_set_auth_token
func kubo_node_set_auth_token(handle uint64, authToken *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	h.authToken = C.GoString(authToken)
	setError(nil)
	return 0
}

// kubo_node_add_crate_redirect registers a handler on the gateway that, for
// /crates/{prefix}/{crate}/{version}.crate requests, looks up the ipfs_cid in
// the registry's index files and 302-redirects to /ipfs/<cid>. Call before
// kubo_node_start_gateway. registryDir is the registry root directory.
//
//export kubo_node_add_crate_redirect
func kubo_node_add_crate_redirect(handle uint64, registryDir *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}
	if h.staticDirs == nil {
		h.staticDirs = make(map[string]string)
	}
	// Stash the registry dir under a special key so start_gateway can use it.
	h.staticDirs["__crate_redirect__"] = C.GoString(registryDir)
	setError(nil)
	return 0
}

// authMiddleware wraps an http.Handler with Bearer token authentication.
func authMiddleware(next http.Handler, token string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		auth := r.Header.Get("Authorization")
		if auth != "Bearer "+token {
			w.Header().Set("WWW-Authenticate", "Bearer")
			http.Error(w, "Unauthorized", http.StatusUnauthorized)
			return
		}
		next.ServeHTTP(w, r)
	})
}

//export kubo_node_api_addrs
func kubo_node_api_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	if h.apiListener == nil {
		setError(fmt.Errorf("api not started"))
		return nil
	}

	setError(nil)
	return C.CString(h.apiListener.Multiaddr().String())
}

//export kubo_node_peer_id
func kubo_node_peer_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	return C.CString(h.node.Identity.String())
}

//export kubo_node_listening_addrs
func kubo_node_listening_addrs(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	addrs, err := h.api.Swarm().LocalAddrs(h.ctx)
	if err != nil {
		setError(fmt.Errorf("local addrs: %w", err))
		return nil
	}

	var parts []string
	for _, a := range addrs {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_connect
func kubo_node_connect(handle uint64, addr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	addrStr := C.GoString(addr)
	maddr, err := ma.NewMultiaddr(addrStr)
	if err != nil {
		setError(fmt.Errorf("parse multiaddr: %w", err))
		return -1
	}

	info, err := peer.AddrInfoFromP2pAddr(maddr)
	if err != nil {
		setError(fmt.Errorf("extract peer info: %w", err))
		return -1
	}

	if err := h.api.Swarm().Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_swarm_peers
func kubo_swarm_peers(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	peers, err := h.api.Swarm().Peers(h.ctx)
	if err != nil {
		setError(fmt.Errorf("swarm peers: %w", err))
		return nil
	}

	var parts []string
	for _, p := range peers {
		addr := ""
		if p.Address() != nil {
			addr = p.Address().String()
		}
		parts = append(parts, p.ID().String()+"\t"+addr)
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_node_id
func kubo_node_id(handle uint64) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	id := h.node.Identity.String()
	pk, err := h.node.Identity.ExtractPublicKey()
	if err != nil {
		setError(fmt.Errorf("extract public key: %w", err))
		return nil
	}
	pkBytes, err := pk.Raw()
	if err != nil {
		setError(fmt.Errorf("raw public key: %w", err))
		return nil
	}

	info := fmt.Sprintf(
		`{"id":"%s","public_key":"%s"}`,
		id,
		base64.StdEncoding.EncodeToString(pkBytes),
	)
	setError(nil)
	return C.CString(info)
}

// ---------------------------------------------------------------------------
// UnixFS helpers
// ---------------------------------------------------------------------------

//export kubo_unixfs_add_bytes
func kubo_unixfs_add_bytes(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	file := files.NewBytesFile(goData)

	p, err := h.api.Unixfs().Add(h.ctx, file, options.Unixfs.Pin(true, ""))
	if err != nil {
		setError(fmt.Errorf("unixfs add: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(p.RootCid().String())
}

//export kubo_unixfs_cat
func kubo_unixfs_cat(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	node, err := h.api.Unixfs().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("unixfs get: %w", err))
		return -1
	}
	defer node.Close()

	file, ok := node.(files.File)
	if !ok {
		setError(fmt.Errorf("node is not a file"))
		return -1
	}

	buf, err := io.ReadAll(file)
	if err != nil {
		setError(fmt.Errorf("read file: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

// ---------------------------------------------------------------------------
// Block API
// ---------------------------------------------------------------------------

//export kubo_block_put
func kubo_block_put(handle uint64, data *C.uint8_t, length C.size_t) *C.char {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return nil
	}

	goData := C.GoBytes(unsafe.Pointer(data), C.int(length))
	stat, err := h.api.Block().Put(h.ctx, bytes.NewReader(goData))
	if err != nil {
		setError(fmt.Errorf("block put: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(stat.Path().RootCid().String())
}

//export kubo_block_get
func kubo_block_get(handle uint64, cidStr *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	reader, err := h.api.Block().Get(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block get: %w", err))
		return -1
	}

	buf, err := io.ReadAll(reader)
	if err != nil {
		setError(fmt.Errorf("read block: %w", err))
		return -1
	}

	if len(buf) == 0 {
		*out = nil
		*outLen = 0
		return 0
	}

	cBuf := C.malloc(C.size_t(len(buf)))
	copy((*[1 << 30]byte)(cBuf)[:len(buf):len(buf)], buf)
	*out = (*C.uint8_t)(cBuf)
	*outLen = C.size_t(len(buf))

	setError(nil)
	return 0
}

//export kubo_block_stat
func kubo_block_stat(handle uint64, cidStr *C.char) int64 {
	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid handle %d", handle))
		return -1
	}

	cidStrGo := C.GoString(cidStr)
	var p path.Path
	var err error

	if strings.HasPrefix(cidStrGo, "/ipfs/") || strings.HasPrefix(cidStrGo, "/ipns/") {
		p, err = path.NewPath(cidStrGo)
	} else {
		var c cid.Cid
		c, err = cid.Decode(cidStrGo)
		if err == nil {
			p = path.FromCid(c)
		}
	}
	if err != nil {
		setError(fmt.Errorf("parse path: %w", err))
		return -1
	}

	stat, err := h.api.Block().Stat(h.ctx, p)
	if err != nil {
		setError(fmt.Errorf("block stat: %w", err))
		return -1
	}

	setError(nil)
	return int64(stat.Size())
}

// ---------------------------------------------------------------------------
// ipfswatch: directory watcher
// ---------------------------------------------------------------------------

var (
	watchMu      sync.Mutex
	watchHandles        = make(map[uint64]*watchHandle)
	watchNext    uint64 = 1
)

type watchHandle struct {
	cancel context.CancelFunc
	done   chan struct{}
}

func isHidden(path string) bool {
	base := filepath.Base(path)
	return len(base) > 0 && base[0] == '.'
}

func isDir(path string) (bool, error) {
	fi, err := os.Stat(path)
	if err != nil {
		return false, err
	}
	return fi.IsDir(), nil
}

func addTree(w *fsnotify.Watcher, root string) error {
	return filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return nil
		}
		d, err := isDir(path)
		if err != nil {
			return nil
		}
		if d {
			if err := w.Add(path); err != nil {
				return err
			}
		}
		return nil
	})
}

//export kubo_watch_path
func kubo_watch_path(repoPath *C.char, watchPath *C.char) uint64 {
	if err := ensurePlugins(); err != nil {
		setError(err)
		return 0
	}

	rp := C.GoString(repoPath)
	wp := C.GoString(watchPath)

	r, err := fsrepo.Open(rp)
	if err != nil {
		setError(fmt.Errorf("open repo: %w", err))
		return 0
	}

	node, err := core.NewNode(context.Background(), &core.BuildCfg{
		Online: true,
		Repo:   r,
	})
	if err != nil {
		r.Close()
		setError(fmt.Errorf("new node: %w", err))
		return 0
	}

	api, err := coreapi.NewCoreAPI(node)
	if err != nil {
		node.Close()
		r.Close()
		setError(fmt.Errorf("core api: %w", err))
		return 0
	}

	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		node.Close()
		r.Close()
		setError(fmt.Errorf("watcher: %w", err))
		return 0
	}

	if err := addTree(watcher, wp); err != nil {
		watcher.Close()
		node.Close()
		r.Close()
		setError(fmt.Errorf("add tree: %w", err))
		return 0
	}

	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan struct{})

	go func() {
		defer close(done)
		defer watcher.Close()
		defer node.Close()
		defer r.Close()

		for {
			select {
			case <-ctx.Done():
				return
			case e, ok := <-watcher.Events:
				if !ok {
					return
				}
				d, err := isDir(e.Name)
				if err != nil {
					continue
				}
				switch e.Op {
				case fsnotify.Remove:
					if d {
						watcher.Remove(e.Name)
					}
				default:
					if e.Op == fsnotify.Create && d {
						addTree(watcher, e.Name)
					}
					go func(name string, isDir bool) {
						file, err := os.Open(name)
						if err != nil {
							return
						}
						defer file.Close()
						st, err := file.Stat()
						if err != nil {
							return
						}
						f, err := files.NewReaderPathFile(name, file, st)
						if err != nil {
							return
						}
						k, err := api.Unixfs().Add(ctx, f)
						if err != nil {
							return
						}
						setError(nil)
						_ = k
						_ = isDir
					}(e.Name, d)
				}
			case err, ok := <-watcher.Errors:
				if !ok {
					return
				}
				_ = err
			}
		}
	}()

	h := &watchHandle{cancel: cancel, done: done}
	watchMu.Lock()
	handle := watchNext
	watchNext++
	watchHandles[handle] = h
	watchMu.Unlock()

	setError(nil)
	return handle
}

//export kubo_watch_stop
func kubo_watch_stop(handle uint64) int64 {
	watchMu.Lock()
	h, ok := watchHandles[handle]
	if ok {
		delete(watchHandles, handle)
	}
	watchMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid watch handle %d", handle))
		return -1
	}
	h.cancel()
	<-h.done
	setError(nil)
	return 0
}
