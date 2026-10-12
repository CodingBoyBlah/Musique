# Entire-codebase performance audit

Completed 2026-10-10 against the original working tree at commit 2d55bb477c40a7b27affc35cac64062baf1c7e74, including the user's pre-existing edits.

**The largest opportunities are backend request ownership, trustworthy shared caches, response-path disk work, playback/session teardown, and bounded streaming.** These can be addressed while leaving frontend components, CSS, layout, fonts, artwork quality and interactions unchanged. Frontend mount/render costs are also documented, but those proposals are a separate optional phase.

No implementation was made. Only nineteen new audit Markdown files were written. All 540 original project files were reviewed and checked in [review_progress.md](review_progress.md), and every original file still matches its baseline hash. Source findings establish unnecessary work and failure conditions; they do not establish measured speedup percentages.

## Where to read

| Document | Purpose |
| --- | --- |
| [review_progress.md](review_progress.md) | Every original file, baseline hash and actual review method/status. |
| [findings.md](findings.md) | Detailed evidence, source lines, cross-file checks and corrections. |
| [system_map.md](system_map.md) | Startup, requests, authentication, playback, lyrics, library, profiles, Stats and task ownership. |
| [interaction_map.md](interaction_map.md) | Every route/control family, Settings action and input/queue ownership contract. |
| [interaction_inventory.md](interaction_inventory.md) | Exact source locations and expressions for all 443 JSX event attributes. |
| [render_graph.md](render_graph.md) / [dependency_map.md](dependency_map.md) | Every JSX tag/component occurrence and frontend import edge across 194 TS/TSX files. |
| [native_ipc_map.md](native_ipc_map.md) | All 149 registered commands, frontend invoke sites, native events and module dependencies. |
| [data_map.md](data_map.md) | All 25 application tables, final columns, keys, indexes and foreign-key edges. |
| [dependency_lock_review.md](dependency_lock_review.md) | Every package field/dependency edge from all five locks. |
| [asset_review.md](asset_review.md) | Every asset/font container, integrity, size, decoded cost and runtime/packaging use. |
| [ui_consistency_notes.md](ui_consistency_notes.md) | Optional typography, copy, accessibility and behavior inconsistencies. |
| [verification_notes.md](verification_notes.md) | Actual read-only checks and their limits. |

## Scope and evidence

The review includes all original first-party frontend/native source, styles, tests, migrations, examples, configs, scripts, documentation, vendored code, lockfiles, assets and ignored signing-material metadata. Four vendored crates are included: CPAL, librespot-core, librespot-audio and librespot-connect.

Git internals, installed node_modules, dist, src-tauri/target and generated native schema output are excluded from manual source review. Installed librespot Player and reqwest feature declarations received targeted boundary checks; this is not a claim that all installed dependency source was audited. Binary assets received format/integrity/dimension/use review; selected historical screenshots/logo/texture were visually inspected. Private signing-key contents were not copied.

- All original 540 entries are checked. A checkbox records the stated review, including binary/structured/sensitive classifications; it is not a claim that binary glyph outlines were manually read as source.
- The 193 frontend source/test roots have zero read-only TypeScript diagnostics. The separate Vite config analysis reports missing Node declarations for path and __dirname.
- All thirteen native migrations apply in memory. Actual schema/query checks confirm duplicate saved-track indexes and loss of repeated cached playlist occurrences.
- All Cargo graph edges resolve; all 285 PNPM snapshot edges reconcile. This is a local graph review, not an online advisory scan or a packaged target-feature/bundle-size measurement.
- Builds, installs, application launches, Vitest/native tests, live service probes and runtime profiling were not run because they can write non-Markdown artifacts/state. Historical PERFORMANCE.md results remain historical.

**Priority below estimates likely benefit and risk from code structure. Actual p50/p95 latency, CPU, memory and underrun changes remain to be measured after implementation.**

## Recommended order with the frontend left untouched

| Order | Change family | Expected benefit | Main acceptance requirement |
| --- | --- | --- | --- |
| 1 | Playback/session lifetime, receiver EOF, recovered Player session | Prevent leaked tasks/devices, CPU spin and repeated broken recovery | One current session/device; exact queue/clock/role semantics. |
| 2 | Shared request/cache miss ownership and user-action scheduling | Less duplicate startup/search/detail traffic and less waiting behind background work | Account/revision isolation, same content/order and provider limits. |
| 3 | Return mapped search/detail data before unrelated serial persistence | Faster result/render/first-play responses | Dependency-complete durable writes, guarded ordering and cache consistency. |
| 4 | Correct atomic playlist/library snapshots and one sync owner | Reliable warm/offline reads; fewer transactions and duplicate syncs | Repeated occurrences, empty playlists, membership and logout safety. |
| 5 | End-to-end network deadlines, bounded retries/bodies and cancellation | Lower worst-case latency/memory during outages and rapid changes | Preserve fallback, quality, mutation idempotency and Retry-After. |
| 6 | YouTube/episode streaming and byte budgets | Less transient memory/wasted downloads; faster reliable seeks | Exact range/object identity; non-range support; same recording/audio quality. |
| 7 | Lyrics first-response/background work and cache outcome semantics | Faster correct lyrics; fewer duplicate/transient-negative misses | Same word timing, provider preference, alternates and sticky selection. |
| 8 | Stats/recommendation hydration and algorithm reuse | Less repeated SQL/HTTP/CPU | Same rankings, ties, explicit preference and displayed data. |
| 9 | Native idle/media/credential/probe work | Lower idle wakeups, repeated IPC and executor blocking | Instant resume, hotplug, accurate presence/notification/account state. |
| 10 | Producer scratch reuse and measured audio callback improvements | Lower playback allocations/CPU with stable output | Sample/continuity/pitch/tail and underrun regression coverage. |

### 1. Make every playback replacement release its old task group

Evidence: playback/mod.rs:278–290,529–534; commands/playback.rs retry/rebuild/quality/cache paths; vendor/librespot-core/src/dealer/manager.rs:150–154; vendor/librespot-connect/src/spirc.rs:631–639.

Explicit shutdown exists and is useful. Replacement paths can instead drop PlaybackInner, whose Drop only closes listening. Tokio handles detach; Spirc/Dealer/events/reports can retain Session/Player and keep old devices or publications alive. Dealer's reconnect closure forms a strong ownership cycle. Spirc also leaves closed command/player-event receivers enabled: recv returning None can remain immediately ready on each select iteration rather than ending/disabling the branch.

A future change should use one bounded stop-and-join owner for success, retry, replacement, logout and failed initialization. Stop audio and reports, close Dealer/Connect, reject late publications, then abort as a final bounded fallback. Handle receiver EOF explicitly. Preserve the existing final-report budget and graceful device deregistration rather than merely aborting all handles.

Validation: repeatedly retry/switch quality/source/login accounts and close receivers with active/paused/idle audio. Count tasks, sessions, registered devices, sockets, report backlog and events after teardown; wait for a stable plateau. Confirm no old-account credential/cache rewrite or old audio and no EOF spin.

### 2. Give the recovered decoder Player the recovered Session

Evidence: playback/mod.rs:715–720,803–814; installed librespot-playback 0.8.0 player.rs:487,572–573 and SetSession handling. App/fork source contains no Player set_session call.

Player is constructed with the first session. Credential recovery can replace the Session used by Spirc while reusing that Player. Decoder/metadata/audio operations can therefore continue using the rejected session even though Connect initialization succeeded.

Bind Player to the final authenticated Session before further loads, or create it after recovery. Preserve event subscriptions, mixer/device ownership, prepared state and Connect clock ordering. This is a backend correctness fix with a direct recovery-latency consequence; a measured general startup improvement is not yet established.

Validation: force the initial credentials to fail and recovery to succeed; assert the decoder and Connect own the same live session and playback reaches audio without another retry.

### 3. Coalesce cold requests at the native boundary

Evidence: internal/pathfinder.rs:106–149,191–197; internal/cache.rs:173–194; vendor core login5.rs:173,spclient.rs:164–199; Home/show/detail/credential callers in findings.md.

Concurrent Home, artist, color, lyrics, report and play requests can miss the same cache independently. Pathfinder can repeatedly download the HTML and multi-MB web-player bundle for hashes. Login5/client-token/AP discovery has similar cache-miss races. Frontend query deduplication already shares matching keys, but direct play APIs and different native callers bypass that owner.

Share one cancellable future per logical key: account/session/grant revision plus resource/market/content variant. Recheck freshness under that ownership and remove in-flight entries on cancellation/error. Obtain fallback OAuth only when needed. Use typed negative outcomes and short failure cooldowns; transient failure is not durable absence. Share narrow artist metadata/overview internally so avatar/detail consumers avoid duplicate full work.

Validation: issue simultaneous identical calls and overlapping batch misses; inspect upstream request counts, hash/token discoveries and result ordering. Repeat after account switch, expired token, forced refresh and a failed/cancelled owner. Do not let a failed owner permanently block later requests.

### 4. Schedule search, controls and current audio ahead of speculation

Evidence: spotify/mod.rs shared eight-slot semaphore/retry wrapper; Home fan-out; library sync; direct Connect state GET; four YouTube ranges per download.

The current eight-slot limit is shared by user actions and background library work. Retries can retain permits during sleeps, while body collection/JSON work happens after permit release. Other HTTP families have separate limits. Simply increasing concurrency could increase rate-limit pressure, memory and latency variance.

Use bounded priority classes for explicit user actions, current-track necessities, background refresh and speculative prefetch. Reserve capacity or fair admission for control/search, share per-service/account cooldowns, and include body/decode memory in budgets. Release admission appropriately during authoritative cooldown without allowing a storm to bypass it. Keep mutations ordered by resource/intent.

Validation: type/search/play/seek while a large sync and next-track preload run, including 429/slow-body conditions. Compare p95 queue wait and completed response latency with upstream request counts and rate-limit compliance. Preserve all background results rather than removing sections/features.

### 5. Take disk work off search/detail response paths

Evidence: commands/spotify.rs:232–238,245–261,127–157; commands/playback.rs:254.

Search maps its response but then waits for up to sixty artist/album/track upserts plus history. Artist main/detail calls are independent but serial. Album and playback matching also wait for catalogue persistence unrelated to the already-prepared response.

Prepare dependency-complete batches, return the same mapped DTO promptly, and persist through a bounded account/revision-aware writer. Preserve FK dependencies, richer COALESCE fields and ISRC. Where a following action requires the cache immediately, share the prepared snapshot/flush barrier rather than race a detached writer. Fetch artist/top/discography and selected independent supplemental requests concurrently within the service budget.

Validation: compare identical DTO values/order/null handling and response timestamps with delayed SQLite writes; trigger immediate like/play/detail navigation after search. Bound writer backlog, test shutdown/logout and errors, and verify search history durability. Timing should show response no longer includes serial unrelated disk commits.

### 6. Make playlist/library caching correct before making it more aggressive

Evidence: commands/spotify.rs:451–458,513–558; library/mod.rs:221–243; commands/library.rs:39,205–207,752,853–855; playlist_tracks schema; verification_notes.md.

Library metadata updates snapshot_id before track rows are refreshed, so live detail can mistake old rows for current. Replacement is not one atomic revision-guarded transaction. Repeated tracks cannot be represented by the current primary key; known-empty complete playlists return no cache. Browsed catalogue playlists and followed membership are conflated. Manual/background syncs both hold a read gate and can overlap.

Separate metadata revision from fully committed track-snapshot revision and followed membership. Model occurrence identity/order and explicit completeness, including zero items. Use one transaction per bounded replacement and one sync owner per account/revision. Check cancellation/epoch inside page acquisition and immediately before each commit; do not rely only on category boundaries. Batch deduplicated catalogue dependency writes while retaining one ordered writer.

Validation: serve warm/cold/empty playlists, duplicates, concurrent sync/detail fills, old/new snapshots, rate-limited pagination and logout during a large category. Assert no partial/mixed snapshots, cross-account refill, duplicate sync or disappeared occurrence. This protects the same intended frontend content while making cache-first reads dependable.

### 7. Bound the whole network operation, including nested vendor retries

Evidence: vendor core http_client.rs:181–226; spclient.rs:494–590; connection/mod.rs:67–71; socket.rs:24; app spotify/mod.rs retry policy.

The vendor transport lacks overall request/body deadlines and can repeat short Retry-After responses indefinitely, beneath outer SpClient retries. The nominal five-second AP timeout is constructed after socket connect is awaited, so it does not bound connect/DNS/proxy negotiation. DNS is synchronous and only the first address is chosen. App reqwest deadlines do not apply to those separate paths.

Use cancellable end-to-end deadlines covering admission, DNS/connect/handshake, auth, retries, headers, body and decode. Prefer asynchronous multi-address connection fallback. Respect authoritative Retry-After; classify retryable errors. Do not repeat non-idempotent playlist/create/add mutations blindly after an uncertain server outcome. Bound external compressed/uncompressed/frame bytes before allocating or parsing.

Validation: stalled DNS/socket/body, unavailable AP, short/long 429, repeated 5xx, malformed frame and cancellation. Check prompt control recovery, bounded task/bytes and successful healthy fallback. Verify RSA/signature verification and auth recording/entitlement rules remain intact.

### 8. Put byte limits around YouTube download and prepared work

Evidence: youtube/stream.rs:74,140; youtube/mod.rs:190–253,293; youtube/format.rs and player.rs; YouTube player watcher.

Declared size is capped at 96 MiB and prepared cache has two entries, but range bodies are collected before aggregate validation. HTTP 200 for an ignored range is only warned about; wrong offsets/lengths are not verified. Four ranges is a per-download bound. Cancelled command ownership may not drop the active download or its fill registry, and clear_prepared can be followed by a late refill.

Stream each range into a shared bounded byte budget; verify status, Content-Range, exact length and object identity. Use a controlled single-stream fallback when ranges are unavailable. Give foreground play precedence over preload, retain cancellation-safe fill guards and reject publication from old source/account generations. Choose the best usable format with all required fields present, preserving the existing quality preference.

Validation: ignored/wrong/oversized/truncated ranges, rapid skips, failed visitor refresh, pin/forget race, source/logout change and two long tracks. Measure peak bytes and requests after cancellation; do not lower bitrate, replace the chosen recording or remove next-track preparation to manufacture a faster number.

### 9. Fix episode seek ownership and non-range producer behavior

Evidence: episode_audio/remote.rs:159,413–457; commands/playback.rs:407–425; episode_audio/mod.rs:159–163; mp4.rs:56.

A sequential non-range response can run ahead and evict blocks that no fill worker can fetch again. Backward seek then waits for a timeout; pause can also lose needed future bytes. A pending seek releases the state lock and later acts on whichever episode is current. Feed-cache pruning at eight entries is not an eight-entry cap.

Add a sequential backpressure/restart policy, protect needed/header bytes, and verify ranged response offsets/ETag. Own each seek by stream identity and latest seek ticket through final commit. Replace 250-ms ahead polling with head/seek/close notification, choose a measured time/bitrate ahead window, and cancel in-flight requests on last-reader close. Bound parsed feeds, directory/feed single-flight, response and MP4 header/index sizes.

Validation: real range and non-range servers; pause while filling; backward/forward seek outside retained blocks; A→B switch during A seek; changed CDN object; offline/malformed feed; long show. Preserve no-temp-file design, resume progress, pitch/speed, transcript clock and region/source matching.

### 10. Return correct lyric data sooner and preserve alternate sources

Evidence: lyrics/mod.rs:166,212–254,332–381,441–478; provider status handling; lyrics/cache.rs:130–200; parse/TTML/alignment sources.

Stages wait for paired providers even when one already produced an acceptable candidate. Current upgrades lack account/manual-selection revision guards. Timeouts/rate limits often become one-day provider misses and possibly a seven-day whole-track miss; force does not consistently bypass those misses. Alternate-source listing parses every stored full payload.

Stream completions through the existing recording/alignment/preference gate. Return an accepted result as soon as the same policy permits, and complete required alternatives/upgrades in a bounded guarded background owner. Do not discard alternate choices or choose arbitrary fastest text. Distinguish absence from transient error/cancellation, use provider cooldowns/deadlines and schema-versioned line-count metadata. Bound crypto/decompression/TTML and move demonstrated CPU-heavy work off async workers.

Validation: exact/nonexact sources, slow/failing/429 providers, no Spotify reference, force refresh, sticky source selection during upgrade, logout and next-track prefetch. Selected/alternate source order, timing, spaces, CJK fragments, translations, duet/backing roles and instrumental gaps must match the intended contract.

### 11. Reuse recommendation and Stats work

Evidence: recommend/mod.rs:591–620,683–700,775–819; commands/stats.rs:182,271,325–345; artist_extras.rs:134–137.

MMR recomputes every candidate's similarity with every previously selected result on every selection. Retaining each candidate's maximum redundancy reduces that repeated factor without changing the diversity calculation. Profile/default recommendation caches need account/taste/library revision ownership and single-flight; full-map eviction creates cold cliffs. Cold network acquisition already has useful bounds.

Build one Stats snapshot, union missing metadata IDs once across ranked groups, avoid discarded all-range discovery work, and reuse bounded aggregate results. Fetch independent artist top/albums and selected album details concurrently. Consolidate artist-extras album hydration while preserving group order. A prepared fallback candidate pool can avoid repeated catalogue-wide RANDOM sorts, but must retain random discovery and explicit-content filtering.

Validation: seeded identical recommendation candidates/ranks/ties/caps/jitter, explicit preferences, account/library/taste changes, zero metadata, cold/warm Stats and retained-history boundaries. Query-plan tests show cached playlist/liked order is already indexed; avoid extra indexes for tiny album sorts without realistic evidence.

### 12. Make cache budgets and publications coherent across services

Evidence: internal/cache.rs; vendor core cache.rs:141–162; recommendation caches; episode feed cache; lyrics/YouTube/report ownership above.

Use count plus byte/age budgets, current/transition pinning and account/market/revision keys. Count-only limits and clear-entire-map eviction do not create a predictable memory/hit-rate contract. Protect account-scoped publication after logout; cache invalidation alone does not stop a running fill.

Move audio cache scan/prune/copy off async startup workers, avoid repeated scanning on rebuild, bound symlink traversal and atomically publish finished files. Failed deletion must not remove accounting for bytes still on disk. Preserve permanent manual YouTube pins, negative TTL meaning, parsed lyric sources and useful stale fallback.

Protect durable user data during any cache/schema change. db/connection.rs:33–43 treats every open/migration error as corruption; :88–89 deletes a database component if quarantine rename fails. Settings, manual YouTube pins and local listening history cannot all be restored from Spotify. Classify failures, retain recoverable originals and handle migration/permission/lock errors without blanket deletion. A faster cache must preserve that data.

Validation: many covers/feeds/resources, failed removal/open file on Windows, cyclic cache symlink, interrupted write, concurrent readers and account switch. Measure both actual allocated bytes and hit rate; shrinking memory by repeatedly forcing cold requests is not a win.

### 13. Shorten native lock and credential critical sections

Evidence: commands/playback.rs:97–108; auth/mod.rs:224–234 and keyring commit paths; auth/http.rs:195–228; commands/auth.rs/profile/settings/credentials; playback/mod.rs:224,589–618.

Playback initialization holds the main state lock over settings/network/player creation, delaying controls and other session consumers. Share a separate initialization future/generation and publish under a short lock. Already-valid tokens can use a checked fast path before the refresh lock. Consolidate profile/status settings into coherent reads and avoid rewriting unchanged snapshots.

Move blocking keyring, filesystem, wallpaper subprocess, browser-launch wait and cache/probe work to bounded workers with explicit generation/commit ordering. The loopback listener deadline currently stops protecting an accepted incomplete socket; bound read/write bytes/time and honour auth cancellation. Keep existing startup keyring overlap and macOS safety probes.

Validation: slow warmup plus pause/state/lyrics, refresh/logout races, stalled callback preconnect, credential edits during validation, startup without audio device and safe probe timeout. No old account resurrection, double Connect initialization or changed onboarding inputs.

### 14. Park idle work and reconcile media/integration state

Evidence: playback/youtube.rs watcher:453; episode polling; media-controls bounded channel; Discord clear/tick; Last.fm keyring/auth flow; theme/window/mem_trim.

YouTube's 250-ms watcher runs empty/paused and can resend media-playing state on position events. Audio device and wallpaper probes also have periodic work. Use event/transition-driven parking where it retains instant resume/hotplug. Keep bounded media queue capacity but reconcile latest final state instead of losing it on a full channel. Clear retained Discord metadata when playback ends.

Pool/cache notification artwork with track-generation identity. Last.fm should use a revocable native credential snapshot/worker and a true wall-clock auth deadline; disconnect must invalidate late auth completion. These are secondary services and must not hold up playback/search.

Measure native WebView flags, idle working-set trimming and whole-process power-throttling policy in packaged builds. Current trimming already waits ninety seconds continuously hidden/minimized and runs once per idle stretch. Do not repeat a historical active-use trimming diagnosis or assume requested renderer flags are honoured on every runtime.

Validation: long idle/paused/hidden playback, window resume, default-device change, Discord reconnect after stop, disabled features, missing notification image, Last.fm disconnect during auth. Record CPU wakeups, battery/CPU, resume faults and underruns alongside responsiveness.

### 15. Reuse DSP output and improve callbacks only with audio evidence

Evidence: sink.rs:303–308,410–444; playout.rs:300,392–405; resample.rs; stretch.rs.

Producer conversion/resampling/stretching can allocate multiple vectors per packet. Reuse bounded scratch and output buffers; Spotify does not currently exploit the same spare-buffer return pool as the YouTube producer. Existing stereo resampling/window reuse is already implemented.

If callback traces show contention, use a preallocated producer/consumer queue and off-callback buffer reclamation rather than callback mutex/free paths. Fix manual-flush epoch/current-chunk/filter reset deliberately. WSOLA has no explicit end-of-input tail flush; test final words and speed transitions before altering buffer APIs.

Validation: sample-equivalence where expected, mono/stereo/channel order, every rate/speed, short clips/EOF, seek/skip versus natural tail, pause/resume, actual device callbacks and p99 callback duration/underruns. Preserve selected audio quality, pitch, transcript/lyric timing and the four-second resilience capacity unless a separate measured buffer policy is agreed.

### 16. Keep active platform error paths bounded

Evidence: vendor/cpal/src/host/wasapi/stream.rs:161–166,351–356; ALSA process_output/drop; CoreAudio callback/listener review.

WASAPI can fail to close a command event handle if its worker already exited, and its fallback priority call casts a thread ID to a handle. Correct cleanup/priority on the real callback thread with checked status. ALSA output retries errors/partial writes without appropriate progress/cancel handling, risking CPU loops, duplicated data or a blocked join. These are supported desktop correctness/resource concerns.

Android capability arrays and web-native closure lifetimes have separate dormant-backend findings. They do not explain current Windows app latency. CoreAudio listener Send and device-open safety patches must remain. Avoid replacing CPAL/rodio versions or lowering hardware periods merely to make an unverified latency claim.

Validation: device removal, callback/worker error, repeated stream opens/closes and handle/thread counts on each actual shipped OS. Test ALSA partial/error progress and cancellation, and retain Mac no-device/VM safety.

## Optional frontend-internal work after the backend phase

These proposals require editing frontend source and are **outside the frontend-untouched implementation boundary**. They can retain the design, but need separate behavior/screenshot verification. None was implemented.

| Opportunity | Evidence / cost | Preservation requirement |
| --- | --- | --- |
| One generation-owned preload/snapshot coordinator | usePlayerControls.ts:212,253 schedules obsolete 1.5-second preloads; remote actions schedule independent snapshots | Same optimistic buttons/skip semantics; backend cancellation can reduce waste first, but removing obsolete scheduling requires caller changes. |
| One reactive device poll owner | useDevices shares listeners/promises but owns per-consumer timers | Prompt remote state, local-playing poll suppression, transfer and focus recovery. |
| Open/visible-demand gating | Closed Immersive uses ambient; AddToPlaylist queries/derives while closed; own-top profile queries run for other users | Keep same opening transition and warm useful data; no blank opening caused by careless deferral. |
| Ambient byte LRU | lib/ambient.ts retains JPEG/palette per cover indefinitely | Pin current/transition covers and retain exact colors/blur; accent cache is already separate and bounded. |
| Stable lyric row/voice/word models | Both renderers recreate word arrays/tone objects at active-row boundaries | Every glyph, word end, voice role, translation, contrast and offset. Existing frame-independent sweep remains. |
| Queue persistence/drag separation | Capped queue/context still serialize large snapshots on drag changes | Same reorder/removal/history and crash-recovery semantics; commit durable order at a deliberate boundary. |
| Measured list windowing or on-open menus/portals | Full TrackRow hooks/menus and long local queues/sidebar mount despite content-visibility | Exact scroll extent, focus, occurrence identity, drag behavior, menus and visible layout. Start with mount traces, not CSS redesign. |
| Cached local filter/sort/tree models | Repeated normalization/sorting and descendant/index work | Identical result set, locale/ties/order, immediate filter feedback and folder counts. |
| Memo correctness before more memo | TrackRow comparator ignores metadata/callbacks | Current context and permissions must update; freezing stale props is not a valid optimization. |
| Video focus/hidden policy and smaller observer/timer scope | Visible-unfocused decode/readback; style MutationObserver geometry; countdown/SyncButton clocks | Same foreground videos/motion, seconds when displayed, scrollbar geometry and instant resume. |
| Correct future-track lyric upgrades and pagination ownership | Current-only upgrade events; late show loadMore and uncancelled intent work | Same alternate choices, fresh lyrics, route/session ownership. |
| Optional shell/module splitting | Eager immersive/dialog imports; Playground primitives used in production | Compare built output; preserve focus restoration, loading geometry and entry animation. Romanization is already dynamic. |

Typeface/copy/accessibility candidates are in ui_consistency_notes.md. They are optional accuracy/design decisions, not a prerequisite for backend speed work.

## Existing improvements to preserve

Shared pooled HTTP clients, compressed responses/HTTP2, SQLite WAL/NORMAL/pool, batched 200-ID artist hydration, query-key deduplication, lazy routes, narrow player selectors, position persistence equality, a single position ticker, sized/DPR-safe covers, bounded queue/Jam/accent stores, per-track lyric single-flight and staged provider budgets, parsed lyric caches, active-word DOM sweep, YouTube Arc/two-track preparation and dropped-child cancellation, bounded audio ring/epochs/heard clock, tail-aware advancement, optimized stereo resampling/WSOLA windows, native media/log workers, auth generations and macOS device safety are already present.

Deleting features, reducing quality, shortening correctness waits, over-memoizing stale rows, increasing every request limit or shrinking every buffer would not meet the user's requirement.

## Measurement and regression plan for later implementation

Instrumentation and tests below are recommendations, not results from this audit. Implement them only in the later code phase. Record packaged release builds separately from development/StrictMode; use consistent hardware, runtime and service/cache conditions.

| Scenario | Measure | Comparison/acceptance |
| --- | --- | --- |
| Cold and warm launch | Process start→window usable; first useful Home data; playback ready; startup critical spans; CPU/working set/JS heap | Separate DB/keyring/session/network versus JS mount cost. Same content and placeholders; no hidden blocking init moved into first click. |
| Search while typing | Keystroke→query admission/headers/mapped response/render; p50/p95; obsolete calls and upstream counts | Keep existing debounce/result-key behavior. Identical results; no obsolete work competing with the next query. |
| Search during sync/preload | Admission wait and completed latency; 429s; writer/backlog bytes | User-action priority without rate-limit violations or missing eventual library results. |
| Warm/cold album/artist/playlist/Home | Hit/miss latency and upstream/SQL counts; first play from card | Cached response should not wait on unnecessary token/session/disk work. Same order and metadata. |
| First audio, resume, next/previous | Click→accepted intent→first decoded chunk→first heard sample; p50/p95; requests and allocations | Same source/quality/recording/queue. Normal nonzero resume should not reload a matching track. |
| Rapid skip/source/logout/recovery | Work still active after supersede; session/device/socket/task/handle counts | No old audio/events/cache publication; task counts settle; decoder uses recovered session. |
| Podcast seek/pause | Commit latency, needed-byte wait, retained byte budget, audible/transcript position | Range and non-range support; no A seek applied to B; no evicted-byte timeout trap. |
| Lyrics initial/upgrade/source change | First accepted correct response, final selected/alternate state, provider requests/bytes/CPU | Never substitute inaccurate fast text; force/transient misses/source revisions behave correctly. |
| Large library/queue/show/transcript | Mount/commit/long tasks, scroll frames, DOM nodes, observers, IPC payload/serialization | Same layout/filter/focus/occurrences. Backend-only work will not erase frontend mount costs. |
| Playback CPU and long-session memory | Decoder/DSP/callback p99, allocations per audio second, underruns, RSS/JS/GPU/native cache bytes | Resource usage reaches bounded plateaus; audio continuity/quality stays the same. |
| Idle/paused/hidden→resume | Wakeups/network/device polls, CPU/power, trim/page faults, immediate resume | No polling spin or stale media state; no hidden-playback underrun regression. |
| Offline/outage/malformed services | Absolute failure/cancel latency, memory bounds, stale fallback, uncertain mutations | Useful cached data stays available; Retry-After respected; no duplicate mutation or permanent transient-negative cache. |

After obtaining baselines, set explicit target improvements for each costly span rather than inventing a universal speedup. The first backend milestones should visibly reduce duplicated work and response-path waits even before any frontend refactor; verify the measured user latency tracks those reductions.

## Implementation milestones

1. Add/collect a packaged baseline; fix lifetime/EOF/recovery ownership and correctness-sensitive cache boundaries first.
2. Add native in-flight sharing, coherent revisions/cooldowns, scheduling budgets and a bounded persistence writer; accelerate the same search/detail/Home responses.
3. Bound transport/streaming, fix episode/range/seek behavior, and improve lyric first-response/background publication under the same content rules.
4. Consolidate Stats/recommendations/media/credential work; verify cache/idle resource plateaus.
5. Reuse audio producer buffers and address measured platform/callback costs under audio regressions.
6. Consider optional frontend-internal improvements only if traces still show material mount/render/serialization cost. Keep CSS/assets/design intact unless an explicitly chosen consistency fix is wanted.

Release gates should include the existing frontend/native tests, cached snapshot/occurrence/account races, interrupted streaming, audible seek/end/speed tests and representative UI snapshots. The current release workflow checks versions/changelogs but does not run those correctness/performance gates.

The audit is complete: 540/540 original files reviewed, all original hashes unchanged, and only nineteen Markdown documents added. The reconciliation is recorded in verification_notes.md. No speed improvement is claimed as implemented or benchmarked here.
