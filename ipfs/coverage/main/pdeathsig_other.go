//go:build !linux && testrunmain

package main

import "os/exec"

func setPdeathsig(p *exec.Cmd) {}
