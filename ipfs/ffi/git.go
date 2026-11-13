package main

/*
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"io"
	"strings"
	"sync"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
)

var (
	gitReposMu    sync.RWMutex
	gitRepos             = make(map[uint64]*git.Repository)
	gitNextHandle uint64 = 1
)

//export kubo_git_clone
func kubo_git_clone(url *C.char, path *C.char, bare C.uint8_t) int64 {
	_, err := git.PlainClone(C.GoString(path), bare != 0, &git.CloneOptions{
		URL:      C.GoString(url),
		Progress: nil,
	})
	if err != nil {
		setError(fmt.Errorf("git clone: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_git_open
func kubo_git_open(path *C.char) uint64 {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git open: %w", err))
		return 0
	}

	gitReposMu.Lock()
	handle := gitNextHandle
	gitNextHandle++
	gitRepos[handle] = repo
	gitReposMu.Unlock()

	setError(nil)
	return handle
}

//export kubo_git_repo_head
func kubo_git_repo_head(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	ref, err := repo.Head()
	if err != nil {
		setError(fmt.Errorf("git head: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(ref.Hash().String())
}

//export kubo_git_repo_free
func kubo_git_repo_free(handle uint64) int64 {
	gitReposMu.Lock()
	_, ok := gitRepos[handle]
	if ok {
		delete(gitRepos, handle)
	}
	gitReposMu.Unlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_git_init
func kubo_git_init(path *C.char, bare C.uint8_t) int64 {
	_, err := git.PlainInit(C.GoString(path), bare != 0)
	if err != nil {
		setError(fmt.Errorf("git init: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_git_repo_is_bare
func kubo_git_repo_is_bare(handle uint64) int64 {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	cfg, err := repo.Config()
	if err != nil {
		setError(fmt.Errorf("git config: %w", err))
		return -1
	}

	if cfg.Core.IsBare {
		return 1
	}
	return 0
}

//export kubo_git_repo_branches
func kubo_git_repo_branches(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	iter, err := repo.Branches()
	if err != nil {
		setError(fmt.Errorf("git branches: %w", err))
		return nil
	}
	defer iter.Close()

	var names []string
	for {
		ref, err := iter.Next()
		if err != nil {
			break
		}
		names = append(names, ref.Name().Short())
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_git_repo_remotes
func kubo_git_repo_remotes(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	remotes, err := repo.Remotes()
	if err != nil {
		setError(fmt.Errorf("git remotes: %w", err))
		return nil
	}

	var names []string
	for _, r := range remotes {
		names = append(names, r.Config().Name)
	}

	setError(nil)
	return C.CString(strings.Join(names, "\n"))
}

//export kubo_git_repo_create_branch
func kubo_git_repo_create_branch(handle uint64, name *C.char, commit_hash *C.char) int64 {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	hash := plumbing.NewHash(C.GoString(commit_hash))
	ref := plumbing.NewHashReference(plumbing.NewBranchReferenceName(C.GoString(name)), hash)
	if err := repo.Storer.SetReference(ref); err != nil {
		setError(fmt.Errorf("git create branch: %w", err))
		return -1
	}

	setError(nil)
	return 0
}

//export kubo_git_repo_commit_lookup
func kubo_git_repo_commit_lookup(handle uint64, hash *C.char) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	commit, err := repo.CommitObject(plumbing.NewHash(C.GoString(hash)))
	if err != nil {
		setError(fmt.Errorf("git commit lookup: %w", err))
		return nil
	}

	msg := commit.Message
	setError(nil)
	return C.CString(msg)
}

//export kubo_git_repo_tree_entries
func kubo_git_repo_tree_entries(handle uint64, hash *C.char) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	tree, err := repo.TreeObject(plumbing.NewHash(C.GoString(hash)))
	if err != nil {
		setError(fmt.Errorf("git tree lookup: %w", err))
		return nil
	}

	var parts []string
	for _, e := range tree.Entries {
		parts = append(parts, fmt.Sprintf("%s\t%s", e.Name, e.Hash.String()))
	}

	setError(nil)
	return C.CString(strings.Join(parts, "\n"))
}

//export kubo_git_repo_blob_read
func kubo_git_repo_blob_read(handle uint64, hash *C.char, out **C.uint8_t, outLen *C.size_t) int64 {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return -1
	}

	blob, err := repo.BlobObject(plumbing.NewHash(C.GoString(hash)))
	if err != nil {
		setError(fmt.Errorf("git blob lookup: %w", err))
		return -1
	}

	reader, err := blob.Reader()
	if err != nil {
		setError(fmt.Errorf("git blob reader: %w", err))
		return -1
	}
	defer reader.Close()

	data, err := io.ReadAll(reader)
	if err != nil {
		setError(fmt.Errorf("git blob read: %w", err))
		return -1
	}

	if len(data) == 0 {
		*out = nil
		*outLen = 0
	} else {
		*out = (*C.uint8_t)(C.CBytes(data))
		*outLen = C.size_t(len(data))
	}

	setError(nil)
	return 0
}

//export kubo_git_repo_status
func kubo_git_repo_status(handle uint64) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	wt, err := repo.Worktree()
	if err != nil {
		setError(fmt.Errorf("git worktree: %w", err))
		return nil
	}

	status, err := wt.Status()
	if err != nil {
		setError(fmt.Errorf("git status: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(status.String())
}

//export kubo_git_repo_diff_trees
func kubo_git_repo_diff_trees(handle uint64, old_hash *C.char, new_hash *C.char) *C.char {
	gitReposMu.RLock()
	repo, ok := gitRepos[handle]
	gitReposMu.RUnlock()

	if !ok {
		setError(fmt.Errorf("invalid git handle %d", handle))
		return nil
	}

	oldTree, err := repo.TreeObject(plumbing.NewHash(C.GoString(old_hash)))
	if err != nil {
		setError(fmt.Errorf("git old tree lookup: %w", err))
		return nil
	}

	newTree, err := repo.TreeObject(plumbing.NewHash(C.GoString(new_hash)))
	if err != nil {
		setError(fmt.Errorf("git new tree lookup: %w", err))
		return nil
	}

	changes, err := oldTree.Diff(newTree)
	if err != nil {
		setError(fmt.Errorf("git diff tree: %w", err))
		return nil
	}

	patch, err := changes.Patch()
	if err != nil {
		setError(fmt.Errorf("git patch: %w", err))
		return nil
	}

	setError(nil)
	return C.CString(patch.String())
}

//export kubo_git_push
func kubo_git_push(path *C.char, remote_name *C.char) int64 {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git push open: %w", err))
		return -1
	}
	remote, err := repo.Remote(C.GoString(remote_name))
	if err != nil {
		// Try default remote
		remote, err = repo.Remote("origin")
		if err != nil {
			setError(fmt.Errorf("git push remote: %w", err))
			return -1
		}
	}
	err = remote.Push(&git.PushOptions{})
	if err != nil && err != git.NoErrAlreadyUpToDate {
		setError(fmt.Errorf("git push: %w", err))
		return -1
	}
	setError(nil)
	return 0
}

//export kubo_git_fetch_all
func kubo_git_fetch_all(path *C.char) int64 {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git fetch open: %w", err))
		return -1
	}
	remotes, err := repo.Remotes()
	if err != nil {
		setError(fmt.Errorf("git fetch remotes: %w", err))
		return -1
	}
	if len(remotes) == 0 {
		setError(fmt.Errorf("git fetch: no remotes configured"))
		return -1
	}
	for _, remote := range remotes {
		name := remote.Config().Name
		if err := repo.Fetch(&git.FetchOptions{RemoteName: name}); err != nil && err != git.NoErrAlreadyUpToDate {
			setError(fmt.Errorf("git fetch %s: %w", name, err))
			return -1
		}
	}
	setError(nil)
	return 0
}

//export kubo_git_blame
func kubo_git_blame(path *C.char, file_path *C.char) *C.char {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git blame open: %w", err))
		return nil
	}
	head, err := repo.Head()
	if err != nil {
		setError(fmt.Errorf("git blame head: %w", err))
		return nil
	}
	commit, err := repo.CommitObject(head.Hash())
	if err != nil {
		setError(fmt.Errorf("git blame commit: %w", err))
		return nil
	}
	blame, err := git.Blame(commit, C.GoString(file_path))
	if err != nil {
		setError(fmt.Errorf("git blame: %w", err))
		return nil
	}
	type blameLine struct {
		Author string `json:"author"`
		Name   string `json:"name"`
		Text   string `json:"text"`
		Date   int64  `json:"date"`
		Hash   string `json:"hash"`
	}
	var lines []blameLine
	for _, l := range blame.Lines {
		lines = append(lines, blameLine{
			Author: l.Author,
			Name:   l.Author,
			Text:   l.Text,
			Date:   l.Date.Unix(),
			Hash:   l.Hash.String(),
		})
	}
	result := map[string]interface{}{
		"path":  blame.Path,
		"rev":   blame.Rev.String(),
		"lines": lines,
	}
	jsonBytes, err := json.Marshal(result)
	if err != nil {
		setError(fmt.Errorf("git blame marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(jsonBytes))
}

//export kubo_git_log
func kubo_git_log(path *C.char, max_count C.int) *C.char {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git log open: %w", err))
		return nil
	}
	logOptions := &git.LogOptions{}
	commits, err := repo.Log(logOptions)
	if err != nil {
		if err == plumbing.ErrReferenceNotFound {
			setError(nil)
			return C.CString("[]")
		}
		setError(fmt.Errorf("git log: %w", err))
		return nil
	}
	defer commits.Close()

	type commitInfo struct {
		Hash    string `json:"hash"`
		Message string `json:"message"`
		Author  string `json:"author"`
		Email   string `json:"email"`
		Date    int64  `json:"date"`
	}
	var result []commitInfo
	count := 0
	max := int(max_count)
	if max <= 0 {
		max = 50
	}
	err = commits.ForEach(func(c *object.Commit) error {
		if count >= max {
			return io.EOF
		}
		result = append(result, commitInfo{
			Hash:    c.Hash.String(),
			Message: c.Message,
			Author:  c.Author.Name,
			Email:   c.Author.Email,
			Date:    c.Author.When.Unix(),
		})
		count++
		return nil
	})
	if err != nil && err != io.EOF {
		setError(fmt.Errorf("git log iterate: %w", err))
		return nil
	}
	jsonBytes, err := json.Marshal(result)
	if err != nil {
		setError(fmt.Errorf("git log marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(jsonBytes))
}

//export kubo_git_tags
func kubo_git_tags(path *C.char) *C.char {
	repo, err := git.PlainOpen(C.GoString(path))
	if err != nil {
		setError(fmt.Errorf("git tags open: %w", err))
		return nil
	}
	tags, err := repo.Tags()
	if err != nil {
		setError(fmt.Errorf("git tags: %w", err))
		return nil
	}
	defer tags.Close()

	var result []string
	err = tags.ForEach(func(ref *plumbing.Reference) error {
		result = append(result, ref.Name().Short())
		return nil
	})
	if err != nil {
		setError(fmt.Errorf("git tags iterate: %w", err))
		return nil
	}
	jsonBytes, err := json.Marshal(result)
	if err != nil {
		setError(fmt.Errorf("git tags marshal: %w", err))
		return nil
	}
	setError(nil)
	return C.CString(string(jsonBytes))
}
