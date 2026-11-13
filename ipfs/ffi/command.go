package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"strings"
	"sync"
	"unsafe"

	cmds "github.com/ipfs/go-ipfs-cmds"
	cli "github.com/ipfs/go-ipfs-cmds/cli"
	kubocli "github.com/ipfs/kubo/cmd/ipfs/kubo"
	oldcmds "github.com/ipfs/kubo/commands"
	"github.com/ipfs/kubo/core"
	commands "github.com/ipfs/kubo/core/commands"
	"github.com/ipfs/kubo/plugin/loader"
	"github.com/ipfs/kubo/repo/fsrepo"
)

type bufCloser struct{ *bytes.Buffer }

func (b *bufCloser) Close() error { return nil }

var (
	pluginCache   = make(map[string]*loader.PluginLoader)
	pluginCacheMu sync.Mutex
)

func cachedPlugins(repoPath string) (*loader.PluginLoader, error) {
	pluginCacheMu.Lock()
	defer pluginCacheMu.Unlock()
	if p, ok := pluginCache[repoPath]; ok {
		return p, nil
	}
	plugins, err := loader.NewPluginLoader(repoPath)
	if err != nil {
		return nil, err
	}
	if err := plugins.Initialize(); err != nil {
		return nil, err
	}
	if err := plugins.Inject(); err != nil {
		return nil, err
	}
	pluginCache[repoPath] = plugins
	return plugins, nil
}

func buildEnvCached(repoPath string, req *cmds.Request) (cmds.Environment, error) {
	var plugins *loader.PluginLoader
	var err error
	if gp := GlobalPlugins(); gp != nil {
		plugins = gp
	} else {
		if err := ensurePlugins(); err != nil {
			return nil, err
		}
		plugins = GlobalPlugins()
	}
	if plugins == nil {
		plugins, err = cachedPlugins(repoPath)
		if err != nil {
			return nil, err
		}
	}
	return &oldcmds.Context{
		ConfigRoot: repoPath,
		ReqLog:     &oldcmds.ReqLog{},
		Plugins:    plugins,
		ConstructNode: func() (n *core.IpfsNode, err error) {
			if req == nil {
				return nil, errors.New("constructing node without a request")
			}
			r, err := fsrepo.Open(repoPath)
			if err != nil {
				return nil, err
			}
			n, err = core.NewNode(context.Background(), &core.BuildCfg{
				Repo: r,
			})
			if err != nil {
				return nil, err
			}
			return n, nil
		},
	}, nil
}

func runCommandOnNode(repoPath string, handle uint64, args []string) (string, error) {
	ctx := context.Background()
	cmdline := append([]string{"ipfs"}, args...)

	req, err := cli.Parse(ctx, cmdline[1:], os.Stdin, kubocli.Root)
	if err != nil {
		return "", err
	}
	if req.Options == nil {
		req.Options = map[string]interface{}{}
	}
	if repoPath != "" {
		req.Options[commands.RepoDirOption] = repoPath
	}

	nodesMu.RLock()
	h, ok := nodes[handle]
	nodesMu.RUnlock()
	if !ok {
		return "", fmt.Errorf("invalid handle %d", handle)
	}

	plugins, _ := cachedPlugins(repoPath)
	env := &oldcmds.Context{
		ConfigRoot: repoPath,
		ReqLog:     &oldcmds.ReqLog{},
		Plugins:    plugins,
		ConstructNode: func() (*core.IpfsNode, error) {
			return h.node, nil
		},
	}

	buf := &bytes.Buffer{}
	bc := &bufCloser{buf}
	re, err := cmds.NewWriterResponseEmitter(bc, req)
	if err != nil {
		return "", err
	}
	exe := cmds.NewExecutor(req.Root)
	if err := exe.Execute(req, re, env); err != nil {
		re.Close()
		return buf.String(), err
	}
	re.Close()
	return buf.String(), nil
}

func runCommand(repoPath string, args []string) (string, error) {
	ctx := context.Background()
	cmdline := append([]string{"ipfs"}, args...)

	req, err := cli.Parse(ctx, cmdline[1:], os.Stdin, kubocli.Root)
	if err != nil {
		return "", err
	}
	if req.Options == nil {
		req.Options = map[string]interface{}{}
	}
	if repoPath != "" {
		req.Options[commands.RepoDirOption] = repoPath
	}

	env, err := buildEnvCached(repoPath, req)
	if err != nil {
		return "", err
	}

	buf := &bytes.Buffer{}
	bc := &bufCloser{buf}
	re, err := cmds.NewWriterResponseEmitter(bc, req)
	if err != nil {
		return "", err
	}
	exe := cmds.NewExecutor(req.Root)
	if err := exe.Execute(req, re, env); err != nil {
		re.Close()
		return buf.String(), err
	}
	re.Close()
	return buf.String(), nil
}

//export kubo_run_command_on_node
func kubo_run_command_on_node(repoPath *C.char, handle C.uint64_t, argc C.int, argv **C.char) *C.char {
	rp := ""
	if repoPath != nil {
		rp = C.GoString(repoPath)
	}
	args := make([]string, 0, int(argc))
	for i := 0; i < int(argc); i++ {
		ptr := (**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(argv)) + uintptr(i)*unsafe.Sizeof((*C.char)(nil))))
		args = append(args, C.GoString(*ptr))
	}
	out, err := runCommandOnNode(rp, uint64(handle), args)
	if err != nil {
		setError(err)
	} else {
		setError(nil)
	}
	if out == "" && err != nil {
		out = err.Error()
	}
	return C.CString(out)
}

//export kubo_run_command
func kubo_run_command(repoPath *C.char, argc C.int, argv **C.char) *C.char {
	rp := ""
	if repoPath != nil {
		rp = C.GoString(repoPath)
	}
	args := make([]string, 0, int(argc))
	for i := 0; i < int(argc); i++ {
		ptr := (**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(argv)) + uintptr(i)*unsafe.Sizeof((*C.char)(nil))))
		args = append(args, C.GoString(*ptr))
	}
	out, err := runCommand(rp, args)
	if err != nil {
		setError(err)
	} else {
		setError(nil)
	}
	if out == "" && err != nil {
		out = err.Error()
	}
	return C.CString(out)
}

//export kubo_run_command_line
func kubo_run_command_line(repoPath *C.char, line *C.char) *C.char {
	rp := ""
	if repoPath != nil {
		rp = C.GoString(repoPath)
	}
	var args []string
	var cur strings.Builder
	inQuote := false
	for _, r := range C.GoString(line) {
		switch {
		case r == '"':
			inQuote = !inQuote
		case !inQuote && (r == ' ' || r == '\t' || r == '\n'):
			if cur.Len() > 0 {
				args = append(args, cur.String())
				cur.Reset()
			}
		default:
			cur.WriteRune(r)
		}
	}
	if cur.Len() > 0 {
		args = append(args, cur.String())
	}
	out, err := runCommand(rp, args)
	if err != nil {
		setError(err)
	} else {
		setError(nil)
	}
	if out == "" && err != nil {
		out = err.Error()
	}
	return C.CString(out)
}
