# Entire codebase performance audit

## Goal

Review every project file, trace all UI actions and backend/playback flows, and recommend substantial speed and resource improvements while preserving the existing UI and UX. Write only Markdown. Do not implement changes, install dependencies, run builds that write artifacts, or overwrite the user's existing edits.

## Next Step

Audit complete. Start with performance_audit.md for the prioritized roadmap; review_progress.md records all 540 reviewed original files. Final reconciliation verified every original hash and nineteen Markdown-only additions.

## Current Phase

All five phases complete as of 2026-10-10.

## Phases

### Phase 1: Inventory and audit setup

- [x] Read the invoked planning-with-files skill and existing session state.
- [x] Record the user's constraints and initial Git state.
- [x] Inventory tracked, untracked, ignored project files, vendored code, assets, and generated-file boundaries.
- [x] Create review_progress.md with one entry per project file.
- **Status:** complete

### Phase 2: Frontend and interaction audit

- [x] Read every frontend source, stylesheet, test, and asset reference.
- [x] Map routes, components, buttons, menus, keyboard controls, stores, hooks, subscriptions, and IPC calls.
- [x] Record rendering, startup, loading, search, image, animation, and memory costs.
- [x] Record optional visual inconsistencies without changing the UI.
- **Status:** complete

### Phase 3: Backend, storage, and service audit

- [x] Read every first-party Rust source, migration, configuration, and example.
- [x] Trace authentication, HTTP, search, catalog, library, recommendations, podcasts, lyrics, social, settings, and persistence.
- [x] Assess concurrency, cancellation, caching, queries, locks, IPC payloads, retries, and resource ownership.
- **Status:** complete

### Phase 4: Playback and vendored code audit

- [x] Trace local, Connect, YouTube, episode, queue, seek, preload, speed, and end-of-track behavior.
- [x] Read every vendored source, manifest, test, example, and patch note.
- [x] Assess realtime audio, decoder/resampler/stretch allocation, buffering, network fetches, and cleanup.
- **Status:** complete

### Phase 5: Verification and synthesis

- [x] Inspect all remaining docs, tooling, workflow, dependency, lock, and binary asset files.
- [x] Reconcile every inventory entry with an honest review status.
- [x] Verify high-impact findings with targeted cross-file reads or read-only experiments.
- [x] Produce system maps, interaction catalog, prioritized performance roadmap, and measurement/UX preservation criteria.
- [x] Verify all original files, including original Markdown, are unchanged.
- **Status:** complete

## Deliverables

- review_progress.md: per-file checklist, scope, baseline hashes, and review notes.
- findings.md: evidence and working discoveries.
- system_map.md: dependency networks and end-to-end workflows.
- interaction_map.md: routes, controls, and their effects.
- performance_audit.md: prioritized findings, risks, and implementation order.
- ui_consistency_notes.md: optional UI observations only.
- progress.md and this plan: resumable audit state.

## Decisions made

| Decision | Reason |
| --- | --- |
| Keep all writes to Markdown | Explicit user requirement. |
| Audit working-tree contents, including existing changes | The running project includes those changes; preserve them. |
| Separate confirmed code costs from hypotheses and measured costs | No invented speedups or benchmarks. |
| Inventory generated/dependency directories as boundaries | Build output and installed packages are distinct from project and vendored code; disclose what was inspected. |
| Work in one agent | No explicit delegation authorization. |

## Errors encountered

| Error | Attempt | Resolution |
| --- | --- | --- |
| Initial combined skill/tool listing exceeded the output limit | 1 | Read focused skill and tool descriptions in smaller calls. |
| Broad recursive metadata listing resolved the PowerShell tilde to the user profile and exceeded the output limit | 1 | Stop that read and use absolute workspace paths with a bounded Python walk; no contents outside the workspace were intentionally reviewed. |

| Python console UnicodeEncodeError when reading shortcut glyph | 1 | Use UTF-8 stdout and reread omitted lines; no source edits. |

## Constraints and completion criteria

Audit completion requires every original project file to have an explicit reviewed status or a disclosed non-code/sensitive/generated classification. A checkbox means the stated review actually occurred, not merely that a path was discovered. Claims must include file/line evidence. Existing user changes must remain untouched. Runtime tests that write database/cache/build files are outside this Markdown-only audit.

