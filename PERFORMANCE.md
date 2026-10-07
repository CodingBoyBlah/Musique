# Performance changes, 7 October 2026

Branch: `optimzation/relevant-title`. Baseline: `bc2bc4ce99a48484e435dbedcc35667cd02476b8`.
The original `main` checkout and its uncommitted files were left untouched.

## Measured results

These are controlled component and kernel measurements, not whole-app speed claims.

| Workload | Baseline | Optimized |
| --- | ---: | ---: |
| Play/pause with 200 track rows mounted | 200 row renders | 1 row render |
| Play/pause with 25 album cards mounted | 25 card renders | 1 card render |
| Queue update: 200 rows, 5,000 queue entries | 1,001,000 ID reads | 6,000 ID reads |
| 60 position updates in the real player store | 60 JSON serializations, 26,220 characters | 0 serializations |
| 30 controlled lyric position updates | 29 renders | 1 render |
| Concurrent refresh burst from two device hooks | 6 IPC calls | 2 IPC calls |
| Device cluster listeners for two consumers | 2 | 1 |
| Stereo FIR resampling, 12 seconds of audio | 125.194 ms median | 67.451 ms median |

The FIR measurement used nine alternating rounds after warmup with
`rustc -C opt-level=3` on Windows, reducing median kernel time by 46.12%.
Samples matched bit-for-bit against the original across one, two and four channels
and 44.1→48, 48→44.1 and 44.1→96 kHz conversions. The same coefficients, tap count,
accumulation order, buffering and audio quality are retained. The change computes
both stereo channels in one traversal. Reproduce with:

```sh
node scripts/benchmark-resampler.mjs
```

The queue measurement includes 1,000 ID reads from existing queue persistence.
The shared index uses a WeakMap so it does not keep old queue snapshots alive.
Lyric clock values and active rows matched the baseline after small paused seeks,
combined seek/resume/pause updates, and manual offset changes.

## Loading and memory

- Extended metadata reads use one SQLite query per 200 entities, replacing one
  query per entity. Live responses share one write transaction per response batch;
  individual upsert statements remain. Cache timestamps, stale fallback, kind
  separation and database separation remain authoritative in SQLite.
- New-release artist requests use at most eight concurrent futures instead of
  up to 30 sequential requests. Top-track and top-artist time ranges fetch in
  parallel and apply results in their original order, preserving earlier successful
  updates if a later range fails. Existing API rate limits and retries remain.
- Library artist lookups reuse the selected IDs instead of scanning and sorting
  the same playlist, liked-song or recently-played tables again. Cached playlist
  artists move into their output rows without cloning.
- YouTube range downloads keep four futures and refill the window as ordered
  chunks arrive. They retain the original chunk size, download limit, validation
  and audio format. Direct `Bytes` responses remove the intermediate Vec copy.
  Cancellation and failures drop pending futures. Local HTTP tests verify byte
  order, rolling concurrency and absence of detached range jobs.
- Cover-accent memory is capped at 128 entries; existing persistent color values
  and extraction rules stay the same. Concurrent accent and lyric requests share
  in-flight work. Window focus listeners are shared and removed after the last
  consumer unmounts. Delayed preloads validate the current track and queue.
- YouTube matching avoids heap allocation for common ASCII titles and keeps
  Unicode scoring unchanged. Prepared audio replaces duplicate entries for the
  same track, and the playback watcher clones track identities only when emitting.

Whole-process memory usage and authenticated playback latency were not measured.
These changes remove specific copies, allocations, duplicate requests and retained
listeners; the queue index has a small memory cost per live snapshot.

## Appearance and validation

Every changed component's JSX is identical to the baseline after TypeScript parsing.
The production stylesheet is byte-identical (SHA256
`c4ecc773eb7a91bb1efd5d5078af1100fd0be382af447a7832b6eff43c953eab`).
No CSS, artwork, fonts, layout, motion constants, settings defaults or dependency
manifests changed. Track-row and album-card fixture screenshots are byte-identical.
Actual play, pause and queue control tests produced identical state and native calls.
Device fixtures reached identical state and released every listener on unmount.

Validation: 67 frontend tests passed; TypeScript, the production Vite build and
the Windows native release build passed; 244 native tests passed. The 15 existing
live-network tests remain ignored.
Browser fixtures mock native IPC; they do not substitute for an authenticated
Spotify/YouTube smoke test. The main JavaScript chunk is 502.41 KB raw / 151.67 KB
gzip versus 500.28 KB / 150.96 KB at baseline; these runtime improvements add about
0.7 KB gzip rather than reducing the initial bundle.

```sh
node node_modules/vitest/vitest.mjs run
node node_modules/typescript/bin/tsc --noEmit
node node_modules/vite/bin/vite.js build
cargo test --manifest-path src-tauri/Cargo.toml --locked --lib
cargo build --manifest-path src-tauri/Cargo.toml --locked --release
```

The frontend, audio and data-loading audits used Antigravity workers with
`gemini-3.8-flash` through the orchestration skill. Their changes were reviewed and
integrated in this isolated worktree, and all worker terminals were released.
