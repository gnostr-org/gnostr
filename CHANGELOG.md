# Changelog

All notable changes to this project will be documented in this file.
See [Conventional Commits](https://www.conventionalcommits.org) for commit guidelines.


## [unreleased]


### Bug Fixes

- Correctly handle DEPTH variable in gh-cancel-queue.sh ([`9f82e37`](9f82e37b2fc51b4fccc09ad235417234c61e4f41))

- Improve error handling and argument parsing ([`0006b31`](0006b31098a4c349a5b032f91adb994ee7602620))


### Build System

- Refactor ([`fc61e51`](fc61e51c5549132e86b0af4d8edba8f4c2d463ea))

- Nip44.vectors.json:sync/hash ([`5daddc7`](5daddc7d3d4e9651677de899c77e79f8fd0a1ab4))

- Let _ = git_commit(dir_path); ([`64e586e`](64e586ead779a4b9a4e9b781023923b945c4cb52))


### Cargo.toml

- Remove udeps ([`6ed065a`](6ed065a50f8bcea843db36b8b8e963f5d22486fe))

- Re-add gnostr_qr ([`b0768b1`](b0768b1e0884c5d5abc86f739edba9adcfc9fddb))

- Build deps ([`94b6851`](94b68519376a42309d5cc5b0d81dbcb02822dba5))


### Documentation

- Update README and project structure ([`0004551`](00045511ab53e724e70debbdd6c6491ab978ad2b))

- Add usage documentation to README ([`000406a`](000406a0ab2e6d31c4c693bf2126380a81451154))


### Features

- Add git info subcommand and display local git details ([`41e4955`](41e4955770c4a0e057f012ceb394c3205d3bb59d))


### Refactoring

- Use internal weeble, wobble, blockheight functions for git tag ([`a79a823`](a79a823a969cb47b5ade3b48a2c992e69f7871fe))

- Clean up command-line arguments ([`0004efe`](0004efe147077ce8cc3a090bcf5b022fcc29c4c6))


### TODO

- Fix tests:bypassed for now ([`bdf1ffd`](bdf1ffdacf6e1639eb00364de80a87fc7ead6a96))


### Testing

- Should panic ([`52d9532`](52d95323c8554c68b3340a5a07a0fa067709f3df))

- Test_websocket_connection_and_message_echo ([`512b02b`](512b02b3f4ad37f05486f782374aaa1b04be5543))

- Skip test if tmux is not installed ([`000fd21`](000fd2145437d0cbb9c1461da1b62d646592acfd))

- Gnostr chat ([`c33f9b6`](c33f9b689f464b6b1fc72216ec490ad936d8ee99))


### gh

- Intermediate:align deps ([`60bf13d`](60bf13d6d12aa0135ae9cb3606fa12307fd3d215))


### gnostr/gh

- Initial impl ([`a671745`](a6717455cd21c5a4b6f8ea107db45751e4b15b54))

- Align deps ([`0d5fc0f`](0d5fc0f3c823b75e896f3d671ca5b9a5fd36dcb7))


### legit/README.md

- Doc command syntax ([`2d611f0`](2d611f0623d5f02811e6bbed75f0f5f26931a82c))


### screenshot.rs

- Testing usage ([`f254c31`](f254c31320f0114459ca31c1a8aa89bc29235c49))


### src

- More tests ([`f7725aa`](f7725aa5f8a3e8227f700dc0b6aca3ede12f9bff))

- More tests ([`5515ef9`](5515ef99d4cb75b9a9a91a2fbb1a690d7c501168))

- More tests ([`4221b78`](4221b78a6e21c92333bfa990f2ebd0f6be5f65f8))

- More tests ([`7f9cc7a`](7f9cc7a85b51469082a25c2b7281ecf5bfa9f291))

- More tests ([`8ad6ae8`](8ad6ae8c11218a8702c6c35e8bea3581a12a0372))


### src/bin/capture_tui.rs

- Osascript ([`837d4f8`](837d4f8e61a7328351b42a098b43cd4e7b53f715))


### src/bin/git-chcekout-b/pr.rs

- Remove ([`6fc8be6`](6fc8be667686aba823d6f805dcda828124d67db6))


### src/bin/gnostr-sha256.rs

- Test hello_panic_query ([`0c99e40`](0c99e403e8655c3788ff1daed9849d998529d31b))


### src/bin/screenshot.rs

- Clap impl ([`e99f52a`](e99f52ab03a5e6ae6c7765c27b3d7dd97d609c39))


### src/lib/cli.rs

- Src/main.rs ([`cab4831`](cab4831c73c7b10ecce3dea81c7ae102c211f9e8))


### src/lib/mod.rs

- Remove eprintln! ([`a92f903`](a92f90329a511ab47bad09da589ad8195a0680c9))


### src/lib/sub_commands/bech32_to_any.rs

- Json output ([`79245c7`](79245c75115129f09b48af4442c731ae880d866d))

- Detect bech32 prefix ([`5acdca5`](5acdca58beea3972d3a3d5e85a0f91159d70eb87))

- Use crate::types ([`ba88577`](ba88577b267c3ef1258ed088937503d536fbb649))


### src/lib/sub_commands/git.rs

- Gnostr git --tag ([`ab0442e`](ab0442eb7ead0155e1148fca65919c63b0ce29ea))

- --tag <empty_string> ([`9936b85`](9936b8560f0f187cf4f6c4d8e304b4a5d2d38802))

- Checkout branch/pr ([`0e31d2c`](0e31d2cce6505fa51efb91a9ddae2f2f7f4d00ed))

- Checkout branch/pr ([`101657a`](101657a2dd4a58752fbe0e362689b0d6e315b260))

- More tests ([`51f31ea`](51f31eaad8e4c1d283668fc5ec1842b5d188c037))

- More tests ([`8d0f907`](8d0f9075aee3c3f256054fee4329011da1915cd8))

- More tests ([`822df09`](822df0944fa142f523c83ddd8995ce795f9f4b8f))

- More tests ([`86202c1`](86202c1893f68c5dc2f0251acebb3cf2cf99d447))

- Gnostr git --serve-ssh ([`5b0ddd6`](5b0ddd6184f2832d167cc68dc2b4b0724b93f5a2))

- Clean up --help ([`3a9266e`](3a9266e69eeeffa37b65d4d3fba85f240ba28731))

- Git fallback ([`46ce549`](46ce5493ecbb1145a29ff0a297d36ca487e96781))


### src/lib/sub_commands/login.rs

- Clean up help ([`86a9332`](86a93326ea77072b4b3b8ff37177189c5f478e11))


### src/lib/sub_commands/tui.rs

- Unwrap_or(..into()) ([`17c4e62`](17c4e6241fd58914983faf28e5b6b6032e50b3c0))


### src/lib/types

- Migrate:refactor ([`188bc8f`](188bc8ff2488b72cab096b507a734250c582e5f1))

- Tests ([`a95d903`](a95d9030fb15bd5779970db46f026934ad3e1c86))


### src/lib/utils/mod.rs

- Bypass test_get_current_working_dir ([`429300f`](429300feb2f8e28b6d152ae48e6e001576b66ab0))


### src/lib/utils/screenshot.rs

- Add unix timestamp ([`264ddde`](264dddeef97bbf973534254594c14160931df6b2))

- Add weeble-blockheight-wobble to timestamp ([`bdb0aee`](bdb0aeec2b383108ac50910f7637d43019ab3198))

- Weeble-blockheight-wobble ([`46f3e0f`](46f3e0f085314e88256a9387c5ab0172ab06e5ce))


## [1907.924434.754357] - 2025-11-20


### Testing

- Dont ignore tests ([`6efce48`](6efce486f439a086dbbfcc416af0bcc607b3cb7a))

- Bypass gitsh test ([`14b77fd`](14b77fdbb0020410fb83b55c172c264c6ad49502))


## [1907.924432.757388] - 2025-11-20


### Testing

- Fix test ([`d433eb8`](d433eb8efb522affa63a6356dda7fcaa91929e36))


## [1907.924429.761700] - 2025-11-20


### Cargo.toml

- Remove sccache as build dep ([`fb9b241`](fb9b2411e149fde5568bcd64a64bd169a012aa70))


### Testing

- Passing ([`902f3c2`](902f3c2bd0e856d387410ce3f903acabf7428fa7))


### cargo-dist

- Update ([`c642d3c`](c642d3c27accddb96774a5b94d6aec993f72754d))


## [1908.924233.83539] - 2025-11-19


### .cargo/config.toml

- Fix sscache typo ([`419b61f`](419b61f191ce7bb95f96694a40f3121d1f057228))


### .github/workflows/gnostr-bot-matrix.yml

- Add gitworkshop remote ([`5552497`](55524977104c56cddbcca27ad6dc9a1002edd3b8))

- Cargo -j8 ([`ee0b835`](ee0b8352938a26c51c54c4294e12b4ead8f16f6e))

- Cgnostr ngit --help ([`0178098`](017809863770b4b813c1db46134252d39ca6f2f6))

- Gnostr --help ([`517feb1`](517feb1101d70cfc32b40f4afb3c7a088e64d152))

- Cargo -j8 ([`6f3b855`](6f3b85537f39e4efbf65555821c12ced262971a2))

- Cargo -j8 ([`02bcb05`](02bcb0575785b52accd3c0eb88c566defc472fd8))

- Cargo -j8 ([`300f88f`](300f88f83042b36c91409c7da1c2a88169d150bc))


### .github/workflows/run-all-workflows.yml

- V4 ([`558ab2c`](558ab2c0e1aa875ffab88811ccd80ffd5d1764ed))


### Bug Fixes

- Suppress all warnings in .cargo/config.toml ([`74b9240`](74b9240d8b7573f84fcd3469170344764b763149))

- Improve error reporting from gitminer ([`00014ef`](00014ef3a8adbcfefc97d12bb9c9afdf9adf04ca))


### Build System

- Make src/empty ([`40aa913`](40aa91361a8bc05fa5013fbe870a44b0190445a0))

- Git_commit ([`df8b5e6`](df8b5e6ee0080848f440888caa66b7defc86cfe0))

- Apply cargo fmt ([`0e450ed`](0e450edcbff29056e59aff8b36c75ed63692ddab))


### Cargo.lock

- Gnostr/relay:v1908.924232.84720 ([`2e9dadc`](2e9dadc17467cfd1ac563a56ebb26a79717bfc66))


### Cargo.toml

- Build optimizations ([`aa103dd`](aa103ddf1448b56cef3ca8cd09f61056acc0ca90))


### Features

- Enable cross-platform Docker builds in ghcr.io.yml ([`8a18810`](8a188107f044d253fbf758bc81f65e599f2de385))

- Add gnostr chat topic connectivity test ([`717cc53`](717cc53dec037d97f53d584b27bdd24d1b7d5155))


### README.md

- Restore ([`d8b2a95`](d8b2a95baec5cf6492a1701f09bf45007c7cbc43))


### Refactoring

- Replace simple-websockets with local gnostr dependency ([`50b91d0`](50b91d02e16783a3b02a081557b367980d7bfc4c))


### Testing

- Repo unborn state ([`378cdf0`](378cdf039be9a2edbcede3a868e270da3919dd78))

- Use git2@0.18.3:explicitly ([`15ae2a6`](15ae2a64cd16a725c98430c451666f1d1bb50868))

- Rexpect:splice:intermediate ([`885cc8c`](885cc8c9dedf40867b23b17e791bb63e6861a3a1))

- Rexpect:splice:intermediate ([`8b2c4c5`](8b2c4c5c96964ca070c6f91a7563c05c92319b5a))

- Rexpect:splice:intermediate ([`90e71b3`](90e71b3a535f2cd3ad003e6cc847fdf0b7ea9d4b))

- Rexpect:splice:intermediate ([`2e0ca08`](2e0ca08569cbdab4a1a611172bc670b883513507))

- Rexpect:splice:intermediate ([`c7f6558`](c7f65583b751339782febfdec5f3766d9aec98a9))

- Rexpect:splice:intermediate ([`0f0af4d`](0f0af4d6deb23a7a940cbbe135db8803dda15ca8))

- Rexpect:splice:intermediate ([`60b20e3`](60b20e3d7dfd49ef84e38307fcd4855abccdfc19))

- Rexpect:splice:intermediate ([`55695e6`](55695e66154106ed6354a9f761c2c0223a455e81))

- Rexpect:splice:intermediate ([`386e2a1`](386e2a1ba9fb1671fd7fac07b43d588c1458d649))

- Rexpect:splice:intermediate ([`3734db7`](3734db78ecd997cf534e0a9940c45bd4179266ab))

- Rexpect:splice:intermediate ([`b282b71`](b282b71cb07ccbf7a1339dc1519f2d775d898340))

- Rexpect:splice:intermediate ([`0a6afb1`](0a6afb15402d703ab9d524aca4faa4d54c924948))

- Passing:rexpect:merge complete ([`73f596f`](73f596fdf648cd348dcd2a875c8ac2d9557f4a72))

- Cleanup ([`b3e8bdb`](b3e8bdba57361d40b07564a9baa490f9ee560da2))

- Cfg unix flags ([`14056f9`](14056f9cb80f54293cd3f9f15f0c0d9d17f23b1c))

- Intermediate ([`7efe5ea`](7efe5ea48a5fa7884cce5ad0c63ff8c97d144706))

- Remove ([`886de83`](886de8394974f284e278d10c6a3169609710f196))

- Repo_path ([`68d0113`](68d011322b04a5a0590fd173cf99de4ea0b1116b))

- Intermediate ([`ad4ac40`](ad4ac40d62b941bd9a5c7a1ee1fa73edd366644d))

- Create test commit ([`0003743`](00037430ccaed0845c2928cd4bcdf6d03d6e3321))

- Use gnostr:ws and tokio_tungstenite ([`f5e1b8a`](f5e1b8a2477b0ca89cacf390876636111f9f237d))

- Refactor for tokio ([`732cb00`](732cb000cf583eebe0a67a3d665f3f5e76e81553))

- Refactor for tokio ([`ae710bc`](ae710bce320c4a7985e1b010e195c67e25307cd4))

- Refactor for tokio ([`886c05f`](886c05ff9f69d46f059a9f975a5c4cadaafac1fb))

- Refactor for tokio ([`dc5bc5f`](dc5bc5f73254a18c633ec26073d34e3a1b03796a))


### examples

- Tests ([`714f069`](714f069d43c1dc5c863955c0f275dc3fba175dbd))

- Tests ([`6dfccc8`](6dfccc8232c11093f9466b700cfcc6d91f5ae4ad))

- Tests ([`4c5f80d`](4c5f80d64fdce509d315b301d0a9caed5d69d6dd))

- Tests ([`4cfeccc`](4cfecccd1562bfa2b464958b73fa26822dd8347d))

- Clean up ([`3c21fc1`](3c21fc131b791474003f318f233fe9edf9beb5ce))


### examples/repl.rs

- Remove ([`d79ee8b`](d79ee8b3c2352b842e9fb01cba193ee70f25db67))


### gnostr/legit

- V1911.921558.782146 ([`a4177e0`](a4177e0584244ff9171623c2b1a2e7ccf9fa1235))

- Src/main/gitminer.rs ([`00056cd`](00056cdf0b8d62c4dc737a7ff73822a995bff4fc))

- Worker:add a new line before nonce ([`0000225`](000022541e6133bfca9952666b021726dd8979d5))

- Worker:add a new line before nonce ([`0006a92`](0006a92695a023b08205818594cc349267a1ece1))

- Worker:add a new line before nonce ([`00089ef`](00089ef3298f435e534a7231b2055d9a7e35d7fe))

- Worker:add a new line before nonce ([`000e26c`](000e26c12c7468aa72032e09db1d6740deb36e04))

- Worker:add a new line before nonce ([`000e376`](000e376c9fe5d7f8b5523e008772a30700cdd04a))


### gnostr/relay

- V1908.924232.84720 ([`468901b`](468901bc4947c5968959672a1834f2a12838cb82))


### make

- Cargo-sort ([`9bc4bbd`](9bc4bbd78f43b7a4be30ba56b7df7f43abab8679))

- Gh-act-run-all ([`70d7ac7`](70d7ac7b7c3b0b3bc1d790de3c2c78a4c3bf59e2))

- Gnostr-chat ([`0acd275`](0acd2753cbf66812dfcc99ced3eb48397a2223d3))


### rexpect

- Intermediate:merge:remove dep ([`c19a8df`](c19a8df4af2976446fcd8cac070113f715b91155))


### src

- Remove unused imports ([`9033972`](903397297918b6f9a6f404ff29425a1ce43ef84e))

- Remove unused imports ([`2bab54f`](2bab54ff96934d6c1f12d5461ef6d41c13db2276))

- Remove deprecated methods ([`ae4fa31`](ae4fa31c4af7b9e99d18d731fc96a976baecb82d))

- Remove deprecated methods ([`cbca8dc`](cbca8dccc67eee25ee5a707c6b18e8441532c8b6))

- Tests ([`f346025`](f346025dc1e0b7649a1e020a73f4995d808c390a))

- Tests ([`c65c4b5`](c65c4b546f13a4755f43d64d58ce9c4f030c9545))

- Tests ([`ce2ffb4`](ce2ffb42fda1c3c5c374024a9cf1a285e4d2da01))

- Tests ([`f6e4a35`](f6e4a357076c01f7d09618079f4bd7185640f949))

- Tests ([`f8d0300`](f8d0300c075313e8cedd3ac5194f7f9b8a2203ac))

- Tests ([`bef0eec`](bef0eec110f40564e353c4e75a8a42a4cca00f1d))


### src/bin

- Clean up ([`07e7ca0`](07e7ca08538a2f4e01e7df5a3a1a193181c881f2))


### src/bin/git-remote-nostr

- Rename ([`80dc97a`](80dc97a676095a511cd56134557a6e1d3fc7e785))


### src/lib

- Disable cube ([`c946399`](c9463994fb474e35d2af46c7512c9446b4bad146))


### src/lib/legit/command.rs

- Gnostr_event ([`0a1cdc1`](0a1cdc13f68ce317546d48f50cdf9c832d293239))


### src/lib/mod

- Cube ([`38f4e66`](38f4e6673288adba878288f3d3b35641b7275afb))


### src/lib/p2p

- Modules ([`ca3b5c6`](ca3b5c618121aad91ecc2e50ec061442e0243588))

- Modules ([`66f434b`](66f434b01079222839b89ce8913ea20f4871fa62))

- Modules ([`d63a2f0`](d63a2f073a164438407680c8f9065b09cbbcb063))

- Modules ([`4dc4209`](4dc4209470709abfbe68af4e91402e3c939df585))

- Modules ([`e71f56e`](e71f56e719968d41d73dd366d1f96a101b340a9a))

- Modules ([`d8e0c43`](d8e0c43f1eee21a60a2c080e3d55f9ea14adcbbf))

- Modules ([`b9bee40`](b9bee401a07458392c0028f6b9c21b935abeda46))

- Modules ([`816fbe7`](816fbe7760510e701d85ba107b96a2bf67930dbc))

- Modules ([`d90bf9d`](d90bf9dc5e7f9bc88d28a2ef0d920f50da2dd341))

- Modules ([`bd3e6a5`](bd3e6a5c54438b8d0c61809f53ca36d1479768e0))

- Modules ([`7ea1dc1`](7ea1dc1c1d8d88319d76dbfc3cca5c23bd818a67))

- Modules ([`d842a0d`](d842a0dc4b67d54e749f2df176ed9732cb5b5b28))

- Modules ([`080d3f8`](080d3f830f5944f3e3ccdfb4f8cb46acc453c2e8))

- README.md ([`5236bc1`](5236bc13c8f78df79a647bccff5fe9d572e52983))

- Tests ([`c74dd03`](c74dd0318d60bb4653a98b9b4c8ca1f466aafc83))

- Tests ([`721c594`](721c594da3abdca12ac6783e9a9389ad69c6733b))

- Chat:behavior ([`a964511`](a964511a6c1805255894d041bc2cc69c85a03811))


### src/lib/sub_commands

- Re-add:Client ([`33efb2e`](33efb2e815770272af8a69f60d17f188ac9f7e67))

- Re-add:Connect ([`a1ce1e5`](a1ce1e5330036f6383f43c88f8c55c61fa40074b))


### src/main.rs

- Tests:error capture ([`3adb93a`](3adb93a3df1bf80e8aac7e2b80179e72d8a66d2a))


### v0.0.0

- Reminder ([`5e4569f`](5e4569fc1af02971a1d39c885895777748023fa4))


## [1911.921558.782146-gnostr/legit] - 2025-10-31


### .github/workflows/run-all-workflows.yml

- Act configs ([`64743be`](64743be818de08d55139c1302738d83bdc763da7))


### Build System

- Detect if RUSTC_WRAPPER is set ([`6476048`](647604811ea5d57fe34f2adba2a562da3d4228ac))


### Testing

- Passing ([`150ed06`](150ed06769cc7530a351708989f21a71af1be325))

- Nocapture logging ([`452827e`](452827e193b6e99bc2fda5e8de0cef9e7d21cd12))

- Legit sub_command test ([`8e95959`](8e9595955207862cab4fcb0f88221fb4819c2ba3))

- Legit sub_command test ([`ed4d4ca`](ed4d4ca4f73a933b56a0b282725fc2f163af3769))


### gnostr/legit

- Know working version ([`59ace1e`](59ace1e689ca44f7708a30d709c44756c8ab6145))


### legit/src/gitminer.rs

- Ensure_gnostr_dirs_exist ([`c7ad8df`](c7ad8df6a230b26ac9187da842258d46bc60084b))


### legit/src/worker.rs

- Nocapture logging ([`14958b5`](14958b53b708c99233b29969f1209e5c7af74fed))

- .gnostr/blobs/reflog mkdir fix ([`93e713c`](93e713cbc83bae3fc77e761268d44041ff5f0032))


### src/bin/generate-server-config.rs

- Crossplatform refactor:windows ([`318b5cb`](318b5cbe601eee9fd8869558e9899b358d789aef))


### src/bin/gnostr-git**

- Print tag_name ([`ccf7927`](ccf7927af23e053d06a3f621fb9d1eebc9ac77eb))


### src/bin/gnostr-git-checkout-*

- Print branch_name ([`6ccb3d9`](6ccb3d9c54761ebaffcc394a45fbed92f45fda3f))


### src/lib/p2p/chat/tests/p2p_chat_integration_tests.rs

- P2p tests ([`3fa824f`](3fa824f6a0bfa4e8aab2c4d06de9466289e17f25))


### src/lib/sub_commands/query.rs

- TODO ([`83c7b04`](83c7b04e776b729b326e4d77dd7a19227aabb3de))


### src/lib/ws.rs

- Tests ([`ba6a5fc`](ba6a5fc410f94b4dcd1c2169e723eb931bc82d28))

- Test:ignore ([`edb8df4`](edb8df439d21cd6779e17d075a5fd49132497046))


## [1912.921277.229598-gnostr/ssh] - 2025-10-29


### .github/workflows/get_time.yml

- Seconds ([`841d586`](841d5867010886a18893fbca98e56be424e0edb9))


### .github/workflows/gnostr-bot-macos/windows.yml

- Even/odd ([`7ab992d`](7ab992de6bb22fc5406c51e96c4d758969f3b9ac))


### .github/workflows/gnostr-bot-ubuntu.yml

- Even/odd ([`040a823`](040a82376c526e627ec2b4aa8f84c7391fb1b9b2))


### .github/workflows/gnostr-bot.yml

- Fetch tags false ([`fa676c5`](fa676c571bb65aa028b16c044d375e3082c9b542))

- Linuxbrew ([`4e5e354`](4e5e3547321f5ecb6e2dc87d3723716648e8cd80))

- Linuxbrew ([`158936e`](158936e9990b9cf4090d76c9e4991c4a49c0a615))

- EVENT announcment ([`1b9abfd`](1b9abfdf2e46a7c86d6892c0d5b4a2378c413109))


### Cargo.toml

- Use gnostr-grammar ([`db1ad89`](db1ad8948c891724da84c7aefaf63cb91a18d0bf))


### gnit

- Working ([`184c0fe`](184c0fe10bf8c09f162246863831594b05ace9e8))


### gnostr-gnit/grammar

- Initial config ([`660a65d`](660a65d3f165dc3744c9b772b32221edc3072cc1))


### gnostr-legit

- Refactor ([`d579df5`](d579df54af265a40385d01b775fa7a4b62567dff))


### legit

- Integration ([`be7d140`](be7d140e061b402faad7321acc6d4e882b9b78e4))


### src/bin/gnostr-git

- Ignore some tests ([`c0a7a47`](c0a7a47de23b6529285db84056d8ddd0a1584112))


### src/bin/gnostr-git-checkout-b.rs

- Tests:use args_vector ([`6386b38`](6386b38df37c3089cda37a23b53a5b464018a50c))


### src/bin/gnostr-git-checkout-pr.rs

- Tests:use args_vector ([`38068a6`](38068a678ad9c5a169aaeae6fb3ef61c9c23325a))


### src/lib/sub_commands/query.rs

- Pub ids:short:-i ([`1c3b8f6`](1c3b8f6809dc5c1a37a4b066f0c0c4f2befd7d89))

- Arg:relay:short:-r ([`389ad84`](389ad84979fab174b012b085e0699f589c269da9))


### templates/base.html

- Gnostr ([`adbba40`](adbba40e016daf0570cc83e0d285b074b3f6ac8e))


## [1912.921034.55846] - 2025-10-27


### Cargo.toml

- V1912.921034.55846 ([`2e6dba6`](2e6dba6dae33bb8bce9fce71af108ee117dec3e7))


## [1912.921034.558464] - 2025-10-27


### src/bin/gnostr-git-tag

- Tests ([`7bcd479`](7bcd479b4db5295e98b65f5c29954af6cc646633))


## [1912.921033.558806---nocapture] - 2025-10-27


### src/lib/sub_commands/custom_event.rs

- Verbose nip34 help ([`74dd9ad`](74dd9adb403ce68bf3e9bbcca5c846359fc2d3a8))


## [1912.920900.732280] - 2025-10-26


### .github/workflows/release.yml

- Revert ([`628665d`](628665de20fb80cdeb7a576832673b4be25376ab))


### plan-dist-manifest.json

- Update ([`c2fb64e`](c2fb64e76ee791c299aad6b9d98e382e3e476a05))


### src/bin/gnostr-git-*

- Tests:intermediate ([`fc3f411`](fc3f411f76c3045a50abb338a9a37dc8996583ac))


### src/bin/gnostr-git-checkout-b.rs

- Passing ([`8295846`](8295846844895733a819ae7d8f282981f0c2c0c9))


## [1912.920892.743278] - 2025-10-26


### Cargo.lock

- V1912.920888.748912 ([`3d09e0b`](3d09e0b730c9fd64bf5c37d3506cdd8352a49f3a))


### dist-workspace.toml

- 0.30.0 ([`b02fab8`](b02fab8ec393e5c6df761671c4bd882e24325c4c))


### src/bin/README.md

- Update ([`7f99cdb`](7f99cdb9eedbfeaf3643731e8088a6da533256d6))


### src/bin/gnostr-git-tag-version.rs

- Update tag version logic ([`c91eff7`](c91eff7320f594f167c99299a4b63f41b6c4bd00))


## [1912-920888.748698.test] - 2025-10-26


### src/bin/gnostr-git

- Commands ([`72a8bc6`](72a8bc68b9932d57e466847593ee6c81fb89a04d))


## [1912/920883/756048/2ed1a9d8da/1662985999-v0.0.126] - 2025-10-26


### .github/workflows/release.yml

- Handle gnostr tag format ([`1662985`](1662985999acff2e2fc8b0336f77e9cc50cf3aca))


## [0.0.126] - 2025-10-26


### .github/workflows/release.yml

- Debug ([`2ed1a9d`](2ed1a9d8daa40003b06e0d3f8aad8030cece9b9e))


### src/main.rs

- Revert ([`0cffc51`](0cffc51adfa1be8a2c8c1cf2018ecd8eb3c7dbbc))


## [0.0.125] - 2025-10-26


### .cargo/config.toml

- Target-dir = '.gnostr' ([`c645bce`](c645bce2506c19252680fd255d52d3d366107858))


### Cargo.toml

- Use gnostr-crawler = { version = 0.0.15, path = crawler } ([`d162836`](d162836778078905d0dacf8730556056331e674b))

- Use time@0.3 ([`283fcd4`](283fcd467ccf510597d7f348a1982790aab29853))

- Serde 1.0.203 ([`0e403b5`](0e403b535d8997fa477c7fe3bec59b6d537afa6f))

- V0.0.125 ([`3c0a372`](3c0a37296f15d36d974f656268ae0f23508e3736))


### Testing

- Ignore ([`2d85d7b`](2d85d7b97dcfa74164fb83c51d5f6e77a88ea7b9))

- Fixed ([`4c0a834`](4c0a83479ef5e01553a132103604fb2f2f7e39b2))

- Intermediate:passing ([`cdf0bdf`](cdf0bdfdeefc5cfad07d065ee507d6ec129d78cf))

- Add ([`8a77b64`](8a77b649690445787502c6b6ce64edc0a0f4d914))

- Refactor for "git-ssh" command ([`73d624b`](73d624b39df39225fbc2351b76b1394f4d36d426))

- Test_gitsh_command_error_output ([`d606263`](d6062634ec3bb669cf9d2c5f3b28c58e0f231c96))

- Ignore a few tests for now ([`7b5f4a2`](7b5f4a220ddafcb9b1343b38dee20491a48e2406))


### crawler/Cargo.toml

- V0.0.15 ([`42cd5af`](42cd5af96cc3c3159285a66e2aa083b5a4f724bb))


### examples

- Remove hashlist/haslist-padded.rs ([`c9e3ef2`](c9e3ef2cfb60636a19bce677c8f6b291a913bbea))


### make

- Gnostr-chat:use ./.gnostr/debug/gnostr chat ([`a735419`](a735419adf15ac59753dbde6025f7977e344e3c4))


### make_just.sh

- Use -j $(NPROC) ([`5ffaafb`](5ffaafbbf530e059d08a5de5a9e208b8319439be))

- Make cargo-build ([`97e2590`](97e25908b67f15fd048407309c0c7d6208398cc9))


### query/Cargo.toml

- V0.0.12 ([`ce5367b`](ce5367bb30bed64e987d0d98c85f8568d9beabc1))


### src/bin/README.md

- Add ([`001bd43`](001bd4309048d981d0ff3ca59792cdc5c67b37d0))

- Update ([`17cc6bf`](17cc6bf2103959c6ccc3b70f3b510bfae8235e86))

- Update ([`3aa738c`](3aa738c5ad4225fe89e29b654749ebe9fcefdc75))

- Update ([`e68064e`](e68064eb96e70f2fbe8affef521ca0981b355f38))


### src/bin/git_remote_nostr

- Update to nostr_0_34_1 ([`9ace744`](9ace74498c137a09a8defed043b3ab9b01afbee9))


### src/bin/gnostr-blockhash.rs

- Test stub ([`99824b4`](99824b43c26ab590ec93cfac2aa375a71da5bee7))


### src/bin/gnostr-blockheight.rs

- Test stub ([`3b2d9fc`](3b2d9fca858ac668fcc82e0ca9c387ddbd1c3755))


### src/bin/gnostr-cube.rs

- Add tests ([`f5cae1c`](f5cae1c1770e899054c449bc8e9878534aa3c78a))


### src/bin/gnostr-cube/query.rs

- Cleanup ([`5cd050c`](5cd050c8125d06da8b23e7064e4a3157c9f5d539))


### src/bin/gnostr-kvs.rs

- Generate_close_peer_id ([`9e10632`](9e1063277cfd9e98c0479f53a1a854fd752dfd9b))


### src/lib/cli.rs

- Fix import path for ArgMatches ([`80d3b9f`](80d3b9f4e57fa23ca53c5343eeabe488a4c98259))


### src/lib/core

- Add ([`73bffa2`](73bffa247239fdef64cee67c3891c14d0a31945b))


### src/lib/p2p/chat/mod.rs

- Remove libp2p::StreamProtocol ([`e5706e2`](e5706e2ae16e7363a79556e10563503d38210fd1))


### src/lib/p2p/chat/msg.rs

- Wrap_text:unused vars ([`0987ad7`](0987ad716dd641bc402d27302df19aa9a7519041))


### src/lib/p2p/mod.rs

- Fix some xor logic ([`39e7edc`](39e7edc825baeba9cb4446bf98e9c80166e3693d))

- Remove Swarm ([`b022a14`](b022a14f63466b257a20b2b384cb3b4478ae8b2e))


### src/lib/sub_commands/chat.rs

- Remove unused imports ([`367c394`](367c394bfdad674e58fcf48ad79b258c42d748d2))


### src/lib/sub_commands/custom_event.rs

- Add docs ([`3048b95`](3048b95ac68217650d1e996e53632b709215817f))

- --help verbose ([`acc16c6`](acc16c60040bab63dd838dc5853b17f4f8afa5fb))

- Verbose --help formatting ([`77d300b`](77d300b3bb5d5d1d2ec9028c7539b6974a7d08eb))


### src/lib/sub_commands/gitsh.rs

- Initial impl ([`50725c6`](50725c6ab996a091ebcfa7af99e6d41ed752c971))

- Mock_ssh ([`b98ab8f`](b98ab8f2ca1fb9de086a6fa0b36d50ba141fda76))


### src/lib/sub_commands/query.rs

- Remove unused imports ([`c937707`](c93770718b77a4a726bb03a64272ea39c91c015a))


### src/lib/sub_commands/sniper.rs

- Add ([`5065859`](5065859e546c7492b13527c5dd07ba27908c9abe))


### src/lib/sub_commands/tui.rs

- Remove unused imports ([`8bd2e8c`](8bd2e8cfdfa5ffa0276c173e87f76a1a115e5d3e))


### src/lib/utils/README.md

- Add ([`dd316ed`](dd316edd8493b630a6e7da31f0e37f7b6e8a3d54))

- Update ([`d000dde`](d000ddec8156506bbe820b8f50060a2e603eb7f4))


### src/lib/utils/RETRY.md

- Add ([`0d210f8`](0d210f8bbc3b027c3387298fe4f3c138110aa1bc))


### src/lib/utils/mod.rs

- Add find_available_port and async_find_available_port ([`94282d5`](94282d596774a1007715c42922188e3d8994d92f))

- Remove unused imports ([`1c0c182`](1c0c182b98b5ecd034fed42b30691a4c7cbc7578))


### src/main.rs

- Remove unused imports ([`b5ff800`](b5ff80061e51c32f7f46f6409287cd968008f279))


### ssh

- V0.0.1 ([`3790565`](37905650703577013a851e573413bc3f45991cd7))


## [1913/920176/797441/8a9888659d/86039d8ff4-cargo-toml-use-gnostr-async-v0.0.7] - 2025-10-22


### Cargo.toml

- Use gnostr-asyncgit = { path = asyncgit, version = 0.0.7, default-features = false } ([`86039d8`](86039d8ff44fe69ff83e51ca529bfe6b4d910aae))


## [1913/920176/797084/8720c4bf6c/8a9888659d-gnostr-asyncgit-v0.0.7] - 2025-10-22


### /src/main.rs

- Add gnostr relay sub_command ([`334ed33`](334ed33b958358a560a7f50e8eeefdc83e7de29f))


### Cargo.toml

- Nightly feature stub ([`e33d25c`](e33d25cd14e31c9475c27688725eced00349dd69))


### Testing

- Intermediate:should_panic ([`be2e909`](be2e9091f1e5ef6b3ef4138e99be34a78bbf1610))

- Intermediate ([`8710b05`](8710b05421472580d74fe65c308290bf399486f2))

- New stubs ([`8d092e9`](8d092e90fd91feb2b16328a4ca0e8f6e66c92baa))

- Intermediate ([`9d131d7`](9d131d79116cf5d092f3c663f41bb6315ed8ed96))

- Intermediate ([`c4d6c18`](c4d6c18e2063cc705dfba3f976e96e1a692f8d49))

- Temp disable ([`27bd952`](27bd952eddca467a7a0095469154058901471edd))

- More tests ([`4c04c55`](4c04c550faa3ad005fd132c85c34c2ae238ae161))

- More tests ([`be4ace9`](be4ace924c00f4f60c18458d1b0bdd1065ed5fea))

- More tests ([`2f9ab51`](2f9ab51b7309a7c56c6be9141760e5fd41d21bf3))

- Iniial impl ([`1ea4417`](1ea44170120c7da18ec163e615518686926980d7))


### app/target

- Remove ([`7fdbe14`](7fdbe145aa5bc70f7a5044571aad5bd791460692))


### asyncgit

- Some ratatui integration ([`2c01fde`](2c01fdee855f3a88120975978cc64c799d347652))

- V0.0.7 ([`8a98886`](8a9888659ddaed5dd5cccf7b2ef4c8af25343176))


### asyncgit/src

- Escape stash view ([`0aa7277`](0aa727796113a6041e53292a745e5f0f6c5c3105))


### asyncgit/src/gitui/state/mod.rs

- Screen ordering ([`c14870f`](c14870fa6b4bbee7f5bff194229912dc054d9955))


### asyncgit/src/sync/commits_info.rs

- Serialize/deserialize ([`58f2d62`](58f2d624a8955083138a246fb003c6c96e868e44))

- Serialize/deserialize ([`ea54c0d`](ea54c0da1d7858fe07e964b8eb8317b11179861d))


### crawler/src/processor.rs

- Add LOCALHOST_8080 ([`24d6b0d`](24d6b0dbe8cbb53bb00703366735ac41ac2c3eec))


### examples/ngit.rs

- Query ([`444f86c`](444f86c48bde0d07ff639408e992a97486c1a710))


### gnostr/core

- Checkpoint ([`8720c4b`](8720c4bf6cf1ad2c84d6534c7f94bd272183931d))


### gnostr_core

- Test refactor:intermediate ([`504a39f`](504a39fbb7478b9cae213fe46e0857c3d80f6482))

- Test refactor:intermediate ([`51b2449`](51b24491ed623b7f90ea19033604a6d0ef50affd))

- Test refactor:intermediate ([`4bee9e2`](4bee9e2a0c6bb5d5e695d5f1ebcd2741ef567493))


### handle_internal_event

- Intermediate ([`ac8a508`](ac8a5089669c701b24da3d3a545412e14565b7cd))


### make

- Gnostr-chat:use debug ([`5d70aa2`](5d70aa2c7fbfbac705fbfa20383403d8f0c84575))


### make_just.sh

- More commands ([`3790caf`](3790caff72bba35ffdc0575b682a97877bb5f7d8))


### qr/Cargo.toml

- Remove a workspace artifact ([`2289f75`](2289f753b91aec63680b7fb1298d5e32c62664d7))


### src

- Intermediate:cargo insta snapshots broken ([`1416d54`](1416d5465c9907716388a4f2da635e466870612b))

- Cargo insta:intermediate refactoring ([`219463a`](219463aa11f07a07ab03c4f124cb7fc4a451ee96))


### src/asyncjob/mod.rs

- Intermediate:fix cargo insta ([`63840e4`](63840e46b4960556421a042d22c3dc3c21dff18d))

- Ignore  test_overwrite: ([`909b11f`](909b11fdb6d154959538cb1a2ffbd45b66b29b59))


### src/bin/gnostr-legit.rs

- Fix convert_to_u32 ([`a3980ca`](a3980ca9e60b1f5af97b6c5a1ccb1ab2d66984ce))


### src/bin/gnostr-query.rs

- Debug logging ([`d60d97c`](d60d97c1b34c41d4d1697936010514e6ef4759c3))


### src/gitui/tests/snapshots

- Remove ([`565fc24`](565fc24dc12ca5eaafc2e974ea77e5bc42d3e07f))


### src/lib

- Refactor:intermediate ([`6bc3c1b`](6bc3c1b1a08711e5bc38f9cc8255bd09aea21f50))

- Refactor:intermediate ([`21ff799`](21ff799d840a838eb97e1d85420d89fc9a07768b))


### src/lib/chat/mod.rs

- Intermediate ([`9465200`](94652005fa9208cf108ef9cb985a273dcb3748e0))


### src/lib/chat/p2p.rs

- Message_id_fn ([`bdd68ba`](bdd68baab36c3632f7d48929b8efeb28f652ad7b))


### src/lib/chat/title.txt

- GNOSTR ([`25a8d0e`](25a8d0e40ed25e80d1b1d16b245889a7ecf61df1))


### src/lib/cli.rs

- Loggin conflicts_with:initial impl ([`342aca3`](342aca3cff1327c399a8f5f0493fedc760a04b87))

- Add Relay sub_command ([`e64eeb0`](e64eeb0d9d287e6a5e84cdd7875e4e53ccac3b1c))


### src/lib/components/topiclist.rs

- Intermediate:chat ([`aa13d5f`](aa13d5f5624559ef21da91998e3dc25161625b65))

- Add chat history ([`4201134`](420113496f6950916cb8d00339bdb1e5cd73019b))

- Chat histories ([`3608b07`](3608b07bec531124e3dd8646aeb17e0d0849ed84))


### src/lib/keys/key_list.rs

- H = help ([`03aa71a`](03aa71a0aaaf91e274510ed8c05e857fa79add43))


### src/lib/p2p

- More tests ([`553bf09`](553bf091f713e8b881abc38e092e9fc1663ae9bf))

- More tests ([`d6d9bd5`](d6d9bd5cd470dfbda6732f8522ced105c2457abd))


### src/lib/p2p/chat

- Migration:working ([`fa6502d`](fa6502d486326ea0f0753ab4788d422f69cdde61))


### src/lib/p2p/kvs.rs

- Refactor ([`cdcded2`](cdcded250ccd53af6195211673935488b12b37c2))


### src/lib/p2p/mod.rs

- Async_prompt ([`b80b781`](b80b781a6f7528c2470b03c15ce6c1647db534e0))

- Swarm.listen_on ([`7c10c5e`](7c10c5ef354a03cb90a6343f2ddc31399b69bc0d))


### src/lib/sub_commands

- Refactor:with tests:intermediate ([`c00f065`](c00f06504abd266f39000b6d81a1a4520089ddc7))

- Refactor:with tests:intermediate ([`32a6074`](32a607453a2f687fee1ea7c42e1b89640aa8229b))


### src/lib/sub_commands/mod.rs

- Add pub mod relay and query ([`1152cdb`](1152cdb66df16c962103971901a424c6ef0863f1))


### src/lib/sub_commands/ngit.rs

- Test refactor:intermediate ([`9f27e93`](9f27e9390638c8a15ed6487e89c1fb57853f69ec))

- Intermedite ([`32aba75`](32aba754540b4f202a1a710ca62f27c5eaa97111))

- Disable async test_ngit_push_command and test_ngit_pull_command ([`72098b3`](72098b3471d7f42dafa3e7229db0cd052b1bfade))


### src/lib/sub_commands/query.rs

- Intermediate test config ([`d5c12db`](d5c12db62e45e9920c875502724e6c9e672a3969))

- Nip34:kind query matrix:initial impl ([`2e6b7d0`](2e6b7d022df421450f222021af09d9f756e4bf2e))

- Add gnostr ngit query:sub_command ([`07624ae`](07624aeed063e2bda095a75efb28d97ee3b8c725))

- Add LOCALHOST_8080 ([`3d9e0d6`](3d9e0d661072c804691f8662a8a50363ca420ced))


### src/lib/sub_commands/relay.rs

- Empty ([`0e8a563`](0e8a563879df16baaab24932069ea9bcbd7ada8f))

- Gnostr relay sub_commands stub ([`75d0c1b`](75d0c1bd2c7302b043ee578301c6094ab05a95bc))


### sub_commands

- Check point ([`f0b9f25`](f0b9f256cb93fa9afa14a069ba18af6ffc14f811))


## [0.0.124] - 2025-09-15


### .github/workflows/gnostr-chat.yml

- Apply yamllint ([`c87a12a`](c87a12a9290d22a05168b9f8328fe79753f10dbb))


### .github/workflows/release.yml

- Fix aarch64-linux-unknonw-gnu: build ([`49b50fc`](49b50fc5475279e99ff54d9d1990fe84e4941846))

- Gnostr release event:inital impl ([`1d5ee58`](1d5ee582e92f7b2178ab37f45c3f362029f4d3e4))

- Apply yamllint ([`a195b87`](a195b874e25fffeb885c4fa131f88d4fb7e11621))


### Dockerfile

- Gnostr repo ([`434706f`](434706ff62c982ef7c8ec995157843252fb90cad))


### dist-workspace.toml

- Add aarch64-unknown-linux-gnu ([`84e9d00`](84e9d008ae8dfe2e4a0a6a99551eec86616fe0d5))


### just

- Docker-shared ([`4493483`](4493483e27f7a4b7e82221f53f10bbe0da738458))


### make_just

- With Dockerfile config ([`dd15517`](dd155170e24cee1ff214cad9a2ac8b2525cf1cb1))


## [0.0.123] - 2025-09-14


### gnostr

- V0.0.123 ([`483cdc4`](483cdc4b4dac9c5db03aaa52067aa4b12be12f4d))


## [gnostr-query@v0.0.11] - 2025-09-14


### gnostr-query

- V0.0.11 ([`35c2b6e`](35c2b6e28eac8445c5655974115c9c9652535578))


## [gnostr-crawler@v0.0.14] - 2025-09-14


### Testing

- Add tests ([`d37d683`](d37d683a1e517d78e1a3a6f92fe922178f2981d3))


### app

- Use gnostr-extensions ([`98a4f1d`](98a4f1dcc496c9d7a7d335fb9cdb450706a94653))


### app/relay

- Remove ([`953aa7c`](953aa7c2cb4e0ef7738b20ff679f0ca4e0f59a99))


### app/src/main.rs

- Apply cargo fmt ([`8732b5f`](8732b5fcf175b72e016b4a6bca87e86c4e1e22d1))


### config/gnostr.toml

- Enable search ([`73b9b3b`](73b9b3ba1e91686e670695c6ecb6fc95f13d7427))


### crawler/src/processor.rs

- BOOTSTRAP_RELAY0 = ws://127.0.0.1:8080 ([`eec8c64`](eec8c643ed3a4f93e2afe8353187cc4642a2b991))

- Wss://gitnostr.com ([`71d258a`](71d258a946c862c678bfef3093b6cacc384ccf88))


### gnostr-query

- Gnostr-relay:compatibility ([`e8f3865`](e8f386568e8fe16fddb4e01ecd7f0e0634eb20e5))


### gnostr_crawler

- V0.0.14 ([`f757a5d`](f757a5d2f470d174d61d09defbbeab1d1a7d2d20))


### query/src/cli.rs

- Default nip-34 kinds ([`407f5a8`](407f5a833682b9ce55ba21d82df03b0fc8b30076))


### query/src/lib.rs

- Quiet ([`97293c6`](97293c6771546c705bfccabb824884a4e62b34c1))


### query/src/main.rs

- Quiet ([`c80058b`](c80058b30c345005a7be5de876fec0c410db5410))


### relay/src/app.rs

- Test_main_app_creation ([`52dcc46`](52dcc46472fc30c6e168afcbafd9de9d3efb496c))


### relay/src/lib.rs

- Add tests ([`6d40a07`](6d40a072a23103f20a7e1ce1e28a83dc5c10fe59))


### relay/src/list.rs

- Add tests ([`f2dfeea`](f2dfeea437c059ea1268f516af3b3741808999be))


### relay/src/session.rs

- Logging config ([`2991bd5`](2991bd57f9c87381de150a55d76012ef729c0080))


### relay/src/setting.rs

- Apply cargo fmt ([`0b75c31`](0b75c31451e1e0fac7208398f3465132602b939c))

- Add tests ([`6777020`](6777020f5355f98e4b2a573359ce9ce613f8de3c))

- Add tests: use CONFIG ([`7a2aac4`](7a2aac4620e576f78912cfb03cdc1626d796d4a9))


### src/bin/gnostr-blockheight.rs

- Get_blockheight_sync() ([`692b53d`](692b53d2da9d945dcbfc066b5b5f0d24736c1bf2))


### src/bin/gnostr-cube.rs

- Apply cargo fmt ([`b558f18`](b558f18497fbb2a3ced19a97bdcb31e9628287cf))


### src/bin/gnostr-query.rs

- Output valid json ([`87d16d1`](87d16d1c6938b391844be17511e5369b58f50a39))

- Json output ([`f7ca80f`](f7ca80f1a3af377af98d7cf8dbc18cf261db02af))

- Apply cargo fmt ([`3422d7a`](3422d7ae78e8a43750e41e7843ec110540f8ce98))

- Quiet ([`d574928`](d574928818253e838e8b276bc7832c340353ea9d))


### src/lib.rs

- Apply cargo fmt ([`a756ee2`](a756ee2891da0c4f11ca338e4a202f6f61008d06))


### src/lib/chat/mod.rs

- Apply cargo fmt ([`91ae7d0`](91ae7d0438d04a93b2fa42b53940409b91edd7a6))

- Additional raw message value ([`2899d4d`](2899d4ddf7a206eee7c12466385da300c9d7ccca))

- Intermediate ([`144ee26`](144ee26f186a198f3b33041da570e589b4c4d2d5))

- Add pub mod tests ([`bb68b91`](bb68b91ed1215ab5d3d6bb3619ee9f55f63560ec))


### src/lib/chat/msg.rs

- Color::Red ([`e0ef2b9`](e0ef2b98cdc3b21061713995f634fc0433fc9521))

- Intermediate ([`e51c7ef`](e51c7ef46b22862dcdac816da78ea6909086088a))


### src/lib/chat/tests/mod.rs

- Add tests ([`a165935`](a1659357fda768aad793dffa2ab209689dbbf665))

- Add tests ([`a260c0f`](a260c0f663521e8ff6a50e5e132cbffd566a574b))


### src/lib/cube/app.rs

- Apply cargo fmt ([`d90bb44`](d90bb447d13a9f2c3ac009dbe450c25d0bf43d52))


### src/lib/cube/handlers/event.rs

- Apply cargo fmt ([`3a0c9c6`](3a0c9c6e39caef349b816496331bb12176aede61))


### src/lib/cube/local_git/mod.rs

- Apply cargo fmt ([`bf26b54`](bf26b54b06c6d25c6c1602ee27345ebf300d851e))


### src/lib/cube/system_command.rs

- Apply cargo fmt ([`b43f9f1`](b43f9f12fa46d21f8d568ba887bfd22b4a5b86e4))


### src/lib/cube/ui/mod.rs

- Apply cargo fmt ([`5e31c06`](5e31c0657e999dc1a484b7d0c23a6be36fdc0478))


### src/lib/mod.rs

- Get_blockheight_async ([`35ba025`](35ba025d55a16143117ce0a1791796ae82442920))

- Get_blockheight_sync() ([`8d98728`](8d987286c321ec428a8bfdcab62c5a07d9f58b00))


### src/lib/sub_commands/chat.rs

- Migrate crate::chat::p2p ([`4d3447d`](4d3447d3d8f3c1e3cff288c50986c945d9a684c1))


### src/lib/utils/mod.rs

- Add tests ([`d9e857f`](d9e857f676f3fa7a87fd5f9fd9ef3f1531b3aafe))


## [gnostr-crawler@v0.0.13] - 2025-09-11


### Cargo.toml

- V0.0.13 ([`afcfd20`](afcfd20390f182e9f86a147368ca9c9d51c27205))


### crawler/Cargo.toml

- V0.0.12 ([`10dbce9`](10dbce9804534faf1dddb4ad895c287feaf078be))


### crawler/src/bin/gnostr-sniper.rs

- Load_file(relays.yaml) ([`d1f5ce8`](d1f5ce8e29c82619b73d3485c6278c4313506fbd))


### src/bin/gnostr-unified.rs

- Apply cargo fmt ([`1f47115`](1f471155e80cb333f3dca3a7a1eb76513863c693))

- App nip34 filter ([`2dba300`](2dba3006868600287da9df0d729c9bd1a5f10133))


### src/lib.rs

- Apply cargo fmt ([`8328a8d`](8328a8d6bdd9c49b687318afd4c77e20ea1d8c44))

- Refactor ([`1a9c8b1`](1a9c8b15916cdd64ed1a0eea7ade7370c5ede4a4))


### src/main.rs

- Intermediate ([`466917c`](466917cd6f3690612b7e0403145ebda74b18b52b))


## [0.0.121] - 2025-09-10


### .github/workflows/gnostr-bot.yml

- Markdown report ([`b177369`](b17736960d21d7677ed46a6ce768033fcc49172a))

- Markdown report ([`daf1a27`](daf1a2741d6c0516a9468d54594f4b22164b59aa))

- Markdown report ([`9987412`](9987412437ddef28c6a7e73d1ef23c79d16c441c))

- Markdown report ([`fb32e2f`](fb32e2f82c1266f7f0ff96fd88cf8396054f235c))

- Nip-34 config ([`cf35c05`](cf35c057d972c586ab86798ce92e1cd903572285))

- Nip-34 config ([`4ed7777`](4ed7777d142381f383fedfc6f664276a90c2f54c))

- Nip-34 config ([`8d22587`](8d225876a7d11e8beea93729c3e7109e938dcf2e))

- Nip-34 config ([`52d57ec`](52d57ec776cff90067d25f047df0f0dd3951ad63))

- Nip-34 config ([`88c95e2`](88c95e2ba7973b8330c628e60e01b92959b32653))

- Nip-34 config ([`f21d927`](f21d92780117cde834ac648048eb2da28e9321b5))

- Nip-34 config ([`f58f081`](f58f0810c87fe5de6ff8420581cf037588b6c6fb))

- Nip-34 config ([`e5dd5d7`](e5dd5d7047bfe88349c8f63e1cb5536e5faf5b05))

- Nip-34 config ([`1452fc3`](1452fc3f0bb0d93276d2b37a7c774c126a579ebf))


### .github/workflows/gnostr-relay.yml

- Initial impl ([`dd500d4`](dd500d4cb332768003bbffae04ca7bdb8eb6d699))

- Markdown report ([`fda883f`](fda883fd29c99a457d370d273c95d9c5e125a2b1))

- Markdown report ([`d2c1c65`](d2c1c65535afe8e70d50bb16fd3675956e397a99))

- Markdown report ([`4856a15`](4856a1560e651c39bba4792167e49740b0e75467))

- Markdown report ([`72e5866`](72e5866be0c108cc4f6341005392dd700a243e49))

- Markdown report ([`42cc7ff`](42cc7fff3b8d0ac523a6b3bb0b6c0c70afe75fac))


### Cargo.toml

- V0.0.121 ([`4c66be7`](4c66be71362361b8b51a57dce99d29199a4586d3))


### README.md

- Update docs ([`cd7d432`](cd7d432fce08e369edd5808076ea959e6cc0eb55))


### gnostr-cube

- Initial impl ([`b26621e`](b26621e020ab19264c072c6efd344d67106c0d14))


### relay/extensions/examples/demo.rs

- Demo.toml ([`b29755a`](b29755ad60e4ee665845d7b99d9cd0840e1404ac))


### relay/extentions/src/auth.rs

- Apply cargo fmt ([`3734cb0`](3734cb0450df4932c4fe76c403c7f88c84a4f4c8))


### relay/extentions/src/count.rs

- Apply cargo fmt ([`75bd435`](75bd435414137b0f5feccf76163059969fd6d1f3))


### relay/extentions/src/metrics.rs

- Apply cargo fmt ([`19e0411`](19e04116671472ade4d1752f6d917719c3908e03))


### relay/extentions/src/rate_limiter

- Apply cargo fmt ([`5e6c896`](5e6c8967a46bc72ea4cd351a018d54f67b7b4ab8))


### relay/extentions/src/search.rs

- Apply cargo fmt ([`bc71dc2`](bc71dc2a2c8c49feb9fd4d8b04893c986f32a80a))


### src/lib/cube

- Initial impl ([`fc26f4d`](fc26f4d1f13ae277b0ab1e69b33f894d31f42d1c))


## [gnostr-relay-extensions-v0.0.1] - 2025-09-04


### gnostr-relay

- Extensions:v0.0.1 ([`33a8daa`](33a8daa3051872245528f1e6ac82190552b5f2b4))


### relay/extensions

- Gnostr-relay-extentions:initial impl ([`46c6719`](46c67195267246416ae07aff9a947d8869fa8f4a))


## [0.0.119] - 2025-09-03


### .github/workflows/matrix.yml

- Cargo install -vv --path . --force ([`9ea6398`](9ea6398df97e6d2f7d6a989bc248fc0fb45f81f3))


### make_just

- Update ([`5257b3a`](5257b3aeec469d136ee83bcd14ac63859ad81f5e))


## [0.0.118] - 2025-09-03


### Cargo.toml

- V0.0.117 ([`aaf4eb2`](aaf4eb2bdc416e7156dcb072741a919978131724))


### src/lib/gnostr/msg.rs

- Solarized_dark/light.rs ([`48290b5`](48290b5e8be3c1a902678c6886037072c6c99e23))


## [0.0.115] - 2025-08-26


### .github/workflows/gnostr-bot.yml

- Export NOTE=note1qqqwldjg7gsg5nxdhwqpn94zzdm8yh7a4ndayakvpqy50emyhv3quazgcu ([`69b1e83`](69b1e8322ec5a1c1d0018025624fe2bf30273a5a))


### Build System

- Fixup os warnings ([`d4e4010`](d4e40105cb7ab21284577e18e6e22442ccd51989))


### Cargo.toml

- Use gnostr-crawler:v0.0.10 ([`bc6a7a0`](bc6a7a06d8691e1f3d3b686e58085752f5d7f338))


### asyncgit/src/gitui/cli/mod.rs

- Docs ([`5531a57`](5531a571caf52cf9ef4f335142cc05a811a3d82b))


### asyncgit/src/gitui/git/remote.rs

- Fix lifetime ([`6c95724`](6c957245470a699fc14b804013bf802e3ffda560))


### asyncgit/src/gitui/gitui_error/mod.rs

- Fix docs ([`6b070a8`](6b070a8bf87b1363fffcf8cb5a85b227fe4f7660))


### asyncgit/src/gitui/screen/mod.rs

- Fix lifetime ([`5ad3155`](5ad315530980c2a6f96ec2fb2d24d3aba2bc8edf))


### asyncgit/src/sync/commit_files.rs

- Cleanup comments ([`b64bd5b`](b64bd5bb413e22e359823ec4b36ed5cfdc443566))


### asyncgit/src/sync/remotes/mod.rs

- Fix lifetime ([`aead196`](aead19600cb7eca95fcab4a6143b9e98441021b1))

- Fix docs ([`5d719b1`](5d719b1f239a99a14fcabad9ea84a229fa1ff1cf))


### asyncgit/src/sync/reword.rs

- Fix lifetime ([`6450340`](6450340583d2201be3c30263622ee0cc2f94d944))


### asyncgit/src/sync/sign.rs

- Cleanup docs ([`d8717ca`](d8717cab786ebfd007682f473ccdedcad94f19a0))


### crawler/Cargo.toml

- V0.0.10 ([`120dff3`](120dff3eceeb799d0c43fc9fc821cb6651187a83))


### crawler/src/bin/gnostr-watch.rs

- Formatting ([`81f9067`](81f90672a9512740a1e407c6b0aca62544563ca1))


### crawler/src/lib.rs

- Let _revwalk ([`61b177f`](61b177fa2f5659f366093f9bf4ffa78bbb68ffe1))


### crawler/src/relays.rs

- De_dup ([`203c691`](203c6917a4f9ceb33125f6a53331a05ddc57e9c1))


### relays.yaml

- Update ([`2ad824f`](2ad824f53b98285f2b30ec1baab776e4325b99c4))


### src/bin/gnostr-sniper.rs

- Clean up ([`e893466`](e89346615ac9dbc167d6161bd787fd6374730a4b))


### src/bin/gnostr-watch.rs

- Clean up ([`927bc0a`](927bc0ad64e818e9a35fc8cead1c9ccd45dc4b60))


### src/lib/chat/mod.rs

- Cleanup comments ([`9d6441d`](9d6441dc2d33b667e3ff31f59672c81e830fd208))


### src/relays.rs

- Get_all ([`b547f2a`](b547f2a307114924c0f6620cc07b23646536c611))


## [1926/911337/726422/6b8f7ceccd/cf1a757ff4-v0.0.115] - 2025-08-23


### .github/workflows/matrix.yml

- Gnostr-bot.yml ([`1624ea1`](1624ea1e09d0623d3d519a24da6815d2e56a12a9))


### .gitignore

- Server.toml ([`87dcdcf`](87dcdcf3a4f9a838cd288f40bfac09bc06880d7f))


### 1929/909608/309784

- Gnostr ([`23f9f97`](23f9f97cdfd472f48f7e0a903e4152cdbd39b85e))


### Cargo.toml

- Use gnostr-asyncgit:v0.0.5 ([`e41362c`](e41362cf392856691e4bbd3454fa7109be24a428))


### asyncgit/src/sync/repository.rs

- As_path() impl ([`899c8d1`](899c8d11ae16c82009cb3115658a0f658b18e2d9))


### crawler/src/bin/gnostr-loris.rs

- Reset ([`2807dc9`](2807dc9e57f512e8bbf4ac3833949281fdfdff84))


### examples

- Remove gnostr-weeble/wobble.rs ([`a2f5b40`](a2f5b40307f99311b6e6b0963c243db1522e5970))


### examples/user-project-directories.rs

- ProjectDirs::from org gnostr gnostr ([`c6984a6`](c6984a61fbb9cba037628c6566a19f2264db5a70))

- Apply cargo fmt ([`8691735`](8691735243b4f9915fa0387206358224959e765b))


### genkeys.sh

- Initial impl ([`33faf1b`](33faf1bf84acca310e04e0dce993b8e6a98a3b43))


### gnostr-asyncgit

- V0.0.5 ([`b2f71f7`](b2f71f75c71073a46727af71ba472b87439addd5))


### gnostr-legit

- Intermediate:initial impl ([`f93f532`](f93f532838a819b40f76bc9f9765a003049d42eb))

- Intermediate:more impl ([`2339291`](23392916f2c73342d2bd8daf3bd0be28227868b8))


### maintainers.yaml

- Add wss://nos.lol ([`6b8f7ce`](6b8f7ceccdab56de9e66e1292f176850bfa87a99))


### server.toml

- Basic ([`de404dc`](de404dc07233ccfe77ed01933eb397b284df167d))

- Remove ([`6eb9116`](6eb9116624ec9c7cd93010dbc310f90574889253))

- Re-add ([`b7831dc`](b7831dcdb03ee0bc3aea4de2eba44f5c30eddd0d))


### src

- Apply cargo clippy fix ([`efc0999`](efc099972d421ed3e4b8b715e8a61d922b194359))


### src/bin

- Intermediate ([`6511f77`](6511f7706a690f574e32124463f2856b7977a807))


### src/bin/generate-server-config.rs

- Initial impl ([`b72c280`](b72c28094312ca9a806fdcd55ca131899ba8895a))

- Add fn move_gnostr_gnit_key() ([`51dd13d`](51dd13df4cf15ae2fd94affaad3f3625d4f7524d))

- Windows cfg:intermediate ([`af9d51f`](af9d51f5723cb6a2d84e289dcc22cc12eb0e1847))

- Return Ok(()) ([`e7b75ea`](e7b75eaa09b118d1d0f0e914302d44493952a7d9))


### src/bin/gnostr-genssh.rs

- Initial impl ([`957431d`](957431d0eea050cc73ca97a10aec57f0e445c266))


### src/bin/gnostr-weeble.rs

- Tests use gnostr::global_rt::global_rt; ([`694627f`](694627f68891dab14eabcef05ba3880516b01b46))


### src/bin/gnostr-wobble.rs

- Refactor:add sync/async and tests ([`44578a8`](44578a8277d1380e16834602f17216e840e0d8ec))


### src/lib/blockhash.rs

- Add blockhash_sync:blockhash_async ([`f2ec8b3`](f2ec8b34c96639301fb167e57c2b3cea0c7dc1fb))

- Remove deps ([`e3811c7`](e3811c7cb9ef3f74ece57708f44e46e736600ad4))


### src/lib/chat

- P2p:intermediate swarm config ([`d0c88c5`](d0c88c5e9baaf66d1960ca074cc2ef44e4f33ca2))

- P2p:swarm config with args ([`7153c37`](7153c37236f8afb0f1607f5c1ce27d079ce00b7f))


### src/lib/chat/mod.rs

- Listevents_subcommand ([`92f5a31`](92f5a31fe3953dcee5c9d956dfb6619b59814b00))


### src/lib/chat/ui.rs

- Intermediate ([`4cb4477`](4cb4477f8d41609d77bb3f16a70304ec0a501296))

- Intermediate ([`4547fb7`](4547fb761eb4be2a49de4a528a45df743ade7e89))

- Remove deps ([`427c284`](427c284600c944c7f715ac99819a97b5a9954c86))

- Add blockheight_sync to test messages ([`ecb4726`](ecb4726ee56beb7ae38bf878a0de3b4422d67c0a))


### src/lib/mod.rs

- Use refactored src/lib/weeble.rs and wobble.rs ([`c0d46ad`](c0d46ad0ee3ce7def5d7aad328898ed0498b2c0b))

- Use crate::utils::pwd::pwd ([`3739758`](373975886603136a9142e518767ad669a2495319))

- ProjectDirs::from("org", "gnostr", "gnostr") ([`19d5831`](19d58316d67c149cefb59f1ff5eef160fe12d93f))


### src/lib/p2p.rs

- Remove deps ([`9e6214a`](9e6214ab4bcda60080c11d2e89e8de12823a34b7))


### src/lib/p2p/mod.rs

- Add deps ([`8b2dc4d`](8b2dc4d2199792245f0908634635a5d4bb9e48d7))

- Add Network ([`5f90e40`](5f90e4013243cf69e1218ec3e2a473361fa1c23d))


### src/lib/sub_commands/convert_key.rs

- Apply cargo fmt ([`96ffd13`](96ffd13b5032fe14e536387827da70434ff0238a))


### src/lib/sub_commands/legit.rs

- Add --repo --pow options ([`26cac3c`](26cac3ced3aff554c1b5fbc15c029e3b762e1d1e))


### src/lib/sub_commands/list_events.rs

- --kinds defaults --output .git/<output>.json ([`1e885a2`](1e885a2392ba2aae72188d91a1491d53641028b6))


### src/lib/tui.rs

- Remove ([`da465c5`](da465c5cc45aaadc154c4d2f1c67964c4a42f6dd))

- Remove ([`448e8b5`](448e8b554883f24dac2ef181b8fb64b680ee4e52))


### src/lib/weeble.rs

- Add weeble_sync/async ([`17777a1`](17777a1199b641e2ff328094291a7c2d6888062f))

- Refactor and add tests ([`91f302b`](91f302b6581473102331575e77c0e2445cc16574))


### src/lib/wobble.rs

- Refactor:add wobble_sync/async and tests ([`00989a1`](00989a1815657246f4993d9195ed9b1f3d5e4eb2))


### src/main.rs

- Let gnostr_subcommands:not mut ([`84d4a3d`](84d4a3dcf7cf28d63b18148e39522830a428251c))

- Logging ([`074e464`](074e464214092151c4049bdaaca352ddf2d0cecd))

- Intermediate ([`1e732fb`](1e732fb4f55d29145f33e2ab9d26d308b3512c71))

- Debug logging ([`151556b`](151556bf7eeb99bebe97e3d05fe3c29779a8ed2c))

- Debug logging ([`d87cf37`](d87cf37c25c80d4e93467de7da6d8d8fd0752c8b))


### ssh_key_permissions.sh

- Initial impl ([`ea27f71`](ea27f715c765e8b8aed89f13103125c1c0172941))


### tui.rs

- NOT NOT valid case ([`2f148f5`](2f148f53f570652ff5c4fde466440b24f57ebec0))


## [0.0.113] - 2025-08-11


### gnostr-kvs

- Initial impl ([`2fee663`](2fee6638248809e9b6df190e556be37873287644))


## [0.0.112] - 2025-08-11


### Cargo.toml

- Use gnostr-crawler:v0.0.9 ([`af5d522`](af5d522792c3ff1d6428745821a95dedd3ff2099))


## [gnostr-crawler=v0.0.9] - 2025-08-11


### .github/workflows/matrix.yml

- Use rustc 1.89 ([`7aea47a`](7aea47afc8d0029e022ecc6b9138fe7a64caa19b))


### crawler/src/bin/gnostr-loris.rs

- Reset ([`1258782`](1258782bae1ea7919e4db63d6020ee6e6a3f3b9c))

- Reset ([`c020355`](c020355aaa0403444f400c715a8a243ac1ffca79))


### gnostr-crawler

- V0.0.9 ([`7160c92`](7160c9249c25690c84cd118cf5db64464aded086))


### src/bin/gnostr-loris.rs

- Remove ([`e7bb1e8`](e7bb1e81742dab7afa161a6c450a7a9f6d7fe1c6))


### src/bin/gnostr-sniper.rs

- Apply cargo clippy --fix ([`59f11a6`](59f11a6162f2f4dea8c7f19f3ac0f8e6ff858d78))


### src/bin/gnostr-watch.rs

- Apply cargo clippy --fix ([`3263daf`](3263daf586b0a20a6932e90f6d253a4452b59480))


### src/lib/p2p/mod.rs

- Add distribvibutedbuted-key-share dependencies ([`c072b85`](c072b85179e240df9a2d5be8c76f8c60062730e2))


### src/processor.rs

- Apply cargo fmt ([`32cfa92`](32cfa9251d4f49800f8ed5430ed2a2131b346cf2))


### src/relays.rs

- Apply cargo clippy --fix ([`34f4f67`](34f4f670d1126b0737c8a75f7c3e06151f7392df))


## [0.0.110] - 2025-07-25


### Cargo.toml

- Add dep gnostr-query ([`961f431`](961f43152b6248b9f5b68b9244fce3c768cb450a))


### app/Cargo.toml

- Use gnostr version "*" ([`b98aecf`](b98aecf59cb741265a8899e14a7a5aa3c63b9d13))


### gnostr-query

- Add ([`c592ffb`](c592ffb557f3e9987f7e6c995a1a9ce575ae4c2d))


## [0.0.109] - 2025-07-25


### .github/workflows/matrix.yml

- Remove run: gnostr-fetch-by-id ([`20adc5f`](20adc5f3a98781b2d6ae6f9f7953bfd8eedd1ce7))


### Cargo.toml

- Apply cargo sort ([`bce20f3`](bce20f3d78b866646f3b058458554fe1334e5e80))

- Add dev dep gnostr-query ([`97fafff`](97fafffe3c55da05d23284c2deed5ca646f4e752))

- Gnostr-query = * ([`8524209`](852420985fa472e7016cb5b81369c6e67ea24f6f))


### crawler/src/processor.rs

- Add/remove some BOOTSTRAP_RELAYS ([`203cb66`](203cb66aa903d22dd44810831418771b0562ee79))


### examples/nostr-sqlite.rs

- Apply cargo fmt ([`fdfe71e`](fdfe71e6b914a634380a4244bca3d61a546e27af))


### src/bin

- Move some to examples ([`a8ecd75`](a8ecd750af45fc64204b9dd6f950285f431207c3))


### src/bin/git_remote_nostr/push.rs

- Apply cargo fmt ([`308a747`](308a7479383bbdebaf01b390ae0edb7ab4b7c3d7))


### src/lib

- Apply cargo fmt ([`b6db416`](b6db4168a798067b10a1da882a51f404ef8e2f30))


### src/lib/args.rs

- Remove ([`e71d141`](e71d1413d7fb17ba1fefcc5f95d0c49385dd5a30))


### src/lib/chat/mod.rs

- Use crate::p2p ([`a7f5636`](a7f5636248597068422547af9bc76807ed32d5e4))


### src/lib/chat/p2p.rs

- Remove ([`0562aa1`](0562aa1f0b0db968db4b23dab8175161c22d96b5))


### src/lib/client.rs

- Use gnostr_crawler-BOOTSTRAP_RELAYS ([`a5bf279`](a5bf27990c84bf4bcdb4d5fe588f3a7ed89bc1c7))


### src/lib/mod.rs

- Code cleanup ([`9e248fa`](9e248fa9635156cd740af659fc22bde505674f8a))


### src/lib/p2p.rs

- Move ([`fe227e9`](fe227e9cdcfd386953d93cc81e86b613650190b7))


## [0.0.108] - 2025-07-22


### .github/workflows/matrix.yml

- Cargo t -vv -- --nocapture ([`afd6f6a`](afd6f6accbbcc4aa88c387b57de280c4012192db))

- Cargo t --no-fail-fast -vv -- --nocapture ([`b3780c6`](b3780c64dce3ecf44962e8ad144ee4b0e08d1fde))

- Cargo test if rustup stable ([`b2b4139`](b2b4139fb2fa3f437bdcaf7ef0f296f8a08cc9e5))


### examples/bitcoin_hashes.rs

- Example ([`06fc2d3`](06fc2d3ae6b55ba88ece351253e442b8b0505e18))


### make

- Fetch-by-id ([`c00e626`](c00e62640f348f382e62a28ffae001b73e0bf89d))


### src/bin/gnostr-sha256.rs

- Use gnostr::utils ([`c1ca52a`](c1ca52a10f3a53d2dfeb26b34bd5ed95033d76b8))


### src/lib/client.rs

- Remove:sendit.nosflare.com from relays ([`5d1e1f9`](5d1e1f9aab69414b9ca05de82feb7729316b0237))


### src/lib/ssh/ssh/commands.rs

- Knob.repo_note ([`3161be4`](3161be44371a0ae69b63996b3c1503646e2d94ba))


### src/lib/ui/style.rs

- Selected_tab:Color::Gray ([`5ba0f7c`](5ba0f7cd58cdf0ddb8788e45dc571ceb465e84c3))


### src/lib/utils.rs

- Cfg(test) ([`1526053`](1526053b907a48b5835fc3d10e82d89b1f38451e))

- Add some pub functions ([`fa4ff4d`](fa4ff4d726561649682c360ee519d00e1a0b293d))


### src/main.rs

- Apply cargo fmt ([`d85a059`](d85a0598abea2ef0915a78c9f2f1c4cb4f27c5e3))


## [0.0.104] - 2025-07-21


### Cargo.lock

- V0.0.101 ([`9c0af6b`](9c0af6b7bb1a81ed47a0ecc285b25498e7ee0e58))


### Cargo.toml

- .gitignore:.gnostr/.git ([`fff7971`](fff797149c03f495680ae1f1b8d3ad98f6a06045))

- Add gnostr-cat:gnostr-xq:dev deps ([`684d2ae`](684d2ae4ebaf55f2ae57416cd39f8f10de5a2d37))

- Gnostr-asyncgit:v0.0.4 ([`61861f3`](61861f3e2f8fd5c5c6c0d6d14cdd5ea127e5e7a8))

- Add relay deps ([`bf517bf`](bf517bfbdd00533d00652db825c02854cf982d23))


### asyncgit/src/gitui/cli/mod.rs

- Apply cargo fmt ([`c5532cf`](c5532cf8e89010c1ed7d6a0d14f31223e00eeaf7))


### asyncgit/src/gitui/gitui_error/mod.rs

- Apply cargo fmt ([`53738b8`](53738b8ac3c689dd8db0eba967c5cb430d8d62db))


### asyncgit/src/sync/commit.rs

- Pub struct SerializableCommit ([`6f59c6d`](6f59c6d81ed2cd076931bb5698b7500e068fa64e))

- Pub fn padded_commit_id ([`2baed3f`](2baed3fd73201b529a91171cc2eb4e652f61156a))


### chat

- Intermediate:handle nsec or hash from 'gnostr --nsec/hash <string>' ([`649ae52`](649ae52560dc782a20ce91b8fe394557238bc952))


### examples/gnostr-chat.rs

- Remove ([`7dab894`](7dab8946e7ebb82c9e1d197573d1eb5b88308f4e))


### examples/tui_input.rs

- Initial immpl ([`0947fdb`](0947fdb6e1ed0593b480f154875aa79ee18936b5))


### examples/ureq_example.rs

- Initial impl ([`73dc8ff`](73dc8ffce6020a23358e87767da3cbcf0f221313))


### gnostr-asyncgit

- V0.0.3 ([`219a77c`](219a77c0b409c413949efcbda130de21ac74eaaf))

- V0.0.4 ([`bfd955a`](bfd955a44936b26520f816de12defc338367a8d3))


### make

- Gnostr-chat:remove examples/gnostr-chat.rs ([`a086e56`](a086e56d020af56d27628d50136312a26b59768d))

- Broadcast_event_list:nip_thirty_four_requests ([`891a99a`](891a99a611c956add489bc711e6f411dfb28875e))


### repo.toml

- Members = ["gnostr"] ([`aac148c`](aac148c67896ece2b92931ab079afa52b521e7c3))


### server.toml

- Welcome_message:extra:toml format ([`d828668`](d828668e879a066554e0cdf1e4a03d6d4a70f19b))

- Welcome_message:extra:toml format ([`46494f9`](46494f9c9e1452be291371c96575c10995d41fd9))

- Welcome_message:extra:toml format ([`cfda739`](cfda739cc2639dcce996826c855a97d3994af0e5))

- Welcome_message:extra:toml format ([`eb28bb3`](eb28bb352c290f0729cf76320596fc50f87178d2))


### src/bin/fetch_by_filter.rs

- Use gnostr_crawler::processor::BOOTSTRAP_RELAYS; ([`91c0622`](91c06223b62a27312cb881341eacb625ac3ea406))

- Apply cargo fmt ([`4d48baf`](4d48baf74a03ccc197ebee733dc6a33cd30c0a3e))


### src/bin/git-ssh.rs

- EXAMPLEs ([`c6c58cf`](c6c58cf85e6fb9ecdcf4607db01638910197460e))


### src/bin/gnostr-fetch-by-kind-and-author.rs

- Arg order ([`c719615`](c719615419965c39955c643b7af96167ffba789d))


### src/bin/gnostr-pull.rs

- Remove ([`fbd21ff`](fbd21ffbecd79518430cfec4f127d7a6ba030466))


### src/bin/gnostr-verify-keypair.rs

- Apply cargo fmt ([`d96ab33`](d96ab33a025a5a5de1dd6e40165c0ae463cfc3d2))


### src/bin/test_relay.rs

- Remove ([`b1296d1`](b1296d1f8a62eca4f6904c222bce630e06d26a8d))


### src/lib/blockheight.rs

- Use crate::utils::ureq_async ([`82814e6`](82814e6f3cee51d97b025ee5b6e62d7fbbd7139c))

- Blockheight_sync:add ([`f6f4c26`](f6f4c2630d8b3939313cdfbfbedf3135d31bf499))

- Ureq_async:ureq_sync ([`feecae6`](feecae651c35f18d5c64439112a0d1a7a613ac2f))

- Unwrap:to_string ([`8997d4b`](8997d4b1c4d767c547f2c12993f568ceda8c6f2c))


### src/lib/chat

- App.topic:type tui_input::Input ([`b6d9f39`](b6d9f3940fb1d508ba5cbd557c17b8b660270302))

- Intermediate:Msg formatting ([`b838019`](b83801932a7eab67dc7fae110a4693abeb0e718a))


### src/lib/chat/mod.rs

- Set_content:with index ([`2695e8f`](2695e8f52171eb74ff5a2b76cc5fca0b0bbfac94))

- Gossipsub:app.topic formatting ([`ae87eac`](ae87eac02923a6125dee90346fdd1b993adaff64))

- Apply cargo fmt ([`9924ca3`](9924ca33408a9b02aec71ea66653134de652a3d1))

- Use gnostr_asyncgit::commit::de/serialize/deserialize ([`d15f864`](d15f864f4164e1c940433f691053b1e48012a79a))

- Apply cargo fmt ([`176a2c1`](176a2c12993e3d362fe8ba856283f456d771f4e7))

- Use gnostr_asyncgit::sync::commit::padded_commit_id; ([`bb3e0bf`](bb3e0bf5a3b28f7192857e320b382df064078548))

- Apply cargo fmt ([`3c9d89f`](3c9d89fad19706763b32dc4448bee9dddb79ae7b))

- Chat_version ([`1f5627b`](1f5627b75284ff05b4bff57cefdf7bf93a6754fb))

- Env::set_var("USER", &name); ([`3d34f8e`](3d34f8e1cf340654b0d90e371747418614f7b4ad))

- Value.send(m).await.unwrap_or(()); ([`341285c`](341285c424dc0106c282ea78618c46d11a36eff7))

- Add some log directives ([`6cf41fd`](6cf41fd247504234f2c0678065788043fd028bc2))

- Env::var("USER") ([`e8df8be`](e8df8be59056f45c199a6916132707ef83f478ce))


### src/lib/chat/msg.rs

- Set_content:index ([`26888f5`](26888f5d7df24c4696cde7e285b8b36eed19642e))

- GitCommitDiff ([`31e4b5c`](31e4b5c7ccae7b0dd81319f3dabc06a7e1cad210))

- Join:weeble/blockheight/wobble ([`4e376da`](4e376da33e782ed4e2395200d604ec18e40ff3f2))


### src/lib/chat/p2p.rs

- Set_content with index ([`4d607ef`](4d607ef9252c246aa24115662bf2750b963b3dd5))

- Debug! begin/end loop ([`9e6fb38`](9e6fb38680013eb7099b305f804390e69da31a5a))

- Pub async fn async_prompt:remove ([`ee1793f`](ee1793ff3f456452ce2a5bacedf5c546c3e84954))

- Handle.await.unwrap_or(()); ([`f8619c8`](f8619c88695dbe7b8f76bc9524525cf79f6f0236))

- Evt_loop ([`9d45ad9`](9d45ad9e236e40d54658dc16df52de23ad88fb7e))

- Apply cargo fmt ([`1124f7c`](1124f7ca4fc36e512d61ab1e6539c38dab7acc31))

- Thread::sleep 250 millis ([`8fdc3f7`](8fdc3f7ef78bc6fe59dbf70b0b75b350b4c708ad))


### src/lib/chat/ui.rs

- Intermediate ([`bed105d`](bed105da6e137bb5336a8450da76647d71b18c5d))

- Contraints:fit git commit message ([`1eb659d`](1eb659dfbc006bfd8fc30ef3c531d7d368f0a412))

- App.topic clone ([`d1d470c`](d1d470c62924684dea140c1f4385b2d92e44dadb))

- Test message <ESC> ([`72542a2`](72542a2e3b405ba80c8986f5b94cdae5aea9e011))

- Test message <ENTER> ([`06ebc11`](06ebc11180db624a1c68cf2d51d2e39a050b90c8))

- Normal Mode <ENTER> message ([`a715c77`](a715c77582ddbf2b9d708bf4294c7e945295b781))

- App:diffs ([`44577b2`](44577b23591bc23efed2f5d0be2519e3e0401874))

- NormalMode:backslash ([`ab30cc7`](ab30cc70a13521cd04792e6331c3b9e2b078b25d))

- Modal Commands:begin ([`62bbbbc`](62bbbbc04358bae2fa1c5f91c5e818420ea2f1d0))

- KeyCode::Char('?') ([`08b1bbc`](08b1bbcede9b4c00ecd7dd1831f1102693c5bb0d))

- App.input_mode:handle Modal Commands ([`2e60287`](2e60287e3eaad748861f11d3973de93fc23a651a))


### src/lib/cli.rs

- Remove dead code ([`426caec`](426caec41998c733f1c125d57207b46726a497ab))


### src/lib/mod.rs

- Weeble/wobble async ([`b1ab61e`](b1ab61ee0a85afa01fedb12ff7773e4691a3c4b7))

- Apply cargo fmt ([`55c91d6`](55c91d6af50c288696c991936d6acde17f494244))

- Pub const VERSION ([`0c4e791`](0c4e791d204946c8e8e9ad67ee47c5a48a2b166f))


### src/lib/ssh

- Logging config ([`9b1f29b`](9b1f29be239afaa7b9a42a6c572683510ad9a874))


### src/lib/ssh/config/repo.rs

- Remove _get_config_file_path() ([`72cc409`](72cc40964fc19fbdbd7c7112870090fc46a47f8c))

- Debug logging ([`2377830`](2377830529066c388cfde0b7f8129c37491db21e))


### src/lib/ssh/config/server.rs

- Apply cargo fmt ([`3bd7e2c`](3bd7e2c4df6eb6a06ed5a8954a93f72adc6013b1))


### src/lib/ssh/ssh/commands.rs

- Debug logging ([`4f3674f`](4f3674fd93e1258cd6bde06699caad9c1537b150))


### src/lib/ssh/ssh/mod.rs

- Handler:welcome_message:extra:toml::Table ([`df92fd4`](df92fd4d8b85fa374a1658373b0343d72d8d6254))


### src/lib/sub_commands/broadcast_events.rs

- Args.nsec:initial impl ([`a2e2b81`](a2e2b81a02e238b9ca3d89bf9e51cf60c390b362))

- Debug for relay in relays ([`bea8afa`](bea8afa4b54fc45d5ebf763c1bb0d781d8a16c6b))


### src/lib/tui.rs

- Clean up code ([`0e57b64`](0e57b6404ce0c71251ca295ab9edf92c434c4e9b))


### src/lib/utils.rs

- Fn byte_array_to_hex_string ([`9c05b38`](9c05b389285730d211d7b616ba56994e805574ef))

- Pub fn parse_json ([`6f285e1`](6f285e15a230e7be6881fd609a2f7c0f64e5f13c))

- Pub fn split_value_by_newline ([`5ef0143`](5ef01432b3bc39b9bb0ddbd9cabdbd918e8a23bc))

- Pub fn value_to_string:pub fn split_json_string ([`e8a0578`](e8a0578cb51c5c086e6e0f7ef82b0dd67c5fc501))

- Pub async fn ureq_async ([`5b077c1`](5b077c14e95bdb30a08f076f7f4cde8274712003))

- Ureq_sync:apply cargo fmt ([`39cec1b`](39cec1b0c15968449eb7ef21c314ae3e2b8ff20a))

- Agent use from_millis(250) ([`9b08535`](9b0853524f0cfa63a742f2b7a5902eb1f9c034ea))

- Ureq expect ([`03cc744`](03cc744ef01c0b06327fde51d65d75324c11881b))

- Docs stubs ([`0418364`](0418364bd7926d9d27f2bfc0c4860105ee8ea84f))

- Clean up comment ([`5d38511`](5d38511fcd0b91831349bb9d1c00e33fc9f914de))

- Intermediate:handle ureq error ([`79293c4`](79293c4276dda1570067c1f4a98ea88df425eeb9))

- Code clean up ([`f25552f`](f25552f1b12c8bac6a8ea1b936a3055dbbe0de93))


### src/lib/verify_keypair.rs

- Initial impl ([`309237b`](309237be4a198c80f04379eac5741472018cee6d))

- Apply cargo fmt ([`eaf6b07`](eaf6b07404bd40aad1119d31e8533d3b0dc25a84))


### src/lib/weeble.rs

- Use log::debug ([`1dad73d`](1dad73d02203b9cbe4083937794546860b68f64a))


### src/lib/wobble.rs

- Use log::debug ([`804b3f6`](804b3f694e09915089beab688fb232a72c15c0d3))


### src/main.rs

- --hash arg ([`b261e9c`](b261e9c07495a3da2bbaaad4ef2d21d135a9b1ab))

- Env::set_var:BLOCKHEIGHT ([`0a50be4`](0a50be41170970be11b6b5213338367318081ed3))


## [0.0.101] - 2025-07-14


### app/relay/examples/demo.rs

- Demo use config/gnostr.toml ([`b0843eb`](b0843eb3684ffcac2bc2c494a21c90d6dd6a12cb))


## [0.0.100] - 2025-07-14


### Cargo.toml

- V0.0.100 ([`a34bb8f`](a34bb8f3bb07603aa6860af819232a2e4b4606ee))


### examples/gnostr-chat.rs

- Apply cargo fmt ([`5e2e8fe`](5e2e8fe21f0fa488764d5a0ecfba7c06e8efb984))


### make

- Plan-dist-manifest ([`d58be42`](d58be42a39e833be48c77ee4f8632023b21635b9))

- Gnostr-chat ([`b175dbe`](b175dbe9b5753cbb8d2e7aa420c89188bf517df2))

- Gnostr-chat:weeble/blockheight/wobble:alias ([`e21430b`](e21430b5bde28319ed47cb4d3b9e872e765ec826))


### plan-dist-manifest.json

- Windows-latest ([`020be7d`](020be7d393b372cc89ea06f7861dd685c8336d21))


### src/bin/gnostr-pull.rs

- Apply cargo fmt ([`c3f532b`](c3f532b51e798a809ae4cbb3a7446baed539b6e3))


### src/lib/chat/mod.rs

- Refactor:if let Some(repo) ([`2522258`](2522258e13153194019d301a29e5f6343267fdfe))


### src/lib/sub_commands/chat.rs

- Apply cargo fmt ([`768f31c`](768f31c38fa2e6aa423e0329a146ffa58b108065))


### src/lib/sub_commands/tui.rs

- Apply cargo fmt ([`8fceb8e`](8fceb8e2c5c2f47467f6cab7a16a32009cf02455))


## [0.0.99] - 2025-07-13


### .github/workflows/matrix.yml

- Simplify ([`9a5b5d6`](9a5b5d6a2a5e1af1881fa7cca8238ca6c53715af))


### Cargo.toml

- Apply cargo sort ([`93cc445`](93cc4453058a4b5615db55d73412071d693e7f73))


### app

- Initial impl ([`7740cb0`](7740cb0d9a21bf4bcc6d2c8a5c4052ed7b95a7f4))

- Gnostr-relay:gpui:initial impl ([`224f842`](224f84280dc6670539513b0c4beaa04bd02c7a76))


### app/config/gnostr.toml

- Default ([`c1f55e9`](c1f55e908aaafbfef61d25bd9dddd185d380622c))


### app/src/main.rs

- GnostrApp:initial impl ([`6707028`](6707028d5f265750efef186dc86df21466541bf2))

- Fn gui:initial impl ([`0ea682d`](0ea682d62104623bc13d355e7fdfe44621364c93))


### make

- Post_event:post_from_files ([`cd23c47`](cd23c47e6c1d5446ee60dc592b21ce11ede611ca))


### plan-dist-manifest.json

- V0.0.98 ([`3888d1a`](3888d1a76f259911a67908645b44f305a042edd3))


### src/bin/post_from_files.rs

- Args ordering ([`105cc3f`](105cc3f80f97148446e3154b7386de98232233a2))


### src/lib/components/topiclist.rs

- Intermediate:apply cargo fmt ([`283e029`](283e029a1eddffb74bafe8acac88bd7c61e66bfc))


### src/lib/mod.rs

- Apply cargo fmt ([`4c428bd`](4c428bdd344218eac9f9e1c2db391a9b5318836a))


### src/lib/sub_commands/react.rs

- Apply cargo fmt ([`1b11492`](1b11492a400e63892b396e09acf5c4458ed78a4d))


### src/main.rs

- Apply cargo fmt ([`00f7e8f`](00f7e8fe61a2b5e29ddf25f1ad0f5d7de5bedb04))


## [0.0.98] - 2025-06-29


### crawler

- V0.0.8 ([`8d403af`](8d403af9e39dc83c05b4ffcab46c1925a06e7e89))


### ghcr.io.yml

- Gnostr tui ([`5edecec`](5edecec75c017f2a03a00f72671b258c5c7d8c07))


## [0.0.97] - 2025-06-19


### gnostr-xq

- V0.0.3 ([`4b6b99e`](4b6b99e8a20f4236872a68c2ec2061cdb6ad881b))


## [0.0.96] - 2025-06-19


### .github/workflows/matrix.yml

- Cargo install cargo-binstall@1.9.0 ([`3f36256`](3f36256dd3929f60797ce7afb067ab5dac9a3604))


### crawler/src/lib.rs

- Add BOOTSTRAP_RELAY4 ([`1b2ad94`](1b2ad9460351320345242dffe85302779a0b13b1))


### crawler/src/processor.rs

- BOOTSTRAP_RELAYS ([`f1f0091`](f1f0091010fef2c84c9c49853860bcdb48a604b2))


### examples/gnostr-remote.rs

- Apply cargo fmt ([`0dfed50`](0dfed50468aca01c385dc11701339cf248f93994))


### gnostr-remote-test

- More ([`ae3d43f`](ae3d43f04f1f39cd318dfec91c1e5d2c280d710e))

- Gnostr react ([`bc4977e`](bc4977ef377142ead988aee0b00286e504565d67))


### make

- Crawler-test-relays ([`c3dd12c`](c3dd12c8311e9ae4c8c71d7344cd5e4703cf1c55))


### src/bin/bech32_to_any.rs

- Remove ([`5986fc4`](5986fc490e192e85b05bbef91ee862c40849c811))


### src/bin/fetch_by_kind_and_author.rs

- Get_weeble ([`0ccfa0a`](0ccfa0a995f9340d2c9817ebdf265752449a7aec))


### src/bin/fetch_by_kind_and_author_limit.rs

- Get_weeble ([`7682f8d`](7682f8d0440226874c40bd6f44ef72eb0d71b482))


### src/bin/fetch_metadata.rs

- Get_weeble ([`83b3fe0`](83b3fe0579907bfc0927abd5d8addec29ac4bd4a))


### src/bin/gnostr-bech32-to-any.rs

- Print event id only ([`559db09`](559db098881bc1e3df1b24589e1ef7132b7bbaa9))


### src/bin/gnostr-fetch-by-id-with-login.rs

- Begin get_weeble ([`8918c68`](8918c687fab3d36ea1d63dbe7a80aed90a8a53e1))


### src/bin/gnostr-fetch-by-id.rs

- Get_weeble ([`d3c514c`](d3c514c2846b577b009d71d83bcba9a15f3bda06))

- BOOTSTRAP_RELAYS ([`e918d19`](e918d19a47358fc8c7b9d693696b0c1613d57548))

- Debug! ([`a66998e`](a66998e3b67641fe226b9f323a01fbfc95bff109))


### src/bin/gnostr-fetch-metadata.rs

- Use BOOTSTRAP_RELAYS ([`f501328`](f5013283baa290b644c59f9927a732a0ae7cb892))


### src/bin/gnostr-privkey-to-bech32.rs

- Use env and no dangling new lines ([`fe7e96f`](fe7e96f8e8895894b307539fdc3f73d97e8c5a45))


### src/bin/gnostr-remote.rs

- Logging config ([`2348763`](2348763de849b103d53074c5ecd0cb5493c63b37))


### src/bin/test_nip46.rs

- Get_weeble ([`7dc1106`](7dc1106c335d7245a893093c7115dfae31888ab9))


### src/bin/test_relay.rs

- Intermediate ([`08c232a`](08c232ab4f36fff5b77ecb730b5f9448638d3ca6))

- Apply cargo fmt ([`4e7e1d9`](4e7e1d9bf762cd8cf407890c2b9751d664d95ead))


### src/lib/cli.rs

- Info warn logging levels ([`6425cab`](6425cabaaa5900d9738c031f213274298e6d6417))


### src/lib/mod.rs

- Weeble/wobble_millis ([`5a48b3a`](5a48b3a3773f9761643196c7b5bbf4621bef51bd))

- Intermediate:impl Probe:test_relay ([`30671d9`](30671d9a0520d8c320c85b1bd18b80f1da55b6f2))

- Debug! logging config ([`34f559e`](34f559e1be408aadca94270dd0625cab99ea5aa1))


### src/lib/popups/inspect_commit.rs

- InspectCommitPopup:comment ([`7d3bfde`](7d3bfde003e40625dbb0d280dba16e7370b6e4f5))


### src/lib/remote

- Poc for distributed/decent CI service ([`9cc6cd6`](9cc6cd62dc3ab4923bdbd6742afa4248d3cec393))


### src/lib/remote/message_stream.rs

- Apply cargo fmt ([`145cb9f`](145cb9fee8c17a9b3c7dcce25ab3d8f113c3c01d))


### src/lib/remote/options.rs

- Clap config ([`776c485`](776c4854c4babfdb19da40e46e4a5917090854e6))


### src/lib/remote/remote_runner.rs

- Apply cargo fmt ([`a9fb94c`](a9fb94c2582e9052844f1015962e9ba4886cddd1))

- Detect not windows:set_executable_perms ([`aff7cb3`](aff7cb30b61581732a70ed78d6312f6e79c46b9d))


### src/lib/sub_commands/react.rs

- Relays = BOOTSTRAP_RELAYS.to_vec(); ([`b0add5c`](b0add5ce9d47aadc1a14c6ea655f3b4051f913ae))


### src/main.rs

- Info warn logging levels ([`ac9a2eb`](ac9a2eb4d110d27a75a7385aafc8762ba2d3aa37))


### sub_commands

- Derive Debug ([`d18c504`](d18c504a071cc512738809f4010d4656e8f18281))


## [1941/901551/193599/9f8d4495/c31cba92-src/lib/dns_resolver.rs.working] - 2025-06-16


### examples/dns_resolver.rs

- Ip output ([`9b0e0c8`](9b0e0c85685ea5c35e5c6fd82dfd18b84ab29e6e))


### src/lib/dns_resolver.rs

- Initial impl ([`9f8d449`](9f8d4495c250abdbaceabe3157c5bdb39535252d))

- Working with example ([`c31cba9`](c31cba921c9816b6d4207d3a052fb434c7360d37))


## [1941/901544/202722/c6bf04ef/95ef5d8c-dns-resolver-initial-impl] - 2025-06-16


### Cargo.toml

- Use trust_dns_resolver: ([`95ef5d8`](95ef5d8c27069d4079bbbbf4c0061efb6ff48195))


### examples/dns_resolver.rs

- Use trust_dns_resolver::TokioAsyncResolver ([`c6bf04e`](c6bf04ef3d0698cc701a1710d9b0171cfeccc244))


### examples/nostr_fetch_by_filter.rs

- Nostr_sdk_0_37_0 ([`15336b2`](15336b2c89ac54d2baeb88989aff4a1384c99cb0))

- Nostr_sdk_0_37_0:apply cargo fmt ([`166bddc`](166bddcc3c1c7b790ef96242cbe3904e5862d568))


### examples/nostr_fetch_git_patch.rs

- Initial impl ([`87a83e4`](87a83e4d01ecb643b0d6fe220e982164bc021283))


### src/bin/fetch_by_filter.rs

- Usage note ([`945a1f9`](945a1f9a5a0d0e3483f5c5adc9cf26fc66866c59))


### src/lib

- Intermediate:app.rs and key_list.rs ([`ec73032`](ec73032ff0672a44370813d6f1b3fdee193d8daf))


### src/lib/app.rs

- Bypass self.do_quit ([`38dd327`](38dd3278260755d699797cef9df488444ccce2a7))


### src/lib/gnostr/ui.rs

- Disable Char q for Quit ([`90cf2b6`](90cf2b61c07874f0aeeccf7437e1710ce5ffff33))


### src/lib/internal.rs

- Filters_to_wire:get_weeble SubscriptionId ([`a9621da`](a9621da018c0459f2d18f988adeb896533609a74))


## [0.0.94] - 2025-06-14


### Testing

- Intermediate ([`43b17f8`](43b17f835d4b3819d400b70c77fa24e8ab4da2bf))

- Intermediate ([`d56bc95`](d56bc95f490ee0a34ecceabf82b2492bd9c256bb))

- Src:cargo t passing ([`9b32ccb`](9b32ccb44f8d4c33088e06e59764ba029b7c73e1))


### asyncgit/Cargo.toml

- Dirs@5.0.1 ([`20bb3a3`](20bb3a3cac75eb18cfeab2ba8b41cd042b1d7773))


### src

- Apply cargo fmt ([`ed817b8`](ed817b8f75fb91f0103b3c5ebe5f7a8d46a3c123))


### src/bin/gnostr-blockheight.rs

- Bypass test ([`33d211b`](33d211b8dba3b2276df66866d78e586a01a1c62c))


## [0.0.93] - 2025-06-12


### Cargo.toml

- Gnostr-crawler:v0.0.7 ([`96a08ca`](96a08ca28740d3e4f3a44e79d0cbb5136bac1c4a))

- Gnostr-crawler:v0.0.7 ([`f9b4002`](f9b40021ec9e49509441c3c06eb1056979b9fb10))


### crawler/Cargo.toml

- V0.0.7 ([`724f04d`](724f04df37b01fc46b963b025a139b1121ace5fb))


### crawler/src/bin/gnostr-watch.rs

- Command.arg:default ([`8d121e8`](8d121e83bf1c98a24762d6ed2ef03899a2f9039e))


### crawler/src/lib.rs

- Processor::BOOTSTRAP_RELAYS ([`8658933`](86589337ebb67c6443272f7d4b9e5de116610b9d))

- App_keys:dirty impl ([`20f8e49`](20f8e490dfd6b15ea97cb3f282c113d1dbdcf46d))


### crawler/src/main.rs

- BOOTSTRAP_RELAYS.get(3) ([`e1f56b5`](e1f56b56dad87c42d18b5fb4a78b47c3fd72d398))

- Apply cargo fmt ([`054ff3b`](054ff3b6ac3e5253680b3c27fccdfaef5f51cfdf))


### crawler/src/relay_manager.rs

- If url not contain ([`a62bceb`](a62bceb3aac68c9fe9f1a8a53df6821ded2b93a8))

- Apply cargo fmt ([`f74bad3`](f74bad3f7da7b89b3907bdb819b1c5050bb74705))


### src/bin/generate_keypair.rs

- Remove ([`cc5730f`](cc5730f9e39ef388d24e1f02edca94c9ff60737a))


### src/bin/gnostr-pi.rs

- Initial impl ([`8d8b8d6`](8d8b8d6704ae5b26f51687470289ebedfef64102))


## [0.0.92] - 2025-06-08


### .github/workflows/matrix.yml

- Fetch-by-id ([`c7642c3`](c7642c35cbec4e51474f47698c3bec77f9be0602))


## [0.0.91] - 2025-06-08


### make

- Fetch-by-id ([`2b41182`](2b411824e81b9a440cebc3ee087ae4bec745e4df))


### src/bin/fetch_by_id.rs

- Remove ([`7563391`](7563391d67d0158b39b3a47bef289749d4a644e3))


### src/bin/gnostr-fetch-by-id-with-login.rs

- Intermediate ([`16c8e7b`](16c8e7bf79410ec8a4a3c8ee9d4fae87ef6f9d8f))

- Intermediate:incomplete impl ([`b1fcc4e`](b1fcc4ef7234a0f1c9c133bcd4b9d3bc14e48a59))


### src/lib/sub_commands/note.rs

- Sub_command_args.verbose ([`712e805`](712e80575400db3fac42cb402736fd80f74bb5ab))


## [0.0.90] - 2025-06-07


### types

- Cargo.toml:v0.7.6 ([`9f2ca95`](9f2ca95f1c63b180c0755de2d2c13cda9b4d3f49))


## [0.0.89] - 2025-06-07


### src/lib/internal.rs

- RelayMessageV5 ([`37a4bbc`](37a4bbc4fa885fad4e7019814d30fd961970eec5))


## [0.0.87] - 2025-06-06


### Cargo.toml

- Exclude .gnostr ([`7aa31f4`](7aa31f438508a400a01c47baa129f4452c539bbc))


### src/bin/git-ssh.rs

- Print SERVER_TOML on error ([`87cdd48`](87cdd48a9e94c57287467145687129da8ddd44cc))

- Print some help ([`cfe10fa`](cfe10faaa354f694cafa38367b6c0cff9e1076de))


### src/lib/ssh/config/repo.rs

- Apply cargo fmt ([`9f7419c`](9f7419cbb34ac2495eae67883126f715f90c999f))

- Load_repo_config ([`eaec9dc`](eaec9dc056efeaf38db675e6b7411792a19a5322))


### src/lib/ssh/ssh/commands.rs

- Print command ([`36c91f9`](36c91f9bf729e96412043dce4d67ccfa93a6dbd7))


## [0.0.86] - 2025-06-04


### src/lib/ssh/config/repo.rs

- Dirs::home_dir impl ([`9bf16a5`](9bf16a51b28e6c4e92bf6a79104e9e9c39a20717))


### src/lib/ssh/config/server.rs

- Derive Debug ([`1502ec1`](1502ec166c6bd8ad5b9da0ba19578bd57f99c9dc))


### src/lib/ssh/state.rs

- State derive Debug ([`8bdc312`](8bdc3123a3a29c4f76a5e2ae46c7296c27abb47f))


## [0.0.85] - 2025-06-01


### Build System

- Update detect brew and OSs ([`a5b5d81`](a5b5d815975f39e5e8ff5135b5b8a793bfd804db))


### Cargo.toml

- Add gnostr-bins deps ([`473915b`](473915bbd63b53bad79bc6ae79d21ede5abb7583))


### gnip44

- Add ([`438f05c`](438f05cd68c40173b697797f2beb2dc08923e45f))


### src/bin

- Add gnostr-bins:migration ([`3bba617`](3bba6173d30a6aac6d3bafc89d6bc66694a8821c))


### src/bin/git-ssh.rs

- Static SERVER_TOML ([`9a16e6d`](9a16e6d11d407bc5ff543143abe2343d054d64c2))


### src/bin/gnostr-objects.rs

- Remove ([`2f1c269`](2f1c269dded404aa2c2cba0fd6dba722acfeaabb))


### src/bin/gnostr-state.rs

- Remove ([`fe43128`](fe43128a1594de94488734d5b935ccc24173c1f9))


### types

- Gnostr-types:add ([`6843219`](6843219a02a1733fdc419c0a439ed65726ecb999))


### vendor

- Remove ([`5c999ff`](5c999ff837c776ffb100b26b10776d9b9faa3cd2))


## [0.0.84] - 2025-05-30


### Cargo.toml

- If not windows:sd-notify ([`5d1c34a`](5d1c34abf4f12288acea644bebab64dee143a0fc))


### src/bin/git-ssh.rs

- Initial config ([`b9f8774`](b9f87747b4c22c775954d8b2b7a54a793e709776))


### src/lib/ssh/mod.rs

- If not windows:sd_notify ([`6e28423`](6e284232d9f276451038eb8b47ef6b58eb1a0917))


## [0.0.83] - 2025-05-30


### dist-workspace.toml

- Add linux-musl ([`19c7a9b`](19c7a9b52286d7d93ebf2165d7cc6b66afafe7d5))


## [0.0.81] - 2025-05-29


### .github/workflows/matrix.yml

- Error formatting:src/bin/gnostr-loris.rs ([`fd44bbe`](fd44bbe011646591db3bda2ade03f7ec0b3acee0))


### Cargo.lock

- Some gnostr-loris deps ([`faadc40`](faadc40aa3df03711699a8cb1944971dff05f8a9))


### Cargo.toml

- Some gnostr-loris deps ([`2cbfc15`](2cbfc15b09820f54d8956672d3f6bb903d87dcc3))


### asyncgit

- Doc stubs ([`8f36aeb`](8f36aeb07c617cbbf13619360ae52729c5f3c55e))


### crawler/src/bin/gnostr-loris.rs

- Args:intermediate ([`cfc5329`](cfc53297d5b03598d4d814ad04324d08cd9924f0))


### crawler/src/bin/gnostr-test.rs

- Apply cargo fmt ([`cdcbd73`](cdcbd735168ca5442a68db0b2dafcf82789b8fc4))


### gnostr-crawler

- V0.0.6 ([`a5979a9`](a5979a975f6b5871c2979c4362e41f82452bcfa9))


### make_just.sh

- Update ([`d112103`](d11210327150f9b6edd7f057de0c1dd5a599915b))


### src/bin/gnostr-loris.rs

- Intermediate ([`49b6115`](49b6115f0bc2a6b18b0cc9285ae89d3660df16cc))

- Intermediate:args ([`d79b8d1`](d79b8d17d3cd230c4aeb106adb41f43e03e47694))

- Parse arg address:working ([`dbb6c8a`](dbb6c8ab2ddd2ad2d0d5dd5a4ea848acd0a3dc6d))

- Error formatting ([`1b5b03c`](1b5b03c48cd7e30cc0f8bfb888c2596447062c90))

- Output.stderr ([`6b4fabf`](6b4fabf83ddd67766180458cf6ebc6db76805ce8))


### src/lib/components/topiclist.rs

- Begin cube dashboard layout ([`6cd4ca2`](6cd4ca2ff5320a3694f1728a81c0a43035b9c0d4))

- Apply cargo fmt ([`32089d6`](32089d6942552ee4e2145e86862573d54abbe784))

- Some detail layout ([`aaea45f`](aaea45f9426956a27efe925cad8cee46fb10ecb1))


### src/lib/strings.rs

- Line numbers for test/design ([`d21b1f8`](d21b1f8f797068f161d7ff95d80edfa4640603f7))


## [gnostr-asyncgit-v0.0.2] - 2025-05-21


### Cargo.toml

- Update homepage/repo ([`80610bc`](80610bcf791a55b60db896893521c1855b8b96fa))

- Gitui:intermediate ([`c0e298e`](c0e298ea88ff57999e1e45e2f058472c2692c94b))

- Intermediate ([`5c5861d`](5c5861dc1f51e5210d9a8a5e06b9ff3c65ee4259))

- Intermediate ([`5e62e5a`](5e62e5ae82b71125b4db7bbcefe1e354d38b51a3))

- Cargo add --dev cargo-insta ([`030ab7f`](030ab7fcd421c928dcc1f728715f0b5528c62e82))


### examples/gnostr-qr.rs

- Add ([`eee50aa`](eee50aa82d1ba47f60b04f359b2d3a1f0abb6b60))


### gitui

- Initial config ([`0f189e2`](0f189e206489e790f49db3310737271799d84925))

- Intermediate ([`80bac99`](80bac99792ad03476559eece10cb287f27677155))


### gnostr-asyncgit

- V0.0.2 ([`52152a2`](52152a2462316e84707bc1e53f190f497725968b))

- V0.0.2 ([`d5c7a08`](d5c7a0811419ff611f9142ed72c90be0a15dd540))


### make

- Install:bins gnostr-asyncgit and git-tui ([`ec039d2`](ec039d23a37e85821adabec9c5ca879c9620757c))


### qr

- V0.0.7 ([`871c817`](871c81756057eee0eb58c7f1c4ed713bbe379ede))


### src

- Crate::gitui::... ([`5bd1c1a`](5bd1c1afa60481595e31e95bb8eab16fe7528554))


### src/gitui

- Intermediate:compiling ([`b0eec9a`](b0eec9a8339a305d23a3921cff83bd45c5ca7d52))


### src/lib.rs

- Warn missing docs ([`f84f280`](f84f28044cbc229c90a4124f859436a710e1cc9d))


### src/lib/cli.rs

- CAUTION! ([`3c21759`](3c21759c8a3c2962d1737f6a29efa7d368923f33))


### src/sync/hooks.rs

- Temp passing tests assert_ne! ([`4deaf34`](4deaf34e257fe9ab67111f23a0eb429049d49353))


## [0.0.78] - 2025-05-20


### Cargo.lock

- Gnostr-crawler:v0.0.2 ([`77ec591`](77ec591500a3df8ec207940a171cf6c4ff7c07fc))


### Cargo.toml

- Exclude vendor ([`8508153`](8508153b1888c7275f31ad2e580677b79127e21c))

- Add reqwest ([`fc32b09`](fc32b0944455287537f1d8f681b57ee080fb9478))

- Cargo add --dev dep-graph ([`4af3d9c`](4af3d9cf569f8895a242c6c6f467b41b847591cd))

- Tui-input:v0.10.1 ([`f663028`](f663028ddb3fd38ee8434f51fed27f3b7c519b69))


### crawler

- Gnostr-crawler ([`1e77288`](1e77288a1bd21aa2e1b58e3eed9cbbc56ac8bd15))

- Update metadata ([`49284be`](49284bee6ab06b0ed731c5a1b1e5e5f51619e616))

- Update:gnostr-sniper ([`4372301`](4372301b1ce12479210363a82f9b041b7f2a99a4))


### crawler/Cargo.toml

- V0.0.5 ([`941abe7`](941abe70839eaec318452081fd74b6744a552798))


### crawler/src/bin/gnostr-sniper.rs

- Intermediate ([`840390c`](840390c9c9e685c8f1d2e68fdb0f3aaaf753da57))

- Remove ([`09536fe`](09536feb69a382a4e404c60e1c9e304cdeec3267))

- Initial impl ([`9b5cb0a`](9b5cb0aa05ca150a675fdd4e19fa6bbad6e6a9f5))


### crawler/src/bin/sniper.rs

- Remove ([`7313c35`](7313c356e9a0b3d2f33127e968883d10cb88021b))


### crawler/src/relays.rs

- Self.print() ([`f20de38`](f20de3843aa80e1b50691626bc5083050e211142))


### examples/git-log.rs

- Use nostr_sdk_0_19_1 ([`0f7853e`](0f7853ee1394e606cc05bef636b85ac9a1b05afe))

- Relay_manager ([`78d89db`](78d89dbc3b74b1ac43bb67ccab00efcd42231e15))


### examples/parse_file.rs

- All ([`6198205`](619820560b20e300a356c98fe7d6d50da2bc801f))

- All ([`9dc1499`](9dc1499c4ef85db3ad31ff5be002ea60be786472))


### gnostr-crawler

- V0.0.3 ([`51fd6d3`](51fd6d305b8cd2360083e69083ac832a9bbd2f2e))

- V0.0.3 ([`96315e1`](96315e1c5918e3ed77552da2920e856e48f44501))

- V0.0.4 ([`7444171`](7444171d1fda16b08f0809d22002f96e32e1bff5))

- V0.0.5 ([`20d46c3`](20d46c345b1e62d732aa55d1e786829f61035eac))


### make

- Crawler:v0.0.2 ([`4c2f1ab`](4c2f1ab05dba9b73e095401c9e8b5513a0ffb47c))

- Dep-graph ([`4bfab1f`](4bfab1f6200e04beb4b267da75ef9b11324e0a19))


### src/bin/gnostr-sniper.rs

- Replace:http/s ws/s ([`b78c92c`](b78c92c4120423af333abf060bcf997cc9f779ac))


### src/bin/gnostr-watch.rs

- Intermediate:nip count ([`eef8611`](eef8611b57dcfb6cc29273d2b09fc5d874ffd81a))

- Intermediate ([`6d8abbc`](6d8abbc376a4e8b94d348739bf2509802a5a2968))

- Nips json formatting ([`5807fba`](5807fba2d61c12c82d02646bbc38fb4468aed701))

- Await ([`4d4b01c`](4d4b01c354c7362b06da463820850dd30229a270))


### src/bin/sniper.rs

- Replace:http/s ws/s ([`22dd339`](22dd33965a79798551274618e6e55e1e4786b313))

- Intermediate ([`2240fb3`](2240fb340f59625cd3f419a57ffa327dd00840c3))

- Intermediate ([`9d94159`](9d941593f7de400f140bc17d08f52c81727648c5))


### src/lib

- CliArgs:intermediate ([`efd24e9`](efd24e9653eddaaefaa1bf675f15376e0e8fb7de))


### src/lib..chat/ui.rs

- Clean up deps ([`55e27e7`](55e27e7429c4f2b507349741a46273c23881c594))


### src/lib.rs

- Async ([`03c28e9`](03c28e9ed28ccbde8173bed93cc8f337de0a6aff))


### src/lib/cli.rs

- Add NgitCommands:GnostrCommands ([`dbcd32d`](dbcd32d326b3abaa057bfbd43b2432b1035daad9))

- Intermediate ([`11e6e8b`](11e6e8bd7a75a689d5f63c3661897d17b1283308))

- Pub fn setup_logging/get_app_cache_path ([`6db19e4`](6db19e4bca77c352a39badea6e3a261a4be0256a))


### src/lib/components/chat_details/mod.rs

- Apply cargo fmt ([`88109e5`](88109e59d458cb1db0b2a3df86bb3146d779513b))


### src/lib/components/mod.rs

- Clean up deps ([`a3a4308`](a3a4308d7c71c0f875975868bd9af953f6eb8a35))


### src/lib/components/topiclist.rs

- Remove unsed deps ([`5b645a2`](5b645a2150b2361b4b602789a2e46dbb954f089d))


### src/lib/gnostr/mod.rs

- Gnostr tui:intermediate ([`d59903e`](d59903e63be607cb1ce7a24db34911a0da93d8a8))

- Clean up deps ([`3096d64`](3096d64e834621dbd5313c476b7799a53e063d66))

- Default for GnostrSubCommands ([`acdb3a4`](acdb3a467960e46e6cbb494fc8ec0a27f7379472))


### src/lib/sub_command/chat.rs

- Remove unused deps ([`10c1aa0`](10c1aa0b6e97b0bad814561a14ad3a486dc26188))


### src/lib/sub_commands/text_note.rs

- Code clean up ([`497dafd`](497dafdc65f9391b21a1505686ab7f21d418f521))

- Apply cargo fmt ([`b3f749f`](b3f749f1a3d753034cd342775f82a86ebce4eafc))


### src/lib/sub_commands/tui.rs

- Intermediate ([`052a8ac`](052a8ac6dc2fddb43bd4e66bf1bd78026d46e02c))

- Migrate:intermediate ([`5543252`](5543252309fca47a37e0188fc76c78f4415a343c))

- Intrermediate ([`9679f13`](9679f1392056a1d060393ad201d6477a7de74ce2))

- Loop:run_app ([`07f4642`](07f4642d6fc7184b668a23e00280ba6e47984022))


### src/lib/tui.rs

- Remove unused deps ([`4b46726`](4b46726f4b0cebf7f4ce79a65813e7cd10bbcfa0))


### src/main.rs

- Code clean up ([`32569f7`](32569f7d40f143e63da17802910612c8818d8657))

- Intermediate:tui subcommands ([`de6b903`](de6b9036b47ddb2c6f45398b5b1ebd6fbe047fb3))

- Trace logging ([`03ca517`](03ca517fd77a3d368b15de5a8e00a76fbb42d028))

- Setup_logging ([`82a1eb6`](82a1eb690dc7258ba5b0a48431d95afc4b9d9d60))

- Bypass use tracing::debug ([`a56d14e`](a56d14e221ea8c898a53f4d9a9786914de1598ab))

- Apply cargo fmt ([`d384367`](d384367cea3e985bcc744a2f11e5e0192d5bffdc))

- Match None args ([`3d9ba25`](3d9ba255300ffae61cd71ec2de307c50651d0617))


### src/relays.rs

- Async ([`982438a`](982438a7069bb46209396b279cc95eaa4528b641))


### ssrc/lib/tui.rs

- Cli:intermediate ([`660e20c`](660e20c12edc1db29b61c28061328541f533bd2f))


### theme.ron

- Example theme ([`46155f4`](46155f467f3f33131f1a4ae7878bd75ac5342812))


## [0.0.70] - 2025-05-07


### crawler

- Initial commit ([`1e559ab`](1e559ab73a8f68887561e368ed0502ddc6f314eb))

- Git-log.rs:json output ([`42bb84a`](42bb84aec2efee208dadf41f82021e67f7dfb5f7))

- Src/main.rs:json output ([`172c96c`](172c96c5d0ab6fc161aca26a475f03547d2b43ba))


## [0.0.68] - 2025-05-06


### make

- Cargo-dist ([`730b7fc`](730b7fceb4c276a30e29a6d1701d57e564c26071))


### src/main.rs

- Default tui ([`ed85d5b`](ed85d5b7bb138353f1fa5c033b5ee5f0004da10f))


## [0.0.66] - 2025-05-05


### src/lib/chat/mod.rs

- Public args ([`7651266`](76512666ce04094277df231e21cdad11e81a241b))


### src/lib/sub_command/chat.rs

- Tracing logging impl ([`520409a`](520409aac57f5fe4d7002021bf9d9878310cacfb))


## [0.0.65] - 2025-05-05


### nostr_sqlite

- :SQLiteDatabase:async config ([`7c1fe73`](7c1fe73b963b3168d15aa60c03d91941e4ea046b))


### src/lib/components/topiclist.rs

- CrossTermEvent ([`0f1e70f`](0f1e70f0fc34e78b87ab57e51307f7c990cae66f))


### src/lib/login/mod.rs

- Debug logging event ([`e2c67b5`](e2c67b5aefa1f32e83eb314521a9026ee884d50c))


## [0.0.64] - 2025-05-04


### examples/nostr-sqlite.rs

- Initial impl ([`de19693`](de19693249596459f66ad22126d32cbe144103ed))


### src/lib/chat/ui.rs

- Default:topic gnostr ([`3f0f02b`](3f0f02be159e5ba47688485c61a2f45d3dd15963))


### src/lib/client.rs

- Add search relay ([`46d8851`](46d885182dc21fe702a6a18236b876e76014fb12))


### src/lib/components/topiclist.rs

- Being get_user_details ([`1901e2a`](1901e2ab2446a0fe31a295aaccab26bb87d9b298))

- Login::get_user_details ([`1248b07`](1248b078e190c7b53db5f890c6aa8d861ee49788))


## [0.0.63] - 2025-05-03


### src/bin/gnostr-chat.rs

- Apply cargo fmt ([`ac01cff`](ac01cffda090042a25a1b571bea94d0a4e67dcea))


### src/lib/app.rs

- Cargo fmt ([`69fbc80`](69fbc8008dfd0c0b9ce63cec0a84d18e1c9fdd3e))


### src/lib/args.rs

- Apply cargo fmt ([`962ddce`](962ddce4c3f78c4150803aa167d4b8c20647dcba))


### src/lib/chat

- GitCommit:Type formatting:intermediate ([`b9e2ae4`](b9e2ae49e7dafeccd756c0d1cc2255f0d0800b26))

- Msg formatting:intermediate ([`7b2a0d9`](7b2a0d95cecb6b43c1d61e9337e502db6e98f7db))


### src/lib/components

- Apply cargo fmt ([`567fe85`](567fe8552bbc117468f2de2d976c6c85f90e8b2d))


### src/lib/mod.rs

- Handle --topic arg or commit id ([`b400cd9`](b400cd9f65f03148d7b93388a33b508b4e7172bf))


### src/lib/popups

- Apply cargo fmt ([`beac77c`](beac77c09baf19806de73a929859e2442ed165cf))


### src/lib/popups/submodules.rs

- Apply cargo fmt ([`b80ee65`](b80ee65077fdd06e61562ce6d6523241b610b18e))


### src/lib/popups/tag_commit.rs

- Apply cargo fmt ([`4cdf858`](4cdf85878bdf1a934537f2abc4cf0c3a6ad2b697))


### src/lib/popups/taglist.rs

- Apply cargo fmt ([`af6366d`](af6366d8b5be42a16d0f0cc365580c3de808c9c8))


### src/lib/queue.rs

- Apply cargo fmt ([`b9c5594`](b9c5594127438580ba8c6d8853f3d3233873d961))


### src/lib/sub_commands/chat.rs

- Apply cargo fmt ([`69cd640`](69cd64098688377c6a36af6e334c1eb3ba55a0d0))


### src/lib/tabs/home.rs

- Apply cargo fmt ([`ed3b5d1`](ed3b5d129865a1e6e4e640bd17a142d9fc91f45f))


### src/lib/tabs/revlog.rs

- Cargo fmt ([`4081012`](40810123ef65a0cd4b6a4783a6ab2df30c1cd217))


### src/lib/tabs/stashing.rs

- Apply cargo fmt ([`ffd7e8c`](ffd7e8c053e370fe5182536269cc3409620e9a8a))


### src/lib/tabs/status.rs

- Apply cargo fmt ([`d5185a2`](d5185a2de960f0ca1d0add2b60956707bbd339ed))


### src/lib/tui.rs

- Apply cargo fmt ([`30b2fb9`](30b2fb9b7a54557d2bdf87d226a16d6e47f32aef))


## [0.0.62] - 2025-05-03


### .github/workflows/release-matrix.yml

- Add windows-latest ([`c6024d3`](c6024d30891ee82d85de3d0e2c6d37d41b936741))

- Follow up ([`e6e5fcb`](e6e5fcb75ad3e524d804623b97cd003de500371d))


### .gitignore

- **DS_Store ([`319d5ee`](319d5eec3459a8d90b08f7745925f76639093e4a))


### Bug Fixes

- Git2:conflict ([`6682ee3`](6682ee3e3c2bb7063e6ff380c9b58080c63ee88a))


### asyncgit/Cargo.toml

- Gnostgr-asyncgit ([`7a560ff`](7a560ffc5431c11c0a9a61033b1072206b226084))


### chat

- Ngit:Box<dyn StdError>>:impl ([`4e9d1a1`](4e9d1a1f06f0a53a4709157f01fd56da7cddb144))


### ghcr.io.yml

- Config ([`0d35612`](0d35612790bfd80156daad79213a3954e272c373))


### maintainers.yaml

- Update ([`35ca5a0`](35ca5a00f2f2ca335f7d154fdc9d9a1f4eb5d968))


### plan-dist-manifest.json

- V0.0.60 ([`11e0d47`](11e0d47f095ac665ae3b17f66822fce75b7ba9ee))


### src/bin/gnostr-chat.rs

- Apply cargo fmt ([`aa603c9`](aa603c9f4d0bd0fe6be966631b9dd41059ce31d8))


### src/lib/app.rs

- OpenExternalChat:initial impl ([`a0fe3d9`](a0fe3d97502fd3ffce1e21c85b12f41610aea740))


### src/lib/chat/mod.rs

- Apply cargo fmt ([`d55535f`](d55535f70d1d421ab8e6a2de999e5d8236157cf7))

- Intermediate ([`d82faf3`](d82faf3dc6ff70512ddce23096ba2b54f4b9e1d5))

- LevelFilter:OFF impl ([`5a1c16f`](5a1c16f4370f2a8382b97677addf1a2a7e35f737))


### src/lib/cli.rs

- ChatCli and ChatCommands ([`87594ad`](87594ad1940af2e2365938dfb24584dfb08fd690))


### src/lib/components/chat_details/mod.rs

- Layout notes ([`5ca7357`](5ca73577d9d5cd3c1feccc32650cc4c93d4fffe7))


### src/lib/components/commit_details/details.rs

- Info header formatting ([`ae8169a`](ae8169acc78918a757334f362ed8d93851700eec))


### src/lib/components/topiclist.rs

- More_text ([`9618857`](961885754f7485578d6bebe555b4be749624b0dd))

- Intermediate ([`733e4f2`](733e4f25c26303c70cd38bcb66e7d16f49ee578b))

- Truncate pubkey ([`5a8e93c`](5a8e93c8e78b5d99188ddb19e806cf0ad77d06da))


### src/lib/keys/ley_list.rs

- Open_external_chat:initial impl ([`21d410d`](21d410d90ec6a74c27287ef1f548013a057bc435))


### src/lib/login/mod.rs

- UserRelayRef:guards ([`b9b6a36`](b9b6a36d0a519fb40f1a821b2d3c8356c9fa995d))


### src/lib/popups/chat.rs

- Initial impl ([`c3d7553`](c3d75531f945945da85b262493bb39cfc099615f))


### src/lib/popups/commit.rs

- Show_chat:initial implt ([`c9d231e`](c9d231e335f0324e14eca163b22e549b9b95c4ec))


### src/lib/popups/mos.rs

- Openchat:initial impl ([`64a2c5c`](64a2c5cb55f8b41eaf77addad208c9741b188bb8))


### src/lib/popups/openchat.rs

- Intial impl ([`72bdeb2`](72bdeb218a6e78597e441555c26050ffb558c992))


### src/lib/queue.rs

- OpenExternalChat:initial impl ([`15911b4`](15911b498cc23e80f53df22f8951bfb1cd211064))


### src/lib/strings.rs

- Open_chat:initial impl ([`f33f96c`](f33f96c54cca29a4197101710ed1f0503025cdd2))


### src/lib/sub_commands

- No use cli::Cli ([`ea9c91e`](ea9c91e96f6bcaeffb7b0bb3959f82d9047a1765))


### src/lib/sub_commands/chat.rs

- --topic arg ([`9cd3d4c`](9cd3d4cdc45a40162636911089c4b099a92d459b))

- Chat/run:initial impl ([`34b0e96`](34b0e967cfa04f4b42079985ad3f3511e0ac51ef))

- Intermediate ([`d685d26`](d685d2683288d3247933e2de0f42f6dc273dd410))


### src/lib/sub_commands/mod.rs

- Add pub mod chat ([`34be020`](34be020dbe58a341b12cadc616678f2241e4035a))


### src/lib/sub_commands/ngit.rs

- Apply cargo fmt ([`e5ed486`](e5ed48618fc49861e877fc855cf321db2523347e))


### src/lib/tui.rs

- Add lifecyle notes ([`b7f3ab4`](b7f3ab42d670d93d9bc775230a1a390e0c749fbc))


### src/lib/utils.rs

- Truncate_chars ([`b07eb43`](b07eb439ed7376cee98674b7e088dbf058975a09))


### src/main.rs

- Add chat subcommand ([`ccc4d8a`](ccc4d8a4a827373276d99a8bef9dee89b41ea0ad))

- Apply cargo fmt ([`6e746bd`](6e746bd95716ce26297c0dde6062b4b6681b2f8f))

- Apply cargo fmt ([`ac9b377`](ac9b377a8a84030748e9562bd55d983cbf76df53))


### topiclist.rs

- Begin nostr fields ([`eb15b2a`](eb15b2a5d04e795197bc3c3dfe5bdf053b0fcb30))


## [0.0.56] - 2025-04-27


### .github/workflows/release.yml

- Update cargo-dist:dirty config ([`2e2ecbd`](2e2ecbd4c279901e49fb3e2d0a6bbec25c444313))


## [0.0.55] - 2025-04-26


### src

- Cargo clippy passing:with many warnings ([`150f053`](150f053a021e1082a6a5e5afe30bfb11c0cd4f1c))

- Apply cargo fmt ([`1794b0c`](1794b0c350e7e4ef6aa79c3e4dda151978c2ab10))


### src/bin/chat.rs

- Use asyncgit::sync::commit::{serialize_commit, deserialize_commit} ([`7c7c18e`](7c7c18e43f5456db95cee37e66ae386fcc019769))

- Intermediate ([`691d986`](691d9869f44a96a77465b00f6c9a3870555b932e))

- Intermediate ([`75841e7`](75841e75e21bc135d78d0c0fc3aa99d6f2f97d1e))

- Intermediate ([`316cb0c`](316cb0c82bf822fa4d4ab45dcfa8aa67845fe50f))

- Intermediate ([`5beb11e`](5beb11ed3d8fbd123f237d4fea502136bfb96fa7))

- Intermediate-working ([`18e0c9f`](18e0c9f29ef1fec33e9bf0228a2f95b832993893))

- If args.debug OR args.trace:display keys.secret_key ([`7dfff63`](7dfff635cefd526153223344bdecf8f2a4ad9a4c))


### src/lib/chat/mod.rs

- Intermediate:migrate chat ([`5f16abd`](5f16abd17b17837fe4b9b1b9fdc80fa412245849))

- Intermediate ([`d0b784a`](d0b784a94393173103698f56d99b6ae5d87e4626))


### src/lib/components/diff.rs

- Begin chat graft ([`beb2874`](beb28740a16c9f333352d713c30abf5a9e6afda1))

- Insert pubkey ([`2e9a5a3`](2e9a5a35ebe119c8e57a550b49058524dc17acf4))

- Pubkey formatting ([`b2ff46c`](b2ff46ca14c8cb67eee3021c12b00aae12e5d4d0))


### src/lib/sub_commands/generate_keypair.rs

- Json output ([`66aaa5a`](66aaa5a9ef88fe5ce54adfcb6ea43d043150e12e))

- Json formatting ([`4dad046`](4dad04680dd8e7de98cf9f289b66fa90068e53a6))


### src/lib/tui.rs

- Public tui functions ([`84d73b3`](84d73b336a854ef7adb59a8192af8386d0cd85ff))


## [0.0.54] - 2025-04-23


### inspect_chat

- Initial impl ([`136f7cd`](136f7cd46cf70d489c62bb82be03d8e865e01062))


### src/lib/popups/inspect_commit.rs

- Diff split notes ([`da1084c`](da1084c6484be785d90572e4f5ccda5c3451638b))


### src/lib/tabs/home.rs

- Apply cargo fmt ([`32e7da3`](32e7da3b8398b60a2ee54171ffceb59640806d7c))


## [src/lib/tabs/home-rs/splitview-constraints-working] - 2025-04-22


### src/bin/chat.rs

- More Cli config ([`56d51f0`](56d51f06497c4d78dba2f8343c809341a8025cf4))

- --info arg ([`97cca26`](97cca268f31f10c65189c29b0f622ecf4fb0164f))


### src/lib/tabs/home.rs

- Topiclist_title:🅖 ([`e6ccaab`](e6ccaab54605c0aef67232f41f6f51cb06c00db4))

- TopicList:split view constraints:working ([`3df5c89`](3df5c8951bd7db9376d5395d7be4d15425df2c05))


### topiclist.rs

- CIRCLE_G_STR:marked ([`8122e62`](8122e6232be72c55de8f66f804a86b6b832cbf4f))


## [1953/893521/285779/a565d4ca/7fb39d02-chat-arg-name-to-env-working] - 2025-04-22


### .github

- From main ([`6f45c66`](6f45c66aef03146ac1bffc708148072fa6564b9a))

- From main ([`5fc7d72`](5fc7d72636491120f997862817b45ffefe0a8e55))


### asyncgit/Cargo.toml

- Add nostr-sdk@0.37.0 ([`2947ec8`](2947ec8dc47f8a5ed9c7a6d5c5bc6619ed6f9463))


### chat_details

- ChatDetailsComponent:more impl ([`fcf3e6f`](fcf3e6f5ee87fece7e876bfe7c8844703e5a46ea))


### chat_details_toggle

- Initial impl ([`44c323d`](44c323de3fac1b4b42e59cf1d5406c68c297e385))


### crates/_tui

- Remove ([`74d0742`](74d0742da19288dd7334745b715a7a62967ccbae))


### crates/ngit

- Remove ([`f225d6e`](f225d6e9b9f0fb29141e6f8da30bce21a0e0d951))

- Remove ([`03acd13`](03acd137bb97dfb6b0f9b81d5c3426c78dfd7239))


### crates/tui

- Add ([`97896dc`](97896dc585bfac684f252d3c7d14e69ba62b7f0e))

- Remove ([`b0be388`](b0be3886169433df644ac37c8c6f9107eff87227))


### crates/tui/asyncgit

- Preliminary async/nostr integration ([`95ad9b0`](95ad9b0d75f25755053d165b76dc3a040ef1a6ac))


### crates/tui/src/bin/ngit.rs

- Pre remove ([`396f4b7`](396f4b78fbf2dd399c0b29dda2573db03d371d4a))


### detail

- Split:better ([`6abfbcb`](6abfbcbe5f690b288198cdff8dcebc538ff71cdc))


### imtermediate

- Crates/tui:provides ngit ([`dee29e7`](dee29e711cbabf18ec44d6acf3ae774629904519))


### install_script.sh

- Make cargo-release-all ([`98ba5a6`](98ba5a60e6cb32a72ce58e0a5af6dc329bf77f24))


### make

- Cargo-install-crates ([`9b8b925`](9b8b9253163d047abbeb980eb3cd9e530da0d515))

- Cargo-release-all ([`0d8ec25`](0d8ec253019e6e99166de60837e9b34bd0918cc1))

- From install_script.sh ([`e2a1bdb`](e2a1bdb5cfbe02c470b88821ccef048e7cd1a18f))


### padded_hash

- Initial impl ([`116e68a`](116e68a07b0d746bc2b4bb92e76f95637fec79a5))


### passing

- Publish dry run ([`c99ac22`](c99ac224a2b6f3940d6093cf9b0f510903942a98))


### src/bin/chat.rs

- Impl ([`8480dc1`](8480dc1fbb3ef16aa63a294981519b759644aa58))

- Topic arg working ([`a565d4c`](a565d4ca85b5b5513d0fec78e737a0807ddeffbb))

- --name arg to env working ([`7fb39d0`](7fb39d020158654fc330d34d5dd38a7550c917ce))


### src/lib/chat

- Being impl ([`6e6917a`](6e6917aefad1d87474c890ef2221474ba8654bcb))


### src/lib/components/chat_details/mod.rs

- Intermediate ([`a96be5c`](a96be5c634319de73426bd591f399a24ea5f387e))

- Intermediate ([`bd3d580`](bd3d580242c8523a0457decbb58c25de7a890998))

- 3-way split ([`37291d5`](37291d50e234866435510adf3336131837ad8a93))


### src/lib/components/mod.rs

- Layout working:add notes ([`a30c082`](a30c0820315c95c697d91158c0d49408840d765d))


### src/lib/components/topiclist.rs

- Commit keys ([`77aa531`](77aa531796ba0de324f928abd05c1f5314df9f74))

- Add constraints ([`8bdd432`](8bdd43243986df646e5d67a93e5fea494285b801))

- Intermediate ([`b4cf508`](b4cf508ce3f07aed561fc406f9f3d79c62a5b386))

- Intermediate ([`f55255f`](f55255f88c22808b597b7a3cef7862c8032acac0))

- Split fixed ([`04c9199`](04c91990ce742d99176e52e7e858331162930589))

- Apply cargo fmt ([`85759eb`](85759eb01cc621b9b5b6181bb539a4b016bdd6e4))


### src/lib/mod.rs

- Chat impl ([`31b95eb`](31b95eb92528ae0cd8a6ca061bc0ea3c5d5a83fb))


### src/lib/popups/display_chat.rs

- Initial impl ([`c25a5bc`](c25a5bcfc44a514e946a0aa758427177423563de))


### src/lib/sub_commands/login.rs

- Disable_cli_spinners:Option<bool> ([`16f889b`](16f889b78a6e9a372a5dd3a0e58b931734309b4b))


### src/lib/sub_commands/set_metadata.rs

- Output json ([`fea9142`](fea9142dfdcb414adc267c352c6e0a7099143964))


### src/lib/tabs/home.rs

- Apply cargo fmt ([`be9d928`](be9d928394b39af4f3eeddb7d1dcb3b931784cc3))


### src/ngit

- Intermediate ([`bb1966a`](bb1966a7f2f56f4a4a5cb53366993234cf4bd178))


### topiclist.rs

- Symbol::EMPTY_SPACE ([`44954c0`](44954c07e0f9c44a8824e04cc5371fab508da7f9))


## [0.0.53] - 2025-04-13


### .github/workflows/release-matrix.yml

- Run gnostr custom-event -k 1 ([`c7b8e87`](c7b8e8777e0114bdeab59a790685f1557797b6d8))


### examples/input-to-sha256.rs

- Example ([`8e49384`](8e49384d03578dc77630e18657b4b3eb40b52570))


### src/global_rt.rs

- Intermediate impl ([`628d32b`](628d32b657a1602d2d38506b0541c4f09038c551))


### src/main.rs

- --hash to args.nsec ([`07fce82`](07fce82f73a095435b49b1eb0377b674abe99a10))


### src/sub_commands/react.rs

- Return json ([`4213c5c`](4213c5c0e4466119ee541158f656e819cc5e4792))


### src/sub_commands/text_note.rs

- Id and bech32:json ([`980cccf`](980cccf962838867fc7f7d3de2aa520795c30636))

- Return json ([`79fd93a`](79fd93a1a3e9cf91a9c67d12dd21d587eebbaa62))

- --hex flag ([`dc5cc66`](dc5cc668d8d5eb58bb8b7e77eb5f94e56f2e1699))


## [0.0.51] - 2025-04-11


### Build System

- Detect architecture ([`fd9b420`](fd9b420694d5d7bf54828fc4b29089c3c3090bf5))


## [1955/891801/829825/54a9efc/c02af3f-v0.0.49] - 2025-04-10


### .justfile

- Config ([`92e99e8`](92e99e83500c6768adee0024fb4e5eba7c2e09df))


### crate

- Gnostr-ngit ([`ff47ac4`](ff47ac494ee2c06e8472abadbfe394772596f0a5))


### gnostr-ngit

- A fork of ngit ([`2b35aa0`](2b35aa075a260ef49cb5c6bf1ca11ab4ab7c9990))


### make-just

- Config ([`d8325a2`](d8325a2e04a4ffb70f48592e69b2566b33a6688c))


## [1982/875027/63166/0f25070/fbb53aa-v0.0.1] - 2024-12-16


### Bug Fixes

- Code cleanup ([`d329260`](d329260d4aeb54a5b9abfc3e5091960a50466f86))

- List events ([`c883e08`](c883e088fbcafd2bb4f7dc9cf960858ccf9983ca))

- `vanity` short `-h` conflict ([`662a670`](662a6707c88247ee11c2b4e536178c5763af4304))

- Append tag key only once (#55) ([`5b7f266`](5b7f266062f3e8ffbc91d4b46ff3404749d5319d))


### Chores

- Bump `nostr-sdk` ([`7e23355`](7e23355b6638bc7ca435a6b2c5505d9192c5d31c))


### Features

- Add support for bech32 encoded keys in commands ([`053eabb`](053eabb19f804060a075adad3814920249d909a8))

- Print bech32 keys ([`59301f5`](59301f5641ebadaac68aa47b3c5b6b020d5a5b6a))

- Add generate keypair command ([`e00c14b`](e00c14ba88bfeae18841ec8e7c5a5b36c78860c8))

- Add key conversion ([`0f3dccc`](0f3dcccc9c3dcebe6d19f3aaaa6ff310a8020e84))

- Add support for converting noteids ([`eb5ece9`](eb5ece90f45a7183c38d277c2a9d04f9abf9174a))

- Add support for LUD06 in metadata-update command ([`cce8464`](cce8464a32d276d79ea1cf22508807dd794041b8))

- List events save to json file ([`542b962`](542b9628d3bbcc1346c6af87e2d4ad84eb65b2c4))

- Broadcast events from json file ([`06c226f`](06c226fc6c8e718d62f819dbae27095182f6cd8e))

- Option to print hex ([`00e9275`](00e92757e130c577b1d1622626b3f95c2100d77d))


### NIP-28

- Add support for creating channels ([`e71bde9`](e71bde9fddf0e101c139cf4d2f059ecbf1db53ca))

- Added kind 41 support ([`f41febc`](f41febcfebedcf67dd8dd7ad0aca86365f9c0b75))

- Add kind 43 support ([`a08abf7`](a08abf73f9c3ac635167f20dd9236e546e6f3601))

- Add support for user mute ([`4ac6458`](4ac64580c0ce5629a539b31ae333419992fbc6b1))


### command

- Gnostr:author:gnostr & 0xtr ([`0f25070`](0f25070648bcb480c0fe057a8995e7bbedac6dc8))


### format

- Cargo fmt and clippy ([`7eef201`](7eef201e371f348476b88e99ff0dcd263f65704f))


### gnostr

- V0.0.1 ([`fbb53aa`](fbb53aadc0411b76314f8fd978a71bcdba28d9cf))


### improve

- `list-events`  (#53) ([`18d01e2`](18d01e261c82159c464dff2ae8ecc75171026c88))


### improvment

- Pretty print events as json ([`7c12338`](7c123385047e6b985b229f6f14d4a14287f6f382))


### package

- Gnostr ([`97e45c8`](97e45c8762f41638e752f8ab8d00ed203cf08cb4))


### rust-toolchain.toml

- Remove ([`1baffc3`](1baffc33e07ce9ebb2380820d2c47dd6bea0ed7f))


### src/sub_commands/set_metadata.rs

- Banner_url ([`a9104f1`](a9104f129b6802a4195f14716a80600ccc22f67b))

Generated by [git-cliff](https://github.com/orhun/git-cliff).
