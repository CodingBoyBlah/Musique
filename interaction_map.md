# UI interactions and their effects

The exact per-element event inventory is [interaction_inventory.md](interaction_inventory.md). Across all 194 TypeScript/TSX files it records 443 JSX event attributes, their element/prop context, source lines and handler expressions. The complete render network additionally records 876 component occurrences in [render_graph.md](render_graph.md). This document explains the shared flows; [native_ipc_map.md](native_ipc_map.md) links every registered native command to its frontend invoke sites.

## Persistent shell controls

| Control | Immediate effect | Work it triggers | Behavior to preserve |
| --- | --- | --- | --- |
| Top search input | Updates typed text, preserves previous results | Debounced dropdown query or replacement URL query | Input stays responsive, previous list remains visible |
| Ctrl/Cmd K | Focuses and selects search input | Existing query can display cached results | Cached dropdown opens without entrance animation |
| Search arrows / Enter / Escape | Moves selected option, selects it, opens full results, or blurs | Catalog selection navigation or track/episode playback | Listbox/combobox accessibility and selection geometry |
| Clear search | Clears local text and refocuses | Full-page URL currently retains old nonempty q | Optional behavior inconsistency, recorded separately |
| Play/pause | Changes optimistic play state | Local serialized transport, remote API or Jam host/guest flow | Same button, instant state, proper rollback |
| Next / previous | Advances queue or restarts at >3 seconds | Debounced local load, remote skip, or Jam command | All owner modes, history and skip signals |
| Shuffle / repeat | Mutates local ordering or remote/Jam options | Queue persistence or owner command | Preserve current local semantics and remote mappings |
| Seek pointer / key | Updates scrub position immediately, commit on release | Native/remote/Jam seek | Pointer capture/cancel, five-second keys and duration clamps |
| Volume drag / wheel / key | Updates local volume immediately | 80ms debounced volume IPC or remote call | Popup geometry, wheel increments, mute/unmute and keyboard behavior |
| Cover in player | Opens immersive overlay | Mounts stage content | Existing transition and same player state |
| Album / artist links | Navigates to detail | Hover query prefetch, detail fetch | Existing route and close-immersive behavior |
| Devices trigger / device row | Opens picker or transfers | Device/state refresh, pause local or resume at current position | Current position and optimistic owner transition |
| Immersive tabs | Selects lyrics/transcript, queue or credits/about | Conditional tab content queries | Same visual tabs and transition |
| Lyric row / transcript sentence | Seeks to timestamp | Native/remote/Jam seek plus clock resync | Correct words, voices, offset and user scroll intent |
| Return-to-current-line pill | Reattaches follow mode | Scroll spring/native transcript scroll | User-controlled scrolling stays interruptible |
| Ctrl/Cmd +/-/0 and wheel | Changes persisted WebView zoom | Native setZoom, responsive layout | OS chrome compensation and same step values |
| Ctrl/Cmd Shift M | Toggles simulated macOS shell | Layout state | Existing development control |
| Modal/menu Escape/Tab/arrows | Closes/traps/navigates top key layer | Focus restoration and handler action | Nested-layer ownership and accessible keyboard flows |
| OS/tray play/pause/skip | Uses shared transports | Same owner-dependent playback logic | Avoid separate queues or tickers |

## Queue and Jam actions

Normal enqueue selects local queue, remote Spotify queue, or Jam queue based on owner. Local play-context resets history and keeps context order. Jam hosts load up to 100 track IDs into Spirc; guests send social commands, with queue-only restrictions. Guest pause holds local listening without pausing the host. Leaving as a guest pauses local playback. Metadata enrichment batches missing IDs and keeps a bounded Jam map.

Delayed preload work must belong to a current playback generation. Queue rollover's `peek` and `advance` choices must agree before a preload can be called the next track.

## Route coverage

The routes are Home, settings, search, library, playlists, playlist/:id, artist/:id, album/:id, profile, user/:id, show/:id, stats and playground. Unknown routes redirect to Home. Every original route and shared control was reviewed.

| Route / source owner | Controls and rendered content | Data/action network | Preservation and recorded edge cases |
| --- | --- | --- | --- |
| Home | Quick-action tiles, personalized shelves, recent/recommendation tracks, range controls, refresh, top lists, releases | Parallel query families; shelf navigation; direct album/artist/playlist loads; local playContext or owner-aware transport; recommendation refresh | Same shelf order, complete rows, art, hover play and placeholders. Direct play can bypass a useful prefetched query; old selection requests need intent ownership. |
| Search | Query results, category tabs, top result play, rows/cards, artist/album/show links | One debounced URL query; search invoke; category switches reuse response; top result fetches detail before playback | Keep previous results and input responsiveness. Podcast-only matches are excluded from hasResults; this is a separate correctness finding. |
| Library | Songs/albums/artists/podcasts tabs, local filtering/sorting, row actions, save/follow controls, sync | Local memberships and saved-show queries; count/query invalidation; shared row/card actions | Same ordering and filters. Songs currently fetch first 200 without a next-page control; saved-album artist DTOs can be empty despite stored associations. |
| Playlists | Folder navigation/breadcrumbs, personalized/library grids, hover prefetch, right-click menus | Rootlist folder tree + playlist catalogue; card link to detail; pin/share/open menu actions | Preserve Spotify nesting/order and collapsed folders. Catalogue cache currently conflates browsed and followed playlists. |
| Playlist detail | Play/pause/shuffle, follow/save, pin/share, edit/delete/remove menu where allowed, filter, sort/order, rows | Cached+fresh detail reads, context playback, mutations and membership invalidation | Occurrence order, permissions and empty complete playlists matter. Current cache schema drops repeated occurrences and can mislabel stale rows as current. |
| Album detail | Play/pause/shuffle, save, pin/share, album-order/filter controls, artist link/avatar, more-by shelf | Album detail + artist detail; save state; row playContext; query prefetch | Preserve disc/track order, current-row indicator and artwork. Avatar currently pays full artist-detail cost; embedded first-page album tracks may be incomplete. |
| Artist detail | Play/shuffle/top tracks, follow, pin/share/menu, album groups, related artists, bio/photos/gallery, tour/merch links | Main artist + overview/extras/follow queries; catalogue/context playback; external opener | Preserve supplemental content even if another section fails. Main artist requests are serial natively; overview/detail work can overlap unnecessarily. |
| Own profile / user profile | Overview/follower/following tabs, follow/unfollow, share, public playlist cards, own top tracks/artists, retry | Internal profile / Web API fallback; follower/following commands; bounded row playback | Only own profile displays personal top data, although current hooks also run on another user's page. Follow invalidation is broader than needed. |
| Show detail | Play/start from episode, follow/unfollow/share, local episode filter/clear, sort, per-episode play, load more, retry | Show and saved status; additional episode pages; usePlayEpisodes and episode source resolution | Resume/fully-played/release-date/region semantics. A late show A page can append to B; same-show pagination should survive refetch coherently. |
| Stats | Week/month/year/all range, refresh/retry, ranked-track play, heatmap hover | One native aggregate DTO per range; local bounded render; playContext for ranked tracks | Same 168 heat cells and bounded lists. Raw history retention limits lifetime labels; metadata misses can trigger HTTP despite offline wording. |
| Settings | Appearance, playback, general/features, window material, credentials, Last.fm, account and updater controls | Persistent stores + settings/native capability/auth/backend commands | Preserve values, bounds, disabled states, credential drafts and platform-specific material fallback. Every setting is mapped below. |
| Playground | Theme/swatch changes, local sample play/pause/icons, segmented sliders/dials/cards/track examples | Local demonstration state and shared UI primitives; no streamed sample audio | Lazy route, not a production playback benchmark. Shared animated controls also have production imports. |

## Shared control and menu contracts

| Owner | Every action family | Backend/state effect |
| --- | --- | --- |
| Sidebar | Home/playlists/stats/library links, pins/playlist entries, folder expansion, collapse, pin/share menus | Navigation; pins/preferences; playlist/rootlist queries; shared link resolution. |
| TitleBar | Account/avatar menu, back/forward, sidebar toggle, lyric/queue/friend/device toggles, minimize/maximize/close, draggable caption | Browser history, player/UI stores, native window/plugin calls, close interception. |
| TopSearch | Type, focus/blur, clear, arrows/Home/End where handled, Enter/select, Escape, Ctrl/Cmd+K | Query/URL coordinator; select result navigation or track/episode transport. Exact handler expressions are in the event inventory. |
| TrackRow | Play/current pause, like/unlike, enqueue, album/artist link, credits, add-to-playlist, remove when allowed, source match, radio/share/context menu | Owner-aware queue/transport; library mutations; credits/add/source modal stores; radio/autoplay and link commands. Comparator currently ignores some changed metadata/callbacks. |
| AlbumCard / ArtistCard / MediaTile | Navigate, hover prefetch, hover play/pause, contextual pin/follow/share | Detail cache/query, context playback and pin/library state. Keep stopped-propagation play clicks distinct from navigation. |
| PlayActions / SaveAlbumButton | Main play/pause, shuffle start, follow/save, pin, share | Existing shared context and transport owner logic; relevant membership mutation. Condensed controls need accessible-name review. |
| EpisodeRow | Play/resume, show navigation, enqueue/context/share, progress presentation | Episode queue/transport and playable URI; preserve episode-specific metadata/date/progress. |
| QueuePanel | Local drag reorder, row jump/remove, history click, clear where offered, remote/Jam queue rows | Local arrays/persistence versus native remote/Spirc/social commands. Preserve occurrence identity and owner selection. |
| JamCard / FriendsPanel | Start/join/share/leave/end, invite input, queue-only choice, kick member, play friend track, profile link, refresh/error paths | Social session commands, Connect refresh/adoption, source/role restrictions. Guest pause does not pause the host. |
| DevicesPopover | Refresh, close, choose active output/transfer | Shared device/state refresh; native transfer; local pause/resume and position handoff. |
| Immersive / PodcastStage | Enter/exit, lyric/queue/credits tabs or transcript/about, artwork/video, notes expansion, transcript row seek | Shared player state, conditional cached data queries, episode media/extras. Current immersive queue is local-only. |
| LyricsPanel / LyricReturnPill | Source picker, refresh, offset/pronunciation options, row seek, follow-return | Native cache-only source switch, lyric fetch/upgrade, preference clock offset, shared seek and interruptible scroll. Preserve words/voices/translations/timings. |
| SpeedControl / player skip | Playback-speed choices and episode skip buttons | Native speed/seek; music normal speed; remembered podcast setting. |
| AddToPlaylistModal | Select existing playlist, typed filtering, Enter exact match/create, cancel | Add/create playlist mutation, invalidate relevant queries; preserve captured track/account intent. |
| EditPlaylistModal | Name/description/visibility/collaborative draft, save, cancel | Playlist mutation and detail/list invalidation; refetch currently can overwrite an open draft. |
| YtMatchModal | Search candidates, preview/selection where offered, pin chosen source, forget override, close | Native candidate/match/pin/forget commands; automatic matching cannot overwrite manual pin. |
| CreditsModal / CreditsList | Open/close, performer/artist navigation | Shared cached credits query, source labels and artist routing. |
| ArtistAbout / ArtistTour | Expand text, photo/gallery selection/close, related links, tour/merch external link | Supplemental cached metadata; native opener; preserve modal focus and image fit. |
| SortToolbar / TrackToolbar | Filter/clear, sort mode/direction, condensed header/selection state | Derived local stable ordering and occurrence keys; no extra remote search for local filters. |
| Dropdown / ContextMenu | Open/select, outside close, Escape, arrows/Home/End/Enter, focus restoration | Controlled selected value/action and top key-layer ownership; listeners exist only while open. |
| Modal / ConfirmDialog / QuitConfirm | Confirm/cancel, permitted backdrop/Escape close, focus trap/restore | Captured action; close/quit native plugin after confirmation when required. |
| Tooltip | Delayed hover/focus show, warm repeat show, hide/unmount | Portal/placement; existing timer cleanup; 140-ms initial and 400-ms warm behavior. |
| Carousel / Shelf / EvenGrid | Horizontal scroll, previous/next/fades; measured complete visible-row layout | Geometry observers and scroll state. Async mounting can currently miss Carousel listener registration. |
| OverlayScrollbar | Thumb drag and page-step | Existing scroll owner, rAF geometry updates, mutation/resize listeners. |
| Toast / error / empty / skeleton / countdown | Dismiss/error reload where provided; loading and relative-time presentation | Bounded renderers, global toast state, countdown clocks. Do not alter feedback copy or placeholder dimensions incidentally. |
| SignInPrompt / account menu | Prepare/open both authorization links, retry/help/cancel/signout/profile navigation | Auth generation and native browser/callback flow; retain the existing user-edited onboarding logic. |
| UpdatePrompt | Available update details, dismiss when permitted, download/progress/error/retry, restart | Official updater/process plugins. No update/download/restart was performed in the audit. |

Passive logo/icon/header/section/skeleton components have no independent application action. Their rendering, motion props and stylesheet edges are included in the render/style inventories rather than invented as backend controls.

## Every Settings action

| Setting/control | Current owner and triggered work | Contract |
| --- | --- | --- |
| Zoom step/reset | lib/zoom, persisted WebView setZoom and keyboard shortcuts | 67%–200%, exact steps/reset and native-caption compensation. |
| Sidebar mode | prefs store, Sidebar derived tree/list | Pins versus every library playlist; preserve pin enrichment and nesting. |
| Accent source | theme store, ThemeEngine, wallpaper/system-accent native probe | Same default, wallpaper and platform system colors. |
| Album colors | prefs/theme-derived detail backgrounds | Keep card/page cover selection and fallback contrast. |
| Animated background | prefs and immersive motion owner | Still/drifting choice; foreground visual motion unchanged. |
| Animated artwork | prefs, Canvas/media query and video owner | Same video quality, fit and visible-play behavior. |
| Audio source | usePlaybackBackend, set_playback_backend and session/prepared teardown | Preserve Free/Premium/unknown-plan semantics, playback transition and truthful source indicator. |
| Audio quality | Native get/set_audio_quality | Applied next track; no quality downgrade proposed. |
| Disk cache limit/reset | Native audio-cache setting/rebuild | 512–8192 MiB in 256-MiB steps; default 2048 MiB. |
| Playback notification | prefs + notification permission + native sized-art notification | Existing enabled state and one accepted-track notification; correct artwork generation. |
| Confirm before closing | prefs + native close interception/QuitConfirm | Same while-playing confirmation behavior. |
| Discord Rich Presence | prefs + media/Discord worker enabled state | Same fields/timestamps and prompt clear on stop. |
| Friend activity | prefs, rail state and conditional panel queries | Disabling closes the panel; no unwanted background friend polling. |
| Listening stats | prefs, sidebar/route gate | Disabled feature does not query or render Stats. |
| Background material | native window effect and backdrop query | Windows Mica/Acrylic/None; macOS vibrancy/solid; Linux unavailable fallback. |
| Transparency/reset | theme preference/native material | Current 10%–70% limits and default; preserve readable fallback. |
| Custom Spotify credentials | Input draft, save/validate/clear, native keyring + account/grant revision | Secrets stay in native storage; preserve locally valid PKCE versus secret validation distinction. Coalesce duplicate validation. |
| Spotify account/connect/signout | Native auth commands and auth-session coordinator | Both required grants, matching account, unknown product, cancellation and cleanup order. |
| Last.fm API/auth/disconnect/clear | Native keyring/status/browser polling/scrobble session | Separate connection survives Spotify signout; late auth must not resurrect disconnect. |
| About/update/support/external links | Updater state and native opener | Same targets/copy and user-triggered install/restart. |

## Global and local input layers

Native tray/OS media events are media:play, media:pause, media:toggle, media:next, media:prev and media:stop. AppInit sends them through the same transport and ownership rules as visible controls. Native window close/show/hide/quit and plugin calls are separate window lifecycle actions.

Ctrl/Cmd+K belongs to search. Ctrl/Cmd+plus/minus/zero (including numpad variants) and modified wheel belong to zoom. Ctrl/Cmd+Shift+M toggles the simulated platform shell. Scrubber arrows move five seconds, Page keys move 10% of duration, Home starts, End targets one second before the end. Volume arrows use the existing increment, Page keys move ten points, Home/End set the endpoints and M toggles mute while focused. These are focused-control handlers, not an invented global media shortcut set.

Modal/menu key layers prevent an underlying handler from consuming the same Escape/arrow event. Pointer capture/cancel, drag thresholds, touch/hover gating, tooltip warm timing and focus restoration are part of behavior preservation. Existing source contains every actual event expression in interaction_inventory.md; dynamic context-menu actions are traced through the shared owner table above.
