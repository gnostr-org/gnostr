package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/p2p/protocol/ping"
	ma "github.com/multiformats/go-multiaddr"
)

var (
	libp2pHostsMu    sync.RWMutex
	libp2pHosts             = make(map[uint64]*libp2pHostHandle)
	libp2pNextHandle uint64 = 1
)

type libp2pHostHandle struct {
	ctx    context.Context
	cancel context.CancelFunc
	host   host.Host
}

//export kubo_libp2p_host_new
func kubo_libp2p_host_new() uint64 {
	ctx, cancel := context.WithCancel(context.Background())

	h, err := libp2p.New(libp2p.ListenAddrStrings("/ip4/127.0.0.1/tcp/0"))
	if err != nil {
		cancel()
		setError(fmt.Errorf("libp2p new: %w", err))
		return 0
	}

	handle := &libp2pHostHandle{
		ctx:    ctx,
		cancel: cancel,
		host:   h,
	}

	libp2pHostsMu.Lock()
	hid := libp2pNextHandle
	libp2pNextHandle++
	libp2pHosts[hid] = handle
	libp2pHostsMu.Unlock()

	setError(nil)
	return hid
}

//export kubo_libp2p_host_close
func kubo_libp2p_host_close(handle uint64) int64 {
	libp2pHostsMu.Lock()
	h, ok := libp2pHosts[handle]
	if ok {
		delete(libp2pHosts, handle)
	}
	libp2pHostsMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	h.cancel()
	if err := h.host.Close(); err != nil {
		setError(fmt.Errorf("close host: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_peer_id
func kubo_libp2p_host_peer_id(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	return C.CString(h.host.ID().String())
}

//export kubo_libp2p_host_listening_addrs
func kubo_libp2p_host_listening_addrs(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	var parts []string
	for _, a := range h.host.Addrs() {
		parts = append(parts, a.String())
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_libp2p_host_connect
func kubo_libp2p_host_connect(handle uint64, addr *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
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

	if err := h.host.Connect(h.ctx, *info); err != nil {
		setError(fmt.Errorf("connect: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_libp2p_host_ping
func kubo_libp2p_host_ping(handle uint64, peer_id *C.char) int64 {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return -1
	}

	pid, err := peer.Decode(C.GoString(peer_id))
	if err != nil {
		setError(fmt.Errorf("decode peer id: %w", err))
		return -1
	}

	ctx, cancel := context.WithTimeout(h.ctx, 10*time.Second)
	defer cancel()

	result := <-ping.Ping(ctx, h.host, pid)
	if result.Error != nil {
		setError(fmt.Errorf("ping: %w", result.Error))
		return -1
	}

	setError(nil)
	return int64(result.RTT.Milliseconds())
}

//export kubo_libp2p_host_protocols
func kubo_libp2p_host_protocols(handle uint64) *C.char {
	libp2pHostsMu.RLock()
	h, ok := libp2pHosts[handle]
	libp2pHostsMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid libp2p handle %d", handle))
		return nil
	}

	protos := h.host.Mux().Protocols()
	var names []string
	for _, p := range protos {
		names = append(names, string(p))
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}
