//go:build linux && testrunmain

package main

import (
	"os/exec"
	"syscall"
)

func setPdeathsig(p *exec.Cmd) {
	p.SysProcAttr = &syscall.SysProcAttr{
		Pdeathsig: syscall.SIGTERM,
	}
}
