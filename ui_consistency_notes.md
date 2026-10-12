# Optional UI and behavior consistency notes

No frontend, stylesheet, asset or copy was changed. These observations are separate from the backend performance plan. The review used every original source/style file and historical README screenshots; it did not run a live browser or make pixel-measurement claims.

## Typography and visual consistency candidates

| Observation | Evidence | Suggested decision, without an automatic redesign |
| --- | --- | --- |
| Artist card titles are 14 px; album/media tile titles are 13.5 px. All three use weight 600 and a 17-px title line height. | src/components/ui/ArtistCard.tsx:186; AlbumCard.tsx:156; MediaTile.tsx:80 | These are closely related grid labels. Decide whether the 0.5-px difference is intentional; compare same-grid screenshots at all zoom levels before choosing a shared token. Do not change all titles merely because different typography exists. |
| Fixed pixel sizes coexist with rem tokens and responsive clamps. Settings alone has 11.5/12/12.5/13/13.5/14-px text and rem-based row titles. | Settings.tsx; index.css; style_inventory.md and render_attributes.md | Preserve the existing hierarchy. If consolidating later, compare actual computed size, weight, optical size and line box for equivalent roles. A global unit replacement could change wrapping and captions. |
| Inter is variable, but CSS declares six separate normal weights across seven subsets. | fonts.css; all seven WOFF2 containers in asset_review.md | Optional range declaration cleanup, not a typography change. Keep weight 400–900 behavior, optical-size axis and all Unicode coverage. Same URLs already share downloads. |
| Public SVG mark is a blue variant; native/banner mark is cream on a navy field. | public/Musique.svg; icons/icon.png; public/banner.png | Could be deliberate context-specific branding. Record it before any asset cleanup; do not automatically recolor the application. |
| The web document still references the Vite favicon. | index.html; public/vite.svg | Optional branding cleanup. It is not a meaningful performance target. |
| Some titlebar/search hover selectors are not pointer-gated like the common controls. | index.css; full CSS review in findings.md | Verify touch/focus/sticky-hover behavior. Preserve existing desktop hover appearance. |
| Broken CoverArt images have no explicit loaded-error placeholder transition. | CoverArt.tsx | A later robustness decision can keep current geometry and fades while handling errors coherently. No current broken-image runtime rate was measured. |

## Copy and data parity

| Observation | Evidence | Why it matters |
| --- | --- | --- |
| Empty lyrics copy says LRCLIB has nothing, though nine providers participate. | LyricsPanel.tsx:308; native lyrics provider registry | An optional provider-neutral message would describe the resolver accurately. Keep current empty-state size and motion. |
| Stats year/all/streak/first-ever semantics exceed the retained raw taste window. | recommend/mod.rs:158–172; StatsPage.tsx; commands/stats.rs | Raw events are trimmed to 180 days/12,000 rows every fifty signals. Either disclose that window or design durable daily/lifetime aggregates. Changing labels or data scope is separate from making the same Stats response faster. |
| Stats' offline/no-network description is conditional. | commands/stats.rs:182 | Missing catalogue metadata can cause HTTP hydration. Make native behavior cache-only if that is the intended contract, or adjust wording separately. |
| Saved album cards can have empty artists even though associations exist locally. | commands/library.rs:245; album_artists table | Backend DTO parity can restore intended text without redesigning the card. |
| Auth/environment/readme comments contain historical storage and privacy simplifications. | .env.example; README.md; credential/Last.fm API adapters | Document native stored-secret protection precisely. User-entered secrets necessarily cross IPC on save; get_all_settings currently returns more than public preferences. These are accuracy/privacy findings, not a request to expose implementation details in the product. |
| Some comments claim stronger lyric-sync or match gates than fallback code guarantees. | lyrics/mod.rs:355; lyrics/align.rs:224; youtube/matching.rs:201 | Agree on the intended recording/timing contract before changing candidate selection for speed. Do not call the fastest provider accurate solely because it answered first. |

## Interaction consistency and correctness

| Observation | Evidence | Current impact / future acceptance |
| --- | --- | --- |
| Podcast-only search matches can display no-results. | Search.tsx:514,630,637 | hasResults omits shows/episodes. Existing category layout should show the response it already receives. |
| Clearing top search on the results route does not clear the nonempty URL q. | TopSearch.tsx:103,238 | Local input and results URL can disagree. Decide the intended empty-search behavior explicitly. |
| Liked Songs shows at most the first 200 despite a larger count. | Library.tsx LikedSongsTab; useLibrary.ts | This is a pagination gap, not evidence that all larger libraries are fully represented. Adding paging/windowing needs a separate UX contract. |
| Cached playlists cannot retain repeated occurrences of one track. | playlist_tracks primary key; repositories; in-memory reproduction in verification_notes.md | Preserve the intended fresh playlist sequence while fixing schema/cache ownership. A recording ID is not an occurrence ID. |
| Queue rail understands local/remote/Jam ownership, but immersive queue and some history clicks use local state. | QueuePanel.tsx; ImmersiveQueue in Immersive.tsx | Do not treat those queues as interchangeable when memoizing or caching; preserve owner-specific play/enqueue/skip semantics. |
| A late show page can append into the newly selected show. | ShowPage.tsx loadMore | Capture show/request generation; new route must receive only its own pages. |
| TrackRow memo equality can retain old play callbacks or old same-ID metadata. | TrackRow.tsx custom comparator | Stable callbacks must reflect the current filter/context and permissions. More aggressive memoization would worsen this bug. |
| Carousel observers can fail to register when the list mounts after initial loading. | Carousel.tsx useCarousel effect | Arrow/fade/scroll behavior must work both for warm and asynchronously loaded data. |
| A refetched playlist object can overwrite an editing draft. A completed old mutation can close a newly opened modal. | EditPlaylistModal.tsx and global modal stores | Separate hydration from in-progress edits; associate completion with the captured modal target/account. |
| Paused lyric offset changes may not immediately re-arm active-row calculation. | lib/lyrics.tsx:50–78,150–161 | Needs a live/test reproduction. Keep timestamp/offset arithmetic and selected source unchanged while verifying. |
| Source/video cache freshness can retain a failed expiring URL. | useCanvas.ts; PodcastStage.tsx; native internal cache TTL handling | URL expiry was not checked live. Refresh on authoritative expiry/failure with unchanged quality and fit. Fully downloaded prepared audio is a different cache and needs no URL refresh. |

## Accessibility without changing the appearance

- Settings switches are supplied without the optional accessible label; their nearby row labels are separate spans. See Settings.tsx:454 and Switch.tsx. A future label/labelledby relationship can preserve every pixel.
- Condensed PlayActions remove visible text without an explicit accessible name in the relevant branch.
- CirclePlayButton can remain keyboard-focusable while visually hidden with opacity/pointer-events; Home RecTile nests it inside an outer button. Verify semantic markup and focus behavior before changing appearance or click propagation.
- Native Windows startup passes an argument disabling renderer accessibility. Verify the actual runtime/accessibility impact; the audit only establishes the configured argument.
- Existing focus traps, key layers, menu arrows, hover-pointer rules, reduced-motion behavior and focus restoration are valuable. Every lazy-mount/windowing suggestion must preserve them.

## Visual regression contract for later work

Compare the same data, viewport, OS material, zoom, hover/focus and open-panel state before and after any later implementation. Cover Home; each library tab; album/artist/playlist/show/profile; search categories and empty/error/loading states; Stats; Settings; normal and immersive player; queue/lyrics/friends/devices; all editing/confirmation/source/credit/auth/update modals.

Use long and multilingual titles, missing/broken artwork, duet/backing/translated lyrics, long transcripts, duplicate playlist entries, empty cached playlists, large queues, platform material fallback, reduced motion and keyboard-only operation. Backend-only changes should leave component/CSS/assets unchanged; compare DTO values/order and playback side effects as well as screenshots.
