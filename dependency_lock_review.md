# Complete dependency lock review

Every non-comment lock entry was parsed in memory and retained below. These graphs include optional/build/dev/platform packages: lock presence does not prove code is compiled, bundled, or executed. The main app resolves from src-tauri/Cargo.lock; the three vendored lockfiles are standalone packaging snapshots, not three additional app runtime graphs. No installer, resolver, audit network service, or build was run.

## src-tauri/Cargo.lock

{"path": "src-tauri/Cargo.lock", "packages": 763, "dependency_edges": 2711, "duplicate_names": 79, "unresolved_or_ambiguous_edges": [], "nonregistry_packages": ["cpal@0.16.0", "librespot-audio@0.8.0", "librespot-connect@0.8.0", "librespot-core@0.8.0", "spotify@1.3.2"]}

### Multiple versions

{"async-broadcast": ["0.5.1", "0.7.2"], "async-io": ["1.13.0", "2.6.0"], "async-lock": ["2.8.0", "3.4.2"], "async-process": ["1.8.1", "2.5.0"], "base64": ["0.21.7", "0.22.1"], "bitflags": ["1.3.2", "2.13.0"], "core-foundation": ["0.9.4", "0.10.1"], "core-graphics": ["0.22.3", "0.25.0"], "core-graphics-types": ["0.1.3", "0.2.0"], "darling": ["0.20.11", "0.23.0"], "darling_core": ["0.20.11", "0.23.0"], "darling_macro": ["0.20.11", "0.23.0"], "event-listener": ["2.5.3", "3.1.0", "5.4.1"], "fastrand": ["1.9.0", "2.4.1"], "foldhash": ["0.1.5", "0.2.0"], "foreign-types": ["0.3.2", "0.5.0"], "foreign-types-shared": ["0.1.1", "0.3.1"], "futures-lite": ["1.13.0", "2.6.1"], "getrandom": ["0.2.17", "0.3.4", "0.4.2"], "hashbrown": ["0.12.3", "0.15.5", "0.16.1", "0.17.1"], "heck": ["0.4.1", "0.5.0"], "hermit-abi": ["0.3.9", "0.5.2"], "hyper-rustls": ["0.26.0", "0.27.9"], "indexmap": ["1.9.3", "2.14.0"], "jni": ["0.21.1", "0.22.4"], "jni-sys": ["0.3.1", "0.4.1"], "linux-raw-sys": ["0.3.8", "0.4.15", "0.12.1"], "memoffset": ["0.7.1", "0.9.1"], "openssl-probe": ["0.1.6", "0.2.1"], "png": ["0.17.16", "0.18.1"], "polling": ["2.8.0", "3.11.0"], "proc-macro-crate": ["1.3.1", "2.0.2", "3.5.0"], "quick-xml": ["0.38.4", "0.39.4"], "r-efi": ["5.3.0", "6.0.0"], "rand": ["0.8.6", "0.9.4"], "rand_chacha": ["0.3.1", "0.9.0"], "rand_core": ["0.6.4", "0.9.5"], "redox_syscall": ["0.5.18", "0.8.1"], "reqwest": ["0.12.28", "0.13.4"], "rustix": ["0.37.28", "0.38.44", "1.1.4"], "rustls": ["0.22.4", "0.23.40"], "rustls-native-certs": ["0.7.3", "0.8.4"], "rustls-webpki": ["0.102.8", "0.103.13"], "schemars": ["0.8.22", "0.9.0", "1.2.1"], "security-framework": ["2.11.1", "3.7.0"], "serde_spanned": ["0.6.9", "1.1.1"], "socket2": ["0.4.10", "0.6.4"], "syn": ["1.0.109", "2.0.117"], "thiserror": ["1.0.69", "2.0.18"], "thiserror-impl": ["1.0.69", "2.0.18"], "tokio-rustls": ["0.25.0", "0.26.4"], "toml": ["0.8.2", "0.9.12+spec-1.1.0", "1.1.2+spec-1.1.0"], "toml_datetime": ["0.6.3", "0.7.5+spec-1.1.0", "1.1.1+spec-1.1.0"], "toml_edit": ["0.19.15", "0.20.2", "0.25.12+spec-1.1.0"], "uuid": ["0.8.2", "1.23.3"], "webpki-roots": ["0.26.11", "1.0.7"], "windows": ["0.44.0", "0.54.0", "0.61.3"], "windows-core": ["0.54.0", "0.61.2", "0.62.2"], "windows-link": ["0.1.3", "0.2.1"], "windows-result": ["0.1.2", "0.3.4", "0.4.1"], "windows-strings": ["0.4.2", "0.5.1"], "windows-sys": ["0.45.0", "0.48.0", "0.52.0", "0.59.0", "0.60.2", "0.61.2"], "windows-targets": ["0.42.2", "0.48.5", "0.52.6", "0.53.5"], "windows_aarch64_gnullvm": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_aarch64_msvc": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_i686_gnu": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_i686_gnullvm": ["0.52.6", "0.53.1"], "windows_i686_msvc": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_x86_64_gnu": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_x86_64_gnullvm": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "windows_x86_64_msvc": ["0.42.2", "0.48.5", "0.52.6", "0.53.1"], "winnow": ["0.5.40", "0.7.15", "1.0.3"], "wit-bindgen": ["0.51.0", "0.57.1"], "zbus": ["3.15.2", "5.16.0"], "zbus_macros": ["3.15.2", "5.16.0"], "zbus_names": ["2.6.1", "4.3.2"], "zvariant": ["3.15.2", "5.12.0"], "zvariant_derive": ["3.15.2", "5.12.0"], "zvariant_utils": ["1.0.1", "3.4.0"]}

### Every resolved package and edge

| Package | Source/checksum | Dependencies |
| --- | --- | --- |
| adler2@2.0.1 | registry; 320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa |  |
| aes@0.8.4 | registry; b169f7a6d4742236a0a00c541b845991d0ac43e546831af1249753ab4c3aa3a0 | cfg-if, cipher, cpufeatures |
| aho-corasick@1.1.4 | registry; ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301 | memchr |
| alloc-no-stdlib@2.0.4 | registry; cc7bb162ec39d46ab1ca8c77bf72e890535becd1751bb45f64c597edb4c8c6b3 |  |
| alloc-stdlib@0.2.4 | registry; 0e76a019e91224d279006ff972f1e984179a6e9feb050adba6ce8274aef23195 | alloc-no-stdlib |
| allocator-api2@0.2.21 | registry; 683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923 |  |
| alsa@0.9.1 | registry; ed7572b7ba83a31e20d1b48970ee402d2e3e0537dcfe0a3ff4d6eb7508617d43 | alsa-sys, bitflags 2.13.0, cfg-if, libc |
| alsa-sys@0.3.1 | registry; db8fee663d06c4e303404ef5f40488a53e062f89ba8bfed81f42325aafad1527 | libc, pkg-config |
| android_system_properties@0.1.5 | registry; 819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311 | libc |
| anyhow@1.0.102 | registry; 7f202df86484c868dbad7eaa557ef785d5c66295e41b460ef922eca0723b842c |  |
| arbitrary@1.4.2 | registry; c3d036a3c4ab069c7b410a2ce876bd74808d2d0888a82667669f8e783a898bf1 | derive_arbitrary |
| arrayvec@0.7.6 | registry; 7c02d123df017efcdfbd739ef81735b36c5ba83ec3c59c80a9d7ecc718f92e50 |  |
| async-broadcast@0.5.1 | registry; 7c48ccdbf6ca6b121e0f586cbc0e73ae440e56c67c30fa0873b4e110d9c26d2b | event-listener 2.5.3, futures-core |
| async-broadcast@0.7.2 | registry; 435a87a52755b8f27fcf321ac4f04b2802e337c8c4872923137471ec39c37532 | event-listener 5.4.1, event-listener-strategy, futures-core, pin-project-lite |
| async-channel@2.5.0 | registry; 924ed96dd52d1b75e9c1a3e6275715fd320f5f9439fb5a4a11fa51f4221158d2 | concurrent-queue, event-listener-strategy, futures-core, pin-project-lite |
| async-compression@0.4.43 | registry; 3976abdc8fe7d1133d43d304afd42abdf5bc3e1319d263d223bde07b5efc4be8 | compression-codecs, compression-core, pin-project-lite, tokio |
| async-executor@1.14.0 | registry; c96bf972d85afc50bf5ab8fe2d54d1586b4e0b46c97c50a0c9e71e2f7bcd812a | async-task, concurrent-queue, fastrand 2.4.1, futures-lite 2.6.1, pin-project-lite, slab |
| async-fs@1.6.0 | registry; 279cf904654eeebfa37ac9bb1598880884924aab82e290aa65c9e77a0e142e06 | async-lock 2.8.0, autocfg, blocking, futures-lite 1.13.0 |
| async-io@1.13.0 | registry; 0fc5b45d93ef0529756f812ca52e44c221b35341892d3dcc34132ac02f3dd2af | async-lock 2.8.0, autocfg, cfg-if, concurrent-queue, futures-lite 1.13.0, log, parking, polling 2.8.0, rustix 0.37.28, slab, socket2 0.4.10, waker-fn |
| async-io@2.6.0 | registry; 456b8a8feb6f42d237746d4b3e9a178494627745c3c56c6ea55d92ba50d026fc | autocfg, cfg-if, concurrent-queue, futures-io, futures-lite 2.6.1, parking, polling 3.11.0, rustix 1.1.4, slab, windows-sys 0.61.2 |
| async-lock@2.8.0 | registry; 287272293e9d8c41773cec55e365490fe034813a2f172f502d6ddcf75b2f582b | event-listener 2.5.3 |
| async-lock@3.4.2 | registry; 290f7f2596bd5b78a9fec8088ccd89180d7f9f55b94b0576823bbbdc72ee8311 | event-listener 5.4.1, event-listener-strategy, pin-project-lite |
| async-process@1.8.1 | registry; ea6438ba0a08d81529c69b36700fa2f95837bfe3e776ab39cde9c14d9149da88 | async-io 1.13.0, async-lock 2.8.0, async-signal, blocking, cfg-if, event-listener 3.1.0, futures-lite 1.13.0, rustix 0.38.44, windows-sys 0.48.0 |
| async-process@2.5.0 | registry; fc50921ec0055cdd8a16de48773bfeec5c972598674347252c0399676be7da75 | async-channel, async-io 2.6.0, async-lock 3.4.2, async-signal, async-task, blocking, cfg-if, event-listener 5.4.1, futures-lite 2.6.1, rustix 1.1.4 |
| async-recursion@1.1.1 | registry; 3b43422f69d8ff38f95f1b2bb76517c91589a924d1559a0e935d7c8ce0274c11 | proc-macro2, quote, syn 2.0.117 |
| async-signal@0.2.14 | registry; 52b5aaafa020cf5053a01f2a60e8ff5dccf550f0f77ec54a4e47285ac2bab485 | async-io 2.6.0, async-lock 3.4.2, atomic-waker, cfg-if, futures-core, futures-io, rustix 1.1.4, signal-hook-registry, slab, windows-sys 0.61.2 |
| async-task@4.7.1 | registry; 8b75356056920673b02621b35afd0f7dda9306d03c79a30f5c56c44cf256e3de |  |
| async-trait@0.1.89 | registry; 9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb | proc-macro2, quote, syn 2.0.117 |
| atk@0.18.2 | registry; 241b621213072e993be4f6f3a9e4b45f65b7e6faad43001be957184b7bb1824b | atk-sys, glib, libc |
| atk-sys@0.18.2 | registry; c5e48b684b0ca77d2bbadeef17424c2ea3c897d44d566a1617e7e8f30614d086 | glib-sys, gobject-sys, libc, system-deps |
| atoi@2.0.0 | registry; f28d99ec8bfea296261ca1af174f24225171fea9664ba9003cbebee704810528 | num-traits |
| atomic-waker@1.1.2 | registry; 1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0 |  |
| autocfg@1.5.1 | registry; f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53 |  |
| base64@0.21.7 | registry; 9d297deb1925b89f2ccc13d7635fa0714f12c87adce1c75356b39ca9b7178567 |  |
| base64@0.22.1 | registry; 72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6 |  |
| base64ct@1.8.3 | registry; 2af50177e190e07a26ab74f8b1efbfe2ef87da2116221318cb1c2e82baf7de06 |  |
| bit-set@0.8.0 | registry; 08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3 | bit-vec |
| bit-vec@0.8.0 | registry; 5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7 |  |
| bitflags@1.3.2 | registry; bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a |  |
| bitflags@2.13.0 | registry; b4388bee8683e3d04af747c73422af53102d2bd24d9eadb6cbc100baef4b43f8 | serde_core |
| block@0.1.6 | registry; 0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a |  |
| block-buffer@0.10.4 | registry; 3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71 | generic-array |
| block-padding@0.3.3 | registry; a8894febbff9f758034a5b8e12d87918f56dfc64a8e1fe757d65e29041538d93 | generic-array |
| block2@0.6.2 | registry; cdeb9d870516001442e364c5220d3574d2da8dc765554b4a617230d33fa58ef5 | objc2 |
| blocking@1.6.2 | registry; e83f8d02be6967315521be875afa792a316e28d57b5a2d401897e2a7921b7f21 | async-channel, async-task, futures-io, futures-lite 2.6.1, piper |
| brotli@8.0.4 | registry; 5cc91aac060a7a1e25823bdccbfb6af1875b88f17c6daac97894eed8207166b3 | alloc-no-stdlib, alloc-stdlib, brotli-decompressor |
| brotli-decompressor@5.0.3 | registry; 3a32acac15fe1967bc3986b2a6347dffc965602354ea6f450ad07e8bfd253583 | alloc-no-stdlib, alloc-stdlib |
| bs58@0.5.1 | registry; bf88ba1141d185c399bee5288d850d63b8369520c1eafc32a0430b5b6c287bf4 | tinyvec |
| bumpalo@3.20.3 | registry; 72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649 |  |
| bytemuck@1.25.0 | registry; c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec |  |
| byteorder@1.5.0 | registry; 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |  |
| bytes@1.11.1 | registry; 1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33 | serde |
| cairo-rs@0.18.5 | registry; 8ca26ef0159422fb77631dc9d17b102f253b876fe1586b03b803e63a309b4ee2 | bitflags 2.13.0, cairo-sys-rs, glib, libc, once_cell, thiserror 1.0.69 |
| cairo-sys-rs@0.18.2 | registry; 685c9fa8e590b8b3d678873528d83411db17242a73fccaed827770ea0fedda51 | glib-sys, libc, system-deps |
| camino@1.2.2 | registry; e629a66d692cb9ff1a1c664e41771b3dcaf961985a9774c0eb0bd1b51cf60a48 | serde_core |
| cargo-platform@0.1.9 | registry; e35af189006b9c0f00a064685c727031e3ed2d8020f7ba284d78cc2671bd36ea | serde |
| cargo_metadata@0.19.2 | registry; dd5eb614ed4c27c5d706420e4320fbe3216ab31fa1c33cd8246ac36dae4479ba | camino, cargo-platform, semver, serde, serde_json, thiserror 2.0.18 |
| cargo_toml@0.22.3 | registry; 374b7c592d9c00c1f4972ea58390ac6b18cbb6ab79011f3bdc90a0b82ca06b77 | serde, toml 0.9.12+spec-1.1.0 |
| cbc@0.1.2 | registry; 26b52a9543ae338f279b96b0b9fed9c8093744685043739079ce85cd58f289a6 | cipher |
| cc@1.2.64 | registry; dad887fd958be91b5098c0248def011f4523ab786cd411be668777e55063501f | find-msvc-tools, shlex |
| cesu8@1.1.0 | registry; 6d43a04d8753f35258c91f8ec639f792891f748a1edbd759cf1dcea3382ad83c |  |
| cfb@0.7.3 | registry; d38f2da7a0a2c4ccf0065be06397cc26a81f4e528be095826eee9d4adbb8c60f | byteorder, fnv, uuid 1.23.3 |
| cfg-expr@0.15.8 | registry; d067ad48b8650848b989a59a86c6c36a995d02d2bf778d45c3c5d57bc2718f02 | smallvec, target-lexicon |
| cfg-if@1.0.4 | registry; 9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801 |  |
| cfg_aliases@0.2.1 | registry; 613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724 |  |
| chrono@0.4.45 | registry; 1aa79e62e7697b8e29b513a68abacf485adcd1fe8284a4316c5ae868e6633327 | iana-time-zone, js-sys, num-traits, serde, wasm-bindgen, windows-link 0.2.1 |
| cipher@0.4.4 | registry; 773f3b9af64447d2ce9850330c473515014aa235e6a783b02db81ff39e4a3dad | crypto-common, inout |
| cocoa@0.24.1 | registry; f425db7937052c684daec3bd6375c8abe2d146dca4b8b143d6db777c39138f3a | bitflags 1.3.2, block, cocoa-foundation, core-foundation 0.9.4, core-graphics 0.22.3, foreign-types 0.3.2, libc, objc |
| cocoa-foundation@0.1.2 | registry; 8c6234cbb2e4c785b456c0644748b1ac416dd045799740356f8363dfe00c93f7 | bitflags 1.3.2, block, core-foundation 0.9.4, core-graphics-types 0.1.3, libc, objc |
| combine@4.6.7 | registry; ba5a308b75df32fe02788e748662718f03fde005016435c444eea572398219fd | bytes, memchr |
| compression-codecs@0.4.38 | registry; ce2548391e9c1929c21bf6aa2680af86fe4c1b33e6cea9ac1cfeec0bd11218cf | brotli, compression-core, flate2, memchr |
| compression-core@0.4.32 | registry; cc14f565cf027a105f7a44ccf9e5b424348421a1d8952a8fc9d499d313107789 |  |
| concurrent-queue@2.5.0 | registry; 4ca0197aee26d1ae37445ee532fefce43251d24cc7c166799f4d46817f1d3973 | crossbeam-utils |
| const-oid@0.9.6 | registry; c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8 |  |
| cookie@0.18.1 | registry; 4ddef33a339a91ea89fb53151bd0a4689cfce27055c291dfa69945475d22c747 | percent-encoding, time, version_check |
| cookie_store@0.22.0 | registry; 3fc4bff745c9b4c7fb1e97b25d13153da2bc7796260141df62378998d070207f | cookie, document-features, idna, log, publicsuffix, serde, serde_derive, serde_json, time, url |
| core-foundation@0.9.4 | registry; 91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f | core-foundation-sys, libc |
| core-foundation@0.10.1 | registry; b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6 | core-foundation-sys, libc |
| core-foundation-sys@0.8.7 | registry; 773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b |  |
| core-graphics@0.22.3 | registry; 2581bbab3b8ffc6fcbd550bf46c355135d16e9ff2a6ea032ad6b9bf1d7efe4fb | bitflags 1.3.2, core-foundation 0.9.4, core-graphics-types 0.1.3, foreign-types 0.3.2, libc |
| core-graphics@0.25.0 | registry; 064badf302c3194842cf2c5d61f56cc88e54a759313879cdf03abdd27d0c3b97 | bitflags 2.13.0, core-foundation 0.10.1, core-graphics-types 0.2.0, foreign-types 0.5.0, libc |
| core-graphics-types@0.1.3 | registry; 45390e6114f68f718cc7a830514a96f903cccd70d02a8f6d9f643ac4ba45afaf | bitflags 1.3.2, core-foundation 0.9.4, libc |
| core-graphics-types@0.2.0 | registry; 3d44a101f213f6c4cdc1853d4b78aef6db6bdfa3468798cc1d9912f4735013eb | bitflags 2.13.0, core-foundation 0.10.1, libc |
| coreaudio-rs@0.13.0 | registry; 1aae284fbaf7d27aa0e292f7677dfbe26503b0d555026f702940805a630eac17 | bitflags 1.3.2, libc, objc2-audio-toolbox, objc2-core-audio, objc2-core-audio-types, objc2-core-foundation |
| cpal@0.16.0 | local/path root | alsa, coreaudio-rs, dasp_sample, jni 0.21.1, js-sys, libc, mach2, ndk, ndk-context, num-derive, num-traits, objc2-audio-toolbox, objc2-core-audio, objc2-core-audio-types, wasm-bindgen, wasm-bindgen-futures, web-sys, windows 0.54.0 |
| cpufeatures@0.2.17 | registry; 59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280 | libc |
| crc@3.4.0 | registry; 5eb8a2a1cd12ab0d987a5d5e825195d372001a4094a0376319d5a0ad71c1ba0d | crc-catalog |
| crc-catalog@2.5.0 | registry; 217698eaf96b4a3f0bc4f3662aaa55bdf913cd54d7204591faa790070c6d0853 |  |
| crc32fast@1.5.0 | registry; 9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511 | cfg-if |
| crossbeam-channel@0.5.15 | registry; 82b8f8f868b36967f9606790d1903570de9ceaf870a7bf9fbbd3016d636a2cb2 | crossbeam-utils |
| crossbeam-queue@0.3.12 | registry; 0f58bbc28f91df819d0aa2a2c00cd19754769c2fad90579b3592b1c9ba7a3115 | crossbeam-utils |
| crossbeam-utils@0.8.21 | registry; d0a5c400df2834b80a4c3327b3aad3a4c4cd4de0629063962b03235697506a28 |  |
| crypto-common@0.1.7 | registry; 78c8292055d1c1df0cce5d180393dc8cce0abec0a7102adb6c7b1eef6016d60a | generic-array, typenum |
| cssparser@0.36.0 | registry; dae61cf9c0abb83bd659dab65b7e4e38d8236824c85f0f804f173567bda257d2 | cssparser-macros, dtoa-short, itoa, phf, smallvec |
| cssparser-macros@0.6.1 | registry; 13b588ba4ac1a99f7f2964d24b3d896ddc6bf847ee3855dbd4366f058cfcd331 | quote, syn 2.0.117 |
| ctor@0.8.0 | registry; 352d39c2f7bef1d6ad73db6f5160efcaed66d94ef8c6c573a8410c00bf909a98 | ctor-proc-macro, dtor |
| ctor-proc-macro@0.0.7 | registry; 52560adf09603e58c9a7ee1fe1dcb95a16927b17c127f0ac02d6e768a0e25bc1 |  |
| ctr@0.9.2 | registry; 0369ee1ad671834580515889b80f2ea915f23b8be8d0daa4bbaf2ac5c7590835 | cipher |
| darling@0.20.11 | registry; fc7f46116c46ff9ab3eb1597a45688b6715c6e628b5c133e288e709a29bcb4ee | darling_core 0.20.11, darling_macro 0.20.11 |
| darling@0.23.0 | registry; 25ae13da2f202d56bd7f91c25fba009e7717a1e4a1cc98a76d844b65ae912e9d | darling_core 0.23.0, darling_macro 0.23.0 |
| darling_core@0.20.11 | registry; 0d00b9596d185e565c2207a0b01f8bd1a135483d02d9b7b0a54b11da8d53412e | fnv, ident_case, proc-macro2, quote, strsim, syn 2.0.117 |
| darling_core@0.23.0 | registry; 9865a50f7c335f53564bb694ef660825eb8610e0a53d3e11bf1b0d3df31e03b0 | ident_case, proc-macro2, quote, strsim, syn 2.0.117 |
| darling_macro@0.20.11 | registry; fc34b93ccb385b40dc71c6fceac4b2ad23662c7eeb248cf10d529b7e055b6ead | darling_core 0.20.11, quote, syn 2.0.117 |
| darling_macro@0.23.0 | registry; ac3984ec7bd6cfa798e62b4a642426a5be0e68f9401cfc2a01e3fa9ea2fcdb8d | darling_core 0.23.0, quote, syn 2.0.117 |
| dasp_sample@0.11.0 | registry; 0c87e182de0887fd5361989c677c4e8f5000cd9491d6d563161a8f3a5519fc7f |  |
| data-encoding@2.11.0 | registry; a4ae5f15dda3c708c0ade84bfee31ccab44a3da4f88015ed22f63732abe300c8 |  |
| dbus@0.9.11 | registry; b942602992bb7acfd1f51c49811c58a610ef9181b6e66f3e519d79b540a3bf73 | libc, libdbus-sys, windows-sys 0.61.2 |
| dbus-crossroads@0.5.3 | registry; 64bff0bd181fba667660276c6b7ebdc50cff37ce593e7adf9e734f89c8f444e8 | dbus |
| der@0.7.10 | registry; e7c1832837b905bbfb5101e07cc24c8deddf52f93225eee6ead5f4d63d53ddcb | const-oid, pem-rfc7468, zeroize |
| deranged@0.5.8 | registry; 7cd812cc2bc1d69d4764bd80df88b4317eaef9e773c75226407d9bc0876b211c | serde_core |
| derivative@2.2.0 | registry; fcc3dd5e9e9c0b295d6e1e4d811fb6f157d5ffd784b8d202fc62eac8035a770b | proc-macro2, quote, syn 1.0.109 |
| derive_arbitrary@1.4.2 | registry; 1e567bd82dcff979e4b03460c307b3cdc9e96fde3d73bed1496d2bc75d9dd62a | proc-macro2, quote, syn 2.0.117 |
| derive_builder@0.20.2 | registry; 507dfb09ea8b7fa618fcf76e953f4f5e192547945816d5358edffe39f6f94947 | derive_builder_macro |
| derive_builder_core@0.20.2 | registry; 2d5bcf7b024d6835cfb3d473887cd966994907effbe9227e8c8219824d06c4e8 | darling 0.20.11, proc-macro2, quote, syn 2.0.117 |
| derive_builder_macro@0.20.2 | registry; ab63b0e2bf4d5928aff72e83a7dace85d7bba5fe12dcc3c5a572d78caffd3f3c | derive_builder_core, syn 2.0.117 |
| derive_more@2.1.1 | registry; d751e9e49156b02b44f9c1815bcb94b984cdcc4396ecc32521c739452808b134 | derive_more-impl |
| derive_more-impl@2.1.1 | registry; 799a97264921d8623a957f6c3b9011f3b5492f557bbb7a5a19b7fa6d06ba8dcb | proc-macro2, quote, rustc_version, syn 2.0.117 |
| des@0.8.1 | registry; ffdd80ce8ce993de27e9f063a444a4d53ce8e8db4c1f00cc03af5ad5a9867a1e | cipher |
| digest@0.10.7 | registry; 9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292 | block-buffer, const-oid, crypto-common, subtle |
| dirs@6.0.0 | registry; c3e8aa94d75141228480295a7d0e7feb620b1a5ad9f12bc40be62411e38cce4e | dirs-sys |
| dirs-sys@0.5.0 | registry; e01a3366d27ee9890022452ee61b2b63a67e6f13f58900b651ff5665f0bb1fab | libc, option-ext, redox_users, windows-sys 0.61.2 |
| discord-rich-presence@1.1.0 | registry; 90c55d69cab17c19677ce3a5f8face993a9e6eaf847fecac3547f3a3ff4a2494 | log, serde, serde_derive, serde_json, serde_repr, thiserror 2.0.18, uuid 0.8.2 |
| dispatch@0.2.0 | registry; bd0c93bb4b0c6d9b77f4435b0ae98c24d17f1c45b2ff844c6151a07256ca923b |  |
| dispatch2@0.3.1 | registry; 1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38 | bitflags 2.13.0, block2, libc, objc2 |
| displaydoc@0.2.6 | registry; 1ac70aa55017e108007fbaf5aa0f54b021c98f92ff8af59d42eda9da96e3dd4f | proc-macro2, quote, syn 2.0.117 |
| dlopen2@0.8.2 | registry; 5e2c5bd4158e66d1e215c49b837e11d62f3267b30c92f1d171c4d3105e3dc4d4 | dlopen2_derive, libc, once_cell, winapi |
| dlopen2_derive@0.4.3 | registry; 0fbbb781877580993a8707ec48672673ec7b81eeba04cfd2310bd28c08e47c8f | proc-macro2, quote, syn 2.0.117 |
| document-features@0.2.11 | registry; 95249b50c6c185bee49034bcb378a49dc2b5dff0be90ff6616d31d64febab05d | litrs |
| dom_query@0.27.0 | registry; 521e380c0c8afb8d9a1e83a1822ee03556fc3e3e7dbc1fd30be14e37f9cb3f89 | bit-set, cssparser, foldhash 0.2.0, html5ever, precomputed-hash, selectors, tendril |
| dotenvy@0.15.7 | registry; 1aaf95b3e5c8f23aa320147307562d361db0ae0d51242340f558153b4eb2439b |  |
| dpi@0.1.2 | registry; d8b14ccef22fc6f5a8f4d7d768562a182c04ce9a3b3157b91390b52ddfdf1a76 | serde |
| dtoa@1.0.11 | registry; 4c3cf4824e2d5f025c7b531afcb2325364084a16806f6d47fbc1f5fbd9960590 |  |
| dtoa-short@0.3.5 | registry; cd1511a7b6a56299bd043a9c167a6d2bfb37bf84a6dfceaba651168adfb43c87 | dtoa |
| dtor@0.3.0 | registry; f1057d6c64987086ff8ed0fd3fbf377a6b7d205cc7715868cd401705f715cbe4 | dtor-proc-macro |
| dtor-proc-macro@0.0.6 | registry; f678cf4a922c215c63e0de95eb1ff08a958a81d47e485cf9da1e27bf6305cfa5 |  |
| dunce@1.0.5 | registry; 92773504d58c093f6de2459af4af33faa518c13451eb8f2b5698ed3d36e7c813 |  |
| dyn-clone@1.0.20 | registry; d0881ea181b1df73ff77ffaaf9c7544ecc11e82fba9b5f27b262a3c73a332555 |  |
| either@1.16.0 | registry; 91622ff5e7162018101f2fea40d6ebf4a78bbe5a49736a2020649edf9693679e | serde |
| embed-resource@3.0.9 | registry; c31a88c8d26de40ed18fe748c547845aa39de1db3afd958f8cb91579f3644bcb | cc, memchr, rustc_version, toml 1.1.2+spec-1.1.0, vswhom, winreg |
| embed_plist@1.2.2 | registry; 4ef6b89e5b37196644d8796de5268852ff179b44e96276cf4290264843743bb7 |  |
| encoding_rs@0.8.35 | registry; 75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3 | cfg-if |
| endi@1.1.1 | registry; 66b7e2430c6dff6a955451e2cfc438f09cea1965a9d6f87f7e3b90decc014099 |  |
| enumflags2@0.7.12 | registry; 1027f7680c853e056ebcec683615fb6fbbc07dbaa13b4d5d9442b146ded4ecef | enumflags2_derive, serde |
| enumflags2_derive@0.7.12 | registry; 67c78a4d8fdf9953a5c9d458f9efe940fd97a0cab0941c075a813ac594733827 | proc-macro2, quote, syn 2.0.117 |
| equivalent@1.0.2 | registry; 877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f |  |
| erased-serde@0.4.10 | registry; d2add8a07dd6a8d93ff627029c51de145e12686fbc36ecb298ac22e74cf02dec | serde, serde_core, typeid |
| errno@0.3.14 | registry; 39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb | libc, windows-sys 0.61.2 |
| etcetera@0.8.0 | registry; 136d1b5283a1ab77bd9257427ffd09d8667ced0570b6f938942bc7568ed5b943 | cfg-if, home, windows-sys 0.48.0 |
| event-listener@2.5.3 | registry; 0206175f82b8d6bf6652ff7d71a1e27fd2e4efde587fd368662814d6ec1d9ce0 |  |
| event-listener@3.1.0 | registry; d93877bcde0eb80ca09131a08d23f0a5c18a620b01db137dba666d18cd9b30c2 | concurrent-queue, parking, pin-project-lite |
| event-listener@5.4.1 | registry; e13b66accf52311f30a0db42147dadea9850cb48cd070028831ae5f5d4b856ab | concurrent-queue, parking, pin-project-lite |
| event-listener-strategy@0.5.4 | registry; 8be9f3dfaaffdae2972880079a491a1a8bb7cbed0b8dd7a347f668b4150a3b93 | event-listener 5.4.1, pin-project-lite |
| extended@0.1.0 | registry; af9673d8203fcb076b19dfd17e38b3d4ae9f44959416ea532ce72415a6020365 |  |
| fastrand@1.9.0 | registry; e51093e27b0797c359783294ca4f0a911c270184cb10f85783b118614a1501be | instant |
| fastrand@2.4.1 | registry; 9f1f227452a390804cdb637b74a86990f2a7d7ba4b7d5693aac9b4dd6defd8d6 |  |
| fdeflate@0.3.7 | registry; 1e6853b52649d4ac5c0bd02320cddc5ba956bdb407c4b75a2c6b75bf51500f8c | simd-adler32 |
| field-offset@0.3.6 | registry; 38e2275cc4e4fc009b0669731a1e5ab7ebf11f469eaede2bab9309a5b4d6057f | memoffset 0.9.1, rustc_version |
| filetime@0.2.29 | registry; 5c287a33c7f0a620c38e641e7f60827713987b3c0f26e8ddc9462cc69cf75759 | cfg-if, libc |
| find-msvc-tools@0.1.9 | registry; 5baebc0774151f905a1a2cc41989300b1e6fbb29aff0ceffa1064fdd3088d582 |  |
| flate2@1.1.9 | registry; 843fba2746e448b37e26a819579957415c8cef339bf08564fe8b7ddbd959573c | crc32fast, miniz_oxide |
| flume@0.11.1 | registry; da0e4dd2a88388a1f4ccc7c9ce104604dab68d9f408dc34cd45823d5a9069095 | futures-core, futures-sink, spin |
| fnv@1.0.7 | registry; 3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1 |  |
| foldhash@0.1.5 | registry; d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2 |  |
| foldhash@0.2.0 | registry; 77ce24cb58228fbb8aa041425bb1050850ac19177686ea6e0f41a70416f56fdb |  |
| foreign-types@0.3.2 | registry; f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1 | foreign-types-shared 0.1.1 |
| foreign-types@0.5.0 | registry; d737d9aa519fb7b749cbc3b962edcf310a8dd1f4b67c91c4f83975dbdd17d965 | foreign-types-macros, foreign-types-shared 0.3.1 |
| foreign-types-macros@0.2.3 | registry; 1a5c6c585bc94aaf2c7b51dd4c2ba22680844aba4c687be581871a6f518c5742 | proc-macro2, quote, syn 2.0.117 |
| foreign-types-shared@0.1.1 | registry; 00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b |  |
| foreign-types-shared@0.3.1 | registry; aa9a19cbb55df58761df49b23516a86d432839add4af60fc256da840f66ed35b |  |
| form_urlencoded@1.2.2 | registry; cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf | percent-encoding |
| futures-channel@0.3.32 | registry; 07bbe89c50d7a535e539b8c17bc0b49bdb77747034daa8087407d655f3f7cc1d | futures-core, futures-sink |
| futures-core@0.3.32 | registry; 7e3450815272ef58cec6d564423f6e755e25379b217b0bc688e295ba24df6b1d |  |
| futures-executor@0.3.32 | registry; baf29c38818342a3b26b5b923639e7b1f4a61fc5e76102d4b1981c6dc7a7579d | futures-core, futures-task, futures-util |
| futures-intrusive@0.5.0 | registry; 1d930c203dd0b6ff06e0201a4a2fe9149b43c684fd4420555b26d21b1a02956f | futures-core, lock_api, parking_lot |
| futures-io@0.3.32 | registry; cecba35d7ad927e23624b22ad55235f2239cfa44fd10428eecbeba6d6a717718 |  |
| futures-lite@1.13.0 | registry; 49a9d51ce47660b1e808d3c990b4709f2f415d928835a17dfd16991515c46bce | fastrand 1.9.0, futures-core, futures-io, memchr, parking, pin-project-lite, waker-fn |
| futures-lite@2.6.1 | registry; f78e10609fe0e0b3f4157ffab1876319b5b0db102a2c60dc4626306dc46b44ad | fastrand 2.4.1, futures-core, futures-io, parking, pin-project-lite |
| futures-macro@0.3.32 | registry; e835b70203e41293343137df5c0664546da5745f82ec9b84d40be8336958447b | proc-macro2, quote, syn 2.0.117 |
| futures-sink@0.3.32 | registry; c39754e157331b013978ec91992bde1ac089843443c49cbc7f46150b0fad0893 |  |
| futures-task@0.3.32 | registry; 037711b3d59c33004d3856fbdc83b99d4ff37a24768fa1be9ce3538a1cde4393 |  |
| futures-timer@3.0.4 | registry; af43fadb8a98512d547e37b4e92e0ced13e205c061b87b4623eff01d918d6968 |  |
| futures-util@0.3.32 | registry; 389ca41296e6190b48053de0321d02a77f32f8a5d2461dd38762c0593805c6d6 | futures-core, futures-io, futures-macro, futures-sink, futures-task, memchr, pin-project-lite, slab |
| gdk@0.18.2 | registry; d9f245958c627ac99d8e529166f9823fb3b838d1d41fd2b297af3075093c2691 | cairo-rs, gdk-pixbuf, gdk-sys, gio, glib, libc, pango |
| gdk-pixbuf@0.18.5 | registry; 50e1f5f1b0bfb830d6ccc8066d18db35c487b1b2b1e8589b5dfe9f07e8defaec | gdk-pixbuf-sys, gio, glib, libc, once_cell |
| gdk-pixbuf-sys@0.18.0 | registry; 3f9839ea644ed9c97a34d129ad56d38a25e6756f99f3a88e15cd39c20629caf7 | gio-sys, glib-sys, gobject-sys, libc, system-deps |
| gdk-sys@0.18.2 | registry; 5c2d13f38594ac1e66619e188c6d5a1adb98d11b2fcf7894fc416ad76aa2f3f7 | cairo-sys-rs, gdk-pixbuf-sys, gio-sys, glib-sys, gobject-sys, libc, pango-sys, pkg-config, system-deps |
| gdkwayland-sys@0.18.2 | registry; 140071d506d223f7572b9f09b5e155afbd77428cd5cc7af8f2694c41d98dfe69 | gdk-sys, glib-sys, gobject-sys, libc, pkg-config, system-deps |
| gdkx11@0.18.2 | registry; 3caa00e14351bebbc8183b3c36690327eb77c49abc2268dd4bd36b856db3fbfe | gdk, gdkx11-sys, gio, glib, libc, x11 |
| gdkx11-sys@0.18.2 | registry; 6e2e7445fe01ac26f11601db260dd8608fe172514eb63b3b5e261ea6b0f4428d | gdk-sys, glib-sys, libc, system-deps, x11 |
| generic-array@0.14.7 | registry; 85649ca51fd72272d7821adaf274ad91c288277713d9c18820d8499a7ff69e9a | typenum, version_check |
| getrandom@0.2.17 | registry; ff2abc00be7fca6ebc474524697ae276ad847ad0a6b3faa4bcb027e9a4614ad0 | cfg-if, js-sys, libc, wasi, wasm-bindgen |
| getrandom@0.3.4 | registry; 899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd | cfg-if, js-sys, libc, r-efi 5.3.0, wasip2, wasm-bindgen |
| getrandom@0.4.2 | registry; 0de51e6874e94e7bf76d726fc5d13ba782deca734ff60d5bb2fb2607c7406555 | cfg-if, libc, r-efi 6.0.0, wasip2, wasip3 |
| gio@0.18.4 | registry; d4fc8f532f87b79cbc51a79748f16a6828fb784be93145a322fa14d06d354c73 | futures-channel, futures-core, futures-io, futures-util, gio-sys, glib, libc, once_cell, pin-project-lite, smallvec, thiserror 1.0.69 |
| gio-sys@0.18.1 | registry; 37566df850baf5e4cb0dfb78af2e4b9898d817ed9263d1090a2df958c64737d2 | glib-sys, gobject-sys, libc, system-deps, winapi |
| glib@0.18.5 | registry; 233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5 | bitflags 2.13.0, futures-channel, futures-core, futures-executor, futures-task, futures-util, gio-sys, glib-macros, glib-sys, gobject-sys, libc, memchr, once_cell, smallvec, thiserror 1.0.69 |
| glib-macros@0.18.5 | registry; 0bb0228f477c0900c880fd78c8759b95c7636dbd7842707f49e132378aa2acdc | heck 0.4.1, proc-macro-crate 2.0.2, proc-macro-error, proc-macro2, quote, syn 2.0.117 |
| glib-sys@0.18.1 | registry; 063ce2eb6a8d0ea93d2bf8ba1957e78dbab6be1c2220dd3daca57d5a9d869898 | libc, system-deps |
| glob@0.3.3 | registry; 0cc23270f6e1808e30a928bdc84dea0b9b4136a8bc82338574f23baf47bbd280 |  |
| gobject-sys@0.18.0 | registry; 0850127b514d1c4a4654ead6dedadb18198999985908e6ffe4436f53c785ce44 | glib-sys, libc, system-deps |
| governor@0.10.4 | registry; 9efcab3c1958580ff1f25a2a41be1668f7603d849bb63af523b208a3cc1223b8 | cfg-if, futures-sink, futures-timer, futures-util, hashbrown 0.16.1, nonzero_ext, parking_lot, portable-atomic, smallvec, spinning_top, web-time |
| gtk@0.18.2 | registry; fd56fb197bfc42bd5d2751f4f017d44ff59fbb58140c6b49f9b3b2bdab08506a | atk, cairo-rs, field-offset, futures-channel, gdk, gdk-pixbuf, gio, glib, gtk-sys, gtk3-macros, libc, pango, pkg-config |
| gtk-sys@0.18.2 | registry; 8f29a1c21c59553eb7dd40e918be54dccd60c52b049b75119d5d96ce6b624414 | atk-sys, cairo-sys-rs, gdk-pixbuf-sys, gdk-sys, gio-sys, glib-sys, gobject-sys, libc, pango-sys, system-deps |
| gtk3-macros@0.18.2 | registry; 52ff3c5b21f14f0736fed6dcfc0bfb4225ebf5725f3c0209edeec181e4d73e9d | proc-macro-crate 1.3.1, proc-macro-error, proc-macro2, quote, syn 2.0.117 |
| h2@0.4.15 | registry; 6cb093c84e8bd9b188d4c4a8cb6579fc016968d14c99882163cd3ff402a4f155 | atomic-waker, bytes, fnv, futures-core, futures-sink, http, indexmap 2.14.0, slab, tokio, tokio-util, tracing |
| hashbrown@0.12.3 | registry; 8a9ee70c43aaf417c914396645a0fa852624801b24ebb7ae78fe8272889ac888 |  |
| hashbrown@0.15.5 | registry; 9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1 | allocator-api2, equivalent, foldhash 0.1.5 |
| hashbrown@0.16.1 | registry; 841d1cc9bed7f9236f321df977030373f4a4163ae1a7dbfe1a51a2c1a51d9100 | allocator-api2, equivalent, foldhash 0.2.0 |
| hashbrown@0.17.1 | registry; ed5909b6e89a2db4456e54cd5f673791d7eca6732202bbf2a9cc504fe2f9b84a |  |
| hashlink@0.10.0 | registry; 7382cf6263419f2d8df38c55d7da83da5c18aef87fc7a7fc1fb1e344edfe14c1 | hashbrown 0.15.5 |
| headers@0.4.1 | registry; b3314d5adb5d94bcdf56771f2e50dbbc80bb4bdf88967526706205ac9eff24eb | base64 0.22.1, bytes, headers-core, http, httpdate, mime, sha1 |
| headers-core@0.3.0 | registry; 54b4a22553d4242c49fddb9ba998a99962b5cc6f22cb5a3482bec22522403ce4 | http |
| heck@0.4.1 | registry; 95505c38b4572b2d910cecb0281560f54b440a19336cbbcb27bf6ce6adc6f5a8 |  |
| heck@0.5.0 | registry; 2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea |  |
| hermit-abi@0.3.9 | registry; d231dfb89cfffdbc30e7fc41579ed6066ad03abda9e567ccafae602b97ec5024 |  |
| hermit-abi@0.5.2 | registry; fc0fef456e4baa96da950455cd02c081ca953b141298e41db3fc7e36b1da849c |  |
| hex@0.4.3 | registry; 7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70 |  |
| hkdf@0.12.4 | registry; 7b5f8eb2ad728638ea2c7d47a21db23b7b58a72ed6a38256b8a1849f15fbbdf7 | hmac |
| hmac@0.12.1 | registry; 6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e | digest |
| home@0.5.12 | registry; cc627f471c528ff0c4a49e1d5e60450c8f6461dd6d10ba9dcd3a61d3dff7728d | windows-sys 0.61.2 |
| html5ever@0.38.0 | registry; 1054432bae2f14e0061e33d23402fbaa67a921d319d56adc6bcf887ddad1cbc2 | log, markup5ever |
| http@1.4.2 | registry; 6970f50e31d6fc17d3fa27329444bfa74e196cf62e95052a3f6fee181dba6425 | bytes, itoa |
| http-body@1.0.1 | registry; 1efedce1fb8e6913f23e0c92de8e62cd5b772a67e7b3946df930a62566c93184 | bytes, http |
| http-body-util@0.1.3 | registry; b021d93e26becf5dc7e1b75b1bed1fd93124b374ceb73f43d4d4eafec896a64a | bytes, futures-core, http, http-body, pin-project-lite |
| httparse@1.10.1 | registry; 6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87 |  |
| httpdate@1.0.3 | registry; df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9 |  |
| hyper@1.10.1 | registry; 55281c53a1894c864990125767da440a4e630446785086f52523b20033b74498 | atomic-waker, bytes, futures-channel, futures-core, h2, http, http-body, httparse, itoa, pin-project-lite, smallvec, tokio, want |
| hyper-proxy2@0.1.0 | registry; 9043b7b23fb0bc4a1c7014c27b50a4fc42cc76206f71d34fc0dfe5b28ddc3faf | bytes, futures-util, headers, http, hyper, hyper-rustls 0.26.0, hyper-util, pin-project-lite, tokio, tokio-rustls 0.25.0, tower-service, webpki, webpki-roots 0.26.11 |
| hyper-rustls@0.26.0 | registry; a0bea761b46ae2b24eb4aef630d8d1c398157b6fc29e6350ecf090a0b70c952c | futures-util, http, hyper, hyper-util, log, rustls 0.22.4, rustls-native-certs 0.7.3, rustls-pki-types, tokio, tokio-rustls 0.25.0, tower-service, webpki-roots 0.26.11 |
| hyper-rustls@0.27.9 | registry; 33ca68d021ef39cf6463ab54c1d0f5daf03377b70561305bb89a8f83aab66e0f | http, hyper, hyper-util, rustls 0.23.40, tokio, tokio-rustls 0.26.4, tower-service, webpki-roots 1.0.7 |
| hyper-util@0.1.20 | registry; 96547c2556ec9d12fb1578c4eaf448b04993e7fb79cbaad930a656880a6bdfa0 | base64 0.22.1, bytes, futures-channel, futures-util, http, http-body, hyper, ipnet, libc, percent-encoding, pin-project-lite, socket2 0.6.4, system-configuration, tokio, tower-service, tracing, windows-registry |
| iana-time-zone@0.1.65 | registry; e31bc9ad994ba00e440a8aa5c9ef0ec67d5cb5e5cb0cc7f8b744a35b389cc470 | android_system_properties, core-foundation-sys, iana-time-zone-haiku, js-sys, log, wasm-bindgen, windows-core 0.62.2 |
| iana-time-zone-haiku@0.1.2 | registry; f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f | cc |
| ico@0.5.0 | registry; 3e795dff5605e0f04bff85ca41b51a96b83e80b281e96231bcaaf1ac35103371 | byteorder, png 0.17.16 |
| icu_collections@2.2.0 | registry; 2984d1cd16c883d7935b9e07e44071dca8d917fd52ecc02c04d5fa0b5a3f191c | displaydoc, potential_utf, utf8_iter, yoke, zerofrom, zerovec |
| icu_locale_core@2.2.0 | registry; 92219b62b3e2b4d88ac5119f8904c10f8f61bf7e95b640d25ba3075e6cac2c29 | displaydoc, litemap, tinystr, writeable, zerovec |
| icu_normalizer@2.2.0 | registry; c56e5ee99d6e3d33bd91c5d85458b6005a22140021cc324cea84dd0e72cff3b4 | icu_collections, icu_normalizer_data, icu_properties, icu_provider, smallvec, zerovec |
| icu_normalizer_data@2.2.0 | registry; da3be0ae77ea334f4da67c12f149704f19f81d1adf7c51cf482943e84a2bad38 |  |
| icu_properties@2.2.0 | registry; bee3b67d0ea5c2cca5003417989af8996f8604e34fb9ddf96208a033901e70de | icu_collections, icu_locale_core, icu_properties_data, icu_provider, zerotrie, zerovec |
| icu_properties_data@2.2.0 | registry; 8e2bbb201e0c04f7b4b3e14382af113e17ba4f63e2c9d2ee626b720cbce54a14 |  |
| icu_provider@2.2.0 | registry; 139c4cf31c8b5f33d7e199446eff9c1e02decfc2f0eec2c8d71f65befa45b421 | displaydoc, icu_locale_core, writeable, yoke, zerofrom, zerotrie, zerovec |
| id-arena@2.3.0 | registry; 3d3067d79b975e8844ca9eb072e16b31c3c1c36928edf9c6789548c524d0d954 |  |
| ident_case@1.0.1 | registry; b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39 |  |
| idna@1.1.0 | registry; 3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de | idna_adapter, smallvec, utf8_iter |
| idna_adapter@1.2.2 | registry; cb68373c0d6620ef8105e855e7745e18b0d00d3bdb07fb532e434244cdb9a714 | icu_normalizer, icu_properties |
| indexmap@1.9.3 | registry; bd070e393353796e801d209ad339e89596eb4c8d430d18ede6a1cced8fafbd99 | autocfg, hashbrown 0.12.3, serde |
| indexmap@2.14.0 | registry; d466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9 | equivalent, hashbrown 0.17.1, serde, serde_core |
| infer@0.19.0 | registry; a588916bfdfd92e71cacef98a63d9b1f0d74d6599980d11894290e7ddefffcf7 | cfb |
| inout@0.1.4 | registry; 879f10e63c20629ecabbb64a8010319738c66a5cd0c29b02d63d272b03751d01 | block-padding, generic-array |
| instant@0.1.13 | registry; e0242819d153cba4b4b05a5a8f2a7e9bbf97b6055b2a002b395c96b5ff3c0222 | cfg-if |
| io-lifetimes@1.0.11 | registry; eae7b9aee968036d54dce06cebaefd919e4472e753296daccd6d344e3e2df0c2 | hermit-abi 0.3.9, libc, windows-sys 0.48.0 |
| ipnet@2.12.0 | registry; d98f6fed1fde3f8c21bc40a1abb88dd75e67924f9cffc3ef95607bad8017f8e2 |  |
| is-docker@0.2.0 | registry; 928bae27f42bc99b60d9ac7334e3a21d10ad8f1835a4e12ec3ec0464765ed1b3 | once_cell |
| is-wsl@0.4.0 | registry; 173609498df190136aa7dea1a91db051746d339e18476eed5ca40521f02d7aa5 | is-docker, once_cell |
| itoa@1.0.18 | registry; 8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682 |  |
| javascriptcore-rs@1.1.2 | registry; ca5671e9ffce8ffba57afc24070e906da7fc4b1ba66f2cabebf61bf2ea257fcc | bitflags 1.3.2, glib, javascriptcore-rs-sys |
| javascriptcore-rs-sys@1.1.1 | registry; af1be78d14ffa4b75b66df31840478fef72b51f8c2465d4ca7c194da9f7a5124 | glib-sys, gobject-sys, libc, system-deps |
| jni@0.21.1 | registry; 1a87aa2bb7d2af34197c04845522473242e1aa17c12f4935d5856491a7fb8c97 | cesu8, cfg-if, combine, jni-sys 0.3.1, log, thiserror 1.0.69, walkdir, windows-sys 0.45.0 |
| jni@0.22.4 | registry; 5efd9a482cf3a427f00d6b35f14332adc7902ce91efb778580e180ff90fa3498 | cfg-if, combine, jni-macros, jni-sys 0.4.1, log, simd_cesu8, thiserror 2.0.18, walkdir, windows-link 0.2.1 |
| jni-macros@0.22.4 | registry; a00109accc170f0bdb141fed3e393c565b6f5e072365c3bd58f5b062591560a3 | proc-macro2, quote, rustc_version, simd_cesu8, syn 2.0.117 |
| jni-sys@0.3.1 | registry; 41a652e1f9b6e0275df1f15b32661cf0d4b78d4d87ddec5e0c3c20f097433258 | jni-sys 0.4.1 |
| jni-sys@0.4.1 | registry; c6377a88cb3910bee9b0fa88d4f42e1d2da8e79915598f65fb0c7ee14c878af2 | jni-sys-macros |
| jni-sys-macros@0.4.1 | registry; 38c0b942f458fe50cdac086d2f946512305e5631e720728f2a61aabcd47a6264 | quote, syn 2.0.117 |
| js-sys@0.3.102 | registry; 03d04c30968dffe80775bd4d7fb676131cd04a1fb46d2686dbffbaec2d9dfd31 | cfg-if, futures-util, wasm-bindgen |
| json-patch@3.0.1 | registry; 863726d7afb6bc2590eeff7135d923545e5e964f004c2ccf8716c25e70a86f08 | jsonptr, serde, serde_json, thiserror 1.0.69 |
| jsonptr@0.6.3 | registry; 5dea2b27dd239b2556ed7a25ba842fe47fd602e7fc7433c2a8d6106d4d9edd70 | serde, serde_json |
| keyboard-types@0.7.0 | registry; b750dcadc39a09dbadd74e118f6dd6598df77fa01df0cfcdc52c28dece74528a | bitflags 2.13.0, serde, unicode-segmentation |
| keyring@2.3.3 | registry; 363387f0019d714aa60cc30ab4fe501a747f4c08fc58f069dd14be971bd495a0 | byteorder, lazy_static, linux-keyutils, secret-service, security-framework 2.11.1, windows-sys 0.52.0 |
| lazy_static@1.5.0 | registry; bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe | spin |
| leb128fmt@0.1.0 | registry; 09edd9e8b54e49e587e4f6295a7d29c3ea94d469cb40ab8ca70b288248a81db2 |  |
| libappindicator@0.9.0 | registry; 03589b9607c868cc7ae54c0b2a22c8dc03dd41692d48f2d7df73615c6a95dc0a | glib, gtk, gtk-sys, libappindicator-sys, log |
| libappindicator-sys@0.9.0 | registry; 6e9ec52138abedcc58dc17a7c6c0c00a2bdb4f3427c7f63fa97fd0d859155caf | gtk-sys, libloading, once_cell |
| libc@0.2.186 | registry; 68ab91017fe16c622486840e4c83c9a37afeff978bd239b5293d61ece587de66 |  |
| libdbus-sys@0.2.7 | registry; 328c4789d42200f1eeec05bd86c9c13c7f091d2ba9a6ea35acdf51f31bc0f043 | pkg-config |
| libloading@0.7.4 | registry; b67380fd3b2fbe7527a606e18729d21c6f3951633d0500574c4dc22d2d638b9f | cfg-if, winapi |
| libm@0.2.16 | registry; b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981 |  |
| libredox@0.1.17 | registry; f02ab6bace2054fb888a3c16f990117b579d14a3088e472d63c6011fa185c9d3 | bitflags 2.13.0, libc, plain, redox_syscall 0.8.1 |
| librespot-audio@0.8.0 | local/path root | aes, bytes, ctr, futures-util, http-body-util, hyper, hyper-util, librespot-core, log, tempfile, thiserror 2.0.18, tokio |
| librespot-connect@0.8.0 | local/path root | futures-util, librespot-core, librespot-playback, librespot-protocol, log, protobuf, rand 0.9.4, serde_json, thiserror 2.0.18, tokio, tokio-stream, uuid 1.23.3 |
| librespot-core@0.8.0 | local/path root | aes, base64 0.22.1, byteorder, bytes, data-encoding, flate2, form_urlencoded, futures-core, futures-util, governor, hmac, http, http-body-util, httparse, hyper, hyper-proxy2, hyper-rustls 0.27.9, hyper-util, librespot-oauth, librespot-protocol, log, nonzero_ext, num-bigint, num-derive, num-integer, num-traits, pbkdf2, pin-project-lite, priority-queue, protobuf, protobuf-json-mapping, quick-xml 0.38.4, rand 0.9.4, rand_distr, rsa, serde, serde_json, sha1, shannon, sysinfo, thiserror 2.0.18, time, tokio, tokio-stream, tokio-tungstenite, tokio-util, url, uuid 1.23.3, vergen-gitcl |
| librespot-metadata@0.8.0 | registry; 9a9c688aa2acd3ed2498e31a95d6f2be49c0f18128db8958450ffd628aa88532 | async-trait, bytes, librespot-core, librespot-protocol, log, protobuf, serde, serde_json, thiserror 2.0.18, uuid 1.23.3 |
| librespot-oauth@0.8.0 | registry; d686417d49c9d2c363392ffe28d6e469daca20a82dc414740930e078f5829661 | log, oauth2, open, reqwest 0.12.28, thiserror 2.0.18, url |
| librespot-playback@0.8.0 | registry; 88258620bf3e6808ea1fadd11639648d77c06280b9f5a4c9d14ea79f6f998af6 | cpal, form_urlencoded, futures-util, librespot-audio, librespot-core, librespot-metadata, log, portable-atomic, rand 0.9.4, rand_distr, rodio, shell-words, symphonia, thiserror 2.0.18, tokio, zerocopy |
| librespot-protocol@0.8.0 | registry; 3e01f0b2d39f83fa162eb91d4a16313bcf99e77daf258abe8f7b7bcb1160b084 | protobuf, protobuf-codegen |
| libsqlite3-sys@0.30.1 | registry; 2e99fb7a497b1e3339bc746195567ed8d3e24945ecd636e3619d20b9de9e9149 | cc, pkg-config, vcpkg |
| linux-keyutils@0.2.5 | registry; 83270a18e9f90d0707c41e9f35efada77b64c0e6f3f1810e71c8368a864d5590 | bitflags 2.13.0, libc |
| linux-raw-sys@0.3.8 | registry; ef53942eb7bf7ff43a617b3e2c1c4a5ecf5944a7c1bc12d7ee39bbb15e5c1519 |  |
| linux-raw-sys@0.4.15 | registry; d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab |  |
| linux-raw-sys@0.12.1 | registry; 32a66949e030da00e8c7d4434b251670a91556f4144941d37452769c25d58a53 |  |
| litemap@0.8.2 | registry; 92daf443525c4cce67b150400bc2316076100ce0b3686209eb8cf3c31612e6f0 |  |
| litrs@0.4.2 | registry; f5e54036fe321fd421e10d732f155734c4e4afd610dd556d9a82833ab3ee0bed |  |
| lock_api@0.4.14 | registry; 224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965 | scopeguard |
| log@0.4.32 | registry; 953f07c43838f8e6f9758cab68bf5bed85465e7587ebe0b823f1bcd81978ad3a |  |
| lru-slab@0.1.2 | registry; 112b39cec0b298b6c1999fee3e31427f74f676e4cb9879ed1a121b43661a4154 |  |
| mac-notification-sys@0.6.15 | registry; fd604973958ddcc11b561193c0fb96ba146506ef2f231ef2e7c35fd2cbc9beca | cc, log, objc2, objc2-foundation, time, uuid 1.23.3 |
| mach2@0.4.3 | registry; d640282b302c0bb0a2a8e0233ead9035e3bed871f0b7e81fe4a1ec829765db44 | libc |
| malloc_buf@0.0.6 | registry; 62bb907fe88d54d8d9ce32a3cceab4218ed2f6b7d35617cafe9adf84e43919cb | libc |
| markup5ever@0.38.0 | registry; 8983d30f2915feeaaab2d6babdd6bc7e9ed1a00b66b5e6d74df19aa9c0e91862 | log, tendril, web_atoms |
| md-5@0.10.6 | registry; d89e7ee0cfbedfc4da3340218492196241d89eefb6dab27de5df917a6d2e78cf | cfg-if, digest |
| md5@0.7.0 | registry; 490cc448043f947bae3cbee9c203358d62dbee0db12107a74be5c30ccfd09771 |  |
| memchr@2.8.2 | registry; 88904434abc2901f197fe8cc55f0445e7ded921dba5911dad2e2b39b48e663c4 |  |
| memoffset@0.7.1 | registry; 5de893c32cde5f383baa4c04c5d6dbdd735cfd4a794b0debdb2bb1b421da5ff4 | autocfg |
| memoffset@0.9.1 | registry; 488016bfae457b036d996092f6cb448677611ce4449e970ceaf42695203f218a | autocfg |
| mime@0.3.17 | registry; 6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a |  |
| minisign-verify@0.2.5 | registry; 22f9645cb765ea72b8111f36c522475d2daa0d22c957a9826437e97534bc4e9e |  |
| miniz_oxide@0.8.9 | registry; 1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316 | adler2, simd-adler32 |
| mio@1.2.1 | registry; 02bd0af71c67b473010cbbc60715ee815645a4dc942899111f494b4b737d6fda | libc, wasi, windows-sys 0.61.2 |
| muda@0.19.2 | registry; 47a2e3dff89cd322c66647942668faee0a2b1f88ea6cbb4d374b4a8d7e92528c | crossbeam-channel, dpi, gtk, keyboard-types, objc2, objc2-app-kit, objc2-core-foundation, objc2-foundation, once_cell, png 0.18.1, serde, thiserror 2.0.18, windows-sys 0.61.2 |
| ndk@0.9.0 | registry; c3f42e7bbe13d351b6bead8286a43aac9534b82bd3cc43e47037f012ebfd62d4 | bitflags 2.13.0, jni-sys 0.3.1, log, ndk-sys, num_enum, raw-window-handle, thiserror 1.0.69 |
| ndk-context@0.1.1 | registry; 27b02d87554356db9e9a873add8782d4ea6e3e58ea071a9adb9a2e8ddb884a8b |  |
| ndk-sys@0.6.0+11769913 | registry; ee6cda3051665f1fb8d9e08fc35c96d5a244fb1be711a03b71118828afc9a873 | jni-sys 0.3.1 |
| new_debug_unreachable@1.0.6 | registry; 650eef8c711430f1a879fdd01d4745a7deea475becfb90269c06775983bbf086 |  |
| nix@0.26.4 | registry; 598beaf3cc6fdd9a5dfb1630c2800c7acd31df7aaf0f565796fba2b53ca1af1b | bitflags 1.3.2, cfg-if, libc, memoffset 0.7.1 |
| nonzero_ext@0.3.0 | registry; 38bf9645c8b145698bb0b18a4637dcacbc421ea49bef2317e4fd8065a387cf21 |  |
| notify-rust@4.18.0 | registry; c5b4c1b4f2aa9f25f63a7a49d3dd0ed567b3670da15330a66b29434be899b891 | futures-lite 2.6.1, log, mac-notification-sys, serde, tauri-winrt-notification, zbus 5.16.0 |
| ntapi@0.4.3 | registry; c3b335231dfd352ffb0f8017f3b6027a4917f7df785ea2143d8af2adc66980ae | winapi |
| num@0.4.3 | registry; 35bd024e8b2ff75562e5f34e7f4905839deb4b22955ef5e73d2fea1b9813cb23 | num-bigint, num-complex, num-integer, num-iter, num-rational, num-traits |
| num-bigint@0.4.6 | registry; a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9 | num-integer, num-traits |
| num-bigint-dig@0.8.6 | registry; e661dda6640fad38e827a6d4a310ff4763082116fe217f279885c97f511bb0b7 | lazy_static, libm, num-integer, num-iter, num-traits, rand 0.8.6, smallvec, zeroize |
| num-complex@0.4.6 | registry; 73f88a1307638156682bada9d7604135552957b7818057dcef22705b4d509495 | num-traits |
| num-conv@0.2.2 | registry; 521739c6d2bac4aa25192232afe6841231376b2b26d4d9fae5ecf8ca5772e441 |  |
| num-derive@0.4.2 | registry; ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202 | proc-macro2, quote, syn 2.0.117 |
| num-integer@0.1.46 | registry; 7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f | num-traits |
| num-iter@0.1.45 | registry; 1429034a0490724d0075ebb2bc9e875d6503c3cf69e235a8941aa757d83ef5bf | autocfg, num-integer, num-traits |
| num-rational@0.4.2 | registry; f83d14da390562dca69fc84082e73e548e1ad308d24accdedd2720017cb37824 | num-bigint, num-integer, num-traits |
| num-traits@0.2.19 | registry; 071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841 | autocfg, libm |
| num_enum@0.7.6 | registry; 5d0bca838442ec211fa11de3a8b0e0e8f3a4522575b5c4c06ed722e005036f26 | num_enum_derive, rustversion |
| num_enum_derive@0.7.6 | registry; 680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8 | proc-macro-crate 3.5.0, proc-macro2, quote, syn 2.0.117 |
| num_threads@0.1.7 | registry; 5c7398b9c8b70908f6371f47ed36737907c87c52af34c268fed0bf0ceb92ead9 | libc |
| oauth2@5.0.0 | registry; 51e219e79014df21a225b1860a479e2dcd7cbd9130f4defd4bd0e191ea31d67d | base64 0.22.1, chrono, getrandom 0.2.17, http, rand 0.8.6, reqwest 0.12.28, serde, serde_json, serde_path_to_error, sha2, thiserror 1.0.69, url |
| objc@0.2.7 | registry; 915b1b472bc21c53464d6c8461c9d3af805ba1ef837e1cac254428f4a77177b1 | malloc_buf |
| objc2@0.6.4 | registry; 3a12a8ed07aefc768292f076dc3ac8c48f3781c8f2d5851dd3d98950e8c5a89f | objc2-encode, objc2-exception-helper |
| objc2-app-kit@0.3.2 | registry; d49e936b501e5c5bf01fda3a9452ff86dc3ea98ad5f283e1455153142d97518c | bitflags 2.13.0, block2, objc2, objc2-core-foundation, objc2-foundation |
| objc2-audio-toolbox@0.3.2 | registry; 6948501a91121d6399b79abaa33a8aa4ea7857fe019f341b8c23ad6e81b79b08 | bitflags 2.13.0, libc, objc2, objc2-core-audio, objc2-core-audio-types, objc2-core-foundation, objc2-foundation |
| objc2-cloud-kit@0.3.2 | registry; 73ad74d880bb43877038da939b7427bba67e9dd42004a18b809ba7d87cee241c | bitflags 2.13.0, objc2, objc2-foundation |
| objc2-core-audio@0.3.2 | registry; e1eebcea8b0dbff5f7c8504f3107c68fc061a3eb44932051c8cf8a68d969c3b2 | dispatch2, objc2, objc2-core-audio-types, objc2-core-foundation |
| objc2-core-audio-types@0.3.2 | registry; 5a89f2ec274a0cf4a32642b2991e8b351a404d290da87bb6a9a9d8632490bd1c | bitflags 2.13.0, objc2 |
| objc2-core-data@0.3.2 | registry; 0b402a653efbb5e82ce4df10683b6b28027616a2715e90009947d50b8dd298fa | objc2, objc2-foundation |
| objc2-core-foundation@0.3.2 | registry; 2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536 | bitflags 2.13.0, dispatch2, objc2 |
| objc2-core-graphics@0.3.2 | registry; e022c9d066895efa1345f8e33e584b9f958da2fd4cd116792e15e07e4720a807 | bitflags 2.13.0, dispatch2, objc2, objc2-core-foundation, objc2-io-surface |
| objc2-core-image@0.3.2 | registry; e5d563b38d2b97209f8e861173de434bd0214cf020e3423a52624cd1d989f006 | objc2, objc2-foundation |
| objc2-core-location@0.3.2 | registry; ca347214e24bc973fc025fd0d36ebb179ff30536ed1f80252706db19ee452009 | objc2, objc2-foundation |
| objc2-core-text@0.3.2 | registry; 0cde0dfb48d25d2b4862161a4d5fcc0e3c24367869ad306b0c9ec0073bfed92d | bitflags 2.13.0, objc2, objc2-core-foundation, objc2-core-graphics |
| objc2-encode@4.1.0 | registry; ef25abbcd74fb2609453eb695bd2f860d389e457f67dc17cafc8b8cbc89d0c33 |  |
| objc2-exception-helper@0.1.1 | registry; c7a1c5fbb72d7735b076bb47b578523aedc40f3c439bea6dfd595c089d79d98a | cc |
| objc2-foundation@0.3.2 | registry; e3e0adef53c21f888deb4fa59fc59f7eb17404926ee8a6f59f5df0fd7f9f3272 | bitflags 2.13.0, block2, libc, objc2, objc2-core-foundation |
| objc2-io-kit@0.3.2 | registry; 33fafba39597d6dc1fb709123dfa8289d39406734be322956a69f0931c73bb15 | libc, objc2-core-foundation |
| objc2-io-surface@0.3.2 | registry; 180788110936d59bab6bd83b6060ffdfffb3b922ba1396b312ae795e1de9d81d | bitflags 2.13.0, objc2, objc2-core-foundation |
| objc2-osa-kit@0.3.2 | registry; f112d1746737b0da274ef79a23aac283376f335f4095a083a267a082f21db0c0 | bitflags 2.13.0, objc2, objc2-app-kit, objc2-foundation |
| objc2-quartz-core@0.3.2 | registry; 96c1358452b371bf9f104e21ec536d37a650eb10f7ee379fff67d2e08d537f1f | bitflags 2.13.0, objc2, objc2-core-foundation, objc2-foundation |
| objc2-ui-kit@0.3.2 | registry; d87d638e33c06f577498cbcc50491496a3ed4246998a7fbba7ccb98b1e7eab22 | bitflags 2.13.0, block2, objc2, objc2-cloud-kit, objc2-core-data, objc2-core-foundation, objc2-core-graphics, objc2-core-image, objc2-core-location, objc2-core-text, objc2-foundation, objc2-quartz-core, objc2-user-notifications |
| objc2-user-notifications@0.3.2 | registry; 9df9128cbbfef73cda168416ccf7f837b62737d748333bfe9ab71c245d76613e | objc2, objc2-foundation |
| objc2-web-kit@0.3.2 | registry; b2e5aaab980c433cf470df9d7af96a7b46a9d892d521a2cbbb2f8a4c16751e7f | bitflags 2.13.0, block2, objc2, objc2-app-kit, objc2-core-foundation, objc2-foundation |
| once_cell@1.21.4 | registry; 9f7c3e4beb33f85d45ae3e3a1792185706c8e16d043238c593331cc7cd313b50 |  |
| open@5.3.5 | registry; 2fbaa89d2ddc8473c78a3adf69eea8cffa28c483b8e02a971ef31527cd0fc92c | dunce, is-wsl, libc, pathdiff |
| openssl-probe@0.1.6 | registry; d05e27ee213611ffe7d6348b942e8f942b37114c00cc03cec254295a4a17852e |  |
| openssl-probe@0.2.1 | registry; 7c87def4c32ab89d880effc9e097653c8da5d6ef28e6b539d313baaacfbafcbe |  |
| option-ext@0.2.0 | registry; 04744f49eae99ab78e0d5c0b603ab218f515ea8cfe5a456d7629ad883a3b6e7d |  |
| ordered-stream@0.2.0 | registry; 9aa2b01e1d916879f73a53d01d1d6cee68adbb31d6d9177a8cfce093cced1d50 | futures-core, pin-project-lite |
| osakit@0.3.1 | registry; 732c71caeaa72c065bb69d7ea08717bd3f4863a4f451402fc9513e29dbd5261b | objc2, objc2-foundation, objc2-osa-kit, serde, serde_json, thiserror 2.0.18 |
| pango@0.18.3 | registry; 7ca27ec1eb0457ab26f3036ea52229edbdb74dee1edd29063f5b9b010e7ebee4 | gio, glib, libc, once_cell, pango-sys |
| pango-sys@0.18.0 | registry; 436737e391a843e5933d6d9aa102cb126d501e815b83601365a948a518555dc5 | glib-sys, gobject-sys, libc, system-deps |
| parking@2.2.1 | registry; f38d5652c16fde515bb1ecef450ab0f6a219d619a7274976324d5e377f7dceba |  |
| parking_lot@0.12.5 | registry; 93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a | lock_api, parking_lot_core |
| parking_lot_core@0.9.12 | registry; 2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1 | cfg-if, libc, redox_syscall 0.5.18, smallvec, windows-link 0.2.1 |
| pathdiff@0.2.3 | registry; df94ce210e5bc13cb6651479fa48d14f601d9858cfe0467f43ae157023b938d3 |  |
| pbkdf2@0.12.2 | registry; f8ed6a7761f76e3b9f92dfb0a60a6a6477c61024b775147ff0973a02653abaf2 | digest, hmac |
| pem-rfc7468@0.7.0 | registry; 88b39c9bfcfc231068454382784bb460aae594343fb030d46e9f50a645418412 | base64ct |
| percent-encoding@2.3.2 | registry; 9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220 |  |
| phf@0.13.1 | registry; c1562dc717473dbaa4c1f85a36410e03c047b2e7df7f45ee938fbef64ae7fadf | phf_macros, phf_shared, serde |
| phf_codegen@0.13.1 | registry; 49aa7f9d80421bca176ca8dbfebe668cc7a2684708594ec9f3c0db0805d5d6e1 | phf_generator, phf_shared |
| phf_generator@0.13.1 | registry; 135ace3a761e564ec88c03a77317a7c6b80bb7f7135ef2544dbe054243b89737 | fastrand 2.4.1, phf_shared |
| phf_macros@0.13.1 | registry; 812f032b54b1e759ccd5f8b6677695d5268c588701effba24601f6932f8269ef | phf_generator, phf_shared, proc-macro2, quote, syn 2.0.117 |
| phf_shared@0.13.1 | registry; e57fef6bc5981e38c2ce2d63bfa546861309f875b8a75f092d1d54ae2d64f266 | siphasher |
| pin-project-lite@0.2.17 | registry; a89322df9ebe1c1578d689c92318e070967d1042b512afbe49518723f4e6d5cd |  |
| piper@0.2.5 | registry; c835479a4443ded371d6c535cbfd8d31ad92c5d23ae9770a61bc155e4992a3c1 | atomic-waker, fastrand 2.4.1, futures-io |
| pkcs1@0.7.5 | registry; c8ffb9f10fa047879315e6625af03c164b16962a5368d724ed16323b68ace47f | der, pkcs8, spki |
| pkcs8@0.10.2 | registry; f950b2377845cebe5cf8b5165cb3cc1a5e0fa5cfa3e1f7f55707d8fd82e0a7b7 | der, spki |
| pkg-config@0.3.33 | registry; 19f132c84eca552bf34cab8ec81f1c1dcc229b811638f9d283dceabe58c5569e |  |
| plain@0.2.3 | registry; b4596b6d070b27117e987119b4dac604f3c58cfb0b191112e24771b2faeac1a6 |  |
| plist@1.9.0 | registry; 092791278e026273c1b65bbdcfbba3a300f2994c896bd01ab01da613c29c46f1 | base64 0.22.1, indexmap 2.14.0, quick-xml 0.39.4, serde, time |
| png@0.17.16 | registry; 82151a2fc869e011c153adc57cf2789ccb8d9906ce52c0b39a6b5697749d7526 | bitflags 1.3.2, crc32fast, fdeflate, flate2, miniz_oxide |
| png@0.18.1 | registry; 60769b8b31b2a9f263dae2776c37b1b28ae246943cf719eb6946a1db05128a61 | bitflags 2.13.0, crc32fast, fdeflate, flate2, miniz_oxide |
| polling@2.8.0 | registry; 4b2d323e8ca7996b3e23126511a523f7e62924d93ecd5ae73b333815b0eb3dce | autocfg, bitflags 1.3.2, cfg-if, concurrent-queue, libc, log, pin-project-lite, windows-sys 0.48.0 |
| polling@3.11.0 | registry; 5d0e4f59085d47d8241c88ead0f274e8a0cb551f3625263c05eb8dd897c34218 | cfg-if, concurrent-queue, hermit-abi 0.5.2, pin-project-lite, rustix 1.1.4, windows-sys 0.61.2 |
| portable-atomic@1.13.1 | registry; c33a9471896f1c69cecef8d20cbe2f7accd12527ce60845ff44c153bb2a21b49 |  |
| potential_utf@0.1.5 | registry; 0103b1cef7ec0cf76490e969665504990193874ea05c85ff9bab8b911d0a0564 | zerovec |
| powerfmt@0.2.0 | registry; 439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391 |  |
| ppv-lite86@0.2.21 | registry; 85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9 | zerocopy |
| precomputed-hash@0.1.1 | registry; 925383efa346730478fb4838dbe9137d2a47675ad789c546d150a6e1dd4ab31c |  |
| prettyplease@0.2.37 | registry; 479ca8adacdd7ce8f1fb39ce9ecccbfe93a3f1344b3d0d97f20bc0196208f62b | proc-macro2, syn 2.0.117 |
| priority-queue@2.7.0 | registry; 93980406f12d9f8140ed5abe7155acb10bb1e69ea55c88960b9c2f117445ef96 | equivalent, indexmap 2.14.0, serde |
| proc-macro-crate@1.3.1 | registry; 7f4c021e1093a56626774e81216a4ce732a735e5bad4868a03f3ed65ca0c3919 | once_cell, toml_edit 0.19.15 |
| proc-macro-crate@2.0.2 | registry; b00f26d3400549137f92511a46ac1cd8ce37cb5598a96d382381458b992a5d24 | toml_datetime 0.6.3, toml_edit 0.20.2 |
| proc-macro-crate@3.5.0 | registry; e67ba7e9b2b56446f1d419b1d807906278ffa1a658a8a5d8a39dcb1f5a78614f | toml_edit 0.25.12+spec-1.1.0 |
| proc-macro-error@1.0.4 | registry; da25490ff9892aab3fcf7c36f08cfb902dd3e71ca0f9f9517bea02a73a5ce38c | proc-macro-error-attr, proc-macro2, quote, syn 1.0.109, version_check |
| proc-macro-error-attr@1.0.4 | registry; a1be40180e52ecc98ad80b184934baf3d0d29f979574e439af5a55274b35f869 | proc-macro2, quote, version_check |
| proc-macro2@1.0.106 | registry; 8fd00f0bb2e90d81d1044c2b32617f68fcb9fa3bb7640c23e9c748e53fb30934 | unicode-ident |
| protobuf@3.7.2 | registry; d65a1d4ddae7d8b5de68153b48f6aa3bba8cb002b243dbdbc55a5afbc98f99f4 | once_cell, protobuf-support, thiserror 1.0.69 |
| protobuf-codegen@3.7.2 | registry; 5d3976825c0014bbd2f3b34f0001876604fe87e0c86cd8fa54251530f1544ace | anyhow, once_cell, protobuf, protobuf-parse, regex, tempfile, thiserror 1.0.69 |
| protobuf-json-mapping@3.7.2 | registry; e0d6e4be637b310d8a5c02fa195243328e2d97fa7df1127a27281ef1187fcb1d | protobuf, protobuf-support, thiserror 1.0.69 |
| protobuf-parse@3.7.2 | registry; b4aeaa1f2460f1d348eeaeed86aea999ce98c1bded6f089ff8514c9d9dbdc973 | anyhow, indexmap 2.14.0, log, protobuf, protobuf-support, tempfile, thiserror 1.0.69, which |
| protobuf-support@3.7.2 | registry; 3e36c2f31e0a47f9280fb347ef5e461ffcd2c52dd520d8e216b52f93b0b0d7d6 | thiserror 1.0.69 |
| psl-types@2.0.11 | registry; 33cb294fe86a74cbcf50d4445b37da762029549ebeea341421c7c70370f86cac |  |
| publicsuffix@2.3.0 | registry; 6f42ea446cab60335f76979ec15e12619a2165b5ae2c12166bef27d283a9fadf | idna, psl-types |
| quick-xml@0.38.4 | registry; b66c2058c55a409d601666cffe35f04333cf1013010882cec174a7467cd4e21c | memchr, serde |
| quick-xml@0.39.4 | registry; cdcc8dd4e2f670d309a5f0e83fe36dfdc05af317008fea29144da1a2ac858e5e | memchr |
| quinn@0.11.9 | registry; b9e20a958963c291dc322d98411f541009df2ced7b5a4f2bd52337638cfccf20 | bytes, cfg_aliases, pin-project-lite, quinn-proto, quinn-udp, rustc-hash, rustls 0.23.40, socket2 0.6.4, thiserror 2.0.18, tokio, tracing, web-time |
| quinn-proto@0.11.14 | registry; 434b42fec591c96ef50e21e886936e66d3cc3f737104fdb9b737c40ffb94c098 | bytes, getrandom 0.3.4, lru-slab, rand 0.9.4, ring, rustc-hash, rustls 0.23.40, rustls-pki-types, slab, thiserror 2.0.18, tinyvec, tracing, web-time |
| quinn-udp@0.5.14 | registry; addec6a0dcad8a8d96a771f815f0eaf55f9d1805756410b39f5fa81332574cbd | cfg_aliases, libc, once_cell, socket2 0.6.4, tracing, windows-sys 0.60.2 |
| quote@1.0.45 | registry; 41f2619966050689382d2b44f664f4bc593e129785a36d6ee376ddf37259b924 | proc-macro2 |
| r-efi@5.3.0 | registry; 69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f |  |
| r-efi@6.0.0 | registry; f8dcc9c7d52a811697d2151c701e0d08956f92b0e24136cf4cf27b57a6a0d9bf |  |
| rand@0.8.6 | registry; 5ca0ecfa931c29007047d1bc58e623ab12e5590e8c7cc53200d5202b69266d8a | libc, rand_chacha 0.3.1, rand_core 0.6.4 |
| rand@0.9.4 | registry; 44c5af06bb1b7d3216d91932aed5265164bf384dc89cd6ba05cf59a35f5f76ea | rand_chacha 0.9.0, rand_core 0.9.5 |
| rand_chacha@0.3.1 | registry; e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88 | ppv-lite86, rand_core 0.6.4 |
| rand_chacha@0.9.0 | registry; d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb | ppv-lite86, rand_core 0.9.5 |
| rand_core@0.6.4 | registry; ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c | getrandom 0.2.17 |
| rand_core@0.9.5 | registry; 76afc826de14238e6e8c374ddcc1fa19e374fd8dd986b0d2af0d02377261d83c | getrandom 0.3.4 |
| rand_distr@0.5.1 | registry; 6a8615d50dcf34fa31f7ab52692afec947c4dd0ab803cc87cb3b0b4570ff7463 | num-traits, rand 0.9.4 |
| raw-window-handle@0.6.2 | registry; 20675572f6f24e9e76ef639bc5552774ed45f1c30e2951e1e99c59888861c539 |  |
| redox_syscall@0.5.18 | registry; ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d | bitflags 2.13.0 |
| redox_syscall@0.8.1 | registry; 5b44b894f2a6e36457d665d1e08c3866add6ed5e70050c1b4ba8a8ddedb02ce7 | bitflags 2.13.0 |
| redox_users@0.5.2 | registry; a4e608c6638b9c18977b00b475ac1f28d14e84b27d8d42f70e0bf1e3dec127ac | getrandom 0.2.17, libredox, thiserror 2.0.18 |
| ref-cast@1.0.25 | registry; f354300ae66f76f1c85c5f84693f0ce81d747e2c3f21a45fef496d89c960bf7d | ref-cast-impl |
| ref-cast-impl@1.0.25 | registry; b7186006dcb21920990093f30e3dea63b7d6e977bf1256be20c3563a5db070da | proc-macro2, quote, syn 2.0.117 |
| regex@1.12.4 | registry; f1292b7759ae1cb9ec195452d1390a074f0cd8541ab7a5a8c31cd6db45d4a6ba | aho-corasick, memchr, regex-automata, regex-syntax |
| regex-automata@0.4.14 | registry; 6e1dd4122fc1595e8162618945476892eefca7b88c52820e74af6262213cae8f | aho-corasick, memchr, regex-syntax |
| regex-syntax@0.8.11 | registry; d6f6ff9a378485b298a5286656da665ba74413d36db0979633275d2e708145d4 |  |
| reqwest@0.12.28 | registry; eddd3ca559203180a307f12d114c268abf583f59b03cb906fd0b3ff8646c1147 | base64 0.22.1, bytes, cookie, cookie_store, futures-channel, futures-core, futures-util, h2, http, http-body, http-body-util, hyper, hyper-rustls 0.27.9, hyper-util, js-sys, log, percent-encoding, pin-project-lite, quinn, rustls 0.23.40, rustls-pki-types, serde, serde_json, serde_urlencoded, sync_wrapper, tokio, tokio-rustls 0.26.4, tower, tower-http, tower-service, url, wasm-bindgen, wasm-bindgen-futures, web-sys, webpki-roots 1.0.7 |
| reqwest@0.13.4 | registry; 219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3 | base64 0.22.1, bytes, futures-core, futures-util, http, http-body, http-body-util, hyper, hyper-rustls 0.27.9, hyper-util, js-sys, log, percent-encoding, pin-project-lite, rustls 0.23.40, rustls-pki-types, rustls-platform-verifier, serde, serde_json, sync_wrapper, tokio, tokio-rustls 0.26.4, tokio-util, tower, tower-http, tower-service, url, wasm-bindgen, wasm-bindgen-futures, wasm-streams, web-sys |
| ring@0.17.14 | registry; a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7 | cc, cfg-if, getrandom 0.2.17, libc, untrusted, windows-sys 0.52.0 |
| rodio@0.21.1 | registry; e40ecf59e742e03336be6a3d53755e789fd05a059fa22dfa0ed624722319e183 | cpal, dasp_sample, num-rational, symphonia |
| roxmltree@0.20.0 | registry; 6c20b6793b5c2fa6553b250154b78d6d0db37e72700ae35fad9387a46f487c97 |  |
| rsa@0.9.10 | registry; b8573f03f5883dcaebdfcf4725caa1ecb9c15b2ef50c43a07b816e06799bb12d | const-oid, digest, num-bigint-dig, num-integer, num-traits, pkcs1, pkcs8, rand_core 0.6.4, signature, spki, subtle, zeroize |
| rustc-hash@2.1.2 | registry; 94300abf3f1ae2e2b8ffb7b58043de3d399c73fa6f4b73826402a5c457614dbe |  |
| rustc_version@0.4.1 | registry; cfcb3a22ef46e85b45de6ee7e79d063319ebb6594faafcf1c225ea92ab6e9b92 | semver |
| rustix@0.37.28 | registry; 519165d378b97752ca44bbe15047d5d3409e875f39327546b42ac81d7e18c1b6 | bitflags 1.3.2, errno, io-lifetimes, libc, linux-raw-sys 0.3.8, windows-sys 0.48.0 |
| rustix@0.38.44 | registry; fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154 | bitflags 2.13.0, errno, libc, linux-raw-sys 0.4.15, windows-sys 0.59.0 |
| rustix@1.1.4 | registry; b6fe4565b9518b83ef4f91bb47ce29620ca828bd32cb7e408f0062e9930ba190 | bitflags 2.13.0, errno, libc, linux-raw-sys 0.12.1, windows-sys 0.61.2 |
| rustls@0.22.4 | registry; bf4ef73721ac7bcd79b2b315da7779d8fc09718c6b3d2d1b2d94850eb8c18432 | log, ring, rustls-pki-types, rustls-webpki 0.102.8, subtle, zeroize |
| rustls@0.23.40 | registry; ef86cd5876211988985292b91c96a8f2d298df24e75989a43a3c73f2d4d8168b | once_cell, ring, rustls-pki-types, rustls-webpki 0.103.13, subtle, zeroize |
| rustls-native-certs@0.7.3 | registry; e5bfb394eeed242e909609f56089eecfe5fda225042e8b171791b9c95f5931e5 | openssl-probe 0.1.6, rustls-pemfile, rustls-pki-types, schannel, security-framework 2.11.1 |
| rustls-native-certs@0.8.4 | registry; dab5152771c58876a2146916e53e35057e1a4dfa2b9df0f0305b07f611fdea4d | openssl-probe 0.2.1, rustls-pki-types, schannel, security-framework 3.7.0 |
| rustls-pemfile@2.2.0 | registry; dce314e5fee3f39953d46bb63bb8a46d40c2f8fb7cc5a3b6cab2bde9721d6e50 | rustls-pki-types |
| rustls-pki-types@1.14.1 | registry; 30a7197ae7eb376e574fe940d068c30fe0462554a3ddbe4eca7838e049c937a9 | web-time, zeroize |
| rustls-platform-verifier@0.7.0 | registry; 26d1e2536ce4f35f4846aa13bff16bd0ff40157cdb14cc056c7b14ba41233ba0 | core-foundation 0.10.1, core-foundation-sys, jni 0.22.4, log, once_cell, rustls 0.23.40, rustls-native-certs 0.8.4, rustls-platform-verifier-android, rustls-webpki 0.103.13, security-framework 3.7.0, security-framework-sys, webpki-root-certs, windows-sys 0.61.2 |
| rustls-platform-verifier-android@0.1.1 | registry; f87165f0995f63a9fbeea62b64d10b4d9d8e78ec6d7d51fb2125fda7bb36788f |  |
| rustls-webpki@0.102.8 | registry; 64ca1bc8749bd4cf37b5ce386cc146580777b4e8572c7b97baf22c83f444bee9 | ring, rustls-pki-types, untrusted |
| rustls-webpki@0.103.13 | registry; 61c429a8649f110dddef65e2a5ad240f747e85f7758a6bccc7e5777bd33f756e | ring, rustls-pki-types, untrusted |
| rustversion@1.0.22 | registry; b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d |  |
| ryu@1.0.23 | registry; 9774ba4a74de5f7b1c1451ed6cd5285a32eddb5cccb8cc655a4e50009e06477f |  |
| same-file@1.0.6 | registry; 93fc1dc3aaa9bfed95e02e6eadabb4baf7e3078b0bd1b4d7b6b0b68378900502 | winapi-util |
| schannel@0.1.29 | registry; 91c1b7e4904c873ef0710c1f407dde2e6287de2bebc1bbbf7d430bb7cbffd939 | windows-sys 0.61.2 |
| schemars@0.8.22 | registry; 3fbf2ae1b8bc8e02df939598064d22402220cd5bbcca1c76f7d6a310974d5615 | dyn-clone, indexmap 1.9.3, schemars_derive, serde, serde_json, url, uuid 1.23.3 |
| schemars@0.9.0 | registry; 4cd191f9397d57d581cddd31014772520aa448f65ef991055d7f61582c65165f | dyn-clone, ref-cast, serde, serde_json |
| schemars@1.2.1 | registry; a2b42f36aa1cd011945615b92222f6bf73c599a102a300334cd7f8dbeec726cc | dyn-clone, ref-cast, serde, serde_json |
| schemars_derive@0.8.22 | registry; 32e265784ad618884abaea0600a9adf15393368d840e0222d101a072f3f7534d | proc-macro2, quote, serde_derive_internals, syn 2.0.117 |
| scopeguard@1.2.0 | registry; 94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49 |  |
| secret-service@3.1.0 | registry; b5204d39df37f06d1944935232fd2dfe05008def7ca599bf28c0800366c8a8f9 | aes, cbc, futures-util, generic-array, hkdf, num, once_cell, rand 0.8.6, serde, sha2, zbus 3.15.2 |
| security-framework@2.11.1 | registry; 897b2245f0b511c87893af39b033e5ca9cce68824c4d7e7630b5a1d339658d02 | bitflags 2.13.0, core-foundation 0.9.4, core-foundation-sys, libc, security-framework-sys |
| security-framework@3.7.0 | registry; b7f4bc775c73d9a02cde8bf7b2ec4c9d12743edf609006c7facc23998404cd1d | bitflags 2.13.0, core-foundation 0.10.1, core-foundation-sys, libc, security-framework-sys |
| security-framework-sys@2.17.0 | registry; 6ce2691df843ecc5d231c0b14ece2acc3efb62c0a398c7e1d875f3983ce020e3 | core-foundation-sys, libc |
| selectors@0.36.1 | registry; c5d9c0c92a92d33f08817311cf3f2c29a3538a8240e94a6a3c622ce652d7e00c | bitflags 2.13.0, cssparser, derive_more, log, new_debug_unreachable, phf, phf_codegen, precomputed-hash, rustc-hash, servo_arc, smallvec |
| semver@1.0.28 | registry; 8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd | serde, serde_core |
| serde@1.0.228 | registry; 9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e | serde_core, serde_derive |
| serde-untagged@0.1.9 | registry; f9faf48a4a2d2693be24c6289dbe26552776eb7737074e6722891fadbe6c5058 | erased-serde, serde, serde_core, typeid |
| serde_core@1.0.228 | registry; 41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad | serde_derive |
| serde_derive@1.0.228 | registry; d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79 | proc-macro2, quote, syn 2.0.117 |
| serde_derive_internals@0.29.1 | registry; 18d26a20a969b9e3fdf2fc2d9f21eda6c40e2de84c9408bb5d3b05d499aae711 | proc-macro2, quote, syn 2.0.117 |
| serde_json@1.0.150 | registry; e8014e44b4736ed0538adeecded0fce2a272f22dc9578a7eb6b2d9993c74cfb9 | itoa, memchr, serde, serde_core, zmij |
| serde_path_to_error@0.1.20 | registry; 10a9ff822e371bb5403e391ecd83e182e0e77ba7f6fe0160b795797109d1b457 | itoa, serde, serde_core |
| serde_repr@0.1.20 | registry; 175ee3e80ae9982737ca543e96133087cbd9a485eecc3bc4de9c1a37b47ea59c | proc-macro2, quote, syn 2.0.117 |
| serde_spanned@0.6.9 | registry; bf41e0cfaf7226dca15e8197172c295a782857fcb97fad1808a166870dee75a3 | serde |
| serde_spanned@1.1.1 | registry; 6662b5879511e06e8999a8a235d848113e942c9124f211511b16466ee2995f26 | serde_core |
| serde_urlencoded@0.7.1 | registry; d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd | form_urlencoded, itoa, ryu, serde |
| serde_with@3.21.0 | registry; 76a5c54c7310e7b8b9577c286d7e399ddd876c3e12b3ed917a8aabc4b96e9e8c | base64 0.22.1, bs58, chrono, hex, indexmap 1.9.3, indexmap 2.14.0, schemars 0.9.0, schemars 1.2.1, serde_core, serde_json, serde_with_macros, time |
| serde_with_macros@3.21.0 | registry; 84d57bc0c8b9a17920c178daa6bb924850d54a9c97ab45194bb8c17ad66bb660 | darling 0.23.0, proc-macro2, quote, syn 2.0.117 |
| serialize-to-javascript@0.1.2 | registry; 04f3666a07a197cdb77cdf306c32be9b7f598d7060d50cfd4d5aa04bfd92f6c5 | serde, serde_json, serialize-to-javascript-impl |
| serialize-to-javascript-impl@0.1.2 | registry; 772ee033c0916d670af7860b6e1ef7d658a4629a6d0b4c8c3e67f09b3765b75d | proc-macro2, quote, syn 2.0.117 |
| servo_arc@0.4.3 | registry; 170fb83ab34de17dc69aa7c67482b22218ddb85da56546f9bd6b929e32a05930 | stable_deref_trait |
| sha1@0.10.6 | registry; e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba | cfg-if, cpufeatures, digest |
| sha2@0.10.9 | registry; a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283 | cfg-if, cpufeatures, digest |
| shannon@0.2.0 | registry; 7ea5b41c9427b56caa7b808cb548a04fb50bb5b9e98590b53f28064ff4174561 | byteorder |
| shell-words@1.1.1 | registry; dc6fe69c597f9c37bfeeeeeb33da3530379845f10be461a66d16d03eca2ded77 |  |
| shlex@2.0.1 | registry; f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba |  |
| signal-hook-registry@1.4.8 | registry; c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b | errno, libc |
| signature@2.2.0 | registry; 77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de | digest, rand_core 0.6.4 |
| simd-adler32@0.3.9 | registry; 703d5c7ef118737c72f1af64ad2f6f8c5e1921f818cdcb97b8fe6fc69bf66214 |  |
| simd_cesu8@1.1.1 | registry; 94f90157bb87cddf702797c5dadfa0be7d266cdf49e22da2fcaa32eff75b2c33 | rustc_version, simdutf8 |
| simdutf8@0.1.5 | registry; e3a9fe34e3e7a50316060351f37187a3f546bce95496156754b601a5fa71b76e |  |
| siphasher@1.0.3 | registry; 8ee5873ec9cce0195efcb7a4e9507a04cd49aec9c83d0389df45b1ef7ba2e649 |  |
| slab@0.4.12 | registry; 0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5 |  |
| smallvec@1.15.2 | registry; 8ed6a63f02c8539c91a8685a86f4099661ba3da017932f6ebbea6de3f0fa7c90 | serde |
| socket2@0.4.10 | registry; 9f7916fc008ca5542385b89a3d3ce689953c143e9304a9bf8beec1de48994c0d | libc, winapi |
| socket2@0.6.4 | registry; 52d1cfed4120b4d927bf7c0f86d2087a4a7d6027c906d9f9d525a80573b9be51 | libc, windows-sys 0.61.2 |
| softbuffer@0.4.8 | registry; aac18da81ebbf05109ab275b157c22a653bb3c12cf884450179942f81bcbf6c3 | bytemuck, js-sys, ndk, objc2, objc2-core-foundation, objc2-core-graphics, objc2-foundation, objc2-quartz-core, raw-window-handle, redox_syscall 0.5.18, tracing, wasm-bindgen, web-sys, windows-sys 0.61.2 |
| soup3@0.5.0 | registry; 471f924a40f31251afc77450e781cb26d55c0b650842efafc9c6cbd2f7cc4f9f | futures-channel, gio, glib, libc, soup3-sys |
| soup3-sys@0.5.0 | registry; 7ebe8950a680a12f24f15ebe1bf70db7af98ad242d9db43596ad3108aab86c27 | gio-sys, glib-sys, gobject-sys, libc, system-deps |
| souvlaki@0.7.3 | registry; ea4544ba17df4ac03d6503ae8abba19adad3ae89203a425945dc4c12d7790bfa | block, cocoa, core-graphics 0.22.3, dbus, dbus-crossroads, dispatch, objc, thiserror 1.0.69, windows 0.44.0 |
| spin@0.9.8 | registry; 6980e8d7511241f8acf4aebddbb1ff938df5eebe98691418c4468d0b72a96a67 | lock_api |
| spinning_top@0.3.0 | registry; d96d2d1d716fb500937168cc09353ffdc7a012be8475ac7308e1bdf0e3923300 | lock_api |
| spki@0.7.3 | registry; d91ed6c858b01f942cd56b37a94b3e0a1798290327d1236e4d9cf4eaca44d29d | base64ct, der |
| spotify@1.3.2 | local/path root | base64 0.22.1, bytes, cipher, cpal, des, discord-rich-presence, dotenvy, flate2, futures-util, keyring, libc, librespot-audio, librespot-connect, librespot-core, librespot-metadata, librespot-playback, librespot-protocol, log, md5, notify-rust, protobuf, rand 0.8.6, reqwest 0.12.28, rodio, roxmltree, serde, serde_json, sha1, sha2, souvlaki, sqlx, tauri, tauri-build, tauri-plugin-notification, tauri-plugin-opener, tauri-plugin-process, tauri-plugin-updater, tauri-winrt-notification, thiserror 1.0.69, tokio, url, window-vibrancy, windows-sys 0.60.2 |
| sqlx@0.8.6 | registry; 1fefb893899429669dcdd979aff487bd78f4064e5e7907e4269081e0ef7d97dc | sqlx-core, sqlx-macros, sqlx-mysql, sqlx-postgres, sqlx-sqlite |
| sqlx-core@0.8.6 | registry; ee6798b1838b6a0f69c007c133b8df5866302197e404e8b6ee8ed3e3a5e68dc6 | base64 0.22.1, bytes, crc, crossbeam-queue, either, event-listener 5.4.1, futures-core, futures-intrusive, futures-io, futures-util, hashbrown 0.15.5, hashlink, indexmap 2.14.0, log, memchr, once_cell, percent-encoding, rustls 0.23.40, serde, serde_json, sha2, smallvec, thiserror 2.0.18, tokio, tokio-stream, tracing, url, webpki-roots 0.26.11 |
| sqlx-macros@0.8.6 | registry; a2d452988ccaacfbf5e0bdbc348fb91d7c8af5bee192173ac3636b5fb6e6715d | proc-macro2, quote, sqlx-core, sqlx-macros-core, syn 2.0.117 |
| sqlx-macros-core@0.8.6 | registry; 19a9c1841124ac5a61741f96e1d9e2ec77424bf323962dd894bdb93f37d5219b | dotenvy, either, heck 0.5.0, hex, once_cell, proc-macro2, quote, serde, serde_json, sha2, sqlx-core, sqlx-mysql, sqlx-postgres, sqlx-sqlite, syn 2.0.117, tokio, url |
| sqlx-mysql@0.8.6 | registry; aa003f0038df784eb8fecbbac13affe3da23b45194bd57dba231c8f48199c526 | atoi, base64 0.22.1, bitflags 2.13.0, byteorder, bytes, crc, digest, dotenvy, either, futures-channel, futures-core, futures-io, futures-util, generic-array, hex, hkdf, hmac, itoa, log, md-5, memchr, once_cell, percent-encoding, rand 0.8.6, rsa, serde, sha1, sha2, smallvec, sqlx-core, stringprep, thiserror 2.0.18, tracing, whoami |
| sqlx-postgres@0.8.6 | registry; db58fcd5a53cf07c184b154801ff91347e4c30d17a3562a635ff028ad5deda46 | atoi, base64 0.22.1, bitflags 2.13.0, byteorder, crc, dotenvy, etcetera, futures-channel, futures-core, futures-util, hex, hkdf, hmac, home, itoa, log, md-5, memchr, once_cell, rand 0.8.6, serde, serde_json, sha2, smallvec, sqlx-core, stringprep, thiserror 2.0.18, tracing, whoami |
| sqlx-sqlite@0.8.6 | registry; c2d12fe70b2c1b4401038055f90f151b78208de1f9f89a7dbfd41587a10c3eea | atoi, flume, futures-channel, futures-core, futures-executor, futures-intrusive, futures-util, libsqlite3-sys, log, percent-encoding, serde, serde_urlencoded, sqlx-core, thiserror 2.0.18, tracing, url |
| stable_deref_trait@1.2.1 | registry; 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |  |
| static_assertions@1.1.0 | registry; a2eb9349b6444b326872e140eb1cf5e7c522154d69e7a0ffb0fb81c06b37543f |  |
| string_cache@0.9.0 | registry; a18596f8c785a729f2819c0f6a7eae6ebeebdfffbfe4214ae6b087f690e31901 | new_debug_unreachable, parking_lot, phf_shared, precomputed-hash |
| string_cache_codegen@0.6.1 | registry; 585635e46db231059f76c5849798146164652513eb9e8ab2685939dd90f29b69 | phf_generator, phf_shared, proc-macro2, quote |
| stringprep@0.1.5 | registry; 7b4df3d392d81bd458a8a621b8bffbd2302a12ffe288a9d931670948749463b1 | unicode-bidi, unicode-normalization, unicode-properties |
| strsim@0.11.1 | registry; 7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f |  |
| subtle@2.6.1 | registry; 13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292 |  |
| swift-rs@1.0.7 | registry; 4057c98e2e852d51fdcfca832aac7b571f6b351ad159f9eda5db1655f8d0c4d7 | base64 0.21.7, serde, serde_json |
| symphonia@0.5.5 | registry; 5773a4c030a19d9bfaa090f49746ff35c75dfddfa700df7a5939d5e076a57039 | lazy_static, symphonia-bundle-flac, symphonia-bundle-mp3, symphonia-codec-aac, symphonia-codec-pcm, symphonia-codec-vorbis, symphonia-core, symphonia-format-isomp4, symphonia-format-ogg, symphonia-format-riff, symphonia-metadata |
| symphonia-bundle-flac@0.5.5 | registry; c91565e180aea25d9b80a910c546802526ffd0072d0b8974e3ebe59b686c9976 | log, symphonia-core, symphonia-metadata, symphonia-utils-xiph |
| symphonia-bundle-mp3@0.5.5 | registry; 4872dd6bb56bf5eac799e3e957aa1981086c3e613b27e0ac23b176054f7c57ed | lazy_static, log, symphonia-core, symphonia-metadata |
| symphonia-codec-aac@0.5.5 | registry; 4c263845aa86881416849c1729a54c7f55164f8b96111dba59de46849e73a790 | lazy_static, log, symphonia-core |
| symphonia-codec-pcm@0.5.5 | registry; 4e89d716c01541ad3ebe7c91ce4c8d38a7cf266a3f7b2f090b108fb0cb031d95 | log, symphonia-core |
| symphonia-codec-vorbis@0.5.5 | registry; f025837c309cd69ffef572750b4a2257b59552c5399a5e49707cc5b1b85d1c73 | log, symphonia-core, symphonia-utils-xiph |
| symphonia-core@0.5.5 | registry; ea00cc4f79b7f6bb7ff87eddc065a1066f3a43fe1875979056672c9ef948c2af | arrayvec, bitflags 1.3.2, bytemuck, lazy_static, log |
| symphonia-format-isomp4@0.5.5 | registry; 243739585d11f81daf8dac8d9f3d18cc7898f6c09a259675fc364b382c30e0a5 | encoding_rs, log, symphonia-core, symphonia-metadata, symphonia-utils-xiph |
| symphonia-format-ogg@0.5.5 | registry; 2b4955c67c1ed3aa8ae8428d04ca8397fbef6a19b2b051e73b5da8b1435639cb | log, symphonia-core, symphonia-metadata, symphonia-utils-xiph |
| symphonia-format-riff@0.5.5 | registry; c2d7c3df0e7d94efb68401d81906eae73c02b40d5ec1a141962c592d0f11a96f | extended, log, symphonia-core, symphonia-metadata |
| symphonia-metadata@0.5.5 | registry; 36306ff42b9ffe6e5afc99d49e121e0bd62fe79b9db7b9681d48e29fa19e6b16 | encoding_rs, lazy_static, log, symphonia-core |
| symphonia-utils-xiph@0.5.5 | registry; ee27c85ab799a338446b68eec77abf42e1a6f1bb490656e121c6e27bfbab9f16 | symphonia-core, symphonia-metadata |
| syn@1.0.109 | registry; 72b64191b275b66ffe2469e8af2c1cfe3bafa67b529ead792a6d0160888b4237 | proc-macro2, quote, unicode-ident |
| syn@2.0.117 | registry; e665b8803e7b1d2a727f4023456bbbbe74da67099c585258af0ad9c5013b9b99 | proc-macro2, quote, unicode-ident |
| sync_wrapper@1.0.2 | registry; 0bf256ce5efdfa370213c1dabab5935a12e49f2c58d15e9eac2870d3b4f27263 | futures-core |
| synstructure@0.13.2 | registry; 728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2 | proc-macro2, quote, syn 2.0.117 |
| sysinfo@0.36.1 | registry; 252800745060e7b9ffb7b2badbd8b31cfa4aa2e61af879d0a3bf2a317c20217d | libc, memchr, ntapi, objc2-core-foundation, objc2-io-kit, windows 0.61.3 |
| system-configuration@0.7.0 | registry; a13f3d0daba03132c0aa9767f98351b3488edc2c100cda2d2ec2b04f3d8d3c8b | bitflags 2.13.0, core-foundation 0.9.4, system-configuration-sys |
| system-configuration-sys@0.6.0 | registry; 8e1d1b10ced5ca923a1fcb8d03e96b8d3268065d724548c0211415ff6ac6bac4 | core-foundation-sys, libc |
| system-deps@6.2.2 | registry; a3e535eb8dded36d55ec13eddacd30dec501792ff23a0b1682c38601b8cf2349 | cfg-expr, heck 0.5.0, pkg-config, toml 0.8.2, version-compare |
| tao@0.35.3 | registry; d1c93047acf68669466a34690ac58cca7010bd1b201e1ec86f1fd0a75d3dd4a9 | bitflags 2.13.0, block2, core-foundation 0.10.1, core-graphics 0.25.0, crossbeam-channel, dbus, dispatch2, dlopen2, dpi, gdkwayland-sys, gdkx11-sys, gtk, jni 0.21.1, libc, log, ndk, ndk-sys, objc2, objc2-app-kit, objc2-foundation, objc2-ui-kit, once_cell, parking_lot, percent-encoding, raw-window-handle, tao-macros, unicode-segmentation, url, windows 0.61.3, windows-core 0.61.2, windows-version, x11-dl |
| tao-macros@0.1.3 | registry; f4e16beb8b2ac17db28eab8bca40e62dbfbb34c0fcdc6d9826b11b7b5d047dfd | proc-macro2, quote, syn 2.0.117 |
| tar@0.4.46 | registry; 3f6221d9a6003c78398e3b239969f352578258df48c8eb051caadae0015bc840 | filetime, libc, xattr |
| target-lexicon@0.12.16 | registry; 61c41af27dd6d1e27b1b16b489db798443478cef1f06a660c96db617ba5de3b1 |  |
| tauri@2.11.2 | registry; 437404997acf375d85f1177afa7e11bb971f274ed6a7b83a2a3e339015f4cc28 | anyhow, bytes, cookie, dirs, dunce, embed_plist, getrandom 0.3.4, glob, gtk, heck 0.5.0, http, jni 0.21.1, libc, log, mime, muda, objc2, objc2-app-kit, objc2-foundation, objc2-ui-kit, objc2-web-kit, percent-encoding, plist, raw-window-handle, reqwest 0.13.4, serde, serde_json, serde_repr, serialize-to-javascript, swift-rs, tauri-build, tauri-macros, tauri-runtime, tauri-runtime-wry, tauri-utils, thiserror 2.0.18, tokio, tray-icon, url, webkit2gtk, webview2-com, window-vibrancy, windows 0.61.3 |
| tauri-build@2.6.2 | registry; 4aa1f9055fc23919a54e4e125052bed16ed04aef0487086e758fe01a67b451c7 | anyhow, cargo_toml, dirs, glob, heck 0.5.0, json-patch, schemars 0.8.22, semver, serde, serde_json, tauri-utils, tauri-winres, walkdir |
| tauri-codegen@2.6.2 | registry; e4a0319528a025a38c4078e7dae2c446f4e63620ddb0659a643ede1cb38f90e9 | base64 0.22.1, brotli, ico, json-patch, plist, png 0.17.16, proc-macro2, quote, semver, serde, serde_json, sha2, syn 2.0.117, tauri-utils, thiserror 2.0.18, time, url, uuid 1.23.3, walkdir |
| tauri-macros@2.6.2 | registry; ae6cb4e3896c21d2f6da5b31251d2faea0153bba56ed0e970f918115dbee4924 | heck 0.5.0, proc-macro2, quote, syn 2.0.117, tauri-codegen, tauri-utils |
| tauri-plugin@2.6.2 | registry; e126abc9e84e35cdfd01596140a73a1850cdb0df0a23acf0185776c30b469a6e | anyhow, glob, plist, schemars 0.8.22, serde, serde_json, tauri-utils, walkdir |
| tauri-plugin-notification@2.4.0 | registry; ad2fd40946aef810c4be9fd33a2d1b9b397cb79042b2d21c81a0a8f204354fd1 | log, notify-rust, rand 0.9.4, serde, serde_json, serde_repr, tauri, tauri-plugin, thiserror 2.0.18, time, url |
| tauri-plugin-opener@2.5.4 | registry; 17e1bea14edce6b793a04e2417e3fd924b9bc4faae83cdee7d714156cceeed29 | dunce, glob, objc2-app-kit, objc2-foundation, open, schemars 0.8.22, serde, serde_json, tauri, tauri-plugin, thiserror 2.0.18, url, windows 0.61.3, zbus 5.16.0 |
| tauri-plugin-process@2.3.1 | registry; d55511a7bf6cd70c8767b02c97bf8134fa434daf3926cfc1be0a0f94132d165a | tauri, tauri-plugin |
| tauri-plugin-updater@2.10.1 | registry; 806d9dac662c2e4594ff03c647a552f2c9bd544e7d0f683ec58f872f952ce4af | base64 0.22.1, dirs, flate2, futures-util, http, infer, log, minisign-verify, osakit, percent-encoding, reqwest 0.13.4, rustls 0.23.40, semver, serde, serde_json, tar, tauri, tauri-plugin, tempfile, thiserror 2.0.18, time, tokio, url, windows-sys 0.60.2, zip |
| tauri-runtime@2.11.2 | registry; 48222d7116c8807eaa6fe2f372e023fae125084e61e6eca6d70b7961cdf129ef | cookie, dpi, gtk, http, jni 0.21.1, objc2, objc2-ui-kit, objc2-web-kit, raw-window-handle, serde, serde_json, tauri-utils, thiserror 2.0.18, url, webkit2gtk, webview2-com, windows 0.61.3 |
| tauri-runtime-wry@2.11.2 | registry; b83849ee63ecb27a8e8d0fe51915ca215076914aca43f96db1179f0f415f6cd9 | gtk, http, jni 0.21.1, log, objc2, objc2-app-kit, once_cell, percent-encoding, raw-window-handle, softbuffer, tao, tauri-runtime, tauri-utils, url, webkit2gtk, webview2-com, windows 0.61.3, wry |
| tauri-utils@2.9.2 | registry; 092379df9a707631978e6c56b1bc2401d387f01e2d4a3c123360d167bbb9aa95 | anyhow, brotli, cargo_metadata, ctor, dom_query, dunce, glob, http, infer, json-patch, log, memchr, phf, plist, proc-macro2, quote, regex, schemars 0.8.22, semver, serde, serde-untagged, serde_json, serde_with, swift-rs, thiserror 2.0.18, toml 1.1.2+spec-1.1.0, url, urlpattern, uuid 1.23.3, walkdir |
| tauri-winres@0.3.6 | registry; cc65d45c68858bfe420dd29e834b5d15dbecf8a07a8a16cf4d532c7b1f69d4b6 | dunce, embed-resource, toml 1.1.2+spec-1.1.0 |
| tauri-winrt-notification@0.7.3 | registry; 9ed071c670382e85fc2f48ae706492d8c338f4f89bf72520d32f8abfe880aade | thiserror 2.0.18, windows 0.61.3, windows-version |
| tempfile@3.27.0 | registry; 32497e9a4c7b38532efcdebeef879707aa9f794296a4f0244f6f69e9bc8574bd | fastrand 2.4.1, getrandom 0.4.2, once_cell, rustix 1.1.4, windows-sys 0.61.2 |
| tendril@0.5.0 | registry; c4790fc369d5a530f4b544b094e31388b9b3a37c0f4652ade4505945f5660d24 | new_debug_unreachable, utf-8 |
| thiserror@1.0.69 | registry; b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52 | thiserror-impl 1.0.69 |
| thiserror@2.0.18 | registry; 4288b5bcbc7920c07a1149a35cf9590a2aa808e0bc1eafaade0b80947865fbc4 | thiserror-impl 2.0.18 |
| thiserror-impl@1.0.69 | registry; 4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1 | proc-macro2, quote, syn 2.0.117 |
| thiserror-impl@2.0.18 | registry; ebc4ee7f67670e9b64d05fa4253e753e016c6c95ff35b89b7941d6b856dec1d5 | proc-macro2, quote, syn 2.0.117 |
| time@0.3.49 | registry; 711a53c2d47bbd818258c498c8dbfe186a2526c631495cfe7e078567f86b8469 | deranged, libc, num-conv, num_threads, powerfmt, serde_core, time-core, time-macros |
| time-core@0.1.9 | registry; 9e1c906769ad99c88eaa54e728060edef082f8e358ff32030cb7c7d315e81109 |  |
| time-macros@0.2.29 | registry; 71c652a3727a9cbb9a02f707f530b618ce00d0ccd762009c8c23bd191df3c17d | num-conv, time-core |
| tinystr@0.8.3 | registry; c8323304221c2a851516f22236c5722a72eaa19749016521d6dff0824447d96d | displaydoc, zerovec |
| tinyvec@1.11.0 | registry; 3e61e67053d25a4e82c844e8424039d9745781b3fc4f32b8d55ed50f5f667ef3 | tinyvec_macros |
| tinyvec_macros@0.1.1 | registry; 1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20 |  |
| tokio@1.52.3 | registry; 8fc7f01b389ac15039e4dc9531aa973a135d7a4135281b12d7c1bc79fd57fffe | bytes, libc, mio, parking_lot, pin-project-lite, signal-hook-registry, socket2 0.6.4, tokio-macros, windows-sys 0.61.2 |
| tokio-macros@2.7.0 | registry; 385a6cb71ab9ab790c5fe8d67f1645e6c450a7ce006a33de03daa956cf70a496 | proc-macro2, quote, syn 2.0.117 |
| tokio-rustls@0.25.0 | registry; 775e0c0f0adb3a2f22a00c4745d728b479985fc15ee7ca6a2608388c5569860f | rustls 0.22.4, rustls-pki-types, tokio |
| tokio-rustls@0.26.4 | registry; 1729aa945f29d91ba541258c8df89027d5792d85a8841fb65e8bf0f4ede4ef61 | rustls 0.23.40, tokio |
| tokio-stream@0.1.18 | registry; 32da49809aab5c3bc678af03902d4ccddea2a87d028d86392a4b1560c6906c70 | futures-core, pin-project-lite, tokio |
| tokio-tungstenite@0.28.0 | registry; d25a406cddcc431a75d3d9afc6a7c0f7428d4891dd973e4d54c56b46127bf857 | futures-util, log, rustls 0.23.40, rustls-pki-types, tokio, tokio-rustls 0.26.4, tungstenite, webpki-roots 0.26.11 |
| tokio-util@0.7.18 | registry; 9ae9cec805b01e8fc3fd2fe289f89149a9b66dd16786abd8b19cfa7b48cb0098 | bytes, futures-core, futures-sink, pin-project-lite, tokio |
| toml@0.8.2 | registry; 185d8ab0dfbb35cf1399a6344d8484209c088f75f8f68230da55d48d95d43e3d | serde, serde_spanned 0.6.9, toml_datetime 0.6.3, toml_edit 0.20.2 |
| toml@0.9.12+spec-1.1.0 | registry; cf92845e79fc2e2def6a5d828f0801e29a2f8acc037becc5ab08595c7d5e9863 | indexmap 2.14.0, serde_core, serde_spanned 1.1.1, toml_datetime 0.7.5+spec-1.1.0, toml_parser, toml_writer, winnow 0.7.15 |
| toml@1.1.2+spec-1.1.0 | registry; 81f3d15e84cbcd896376e6730314d59fb5a87f31e4b038454184435cd57defee | indexmap 2.14.0, serde_core, serde_spanned 1.1.1, toml_datetime 1.1.1+spec-1.1.0, toml_parser, toml_writer, winnow 1.0.3 |
| toml_datetime@0.6.3 | registry; 7cda73e2f1397b1262d6dfdcef8aafae14d1de7748d66822d3bfeeb6d03e5e4b | serde |
| toml_datetime@0.7.5+spec-1.1.0 | registry; 92e1cfed4a3038bc5a127e35a2d360f145e1f4b971b551a2ba5fd7aedf7e1347 | serde_core |
| toml_datetime@1.1.1+spec-1.1.0 | registry; 3165f65f62e28e0115a00b2ebdd37eb6f3b641855f9d636d3cd4103767159ad7 | serde_core |
| toml_edit@0.19.15 | registry; 1b5bb770da30e5cbfde35a2d7b9b8a2c4b8ef89548a7a6aeab5c9a576e3e7421 | indexmap 2.14.0, toml_datetime 0.6.3, winnow 0.5.40 |
| toml_edit@0.20.2 | registry; 396e4d48bbb2b7554c944bde63101b5ae446cff6ec4a24227428f15eb72ef338 | indexmap 2.14.0, serde, serde_spanned 0.6.9, toml_datetime 0.6.3, winnow 0.5.40 |
| toml_edit@0.25.12+spec-1.1.0 | registry; d2153edc6955a6c354fad8f5efd38b6a8769bdccf9fe50f8e1329f81b0baa5d7 | indexmap 2.14.0, toml_datetime 1.1.1+spec-1.1.0, toml_parser, winnow 1.0.3 |
| toml_parser@1.1.2+spec-1.1.0 | registry; a2abe9b86193656635d2411dc43050282ca48aa31c2451210f4202550afb7526 | winnow 1.0.3 |
| toml_writer@1.1.1+spec-1.1.0 | registry; 756daf9b1013ebe47a8776667b466417e2d4c5679d441c26230efd9ef78692db |  |
| tower@0.5.3 | registry; ebe5ef63511595f1344e2d5cfa636d973292adc0eec1f0ad45fae9f0851ab1d4 | futures-core, futures-util, pin-project-lite, sync_wrapper, tokio, tower-layer, tower-service |
| tower-http@0.6.11 | registry; 4cfcf7e2740e6fc6d4d688b4ef00650406bb94adf4731e43c096c3a19fe40840 | async-compression, bitflags 2.13.0, bytes, futures-core, futures-util, http, http-body, http-body-util, pin-project-lite, tokio, tokio-util, tower, tower-layer, tower-service, url |
| tower-layer@0.3.3 | registry; 121c2a6cda46980bb0fcd1647ffaf6cd3fc79a013de288782836f6df9c48780e |  |
| tower-service@0.3.3 | registry; 8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3 |  |
| tracing@0.1.44 | registry; 63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100 | log, pin-project-lite, tracing-attributes, tracing-core |
| tracing-attributes@0.1.31 | registry; 7490cfa5ec963746568740651ac6781f701c9c5ea257c58e057f3ba8cf69e8da | proc-macro2, quote, syn 2.0.117 |
| tracing-core@0.1.36 | registry; db97caf9d906fbde555dd62fa95ddba9eecfd14cb388e4f491a66d74cd5fb79a | once_cell |
| tray-icon@0.23.1 | registry; 15edbb0d80583e85ee8df283410038e17314df5cba30da2087a54a85216c0773 | crossbeam-channel, dirs, libappindicator, muda, objc2, objc2-app-kit, objc2-core-foundation, objc2-core-graphics, objc2-foundation, once_cell, png 0.18.1, serde, thiserror 2.0.18, windows-sys 0.61.2 |
| try-lock@0.2.5 | registry; e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b |  |
| tungstenite@0.28.0 | registry; 8628dcc84e5a09eb3d8423d6cb682965dea9133204e8fb3efee74c2a0c259442 | bytes, data-encoding, http, httparse, log, rand 0.9.4, rustls 0.23.40, rustls-pki-types, sha1, thiserror 2.0.18, utf-8 |
| typeid@1.0.3 | registry; bc7d623258602320d5c55d1bc22793b57daff0ec7efc270ea7d55ce1d5f5471c |  |
| typenum@1.20.1 | registry; b6f5e870be6c3b371b77fe0ee0bafb859fa4964b4404c27de1d380043c4dda20 |  |
| uds_windows@1.2.1 | registry; f2f6fb2847f6742cd76af783a2a2c49e9375d0a111c7bef6f71cd9e738c72d6e | memoffset 0.9.1, tempfile, windows-sys 0.61.2 |
| unic-char-property@0.9.0 | registry; a8c57a407d9b6fa02b4795eb81c5b6652060a15a7903ea981f3d723e6c0be221 | unic-char-range |
| unic-char-range@0.9.0 | registry; 0398022d5f700414f6b899e10b8348231abf9173fa93144cbc1a43b9793c1fbc |  |
| unic-common@0.9.0 | registry; 80d7ff825a6a654ee85a63e80f92f054f904f21e7d12da4e22f9834a4aaa35bc |  |
| unic-ucd-ident@0.9.0 | registry; e230a37c0381caa9219d67cf063aa3a375ffed5bf541a452db16e744bdab6987 | unic-char-property, unic-char-range, unic-ucd-version |
| unic-ucd-version@0.9.0 | registry; 96bd2f2237fe450fcd0a1d2f5f4e91711124f7857ba2e964247776ebeeb7b0c4 | unic-common |
| unicode-bidi@0.3.18 | registry; 5c1cb5db39152898a79168971543b1cb5020dff7fe43c8dc468b0885f5e29df5 |  |
| unicode-ident@1.0.24 | registry; e6e4313cd5fcd3dad5cafa179702e2b244f760991f45397d14d4ebf38247da75 |  |
| unicode-normalization@0.1.25 | registry; 5fd4f6878c9cb28d874b009da9e8d183b5abc80117c40bbd187a1fde336be6e8 | tinyvec |
| unicode-properties@0.1.4 | registry; 7df058c713841ad818f1dc5d3fd88063241cc61f49f5fbea4b951e8cf5a8d71d |  |
| unicode-segmentation@1.13.3 | registry; c6f5d3c3b1bf09027a88a6bc961fc00497d651009560b5463668dc81b0fa87a8 |  |
| unicode-xid@0.2.6 | registry; ebc1c04c71510c7f702b52b7c350734c9ff1295c464a03335b00bb84fc54f853 |  |
| untrusted@0.9.0 | registry; 8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1 |  |
| url@2.5.8 | registry; ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed | form_urlencoded, idna, percent-encoding, serde, serde_derive |
| urlpattern@0.3.0 | registry; 70acd30e3aa1450bc2eece896ce2ad0d178e9c079493819301573dae3c37ba6d | regex, serde, unic-ucd-ident, url |
| utf-8@0.7.6 | registry; 09cc8ee72d2a9becf2f2febe0205bbed8fc6615b7cb429ad062dc7b7ddd036a9 |  |
| utf8_iter@1.0.4 | registry; b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be |  |
| uuid@0.8.2 | registry; bc5cf98d8186244414c848017f0e2676b3fcb46807f6668a97dfe67359a3c4b7 | getrandom 0.2.17 |
| uuid@1.23.3 | registry; 144d6b123cef80b301b8f72a9e2ca4370ddec21950d0a103dd22c437006d2db7 | getrandom 0.4.2, js-sys, serde_core, wasm-bindgen |
| vcpkg@0.2.15 | registry; accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426 |  |
| vergen@9.0.6 | registry; 6b2bf58be11fc9414104c6d3a2e464163db5ef74b12296bda593cac37b6e4777 | anyhow, derive_builder, rustversion, time, vergen-lib |
| vergen-gitcl@1.0.8 | registry; b9dfc1de6eb2e08a4ddf152f1b179529638bedc0ea95e6d667c014506377aefe | anyhow, derive_builder, rustversion, time, vergen, vergen-lib |
| vergen-lib@0.1.6 | registry; 9b07e6010c0f3e59fcb164e0163834597da68d1f864e2b8ca49f74de01e9c166 | anyhow, derive_builder, rustversion |
| version-compare@0.2.1 | registry; 03c2856837ef78f57382f06b2b8563a2f512f7185d732608fd9176cb3b8edf0e |  |
| version_check@0.9.5 | registry; 0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a |  |
| vswhom@0.1.0 | registry; be979b7f07507105799e854203b470ff7c78a1639e330a58f183b5fea574608b | libc, vswhom-sys |
| vswhom-sys@0.1.3 | registry; fb067e4cbd1ff067d1df46c9194b5de0e98efd2810bbc95c5d5e5f25a3231150 | cc, libc |
| waker-fn@1.2.0 | registry; 317211a0dc0ceedd78fb2ca9a44aed3d7b9b26f81870d485c07122b4350673b7 |  |
| walkdir@2.5.0 | registry; 29790946404f91d9c5d06f9874efddea1dc06c5efe94541a7d6863108e3a5e4b | same-file, winapi-util |
| want@0.3.1 | registry; bfa7760aed19e106de2c7c0b581b509f2f25d3dacaf737cb82ac61bc6d760b0e | try-lock |
| wasi@0.11.1+wasi-snapshot-preview1 | registry; ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b |  |
| wasip2@1.0.4+wasi-0.2.12 | registry; b67efb37e106e55ce722a510d6b5f9c17f083e5fc79afc2badeb12cc313d9487 | wit-bindgen 0.57.1 |
| wasip3@0.4.0+wasi-0.3.0-rc-2026-01-06 | registry; 5428f8bf88ea5ddc08faddef2ac4a67e390b88186c703ce6dbd955e1c145aca5 | wit-bindgen 0.51.0 |
| wasite@0.1.0 | registry; b8dad83b4f25e74f184f64c43b150b91efe7647395b42289f38e50566d82855b |  |
| wasm-bindgen@0.2.125 | registry; 8ddb3f79143bced6de84270411622a2699cee572fc0875aeaf1e7867cf9fca1a | cfg-if, once_cell, rustversion, wasm-bindgen-macro, wasm-bindgen-shared |
| wasm-bindgen-futures@0.4.75 | registry; 503b14d284f2c8dac03b819967e155ea753f573586193b2b2c95990cb5d69280 | js-sys, wasm-bindgen |
| wasm-bindgen-macro@0.2.125 | registry; 4e21a184b13fb19e157296e2c46056aec9092264fab83e4ba59e68c61b323c3d | quote, wasm-bindgen-macro-support |
| wasm-bindgen-macro-support@0.2.125 | registry; fecefd9c35bd935a20fc3fc344b5f29138961e4f47fb03297d88f2587afb5ebd | bumpalo, proc-macro2, quote, syn 2.0.117, wasm-bindgen-shared |
| wasm-bindgen-shared@0.2.125 | registry; 23939e44bb9a5d7576fa2b563dc2e136628f1224e88a8deed09e04858b77871f | unicode-ident |
| wasm-encoder@0.244.0 | registry; 990065f2fe63003fe337b932cfb5e3b80e0b4d0f5ff650e6985b1048f62c8319 | leb128fmt, wasmparser |
| wasm-metadata@0.244.0 | registry; bb0e353e6a2fbdc176932bbaab493762eb1255a7900fe0fea1a2f96c296cc909 | anyhow, indexmap 2.14.0, wasm-encoder, wasmparser |
| wasm-streams@0.5.0 | registry; 9d1ec4f6517c9e11ae630e200b2b65d193279042e28edd4a2cda233e46670bbb | futures-util, js-sys, wasm-bindgen, wasm-bindgen-futures, web-sys |
| wasmparser@0.244.0 | registry; 47b807c72e1bac69382b3a6fb3dbe8ea4c0ed87ff5629b8685ae6b9a611028fe | bitflags 2.13.0, hashbrown 0.15.5, indexmap 2.14.0, semver |
| web-sys@0.3.102 | registry; a6430a72df5eb332242960fe84b3002a241163998241eb596d4f739b9757061d | js-sys, wasm-bindgen |
| web-time@1.1.0 | registry; 5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb | js-sys, wasm-bindgen |
| web_atoms@0.2.4 | registry; d7cff6eef815df1834fd250e3a2ff436044d82a9f1bc1980ca1dbdf07effc538 | phf, phf_codegen, string_cache, string_cache_codegen |
| webkit2gtk@2.0.2 | registry; a1027150013530fb2eaf806408df88461ae4815a45c541c8975e61d6f2fc4793 | bitflags 1.3.2, cairo-rs, gdk, gdk-sys, gio, gio-sys, glib, glib-sys, gobject-sys, gtk, gtk-sys, javascriptcore-rs, libc, once_cell, soup3, webkit2gtk-sys |
| webkit2gtk-sys@2.0.2 | registry; 916a5f65c2ef0dfe12fff695960a2ec3d4565359fdbb2e9943c974e06c734ea5 | bitflags 1.3.2, cairo-sys-rs, gdk-sys, gio-sys, glib-sys, gobject-sys, gtk-sys, javascriptcore-rs-sys, libc, pkg-config, soup3-sys, system-deps |
| webpki@0.22.4 | registry; ed63aea5ce73d0ff405984102c42de94fc55a6b75765d621c65262469b3c9b53 | ring, untrusted |
| webpki-root-certs@1.0.8 | registry; 0d46a5a140e6f7afeccd8eae97eff335163939eac8b929834875168b29b3d267 | rustls-pki-types |
| webpki-roots@0.26.11 | registry; 521bc38abb08001b01866da9f51eb7c5d647a19260e00054a8c7fd5f9e57f7a9 | webpki-roots 1.0.7 |
| webpki-roots@1.0.7 | registry; 52f5ee44c96cf55f1b349600768e3ece3a8f26010c05265ab73f945bb1a2eb9d | rustls-pki-types |
| webview2-com@0.38.2 | registry; 7130243a7a5b33c54a444e54842e6a9e133de08b5ad7b5861cd8ed9a6a5bc96a | webview2-com-macros, webview2-com-sys, windows 0.61.3, windows-core 0.61.2, windows-implement, windows-interface |
| webview2-com-macros@0.8.1 | registry; 67a921c1b6914c367b2b823cd4cde6f96beec77d30a939c8199bb377cf9b9b54 | proc-macro2, quote, syn 2.0.117 |
| webview2-com-sys@0.38.2 | registry; 381336cfffd772377d291702245447a5251a2ffa5bad679c99e61bc48bacbf9c | thiserror 2.0.18, windows 0.61.3, windows-core 0.61.2 |
| which@4.4.2 | registry; 87ba24419a2078cd2b0f2ede2691b6c66d8e47836da3b6db8265ebad47afbfc7 | either, home, once_cell, rustix 0.38.44 |
| whoami@1.6.1 | registry; 5d4a4db5077702ca3015d3d02d74974948aba2ad9e12ab7df718ee64ccd7e97d | libredox, wasite |
| winapi@0.3.9 | registry; 5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419 | winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu |
| winapi-i686-pc-windows-gnu@0.4.0 | registry; ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6 |  |
| winapi-util@0.1.11 | registry; c2a7b1c03c876122aa43f3020e6c3c3ee5c05081c9a00739faf7503aeba10d22 | windows-sys 0.61.2 |
| winapi-x86_64-pc-windows-gnu@0.4.0 | registry; 712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f |  |
| window-vibrancy@0.6.0 | registry; d9bec5a31f3f9362f2258fd0e9c9dd61a9ca432e7306cc78c444258f0dce9a9c | objc2, objc2-app-kit, objc2-core-foundation, objc2-foundation, raw-window-handle, windows-sys 0.59.0, windows-version |
| windows@0.44.0 | registry; 9e745dab35a0c4c77aa3ce42d595e13d2003d6902d6b08c9ef5fc326d08da12b | windows-targets 0.42.2 |
| windows@0.54.0 | registry; 9252e5725dbed82865af151df558e754e4a3c2c30818359eb17465f1346a1b49 | windows-core 0.54.0, windows-targets 0.52.6 |
| windows@0.61.3 | registry; 9babd3a767a4c1aef6900409f85f5d53ce2544ccdfaa86dad48c91782c6d6893 | windows-collections, windows-core 0.61.2, windows-future, windows-link 0.1.3, windows-numerics |
| windows-collections@0.2.0 | registry; 3beeceb5e5cfd9eb1d76b381630e82c4241ccd0d27f1a39ed41b2760b255c5e8 | windows-core 0.61.2 |
| windows-core@0.54.0 | registry; 12661b9c89351d684a50a8a643ce5f608e20243b9fb84687800163429f161d65 | windows-result 0.1.2, windows-targets 0.52.6 |
| windows-core@0.61.2 | registry; c0fdd3ddb90610c7638aa2b3a3ab2904fb9e5cdbecc643ddb3647212781c4ae3 | windows-implement, windows-interface, windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-core@0.62.2 | registry; b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb | windows-implement, windows-interface, windows-link 0.2.1, windows-result 0.4.1, windows-strings 0.5.1 |
| windows-future@0.2.1 | registry; fc6a41e98427b19fe4b73c550f060b59fa592d7d686537eebf9385621bfbad8e | windows-core 0.61.2, windows-link 0.1.3, windows-threading |
| windows-implement@0.60.2 | registry; 053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf | proc-macro2, quote, syn 2.0.117 |
| windows-interface@0.59.3 | registry; 3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358 | proc-macro2, quote, syn 2.0.117 |
| windows-link@0.1.3 | registry; 5e6ad25900d524eaabdbbb96d20b4311e1e7ae1699af4fb28c17ae66c80d798a |  |
| windows-link@0.2.1 | registry; f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5 |  |
| windows-numerics@0.2.0 | registry; 9150af68066c4c5c07ddc0ce30421554771e528bde427614c61038bc2c92c2b1 | windows-core 0.61.2, windows-link 0.1.3 |
| windows-registry@0.6.1 | registry; 02752bf7fbdcce7f2a27a742f798510f3e5ad88dbe84871e5168e2120c3d5720 | windows-link 0.2.1, windows-result 0.4.1, windows-strings 0.5.1 |
| windows-result@0.1.2 | registry; 5e383302e8ec8515204254685643de10811af0ed97ea37210dc26fb0032647f8 | windows-targets 0.52.6 |
| windows-result@0.3.4 | registry; 56f42bd332cc6c8eac5af113fc0c1fd6a8fd2aa08a0119358686e5160d0586c6 | windows-link 0.1.3 |
| windows-result@0.4.1 | registry; 7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5 | windows-link 0.2.1 |
| windows-strings@0.4.2 | registry; 56e6c93f3a0c3b36176cb1327a4958a0353d5d166c2a35cb268ace15e91d3b57 | windows-link 0.1.3 |
| windows-strings@0.5.1 | registry; 7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091 | windows-link 0.2.1 |
| windows-sys@0.45.0 | registry; 75283be5efb2831d37ea142365f009c02ec203cd29a3ebecbc093d52315b66d0 | windows-targets 0.42.2 |
| windows-sys@0.48.0 | registry; 677d2418bec65e3338edb076e806bc1ec15693c5d0104683f2efe857f61056a9 | windows-targets 0.48.5 |
| windows-sys@0.52.0 | registry; 282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d | windows-targets 0.52.6 |
| windows-sys@0.59.0 | registry; 1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b | windows-targets 0.52.6 |
| windows-sys@0.60.2 | registry; f2f500e4d28234f72040990ec9d39e3a6b950f9f22d3dba18416c35882612bcb | windows-targets 0.53.5 |
| windows-sys@0.61.2 | registry; ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc | windows-link 0.2.1 |
| windows-targets@0.42.2 | registry; 8e5180c00cd44c9b1c88adb3693291f1cd93605ded80c250a75d472756b4d071 | windows_aarch64_gnullvm 0.42.2, windows_aarch64_msvc 0.42.2, windows_i686_gnu 0.42.2, windows_i686_msvc 0.42.2, windows_x86_64_gnu 0.42.2, windows_x86_64_gnullvm 0.42.2, windows_x86_64_msvc 0.42.2 |
| windows-targets@0.48.5 | registry; 9a2fa6e2155d7247be68c096456083145c183cbbbc2764150dda45a87197940c | windows_aarch64_gnullvm 0.48.5, windows_aarch64_msvc 0.48.5, windows_i686_gnu 0.48.5, windows_i686_msvc 0.48.5, windows_x86_64_gnu 0.48.5, windows_x86_64_gnullvm 0.48.5, windows_x86_64_msvc 0.48.5 |
| windows-targets@0.52.6 | registry; 9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973 | windows_aarch64_gnullvm 0.52.6, windows_aarch64_msvc 0.52.6, windows_i686_gnu 0.52.6, windows_i686_gnullvm 0.52.6, windows_i686_msvc 0.52.6, windows_x86_64_gnu 0.52.6, windows_x86_64_gnullvm 0.52.6, windows_x86_64_msvc 0.52.6 |
| windows-targets@0.53.5 | registry; 4945f9f551b88e0d65f3db0bc25c33b8acea4d9e41163edf90dcd0b19f9069f3 | windows-link 0.2.1, windows_aarch64_gnullvm 0.53.1, windows_aarch64_msvc 0.53.1, windows_i686_gnu 0.53.1, windows_i686_gnullvm 0.53.1, windows_i686_msvc 0.53.1, windows_x86_64_gnu 0.53.1, windows_x86_64_gnullvm 0.53.1, windows_x86_64_msvc 0.53.1 |
| windows-threading@0.1.0 | registry; b66463ad2e0ea3bbf808b7f1d371311c80e115c0b71d60efc142cafbcfb057a6 | windows-link 0.1.3 |
| windows-version@0.1.7 | registry; e4060a1da109b9d0326b7262c8e12c84df67cc0dbc9e33cf49e01ccc2eb63631 | windows-link 0.2.1 |
| windows_aarch64_gnullvm@0.42.2 | registry; 597a5118570b68bc08d8d59125332c54f1ba9d9adeedeef5b99b02ba2b0698f8 |  |
| windows_aarch64_gnullvm@0.48.5 | registry; 2b38e32f0abccf9987a4e3079dfb67dcd799fb61361e53e2882c3cbaf0d905d8 |  |
| windows_aarch64_gnullvm@0.52.6 | registry; 32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3 |  |
| windows_aarch64_gnullvm@0.53.1 | registry; a9d8416fa8b42f5c947f8482c43e7d89e73a173cead56d044f6a56104a6d1b53 |  |
| windows_aarch64_msvc@0.42.2 | registry; e08e8864a60f06ef0d0ff4ba04124db8b0fb3be5776a5cd47641e942e58c4d43 |  |
| windows_aarch64_msvc@0.48.5 | registry; dc35310971f3b2dbbf3f0690a219f40e2d9afcf64f9ab7cc1be722937c26b4bc |  |
| windows_aarch64_msvc@0.52.6 | registry; 09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469 |  |
| windows_aarch64_msvc@0.53.1 | registry; b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006 |  |
| windows_i686_gnu@0.42.2 | registry; c61d927d8da41da96a81f029489353e68739737d3beca43145c8afec9a31a84f |  |
| windows_i686_gnu@0.48.5 | registry; a75915e7def60c94dcef72200b9a8e58e5091744960da64ec734a6c6e9b3743e |  |
| windows_i686_gnu@0.52.6 | registry; 8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b |  |
| windows_i686_gnu@0.53.1 | registry; 960e6da069d81e09becb0ca57a65220ddff016ff2d6af6a223cf372a506593a3 |  |
| windows_i686_gnullvm@0.52.6 | registry; 0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66 |  |
| windows_i686_gnullvm@0.53.1 | registry; fa7359d10048f68ab8b09fa71c3daccfb0e9b559aed648a8f95469c27057180c |  |
| windows_i686_msvc@0.42.2 | registry; 44d840b6ec649f480a41c8d80f9c65108b92d89345dd94027bfe06ac444d1060 |  |
| windows_i686_msvc@0.48.5 | registry; 8f55c233f70c4b27f66c523580f78f1004e8b5a8b659e05a4eb49d4166cca406 |  |
| windows_i686_msvc@0.52.6 | registry; 240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66 |  |
| windows_i686_msvc@0.53.1 | registry; 1e7ac75179f18232fe9c285163565a57ef8d3c89254a30685b57d83a38d326c2 |  |
| windows_x86_64_gnu@0.42.2 | registry; 8de912b8b8feb55c064867cf047dda097f92d51efad5b491dfb98f6bbb70cb36 |  |
| windows_x86_64_gnu@0.48.5 | registry; 53d40abd2583d23e4718fddf1ebec84dbff8381c07cae67ff7768bbf19c6718e |  |
| windows_x86_64_gnu@0.52.6 | registry; 147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78 |  |
| windows_x86_64_gnu@0.53.1 | registry; 9c3842cdd74a865a8066ab39c8a7a473c0778a3f29370b5fd6b4b9aa7df4a499 |  |
| windows_x86_64_gnullvm@0.42.2 | registry; 26d41b46a36d453748aedef1486d5c7a85db22e56aff34643984ea85514e94a3 |  |
| windows_x86_64_gnullvm@0.48.5 | registry; 0b7b52767868a23d5bab768e390dc5f5c55825b6d30b86c844ff2dc7414044cc |  |
| windows_x86_64_gnullvm@0.52.6 | registry; 24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d |  |
| windows_x86_64_gnullvm@0.53.1 | registry; 0ffa179e2d07eee8ad8f57493436566c7cc30ac536a3379fdf008f47f6bb7ae1 |  |
| windows_x86_64_msvc@0.42.2 | registry; 9aec5da331524158c6d1a4ac0ab1541149c0b9505fde06423b02f5ef0106b9f0 |  |
| windows_x86_64_msvc@0.48.5 | registry; ed94fce61571a4006852b7389a063ab983c02eb1bb37b47f8272ce92d06d9538 |  |
| windows_x86_64_msvc@0.52.6 | registry; 589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec |  |
| windows_x86_64_msvc@0.53.1 | registry; d6bbff5f0aada427a1e5a6da5f1f98158182f26556f345ac9e04d36d0ebed650 |  |
| winnow@0.5.40 | registry; f593a95398737aeed53e489c785df13f3618e41dbcd6718c6addbf1395aa6876 | memchr |
| winnow@0.7.15 | registry; df79d97927682d2fd8adb29682d1140b343be4ac0f08fd68b7765d9c059d3945 |  |
| winnow@1.0.3 | registry; 0592e1c9d151f854e6fd382574c3a0855250e1d9b2f99d9281c6e6391af352f1 | memchr |
| winreg@0.55.0 | registry; cb5a765337c50e9ec252c2069be9bf91c7df47afb103b642ba3a53bf8101be97 | cfg-if, windows-sys 0.59.0 |
| wit-bindgen@0.51.0 | registry; d7249219f66ced02969388cf2bb044a09756a083d0fab1e566056b04d9fbcaa5 | wit-bindgen-rust-macro |
| wit-bindgen@0.57.1 | registry; 1ebf944e87a7c253233ad6766e082e3cd714b5d03812acc24c318f549614536e |  |
| wit-bindgen-core@0.51.0 | registry; ea61de684c3ea68cb082b7a88508a8b27fcc8b797d738bfc99a82facf1d752dc | anyhow, heck 0.5.0, wit-parser |
| wit-bindgen-rust@0.51.0 | registry; b7c566e0f4b284dd6561c786d9cb0142da491f46a9fbed79ea69cdad5db17f21 | anyhow, heck 0.5.0, indexmap 2.14.0, prettyplease, syn 2.0.117, wasm-metadata, wit-bindgen-core, wit-component |
| wit-bindgen-rust-macro@0.51.0 | registry; 0c0f9bfd77e6a48eccf51359e3ae77140a7f50b1e2ebfe62422d8afdaffab17a | anyhow, prettyplease, proc-macro2, quote, syn 2.0.117, wit-bindgen-core, wit-bindgen-rust |
| wit-component@0.244.0 | registry; 9d66ea20e9553b30172b5e831994e35fbde2d165325bec84fc43dbf6f4eb9cb2 | anyhow, bitflags 2.13.0, indexmap 2.14.0, log, serde, serde_derive, serde_json, wasm-encoder, wasm-metadata, wasmparser, wit-parser |
| wit-parser@0.244.0 | registry; ecc8ac4bc1dc3381b7f59c34f00b67e18f910c2c0f50015669dde7def656a736 | anyhow, id-arena, indexmap 2.14.0, log, semver, serde, serde_derive, serde_json, unicode-xid, wasmparser |
| writeable@0.6.3 | registry; 1ffae5123b2d3fc086436f8834ae3ab053a283cfac8fe0a0b8eaae044768a4c4 |  |
| wry@0.55.1 | registry; 186f9871daa55fd9c016578b810d149de58367113db7fb72b462d2323ce19514 | base64 0.22.1, block2, cookie, crossbeam-channel, dirs, dom_query, dpi, dunce, gdkx11, gtk, http, javascriptcore-rs, jni 0.21.1, libc, ndk, objc2, objc2-app-kit, objc2-core-foundation, objc2-foundation, objc2-ui-kit, objc2-web-kit, once_cell, percent-encoding, raw-window-handle, sha2, soup3, tao-macros, thiserror 2.0.18, url, webkit2gtk, webkit2gtk-sys, webview2-com, windows 0.61.3, windows-core 0.61.2, windows-version, x11-dl |
| x11@2.21.0 | registry; 502da5464ccd04011667b11c435cb992822c2c0dbde1770c988480d312a0db2e | libc, pkg-config |
| x11-dl@2.21.0 | registry; 38735924fedd5314a6e548792904ed8c6de6636285cb9fec04d5b1db85c1516f | libc, once_cell, pkg-config |
| xattr@1.6.1 | registry; 32e45ad4206f6d2479085147f02bc2ef834ac85886624a23575ae137c8aa8156 | libc, rustix 1.1.4 |
| xdg-home@1.3.0 | registry; ec1cdab258fb55c0da61328dc52c8764709b249011b2cad0454c72f0bf10a1f6 | libc, windows-sys 0.59.0 |
| yoke@0.8.3 | registry; 709fe23a0424b6a435d82152b1bd3fdfb0833487d5fa90d05d42762a9891fef5 | stable_deref_trait, yoke-derive, zerofrom |
| yoke-derive@0.8.2 | registry; de844c262c8848816172cef550288e7dc6c7b7814b4ee56b3e1553f275f1858e | proc-macro2, quote, syn 2.0.117, synstructure |
| zbus@3.15.2 | registry; 675d170b632a6ad49804c8cf2105d7c31eddd3312555cffd4b740e08e97c25e6 | async-broadcast 0.5.1, async-executor, async-fs, async-io 1.13.0, async-lock 2.8.0, async-process 1.8.1, async-recursion, async-task, async-trait, blocking, byteorder, derivative, enumflags2, event-listener 2.5.3, futures-core, futures-sink, futures-util, hex, nix, once_cell, ordered-stream, rand 0.8.6, serde, serde_repr, sha1, static_assertions, tracing, uds_windows, winapi, xdg-home, zbus_macros 3.15.2, zbus_names 2.6.1, zvariant 3.15.2 |
| zbus@5.16.0 | registry; eee682d202a77e4a9f3b2c2bdf48a7b28af5c08c34ddf66f98c93e5e39464285 | async-broadcast 0.7.2, async-executor, async-io 2.6.0, async-lock 3.4.2, async-process 2.5.0, async-recursion, async-task, async-trait, blocking, enumflags2, event-listener 5.4.1, futures-core, futures-lite 2.6.1, hex, libc, ordered-stream, rustix 1.1.4, serde, serde_repr, tracing, uds_windows, uuid 1.23.3, windows-sys 0.61.2, winnow 1.0.3, zbus_macros 5.16.0, zbus_names 4.3.2, zvariant 5.12.0 |
| zbus_macros@3.15.2 | registry; 7131497b0f887e8061b430c530240063d33bf9455fa34438f388a245da69e0a5 | proc-macro-crate 1.3.1, proc-macro2, quote, regex, syn 1.0.109, zvariant_utils 1.0.1 |
| zbus_macros@5.16.0 | registry; adf1bd45a81a103745b1757754762a26e8cd01e4532e4d6c8ec431624b80d1d6 | proc-macro-crate 3.5.0, proc-macro2, quote, syn 2.0.117, zbus_names 4.3.2, zvariant 5.12.0, zvariant_utils 3.4.0 |
| zbus_names@2.6.1 | registry; 437d738d3750bed6ca9b8d423ccc7a8eb284f6b1d6d4e225a0e4e6258d864c8d | serde, static_assertions, zvariant 3.15.2 |
| zbus_names@4.3.2 | registry; 7074f3e50b894eac91750142016d30d0a89be8e67dbfd9704fb875825760e52d | serde, winnow 1.0.3, zvariant 5.12.0 |
| zerocopy@0.8.52 | registry; ce1022995ff5ff5d841ad7d994facc23098cd40152f2c1d11cd607c6f530653f | zerocopy-derive |
| zerocopy-derive@0.8.52 | registry; 1ae7f38b72ec2a254e2b87ef277cf2cd4fb97cbebf944faa6f33354da0867930 | proc-macro2, quote, syn 2.0.117 |
| zerofrom@0.1.8 | registry; 0ec05a11813ea801ff6d75110ad09cd0824ddba17dfe17128ea0d5f68e6c5272 | zerofrom-derive |
| zerofrom-derive@0.1.7 | registry; 11532158c46691caf0f2593ea8358fed6bbf68a0315e80aae9bd41fbade684a1 | proc-macro2, quote, syn 2.0.117, synstructure |
| zeroize@1.9.0 | registry; e13c156562582aa81c60cb29407084cdb54c4164760106ab78e6c5b0858cf64e |  |
| zerotrie@0.2.4 | registry; 0f9152d31db0792fa83f70fb2f83148effb5c1f5b8c7686c3459e361d9bc20bf | displaydoc, yoke, zerofrom |
| zerovec@0.11.6 | registry; 90f911cbc359ab6af17377d242225f4d75119aec87ea711a880987b18cd7b239 | yoke, zerofrom, zerovec-derive |
| zerovec-derive@0.11.3 | registry; 625dc425cab0dca6dc3c3319506e6593dcb08a9f387ea3b284dbd52a92c40555 | proc-macro2, quote, syn 2.0.117 |
| zip@4.6.1 | registry; caa8cd6af31c3b31c6631b8f483848b91589021b28fffe50adada48d4f4d2ed1 | arbitrary, crc32fast, indexmap 2.14.0, memchr |
| zmij@1.0.21 | registry; b8848ee67ecc8aedbaf3e4122217aff892639231befc6a1b58d29fff4c2cabaa |  |
| zvariant@3.15.2 | registry; 4eef2be88ba09b358d3b58aca6e41cd853631d44787f319a1383ca83424fb2db | byteorder, enumflags2, libc, serde, static_assertions, zvariant_derive 3.15.2 |
| zvariant@5.12.0 | registry; a192a0bde63360d77a7523c833d4b4ce6070a927e2c53246e4c540b1a3e27be0 | endi, enumflags2, serde, winnow 1.0.3, zvariant_derive 5.12.0, zvariant_utils 3.4.0 |
| zvariant_derive@3.15.2 | registry; 37c24dc0bed72f5f90d1f8bb5b07228cbf63b3c6e9f82d82559d4bae666e7ed9 | proc-macro-crate 1.3.1, proc-macro2, quote, syn 1.0.109, zvariant_utils 1.0.1 |
| zvariant_derive@5.12.0 | registry; 90bc6cde9c01c511074be97f7ccb6c19d0da89e3f8662e812e999dcfd4638737 | proc-macro-crate 3.5.0, proc-macro2, quote, syn 2.0.117, zvariant_utils 3.4.0 |
| zvariant_utils@1.0.1 | registry; 7234f0d811589db492d16893e3f21e8e2fd282e6d01b0cddee310322062cc200 | proc-macro2, quote, syn 1.0.109 |
| zvariant_utils@3.4.0 | registry; 1e8535915cfa75547e559d8c68e8139909a4aeee076831e4ef7fc59d8172c4d6 | proc-macro2, quote, serde, syn 2.0.117, winnow 1.0.3 |

## src-tauri/vendor/librespot-audio/Cargo.lock

{"path": "src-tauri/vendor/librespot-audio/Cargo.lock", "packages": 329, "dependency_edges": 882, "duplicate_names": 31, "unresolved_or_ambiguous_edges": [], "nonregistry_packages": ["librespot-audio@0.8.0"]}

### Multiple versions

{"core-foundation": ["0.9.4", "0.10.1"], "getrandom": ["0.2.16", "0.3.4"], "hashbrown": ["0.15.5", "0.16.0"], "hyper-rustls": ["0.26.0", "0.27.7"], "linux-raw-sys": ["0.4.15", "0.11.0"], "rand": ["0.8.5", "0.9.2"], "rand_chacha": ["0.3.1", "0.9.0"], "rand_core": ["0.6.4", "0.9.3"], "rustix": ["0.38.44", "1.1.2"], "rustls": ["0.22.4", "0.23.35"], "rustls-native-certs": ["0.7.3", "0.8.2"], "rustls-webpki": ["0.102.8", "0.103.8"], "security-framework": ["2.11.1", "3.5.1"], "thiserror": ["1.0.69", "2.0.17"], "thiserror-impl": ["1.0.69", "2.0.17"], "tokio-rustls": ["0.25.0", "0.26.4"], "webpki-roots": ["0.26.11", "1.0.4"], "windows-core": ["0.61.2", "0.62.2"], "windows-link": ["0.1.3", "0.2.1"], "windows-result": ["0.3.4", "0.4.1"], "windows-strings": ["0.4.2", "0.5.1"], "windows-sys": ["0.52.0", "0.59.0", "0.60.2", "0.61.2"], "windows-targets": ["0.52.6", "0.53.5"], "windows_aarch64_gnullvm": ["0.52.6", "0.53.1"], "windows_aarch64_msvc": ["0.52.6", "0.53.1"], "windows_i686_gnu": ["0.52.6", "0.53.1"], "windows_i686_gnullvm": ["0.52.6", "0.53.1"], "windows_i686_msvc": ["0.52.6", "0.53.1"], "windows_x86_64_gnu": ["0.52.6", "0.53.1"], "windows_x86_64_gnullvm": ["0.52.6", "0.53.1"], "windows_x86_64_msvc": ["0.52.6", "0.53.1"]}

### Every resolved package and edge

| Package | Source/checksum | Dependencies |
| --- | --- | --- |
| adler2@2.0.1 | registry; 320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa |  |
| aes@0.8.4 | registry; b169f7a6d4742236a0a00c541b845991d0ac43e546831af1249753ab4c3aa3a0 | cfg-if, cipher, cpufeatures |
| aho-corasick@1.1.4 | registry; ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301 | memchr |
| allocator-api2@0.2.21 | registry; 683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923 |  |
| android_system_properties@0.1.5 | registry; 819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311 | libc |
| anyhow@1.0.100 | registry; a23eb6b1614318a8071c9b2521f36b424b2c83db5eb3a0fead4a6c0809af6e61 |  |
| atomic-waker@1.1.2 | registry; 1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0 |  |
| autocfg@1.5.0 | registry; c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8 |  |
| base64@0.22.1 | registry; 72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6 |  |
| base64ct@1.8.0 | registry; 55248b47b0caf0546f7988906588779981c43bb1bc9d0c44087278f80cdb44ba |  |
| bitflags@2.10.0 | registry; 812e12b5285cc515a9c72a5c1d3b6d46a19dac5acfef5265968c166106e31dd3 |  |
| block-buffer@0.10.4 | registry; 3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71 | generic-array |
| bumpalo@3.19.0 | registry; 46c5e41b57b8bba42a04676d81cb89e9ee8e859a1a66f80a5a72e1cb76b34d43 |  |
| byteorder@1.5.0 | registry; 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |  |
| bytes@1.10.1 | registry; d71b6127be86fdcfddb610f7182ac57211d4b18a3e9c82eb2d17662f2227ad6a |  |
| cc@1.2.45 | registry; 35900b6c8d709fb1d854671ae27aeaa9eec2f8b01b364e1619a40da3e6fe2afe | find-msvc-tools, shlex |
| cfg-if@1.0.4 | registry; 9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801 |  |
| cfg_aliases@0.2.1 | registry; 613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724 |  |
| chrono@0.4.42 | registry; 145052bdd345b87320e369255277e3fb5152762ad123a901ef5c262dd38fe8d2 | iana-time-zone, js-sys, num-traits, serde, wasm-bindgen, windows-link 0.2.1 |
| cipher@0.4.4 | registry; 773f3b9af64447d2ce9850330c473515014aa235e6a783b02db81ff39e4a3dad | crypto-common, inout |
| const-oid@0.9.6 | registry; c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8 |  |
| core-foundation@0.9.4 | registry; 91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f | core-foundation-sys, libc |
| core-foundation@0.10.1 | registry; b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6 | core-foundation-sys, libc |
| core-foundation-sys@0.8.7 | registry; 773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b |  |
| cpufeatures@0.2.17 | registry; 59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280 | libc |
| crc32fast@1.5.0 | registry; 9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511 | cfg-if |
| crypto-common@0.1.6 | registry; 1bfb12502f3fc46cca1bb51ac28df9d618d813cdc3d2f25b9fe775a34af26bb3 | generic-array, typenum |
| ctr@0.9.2 | registry; 0369ee1ad671834580515889b80f2ea915f23b8be8d0daa4bbaf2ac5c7590835 | cipher |
| darling@0.20.11 | registry; fc7f46116c46ff9ab3eb1597a45688b6715c6e628b5c133e288e709a29bcb4ee | darling_core, darling_macro |
| darling_core@0.20.11 | registry; 0d00b9596d185e565c2207a0b01f8bd1a135483d02d9b7b0a54b11da8d53412e | fnv, ident_case, proc-macro2, quote, strsim, syn |
| darling_macro@0.20.11 | registry; fc34b93ccb385b40dc71c6fceac4b2ad23662c7eeb248cf10d529b7e055b6ead | darling_core, quote, syn |
| data-encoding@2.9.0 | registry; 2a2330da5de22e8a3cb63252ce2abb30116bf5265e89c0e01bc17015ce30a476 |  |
| der@0.7.10 | registry; e7c1832837b905bbfb5101e07cc24c8deddf52f93225eee6ead5f4d63d53ddcb | const-oid, pem-rfc7468, zeroize |
| deranged@0.5.5 | registry; ececcb659e7ba858fb4f10388c250a7252eb0a27373f1a72b8748afdd248e587 | powerfmt |
| derive_builder@0.20.2 | registry; 507dfb09ea8b7fa618fcf76e953f4f5e192547945816d5358edffe39f6f94947 | derive_builder_macro |
| derive_builder_core@0.20.2 | registry; 2d5bcf7b024d6835cfb3d473887cd966994907effbe9227e8c8219824d06c4e8 | darling, proc-macro2, quote, syn |
| derive_builder_macro@0.20.2 | registry; ab63b0e2bf4d5928aff72e83a7dace85d7bba5fe12dcc3c5a572d78caffd3f3c | derive_builder_core, syn |
| digest@0.10.7 | registry; 9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292 | block-buffer, const-oid, crypto-common, subtle |
| displaydoc@0.2.5 | registry; 97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0 | proc-macro2, quote, syn |
| either@1.15.0 | registry; 48c757948c5ede0e46177b7add2e67155f70e33c07fea8284df6576da70b3719 |  |
| equivalent@1.0.2 | registry; 877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f |  |
| errno@0.3.14 | registry; 39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb | libc, windows-sys 0.61.2 |
| fastrand@2.3.0 | registry; 37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be |  |
| find-msvc-tools@0.1.4 | registry; 52051878f80a721bb68ebfbc930e07b65ba72f2da88968ea5c06fd6ca3d3a127 |  |
| flate2@1.1.5 | registry; bfe33edd8e85a12a67454e37f8c75e730830d83e313556ab9ebf9ee7fbeb3bfb | crc32fast, miniz_oxide |
| fnv@1.0.7 | registry; 3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1 |  |
| foldhash@0.1.5 | registry; d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2 |  |
| foreign-types@0.3.2 | registry; f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1 | foreign-types-shared |
| foreign-types-shared@0.1.1 | registry; 00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b |  |
| form_urlencoded@1.2.2 | registry; cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf | percent-encoding |
| futures-channel@0.3.31 | registry; 2dff15bf788c671c1934e366d07e30c1814a8ef514e1af724a602e8a2fbe1b10 | futures-core, futures-sink |
| futures-core@0.3.31 | registry; 05f29059c0c2090612e8d742178b0580d2dc940c837851ad723096f87af6663e |  |
| futures-io@0.3.31 | registry; 9e5c1b78ca4aae1ac06c48a526a655760685149f0d465d21f37abfe57ce075c6 |  |
| futures-sink@0.3.31 | registry; e575fab7d1e0dcb8d0c7bcf9a63ee213816ab51902e6d244a95819acacf1d4f7 |  |
| futures-task@0.3.31 | registry; f90f7dce0722e95104fcb095585910c0977252f286e354b5e3bd38902cd99988 |  |
| futures-timer@3.0.3 | registry; f288b0a4f20f9a56b5d1da57e2227c661b7b16168e2f72365f57b63326e29b24 |  |
| futures-util@0.3.31 | registry; 9fa08315bb612088cc391249efdc3bc77536f16c91f6cf495e6fbe85b20a4a81 | futures-core, futures-io, futures-sink, futures-task, memchr, pin-project-lite, pin-utils, slab |
| generic-array@0.14.9 | registry; 4bb6743198531e02858aeaea5398fcc883e71851fcbcb5a2f773e2fb6cb1edf2 | typenum, version_check |
| getrandom@0.2.16 | registry; 335ff9f135e4384c8150d6f27c6daed433577f86b4750418338c01a1a2528592 | cfg-if, js-sys, libc, wasi, wasm-bindgen |
| getrandom@0.3.4 | registry; 899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd | cfg-if, js-sys, libc, r-efi, wasip2, wasm-bindgen |
| governor@0.10.1 | registry; 444405bbb1a762387aa22dd569429533b54a1d8759d35d3b64cb39b0293eaa19 | cfg-if, futures-sink, futures-timer, futures-util, hashbrown 0.15.5, nonzero_ext, parking_lot, portable-atomic, smallvec, spinning_top, web-time |
| h2@0.4.12 | registry; f3c0b69cfcb4e1b9f1bf2f53f95f766e4661169728ec61cd3fe5a0166f2d1386 | atomic-waker, bytes, fnv, futures-core, futures-sink, http, indexmap, slab, tokio, tokio-util, tracing |
| hashbrown@0.15.5 | registry; 9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1 | allocator-api2, equivalent, foldhash |
| hashbrown@0.16.0 | registry; 5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d |  |
| headers@0.4.1 | registry; b3314d5adb5d94bcdf56771f2e50dbbc80bb4bdf88967526706205ac9eff24eb | base64, bytes, headers-core, http, httpdate, mime, sha1 |
| headers-core@0.3.0 | registry; 54b4a22553d4242c49fddb9ba998a99962b5cc6f22cb5a3482bec22522403ce4 | http |
| hmac@0.12.1 | registry; 6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e | digest |
| home@0.5.11 | registry; 589533453244b0995c858700322199b2becb13b627df2851f64a2775d024abcf | windows-sys 0.59.0 |
| http@1.3.1 | registry; f4a85d31aea989eead29a3aaf9e1115a180df8282431156e533de47660892565 | bytes, fnv, itoa |
| http-body@1.0.1 | registry; 1efedce1fb8e6913f23e0c92de8e62cd5b772a67e7b3946df930a62566c93184 | bytes, http |
| http-body-util@0.1.3 | registry; b021d93e26becf5dc7e1b75b1bed1fd93124b374ceb73f43d4d4eafec896a64a | bytes, futures-core, http, http-body, pin-project-lite |
| httparse@1.10.1 | registry; 6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87 |  |
| httpdate@1.0.3 | registry; df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9 |  |
| hyper@1.7.0 | registry; eb3aa54a13a0dfe7fbe3a59e0c76093041720fdc77b110cc0fc260fafb4dc51e | atomic-waker, bytes, futures-channel, futures-core, h2, http, http-body, httparse, itoa, pin-project-lite, pin-utils, smallvec, tokio, want |
| hyper-proxy2@0.1.0 | registry; 9043b7b23fb0bc4a1c7014c27b50a4fc42cc76206f71d34fc0dfe5b28ddc3faf | bytes, futures-util, headers, http, hyper, hyper-rustls 0.26.0, hyper-tls, hyper-util, native-tls, pin-project-lite, rustls-native-certs 0.7.3, tokio, tokio-native-tls, tokio-rustls 0.25.0, tower-service, webpki, webpki-roots 0.26.11 |
| hyper-rustls@0.26.0 | registry; a0bea761b46ae2b24eb4aef630d8d1c398157b6fc29e6350ecf090a0b70c952c | futures-util, http, hyper, hyper-util, log, rustls 0.22.4, rustls-native-certs 0.7.3, rustls-pki-types, tokio, tokio-rustls 0.25.0, tower-service, webpki-roots 0.26.11 |
| hyper-rustls@0.27.7 | registry; e3c93eb611681b207e1fe55d5a71ecf91572ec8a6705cdb6857f7d8d5242cf58 | http, hyper, hyper-util, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-rustls 0.26.4, tower-service, webpki-roots 1.0.4 |
| hyper-tls@0.6.0 | registry; 70206fc6890eaca9fde8a0bf71caa2ddfc9fe045ac9e5c70df101a7dbde866e0 | bytes, http-body-util, hyper, hyper-util, native-tls, tokio, tokio-native-tls, tower-service |
| hyper-util@0.1.17 | registry; 3c6995591a8f1380fcb4ba966a252a4b29188d51d2b89e3a252f5305be65aea8 | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, hyper, ipnet, libc, percent-encoding, pin-project-lite, socket2, system-configuration, tokio, tower-service, tracing, windows-registry |
| iana-time-zone@0.1.64 | registry; 33e57f83510bb73707521ebaffa789ec8caf86f9657cad665b092b581d40e9fb | android_system_properties, core-foundation-sys, iana-time-zone-haiku, js-sys, log, wasm-bindgen, windows-core 0.62.2 |
| iana-time-zone-haiku@0.1.2 | registry; f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f | cc |
| icu_collections@2.1.1 | registry; 4c6b649701667bbe825c3b7e6388cb521c23d88644678e83c0c4d0a621a34b43 | displaydoc, potential_utf, yoke, zerofrom, zerovec |
| icu_locale_core@2.1.1 | registry; edba7861004dd3714265b4db54a3c390e880ab658fec5f7db895fae2046b5bb6 | displaydoc, litemap, tinystr, writeable, zerovec |
| icu_normalizer@2.1.1 | registry; 5f6c8828b67bf8908d82127b2054ea1b4427ff0230ee9141c54251934ab1b599 | icu_collections, icu_normalizer_data, icu_properties, icu_provider, smallvec, zerovec |
| icu_normalizer_data@2.1.1 | registry; 7aedcccd01fc5fe81e6b489c15b247b8b0690feb23304303a9e560f37efc560a |  |
| icu_properties@2.1.1 | registry; e93fcd3157766c0c8da2f8cff6ce651a31f0810eaa1c51ec363ef790bbb5fb99 | icu_collections, icu_locale_core, icu_properties_data, icu_provider, zerotrie, zerovec |
| icu_properties_data@2.1.1 | registry; 02845b3647bb045f1100ecd6480ff52f34c35f82d9880e029d329c21d1054899 |  |
| icu_provider@2.1.1 | registry; 85962cf0ce02e1e0a629cc34e7ca3e373ce20dda4c4d7294bbd0bf1fdb59e614 | displaydoc, icu_locale_core, writeable, yoke, zerofrom, zerotrie, zerovec |
| ident_case@1.0.1 | registry; b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39 |  |
| idna@1.1.0 | registry; 3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de | idna_adapter, smallvec, utf8_iter |
| idna_adapter@1.2.1 | registry; 3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344 | icu_normalizer, icu_properties |
| indexmap@2.12.0 | registry; 6717a8d2a5a929a1a2eb43a12812498ed141a0bcfb7e8f7844fbdbe4303bba9f | equivalent, hashbrown 0.16.0 |
| inout@0.1.4 | registry; 879f10e63c20629ecabbb64a8010319738c66a5cd0c29b02d63d272b03751d01 | generic-array |
| ipnet@2.11.0 | registry; 469fb0b9cefa57e3ef31275ee7cacb78f2fdca44e4765491884a2b119d4eb130 |  |
| iri-string@0.7.9 | registry; 4f867b9d1d896b67beb18518eda36fdb77a32ea590de864f1325b294a6d14397 | memchr, serde |
| is-docker@0.2.0 | registry; 928bae27f42bc99b60d9ac7334e3a21d10ad8f1835a4e12ec3ec0464765ed1b3 | once_cell |
| is-wsl@0.4.0 | registry; 173609498df190136aa7dea1a91db051746d339e18476eed5ca40521f02d7aa5 | is-docker, once_cell |
| itoa@1.0.15 | registry; 4a5f13b858c8d314ee3e8f639011f7ccefe71f97f96e50151fb991f267928e2c |  |
| js-sys@0.3.82 | registry; b011eec8cc36da2aab2d5cff675ec18454fad408585853910a202391cf9f8e65 | once_cell, wasm-bindgen |
| lazy_static@1.5.0 | registry; bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe | spin |
| libc@0.2.177 | registry; 2874a2af47a2325c2001a6e6fad9b16a53b802102b528163885171cf92b15976 |  |
| libm@0.2.15 | registry; f9fbbcab51052fe104eb5e5d351cf728d30a5be1fe14d9be8a3b097481fb97de |  |
| librespot-audio@0.8.0 | local/path root | aes, bytes, ctr, futures-util, http-body-util, hyper, hyper-util, librespot-core, log, tempfile, thiserror 2.0.17, tokio |
| librespot-core@0.8.0 | registry; 168bbe1c416980ddd9a969ebd6b50fb6c924eb1a3ded194285fa8ec0e2b1c68b | aes, base64, byteorder, bytes, data-encoding, flate2, form_urlencoded, futures-core, futures-util, governor, hmac, http, http-body-util, httparse, hyper, hyper-proxy2, hyper-rustls 0.27.7, hyper-tls, hyper-util, librespot-oauth, librespot-protocol, log, nonzero_ext, num-bigint, num-derive, num-integer, num-traits, pbkdf2, pin-project-lite, priority-queue, protobuf, protobuf-json-mapping, quick-xml, rand 0.9.2, rand_distr, rsa, serde, serde_json, sha1, shannon, sysinfo, thiserror 2.0.17, time, tokio, tokio-stream, tokio-tungstenite, tokio-util, url, uuid, vergen-gitcl |
| librespot-oauth@0.8.0 | registry; d686417d49c9d2c363392ffe28d6e469daca20a82dc414740930e078f5829661 | log, oauth2, open, reqwest, thiserror 2.0.17, url |
| librespot-protocol@0.8.0 | registry; 3e01f0b2d39f83fa162eb91d4a16313bcf99e77daf258abe8f7b7bcb1160b084 | protobuf, protobuf-codegen |
| linux-raw-sys@0.4.15 | registry; d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab |  |
| linux-raw-sys@0.11.0 | registry; df1d3c3b53da64cf5760482273a98e575c651a67eec7f77df96b5b642de8f039 |  |
| litemap@0.8.1 | registry; 6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77 |  |
| lock_api@0.4.14 | registry; 224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965 | scopeguard |
| log@0.4.28 | registry; 34080505efa8e45a4b816c349525ebe327ceaa8559756f0356cba97ef3bf7432 |  |
| lru-slab@0.1.2 | registry; 112b39cec0b298b6c1999fee3e31427f74f676e4cb9879ed1a121b43661a4154 |  |
| memchr@2.7.6 | registry; f52b00d39961fc5b2736ea853c9cc86238e165017a493d1d5c8eac6bdc4cc273 |  |
| mime@0.3.17 | registry; 6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a |  |
| miniz_oxide@0.8.9 | registry; 1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316 | adler2, simd-adler32 |
| mio@1.1.0 | registry; 69d83b0086dc8ecf3ce9ae2874b2d1290252e2a30720bea58a5c6639b0092873 | libc, wasi, windows-sys 0.61.2 |
| native-tls@0.2.14 | registry; 87de3442987e9dbec73158d5c715e7ad9072fda936bb03d19d7fa10e00520f0e | libc, log, openssl, openssl-probe, openssl-sys, schannel, security-framework 2.11.1, security-framework-sys, tempfile |
| nonzero_ext@0.3.0 | registry; 38bf9645c8b145698bb0b18a4637dcacbc421ea49bef2317e4fd8065a387cf21 |  |
| ntapi@0.4.1 | registry; e8a3895c6391c39d7fe7ebc444a87eb2991b2a0bc718fdabd071eec617fc68e4 | winapi |
| num-bigint@0.4.6 | registry; a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9 | num-integer, num-traits |
| num-bigint-dig@0.8.5 | registry; 82c79c15c05d4bf82b6f5ef163104cc81a760d8e874d38ac50ab67c8877b647b | lazy_static, libm, num-integer, num-iter, num-traits, rand 0.8.5, smallvec, zeroize |
| num-conv@0.1.0 | registry; 51d515d32fb182ee37cda2ccdcb92950d6a3c2893aa280e540671c2cd0f3b1d9 |  |
| num-derive@0.4.2 | registry; ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202 | proc-macro2, quote, syn |
| num-integer@0.1.46 | registry; 7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f | num-traits |
| num-iter@0.1.45 | registry; 1429034a0490724d0075ebb2bc9e875d6503c3cf69e235a8941aa757d83ef5bf | autocfg, num-integer, num-traits |
| num-traits@0.2.19 | registry; 071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841 | autocfg, libm |
| num_threads@0.1.7 | registry; 5c7398b9c8b70908f6371f47ed36737907c87c52af34c268fed0bf0ceb92ead9 | libc |
| oauth2@5.0.0 | registry; 51e219e79014df21a225b1860a479e2dcd7cbd9130f4defd4bd0e191ea31d67d | base64, chrono, getrandom 0.2.16, http, rand 0.8.5, reqwest, serde, serde_json, serde_path_to_error, sha2, thiserror 1.0.69, url |
| objc2-core-foundation@0.3.2 | registry; 2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536 | bitflags |
| objc2-io-kit@0.3.2 | registry; 33fafba39597d6dc1fb709123dfa8289d39406734be322956a69f0931c73bb15 | libc, objc2-core-foundation |
| once_cell@1.21.3 | registry; 42f5e15c9953c5e4ccceeb2e7382a716482c34515315f7b03532b8b4e8393d2d |  |
| open@5.3.2 | registry; e2483562e62ea94312f3576a7aca397306df7990b8d89033e18766744377ef95 | is-wsl, libc, pathdiff |
| openssl@0.10.75 | registry; 08838db121398ad17ab8531ce9de97b244589089e290a384c900cb9ff7434328 | bitflags, cfg-if, foreign-types, libc, once_cell, openssl-macros, openssl-sys |
| openssl-macros@0.1.1 | registry; a948666b637a0f465e8564c73e89d4dde00d72d4d473cc972f390fc3dcee7d9c | proc-macro2, quote, syn |
| openssl-probe@0.1.6 | registry; d05e27ee213611ffe7d6348b942e8f942b37114c00cc03cec254295a4a17852e |  |
| openssl-sys@0.9.111 | registry; 82cab2d520aa75e3c58898289429321eb788c3106963d0dc886ec7a5f4adc321 | cc, libc, pkg-config, vcpkg |
| parking_lot@0.12.5 | registry; 93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a | lock_api, parking_lot_core |
| parking_lot_core@0.9.12 | registry; 2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1 | cfg-if, libc, redox_syscall, smallvec, windows-link 0.2.1 |
| pathdiff@0.2.3 | registry; df94ce210e5bc13cb6651479fa48d14f601d9858cfe0467f43ae157023b938d3 |  |
| pbkdf2@0.12.2 | registry; f8ed6a7761f76e3b9f92dfb0a60a6a6477c61024b775147ff0973a02653abaf2 | digest, hmac |
| pem-rfc7468@0.7.0 | registry; 88b39c9bfcfc231068454382784bb460aae594343fb030d46e9f50a645418412 | base64ct |
| percent-encoding@2.3.2 | registry; 9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220 |  |
| pin-project-lite@0.2.16 | registry; 3b3cff922bd51709b605d9ead9aa71031d81447142d828eb4a6eba76fe619f9b |  |
| pin-utils@0.1.0 | registry; 8b870d8c151b6f2fb93e84a13146138f05d02ed11c7e7c54f8826aaaf7c9f184 |  |
| pkcs1@0.7.5 | registry; c8ffb9f10fa047879315e6625af03c164b16962a5368d724ed16323b68ace47f | der, pkcs8, spki |
| pkcs8@0.10.2 | registry; f950b2377845cebe5cf8b5165cb3cc1a5e0fa5cfa3e1f7f55707d8fd82e0a7b7 | der, spki |
| pkg-config@0.3.32 | registry; 7edddbd0b52d732b21ad9a5fab5c704c14cd949e5e9a1ec5929a24fded1b904c |  |
| portable-atomic@1.11.1 | registry; f84267b20a16ea918e43c6a88433c2d54fa145c92a811b5b047ccbe153674483 |  |
| potential_utf@0.1.4 | registry; b73949432f5e2a09657003c25bca5e19a0e9c84f8058ca374f49e0ebe605af77 | zerovec |
| powerfmt@0.2.0 | registry; 439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391 |  |
| ppv-lite86@0.2.21 | registry; 85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9 | zerocopy |
| priority-queue@2.7.0 | registry; 93980406f12d9f8140ed5abe7155acb10bb1e69ea55c88960b9c2f117445ef96 | equivalent, indexmap, serde |
| proc-macro2@1.0.103 | registry; 5ee95bc4ef87b8d5ba32e8b7714ccc834865276eab0aed5c9958d00ec45f49e8 | unicode-ident |
| protobuf@3.7.2 | registry; d65a1d4ddae7d8b5de68153b48f6aa3bba8cb002b243dbdbc55a5afbc98f99f4 | once_cell, protobuf-support, thiserror 1.0.69 |
| protobuf-codegen@3.7.2 | registry; 5d3976825c0014bbd2f3b34f0001876604fe87e0c86cd8fa54251530f1544ace | anyhow, once_cell, protobuf, protobuf-parse, regex, tempfile, thiserror 1.0.69 |
| protobuf-json-mapping@3.7.2 | registry; e0d6e4be637b310d8a5c02fa195243328e2d97fa7df1127a27281ef1187fcb1d | protobuf, protobuf-support, thiserror 1.0.69 |
| protobuf-parse@3.7.2 | registry; b4aeaa1f2460f1d348eeaeed86aea999ce98c1bded6f089ff8514c9d9dbdc973 | anyhow, indexmap, log, protobuf, protobuf-support, tempfile, thiserror 1.0.69, which |
| protobuf-support@3.7.2 | registry; 3e36c2f31e0a47f9280fb347ef5e461ffcd2c52dd520d8e216b52f93b0b0d7d6 | thiserror 1.0.69 |
| quick-xml@0.38.3 | registry; 42a232e7487fc2ef313d96dde7948e7a3c05101870d8985e4fd8d26aedd27b89 | memchr, serde |
| quinn@0.11.9 | registry; b9e20a958963c291dc322d98411f541009df2ced7b5a4f2bd52337638cfccf20 | bytes, cfg_aliases, pin-project-lite, quinn-proto, quinn-udp, rustc-hash, rustls 0.23.35, socket2, thiserror 2.0.17, tokio, tracing, web-time |
| quinn-proto@0.11.13 | registry; f1906b49b0c3bc04b5fe5d86a77925ae6524a19b816ae38ce1e426255f1d8a31 | bytes, getrandom 0.3.4, lru-slab, rand 0.9.2, ring, rustc-hash, rustls 0.23.35, rustls-pki-types, slab, thiserror 2.0.17, tinyvec, tracing, web-time |
| quinn-udp@0.5.14 | registry; addec6a0dcad8a8d96a771f815f0eaf55f9d1805756410b39f5fa81332574cbd | cfg_aliases, libc, once_cell, socket2, tracing, windows-sys 0.60.2 |
| quote@1.0.42 | registry; a338cc41d27e6cc6dce6cefc13a0729dfbb81c262b1f519331575dd80ef3067f | proc-macro2 |
| r-efi@5.3.0 | registry; 69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f |  |
| rand@0.8.5 | registry; 34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404 | libc, rand_chacha 0.3.1, rand_core 0.6.4 |
| rand@0.9.2 | registry; 6db2770f06117d490610c7488547d543617b21bfa07796d7a12f6f1bd53850d1 | rand_chacha 0.9.0, rand_core 0.9.3 |
| rand_chacha@0.3.1 | registry; e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88 | ppv-lite86, rand_core 0.6.4 |
| rand_chacha@0.9.0 | registry; d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb | ppv-lite86, rand_core 0.9.3 |
| rand_core@0.6.4 | registry; ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c | getrandom 0.2.16 |
| rand_core@0.9.3 | registry; 99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38 | getrandom 0.3.4 |
| rand_distr@0.5.1 | registry; 6a8615d50dcf34fa31f7ab52692afec947c4dd0ab803cc87cb3b0b4570ff7463 | num-traits, rand 0.9.2 |
| redox_syscall@0.5.18 | registry; ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d | bitflags |
| regex@1.12.2 | registry; 843bc0191f75f3e22651ae5f1e72939ab2f72a4bc30fa80a066bd66edefc24d4 | aho-corasick, memchr, regex-automata, regex-syntax |
| regex-automata@0.4.13 | registry; 5276caf25ac86c8d810222b3dbb938e512c55c6831a10f3e6ed1c93b84041f1c | aho-corasick, memchr, regex-syntax |
| regex-syntax@0.8.8 | registry; 7a2d987857b319362043e95f5353c0535c1f58eec5336fdfcf626430af7def58 |  |
| reqwest@0.12.24 | registry; 9d0946410b9f7b082a427e4ef5c8ff541a88b357bc6c637c40db3a68ac70a36f | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, http-body-util, hyper, hyper-rustls 0.27.7, hyper-tls, hyper-util, js-sys, log, native-tls, percent-encoding, pin-project-lite, quinn, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, serde, serde_json, serde_urlencoded, sync_wrapper, tokio, tokio-native-tls, tokio-rustls 0.26.4, tower, tower-http, tower-service, url, wasm-bindgen, wasm-bindgen-futures, web-sys, webpki-roots 1.0.4 |
| ring@0.17.14 | registry; a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7 | cc, cfg-if, getrandom 0.2.16, libc, untrusted, windows-sys 0.52.0 |
| rsa@0.9.8 | registry; 78928ac1ed176a5ca1d17e578a1825f3d81ca54cf41053a592584b020cfd691b | const-oid, digest, num-bigint-dig, num-integer, num-traits, pkcs1, pkcs8, rand_core 0.6.4, signature, spki, subtle, zeroize |
| rustc-hash@2.1.1 | registry; 357703d41365b4b27c590e3ed91eabb1b663f07c4c084095e60cbed4362dff0d |  |
| rustix@0.38.44 | registry; fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154 | bitflags, errno, libc, linux-raw-sys 0.4.15, windows-sys 0.59.0 |
| rustix@1.1.2 | registry; cd15f8a2c5551a84d56efdc1cd049089e409ac19a3072d5037a17fd70719ff3e | bitflags, errno, libc, linux-raw-sys 0.11.0, windows-sys 0.61.2 |
| rustls@0.22.4 | registry; bf4ef73721ac7bcd79b2b315da7779d8fc09718c6b3d2d1b2d94850eb8c18432 | log, ring, rustls-pki-types, rustls-webpki 0.102.8, subtle, zeroize |
| rustls@0.23.35 | registry; 533f54bc6a7d4f647e46ad909549eda97bf5afc1585190ef692b4286b198bd8f | once_cell, ring, rustls-pki-types, rustls-webpki 0.103.8, subtle, zeroize |
| rustls-native-certs@0.7.3 | registry; e5bfb394eeed242e909609f56089eecfe5fda225042e8b171791b9c95f5931e5 | openssl-probe, rustls-pemfile, rustls-pki-types, schannel, security-framework 2.11.1 |
| rustls-native-certs@0.8.2 | registry; 9980d917ebb0c0536119ba501e90834767bffc3d60641457fd84a1f3fd337923 | openssl-probe, rustls-pki-types, schannel, security-framework 3.5.1 |
| rustls-pemfile@2.2.0 | registry; dce314e5fee3f39953d46bb63bb8a46d40c2f8fb7cc5a3b6cab2bde9721d6e50 | rustls-pki-types |
| rustls-pki-types@1.13.0 | registry; 94182ad936a0c91c324cd46c6511b9510ed16af436d7b5bab34beab0afd55f7a | web-time, zeroize |
| rustls-webpki@0.102.8 | registry; 64ca1bc8749bd4cf37b5ce386cc146580777b4e8572c7b97baf22c83f444bee9 | ring, rustls-pki-types, untrusted |
| rustls-webpki@0.103.8 | registry; 2ffdfa2f5286e2247234e03f680868ac2815974dc39e00ea15adc445d0aafe52 | ring, rustls-pki-types, untrusted |
| rustversion@1.0.22 | registry; b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d |  |
| ryu@1.0.20 | registry; 28d3b2b1366ec20994f1fd18c3c594f05c5dd4bc44d8bb0c1c632c8d6829481f |  |
| schannel@0.1.28 | registry; 891d81b926048e76efe18581bf793546b4c0eaf8448d72be8de2bbee5fd166e1 | windows-sys 0.61.2 |
| scopeguard@1.2.0 | registry; 94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49 |  |
| security-framework@2.11.1 | registry; 897b2245f0b511c87893af39b033e5ca9cce68824c4d7e7630b5a1d339658d02 | bitflags, core-foundation 0.9.4, core-foundation-sys, libc, security-framework-sys |
| security-framework@3.5.1 | registry; b3297343eaf830f66ede390ea39da1d462b6b0c1b000f420d0a83f898bbbe6ef | bitflags, core-foundation 0.10.1, core-foundation-sys, libc, security-framework-sys |
| security-framework-sys@2.15.0 | registry; cc1f0cbffaac4852523ce30d8bd3c5cdc873501d96ff467ca09b6767bb8cd5c0 | core-foundation-sys, libc |
| serde@1.0.228 | registry; 9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e | serde_core, serde_derive |
| serde_core@1.0.228 | registry; 41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad | serde_derive |
| serde_derive@1.0.228 | registry; d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79 | proc-macro2, quote, syn |
| serde_json@1.0.145 | registry; 402a6f66d8c709116cf22f558eab210f5a50187f702eb4d7e5ef38d9a7f1c79c | itoa, memchr, ryu, serde, serde_core |
| serde_path_to_error@0.1.20 | registry; 10a9ff822e371bb5403e391ecd83e182e0e77ba7f6fe0160b795797109d1b457 | itoa, serde, serde_core |
| serde_urlencoded@0.7.1 | registry; d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd | form_urlencoded, itoa, ryu, serde |
| sha1@0.10.6 | registry; e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba | cfg-if, cpufeatures, digest |
| sha2@0.10.9 | registry; a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283 | cfg-if, cpufeatures, digest |
| shannon@0.2.0 | registry; 7ea5b41c9427b56caa7b808cb548a04fb50bb5b9e98590b53f28064ff4174561 | byteorder |
| shlex@1.3.0 | registry; 0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64 |  |
| signature@2.2.0 | registry; 77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de | digest, rand_core 0.6.4 |
| simd-adler32@0.3.7 | registry; d66dc143e6b11c1eddc06d5c423cfc97062865baf299914ab64caa38182078fe |  |
| slab@0.4.11 | registry; 7a2ae44ef20feb57a68b23d846850f861394c2e02dc425a50098ae8c90267589 |  |
| smallvec@1.15.1 | registry; 67b1b7a3b5fe4f1376887184045fcf45c69e92af734b7aaddc05fb777b6fbd03 |  |
| socket2@0.6.1 | registry; 17129e116933cf371d018bb80ae557e889637989d8638274fb25622827b03881 | libc, windows-sys 0.60.2 |
| spin@0.9.8 | registry; 6980e8d7511241f8acf4aebddbb1ff938df5eebe98691418c4468d0b72a96a67 |  |
| spinning_top@0.3.0 | registry; d96d2d1d716fb500937168cc09353ffdc7a012be8475ac7308e1bdf0e3923300 | lock_api |
| spki@0.7.3 | registry; d91ed6c858b01f942cd56b37a94b3e0a1798290327d1236e4d9cf4eaca44d29d | base64ct, der |
| stable_deref_trait@1.2.1 | registry; 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |  |
| strsim@0.11.1 | registry; 7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f |  |
| subtle@2.6.1 | registry; 13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292 |  |
| syn@2.0.109 | registry; 2f17c7e013e88258aa9543dcbe81aca68a667a9ac37cd69c9fbc07858bfe0e2f | proc-macro2, quote, unicode-ident |
| sync_wrapper@1.0.2 | registry; 0bf256ce5efdfa370213c1dabab5935a12e49f2c58d15e9eac2870d3b4f27263 | futures-core |
| synstructure@0.13.2 | registry; 728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2 | proc-macro2, quote, syn |
| sysinfo@0.36.1 | registry; 252800745060e7b9ffb7b2badbd8b31cfa4aa2e61af879d0a3bf2a317c20217d | libc, memchr, ntapi, objc2-core-foundation, objc2-io-kit, windows |
| system-configuration@0.6.1 | registry; 3c879d448e9d986b661742763247d3693ed13609438cf3d006f51f5368a5ba6b | bitflags, core-foundation 0.9.4, system-configuration-sys |
| system-configuration-sys@0.6.0 | registry; 8e1d1b10ced5ca923a1fcb8d03e96b8d3268065d724548c0211415ff6ac6bac4 | core-foundation-sys, libc |
| tempfile@3.23.0 | registry; 2d31c77bdf42a745371d260a26ca7163f1e0924b64afa0b688e61b5a9fa02f16 | fastrand, getrandom 0.3.4, once_cell, rustix 1.1.2, windows-sys 0.61.2 |
| thiserror@1.0.69 | registry; b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52 | thiserror-impl 1.0.69 |
| thiserror@2.0.17 | registry; f63587ca0f12b72a0600bcba1d40081f830876000bb46dd2337a3051618f4fc8 | thiserror-impl 2.0.17 |
| thiserror-impl@1.0.69 | registry; 4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1 | proc-macro2, quote, syn |
| thiserror-impl@2.0.17 | registry; 3ff15c8ecd7de3849db632e14d18d2571fa09dfc5ed93479bc4485c7a517c913 | proc-macro2, quote, syn |
| time@0.3.44 | registry; 91e7d9e3bb61134e77bde20dd4825b97c010155709965fedf0f49bb138e52a9d | deranged, itoa, libc, num-conv, num_threads, powerfmt, serde, time-core, time-macros |
| time-core@0.1.6 | registry; 40868e7c1d2f0b8d73e4a8c7f0ff63af4f6d19be117e90bd73eb1d62cf831c6b |  |
| time-macros@0.2.24 | registry; 30cfb0125f12d9c277f35663a0a33f8c30190f4e4574868a330595412d34ebf3 | num-conv, time-core |
| tinystr@0.8.2 | registry; 42d3e9c45c09de15d06dd8acf5f4e0e399e85927b7f00711024eb7ae10fa4869 | displaydoc, zerovec |
| tinyvec@1.10.0 | registry; bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa | tinyvec_macros |
| tinyvec_macros@0.1.1 | registry; 1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20 |  |
| tokio@1.48.0 | registry; ff360e02eab121e0bc37a2d3b4d4dc622e6eda3a8e5253d5435ecf5bd4c68408 | bytes, libc, mio, pin-project-lite, socket2, tokio-macros, windows-sys 0.61.2 |
| tokio-macros@2.6.0 | registry; af407857209536a95c8e56f8231ef2c2e2aff839b22e07a1ffcbc617e9db9fa5 | proc-macro2, quote, syn |
| tokio-native-tls@0.3.1 | registry; bbae76ab933c85776efabc971569dd6119c580d8f5d448769dec1764bf796ef2 | native-tls, tokio |
| tokio-rustls@0.25.0 | registry; 775e0c0f0adb3a2f22a00c4745d728b479985fc15ee7ca6a2608388c5569860f | rustls 0.22.4, rustls-pki-types, tokio |
| tokio-rustls@0.26.4 | registry; 1729aa945f29d91ba541258c8df89027d5792d85a8841fb65e8bf0f4ede4ef61 | rustls 0.23.35, tokio |
| tokio-stream@0.1.17 | registry; eca58d7bba4a75707817a2c44174253f9236b2d5fbd055602e9d5c07c139a047 | futures-core, pin-project-lite, tokio |
| tokio-tungstenite@0.28.0 | registry; d25a406cddcc431a75d3d9afc6a7c0f7428d4891dd973e4d54c56b46127bf857 | futures-util, log, native-tls, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-native-tls, tokio-rustls 0.26.4, tungstenite, webpki-roots 0.26.11 |
| tokio-util@0.7.17 | registry; 2efa149fe76073d6e8fd97ef4f4eca7b67f599660115591483572e406e165594 | bytes, futures-core, futures-sink, pin-project-lite, tokio |
| tower@0.5.2 | registry; d039ad9159c98b70ecfd540b2573b97f7f52c3e8d9f8ad57a24b916a536975f9 | futures-core, futures-util, pin-project-lite, sync_wrapper, tokio, tower-layer, tower-service |
| tower-http@0.6.6 | registry; adc82fd73de2a9722ac5da747f12383d2bfdb93591ee6c58486e0097890f05f2 | bitflags, bytes, futures-util, http, http-body, iri-string, pin-project-lite, tower, tower-layer, tower-service |
| tower-layer@0.3.3 | registry; 121c2a6cda46980bb0fcd1647ffaf6cd3fc79a013de288782836f6df9c48780e |  |
| tower-service@0.3.3 | registry; 8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3 |  |
| tracing@0.1.41 | registry; 784e0ac535deb450455cbfa28a6f0df145ea1bb7ae51b821cf5e7927fdcfbdd0 | pin-project-lite, tracing-core |
| tracing-core@0.1.34 | registry; b9d12581f227e93f094d3af2ae690a574abb8a2b9b7a96e7cfe9647b2b617678 | once_cell |
| try-lock@0.2.5 | registry; e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b |  |
| tungstenite@0.28.0 | registry; 8628dcc84e5a09eb3d8423d6cb682965dea9133204e8fb3efee74c2a0c259442 | bytes, data-encoding, http, httparse, log, native-tls, rand 0.9.2, rustls 0.23.35, rustls-pki-types, sha1, thiserror 2.0.17, utf-8 |
| typenum@1.19.0 | registry; 562d481066bde0658276a35467c4af00bdc6ee726305698a55b86e61d7ad82bb |  |
| unicode-ident@1.0.22 | registry; 9312f7c4f6ff9069b165498234ce8be658059c6728633667c526e27dc2cf1df5 |  |
| untrusted@0.9.0 | registry; 8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1 |  |
| url@2.5.7 | registry; 08bc136a29a3d1758e07a9cca267be308aeebf5cfd5a10f3f67ab2097683ef5b | form_urlencoded, idna, percent-encoding, serde |
| utf-8@0.7.6 | registry; 09cc8ee72d2a9becf2f2febe0205bbed8fc6615b7cb429ad062dc7b7ddd036a9 |  |
| utf8_iter@1.0.4 | registry; b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be |  |
| uuid@1.18.1 | registry; 2f87b8aa10b915a06587d0dec516c282ff295b475d94abf425d62b57710070a2 | getrandom 0.3.4 |
| vcpkg@0.2.15 | registry; accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426 |  |
| vergen@9.0.6 | registry; 6b2bf58be11fc9414104c6d3a2e464163db5ef74b12296bda593cac37b6e4777 | anyhow, derive_builder, rustversion, time, vergen-lib |
| vergen-gitcl@1.0.8 | registry; b9dfc1de6eb2e08a4ddf152f1b179529638bedc0ea95e6d667c014506377aefe | anyhow, derive_builder, rustversion, time, vergen, vergen-lib |
| vergen-lib@0.1.6 | registry; 9b07e6010c0f3e59fcb164e0163834597da68d1f864e2b8ca49f74de01e9c166 | anyhow, derive_builder, rustversion |
| version_check@0.9.5 | registry; 0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a |  |
| want@0.3.1 | registry; bfa7760aed19e106de2c7c0b581b509f2f25d3dacaf737cb82ac61bc6d760b0e | try-lock |
| wasi@0.11.1+wasi-snapshot-preview1 | registry; ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b |  |
| wasip2@1.0.1+wasi-0.2.4 | registry; 0562428422c63773dad2c345a1882263bbf4d65cf3f42e90921f787ef5ad58e7 | wit-bindgen |
| wasm-bindgen@0.2.105 | registry; da95793dfc411fbbd93f5be7715b0578ec61fe87cb1a42b12eb625caa5c5ea60 | cfg-if, once_cell, rustversion, wasm-bindgen-macro, wasm-bindgen-shared |
| wasm-bindgen-futures@0.4.55 | registry; 551f88106c6d5e7ccc7cd9a16f312dd3b5d36ea8b4954304657d5dfba115d4a0 | cfg-if, js-sys, once_cell, wasm-bindgen, web-sys |
| wasm-bindgen-macro@0.2.105 | registry; 04264334509e04a7bf8690f2384ef5265f05143a4bff3889ab7a3269adab59c2 | quote, wasm-bindgen-macro-support |
| wasm-bindgen-macro-support@0.2.105 | registry; 420bc339d9f322e562942d52e115d57e950d12d88983a14c79b86859ee6c7ebc | bumpalo, proc-macro2, quote, syn, wasm-bindgen-shared |
| wasm-bindgen-shared@0.2.105 | registry; 76f218a38c84bcb33c25ec7059b07847d465ce0e0a76b995e134a45adcb6af76 | unicode-ident |
| web-sys@0.3.82 | registry; 3a1f95c0d03a47f4ae1f7a64643a6bb97465d9b740f0fa8f90ea33915c99a9a1 | js-sys, wasm-bindgen |
| web-time@1.1.0 | registry; 5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb | js-sys, wasm-bindgen |
| webpki@0.22.4 | registry; ed63aea5ce73d0ff405984102c42de94fc55a6b75765d621c65262469b3c9b53 | ring, untrusted |
| webpki-roots@0.26.11 | registry; 521bc38abb08001b01866da9f51eb7c5d647a19260e00054a8c7fd5f9e57f7a9 | webpki-roots 1.0.4 |
| webpki-roots@1.0.4 | registry; b2878ef029c47c6e8cf779119f20fcf52bde7ad42a731b2a304bc221df17571e | rustls-pki-types |
| which@4.4.2 | registry; 87ba24419a2078cd2b0f2ede2691b6c66d8e47836da3b6db8265ebad47afbfc7 | either, home, once_cell, rustix 0.38.44 |
| winapi@0.3.9 | registry; 5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419 | winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu |
| winapi-i686-pc-windows-gnu@0.4.0 | registry; ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6 |  |
| winapi-x86_64-pc-windows-gnu@0.4.0 | registry; 712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f |  |
| windows@0.61.3 | registry; 9babd3a767a4c1aef6900409f85f5d53ce2544ccdfaa86dad48c91782c6d6893 | windows-collections, windows-core 0.61.2, windows-future, windows-link 0.1.3, windows-numerics |
| windows-collections@0.2.0 | registry; 3beeceb5e5cfd9eb1d76b381630e82c4241ccd0d27f1a39ed41b2760b255c5e8 | windows-core 0.61.2 |
| windows-core@0.61.2 | registry; c0fdd3ddb90610c7638aa2b3a3ab2904fb9e5cdbecc643ddb3647212781c4ae3 | windows-implement, windows-interface, windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-core@0.62.2 | registry; b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb | windows-implement, windows-interface, windows-link 0.2.1, windows-result 0.4.1, windows-strings 0.5.1 |
| windows-future@0.2.1 | registry; fc6a41e98427b19fe4b73c550f060b59fa592d7d686537eebf9385621bfbad8e | windows-core 0.61.2, windows-link 0.1.3, windows-threading |
| windows-implement@0.60.2 | registry; 053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf | proc-macro2, quote, syn |
| windows-interface@0.59.3 | registry; 3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358 | proc-macro2, quote, syn |
| windows-link@0.1.3 | registry; 5e6ad25900d524eaabdbbb96d20b4311e1e7ae1699af4fb28c17ae66c80d798a |  |
| windows-link@0.2.1 | registry; f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5 |  |
| windows-numerics@0.2.0 | registry; 9150af68066c4c5c07ddc0ce30421554771e528bde427614c61038bc2c92c2b1 | windows-core 0.61.2, windows-link 0.1.3 |
| windows-registry@0.5.3 | registry; 5b8a9ed28765efc97bbc954883f4e6796c33a06546ebafacbabee9696967499e | windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-result@0.3.4 | registry; 56f42bd332cc6c8eac5af113fc0c1fd6a8fd2aa08a0119358686e5160d0586c6 | windows-link 0.1.3 |
| windows-result@0.4.1 | registry; 7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5 | windows-link 0.2.1 |
| windows-strings@0.4.2 | registry; 56e6c93f3a0c3b36176cb1327a4958a0353d5d166c2a35cb268ace15e91d3b57 | windows-link 0.1.3 |
| windows-strings@0.5.1 | registry; 7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091 | windows-link 0.2.1 |
| windows-sys@0.52.0 | registry; 282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d | windows-targets 0.52.6 |
| windows-sys@0.59.0 | registry; 1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b | windows-targets 0.52.6 |
| windows-sys@0.60.2 | registry; f2f500e4d28234f72040990ec9d39e3a6b950f9f22d3dba18416c35882612bcb | windows-targets 0.53.5 |
| windows-sys@0.61.2 | registry; ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc | windows-link 0.2.1 |
| windows-targets@0.52.6 | registry; 9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973 | windows_aarch64_gnullvm 0.52.6, windows_aarch64_msvc 0.52.6, windows_i686_gnu 0.52.6, windows_i686_gnullvm 0.52.6, windows_i686_msvc 0.52.6, windows_x86_64_gnu 0.52.6, windows_x86_64_gnullvm 0.52.6, windows_x86_64_msvc 0.52.6 |
| windows-targets@0.53.5 | registry; 4945f9f551b88e0d65f3db0bc25c33b8acea4d9e41163edf90dcd0b19f9069f3 | windows-link 0.2.1, windows_aarch64_gnullvm 0.53.1, windows_aarch64_msvc 0.53.1, windows_i686_gnu 0.53.1, windows_i686_gnullvm 0.53.1, windows_i686_msvc 0.53.1, windows_x86_64_gnu 0.53.1, windows_x86_64_gnullvm 0.53.1, windows_x86_64_msvc 0.53.1 |
| windows-threading@0.1.0 | registry; b66463ad2e0ea3bbf808b7f1d371311c80e115c0b71d60efc142cafbcfb057a6 | windows-link 0.1.3 |
| windows_aarch64_gnullvm@0.52.6 | registry; 32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3 |  |
| windows_aarch64_gnullvm@0.53.1 | registry; a9d8416fa8b42f5c947f8482c43e7d89e73a173cead56d044f6a56104a6d1b53 |  |
| windows_aarch64_msvc@0.52.6 | registry; 09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469 |  |
| windows_aarch64_msvc@0.53.1 | registry; b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006 |  |
| windows_i686_gnu@0.52.6 | registry; 8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b |  |
| windows_i686_gnu@0.53.1 | registry; 960e6da069d81e09becb0ca57a65220ddff016ff2d6af6a223cf372a506593a3 |  |
| windows_i686_gnullvm@0.52.6 | registry; 0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66 |  |
| windows_i686_gnullvm@0.53.1 | registry; fa7359d10048f68ab8b09fa71c3daccfb0e9b559aed648a8f95469c27057180c |  |
| windows_i686_msvc@0.52.6 | registry; 240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66 |  |
| windows_i686_msvc@0.53.1 | registry; 1e7ac75179f18232fe9c285163565a57ef8d3c89254a30685b57d83a38d326c2 |  |
| windows_x86_64_gnu@0.52.6 | registry; 147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78 |  |
| windows_x86_64_gnu@0.53.1 | registry; 9c3842cdd74a865a8066ab39c8a7a473c0778a3f29370b5fd6b4b9aa7df4a499 |  |
| windows_x86_64_gnullvm@0.52.6 | registry; 24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d |  |
| windows_x86_64_gnullvm@0.53.1 | registry; 0ffa179e2d07eee8ad8f57493436566c7cc30ac536a3379fdf008f47f6bb7ae1 |  |
| windows_x86_64_msvc@0.52.6 | registry; 589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec |  |
| windows_x86_64_msvc@0.53.1 | registry; d6bbff5f0aada427a1e5a6da5f1f98158182f26556f345ac9e04d36d0ebed650 |  |
| wit-bindgen@0.46.0 | registry; f17a85883d4e6d00e8a97c586de764dabcc06133f7f1d55dce5cdc070ad7fe59 |  |
| writeable@0.6.2 | registry; 9edde0db4769d2dc68579893f2306b26c6ecfbe0ef499b013d731b7b9247e0b9 |  |
| yoke@0.8.1 | registry; 72d6e5c6afb84d73944e5cedb052c4680d5657337201555f9f2a16b7406d4954 | stable_deref_trait, yoke-derive, zerofrom |
| yoke-derive@0.8.1 | registry; b659052874eb698efe5b9e8cf382204678a0086ebf46982b79d6ca3182927e5d | proc-macro2, quote, syn, synstructure |
| zerocopy@0.8.27 | registry; 0894878a5fa3edfd6da3f88c4805f4c8558e2b996227a3d864f47fe11e38282c | zerocopy-derive |
| zerocopy-derive@0.8.27 | registry; 88d2b8d9c68ad2b9e4340d7832716a4d21a22a1154777ad56ea55c51a9cf3831 | proc-macro2, quote, syn |
| zerofrom@0.1.6 | registry; 50cc42e0333e05660c3587f3bf9d0478688e15d870fab3346451ce7f8c9fbea5 | zerofrom-derive |
| zerofrom-derive@0.1.6 | registry; d71e5d6e06ab090c67b5e44993ec16b72dcbaabc526db883a360057678b48502 | proc-macro2, quote, syn, synstructure |
| zeroize@1.8.2 | registry; b97154e67e32c85465826e8bcc1c59429aaaf107c1e4a9e53c8d8ccd5eff88d0 |  |
| zerotrie@0.2.3 | registry; 2a59c17a5562d507e4b54960e8569ebee33bee890c70aa3fe7b97e85a9fd7851 | displaydoc, yoke, zerofrom |
| zerovec@0.11.5 | registry; 6c28719294829477f525be0186d13efa9a3c602f7ec202ca9e353d310fb9a002 | yoke, zerofrom, zerovec-derive |
| zerovec-derive@0.11.2 | registry; eadce39539ca5cb3985590102671f2567e659fca9666581ad3411d59207951f3 | proc-macro2, quote, syn |

## src-tauri/vendor/librespot-connect/Cargo.lock

{"path": "src-tauri/vendor/librespot-connect/Cargo.lock", "packages": 346, "dependency_edges": 955, "duplicate_names": 32, "unresolved_or_ambiguous_edges": [], "nonregistry_packages": ["librespot-connect@0.8.0"]}

### Multiple versions

{"bitflags": ["1.3.2", "2.10.0"], "core-foundation": ["0.9.4", "0.10.1"], "getrandom": ["0.2.16", "0.3.4"], "hashbrown": ["0.15.5", "0.16.0"], "hyper-rustls": ["0.26.0", "0.27.7"], "linux-raw-sys": ["0.4.15", "0.11.0"], "rand": ["0.8.5", "0.9.2"], "rand_chacha": ["0.3.1", "0.9.0"], "rand_core": ["0.6.4", "0.9.3"], "rustix": ["0.38.44", "1.1.2"], "rustls": ["0.22.4", "0.23.35"], "rustls-native-certs": ["0.7.3", "0.8.2"], "rustls-webpki": ["0.102.8", "0.103.8"], "security-framework": ["2.11.1", "3.5.1"], "thiserror": ["1.0.69", "2.0.17"], "thiserror-impl": ["1.0.69", "2.0.17"], "tokio-rustls": ["0.25.0", "0.26.4"], "webpki-roots": ["0.26.11", "1.0.4"], "windows-core": ["0.61.2", "0.62.2"], "windows-link": ["0.1.3", "0.2.1"], "windows-result": ["0.3.4", "0.4.1"], "windows-strings": ["0.4.2", "0.5.1"], "windows-sys": ["0.52.0", "0.59.0", "0.60.2", "0.61.2"], "windows-targets": ["0.52.6", "0.53.5"], "windows_aarch64_gnullvm": ["0.52.6", "0.53.1"], "windows_aarch64_msvc": ["0.52.6", "0.53.1"], "windows_i686_gnu": ["0.52.6", "0.53.1"], "windows_i686_gnullvm": ["0.52.6", "0.53.1"], "windows_i686_msvc": ["0.52.6", "0.53.1"], "windows_x86_64_gnu": ["0.52.6", "0.53.1"], "windows_x86_64_gnullvm": ["0.52.6", "0.53.1"], "windows_x86_64_msvc": ["0.52.6", "0.53.1"]}

### Every resolved package and edge

| Package | Source/checksum | Dependencies |
| --- | --- | --- |
| adler2@2.0.1 | registry; 320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa |  |
| aes@0.8.4 | registry; b169f7a6d4742236a0a00c541b845991d0ac43e546831af1249753ab4c3aa3a0 | cfg-if, cipher, cpufeatures |
| aho-corasick@1.1.4 | registry; ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301 | memchr |
| allocator-api2@0.2.21 | registry; 683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923 |  |
| android_system_properties@0.1.5 | registry; 819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311 | libc |
| anyhow@1.0.100 | registry; a23eb6b1614318a8071c9b2521f36b424b2c83db5eb3a0fead4a6c0809af6e61 |  |
| arrayvec@0.7.6 | registry; 7c02d123df017efcdfbd739ef81735b36c5ba83ec3c59c80a9d7ecc718f92e50 |  |
| async-trait@0.1.89 | registry; 9035ad2d096bed7955a320ee7e2230574d28fd3c3a0f186cbea1ff3c7eed5dbb | proc-macro2, quote, syn |
| atomic-waker@1.1.2 | registry; 1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0 |  |
| autocfg@1.5.0 | registry; c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8 |  |
| base64@0.22.1 | registry; 72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6 |  |
| base64ct@1.8.0 | registry; 55248b47b0caf0546f7988906588779981c43bb1bc9d0c44087278f80cdb44ba |  |
| bitflags@1.3.2 | registry; bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a |  |
| bitflags@2.10.0 | registry; 812e12b5285cc515a9c72a5c1d3b6d46a19dac5acfef5265968c166106e31dd3 |  |
| block-buffer@0.10.4 | registry; 3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71 | generic-array |
| bumpalo@3.19.0 | registry; 46c5e41b57b8bba42a04676d81cb89e9ee8e859a1a66f80a5a72e1cb76b34d43 |  |
| bytemuck@1.24.0 | registry; 1fbdf580320f38b612e485521afda1ee26d10cc9884efaaa750d383e13e3c5f4 |  |
| byteorder@1.5.0 | registry; 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |  |
| bytes@1.10.1 | registry; d71b6127be86fdcfddb610f7182ac57211d4b18a3e9c82eb2d17662f2227ad6a |  |
| cc@1.2.45 | registry; 35900b6c8d709fb1d854671ae27aeaa9eec2f8b01b364e1619a40da3e6fe2afe | find-msvc-tools, shlex |
| cfg-if@1.0.4 | registry; 9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801 |  |
| cfg_aliases@0.2.1 | registry; 613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724 |  |
| chrono@0.4.42 | registry; 145052bdd345b87320e369255277e3fb5152762ad123a901ef5c262dd38fe8d2 | iana-time-zone, js-sys, num-traits, serde, wasm-bindgen, windows-link 0.2.1 |
| cipher@0.4.4 | registry; 773f3b9af64447d2ce9850330c473515014aa235e6a783b02db81ff39e4a3dad | crypto-common, inout |
| const-oid@0.9.6 | registry; c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8 |  |
| core-foundation@0.9.4 | registry; 91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f | core-foundation-sys, libc |
| core-foundation@0.10.1 | registry; b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6 | core-foundation-sys, libc |
| core-foundation-sys@0.8.7 | registry; 773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b |  |
| cpufeatures@0.2.17 | registry; 59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280 | libc |
| crc32fast@1.5.0 | registry; 9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511 | cfg-if |
| crypto-common@0.1.6 | registry; 1bfb12502f3fc46cca1bb51ac28df9d618d813cdc3d2f25b9fe775a34af26bb3 | generic-array, typenum |
| ctr@0.9.2 | registry; 0369ee1ad671834580515889b80f2ea915f23b8be8d0daa4bbaf2ac5c7590835 | cipher |
| darling@0.20.11 | registry; fc7f46116c46ff9ab3eb1597a45688b6715c6e628b5c133e288e709a29bcb4ee | darling_core, darling_macro |
| darling_core@0.20.11 | registry; 0d00b9596d185e565c2207a0b01f8bd1a135483d02d9b7b0a54b11da8d53412e | fnv, ident_case, proc-macro2, quote, strsim, syn |
| darling_macro@0.20.11 | registry; fc34b93ccb385b40dc71c6fceac4b2ad23662c7eeb248cf10d529b7e055b6ead | darling_core, quote, syn |
| data-encoding@2.9.0 | registry; 2a2330da5de22e8a3cb63252ce2abb30116bf5265e89c0e01bc17015ce30a476 |  |
| der@0.7.10 | registry; e7c1832837b905bbfb5101e07cc24c8deddf52f93225eee6ead5f4d63d53ddcb | const-oid, pem-rfc7468, zeroize |
| deranged@0.5.5 | registry; ececcb659e7ba858fb4f10388c250a7252eb0a27373f1a72b8748afdd248e587 | powerfmt |
| derive_builder@0.20.2 | registry; 507dfb09ea8b7fa618fcf76e953f4f5e192547945816d5358edffe39f6f94947 | derive_builder_macro |
| derive_builder_core@0.20.2 | registry; 2d5bcf7b024d6835cfb3d473887cd966994907effbe9227e8c8219824d06c4e8 | darling, proc-macro2, quote, syn |
| derive_builder_macro@0.20.2 | registry; ab63b0e2bf4d5928aff72e83a7dace85d7bba5fe12dcc3c5a572d78caffd3f3c | derive_builder_core, syn |
| digest@0.10.7 | registry; 9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292 | block-buffer, const-oid, crypto-common, subtle |
| displaydoc@0.2.5 | registry; 97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0 | proc-macro2, quote, syn |
| either@1.15.0 | registry; 48c757948c5ede0e46177b7add2e67155f70e33c07fea8284df6576da70b3719 |  |
| encoding_rs@0.8.35 | registry; 75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3 | cfg-if |
| equivalent@1.0.2 | registry; 877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f |  |
| errno@0.3.14 | registry; 39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb | libc, windows-sys 0.61.2 |
| fastrand@2.3.0 | registry; 37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be |  |
| find-msvc-tools@0.1.4 | registry; 52051878f80a721bb68ebfbc930e07b65ba72f2da88968ea5c06fd6ca3d3a127 |  |
| flate2@1.1.5 | registry; bfe33edd8e85a12a67454e37f8c75e730830d83e313556ab9ebf9ee7fbeb3bfb | crc32fast, miniz_oxide |
| fnv@1.0.7 | registry; 3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1 |  |
| foldhash@0.1.5 | registry; d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2 |  |
| foreign-types@0.3.2 | registry; f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1 | foreign-types-shared |
| foreign-types-shared@0.1.1 | registry; 00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b |  |
| form_urlencoded@1.2.2 | registry; cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf | percent-encoding |
| futures-channel@0.3.31 | registry; 2dff15bf788c671c1934e366d07e30c1814a8ef514e1af724a602e8a2fbe1b10 | futures-core, futures-sink |
| futures-core@0.3.31 | registry; 05f29059c0c2090612e8d742178b0580d2dc940c837851ad723096f87af6663e |  |
| futures-io@0.3.31 | registry; 9e5c1b78ca4aae1ac06c48a526a655760685149f0d465d21f37abfe57ce075c6 |  |
| futures-sink@0.3.31 | registry; e575fab7d1e0dcb8d0c7bcf9a63ee213816ab51902e6d244a95819acacf1d4f7 |  |
| futures-task@0.3.31 | registry; f90f7dce0722e95104fcb095585910c0977252f286e354b5e3bd38902cd99988 |  |
| futures-timer@3.0.3 | registry; f288b0a4f20f9a56b5d1da57e2227c661b7b16168e2f72365f57b63326e29b24 |  |
| futures-util@0.3.31 | registry; 9fa08315bb612088cc391249efdc3bc77536f16c91f6cf495e6fbe85b20a4a81 | futures-core, futures-io, futures-sink, futures-task, memchr, pin-project-lite, pin-utils, slab |
| generic-array@0.14.9 | registry; 4bb6743198531e02858aeaea5398fcc883e71851fcbcb5a2f773e2fb6cb1edf2 | typenum, version_check |
| getrandom@0.2.16 | registry; 335ff9f135e4384c8150d6f27c6daed433577f86b4750418338c01a1a2528592 | cfg-if, js-sys, libc, wasi, wasm-bindgen |
| getrandom@0.3.4 | registry; 899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd | cfg-if, js-sys, libc, r-efi, wasip2, wasm-bindgen |
| governor@0.10.1 | registry; 444405bbb1a762387aa22dd569429533b54a1d8759d35d3b64cb39b0293eaa19 | cfg-if, futures-sink, futures-timer, futures-util, hashbrown 0.15.5, nonzero_ext, parking_lot, portable-atomic, smallvec, spinning_top, web-time |
| h2@0.4.12 | registry; f3c0b69cfcb4e1b9f1bf2f53f95f766e4661169728ec61cd3fe5a0166f2d1386 | atomic-waker, bytes, fnv, futures-core, futures-sink, http, indexmap, slab, tokio, tokio-util, tracing |
| hashbrown@0.15.5 | registry; 9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1 | allocator-api2, equivalent, foldhash |
| hashbrown@0.16.0 | registry; 5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d |  |
| headers@0.4.1 | registry; b3314d5adb5d94bcdf56771f2e50dbbc80bb4bdf88967526706205ac9eff24eb | base64, bytes, headers-core, http, httpdate, mime, sha1 |
| headers-core@0.3.0 | registry; 54b4a22553d4242c49fddb9ba998a99962b5cc6f22cb5a3482bec22522403ce4 | http |
| hmac@0.12.1 | registry; 6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e | digest |
| home@0.5.11 | registry; 589533453244b0995c858700322199b2becb13b627df2851f64a2775d024abcf | windows-sys 0.59.0 |
| http@1.3.1 | registry; f4a85d31aea989eead29a3aaf9e1115a180df8282431156e533de47660892565 | bytes, fnv, itoa |
| http-body@1.0.1 | registry; 1efedce1fb8e6913f23e0c92de8e62cd5b772a67e7b3946df930a62566c93184 | bytes, http |
| http-body-util@0.1.3 | registry; b021d93e26becf5dc7e1b75b1bed1fd93124b374ceb73f43d4d4eafec896a64a | bytes, futures-core, http, http-body, pin-project-lite |
| httparse@1.10.1 | registry; 6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87 |  |
| httpdate@1.0.3 | registry; df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9 |  |
| hyper@1.7.0 | registry; eb3aa54a13a0dfe7fbe3a59e0c76093041720fdc77b110cc0fc260fafb4dc51e | atomic-waker, bytes, futures-channel, futures-core, h2, http, http-body, httparse, itoa, pin-project-lite, pin-utils, smallvec, tokio, want |
| hyper-proxy2@0.1.0 | registry; 9043b7b23fb0bc4a1c7014c27b50a4fc42cc76206f71d34fc0dfe5b28ddc3faf | bytes, futures-util, headers, http, hyper, hyper-rustls 0.26.0, hyper-tls, hyper-util, native-tls, pin-project-lite, rustls-native-certs 0.7.3, tokio, tokio-native-tls, tokio-rustls 0.25.0, tower-service, webpki, webpki-roots 0.26.11 |
| hyper-rustls@0.26.0 | registry; a0bea761b46ae2b24eb4aef630d8d1c398157b6fc29e6350ecf090a0b70c952c | futures-util, http, hyper, hyper-util, log, rustls 0.22.4, rustls-native-certs 0.7.3, rustls-pki-types, tokio, tokio-rustls 0.25.0, tower-service, webpki-roots 0.26.11 |
| hyper-rustls@0.27.7 | registry; e3c93eb611681b207e1fe55d5a71ecf91572ec8a6705cdb6857f7d8d5242cf58 | http, hyper, hyper-util, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-rustls 0.26.4, tower-service, webpki-roots 1.0.4 |
| hyper-tls@0.6.0 | registry; 70206fc6890eaca9fde8a0bf71caa2ddfc9fe045ac9e5c70df101a7dbde866e0 | bytes, http-body-util, hyper, hyper-util, native-tls, tokio, tokio-native-tls, tower-service |
| hyper-util@0.1.17 | registry; 3c6995591a8f1380fcb4ba966a252a4b29188d51d2b89e3a252f5305be65aea8 | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, hyper, ipnet, libc, percent-encoding, pin-project-lite, socket2, system-configuration, tokio, tower-service, tracing, windows-registry |
| iana-time-zone@0.1.64 | registry; 33e57f83510bb73707521ebaffa789ec8caf86f9657cad665b092b581d40e9fb | android_system_properties, core-foundation-sys, iana-time-zone-haiku, js-sys, log, wasm-bindgen, windows-core 0.62.2 |
| iana-time-zone-haiku@0.1.2 | registry; f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f | cc |
| icu_collections@2.1.1 | registry; 4c6b649701667bbe825c3b7e6388cb521c23d88644678e83c0c4d0a621a34b43 | displaydoc, potential_utf, yoke, zerofrom, zerovec |
| icu_locale_core@2.1.1 | registry; edba7861004dd3714265b4db54a3c390e880ab658fec5f7db895fae2046b5bb6 | displaydoc, litemap, tinystr, writeable, zerovec |
| icu_normalizer@2.1.1 | registry; 5f6c8828b67bf8908d82127b2054ea1b4427ff0230ee9141c54251934ab1b599 | icu_collections, icu_normalizer_data, icu_properties, icu_provider, smallvec, zerovec |
| icu_normalizer_data@2.1.1 | registry; 7aedcccd01fc5fe81e6b489c15b247b8b0690feb23304303a9e560f37efc560a |  |
| icu_properties@2.1.1 | registry; e93fcd3157766c0c8da2f8cff6ce651a31f0810eaa1c51ec363ef790bbb5fb99 | icu_collections, icu_locale_core, icu_properties_data, icu_provider, zerotrie, zerovec |
| icu_properties_data@2.1.1 | registry; 02845b3647bb045f1100ecd6480ff52f34c35f82d9880e029d329c21d1054899 |  |
| icu_provider@2.1.1 | registry; 85962cf0ce02e1e0a629cc34e7ca3e373ce20dda4c4d7294bbd0bf1fdb59e614 | displaydoc, icu_locale_core, writeable, yoke, zerofrom, zerotrie, zerovec |
| ident_case@1.0.1 | registry; b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39 |  |
| idna@1.1.0 | registry; 3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de | idna_adapter, smallvec, utf8_iter |
| idna_adapter@1.2.1 | registry; 3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344 | icu_normalizer, icu_properties |
| indexmap@2.12.0 | registry; 6717a8d2a5a929a1a2eb43a12812498ed141a0bcfb7e8f7844fbdbe4303bba9f | equivalent, hashbrown 0.16.0 |
| inout@0.1.4 | registry; 879f10e63c20629ecabbb64a8010319738c66a5cd0c29b02d63d272b03751d01 | generic-array |
| ipnet@2.11.0 | registry; 469fb0b9cefa57e3ef31275ee7cacb78f2fdca44e4765491884a2b119d4eb130 |  |
| iri-string@0.7.9 | registry; 4f867b9d1d896b67beb18518eda36fdb77a32ea590de864f1325b294a6d14397 | memchr, serde |
| is-docker@0.2.0 | registry; 928bae27f42bc99b60d9ac7334e3a21d10ad8f1835a4e12ec3ec0464765ed1b3 | once_cell |
| is-wsl@0.4.0 | registry; 173609498df190136aa7dea1a91db051746d339e18476eed5ca40521f02d7aa5 | is-docker, once_cell |
| itoa@1.0.15 | registry; 4a5f13b858c8d314ee3e8f639011f7ccefe71f97f96e50151fb991f267928e2c |  |
| js-sys@0.3.82 | registry; b011eec8cc36da2aab2d5cff675ec18454fad408585853910a202391cf9f8e65 | once_cell, wasm-bindgen |
| lazy_static@1.5.0 | registry; bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe | spin |
| libc@0.2.177 | registry; 2874a2af47a2325c2001a6e6fad9b16a53b802102b528163885171cf92b15976 |  |
| libm@0.2.15 | registry; f9fbbcab51052fe104eb5e5d351cf728d30a5be1fe14d9be8a3b097481fb97de |  |
| librespot-audio@0.8.0 | registry; b3fe76acb49f58165484303edf0e7bd778f0e6d96f5c59e9d6b6fde1a90d36ff | aes, bytes, ctr, futures-util, http-body-util, hyper, hyper-util, librespot-core, log, tempfile, thiserror 2.0.17, tokio |
| librespot-connect@0.8.0 | local/path root | futures-util, librespot-core, librespot-playback, librespot-protocol, log, protobuf, rand 0.9.2, serde_json, thiserror 2.0.17, tokio, tokio-stream, uuid |
| librespot-core@0.8.0 | registry; 168bbe1c416980ddd9a969ebd6b50fb6c924eb1a3ded194285fa8ec0e2b1c68b | aes, base64, byteorder, bytes, data-encoding, flate2, form_urlencoded, futures-core, futures-util, governor, hmac, http, http-body-util, httparse, hyper, hyper-proxy2, hyper-rustls 0.27.7, hyper-tls, hyper-util, librespot-oauth, librespot-protocol, log, nonzero_ext, num-bigint, num-derive, num-integer, num-traits, pbkdf2, pin-project-lite, priority-queue, protobuf, protobuf-json-mapping, quick-xml, rand 0.9.2, rand_distr, rsa, serde, serde_json, sha1, shannon, sysinfo, thiserror 2.0.17, time, tokio, tokio-stream, tokio-tungstenite, tokio-util, url, uuid, vergen-gitcl |
| librespot-metadata@0.8.0 | registry; 9a9c688aa2acd3ed2498e31a95d6f2be49c0f18128db8958450ffd628aa88532 | async-trait, bytes, librespot-core, librespot-protocol, log, protobuf, serde, serde_json, thiserror 2.0.17, uuid |
| librespot-oauth@0.8.0 | registry; d686417d49c9d2c363392ffe28d6e469daca20a82dc414740930e078f5829661 | log, oauth2, open, reqwest, thiserror 2.0.17, url |
| librespot-playback@0.8.0 | registry; 88258620bf3e6808ea1fadd11639648d77c06280b9f5a4c9d14ea79f6f998af6 | form_urlencoded, futures-util, librespot-audio, librespot-core, librespot-metadata, log, portable-atomic, rand 0.9.2, rand_distr, shell-words, symphonia, thiserror 2.0.17, tokio, zerocopy |
| librespot-protocol@0.8.0 | registry; 3e01f0b2d39f83fa162eb91d4a16313bcf99e77daf258abe8f7b7bcb1160b084 | protobuf, protobuf-codegen |
| linux-raw-sys@0.4.15 | registry; d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab |  |
| linux-raw-sys@0.11.0 | registry; df1d3c3b53da64cf5760482273a98e575c651a67eec7f77df96b5b642de8f039 |  |
| litemap@0.8.1 | registry; 6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77 |  |
| lock_api@0.4.14 | registry; 224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965 | scopeguard |
| log@0.4.28 | registry; 34080505efa8e45a4b816c349525ebe327ceaa8559756f0356cba97ef3bf7432 |  |
| lru-slab@0.1.2 | registry; 112b39cec0b298b6c1999fee3e31427f74f676e4cb9879ed1a121b43661a4154 |  |
| memchr@2.7.6 | registry; f52b00d39961fc5b2736ea853c9cc86238e165017a493d1d5c8eac6bdc4cc273 |  |
| mime@0.3.17 | registry; 6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a |  |
| miniz_oxide@0.8.9 | registry; 1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316 | adler2, simd-adler32 |
| mio@1.1.0 | registry; 69d83b0086dc8ecf3ce9ae2874b2d1290252e2a30720bea58a5c6639b0092873 | libc, wasi, windows-sys 0.61.2 |
| native-tls@0.2.14 | registry; 87de3442987e9dbec73158d5c715e7ad9072fda936bb03d19d7fa10e00520f0e | libc, log, openssl, openssl-probe, openssl-sys, schannel, security-framework 2.11.1, security-framework-sys, tempfile |
| nonzero_ext@0.3.0 | registry; 38bf9645c8b145698bb0b18a4637dcacbc421ea49bef2317e4fd8065a387cf21 |  |
| ntapi@0.4.1 | registry; e8a3895c6391c39d7fe7ebc444a87eb2991b2a0bc718fdabd071eec617fc68e4 | winapi |
| num-bigint@0.4.6 | registry; a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9 | num-integer, num-traits |
| num-bigint-dig@0.8.5 | registry; 82c79c15c05d4bf82b6f5ef163104cc81a760d8e874d38ac50ab67c8877b647b | lazy_static, libm, num-integer, num-iter, num-traits, rand 0.8.5, smallvec, zeroize |
| num-conv@0.1.0 | registry; 51d515d32fb182ee37cda2ccdcb92950d6a3c2893aa280e540671c2cd0f3b1d9 |  |
| num-derive@0.4.2 | registry; ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202 | proc-macro2, quote, syn |
| num-integer@0.1.46 | registry; 7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f | num-traits |
| num-iter@0.1.45 | registry; 1429034a0490724d0075ebb2bc9e875d6503c3cf69e235a8941aa757d83ef5bf | autocfg, num-integer, num-traits |
| num-traits@0.2.19 | registry; 071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841 | autocfg, libm |
| num_threads@0.1.7 | registry; 5c7398b9c8b70908f6371f47ed36737907c87c52af34c268fed0bf0ceb92ead9 | libc |
| oauth2@5.0.0 | registry; 51e219e79014df21a225b1860a479e2dcd7cbd9130f4defd4bd0e191ea31d67d | base64, chrono, getrandom 0.2.16, http, rand 0.8.5, reqwest, serde, serde_json, serde_path_to_error, sha2, thiserror 1.0.69, url |
| objc2-core-foundation@0.3.2 | registry; 2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536 | bitflags 2.10.0 |
| objc2-io-kit@0.3.2 | registry; 33fafba39597d6dc1fb709123dfa8289d39406734be322956a69f0931c73bb15 | libc, objc2-core-foundation |
| once_cell@1.21.3 | registry; 42f5e15c9953c5e4ccceeb2e7382a716482c34515315f7b03532b8b4e8393d2d |  |
| open@5.3.2 | registry; e2483562e62ea94312f3576a7aca397306df7990b8d89033e18766744377ef95 | is-wsl, libc, pathdiff |
| openssl@0.10.75 | registry; 08838db121398ad17ab8531ce9de97b244589089e290a384c900cb9ff7434328 | bitflags 2.10.0, cfg-if, foreign-types, libc, once_cell, openssl-macros, openssl-sys |
| openssl-macros@0.1.1 | registry; a948666b637a0f465e8564c73e89d4dde00d72d4d473cc972f390fc3dcee7d9c | proc-macro2, quote, syn |
| openssl-probe@0.1.6 | registry; d05e27ee213611ffe7d6348b942e8f942b37114c00cc03cec254295a4a17852e |  |
| openssl-sys@0.9.111 | registry; 82cab2d520aa75e3c58898289429321eb788c3106963d0dc886ec7a5f4adc321 | cc, libc, pkg-config, vcpkg |
| parking_lot@0.12.5 | registry; 93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a | lock_api, parking_lot_core |
| parking_lot_core@0.9.12 | registry; 2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1 | cfg-if, libc, redox_syscall, smallvec, windows-link 0.2.1 |
| pathdiff@0.2.3 | registry; df94ce210e5bc13cb6651479fa48d14f601d9858cfe0467f43ae157023b938d3 |  |
| pbkdf2@0.12.2 | registry; f8ed6a7761f76e3b9f92dfb0a60a6a6477c61024b775147ff0973a02653abaf2 | digest, hmac |
| pem-rfc7468@0.7.0 | registry; 88b39c9bfcfc231068454382784bb460aae594343fb030d46e9f50a645418412 | base64ct |
| percent-encoding@2.3.2 | registry; 9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220 |  |
| pin-project-lite@0.2.16 | registry; 3b3cff922bd51709b605d9ead9aa71031d81447142d828eb4a6eba76fe619f9b |  |
| pin-utils@0.1.0 | registry; 8b870d8c151b6f2fb93e84a13146138f05d02ed11c7e7c54f8826aaaf7c9f184 |  |
| pkcs1@0.7.5 | registry; c8ffb9f10fa047879315e6625af03c164b16962a5368d724ed16323b68ace47f | der, pkcs8, spki |
| pkcs8@0.10.2 | registry; f950b2377845cebe5cf8b5165cb3cc1a5e0fa5cfa3e1f7f55707d8fd82e0a7b7 | der, spki |
| pkg-config@0.3.32 | registry; 7edddbd0b52d732b21ad9a5fab5c704c14cd949e5e9a1ec5929a24fded1b904c |  |
| portable-atomic@1.11.1 | registry; f84267b20a16ea918e43c6a88433c2d54fa145c92a811b5b047ccbe153674483 |  |
| potential_utf@0.1.4 | registry; b73949432f5e2a09657003c25bca5e19a0e9c84f8058ca374f49e0ebe605af77 | zerovec |
| powerfmt@0.2.0 | registry; 439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391 |  |
| ppv-lite86@0.2.21 | registry; 85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9 | zerocopy |
| priority-queue@2.7.0 | registry; 93980406f12d9f8140ed5abe7155acb10bb1e69ea55c88960b9c2f117445ef96 | equivalent, indexmap, serde |
| proc-macro2@1.0.103 | registry; 5ee95bc4ef87b8d5ba32e8b7714ccc834865276eab0aed5c9958d00ec45f49e8 | unicode-ident |
| protobuf@3.7.2 | registry; d65a1d4ddae7d8b5de68153b48f6aa3bba8cb002b243dbdbc55a5afbc98f99f4 | once_cell, protobuf-support, thiserror 1.0.69 |
| protobuf-codegen@3.7.2 | registry; 5d3976825c0014bbd2f3b34f0001876604fe87e0c86cd8fa54251530f1544ace | anyhow, once_cell, protobuf, protobuf-parse, regex, tempfile, thiserror 1.0.69 |
| protobuf-json-mapping@3.7.2 | registry; e0d6e4be637b310d8a5c02fa195243328e2d97fa7df1127a27281ef1187fcb1d | protobuf, protobuf-support, thiserror 1.0.69 |
| protobuf-parse@3.7.2 | registry; b4aeaa1f2460f1d348eeaeed86aea999ce98c1bded6f089ff8514c9d9dbdc973 | anyhow, indexmap, log, protobuf, protobuf-support, tempfile, thiserror 1.0.69, which |
| protobuf-support@3.7.2 | registry; 3e36c2f31e0a47f9280fb347ef5e461ffcd2c52dd520d8e216b52f93b0b0d7d6 | thiserror 1.0.69 |
| quick-xml@0.38.3 | registry; 42a232e7487fc2ef313d96dde7948e7a3c05101870d8985e4fd8d26aedd27b89 | memchr, serde |
| quinn@0.11.9 | registry; b9e20a958963c291dc322d98411f541009df2ced7b5a4f2bd52337638cfccf20 | bytes, cfg_aliases, pin-project-lite, quinn-proto, quinn-udp, rustc-hash, rustls 0.23.35, socket2, thiserror 2.0.17, tokio, tracing, web-time |
| quinn-proto@0.11.13 | registry; f1906b49b0c3bc04b5fe5d86a77925ae6524a19b816ae38ce1e426255f1d8a31 | bytes, getrandom 0.3.4, lru-slab, rand 0.9.2, ring, rustc-hash, rustls 0.23.35, rustls-pki-types, slab, thiserror 2.0.17, tinyvec, tracing, web-time |
| quinn-udp@0.5.14 | registry; addec6a0dcad8a8d96a771f815f0eaf55f9d1805756410b39f5fa81332574cbd | cfg_aliases, libc, once_cell, socket2, tracing, windows-sys 0.60.2 |
| quote@1.0.42 | registry; a338cc41d27e6cc6dce6cefc13a0729dfbb81c262b1f519331575dd80ef3067f | proc-macro2 |
| r-efi@5.3.0 | registry; 69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f |  |
| rand@0.8.5 | registry; 34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404 | libc, rand_chacha 0.3.1, rand_core 0.6.4 |
| rand@0.9.2 | registry; 6db2770f06117d490610c7488547d543617b21bfa07796d7a12f6f1bd53850d1 | rand_chacha 0.9.0, rand_core 0.9.3 |
| rand_chacha@0.3.1 | registry; e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88 | ppv-lite86, rand_core 0.6.4 |
| rand_chacha@0.9.0 | registry; d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb | ppv-lite86, rand_core 0.9.3 |
| rand_core@0.6.4 | registry; ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c | getrandom 0.2.16 |
| rand_core@0.9.3 | registry; 99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38 | getrandom 0.3.4 |
| rand_distr@0.5.1 | registry; 6a8615d50dcf34fa31f7ab52692afec947c4dd0ab803cc87cb3b0b4570ff7463 | num-traits, rand 0.9.2 |
| redox_syscall@0.5.18 | registry; ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d | bitflags 2.10.0 |
| regex@1.12.2 | registry; 843bc0191f75f3e22651ae5f1e72939ab2f72a4bc30fa80a066bd66edefc24d4 | aho-corasick, memchr, regex-automata, regex-syntax |
| regex-automata@0.4.13 | registry; 5276caf25ac86c8d810222b3dbb938e512c55c6831a10f3e6ed1c93b84041f1c | aho-corasick, memchr, regex-syntax |
| regex-syntax@0.8.8 | registry; 7a2d987857b319362043e95f5353c0535c1f58eec5336fdfcf626430af7def58 |  |
| reqwest@0.12.24 | registry; 9d0946410b9f7b082a427e4ef5c8ff541a88b357bc6c637c40db3a68ac70a36f | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, http-body-util, hyper, hyper-rustls 0.27.7, hyper-tls, hyper-util, js-sys, log, native-tls, percent-encoding, pin-project-lite, quinn, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, serde, serde_json, serde_urlencoded, sync_wrapper, tokio, tokio-native-tls, tokio-rustls 0.26.4, tower, tower-http, tower-service, url, wasm-bindgen, wasm-bindgen-futures, web-sys, webpki-roots 1.0.4 |
| ring@0.17.14 | registry; a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7 | cc, cfg-if, getrandom 0.2.16, libc, untrusted, windows-sys 0.52.0 |
| rsa@0.9.8 | registry; 78928ac1ed176a5ca1d17e578a1825f3d81ca54cf41053a592584b020cfd691b | const-oid, digest, num-bigint-dig, num-integer, num-traits, pkcs1, pkcs8, rand_core 0.6.4, signature, spki, subtle, zeroize |
| rustc-hash@2.1.1 | registry; 357703d41365b4b27c590e3ed91eabb1b663f07c4c084095e60cbed4362dff0d |  |
| rustix@0.38.44 | registry; fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154 | bitflags 2.10.0, errno, libc, linux-raw-sys 0.4.15, windows-sys 0.59.0 |
| rustix@1.1.2 | registry; cd15f8a2c5551a84d56efdc1cd049089e409ac19a3072d5037a17fd70719ff3e | bitflags 2.10.0, errno, libc, linux-raw-sys 0.11.0, windows-sys 0.61.2 |
| rustls@0.22.4 | registry; bf4ef73721ac7bcd79b2b315da7779d8fc09718c6b3d2d1b2d94850eb8c18432 | log, ring, rustls-pki-types, rustls-webpki 0.102.8, subtle, zeroize |
| rustls@0.23.35 | registry; 533f54bc6a7d4f647e46ad909549eda97bf5afc1585190ef692b4286b198bd8f | once_cell, ring, rustls-pki-types, rustls-webpki 0.103.8, subtle, zeroize |
| rustls-native-certs@0.7.3 | registry; e5bfb394eeed242e909609f56089eecfe5fda225042e8b171791b9c95f5931e5 | openssl-probe, rustls-pemfile, rustls-pki-types, schannel, security-framework 2.11.1 |
| rustls-native-certs@0.8.2 | registry; 9980d917ebb0c0536119ba501e90834767bffc3d60641457fd84a1f3fd337923 | openssl-probe, rustls-pki-types, schannel, security-framework 3.5.1 |
| rustls-pemfile@2.2.0 | registry; dce314e5fee3f39953d46bb63bb8a46d40c2f8fb7cc5a3b6cab2bde9721d6e50 | rustls-pki-types |
| rustls-pki-types@1.13.0 | registry; 94182ad936a0c91c324cd46c6511b9510ed16af436d7b5bab34beab0afd55f7a | web-time, zeroize |
| rustls-webpki@0.102.8 | registry; 64ca1bc8749bd4cf37b5ce386cc146580777b4e8572c7b97baf22c83f444bee9 | ring, rustls-pki-types, untrusted |
| rustls-webpki@0.103.8 | registry; 2ffdfa2f5286e2247234e03f680868ac2815974dc39e00ea15adc445d0aafe52 | ring, rustls-pki-types, untrusted |
| rustversion@1.0.22 | registry; b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d |  |
| ryu@1.0.20 | registry; 28d3b2b1366ec20994f1fd18c3c594f05c5dd4bc44d8bb0c1c632c8d6829481f |  |
| schannel@0.1.28 | registry; 891d81b926048e76efe18581bf793546b4c0eaf8448d72be8de2bbee5fd166e1 | windows-sys 0.61.2 |
| scopeguard@1.2.0 | registry; 94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49 |  |
| security-framework@2.11.1 | registry; 897b2245f0b511c87893af39b033e5ca9cce68824c4d7e7630b5a1d339658d02 | bitflags 2.10.0, core-foundation 0.9.4, core-foundation-sys, libc, security-framework-sys |
| security-framework@3.5.1 | registry; b3297343eaf830f66ede390ea39da1d462b6b0c1b000f420d0a83f898bbbe6ef | bitflags 2.10.0, core-foundation 0.10.1, core-foundation-sys, libc, security-framework-sys |
| security-framework-sys@2.15.0 | registry; cc1f0cbffaac4852523ce30d8bd3c5cdc873501d96ff467ca09b6767bb8cd5c0 | core-foundation-sys, libc |
| serde@1.0.228 | registry; 9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e | serde_core, serde_derive |
| serde_core@1.0.228 | registry; 41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad | serde_derive |
| serde_derive@1.0.228 | registry; d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79 | proc-macro2, quote, syn |
| serde_json@1.0.145 | registry; 402a6f66d8c709116cf22f558eab210f5a50187f702eb4d7e5ef38d9a7f1c79c | itoa, memchr, ryu, serde, serde_core |
| serde_path_to_error@0.1.20 | registry; 10a9ff822e371bb5403e391ecd83e182e0e77ba7f6fe0160b795797109d1b457 | itoa, serde, serde_core |
| serde_urlencoded@0.7.1 | registry; d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd | form_urlencoded, itoa, ryu, serde |
| sha1@0.10.6 | registry; e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba | cfg-if, cpufeatures, digest |
| sha2@0.10.9 | registry; a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283 | cfg-if, cpufeatures, digest |
| shannon@0.2.0 | registry; 7ea5b41c9427b56caa7b808cb548a04fb50bb5b9e98590b53f28064ff4174561 | byteorder |
| shell-words@1.1.0 | registry; 24188a676b6ae68c3b2cb3a01be17fbf7240ce009799bb56d5b1409051e78fde |  |
| shlex@1.3.0 | registry; 0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64 |  |
| signature@2.2.0 | registry; 77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de | digest, rand_core 0.6.4 |
| simd-adler32@0.3.7 | registry; d66dc143e6b11c1eddc06d5c423cfc97062865baf299914ab64caa38182078fe |  |
| slab@0.4.11 | registry; 7a2ae44ef20feb57a68b23d846850f861394c2e02dc425a50098ae8c90267589 |  |
| smallvec@1.15.1 | registry; 67b1b7a3b5fe4f1376887184045fcf45c69e92af734b7aaddc05fb777b6fbd03 |  |
| socket2@0.6.1 | registry; 17129e116933cf371d018bb80ae557e889637989d8638274fb25622827b03881 | libc, windows-sys 0.60.2 |
| spin@0.9.8 | registry; 6980e8d7511241f8acf4aebddbb1ff938df5eebe98691418c4468d0b72a96a67 |  |
| spinning_top@0.3.0 | registry; d96d2d1d716fb500937168cc09353ffdc7a012be8475ac7308e1bdf0e3923300 | lock_api |
| spki@0.7.3 | registry; d91ed6c858b01f942cd56b37a94b3e0a1798290327d1236e4d9cf4eaca44d29d | base64ct, der |
| stable_deref_trait@1.2.1 | registry; 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |  |
| strsim@0.11.1 | registry; 7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f |  |
| subtle@2.6.1 | registry; 13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292 |  |
| symphonia@0.5.5 | registry; 5773a4c030a19d9bfaa090f49746ff35c75dfddfa700df7a5939d5e076a57039 | lazy_static, symphonia-bundle-flac, symphonia-bundle-mp3, symphonia-codec-vorbis, symphonia-core, symphonia-format-ogg, symphonia-metadata |
| symphonia-bundle-flac@0.5.5 | registry; c91565e180aea25d9b80a910c546802526ffd0072d0b8974e3ebe59b686c9976 | log, symphonia-core, symphonia-metadata, symphonia-utils-xiph |
| symphonia-bundle-mp3@0.5.5 | registry; 4872dd6bb56bf5eac799e3e957aa1981086c3e613b27e0ac23b176054f7c57ed | lazy_static, log, symphonia-core, symphonia-metadata |
| symphonia-codec-vorbis@0.5.5 | registry; f025837c309cd69ffef572750b4a2257b59552c5399a5e49707cc5b1b85d1c73 | log, symphonia-core, symphonia-utils-xiph |
| symphonia-core@0.5.5 | registry; ea00cc4f79b7f6bb7ff87eddc065a1066f3a43fe1875979056672c9ef948c2af | arrayvec, bitflags 1.3.2, bytemuck, lazy_static, log |
| symphonia-format-ogg@0.5.5 | registry; 2b4955c67c1ed3aa8ae8428d04ca8397fbef6a19b2b051e73b5da8b1435639cb | log, symphonia-core, symphonia-metadata, symphonia-utils-xiph |
| symphonia-metadata@0.5.5 | registry; 36306ff42b9ffe6e5afc99d49e121e0bd62fe79b9db7b9681d48e29fa19e6b16 | encoding_rs, lazy_static, log, symphonia-core |
| symphonia-utils-xiph@0.5.5 | registry; ee27c85ab799a338446b68eec77abf42e1a6f1bb490656e121c6e27bfbab9f16 | symphonia-core, symphonia-metadata |
| syn@2.0.109 | registry; 2f17c7e013e88258aa9543dcbe81aca68a667a9ac37cd69c9fbc07858bfe0e2f | proc-macro2, quote, unicode-ident |
| sync_wrapper@1.0.2 | registry; 0bf256ce5efdfa370213c1dabab5935a12e49f2c58d15e9eac2870d3b4f27263 | futures-core |
| synstructure@0.13.2 | registry; 728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2 | proc-macro2, quote, syn |
| sysinfo@0.36.1 | registry; 252800745060e7b9ffb7b2badbd8b31cfa4aa2e61af879d0a3bf2a317c20217d | libc, memchr, ntapi, objc2-core-foundation, objc2-io-kit, windows |
| system-configuration@0.6.1 | registry; 3c879d448e9d986b661742763247d3693ed13609438cf3d006f51f5368a5ba6b | bitflags 2.10.0, core-foundation 0.9.4, system-configuration-sys |
| system-configuration-sys@0.6.0 | registry; 8e1d1b10ced5ca923a1fcb8d03e96b8d3268065d724548c0211415ff6ac6bac4 | core-foundation-sys, libc |
| tempfile@3.23.0 | registry; 2d31c77bdf42a745371d260a26ca7163f1e0924b64afa0b688e61b5a9fa02f16 | fastrand, getrandom 0.3.4, once_cell, rustix 1.1.2, windows-sys 0.61.2 |
| thiserror@1.0.69 | registry; b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52 | thiserror-impl 1.0.69 |
| thiserror@2.0.17 | registry; f63587ca0f12b72a0600bcba1d40081f830876000bb46dd2337a3051618f4fc8 | thiserror-impl 2.0.17 |
| thiserror-impl@1.0.69 | registry; 4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1 | proc-macro2, quote, syn |
| thiserror-impl@2.0.17 | registry; 3ff15c8ecd7de3849db632e14d18d2571fa09dfc5ed93479bc4485c7a517c913 | proc-macro2, quote, syn |
| time@0.3.44 | registry; 91e7d9e3bb61134e77bde20dd4825b97c010155709965fedf0f49bb138e52a9d | deranged, itoa, libc, num-conv, num_threads, powerfmt, serde, time-core, time-macros |
| time-core@0.1.6 | registry; 40868e7c1d2f0b8d73e4a8c7f0ff63af4f6d19be117e90bd73eb1d62cf831c6b |  |
| time-macros@0.2.24 | registry; 30cfb0125f12d9c277f35663a0a33f8c30190f4e4574868a330595412d34ebf3 | num-conv, time-core |
| tinystr@0.8.2 | registry; 42d3e9c45c09de15d06dd8acf5f4e0e399e85927b7f00711024eb7ae10fa4869 | displaydoc, zerovec |
| tinyvec@1.10.0 | registry; bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa | tinyvec_macros |
| tinyvec_macros@0.1.1 | registry; 1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20 |  |
| tokio@1.48.0 | registry; ff360e02eab121e0bc37a2d3b4d4dc622e6eda3a8e5253d5435ecf5bd4c68408 | bytes, libc, mio, pin-project-lite, socket2, tokio-macros, windows-sys 0.61.2 |
| tokio-macros@2.6.0 | registry; af407857209536a95c8e56f8231ef2c2e2aff839b22e07a1ffcbc617e9db9fa5 | proc-macro2, quote, syn |
| tokio-native-tls@0.3.1 | registry; bbae76ab933c85776efabc971569dd6119c580d8f5d448769dec1764bf796ef2 | native-tls, tokio |
| tokio-rustls@0.25.0 | registry; 775e0c0f0adb3a2f22a00c4745d728b479985fc15ee7ca6a2608388c5569860f | rustls 0.22.4, rustls-pki-types, tokio |
| tokio-rustls@0.26.4 | registry; 1729aa945f29d91ba541258c8df89027d5792d85a8841fb65e8bf0f4ede4ef61 | rustls 0.23.35, tokio |
| tokio-stream@0.1.17 | registry; eca58d7bba4a75707817a2c44174253f9236b2d5fbd055602e9d5c07c139a047 | futures-core, pin-project-lite, tokio |
| tokio-tungstenite@0.28.0 | registry; d25a406cddcc431a75d3d9afc6a7c0f7428d4891dd973e4d54c56b46127bf857 | futures-util, log, native-tls, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-native-tls, tokio-rustls 0.26.4, tungstenite, webpki-roots 0.26.11 |
| tokio-util@0.7.17 | registry; 2efa149fe76073d6e8fd97ef4f4eca7b67f599660115591483572e406e165594 | bytes, futures-core, futures-sink, pin-project-lite, tokio |
| tower@0.5.2 | registry; d039ad9159c98b70ecfd540b2573b97f7f52c3e8d9f8ad57a24b916a536975f9 | futures-core, futures-util, pin-project-lite, sync_wrapper, tokio, tower-layer, tower-service |
| tower-http@0.6.6 | registry; adc82fd73de2a9722ac5da747f12383d2bfdb93591ee6c58486e0097890f05f2 | bitflags 2.10.0, bytes, futures-util, http, http-body, iri-string, pin-project-lite, tower, tower-layer, tower-service |
| tower-layer@0.3.3 | registry; 121c2a6cda46980bb0fcd1647ffaf6cd3fc79a013de288782836f6df9c48780e |  |
| tower-service@0.3.3 | registry; 8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3 |  |
| tracing@0.1.41 | registry; 784e0ac535deb450455cbfa28a6f0df145ea1bb7ae51b821cf5e7927fdcfbdd0 | pin-project-lite, tracing-core |
| tracing-core@0.1.34 | registry; b9d12581f227e93f094d3af2ae690a574abb8a2b9b7a96e7cfe9647b2b617678 | once_cell |
| try-lock@0.2.5 | registry; e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b |  |
| tungstenite@0.28.0 | registry; 8628dcc84e5a09eb3d8423d6cb682965dea9133204e8fb3efee74c2a0c259442 | bytes, data-encoding, http, httparse, log, native-tls, rand 0.9.2, rustls 0.23.35, rustls-pki-types, sha1, thiserror 2.0.17, utf-8 |
| typenum@1.19.0 | registry; 562d481066bde0658276a35467c4af00bdc6ee726305698a55b86e61d7ad82bb |  |
| unicode-ident@1.0.22 | registry; 9312f7c4f6ff9069b165498234ce8be658059c6728633667c526e27dc2cf1df5 |  |
| untrusted@0.9.0 | registry; 8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1 |  |
| url@2.5.7 | registry; 08bc136a29a3d1758e07a9cca267be308aeebf5cfd5a10f3f67ab2097683ef5b | form_urlencoded, idna, percent-encoding, serde |
| utf-8@0.7.6 | registry; 09cc8ee72d2a9becf2f2febe0205bbed8fc6615b7cb429ad062dc7b7ddd036a9 |  |
| utf8_iter@1.0.4 | registry; b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be |  |
| uuid@1.18.1 | registry; 2f87b8aa10b915a06587d0dec516c282ff295b475d94abf425d62b57710070a2 | getrandom 0.3.4 |
| vcpkg@0.2.15 | registry; accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426 |  |
| vergen@9.0.6 | registry; 6b2bf58be11fc9414104c6d3a2e464163db5ef74b12296bda593cac37b6e4777 | anyhow, derive_builder, rustversion, time, vergen-lib |
| vergen-gitcl@1.0.8 | registry; b9dfc1de6eb2e08a4ddf152f1b179529638bedc0ea95e6d667c014506377aefe | anyhow, derive_builder, rustversion, time, vergen, vergen-lib |
| vergen-lib@0.1.6 | registry; 9b07e6010c0f3e59fcb164e0163834597da68d1f864e2b8ca49f74de01e9c166 | anyhow, derive_builder, rustversion |
| version_check@0.9.5 | registry; 0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a |  |
| want@0.3.1 | registry; bfa7760aed19e106de2c7c0b581b509f2f25d3dacaf737cb82ac61bc6d760b0e | try-lock |
| wasi@0.11.1+wasi-snapshot-preview1 | registry; ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b |  |
| wasip2@1.0.1+wasi-0.2.4 | registry; 0562428422c63773dad2c345a1882263bbf4d65cf3f42e90921f787ef5ad58e7 | wit-bindgen |
| wasm-bindgen@0.2.105 | registry; da95793dfc411fbbd93f5be7715b0578ec61fe87cb1a42b12eb625caa5c5ea60 | cfg-if, once_cell, rustversion, wasm-bindgen-macro, wasm-bindgen-shared |
| wasm-bindgen-futures@0.4.55 | registry; 551f88106c6d5e7ccc7cd9a16f312dd3b5d36ea8b4954304657d5dfba115d4a0 | cfg-if, js-sys, once_cell, wasm-bindgen, web-sys |
| wasm-bindgen-macro@0.2.105 | registry; 04264334509e04a7bf8690f2384ef5265f05143a4bff3889ab7a3269adab59c2 | quote, wasm-bindgen-macro-support |
| wasm-bindgen-macro-support@0.2.105 | registry; 420bc339d9f322e562942d52e115d57e950d12d88983a14c79b86859ee6c7ebc | bumpalo, proc-macro2, quote, syn, wasm-bindgen-shared |
| wasm-bindgen-shared@0.2.105 | registry; 76f218a38c84bcb33c25ec7059b07847d465ce0e0a76b995e134a45adcb6af76 | unicode-ident |
| web-sys@0.3.82 | registry; 3a1f95c0d03a47f4ae1f7a64643a6bb97465d9b740f0fa8f90ea33915c99a9a1 | js-sys, wasm-bindgen |
| web-time@1.1.0 | registry; 5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb | js-sys, wasm-bindgen |
| webpki@0.22.4 | registry; ed63aea5ce73d0ff405984102c42de94fc55a6b75765d621c65262469b3c9b53 | ring, untrusted |
| webpki-roots@0.26.11 | registry; 521bc38abb08001b01866da9f51eb7c5d647a19260e00054a8c7fd5f9e57f7a9 | webpki-roots 1.0.4 |
| webpki-roots@1.0.4 | registry; b2878ef029c47c6e8cf779119f20fcf52bde7ad42a731b2a304bc221df17571e | rustls-pki-types |
| which@4.4.2 | registry; 87ba24419a2078cd2b0f2ede2691b6c66d8e47836da3b6db8265ebad47afbfc7 | either, home, once_cell, rustix 0.38.44 |
| winapi@0.3.9 | registry; 5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419 | winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu |
| winapi-i686-pc-windows-gnu@0.4.0 | registry; ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6 |  |
| winapi-x86_64-pc-windows-gnu@0.4.0 | registry; 712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f |  |
| windows@0.61.3 | registry; 9babd3a767a4c1aef6900409f85f5d53ce2544ccdfaa86dad48c91782c6d6893 | windows-collections, windows-core 0.61.2, windows-future, windows-link 0.1.3, windows-numerics |
| windows-collections@0.2.0 | registry; 3beeceb5e5cfd9eb1d76b381630e82c4241ccd0d27f1a39ed41b2760b255c5e8 | windows-core 0.61.2 |
| windows-core@0.61.2 | registry; c0fdd3ddb90610c7638aa2b3a3ab2904fb9e5cdbecc643ddb3647212781c4ae3 | windows-implement, windows-interface, windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-core@0.62.2 | registry; b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb | windows-implement, windows-interface, windows-link 0.2.1, windows-result 0.4.1, windows-strings 0.5.1 |
| windows-future@0.2.1 | registry; fc6a41e98427b19fe4b73c550f060b59fa592d7d686537eebf9385621bfbad8e | windows-core 0.61.2, windows-link 0.1.3, windows-threading |
| windows-implement@0.60.2 | registry; 053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf | proc-macro2, quote, syn |
| windows-interface@0.59.3 | registry; 3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358 | proc-macro2, quote, syn |
| windows-link@0.1.3 | registry; 5e6ad25900d524eaabdbbb96d20b4311e1e7ae1699af4fb28c17ae66c80d798a |  |
| windows-link@0.2.1 | registry; f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5 |  |
| windows-numerics@0.2.0 | registry; 9150af68066c4c5c07ddc0ce30421554771e528bde427614c61038bc2c92c2b1 | windows-core 0.61.2, windows-link 0.1.3 |
| windows-registry@0.5.3 | registry; 5b8a9ed28765efc97bbc954883f4e6796c33a06546ebafacbabee9696967499e | windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-result@0.3.4 | registry; 56f42bd332cc6c8eac5af113fc0c1fd6a8fd2aa08a0119358686e5160d0586c6 | windows-link 0.1.3 |
| windows-result@0.4.1 | registry; 7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5 | windows-link 0.2.1 |
| windows-strings@0.4.2 | registry; 56e6c93f3a0c3b36176cb1327a4958a0353d5d166c2a35cb268ace15e91d3b57 | windows-link 0.1.3 |
| windows-strings@0.5.1 | registry; 7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091 | windows-link 0.2.1 |
| windows-sys@0.52.0 | registry; 282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d | windows-targets 0.52.6 |
| windows-sys@0.59.0 | registry; 1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b | windows-targets 0.52.6 |
| windows-sys@0.60.2 | registry; f2f500e4d28234f72040990ec9d39e3a6b950f9f22d3dba18416c35882612bcb | windows-targets 0.53.5 |
| windows-sys@0.61.2 | registry; ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc | windows-link 0.2.1 |
| windows-targets@0.52.6 | registry; 9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973 | windows_aarch64_gnullvm 0.52.6, windows_aarch64_msvc 0.52.6, windows_i686_gnu 0.52.6, windows_i686_gnullvm 0.52.6, windows_i686_msvc 0.52.6, windows_x86_64_gnu 0.52.6, windows_x86_64_gnullvm 0.52.6, windows_x86_64_msvc 0.52.6 |
| windows-targets@0.53.5 | registry; 4945f9f551b88e0d65f3db0bc25c33b8acea4d9e41163edf90dcd0b19f9069f3 | windows-link 0.2.1, windows_aarch64_gnullvm 0.53.1, windows_aarch64_msvc 0.53.1, windows_i686_gnu 0.53.1, windows_i686_gnullvm 0.53.1, windows_i686_msvc 0.53.1, windows_x86_64_gnu 0.53.1, windows_x86_64_gnullvm 0.53.1, windows_x86_64_msvc 0.53.1 |
| windows-threading@0.1.0 | registry; b66463ad2e0ea3bbf808b7f1d371311c80e115c0b71d60efc142cafbcfb057a6 | windows-link 0.1.3 |
| windows_aarch64_gnullvm@0.52.6 | registry; 32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3 |  |
| windows_aarch64_gnullvm@0.53.1 | registry; a9d8416fa8b42f5c947f8482c43e7d89e73a173cead56d044f6a56104a6d1b53 |  |
| windows_aarch64_msvc@0.52.6 | registry; 09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469 |  |
| windows_aarch64_msvc@0.53.1 | registry; b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006 |  |
| windows_i686_gnu@0.52.6 | registry; 8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b |  |
| windows_i686_gnu@0.53.1 | registry; 960e6da069d81e09becb0ca57a65220ddff016ff2d6af6a223cf372a506593a3 |  |
| windows_i686_gnullvm@0.52.6 | registry; 0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66 |  |
| windows_i686_gnullvm@0.53.1 | registry; fa7359d10048f68ab8b09fa71c3daccfb0e9b559aed648a8f95469c27057180c |  |
| windows_i686_msvc@0.52.6 | registry; 240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66 |  |
| windows_i686_msvc@0.53.1 | registry; 1e7ac75179f18232fe9c285163565a57ef8d3c89254a30685b57d83a38d326c2 |  |
| windows_x86_64_gnu@0.52.6 | registry; 147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78 |  |
| windows_x86_64_gnu@0.53.1 | registry; 9c3842cdd74a865a8066ab39c8a7a473c0778a3f29370b5fd6b4b9aa7df4a499 |  |
| windows_x86_64_gnullvm@0.52.6 | registry; 24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d |  |
| windows_x86_64_gnullvm@0.53.1 | registry; 0ffa179e2d07eee8ad8f57493436566c7cc30ac536a3379fdf008f47f6bb7ae1 |  |
| windows_x86_64_msvc@0.52.6 | registry; 589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec |  |
| windows_x86_64_msvc@0.53.1 | registry; d6bbff5f0aada427a1e5a6da5f1f98158182f26556f345ac9e04d36d0ebed650 |  |
| wit-bindgen@0.46.0 | registry; f17a85883d4e6d00e8a97c586de764dabcc06133f7f1d55dce5cdc070ad7fe59 |  |
| writeable@0.6.2 | registry; 9edde0db4769d2dc68579893f2306b26c6ecfbe0ef499b013d731b7b9247e0b9 |  |
| yoke@0.8.1 | registry; 72d6e5c6afb84d73944e5cedb052c4680d5657337201555f9f2a16b7406d4954 | stable_deref_trait, yoke-derive, zerofrom |
| yoke-derive@0.8.1 | registry; b659052874eb698efe5b9e8cf382204678a0086ebf46982b79d6ca3182927e5d | proc-macro2, quote, syn, synstructure |
| zerocopy@0.8.27 | registry; 0894878a5fa3edfd6da3f88c4805f4c8558e2b996227a3d864f47fe11e38282c | zerocopy-derive |
| zerocopy-derive@0.8.27 | registry; 88d2b8d9c68ad2b9e4340d7832716a4d21a22a1154777ad56ea55c51a9cf3831 | proc-macro2, quote, syn |
| zerofrom@0.1.6 | registry; 50cc42e0333e05660c3587f3bf9d0478688e15d870fab3346451ce7f8c9fbea5 | zerofrom-derive |
| zerofrom-derive@0.1.6 | registry; d71e5d6e06ab090c67b5e44993ec16b72dcbaabc526db883a360057678b48502 | proc-macro2, quote, syn, synstructure |
| zeroize@1.8.2 | registry; b97154e67e32c85465826e8bcc1c59429aaaf107c1e4a9e53c8d8ccd5eff88d0 |  |
| zerotrie@0.2.3 | registry; 2a59c17a5562d507e4b54960e8569ebee33bee890c70aa3fe7b97e85a9fd7851 | displaydoc, yoke, zerofrom |
| zerovec@0.11.5 | registry; 6c28719294829477f525be0186d13efa9a3c602f7ec202ca9e353d310fb9a002 | yoke, zerofrom, zerovec-derive |
| zerovec-derive@0.11.2 | registry; eadce39539ca5cb3985590102671f2567e659fca9666581ad3411d59207951f3 | proc-macro2, quote, syn |

## src-tauri/vendor/librespot-core/Cargo.lock

{"path": "src-tauri/vendor/librespot-core/Cargo.lock", "packages": 327, "dependency_edges": 869, "duplicate_names": 31, "unresolved_or_ambiguous_edges": [], "nonregistry_packages": ["librespot-core@0.8.0"]}

### Multiple versions

{"core-foundation": ["0.9.4", "0.10.1"], "getrandom": ["0.2.16", "0.3.4"], "hashbrown": ["0.15.5", "0.16.0"], "hyper-rustls": ["0.26.0", "0.27.7"], "linux-raw-sys": ["0.4.15", "0.11.0"], "rand": ["0.8.5", "0.9.2"], "rand_chacha": ["0.3.1", "0.9.0"], "rand_core": ["0.6.4", "0.9.3"], "rustix": ["0.38.44", "1.1.2"], "rustls": ["0.22.4", "0.23.35"], "rustls-native-certs": ["0.7.3", "0.8.2"], "rustls-webpki": ["0.102.8", "0.103.8"], "security-framework": ["2.11.1", "3.5.1"], "thiserror": ["1.0.69", "2.0.17"], "thiserror-impl": ["1.0.69", "2.0.17"], "tokio-rustls": ["0.25.0", "0.26.4"], "webpki-roots": ["0.26.11", "1.0.4"], "windows-core": ["0.61.2", "0.62.2"], "windows-link": ["0.1.3", "0.2.1"], "windows-result": ["0.3.4", "0.4.1"], "windows-strings": ["0.4.2", "0.5.1"], "windows-sys": ["0.52.0", "0.59.0", "0.60.2", "0.61.2"], "windows-targets": ["0.52.6", "0.53.5"], "windows_aarch64_gnullvm": ["0.52.6", "0.53.1"], "windows_aarch64_msvc": ["0.52.6", "0.53.1"], "windows_i686_gnu": ["0.52.6", "0.53.1"], "windows_i686_gnullvm": ["0.52.6", "0.53.1"], "windows_i686_msvc": ["0.52.6", "0.53.1"], "windows_x86_64_gnu": ["0.52.6", "0.53.1"], "windows_x86_64_gnullvm": ["0.52.6", "0.53.1"], "windows_x86_64_msvc": ["0.52.6", "0.53.1"]}

### Every resolved package and edge

| Package | Source/checksum | Dependencies |
| --- | --- | --- |
| adler2@2.0.1 | registry; 320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa |  |
| aes@0.8.4 | registry; b169f7a6d4742236a0a00c541b845991d0ac43e546831af1249753ab4c3aa3a0 | cfg-if, cipher, cpufeatures |
| aho-corasick@1.1.4 | registry; ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301 | memchr |
| allocator-api2@0.2.21 | registry; 683d7910e743518b0e34f1186f92494becacb047c7b6bf616c96772180fef923 |  |
| android_system_properties@0.1.5 | registry; 819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311 | libc |
| anyhow@1.0.100 | registry; a23eb6b1614318a8071c9b2521f36b424b2c83db5eb3a0fead4a6c0809af6e61 |  |
| atomic-waker@1.1.2 | registry; 1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0 |  |
| autocfg@1.5.0 | registry; c08606f8c3cbf4ce6ec8e28fb0014a2c086708fe954eaa885384a6165172e7e8 |  |
| base64@0.22.1 | registry; 72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6 |  |
| base64ct@1.8.0 | registry; 55248b47b0caf0546f7988906588779981c43bb1bc9d0c44087278f80cdb44ba |  |
| bitflags@2.10.0 | registry; 812e12b5285cc515a9c72a5c1d3b6d46a19dac5acfef5265968c166106e31dd3 |  |
| block-buffer@0.10.4 | registry; 3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71 | generic-array |
| bumpalo@3.19.0 | registry; 46c5e41b57b8bba42a04676d81cb89e9ee8e859a1a66f80a5a72e1cb76b34d43 |  |
| byteorder@1.5.0 | registry; 1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b |  |
| bytes@1.10.1 | registry; d71b6127be86fdcfddb610f7182ac57211d4b18a3e9c82eb2d17662f2227ad6a |  |
| cc@1.2.45 | registry; 35900b6c8d709fb1d854671ae27aeaa9eec2f8b01b364e1619a40da3e6fe2afe | find-msvc-tools, shlex |
| cfg-if@1.0.4 | registry; 9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801 |  |
| cfg_aliases@0.2.1 | registry; 613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724 |  |
| chrono@0.4.42 | registry; 145052bdd345b87320e369255277e3fb5152762ad123a901ef5c262dd38fe8d2 | iana-time-zone, js-sys, num-traits, serde, wasm-bindgen, windows-link 0.2.1 |
| cipher@0.4.4 | registry; 773f3b9af64447d2ce9850330c473515014aa235e6a783b02db81ff39e4a3dad | crypto-common, inout |
| const-oid@0.9.6 | registry; c2459377285ad874054d797f3ccebf984978aa39129f6eafde5cdc8315b612f8 |  |
| core-foundation@0.9.4 | registry; 91e195e091a93c46f7102ec7818a2aa394e1e1771c3ab4825963fa03e45afb8f | core-foundation-sys, libc |
| core-foundation@0.10.1 | registry; b2a6cd9ae233e7f62ba4e9353e81a88df7fc8a5987b8d445b4d90c879bd156f6 | core-foundation-sys, libc |
| core-foundation-sys@0.8.7 | registry; 773648b94d0e5d620f64f280777445740e61fe701025087ec8b57f45c791888b |  |
| cpufeatures@0.2.17 | registry; 59ed5838eebb26a2bb2e58f6d5b5316989ae9d08bab10e0e6d103e656d1b0280 | libc |
| crc32fast@1.5.0 | registry; 9481c1c90cbf2ac953f07c8d4a58aa3945c425b7185c9154d67a65e4230da511 | cfg-if |
| crypto-common@0.1.6 | registry; 1bfb12502f3fc46cca1bb51ac28df9d618d813cdc3d2f25b9fe775a34af26bb3 | generic-array, typenum |
| darling@0.20.11 | registry; fc7f46116c46ff9ab3eb1597a45688b6715c6e628b5c133e288e709a29bcb4ee | darling_core, darling_macro |
| darling_core@0.20.11 | registry; 0d00b9596d185e565c2207a0b01f8bd1a135483d02d9b7b0a54b11da8d53412e | fnv, ident_case, proc-macro2, quote, strsim, syn |
| darling_macro@0.20.11 | registry; fc34b93ccb385b40dc71c6fceac4b2ad23662c7eeb248cf10d529b7e055b6ead | darling_core, quote, syn |
| data-encoding@2.9.0 | registry; 2a2330da5de22e8a3cb63252ce2abb30116bf5265e89c0e01bc17015ce30a476 |  |
| der@0.7.10 | registry; e7c1832837b905bbfb5101e07cc24c8deddf52f93225eee6ead5f4d63d53ddcb | const-oid, pem-rfc7468, zeroize |
| deranged@0.5.5 | registry; ececcb659e7ba858fb4f10388c250a7252eb0a27373f1a72b8748afdd248e587 | powerfmt |
| derive_builder@0.20.2 | registry; 507dfb09ea8b7fa618fcf76e953f4f5e192547945816d5358edffe39f6f94947 | derive_builder_macro |
| derive_builder_core@0.20.2 | registry; 2d5bcf7b024d6835cfb3d473887cd966994907effbe9227e8c8219824d06c4e8 | darling, proc-macro2, quote, syn |
| derive_builder_macro@0.20.2 | registry; ab63b0e2bf4d5928aff72e83a7dace85d7bba5fe12dcc3c5a572d78caffd3f3c | derive_builder_core, syn |
| digest@0.10.7 | registry; 9ed9a281f7bc9b7576e61468ba615a66a5c8cfdff42420a70aa82701a3b1e292 | block-buffer, const-oid, crypto-common, subtle |
| displaydoc@0.2.5 | registry; 97369cbbc041bc366949bc74d34658d6cda5621039731c6310521892a3a20ae0 | proc-macro2, quote, syn |
| either@1.15.0 | registry; 48c757948c5ede0e46177b7add2e67155f70e33c07fea8284df6576da70b3719 |  |
| equivalent@1.0.2 | registry; 877a4ace8713b0bcf2a4e7eec82529c029f1d0619886d18145fea96c3ffe5c0f |  |
| errno@0.3.14 | registry; 39cab71617ae0d63f51a36d69f866391735b51691dbda63cf6f96d042b63efeb | libc, windows-sys 0.61.2 |
| fastrand@2.3.0 | registry; 37909eebbb50d72f9059c3b6d82c0463f2ff062c9e95845c43a6c9c0355411be |  |
| find-msvc-tools@0.1.4 | registry; 52051878f80a721bb68ebfbc930e07b65ba72f2da88968ea5c06fd6ca3d3a127 |  |
| flate2@1.1.5 | registry; bfe33edd8e85a12a67454e37f8c75e730830d83e313556ab9ebf9ee7fbeb3bfb | crc32fast, miniz_oxide |
| fnv@1.0.7 | registry; 3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1 |  |
| foldhash@0.1.5 | registry; d9c4f5dac5e15c24eb999c26181a6ca40b39fe946cbe4c263c7209467bc83af2 |  |
| foreign-types@0.3.2 | registry; f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1 | foreign-types-shared |
| foreign-types-shared@0.1.1 | registry; 00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b |  |
| form_urlencoded@1.2.2 | registry; cb4cb245038516f5f85277875cdaa4f7d2c9a0fa0468de06ed190163b1581fcf | percent-encoding |
| futures-channel@0.3.31 | registry; 2dff15bf788c671c1934e366d07e30c1814a8ef514e1af724a602e8a2fbe1b10 | futures-core, futures-sink |
| futures-core@0.3.31 | registry; 05f29059c0c2090612e8d742178b0580d2dc940c837851ad723096f87af6663e |  |
| futures-io@0.3.31 | registry; 9e5c1b78ca4aae1ac06c48a526a655760685149f0d465d21f37abfe57ce075c6 |  |
| futures-sink@0.3.31 | registry; e575fab7d1e0dcb8d0c7bcf9a63ee213816ab51902e6d244a95819acacf1d4f7 |  |
| futures-task@0.3.31 | registry; f90f7dce0722e95104fcb095585910c0977252f286e354b5e3bd38902cd99988 |  |
| futures-timer@3.0.3 | registry; f288b0a4f20f9a56b5d1da57e2227c661b7b16168e2f72365f57b63326e29b24 |  |
| futures-util@0.3.31 | registry; 9fa08315bb612088cc391249efdc3bc77536f16c91f6cf495e6fbe85b20a4a81 | futures-core, futures-io, futures-sink, futures-task, memchr, pin-project-lite, pin-utils, slab |
| generic-array@0.14.9 | registry; 4bb6743198531e02858aeaea5398fcc883e71851fcbcb5a2f773e2fb6cb1edf2 | typenum, version_check |
| getrandom@0.2.16 | registry; 335ff9f135e4384c8150d6f27c6daed433577f86b4750418338c01a1a2528592 | cfg-if, js-sys, libc, wasi, wasm-bindgen |
| getrandom@0.3.4 | registry; 899def5c37c4fd7b2664648c28120ecec138e4d395b459e5ca34f9cce2dd77fd | cfg-if, js-sys, libc, r-efi, wasip2, wasm-bindgen |
| governor@0.10.1 | registry; 444405bbb1a762387aa22dd569429533b54a1d8759d35d3b64cb39b0293eaa19 | cfg-if, futures-sink, futures-timer, futures-util, hashbrown 0.15.5, nonzero_ext, parking_lot, portable-atomic, smallvec, spinning_top, web-time |
| h2@0.4.12 | registry; f3c0b69cfcb4e1b9f1bf2f53f95f766e4661169728ec61cd3fe5a0166f2d1386 | atomic-waker, bytes, fnv, futures-core, futures-sink, http, indexmap, slab, tokio, tokio-util, tracing |
| hashbrown@0.15.5 | registry; 9229cfe53dfd69f0609a49f65461bd93001ea1ef889cd5529dd176593f5338a1 | allocator-api2, equivalent, foldhash |
| hashbrown@0.16.0 | registry; 5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d |  |
| headers@0.4.1 | registry; b3314d5adb5d94bcdf56771f2e50dbbc80bb4bdf88967526706205ac9eff24eb | base64, bytes, headers-core, http, httpdate, mime, sha1 |
| headers-core@0.3.0 | registry; 54b4a22553d4242c49fddb9ba998a99962b5cc6f22cb5a3482bec22522403ce4 | http |
| hmac@0.12.1 | registry; 6c49c37c09c17a53d937dfbb742eb3a961d65a994e6bcdcf37e7399d0cc8ab5e | digest |
| home@0.5.11 | registry; 589533453244b0995c858700322199b2becb13b627df2851f64a2775d024abcf | windows-sys 0.59.0 |
| http@1.3.1 | registry; f4a85d31aea989eead29a3aaf9e1115a180df8282431156e533de47660892565 | bytes, fnv, itoa |
| http-body@1.0.1 | registry; 1efedce1fb8e6913f23e0c92de8e62cd5b772a67e7b3946df930a62566c93184 | bytes, http |
| http-body-util@0.1.3 | registry; b021d93e26becf5dc7e1b75b1bed1fd93124b374ceb73f43d4d4eafec896a64a | bytes, futures-core, http, http-body, pin-project-lite |
| httparse@1.10.1 | registry; 6dbf3de79e51f3d586ab4cb9d5c3e2c14aa28ed23d180cf89b4df0454a69cc87 |  |
| httpdate@1.0.3 | registry; df3b46402a9d5adb4c86a0cf463f42e19994e3ee891101b1841f30a545cb49a9 |  |
| hyper@1.7.0 | registry; eb3aa54a13a0dfe7fbe3a59e0c76093041720fdc77b110cc0fc260fafb4dc51e | atomic-waker, bytes, futures-channel, futures-core, h2, http, http-body, httparse, itoa, pin-project-lite, pin-utils, smallvec, tokio, want |
| hyper-proxy2@0.1.0 | registry; 9043b7b23fb0bc4a1c7014c27b50a4fc42cc76206f71d34fc0dfe5b28ddc3faf | bytes, futures-util, headers, http, hyper, hyper-rustls 0.26.0, hyper-tls, hyper-util, native-tls, pin-project-lite, rustls-native-certs 0.7.3, tokio, tokio-native-tls, tokio-rustls 0.25.0, tower-service, webpki, webpki-roots 0.26.11 |
| hyper-rustls@0.26.0 | registry; a0bea761b46ae2b24eb4aef630d8d1c398157b6fc29e6350ecf090a0b70c952c | futures-util, http, hyper, hyper-util, log, rustls 0.22.4, rustls-native-certs 0.7.3, rustls-pki-types, tokio, tokio-rustls 0.25.0, tower-service, webpki-roots 0.26.11 |
| hyper-rustls@0.27.7 | registry; e3c93eb611681b207e1fe55d5a71ecf91572ec8a6705cdb6857f7d8d5242cf58 | http, hyper, hyper-util, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-rustls 0.26.4, tower-service, webpki-roots 1.0.4 |
| hyper-tls@0.6.0 | registry; 70206fc6890eaca9fde8a0bf71caa2ddfc9fe045ac9e5c70df101a7dbde866e0 | bytes, http-body-util, hyper, hyper-util, native-tls, tokio, tokio-native-tls, tower-service |
| hyper-util@0.1.17 | registry; 3c6995591a8f1380fcb4ba966a252a4b29188d51d2b89e3a252f5305be65aea8 | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, hyper, ipnet, libc, percent-encoding, pin-project-lite, socket2, system-configuration, tokio, tower-service, tracing, windows-registry |
| iana-time-zone@0.1.64 | registry; 33e57f83510bb73707521ebaffa789ec8caf86f9657cad665b092b581d40e9fb | android_system_properties, core-foundation-sys, iana-time-zone-haiku, js-sys, log, wasm-bindgen, windows-core 0.62.2 |
| iana-time-zone-haiku@0.1.2 | registry; f31827a206f56af32e590ba56d5d2d085f558508192593743f16b2306495269f | cc |
| icu_collections@2.1.1 | registry; 4c6b649701667bbe825c3b7e6388cb521c23d88644678e83c0c4d0a621a34b43 | displaydoc, potential_utf, yoke, zerofrom, zerovec |
| icu_locale_core@2.1.1 | registry; edba7861004dd3714265b4db54a3c390e880ab658fec5f7db895fae2046b5bb6 | displaydoc, litemap, tinystr, writeable, zerovec |
| icu_normalizer@2.1.1 | registry; 5f6c8828b67bf8908d82127b2054ea1b4427ff0230ee9141c54251934ab1b599 | icu_collections, icu_normalizer_data, icu_properties, icu_provider, smallvec, zerovec |
| icu_normalizer_data@2.1.1 | registry; 7aedcccd01fc5fe81e6b489c15b247b8b0690feb23304303a9e560f37efc560a |  |
| icu_properties@2.1.1 | registry; e93fcd3157766c0c8da2f8cff6ce651a31f0810eaa1c51ec363ef790bbb5fb99 | icu_collections, icu_locale_core, icu_properties_data, icu_provider, zerotrie, zerovec |
| icu_properties_data@2.1.1 | registry; 02845b3647bb045f1100ecd6480ff52f34c35f82d9880e029d329c21d1054899 |  |
| icu_provider@2.1.1 | registry; 85962cf0ce02e1e0a629cc34e7ca3e373ce20dda4c4d7294bbd0bf1fdb59e614 | displaydoc, icu_locale_core, writeable, yoke, zerofrom, zerotrie, zerovec |
| ident_case@1.0.1 | registry; b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39 |  |
| idna@1.1.0 | registry; 3b0875f23caa03898994f6ddc501886a45c7d3d62d04d2d90788d47be1b1e4de | idna_adapter, smallvec, utf8_iter |
| idna_adapter@1.2.1 | registry; 3acae9609540aa318d1bc588455225fb2085b9ed0c4f6bd0d9d5bcd86f1a0344 | icu_normalizer, icu_properties |
| indexmap@2.12.0 | registry; 6717a8d2a5a929a1a2eb43a12812498ed141a0bcfb7e8f7844fbdbe4303bba9f | equivalent, hashbrown 0.16.0 |
| inout@0.1.4 | registry; 879f10e63c20629ecabbb64a8010319738c66a5cd0c29b02d63d272b03751d01 | generic-array |
| ipnet@2.11.0 | registry; 469fb0b9cefa57e3ef31275ee7cacb78f2fdca44e4765491884a2b119d4eb130 |  |
| iri-string@0.7.9 | registry; 4f867b9d1d896b67beb18518eda36fdb77a32ea590de864f1325b294a6d14397 | memchr, serde |
| is-docker@0.2.0 | registry; 928bae27f42bc99b60d9ac7334e3a21d10ad8f1835a4e12ec3ec0464765ed1b3 | once_cell |
| is-wsl@0.4.0 | registry; 173609498df190136aa7dea1a91db051746d339e18476eed5ca40521f02d7aa5 | is-docker, once_cell |
| itoa@1.0.15 | registry; 4a5f13b858c8d314ee3e8f639011f7ccefe71f97f96e50151fb991f267928e2c |  |
| js-sys@0.3.82 | registry; b011eec8cc36da2aab2d5cff675ec18454fad408585853910a202391cf9f8e65 | once_cell, wasm-bindgen |
| lazy_static@1.5.0 | registry; bbd2bcb4c963f2ddae06a2efc7e9f3591312473c50c6685e1f298068316e66fe | spin |
| libc@0.2.177 | registry; 2874a2af47a2325c2001a6e6fad9b16a53b802102b528163885171cf92b15976 |  |
| libm@0.2.15 | registry; f9fbbcab51052fe104eb5e5d351cf728d30a5be1fe14d9be8a3b097481fb97de |  |
| librespot-core@0.8.0 | local/path root | aes, base64, byteorder, bytes, data-encoding, flate2, form_urlencoded, futures-core, futures-util, governor, hmac, http, http-body-util, httparse, hyper, hyper-proxy2, hyper-rustls 0.27.7, hyper-tls, hyper-util, librespot-oauth, librespot-protocol, log, nonzero_ext, num-bigint, num-derive, num-integer, num-traits, pbkdf2, pin-project-lite, priority-queue, protobuf, protobuf-json-mapping, quick-xml, rand 0.9.2, rand_distr, rsa, serde, serde_json, sha1, shannon, sysinfo, thiserror 2.0.17, time, tokio, tokio-stream, tokio-tungstenite, tokio-util, url, uuid, vergen-gitcl |
| librespot-oauth@0.8.0 | registry; d686417d49c9d2c363392ffe28d6e469daca20a82dc414740930e078f5829661 | log, oauth2, open, reqwest, thiserror 2.0.17, url |
| librespot-protocol@0.8.0 | registry; 3e01f0b2d39f83fa162eb91d4a16313bcf99e77daf258abe8f7b7bcb1160b084 | protobuf, protobuf-codegen |
| linux-raw-sys@0.4.15 | registry; d26c52dbd32dccf2d10cac7725f8eae5296885fb5703b261f7d0a0739ec807ab |  |
| linux-raw-sys@0.11.0 | registry; df1d3c3b53da64cf5760482273a98e575c651a67eec7f77df96b5b642de8f039 |  |
| litemap@0.8.1 | registry; 6373607a59f0be73a39b6fe456b8192fcc3585f602af20751600e974dd455e77 |  |
| lock_api@0.4.14 | registry; 224399e74b87b5f3557511d98dff8b14089b3dadafcab6bb93eab67d3aace965 | scopeguard |
| log@0.4.28 | registry; 34080505efa8e45a4b816c349525ebe327ceaa8559756f0356cba97ef3bf7432 |  |
| lru-slab@0.1.2 | registry; 112b39cec0b298b6c1999fee3e31427f74f676e4cb9879ed1a121b43661a4154 |  |
| memchr@2.7.6 | registry; f52b00d39961fc5b2736ea853c9cc86238e165017a493d1d5c8eac6bdc4cc273 |  |
| mime@0.3.17 | registry; 6877bb514081ee2a7ff5ef9de3281f14a4dd4bceac4c09388074a6b5df8a139a |  |
| miniz_oxide@0.8.9 | registry; 1fa76a2c86f704bdb222d66965fb3d63269ce38518b83cb0575fca855ebb6316 | adler2, simd-adler32 |
| mio@1.1.0 | registry; 69d83b0086dc8ecf3ce9ae2874b2d1290252e2a30720bea58a5c6639b0092873 | libc, wasi, windows-sys 0.61.2 |
| native-tls@0.2.14 | registry; 87de3442987e9dbec73158d5c715e7ad9072fda936bb03d19d7fa10e00520f0e | libc, log, openssl, openssl-probe, openssl-sys, schannel, security-framework 2.11.1, security-framework-sys, tempfile |
| nonzero_ext@0.3.0 | registry; 38bf9645c8b145698bb0b18a4637dcacbc421ea49bef2317e4fd8065a387cf21 |  |
| ntapi@0.4.1 | registry; e8a3895c6391c39d7fe7ebc444a87eb2991b2a0bc718fdabd071eec617fc68e4 | winapi |
| num-bigint@0.4.6 | registry; a5e44f723f1133c9deac646763579fdb3ac745e418f2a7af9cd0c431da1f20b9 | num-integer, num-traits |
| num-bigint-dig@0.8.5 | registry; 82c79c15c05d4bf82b6f5ef163104cc81a760d8e874d38ac50ab67c8877b647b | lazy_static, libm, num-integer, num-iter, num-traits, rand 0.8.5, smallvec, zeroize |
| num-conv@0.1.0 | registry; 51d515d32fb182ee37cda2ccdcb92950d6a3c2893aa280e540671c2cd0f3b1d9 |  |
| num-derive@0.4.2 | registry; ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202 | proc-macro2, quote, syn |
| num-integer@0.1.46 | registry; 7969661fd2958a5cb096e56c8e1ad0444ac2bbcd0061bd28660485a44879858f | num-traits |
| num-iter@0.1.45 | registry; 1429034a0490724d0075ebb2bc9e875d6503c3cf69e235a8941aa757d83ef5bf | autocfg, num-integer, num-traits |
| num-traits@0.2.19 | registry; 071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841 | autocfg, libm |
| num_threads@0.1.7 | registry; 5c7398b9c8b70908f6371f47ed36737907c87c52af34c268fed0bf0ceb92ead9 | libc |
| oauth2@5.0.0 | registry; 51e219e79014df21a225b1860a479e2dcd7cbd9130f4defd4bd0e191ea31d67d | base64, chrono, getrandom 0.2.16, http, rand 0.8.5, reqwest, serde, serde_json, serde_path_to_error, sha2, thiserror 1.0.69, url |
| objc2-core-foundation@0.3.2 | registry; 2a180dd8642fa45cdb7dd721cd4c11b1cadd4929ce112ebd8b9f5803cc79d536 | bitflags |
| objc2-io-kit@0.3.2 | registry; 33fafba39597d6dc1fb709123dfa8289d39406734be322956a69f0931c73bb15 | libc, objc2-core-foundation |
| once_cell@1.21.3 | registry; 42f5e15c9953c5e4ccceeb2e7382a716482c34515315f7b03532b8b4e8393d2d |  |
| open@5.3.2 | registry; e2483562e62ea94312f3576a7aca397306df7990b8d89033e18766744377ef95 | is-wsl, libc, pathdiff |
| openssl@0.10.75 | registry; 08838db121398ad17ab8531ce9de97b244589089e290a384c900cb9ff7434328 | bitflags, cfg-if, foreign-types, libc, once_cell, openssl-macros, openssl-sys |
| openssl-macros@0.1.1 | registry; a948666b637a0f465e8564c73e89d4dde00d72d4d473cc972f390fc3dcee7d9c | proc-macro2, quote, syn |
| openssl-probe@0.1.6 | registry; d05e27ee213611ffe7d6348b942e8f942b37114c00cc03cec254295a4a17852e |  |
| openssl-sys@0.9.111 | registry; 82cab2d520aa75e3c58898289429321eb788c3106963d0dc886ec7a5f4adc321 | cc, libc, pkg-config, vcpkg |
| parking_lot@0.12.5 | registry; 93857453250e3077bd71ff98b6a65ea6621a19bb0f559a85248955ac12c45a1a | lock_api, parking_lot_core |
| parking_lot_core@0.9.12 | registry; 2621685985a2ebf1c516881c026032ac7deafcda1a2c9b7850dc81e3dfcb64c1 | cfg-if, libc, redox_syscall, smallvec, windows-link 0.2.1 |
| pathdiff@0.2.3 | registry; df94ce210e5bc13cb6651479fa48d14f601d9858cfe0467f43ae157023b938d3 |  |
| pbkdf2@0.12.2 | registry; f8ed6a7761f76e3b9f92dfb0a60a6a6477c61024b775147ff0973a02653abaf2 | digest, hmac |
| pem-rfc7468@0.7.0 | registry; 88b39c9bfcfc231068454382784bb460aae594343fb030d46e9f50a645418412 | base64ct |
| percent-encoding@2.3.2 | registry; 9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220 |  |
| pin-project-lite@0.2.16 | registry; 3b3cff922bd51709b605d9ead9aa71031d81447142d828eb4a6eba76fe619f9b |  |
| pin-utils@0.1.0 | registry; 8b870d8c151b6f2fb93e84a13146138f05d02ed11c7e7c54f8826aaaf7c9f184 |  |
| pkcs1@0.7.5 | registry; c8ffb9f10fa047879315e6625af03c164b16962a5368d724ed16323b68ace47f | der, pkcs8, spki |
| pkcs8@0.10.2 | registry; f950b2377845cebe5cf8b5165cb3cc1a5e0fa5cfa3e1f7f55707d8fd82e0a7b7 | der, spki |
| pkg-config@0.3.32 | registry; 7edddbd0b52d732b21ad9a5fab5c704c14cd949e5e9a1ec5929a24fded1b904c |  |
| portable-atomic@1.11.1 | registry; f84267b20a16ea918e43c6a88433c2d54fa145c92a811b5b047ccbe153674483 |  |
| potential_utf@0.1.4 | registry; b73949432f5e2a09657003c25bca5e19a0e9c84f8058ca374f49e0ebe605af77 | zerovec |
| powerfmt@0.2.0 | registry; 439ee305def115ba05938db6eb1644ff94165c5ab5e9420d1c1bcedbba909391 |  |
| ppv-lite86@0.2.21 | registry; 85eae3c4ed2f50dcfe72643da4befc30deadb458a9b590d720cde2f2b1e97da9 | zerocopy |
| priority-queue@2.7.0 | registry; 93980406f12d9f8140ed5abe7155acb10bb1e69ea55c88960b9c2f117445ef96 | equivalent, indexmap, serde |
| proc-macro2@1.0.103 | registry; 5ee95bc4ef87b8d5ba32e8b7714ccc834865276eab0aed5c9958d00ec45f49e8 | unicode-ident |
| protobuf@3.7.2 | registry; d65a1d4ddae7d8b5de68153b48f6aa3bba8cb002b243dbdbc55a5afbc98f99f4 | once_cell, protobuf-support, thiserror 1.0.69 |
| protobuf-codegen@3.7.2 | registry; 5d3976825c0014bbd2f3b34f0001876604fe87e0c86cd8fa54251530f1544ace | anyhow, once_cell, protobuf, protobuf-parse, regex, tempfile, thiserror 1.0.69 |
| protobuf-json-mapping@3.7.2 | registry; e0d6e4be637b310d8a5c02fa195243328e2d97fa7df1127a27281ef1187fcb1d | protobuf, protobuf-support, thiserror 1.0.69 |
| protobuf-parse@3.7.2 | registry; b4aeaa1f2460f1d348eeaeed86aea999ce98c1bded6f089ff8514c9d9dbdc973 | anyhow, indexmap, log, protobuf, protobuf-support, tempfile, thiserror 1.0.69, which |
| protobuf-support@3.7.2 | registry; 3e36c2f31e0a47f9280fb347ef5e461ffcd2c52dd520d8e216b52f93b0b0d7d6 | thiserror 1.0.69 |
| quick-xml@0.38.3 | registry; 42a232e7487fc2ef313d96dde7948e7a3c05101870d8985e4fd8d26aedd27b89 | memchr, serde |
| quinn@0.11.9 | registry; b9e20a958963c291dc322d98411f541009df2ced7b5a4f2bd52337638cfccf20 | bytes, cfg_aliases, pin-project-lite, quinn-proto, quinn-udp, rustc-hash, rustls 0.23.35, socket2, thiserror 2.0.17, tokio, tracing, web-time |
| quinn-proto@0.11.13 | registry; f1906b49b0c3bc04b5fe5d86a77925ae6524a19b816ae38ce1e426255f1d8a31 | bytes, getrandom 0.3.4, lru-slab, rand 0.9.2, ring, rustc-hash, rustls 0.23.35, rustls-pki-types, slab, thiserror 2.0.17, tinyvec, tracing, web-time |
| quinn-udp@0.5.14 | registry; addec6a0dcad8a8d96a771f815f0eaf55f9d1805756410b39f5fa81332574cbd | cfg_aliases, libc, once_cell, socket2, tracing, windows-sys 0.60.2 |
| quote@1.0.42 | registry; a338cc41d27e6cc6dce6cefc13a0729dfbb81c262b1f519331575dd80ef3067f | proc-macro2 |
| r-efi@5.3.0 | registry; 69cdb34c158ceb288df11e18b4bd39de994f6657d83847bdffdbd7f346754b0f |  |
| rand@0.8.5 | registry; 34af8d1a0e25924bc5b7c43c079c942339d8f0a8b57c39049bef581b46327404 | libc, rand_chacha 0.3.1, rand_core 0.6.4 |
| rand@0.9.2 | registry; 6db2770f06117d490610c7488547d543617b21bfa07796d7a12f6f1bd53850d1 | rand_chacha 0.9.0, rand_core 0.9.3 |
| rand_chacha@0.3.1 | registry; e6c10a63a0fa32252be49d21e7709d4d4baf8d231c2dbce1eaa8141b9b127d88 | ppv-lite86, rand_core 0.6.4 |
| rand_chacha@0.9.0 | registry; d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb | ppv-lite86, rand_core 0.9.3 |
| rand_core@0.6.4 | registry; ec0be4795e2f6a28069bec0b5ff3e2ac9bafc99e6a9a7dc3547996c5c816922c | getrandom 0.2.16 |
| rand_core@0.9.3 | registry; 99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38 | getrandom 0.3.4 |
| rand_distr@0.5.1 | registry; 6a8615d50dcf34fa31f7ab52692afec947c4dd0ab803cc87cb3b0b4570ff7463 | num-traits, rand 0.9.2 |
| redox_syscall@0.5.18 | registry; ed2bf2547551a7053d6fdfafda3f938979645c44812fbfcda098faae3f1a362d | bitflags |
| regex@1.12.2 | registry; 843bc0191f75f3e22651ae5f1e72939ab2f72a4bc30fa80a066bd66edefc24d4 | aho-corasick, memchr, regex-automata, regex-syntax |
| regex-automata@0.4.13 | registry; 5276caf25ac86c8d810222b3dbb938e512c55c6831a10f3e6ed1c93b84041f1c | aho-corasick, memchr, regex-syntax |
| regex-syntax@0.8.8 | registry; 7a2d987857b319362043e95f5353c0535c1f58eec5336fdfcf626430af7def58 |  |
| reqwest@0.12.24 | registry; 9d0946410b9f7b082a427e4ef5c8ff541a88b357bc6c637c40db3a68ac70a36f | base64, bytes, futures-channel, futures-core, futures-util, http, http-body, http-body-util, hyper, hyper-rustls 0.27.7, hyper-tls, hyper-util, js-sys, log, native-tls, percent-encoding, pin-project-lite, quinn, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, serde, serde_json, serde_urlencoded, sync_wrapper, tokio, tokio-native-tls, tokio-rustls 0.26.4, tower, tower-http, tower-service, url, wasm-bindgen, wasm-bindgen-futures, web-sys, webpki-roots 1.0.4 |
| ring@0.17.14 | registry; a4689e6c2294d81e88dc6261c768b63bc4fcdb852be6d1352498b114f61383b7 | cc, cfg-if, getrandom 0.2.16, libc, untrusted, windows-sys 0.52.0 |
| rsa@0.9.8 | registry; 78928ac1ed176a5ca1d17e578a1825f3d81ca54cf41053a592584b020cfd691b | const-oid, digest, num-bigint-dig, num-integer, num-traits, pkcs1, pkcs8, rand_core 0.6.4, signature, spki, subtle, zeroize |
| rustc-hash@2.1.1 | registry; 357703d41365b4b27c590e3ed91eabb1b663f07c4c084095e60cbed4362dff0d |  |
| rustix@0.38.44 | registry; fdb5bc1ae2baa591800df16c9ca78619bf65c0488b41b96ccec5d11220d8c154 | bitflags, errno, libc, linux-raw-sys 0.4.15, windows-sys 0.59.0 |
| rustix@1.1.2 | registry; cd15f8a2c5551a84d56efdc1cd049089e409ac19a3072d5037a17fd70719ff3e | bitflags, errno, libc, linux-raw-sys 0.11.0, windows-sys 0.61.2 |
| rustls@0.22.4 | registry; bf4ef73721ac7bcd79b2b315da7779d8fc09718c6b3d2d1b2d94850eb8c18432 | log, ring, rustls-pki-types, rustls-webpki 0.102.8, subtle, zeroize |
| rustls@0.23.35 | registry; 533f54bc6a7d4f647e46ad909549eda97bf5afc1585190ef692b4286b198bd8f | once_cell, ring, rustls-pki-types, rustls-webpki 0.103.8, subtle, zeroize |
| rustls-native-certs@0.7.3 | registry; e5bfb394eeed242e909609f56089eecfe5fda225042e8b171791b9c95f5931e5 | openssl-probe, rustls-pemfile, rustls-pki-types, schannel, security-framework 2.11.1 |
| rustls-native-certs@0.8.2 | registry; 9980d917ebb0c0536119ba501e90834767bffc3d60641457fd84a1f3fd337923 | openssl-probe, rustls-pki-types, schannel, security-framework 3.5.1 |
| rustls-pemfile@2.2.0 | registry; dce314e5fee3f39953d46bb63bb8a46d40c2f8fb7cc5a3b6cab2bde9721d6e50 | rustls-pki-types |
| rustls-pki-types@1.13.0 | registry; 94182ad936a0c91c324cd46c6511b9510ed16af436d7b5bab34beab0afd55f7a | web-time, zeroize |
| rustls-webpki@0.102.8 | registry; 64ca1bc8749bd4cf37b5ce386cc146580777b4e8572c7b97baf22c83f444bee9 | ring, rustls-pki-types, untrusted |
| rustls-webpki@0.103.8 | registry; 2ffdfa2f5286e2247234e03f680868ac2815974dc39e00ea15adc445d0aafe52 | ring, rustls-pki-types, untrusted |
| rustversion@1.0.22 | registry; b39cdef0fa800fc44525c84ccb54a029961a8215f9619753635a9c0d2538d46d |  |
| ryu@1.0.20 | registry; 28d3b2b1366ec20994f1fd18c3c594f05c5dd4bc44d8bb0c1c632c8d6829481f |  |
| schannel@0.1.28 | registry; 891d81b926048e76efe18581bf793546b4c0eaf8448d72be8de2bbee5fd166e1 | windows-sys 0.61.2 |
| scopeguard@1.2.0 | registry; 94143f37725109f92c262ed2cf5e59bce7498c01bcc1502d7b9afe439a4e9f49 |  |
| security-framework@2.11.1 | registry; 897b2245f0b511c87893af39b033e5ca9cce68824c4d7e7630b5a1d339658d02 | bitflags, core-foundation 0.9.4, core-foundation-sys, libc, security-framework-sys |
| security-framework@3.5.1 | registry; b3297343eaf830f66ede390ea39da1d462b6b0c1b000f420d0a83f898bbbe6ef | bitflags, core-foundation 0.10.1, core-foundation-sys, libc, security-framework-sys |
| security-framework-sys@2.15.0 | registry; cc1f0cbffaac4852523ce30d8bd3c5cdc873501d96ff467ca09b6767bb8cd5c0 | core-foundation-sys, libc |
| serde@1.0.228 | registry; 9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e | serde_core, serde_derive |
| serde_core@1.0.228 | registry; 41d385c7d4ca58e59fc732af25c3983b67ac852c1a25000afe1175de458b67ad | serde_derive |
| serde_derive@1.0.228 | registry; d540f220d3187173da220f885ab66608367b6574e925011a9353e4badda91d79 | proc-macro2, quote, syn |
| serde_json@1.0.145 | registry; 402a6f66d8c709116cf22f558eab210f5a50187f702eb4d7e5ef38d9a7f1c79c | itoa, memchr, ryu, serde, serde_core |
| serde_path_to_error@0.1.20 | registry; 10a9ff822e371bb5403e391ecd83e182e0e77ba7f6fe0160b795797109d1b457 | itoa, serde, serde_core |
| serde_urlencoded@0.7.1 | registry; d3491c14715ca2294c4d6a88f15e84739788c1d030eed8c110436aafdaa2f3fd | form_urlencoded, itoa, ryu, serde |
| sha1@0.10.6 | registry; e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba | cfg-if, cpufeatures, digest |
| sha2@0.10.9 | registry; a7507d819769d01a365ab707794a4084392c824f54a7a6a7862f8c3d0892b283 | cfg-if, cpufeatures, digest |
| shannon@0.2.0 | registry; 7ea5b41c9427b56caa7b808cb548a04fb50bb5b9e98590b53f28064ff4174561 | byteorder |
| shlex@1.3.0 | registry; 0fda2ff0d084019ba4d7c6f371c95d8fd75ce3524c3cb8fb653a3023f6323e64 |  |
| signature@2.2.0 | registry; 77549399552de45a898a580c1b41d445bf730df867cc44e6c0233bbc4b8329de | digest, rand_core 0.6.4 |
| simd-adler32@0.3.7 | registry; d66dc143e6b11c1eddc06d5c423cfc97062865baf299914ab64caa38182078fe |  |
| slab@0.4.11 | registry; 7a2ae44ef20feb57a68b23d846850f861394c2e02dc425a50098ae8c90267589 |  |
| smallvec@1.15.1 | registry; 67b1b7a3b5fe4f1376887184045fcf45c69e92af734b7aaddc05fb777b6fbd03 |  |
| socket2@0.6.1 | registry; 17129e116933cf371d018bb80ae557e889637989d8638274fb25622827b03881 | libc, windows-sys 0.60.2 |
| spin@0.9.8 | registry; 6980e8d7511241f8acf4aebddbb1ff938df5eebe98691418c4468d0b72a96a67 |  |
| spinning_top@0.3.0 | registry; d96d2d1d716fb500937168cc09353ffdc7a012be8475ac7308e1bdf0e3923300 | lock_api |
| spki@0.7.3 | registry; d91ed6c858b01f942cd56b37a94b3e0a1798290327d1236e4d9cf4eaca44d29d | base64ct, der |
| stable_deref_trait@1.2.1 | registry; 6ce2be8dc25455e1f91df71bfa12ad37d7af1092ae736f3a6cd0e37bc7810596 |  |
| strsim@0.11.1 | registry; 7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f |  |
| subtle@2.6.1 | registry; 13c2bddecc57b384dee18652358fb23172facb8a2c51ccc10d74c157bdea3292 |  |
| syn@2.0.109 | registry; 2f17c7e013e88258aa9543dcbe81aca68a667a9ac37cd69c9fbc07858bfe0e2f | proc-macro2, quote, unicode-ident |
| sync_wrapper@1.0.2 | registry; 0bf256ce5efdfa370213c1dabab5935a12e49f2c58d15e9eac2870d3b4f27263 | futures-core |
| synstructure@0.13.2 | registry; 728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2 | proc-macro2, quote, syn |
| sysinfo@0.36.1 | registry; 252800745060e7b9ffb7b2badbd8b31cfa4aa2e61af879d0a3bf2a317c20217d | libc, memchr, ntapi, objc2-core-foundation, objc2-io-kit, windows |
| system-configuration@0.6.1 | registry; 3c879d448e9d986b661742763247d3693ed13609438cf3d006f51f5368a5ba6b | bitflags, core-foundation 0.9.4, system-configuration-sys |
| system-configuration-sys@0.6.0 | registry; 8e1d1b10ced5ca923a1fcb8d03e96b8d3268065d724548c0211415ff6ac6bac4 | core-foundation-sys, libc |
| tempfile@3.23.0 | registry; 2d31c77bdf42a745371d260a26ca7163f1e0924b64afa0b688e61b5a9fa02f16 | fastrand, getrandom 0.3.4, once_cell, rustix 1.1.2, windows-sys 0.61.2 |
| thiserror@1.0.69 | registry; b6aaf5339b578ea85b50e080feb250a3e8ae8cfcdff9a461c9ec2904bc923f52 | thiserror-impl 1.0.69 |
| thiserror@2.0.17 | registry; f63587ca0f12b72a0600bcba1d40081f830876000bb46dd2337a3051618f4fc8 | thiserror-impl 2.0.17 |
| thiserror-impl@1.0.69 | registry; 4fee6c4efc90059e10f81e6d42c60a18f76588c3d74cb83a0b242a2b6c7504c1 | proc-macro2, quote, syn |
| thiserror-impl@2.0.17 | registry; 3ff15c8ecd7de3849db632e14d18d2571fa09dfc5ed93479bc4485c7a517c913 | proc-macro2, quote, syn |
| time@0.3.44 | registry; 91e7d9e3bb61134e77bde20dd4825b97c010155709965fedf0f49bb138e52a9d | deranged, itoa, libc, num-conv, num_threads, powerfmt, serde, time-core, time-macros |
| time-core@0.1.6 | registry; 40868e7c1d2f0b8d73e4a8c7f0ff63af4f6d19be117e90bd73eb1d62cf831c6b |  |
| time-macros@0.2.24 | registry; 30cfb0125f12d9c277f35663a0a33f8c30190f4e4574868a330595412d34ebf3 | num-conv, time-core |
| tinystr@0.8.2 | registry; 42d3e9c45c09de15d06dd8acf5f4e0e399e85927b7f00711024eb7ae10fa4869 | displaydoc, zerovec |
| tinyvec@1.10.0 | registry; bfa5fdc3bce6191a1dbc8c02d5c8bffcf557bafa17c124c5264a458f1b0613fa | tinyvec_macros |
| tinyvec_macros@0.1.1 | registry; 1f3ccbac311fea05f86f61904b462b55fb3df8837a366dfc601a0161d0532f20 |  |
| tokio@1.48.0 | registry; ff360e02eab121e0bc37a2d3b4d4dc622e6eda3a8e5253d5435ecf5bd4c68408 | bytes, libc, mio, pin-project-lite, socket2, tokio-macros, windows-sys 0.61.2 |
| tokio-macros@2.6.0 | registry; af407857209536a95c8e56f8231ef2c2e2aff839b22e07a1ffcbc617e9db9fa5 | proc-macro2, quote, syn |
| tokio-native-tls@0.3.1 | registry; bbae76ab933c85776efabc971569dd6119c580d8f5d448769dec1764bf796ef2 | native-tls, tokio |
| tokio-rustls@0.25.0 | registry; 775e0c0f0adb3a2f22a00c4745d728b479985fc15ee7ca6a2608388c5569860f | rustls 0.22.4, rustls-pki-types, tokio |
| tokio-rustls@0.26.4 | registry; 1729aa945f29d91ba541258c8df89027d5792d85a8841fb65e8bf0f4ede4ef61 | rustls 0.23.35, tokio |
| tokio-stream@0.1.17 | registry; eca58d7bba4a75707817a2c44174253f9236b2d5fbd055602e9d5c07c139a047 | futures-core, pin-project-lite, tokio |
| tokio-tungstenite@0.28.0 | registry; d25a406cddcc431a75d3d9afc6a7c0f7428d4891dd973e4d54c56b46127bf857 | futures-util, log, native-tls, rustls 0.23.35, rustls-native-certs 0.8.2, rustls-pki-types, tokio, tokio-native-tls, tokio-rustls 0.26.4, tungstenite, webpki-roots 0.26.11 |
| tokio-util@0.7.17 | registry; 2efa149fe76073d6e8fd97ef4f4eca7b67f599660115591483572e406e165594 | bytes, futures-core, futures-sink, pin-project-lite, tokio |
| tower@0.5.2 | registry; d039ad9159c98b70ecfd540b2573b97f7f52c3e8d9f8ad57a24b916a536975f9 | futures-core, futures-util, pin-project-lite, sync_wrapper, tokio, tower-layer, tower-service |
| tower-http@0.6.6 | registry; adc82fd73de2a9722ac5da747f12383d2bfdb93591ee6c58486e0097890f05f2 | bitflags, bytes, futures-util, http, http-body, iri-string, pin-project-lite, tower, tower-layer, tower-service |
| tower-layer@0.3.3 | registry; 121c2a6cda46980bb0fcd1647ffaf6cd3fc79a013de288782836f6df9c48780e |  |
| tower-service@0.3.3 | registry; 8df9b6e13f2d32c91b9bd719c00d1958837bc7dec474d94952798cc8e69eeec3 |  |
| tracing@0.1.41 | registry; 784e0ac535deb450455cbfa28a6f0df145ea1bb7ae51b821cf5e7927fdcfbdd0 | pin-project-lite, tracing-core |
| tracing-core@0.1.34 | registry; b9d12581f227e93f094d3af2ae690a574abb8a2b9b7a96e7cfe9647b2b617678 | once_cell |
| try-lock@0.2.5 | registry; e421abadd41a4225275504ea4d6566923418b7f05506fbc9c0fe86ba7396114b |  |
| tungstenite@0.28.0 | registry; 8628dcc84e5a09eb3d8423d6cb682965dea9133204e8fb3efee74c2a0c259442 | bytes, data-encoding, http, httparse, log, native-tls, rand 0.9.2, rustls 0.23.35, rustls-pki-types, sha1, thiserror 2.0.17, utf-8 |
| typenum@1.19.0 | registry; 562d481066bde0658276a35467c4af00bdc6ee726305698a55b86e61d7ad82bb |  |
| unicode-ident@1.0.22 | registry; 9312f7c4f6ff9069b165498234ce8be658059c6728633667c526e27dc2cf1df5 |  |
| untrusted@0.9.0 | registry; 8ecb6da28b8a351d773b68d5825ac39017e680750f980f3a1a85cd8dd28a47c1 |  |
| url@2.5.7 | registry; 08bc136a29a3d1758e07a9cca267be308aeebf5cfd5a10f3f67ab2097683ef5b | form_urlencoded, idna, percent-encoding, serde |
| utf-8@0.7.6 | registry; 09cc8ee72d2a9becf2f2febe0205bbed8fc6615b7cb429ad062dc7b7ddd036a9 |  |
| utf8_iter@1.0.4 | registry; b6c140620e7ffbb22c2dee59cafe6084a59b5ffc27a8859a5f0d494b5d52b6be |  |
| uuid@1.18.1 | registry; 2f87b8aa10b915a06587d0dec516c282ff295b475d94abf425d62b57710070a2 | getrandom 0.3.4 |
| vcpkg@0.2.15 | registry; accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426 |  |
| vergen@9.0.6 | registry; 6b2bf58be11fc9414104c6d3a2e464163db5ef74b12296bda593cac37b6e4777 | anyhow, derive_builder, rustversion, time, vergen-lib |
| vergen-gitcl@1.0.8 | registry; b9dfc1de6eb2e08a4ddf152f1b179529638bedc0ea95e6d667c014506377aefe | anyhow, derive_builder, rustversion, time, vergen, vergen-lib |
| vergen-lib@0.1.6 | registry; 9b07e6010c0f3e59fcb164e0163834597da68d1f864e2b8ca49f74de01e9c166 | anyhow, derive_builder, rustversion |
| version_check@0.9.5 | registry; 0b928f33d975fc6ad9f86c8f283853ad26bdd5b10b7f1542aa2fa15e2289105a |  |
| want@0.3.1 | registry; bfa7760aed19e106de2c7c0b581b509f2f25d3dacaf737cb82ac61bc6d760b0e | try-lock |
| wasi@0.11.1+wasi-snapshot-preview1 | registry; ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b |  |
| wasip2@1.0.1+wasi-0.2.4 | registry; 0562428422c63773dad2c345a1882263bbf4d65cf3f42e90921f787ef5ad58e7 | wit-bindgen |
| wasm-bindgen@0.2.105 | registry; da95793dfc411fbbd93f5be7715b0578ec61fe87cb1a42b12eb625caa5c5ea60 | cfg-if, once_cell, rustversion, wasm-bindgen-macro, wasm-bindgen-shared |
| wasm-bindgen-futures@0.4.55 | registry; 551f88106c6d5e7ccc7cd9a16f312dd3b5d36ea8b4954304657d5dfba115d4a0 | cfg-if, js-sys, once_cell, wasm-bindgen, web-sys |
| wasm-bindgen-macro@0.2.105 | registry; 04264334509e04a7bf8690f2384ef5265f05143a4bff3889ab7a3269adab59c2 | quote, wasm-bindgen-macro-support |
| wasm-bindgen-macro-support@0.2.105 | registry; 420bc339d9f322e562942d52e115d57e950d12d88983a14c79b86859ee6c7ebc | bumpalo, proc-macro2, quote, syn, wasm-bindgen-shared |
| wasm-bindgen-shared@0.2.105 | registry; 76f218a38c84bcb33c25ec7059b07847d465ce0e0a76b995e134a45adcb6af76 | unicode-ident |
| web-sys@0.3.82 | registry; 3a1f95c0d03a47f4ae1f7a64643a6bb97465d9b740f0fa8f90ea33915c99a9a1 | js-sys, wasm-bindgen |
| web-time@1.1.0 | registry; 5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb | js-sys, wasm-bindgen |
| webpki@0.22.4 | registry; ed63aea5ce73d0ff405984102c42de94fc55a6b75765d621c65262469b3c9b53 | ring, untrusted |
| webpki-roots@0.26.11 | registry; 521bc38abb08001b01866da9f51eb7c5d647a19260e00054a8c7fd5f9e57f7a9 | webpki-roots 1.0.4 |
| webpki-roots@1.0.4 | registry; b2878ef029c47c6e8cf779119f20fcf52bde7ad42a731b2a304bc221df17571e | rustls-pki-types |
| which@4.4.2 | registry; 87ba24419a2078cd2b0f2ede2691b6c66d8e47836da3b6db8265ebad47afbfc7 | either, home, once_cell, rustix 0.38.44 |
| winapi@0.3.9 | registry; 5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419 | winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu |
| winapi-i686-pc-windows-gnu@0.4.0 | registry; ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6 |  |
| winapi-x86_64-pc-windows-gnu@0.4.0 | registry; 712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f |  |
| windows@0.61.3 | registry; 9babd3a767a4c1aef6900409f85f5d53ce2544ccdfaa86dad48c91782c6d6893 | windows-collections, windows-core 0.61.2, windows-future, windows-link 0.1.3, windows-numerics |
| windows-collections@0.2.0 | registry; 3beeceb5e5cfd9eb1d76b381630e82c4241ccd0d27f1a39ed41b2760b255c5e8 | windows-core 0.61.2 |
| windows-core@0.61.2 | registry; c0fdd3ddb90610c7638aa2b3a3ab2904fb9e5cdbecc643ddb3647212781c4ae3 | windows-implement, windows-interface, windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-core@0.62.2 | registry; b8e83a14d34d0623b51dce9581199302a221863196a1dde71a7663a4c2be9deb | windows-implement, windows-interface, windows-link 0.2.1, windows-result 0.4.1, windows-strings 0.5.1 |
| windows-future@0.2.1 | registry; fc6a41e98427b19fe4b73c550f060b59fa592d7d686537eebf9385621bfbad8e | windows-core 0.61.2, windows-link 0.1.3, windows-threading |
| windows-implement@0.60.2 | registry; 053e2e040ab57b9dc951b72c264860db7eb3b0200ba345b4e4c3b14f67855ddf | proc-macro2, quote, syn |
| windows-interface@0.59.3 | registry; 3f316c4a2570ba26bbec722032c4099d8c8bc095efccdc15688708623367e358 | proc-macro2, quote, syn |
| windows-link@0.1.3 | registry; 5e6ad25900d524eaabdbbb96d20b4311e1e7ae1699af4fb28c17ae66c80d798a |  |
| windows-link@0.2.1 | registry; f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5 |  |
| windows-numerics@0.2.0 | registry; 9150af68066c4c5c07ddc0ce30421554771e528bde427614c61038bc2c92c2b1 | windows-core 0.61.2, windows-link 0.1.3 |
| windows-registry@0.5.3 | registry; 5b8a9ed28765efc97bbc954883f4e6796c33a06546ebafacbabee9696967499e | windows-link 0.1.3, windows-result 0.3.4, windows-strings 0.4.2 |
| windows-result@0.3.4 | registry; 56f42bd332cc6c8eac5af113fc0c1fd6a8fd2aa08a0119358686e5160d0586c6 | windows-link 0.1.3 |
| windows-result@0.4.1 | registry; 7781fa89eaf60850ac3d2da7af8e5242a5ea78d1a11c49bf2910bb5a73853eb5 | windows-link 0.2.1 |
| windows-strings@0.4.2 | registry; 56e6c93f3a0c3b36176cb1327a4958a0353d5d166c2a35cb268ace15e91d3b57 | windows-link 0.1.3 |
| windows-strings@0.5.1 | registry; 7837d08f69c77cf6b07689544538e017c1bfcf57e34b4c0ff58e6c2cd3b37091 | windows-link 0.2.1 |
| windows-sys@0.52.0 | registry; 282be5f36a8ce781fad8c8ae18fa3f9beff57ec1b52cb3de0789201425d9a33d | windows-targets 0.52.6 |
| windows-sys@0.59.0 | registry; 1e38bc4d79ed67fd075bcc251a1c39b32a1776bbe92e5bef1f0bf1f8c531853b | windows-targets 0.52.6 |
| windows-sys@0.60.2 | registry; f2f500e4d28234f72040990ec9d39e3a6b950f9f22d3dba18416c35882612bcb | windows-targets 0.53.5 |
| windows-sys@0.61.2 | registry; ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc | windows-link 0.2.1 |
| windows-targets@0.52.6 | registry; 9b724f72796e036ab90c1021d4780d4d3d648aca59e491e6b98e725b84e99973 | windows_aarch64_gnullvm 0.52.6, windows_aarch64_msvc 0.52.6, windows_i686_gnu 0.52.6, windows_i686_gnullvm 0.52.6, windows_i686_msvc 0.52.6, windows_x86_64_gnu 0.52.6, windows_x86_64_gnullvm 0.52.6, windows_x86_64_msvc 0.52.6 |
| windows-targets@0.53.5 | registry; 4945f9f551b88e0d65f3db0bc25c33b8acea4d9e41163edf90dcd0b19f9069f3 | windows-link 0.2.1, windows_aarch64_gnullvm 0.53.1, windows_aarch64_msvc 0.53.1, windows_i686_gnu 0.53.1, windows_i686_gnullvm 0.53.1, windows_i686_msvc 0.53.1, windows_x86_64_gnu 0.53.1, windows_x86_64_gnullvm 0.53.1, windows_x86_64_msvc 0.53.1 |
| windows-threading@0.1.0 | registry; b66463ad2e0ea3bbf808b7f1d371311c80e115c0b71d60efc142cafbcfb057a6 | windows-link 0.1.3 |
| windows_aarch64_gnullvm@0.52.6 | registry; 32a4622180e7a0ec044bb555404c800bc9fd9ec262ec147edd5989ccd0c02cd3 |  |
| windows_aarch64_gnullvm@0.53.1 | registry; a9d8416fa8b42f5c947f8482c43e7d89e73a173cead56d044f6a56104a6d1b53 |  |
| windows_aarch64_msvc@0.52.6 | registry; 09ec2a7bb152e2252b53fa7803150007879548bc709c039df7627cabbd05d469 |  |
| windows_aarch64_msvc@0.53.1 | registry; b9d782e804c2f632e395708e99a94275910eb9100b2114651e04744e9b125006 |  |
| windows_i686_gnu@0.52.6 | registry; 8e9b5ad5ab802e97eb8e295ac6720e509ee4c243f69d781394014ebfe8bbfa0b |  |
| windows_i686_gnu@0.53.1 | registry; 960e6da069d81e09becb0ca57a65220ddff016ff2d6af6a223cf372a506593a3 |  |
| windows_i686_gnullvm@0.52.6 | registry; 0eee52d38c090b3caa76c563b86c3a4bd71ef1a819287c19d586d7334ae8ed66 |  |
| windows_i686_gnullvm@0.53.1 | registry; fa7359d10048f68ab8b09fa71c3daccfb0e9b559aed648a8f95469c27057180c |  |
| windows_i686_msvc@0.52.6 | registry; 240948bc05c5e7c6dabba28bf89d89ffce3e303022809e73deaefe4f6ec56c66 |  |
| windows_i686_msvc@0.53.1 | registry; 1e7ac75179f18232fe9c285163565a57ef8d3c89254a30685b57d83a38d326c2 |  |
| windows_x86_64_gnu@0.52.6 | registry; 147a5c80aabfbf0c7d901cb5895d1de30ef2907eb21fbbab29ca94c5b08b1a78 |  |
| windows_x86_64_gnu@0.53.1 | registry; 9c3842cdd74a865a8066ab39c8a7a473c0778a3f29370b5fd6b4b9aa7df4a499 |  |
| windows_x86_64_gnullvm@0.52.6 | registry; 24d5b23dc417412679681396f2b49f3de8c1473deb516bd34410872eff51ed0d |  |
| windows_x86_64_gnullvm@0.53.1 | registry; 0ffa179e2d07eee8ad8f57493436566c7cc30ac536a3379fdf008f47f6bb7ae1 |  |
| windows_x86_64_msvc@0.52.6 | registry; 589f6da84c646204747d1270a2a5661ea66ed1cced2631d546fdfb155959f9ec |  |
| windows_x86_64_msvc@0.53.1 | registry; d6bbff5f0aada427a1e5a6da5f1f98158182f26556f345ac9e04d36d0ebed650 |  |
| wit-bindgen@0.46.0 | registry; f17a85883d4e6d00e8a97c586de764dabcc06133f7f1d55dce5cdc070ad7fe59 |  |
| writeable@0.6.2 | registry; 9edde0db4769d2dc68579893f2306b26c6ecfbe0ef499b013d731b7b9247e0b9 |  |
| yoke@0.8.1 | registry; 72d6e5c6afb84d73944e5cedb052c4680d5657337201555f9f2a16b7406d4954 | stable_deref_trait, yoke-derive, zerofrom |
| yoke-derive@0.8.1 | registry; b659052874eb698efe5b9e8cf382204678a0086ebf46982b79d6ca3182927e5d | proc-macro2, quote, syn, synstructure |
| zerocopy@0.8.27 | registry; 0894878a5fa3edfd6da3f88c4805f4c8558e2b996227a3d864f47fe11e38282c | zerocopy-derive |
| zerocopy-derive@0.8.27 | registry; 88d2b8d9c68ad2b9e4340d7832716a4d21a22a1154777ad56ea55c51a9cf3831 | proc-macro2, quote, syn |
| zerofrom@0.1.6 | registry; 50cc42e0333e05660c3587f3bf9d0478688e15d870fab3346451ce7f8c9fbea5 | zerofrom-derive |
| zerofrom-derive@0.1.6 | registry; d71e5d6e06ab090c67b5e44993ec16b72dcbaabc526db883a360057678b48502 | proc-macro2, quote, syn, synstructure |
| zeroize@1.8.2 | registry; b97154e67e32c85465826e8bcc1c59429aaaf107c1e4a9e53c8d8ccd5eff88d0 |  |
| zerotrie@0.2.3 | registry; 2a59c17a5562d507e4b54960e8569ebee33bee890c70aa3fe7b97e85a9fd7851 | displaydoc, yoke, zerofrom |
| zerovec@0.11.5 | registry; 6c28719294829477f525be0186d13efa9a3c602f7ec202ca9e353d310fb9a002 | yoke, zerofrom, zerovec-derive |
| zerovec-derive@0.11.2 | registry; eadce39539ca5cb3985590102671f2567e659fca9666581ad3411d59207951f3 | proc-macro2, quote, syn |

## pnpm-lock.yaml

{"lockVersion": "9.0", "settings": {"autoInstallPeers": true, "excludeLinksFromLockfile": false}, "importers": ["."], "package_records": 214, "snapshots": 214, "manifest_specifier_mismatches": [], "snapshot_missing_package_roots": [], "deprecated_or_requiresBuild_or_custom_tarball": []}

### Every importer field

```json
{
  ".": {
    "dependencies": {
      "@tanstack/react-query": {
        "specifier": "^5.0.0",
        "version": "5.101.0(react@19.2.7)"
      },
      "@tauri-apps/api": {
        "specifier": "^2",
        "version": "2.11.0"
      },
      "@tauri-apps/plugin-notification": {
        "specifier": "^2.4.0",
        "version": "2.4.0"
      },
      "@tauri-apps/plugin-opener": {
        "specifier": "^2",
        "version": "2.5.4"
      },
      "@tauri-apps/plugin-process": {
        "specifier": "^2",
        "version": "2.3.1"
      },
      "@tauri-apps/plugin-updater": {
        "specifier": "^2",
        "version": "2.10.1"
      },
      "framer-motion": {
        "specifier": "^12.40.0",
        "version": "12.40.0(react-dom@19.2.7(react@19.2.7))(react@19.2.7)"
      },
      "pinyin-pro": {
        "specifier": "^3.28.1",
        "version": "3.28.1"
      },
      "react": {
        "specifier": "^19.1.0",
        "version": "19.2.7"
      },
      "react-dom": {
        "specifier": "^19.1.0",
        "version": "19.2.7(react@19.2.7)"
      },
      "react-router-dom": {
        "specifier": "^6.26.0",
        "version": "6.30.4(react-dom@19.2.7(react@19.2.7))(react@19.2.7)"
      },
      "wanakana": {
        "specifier": "^5.3.1",
        "version": "5.3.1"
      },
      "zustand": {
        "specifier": "^5.0.0",
        "version": "5.0.14(@types/react@19.2.17)(react@19.2.7)"
      }
    },
    "devDependencies": {
      "@tailwindcss/vite": {
        "specifier": "^4.0.0",
        "version": "4.3.1(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0))"
      },
      "@tauri-apps/cli": {
        "specifier": "^2",
        "version": "2.11.2"
      },
      "@types/react": {
        "specifier": "^19.1.8",
        "version": "19.2.17"
      },
      "@types/react-dom": {
        "specifier": "^19.1.6",
        "version": "19.2.3(@types/react@19.2.17)"
      },
      "@vitejs/plugin-react": {
        "specifier": "^4.6.0",
        "version": "4.7.0(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0))"
      },
      "tailwindcss": {
        "specifier": "^4.0.0",
        "version": "4.3.1"
      },
      "typescript": {
        "specifier": "~5.8.3",
        "version": "5.8.3"
      },
      "vite": {
        "specifier": "^7.0.4",
        "version": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)"
      },
      "vitest": {
        "specifier": "^3.0.0",
        "version": "3.2.6(jiti@2.7.0)(lightningcss@1.32.0)"
      }
    }
  }
}
```

### Every package field and snapshot edge

#### packages

| Resolved identity | Complete record |
| --- | --- |
| @babel/code-frame@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-Aup7aUOfpbAUg2ROOJN6Iw5f9DMBlzu0mIkm/malLQFN/YQgO48wCj0Kxa3sEHJvPVFg7siR+qRInwXd2qhQKw=="}}` |
| @babel/compat-data@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-locTkQyKvwIEgBzVrn8693ebc97F2U8ZHjbXwDXJ5Fn2TCpNwTlKcaKLkdHop5c/icOFE7qt7Q9JC5hnKNa6Gg=="}}` |
| @babel/core@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-RgHBCvtjbOK2gXSNBNIkNoEc9qoVEtau3hj8gEqKQuL3HZAibKarWFEI3Lfm6EYKkLalOh8eSrj9b+ch9H/VBA=="}}` |
| @babel/generator@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-DkXD5OJQaAQIdZ1bt3UZdEnHAn9Imd3IVBdX03UFe+ony9Ojw5pzr9YVKGDY1jt+Gcn/FnGkNf8r+Vj5NOJWtQ=="}}` |
| @babel/helper-compilation-targets@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-wem6WaBj4NaVYVdNhLPPVacES6ZJ+KBBfSkTMD3YZxbP3rm3Di85tJU5ljaUNhaOynt+Aj0xruhYuzQBt8n71g=="}}` |
| @babel/helper-globals@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-3nQVUAtvkKH9zahfWgw96Jc/uFOmjACE1kQz82E2lqWmHBgjzbNlsC22nuQTfahmWeQtTq5nQ/4Nnd2A1wj4zA=="}}` |
| @babel/helper-module-imports@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-ejHwrQQYcm9xnTivShn2IDOlIzInN34AXskvq9QicvCtEzq1Vzclu/tKF8Jq1Cg8JG2GL6/EmjgsCT7lXepE3g=="}}` |
| @babel/helper-module-transforms@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "peerDependencies": {"@babel/core": "^7.0.0"}, "resolution": {"integrity": "sha512-UPUVSyXbOh627KiCIGQSgwWzGeBKLkaJ9PJEdrngIwMSzxLR4jS4+f1f1jb7VzBbg8nFLaYotvVPFCTqdrmTAg=="}}` |
| @babel/helper-plugin-utils@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-G7sHYigPY17oO5SYWnfD/0MTBwVR781S/JI643e/JhUYgVgWE/61SoW3NH9KWUKyKq5LVh3npif99Wkt6j86Jw=="}}` |
| @babel/helper-string-parser@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-Pb5ijPrZ89GDH8223L4UP8i6QApWxs04RbPQJTeWDV0/keR2E36MeKnyr6LYmUUvqRRI+Iv87SuF1W6ErINzYw=="}}` |
| @babel/helper-validator-identifier@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-qehxGkRj55h/ff8EMaJ+cYhyaKlHIxqYDn682wQD7RNp9UujOQsHog2uS0r2vzr4pW+sXf90NeeayjcNaX3fFg=="}}` |
| @babel/helper-validator-option@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-N9ZErrD+yW5geCDtBqnOoxmR8+tNKiGuxKlDpuJxfsqpa2dFcexaziGAE/qoHLiDDreVNMupxGmSoNlyvsA3gw=="}}` |
| @babel/helpers@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-1k2lAGRMfHTcwuNYcCNUmaUffmQv8KWMfh2iJUUeRlwlwH4FdNG7mfPI10NPfLHJFThE4Tyr4mv7kTNZOiPuBg=="}}` |
| @babel/parser@7.29.7 | `{"engines": {"node": ">=6.0.0"}, "hasBin": true, "resolution": {"integrity": "sha512-hnORnjP/1P/zFEndoeX+n+t1RwWRJiJpM/jO7FW32Kn9r5+sJB2JWOdYo4L6k78j15eCwY3Gm/7364B1EMwtNg=="}}` |
| @babel/plugin-transform-react-jsx-self@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "peerDependencies": {"@babel/core": "^7.0.0-0"}, "resolution": {"integrity": "sha512-TL0hMc9xzy86VD31nUiwzd5otRAcyEPcsegCxolO0PvcXuH1v0kECe/UIznYFihpkvU5wg/jk4v0TTEFfm53fw=="}}` |
| @babel/plugin-transform-react-jsx-source@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "peerDependencies": {"@babel/core": "^7.0.0-0"}, "resolution": {"integrity": "sha512-06IyK09H3wi4cGbhDBwp5gUGo0IKtnYa8tyTiephirPCK6fbobVGiXMMI5zLQ4aKEYP3wZ3ArU44o+8KMrSG/Q=="}}` |
| @babel/template@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-puq+Gf35oI24FeN11LkoUQFqv9uwNeWpxXZi/Ji3rRIoKAzKnxRaZ+Gkj0vKS9ZCiTESfng1N9LyOyXvo+m+Gg=="}}` |
| @babel/traverse@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-EhlfNQtZ+NK22w5BM61ciuiq1m58ed33Wr1Xan//ZRTy6hgjnwyCffRYwzsGXdASJSUJ1guZILsErh1eQcl+zw=="}}` |
| @babel/types@7.29.7 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-4zBIxpPzowiZpusoFkyGVwakdRJUyuH5PxQ/PrqghfdFWWasvnCdPfQXHrenDai+gyLARulZjZowCOj6fjT4pA=="}}` |
| @esbuild/aix-ppc64@0.27.7 | `{"cpu": ["ppc64"], "engines": {"node": ">=18"}, "os": ["aix"], "resolution": {"integrity": "sha512-EKX3Qwmhz1eMdEJokhALr0YiD0lhQNwDqkPYyPhiSwKrh7/4KRjQc04sZ8db+5DVVnZ1LmbNDI1uAMPEUBnQPg=="}}` |
| @esbuild/android-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["android"], "resolution": {"integrity": "sha512-62dPZHpIXzvChfvfLJow3q5dDtiNMkwiRzPylSCfriLvZeq0a1bWChrGx/BbUbPwOrsWKMn8idSllklzBy+dgQ=="}}` |
| @esbuild/android-arm@0.27.7 | `{"cpu": ["arm"], "engines": {"node": ">=18"}, "os": ["android"], "resolution": {"integrity": "sha512-jbPXvB4Yj2yBV7HUfE2KHe4GJX51QplCN1pGbYjvsyCZbQmies29EoJbkEc+vYuU5o45AfQn37vZlyXy4YJ8RQ=="}}` |
| @esbuild/android-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["android"], "resolution": {"integrity": "sha512-x5VpMODneVDb70PYV2VQOmIUUiBtY3D3mPBG8NxVk5CogneYhkR7MmM3yR/uMdITLrC1ml/NV1rj4bMJuy9MCg=="}}` |
| @esbuild/darwin-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["darwin"], "resolution": {"integrity": "sha512-5lckdqeuBPlKUwvoCXIgI2D9/ABmPq3Rdp7IfL70393YgaASt7tbju3Ac+ePVi3KDH6N2RqePfHnXkaDtY9fkw=="}}` |
| @esbuild/darwin-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["darwin"], "resolution": {"integrity": "sha512-rYnXrKcXuT7Z+WL5K980jVFdvVKhCHhUwid+dDYQpH+qu+TefcomiMAJpIiC2EM3Rjtq0sO3StMV/+3w3MyyqQ=="}}` |
| @esbuild/freebsd-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["freebsd"], "resolution": {"integrity": "sha512-B48PqeCsEgOtzME2GbNM2roU29AMTuOIN91dsMO30t+Ydis3z/3Ngoj5hhnsOSSwNzS+6JppqWsuhTp6E82l2w=="}}` |
| @esbuild/freebsd-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["freebsd"], "resolution": {"integrity": "sha512-jOBDK5XEjA4m5IJK3bpAQF9/Lelu/Z9ZcdhTRLf4cajlB+8VEhFFRjWgfy3M1O4rO2GQ/b2dLwCUGpiF/eATNQ=="}}` |
| @esbuild/linux-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-RZPHBoxXuNnPQO9rvjh5jdkRmVizktkT7TCDkDmQ0W2SwHInKCAV95GRuvdSvA7w4VMwfCjUiPwDi0ZO6Nfe9A=="}}` |
| @esbuild/linux-arm@0.27.7 | `{"cpu": ["arm"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-RkT/YXYBTSULo3+af8Ib0ykH8u2MBh57o7q/DAs3lTJlyVQkgQvlrPTnjIzzRPQyavxtPtfg0EopvDyIt0j1rA=="}}` |
| @esbuild/linux-ia32@0.27.7 | `{"cpu": ["ia32"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-GA48aKNkyQDbd3KtkplYWT102C5sn/EZTY4XROkxONgruHPU72l+gW+FfF8tf2cFjeHaRbWpOYa/uRBz/Xq1Pg=="}}` |
| @esbuild/linux-loong64@0.27.7 | `{"cpu": ["loong64"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-a4POruNM2oWsD4WKvBSEKGIiWQF8fZOAsycHOt6JBpZ+JN2n2JH9WAv56SOyu9X5IqAjqSIPTaJkqN8F7XOQ5Q=="}}` |
| @esbuild/linux-mips64el@0.27.7 | `{"cpu": ["mips64el"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-KabT5I6StirGfIz0FMgl1I+R1H73Gp0ofL9A3nG3i/cYFJzKHhouBV5VWK1CSgKvVaG4q1RNpCTR2LuTVB3fIw=="}}` |
| @esbuild/linux-ppc64@0.27.7 | `{"cpu": ["ppc64"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-gRsL4x6wsGHGRqhtI+ifpN/vpOFTQtnbsupUF5R5YTAg+y/lKelYR1hXbnBdzDjGbMYjVJLJTd2OFmMewAgwlQ=="}}` |
| @esbuild/linux-riscv64@0.27.7 | `{"cpu": ["riscv64"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-hL25LbxO1QOngGzu2U5xeXtxXcW+/GvMN3ejANqXkxZ/opySAZMrc+9LY/WyjAan41unrR3YrmtTsUpwT66InQ=="}}` |
| @esbuild/linux-s390x@0.27.7 | `{"cpu": ["s390x"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-2k8go8Ycu1Kb46vEelhu1vqEP+UeRVj2zY1pSuPdgvbd5ykAw82Lrro28vXUrRmzEsUV0NzCf54yARIK8r0fdw=="}}` |
| @esbuild/linux-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["linux"], "resolution": {"integrity": "sha512-hzznmADPt+OmsYzw1EE33ccA+HPdIqiCRq7cQeL1Jlq2gb1+OyWBkMCrYGBJ+sxVzve2ZJEVeePbLM2iEIZSxA=="}}` |
| @esbuild/netbsd-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["netbsd"], "resolution": {"integrity": "sha512-b6pqtrQdigZBwZxAn1UpazEisvwaIDvdbMbmrly7cDTMFnw/+3lVxxCTGOrkPVnsYIosJJXAsILG9XcQS+Yu6w=="}}` |
| @esbuild/netbsd-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["netbsd"], "resolution": {"integrity": "sha512-OfatkLojr6U+WN5EDYuoQhtM+1xco+/6FSzJJnuWiUw5eVcicbyK3dq5EeV/QHT1uy6GoDhGbFpprUiHUYggrw=="}}` |
| @esbuild/openbsd-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["openbsd"], "resolution": {"integrity": "sha512-AFuojMQTxAz75Fo8idVcqoQWEHIXFRbOc1TrVcFSgCZtQfSdc1RXgB3tjOn/krRHENUB4j00bfGjyl2mJrU37A=="}}` |
| @esbuild/openbsd-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["openbsd"], "resolution": {"integrity": "sha512-+A1NJmfM8WNDv5CLVQYJ5PshuRm/4cI6WMZRg1by1GwPIQPCTs1GLEUHwiiQGT5zDdyLiRM/l1G0Pv54gvtKIg=="}}` |
| @esbuild/openharmony-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["openharmony"], "resolution": {"integrity": "sha512-+KrvYb/C8zA9CU/g0sR6w2RBw7IGc5J2BPnc3dYc5VJxHCSF1yNMxTV5LQ7GuKteQXZtspjFbiuW5/dOj7H4Yw=="}}` |
| @esbuild/sunos-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["sunos"], "resolution": {"integrity": "sha512-ikktIhFBzQNt/QDyOL580ti9+5mL/YZeUPKU2ivGtGjdTYoqz6jObj6nOMfhASpS4GU4Q/Clh1QtxWAvcYKamA=="}}` |
| @esbuild/win32-arm64@0.27.7 | `{"cpu": ["arm64"], "engines": {"node": ">=18"}, "os": ["win32"], "resolution": {"integrity": "sha512-7yRhbHvPqSpRUV7Q20VuDwbjW5kIMwTHpptuUzV+AA46kiPze5Z7qgt6CLCK3pWFrHeNfDd1VKgyP4O+ng17CA=="}}` |
| @esbuild/win32-ia32@0.27.7 | `{"cpu": ["ia32"], "engines": {"node": ">=18"}, "os": ["win32"], "resolution": {"integrity": "sha512-SmwKXe6VHIyZYbBLJrhOoCJRB/Z1tckzmgTLfFYOfpMAx63BJEaL9ExI8x7v0oAO3Zh6D/Oi1gVxEYr5oUCFhw=="}}` |
| @esbuild/win32-x64@0.27.7 | `{"cpu": ["x64"], "engines": {"node": ">=18"}, "os": ["win32"], "resolution": {"integrity": "sha512-56hiAJPhwQ1R4i+21FVF7V8kSD5zZTdHcVuRFMW0hn753vVfQN8xlx4uOPT4xoGH0Z/oVATuR82AiqSTDIpaHg=="}}` |
| @jridgewell/gen-mapping@0.3.13 | `{"resolution": {"integrity": "sha512-2kkt/7niJ6MgEPxF0bYdQ6etZaA+fQvDcLKckhy1yIQOzaoKjBBjSj63/aLVjYE3qhRt5dvM+uUyfCg6UKCBbA=="}}` |
| @jridgewell/remapping@2.3.5 | `{"resolution": {"integrity": "sha512-LI9u/+laYG4Ds1TDKSJW2YPrIlcVYOwi2fUC6xB43lueCjgxV4lffOCZCtYFiH6TNOX+tQKXx97T4IKHbhyHEQ=="}}` |
| @jridgewell/resolve-uri@3.1.2 | `{"engines": {"node": ">=6.0.0"}, "resolution": {"integrity": "sha512-bRISgCIjP20/tbWSPWMEi54QVPRZExkuD9lJL+UIxUKtwVJA8wW1Trb1jMs1RFXo1CBTNZ/5hpC9QvmKWdopKw=="}}` |
| @jridgewell/sourcemap-codec@1.5.5 | `{"resolution": {"integrity": "sha512-cYQ9310grqxueWbl+WuIUIaiUaDcj7WOq5fVhEljNVgRfOUhY9fy2zTvfoqWsnebh8Sl70VScFbICvJnLKB0Og=="}}` |
| @jridgewell/trace-mapping@0.3.31 | `{"resolution": {"integrity": "sha512-zzNR+SdQSDJzc8joaeP8QQoCQr8NuYx2dIIytl1QeBEZHJ9uW6hebsrYgbz8hJwUQao3TWCMtmfV8Nu1twOLAw=="}}` |
| @remix-run/router@1.23.3 | `{"engines": {"node": ">=14.0.0"}, "resolution": {"integrity": "sha512-4An71tdz9X8+3sI4Qqqd2LWd9vS39J7sqd9EU4Scw7TJE/qB10Flv/UuqbPVgfQV9XoK8Np6jNquZitnZq5i+Q=="}}` |
| @rolldown/pluginutils@1.0.0-beta.27 | `{"resolution": {"integrity": "sha512-+d0F4MKMCbeVUJwG96uQ4SgAznZNSq93I3V+9NHA4OpvqG8mRCpGdKmK8l/dl02h2CCDHwW2FqilnTyDcAnqjA=="}}` |
| @rollup/rollup-android-arm-eabi@4.62.0 | `{"cpu": ["arm"], "os": ["android"], "resolution": {"integrity": "sha512-IPIQ55ythEHkfEd9jMEi32OQ7SxURsGA43JI22lj01OLZNt2NUbJX8YUHxkVWyQ6daHPNn0truF5nSj3DQp6YQ=="}}` |
| @rollup/rollup-android-arm64@4.62.0 | `{"cpu": ["arm64"], "os": ["android"], "resolution": {"integrity": "sha512-M6s9cr10MibETyo8JsOkq+Lo1+lU6hcvb1MApnUql5qte/5hMEgzlN8/ReIKNfRV8rrqX50W1BX9zoUhC192RA=="}}` |
| @rollup/rollup-darwin-arm64@4.62.0 | `{"cpu": ["arm64"], "os": ["darwin"], "resolution": {"integrity": "sha512-BqCoMoIbn0keKys+dEAdBa70EtOwV1bEsQCUgU9FdiZmmMge/Zk7LlkYGqbrdHR+Frnt0E1FOanly+rlwvvQzw=="}}` |
| @rollup/rollup-darwin-x64@4.62.0 | `{"cpu": ["x64"], "os": ["darwin"], "resolution": {"integrity": "sha512-SIMzST3VFNXDAbeIWDWiFCNM5qncUBDWaEV7NfE7oZbDt2mgfW4MvbKdbYiGOLoM32gbTv608UMd0XktEYSD7w=="}}` |
| @rollup/rollup-freebsd-arm64@4.62.0 | `{"cpu": ["arm64"], "os": ["freebsd"], "resolution": {"integrity": "sha512-ezjfSQMP7ArdUsbBwbQIfwAlhE84I2iVnzQNCFSveqV42q+BmKlzVpf7mxv5EchLcoWU4y6/heFzVg1F+hodUQ=="}}` |
| @rollup/rollup-freebsd-x64@4.62.0 | `{"cpu": ["x64"], "os": ["freebsd"], "resolution": {"integrity": "sha512-9+qTWGW9AZRhnUgwtTwzNwcPlL87ngkeN0LA+q1bADvmY9aNvWaF2TFW8BZgnQPYxpDI7+rMVLivcd4V737TAQ=="}}` |
| @rollup/rollup-linux-arm-gnueabihf@4.62.0 | `{"cpu": ["arm"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-T1dMEQhXA/jkJ/jyMIw9IovK8bSUq7A8kLIlvZTb/6YIVsp2zLavr4F3oyllHWo7eIVJRyE5n3tUjQJEbE1IuQ=="}}` |
| @rollup/rollup-linux-arm-musleabihf@4.62.0 | `{"cpu": ["arm"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-2as0LgT7qQpyceQq6VUJYnumUMUrgGQCWIiDIN9DE0/tglsk6o66uCB4f3djRawAltvfCNLyZZrsqbPA6inCsA=="}}` |
| @rollup/rollup-linux-arm64-gnu@4.62.0 | `{"cpu": ["arm64"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-bVURMg+6eNN9C/yc0aVjooZcwTTtYF4YW3xta5pP0//r3o1V8gXEHXWCndj47w/HhwsFroZrFhR+6uQP5T0n0g=="}}` |
| @rollup/rollup-linux-arm64-musl@4.62.0 | `{"cpu": ["arm64"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-Ful8pM/2yYI83PViWdFdpZhdI8HJ5qsXANe5atypbHDf+KIBBDsZsbyy8hbXnULVvW9NsTh5DHwbcBftyLTfiw=="}}` |
| @rollup/rollup-linux-loong64-gnu@4.62.0 | `{"cpu": ["loong64"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-9Gp/DgrkzfUBmNPVTyPTvay+4xEP7M/clXpj3efXBcm6uTIVIgDg4rqUpqKXvLEuFRVuEpSAOkhgNeecvaZ4Cg=="}}` |
| @rollup/rollup-linux-loong64-musl@4.62.0 | `{"cpu": ["loong64"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-m9tsJz54LUXkSYM8+8PG81B9IKK5r+2T0clMq4QrS16xFosufU7firBDAZEsDheDs7wTlP7h3++S7lMsU955HA=="}}` |
| @rollup/rollup-linux-ppc64-gnu@4.62.0 | `{"cpu": ["ppc64"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-3UvJ5PNVU16aJf6M3tFI24pWzAl2/ynfbyRN3ICyQajK1lSkrnVYNnLz3v04J32qKa0FczJc22zeToc0lr2A3w=="}}` |
| @rollup/rollup-linux-ppc64-musl@4.62.0 | `{"cpu": ["ppc64"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-vRWUAbYLGHBZS6Q8Msb2sfnf1fvJf+47t8l/TwOerM2qArzy+IeNMTHrYLHXh95h8MoatPHI5hhSZNs+mGXKPg=="}}` |
| @rollup/rollup-linux-riscv64-gnu@4.62.0 | `{"cpu": ["riscv64"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-c00T5SYENHAt86cfW47URaP3Us5vLC/4QO7GYud1G5VNRffCwwCuBspwqYrriuJB+5m0WFzClCn9wed0FBjKvg=="}}` |
| @rollup/rollup-linux-riscv64-musl@4.62.0 | `{"cpu": ["riscv64"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-krrCDilhXOwFkSkO3Wm9I/f9H0L92XHHwy2fwxjukxIbh0dem8gZqOW5Y8BsHrpJv5qwlRBV+Wl4ZFyRWhUpwg=="}}` |
| @rollup/rollup-linux-s390x-gnu@4.62.0 | `{"cpu": ["s390x"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-7pfYFSTc4/rUC/FtAI0Qp6QthDBCIi6/AuP1xYqFk5vanI6KnL5dWKP60OM/05LOsbwTmIcvr6eXC4CJuJ75IA=="}}` |
| @rollup/rollup-linux-x64-gnu@4.62.0 | `{"cpu": ["x64"], "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-7SDIalKeIpG0Ifogbbdn58HmSotYMlf23K3dCJEmiVd9Fg36Vmni82iPQec27N3wY4Bvbxftkxz6vSx9OcouTg=="}}` |
| @rollup/rollup-linux-x64-musl@4.62.0 | `{"cpu": ["x64"], "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-eRZevouTH2i1HeAVLqJuLnt256krQkGY0TN6WsTmsIhuzbh457HuWDMakKwmi0Cjadux983CoSr8Lim2QhUIFw=="}}` |
| @rollup/rollup-openbsd-x64@4.62.0 | `{"cpu": ["x64"], "os": ["openbsd"], "resolution": {"integrity": "sha512-3oVS7FLGa4U1qcvao9ylGxrjXZyUQqR8UwxEcnUEyPX53O/C/mKDZegNXTdHCP+h3e6ta/f1EN38Yif1mmZHYg=="}}` |
| @rollup/rollup-openharmony-arm64@4.62.0 | `{"cpu": ["arm64"], "os": ["openharmony"], "resolution": {"integrity": "sha512-yTB9TgfWj5wHe5QgktAgXTLLot1gvEjl1NiPPAUiCs4oPrIWFl5V4nC3GrkNdj9LaAU4s94nVrGbGOCqUpyWsg=="}}` |
| @rollup/rollup-win32-arm64-msvc@4.62.0 | `{"cpu": ["arm64"], "os": ["win32"], "resolution": {"integrity": "sha512-5LOhoaesY3doG1c+ac/2JtgREpKoJr5bUHH8tKY0V8di7+uSV6BwLs2PlR0/yzefGOkR+wE7ZolZphHCsyG5Rw=="}}` |
| @rollup/rollup-win32-ia32-msvc@4.62.0 | `{"cpu": ["ia32"], "os": ["win32"], "resolution": {"integrity": "sha512-yYkWHhmbhRTWTnWos5HC4GcPQfjlzzCNbM9e/+GXrLuaBXYA3qSDR9f0Vgufd5S8yX81U8jPKp7ZnAjZFMtRnw=="}}` |
| @rollup/rollup-win32-x64-gnu@4.62.0 | `{"cpu": ["x64"], "os": ["win32"], "resolution": {"integrity": "sha512-SoTb6lPg25xZlA2ibwQ++ahCCnH+FP0qmEuafMJ4gznZKOlXioKEAeJLgCrqjM98ACziXM9V1amFjICVL4IFoA=="}}` |
| @rollup/rollup-win32-x64-msvc@4.62.0 | `{"cpu": ["x64"], "os": ["win32"], "resolution": {"integrity": "sha512-5L+T1fMX4RIEBoZzT0+sQ0PhTS36NULFmMXtl1TZo44TMAROIMHbZufSOjVWt/Y622BtxgxtaNOokbTDvfsrZA=="}}` |
| @tailwindcss/node@4.3.1 | `{"resolution": {"integrity": "sha512-6NDaqRoAMSXD1mr/RXu0HBvNE9a2n5tHPsxu9XHLws8o4Twes5rBM2205SUUiJ9goAtadrN6xTGX0UDEwp/N4A=="}}` |
| @tailwindcss/oxide-android-arm64@4.3.1 | `{"cpu": ["arm64"], "engines": {"node": ">= 20"}, "os": ["android"], "resolution": {"integrity": "sha512-SVlyf61g374l5cHyg8x9kf5xmLcOaxvOTsbsqDnSsDJaKOEFZ7GCvi84VAVGpxojYOs1+3K6M0UjXfqPU8vmOQ=="}}` |
| @tailwindcss/oxide-darwin-arm64@4.3.1 | `{"cpu": ["arm64"], "engines": {"node": ">= 20"}, "os": ["darwin"], "resolution": {"integrity": "sha512-hVnWLwv+e/l7c4WKyVtHVrIPvYdqWHjRB3MDIqARynzFtnQg85kmQEFCbV9Ja0VVx4xXTIiDWY60Y7iz/iNoDA=="}}` |
| @tailwindcss/oxide-darwin-x64@4.3.1 | `{"cpu": ["x64"], "engines": {"node": ">= 20"}, "os": ["darwin"], "resolution": {"integrity": "sha512-Cf7abu0WVgbhU7ANgPUnSAvm7nCvMweusHb8FnaHlLfv/Caq4GYaEZg7ZImzzmjx4lIAfuS8q+eLIS7A7IzxIg=="}}` |
| @tailwindcss/oxide-freebsd-x64@4.3.1 | `{"cpu": ["x64"], "engines": {"node": ">= 20"}, "os": ["freebsd"], "resolution": {"integrity": "sha512-ZZqzX2Y+GXtXXfqSfpJhDm60OoZfvLHLCgm+J7NVqgHHJjG/m9ugZI77RwTsVd4fnBJuCFP6Ae6kTJb71UdS8g=="}}` |
| @tailwindcss/oxide-linux-arm-gnueabihf@4.3.1 | `{"cpu": ["arm"], "engines": {"node": ">= 20"}, "os": ["linux"], "resolution": {"integrity": "sha512-/Ah/xik0LaMYfv9DZ0S/t4pBlBNYOcqtRwusjgovHkvT8ixueWCLyJjsaF5kQIckjb4IT8Q6K6p/iPmZMixYgg=="}}` |
| @tailwindcss/oxide-linux-arm64-gnu@4.3.1 | `{"cpu": ["arm64"], "engines": {"node": ">= 20"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-gqdFoVJlw444GvpnheZLHmvTzSxI/cOUUh2KSNejQjTcYkW062SVD+En0rUgD+QV91bz1XGIGtt1HJd48xUGbQ=="}}` |
| @tailwindcss/oxide-linux-arm64-musl@4.3.1 | `{"cpu": ["arm64"], "engines": {"node": ">= 20"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-Bwv9KwOvE0VKa86xPFif9b9c3Y1NxOV1P0gLti/IYaWEsQYZXDlxfGEtA8mdDZ7SG3wyNXAWYT5SIn3giL57oA=="}}` |
| @tailwindcss/oxide-linux-x64-gnu@4.3.1 | `{"cpu": ["x64"], "engines": {"node": ">= 20"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-Ymi8O8T15HYQdOUWUtTI6ldN0neHP85FC+Qz32xTcZ7iJXtem/x8ITev0o1e9e5rkqj4lONZfTRLvkmin1+tKg=="}}` |
| @tailwindcss/oxide-linux-x64-musl@4.3.1 | `{"cpu": ["x64"], "engines": {"node": ">= 20"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-M+P/91qJ6uILLw4k2G93GMDRAXj61SMvFQYt39AqvUqYgExXpLL5aepfns7sj4HiAQeolirQF9E0lzRvdf4zPQ=="}}` |
| @tailwindcss/oxide-wasm32-wasi@4.3.1 | `{"bundledDependencies": ["@napi-rs/wasm-runtime", "@emnapi/core", "@emnapi/runtime", "@tybys/wasm-util", "@emnapi/wasi-threads", "tslib"], "cpu": ["wasm32"], "engines": {"node": ">=14.0.0"}, "resolution": {"integrity": "sha512-zsM8uOeqvVGHsAXsJxsT28ttosFahLJKCLOTUBqRAtKnVgGSRitds9T432QiT8b77Yga7JIBkulIRRlJPtYhRA=="}}` |
| @tailwindcss/oxide-win32-arm64-msvc@4.3.1 | `{"cpu": ["arm64"], "engines": {"node": ">= 20"}, "os": ["win32"], "resolution": {"integrity": "sha512-aiNvSq9BsVk8V513lDKlrCFAgf8qBMPZTpgEhInL+NwQqs97mYmupVMrPrgBBSL8Pv/0zXu9MrMF9rMun1ZeNg=="}}` |
| @tailwindcss/oxide-win32-x64-msvc@4.3.1 | `{"cpu": ["x64"], "engines": {"node": ">= 20"}, "os": ["win32"], "resolution": {"integrity": "sha512-xDEyu1rg290472FEGaKHnzyDyh5QH+AlWvsU5hMoMtPpzmKlRI0jaYKCgSHDYtaQWZOYbMaduSyCwFwY4n1HmA=="}}` |
| @tailwindcss/oxide@4.3.1 | `{"engines": {"node": ">= 20"}, "resolution": {"integrity": "sha512-yVPyo8RNkabVr3O2EhHEE0Rewu7YKzc1DhIqfL46LKveFrmu9XbDazNOJY7/GRuvw1h6u3utWnR29H/p5JPlgA=="}}` |
| @tailwindcss/vite@4.3.1 | `{"peerDependencies": {"vite": "^5.2.0 \|\| ^6 \|\| ^7 \|\| ^8"}, "resolution": {"integrity": "sha512-hItDHuIIlEV61R+faXu66s1K36aTurO/Qw0e45Vskz57gXl9pWOT6eg3zmcEui6CZXddbN7zd41bwmvag4JGwQ=="}}` |
| @tanstack/query-core@5.101.0 | `{"resolution": {"integrity": "sha512-cQetA74EB+seWySv1TTKr828TnP0u39m6LykwDXIo84SNortpDkp30TMEjkqtYCNP9c40uT/iwl6MLiufEt0Ow=="}}` |
| @tanstack/react-query@5.101.0 | `{"peerDependencies": {"react": "^18 \|\| ^19"}, "resolution": {"integrity": "sha512-rLlJXSpkqfizLWgkR5+eLeIk0MvTx/meEIR7LRjxic+qxiQP8zVjq7BqQkiCMNLQBlLfuOLqqr6KO5GtrDlmSg=="}}` |
| @tauri-apps/api@2.11.0 | `{"resolution": {"integrity": "sha512-7CinYODhky9lmO23xHnUFv0Xt43fbtWMyxZcLcRBlFkcgXKuEirBvHpmtJ89YMhyeGcq20Wuc47Fa4XjyniywA=="}}` |
| @tauri-apps/cli-darwin-arm64@2.11.2 | `{"cpu": ["arm64"], "engines": {"node": ">= 10"}, "os": ["darwin"], "resolution": {"integrity": "sha512-+4UZzLt+eOAEQCwgd+TqKgyUJMrvx+BgdXLLaqJYmPqzP+nE6YZr/hY6CWLYGQb8jFn99jEkmC6uA3tNvamA1w=="}}` |
| @tauri-apps/cli-darwin-x64@2.11.2 | `{"cpu": ["x64"], "engines": {"node": ">= 10"}, "os": ["darwin"], "resolution": {"integrity": "sha512-VjYYtZUPqDMLutSfJEyxFE3Bz+DPi7c8wC3imckgvciLDZLq4qwKJxBicg0BXGhXjJsl8vKWgWRFNMPELQ+Xyg=="}}` |
| @tauri-apps/cli-linux-arm-gnueabihf@2.11.2 | `{"cpu": ["arm"], "engines": {"node": ">= 10"}, "os": ["linux"], "resolution": {"integrity": "sha512-yMemD6f4i95AQriS8EazyOFzbE34yjnP16i3IOzpHGQvBoy2DjypFMFBq0NtPuITURv/cOGguRtHR5d79/9CSA=="}}` |
| @tauri-apps/cli-linux-arm64-gnu@2.11.2 | `{"cpu": ["arm64"], "engines": {"node": ">= 10"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-cgI91D2wL8GSgoWwZXDqt+DwnuZCP2/bz03QAE4TrhgAKIsrB4hX26W/H1EONPUUNkqrsgeCD0wU6pcNjV/5kw=="}}` |
| @tauri-apps/cli-linux-arm64-musl@2.11.2 | `{"cpu": ["arm64"], "engines": {"node": ">= 10"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-X1rm0BERqAAggtYTESSgXrS3sz4Sb/OiPiz54UqISlXW+GkR3vNIGnsy/lejNmoXGVqri3Q53BCfQiclOIyRPw=="}}` |
| @tauri-apps/cli-linux-riscv64-gnu@2.11.2 | `{"cpu": ["riscv64"], "engines": {"node": ">= 10"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-usbMLJbT3KtkOrBMDVeGYNM35aTHXx38SJSzTMSqqjeUIOQ+iVPjb2yAGNAE+KqmBbAx4FOFIyMeKXx2M/JKGQ=="}}` |
| @tauri-apps/cli-linux-x64-gnu@2.11.2 | `{"cpu": ["x64"], "engines": {"node": ">= 10"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-Ru4gwJKPG0ctVGchRGpRup4Y4lW2SSfFnrbQcyHhCliKy4g8Qz97TrUgCur4CbWyAgKxvGh3SjrkA0LDYzDGiw=="}}` |
| @tauri-apps/cli-linux-x64-musl@2.11.2 | `{"cpu": ["x64"], "engines": {"node": ">= 10"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-eUm7T6clN1MMmNSRQ9gaWsQdyehQx2Gmn5hht/QUlqZQI/qcP2OJK5dnaxqwFzCr2HdsEo9ydxaqcS1oJzMvUw=="}}` |
| @tauri-apps/cli-win32-arm64-msvc@2.11.2 | `{"cpu": ["arm64"], "engines": {"node": ">= 10"}, "os": ["win32"], "resolution": {"integrity": "sha512-HeeZW80jU+gVTOEX4X/hC6NVSAdDVXajwP5fxIZ/3z9WvUC7qrudX2GMTilYq6Dg0e0sk0XgsAJD1hZ5wPBXUA=="}}` |
| @tauri-apps/cli-win32-ia32-msvc@2.11.2 | `{"cpu": ["ia32"], "engines": {"node": ">= 10"}, "os": ["win32"], "resolution": {"integrity": "sha512-YhjQNZcXfbkCLyazSv1nPnJ9iRFE1wm6kc51FDbU10/Dk09io+6PAGMLjkxnX2GdM0qMnDmTjstY8mTDVvtKeA=="}}` |
| @tauri-apps/cli-win32-x64-msvc@2.11.2 | `{"cpu": ["x64"], "engines": {"node": ">= 10"}, "os": ["win32"], "resolution": {"integrity": "sha512-d2JchlFIpZevZVReyqhQOekJmb1UH3rhZ5VX6sH3ty9ETE0TKQavpihvoScUXfKKpW6HZC0MrFGRU0ZtD+w3gA=="}}` |
| @tauri-apps/cli@2.11.2 | `{"engines": {"node": ">= 10"}, "hasBin": true, "resolution": {"integrity": "sha512-bk3HemqvGRoy+5D/dVMUQHKMYLglD0jVnMm/0iGMH6ufZ+p8r14m6BpIixwij3PBvZdvORUp1YifTD8QxVZ1Nw=="}}` |
| @tauri-apps/plugin-notification@2.4.0 | `{"resolution": {"integrity": "sha512-xlJXMcUoKOjNupzDue5wrEsa1wytf+l/2gCAPhafHyP683Y3N7J/8clUWLZ3vpnwkpT2C1zcLMMQFjjecIG2xg=="}}` |
| @tauri-apps/plugin-opener@2.5.4 | `{"resolution": {"integrity": "sha512-1HnPkb+AmgO29HBazm4uPLKB+r7zzcTBW1d0fyYp1uP+jwtpoiNDGKMMzz58SFp49nOIrxdE3aUJtT57lfO9CQ=="}}` |
| @tauri-apps/plugin-process@2.3.1 | `{"resolution": {"integrity": "sha512-nCa4fGVaDL/B9ai03VyPOjfAHRHSBz5v6F/ObsB73r/dA3MHHhZtldaDMIc0V/pnUw9ehzr2iEG+XkSEyC0JJA=="}}` |
| @tauri-apps/plugin-updater@2.10.1 | `{"resolution": {"integrity": "sha512-NFYMg+tWOZPJdzE/PpFj2qfqwAWwNS3kXrb1tm1gnBJ9mYzZ4WDRrwy8udzWoAnfGCHLuePNLY1WVCNHnh3eRA=="}}` |
| @types/babel__core@7.20.5 | `{"resolution": {"integrity": "sha512-qoQprZvz5wQFJwMDqeseRXWv3rqMvhgpbXFfVyWhbx9X47POIA6i/+dXefEmZKoAgOaTdaIgNSMqMIU61yRyzA=="}}` |
| @types/babel__generator@7.27.0 | `{"resolution": {"integrity": "sha512-ufFd2Xi92OAVPYsy+P4n7/U7e68fex0+Ee8gSG9KX7eo084CWiQ4sdxktvdl0bOPupXtVJPY19zk6EwWqUQ8lg=="}}` |
| @types/babel__template@7.4.4 | `{"resolution": {"integrity": "sha512-h/NUaSyG5EyxBIp8YRxo4RMe2/qQgvyowRwVMzhYhBCONbW8PUsg4lkFMrhgZhUe5z3L3MiLDuvyJ/CaPa2A8A=="}}` |
| @types/babel__traverse@7.28.0 | `{"resolution": {"integrity": "sha512-8PvcXf70gTDZBgt9ptxJ8elBeBjcLOAcOtoO/mPJjtji1+CdGbHgm77om1GrsPxsiE+uXIpNSK64UYaIwQXd4Q=="}}` |
| @types/chai@5.2.3 | `{"resolution": {"integrity": "sha512-Mw558oeA9fFbv65/y4mHtXDs9bPnFMZAL/jxdPFUpOHHIXX91mcgEHbS5Lahr+pwZFR8A7GQleRWeI6cGFC2UA=="}}` |
| @types/deep-eql@4.0.2 | `{"resolution": {"integrity": "sha512-c9h9dVVMigMPc4bwTvC5dxqtqJZwQPePsWjPlpSOnojbor6pGqdk541lfA7AqFQr5pB1BRdq0juY9db81BwyFw=="}}` |
| @types/estree@1.0.9 | `{"resolution": {"integrity": "sha512-GhdPgy1el4/ImP05X05Uw4cw2/M93BCUmnEvWZNStlCzEKME4Fkk+YpoA5OiHNQmoS7Cafb8Xa3Pya8m1Qrzeg=="}}` |
| @types/react-dom@19.2.3 | `{"peerDependencies": {"@types/react": "^19.2.0"}, "resolution": {"integrity": "sha512-jp2L/eY6fn+KgVVQAOqYItbF0VY/YApe5Mz2F0aykSO8gx31bYCZyvSeYxCHKvzHG5eZjc+zyaS5BrBWya2+kQ=="}}` |
| @types/react@19.2.17 | `{"resolution": {"integrity": "sha512-MXfmqaVPEVgkBT/aY0aGCkRWWtByiYQXo3xdQ8r5RzuFrPiRn8Gar2tQdXSUQ2GKV3bkXckek89V8wQBY2Q/Aw=="}}` |
| @vitejs/plugin-react@4.7.0 | `{"engines": {"node": "^14.18.0 \|\| >=16.0.0"}, "peerDependencies": {"vite": "^4.2.0 \|\| ^5.0.0 \|\| ^6.0.0 \|\| ^7.0.0"}, "resolution": {"integrity": "sha512-gUu9hwfWvvEDBBmgtAowQCojwZmJ5mcLn3aufeCsitijs3+f2NsrPtlAWIR6OPiqljl96GVCUbLe0HyqIpVaoA=="}}` |
| @vitest/expect@3.2.6 | `{"resolution": {"integrity": "sha512-1+7q9BtaKzEmO+fmNT3kYvoNn5Y71XWAx2Q5HRim4tTVRQVRv4uJFAQ5FbK0OPUeNP/WmVCpxYxoJdvuHVjzBQ=="}}` |
| @vitest/mocker@3.2.6 | `{"peerDependencies": {"msw": "^2.4.9", "vite": "^5.0.0 \|\| ^6.0.0 \|\| ^7.0.0-0"}, "peerDependenciesMeta": {"msw": {"optional": true}, "vite": {"optional": true}}, "resolution": {"integrity": "sha512-EZOrpDbkKotFAP7wPAQV1UIyoGOk4oX7ynWhBhLB7v+meMHbQhU16oPpIYGTTe4oFlhpryGpgpcZP/sin3hYuw=="}}` |
| @vitest/pretty-format@3.2.6 | `{"resolution": {"integrity": "sha512-lb7XXXzmm2h2ASzFnRvQpDo6onT1NmMJA3tkGTWiBFtRJ9lxGY3d3mm/Apt36gej2bkkOVLL/yTOtufDaFa/jA=="}}` |
| @vitest/runner@3.2.6 | `{"resolution": {"integrity": "sha512-HYcoSj1w5tcgUnzoF0HcyaAQjpA1gj9ftUJ7iSJSuipc02jW9gKkigwZbjFldAfYHA1fa8UZVRftdMY5msWM9Q=="}}` |
| @vitest/snapshot@3.2.6 | `{"resolution": {"integrity": "sha512-H+ZjNTWGpObenh0YnlBctAPnJSI20P81PL8BPzWpx54YXLLTm8hEsWawtcYLMrwvpK48hGxLLbCS+1KRXhsKhw=="}}` |
| @vitest/spy@3.2.6 | `{"resolution": {"integrity": "sha512-oq6BbH68WzcWmwtBrU9nqLeaXTR4XwJF7FSLkKEZo4i6eoXcrxjcwSuTvWBIRUTC6VC72nXYunzqgZA+IKdtxg=="}}` |
| @vitest/utils@3.2.6 | `{"resolution": {"integrity": "sha512-lI23nIs4bnT3T8NIoh+vFaz5s2/DdP0Jgt2jxwgWljvwn82cLJtyi/If+fjFyoLMGIOz0U/fKvWE0d4jsNQEfg=="}}` |
| assertion-error@2.0.1 | `{"engines": {"node": ">=12"}, "resolution": {"integrity": "sha512-Izi8RQcffqCeNVgFigKli1ssklIbpHnCYc6AknXGYoB6grJqyeby7jv12JUQgmTAnIDnbck1uxksT4dzN3PWBA=="}}` |
| baseline-browser-mapping@2.10.37 | `{"engines": {"node": ">=6.0.0"}, "hasBin": true, "resolution": {"integrity": "sha512-girxaJ7WZssDOFhzCGZTDKoTa1gk6A1TbflaYTpykLJ4UU9Fz9kx1aREM8JCuoVHbL8X8T/mJg7w2oYSq72Oig=="}}` |
| browserslist@4.28.2 | `{"engines": {"node": "^6 \|\| ^7 \|\| ^8 \|\| ^9 \|\| ^10 \|\| ^11 \|\| ^12 \|\| >=13.7"}, "hasBin": true, "resolution": {"integrity": "sha512-48xSriZYYg+8qXna9kwqjIVzuQxi+KYWp2+5nCYnYKPTr0LvD89Jqk2Or5ogxz0NUMfIjhh2lIUX/LyX9B4oIg=="}}` |
| cac@6.7.14 | `{"engines": {"node": ">=8"}, "resolution": {"integrity": "sha512-b6Ilus+c3RrdDk+JhLKUAQfzzgLEPy6wcXqS7f/xe1EETvsDP6GORG7SFuOs6cID5YkqchW/LXZbX5bc8j7ZcQ=="}}` |
| caniuse-lite@1.0.30001799 | `{"resolution": {"integrity": "sha512-hG1bReV+OUU+MOqK4t/ZWI0tZOyz3rqS9XuhOUz1cIcbwBKjOyJEJuw9ER5JuNyqxNk8u/JUVbGibBOL1yrjFw=="}}` |
| chai@5.3.3 | `{"engines": {"node": ">=18"}, "resolution": {"integrity": "sha512-4zNhdJD/iOjSH0A05ea+Ke6MU5mmpQcbQsSOkgdaUMJ9zTlDTD/GYlwohmIE2u0gaxHYiVHEn1Fw9mZ/ktJWgw=="}}` |
| check-error@2.1.3 | `{"engines": {"node": ">= 16"}, "resolution": {"integrity": "sha512-PAJdDJusoxnwm1VwW07VWwUN1sl7smmC3OKggvndJFadxxDRyFJBX/ggnu/KE4kQAB7a3Dp8f/YXC1FlUprWmA=="}}` |
| convert-source-map@2.0.0 | `{"resolution": {"integrity": "sha512-Kvp459HrV2FEJ1CAsi1Ku+MY3kasH19TFykTz2xWmMeq6bk2NU3XXvfJ+Q61m0xktWwt+1HSYf3JZsTms3aRJg=="}}` |
| csstype@3.2.3 | `{"resolution": {"integrity": "sha512-z1HGKcYy2xA8AGQfwrn0PAy+PB7X/GSj3UVJW9qKyn43xWa+gl5nXmU4qqLMRzWVLFC8KusUX8T/0kCiOYpAIQ=="}}` |
| debug@4.4.3 | `{"engines": {"node": ">=6.0"}, "peerDependencies": {"supports-color": "*"}, "peerDependenciesMeta": {"supports-color": {"optional": true}}, "resolution": {"integrity": "sha512-RGwwWnwQvkVfavKVt22FGLw+xYSdzARwm0ru6DhTVA3umU5hZc28V3kO4stgYryrTlLpuvgI9GiijltAjNbcqA=="}}` |
| deep-eql@5.0.2 | `{"engines": {"node": ">=6"}, "resolution": {"integrity": "sha512-h5k/5U50IJJFpzfL6nO9jaaumfjO/f2NjK/oYB2Djzm4p9L+3T9qWpZqZ2hAbLPuuYq9wrU08WQyBTL5GbPk5Q=="}}` |
| detect-libc@2.1.2 | `{"engines": {"node": ">=8"}, "resolution": {"integrity": "sha512-Btj2BOOO83o3WyH59e8MgXsxEQVcarkUOpEYrubB0urwnN10yQ364rsiByU11nZlqWYZm05i/of7io4mzihBtQ=="}}` |
| electron-to-chromium@1.5.372 | `{"resolution": {"integrity": "sha512-M3yhbAlilnwqC8D21t28UCDGHyitShTmmLRU/H+b74P6Ski16Nb9HONYEaVpMj/pwC7BEo5B95FpjODLCWbtfA=="}}` |
| enhanced-resolve@5.21.6 | `{"engines": {"node": ">=10.13.0"}, "resolution": {"integrity": "sha512-aNnGCvbJ/RIyWo1IuhNdVjnNF+EjH9wpzpNHt+ci/m9He9LJvUN8wrCcXjp9cWsGNAuvSpVFTx/vraAFQ8qGjQ=="}}` |
| es-module-lexer@1.7.0 | `{"resolution": {"integrity": "sha512-jEQoCwk8hyb2AZziIOLhDqpm5+2ww5uIE6lkO/6jcOCusfk6LhMHpXXfBLXTZ7Ydyt0j4VoUQv6uGNYbdW+kBA=="}}` |
| esbuild@0.27.7 | `{"engines": {"node": ">=18"}, "hasBin": true, "resolution": {"integrity": "sha512-IxpibTjyVnmrIQo5aqNpCgoACA/dTKLTlhMHihVHhdkxKyPO1uBBthumT0rdHmcsk9uMonIWS0m4FljWzILh3w=="}}` |
| escalade@3.2.0 | `{"engines": {"node": ">=6"}, "resolution": {"integrity": "sha512-WUj2qlxaQtO4g6Pq5c29GTcWGDyd8itL8zTlipgECz3JesAiiOKotd8JU6otB3PACgG6xkJUyVhboMS+bje/jA=="}}` |
| estree-walker@3.0.3 | `{"resolution": {"integrity": "sha512-7RUKfXgSMMkzt6ZuXmqapOurLGPPfgj6l9uRZ7lRGolvk0y2yocc35LdcxKC5PQZdn2DMqioAQ2NoWcrTKmm6g=="}}` |
| expect-type@1.3.0 | `{"engines": {"node": ">=12.0.0"}, "resolution": {"integrity": "sha512-knvyeauYhqjOYvQ66MznSMs83wmHrCycNEN6Ao+2AeYEfxUIkuiVxdEa1qlGEPK+We3n0THiDciYSsCcgW/DoA=="}}` |
| fdir@6.5.0 | `{"engines": {"node": ">=12.0.0"}, "peerDependencies": {"picomatch": "^3 \|\| ^4"}, "peerDependenciesMeta": {"picomatch": {"optional": true}}, "resolution": {"integrity": "sha512-tIbYtZbucOs0BRGqPJkshJUYdL+SDH7dVM8gjy+ERp3WAUjLEFJE+02kanyHtwjWOnwrKYBiwAmM0p4kLJAnXg=="}}` |
| framer-motion@12.40.0 | `{"peerDependencies": {"@emotion/is-prop-valid": "*", "react": "^18.0.0 \|\| ^19.0.0", "react-dom": "^18.0.0 \|\| ^19.0.0"}, "peerDependenciesMeta": {"@emotion/is-prop-valid": {"optional": true}, "react": {"optional": true}, "react-dom": {"optional": true}}, "resolution": {"integrity": "sha512-uaBd3qC1v3KQqBEjwTUd183K6PbS+j0yR9w9VmEOLWA/tnUcSn8Xa3uck7t4dgpDoUss8xQTcj8W2L07lrnLFg=="}}` |
| fsevents@2.3.3 | `{"engines": {"node": "^8.16.0 \|\| ^10.6.0 \|\| >=11.0.0"}, "os": ["darwin"], "resolution": {"integrity": "sha512-5xoDfX+fL7faATnagmWPpbFtwh/R77WmMMqqHGS65C3vvB0YHrgF+B1YmZ3441tMj5n63k0212XNoJwzlhffQw=="}}` |
| gensync@1.0.0-beta.2 | `{"engines": {"node": ">=6.9.0"}, "resolution": {"integrity": "sha512-3hN7NaskYvMDLQY55gnW3NQ+mesEAepTqlg+VEbj7zzqEMBVNhzcGYYeqFo/TlYz6eQiFcp1HcsCZO+nGgS8zg=="}}` |
| graceful-fs@4.2.11 | `{"resolution": {"integrity": "sha512-RbJ5/jmFcNNCcDV5o9eTnBLJ/HszWV0P73bc+Ff4nS/rJj+YaS6IGyiOL0VoBYX+l1Wrl3k63h/KrH+nhJ0XvQ=="}}` |
| jiti@2.7.0 | `{"hasBin": true, "resolution": {"integrity": "sha512-AC/7JofJvZGrrneWNaEnJeOLUx+JlGt7tNa0wZiRPT4MY1wmfKjt2+6O2p2uz2+skll8OZZmJMNqeke7kKbNgQ=="}}` |
| js-tokens@4.0.0 | `{"resolution": {"integrity": "sha512-RdJUflcE3cUzKiMqQgsCu06FPu9UdIJO0beYbPhHN4k6apgJtifcoCtT9bcxOpYBtpD2kCM6Sbzg4CausW/PKQ=="}}` |
| js-tokens@9.0.1 | `{"resolution": {"integrity": "sha512-mxa9E9ITFOt0ban3j6L5MpjwegGz6lBQmM1IJkWeBZGcMxto50+eWdjC/52xDbS2vy0k7vIMK0Fe2wfL9OQSpQ=="}}` |
| jsesc@3.1.0 | `{"engines": {"node": ">=6"}, "hasBin": true, "resolution": {"integrity": "sha512-/sM3dO2FOzXjKQhJuo0Q173wf2KOo8t4I8vHy6lF9poUp7bKT0/NHE8fPX23PwfhnykfqnC2xRxOnVw5XuGIaA=="}}` |
| json5@2.2.3 | `{"engines": {"node": ">=6"}, "hasBin": true, "resolution": {"integrity": "sha512-XmOWe7eyHYH14cLdVPoyg+GOH3rYX++KpzrylJwSW98t3Nk+U8XOl8FWKOgwtzdb8lXGf6zYwDUzeHMWfxasyg=="}}` |
| lightningcss-android-arm64@1.32.0 | `{"cpu": ["arm64"], "engines": {"node": ">= 12.0.0"}, "os": ["android"], "resolution": {"integrity": "sha512-YK7/ClTt4kAK0vo6w3X+Pnm0D2cf2vPHbhOXdoNti1Ga0al1P4TBZhwjATvjNwLEBCnKvjJc2jQgHXH0NEwlAg=="}}` |
| lightningcss-darwin-arm64@1.32.0 | `{"cpu": ["arm64"], "engines": {"node": ">= 12.0.0"}, "os": ["darwin"], "resolution": {"integrity": "sha512-RzeG9Ju5bag2Bv1/lwlVJvBE3q6TtXskdZLLCyfg5pt+HLz9BqlICO7LZM7VHNTTn/5PRhHFBSjk5lc4cmscPQ=="}}` |
| lightningcss-darwin-x64@1.32.0 | `{"cpu": ["x64"], "engines": {"node": ">= 12.0.0"}, "os": ["darwin"], "resolution": {"integrity": "sha512-U+QsBp2m/s2wqpUYT/6wnlagdZbtZdndSmut/NJqlCcMLTWp5muCrID+K5UJ6jqD2BFshejCYXniPDbNh73V8w=="}}` |
| lightningcss-freebsd-x64@1.32.0 | `{"cpu": ["x64"], "engines": {"node": ">= 12.0.0"}, "os": ["freebsd"], "resolution": {"integrity": "sha512-JCTigedEksZk3tHTTthnMdVfGf61Fky8Ji2E4YjUTEQX14xiy/lTzXnu1vwiZe3bYe0q+SpsSH/CTeDXK6WHig=="}}` |
| lightningcss-linux-arm-gnueabihf@1.32.0 | `{"cpu": ["arm"], "engines": {"node": ">= 12.0.0"}, "os": ["linux"], "resolution": {"integrity": "sha512-x6rnnpRa2GL0zQOkt6rts3YDPzduLpWvwAF6EMhXFVZXD4tPrBkEFqzGowzCsIWsPjqSK+tyNEODUBXeeVHSkw=="}}` |
| lightningcss-linux-arm64-gnu@1.32.0 | `{"cpu": ["arm64"], "engines": {"node": ">= 12.0.0"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-0nnMyoyOLRJXfbMOilaSRcLH3Jw5z9HDNGfT/gwCPgaDjnx0i8w7vBzFLFR1f6CMLKF8gVbebmkUN3fa/kQJpQ=="}}` |
| lightningcss-linux-arm64-musl@1.32.0 | `{"cpu": ["arm64"], "engines": {"node": ">= 12.0.0"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-UpQkoenr4UJEzgVIYpI80lDFvRmPVg6oqboNHfoH4CQIfNA+HOrZ7Mo7KZP02dC6LjghPQJeBsvXhJod/wnIBg=="}}` |
| lightningcss-linux-x64-gnu@1.32.0 | `{"cpu": ["x64"], "engines": {"node": ">= 12.0.0"}, "libc": ["glibc"], "os": ["linux"], "resolution": {"integrity": "sha512-V7Qr52IhZmdKPVr+Vtw8o+WLsQJYCTd8loIfpDaMRWGUZfBOYEJeyJIkqGIDMZPwPx24pUMfwSxxI8phr/MbOA=="}}` |
| lightningcss-linux-x64-musl@1.32.0 | `{"cpu": ["x64"], "engines": {"node": ">= 12.0.0"}, "libc": ["musl"], "os": ["linux"], "resolution": {"integrity": "sha512-bYcLp+Vb0awsiXg/80uCRezCYHNg1/l3mt0gzHnWV9XP1W5sKa5/TCdGWaR/zBM2PeF/HbsQv/j2URNOiVuxWg=="}}` |
| lightningcss-win32-arm64-msvc@1.32.0 | `{"cpu": ["arm64"], "engines": {"node": ">= 12.0.0"}, "os": ["win32"], "resolution": {"integrity": "sha512-8SbC8BR40pS6baCM8sbtYDSwEVQd4JlFTOlaD3gWGHfThTcABnNDBda6eTZeqbofalIJhFx0qKzgHJmcPTnGdw=="}}` |
| lightningcss-win32-x64-msvc@1.32.0 | `{"cpu": ["x64"], "engines": {"node": ">= 12.0.0"}, "os": ["win32"], "resolution": {"integrity": "sha512-Amq9B/SoZYdDi1kFrojnoqPLxYhQ4Wo5XiL8EVJrVsB8ARoC1PWW6VGtT0WKCemjy8aC+louJnjS7U18x3b06Q=="}}` |
| lightningcss@1.32.0 | `{"engines": {"node": ">= 12.0.0"}, "resolution": {"integrity": "sha512-NXYBzinNrblfraPGyrbPoD19C1h9lfI/1mzgWYvXUTe414Gz/X1FD2XBZSZM7rRTrMA8JL3OtAaGifrIKhQ5yQ=="}}` |
| loupe@3.2.1 | `{"resolution": {"integrity": "sha512-CdzqowRJCeLU72bHvWqwRBBlLcMEtIvGrlvef74kMnV2AolS9Y8xUv1I0U/MNAWMhBlKIoyuEgoJ0t/bbwHbLQ=="}}` |
| lru-cache@5.1.1 | `{"resolution": {"integrity": "sha512-KpNARQA3Iwv+jTA0utUVVbrh+Jlrr1Fv0e56GGzAFOXN7dk/FviaDW8LHmK52DlcH4WP2n6gI8vN1aesBFgo9w=="}}` |
| magic-string@0.30.21 | `{"resolution": {"integrity": "sha512-vd2F4YUyEXKGcLHoq+TEyCjxueSeHnFxyyjNp80yg0XV4vUhnDer/lvvlqM/arB5bXQN5K2/3oinyCRyx8T2CQ=="}}` |
| motion-dom@12.40.0 | `{"resolution": {"integrity": "sha512-HxU3ZaBwNPVQUBQf1xxgq+7JrPNZvjLVxgbpEZL7RrWJnsxOf0/OM+yrHG9ogLQ31Do/r57Oz2gQWPK+6q62mg=="}}` |
| motion-utils@12.39.0 | `{"resolution": {"integrity": "sha512-8nadJAJjTtqRkmRF36FoJTrywK9nnFmnPwnSMyxaOCU7GDjN9RTMJIxx9De8ErM+vpPhMccr/6fo5WciyQLnMQ=="}}` |
| ms@2.1.3 | `{"resolution": {"integrity": "sha512-6FlzubTLZG3J2a/NVCAleEhjzq5oxgHyaCU9yYXvcLsvoVaHJq/s5xXI6/XXP6tz7R9xAOtHnSO/tXtF3WRTlA=="}}` |
| nanoid@3.3.12 | `{"engines": {"node": "^10 \|\| ^12 \|\| ^13.7 \|\| ^14 \|\| >=15.0.1"}, "hasBin": true, "resolution": {"integrity": "sha512-ZB9RH/39qpq5Vu6Y+NmUaFhQR6pp+M2Xt76XBnEwDaGcVAqhlvxrl3B2bKS5D3NH3QR76v3aSrKaF/Kiy7lEtQ=="}}` |
| node-releases@2.0.47 | `{"engines": {"node": ">=18"}, "resolution": {"integrity": "sha512-Uzmd6LXpouKo8EUK68IjH4+E01w/hXyV3R3g/geCJo+rXLNfh1xucB+LOzYEOQPSiUK3h/xZf0cQGcSsmyL2Og=="}}` |
| pathe@2.0.3 | `{"resolution": {"integrity": "sha512-WUjGcAqP1gQacoQe+OBJsFA7Ld4DyXuUIjZ5cc75cLHvJ7dtNsTugphxIADwspS+AraAUePCKrSVtPLFj/F88w=="}}` |
| pathval@2.0.1 | `{"engines": {"node": ">= 14.16"}, "resolution": {"integrity": "sha512-//nshmD55c46FuFw26xV/xFAaB5HF9Xdap7HJBBnrKdAd6/GxDBaNA1870O79+9ueg61cZLSVc+OaFlfmObYVQ=="}}` |
| picocolors@1.1.1 | `{"resolution": {"integrity": "sha512-xceH2snhtb5M9liqDsmEw56le376mTZkEX/jEb/RxNFyegNul7eNslCXP9FDj/Lcu0X8KEyMceP2ntpaHrDEVA=="}}` |
| picomatch@4.0.4 | `{"engines": {"node": ">=12"}, "resolution": {"integrity": "sha512-QP88BAKvMam/3NxH6vj2o21R6MjxZUAd6nlwAS/pnGvN9IVLocLHxGYIzFhg6fUQ+5th6P4dv4eW9jX3DSIj7A=="}}` |
| pinyin-pro@3.28.1 | `{"resolution": {"integrity": "sha512-oqz8ulwRgtUXRi0vbqEfGNly19zpyCxYrjhkk5TibGcgSW6eNwS5woajCXRwqURi8Ehc2yOFTiB4uNoZ+NJOnA=="}}` |
| postcss@8.5.15 | `{"engines": {"node": "^10 \|\| ^12 \|\| >=14"}, "resolution": {"integrity": "sha512-FfR8sjd4em2T6fb3I2MwAJU7HWVMr9zba+enmQeeWFfCbm+UOC/0X4DS8XtpUTMwWMGbjKYP7xjfNekzyGmB3A=="}}` |
| react-dom@19.2.7 | `{"peerDependencies": {"react": "^19.2.7"}, "resolution": {"integrity": "sha512-t0BRVXvbiE/o20Hfw669rLbMCDWtYZLvmJigy2f0MxsXF+71pxhR3xOkspmsO8h3ZlNzyibAmtCa3l4lYKk6gQ=="}}` |
| react-refresh@0.17.0 | `{"engines": {"node": ">=0.10.0"}, "resolution": {"integrity": "sha512-z6F7K9bV85EfseRCp2bzrpyQ0Gkw1uLoCel9XBVWPg/TjRj94SkJzUTGfOa4bs7iJvBWtQG0Wq7wnI0syw3EBQ=="}}` |
| react-router-dom@6.30.4 | `{"engines": {"node": ">=14.0.0"}, "peerDependencies": {"react": ">=16.8", "react-dom": ">=16.8"}, "resolution": {"integrity": "sha512-q4HvNl+mmDdkS0g+MqiBZNteQJCuimWoOyHMy4T/RQLAn9Z29+E91QXRaxOujeMl2HTzRSS0KFPd7lxX3PjV0Q=="}}` |
| react-router@6.30.4 | `{"engines": {"node": ">=14.0.0"}, "peerDependencies": {"react": ">=16.8"}, "resolution": {"integrity": "sha512-SVUsDe+DybHM/WmYKIVYhZh1o5Dcuf16yM6WjG02Q9XVFMZIJyHYhwrr6bFBXZkVP6z69kNkMyBCujt8FaFLJA=="}}` |
| react@19.2.7 | `{"engines": {"node": ">=0.10.0"}, "resolution": {"integrity": "sha512-HNe9WslTbXmFK8o8cmwgAeJFSBvt1bPdHCVKtaaV+WlAN36mpT4hcRpwbf3fY56ar2oIXzsBpOAiIRHAdY0OlQ=="}}` |
| rollup@4.62.0 | `{"engines": {"node": ">=18.0.0", "npm": ">=8.0.0"}, "hasBin": true, "resolution": {"integrity": "sha512-nc72Wgq62I7rtDV4izT5/aaS0zxy3kttkinf9586ApknY3jZO9NYsmtc24fUckA0X7Q2v+ML4a15pdUlV5V/jA=="}}` |
| scheduler@0.27.0 | `{"resolution": {"integrity": "sha512-eNv+WrVbKu1f3vbYJT/xtiF5syA5HPIMtf9IgY/nKg0sWqzAUEvqY/xm7OcZc/qafLx/iO9FgOmeSAp4v5ti/Q=="}}` |
| semver@6.3.1 | `{"hasBin": true, "resolution": {"integrity": "sha512-BR7VvDCVHO+q2xBEWskxS6DJE1qRnb7DxzUrogb71CWoSficBxYsiAGd+Kl0mmq/MprG9yArRkyrQxTO6XjMzA=="}}` |
| siginfo@2.0.0 | `{"resolution": {"integrity": "sha512-ybx0WO1/8bSBLEWXZvEd7gMW3Sn3JFlW3TvX1nREbDLRNQNaeNN8WK0meBwPdAaOI7TtRRRJn/Es1zhrrCHu7g=="}}` |
| source-map-js@1.2.1 | `{"engines": {"node": ">=0.10.0"}, "resolution": {"integrity": "sha512-UXWMKhLOwVKb728IUtQPXxfYU+usdybtUrK/8uGE8CQMvrhOpwvzDBwj0QhSL7MQc7vIsISBG8VQ8+IDQxpfQA=="}}` |
| stackback@0.0.2 | `{"resolution": {"integrity": "sha512-1XMJE5fQo1jGH6Y/7ebnwPOBEkIEnT4QF32d5R1+VXdXveM0IBMJt8zfaxX1P3QhVwrYe+576+jkANtSS2mBbw=="}}` |
| std-env@3.10.0 | `{"resolution": {"integrity": "sha512-5GS12FdOZNliM5mAOxFRg7Ir0pWz8MdpYm6AY6VPkGpbA7ZzmbzNcBJQ0GPvvyWgcY7QAhCgf9Uy89I03faLkg=="}}` |
| strip-literal@3.1.0 | `{"resolution": {"integrity": "sha512-8r3mkIM/2+PpjHoOtiAW8Rg3jJLHaV7xPwG+YRGrv6FP0wwk/toTpATxWYOW0BKdWwl82VT2tFYi5DlROa0Mxg=="}}` |
| tailwindcss@4.3.1 | `{"resolution": {"integrity": "sha512-hk+TB1m+K8CYNrP6rjQaq/Y+4Zylwpa87mLYBKCunwnnQ9p+fHb7kmSfGqyEJoxF/O6CDyABWVFEafNSYKll+Q=="}}` |
| tapable@2.3.3 | `{"engines": {"node": ">=6"}, "resolution": {"integrity": "sha512-uxc/zpqFg6x7C8vOE7lh6Lbda8eEL9zmVm/PLeTPBRhh1xCgdWaQ+J1CUieGpIfm2HdtsUpRv+HshiasBMcc6A=="}}` |
| tinybench@2.9.0 | `{"resolution": {"integrity": "sha512-0+DUvqWMValLmha6lr4kD8iAMK1HzV0/aKnCtWb9v9641TnP/MFb7Pc2bxoxQjTXAErryXVgUOfv2YqNllqGeg=="}}` |
| tinyexec@0.3.2 | `{"resolution": {"integrity": "sha512-KQQR9yN7R5+OSwaK0XQoj22pwHoTlgYqmUscPYoknOoWCWfj/5/ABTMRi69FrKU5ffPVh5QcFikpWJI/P1ocHA=="}}` |
| tinyglobby@0.2.17 | `{"engines": {"node": ">=12.0.0"}, "resolution": {"integrity": "sha512-wXR/dYpcqKmfWpEdZjiKJOwCNFndD0DMnrW/cYjVGttEkBfVgcLFHoNrlj47mjOVic9yyNu65alsgF4NQyTa2g=="}}` |
| tinypool@1.1.1 | `{"engines": {"node": "^18.0.0 \|\| >=20.0.0"}, "resolution": {"integrity": "sha512-Zba82s87IFq9A9XmjiX5uZA/ARWDrB03OHlq+Vw1fSdt0I+4/Kutwy8BP4Y/y/aORMo61FQ0vIb5j44vSo5Pkg=="}}` |
| tinyrainbow@2.0.0 | `{"engines": {"node": ">=14.0.0"}, "resolution": {"integrity": "sha512-op4nsTR47R6p0vMUUoYl/a+ljLFVtlfaXkLQmqfLR1qHma1h/ysYk4hEXZ880bf2CYgTskvTa/e196Vd5dDQXw=="}}` |
| tinyspy@4.0.4 | `{"engines": {"node": ">=14.0.0"}, "resolution": {"integrity": "sha512-azl+t0z7pw/z958Gy9svOTuzqIk6xq+NSheJzn5MMWtWTFywIacg2wUlzKFGtt3cthx0r2SxMK0yzJOR0IES7Q=="}}` |
| tslib@2.8.1 | `{"resolution": {"integrity": "sha512-oJFu94HQb+KVduSUQL7wnpmqnfmLsOA/nAh6b6EH0wCEoK0/mPeXU6c3wKDV83MkOuHPRHtSXKKU99IBazS/2w=="}}` |
| typescript@5.8.3 | `{"engines": {"node": ">=14.17"}, "hasBin": true, "resolution": {"integrity": "sha512-p1diW6TqL9L07nNxvRMM7hMMw4c5XOo/1ibL4aAIGmSAt9slTE1Xgw5KWuof2uTOvCg9BY7ZRi+GaF+7sfgPeQ=="}}` |
| update-browserslist-db@1.2.3 | `{"hasBin": true, "peerDependencies": {"browserslist": ">= 4.21.0"}, "resolution": {"integrity": "sha512-Js0m9cx+qOgDxo0eMiFGEueWztz+d4+M3rGlmKPT+T4IS/jP4ylw3Nwpu6cpTTP8R1MAC1kF4VbdLt3ARf209w=="}}` |
| vite-node@3.2.4 | `{"engines": {"node": "^18.0.0 \|\| ^20.0.0 \|\| >=22.0.0"}, "hasBin": true, "resolution": {"integrity": "sha512-EbKSKh+bh1E1IFxeO0pg1n4dvoOTt0UDiXMd/qn++r98+jPO1xtJilvXldeuQ8giIB5IkpjCgMleHMNEsGH6pg=="}}` |
| vite@7.3.5 | `{"engines": {"node": "^20.19.0 \|\| >=22.12.0"}, "hasBin": true, "peerDependencies": {"@types/node": "^20.19.0 \|\| >=22.12.0", "jiti": ">=1.21.0", "less": "^4.0.0", "lightningcss": "^1.21.0", "sass": "^1.70.0", "sass-embedded": "^1.70.0", "stylus": ">=0.54.8", "sugarss": "^5.0.0", "terser": "^5.16.0", "tsx": "^4.8.1", "yaml": "^2.4.2"}, "peerDependenciesMeta": {"@types/node": {"optional": true}, "jiti": {"optional": true}, "less": {"optional": true}, "lightningcss": {"optional": true}, "sass": {"optional": true}, "sass-embedded": {"optional": true}, "stylus": {"optional": true}, "sugarss": {"optional": true}, "terser": {"optional": true}, "tsx": {"optional": true}, "yaml": {"optional": true}}, "resolution": {"integrity": "sha512-KuOaNhcnGFN2zIPGA7wRmzF+lJA1sea7rHq17aiJ++9lzY1WWG6Jpwqwe1KNbRVPIqHmr8GLYx7jbrQcN/7/ww=="}}` |
| vitest@3.2.6 | `{"engines": {"node": "^18.0.0 \|\| ^20.0.0 \|\| >=22.0.0"}, "hasBin": true, "peerDependencies": {"@edge-runtime/vm": "*", "@types/debug": "^4.1.12", "@types/node": "^18.0.0 \|\| ^20.0.0 \|\| >=22.0.0", "@vitest/browser": "3.2.6", "@vitest/ui": "3.2.6", "happy-dom": "*", "jsdom": "*"}, "peerDependenciesMeta": {"@edge-runtime/vm": {"optional": true}, "@types/debug": {"optional": true}, "@types/node": {"optional": true}, "@vitest/browser": {"optional": true}, "@vitest/ui": {"optional": true}, "happy-dom": {"optional": true}, "jsdom": {"optional": true}}, "resolution": {"integrity": "sha512-xejya+bT/j/+R/AGa1XOfRxLmNUlLtlwjRsFUILF+xHfzElmGcmFydy2gqqIrd62ptIEfwVMofd19uNWD9L7Nw=="}}` |
| wanakana@5.3.1 | `{"engines": {"node": ">=12"}, "resolution": {"integrity": "sha512-OSDqupzTlzl2LGyqTdhcXcl6ezMiFhcUwLBP8YKaBIbMYW1wAwDvupw2T9G9oVaKT9RmaSpyTXjxddFPUcFFIw=="}}` |
| why-is-node-running@2.3.0 | `{"engines": {"node": ">=8"}, "hasBin": true, "resolution": {"integrity": "sha512-hUrmaWBdVDcxvYqnyh09zunKzROWjbZTiNy8dBEjkS7ehEDQibXJ7XvlmtbwuTclUiIyN+CyXQD4Vmko8fNm8w=="}}` |
| yallist@3.1.1 | `{"resolution": {"integrity": "sha512-a4UGQaWPH59mOXUYnAG2ewncQS4i4F43Tv3JoAM+s2VDAmS9NsK8GpDMLrCHPksFT7h3K6TOoUNn2pb7RoXx4g=="}}` |
| zustand@5.0.14 | `{"engines": {"node": ">=12.20.0"}, "peerDependencies": {"@types/react": ">=18.0.0", "immer": ">=9.0.6", "react": ">=18.0.0", "use-sync-external-store": ">=1.2.0"}, "peerDependenciesMeta": {"@types/react": {"optional": true}, "immer": {"optional": true}, "react": {"optional": true}, "use-sync-external-store": {"optional": true}}, "resolution": {"integrity": "sha512-/8tAspM5LMPr28b3fwLYrtdj77ECpfZviaP75CMTnwO8ISyaE4GDIG/9rDDYq/cH9D2Xw2A2RXglLInmVBQB/g=="}}` |

#### snapshots

| Resolved identity | Complete record |
| --- | --- |
| @babel/code-frame@7.29.7 | `{"dependencies": {"@babel/helper-validator-identifier": "7.29.7", "js-tokens": "4.0.0", "picocolors": "1.1.1"}}` |
| @babel/compat-data@7.29.7 | `{}` |
| @babel/core@7.29.7 | `{"dependencies": {"@babel/code-frame": "7.29.7", "@babel/generator": "7.29.7", "@babel/helper-compilation-targets": "7.29.7", "@babel/helper-module-transforms": "7.29.7(@babel/core@7.29.7)", "@babel/helpers": "7.29.7", "@babel/parser": "7.29.7", "@babel/template": "7.29.7", "@babel/traverse": "7.29.7", "@babel/types": "7.29.7", "@jridgewell/remapping": "2.3.5", "convert-source-map": "2.0.0", "debug": "4.4.3", "gensync": "1.0.0-beta.2", "json5": "2.2.3", "semver": "6.3.1"}, "transitivePeerDependencies": ["supports-color"]}` |
| @babel/generator@7.29.7 | `{"dependencies": {"@babel/parser": "7.29.7", "@babel/types": "7.29.7", "@jridgewell/gen-mapping": "0.3.13", "@jridgewell/trace-mapping": "0.3.31", "jsesc": "3.1.0"}}` |
| @babel/helper-compilation-targets@7.29.7 | `{"dependencies": {"@babel/compat-data": "7.29.7", "@babel/helper-validator-option": "7.29.7", "browserslist": "4.28.2", "lru-cache": "5.1.1", "semver": "6.3.1"}}` |
| @babel/helper-globals@7.29.7 | `{}` |
| @babel/helper-module-imports@7.29.7 | `{"dependencies": {"@babel/traverse": "7.29.7", "@babel/types": "7.29.7"}, "transitivePeerDependencies": ["supports-color"]}` |
| @babel/helper-module-transforms@7.29.7(@babel/core@7.29.7) | `{"dependencies": {"@babel/core": "7.29.7", "@babel/helper-module-imports": "7.29.7", "@babel/helper-validator-identifier": "7.29.7", "@babel/traverse": "7.29.7"}, "transitivePeerDependencies": ["supports-color"]}` |
| @babel/helper-plugin-utils@7.29.7 | `{}` |
| @babel/helper-string-parser@7.29.7 | `{}` |
| @babel/helper-validator-identifier@7.29.7 | `{}` |
| @babel/helper-validator-option@7.29.7 | `{}` |
| @babel/helpers@7.29.7 | `{"dependencies": {"@babel/template": "7.29.7", "@babel/types": "7.29.7"}}` |
| @babel/parser@7.29.7 | `{"dependencies": {"@babel/types": "7.29.7"}}` |
| @babel/plugin-transform-react-jsx-self@7.29.7(@babel/core@7.29.7) | `{"dependencies": {"@babel/core": "7.29.7", "@babel/helper-plugin-utils": "7.29.7"}}` |
| @babel/plugin-transform-react-jsx-source@7.29.7(@babel/core@7.29.7) | `{"dependencies": {"@babel/core": "7.29.7", "@babel/helper-plugin-utils": "7.29.7"}}` |
| @babel/template@7.29.7 | `{"dependencies": {"@babel/code-frame": "7.29.7", "@babel/parser": "7.29.7", "@babel/types": "7.29.7"}}` |
| @babel/traverse@7.29.7 | `{"dependencies": {"@babel/code-frame": "7.29.7", "@babel/generator": "7.29.7", "@babel/helper-globals": "7.29.7", "@babel/parser": "7.29.7", "@babel/template": "7.29.7", "@babel/types": "7.29.7", "debug": "4.4.3"}, "transitivePeerDependencies": ["supports-color"]}` |
| @babel/types@7.29.7 | `{"dependencies": {"@babel/helper-string-parser": "7.29.7", "@babel/helper-validator-identifier": "7.29.7"}}` |
| @esbuild/aix-ppc64@0.27.7 | `{"optional": true}` |
| @esbuild/android-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/android-arm@0.27.7 | `{"optional": true}` |
| @esbuild/android-x64@0.27.7 | `{"optional": true}` |
| @esbuild/darwin-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/darwin-x64@0.27.7 | `{"optional": true}` |
| @esbuild/freebsd-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/freebsd-x64@0.27.7 | `{"optional": true}` |
| @esbuild/linux-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/linux-arm@0.27.7 | `{"optional": true}` |
| @esbuild/linux-ia32@0.27.7 | `{"optional": true}` |
| @esbuild/linux-loong64@0.27.7 | `{"optional": true}` |
| @esbuild/linux-mips64el@0.27.7 | `{"optional": true}` |
| @esbuild/linux-ppc64@0.27.7 | `{"optional": true}` |
| @esbuild/linux-riscv64@0.27.7 | `{"optional": true}` |
| @esbuild/linux-s390x@0.27.7 | `{"optional": true}` |
| @esbuild/linux-x64@0.27.7 | `{"optional": true}` |
| @esbuild/netbsd-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/netbsd-x64@0.27.7 | `{"optional": true}` |
| @esbuild/openbsd-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/openbsd-x64@0.27.7 | `{"optional": true}` |
| @esbuild/openharmony-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/sunos-x64@0.27.7 | `{"optional": true}` |
| @esbuild/win32-arm64@0.27.7 | `{"optional": true}` |
| @esbuild/win32-ia32@0.27.7 | `{"optional": true}` |
| @esbuild/win32-x64@0.27.7 | `{"optional": true}` |
| @jridgewell/gen-mapping@0.3.13 | `{"dependencies": {"@jridgewell/sourcemap-codec": "1.5.5", "@jridgewell/trace-mapping": "0.3.31"}}` |
| @jridgewell/remapping@2.3.5 | `{"dependencies": {"@jridgewell/gen-mapping": "0.3.13", "@jridgewell/trace-mapping": "0.3.31"}}` |
| @jridgewell/resolve-uri@3.1.2 | `{}` |
| @jridgewell/sourcemap-codec@1.5.5 | `{}` |
| @jridgewell/trace-mapping@0.3.31 | `{"dependencies": {"@jridgewell/resolve-uri": "3.1.2", "@jridgewell/sourcemap-codec": "1.5.5"}}` |
| @remix-run/router@1.23.3 | `{}` |
| @rolldown/pluginutils@1.0.0-beta.27 | `{}` |
| @rollup/rollup-android-arm-eabi@4.62.0 | `{"optional": true}` |
| @rollup/rollup-android-arm64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-darwin-arm64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-darwin-x64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-freebsd-arm64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-freebsd-x64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-arm-gnueabihf@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-arm-musleabihf@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-arm64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-arm64-musl@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-loong64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-loong64-musl@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-ppc64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-ppc64-musl@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-riscv64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-riscv64-musl@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-s390x-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-x64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-linux-x64-musl@4.62.0 | `{"optional": true}` |
| @rollup/rollup-openbsd-x64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-openharmony-arm64@4.62.0 | `{"optional": true}` |
| @rollup/rollup-win32-arm64-msvc@4.62.0 | `{"optional": true}` |
| @rollup/rollup-win32-ia32-msvc@4.62.0 | `{"optional": true}` |
| @rollup/rollup-win32-x64-gnu@4.62.0 | `{"optional": true}` |
| @rollup/rollup-win32-x64-msvc@4.62.0 | `{"optional": true}` |
| @tailwindcss/node@4.3.1 | `{"dependencies": {"@jridgewell/remapping": "2.3.5", "enhanced-resolve": "5.21.6", "jiti": "2.7.0", "lightningcss": "1.32.0", "magic-string": "0.30.21", "source-map-js": "1.2.1", "tailwindcss": "4.3.1"}}` |
| @tailwindcss/oxide-android-arm64@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-darwin-arm64@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-darwin-x64@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-freebsd-x64@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-linux-arm-gnueabihf@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-linux-arm64-gnu@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-linux-arm64-musl@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-linux-x64-gnu@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-linux-x64-musl@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-wasm32-wasi@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-win32-arm64-msvc@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide-win32-x64-msvc@4.3.1 | `{"optional": true}` |
| @tailwindcss/oxide@4.3.1 | `{"optionalDependencies": {"@tailwindcss/oxide-android-arm64": "4.3.1", "@tailwindcss/oxide-darwin-arm64": "4.3.1", "@tailwindcss/oxide-darwin-x64": "4.3.1", "@tailwindcss/oxide-freebsd-x64": "4.3.1", "@tailwindcss/oxide-linux-arm-gnueabihf": "4.3.1", "@tailwindcss/oxide-linux-arm64-gnu": "4.3.1", "@tailwindcss/oxide-linux-arm64-musl": "4.3.1", "@tailwindcss/oxide-linux-x64-gnu": "4.3.1", "@tailwindcss/oxide-linux-x64-musl": "4.3.1", "@tailwindcss/oxide-wasm32-wasi": "4.3.1", "@tailwindcss/oxide-win32-arm64-msvc": "4.3.1", "@tailwindcss/oxide-win32-x64-msvc": "4.3.1"}}` |
| @tailwindcss/vite@4.3.1(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0)) | `{"dependencies": {"@tailwindcss/node": "4.3.1", "@tailwindcss/oxide": "4.3.1", "tailwindcss": "4.3.1", "vite": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)"}}` |
| @tanstack/query-core@5.101.0 | `{}` |
| @tanstack/react-query@5.101.0(react@19.2.7) | `{"dependencies": {"@tanstack/query-core": "5.101.0", "react": "19.2.7"}}` |
| @tauri-apps/api@2.11.0 | `{}` |
| @tauri-apps/cli-darwin-arm64@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-darwin-x64@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-arm-gnueabihf@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-arm64-gnu@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-arm64-musl@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-riscv64-gnu@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-x64-gnu@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-linux-x64-musl@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-win32-arm64-msvc@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-win32-ia32-msvc@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli-win32-x64-msvc@2.11.2 | `{"optional": true}` |
| @tauri-apps/cli@2.11.2 | `{"optionalDependencies": {"@tauri-apps/cli-darwin-arm64": "2.11.2", "@tauri-apps/cli-darwin-x64": "2.11.2", "@tauri-apps/cli-linux-arm-gnueabihf": "2.11.2", "@tauri-apps/cli-linux-arm64-gnu": "2.11.2", "@tauri-apps/cli-linux-arm64-musl": "2.11.2", "@tauri-apps/cli-linux-riscv64-gnu": "2.11.2", "@tauri-apps/cli-linux-x64-gnu": "2.11.2", "@tauri-apps/cli-linux-x64-musl": "2.11.2", "@tauri-apps/cli-win32-arm64-msvc": "2.11.2", "@tauri-apps/cli-win32-ia32-msvc": "2.11.2", "@tauri-apps/cli-win32-x64-msvc": "2.11.2"}}` |
| @tauri-apps/plugin-notification@2.4.0 | `{"dependencies": {"@tauri-apps/api": "2.11.0"}}` |
| @tauri-apps/plugin-opener@2.5.4 | `{"dependencies": {"@tauri-apps/api": "2.11.0"}}` |
| @tauri-apps/plugin-process@2.3.1 | `{"dependencies": {"@tauri-apps/api": "2.11.0"}}` |
| @tauri-apps/plugin-updater@2.10.1 | `{"dependencies": {"@tauri-apps/api": "2.11.0"}}` |
| @types/babel__core@7.20.5 | `{"dependencies": {"@babel/parser": "7.29.7", "@babel/types": "7.29.7", "@types/babel__generator": "7.27.0", "@types/babel__template": "7.4.4", "@types/babel__traverse": "7.28.0"}}` |
| @types/babel__generator@7.27.0 | `{"dependencies": {"@babel/types": "7.29.7"}}` |
| @types/babel__template@7.4.4 | `{"dependencies": {"@babel/parser": "7.29.7", "@babel/types": "7.29.7"}}` |
| @types/babel__traverse@7.28.0 | `{"dependencies": {"@babel/types": "7.29.7"}}` |
| @types/chai@5.2.3 | `{"dependencies": {"@types/deep-eql": "4.0.2", "assertion-error": "2.0.1"}}` |
| @types/deep-eql@4.0.2 | `{}` |
| @types/estree@1.0.9 | `{}` |
| @types/react-dom@19.2.3(@types/react@19.2.17) | `{"dependencies": {"@types/react": "19.2.17"}}` |
| @types/react@19.2.17 | `{"dependencies": {"csstype": "3.2.3"}}` |
| @vitejs/plugin-react@4.7.0(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0)) | `{"dependencies": {"@babel/core": "7.29.7", "@babel/plugin-transform-react-jsx-self": "7.29.7(@babel/core@7.29.7)", "@babel/plugin-transform-react-jsx-source": "7.29.7(@babel/core@7.29.7)", "@rolldown/pluginutils": "1.0.0-beta.27", "@types/babel__core": "7.20.5", "react-refresh": "0.17.0", "vite": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)"}, "transitivePeerDependencies": ["supports-color"]}` |
| @vitest/expect@3.2.6 | `{"dependencies": {"@types/chai": "5.2.3", "@vitest/spy": "3.2.6", "@vitest/utils": "3.2.6", "chai": "5.3.3", "tinyrainbow": "2.0.0"}}` |
| @vitest/mocker@3.2.6(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0)) | `{"dependencies": {"@vitest/spy": "3.2.6", "estree-walker": "3.0.3", "magic-string": "0.30.21"}, "optionalDependencies": {"vite": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)"}}` |
| @vitest/pretty-format@3.2.6 | `{"dependencies": {"tinyrainbow": "2.0.0"}}` |
| @vitest/runner@3.2.6 | `{"dependencies": {"@vitest/utils": "3.2.6", "pathe": "2.0.3", "strip-literal": "3.1.0"}}` |
| @vitest/snapshot@3.2.6 | `{"dependencies": {"@vitest/pretty-format": "3.2.6", "magic-string": "0.30.21", "pathe": "2.0.3"}}` |
| @vitest/spy@3.2.6 | `{"dependencies": {"tinyspy": "4.0.4"}}` |
| @vitest/utils@3.2.6 | `{"dependencies": {"@vitest/pretty-format": "3.2.6", "loupe": "3.2.1", "tinyrainbow": "2.0.0"}}` |
| assertion-error@2.0.1 | `{}` |
| baseline-browser-mapping@2.10.37 | `{}` |
| browserslist@4.28.2 | `{"dependencies": {"baseline-browser-mapping": "2.10.37", "caniuse-lite": "1.0.30001799", "electron-to-chromium": "1.5.372", "node-releases": "2.0.47", "update-browserslist-db": "1.2.3(browserslist@4.28.2)"}}` |
| cac@6.7.14 | `{}` |
| caniuse-lite@1.0.30001799 | `{}` |
| chai@5.3.3 | `{"dependencies": {"assertion-error": "2.0.1", "check-error": "2.1.3", "deep-eql": "5.0.2", "loupe": "3.2.1", "pathval": "2.0.1"}}` |
| check-error@2.1.3 | `{}` |
| convert-source-map@2.0.0 | `{}` |
| csstype@3.2.3 | `{}` |
| debug@4.4.3 | `{"dependencies": {"ms": "2.1.3"}}` |
| deep-eql@5.0.2 | `{}` |
| detect-libc@2.1.2 | `{}` |
| electron-to-chromium@1.5.372 | `{}` |
| enhanced-resolve@5.21.6 | `{"dependencies": {"graceful-fs": "4.2.11", "tapable": "2.3.3"}}` |
| es-module-lexer@1.7.0 | `{}` |
| esbuild@0.27.7 | `{"optionalDependencies": {"@esbuild/aix-ppc64": "0.27.7", "@esbuild/android-arm": "0.27.7", "@esbuild/android-arm64": "0.27.7", "@esbuild/android-x64": "0.27.7", "@esbuild/darwin-arm64": "0.27.7", "@esbuild/darwin-x64": "0.27.7", "@esbuild/freebsd-arm64": "0.27.7", "@esbuild/freebsd-x64": "0.27.7", "@esbuild/linux-arm": "0.27.7", "@esbuild/linux-arm64": "0.27.7", "@esbuild/linux-ia32": "0.27.7", "@esbuild/linux-loong64": "0.27.7", "@esbuild/linux-mips64el": "0.27.7", "@esbuild/linux-ppc64": "0.27.7", "@esbuild/linux-riscv64": "0.27.7", "@esbuild/linux-s390x": "0.27.7", "@esbuild/linux-x64": "0.27.7", "@esbuild/netbsd-arm64": "0.27.7", "@esbuild/netbsd-x64": "0.27.7", "@esbuild/openbsd-arm64": "0.27.7", "@esbuild/openbsd-x64": "0.27.7", "@esbuild/openharmony-arm64": "0.27.7", "@esbuild/sunos-x64": "0.27.7", "@esbuild/win32-arm64": "0.27.7", "@esbuild/win32-ia32": "0.27.7", "@esbuild/win32-x64": "0.27.7"}}` |
| escalade@3.2.0 | `{}` |
| estree-walker@3.0.3 | `{"dependencies": {"@types/estree": "1.0.9"}}` |
| expect-type@1.3.0 | `{}` |
| fdir@6.5.0(picomatch@4.0.4) | `{"optionalDependencies": {"picomatch": "4.0.4"}}` |
| framer-motion@12.40.0(react-dom@19.2.7(react@19.2.7))(react@19.2.7) | `{"dependencies": {"motion-dom": "12.40.0", "motion-utils": "12.39.0", "tslib": "2.8.1"}, "optionalDependencies": {"react": "19.2.7", "react-dom": "19.2.7(react@19.2.7)"}}` |
| fsevents@2.3.3 | `{"optional": true}` |
| gensync@1.0.0-beta.2 | `{}` |
| graceful-fs@4.2.11 | `{}` |
| jiti@2.7.0 | `{}` |
| js-tokens@4.0.0 | `{}` |
| js-tokens@9.0.1 | `{}` |
| jsesc@3.1.0 | `{}` |
| json5@2.2.3 | `{}` |
| lightningcss-android-arm64@1.32.0 | `{"optional": true}` |
| lightningcss-darwin-arm64@1.32.0 | `{"optional": true}` |
| lightningcss-darwin-x64@1.32.0 | `{"optional": true}` |
| lightningcss-freebsd-x64@1.32.0 | `{"optional": true}` |
| lightningcss-linux-arm-gnueabihf@1.32.0 | `{"optional": true}` |
| lightningcss-linux-arm64-gnu@1.32.0 | `{"optional": true}` |
| lightningcss-linux-arm64-musl@1.32.0 | `{"optional": true}` |
| lightningcss-linux-x64-gnu@1.32.0 | `{"optional": true}` |
| lightningcss-linux-x64-musl@1.32.0 | `{"optional": true}` |
| lightningcss-win32-arm64-msvc@1.32.0 | `{"optional": true}` |
| lightningcss-win32-x64-msvc@1.32.0 | `{"optional": true}` |
| lightningcss@1.32.0 | `{"dependencies": {"detect-libc": "2.1.2"}, "optionalDependencies": {"lightningcss-android-arm64": "1.32.0", "lightningcss-darwin-arm64": "1.32.0", "lightningcss-darwin-x64": "1.32.0", "lightningcss-freebsd-x64": "1.32.0", "lightningcss-linux-arm-gnueabihf": "1.32.0", "lightningcss-linux-arm64-gnu": "1.32.0", "lightningcss-linux-arm64-musl": "1.32.0", "lightningcss-linux-x64-gnu": "1.32.0", "lightningcss-linux-x64-musl": "1.32.0", "lightningcss-win32-arm64-msvc": "1.32.0", "lightningcss-win32-x64-msvc": "1.32.0"}}` |
| loupe@3.2.1 | `{}` |
| lru-cache@5.1.1 | `{"dependencies": {"yallist": "3.1.1"}}` |
| magic-string@0.30.21 | `{"dependencies": {"@jridgewell/sourcemap-codec": "1.5.5"}}` |
| motion-dom@12.40.0 | `{"dependencies": {"motion-utils": "12.39.0"}}` |
| motion-utils@12.39.0 | `{}` |
| ms@2.1.3 | `{}` |
| nanoid@3.3.12 | `{}` |
| node-releases@2.0.47 | `{}` |
| pathe@2.0.3 | `{}` |
| pathval@2.0.1 | `{}` |
| picocolors@1.1.1 | `{}` |
| picomatch@4.0.4 | `{}` |
| pinyin-pro@3.28.1 | `{}` |
| postcss@8.5.15 | `{"dependencies": {"nanoid": "3.3.12", "picocolors": "1.1.1", "source-map-js": "1.2.1"}}` |
| react-dom@19.2.7(react@19.2.7) | `{"dependencies": {"react": "19.2.7", "scheduler": "0.27.0"}}` |
| react-refresh@0.17.0 | `{}` |
| react-router-dom@6.30.4(react-dom@19.2.7(react@19.2.7))(react@19.2.7) | `{"dependencies": {"@remix-run/router": "1.23.3", "react": "19.2.7", "react-dom": "19.2.7(react@19.2.7)", "react-router": "6.30.4(react@19.2.7)"}}` |
| react-router@6.30.4(react@19.2.7) | `{"dependencies": {"@remix-run/router": "1.23.3", "react": "19.2.7"}}` |
| react@19.2.7 | `{}` |
| rollup@4.62.0 | `{"dependencies": {"@types/estree": "1.0.9"}, "optionalDependencies": {"@rollup/rollup-android-arm-eabi": "4.62.0", "@rollup/rollup-android-arm64": "4.62.0", "@rollup/rollup-darwin-arm64": "4.62.0", "@rollup/rollup-darwin-x64": "4.62.0", "@rollup/rollup-freebsd-arm64": "4.62.0", "@rollup/rollup-freebsd-x64": "4.62.0", "@rollup/rollup-linux-arm-gnueabihf": "4.62.0", "@rollup/rollup-linux-arm-musleabihf": "4.62.0", "@rollup/rollup-linux-arm64-gnu": "4.62.0", "@rollup/rollup-linux-arm64-musl": "4.62.0", "@rollup/rollup-linux-loong64-gnu": "4.62.0", "@rollup/rollup-linux-loong64-musl": "4.62.0", "@rollup/rollup-linux-ppc64-gnu": "4.62.0", "@rollup/rollup-linux-ppc64-musl": "4.62.0", "@rollup/rollup-linux-riscv64-gnu": "4.62.0", "@rollup/rollup-linux-riscv64-musl": "4.62.0", "@rollup/rollup-linux-s390x-gnu": "4.62.0", "@rollup/rollup-linux-x64-gnu": "4.62.0", "@rollup/rollup-linux-x64-musl": "4.62.0", "@rollup/rollup-openbsd-x64": "4.62.0", "@rollup/rollup-openharmony-arm64": "4.62.0", "@rollup/rollup-win32-arm64-msvc": "4.62.0", "@rollup/rollup-win32-ia32-msvc": "4.62.0", "@rollup/rollup-win32-x64-gnu": "4.62.0", "@rollup/rollup-win32-x64-msvc": "4.62.0", "fsevents": "2.3.3"}}` |
| scheduler@0.27.0 | `{}` |
| semver@6.3.1 | `{}` |
| siginfo@2.0.0 | `{}` |
| source-map-js@1.2.1 | `{}` |
| stackback@0.0.2 | `{}` |
| std-env@3.10.0 | `{}` |
| strip-literal@3.1.0 | `{"dependencies": {"js-tokens": "9.0.1"}}` |
| tailwindcss@4.3.1 | `{}` |
| tapable@2.3.3 | `{}` |
| tinybench@2.9.0 | `{}` |
| tinyexec@0.3.2 | `{}` |
| tinyglobby@0.2.17 | `{"dependencies": {"fdir": "6.5.0(picomatch@4.0.4)", "picomatch": "4.0.4"}}` |
| tinypool@1.1.1 | `{}` |
| tinyrainbow@2.0.0 | `{}` |
| tinyspy@4.0.4 | `{}` |
| tslib@2.8.1 | `{}` |
| typescript@5.8.3 | `{}` |
| update-browserslist-db@1.2.3(browserslist@4.28.2) | `{"dependencies": {"browserslist": "4.28.2", "escalade": "3.2.0", "picocolors": "1.1.1"}}` |
| vite-node@3.2.4(jiti@2.7.0)(lightningcss@1.32.0) | `{"dependencies": {"cac": "6.7.14", "debug": "4.4.3", "es-module-lexer": "1.7.0", "pathe": "2.0.3", "vite": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)"}, "transitivePeerDependencies": ["@types/node", "jiti", "less", "lightningcss", "sass", "sass-embedded", "stylus", "sugarss", "supports-color", "terser", "tsx", "yaml"]}` |
| vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0) | `{"dependencies": {"esbuild": "0.27.7", "fdir": "6.5.0(picomatch@4.0.4)", "picomatch": "4.0.4", "postcss": "8.5.15", "rollup": "4.62.0", "tinyglobby": "0.2.17"}, "optionalDependencies": {"fsevents": "2.3.3", "jiti": "2.7.0", "lightningcss": "1.32.0"}}` |
| vitest@3.2.6(jiti@2.7.0)(lightningcss@1.32.0) | `{"dependencies": {"@types/chai": "5.2.3", "@vitest/expect": "3.2.6", "@vitest/mocker": "3.2.6(vite@7.3.5(jiti@2.7.0)(lightningcss@1.32.0))", "@vitest/pretty-format": "3.2.6", "@vitest/runner": "3.2.6", "@vitest/snapshot": "3.2.6", "@vitest/spy": "3.2.6", "@vitest/utils": "3.2.6", "chai": "5.3.3", "debug": "4.4.3", "expect-type": "1.3.0", "magic-string": "0.30.21", "pathe": "2.0.3", "picomatch": "4.0.4", "std-env": "3.10.0", "tinybench": "2.9.0", "tinyexec": "0.3.2", "tinyglobby": "0.2.17", "tinypool": "1.1.1", "tinyrainbow": "2.0.0", "vite": "7.3.5(jiti@2.7.0)(lightningcss@1.32.0)", "vite-node": "3.2.4(jiti@2.7.0)(lightningcss@1.32.0)", "why-is-node-running": "2.3.0"}, "transitivePeerDependencies": ["jiti", "less", "lightningcss", "msw", "sass", "sass-embedded", "stylus", "sugarss", "supports-color", "terser", "tsx", "yaml"]}` |
| wanakana@5.3.1 | `{}` |
| why-is-node-running@2.3.0 | `{"dependencies": {"siginfo": "2.0.0", "stackback": "0.0.2"}}` |
| yallist@3.1.1 | `{}` |
| zustand@5.0.14(@types/react@19.2.17)(react@19.2.7) | `{"optionalDependencies": {"@types/react": "19.2.17", "react": "19.2.7"}}` |

