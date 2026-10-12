# Musique implementation tasks

Created 2026-10-11 from the completed codebase audit. All implementation tasks below are pending. Creating this file does not implement a fix or establish a measured speed improvement.

The audit reviewed 540 original files at commit `2d55bb477c40a7b27affc35cac64062baf1c7e74`, including the existing working-tree edits. Source locations describe that audited state and must be rechecked before implementation. Preserve existing user edits, especially the authentication changes and `SignInPrompt.css`.

## How to use this backlog

Each task records the problem, proposed work, evidence, dependencies, and completion checks. Check a task only after its acceptance conditions pass. Record the implementation commit, validation performed, measured result where relevant, and remaining limitations beside the task. A completed investigation may conclude that no code change is warranted; record that conclusion and its evidence.

Priority describes implementation order and impact, rather than a formal security severity rating:

| Priority | Meaning |
| --- | --- |
| P0 | Protect durable data, account isolation, credentials, or essential playback correctness. Address early. |
| P1 | Remove confirmed functional defects, obsolete work, unbounded waits, or substantial duplicated work. |
| P2 | Improve resource use or secondary services after the shared ownership work. |
| P3 | Optional cleanup, UI consistency decisions, or inactive-platform work. |

Evidence labels distinguish what the audit establishes:

| Label | Meaning |
| --- | --- |
| Confirmed | Source tracing or the documented in-memory checks establish the mechanism or defect. Its frequency and runtime impact still need measurement. |
| Structural | Source establishes repeated work, allocation, missing bounds, or potentially slow sequencing. Benefit depends on workload. |
| Investigate | A live reproduction, profile, contract decision, or target-platform check must precede a fix. |
| Optional | Frontend, copy, accessibility, branding, or packaging work outside the initial backend-only phase. |
| Dormant | Applies to a backend or configuration outside the ordinary shipped Windows runtime. |

## Scope and preservation requirements

- The initial implementation phase should preserve frontend components, CSS, layout, motion, fonts, artwork, and interactions. Backend DTOs must retain their intended values, ordering, null handling, and ownership semantics. Frontend tasks are listed separately for a later phase.
- Preserve audio source, selected recording, bitrate preference, pitch, channel ordering, lyric/transcript timing, heard position, natural tail, resume progress, and local/remote/Jam roles.
- Preserve useful stale/offline reads, manual YouTube pins, alternate lyric sources, sticky source choices, settings, and local listening history.
- Preserve pooled HTTP clients, existing query deduplication, lazy routes, bounded stores, parsed lyric caches, lyric single-flight, audio worker separation, the bounded audio ring, and native media/log workers.
- Give each asynchronous operation an explicit account, resource, revision, and cancellation owner where applicable. Clearing a cache does not invalidate a running producer by itself.
- Set limits and latency targets from representative measurements. Increasing all concurrency, lowering audio quality, deleting sections, or shrinking every buffer does not satisfy the performance objective.
- The audit ran read-only compiler/schema/graph checks. It did not run application builds, native/frontend tests, live services, or runtime profiling. The implementation phase must collect those results.
- Documenting all work here is not authorization to publish a release, change the product design, or implement every optional decision.

## Recommended delivery order

| Milestone | Tasks | Exit condition |
| --- | --- | --- |
| Baseline and preservation | T001-T003 | Reproducible measurements, captured behavior contracts, explicit account/revision rules. |
| Data and playback safety | T004-T019 | Recoverable data retained; current session/intent ownership correct; reliable playlist/library snapshots and complete pagination contracts. |
| Request and persistence ownership | T020-T032 | Shared misses, fair admission, bounded writer/transport, prompt controls, correct cache publication. |
| Streaming and lyrics | T033-T045 | Bounded downloads and seek waits; cancellation propagates; lyrics retain accuracy and manual selection. |
| Secondary services and audio | T046-T058 | Coherent Stats/recommendations/media state; bounded idle work; measured audio/platform fixes. |
| Optional frontend work | T059-T074 | Selected tasks reproduce their issue or show material cost; behavior and visuals verified. |
| Tooling and release readiness | T075-T080 | Required checks, patch records, packaging evidence, and release validation complete. |

The ranges group related work. They are not a strict serial dependency chain. Address urgent correctness fixes alongside baseline collection; do not delay a known data-preservation fix for a full benchmark campaign.

## Baseline and shared contracts

### T001 - Collect packaged performance and resource baselines

P1, Structural. Evidence: [measurement plan](performance_audit.md#measurement-and-regression-plan-for-later-implementation), [verification limits](verification_notes.md).

- [ ] Record the commit, dirty-tree state, OS, hardware, audio device, WebView/runtime, build mode, account/market, enabled integrations, and cache state for each run.
- [ ] Capture cold/warm launch, first useful Home data, search, search during sync/preload, details, first audio, resume, next/previous, podcast seek, lyrics, long-session memory, and hidden/paused resume.
- [ ] Separate admission wait, authentication, upstream request/body, mapping, SQL, IPC, render, decode, and first-heard-sample spans. Report p50/p95 for user latency and p99 for callback work where useful.
- [ ] Count upstream requests, retries, cancellations, writer backlog, tasks/sessions/devices/sockets/handles, allocated cache bytes, CPU wakeups, JS/native/GPU memory, and underruns.
- [ ] Compare packaged release builds separately from development and StrictMode. Use repeated runs with the same fixture and service/cache conditions.

Done when a reproducible baseline and per-change target exist. Avoid universal speedup percentages derived from source inspection. Instrumentation must redact credentials and avoid adding material work to audio callbacks.

### T002 - Establish regression fixtures for data, ownership, and audio

P1, Structural. Evidence: [interaction map](interaction_map.md), [system map](system_map.md), existing tests described in [findings](findings.md).

- [ ] Capture representative DTOs and ordering for Home, search, artist/album/playlist/show/library/profile/Stats, including missing metadata, empty data, and duplicates.
- [ ] Prepare fixtures for account switch/logout during acquisition and commit, initial credential failure followed by recovery, slow/failed services, and rapid transport intents.
- [ ] Prepare deterministic range and non-range media responses, incorrect ranges, truncated/oversized bodies, changed object identity, and cancelled requests.
- [ ] Preserve existing lyric parser/alignment/auth/listener/cover/queue/folder/transcript/progress tests. Add meaningful tests with the corresponding implementation, rather than tests that merely restate it.
- [ ] Capture short clips, EOF, natural transitions, seeks/skips, speed changes, mono/stereo, device switching, and heard-clock comparisons.

Done when each milestone has an explicit regression contract. Dependencies: T001 for measured baselines. This task defines test needs; it does not require a broad speculative test suite before fixes begin.

### T003 - Define account, resource revision, and cancellation ownership

P0, Confirmed cross-service publication gaps. Evidence: account/cache/lyrics/recommendation/library/playback sections in [findings](findings.md).

- [ ] Define which data is account-specific, market-specific, catalogue-shared, source-specific, or manually pinned, and identify its authoritative revision.
- [ ] Carry the captured account epoch and relevant resource revision through acquisition, background work, events, and final database/cache publication.
- [ ] Check ownership immediately before commit/publication, including inside paginated operations. Use a transaction or synchronized publication boundary so the check cannot race the mutation.
- [ ] Specify shared-request cancellation behavior. One consumer leaving must not cancel work still needed by another; the last consumer or invalidated generation must release obsolete work.
- [ ] Retain cleanup guards for in-flight registries and pending request maps on success, failure, timeout, and cancellation.

Done when logout and account/source changes cannot resurrect old state. This is a contract used by later tasks, not a requirement to build a single universal framework.

## Data protection and playback correctness

### T004 - Preserve databases on open and migration failures

P0, Confirmed. Evidence: `src-tauri/src/db/connection.rs:33-43,88-89`; [cache/data preservation recommendation](performance_audit.md#12-make-cache-budgets-and-publications-coherent-across-services).

The current recovery path treats any open or migration error as corruption, and can delete a database component when quarantine rename fails. Settings, manual matches, and local history are not fully recoverable from Spotify.

- [ ] Classify lock, permission, migration, disk, and actual corruption failures before selecting recovery.
- [ ] Preserve recoverable originals and SQLite sidecar consistency. Make failed quarantine non-destructive and report a useful error.
- [ ] Define a backup/recovery procedure before schema changes; do not overwrite the only copy during a failed upgrade.

Done when injected open/migration/rename failures leave the original data recoverable. Validate existing database upgrades and settings/pins/history retention. Dependencies: T002. Revisit this before every later migration.

### T005 - Correct vendor token cache races and redact credentials

P0, Confirmed. Evidence: vendored `token.rs:79-82`, token/login5/spclient trace statements; [vendor findings](findings.md#vendored-core-transportsessionauthentication).

- [ ] Perform token lookup, expiry removal, and result cloning under one consistent lock; do not reuse a vector index found before concurrent removals.
- [ ] Include client identity and scopes in token cache identity, with session/grant revision where required.
- [ ] Redact tokens and secret-bearing structures in trace/debug/error paths while retaining safe request/status diagnostics.

Done when concurrent expiry tests cannot use invalid indices or another client's token, and log fixtures contain no token material. Preserve auth fallback and entitlement checks. Dependencies: T003.

### T006 - Restrict settings DTOs to public preferences

P0, Confirmed. Evidence: `commands/settings.rs:51`, authentication/settings review in [findings](findings.md).

- [ ] Inventory frontend settings reads and define an explicit public-preference allowlist or typed DTO.
- [ ] Exclude stored playback tokens and other secret-bearing rows from `get_all_settings`.
- [ ] Keep credential getters native-only or limited to safe status metadata; preserve explicit user-entered credential save flows.
- [ ] Update storage/privacy documentation to reflect actual behavior without copying secret contents.

Done when preference hydration remains complete and secret rows cannot appear in the settings response. Validate unknown keys, older databases, and custom credential status. Dependencies: T003; coordinate documentation with T078.

### T007 - Centralize playback/session teardown

P0, Confirmed. Evidence: `playback/mod.rs:278-290,529-534`, retry/quality/cache replacement paths, Dealer strong ownership cycle; [roadmap](performance_audit.md#1-make-every-playback-replacement-release-its-old-task-group).

- [ ] Route successful replacement, retry, quality/cache changes, logout, failed initialization, and drop fallback through one explicit teardown owner.
- [ ] Stop audio, reject old publications, close Connect/Dealer, finish bounded listen reporting, and stop credential/background work owned by the old session.
- [ ] Retain and join task handles with a deadline. Use abort as the final fallback after graceful cleanup.
- [ ] Break Dealer reconnect ownership cycles explicitly; session invalidation alone is insufficient.

Done when repeated rebuild/login/source cycles leave one current session/device and task/socket/handle counts settle. No old audio, reports, events, or credential/cache writes may escape. Preserve graceful deregistration and final-report semantics. Dependencies: T003, T011, T030 for complete ownership.

### T008 - Stop Connect polling closed receivers

P0, Confirmed. Evidence: vendored `librespot-connect/src/spirc.rs:631-639`.

- [ ] Treat `recv() == None` as an explicit exit or disable that receiver branch permanently.
- [ ] Make shutdown effective before Connect establishment as well as after it.
- [ ] Cover command and player-event channels, including failed setup and replacement.

Done when closing either receiver cannot create an immediately ready select-loop spin. Validate idle, paused, playing, and initialization states; assert bounded exit and stable CPU. Dependencies: T007. Preserve ordered command handling and Jam role behavior.

### T009 - Bind recovered Player and Connect to the same session

P0, Confirmed. Evidence: `playback/mod.rs:715-720,803-814`; targeted installed librespot-playback 0.8.0 `SetSession` inspection; [recovery recommendation](performance_audit.md#2-give-the-recovered-decoder-player-the-recovered-session).

- [ ] Construct Player after final authentication or explicitly send the recovered session before further loads.
- [ ] Preserve player-event subscriptions, mixer/device ownership, prepared state, and heard-clock publication order.
- [ ] Ensure failed recovery releases all intermediate resources.

Done when forced initial credential rejection followed by successful recovery produces audio with Player and Connect using the same live session. No second manual retry should be necessary. Dependencies: T007.

### T010 - Own transport intents and resume matching audio in place

P0, Confirmed. Evidence: `commands/playback.rs:97-108,906-912`, Spotify intent gaps and accepted-history writes in [findings](findings.md#spotify-transportsession-lifecycle-complete).

- [ ] Use a separate shared initialization owner and publish under a short state lock, so network warmup does not hold the control mutex.
- [ ] Guard Spotify load/retry/pause/seek/backend-switch work with a coherent latest-intent/source generation.
- [ ] Resume a matching paused track at nonzero position without reloading; seek only when the request expresses an intentional position change or meaningful drift.
- [ ] Record recently played from authoritative accepted/audible transitions, with account identity, rather than from an obsolete command returning success.
- [ ] Keep preferences/backend state coherent and preserve ordered volume/mute persistence if commands overlap.

Done when a slow old initialization/load cannot overtake a newer pause/play, normal resume retains loaded audio, and superseded requests do not create false history. Validate local, remote, Jam, unavailable-track recovery, and source switches. Dependencies: T003, T007, T009.

### T011 - Bound listening report work and validate heard-audio accounting

P1, Structural and Investigate. Evidence: `playback/listening.rs:146-147,346-367,389`, CountingSink/frame accounting.

- [ ] Replace unbounded detached reporting with owned, bounded batching and an explicit backlog/durability policy.
- [ ] Preserve report sequence IDs, contexts, identical retry payloads, entitlement behavior, and zero-audio suppression.
- [ ] Reuse available audio identity metadata instead of fetching it for every report.
- [ ] Verify whether accepted decoded frames later flushed by skip/seek should count; compare against actual ring consumption before altering accounting.

Done when an unavailable report service plus rapid skips cannot grow retained sessions/backlog without bound, and teardown honors a documented report budget. Audible accounting changes require evidence and report regressions. Dependencies: T003, T007, T028.

### T012 - Keep Connect controls responsive during network publication

P1, Confirmed inline waits; Structural performance impact. Evidence: Spirc notify/context/disconnect/final-state paths in [findings](findings.md#complete-connect-fork-review).

- [ ] Bound network waits in command handling, first-context resolution, disconnect, and final state deletion.
- [ ] Publish local state snapshots promptly; move only replaceable state publication to an owned/coalesced worker.
- [ ] Preserve ordered queue mutations and non-replaceable commands. Keep final-state reconciliation when notifications are coalesced.
- [ ] Make dealer cluster/playlist throttling deliver a trailing invalidation rather than dropping the final change in a burst.

Done when slow/offline services do not stall local control handling indefinitely and the final remote state is reflected. Test control bursts, notification/refetch races, initialization shutdown, and Jam continuity. Dependencies: T007, T027-T031.

### T013 - Correct remote track/episode transfer and retry outcomes

P1, Confirmed. Evidence: `commands/connect.rs:147-155,234-291`, `useDevices.ts:215`.

- [ ] Build track and episode URIs through the existing playable-URI helper; never prefix an already-qualified episode URI with `spotify:track:`.
- [ ] Apply shared admission, deadlines, status classification, and provider cooldowns to direct remote state/play requests.
- [ ] Transfer/retry only for the appropriate failure class. Check and return the final retry outcome instead of reporting unconditional success.
- [ ] Use URL query builders consistently for device parameters and preserve transfer position/context.

Done when podcast transfer retains episode identity and position, failed retries report failure, and uncertain outcomes cannot blindly duplicate an applied action. Validate tracks, episodes, missing devices, 401/403/404/429/5xx, and Jam ownership. Dependencies: T027, T028.

## Library, catalogue, and database correctness

### T014 - Commit complete playlist snapshots atomically

P0, Confirmed. Evidence: `commands/spotify.rs:451-458,513-558`, `library/mod.rs:221-243`.

- [ ] Separate playlist list-metadata revision from the revision of fully committed track rows.
- [ ] Acquire pages under a shared playlist/account/snapshot fill owner.
- [ ] Replace rows and completeness/snapshot markers in one guarded transaction, after dependencies are ready.
- [ ] Reject obsolete account/snapshot fills at commit and prevent mixed old/new revisions.
- [ ] Represent a complete zero-item playlist explicitly for warm/offline reads.

Done when metadata-only sync cannot mark old track rows current, readers see an entire old or new snapshot, and empty playlists use cache correctly. Test overlapping sync/detail fills, cancellation, logout, changed snapshots, and pagination failures. Dependencies: T003, T004, T015.

### T015 - Preserve playlist occurrence identity and order

P0, Confirmed by in-memory schema reproduction. Evidence: `playlist_tracks` primary key, repository `INSERT OR IGNORE`, [data map](data_map.md), [verification](verification_notes.md).

- [ ] Design a migration that supports multiple occurrences of the same recording in one playlist.
- [ ] Use snapshot-scoped occurrence identity/order or authoritative item identity, rather than recording ID alone.
- [ ] Update insert, remove, reorder, and cache hydration contracts so one occurrence can be acted upon deliberately.
- [ ] Preserve duplicates in DTOs and document any frontend occurrence-key changes for the optional list phase.

Done when a fixture such as A, B, A survives persistence/readback in the same order and removal targets the intended occurrence. Validate migration from existing caches and no loss of durable user data. Dependencies: T004; coordinate T064, T070.

### T016 - Give manual and background library sync one owner

P0, Confirmed. Evidence: shared read gate, category-only epoch checks, fifteen-second logout gate, `is_syncing` always false in `commands/library.rs`.

- [ ] Coalesce or serialize sync per account/revision, including manual and periodic requests.
- [ ] Carry cancellation/epoch checks through every page and immediately before each transaction.
- [ ] Expose truthful sync state and preserve existing progress/feedback semantics.
- [ ] Stop obsolete work before purge; do not rely on a large category finishing within logout's timeout.

Done when manual/background sync cannot overlap unintentionally and logout cannot be followed by old rows reappearing. Validate large categories, repeated clicks, 429 waits, failure/retry, and account switch. Dependencies: T003, T014, T027, T028.

### T017 - Separate followed membership from browsed catalogue data

P1, Confirmed. Evidence: `get_my_playlists` selects all cached playlists; `sync_playlists` does not remove unfollowed membership; library mirror/album DTO findings.

- [ ] Model followed playlist membership independently from detail-cache existence and prune membership authoritatively during sync.
- [ ] Keep browsed playlists available offline without adding them to the user's library.
- [ ] Keep true owner identity separate from display name.
- [ ] Hydrate saved-album artists from existing associations.
- [ ] Make successful save/remove mirrors dependency-complete or retain a guarded membership intent; do not silently swallow a failed local mirror and show an incorrect liked state.

Done when browsing does not change library membership, unfollowed items leave the library, album artists are correct, and a successful mutation is reflected immediately. Dependencies: T003, T014, T016, T024.

### T018 - Batch database work and remove verified redundant indexes

P2, Confirmed duplication and Structural write cost. Evidence: catalogue/library repositories, duplicate saved-track `added_at` indexes in migrations 0001/0007, `get_saved_track_ids` unbounded IN list.

- [ ] Batch deduplicated artist/album/track dependencies in bounded transactions, preserving COALESCE enrichment, ISRC, and association order.
- [ ] Replace top-track/top-artist lists transactionally so concurrent readers cannot see partial replacement.
- [ ] Deduplicate and chunk saved-track ID lookup inputs according to the actual SQLite parameter limit.
- [ ] Remove the verified equivalent saved-track index through a migration, checking relevant query plans before and after.
- [ ] Investigate other redundant indexes and search-history timestamp ties using real query use; avoid blanket index deletion.

Done when large sync/write workloads have fewer commits without changed data/order, and indexed read plans remain suitable. Dependencies: T004, T016, T024. Extra archival or indexes require measured workload evidence.

### T019 - Complete catalogue and saved-show pagination contracts

P1, Confirmed gaps. Evidence: `SpPage` only retains items; album first-page tracks, artist first fifty discography items, saved-show five-hundred cap in [findings](findings.md).

- [ ] Retain pagination metadata needed by native callers and identify which commands promise full versus paged results.
- [ ] Support albums and artist discographies beyond embedded first pages without changing item order or dropping unavailable entries incorrectly.
- [ ] Replace silent saved-show truncation with complete bounded acquisition or an explicit paged contract.
- [ ] Share page requests and cancellation by account/resource/offset/revision; avoid a single unbounded response for very large resources.

Done when fixtures beyond every existing cap are reachable under the intended contract. Do not describe truncated data as a performance optimization. Frontend Liked Songs/show paging is T068. Dependencies: T003, T020, T027, T028.

## Shared requests, caches, scheduling, and transport

### T020 - Share native cold requests and overlapping metadata misses

P1, Structural. Evidence: `internal/cache.rs:173-194`, direct play APIs bypassing frontend query ownership, [native request recommendation](performance_audit.md#3-coalesce-cold-requests-at-the-native-boundary).

- [ ] Add narrowly scoped shared in-flight ownership for equivalent native requests and overlapping entity batches.
- [ ] Key by account/session/grant revision, resource, market, and content variant as appropriate; recheck freshness under the owner.
- [ ] Preserve requested order/cardinality, unavailable-item handling, useful stale fallback, and cache-only source selection.
- [ ] Remove failed/cancelled registry entries with cleanup guards and distinguish durable absence from transient failure.

Done when simultaneous identical calls produce one upstream acquisition and later requests recover after owner failure/cancellation. Test forced refresh, logout, expired credentials, and overlapping batches. Dependencies: T003; coordinate T021-T025 and T027.

### T021 - Share Pathfinder hash and vendor discovery refreshes

P1, Structural. Evidence: `internal/pathfinder.rs:106-149,191-197`, login5/client-token/AP discovery miss races.

- [ ] Share HTML/web-player bundle hash discovery, recheck under ownership, and persist only verified hashes by bundle revision.
- [ ] Acquire fallback OAuth only when the first-party request actually needs fallback.
- [ ] Coalesce login5, client-token, and AP resolution by their real session/client identities; retain intended Musixmatch/visitor refresh locking.
- [ ] Use short failure cooldowns and invalidate failed/stale discoveries without permanently blocking retry.
- [ ] Refresh depleted AP endpoint groups independently while retaining service preference/fallback order.

Done when concurrent Home/lyrics/detail/play/report requests avoid duplicate discovery and healthy first-party calls avoid unnecessary fallback token work. Validate one hash refresh retry and recovery from failed owners. Dependencies: T003, T005, T020, T028.

### T022 - Share auth/profile/credential validation work

P1, Structural. Evidence: valid-token refresh lock, sequential profile fallback/rewrite, Settings validation duplication.

- [ ] Add a checked valid-token fast path, rechecking expiry/revision under refresh ownership when refresh is needed.
- [ ] Share profile fetches by account/grant revision and avoid unchanged settings rewrites or broad invalidation when product is unchanged.
- [ ] Read related auth/profile/backend/preferences as coherent indexed snapshots rather than serial settings queries.
- [ ] Share custom-credential validation by credential revision and separate locally valid PKCE formatting from successful secret authentication.
- [ ] Preserve unknown versus Free/Premium plans, explicit empty-avatar removal, dual-grant onboarding, and logout ordering.

Done when simultaneous reads/validations do not multiply authentication work and stale completions cannot overwrite edited credentials. Dependencies: T003, T005, T020, T049. Frontend draft/observer changes are T060/T069.

### T023 - Give caches count, byte, age, and publication budgets

P1, Structural. Evidence: internal cache without eviction, recommendation full-map eviction, episode feed cache, prepared audio and ambient caches.

- [ ] Inventory memory/disk/query cache ownership and choose explicit byte/count/age budgets from T001.
- [ ] Use incremental eviction rather than clear-all cliffs where useful; pin current and transition resources.
- [ ] Preserve stale fallback, parser schema versions, negative outcome meaning, manual pins, and market/account isolation.
- [ ] Invalidate running fills when cache/source/account generations change and guard late publication.
- [ ] Measure allocated bytes and hit rate together; a smaller cache that repeatedly forces cold work can worsen latency.

Done when realistic long sessions reach stable resource plateaus without lost user data or frequent cold cliffs. Query retention changes and frontend ambient work require separate evidence and T062. Dependencies: T003, T020.

### T024 - Move unrelated persistence off response paths

P1, Structural. Evidence: search waits for up to sixty writes/history; album/artist/matching await prepared-data persistence; [writer recommendation](performance_audit.md#5-take-disk-work-off-searchdetail-response-paths).

- [ ] Prepare dependency-complete batches and return mapped search/detail/matching DTOs promptly.
- [ ] Use a bounded, account/revision-aware writer with error reporting, ordered commits, and controlled shutdown.
- [ ] Preserve richer metadata, ISRC, FK dependencies, search-history durability, and mutation ordering.
- [ ] Share an immediately available prepared snapshot or flush barrier for a following like/play/detail action that requires local data.
- [ ] Define backlog limits and admission behavior rather than detaching unlimited writes.

Done when delayed SQLite writes no longer extend unrelated response latency and immediate follow-up actions remain correct. Test logout, process shutdown, writer failures, and superseded search history semantics. Dependencies: T003, T018, T020.

### T025 - Shorten detail and cached Home/show paths

P1, Structural. Evidence: serial artist/discography/top requests, avatar consumers using full details, cached Home/show token/session prerequisites.

- [ ] Fetch independent artist detail/discography/top components concurrently within service budgets.
- [ ] Share narrow artist overview/avatar data so small consumers do not acquire full detail unnecessarily.
- [ ] Let fresh cached Home/show/saved-show reads use the authenticated account snapshot before token/session warmup when safe.
- [ ] Include timezone/content variants and personal resume state in appropriate cache identity/freshness rules.
- [ ] Keep useful stale data on live acquisition failure and preserve existing content/order/placeholders.

Done when warm data avoids unrelated session/auth/disk waits and concurrent detail calls share metadata. Test offline, expired token, account switch, first play from a card, and resume-point freshness. Dependencies: T003, T019-T024, T027.

### T026 - Consolidate supplemental/profile/social acquisition

P2, Confirmed duplicate-color gap and Structural repeated work. Evidence: artist extras, extracted colors, profile and social command sections in [findings](findings.md).

- [ ] Union album IDs for artist extras and split hydrated results back into original groups; overlap independent bounded kinds.
- [ ] Batch color cache reads/writes, dedupe URLs, and map every occurrence so a duplicate cold URL fills all requested positions.
- [ ] Parallelize independent profile user/playlist requests, distinguish valid zero playlists from failure, and invalidate only affected follow/profile keys.
- [ ] Replace repeated radio/context `Vec.contains` scans with ordered membership sets where equivalent.
- [ ] Validate invite URLs with parsed scheme/host and bounded response data; retain supported forms and guest endpoint allowlists.
- [ ] Optionally cache a confirmed Jam leave endpoint version per session and Odesli results with bounded freshness; preserve fallback.

Done when output grouping/order/cardinality is identical, empty profiles avoid unnecessary fallback, and malformed invites fail safely. Dependencies: T003, T020, T027-T028. Do not drop Jam payloads or supplemental content.

### T027 - Prioritize user work with bounded fair admission

P1, Structural. Evidence: shared eight-slot Spotify semaphore, direct Connect requests, Home/library fan-out and per-download YouTube ranges.

- [ ] Define explicit user action, current-track necessity, background refresh, and speculative preload classes.
- [ ] Reserve or fairly admit control/search capacity while allowing background work to progress.
- [ ] Budget per-service/account request, body collection, parsing, and byte usage rather than only response headers.
- [ ] Coordinate authoritative cooldowns and release admission during retry sleeps without allowing bypass storms.
- [ ] Preserve mutation ordering by resource/intent and avoid simply raising every concurrency limit.

Done when search/play/seek during large sync/preload has lower measured queue wait without starvation, extra 429 pressure, or missing eventual background results. Dependencies: T001, T003, T020, T028.

### T028 - Bound whole network operations and classify retries

P1, Confirmed missing deadlines and unsafe mutation retry paths. Evidence: app Spotify retry wrapper and vendor HTTP/SpClient nested retries.

- [ ] Cover admission, DNS/connect/proxy/handshake, auth, nested retries, headers, body, and decode with cancellable absolute deadlines.
- [ ] Bound retry attempts and use backoff/failover appropriate to error class.
- [ ] Respect the full authoritative Retry-After and share cooldown state; remove the thirty-second early-retry clamp behavior.
- [ ] Separate idempotent reads from playlist/create/add and other uncertain mutations. Reconcile uncertain outcomes before replaying.
- [ ] Limit compressed/uncompressed response and frame sizes before allocation/parse, with format-specific generous limits.

Done when stalled/429/5xx/malformed services fail or cancel within documented bounds, healthy fallback remains available, and retries cannot duplicate uncertain mutations blindly. Dependencies: T003, T027; extend through T029-T034 and T039-T044.

### T029 - Correct AP connection deadlines and frame validation

P1, Confirmed. Evidence: vendor `connection/mod.rs:67-71`, `socket.rs:24`, handshake/codec/channel/audio-key/Mercury unchecked boundaries.

- [ ] Place socket connect inside the timed asynchronous operation, covering DNS, proxy negotiation, and handshake.
- [ ] Use asynchronous multi-address connection fallback and an outer budget spanning AP attempts and authentication.
- [ ] Check handshake frame lengths before subtraction/resizing, encoded lengths before narrowing, and packet lengths before split/index/copy.
- [ ] Retain RSA server-signature verification and authentication/entitlement behavior.
- [ ] Handle short legacy authentication blobs safely, while recording that this path is dormant for the app's access-token login.

Done when stalled DNS/connect and malformed/short frames produce bounded errors rather than hangs, oversized allocations, or panics. Dependencies: T028. Validate healthy IPv4/IPv6/fallback routes and proxy behavior where supported.

### T030 - Own pending vendor requests, channels, and Dealer replies

P1, Confirmed missing cleanup/bounds. Evidence: audio-key pending map, Mercury/channel records, session sender, Dealer unbounded request/reply queues.

- [ ] Add deadlines and cancellation/drop cleanup for pending audio-key/Mercury/reply requests, preserving same-sequence audio-key retry behavior.
- [ ] Bound outstanding requests and queued payload bytes; define ordering/backpressure for commands that must not be dropped.
- [ ] Release closed channel/subscription records and retain handles for connect/dispatch/body/reply tasks.
- [ ] Bound Dealer close/reconnect waits and break the strong-session cycle during teardown.
- [ ] Bound gzip expansion and JSON/protobuf payloads; share immutable payloads where safe instead of cloning every subscriber body.

Done when cancelled requests and failing services cannot retain maps/tasks/session references indefinitely. Validate delayed same-sequence audio-key responses, orderly shutdown, and command delivery under backlog. Dependencies: T007, T028-T029.

### T031 - Correct Dealer routing and Connect context caching

P1, Confirmed. Evidence: `dealer/maps.rs:146`, `context_resolver.rs:145`, `state/context.rs:402-409`, queue deserialization.

- [ ] Visit matching descendant Dealer subscribers even after a parent match; combine results after traversal.
- [ ] Correct elapsed-time direction in context negative-cache expiry, classify transient failure versus absence, and bound/prune the map.
- [ ] Make clear semantics explicit for pending versus negative entries.
- [ ] Use URI/occurrence-aware context indexes for large merges, preserving UID and repeated-track semantics.
- [ ] Validate remote vector/payload limits and adopt an explicit tolerant/strict malformed queue-entry policy with safe diagnostics.

Done when parent subscriptions do not suppress children, failed contexts retry after the intended interval, and queue order/occurrences remain correct. Confirm actual app overlap before attributing a missing Jam event to routing. Dependencies: T003, T015, T030.

### T032 - Bound Spotify CDN fetches and publish audio cache files atomically

P1, Confirmed missing body/range/cache safeguards. Evidence: vendored audio `fetch/receive.rs`, vendor core `cache.rs:141-162`.

- [ ] Preserve the existing ten-second header timeout/mirror fallback; add whole-body deadlines and cancellation.
- [ ] Stream bounded pieces, validate Content-Range/actual sizes, and cap throughput-derived request sizes before arithmetic/coverage updates.
- [ ] Own receiver tasks through Session/Player/audio fetch, with current/preloaded-file budgets.
- [ ] Move copies/writes/scans/prune to bounded blocking workers and atomically publish complete files.
- [ ] Bound symlink traversal to the cache root with cycle detection, avoid rebuild rescans, and retain accounting for files whose deletion failed.

Done when oversized/wrong/truncated ranges and open-file deletion failures cannot corrupt cache coverage or exceed budgets silently. Preserve useful readahead and audio reads over persistence. Dependencies: T007, T023, T027-T030, T049.
