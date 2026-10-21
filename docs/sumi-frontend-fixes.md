# Frontend audit fix pass

Branch: `fix/sumi-frontend-audit`

This pass applies the Sumi Fix workflow, with accessibility, component, responsive-layout, design-memory, and micro-copy guidance. It preserves the existing dark music-player design. Audit numbers below refer to the earlier screenshot audit; grouped entries describe the scope implemented, not a claim that every part of each finding is closed.

## Changes

| Area | Before | After | Audit references |
| --- | --- | --- | --- |
| App layout | Content could scroll behind the player; compact sidebar thresholds disagreed | Player has reserved space; thresholds agree at 1000px; page content has a maximum width | 5, 97, 98, 117 |
| Player and lyrics | Small seek target, hidden volume thumb/timestamps, no lyrics heading or close control | Larger seek target, visible thumb and timestamps, Lyrics heading and close button | 3, 4, 10, 73, 118 |
| Navigation | Low-contrast shortcut hint, inconsistent Friends naming, no skip link | Clearer hint and labels, round account control, keyboard skip-to-content link | 2, 8, 9, 103 |
| Cards | Square artist framing, clipped descriptions, unlabeled card links | Circular artist art, two-line media captions, accessible link names | 19, 24, 25, 36, 58, 104 |
| Home actions | Mouse-oriented quick actions and nested recommendation buttons | One native button per action, keyboard activation, named playback controls, no permanently reserved play-button gap | 20, 102 |
| Shared appearance | Faint secondary text, blue liked-button blobs, missing token aliases | Stronger secondary text, quieter liked state, shared surface/radius/shadow tokens | 26, 38, 45, 99–101 |
| Accent colors | Midtone accents could receive unreadable white text | Black/white selection guarantees at least 4.5:1 for the accent-text token | 99 |
| Page actions | Primary labels disappeared at compact widths; secondary actions were separated | Labeled Play/Shuffle controls wrap alongside secondary actions | 44, 96 |
| Library and metadata | Generic library heading, unclear sort arrow, uppercase playlist count, missing album total | Tab-specific headings, Ascending/Descending labels, sentence-case song count, album duration | 31, 34, 43, 47 |
| Podcasts | Raw duration/language values and heart-based Follow affordance | Readable durations and progress, language names, Plus/Check follow state | 62–65 |
| Search | Blank search redirected away; clearing could leave stale results | Useful empty prompt and library link; clearing restores that prompt; podcast results included in empty-result detection | 6, 94, 111 |
| Settings | Centered heading, conflicting backing-vocal wording, unclear disabled service action | Left-aligned heading, consistent copy, required switch labels, disabled-action explanation, Built-in badge | 70, 75, 76, 78, 79 |
| Menus and sharing | Translucent menus, mixed destructive actions, technical share wording | Solid viewport-bounded menus, destructive actions last with separator, clearer share and playlist-sharing copy | 49, 50, 113 |
| Loading and artwork | Centered loaders on detail pages; failed images could stay blank | Detail-page skeletons and a shared failed-image fallback | 27, 57, 91 |
| Stats | Tight statistic tiles and unexplained heatmap intensity | Wider minimum tiles, more hour labels, intensity legend | 66, 67 |
| Production boundary | Production components imported controls from playground files | Shared UI controls extracted; playground route and bundle limited to development | 88–90, 106 |
| Supporting copy | Queue instructions referenced a nonexistent plus control | Actionable queue instructions, consistent empty-queue wording, clearer credit roles | 107, 114, 115 |

Additional changes soften the greeting weight, align the Liked Songs treatment with the accent, deduplicate recommendation IDs, brighten disabled carousel arrows, add artwork boundaries, and improve immersive lyric contrast.

## Validation

- `pnpm test`: 19 files, 101 tests passed. Includes an accent-contrast regression test across dark, midtone, and bright colors.
- `pnpm build`: TypeScript and production build passed. Playground is absent from production output. Existing main-chunk size warning remains.
- `git diff --check`: passed.
- Browser checks used a local mocked Tauri/Spotify harness; no real account mutations. Checked compact (800×520) and wide (1440×900) layouts, Home, Library artists, Settings, Playlist, Lyrics, Search, and signed-out behavior.
- Keyboard Tab then Enter on the skip link focuses `main-content`.
- Search typing opens results; Ctrl+A and Backspace restores the empty prompt.
- Signed-out UI omits playback controls. Home browser error check was clear after fixing nested buttons.
- Screenshots are kept locally under `.planning/sumi-fixes/`.

## Remaining scope

This is not a complete closure of the audit. Remaining work includes paused-lyric fading, error-state consistency, broader spacing/inline-style cleanup, toast and panel interactions, and raw image elements outside the shared artwork component. Recommendation deduplication is by Spotify ID, not by recording identity. Narrow layouts with a side panel can still hide secondary player controls. Only shared accent-text consumers benefit from dynamic contrast selection; hardcoded colors elsewhere need further review.

New-playlist flows, playlist date-added metadata, podcast mark-played features, and other backend-dependent additions were not introduced. Native window behavior, live Spotify playback, and all runtime error permutations still require testing in the desktop application.

## Sumi qualitative assessment

These are subjective design estimates for the inspected surfaces, not accessibility certification or measured product scores.

| Dimension | Before | After | Main improvement |
| --- | --- | --- | --- |
| Typography | 6/10 | 7/10 | Clearer hierarchy and metadata |
| Color | 5/10 | 7/10 | Contrast and semantic tokens |
| Spacing | 5/10 | 6/10 | Player clearance and action grouping |
| Layout | 5/10 | 7/10 | Consistent compact behavior |
| Components | 5/10 | 7/10 | Artwork, loading, and menu states |
| Accessibility | 4/10 | 7/10 | Keyboard actions, labels, and contrast |
| Code quality | 6/10 | 7/10 | Shared controls and production boundary |
