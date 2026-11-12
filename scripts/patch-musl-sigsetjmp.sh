#!/usr/bin/env bash
# Patch rust's self-contained musl libc.a for aarch64-unknown-linux-musl.
#
# musl's aarch64 sigsetjmp starts with `cbz x1,setjmp` (musl
# src/signal/aarch64/sigsetjmp.s). That encodes as R_AARCH64_CONDBR19, a
# conditional branch with only +-1MB reach that neither ld.bfd nor lld can
# thunk. In large static binaries the linker places setjmp and sigsetjmp
# more than 1MB apart and the final link dies with:
#
#   relocation truncated to fit: R_AARCH64_CONDBR19 against symbol `setjmp'
#
# This is a known, still-unfixed musl quirk (musl 1.2.5 fixed only the sh
# variant). Reassembling sigsetjmp.s with the CBZ split into
# `cbnz x1,1f; b setjmp; 1:` restores correctness: the unconditional `b`
# has +-128MB reach. Semantics are unchanged.
#
# Idempotent: if sigsetjmp.lo no longer contains a CONDBR19 relocation the
# script exits quietly. Requires GNU ar/readelf and an aarch64 assembler
# (binutils-aarch64-linux-gnu on Debian/Ubuntu, or the musl cross
# toolchain's *-as). Only the rustup-managed self-contained libc.a for the
# given target is modified.
set -euo pipefail

TARGET="${1:-aarch64-unknown-linux-musl}"
SYSROOT="${RUST_SYSROOT:-$(rustc --print sysroot)}"
LIBC_A="$SYSROOT/lib/rustlib/$TARGET/lib/self-contained/libc.a"
AS="${AARCH64_AS:-aarch64-linux-gnu-as}"

if [ ! -f "$LIBC_A" ]; then
  echo "patch-musl-sigsetjmp: $LIBC_A not found; is the $TARGET std installed?" >&2
  exit 1
fi
if ! command -v "$AS" >/dev/null 2>&1; then
  echo "patch-musl-sigsetjmp: assembler '$AS' not found." >&2
  echo "  Debian/Ubuntu: sudo apt-get install binutils-aarch64-linux-gnu" >&2
  echo "  or set AARCH64_AS to an aarch64-*-as on PATH." >&2
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
( cd "$tmp" && ar x "$LIBC_A" sigsetjmp.lo )

# Match CONDBR relocations; readelf truncates the type column (e.g.
# "R_AARCH64_CONDBR1"), so match the stem.
if ! readelf -r "$tmp/sigsetjmp.lo" | grep -q 'CONDBR'; then
  echo "patch-musl-sigsetjmp: sigsetjmp.lo has no CONDBR19; already patched, nothing to do"
  exit 0
fi

cat > "$tmp/sigsetjmp.s" <<'EOF'
# musl src/signal/aarch64/sigsetjmp.s with one fix:
#   cbz x1,setjmp  ->  cbnz x1,1f ; b setjmp ; 1:
# so the tail jump to setjmp no longer relies on a CONDBR19 (+-1MB)
# relocation that linkers cannot thunk; the unconditional b has +-128MB.
.global sigsetjmp
.global __sigsetjmp
.type sigsetjmp,%function
.type __sigsetjmp,%function
sigsetjmp:
__sigsetjmp:
	cbnz x1,1f
	b setjmp
1:

	str x30,[x0,#176]
	str x19,[x0,#176+8+8]
	mov x19,x0

	bl setjmp

	mov w1,w0
	mov x0,x19
	ldr x30,[x0,#176]
	ldr x19,[x0,#176+8+8]

.hidden __sigsetjmp_tail
	b __sigsetjmp_tail

.section .note.GNU-stack,"",@progbits
EOF

"$AS" -o "$tmp/sigsetjmp.lo" "$tmp/sigsetjmp.s"

if readelf -r "$tmp/sigsetjmp.lo" | grep -q 'CONDBR'; then
  echo "patch-musl-sigsetjmp: patched object still contains CONDBR19?!" >&2
  exit 1
fi

( cd "$tmp" && ar r "$LIBC_A" sigsetjmp.lo )
echo "patch-musl-sigsetjmp: patched $LIBC_A"
