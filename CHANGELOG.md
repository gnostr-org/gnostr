
### src/lib/sub_commands/git.rs

* refactor tag logic (cb43b48)



### Bug Fixes

* Correctly handle DEPTH variable in gh-cancel-queue.sh (9f82e37)

* improve error handling and argument parsing (0006b31)



### Build System

* refactor (fc61e51)

* nip44.vectors.json:sync/hash (5daddc7)

* let _ = git_commit(dir_path); (64e586e)



### Cargo.toml

* remove udeps (6ed065a)

* re-add gnostr_qr (b0768b1)

* build deps (94b6851)



### Documentation

* update README and project structure (0004551)

* add usage documentation to README (000406a)



### Features

* Add git info subcommand and display local git details (41e4955)



### README.md

* update (b130011)



### Refactoring

* Use internal weeble, wobble, blockheight functions for git tag (a79a823)

* clean up command-line arguments (0004efe)



### TODO

* fix tests:bypassed for now (bdf1ffd)



### Testing

* should panic (52d9532)

* test_websocket_connection_and_message_echo (512b02b)

* skip test if tmux is not installed (000fd21)

* gnostr chat (c33f9b6)



### gh

* intermediate:align deps (60bf13d)



### gnostr/gh

* initial impl (a671745)

* align deps (0d5fc0f)



### legit/README.md

* doc command syntax (2d611f0)



### screenshot.rs

* testing usage (f254c31)



### src

* more tests (f7725aa)

* more tests (5515ef9)

* more tests (4221b78)

* more tests (7f9cc7a)

* more tests (8ad6ae8)



### src/bin/capture_tui.rs

* osascript (837d4f8)



### src/bin/git-chcekout-b/pr.rs

* remove (6fc8be6)



### src/bin/gnostr-sha256.rs

* test hello_panic_query (0c99e40)



### src/bin/screenshot.rs

* clap impl (e99f52a)



### src/lib/cli.rs

* src/main.rs (cab4831)



### src/lib/mod.rs

* remove eprintln! (a92f903)



### src/lib/sub_commands/bech32_to_any.rs

* json output (79245c7)

* detect bech32 prefix (5acdca5)

* use crate::types (ba88577)



### src/lib/sub_commands/git.rs

* gnostr git --tag (ab0442e)

* --tag <empty_string> (9936b85)

* checkout branch/pr (0e31d2c)

* checkout branch/pr (101657a)

* more tests (51f31ea)

* more tests (8d0f907)

* more tests (822df09)

* more tests (86202c1)

* gnostr git --serve-ssh (5b0ddd6)

* clean up --help (3a9266e)

* git fallback (46ce549)

* add --tag-version (0ff1b9f)

* add --tag-pr-version (1c84739)



### src/lib/sub_commands/login.rs

* clean up help (86a9332)



### src/lib/sub_commands/tui.rs

* unwrap_or(..into()) (17c4e62)



### src/lib/types

* migrate:refactor (188bc8f)

* tests (a95d903)



### src/lib/utils/mod.rs

* bypass test_get_current_working_dir (429300f)



### src/lib/utils/screenshot.rs

* add unix timestamp (264ddde)

* add weeble-blockheight-wobble to timestamp (bdb0aee)

* weeble-blockheight-wobble (46f3e0f)



### Testing

* dont ignore tests (6efce48)

* bypass gitsh test (14b77fd)



### Testing

* fix test (d433eb8)



### Cargo.toml

* remove sccache as build dep (fb9b241)



### Testing

* passing (902f3c2)



### cargo-dist

* update (c642d3c)



### .cargo/config.toml

* fix sscache typo (419b61f)



### .github/workflows/gnostr-bot-matrix.yml

* add gitworkshop remote (5552497)

* cargo -j8 (ee0b835)

* cgnostr ngit --help (0178098)

* gnostr --help (517feb1)

* cargo -j8 (6f3b855)

* cargo -j8 (02bcb05)

* cargo -j8 (300f88f)



### .github/workflows/run-all-workflows.yml

* v4 (558ab2c)



### Bug Fixes

* Suppress all warnings in .cargo/config.toml (74b9240)

* Improve error reporting from gitminer (00014ef)



### Build System

* make src/empty (40aa913)

* git_commit (df8b5e6)

* apply cargo fmt (0e450ed)



### Cargo.lock

* gnostr/relay:v1908.924232.84720 (2e9dadc)



### Cargo.toml

* build optimizations (aa103dd)



### Features

* Enable cross-platform Docker builds in ghcr.io.yml (8a18810)

* Add gnostr chat topic connectivity test (717cc53)



### README.md

* restore (d8b2a95)



### Refactoring

* replace simple-websockets with local gnostr dependency (50b91d0)



### Testing

* repo unborn state (378cdf0)

* use git2@0.18.3:explicitly (15ae2a6)

* rexpect:splice:intermediate (885cc8c)

* rexpect:splice:intermediate (8b2c4c5)

* rexpect:splice:intermediate (90e71b3)

* rexpect:splice:intermediate (2e0ca08)

* rexpect:splice:intermediate (c7f6558)

* rexpect:splice:intermediate (0f0af4d)

* rexpect:splice:intermediate (60b20e3)

* rexpect:splice:intermediate (55695e6)

* rexpect:splice:intermediate (386e2a1)

* rexpect:splice:intermediate (3734db7)

* rexpect:splice:intermediate (b282b71)

* rexpect:splice:intermediate (0a6afb1)

* passing:rexpect:merge complete (73f596f)

* cleanup (b3e8bdb)

* cfg unix flags (14056f9)

* intermediate (7efe5ea)

* remove (886de83)

* repo_path (68d0113)

* intermediate (ad4ac40)

* create test commit (0003743)

* use gnostr:ws and tokio_tungstenite (f5e1b8a)

* refactor for tokio (732cb00)

* refactor for tokio (ae710bc)

* refactor for tokio (886c05f)

* refactor for tokio (dc5bc5f)



### examples

* tests (714f069)

* tests (6dfccc8)

* tests (4c5f80d)

* tests (4cfeccc)

* clean up (3c21fc1)



### examples/repl.rs

* remove (d79ee8b)



### gnostr/legit

* v1911.921558.782146 (a4177e0)

* src/main/gitminer.rs (00056cd)

* worker:add a new line before nonce (0000225)

* worker:add a new line before nonce (0006a92)

* worker:add a new line before nonce (00089ef)

* worker:add a new line before nonce (000e26c)

* worker:add a new line before nonce (000e376)



### gnostr/relay

* v1908.924232.84720 (468901b)



### make

* cargo-sort (9bc4bbd)

* gh-act-run-all (70d7ac7)

* gnostr-chat (0acd275)



### rexpect

* intermediate:merge:remove dep (c19a8df)



### src

* remove unused imports (9033972)

* remove unused imports (2bab54f)

* remove deprecated methods (ae4fa31)

* remove deprecated methods (cbca8dc)

* tests (f346025)

* tests (c65c4b5)

* tests (ce2ffb4)

* tests (f6e4a35)

* tests (f8d0300)

* tests (bef0eec)



### src/bin

* clean up (07e7ca0)



### src/bin/git-remote-nostr

* rename (80dc97a)



### src/lib

* disable cube (c946399)



### src/lib/legit/command.rs

* gnostr_event (0a1cdc1)



### src/lib/mod

* cube (38f4e66)



### src/lib/p2p

* modules (ca3b5c6)

* modules (66f434b)

* modules (d63a2f0)

* modules (4dc4209)

* modules (e71f56e)

* modules (d8e0c43)

* modules (b9bee40)

* modules (816fbe7)

* modules (d90bf9d)

* modules (bd3e6a5)

* modules (7ea1dc1)

* modules (d842a0d)

* modules (080d3f8)

* README.md (5236bc1)

* tests (c74dd03)

* tests (721c594)

* chat:behavior (a964511)



### src/lib/sub_commands

* re-add:Client (33efb2e)

* re-add:Connect (a1ce1e5)



### src/main.rs

* tests:error capture (3adb93a)



### v0.0.0

* reminder (5e4569f)



### .github/workflows/run-all-workflows.yml

* act configs (64743be)



### Build System

* detect if RUSTC_WRAPPER is set (6476048)



### Testing

* passing (150ed06)

* nocapture logging (452827e)

* legit sub_command test (8e95959)

* legit sub_command test (ed4d4ca)



### gnostr/legit

* know working version (59ace1e)



### legit/src/gitminer.rs

* ensure_gnostr_dirs_exist (c7ad8df)



### legit/src/worker.rs

* nocapture logging (14958b5)

* .gnostr/blobs/reflog mkdir fix (93e713c)



### src/bin/generate-server-config.rs

* crossplatform refactor:windows (318b5cb)



### src/bin/gnostr-git**

* print tag_name (ccf7927)



### src/bin/gnostr-git-checkout-*

* print branch_name (6ccb3d9)



### src/lib/p2p/chat/tests/p2p_chat_integration_tests.rs

* p2p tests (3fa824f)



### src/lib/sub_commands/query.rs

* TODO (83c7b04)



### src/lib/ws.rs

* tests (ba6a5fc)

* test:ignore (edb8df4)



### .github/workflows/get_time.yml

* seconds (841d586)



### .github/workflows/gnostr-bot-macos/windows.yml

* even/odd (7ab992d)



### .github/workflows/gnostr-bot-ubuntu.yml

* even/odd (040a823)



### .github/workflows/gnostr-bot.yml

* fetch tags false (fa676c5)

* linuxbrew (4e5e354)

* linuxbrew (158936e)

* EVENT announcment (1b9abfd)



### Cargo.toml

* use gnostr-grammar (db1ad89)



### gnit

* working (184c0fe)



### gnostr-gnit/grammar

* initial config (660a65d)



### gnostr-legit

* refactor (d579df5)



### legit

* integration (be7d140)



### src/bin/gnostr-git

* ignore some tests (c0a7a47)



### src/bin/gnostr-git-checkout-b.rs

* tests:use args_vector (6386b38)



### src/bin/gnostr-git-checkout-pr.rs

* tests:use args_vector (38068a6)



### src/lib/sub_commands/query.rs

* pub ids:short:-i (1c3b8f6)

* arg:relay:short:-r (389ad84)



### templates/base.html

* gnostr (adbba40)



### Cargo.toml

* v1912.921034.55846 (2e6dba6)



### src/bin/gnostr-git-tag

* tests (7bcd479)



### src/lib/sub_commands/custom_event.rs

* verbose nip34 help (74dd9ad)



### .github/workflows/release.yml

* revert (628665d)



### plan-dist-manifest.json

* update (c2fb64e)



### src/bin/gnostr-git-*

* tests:intermediate (fc3f411)



### src/bin/gnostr-git-checkout-b.rs

* passing (8295846)



### Cargo.lock

* v1912.920888.748912 (3d09e0b)



### dist-workspace.toml

* 0.30.0 (b02fab8)



### src/bin/README.md

* update (7f99cdb)



### src/bin/gnostr-git-tag-version.rs

* update tag version logic (c91eff7)



### src/bin/gnostr-git

* commands (72a8bc6)



### .github/workflows/release.yml

* handle gnostr tag format (1662985)



### .github/workflows/release.yml

* debug (2ed1a9d)



### src/main.rs

* revert (0cffc51)



### .cargo/config.toml

* target-dir = '.gnostr' (c645bce)



### Cargo.toml

* use gnostr-crawler = { version = 0.0.15, path = crawler } (d162836)

* use time@0.3 (283fcd4)

* serde 1.0.203 (0e403b5)

* v0.0.125 (3c0a372)



### Testing

* ignore (2d85d7b)

* fixed (4c0a834)

* intermediate:passing (cdf0bdf)

* add (8a77b64)

* refactor for "git-ssh" command (73d624b)

* test_gitsh_command_error_output (d606263)

* ignore a few tests for now (7b5f4a2)



### crawler/Cargo.toml

* v0.0.15 (42cd5af)



### examples

* remove hashlist/haslist-padded.rs (c9e3ef2)



### make

* gnostr-chat:use ./.gnostr/debug/gnostr chat (a735419)



### make_just.sh

* use -j $(NPROC) (5ffaafb)

* make cargo-build (97e2590)



### query/Cargo.toml

* v0.0.12 (ce5367b)



### src/bin/README.md

* add (001bd43)

* update (17cc6bf)

* update (3aa738c)

* update (e68064e)



### src/bin/git_remote_nostr

* update to nostr_0_34_1 (9ace744)



### src/bin/gnostr-blockhash.rs

* test stub (99824b4)



### src/bin/gnostr-blockheight.rs

* test stub (3b2d9fc)



### src/bin/gnostr-cube.rs

* add tests (f5cae1c)



### src/bin/gnostr-cube/query.rs

* cleanup (5cd050c)



### src/bin/gnostr-kvs.rs

* generate_close_peer_id (9e10632)



### src/lib/cli.rs

* fix import path for ArgMatches (80d3b9f)



### src/lib/core

* add (73bffa2)



### src/lib/p2p/chat/mod.rs

* remove libp2p::StreamProtocol (e5706e2)



### src/lib/p2p/chat/msg.rs

* wrap_text:unused vars (0987ad7)



### src/lib/p2p/mod.rs

* fix some xor logic (39e7edc)

* remove Swarm (b022a14)



### src/lib/sub_commands/chat.rs

* remove unused imports (367c394)



### src/lib/sub_commands/custom_event.rs

* add docs (3048b95)

* --help verbose (acc16c6)

* verbose --help formatting (77d300b)



### src/lib/sub_commands/gitsh.rs

* initial impl (50725c6)

* mock_ssh (b98ab8f)



### src/lib/sub_commands/query.rs

* remove unused imports (c937707)



### src/lib/sub_commands/sniper.rs

* add (5065859)



### src/lib/sub_commands/tui.rs

* remove unused imports (8bd2e8c)



### src/lib/utils/README.md

* add (dd316ed)

* update (d000dde)



### src/lib/utils/RETRY.md

* add (0d210f8)



### src/lib/utils/mod.rs

* add find_available_port and async_find_available_port (94282d5)

* remove unused imports (1c0c182)



### src/main.rs

* remove unused imports (b5ff800)



### ssh

* v0.0.1 (3790565)



### Cargo.toml

* use gnostr-asyncgit = { path = asyncgit, version = 0.0.7, default-features = false } (86039d8)



### /src/main.rs

* add gnostr relay sub_command (334ed33)



### Cargo.toml

* nightly feature stub (e33d25c)



### Testing

* intermediate:should_panic (be2e909)

* intermediate (8710b05)

* new stubs (8d092e9)

* intermediate (9d131d7)

* intermediate (c4d6c18)

* temp disable (27bd952)

* more tests (4c04c55)

* more tests (be4ace9)

* more tests (2f9ab51)

* iniial impl (1ea4417)



### app/target

* remove (7fdbe14)



### asyncgit

* some ratatui integration (2c01fde)

* v0.0.7 (8a98886)



### asyncgit/src

* escape stash view (0aa7277)



### asyncgit/src/gitui/state/mod.rs

* screen ordering (c14870f)



### asyncgit/src/sync/commits_info.rs

* serialize/deserialize (58f2d62)

* serialize/deserialize (ea54c0d)



### crawler/src/processor.rs

* add LOCALHOST_8080 (24d6b0d)



### examples/ngit.rs

* query (444f86c)



### gnostr/core

* checkpoint (8720c4b)



### gnostr_core

* test refactor:intermediate (504a39f)

* test refactor:intermediate (51b2449)

* test refactor:intermediate (4bee9e2)



### handle_internal_event

* intermediate (ac8a508)



### make

* gnostr-chat:use debug (5d70aa2)



### make_just.sh

* more commands (3790caf)



### qr/Cargo.toml

* remove a workspace artifact (2289f75)



### src

* intermediate:cargo insta snapshots broken (1416d54)

* cargo insta:intermediate refactoring (219463a)



### src/asyncjob/mod.rs

* intermediate:fix cargo insta (63840e4)

* ignore  test_overwrite: (909b11f)



### src/bin/gnostr-legit.rs

* fix convert_to_u32 (a3980ca)



### src/bin/gnostr-query.rs

* debug logging (d60d97c)



### src/gitui/tests/snapshots

* remove (565fc24)



### src/lib

* refactor:intermediate (6bc3c1b)

* refactor:intermediate (21ff799)



### src/lib/chat/mod.rs

* intermediate (9465200)



### src/lib/chat/p2p.rs

* message_id_fn (bdd68ba)



### src/lib/chat/title.txt

* GNOSTR (25a8d0e)



### src/lib/cli.rs

* loggin conflicts_with:initial impl (342aca3)

* add Relay sub_command (e64eeb0)



### src/lib/components/topiclist.rs

* intermediate:chat (aa13d5f)

* add chat history (4201134)

* chat histories (3608b07)



### src/lib/keys/key_list.rs

* H = help (03aa71a)



### src/lib/p2p

* more tests (553bf09)

* more tests (d6d9bd5)



### src/lib/p2p/chat

* migration:working (fa6502d)



### src/lib/p2p/kvs.rs

* refactor (cdcded2)



### src/lib/p2p/mod.rs

* async_prompt (b80b781)

* swarm.listen_on (7c10c5e)



### src/lib/sub_commands

* refactor:with tests:intermediate (c00f065)

* refactor:with tests:intermediate (32a6074)



### src/lib/sub_commands/mod.rs

* add pub mod relay and query (1152cdb)



### src/lib/sub_commands/ngit.rs

* test refactor:intermediate (9f27e93)

* intermedite (32aba75)

* disable async test_ngit_push_command and test_ngit_pull_command (72098b3)



### src/lib/sub_commands/query.rs

* intermediate test config (d5c12db)

* nip34:kind query matrix:initial impl (2e6b7d0)

* add gnostr ngit query:sub_command (07624ae)

* add LOCALHOST_8080 (3d9e0d6)



### src/lib/sub_commands/relay.rs

* empty (0e8a563)

* gnostr relay sub_commands stub (75d0c1b)



### sub_commands

* check point (f0b9f25)



### .github/workflows/gnostr-chat.yml

* apply yamllint (c87a12a)



### .github/workflows/release.yml

* fix aarch64-linux-unknonw-gnu: build (49b50fc)

* gnostr release event:inital impl (1d5ee58)

* apply yamllint (a195b87)



### Dockerfile

* gnostr repo (434706f)



### dist-workspace.toml

* add aarch64-unknown-linux-gnu (84e9d00)



### just

* docker-shared (4493483)



### make_just

* with Dockerfile config (dd15517)



### gnostr

* v0.0.123 (483cdc4)



### gnostr-query

* v0.0.11 (35c2b6e)



### Testing

* add tests (d37d683)



### app

* use gnostr-extensions (98a4f1d)



### app/relay

* remove (953aa7c)



### app/src/main.rs

* apply cargo fmt (8732b5f)



### config/gnostr.toml

* enable search (73b9b3b)



### crawler/src/processor.rs

* BOOTSTRAP_RELAY0 = ws://127.0.0.1:8080 (eec8c64)

* wss://gitnostr.com (71d258a)



### gnostr-query

* gnostr-relay:compatibility (e8f3865)



### gnostr_crawler

* v0.0.14 (f757a5d)



### query/src/cli.rs

* default nip-34 kinds (407f5a8)



### query/src/lib.rs

* quiet (97293c6)



### query/src/main.rs

* quiet (c80058b)



### relay/src/app.rs

* test_main_app_creation (52dcc46)



### relay/src/lib.rs

* add tests (6d40a07)



### relay/src/list.rs

* add tests (f2dfeea)



### relay/src/session.rs

* logging config (2991bd5)



### relay/src/setting.rs

* apply cargo fmt (0b75c31)

* add tests (6777020)

* add tests: use CONFIG (7a2aac4)



### src/bin/gnostr-blockheight.rs

* get_blockheight_sync() (692b53d)



### src/bin/gnostr-cube.rs

* apply cargo fmt (b558f18)



### src/bin/gnostr-query.rs

* output valid json (87d16d1)

* json output (f7ca80f)

* apply cargo fmt (3422d7a)

* quiet (d574928)



### src/lib.rs

* apply cargo fmt (a756ee2)



### src/lib/chat/mod.rs

* apply cargo fmt (91ae7d0)

* additional raw message value (2899d4d)

* intermediate (144ee26)

* add pub mod tests (bb68b91)



### src/lib/chat/msg.rs

* Color::Red (e0ef2b9)

* intermediate (e51c7ef)



### src/lib/chat/tests/mod.rs

* add tests (a165935)

* add tests (a260c0f)



### src/lib/cube/app.rs

* apply cargo fmt (d90bb44)



### src/lib/cube/handlers/event.rs

* apply cargo fmt (3a0c9c6)



### src/lib/cube/local_git/mod.rs

* apply cargo fmt (bf26b54)



### src/lib/cube/system_command.rs

* apply cargo fmt (b43f9f1)



### src/lib/cube/ui/mod.rs

* apply cargo fmt (5e31c06)



### src/lib/mod.rs

* get_blockheight_async (35ba025)

* get_blockheight_sync() (8d98728)



### src/lib/sub_commands/chat.rs

* migrate crate::chat::p2p (4d3447d)



### src/lib/utils/mod.rs

* add tests (d9e857f)



### Cargo.toml

* v0.0.13 (afcfd20)



### crawler/Cargo.toml

* v0.0.12 (10dbce9)



### crawler/src/bin/gnostr-sniper.rs

* load_file(relays.yaml) (d1f5ce8)



### src/bin/gnostr-unified.rs

* apply cargo fmt (1f47115)

* app nip34 filter (2dba300)



### src/lib.rs

* apply cargo fmt (8328a8d)

* refactor (1a9c8b1)



### src/main.rs

* intermediate (466917c)



### .github/workflows/gnostr-bot.yml

* markdown report (b177369)

* markdown report (daf1a27)

* markdown report (9987412)

* markdown report (fb32e2f)

* nip-34 config (cf35c05)

* nip-34 config (4ed7777)

* nip-34 config (8d22587)

* nip-34 config (52d57ec)

* nip-34 config (88c95e2)

* nip-34 config (f21d927)

* nip-34 config (f58f081)

* nip-34 config (e5dd5d7)

* nip-34 config (1452fc3)



### .github/workflows/gnostr-relay.yml

* initial impl (dd500d4)

* markdown report (fda883f)

* markdown report (d2c1c65)

* markdown report (4856a15)

* markdown report (72e5866)

* markdown report (42cc7ff)



### Cargo.toml

* v0.0.121 (4c66be7)



### README.md

* update docs (cd7d432)



### gnostr-cube

* initial impl (b26621e)



### relay/extensions/examples/demo.rs

* demo.toml (b29755a)



### relay/extentions/src/auth.rs

* apply cargo fmt (3734cb0)



### relay/extentions/src/count.rs

* apply cargo fmt (75bd435)



### relay/extentions/src/metrics.rs

* apply cargo fmt (19e0411)



### relay/extentions/src/rate_limiter

* apply cargo fmt (5e6c896)



### relay/extentions/src/search.rs

* apply cargo fmt (bc71dc2)



### src/lib/cube

* initial impl (fc26f4d)



### gnostr-relay

* extensions:v0.0.1 (33a8daa)



### relay/extensions

* gnostr-relay-extentions:initial impl (46c6719)



### .github/workflows/matrix.yml

* cargo install -vv --path . --force (9ea6398)



### make_just

* update (5257b3a)



### Cargo.toml

* v0.0.117 (aaf4eb2)



### src/lib/gnostr/msg.rs

* solarized_dark/light.rs (48290b5)



### .github/workflows/gnostr-bot.yml

* export NOTE=note1qqqwldjg7gsg5nxdhwqpn94zzdm8yh7a4ndayakvpqy50emyhv3quazgcu (69b1e83)



### Build System

* fixup os warnings (d4e4010)



### Cargo.toml

* use gnostr-crawler:v0.0.10 (bc6a7a0)



### asyncgit/src/gitui/cli/mod.rs

* docs (5531a57)



### asyncgit/src/gitui/git/remote.rs

* fix lifetime (6c95724)



### asyncgit/src/gitui/gitui_error/mod.rs

* fix docs (6b070a8)



### asyncgit/src/gitui/screen/mod.rs

* fix lifetime (5ad3155)



### asyncgit/src/sync/commit_files.rs

* cleanup comments (b64bd5b)



### asyncgit/src/sync/remotes/mod.rs

* fix lifetime (aead196)

* fix docs (5d719b1)



### asyncgit/src/sync/reword.rs

* fix lifetime (6450340)



### asyncgit/src/sync/sign.rs

* cleanup docs (d8717ca)



### crawler/Cargo.toml

* v0.0.10 (120dff3)



### crawler/src/bin/gnostr-watch.rs

* formatting (81f9067)



### crawler/src/lib.rs

* let _revwalk (61b177f)



### crawler/src/relays.rs

* de_dup (203c691)



### relays.yaml

* update (2ad824f)



### src/bin/gnostr-sniper.rs

* clean up (e893466)



### src/bin/gnostr-watch.rs

* clean up (927bc0a)



### src/lib/chat/mod.rs

* cleanup comments (9d6441d)



### src/relays.rs

* get_all (b547f2a)



### .github/workflows/matrix.yml

* gnostr-bot.yml (1624ea1)



### .gitignore

* server.toml (87dcdcf)



### 1929/909608/309784

* gnostr (23f9f97)



### Cargo.toml

* use gnostr-asyncgit:v0.0.5 (e41362c)



### asyncgit/src/sync/repository.rs

* as_path() impl (899c8d1)



### crawler/src/bin/gnostr-loris.rs

* reset (2807dc9)



### examples

* remove gnostr-weeble/wobble.rs (a2f5b40)



### examples/user-project-directories.rs

* ProjectDirs::from org gnostr gnostr (c6984a6)

* apply cargo fmt (8691735)



### genkeys.sh

* initial impl (33faf1b)



### gnostr-asyncgit

* v0.0.5 (b2f71f7)



### gnostr-legit

* intermediate:initial impl (f93f532)

* intermediate:more impl (2339291)



### maintainers.yaml

* add wss://nos.lol (6b8f7ce)



### server.toml

* basic (de404dc)

* remove (6eb9116)

* re-add (b7831dc)



### src

* apply cargo clippy fix (efc0999)



### src/bin

* intermediate (6511f77)



### src/bin/generate-server-config.rs

* initial impl (b72c280)

* add fn move_gnostr_gnit_key() (51dd13d)

* windows cfg:intermediate (af9d51f)

* return Ok(()) (e7b75ea)



### src/bin/gnostr-genssh.rs

* initial impl (957431d)



### src/bin/gnostr-weeble.rs

* tests use gnostr::global_rt::global_rt; (694627f)



### src/bin/gnostr-wobble.rs

* refactor:add sync/async and tests (44578a8)



### src/lib/blockhash.rs

* add blockhash_sync:blockhash_async (f2ec8b3)

* remove deps (e3811c7)



### src/lib/chat

* p2p:intermediate swarm config (d0c88c5)

* p2p:swarm config with args (7153c37)



### src/lib/chat/mod.rs

* listevents_subcommand (92f5a31)



### src/lib/chat/ui.rs

* intermediate (4cb4477)

* intermediate (4547fb7)

* remove deps (427c284)

* add blockheight_sync to test messages (ecb4726)



### src/lib/mod.rs

* use refactored src/lib/weeble.rs and wobble.rs (c0d46ad)

* use crate::utils::pwd::pwd (3739758)

* ProjectDirs::from("org", "gnostr", "gnostr") (19d5831)



### src/lib/p2p.rs

* remove deps (9e6214a)



### src/lib/p2p/mod.rs

* add deps (8b2dc4d)

* add Network (5f90e40)



### src/lib/sub_commands/convert_key.rs

* apply cargo fmt (96ffd13)



### src/lib/sub_commands/legit.rs

* add --repo --pow options (26cac3c)



### src/lib/sub_commands/list_events.rs

* --kinds defaults --output .git/<output>.json (1e885a2)



### src/lib/tui.rs

* remove (da465c5)

* remove (448e8b5)



### src/lib/weeble.rs

* add weeble_sync/async (17777a1)

* refactor and add tests (91f302b)



### src/lib/wobble.rs

* refactor:add wobble_sync/async and tests (00989a1)



### src/main.rs

* let gnostr_subcommands:not mut (84d4a3d)

* logging (074e464)

* intermediate (1e732fb)

* debug logging (151556b)

* debug logging (d87cf37)



### ssh_key_permissions.sh

* initial impl (ea27f71)



### tui.rs

* NOT NOT valid case (2f148f5)



### gnostr-kvs

* initial impl (2fee663)



### Cargo.toml

* use gnostr-crawler:v0.0.9 (af5d522)



### .github/workflows/matrix.yml

* use rustc 1.89 (7aea47a)



### crawler/src/bin/gnostr-loris.rs

* reset (1258782)

* reset (c020355)



### gnostr-crawler

* v0.0.9 (7160c92)



### src/bin/gnostr-loris.rs

* remove (e7bb1e8)



### src/bin/gnostr-sniper.rs

* apply cargo clippy --fix (59f11a6)



### src/bin/gnostr-watch.rs

* apply cargo clippy --fix (3263daf)



### src/lib/p2p/mod.rs

* add distribvibutedbuted-key-share dependencies (c072b85)



### src/processor.rs

* apply cargo fmt (32cfa92)



### src/relays.rs

* apply cargo clippy --fix (34f4f67)



### Cargo.toml

* add dep gnostr-query (961f431)



### app/Cargo.toml

* use gnostr version "*" (b98aecf)



### gnostr-query

* add (c592ffb)



### .github/workflows/matrix.yml

* remove run: gnostr-fetch-by-id (20adc5f)



### Cargo.toml

* apply cargo sort (bce20f3)

* add dev dep gnostr-query (97fafff)

* gnostr-query = * (8524209)



### crawler/src/processor.rs

* add/remove some BOOTSTRAP_RELAYS (203cb66)



### examples/nostr-sqlite.rs

* apply cargo fmt (fdfe71e)



### src/bin

* move some to examples (a8ecd75)



### src/bin/git_remote_nostr/push.rs

* apply cargo fmt (308a747)



### src/lib

* apply cargo fmt (b6db416)



### src/lib/args.rs

* remove (e71d141)



### src/lib/chat/mod.rs

* use crate::p2p (a7f5636)



### src/lib/chat/p2p.rs

* remove (0562aa1)



### src/lib/client.rs

* use gnostr_crawler-BOOTSTRAP_RELAYS (a5bf279)



### src/lib/mod.rs

* code cleanup (9e248fa)



### src/lib/p2p.rs

* move (fe227e9)



### .github/workflows/matrix.yml

* cargo t -vv -- --nocapture (afd6f6a)

* cargo t --no-fail-fast -vv -- --nocapture (b3780c6)

* cargo test if rustup stable (b2b4139)



### examples/bitcoin_hashes.rs

* example (06fc2d3)



### make

* fetch-by-id (c00e626)



### src/bin/gnostr-sha256.rs

* use gnostr::utils (c1ca52a)



### src/lib/client.rs

* remove:sendit.nosflare.com from relays (5d1e1f9)



### src/lib/ssh/ssh/commands.rs

* knob.repo_note (3161be4)



### src/lib/ui/style.rs

* selected_tab:Color::Gray (5ba0f7c)



### src/lib/utils.rs

* cfg(test) (1526053)

* add some pub functions (fa4ff4d)



### src/main.rs

* apply cargo fmt (d85a059)



### Cargo.lock

* v0.0.101 (9c0af6b)



### Cargo.toml

* .gitignore:.gnostr/.git (fff7971)

* add gnostr-cat:gnostr-xq:dev deps (684d2ae)

* gnostr-asyncgit:v0.0.4 (61861f3)

* add relay deps (bf517bf)



### asyncgit/src/gitui/cli/mod.rs

* apply cargo fmt (c5532cf)



### asyncgit/src/gitui/gitui_error/mod.rs

* apply cargo fmt (53738b8)



### asyncgit/src/sync/commit.rs

* pub struct SerializableCommit (6f59c6d)

* pub fn padded_commit_id (2baed3f)



### chat

* intermediate:handle nsec or hash from 'gnostr --nsec/hash <string>' (649ae52)



### examples/gnostr-chat.rs

* remove (7dab894)



### examples/tui_input.rs

* initial immpl (0947fdb)



### examples/ureq_example.rs

* initial impl (73dc8ff)



### gnostr-asyncgit

* v0.0.3 (219a77c)

* v0.0.4 (bfd955a)



### make

* gnostr-chat:remove examples/gnostr-chat.rs (a086e56)

* broadcast_event_list:nip_thirty_four_requests (891a99a)



### repo.toml

* members = ["gnostr"] (aac148c)



### server.toml

* welcome_message:extra:toml format (d828668)

* welcome_message:extra:toml format (46494f9)

* welcome_message:extra:toml format (cfda739)

* welcome_message:extra:toml format (eb28bb3)



### src/bin/fetch_by_filter.rs

* use gnostr_crawler::processor::BOOTSTRAP_RELAYS; (91c0622)

* apply cargo fmt (4d48baf)



### src/bin/git-ssh.rs

* EXAMPLEs (c6c58cf)



### src/bin/gnostr-fetch-by-kind-and-author.rs

* arg order (c719615)



### src/bin/gnostr-pull.rs

* remove (fbd21ff)



### src/bin/gnostr-verify-keypair.rs

* apply cargo fmt (d96ab33)



### src/bin/test_relay.rs

* remove (b1296d1)



### src/lib/blockheight.rs

* use crate::utils::ureq_async (82814e6)

* blockheight_sync:add (f6f4c26)

* ureq_async:ureq_sync (feecae6)

* unwrap:to_string (8997d4b)



### src/lib/chat

* app.topic:type tui_input::Input (b6d9f39)

* intermediate:Msg formatting (b838019)



### src/lib/chat/mod.rs

* set_content:with index (2695e8f)

* gossipsub:app.topic formatting (ae87eac)

* apply cargo fmt (9924ca3)

* use gnostr_asyncgit::commit::de/serialize/deserialize (d15f864)

* apply cargo fmt (176a2c1)

* use gnostr_asyncgit::sync::commit::padded_commit_id; (bb3e0bf)

* apply cargo fmt (3c9d89f)

* chat_version (1f5627b)

* env::set_var("USER", &name); (3d34f8e)

* value.send(m).await.unwrap_or(()); (341285c)

* add some log directives (6cf41fd)

* env::var("USER") (e8df8be)



### src/lib/chat/msg.rs

* set_content:index (26888f5)

* GitCommitDiff (31e4b5c)

* join:weeble/blockheight/wobble (4e376da)



### src/lib/chat/p2p.rs

* set_content with index (4d607ef)

* debug! begin/end loop (9e6fb38)

* pub async fn async_prompt:remove (ee1793f)

* handle.await.unwrap_or(()); (f8619c8)

* evt_loop (9d45ad9)

* apply cargo fmt (1124f7c)

* thread::sleep 250 millis (8fdc3f7)



### src/lib/chat/ui.rs

* intermediate (bed105d)

* contraints:fit git commit message (1eb659d)

* app.topic clone (d1d470c)

* test message <ESC> (72542a2)

* test message <ENTER> (06ebc11)

* Normal Mode <ENTER> message (a715c77)

* App:diffs (44577b2)

* NormalMode:backslash (ab30cc7)

* Modal Commands:begin (62bbbbc)

* KeyCode::Char('?') (08b1bbc)

* app.input_mode:handle Modal Commands (2e60287)



### src/lib/cli.rs

* remove dead code (426caec)



### src/lib/mod.rs

* weeble/wobble async (b1ab61e)

* apply cargo fmt (55c91d6)

* pub const VERSION (0c4e791)



### src/lib/ssh

* logging config (9b1f29b)



### src/lib/ssh/config/repo.rs

* remove _get_config_file_path() (72cc409)

* debug logging (2377830)



### src/lib/ssh/config/server.rs

* apply cargo fmt (3bd7e2c)



### src/lib/ssh/ssh/commands.rs

* debug logging (4f3674f)



### src/lib/ssh/ssh/mod.rs

* Handler:welcome_message:extra:toml::Table (df92fd4)



### src/lib/sub_commands/broadcast_events.rs

* args.nsec:initial impl (a2e2b81)

* debug for relay in relays (bea8afa)



### src/lib/tui.rs

* clean up code (0e57b64)



### src/lib/utils.rs

* fn byte_array_to_hex_string (9c05b38)

* pub fn parse_json (6f285e1)

* pub fn split_value_by_newline (5ef0143)

* pub fn value_to_string:pub fn split_json_string (e8a0578)

* pub async fn ureq_async (5b077c1)

* ureq_sync:apply cargo fmt (39cec1b)

* agent use from_millis(250) (9b08535)

* ureq expect (03cc744)

* docs stubs (0418364)

* clean up comment (5d38511)

* intermediate:handle ureq error (79293c4)

* code clean up (f25552f)



### src/lib/verify_keypair.rs

* initial impl (309237b)

* apply cargo fmt (eaf6b07)



### src/lib/weeble.rs

* use log::debug (1dad73d)



### src/lib/wobble.rs

* use log::debug (804b3f6)



### src/main.rs

* --hash arg (b261e9c)

* env::set_var:BLOCKHEIGHT (0a50be4)



### app/relay/examples/demo.rs

* demo use config/gnostr.toml (b0843eb)



### Cargo.toml

* v0.0.100 (a34bb8f)



### examples/gnostr-chat.rs

* apply cargo fmt (5e2e8fe)



### make

* plan-dist-manifest (d58be42)

* gnostr-chat (b175dbe)

* gnostr-chat:weeble/blockheight/wobble:alias (e21430b)



### plan-dist-manifest.json

* windows-latest (020be7d)



### src/bin/gnostr-pull.rs

* apply cargo fmt (c3f532b)



### src/lib/chat/mod.rs

* refactor:if let Some(repo) (2522258)



### src/lib/sub_commands/chat.rs

* apply cargo fmt (768f31c)



### src/lib/sub_commands/tui.rs

* apply cargo fmt (8fceb8e)



### .github/workflows/matrix.yml

* simplify (9a5b5d6)



### Cargo.toml

* apply cargo sort (93cc445)



### app

* initial impl (7740cb0)

* gnostr-relay:gpui:initial impl (224f842)



### app/config/gnostr.toml

* default (c1f55e9)



### app/src/main.rs

* GnostrApp:initial impl (6707028)

* fn gui:initial impl (0ea682d)



### make

* post_event:post_from_files (cd23c47)



### plan-dist-manifest.json

* v0.0.98 (3888d1a)



### src/bin/post_from_files.rs

* args ordering (105cc3f)



### src/lib/components/topiclist.rs

* intermediate:apply cargo fmt (283e029)



### src/lib/mod.rs

* apply cargo fmt (4c428bd)



### src/lib/sub_commands/react.rs

* apply cargo fmt (1b11492)



### src/main.rs

* apply cargo fmt (00f7e8f)



### crawler

* v0.0.8 (8d403af)



### ghcr.io.yml

* gnostr tui (5edecec)



### gnostr-xq

* v0.0.3 (4b6b99e)



### .github/workflows/matrix.yml

* cargo install cargo-binstall@1.9.0 (3f36256)



### crawler/src/lib.rs

* add BOOTSTRAP_RELAY4 (1b2ad94)



### crawler/src/processor.rs

* BOOTSTRAP_RELAYS (f1f0091)



### examples/gnostr-remote.rs

* apply cargo fmt (0dfed50)



### gnostr-remote-test

* more (ae3d43f)

* gnostr react (bc4977e)



### make

* crawler-test-relays (c3dd12c)



### src/bin/bech32_to_any.rs

* remove (5986fc4)



### src/bin/fetch_by_kind_and_author.rs

* get_weeble (0ccfa0a)



### src/bin/fetch_by_kind_and_author_limit.rs

* get_weeble (7682f8d)



### src/bin/fetch_metadata.rs

* get_weeble (83b3fe0)



### src/bin/gnostr-bech32-to-any.rs

* print event id only (559db09)



### src/bin/gnostr-fetch-by-id-with-login.rs

* begin get_weeble (8918c68)



### src/bin/gnostr-fetch-by-id.rs

* get_weeble (d3c514c)

* BOOTSTRAP_RELAYS (e918d19)

* debug! (a66998e)



### src/bin/gnostr-fetch-metadata.rs

* use BOOTSTRAP_RELAYS (f501328)



### src/bin/gnostr-privkey-to-bech32.rs

* use env and no dangling new lines (fe7e96f)



### src/bin/gnostr-remote.rs

* logging config (2348763)



### src/bin/test_nip46.rs

* get_weeble (7dc1106)



### src/bin/test_relay.rs

* intermediate (08c232a)

* apply cargo fmt (4e7e1d9)



### src/lib/cli.rs

* info warn logging levels (6425cab)



### src/lib/mod.rs

* weeble/wobble_millis (5a48b3a)

* intermediate:impl Probe:test_relay (30671d9)

* debug! logging config (34f559e)



### src/lib/popups/inspect_commit.rs

* InspectCommitPopup:comment (7d3bfde)



### src/lib/remote

* poc for distributed/decent CI service (9cc6cd6)



### src/lib/remote/message_stream.rs

* apply cargo fmt (145cb9f)



### src/lib/remote/options.rs

* clap config (776c485)



### src/lib/remote/remote_runner.rs

* apply cargo fmt (a9fb94c)

* detect not windows:set_executable_perms (aff7cb3)



### src/lib/sub_commands/react.rs

* relays = BOOTSTRAP_RELAYS.to_vec(); (b0add5c)



### src/main.rs

* info warn logging levels (ac9a2eb)



### sub_commands

* derive Debug (d18c504)



### examples/dns_resolver.rs

* ip output (9b0e0c8)



### src/lib/dns_resolver.rs

* initial impl (9f8d449)

* working with example (c31cba9)



### Cargo.toml

* use trust_dns_resolver: (95ef5d8)



### examples/dns_resolver.rs

* use trust_dns_resolver::TokioAsyncResolver (c6bf04e)



### examples/nostr_fetch_by_filter.rs

* nostr_sdk_0_37_0 (15336b2)

* nostr_sdk_0_37_0:apply cargo fmt (166bddc)



### examples/nostr_fetch_git_patch.rs

* initial impl (87a83e4)



### src/bin/fetch_by_filter.rs

* Usage note (945a1f9)



### src/lib

* intermediate:app.rs and key_list.rs (ec73032)



### src/lib/app.rs

* bypass self.do_quit (38dd327)



### src/lib/gnostr/ui.rs

* disable Char q for Quit (90cf2b6)



### src/lib/internal.rs

* filters_to_wire:get_weeble SubscriptionId (a9621da)



### Testing

* intermediate (43b17f8)

* intermediate (d56bc95)

* src:cargo t passing (9b32ccb)



### asyncgit/Cargo.toml

* dirs@5.0.1 (20bb3a3)



### src

* apply cargo fmt (ed817b8)



### src/bin/gnostr-blockheight.rs

* bypass test (33d211b)



### Cargo.toml

* gnostr-crawler:v0.0.7 (96a08ca)

* gnostr-crawler:v0.0.7 (f9b4002)



### crawler/Cargo.toml

* v0.0.7 (724f04d)



### crawler/src/bin/gnostr-watch.rs

* command.arg:default (8d121e8)



### crawler/src/lib.rs

* processor::BOOTSTRAP_RELAYS (8658933)

* app_keys:dirty impl (20f8e49)



### crawler/src/main.rs

* BOOTSTRAP_RELAYS.get(3) (e1f56b5)

* apply cargo fmt (054ff3b)



### crawler/src/relay_manager.rs

* if url not contain (a62bceb)

* apply cargo fmt (f74bad3)



### src/bin/generate_keypair.rs

* remove (cc5730f)



### src/bin/gnostr-pi.rs

* initial impl (8d8b8d6)



### .github/workflows/matrix.yml

* fetch-by-id (c7642c3)



### make

* fetch-by-id (2b41182)



### src/bin/fetch_by_id.rs

* remove (7563391)



### src/bin/gnostr-fetch-by-id-with-login.rs

* intermediate (16c8e7b)

* intermediate:incomplete impl (b1fcc4e)



### src/lib/sub_commands/note.rs

* sub_command_args.verbose (712e805)



### types

* Cargo.toml:v0.7.6 (9f2ca95)



### src/lib/internal.rs

* RelayMessageV5 (37a4bbc)



### Cargo.toml

* exclude .gnostr (7aa31f4)



### src/bin/git-ssh.rs

* print SERVER_TOML on error (87cdd48)

* print some help (cfe10fa)



### src/lib/ssh/config/repo.rs

* apply cargo fmt (9f7419c)

* load_repo_config (eaec9dc)



### src/lib/ssh/ssh/commands.rs

* print command (36c91f9)



### src/lib/ssh/config/repo.rs

* dirs::home_dir impl (9bf16a5)



### src/lib/ssh/config/server.rs

* derive Debug (1502ec1)



### src/lib/ssh/state.rs

* State derive Debug (8bdc312)



### Build System

* update detect brew and OSs (a5b5d81)



### Cargo.toml

* add gnostr-bins deps (473915b)



### gnip44

* add (438f05c)



### src/bin

* add gnostr-bins:migration (3bba617)



### src/bin/git-ssh.rs

* static SERVER_TOML (9a16e6d)



### src/bin/gnostr-objects.rs

* remove (2f1c269)



### src/bin/gnostr-state.rs

* remove (fe43128)



### types

* gnostr-types:add (6843219)



### vendor

* remove (5c999ff)



### Cargo.toml

* if not windows:sd-notify (5d1c34a)



### src/bin/git-ssh.rs

* initial config (b9f8774)



### src/lib/ssh/mod.rs

* if not windows:sd_notify (6e28423)



### dist-workspace.toml

* add linux-musl (19c7a9b)



### .github/workflows/matrix.yml

* error formatting:src/bin/gnostr-loris.rs (fd44bbe)



### Cargo.lock

* some gnostr-loris deps (faadc40)



### Cargo.toml

* some gnostr-loris deps (2cbfc15)



### asyncgit

* doc stubs (8f36aeb)



### crawler/src/bin/gnostr-loris.rs

* args:intermediate (cfc5329)



### crawler/src/bin/gnostr-test.rs

* apply cargo fmt (cdcbd73)



### gnostr-crawler

* v0.0.6 (a5979a9)



### make_just.sh

* update (d112103)



### src/bin/gnostr-loris.rs

* intermediate (49b6115)

* intermediate:args (d79b8d1)

* parse arg address:working (dbb6c8a)

* error formatting (1b5b03c)

* output.stderr (6b4fabf)



### src/lib/components/topiclist.rs

* begin cube dashboard layout (6cd4ca2)

* apply cargo fmt (32089d6)

* some detail layout (aaea45f)



### src/lib/strings.rs

* line numbers for test/design (d21b1f8)



### Cargo.toml

* update homepage/repo (80610bc)

* gitui:intermediate (c0e298e)

* intermediate (5c5861d)

* intermediate (5e62e5a)

* cargo add --dev cargo-insta (030ab7f)



### examples/gnostr-qr.rs

* add (eee50aa)



### gitui

* initial config (0f189e2)

* intermediate (80bac99)



### gnostr-asyncgit

* v0.0.2 (52152a2)

* v0.0.2 (d5c7a08)



### make

* install:bins gnostr-asyncgit and git-tui (ec039d2)



### qr

* v0.0.7 (871c817)



### src

* crate::gitui::... (5bd1c1a)



### src/gitui

* intermediate:compiling (b0eec9a)



### src/lib.rs

* warn missing docs (f84f280)



### src/lib/cli.rs

* CAUTION! (3c21759)



### src/sync/hooks.rs

* temp passing tests assert_ne! (4deaf34)



### Cargo.lock

* gnostr-crawler:v0.0.2 (77ec591)



### Cargo.toml

* exclude vendor (8508153)

* add reqwest (fc32b09)

* cargo add --dev dep-graph (4af3d9c)

* tui-input:v0.10.1 (f663028)



### crawler

* gnostr-crawler (1e77288)

* update metadata (49284be)

* update:gnostr-sniper (4372301)



### crawler/Cargo.toml

* v0.0.5 (941abe7)



### crawler/src/bin/gnostr-sniper.rs

* intermediate (840390c)

* remove (09536fe)

* initial impl (9b5cb0a)



### crawler/src/bin/sniper.rs

* remove (7313c35)



### crawler/src/relays.rs

* self.print() (f20de38)



### examples/git-log.rs

* use nostr_sdk_0_19_1 (0f7853e)

* relay_manager (78d89db)



### examples/parse_file.rs

* all (6198205)

* all (9dc1499)



### gnostr-crawler

* v0.0.3 (51fd6d3)

* v0.0.3 (96315e1)

* v0.0.4 (7444171)

* v0.0.5 (20d46c3)



### make

* crawler:v0.0.2 (4c2f1ab)

* dep-graph (4bfab1f)



### src/bin/gnostr-sniper.rs

* replace:http/s ws/s (b78c92c)



### src/bin/gnostr-watch.rs

* intermediate:nip count (eef8611)

* intermediate (6d8abbc)

* nips json formatting (5807fba)

* await (4d4b01c)



### src/bin/sniper.rs

* replace:http/s ws/s (22dd339)

* intermediate (2240fb3)

* intermediate (9d94159)



### src/lib

* CliArgs:intermediate (efd24e9)



### src/lib..chat/ui.rs

* clean up deps (55e27e7)



### src/lib.rs

* async (03c28e9)



### src/lib/cli.rs

* add NgitCommands:GnostrCommands (dbcd32d)

* intermediate (11e6e8b)

* pub fn setup_logging/get_app_cache_path (6db19e4)



### src/lib/components/chat_details/mod.rs

* apply cargo fmt (88109e5)



### src/lib/components/mod.rs

* clean up deps (a3a4308)



### src/lib/components/topiclist.rs

* remove unsed deps (5b645a2)



### src/lib/gnostr/mod.rs

* gnostr tui:intermediate (d59903e)

* clean up deps (3096d64)

* Default for GnostrSubCommands (acdb3a4)



### src/lib/sub_command/chat.rs

* remove unused deps (10c1aa0)



### src/lib/sub_commands/text_note.rs

* code clean up (497dafd)

* apply cargo fmt (b3f749f)



### src/lib/sub_commands/tui.rs

* intermediate (052a8ac)

* migrate:intermediate (5543252)

* intrermediate (9679f13)

* loop:run_app (07f4642)



### src/lib/tui.rs

* remove unused deps (4b46726)



### src/main.rs

* code clean up (32569f7)

* intermediate:tui subcommands (de6b903)

* trace logging (03ca517)

* setup_logging (82a1eb6)

* bypass use tracing::debug (a56d14e)

* apply cargo fmt (d384367)

* match None args (3d9ba25)



### src/relays.rs

* async (982438a)



### ssrc/lib/tui.rs

* cli:intermediate (660e20c)



### theme.ron

* example theme (46155f4)



### crawler

* initial commit (1e559ab)

* git-log.rs:json output (42bb84a)

* src/main.rs:json output (172c96c)



### make

* cargo-dist (730b7fc)



### src/main.rs

* default tui (ed85d5b)



### src/lib/chat/mod.rs

* public args (7651266)



### src/lib/sub_command/chat.rs

* tracing logging impl (520409a)



### nostr_sqlite

* :SQLiteDatabase:async config (7c1fe73)



### src/lib/components/topiclist.rs

* CrossTermEvent (0f1e70f)



### src/lib/login/mod.rs

* debug logging event (e2c67b5)



### examples/nostr-sqlite.rs

* initial impl (de19693)



### src/lib/chat/ui.rs

* default:topic gnostr (3f0f02b)



### src/lib/client.rs

* add search relay (46d8851)



### src/lib/components/topiclist.rs

* being get_user_details (1901e2a)

* login::get_user_details (1248b07)



### src/bin/gnostr-chat.rs

* apply cargo fmt (ac01cff)



### src/lib/app.rs

* cargo fmt (69fbc80)



### src/lib/args.rs

* apply cargo fmt (962ddce)



### src/lib/chat

* GitCommit:Type formatting:intermediate (b9e2ae4)

* Msg formatting:intermediate (7b2a0d9)



### src/lib/components

* apply cargo fmt (567fe85)



### src/lib/mod.rs

* handle --topic arg or commit id (b400cd9)



### src/lib/popups

* apply cargo fmt (beac77c)



### src/lib/popups/submodules.rs

* apply cargo fmt (b80ee65)



### src/lib/popups/tag_commit.rs

* apply cargo fmt (4cdf858)



### src/lib/popups/taglist.rs

* apply cargo fmt (af6366d)



### src/lib/queue.rs

* apply cargo fmt (b9c5594)



### src/lib/sub_commands/chat.rs

* apply cargo fmt (69cd640)



### src/lib/tabs/home.rs

* apply cargo fmt (ed3b5d1)



### src/lib/tabs/revlog.rs

* cargo fmt (4081012)



### src/lib/tabs/stashing.rs

* apply cargo fmt (ffd7e8c)



### src/lib/tabs/status.rs

* apply cargo fmt (d5185a2)



### src/lib/tui.rs

* apply cargo fmt (30b2fb9)



### .github/workflows/release-matrix.yml

* add windows-latest (c6024d3)

* follow up (e6e5fcb)



### .gitignore

* **DS_Store (319d5ee)



### Bug Fixes

* git2:conflict (6682ee3)



### asyncgit/Cargo.toml

* gnostgr-asyncgit (7a560ff)



### chat

* ngit:Box<dyn StdError>>:impl (4e9d1a1)



### ghcr.io.yml

* config (0d35612)



### maintainers.yaml

* update (35ca5a0)



### plan-dist-manifest.json

* v0.0.60 (11e0d47)



### src/bin/gnostr-chat.rs

* apply cargo fmt (aa603c9)



### src/lib/app.rs

* OpenExternalChat:initial impl (a0fe3d9)



### src/lib/chat/mod.rs

* apply cargo fmt (d55535f)

* intermediate (d82faf3)

* LevelFilter:OFF impl (5a1c16f)



### src/lib/cli.rs

* ChatCli and ChatCommands (87594ad)



### src/lib/components/chat_details/mod.rs

* layout notes (5ca7357)



### src/lib/components/commit_details/details.rs

* Info header formatting (ae8169a)



### src/lib/components/topiclist.rs

* more_text (9618857)

* intermediate (733e4f2)

* truncate pubkey (5a8e93c)



### src/lib/keys/ley_list.rs

* open_external_chat:initial impl (21d410d)



### src/lib/login/mod.rs

* UserRelayRef:guards (b9b6a36)



### src/lib/popups/chat.rs

* initial impl (c3d7553)



### src/lib/popups/commit.rs

* show_chat:initial implt (c9d231e)



### src/lib/popups/mos.rs

* openchat:initial impl (64a2c5c)



### src/lib/popups/openchat.rs

* intial impl (72bdeb2)



### src/lib/queue.rs

* OpenExternalChat:initial impl (15911b4)



### src/lib/strings.rs

* open_chat:initial impl (f33f96c)



### src/lib/sub_commands

* no use cli::Cli (ea9c91e)



### src/lib/sub_commands/chat.rs

* --topic arg (9cd3d4c)

* chat/run:initial impl (34b0e96)

* intermediate (d685d26)



### src/lib/sub_commands/mod.rs

* add pub mod chat (34be020)



### src/lib/sub_commands/ngit.rs

* apply cargo fmt (e5ed486)



### src/lib/tui.rs

* add lifecyle notes (b7f3ab4)



### src/lib/utils.rs

* truncate_chars (b07eb43)



### src/main.rs

* add chat subcommand (ccc4d8a)

* apply cargo fmt (6e746bd)

* apply cargo fmt (ac9b377)



### topiclist.rs

* begin nostr fields (eb15b2a)



### .github/workflows/release.yml

* update cargo-dist:dirty config (2e2ecbd)



### src

* cargo clippy passing:with many warnings (150f053)

* apply cargo fmt (1794b0c)



### src/bin/chat.rs

* use asyncgit::sync::commit::{serialize_commit, deserialize_commit} (7c7c18e)

* intermediate (691d986)

* intermediate (75841e7)

* intermediate (316cb0c)

* intermediate (5beb11e)

* intermediate-working (18e0c9f)

* if args.debug OR args.trace:display keys.secret_key (7dfff63)



### src/lib/chat/mod.rs

* intermediate:migrate chat (5f16abd)

* intermediate (d0b784a)



### src/lib/components/diff.rs

* begin chat graft (beb2874)

* insert pubkey (2e9a5a3)

* pubkey formatting (b2ff46c)



### src/lib/sub_commands/generate_keypair.rs

* json output (66aaa5a)

* json formatting (4dad046)



### src/lib/tui.rs

* public tui functions (84d73b3)



### inspect_chat

* initial impl (136f7cd)



### src/lib/popups/inspect_commit.rs

* diff split notes (da1084c)



### src/lib/tabs/home.rs

* apply cargo fmt (32e7da3)



### src/bin/chat.rs

* more Cli config (56d51f0)

* --info arg (97cca26)



### src/lib/tabs/home.rs

* topiclist_title:🅖 (e6ccaab)

* TopicList:split view constraints:working (3df5c89)



### topiclist.rs

* CIRCLE_G_STR:marked (8122e62)



### .github

* from main (6f45c66)

* from main (5fc7d72)



### asyncgit/Cargo.toml

* add nostr-sdk@0.37.0 (2947ec8)



### chat_details

* ChatDetailsComponent:more impl (fcf3e6f)



### chat_details_toggle

* initial impl (44c323d)



### crates/_tui

* remove (74d0742)



### crates/ngit

* remove (f225d6e)

* remove (03acd13)



### crates/tui

* add (97896dc)

* remove (b0be388)



### crates/tui/asyncgit

* preliminary async/nostr integration (95ad9b0)



### crates/tui/src/bin/ngit.rs

* pre remove (396f4b7)



### detail

* split:better (6abfbcb)



### imtermediate

* crates/tui:provides ngit (dee29e7)



### install_script.sh

* make cargo-release-all (98ba5a6)



### make

* cargo-install-crates (9b8b925)

* cargo-release-all (0d8ec25)

* from install_script.sh (e2a1bdb)



### padded_hash

* initial impl (116e68a)



### passing

* publish dry run (c99ac22)



### src/bin/chat.rs

* impl (8480dc1)

* topic arg working (a565d4c)

* --name arg to env working (7fb39d0)



### src/lib/chat

* being impl (6e6917a)



### src/lib/components/chat_details/mod.rs

* intermediate (a96be5c)

* intermediate (bd3d580)

* 3-way split (37291d5)



### src/lib/components/mod.rs

* layout working:add notes (a30c082)



### src/lib/components/topiclist.rs

* commit keys (77aa531)

* add constraints (8bdd432)

* intermediate (b4cf508)

* intermediate (f55255f)

* split fixed (04c9199)

* apply cargo fmt (85759eb)



### src/lib/mod.rs

* chat impl (31b95eb)



### src/lib/popups/display_chat.rs

* initial impl (c25a5bc)



### src/lib/sub_commands/login.rs

* disable_cli_spinners:Option<bool> (16f889b)



### src/lib/sub_commands/set_metadata.rs

* output json (fea9142)



### src/lib/tabs/home.rs

* apply cargo fmt (be9d928)



### src/ngit

* intermediate (bb1966a)



### topiclist.rs

* symbol::EMPTY_SPACE (44954c0)



### .github/workflows/release-matrix.yml

* run gnostr custom-event -k 1 (c7b8e87)



### examples/input-to-sha256.rs

* example (8e49384)



### src/global_rt.rs

* intermediate impl (628d32b)



### src/main.rs

* --hash to args.nsec (07fce82)



### src/sub_commands/react.rs

* return json (4213c5c)



### src/sub_commands/text_note.rs

* id and bech32:json (980cccf)

* return json (79fd93a)

* --hex flag (dc5cc66)



### Build System

* detect architecture (fd9b420)



### .justfile

* config (92e99e8)



### crate

* gnostr-ngit (ff47ac4)



### gnostr-ngit

* a fork of ngit (2b35aa0)



### make-just

* config (d8325a2)



### Bug Fixes

* code cleanup (d329260)

* list events (c883e08)

* `vanity` short `-h` conflict (662a670)

* append tag key only once (#55) (5b7f266)



### Chores

* bump `nostr-sdk` (7e23355)



### Features

* Add support for bech32 encoded keys in commands (053eabb)

* print bech32 keys (59301f5)

* add generate keypair command (e00c14b)

* Add key conversion (0f3dccc)

* Add support for converting noteids (eb5ece9)

* Add support for LUD06 in metadata-update command (cce8464)

* list events save to json file (542b962)

* broadcast events from json file (06c226f)

* option to print hex (00e9275)



### NIP-28

* Add support for creating channels (e71bde9)

* Added kind 41 support (f41febc)

* Add kind 43 support (a08abf7)

* Add support for user mute (4ac6458)



### command

* gnostr:author:gnostr & 0xtr (0f25070)



### format

* cargo fmt and clippy (7eef201)



### gnostr

* v0.0.1 (fbb53aa)



### improve

* `list-events`  (#53) (18d01e2)



### improvment

* Pretty print events as json (7c12338)



### package

* gnostr (97e45c8)



### rust-toolchain.toml

* remove (1baffc3)



### src/sub_commands/set_metadata.rs

* banner_url (a9104f1)


