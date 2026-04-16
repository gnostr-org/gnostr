#!/usr/bin/env bash

git add scripts/git-legit-verbose-tests.sh

# Colors for scannability
CYAN='\033[0;36m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}=== git-legit Verbose Stress Test (Violation Check) ===${NC}"

cargo run --bin git-legit -- -m "---New Test---"

make_weeble_blockheight_wobble() {
mkdir -p ./weeble
mkdir -p ./weeble/blockheight
mkdir -p ./weeble/blockheight/wobble
git add ./weeble

echo "$(gnostr --weeble)" > ./weeble/weeble
echo "$(gnostr --blockheight)" > ./weeble/blockheight/blockheight
echo "$(gnostr --wobble)" > ./weeble/blockheight/wobble/wobble
git add ./weeble
}
make_weeble_blockheight_wobble
# 1. Test Shell Injection & Metacharacters
# This checks if your Rust binary properly treats these as literals 
# or if they accidentally execute/break the gnostr call.
echo -e "${YELLOW}Testing Shell Metacharacters...${NC}"
cargo run --bin git-legit -- -m "Metachars: & | ; < > ( ) $  ! # [ ] { } * ? "

make_weeble_blockheight_wobble
# 2. Test Git Forbidden Patterns
# Patterns like '..' or '~' can break branch names/refs, but should be fine in a message.
echo -e "${YELLOW}Testing Git Sensitive Sequences...${NC}"
cargo run --bin git-legit -- -m "Git Refs: HEAD~1 ^master:path/to/file .. / : @ { "

make_weeble_blockheight_wobble
# 3. Test Control Characters and Whitespace
# Validates your \n, \t, and \r (now \n) logic under heavy load.
echo -e "${YELLOW}Testing Control Sequence Bloat...${NC}"
cargo run --bin git-legit -- -m "Line1\n\n\t\tDoubleTabbed\n\rCarriageToNewline\n\t\\\\LiteralBackslash"

make_weeble_blockheight_wobble
# 4. Test Multi-Line Violation (The "Double Dash" and "Comment" test)
# In standard git, '#' at the start of a line is a comment. We check if your miner preserves it.
echo -e "${YELLOW}Testing Comment and Flag Preservation...${NC}"
cargo run --bin git-legit -- -m "# This should not be a comment" -m "--not-a-flag" -m "Summary with 'quotes' and \"double quotes\""

make_weeble_blockheight_wobble
# 5. The "Binary/High-Bit" Test (UTF-8)
# Since you're dealing with Nostr and Bitcoin, emojis and UTF-8 symbols are vital.
echo -e "${YELLOW}Testing UTF-8 and Emojis...${NC}"
cargo run --bin git-legit -- -m "Sovereign ⚡ Bitcoin ₿ Nostr 🤙"

make_weeble_blockheight_wobble
# Testing a P2WSH (SegWit) script hex and a Taproot output
cargo run --bin git-legit -- -m "OP_PUSHBYTES_32 79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798" \
    -m "witness_v1: 51200000000000000000000000000000000000000000000000000000000000000000"

make_weeble_blockheight_wobble
# Testing raw JSON payload for a Kind 1 event
cargo run --bin git-legit -- -m '{"kind":1,"content":"Sovereign Commit","tags":[["t","bip64mod"]],"pubkey":"randy_pubkey"}'# Note: Using your unescape logic to pass a "null" placeholder if you ever implement \0

make_weeble_blockheight_wobble
cargo run --bin git-legit -- -m "Data_Start\n\0\nData_End"
# This should render the word "DANGER" in red in your terminal log

make_weeble_blockheight_wobble
cargo run --bin git-legit -- -m "Status: \x1b[31mDANGER\x1b[0m - Linker Mismatch detected."

make_weeble_blockheight_wobble
cargo run --bin git-legit -- -m "---EndTest---"

echo "\n${CYAN}=== Stress Test Complete. Check 'git log -1' for results ===${NC}"
