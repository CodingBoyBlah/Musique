# Frontend source inspection index

Structural inspection of every complete TypeScript file. Entries include imports, declarations, state/query/store/listener/timer work and data iteration. Manual review status remains in review_progress.md.

## src/App.tsx

Declarations: AppInit:72, App:437.

```text
73 useCredentialsStore((s) => s.setFromCredentials)
74 useCredentialsStore((s) => s.setStatus)
75 useAuth()
76 useAuthStore((s) => s.loggedIn)
77 usePlayerStore((s) => s.onEvent)
79 useInvalidateLibrary()
82 useJamSync(isLoggedIn)
86 usePlayerStore((s) => s.currentId)
87 usePrefsStore((s) => s.podcastSpeed)
88 useEffect(() => { const rate = isEpisodeId(currentIdForSpeed) ? podcastSpeed : 1; usePlayerStore.getState().setPlaybackRate(rate); setPlaybackSpeed(rate).catch(() => {}); }, [currentIdForSpeed, podcastSpeed])
90 usePlayerStore.getState().setPlaybackRate(rate)
90 usePlayerStore.getState()
95 useEffect(() => { let off: (() => void) | null = null; let gone = false; listen("library:playlist-changed", () => { queryClient.invalidateQueries({ queryKey: ["library", "rootlist"] }); queryClient.invalidateQueries({ queryKey: ["playlist"] }); }) .then((u) => (gone ? u() : (off = u))) .catch(() => {}); return () => { gone = true; off?.(); }; // eslint-disable-next-line react-hooks/exhaustive-deps }, [])
98 listen("library:playlist-changed", () => { queryClient.invalidateQueries({ queryKey: ["library", "rootlist"] }); queryClient.invalidateQueries({ queryKey: ["playlist"] }); }) .then((u) => (gone ? u() : (off = u))) .catch(() => {})
98 listen("library:playlist-changed", () => { queryClient.invalidateQueries({ queryKey: ["library", "rootlist"] }); queryClient.invalidateQueries({ queryKey: ["playlist"] }); }) .then((u) => (gone ? u() : (off = u)))
98 listen("library:playlist-changed", () => { queryClient.invalidateQueries({ queryKey: ["library", "rootlist"] }); queryClient.invalidateQueries({ queryKey: ["playlist"] }); })
99 queryClient.invalidateQueries({ queryKey: ["library", "rootlist"] })
100 queryClient.invalidateQueries({ queryKey: ["playlist"] })
110 useQueryClient()
111 usePlayerStore((s) => s.currentTrack)
112 useQueueStore((s) => s.queue)
113 useRef<string | null>(null)
119 useRef<Set<string>>(new Set())
121 useRef<{ artist: string; track: string; album: string; startedAt: number; durationMs: number; scrobbled: boolean; } | null>(null)
129 useRef<ReturnType<typeof setTimeout> | null>(null)
131 useEffect(() => { return () => { if (preloadTimerRef.current) clearTimeout(preloadTimerRef.current); }; }, [])
133 clearTimeout(preloadTimerRef.current)
138 useEffect(() => { if (usePrefsStore.getState().notifyOnTrack) { requestNotificationPermission().catch(() => {}); } }, [])
139 usePrefsStore.getState()
145 useEffect(() => { setDiscordEnabled(usePrefsStore.getState().discordPresence); }, [])
146 usePrefsStore.getState()
152 useEffect(() => { const effect = useUIStore.getState().windowEffect; setWindowEffect(effect).catch(() => {}); }, [])
153 useUIStore.getState()
161 useEffect(() => { const block = (e: MouseEvent) => e.preventDefault(); document.addEventListener("contextmenu", block); return () => document.removeEventListener("contextmenu", block); }, [])
168 useEffect(() => { /* load the creds, and then auto validate (helps unfetched data) and then cache */ getCredentials() .then((c) => { setFromCredentials(c); if (c && c.is_custom && c.has_secret) { setCredStatus("validating"); validateCredentials() .then((r) => setCredStatus(r.valid ? "valid" : "invalid")) .catch(() => setCredStatus("invalid")); } else { setCredStatus("valid"); } }) .catch(() => {}); const { volume, muted } = usePlayerStore.getState(); setVolume(volume).catch(() => {}); setMuted(muted).catch(() => {}); }, [setFromCredentials, setCredStatus])
183 usePlayerStore.getState()
193 useEffect(() => { if (isLoggedIn) { warmupPlayback().catch(() => {}); queryClient .prefetchQuery({ queryKey: ["recommendations", "home"], queryFn: () => getRecommendations(undefined, 16), staleTime: 10 * 60_000, }) .catch(() => {}); } }, [isLoggedIn, queryClient])
209 useEffect(() => { if (currentTrack) prefetchLyrics(queryClient, currentTrack); upcoming.slice(0, 3).forEach((t) => prefetchLyrics(queryClient, t)); }, [currentTrack, upcoming, queryClient])
214 useEffect(() => { const win = getCurrentWindow(); const unlisten = win.onCloseRequested((e) => { const { promptOnClose } = usePrefsStore.getState(); const { isPlaying } = usePlayerStore.getState(); if (promptOnClose && isPlaying) { e.preventDefault(); useUIStore.getState().setQuitConfirmOpen(true); } }); return () => { unlisten.then((u) => u()).catch(() => {}); }; }, [])
217 usePrefsStore.getState()
218 usePlayerStore.getState()
221 useUIStore.getState().setQuitConfirmOpen(true)
221 useUIStore.getState()
225 unlisten.then((u) => u()).catch(() => {})
225 unlisten.then((u) => u())
229 useCallback(() => { const s = scrobbleRef.current; if (!s || s.scrobbled || s.durationMs < 30_000) return; const elapsed = Math.floor(Date.now() / 1000) - s.startedAt; const threshold = Math.min(s.durationMs / 2, 240_000) / 1000; if (elapsed >= threshold) { lastfmScrobble(s.artist, s.track, s.album, s.startedAt); s.scrobbled = true; } }, [])
241 useCallback( (payload: unknown) => { if (!useAuthStore.getState().loggedIn) return; onEvent(payload); const isRemote = usePlayerStore.getState().isRemotePlayback; const msg = payload as { type: string; track_id?: string | null }; if (isRemote && msg.type !== "playing") { return; } /* in a jam spotify owns the queue: spirc advances, skips unplayable songs and preloads by itself, and the app's queue must not step in (it used to, and played its own next song over the jam's) */ const inJam = !!jamRole(); if (msg.type === "playing") { const { currentTrack } = usePlayerStore.getState(); // it played so let it be autoretried again if it ever dies later if (msg.track_id) retriedUnavailable.current.delete(msg.track_id); if (currentTrack && !inJam) { replenishQueue(currentTrack).catch(() => {}); } /
243 useAuthStore.getState()
245 usePlayerStore.getState()
258 usePlayerStore.getState()
275 useQueueStore.getState()
276 useSpeedDialStore.getState().recordTrack(currentTrack, activeContextId)
276 useSpeedDialStore.getState()
279 usePrefsStore.getState()
283 currentTrack.artists.map((a) => a.name)
298 clearTimeout(preloadTimerRef.current)
300 useQueueStore.getState().peek(currentTrack)
300 useQueueStore.getState()
302 setTimeout(() => { preloadTimerRef.current = null; const playing = usePlayerStore.getState().currentTrack; if (!jamRole() && playing?.id === currentTrack.id && useQueueStore.getState().peek(playing)?.id === next.id) { preloadTrack(next.id).catch(() => {}); } }, 1200)
304 usePlayerStore.getState()
305 useQueueStore.getState().peek(playing)
305 useQueueStore.getState()
325 usePlayerStore.getState()
335 useQueueStore.getState().advance(currentTrack)
335 useQueueStore.getState()
337 usePlayerStore.getState().setCurrentTrack(next)
337 usePlayerStore.getState()
338 usePlayerStore.getState().setPlaying(true)
338 usePlayerStore.getState()
349 usePlayerStore.getState()
350 useQueueStore.getState().peek(currentTrack)
350 useQueueStore.getState()
361 usePlayerStore.getState()
364 useQueueStore.getState().advance(currentTrack)
364 useQueueStore.getState()
367 usePlayerStore.getState().setPlaying(true)
367 usePlayerStore.getState()
381 useEffect(() => { let unlisten: (() => void) | null = null; let gone = false; listen("player:event", (e) => handlePlayerEvent(e.payload)) .then((u) => { if (gone) u(); else unlisten = u; }) .catch(() => {}); return () => { gone = true; unlisten?.(); }; }, [handlePlayerEvent])
384 listen("player:event", (e) => handlePlayerEvent(e.payload)) .then((u) => { if (gone) u(); else unlisten = u; }) .catch(() => {})
384 listen("player:event", (e) => handlePlayerEvent(e.payload)) .then((u) => { if (gone) u(); else unlisten = u; })
384 listen("player:event", (e) => handlePlayerEvent(e.payload))
392 unlisten?.()
399 usePlayerStore((s) => s.currentTrack?.id ?? null)
400 useEffect(() => { if (!jamRole() || !nowPlayingId || nowPlayingId === lastTrackId.current) return; if (!usePlayerStore.getState().isPlaying) return; handlePlayerEvent({ type: "playing", track_id: nowPlayingId }); }, [nowPlayingId, handlePlayerEvent])
402 usePlayerStore.getState()
407 useEffect(() => { let unlisten: (() => void) | null = null; let gone = false; listen("library:synced", () => invalidateLibrary()) .then((u) => { if (gone) u(); else unlisten = u; }) .catch(() => {}); return () => { gone = true; unlisten?.(); }; }, [invalidateLibrary])
410 listen("library:synced", () => invalidateLibrary()) .then((u) => { if (gone) u(); else unlisten = u; }) .catch(() => {})
410 listen("library:synced", () => invalidateLibrary()) .then((u) => { if (gone) u(); else unlisten = u; })
410 listen("library:synced", () => invalidateLibrary())
418 unlisten?.()
423 useEffect(() => { return cleanupAsyncListeners([ listen("media:play", () => transportPlay()), listen("media:pause", () => transportPause()), listen("media:stop", () => transportPause()), listen("media:toggle", () => transportTogglePlay()), listen("media:next", () => transportNext()), listen("media:prev", () => transportPrev()), ]); }, [])
425 listen("media:play", () => transportPlay())
426 listen("media:pause", () => transportPause())
427 listen("media:stop", () => transportPause())
428 listen("media:toggle", () => transportTogglePlay())
429 listen("media:next", () => transportNext())
430 listen("media:prev", () => transportPrev())
```

## src/api/auth.ts

Declarations: LoginLinks:5.

```text
10 invoke("start_login", { onLinks: channel, openBrowser })
12 invoke("logout")
13 invoke("get_auth_status")
14 invoke("get_profile")
```

## src/api/connect.ts

Declarations: SpotifyDevice:4, DevicesPayload:15, RemotePlaybackState:20, RemoteQueue:71, RemoteRepeat:83.

```text
31 invoke("get_devices")
34 invoke("get_playback_state")
37 invoke("transfer_playback", { deviceId, play })
44 invoke("remote_play_track", { deviceId, trackId: trackId ?? undefined, positionMs: positionMs != null ? Math.round(positionMs) : undefined, })
51 invoke("remote_play", { deviceId })
54 invoke("remote_pause", { deviceId })
57 invoke("remote_next", { deviceId })
60 invoke("remote_previous", { deviceId })
63 invoke("remote_seek", { positionMs: Math.round(positionMs) })
66 invoke("remote_set_volume", { volumePercent: Math.round(volumePercent) })
69 invoke("get_musique_device_id")
78 invoke("get_remote_queue")
81 invoke("remote_set_shuffle", { state })
86 invoke("remote_set_repeat", { state })
90 invoke("remote_add_to_queue", { id })
99 invoke("remote_play_context", { deviceId: opts.deviceId ?? null, contextUri: opts.contextUri, offsetUri: opts.offsetUri ?? null, offsetPosition: opts.offsetPosition ?? null, positionMs: opts.positionMs != null ? Math.round(opts.positionMs) : null, })
```

## src/api/credentials.ts

Declarations: Credentials:3, ValidationResult:9.

```text
18 invoke("save_credentials", { clientId, clientSecret: clientSecret ?? null, })
24 invoke("get_credentials")
27 invoke("validate_credentials")
30 invoke("clear_credentials")
```

## src/api/internal.ts

Declarations: Canvas:12, CreditPerson:23, TrackCredits:30, RootItem:39, RadioResult:46, ArtistExtras:65, ExtractedColor:78, HomeItem:88, HomeFeed:96, ArtistOverview:105.

```text
10 invoke("get_tracks_metadata", { ids })
21 invoke("get_canvas", { trackId })
37 invoke("get_track_credits", { trackId })
44 invoke("get_playlist_folders")
55 invoke("get_track_radio", { trackId })
59 invoke("get_station", { seed })
63 invoke("get_autoplay_tracks", { contextUri, recentTrackIds })
76 invoke("get_artist_extras", { id })
86 invoke("get_extracted_colors", { imageUrls })
103 invoke("get_home_feed", { timeZone })
122 invoke("get_artist_overview", { id })
```

## src/api/lastfm.ts

Declarations: LastfmStatus:3, lastfmNowPlaying:29, lastfmScrobble:33.

```text
10 invoke("lastfm_status")
13 invoke("lastfm_save_api", { apiKey, apiSecret })
17 invoke("lastfm_start_auth")
21 invoke("lastfm_finish_auth", { token })
24 invoke("lastfm_disconnect")
27 invoke("lastfm_clear")
30 invoke("lastfm_now_playing", { artist, track, album }).catch(() => {})
30 invoke("lastfm_now_playing", { artist, track, album })
34 invoke("lastfm_scrobble", { artist, track, album, timestamp }).catch(() => {})
34 invoke("lastfm_scrobble", { artist, track, album, timestamp })
```

## src/api/library.ts

Declarations: module constants/functions.

```text
6 invoke("sync_library")
9 invoke("get_liked_songs", { limit, offset })
12 invoke("get_liked_songs_count")
15 invoke("get_my_playlists")
18 invoke("get_saved_albums")
21 invoke("get_followed_artists")
24 invoke("get_library_status")
27 invoke("get_top_tracks", { timeRange })
30 invoke("get_top_artists", { timeRange })
33 invoke("get_recently_played")
36 invoke("get_new_releases")
39 invoke("save_track", { id })
42 invoke("unsave_track", { id })
45 invoke("get_saved_track_ids", { ids })
48 invoke("follow_artist", { id })
51 invoke("unfollow_artist", { id })
54 invoke("save_album", { id })
57 invoke("unsave_album", { id })
60 invoke("is_album_saved", { id })
63 invoke("is_artist_followed", { id })
67 invoke("add_track_to_playlist", { playlistId, trackId, position: position ?? null })
70 invoke("remove_track_from_playlist", { playlistId, trackId })
73 invoke("create_playlist", { name, description, public: isPublic })
79 invoke("update_playlist_details", { id, name: changes.name ?? null, description: changes.description ?? null, public: changes.public ?? null, collaborative: changes.collaborative ?? null, })
88 invoke("follow_playlist", { id })
92 invoke("unfollow_playlist", { id })
95 invoke("is_playlist_followed", { id })
```

## src/api/lyrics.test.ts

Declarations: module constants/functions.

```text
31 Promise.all([ getLyrics(track), getLyrics(track, true), getLyrics({ ...track, external_ids: { isrc: "NEW-ISRC" } }), ])
```

## src/api/lyrics.ts

Declarations: LineRole:7, LyricWord:9, LyricLine:15, Alternate:29, Lyrics:36, getLyrics:60.

```text
77 invoke<Lyrics>("get_lyrics", args).finally(() => { if (!force) inflightLyrics.delete(key); })
77 invoke<Lyrics>("get_lyrics", args)
```

## src/api/media.ts

Declarations: updateNowPlaying:4, setDiscordEnabled:20, requestNotificationPermission:29, showTrackNotification:43.

```text
5 invoke("update_now_playing", { title: track.name, artist: track.artists.map((a) => a.name).join(", "), album: track.album?.name ?? "", coverUrl: track.album?.image_url ?? null, trackUrl: track.id ? track.id.startsWith("spotify:episode:") ? `https://open.spotify.com/episode/${track.id.slice("spotify:episode:".length)}` : `https://open.spotify.com/track/${track.id}` : null, durationMs: track.duration_ms, positionMs, }).catch(() => {})
5 invoke("update_now_playing", { title: track.name, artist: track.artists.map((a) => a.name).join(", "), album: track.album?.name ?? "", coverUrl: track.album?.image_url ?? null, trackUrl: track.id ? track.id.startsWith("spotify:episode:") ? `https://open.spotify.com/episode/${track.id.slice("spotify:episode:".length)}` : `https://open.spotify.com/track/${track.id}` : null, durationMs: track.duration_ms, positionMs, })
7 track.artists.map((a) => a.name)
21 invoke("set_discord_enabled", { enabled }).catch(() => {})
21 invoke("set_discord_enabled", { enabled })
45 track.artists.map((a) => a.name)
58 invoke("show_playback_notification", { title, body, icon, })
```

## src/api/playback.ts

Declarations: VolumeState:9, ConnectQueueEntry:64, ConnectState:72, AudioQuality:126, PlaybackBackend:143, BackendState:145, YtMatch:159, YtCandidate:170.

```text
28 usePlayerStore.getState()
42 useQueueStore.getState()
43 contextTracks.some((t) => t.id === id)
46 invoke("warmup_playback")
50 usePlayerStore.getState()
55 invoke<void>("play_track", { id, contextUri: contextUriFor(id) }).catch(failPlayback)
55 invoke<void>("play_track", { id, contextUri: contextUriFor(id) })
58 invoke("retry_play_track", { id, contextUri: contextUriFor(id) })
88 invoke("get_connect_state")
90 invoke("connect_load_tracks", { ids, index, positionMs: Math.max(0, Math.floor(positionMs)), startPlaying, keepStream })
91 invoke("connect_add_to_queue", { id })
92 invoke("connect_skip_to", { id })
93 invoke("connect_next")
94 invoke("connect_prev")
95 invoke("connect_set_shuffle", { on })
96 invoke("connect_set_repeat", { context, track })
98 invoke("jam_hold", { hold })
99 invoke("pause_playback")
100 invoke("resume_playback")
104 usePlayerStore.getState()
106 Promise.resolve()
110 usePlayerStore.getState()
117 invoke<void>("resume_or_play", { id, positionMs: safePos, contextUri: contextUriFor(id) }).catch(failPlayback)
117 invoke<void>("resume_or_play", { id, positionMs: safePos, contextUri: contextUriFor(id) })
119 invoke("stop_playback")
120 invoke("seek_playback", { positionMs })
121 invoke("preload_track", { id })
122 invoke("set_volume", { level })
123 invoke("set_muted", { muted })
124 invoke("get_volume")
129 invoke("get_audio_quality")
132 invoke("set_audio_quality", { quality })
153 invoke("get_playback_backend")
156 invoke("set_playback_backend", { mode })
180 invoke("get_yt_match", { trackId })
187 invoke("search_yt_candidates", { trackId })
191 invoke("pin_yt_match", { trackId, videoId })
195 invoke("forget_yt_match", { trackId })
198 invoke("get_audio_cache_limit")
201 invoke("set_audio_cache_limit", { limitMb })
205 invoke("set_playback_speed", { speed })
```

## src/api/podcasts.ts

Declarations: module constants/functions.

```text
6 invoke("get_show", { id })
9 invoke("get_show_episodes", { id, offset })
11 invoke("get_episode", { id })
13 invoke("get_saved_shows")
15 invoke("save_show", { id })
17 invoke("unsave_show", { id })
19 invoke("is_show_saved", { id })
23 invoke("get_episode_transcript", { id })
26 invoke("get_episode_media", { id })
```

## src/api/profile.ts

Declarations: UserProfile:4, ProfileCard:17.

```text
27 invoke("get_user_profile", { id: id ?? null })
30 invoke("get_user_followers", { id: id ?? null })
33 invoke("get_user_following", { id: id ?? null })
36 invoke("set_user_followed", { id, follow })
39 invoke("is_user_followed", { id })
```

## src/api/settings.ts

Declarations: module constants/functions.

```text
4 invoke<string | null>("get_setting", { key })
7 invoke<void>("set_setting", { key, value })
10 invoke<Record<string, string>>("get_all_settings")
```

## src/api/share.ts

Declarations: module constants/functions.

```text
5 invoke<string>("resolve_odesli", { url })
```

## src/api/social.ts

Declarations: FriendActivity:3, JamMember:22, JamSession:31, JamUpdateReason:47, JamUpdate:53, JamCommand:70.

```text
20 invoke("get_friend_activity")
58 invoke("get_jam")
59 invoke("start_jam")
60 invoke("join_jam", { link })
62 invoke("leave_jam", { sessionId, isHost })
64 invoke("set_jam_queue_only", { enabled })
66 invoke("kick_jam_member", { sessionId, memberId })
78 invoke("jam_command", { sessionId, command })
```

## src/api/spotify.ts

Declarations: module constants/functions.

```text
13 invoke("search", { query, types })
15 invoke("get_artist", { id })
16 invoke("get_album", { id })
17 invoke("get_track", { id })
18 invoke("get_playlist", { id })
19 invoke("get_cached_playlist", { id })
26 invoke("get_recommendations", { seedArtistIds: seedArtistIds ?? null, excludeTrackIds: excludeTrackIds ?? null, limit: limit ?? null, })
42 invoke("record_listen_event", { trackId, eventType, msPlayed, durationMs, contextType: contextType ?? null, contextId: contextId ?? null, })
```

## src/api/stats.ts

Declarations: StatsRange:4, Ranked:6, ListeningStats:11.

```text
33 invoke("get_listening_stats", { range })
```

## src/api/theme.ts

Declarations: module constants/functions.

```text
4 invoke("get_wallpaper_data_url")
8 invoke("get_system_accent")
```

## src/api/window.ts

Declarations: WindowEffect:3.

```text
7 invoke("set_window_effect", { mode })
13 invoke("get_backdrop_active")
```

## src/components/ErrorBoundary.tsx

Declarations: Props:4, State:7, ErrorBoundary:15.

```text
```

## src/components/ThemeEngine.tsx

Declarations: ThemeEngine:8.

```text
9 useThemeStore((s) => s.source)
10 useThemeStore((s) => s.albumColors)
11 useThemeStore((s) => s.refreshKey)
12 useUIStore((s) => s.pageTint)
16 useState<string | null>(null)
17 useEffect(() => { let cancelled = false; const resolve = async () => { let hex: string | null = null; if (source === "wallpaper") { const data = await getWallpaperDataUrl().catch(() => null); if (data) { hex = await dataUrlAccent(data); if (!cancelled && hex) setBase(hex); } return; } else if (source === "system") { hex = await getSystemAccent().catch(() => null); } if (!cancelled) setBase(hex); }; resolve(); if (source === "wallpaper" || source === "system") { const refresh = () => { if (!document.hidden) resolve(); }; window.addEventListener("focus", refresh); document.addEventListener("visibilitychange", refresh); const iv = source === "wallpaper" ? window.setInterval(resolve, 20000) : 0; return () => { cancelled = true; window.removeEventListener("focus", refresh); document.removeEvent
40 window.setInterval(resolve, 20000)
45 window.clearInterval(iv)
52 useEffect(() => { let cancelled = false; (async () => { let effective = base; if (albumColors && pageTint) { const cover = await loadCoverAccent(pageTint); if (cover) effective = cover; } if (!cancelled) applyAccent(effective); })(); return () => { cancelled = true; }; }, [base, albumColors, pageTint])
```

## src/components/UpdatePrompt.tsx

Declarations: UpdatePrompt:12.

```text
13 useUpdaterStore((s) => s.stage)
14 useUpdaterStore((s) => s.open)
15 useUpdaterStore((s) => s.version)
16 useUpdaterStore((s) => s.notes)
17 useUpdaterStore((s) => s.progress)
18 useUpdaterStore((s) => s.error)
19 useUpdaterStore((s) => s.dismiss)
22 useEffect(() => { runUpdateCheck(); }, [])
```

## src/components/auth/ProfileCard.tsx

Declarations: Props:1, ProfileCard:8.

```text
```

## src/components/layout/DevicesPopover.tsx

Declarations: getDeviceIcon:18, PopoverPos:27, findTrigger:30, measurePos:40, DevicesPopover:56, DevicesPopoverCard:65, onPointerDown:102, onKeyDown:108.

```text
31 document.querySelectorAll<HTMLElement>("[data-devices-trigger]")
33 t.getBoundingClientRect()
43 trigger.getBoundingClientRect()
57 useDevices()
76 useRef<HTMLDivElement>(null)
77 useState(false)
81 useState<PopoverPos>(measurePos)
83 useRef<HTMLElement | null>( document.activeElement instanceof HTMLElement ? document.activeElement : null, )
87 useLayoutEffect(() => { const updatePos = () => setPos(measurePos()); updatePos(); window.addEventListener("resize", updatePos); return () => window.removeEventListener("resize", updatePos); }, [])
101 useEffect(() => { function onPointerDown(e: MouseEvent) { const isTrigger = Boolean((e.target as Element)?.closest?.("[data-devices-trigger]")); if (popoverRef.current && !popoverRef.current.contains(e.target as Node) && !isTrigger) { setDevicesOpen(false); } } function onKeyDown(e: KeyboardEvent) { if (e.key === "Escape") { e.stopPropagation(); setDevicesOpen(false); (returnFocus.current ?? findTrigger())?.focus?.(); } } document.addEventListener("pointerdown", onPointerDown); document.addEventListener("keydown", onKeyDown); return () => { document.removeEventListener("pointerdown", onPointerDown); document.removeEventListener("keydown", onKeyDown); }; }, [setDevicesOpen])
129 setTimeout(() => setIsRefreshing(false), 250)
245 devices.map((device: SpotifyDevice) => { const Icon = getDeviceIcon(device.type); const isMusiqueDevice = Boolean(musiqueDeviceId && device.id === musiqueDeviceId) || device.name.toLowerCase() === "musique"; const isActive = isRemotePlayback ? Boolean(activeDevice?.id ? activeDevice.id === device.id : device.is_active) : isMusiqueDevice; const isTransferring = transferringId === device.id; return ( <button key={device.id ?? device.name} type="button" className="dev-row" data-active={isActive || undefined} aria-current={isActive || undefined} aria-busy={isTransferring || undefined} onClick={() => { if (device.id && !isActive && !isTransferring) { transfer(device.id); } }} disabled={isTransferring || isActive} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", g
```

## src/components/layout/FriendsPanel.tsx

Declarations: ago:21, asTrack:31, contextOf:45, Bars:54, play:74, FriendsPanel:148.

```text
57 [0, 1, 2].map((i) => ( <span key={i} className="eq-bar-q" style={{ width: 2, height: "100%", borderRadius: 1, background: "var(--color-accent)", ["--eq-delay" as string]: `${i * 0.18}s` }} /> ))
76 useQueueStore.getState().playContext([track], 0, `friend-${f.user_id}`)
76 useQueueStore.getState()
78 usePlayerStore.getState().setCurrentTrack(start)
78 usePlayerStore.getState()
149 usePlayerStore((s) => s.toggleFriends)
150 useQuery({ queryKey: ["friend-activity"], queryFn: getFriendActivity, refetchInterval: 60_000, staleTime: 30_000, retry: false, })
158 data.filter((f) => now - f.timestamp < LIVE_MS)
159 data.filter((f) => now - f.timestamp >= LIVE_MS)
208 live.map((f) => <FriendRow key={f.user_id} f={f} now={now} live />)
214 earlier.map((f) => <FriendRow key={f.user_id} f={f} now={now} live={false} />)
```

## src/components/layout/Immersive.tsx

Declarations: LiveVideoRoom:62, AmbientBg:112, roomFill:186, Sleeve:203, ImmersiveLyrics:305, seekTo:350, secondaryInset:470, Centered:488, ImmersiveQueue:498, jump:504, Immersive:542, PanelTab:736.

```text
63 useRef<HTMLCanvasElement>(null)
64 useState(false)
65 useEffect(() => { const cv = ref.current; const ctx = cv?.getContext("2d"); if (!cv || !ctx) return; setShown(false); let dead = false; let drewOnce = false; let handle = 0; const W = cv.width; const H = cv.height; // drawn oversize so the blur has real pixels at the edges (no dark rim) const over = 0.4; const hasRvfc = "requestVideoFrameCallback" in video; const draw = () => { if (dead) return; if (video.readyState >= 2 && !document.hidden) { ctx.filter = "blur(4px) saturate(1.75)"; ctx.drawImage(video, (-W * over) / 2, (-H * over) / 2, W * (1 + over), H * (1 + over)); if (!drewOnce) { drewOnce = true; setShown(true); } } handle = hasRvfc ? (video as HTMLVideoElement & { requestVideoFrameCallback: (cb: () => void) => number }).requestVideoFrameCallback(draw) : requestAnimationFrame(draw);
82 ctx.drawImage(video, (-W * over) / 2, (-H * over) / 2, W * (1 + over), H * (1 + over))
90 requestAnimationFrame(draw)
96 cancelAnimationFrame(handle)
113 useAmbient(url)
115 useReducedMotion()
119 useWindowActive()
121 usePrefsStore((s) => s.ambientMotion)
123 usePlayerStore((s) => s.isPlaying)
135 useRef({ key: "", z: 0 })
212 useReducedMotion()
217 useState(true)
218 useEffect(() => setCors(true), [canvas?.url])
306 usePlayerStore((s) => s.currentTrack)
307 usePlayerStore((s) => s.setPosition)
308 useReducedMotion()
309 useLyrics(track)
310 usePlayerControls()
312 useMemo(() => buildRows(data), [data])
313 useMemo(() => rows.map((r) => r.startMs), [rows])
313 rows.map((r) => r.startMs)
316 useLyricClock()
317 useActiveRow(rowStarts, getClock, synced)
319 usePlayerStore((s) => s.currentTrack?.id)
320 useRef<HTMLDivElement>(null)
321 useRef<(HTMLDivElement | null)[]>([])
322 useMoreContrast()
336 useCallback((el: HTMLDivElement, cont: HTMLDivElement) => { const H = cont.clientHeight; const top = el.offsetTop - cont.offsetTop; return Math.min( top - H * 0.3 + el.clientHeight / 2, top - H * 0.04, ); }, [])
346 useLyricFollow({ scrollRef, rowRefs, active, resetKey: rows, trackKey: followTrackId, targetFor, reduceMotion, })
365 rows.map((row, ri) => { const isActive = synced && ri === active; const tone = synced ? lyricTone(Math.abs(ri - active), ri < active, moreContrast) : { blur: 0, alpha: 0.9 }; const multi = row.voices.length > 1; const layout = voiceLayout(row); const ease = "0.45s cubic-bezier(0.22, 1, 0.36, 1)"; return ( <div key={ri} ref={(el) => { rowRefs.current[ri] = el; }} onClick={() => seekTo(ri)} className="lyr-row" data-seekable={synced} data-active={isActive} style={{ cursor: synced ? "pointer" : "default", /* Constant box: the padding never changes. Growing the active row reflowed the whole list on every line and moved the scroll target out from under the tween chasing it, so size is carried by transform, which does not relayout. The list is laid out at the ACTIVE size and everything else is sc
405 row.voices.map((voice, vi) => { const isDuet = voice.role === "duet"; // lead left, duet right, a backing vocal on the side of the // singer it's under (lib/lyricsRows voiceLayout) const { side: align, secondary: isSecondary } = layout[vi]; /* Each voice shrinks toward its OWN side. The whole row used to scale from one origin - left unless the row was a lone duet line - so a right-aligned duet line with a backing vocal shrank away from the right edge and floated mid-screen. Vertically, a stacked pair draws toward the seam between them, as the row-level scale did, instead of spreading apart. */ const n = row.voices.length; const originY = n === 1 ? "center" : vi === 0 ? "bottom" : vi === n - 1 ? "top" : "center"; /* rem-anchored, so the user's text size carries through; the vw term still le
499 useQueueStore((s) => s.queue)
500 usePlayerStore((s) => s.setCurrentTrack)
508 useQueueStore.setState({ queue: queue.slice(i + 1) })
510 usePlayerStore.getState().setPlaying(true)
510 usePlayerStore.getState()
511 usePlayerStore.getState().setTargetState("playing")
511 usePlayerStore.getState()
518 queue.map((t, i) => ( <button key={`${t.id}-${i}`} onClick={() => jump(i)} // hover, press and focus from .row-btn - CSS, so a re-render can't wipe them className="row-btn" style={{ gap: 12, padding: "8px 10px", borderRadius: 10 }} > {t.album?.image_url ? <img src={coverUrl(t.album.image_url, 44) ?? t.album.image_url} alt="" style={{ width: 44, height: 44, borderRadius: 6, objectFit: "cover", flexShrink: 0 }} /> : <div style={{ width: 44, height: 44, borderRadius: 6, background: "rgba(255,255,255,0.08)", display: "flex", alignItems: "center", justifyContent: "center", flexShrink: 0 }}><Music size={18} style={{ color: "rgba(255,255,255,0.4)" }} /></div>} <div style={{ minWidth: 0, flex: 1 }}> <p style={{ margin: 0, fontSize: 14, fontWeight: 600, color: "#fff", overflow: "hidden", textOverfl
531 t.artists.map((a) => a.name)
543 usePlayerStore((s) => s.immersiveOpen)
544 usePlayerStore((s) => s.setImmersiveOpen)
545 usePlayerStore((s) => s.immersivePanel)
546 usePlayerStore((s) => s.setImmersivePanel)
547 usePlayerStore((s) => s.currentTrack)
550 useAmbient(track?.album?.image_url)
552 useCanvas(open ? track?.id : null)
553 useState<HTMLVideoElement | null>(null)
557 useVideoAmbient(liveVideo)
562 useEffect(() => { if (!open) return; const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); }; window.addEventListener("keydown", onKey); return () => window.removeEventListener("keydown", onKey); }, [open, setOpen])
589 usePlayerStore.getState()
589 useUIStore.getState().setImmersiveCovered(true)
589 useUIStore.getState()
```

## src/components/layout/JamCard.tsx

Declarations: copy:15, Face:24, Faces:51, hostOf:67, JamCard:73, run:85, adopt:96.

```text
55 shown.map((m, i) => ( <span key={m.id} style={{ marginLeft: i === 0 ? 0 : -8, zIndex: shown.length - i, display: "flex" }}> <Face m={m} size={28} ring /> </span> ))
68 jam.members.find((m) => m.is_host)
74 useQueryClient()
75 useJamStore((s) => s.session)
76 usePlaybackBackend()
80 useState(false)
81 useState(false)
82 useState(false)
83 useState("")
97 useJamStore.getState().setSession(next)
97 useJamStore.getState()
98 qc.setQueryData(["jam"], next)
117 jam.members.filter((m) => !m.is_current_user)
152 jam.members.map((m) => ( <div key={m.id} style={{ display: "flex", alignItems: "center", gap: 8 }}> <Face m={m} size={22} /> <span style={{ flex: 1, minWidth: 0, fontSize: 12.5, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}> {m.is_current_user ? "You" : m.name} </span> {m.is_host && <span className="t-caption" style={{ fontSize: 11, color: "var(--color-text-dim)" }}>Host</span>} {jam.is_host && !m.is_host && !m.is_current_user && ( <Tooltip label={`Remove ${m.name}`} side="top" align="end"> <button type="button" className="btn-icon" aria-label={`Remove ${m.name} from the Jam`} disabled={busy} onClick={() => run(() => kickJamMember(jam.session_id, m.id), `Couldn't remove ${m.name}`)} style={{ width: 22, height: 22, borderRadius: 6 }} > 
220 qc.setQueryData(["jam"], null)
266 setTimeout(() => refreshConnectState(true), 1500)
```

## src/components/layout/Layout.tsx

Declarations: Layout:52, onGlobalKey:199.

```text
53 usePlayerStore((s) => s.queueOpen)
54 usePlayerStore((s) => s.lyricsOpen)
56 usePlayerStore((s) => s.friendsOpen)
57 usePrefsStore((s) => s.showFriends)
59 usePlayerStore((s) => s.immersiveOpen)
61 usePositionTicker()
62 useUIStore((s) => s.windowEffect)
63 useUIStore((s) => s.materialTransparency)
64 useUIStore((s) => s.pageTint)
65 useUIStore((s) => s.backdropActive)
66 useUIStore((s) => s.setBackdropActive)
67 useLocation()
68 useReducedMotion()
69 useRef<HTMLElement>(null)
71 useUIStore((s) => s.sidebarCollapsed)
72 useUIStore((s) => s.macSimulated)
73 usePrefsStore((s) => s.uiZoom)
78 useState(false)
79 useState(false)
80 useUIStore((s) => s.immersiveCovered)
81 useUIStore((s) => s.setImmersiveCovered)
88 useEffect(() => { if (!immersiveOpen) { setShellHidden(false); if (immersiveCovered) setImmersiveCovered(false); return; } if (immersiveCovered) { setShellHidden(true); return; } const t = setTimeout(() => setShellHidden(true), 700); return () => clearTimeout(t); }, [immersiveOpen, immersiveCovered, setImmersiveCovered])
98 setTimeout(() => setShellHidden(true), 700)
99 clearTimeout(t)
102 useEffect(() => { if (typeof window === "undefined") return; const checkCrush = () => { const w = window.innerWidth; const isCollapsed = sidebarCollapsed || w < 768; const sw = isCollapsed ? collapsedSidebarW : 232; const rpw = lyricsOpen ? 366 : queueOpen || friendsOpen ? 272 : 0; const nextCrush = w - sw - rpw < 340; setWillCrushMain((prev) => (prev === nextCrush ? prev : nextCrush)); }; checkCrush(); let rId = 0; const onResizeThrottled = () => { if (rId) return; rId = requestAnimationFrame(() => { rId = 0; checkCrush(); }); }; window.addEventListener("resize", onResizeThrottled, { passive: true }); return () => { window.removeEventListener("resize", onResizeThrottled); if (rId) cancelAnimationFrame(rId); }; }, [sidebarCollapsed, collapsedSidebarW, lyricsOpen, queueOpen, friendsOpen])
118 requestAnimationFrame(() => { rId = 0; checkCrush(); })
127 cancelAnimationFrame(rId)
146 useMotionValue(spacerWidth)
147 useTransform(frameRail, (v) => Math.max(0, v))
148 useTransform(railW, (v) => -v)
149 useTransform(railW, (v) => -v / 2)
151 useLayoutEffect(() => { const from = frameRail.get(); if (from === spacerWidth) return; if (reduceMotion || !mainRef.current) { frameRail.jump(spacerWidth); return; } const main = mainRef.current; const opening = spacerWidth > from; const seconds = opening ? RAIL_OPEN_S : RAIL_CLOSE_S; let live = true; let unpin = () => {}; // after the commit but before the frame is painted: the JS-columned grids // re-render synchronously inside the pin, which React refuses mid-commit queueMicrotask(() => { if (live) unpin = pinRailGrids(main, from - spacerWidth, seconds * 1000); }); const ctl = animate(frameRail, spacerWidth, opening ? RAIL_OPEN : RAIL_CLOSE); ctl.then(() => { if (live) unpin(); }); // the pins hold the landing width, so releasing them moves nothing - it // only hands the grids back to 
187 useEffect(() => { if (mainRef.current) mainRef.current.scrollTop = 0; }, [location.pathname])
191 useEffect(() => { getBackdropActive().then(setBackdropActive).catch(() => setBackdropActive(false)); }, [setBackdropActive])
195 useUIStore((s) => s.toggleMacSimulated)
198 useEffect(() => { function onGlobalKey(e: KeyboardEvent) { if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === "m") { e.preventDefault(); toggleMacSimulated(); } } window.addEventListener("keydown", onGlobalKey); return () => window.removeEventListener("keydown", onGlobalKey); }, [toggleMacSimulated])
```

## src/components/layout/LyricReturnPill.tsx

Declarations: ReturnPill:10.

```text
```

## src/components/layout/LyricsPanel.tsx

Declarations: LyricsPanel:48, TranscriptPanel:53, SongLyricsPanel:78, seekTo:170, subText:457, Pill:472, CenterNote:511, RetryBtn:552.

```text
49 usePlayerStore((s) => !!s.currentTrack && isEpisodeId(s.currentTrack.id))
54 usePlayerStore((s) => s.currentTrack)
55 useAmbient(track?.album?.image_url)
79 usePlayerStore((s) => s.currentTrack)
80 usePlayerStore((s) => s.setPosition)
81 useReducedMotion()
83 useLyrics(track)
84 useAmbient(track?.album?.image_url)
88 usePlayerStore((s) => s.lyricsShowTranslation)
89 usePlayerStore((s) => s.setLyricsShowTranslation)
90 usePlayerStore((s) => s.lyricsShowRoman)
91 usePlayerStore((s) => s.setLyricsShowRoman)
99 useMemo<Row[]>(() => buildRows(data), [data])
101 useMemo(() => rows.map((r) => r.startMs), [rows])
101 rows.map((r) => r.startMs)
104 useMemo( () => rows.flatMap((r) => r.voices.map((v) => v.text)), [rows], )
105 r.voices.map((v) => v.text)
108 useMemo(() => { let o = 0; return rows.map((r) => { const s = o; o += r.voices.length; return s; }); }, [rows])
110 rows.map((r) => { const s = o; o += r.voices.length; return s; })
117 useMemo(() => detectLyricScript(flatTexts), [flatTexts])
123 useState(false)
124 useState<string[] | null>(null)
125 useState(false)
126 useEffect(() => { setRomaji(null); setPron(false); }, [track?.id])
130 useEffect(() => { if (!pron || !canPron || !flatTexts.length || romaji) return; let cancelled = false; setRomanizing(true); romanizeLines(flatTexts, script) .then((r) => { if (!cancelled) setRomaji(r); }) .catch(() => { if (!cancelled) setRomaji(flatTexts.map(() => "")); }) .finally(() => { if (!cancelled) setRomanizing(false); }); return () => { cancelled = true; }; }, [pron, canPron, flatTexts, script, romaji])
139 flatTexts.map(() => "")
150 useLyricClock()
151 useActiveRow(rowStarts, getClock, synced)
157 usePlayerStore((s) => s.currentTrack?.id)
158 useRef<HTMLDivElement>(null)
159 useRef<(HTMLDivElement | null)[]>([])
160 useMoreContrast()
161 useCallback( (el: HTMLDivElement, cont: HTMLDivElement) => el.offsetTop - cont.clientHeight * 0.4 + el.clientHeight / 2, [], )
166 useLyricFollow({ scrollRef, rowRefs, active, resetKey: rows, trackKey: followTrackId, targetFor, reduceMotion, })
321 rows.map((row, ri) => { const isActive = synced && ri === active; const tone = synced ? lyricTone(Math.abs(ri - active), ri < active, moreContrast) : { blur: 0, alpha: 0.92 }; const multi = row.voices.length > 1; const layout = voiceLayout(row); return ( <div key={ri} ref={(el) => { rowRefs.current[ri] = el; }} onClick={() => seekTo(ri)} // hover + press live in styles/lyrics.css className="lyr-prow" data-seekable={synced} data-active={isActive} style={{ /* Constant box. Growing the active row's padding reflowed the whole list on every line, and scaling it from `left center` grew it rightward without the layout knowing - which is what ran long lines off the edge of this panel. Emphasis is light only now, the same as the immersive view. */ padding: "6px 10px", cursor: synced ? "pointer" : "
362 row.voices.map((voice, vi) => { const isDuet = voice.role === "duet"; // lead left, duet right, a backing vocal on the side of // the singer it's under (lib/lyricsRows voiceLayout) const { side: align, secondary: isSecondary } = layout[vi]; // Lead voice is primary; bg voice is visually subordinate (smaller, lower opacity, indented) // rem (19 / 22 / 25px at the default size), so the user's text size carries through const size = isSecondary ? "1.357rem" : isDuet && vi > 0 ? "1.571rem" : "1.786rem"; const weight = isSecondary ? 700 : 800; const romIdx = rowOffsets[ri] + vi; const words = lyricWords(voice, row.startMs, row.endMs); return ( <div key={vi} style={{ // a backing vocal hangs off its singer's side: // indented from that edge, with the rule on it. // no width: 100% - with the inden
```

## src/components/layout/PlayerBar.tsx

Declarations: IconBtn:36, PlayPauseButton:94, Marquee:151, rubberband:212, VolumePopover:217, track:257, onPointerDown:267, release:277, onKeyDown:285, fracFromClientX:463, setDrag:471, endDrag:476, handlePointerDown:487, handlePointerMove:497, handlePointerUp:504, handlePointerCancel:513, handleKeyDown:519, PlayerBar:674, handleClickOutside:749, handleKey:759, applyVolume:785, handleVolumeChange:802, handleMuteToggle:823, handleVolumeButtonClick:833.

```text
159 useRef<HTMLParagraphElement>(null)
160 useState(0)
162 useEffect(() => { if (!active) { setShift(0); return; } if (window.matchMedia?.("(prefers-reduced-motion: reduce)").matches) return; // measured on the clipping <p>: its scrollWidth includes what overflows, // while the inline span inside reports 0 const o = outer.current; if (!o) return; const over = o.scrollWidth - o.clientWidth; setShift(over > 2 ? over : 0); }, [active, children])
177 [className, running ? "pb-marquee" : ""].filter(Boolean)
226 useRef<HTMLDivElement>(null)
227 useRef<DOMRect | null>(null)
228 useState(false)
231 useRef({ shown, onVolume })
235 useMotionValue(0)
236 useTransform(stretch, (x) => 1 + Math.abs(x) / VOL_TRACK_H)
238 useTransform(stretch, (x) => (x >= 0 ? 1 : 0))
240 useEffect(() => { capsuleRef.current?.focus({ preventScroll: true }); }, [])
244 useEffect(() => { const el = popRef.current; if (!el) return; const onWheel = (e: WheelEvent) => { if (e.ctrlKey || e.metaKey) return; // that's zoom e.preventDefault(); const { shown: v, onVolume: set } = live.current; set(clampVol(v + (e.deltaY < 0 ? VOL_STEP : -VOL_STEP))); }; el.addEventListener("wheel", onWheel, { passive: false }); return () => el.removeEventListener("wheel", onWheel); }, [popRef])
271 e.currentTarget.getBoundingClientRect()
444 usePlayerStore((s) => s.positionMs)
445 useState(false)
446 useState(false)
447 useState<number | null>(null)
448 useState<number | null>(null)
449 useRef<HTMLDivElement>(null)
452 useRef(false)
453 useRef<number | null>(null)
466 el.getBoundingClientRect()
521 usePlayerStore.getState()
675 useQueryClient()
676 usePlayerStore((s) => s.isPlaying)
677 usePlayerStore((s) => s.currentTrack)
678 usePlayerStore((s) => s.durationMs)
679 usePlayerStore((s) => s.volume)
680 usePlayerStore((s) => s.muted)
681 usePlayerStore((s) => s.setVolume)
682 usePlayerStore((s) => s.setMuted)
683 usePlayerStore((s) => s.setImmersiveOpen)
685 usePlayerControls()
686 usePlayerStore((s) => isEpisodeId(s.currentId))
687 useDevices()
689 useQueueStore( useShallow((s) => ({ localShuffle: s.shuffle, localRepeat: s.repeat, toggleShuffle: s.toggleShuffle, cycleRepeat: s.cycleRepeat, })) )
690 useShallow((s) => ({ localShuffle: s.shuffle, localRepeat: s.repeat, toggleShuffle: s.toggleShuffle, cycleRepeat: s.cycleRepeat, }))
698 usePlayerStore((s) => s.remoteShuffle)
699 usePlayerStore((s) => s.remoteRepeat)
701 useJamStore(useShallow((s) => (s.session && s.connect ? { shuffle: s.connect.shuffle, repeat: s.connect.repeat_track ? "one" as const : s.connect.repeat_context ? "all" as const : "none" as const } : null)))
701 useShallow((s) => (s.session && s.connect ? { shuffle: s.connect.shuffle, repeat: s.connect.repeat_track ? "one" as const : s.connect.repeat_context ? "all" as const : "none" as const } : null))
715 useRef<HTMLDivElement>(null)
717 useState(false)
718 useState({ showSecondaryControls: true, showInlineVolume: true })
720 useEffect(() => { const el = barContainerRef.current; if (!el) return; const ro = new ResizeObserver((entries) => { const entry = entries[0]; if (!entry) return; const w = entry.contentRect.width; const sec = w >= 560; const vol = w >= 660; setLayoutMode((prev) => { if (prev.showSecondaryControls === sec && prev.showInlineVolume === vol) { return prev; } return { showSecondaryControls: sec, showInlineVolume: vol }; }); }); ro.observe(el); return () => ro.disconnect(); }, [])
743 useState(false)
744 useRef<HTMLDivElement>(null)
745 useRef<HTMLSpanElement>(null)
747 useEffect(() => { if (!volPopupOpen) return; function handleClickOutside(e: MouseEvent) { if ( volPopupRef.current && !volPopupRef.current.contains(e.target as Node) && volBtnRef.current && !volBtnRef.current.contains(e.target as Node) ) { setVolPopupOpen(false); } } function handleKey(e: KeyboardEvent) { if (e.key !== "Escape") return; e.stopPropagation(); setVolPopupOpen(false); volBtnRef.current?.querySelector("button")?.focus(); } window.addEventListener("mousedown", handleClickOutside); // on document, not window: stopPropagation here keeps Escape from also // reaching window-level handlers (Immersive closes on Escape) document.addEventListener("keydown", handleKey); return () => { window.removeEventListener("mousedown", handleClickOutside); document.removeEventListener("keydown", han
763 volBtnRef.current?.querySelector("button")?.focus()
763 volBtnRef.current?.querySelector("button")
775 useEffect(() => { if (showInlineVolume && volPopupOpen) { setVolPopupOpen(false); } }, [showInlineVolume, volPopupOpen])
781 useRef<ReturnType<typeof setTimeout> | null>(null)
787 usePlayerStore.getState()
792 clearTimeout(volDebounce.current)
793 setTimeout(() => { if (usePlayerStore.getState().isRemotePlayback) { remoteSetVolume(v).catch(() => {}); } else { apiSetVolume(v).catch(() => {}); } }, 80)
794 usePlayerStore.getState()
807 useRef(applyVolume)
809 useEffect(() => { const el = volBtnRef.current; if (!el) return; const onWheel = (e: WheelEvent) => { if (e.ctrlKey || e.metaKey) return; // that's zoom e.preventDefault(); const st = usePlayerStore.getState(); const cur = st.muted ? 0 : st.volume; applyVolumeRef.current(clampVol(cur + (e.deltaY < 0 ? VOL_STEP : -VOL_STEP))); }; el.addEventListener("wheel", onWheel, { passive: false }); return () => el.removeEventListener("wheel", onWheel); }, [])
815 usePlayerStore.getState()
826 usePlayerStore.getState()
957 usePlayerStore.getState()
993 usePlayerStore.getState()
1119 currentTrack.artists.map((a, i) => ( <span key={a.id || i}> {i > 0 && ", "} {a.id ? ( <Link to={`/artist/${a.id}`} onClick={() => setImmersiveOpen(false)} className="pb-link" style={{ color: "inherit" }} onPointerEnter={() => prefetchArtist(qc, a.id)} > {a.name} </Link> ) : ( <span>{a.name}</span> )} </span> ))
```

## src/components/layout/QueuePanel.tsx

Declarations: entryUid:28, Cover:38, AddedBy:54, Equaliser:144, SectionHead:188, EmptyRow:209, RemoteQueue:217, JamQueue:248, QueuePanel:298, playItem:332.

```text
117 track.artists.map((a) => a.name)
147 [0, 1, 2].map((i) => ( <span key={i} className="eq-bar-q" style={{ width: 3, borderRadius: 2, background: "var(--color-accent)", height: "100%", ["--eq-delay" as string]: `${i * 0.18}s`, }} /> ))
180 track.artists.map((a) => a.name)
218 useQuery({ queryKey: ["remote-queue"], queryFn: getRemoteQueue, refetchInterval: 5000, staleTime: 2000, })
236 items.slice(0, 30).map((track, i) => ( <QueueTrackRow key={track.id + i} track={track} /> ))
249 useJamStore((s) => s.connect?.next ?? null)
250 useJamStore((s) => s.meta)
251 useJamStore((s) => s.session)
254 useMemo(() => { const byId = new Map((session?.members ?? []).map((m) => [m.id, m])); const rows = (next ?? []) .map((e, i) => { const id = uriToId(e.uri); const track = id ? meta[id] : undefined; return track ? { track, key: `${e.uid || e.uri}-${i}`, queued: e.provider === "queue", by: e.queued_by ? byId.get(e.queued_by) ?? null : null } : null; }) .filter((r): r is NonNullable<typeof r> => !!r) .slice(0, 50); return { added: rows.filter((r) => r.queued), upNext: rows.filter((r) => !r.queued), }; }, [session?.members, next, meta])
255 (session?.members ?? []).map((m) => [m.id, m])
256 (next ?? []) .map((e, i) => { const id = uriToId(e.uri); const track = id ? meta[id] : undefined; return track ? { track, key: `${e.uid || e.uri}-${i}`, queued: e.provider === "queue", by: e.queued_by ? byId.get(e.queued_by) ?? null : null } : null; }) .filter((r): r is NonNullable<typeof r> => !!r)
256 (next ?? []) .map((e, i) => { const id = uriToId(e.uri); const track = id ? meta[id] : undefined; return track ? { track, key: `${e.uid || e.uri}-${i}`, queued: e.provider === "queue", by: e.queued_by ? byId.get(e.queued_by) ?? null : null } : null; })
265 rows.filter((r) => r.queued)
266 rows.filter((r) => !r.queued)
285 added.map(row)
290 upNext.map(row)
299 usePlayerStore((s) => s.toggleQueue)
300 usePlayerStore((s) => s.setCurrentTrack)
301 usePlayerStore((s) => s.currentTrack)
302 usePlayerStore((s) => s.isPlaying)
303 usePlayerStore((s) => s.isRemotePlayback)
304 usePlayerStore((s) => s.activeDevice?.name ?? "device")
305 useJamStore((s) => !!s.session)
307 useQueueStore((s) => s.queue)
308 useQueueStore((s) => s.history)
309 useQueueStore((s) => s.removeAt)
310 useQueueStore((s) => s.setQueue)
311 useQueueStore((s) => s.clearQueue)
312 useQueueStore((s) => s.clearHistory)
317 useMemo(() => { const seen = new Map<number, number>(); const keyedList = queue.map((track) => { const uid = entryUid(track); const n = seen.get(uid) ?? 0; seen.set(uid, n + 1); return { track, key: n === 0 ? `q${uid}` : `q${uid}-${n}` }; }); return { keyed: keyedList, keys: keyedList.map((k) => k.key) }; }, [queue])
319 queue.map((track) => { const uid = entryUid(track); const n = seen.get(uid) ?? 0; seen.set(uid, n + 1); return { track, key: n === 0 ? `q${uid}` : `q${uid}-${n}` }; })
325 keyedList.map((k) => k.key)
328 keyed.map((k) => [k.key, k.track])
329 order.map((k) => byKey.get(k)!).filter(Boolean)
329 order.map((k) => byKey.get(k)!)
334 usePlayerStore.getState().setPlaying(true)
334 usePlayerStore.getState()
335 usePlayerStore.getState().setTargetState("playing")
335 usePlayerStore.getState()
403 keyed.map(({ track, key }, i) => ( <Reorder.Item as="div" key={key} value={key} initial={{ opacity: 0, height: 0 }} animate={{ opacity: 1, height: "auto" }} exit={{ opacity: 0, height: 0, transition: { duration: 0.18, ease: EASE_DRAWER } }} transition={{ duration: 0.2, ease: EASE_OUT }} whileDrag={{ scale: 1.02, boxShadow: "0 10px 26px rgba(0,0,0,0.45)", backgroundColor: "rgba(40,40,46,0.96)", zIndex: 30, }} style={{ position: "relative", borderRadius: 9 }} > <QueueTrackRow track={track} reorderable onRemove={() => removeAt(i)} /> </Reorder.Item> ))
437 [...history].reverse().slice(0, 10).map((track, i) => ( <QueueTrackRow key={track.id + i} track={track} dim onPlay={() => playItem(track)} /> ))
```

## src/components/layout/Sidebar.tsx

Declarations: NavItem:41, useRailIndicator:129, RailIndicator:183, Section:211, Sidebar:291.

```text
130 useReducedMotion()
131 useMotionValue(0)
132 useMotionValue(0)
133 useTransform([top, bottom], ([t, b]: number[]) => Math.max(0, b - t))
134 useState(false)
135 useRef(false)
137 useCallback(() => { const c = containerRef.current; const el = enabled ? c?.querySelector<HTMLElement>('.sb-item[data-active="true"]') : null; if (!c || !el) { shown.current = false; setVisible(false); return; } // measure from the item's centre and untransformed height, so a press // scale still running on the clicked item doesn't skew the target const cr = c.getBoundingClientRect(); const r = el.getBoundingClientRect(); const mid = r.top + r.height / 2 - cr.top + c.scrollTop; const t = mid - el.offsetHeight / 2; const b = mid + el.offsetHeight / 2; // first appearance (rail just collapsed, or arriving from a page with no // rail item) lands in place and fades in; only item-to-item moves travel if (!shown.current || reduceMotion) { top.jump(t); bottom.jump(b); shown.current = true; setVis
139 c?.querySelector<HTMLElement>('.sb-item[data-active="true"]')
147 c.getBoundingClientRect()
148 el.getBoundingClientRect()
169 useLayoutEffect(measure, deps)
172 useEffect(() => { const c = containerRef.current; if (!c || typeof ResizeObserver === "undefined") return; const ro = new ResizeObserver(() => measure()); ro.observe(c); return () => ro.disconnect(); }, [containerRef, measure])
222 children.some(Boolean)
292 useNavigate()
293 useLocation()
294 usePinsStore((s) => s.pins)
295 usePinsStore((s) => s.removePin)
296 usePrefsStore((s) => s.sidebarMode)
297 usePrefsStore((s) => s.showStats)
298 useMyPlaylists()
299 useQueryClient()
300 useContextMenu()
311 usePlaylistFolders()
312 useState<Set<string>>(() => { try { return new Set(JSON.parse(localStorage.getItem("sidebar-closed-folders") ?? "[]") as string[]); } catch { return new Set(); } })
314 localStorage.getItem("sidebar-closed-folders")
319 useCallback((id: string) => { setClosedFolders((prev) => { const next = new Set(prev); if (next.has(id)) next.delete(id); else next.add(id); try { localStorage.setItem("sidebar-closed-folders", JSON.stringify([...next])); } catch { /* private mode */ } return next; }); }, [])
324 localStorage.setItem("sidebar-closed-folders", JSON.stringify([...next]))
328 useMemo(() => new Map(myPlaylists.map((pl) => [pl.id, pl])), [myPlaylists])
328 myPlaylists.map((pl) => [pl.id, pl])
329 useMemo(() => { const m = new Map<string, { name: string | null; image_url: string | null }>(); (function walk(items: RootItem[]) { for (const it of items) { if (it.kind === "playlist") m.set(it.id, { name: it.name, image_url: it.image_url }); else walk(it.children); } })(rootTree ?? []); return m; }, [rootTree])
343 useMemo( () => (useTree ? flattenRows(rootTree!, new Set()) : []), [useTree, rootTree], )
347 useMemo( () => (useTree ? flattenRows(rootTree!, closedFolders) : []), [useTree, rootTree, closedFolders], )
352 useCallback((id: string): PinnedItem => { const pl = syncedById.get(id); const meta = rootNames.get(id); return { id, name: pl?.name ?? meta?.name ?? "Playlist", image_url: pl?.image_url ?? meta?.image_url ?? null, type: "playlist" }; }, [syncedById, rootNames])
358 useMemo(() => { if (useTree) { return allRows .filter((r) => r.kind === "playlist") .map((r) => asEntry(r.id)); } if (showAllPlaylists) { return myPlaylists.map((pl) => ({ id: pl.id, name: pl.name, image_url: pl.image_url, type: "playlist" as const })); } return pins; }, [useTree, allRows, asEntry, showAllPlaylists, myPlaylists, pins])
360 allRows .filter((r) => r.kind === "playlist") .map((r) => asEntry(r.id))
360 allRows .filter((r) => r.kind === "playlist")
365 myPlaylists.map((pl) => ({ id: pl.id, name: pl.name, image_url: pl.image_url, type: "playlist" as const }))
370 useMemo( () => new Map(allRows.filter((r) => r.kind === "playlist").map((r) => [r.id, r.depth])), [allRows], )
371 allRows.filter((r) => r.kind === "playlist").map((r) => [r.id, r.depth])
371 allRows.filter((r) => r.kind === "playlist")
382 entries.some((p) => p.id === openId && p.type === openType)
385 useUIStore((s) => s.sidebarCollapsed)
386 useUIStore((s) => s.macSimulated)
387 useState(() => typeof window !== "undefined" ? window.innerWidth < 768 : false )
391 useEffect(() => { if (typeof window === "undefined") return; const mql = window.matchMedia("(max-width: 767px)"); const handler = (e: MediaQueryListEvent) => setIsNarrow(e.matches); setIsNarrow(mql.matches); mql.addEventListener("change", handler); return () => mql.removeEventListener("change", handler); }, [])
409 usePrefsStore((s) => s.uiZoom)
413 useRef<HTMLDivElement>(null)
414 useRailIndicator(railRef, isCollapsed, [isCollapsed, path, location.search, entries.length, sidebarMode])
416 useState(true)
417 useState(true)
418 useState(true)
456 ["#ff5f57", "#febc2e", "#28c840"].map((c) => ( <div key={c} style={{ width: px(12), height: px(12), flexShrink: 0, borderRadius: "50%", background: c, boxShadow: "inset 0 0 0 1px rgba(0, 0, 0, 0.18)", }} /> ))
528 (useTree ? (isCollapsed ? allRows.filter((r) => r.kind === "playlist") : openRows).map((r) => (r.kind === "folder" ? r : asEntry(r.id))) : entries ).map((row) => { if ("kind" in row && row.kind === "folder") { const fIndent = row.depth * 14; return ( <button key={`folder-${row.id}`} type="button" className="sb-item focus-ring" onClick={() => toggleFolder(row.id)} onDoubleClick={() => navigate(`/playlists?folder=${encodeURIComponent(row.id)}`)} aria-expanded={row.open} title={`${row.name} · ${row.count} playlists (double-click to open)`} style={{ position: "relative", display: "flex", alignItems: "center", gap: 10, height: 34, width: `calc(100% - ${fIndent}px)`, marginLeft: fIndent, padding: "0 8px", border: "none", background: "transparent", borderRadius: 8, cursor: "pointer", color: "var(
529 (isCollapsed ? allRows.filter((r) => r.kind === "playlist") : openRows).map((r) => (r.kind === "folder" ? r : asEntry(r.id)))
529 allRows.filter((r) => r.kind === "playlist")
```

## src/components/layout/TitleBar.tsx

Declarations: CaptionBtn:34, CapsuleButton:79, useHistoryEdges:112, AccountMenuItem:149, AccountMenu:174, WindowCaptionControls:352, TitleBar:430.

```text
113 useLocation()
114 useState({ back: true, forward: true })
116 useEffect(() => { const read = () => { const idx = (window.history.state as { idx?: number } | null)?.idx; if (typeof idx !== "number") { setEdges({ back: true, forward: true }); return; } setEdges({ back: idx > 0, forward: idx < window.history.length - 1 }); }; read(); window.addEventListener("popstate", read); return () => window.removeEventListener("popstate", read); }, [location.key])
175 useState(false)
176 useNavigate()
177 useRef<HTMLDivElement>(null)
178 useRef<HTMLDivElement>(null)
180 useState(false)
181 useAuthStore((s) => s.displayName)
182 useAuthStore((s) => s.imageUrl)
183 useAuthStore((s) => s.loggedIn)
184 useAuthStore((s) => s.product)
185 useCredentialsStore((s) => s.status)
186 useAuth()
188 ref.current?.querySelector<HTMLButtonElement>(".tb-btn")
190 useEffect(() => { if (!open) return; const onDown = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false); }; // Escape closes and hands focus back to the avatar; arrows walk the items const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") { e.stopPropagation(); setOpen(false); trigger()?.focus(); return; } if (e.key !== "ArrowDown" && e.key !== "ArrowUp" && e.key !== "Home" && e.key !== "End") return; const items = Array.from(menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? []); if (items.length === 0) return; e.preventDefault(); const at = items.indexOf(document.activeElement as HTMLButtonElement); const next = e.key === "Home" ? 0 : e.key === "End" ? items.length - 1 : e.key === "ArrowDown" ? (at + 1) % items.l
204 menuRef.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')
225 useEffect(() => { if (!open || !keyboardOpened) return; menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')?.focus(); }, [open, keyboardOpened])
227 menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')?.focus()
227 menuRef.current?.querySelector<HTMLButtonElement>('[role="menuitem"]')
353 useState(false)
354 useUIStore((s) => s.macSimulated)
356 useEffect(() => { if (isMac) return; const win = getCurrentWindow(); win.isMaximized().then(setMaximized).catch(() => {}); const unlisten = win.onResized(() => { win.isMaximized().then(setMaximized).catch(() => {}); }); return () => { unlisten.then((u) => u()).catch(() => {}); }; }, [])
363 unlisten.then((u) => u()).catch(() => {})
363 unlisten.then((u) => u())
431 useNavigate()
432 usePlayerStore((s) => s.lyricsOpen)
433 usePlayerStore((s) => s.toggleLyrics)
434 usePlayerStore((s) => s.queueOpen)
435 usePlayerStore((s) => s.toggleQueue)
436 usePlayerStore((s) => s.friendsOpen)
437 usePlayerStore((s) => s.toggleFriends)
438 usePrefsStore((s) => s.showFriends)
439 usePlayerStore((s) => s.currentTrack !== null)
440 useUIStore((s) => s.sidebarCollapsed)
441 useUIStore((s) => s.toggleSidebar)
442 useUIStore((s) => s.macSimulated)
443 useDevices()
444 useHistoryEdges()
```

## src/components/layout/TopSearch.tsx

Declarations: rowStyle:53, TopSearch:61, onKey:110, FlatItem:135, openResults:152, handleSelect:160, handleKeyDown:186.

```text
62 useNavigate()
63 useLocation()
64 usePlayerStore((s) => s.setCurrentTrack)
68 useState(urlQuery)
69 useState(urlQuery.trim())
70 useState(false)
71 useState(0)
72 useRef<HTMLInputElement>(null)
73 useRef<HTMLDivElement>(null)
76 useRef<string | null>(null)
80 useRef(false)
85 useEffect(() => { if (onSearchPage && urlQuery === pushedRef.current) return; pushedRef.current = null; setQuery(urlQuery); setDebouncedQuery(urlQuery.trim()); }, [location.key, onSearchPage, urlQuery])
92 useEffect(() => { // short: every millisecond here sits between a keystroke and the page // responding to it. keepPreviousData dims the old results meanwhile, so a // shorter wait costs no flashing. const t = setTimeout(() => setDebouncedQuery(query.trim()), onSearchPage ? 120 : 110); return () => clearTimeout(t); }, [query, onSearchPage])
96 setTimeout(() => setDebouncedQuery(query.trim()), onSearchPage ? 120 : 110)
97 clearTimeout(t)
102 useEffect(() => { if (!onSearchPage || !debouncedQuery || debouncedQuery === urlQuery.trim()) return; pushedRef.current = debouncedQuery; navigate(`/search?q=${encodeURIComponent(debouncedQuery)}`, { replace: true }); }, [debouncedQuery, onSearchPage, urlQuery, navigate])
109 useEffect(() => { function onKey(e: KeyboardEvent) { if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === "k") { e.preventDefault(); instantOpenRef.current = true; inputRef.current?.focus(); inputRef.current?.select(); } } window.addEventListener("keydown", onKey); return () => window.removeEventListener("keydown", onKey); }, [])
124 useSearch(onSearchPage ? "" : debouncedQuery)
145 tracks.map((t) => ({ type: "track" as const, data: t }))
146 albums.map((a) => ({ type: "album" as const, data: a }))
147 artists.map((ar) => ({ type: "artist" as const, data: ar }))
148 shows.map((sh) => ({ type: "show" as const, data: sh }))
149 episodes.map((ep) => ({ type: "episode" as const, data: ep }))
203 useEffect(() => { const el = listRef.current?.querySelector(`[data-index="${selectedIndex}"]`) as HTMLElement | null; el?.scrollIntoView({ block: "nearest" }); }, [selectedIndex])
204 listRef.current?.querySelector(`[data-index="${selectedIndex}"]`)
324 tracks.map((t, i) => { const idx = 1 + i; const sel = selectedIndex === idx; return ( <div key={t.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel} onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}> <div style={{ position: "relative", width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}> <CoverArt url={t.album?.image_url} alt={t.name} size={34} /> {sel && ( <div style={{ position: "absolute", inset: 0, background: "rgba(0, 0, 0, 0.4)", display: "flex", alignItems: "center", justifyContent: "center" }}> <Play size={14} fill="#fff" strokeWidth={0} /> </div> )} </div> <div style={{ flex: 1, minWidth: 0 }}> <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.
340 t.artists.map((a) => a.name)
350 albums.map((a, i) => { const idx = 1 + tracks.length + i; const sel = selectedIndex === idx; return ( <div key={a.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel} onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}> <div style={{ width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}> <CoverArt url={a.image_url} alt={a.name} size={34} /> </div> <div style={{ flex: 1, minWidth: 0 }}> <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{a.name}</p> <p style={ROW_SUB}>{a.artists.map((ar) => ar.name).join(", ")}</p> </div> <span style={KIND_TAG}>Album</span> </div> ); })
361 a.artists.map((ar) => ar.name)
369 artists.map((ar, i) => { const idx = 1 + tracks.length + albums.length + i; const sel = selectedIndex === idx; return ( <div key={ar.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel} onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}> <div style={{ width: 34, height: 34, borderRadius: "50%", overflow: "hidden", flexShrink: 0 }}> <CoverArt url={ar.image_url} alt={ar.name} size={34} /> </div> <div style={{ flex: 1, minWidth: 0 }}> <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{ar.name}</p> </div> <span style={KIND_TAG}>Artist</span> </div> ); })
387 shows.map((sh, i) => { const idx = 1 + tracks.length + albums.length + artists.length + i; const sel = selectedIndex === idx; return ( <div key={sh.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel} onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}> <div style={{ width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}> <CoverArt url={sh.image_url} alt={sh.name} size={34} /> </div> <div style={{ flex: 1, minWidth: 0 }}> <p style={{ ...ROW_TITLE, color: sel ? "#fff" : "rgba(255, 255, 255, 0.9)" }}>{sh.name}</p> <p style={ROW_SUB}>{sh.publisher}</p> </div> <span style={KIND_TAG}>Podcast</span> </div> ); })
404 episodes.map((ep, i) => { const idx = 1 + tracks.length + albums.length + artists.length + shows.length + i; const sel = selectedIndex === idx; return ( <div key={ep.id} data-index={idx} id={`ts-opt-${idx}`} className="ts-row" role="option" aria-selected={sel} onClick={() => handleSelect(idx)} onMouseEnter={() => setSelectedIndex(idx)} style={rowStyle(sel)}> <div style={{ position: "relative", width: 34, height: 34, borderRadius: 6, overflow: "hidden", flexShrink: 0 }}> <CoverArt url={ep.image_url} alt={ep.name} size={34} /> {sel && ( <div style={{ position: "absolute", inset: 0, background: "rgba(0, 0, 0, 0.4)", display: "flex", alignItems: "center", justifyContent: "center" }}> <Play size={14} fill="#fff" strokeWidth={0} /> </div> )} </div> <div style={{ flex: 1, minWidth: 0 }}> <p style
```

## src/components/playground/AnimatedIcons.tsx

Declarations: AnimatedPlayPause:4, AnimatedVolumeIcon:64, AnimatedHeart:92.

```text
```

## src/components/playground/PlaygroundCards.tsx

Declarations: MusiqueAlbumCard:6, GenreBucketAlbum:104, GenreBucketCard:160, cycleCrate:174.

```text
17 useState(false)
18 useState(false)
169 useState(false)
170 useState(false)
171 useState(0)
237 visibleCards.map((album, idx) => { // Natural tilted fan layout matching user sketch const restingTransforms = [ { x: -24, y: 10, rotate: -24, scale: 0.92, zIndex: 1 }, { x: -2, y: -6, rotate: -2, scale: 0.98, zIndex: 2 }, { x: 22, y: 6, rotate: 13, scale: 1.0, zIndex: 3 }, ][idx]; const hoverTransforms = [ { x: -32, y: -16, rotate: -28, scale: 0.95, zIndex: 1 }, { x: 0, y: -26, rotate: 0, scale: 1.05, zIndex: 2 }, { x: 30, y: -14, rotate: 18, scale: 1.02, zIndex: 3 }, ][idx]; const currentTransform = hover ? hoverTransforms : restingTransforms; return ( <motion.div key={`${album.id}-${idx}`} animate={{ x: currentTransform.x, y: currentTransform.y, rotate: currentTransform.rotate, scale: currentTransform.scale, }} transition={{ type: "spring", stiffness: 380, damping: 26 }} style={{ positi
```

## src/components/playground/PlaygroundControls.tsx

Declarations: PrimaryPlayButton:7, PulsePlayButton:60, SegmentedControl:118.

```text
136 useRef<HTMLDivElement>(null)
137 useRef<(HTMLButtonElement | null)[]>([])
138 useState<number | null>(null)
140 useRef(false)
141 useRef(0)
143 disabled?.includes(opt)
158 tab.getBoundingClientRect()
236 options.map((opt, idx) => { const off = isDisabled(opt); const active = opt === activeOption && !off; const adornment = adornments?.[opt]; return ( <button key={opt} ref={(el) => { tabRefs.current[idx] = el; }} // aria-disabled rather than the `disabled` attribute on purpose: // a disabled button swallows pointer events, which would stop any // adornment tooltip from opening - exactly when it's the only // thing explaining why the option is greyed out. aria-disabled={off || undefined} role="radio" aria-checked={active} className="seg-opt" onClick={() => { if (!off) onChange(opt); }} style={{ position: "relative", display: "inline-flex", alignItems: "center", gap: 5, padding: "7px clamp(8px, 1.4vw, 16px)", borderRadius: 8, border: "none", background: "transparent", fontSize: "0.875rem", fon
```

## src/components/playground/PlaygroundDials.tsx

Declarations: SpringToggle:6, PlayerBarVolumeSlider:68.

```text
13 useState(initial)
73 useState(initial)
74 useState(false)
```

## src/components/playground/PlaygroundThemeBar.tsx

Declarations: PlaygroundThemeBar:15.

```text
16 useState<string>("#5873D8")
74 PRESET_ACCENTS.map((preset) => { const isSelected = activeHex.toLowerCase() === preset.hex.toLowerCase(); return ( <motion.button key={preset.hex} whileHover={{ scale: 1.04 }} whileTap={{ scale: 0.96 }} onClick={() => pickAccent(preset.hex)} title={`${preset.name} (${preset.hex})`} style={{ display: "flex", alignItems: "center", gap: 7, padding: "7px 13px 7px 8px", borderRadius: 99, border: isSelected ? `1.5px solid ${preset.hex}` : "1px solid rgba(255, 255, 255, 0.1)", background: isSelected ? "rgba(255, 255, 255, 0.08)" : "rgba(255, 255, 255, 0.02)", cursor: "pointer", transition: "border-color 0.15s ease, background 0.15s ease", }} > <span style={{ width: 14, height: 14, borderRadius: 99, background: preset.hex, boxShadow: isSelected ? `0 0 12px ${preset.hex}` : "none", display: "flex",
```

## src/components/playground/PlaygroundTracks.tsx

Declarations: TrackRowItem:15, AlbumTrackRowsSection:170.

```text
26 useState(false)
27 useState(index === 1)
102 [0.4, 1.0, 0.6].map((_, i) => ( <motion.div key={i} animate={{ height: ["20%", "100%", "20%"] }} transition={{ repeat: Infinity, duration: 0.55 + i * 0.15, ease: "easeInOut" }} style={{ flex: 1, borderRadius: 1, background: "var(--color-accent)" }} /> ))
171 useState<string>("2")
175 SAMPLE_TRACKS.map((t, i) => ( <TrackRowItem key={t.id} track={t} index={i} isPlaying={playingId === t.id} onTogglePlay={() => setPlayingId(playingId === t.id ? "" : t.id)} /> ))
```

## src/components/podcast/PodcastStage.tsx

Declarations: episodeIdOf:12, PodcastStage:19, EpisodeAbout:98.

```text
21 usePrefsStore((s) => s.showCanvas)
22 useQuery({ queryKey: ["episode-media", id], queryFn: () => getEpisodeMedia(id), staleTime: Infinity, retry: false, enabled: animated, })
29 useState(true)
100 useQuery({ queryKey: ["episode", id], queryFn: () => getEpisode(id), staleTime: 600_000, retry: false })
```

## src/components/podcast/TranscriptView.tsx

Declarations: activeLine:13, Turn:27, toTurns:37, speakerColor:59, TranscriptView:154.

```text
115 turn.lines.map(({ line, index }) => { const state = !synced ? "static" : index === mark ? "now" : index < mark ? "past" : "next"; return ( <span key={index} data-line={index} role={synced ? "button" : undefined} tabIndex={synced ? 0 : undefined} onClick={synced ? () => onSeek(line.start_ms) : undefined} onKeyDown={synced ? (e) => { if (e.key === "Enter") onSeek(line.start_ms); } : undefined} className="tr-sentence" style={{ cursor: synced ? "pointer" : "text", borderRadius: 6, color: state === "now" || state === "static" ? `rgb(${ink})` : state === "past" ? `rgba(${ink}, 0.36)` : `rgba(${ink}, 0.58)`, textShadow: state === "now" ? `0 0 22px rgba(${ink}, 0.28)` : "none", transition: "color 0.35s ease, text-shadow 0.35s ease", userSelect: synced ? "none" : "text", }} > {line.text}{" "} </spa
166 useQuery({ queryKey: ["transcript", id], queryFn: () => getEpisodeTranscript(id), staleTime: Infinity, retry: false, })
177 usePlayerStore((s) => (data && data.synced ? activeLine(data, s.positionMs) : -1))
178 useMemo(() => (data ? toTurns(data.lines) : []), [data])
185 useState<Transcript | null>(null)
186 useEffect(() => { if (data) startTransition(() => setShownFor(data)); }, [data])
191 useRef<HTMLDivElement>(null)
192 useState(0)
193 useState(0)
196 useState(false)
199 useEffect(() => { if (!userScrolledAt) return; const t = window.setTimeout(() => tick((n) => n + 1), 5100); return () => window.clearTimeout(t); }, [userScrolledAt])
201 window.setTimeout(() => tick((n) => n + 1), 5100)
202 window.clearTimeout(t)
207 useRef(false)
208 useEffect(() => { const box = scroller.current; const el = box?.querySelector<HTMLElement>(`[data-line="${current}"]`); if (!box || !el) return; const first = !placed.current; placed.current = true; const top = el.offsetTop - box.clientHeight * 0.36; if (following) box.scrollTo({ top, behavior: first ? "auto" : "smooth" }); else setNowAbove(el.offsetTop < box.scrollTop); }, [current, following, ready])
210 box?.querySelector<HTMLElement>(`[data-line="${current}"]`)
219 useCallback((ms: number) => { setUserScrolledAt(0); transportSeek(ms); }, [])
248 turns.map((turn, ti) => { const prev = turns[ti - 1]; const first = turn.lines[0].index; const last = turn.lines[turn.lines.length - 1].index; // where the playhead sits relative to this paragraph, collapsed so // that a paragraph it is not inside gets the same prop from one // sentence to the next, and its memo holds const mark = !data.synced ? -1 : current > last ? Infinity : current < first ? -1 : current; return ( <Paragraph key={ti} turn={turn} showSpeaker={!turn.heading && !!turn.speaker && (!prev || prev.heading || prev.speaker !== turn.speaker)} synced={data.synced} mark={mark} dim={data.synced && current >= 0 && !(current >= first && current <= last)} ink={ink} big={big} onSeek={seek} /> ); })
```

## src/components/podcast/transcript.test.ts

Declarations: module constants/functions.

```text
17 turns.map((t) => (t.heading ? "#" : t.speaker))
18 turns[1].lines.map((x) => x.line.text)
```

## src/components/ui/AddToPlaylistModal.tsx

Declarations: AddToPlaylistModal:14, add:27, createAndAdd:43.

```text
15 useAddToPlaylistStore((s) => s.track)
16 useAddToPlaylistStore((s) => s.close)
17 useMyPlaylists()
18 useQueryClient()
19 useRef<HTMLInputElement>(null)
21 useState("")
22 useState(false)
25 playlists.filter((p) => p.name.toLowerCase().includes(filter.trim().toLowerCase()))
25 p.name.toLowerCase().includes(filter.trim().toLowerCase())
33 qc.invalidateQueries({ queryKey: ["playlist", playlistId] })
34 qc.invalidateQueries({ queryKey: LIBRARY_KEYS.playlists })
51 qc.invalidateQueries({ queryKey: LIBRARY_KEYS.playlists })
97 playlists.find((p) => p.name.toLowerCase() === q)
124 shown.map((p) => ( <button key={p.id} onClick={() => add(p.id, p.name)} disabled={busy} className="row-btn" style={rowBtn}> {p.image_url ? <img src={coverUrl(p.image_url, 40) ?? p.image_url} alt="" style={{ ...thumb, objectFit: "cover" }} /> : <span style={{ ...thumb, background: "var(--color-surface-2)" }}><ListMusic size={16} style={{ color: "var(--color-text-dim)" }} /></span>} <span style={{ minWidth: 0, flex: 1, textAlign: "left" }}> <span style={{ display: "block", fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span> <span className="t-caption tnum" style={{ display: "block", fontSize: 11.5, color: "var(--color-text-dim)" }}>{p.total_tracks} tracks</span> </span> </button> ))
```

## src/components/ui/AlbumCard.tsx

Declarations: Props:23, AlbumGrid:30, AlbumCardImpl:46, handlePlayAlbum:58.

```text
31 useId()
47 useReflowPulse()
49 useState(false)
52 useState(false)
53 useQueryClient()
55 usePlayerStore((s) => Boolean(album.id && s.currentTrack?.album?.id === album.id))
56 usePlayerStore((s) => Boolean(s.isPlaying && album.id && s.currentTrack?.album?.id === album.id))
75 useQueueStore.getState().playContext(tracks, 0, album.id, `spotify:album:${album.id}`)
75 useQueueStore.getState()
77 usePlayerStore.getState().setCurrentTrack(start)
77 usePlayerStore.getState()
78 playTrack(start.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(() => {})
78 usePlayerStore.getState().setPlaying(true)
78 usePlayerStore.getState()
204 album.artists?.map((a) => a.name)
```

## src/components/ui/ArtistAbout.tsx

Declarations: AboutStats:11, Props:18, linkLabel:29, Links:34, Cities:57, StatTile:80, AboutModal:90, ArtistAbout:148.

```text
38 links.map((l) => ( <motion.button key={l.url} type="button" className="ghost-pill focus-ring" whileTap={PRESS} transition={PRESS_TRANSITION} onClick={() => openUrl(l.url).catch(() => {})} style={{ height: 30, padding: "0 12px", borderRadius: 99, color: "#fff", fontSize: 12.5, fontWeight: 600, display: "inline-flex", alignItems: "center", gap: 5, cursor: "pointer" }} > {linkLabel(l.name)} <ArrowUpRight size={12} /> </motion.button> ))
58 cities.map((c) => c.listeners ?? 0)
61 cities.slice(0, 5).map((c, i) => ( <li key={`${c.city}-${c.country}`} title={c.listeners != null ? `${c.listeners.toLocaleString()} listeners` : undefined}> <div style={{ display: "flex", justifyContent: "space-between", gap: 10, fontSize: 13, marginBottom: 5 }}> <span style={{ color: "var(--color-text-hi)", fontWeight: 600, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}> <span className="tnum" style={{ color: "var(--color-text-dim)", marginRight: 8 }}>{i + 1}</span> {c.city} {c.country && <span style={{ color: "var(--color-text-dim)", fontWeight: 500 }}>, {c.country}</span>} </span> {c.listeners != null && <span className="tnum" style={{ color: "var(--color-text-dim)", flexShrink: 0 }}>{compact.format(c.listeners)}</span>} </div> <div style={{ height: 4, borderRadiu
62 c.listeners.toLocaleString()
91 useState(0)
128 stats.activeYears.includes("present")
150 useState(false)
151 useState(false)
226 stats.activeYears.includes("present")
```

## src/components/ui/ArtistCard.tsx

Declarations: Props:13, ArtistGrid:23, ArtistCardImpl:39.

```text
24 useId()
40 useReflowPulse()
41 useState(false)
42 useQueryClient()
```

## src/components/ui/ArtistTour.tsx

Declarations: dateParts:10, ArtistTour:23.

```text
30 concerts.slice(0, 9).map((c, i) => { const d = dateParts(c.date); return ( <div key={`${c.title}-${i}`} style={{ display: "flex", alignItems: "center", gap: 14, padding: 12, borderRadius: 14, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", minWidth: 0 }} > <div className="tnum" style={{ width: 52, height: 56, flexShrink: 0, borderRadius: 10, overflow: "hidden", background: "rgba(255,255,255,0.06)", display: "flex", flexDirection: "column", alignItems: "center" }}> <div style={{ width: "100%", textAlign: "center", fontSize: 10.5, fontWeight: 800, letterSpacing: "0.06em", color: "#fff", background: "var(--color-accent)", padding: "3px 0" }}>{d?.month ?? "TBA"}</div> <div style={{ flex: 1, display: "flex", alignItems: "center", fontSize: 21, fontWeight: 800, c
```

## src/components/ui/Carousel.tsx

Declarations: prefersReducedMotion:15, CarouselState:19, useCarousel:26, CarouselControls:76, CarouselTrack:113.

```text
27 useRef<HTMLDivElement>(null)
28 useState({ canPrev: false, canNext: false })
30 useCallback(() => { const el = ref.current; if (!el) return; // is the first / last tile actually cut off? scrollLeft can't answer that: // the track's side padding makes the first snap point ~2px in, and at // fractional display scaling that read as "scrolled", fading the first // tile of a row that was sitting at its start. 1px of slack for rounding. const first = el.firstElementChild; const last = el.lastElementChild; if (!first || !last) return; const box = el.getBoundingClientRect(); const canPrev = first.getBoundingClientRect().left < box.left - 1; const canNext = last.getBoundingClientRect().right > box.right + 1; setEdges((e) => (e.canPrev === canPrev && e.canNext === canNext ? e : { canPrev, canNext })); }, [])
40 el.getBoundingClientRect()
41 first.getBoundingClientRect()
42 last.getBoundingClientRect()
46 useLayoutEffect(() => { update(); // eslint-disable-next-line react-hooks/exhaustive-deps }, deps)
51 useEffect(() => { const el = ref.current; if (!el) return; el.addEventListener("scroll", update, { passive: true }); const ro = new ResizeObserver(update); ro.observe(el); return () => { el.removeEventListener("scroll", update); ro.disconnect(); }; }, [update])
63 useCallback((dir: -1 | 1) => { const el = ref.current; if (!el) return; const first = el.firstElementChild as HTMLElement | null; const gap = parseFloat(getComputedStyle(el).columnGap) || 0; const tile = first ? first.offsetWidth + gap : 0; const step = Math.max(tile, el.clientWidth - tile); el.scrollBy({ left: dir * step, behavior: prefersReducedMotion() ? "auto" : "smooth" }); }, [])
156 items.map((item, i) => ( <div key={getKey(item)} role="listitem" style={{ flex: `0 0 ${typeof itemWidth === "number" ? `${itemWidth}px` : itemWidth}`, width: itemWidth, maxWidth: itemWidth, minWidth: 0, height: itemHeight, scrollSnapAlign: "start", }} > {renderItem(item, i)} </div> ))
```

## src/components/ui/CirclePlayButton.tsx

Declarations: CirclePlayButton:5.

```text
```

## src/components/ui/ConfirmDialog.tsx

Declarations: ConfirmDialog:6.

```text
25 useRef<HTMLButtonElement>(null)
```

## src/components/ui/ContextMenu.tsx

Declarations: MenuEntry:7, Anchor:20, MenuState:24, useContextMenu:30, ContextMenuView:56, MenuRow:134.

```text
31 useState<MenuState | null>(null)
33 useCallback( (entries: MenuEntry[]) => (e: React.MouseEvent) => { e.preventDefault(); e.stopPropagation(); const target = e.currentTarget as HTMLElement | null; if (e.type !== "contextmenu" && target?.getBoundingClientRect) { setMenu({ anchor: { kind: "trigger", rect: target.getBoundingClientRect() }, entries }); } else { setMenu({ anchor: { kind: "point", x: e.clientX, y: e.clientY }, entries }); } }, [], )
39 target.getBoundingClientRect()
47 useCallback(() => setMenu(null), [])
61 useRef<HTMLDivElement>(null)
62 useState<{ x: number; y: number; origin: string } | null>(null)
66 useLayoutEffect(() => { const el = ref.current; if (!el) return; const r = el.getBoundingClientRect(); const W = window.innerWidth; const H = window.innerHeight; if (anchor.kind === "point") { const nx = anchor.x + r.width > W ? W - r.width - 8 : anchor.x; const ny = anchor.y + r.height > H ? H - r.height - 8 : anchor.y; setPos({ x: nx, y: ny, origin: "top left" }); return; } const a = anchor.rect; const below = a.bottom + 6 + r.height <= H - 8; const x = Math.max(8, Math.min(a.right - r.width, W - r.width - 8)); const y = below ? a.bottom + 6 : Math.max(8, a.top - 6 - r.height); setPos({ x, y, origin: below ? "top right" : "bottom right" }); }, [anchor])
69 el.getBoundingClientRect()
85 useEffect(() => { const onDown = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) onClose(); }; window.addEventListener("mousedown", onDown); window.addEventListener("blur", onClose); return () => { window.removeEventListener("mousedown", onDown); window.removeEventListener("blur", onClose); }; }, [onClose])
97 useMenuKeyboard(ref, true, onClose)
126 entries.map((entry, i) => ( <MenuRow key={i} entry={entry} onClose={onClose} /> ))
```

## src/components/ui/CoverArt.tsx

Declarations: Props:5, CoverArt:20, FadeImg:81.

```text
96 useRef<HTMLImageElement>(null)
98 useState<"waiting" | "loaded" | "cached">("waiting")
100 useLayoutEffect(() => { const img = ref.current; if (img && img.complete && img.naturalWidth > 0) setState("cached"); else setState("waiting"); }, [src])
```

## src/components/ui/CreditsList.tsx

Declarations: Avatar:9, Skeleton:40, CreditsList:59.

```text
43 [0, 1, 2, 3].map((i) => ( <div key={i} style={{ display: "flex", alignItems: "center", gap: 12 }}> <span style={{ width: 40, height: 40, borderRadius: "50%", background: "rgba(255,255,255,0.06)" }} /> <span style={{ display: "flex", flexDirection: "column", gap: 6 }}> <span style={{ width: 120 + i * 18, height: 11, borderRadius: 4, background: "rgba(255,255,255,0.08)" }} /> <span style={{ width: 70, height: 9, borderRadius: 4, background: "rgba(255,255,255,0.05)" }} /> </span> </div> ))
60 useQuery({ queryKey: ["credits", trackId], queryFn: () => getTrackCredits(trackId), staleTime: Infinity, retry: false, })
78 data.sections.map((sec) => ( <section key={sec.title}> <h3 style={{ margin: "0 0 10px", fontSize: 11, fontWeight: 700, letterSpacing: "0.08em", textTransform: "uppercase", color: dim }}>{sec.title}</h3> <div style={{ display: "flex", flexDirection: "column", gap: 4 }}> {sec.people.map((p, i) => { const body = ( <> <Avatar person={p} size={40} /> <span style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", gap: 5 }}> <span style={{ fontSize: 14, fontWeight: 650, color: hi, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span> {p.roles.length > 0 && ( <span style={{ display: "flex", flexWrap: "wrap", gap: 4 }}> {p.roles.map((r) => ( <span key={r} style={{ fontSize: 11, fontWeight: 600, lineHeight: "18px", padding: "0 7px", borderRadius: 99, b
82 sec.people.map((p, i) => { const body = ( <> <Avatar person={p} size={40} /> <span style={{ minWidth: 0, flex: 1, display: "flex", flexDirection: "column", gap: 5 }}> <span style={{ fontSize: 14, fontWeight: 650, color: hi, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{p.name}</span> {p.roles.length > 0 && ( <span style={{ display: "flex", flexWrap: "wrap", gap: 4 }}> {p.roles.map((r) => ( <span key={r} style={{ fontSize: 11, fontWeight: 600, lineHeight: "18px", padding: "0 7px", borderRadius: 99, background: chipBg, color: dark ? "rgba(255,255,255,0.85)" : "var(--color-text)" }}> {r} </span> ))} </span> )} </span> </> ); const style = { display: "flex", alignItems: "center", gap: 12, textDecoration: "none", color: "inherit", borderRadius: 10, padding: "6px 8px", m
90 p.roles.map((r) => ( <span key={r} style={{ fontSize: 11, fontWeight: 600, lineHeight: "18px", padding: "0 7px", borderRadius: 99, background: chipBg, color: dark ? "rgba(255,255,255,0.85)" : "var(--color-text)" }}> {r} </span> ))
```

## src/components/ui/CreditsModal.tsx

Declarations: CreditsModal:7.

```text
8 useCreditsStore((s) => s.track)
9 useCreditsStore((s) => s.close)
29 track.artists.map((a) => a.name)
```

## src/components/ui/Dropdown.tsx

Declarations: DropdownOption:7, Props:12, Dropdown:23.

```text
26 useState(false)
27 useRef<HTMLDivElement>(null)
28 useRef<HTMLDivElement>(null)
29 options.find((o) => o.value === value)
30 useCallback(() => setOpen(false), [])
32 useEffect(() => { if (!open) return; const onDown = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false); }; window.addEventListener("mousedown", onDown); return () => window.removeEventListener("mousedown", onDown); }, [open])
41 useMenuKeyboard(menuRef, open, close)
100 options.map((o) => { const active = o.value === value; return ( <button key={o.value} role="menuitemradio" aria-checked={active} className="row-btn" onClick={() => { onChange(o.value); setOpen(false); }} style={{ gap: 8, height: 32, padding: "0 9px", borderRadius: 7, color: active ? "var(--color-text-hi)" : "var(--color-text)", fontSize: 12.5, fontWeight: active ? 600 : 500, whiteSpace: "nowrap", }} > <span style={{ flex: 1 }}>{o.label}</span> {active && <Check size={14} strokeWidth={2.5} style={{ color: "var(--color-accent)" }} />} </button> ); })
```

## src/components/ui/EditPlaylistModal.tsx

Declarations: Visibility:12, visibilityOf:26, Field:32, EditPlaylistModal:74, save:109.

```text
85 useRef<HTMLInputElement>(null)
86 useState(playlist.name)
87 useState(playlist.description ?? "")
88 useState<Visibility>(visibilityOf(playlist))
89 useState(false)
92 useEffect(() => { if (!open) return; setName(playlist.name); setDescription(playlist.description ?? ""); setVisibility(visibilityOf(playlist)); }, [open, playlist])
100 useMemo( () => name.trim() !== playlist.name || description.trim() !== (playlist.description ?? "") || visibility !== initialVis, [name, description, visibility, playlist, initialVis], )
206 VISIBILITY.map((v) => ( <span key={v.value}><b>{v.label}</b>: {v.hint}</span> ))
219 VISIBILITY.map((v) => v.label)
220 VISIBILITY.find((v) => v.value === visibility)
221 VISIBILITY.find((v) => v.label === label)
```

## src/components/ui/EmptyState.tsx

Declarations: EmptyStateProps:3, EmptyState:13.

```text
```

## src/components/ui/EpisodeRow.tsx

Declarations: releaseLabel:17.

```text
45 useState(false)
46 useQueueStore((s) => s.enqueue)
47 useContextMenu()
```

## src/components/ui/EvenGrid.tsx

Declarations: EvenGridProps:6, GridMetrics:19, calculateGridMetrics:24, EvenGrid:56, EvenGridSkeleton:215.

```text
68 useRef<HTMLDivElement>(null)
69 useRef<number>( typeof window !== "undefined" ? Math.max(320, window.innerWidth - 320) : 800 )
73 useState<GridMetrics>(() => { return calculateGridMetrics( lastWidthRef.current, items.length, minColWidth, gap, maxRows, minCols, maxCols ); })
85 useRef({ itemsLength: items.length, minColWidth, gap, maxRows, minCols, maxCols, })
102 useLayoutEffect(() => { const el = containerRef.current; if (!el) return; let rafId = 0; const measure = (w: number) => { if (w <= 0) return; lastWidthRef.current = w; const { itemsLength, minColWidth, gap, maxRows, minCols, maxCols } = paramsRef.current; const next = calculateGridMetrics(w, itemsLength, minColWidth, gap, maxRows, minCols, maxCols); setMetrics((prev) => { if (prev.effectiveCols === next.effectiveCols && prev.visibleCount === next.visibleCount) { return prev; } return next; }); }; measure(el.getBoundingClientRect().width); const ro = new ResizeObserver((entries) => { if (rafId) cancelAnimationFrame(rafId); rafId = requestAnimationFrame(() => { rafId = 0; for (const entry of entries) { measure(entry.contentRect.width); } }); }); ro.observe(el); return () => { if (rafId) canc
120 el.getBoundingClientRect()
123 cancelAnimationFrame(rafId)
124 requestAnimationFrame(() => { rafId = 0; for (const entry of entries) { measure(entry.contentRect.width); } })
134 cancelAnimationFrame(rafId)
140 useRailPin(containerRef, () => { const w = containerRef.current?.getBoundingClientRect().width ?? 0; if (w <= 0) return; lastWidthRef.current = w; const { itemsLength, minColWidth, gap, maxRows, minCols, maxCols } = paramsRef.current; const next = calculateGridMetrics(w, itemsLength, minColWidth, gap, maxRows, minCols, maxCols); setMetrics((prev) => prev.effectiveCols === next.effectiveCols && prev.visibleCount === next.visibleCount ? prev : next, ); })
141 containerRef.current?.getBoundingClientRect()
152 useMemo(() => { return calculateGridMetrics( lastWidthRef.current, items.length, minColWidth, gap, maxRows, minCols, maxCols ); }, [items.length, minColWidth, gap, maxRows, minCols, maxCols])
164 useEffect(() => { setMetrics((prev) => { if (prev.effectiveCols === derivedMetrics.effectiveCols && prev.visibleCount === derivedMetrics.visibleCount) { return prev; } return derivedMetrics; }); }, [derivedMetrics])
175 useId()
193 visibleItems.map((item, index) => { const key = getKey ? getKey(item, index) : index; return ( <motion.div key={key} layout="position" transformTemplate={zTransform} transition={getGridItemTransition(index)} style={{ minWidth: 0, width: "100%", }} > {renderItem(item, index)} </motion.div> ); })
232 useRef<HTMLDivElement>(null)
234 useState<number>(() => { const initialWidth = typeof window !== "undefined" ? Math.max(320, window.innerWidth - 320) : 800; const fitCols = Math.floor((initialWidth + gap) / (minColWidth + gap)); return Math.min(maxCols, Math.max(minCols, fitCols)); })
240 useLayoutEffect(() => { const el = containerRef.current; if (!el) return; const measure = (w: number) => { if (w <= 0) return; const fitCols = Math.floor((w + gap) / (minColWidth + gap)); const nextCols = Math.min(maxCols, Math.max(minCols, fitCols)); setCols((prev) => (prev === nextCols ? prev : nextCols)); }; measure(el.getBoundingClientRect().width); const ro = new ResizeObserver((entries) => { for (const entry of entries) { measure(entry.contentRect.width); } }); ro.observe(el); return () => ro.disconnect(); }, [gap, minColWidth, maxCols, minCols])
250 el.getBoundingClientRect()
262 useRailPin(containerRef, () => { const w = containerRef.current?.getBoundingClientRect().width ?? 0; if (w <= 0) return; const fitCols = Math.floor((w + gap) / (minColWidth + gap)); const nextCols = Math.min(maxCols, Math.max(minCols, fitCols)); setCols((prev) => (prev === nextCols ? prev : nextCols)); })
263 containerRef.current?.getBoundingClientRect()
283 Array.from({ length: totalCount }).map((_, i) => ( <div key={i} style={{ aspectRatio, borderRadius, background: "var(--color-surface)", }} /> ))
```

## src/components/ui/ExpandableDescription.tsx

Declarations: Props:4, ExpandableDescription:12.

```text
19 useState(false)
20 useMemo(() => cleanDescription(text), [text])
```

## src/components/ui/HomeFeedShelves.tsx

Declarations: HomeFeedShelves:16.

```text
17 useQuery({ queryKey: ["library", "home-feed"], queryFn: () => getHomeFeed(Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"), staleTime: 30 * 60_000, retry: false, })
26 data.sections.map((section) => { // only things that have a page to open const items = section.items.filter((it) => ROUTE[it.kind]); if (items.length < 2) return null; return ( <Shelf key={section.id} id={`home-feed-${section.id.replace(/[^a-zA-Z0-9]/g, "")}`} title={section.title} items={items} getKey={(it) => `${it.kind}-${it.id}`} renderItem={(it, i) => ( <MediaTile to={`/${ROUTE[it.kind]}/${it.id}`} imageUrl={it.image_url} title={it.name} subtitle={it.subtitle} index={i} round={it.kind === "artist"} /> )} /> ); })
28 section.items.filter((it) => ROUTE[it.kind])
```

## src/components/ui/Loader.tsx

Declarations: Props:1, Loader:15.

```text
```

## src/components/ui/MediaTile.tsx

Declarations: module constants/functions.

```text
32 useReflowPulse()
33 useState(false)
```

## src/components/ui/Modal.tsx

Declarations: Props:6, Modal:24.

```text
27 useRef<HTMLDivElement>(null)
28 useModalA11y(panelRef, open, onClose, initialFocus)
```

## src/components/ui/MusiqueLogo.tsx

Declarations: MusiqueLogoProps:1, MusiqueLogo:7.

```text
```

## src/components/ui/OverlayScrollbar.tsx

Declarations: OverlayScrollbar:17, onThumbDown:70, onThumbMove:80, endDrag:89, onTrackDown:94.

```text
18 useRef<HTMLDivElement>(null)
19 useState({ size: 0, offset: 0, on: false })
20 useState(false)
21 useState(false)
22 useRef<{ y: number; top: number } | null>(null)
24 useEffect(() => { const el = target.current; if (!el) return; let raf = 0; let idle: ReturnType<typeof setTimeout> | undefined; const update = () => { raf = 0; const track = trackRef.current?.clientHeight ?? el.clientHeight; const { scrollHeight: sh, clientHeight: ch, scrollTop: st } = el; if (sh <= ch + 1) { setM((p) => (p.on ? { size: 0, offset: 0, on: false } : p)); return; } const size = Math.max(MIN_THUMB, (ch / sh) * track); const offset = (st / (sh - ch)) * (track - size); setM((p) => (p.on && p.size === size && p.offset === offset ? p : { size, offset, on: true })); }; // at most one measure per frame, however many events arrive const schedule = () => { if (!raf) raf = requestAnimationFrame(update); }; const onScroll = () => { schedule(); setScrolling(true); clearTimeout(idle); idl
44 requestAnimationFrame(update)
49 clearTimeout(idle)
50 setTimeout(() => setScrolling(false), IDLE_MS)
65 cancelAnimationFrame(raf)
66 clearTimeout(idle)
98 track.getBoundingClientRect()
```

## src/components/ui/PageHeader.tsx

Declarations: Props:9, PageHeader:29.

```text
30 useUIStore((s) => s.setPageTint)
33 useEffect(() => { setPageTint(imageUrl ?? null); return () => setPageTint(null); }, [imageUrl, setPageTint])
```

## src/components/ui/PlayActions.tsx

Declarations: ContextItem:22, Props:24, PlayActions:37, playRemote:103, onPlay:118, onShuffle:141.

```text
38 usePlayerStore((s) => s.setCurrentTrack)
39 usePlayerStore((s) => s.currentTrack)
40 usePlayerStore((s) => s.isPlaying)
41 usePlayerStore((s) => s.sessionReady)
42 useQueueStore((s) => s.playContext)
43 useQueueStore((s) => s.playContextShuffled)
44 useQueueStore((s) => s.contextId)
45 useQueueStore((s) => s.shuffle)
46 usePinsStore((s) => s.pins)
47 usePinsStore((s) => s.togglePin)
48 usePrefsStore((s) => s.sidebarMode)
49 useContextMenu()
56 useRef<HTMLDivElement>(null)
57 useRef<HTMLDivElement>(null)
58 useState(false)
62 useRef(0)
63 useRef(false)
64 useEffect(() => { const el = rootRef.current; if (!el || typeof ResizeObserver === "undefined") return; const ro = new ResizeObserver((entries) => { const w = entries[0]?.contentRect.width ?? 0; if (w <= 0) return; if (!compactRef.current) { const gap = parseFloat(getComputedStyle(el).columnGap) || 0; const kids = Array.from(el.children) as HTMLElement[]; const used = kids.reduce((sum, k) => (k === spacerRef.current ? sum : sum + k.offsetWidth), 0); need.current = used + gap * Math.max(0, kids.length - 1); } const next = w < need.current; compactRef.current = next; setCompact(next); }); ro.observe(el); return () => ro.disconnect(); }, [])
96 pins.some((p) => p.id === pinItem.id)
104 usePlayerStore.getState()
119 pausePlayback().catch(() => {})
119 pausePlayback()
122 usePlayerStore.getState()
132 useSpeedDialStore.getState().recordPlaylist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
132 useSpeedDialStore.getState()
134 useSpeedDialStore.getState().recordAlbum({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
134 useSpeedDialStore.getState()
136 useSpeedDialStore.getState().recordArtist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
136 useSpeedDialStore.getState()
148 useSpeedDialStore.getState().recordPlaylist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
148 useSpeedDialStore.getState()
150 useSpeedDialStore.getState().recordAlbum({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
150 useSpeedDialStore.getState()
152 useSpeedDialStore.getState().recordArtist({ id: pinItem.id, name: pinItem.name, image_url: pinItem.image_url })
152 useSpeedDialStore.getState()
```

## src/components/ui/QuitConfirm.tsx

Declarations: QuitConfirm:7.

```text
8 useUIStore((s) => s.quitConfirmOpen)
9 useUIStore((s) => s.setQuitConfirmOpen)
11 useRef<HTMLButtonElement>(null)
```

## src/components/ui/ReleaseCountdown.tsx

Declarations: ReleaseCountdown:6.

```text
7 useState(() => msUntilRelease(date))
9 useEffect(() => { setRemaining(msUntilRelease(date)); const id = setInterval(() => setRemaining(msUntilRelease(date)), 1000); return () => clearInterval(id); }, [date])
11 setInterval(() => setRemaining(msUntilRelease(date)), 1000)
12 clearInterval(id)
```

## src/components/ui/SaveAlbumButton.tsx

Declarations: SaveAlbumButton:10.

```text
11 useIsAlbumSaved(id)
12 useToggleAlbumSave()
```

## src/components/ui/SectionTitle.tsx

Declarations: SectionTitle:5, ShowAllButton:51.

```text
```

## src/components/ui/Shelf.tsx

Declarations: Shelf:10.

```text
25 useCarousel([items.length])
```

## src/components/ui/SignInPrompt.tsx

Declarations: LoginLinksPanel:27, copy:40, open:61, SignInPrompt:161, showLinks:213.

```text
36 useState<string | null>(null)
37 useState(false)
38 useState("")
80 LOGIN_TABS.map(({ key, label, description }) => ( <div className="sign-in-dropdown-link" key={key}> <div className="sign-in-dropdown-row"> <button type="button" className="sign-in-copy-link" disabled={!links} aria-label={`Copy ${label} link`} onClick={() => void copy(key)} > <span className="sign-in-tab-icon" aria-hidden="true"> <Globe size={17} /> </span> <span className="sign-in-link-label"> <strong>{label}</strong> <span>{description}</span> </span> <span className="sign-in-copy-label"> {copied === key ? ( <> <Check size={13} /> Copied </> ) : ( "Copy" )} </span> </button> <button type="button" className="sign-in-open-link" disabled={!links} aria-label={`Open ${label} tab`} title={`Open ${label} tab`} onClick={() => void open(key)} > <ArrowUpRight size={17} aria-hidden="true" /> </butto
170 useAuth()
171 useState(false)
172 useState(false)
173 useRef<HTMLDivElement>(null)
174 useRef<HTMLButtonElement>(null)
175 useRef<HTMLButtonElement>(null)
176 useRef(false)
177 useId()
178 useId()
180 useEffect(() => { if (!linksOpen) return; const layer = pushKeyLayer(); const onPointer = (event: PointerEvent) => { if (!copyRef.current?.contains(event.target as Node)) setLinksOpen(false); }; const onKey = (event: KeyboardEvent) => { if (event.key !== "Escape" || !isTopKeyLayer(layer)) return; event.preventDefault(); event.stopPropagation(); setLinksOpen(false); copyButtonRef.current?.focus(); }; document.addEventListener("pointerdown", onPointer); window.addEventListener("keydown", onKey, true); return () => { document.removeEventListener("pointerdown", onPointer); window.removeEventListener("keydown", onKey, true); popKeyLayer(layer); }; }, [linksOpen])
202 useEffect(() => { if (!linksOpen || !focusDropdown.current) return; const first = copyRef.current?.querySelector<HTMLButtonElement>( ".sign-in-copy-link:not(:disabled)", ); if (first) { first.focus(); focusDropdown.current = false; } }, [linksOpen, loginLinks])
204 copyRef.current?.querySelector<HTMLButtonElement>( ".sign-in-copy-link:not(:disabled)", )
```

## src/components/ui/Skeletons.tsx

Declarations: TrackRowsSkeleton:10, CardGridSkeleton:40, TopResultSkeleton:68.

```text
13 Array.from({ length: count }).map((_, i) => ( <div key={i} style={{ display: "flex", alignItems: "center", gap: 14, height: 54, padding: "0 12px", // later rows fade out, so the list reads as "more coming" not a wall opacity: Math.max(0.25, 1 - i * 0.09), }} > <div style={{ width: 16, height: 10, borderRadius: 3, background: BLOCK, flexShrink: 0 }} /> {showCover && <div style={{ width: 38, height: 38, borderRadius: 6, background: BLOCK, flexShrink: 0 }} />} <div style={{ display: "flex", flexDirection: "column", gap: 7, flex: 1, minWidth: 0 }}> <div style={{ width: `${46 + ((i * 17) % 30)}%`, height: 11, borderRadius: 4, background: BLOCK }} /> <div style={{ width: `${24 + ((i * 11) % 20)}%`, height: 9, borderRadius: 4, background: BLOCK, opacity: 0.7 }} /> </div> <div style={{ width: 30, 
56 Array.from({ length: count }).map((_, i) => ( <div key={i} style={{ display: "flex", flexDirection: "column", gap: 10, padding: TILE_PAD }}> <div style={{ width: "100%", aspectRatio: "1 / 1", borderRadius: round ? "50%" : 8, background: BLOCK }} /> <div style={{ width: "70%", height: 11, borderRadius: 4, background: BLOCK, alignSelf: round ? "center" : undefined }} /> <div style={{ width: "45%", height: 9, borderRadius: 4, background: BLOCK, opacity: 0.7, alignSelf: round ? "center" : undefined }} /> </div> ))
```

## src/components/ui/SortDirectionToggle.tsx

Declarations: SortDir:6, SortDirectionToggle:11.

```text
```

## src/components/ui/SortToolbar.tsx

Declarations: SortKey:6, Dir:7, NamedPop:9, useSortTools:20.

```text
24 useState("")
25 useState<SortKey>("name")
26 useState<Dir>("asc")
28 useMemo(() => { let arr = items; const q = query.trim().toLowerCase(); if (q) arr = arr.filter((i) => i.name.toLowerCase().includes(q)); const cmp = sort === "name" ? (a: T, b: T) => a.name.localeCompare(b.name) : (a: T, b: T) => (a.popularity ?? 0) - (b.popularity ?? 0); arr = [...arr].sort(cmp); if (dir === "desc") arr.reverse(); return arr; }, [items, query, sort, dir])
31 arr.filter((i) => i.name.toLowerCase().includes(q))
31 i.name.toLowerCase().includes(q)
36 [...arr].sort(cmp)
41 useMemo(() => ( <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 14px" }}> <div className="focus-within-ring" style={{ display: "flex", alignItems: "center", gap: 8, height: 32, flex: "0 1 280px", minWidth: 0, padding: "0 11px", borderRadius: 8, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", transition: "border-color 0.14s ease, box-shadow 0.14s ease", }} > <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} /> <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder={placeholder} aria-label={placeholder} spellCheck={false} style={{ flex: 1, minWidth: 0, height: "100%", border: "none", outline: "none", backgrou
```

## src/components/ui/SpeedControl.tsx

Declarations: speedLabel:9, SpeedControl:16, SkipSeconds:128.

```text
17 usePrefsStore((s) => s.podcastSpeed)
18 usePrefsStore((s) => s.setPodcastSpeed)
19 useState(false)
20 useRef<HTMLDivElement>(null)
22 useEffect(() => { if (!open) return; const onDown = (e: PointerEvent) => { if (!ref.current?.contains(e.target as Node)) setOpen(false); }; const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false); window.addEventListener("pointerdown", onDown); window.addEventListener("keydown", onKey); return () => { window.removeEventListener("pointerdown", onDown); window.removeEventListener("keydown", onKey); }; }, [open])
88 SPEEDS.map((v) => ( <button key={v} type="button" role="menuitemradio" aria-checked={v === speed} className="row-btn tnum" onClick={() => { setSpeed(v); setOpen(false); }} style={{ width: "100%", display: "flex", alignItems: "center", justifyContent: "space-between", gap: 10, padding: "6px 9px", borderRadius: 8, border: "none", background: v === speed ? "rgba(255,255,255,0.08)" : "transparent", color: v === speed ? "var(--color-text-hi)" : "var(--color-text)", fontSize: 13, fontWeight: v === speed ? 700 : 500, cursor: "pointer", textAlign: "left", }} > <span>{v === 1 ? "Normal" : speedLabel(v)}</span> {v === speed && <Check size={13} strokeWidth={2.6} style={{ color: "var(--color-accent)" }} />} </button> ))
```

## src/components/ui/Switch.tsx

Declarations: Switch:5.

```text
```

## src/components/ui/SyncButton.tsx

Declarations: useRelativeTime:11, SyncButton:31.

```text
12 useState(() => Date.now())
13 useEffect(() => { if (!at) return; setNow(Date.now()); const t = window.setInterval(() => setNow(Date.now()), 60_000); return () => window.clearInterval(t); }, [at])
16 window.setInterval(() => setNow(Date.now()), 60_000)
17 window.clearInterval(t)
32 useSyncLibrary()
35 useIsMutating({ mutationKey: LIBRARY_SYNC_KEY })
36 useLibraryStatus()
37 useRelativeTime(status?.last_synced)
```

## src/components/ui/Toaster.tsx

Declarations: Toaster:12.

```text
13 useToastStore((s) => s.toasts)
14 useToastStore((s) => s.remove)
25 toasts.map((t) => { const { Icon, color } = KIND_ICON[t.kind]; return ( <motion.div key={t.id} layout role={t.kind === "error" ? "alert" : "status"} className="toast glass-solid-fallback" initial={{ opacity: 0, y: 14, scale: 0.96 }} animate={{ opacity: 1, y: 0, scale: 1 }} // leaves the way it came, and faster than it arrived exit={{ opacity: 0, y: 8, scale: 0.97, transition: { duration: 0.15, ease: [0.23, 1, 0.32, 1] } }} transition={{ type: "spring", bounce: 0, duration: 0.35 }} style={{ display: "flex", alignItems: "center", gap: 9, padding: t.action ? "6px 6px 6px 16px" : "10px 18px", borderRadius: 99, background: "var(--color-popover)", backdropFilter: "blur(20px)", WebkitBackdropFilter: "blur(20px)", border: "1px solid rgba(255,255,255,0.14)", boxShadow: "0 12px 36px rgba(0,0,0,0.55)
59 useToastStore.getState().toasts.some((x) => x.id === t.id)
59 useToastStore.getState()
```

## src/components/ui/Tooltip.tsx

Declarations: Props:15, Tooltip:31.

```text
32 useState(false)
34 useState(false)
35 useRef(false)
46 useState(align)
50 useState<DOMRect | null>(null)
51 useRef<HTMLSpanElement>(null)
52 useRef<number | undefined>(undefined)
55 useEffect(() => { return () => { if (timerRef.current) window.clearTimeout(timerRef.current); if (openRef.current) { openRef.current = false; openCount = Math.max(0, openCount - 1); lastClosedAt = performance.now(); } }; }, [])
57 window.clearTimeout(timerRef.current)
69 useEffect(() => { if (!open) return; const dismiss = () => setOpen(false); const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); }; window.addEventListener("scroll", dismiss, true); window.addEventListener("resize", dismiss); window.addEventListener("keydown", onKey); return () => { window.removeEventListener("scroll", dismiss, true); window.removeEventListener("resize", dismiss); window.removeEventListener("keydown", onKey); }; }, [open])
87 triggerRef.current.getBoundingClientRect()
90 triggerRef.current.getBoundingClientRect()
101 window.clearTimeout(timerRef.current)
108 window.setTimeout(() => { setOpen(true); }, 140)
124 handleMouseEnter()
129 window.clearTimeout(timerRef.current)
```

## src/components/ui/TrackRow.tsx

Declarations: Props:28, ActionBtn:46, TrackRowImpl:80, handlePlayToggle:98, handleEnqueue:114.

```text
84 useState(false)
86 useQueryClient()
87 useNavigate()
88 useAddToPlaylistStore((s) => s.open)
89 useYtMatchStore((s) => s.open)
92 usePlaybackBackend()
93 useContextMenu()
95 usePlayerStore((s) => s.currentTrack?.id === track.id)
96 usePlayerStore((s) => s.isPlaying && s.currentTrack?.id === track.id)
111 useQueueStore((s) => queueHasTrack(s.queue, track.id))
112 useState(false)
119 setTimeout(() => setJustAdded(false), 2200)
146 useCreditsStore.getState().open(track)
146 useCreditsStore.getState()
274 [0.4, 1.0, 0.6].map((_, i) => ( <div key={i} className="eq-bar" style={{ flex: 1, height: "100%", borderRadius: 1, background: "var(--color-accent)", ["--eq-dur" as string]: `${0.55 + i * 0.15}s`, }} /> ))
297 track.artists.map((a, i) => ( <span key={a.id}> {i > 0 && ", "} <Link to={`/artist/${a.id}`} onClick={stop} className="link-inline" onMouseEnter={() => prefetchArtist(qc, a.id)} > {a.name} </Link> </span> ))
```

## src/components/ui/TrackToolbar.tsx

Declarations: OrderKey:8, Dir:9, useTrackTools:14.

```text
21 useState("")
22 useState<OrderKey>("default")
23 useState<Dir>("asc")
27 useMemo(() => { const m = new Map<TrackItem, string>(); tracks.forEach((t, i) => m.set(t, `${t.id}#${i}`)); return m; }, [tracks])
33 useMemo(() => { let arr = tracks; const q = query.trim().toLowerCase(); if (q) { arr = arr.filter( (t) => t.name.toLowerCase().includes(q) || t.artists.some((a) => a.name.toLowerCase().includes(q)), ); } if (order !== "default") { const cmp: Record<Exclude<OrderKey, "default">, (a: TrackItem, b: TrackItem) => number> = { name: (a, b) => a.name.localeCompare(b.name), artist: (a, b) => firstArtist(a).localeCompare(firstArtist(b)), album: (a, b) => albumName(a).localeCompare(albumName(b)), duration: (a, b) => a.duration_ms - b.duration_ms, }; arr = [...arr].sort(cmp[order]); } else if (dir === "desc") { // default order has no comparator, so descending just reverses it arr = [...arr].reverse(); } if (order !== "default" && dir === "desc") arr = [...arr].reverse(); return arr; }, [tracks, quer
38 arr.filter( (t) => t.name.toLowerCase().includes(q) || t.artists.some((a) => a.name.toLowerCase().includes(q)), )
40 t.name.toLowerCase().includes(q)
41 t.artists.some((a) => a.name.toLowerCase().includes(q))
41 a.name.toLowerCase().includes(q)
52 [...arr].sort(cmp[order])
62 useMemo(() => view.map((t) => keyOf.get(t) ?? t.id), [view, keyOf])
62 view.map((t) => keyOf.get(t) ?? t.id)
64 useMemo<DropdownOption<OrderKey>[]>(() => [ { value: "default", label: defaultOrderLabel }, { value: "name", label: "Name" }, { value: "artist", label: "Artist" }, { value: "album", label: "Album" }, { value: "duration", label: "Duration" }, ], [defaultOrderLabel])
72 useMemo(() => ( <div style={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 10, width: "100%", boxSizing: "border-box", padding: "0 2px 12px" }}> {/* filter box */} <div className="focus-within-ring" style={{ display: "flex", alignItems: "center", gap: 8, height: 32, flex: "0 1 280px", minWidth: 0, padding: "0 11px", borderRadius: 8, background: "var(--color-glass)", border: "1px solid var(--color-glass-border)", transition: "border-color 0.14s ease, box-shadow 0.14s ease", }} > <Search size={14} strokeWidth={2.2} style={{ color: "var(--color-text-dim)", flexShrink: 0 }} /> <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Find in tracks" aria-label="Find in tracks" spellCheck={false} style={{ flex: 1, minWidth: 0, height: "100%", border: "none", 
```

## src/components/ui/YtMatchModal.tsx

Declarations: YtMatchModal:30, pick:52, reset:67.

```text
31 useYtMatchStore((s) => s.trackId)
32 useYtMatchStore((s) => s.label)
33 useYtMatchStore((s) => s.close)
34 useQueryClient()
36 useState(false)
39 useQuery({ queryKey: ["yt-match", trackId], queryFn: () => getYtMatch(trackId!), enabled: open, })
45 useQuery({ queryKey: ["yt-candidates", trackId], queryFn: () => searchYtCandidates(trackId!), enabled: open, staleTime: 60_000, })
57 qc.invalidateQueries({ queryKey: ["yt-match", trackId] })
72 qc.invalidateQueries({ queryKey: ["yt-match", trackId] })
140 candidates.map((c) => { const active = c.video_id === match?.video_id; return ( <button key={c.video_id} onClick={() => pick(c.video_id)} disabled={busy} className="row-btn" data-active={active} aria-current={active || undefined} style={rowBtn} > <span style={{ ...thumb, background: "var(--color-surface-2)" }}> {active ? <Check size={16} style={{ color: "var(--color-accent)" }} /> : <Music size={15} style={{ color: "var(--color-text-dim)" }} />} </span> <span style={{ minWidth: 0, flex: 1, textAlign: "left" }}> <span style={{ display: "block", fontSize: 13.5, fontWeight: 600, color: "var(--color-text-hi)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}> {c.title} {c.explicit && <span style={explicitTag}>E</span>} </span> <span className="t-caption tnum" style={{ disp
163 [c.artists.join(", "), c.album, c.duration_ms != null ? fmtMs(c.duration_ms) : null] .filter(Boolean)
```

## src/components/ui/queueMembership.test.ts

Declarations: module constants/functions.

```text
```

## src/components/ui/queueMembership.ts

Declarations: Queue:1, queueHasTrack:7.

```text
```

## src/hooks/useAlbum.ts

Declarations: useAlbum:5.

```text
6 useQuery({ queryKey: ["album", id], queryFn: () => getAlbum(id!), enabled: !!id, staleTime: 300_000, })
```

## src/hooks/useAmbient.ts

Declarations: AmbientState:5, useAmbient:23.

```text
25 useState(() => { const hit = peekAmbient(src); return { src, ambient: hit ?? AMBIENT_FALLBACK, ready: !!hit }; })
36 useEffect(() => { let live = true; loadAmbient(src).then((a) => { if (live) setState((s) => (s.src === src && s.ambient === a && s.ready ? s : { src, ambient: a, ready: true })); }); return () => { live = false; }; }, [src])
44 useMemo(() => ({ ...state.ambient, ready: state.ready }), [state.ambient, state.ready])
```

## src/hooks/useArtist.ts

Declarations: useArtist:5.

```text
6 useQuery({ queryKey: ["artist", id], queryFn: () => getArtist(id!), enabled: !!id, staleTime: 300_000, })
```

## src/hooks/useAuth.ts

Declarations: useAuth:8.

```text
9 useAuthStore()
10 useQueryClient()
11 useQuery({ queryKey: ["auth-status"], queryFn: async () => { const generation = useAuthStore.getState().generation; const status = await getAuthStatus(); if (currentAuthRequest(generation)) useAuthStore.getState().setFromStatus(status); return status; }, enabled: store.phase === "idle", staleTime: 60_000, refetchOnWindowFocus: true, refetchInterval: 60_000, })
14 useAuthStore.getState()
16 useAuthStore.getState().setFromStatus(status)
16 useAuthStore.getState()
26 useQuery({ queryKey: ["account-profile", store.userId], queryFn: async () => { const generation = useAuthStore.getState().generation; const profile = await getProfile(); if (currentAuthRequest(generation) && useAuthStore.getState().loggedIn) { const status = { logged_in: true, user_id: profile.id, display_name: profile.display_name, email: profile.email, product: profile.product, image_url: profile.image_url, }; useAuthStore.getState().setFromStatus(status); qc.setQueryData(["auth-status"], status); void qc.invalidateQueries({ queryKey: ["playback-backend"] }); } return profile; }, enabled: store.loggedIn && store.phase === "idle", staleTime: 60_000, refetchOnWindowFocus: true, refetchInterval: store.product ? 300_000 : 15_000, retry: 2, })
29 useAuthStore.getState()
31 useAuthStore.getState()
36 useAuthStore.getState().setFromStatus(status)
36 useAuthStore.getState()
37 qc.setQueryData(["auth-status"], status)
38 qc.invalidateQueries({ queryKey: ["playback-backend"] })
49 useMutation({ mutationFn: (openBrowser: boolean) => loginSession(qc, openBrowser), onError: (err) => { console.error("[auth] login failed:", err); toast.error(errMsg(err)); }, })
53 useMutation({ mutationFn: () => logoutSession(qc), onError: (err) => { console.error("[auth] logout failed:", err); toast.error(`Could not finish signing out: ${errMsg(err)}`); }, })
```

## src/hooks/useCanvas.ts

Declarations: useCanvas:8.

```text
9 usePrefsStore((s) => s.showCanvas)
10 useQuery({ queryKey: ["canvas", trackId], queryFn: () => getCanvas(trackId!), enabled: enabled && !!trackId && !isEpisodeId(trackId), staleTime: Infinity, gcTime: 30 * 60_000, retry: false, })
```

## src/hooks/useDevices.ts

Declarations: setupClusterListener:30, useDevices:53.

```text
33 listen("connect:cluster-changed", () => { pushSeen = true; const now = Date.now(); if (now - lastPushRefresh < 500) return; lastPushRefresh = now; clusterListeners.forEach((l) => l()); }) .then((unlisten) => { clusterPending = false; if (clusterListeners.size === 0) { unlisten(); } else { clusterUnlisten = unlisten; } }) .catch(() => { clusterPending = false; })
33 listen("connect:cluster-changed", () => { pushSeen = true; const now = Date.now(); if (now - lastPushRefresh < 500) return; lastPushRefresh = now; clusterListeners.forEach((l) => l()); }) .then((unlisten) => { clusterPending = false; if (clusterListeners.size === 0) { unlisten(); } else { clusterUnlisten = unlisten; } })
33 listen("connect:cluster-changed", () => { pushSeen = true; const now = Date.now(); if (now - lastPushRefresh < 500) return; lastPushRefresh = now; clusterListeners.forEach((l) => l()); })
43 unlisten()
54 useAuthStore((s) => s.loggedIn)
55 usePlayerStore((s) => s.devices)
56 usePlayerStore((s) => s.setDevices)
57 usePlayerStore((s) => s.activeDevice)
58 usePlayerStore((s) => s.setActiveDevice)
59 usePlayerStore((s) => s.musiqueDeviceId)
60 usePlayerStore((s) => s.setMusiqueDeviceId)
61 usePlayerStore((s) => s.isRemotePlayback)
62 usePlayerStore((s) => s.devicesOpen)
63 usePlayerStore((s) => s.toggleDevices)
64 usePlayerStore((s) => s.setDevicesOpen)
65 usePlayerStore((s) => s.syncRemotePlayback)
67 useState<string | null>(null)
70 useEffect(() => { if (!musiqueDeviceId) { getMusiqueDeviceId() .then((id) => { if (id) setMusiqueDeviceId(id); }) .catch(() => {}); } }, [musiqueDeviceId, setMusiqueDeviceId])
80 useCallback(async () => { if (!useAuthStore.getState().loggedIn) return; const generation = useAuthStore.getState().generation; if (inflightDevices) return inflightDevices; inflightDevices = (async () => { try { const payload = await getDevices(); if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return; if (payload) { if (payload.musique_device_id && !usePlayerStore.getState().musiqueDeviceId) { setMusiqueDeviceId(payload.musique_device_id); } setDevices(payload.devices ?? []); const currentActive = payload.devices?.find((d) => d.is_active) ?? null; if (currentActive) { setActiveDevice(currentActive); } } } catch { // Quietly ignore network/auth errors in background poll } finally { inflightDevices = null; } })(); return inflightDevices; }, [setDe
81 useAuthStore.getState()
82 useAuthStore.getState()
84 (async () => { try { const payload = await getDevices(); if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return; if (payload) { if (payload.musique_device_id && !usePlayerStore.getState().musiqueDeviceId) { setMusiqueDeviceId(payload.musique_device_id); } setDevices(payload.devices ?? []); const currentActive = payload.devices?.find((d) => d.is_active) ?? null; if (currentActive) { setActiveDevice(currentActive); } } } catch { // Quietly ignore network/auth errors in background poll } finally { inflightDevices = null; } })()
87 useAuthStore.getState()
87 useAuthStore.getState()
89 usePlayerStore.getState()
93 payload.devices?.find((d) => d.is_active)
107 useCallback(async () => { if (!useAuthStore.getState().loggedIn) return; const generation = useAuthStore.getState().generation; if (inflightPlayback) return inflightPlayback; inflightPlayback = (async () => { try { const state = await getPlaybackState(); if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return; syncRemotePlayback(state); } catch { // Quietly ignore } finally { inflightPlayback = null; } })(); return inflightPlayback; }, [syncRemotePlayback])
108 useAuthStore.getState()
109 useAuthStore.getState()
111 (async () => { try { const state = await getPlaybackState(); if (!useAuthStore.getState().loggedIn || useAuthStore.getState().generation !== generation) return; syncRemotePlayback(state); } catch { // Quietly ignore } finally { inflightPlayback = null; } })()
114 useAuthStore.getState()
114 useAuthStore.getState()
126 useEffect(() => { refreshDevices(); refreshPlayback(); }, [loggedIn, refreshDevices, refreshPlayback])
132 useState(pushSeen)
133 useEffect(() => { const onCluster = () => { setPushLive(true); refreshDevices(); refreshPlayback(); }; clusterListeners.add(onCluster); setupClusterListener(); return () => { clusterListeners.delete(onCluster); if (clusterListeners.size === 0 && clusterUnlisten) { clusterUnlisten(); clusterUnlisten = null; } }; }, [refreshDevices, refreshPlayback])
145 clusterUnlisten()
152 useEffect(() => { if (!devicesOpen) return; refreshDevices(); const timer = setInterval(refreshDevices, 3000); return () => clearInterval(timer); }, [devicesOpen, refreshDevices])
155 setInterval(refreshDevices, 3000)
156 clearInterval(timer)
163 useEffect(() => { const store = usePlayerStore.getState(); const isPlayingLocal = store.isPlaying && !isRemotePlayback; if (isPlayingLocal && !devicesOpen) return; // with pushes flowing, polls only keep the remote scrubber honest and // catch anything a push missed const intervalMs = pushLive ? devicesOpen ? 8000 : isRemotePlayback ? 5000 : 30000 : devicesOpen ? 2500 : isRemotePlayback ? 3000 : 10000; const timer = setInterval(() => { if (typeof document !== "undefined" && document.visibilityState === "hidden") return; refreshPlayback(); }, intervalMs); return () => clearInterval(timer); }, [isRemotePlayback, devicesOpen, refreshPlayback, pushLive])
164 usePlayerStore.getState()
173 setInterval(() => { if (typeof document !== "undefined" && document.visibilityState === "hidden") return; refreshPlayback(); }, intervalMs)
177 clearInterval(timer)
180 useCallback( async (deviceId: string) => { setTransferringId(deviceId); try { const store = usePlayerStore.getState(); const targetDevice = store.devices.find((d) => d.id === deviceId); const isMusique = Boolean(store.musiqueDeviceId && deviceId === store.musiqueDeviceId) || targetDevice?.name.toLowerCase() === "musique"; if (isMusique) { // Transferring to THIS computer const track = store.currentTrack; const pos = Math.max(0, Math.floor(store.positionMs || 0)); store.setIsRemotePlayback(false); store.setTargetState("playing"); store.setPlaying(true); store.setPosition(pos); store.setLastPlayingAt(Date.now()); if (track) { // Instantly start local playback at the exact same position without starting over await resumeOrPlay(track.id, pos).catch((e) => { console.error("[useDevices] resumeOr
184 usePlayerStore.getState()
185 store.devices.find((d) => d.id === deviceId)
214 pausePlayback().catch(() => {})
214 pausePlayback()
218 setTimeout(() => { refreshPlayback(); refreshDevices(); }, 1500)
222 setTimeout(() => { refreshPlayback(); refreshDevices(); }, 3500)
```

## src/hooks/useEvenColumns.test.ts

Declarations: module constants/functions.

```text
```

## src/hooks/useEvenColumns.ts

Declarations: evenColumns:7, useEvenColumns:16.

```text
17 useState(1)
18 useLayoutEffect(() => { const el = ref.current; if (!el) return; const measure = () => setCols(evenColumns(el.clientWidth, count, minItem, gap)); measure(); if (typeof ResizeObserver === "undefined") return; const ro = new ResizeObserver(measure); ro.observe(el); return () => ro.disconnect(); }, [ref, count, minItem, gap])
29 useRailPin(ref, () => { const el = ref.current; if (el) setCols(evenColumns(el.clientWidth, count, minItem, gap)); })
```

## src/hooks/useJamSync.ts

Declarations: hostName:11, namesIn:15, adopt:20, refreshJam:27, useJamSync:44.

```text
12 s?.members.find((m) => m.is_host)
16 (b?.members ?? []).map((m) => m.id)
17 a.members.filter((m) => !other.has(m.id) && !m.is_current_user).map((m) => m.name)
17 a.members.filter((m) => !other.has(m.id) && !m.is_current_user)
21 useJamStore.getState()
22 useJamStore.getState().setSession(next)
22 useJamStore.getState()
28 useAuthStore.getState()
30 useAuthStore.getState()
30 useAuthStore.getState()
31 useJamStore.getState()
32 useJamStore.getState()
45 useQueryClient()
47 useEffect(() => { if (!enabled) return; let gone = false; const offs: Array<() => void> = []; const keep = (p: Promise<() => void>) => p.then((u) => (gone ? u() : offs.push(u))).catch(() => {}); refreshJam().then(() => qc.setQueryData(["jam"], useJamStore.getState().session)); keep( listen<JamUpdate | null>("social:jam-updated", async (e) => { if (gone || !useAuthStore.getState().loggedIn) return; const update = e.payload; const prev = useJamStore.getState().session; const reason = update?.reason ?? "UNKNOWN_UPDATE_TYPE"; switch (reason) { case "SESSION_DELETED": if (prev) { toast(prev.is_host ? "Your Jam ended" : `${hostName(prev)} ended the Jam`); await leftJam(prev.is_host); } break; case "YOU_WERE_KICKED": if (prev) toast("You were removed from the Jam"); await leftJam(false); break; c
54 qc.setQueryData(["jam"], useJamStore.getState().session)
54 useJamStore.getState()
57 listen<JamUpdate | null>("social:jam-updated", async (e) => { if (gone || !useAuthStore.getState().loggedIn) return; const update = e.payload; const prev = useJamStore.getState().session; const reason = update?.reason ?? "UNKNOWN_UPDATE_TYPE"; switch (reason) { case "SESSION_DELETED": if (prev) { toast(prev.is_host ? "Your Jam ended" : `${hostName(prev)} ended the Jam`); await leftJam(prev.is_host); } break; case "YOU_WERE_KICKED": if (prev) toast("You were removed from the Jam"); await leftJam(false); break; case "YOU_LEFT": if (prev) await leftJam(prev.is_host); break; default: { const next = update?.session ?? (await refreshJam()); if (!next) break; if (prev && prev.session_id === next.session_id) { if (reason === "USER_JOINED") namesIn(next, prev).forEach((n) => toast(`${n} joined the 
58 useAuthStore.getState()
60 useJamStore.getState()
88 qc.setQueryData(["jam"], useJamStore.getState().session)
88 useJamStore.getState()
92 listen<ConnectState>("connect:state", (e) => { if (!gone && useAuthStore.getState().loggedIn) applyConnectState(e.payload); })
93 useAuthStore.getState()
97 setInterval(() => { refreshJam().then(() => qc.setQueryData(["jam"], useJamStore.getState().session)); }, 60_000)
98 qc.setQueryData(["jam"], useJamStore.getState().session)
98 useJamStore.getState()
103 clearInterval(poll)
```

## src/hooks/useLibrary.ts

Declarations: useLibraryStatus:43, useLikedSongs:50, useLikedSongsCount:57, useMyPlaylists:64, useSavedAlbums:71, useFollowedArtists:78, useTopTracks:85, useTopArtists:95, useRecentlyPlayed:105, useNewReleases:112, useSyncLibrary:122, useInvalidateLibrary:134, useSavedTrackIds:140, useToggleLike:150, useIsArtistFollowed:163, useToggleFollow:172, useIsAlbumSaved:185, useToggleAlbumSave:194.

```text
44 useQuery({ queryKey: LIBRARY_KEYS.status, queryFn: getLibraryStatus, })
51 useQuery({ queryKey: LIBRARY_KEYS.liked(limit, offset), queryFn: () => getLikedSongs(limit, offset), })
58 useQuery({ queryKey: LIBRARY_KEYS.likedCount, queryFn: getLikedSongsCount, })
65 useQuery({ queryKey: LIBRARY_KEYS.playlists, queryFn: getMyPlaylists, })
72 useQuery({ queryKey: LIBRARY_KEYS.albums, queryFn: getSavedAlbums, })
79 useQuery({ queryKey: LIBRARY_KEYS.artists, queryFn: getFollowedArtists, })
86 useQuery({ queryKey: LIBRARY_KEYS.topTracks(range), queryFn: () => getTopTracks(range), // switching range keeps the last range on screen (dimmed) until the new // one lands, instead of blanking to a skeleton placeholderData: keepPreviousData, })
96 useQuery({ queryKey: LIBRARY_KEYS.topArtists(range), queryFn: () => getTopArtists(range), // switching range keeps the last range on screen (dimmed) until the new // one lands, instead of blanking to a skeleton placeholderData: keepPreviousData, })
106 useQuery({ queryKey: LIBRARY_KEYS.recent, queryFn: getRecentlyPlayed, })
113 useQuery({ queryKey: LIBRARY_KEYS.newReleases, queryFn: getNewReleases, })
123 useQueryClient()
124 useMutation({ mutationKey: LIBRARY_SYNC_KEY, mutationFn: syncLibrary, onSuccess: () => { qc.invalidateQueries({ queryKey: ["library"] }); }, })
128 qc.invalidateQueries({ queryKey: ["library"] })
135 useQueryClient()
136 useCallback(() => qc.invalidateQueries({ queryKey: ["library"] }), [qc])
136 qc.invalidateQueries({ queryKey: ["library"] })
141 useMemo(() => [...ids].sort().join(","), [ids])
141 [...ids].sort()
142 useQuery({ queryKey: ["library", "saved-ids", sortedKey], queryFn: () => getSavedTrackIds(ids), enabled: ids.length > 0, })
151 useQueryClient()
152 useMutation({ mutationFn: ({ id, liked }: { id: string; liked: boolean }) => liked ? unsaveTrack(id) : saveTrack(id), onSuccess: () => { qc.invalidateQueries({ queryKey: ["library", "saved-ids"] }); qc.invalidateQueries({ queryKey: ["library", "liked"] }); }, })
156 qc.invalidateQueries({ queryKey: ["library", "saved-ids"] })
157 qc.invalidateQueries({ queryKey: ["library", "liked"] })
164 useQuery({ queryKey: ["library", "follows", id], queryFn: () => isArtistFollowed(id!), enabled: !!id, })
173 useQueryClient()
174 useMutation({ mutationFn: ({ id, following }: { id: string; following: boolean }) => following ? unfollowArtist(id) : followArtist(id), onSuccess: () => { qc.invalidateQueries({ queryKey: ["library", "follows"] }); qc.invalidateQueries({ queryKey: ["library", "artists"] }); }, })
178 qc.invalidateQueries({ queryKey: ["library", "follows"] })
179 qc.invalidateQueries({ queryKey: ["library", "artists"] })
186 useQuery({ queryKey: ["library", "album-saved", id], queryFn: () => isAlbumSaved(id!), enabled: !!id, })
195 useQueryClient()
196 useMutation({ mutationFn: ({ id, saved }: { id: string; saved: boolean }) => saved ? unsaveAlbum(id) : saveAlbum(id), onMutate: ({ id, saved }) => { qc.setQueryData(["library", "album-saved", id], !saved); }, onError: (_e, { id, saved }) => { qc.setQueryData(["library", "album-saved", id], saved); }, onSettled: () => { qc.invalidateQueries({ queryKey: ["library", "album-saved"] }); qc.invalidateQueries({ queryKey: ["library", "albums"] }); }, })
200 qc.setQueryData(["library", "album-saved", id], !saved)
203 qc.setQueryData(["library", "album-saved", id], saved)
206 qc.invalidateQueries({ queryKey: ["library", "album-saved"] })
207 qc.invalidateQueries({ queryKey: ["library", "albums"] })
```

## src/hooks/useLyricFollow.ts

Declarations: useLyricFollow:27.

```text
48 useState(false)
49 useRef(false)
50 useRef(false)
51 useRef<{ el: HTMLElement; spring: ScrollSpring } | null>(null)
52 useRef(0)
53 useRef(active)
55 useRef(targetFor)
66 useCallback(() => { window.clearTimeout(idleRef.current); detachedRef.current = false; setDetached(false); }, [])
67 window.clearTimeout(idleRef.current)
73 useEffect(() => { placedRef.current = false; attach(); }, [trackKey, attach])
81 useEffect(() => { if (!detachedRef.current) placedRef.current = false; }, [resetKey])
85 useEffect(() => { if (detached || active < 0) return; const el = rowRefs.current?.[active]; const cont = scrollRef.current; if (!el || !cont) return; const spring = springFor(cont); const target = targetForRef.current(el, cont); if (!placedRef.current || reduceMotion) { spring.jump(target); placedRef.current = true; return; } spring.to(target); // the spring is left running on purpose: the next line re-targets it mid-flight }, [active, resetKey, reduceMotion, detached, rowRefs, scrollRef])
103 useEffect(() => { const cont = scrollRef.current; if (!cont) return; const detach = () => { springRef.current?.spring.stop(); if (!detachedRef.current) { detachedRef.current = true; setDetached(true); } window.clearTimeout(idleRef.current); idleRef.current = window.setTimeout(attach, IDLE_RESUME_MS); }; const onPointerDown = (e: PointerEvent) => { // middle-button autoscroll, or a press on the scrollbar itself (outside // the client box). a plain click on a line is a seek, not a scroll. const onBar = e.target === cont && (e.offsetX >= cont.clientWidth || e.offsetY >= cont.clientHeight); if (e.button === 1 || onBar) detach(); }; const onKey = (e: KeyboardEvent) => { if (SCROLL_KEYS.has(e.key)) detach(); }; // scrolled the current line back into the middle of the view by hand: // that is the
113 window.clearTimeout(idleRef.current)
114 window.setTimeout(attach, IDLE_RESUME_MS)
132 cont.getBoundingClientRect()
133 el.getBoundingClientRect()
136 window.clearTimeout(idleRef.current)
137 window.setTimeout(attach, 400)
156 useEffect( () => () => { window.clearTimeout(idleRef.current); springRef.current?.spring.stop(); }, [], )
158 window.clearTimeout(idleRef.current)
```

## src/hooks/useLyrics.ts

Declarations: useLyrics:9.

```text
10 useState<TrackItem | null>(track)
12 useEffect(() => { const timer = setTimeout(() => { setDebouncedTrack(track); }, 280); return () => clearTimeout(timer); }, [track?.id])
13 setTimeout(() => { setDebouncedTrack(track); }, 280)
16 clearTimeout(timer)
19 useQueryClient()
22 useQuery<Lyrics>({ queryKey: ["lyrics", trackId], queryFn: () => getLyrics(debouncedTrack!), enabled: !!debouncedTrack && !isEpisodeId(debouncedTrack.id), staleTime: Infinity, gcTime: 60 * 60_000, retry: 1, })
34 useRef(trackId)
45 useEffect(() => subscribeLyricsUpgrades(queryClient, () => trackIdRef.current), [queryClient])
45 subscribeLyricsUpgrades(queryClient, () => trackIdRef.current)
```

## src/hooks/useMenuKeyboard.ts

Declarations: useMenuKeyboard:11.

```text
18 useRef(onClose)
21 useEffect(() => { if (!open) return; const layer = pushKeyLayer(); const returnTo = document.activeElement as HTMLElement | null; const items = () => Array.from(ref.current?.querySelectorAll<HTMLElement>('[role="menuitem"],[role="menuitemradio"]') ?? []); // after paint, so the menu exists const raf = requestAnimationFrame(() => { const list = items(); const checked = list.find((el) => el.getAttribute("aria-checked") === "true"); (checked ?? list[0])?.focus({ preventScroll: true }); }); const onKey = (e: KeyboardEvent) => { if (!isTopKeyLayer(layer)) return; const list = items(); if (!list.length) return; const i = list.indexOf(document.activeElement as HTMLElement); switch (e.key) { case "ArrowDown": e.preventDefault(); list[(i + 1 + list.length) % list.length]?.focus(); break; case "Arro
26 ref.current?.querySelectorAll<HTMLElement>('[role="menuitem"],[role="menuitemradio"]')
29 requestAnimationFrame(() => { const list = items(); const checked = list.find((el) => el.getAttribute("aria-checked") === "true"); (checked ?? list[0])?.focus({ preventScroll: true }); })
31 list.find((el) => el.getAttribute("aria-checked") === "true")
69 cancelAnimationFrame(raf)
```

## src/hooks/useModalA11y.ts

Declarations: useModalA11y:22.

```text
28 useRef(onClose)
31 useEffect(() => { if (!open) return; const returnTo = document.activeElement as HTMLElement | null; const id = pushKeyLayer(); const raf = requestAnimationFrame(() => { const target = initialFocus?.current ?? ref.current?.querySelector<HTMLElement>(FOCUSABLE) ?? ref.current; target?.focus({ preventScroll: true }); }); const onKey = (e: KeyboardEvent) => { if (!isTopKeyLayer(id)) return; if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); closeRef.current(); return; } if (e.key !== "Tab" || !ref.current) return; const els = Array.from(ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)).filter( (el) => el.offsetParent !== null || el === document.activeElement, ); if (!els.length) { e.preventDefault(); return; } const first = els[0]; const last = els[els.length - 1]; const ac
36 requestAnimationFrame(() => { const target = initialFocus?.current ?? ref.current?.querySelector<HTMLElement>(FOCUSABLE) ?? ref.current; target?.focus({ preventScroll: true }); })
38 ref.current?.querySelector<HTMLElement>(FOCUSABLE)
51 Array.from(ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)).filter( (el) => el.offsetParent !== null || el === document.activeElement, )
51 ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)
71 cancelAnimationFrame(raf)
```

## src/hooks/usePlayEpisodes.ts

Declarations: usePlayEpisodes:14.

```text
15 usePlayerStore((s) => s.setCurrentTrack)
16 useQueueStore((s) => s.playContext)
18 useCallback( (index: number) => { const ep = episodes[index]; if (!ep) return; const tracks = episodes.map(episodeToTrack); if (usePlayerStore.getState().currentId === tracks[index].id) { transportTogglePlay(); return; } const start = playContext(tracks, index, contextId); if (!start) return; setCurrentTrack(start); const pos = !ep.fully_played && ep.resume_position_ms ? ep.resume_position_ms : 0; (pos > 0 ? resumeOrPlay(start.id, pos) : playTrack(start.id)).catch((e) => toast.error(`Couldn't play episode: ${errMsg(e)}`), ); }, [episodes, contextId, playContext, setCurrentTrack], )
22 episodes.map(episodeToTrack)
23 usePlayerStore.getState()
```

## src/hooks/usePlaybackBackend.ts

Declarations: usePlaybackBackend:16.

```text
17 useQuery({ queryKey: ["playback-backend"], queryFn: getPlaybackBackend, staleTime: 300_000, })
```

## src/hooks/usePlayerControls.ts

Declarations: safePlayTrack:21, executeTransport:32, transportTogglePlay:63, transportPlay:117, transportPause:152, transportNext:180, transportPrev:217, transportSeek:259, usePlayerControls:285.

```text
22 usePlayerStore.getState().setIsRemotePlayback(false)
22 usePlayerStore.getState()
24 clearTimeout(playDebounceTimer)
25 setTimeout(() => { if (lastTargetTrackId === id) { playTrack(id).catch((e) => console.error("[transport] safePlayTrack error:", e)); } }, debounceMs)
40 usePlayerStore.getState()
43 pausePlayback().catch((e) => console.error("[transport] pause error:", e))
43 pausePlayback()
49 usePlayerStore.getState().setPlaying(false)
49 usePlayerStore.getState()
50 usePlayerStore.getState().clearTargetState()
50 usePlayerStore.getState()
64 usePlayerStore.getState()
73 remotePlay() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300); }) .catch((e) => { console.error("[transport] remote play error, falling back to local playback:", e); const store = usePlayerStore.getState(); store.setIsRemotePlayback(false); if (store.currentTrack) { resumeOrPlay(store.currentTrack.id, store.positionMs).catch(() => {}); } else { store.setPlaying(false); toast.error("Remote device unavailable"); } })
75 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300)
77 usePlayerStore.getState().syncRemotePlayback(st)
77 usePlayerStore.getState()
82 usePlayerStore.getState()
92 remotePause() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300); }) .catch((e) => { console.error("[transport] remote pause error:", e); usePlayerStore.getState().setPlaying(true); toast.error("Unable to control remote device"); })
94 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300)
96 usePlayerStore.getState().syncRemotePlayback(st)
96 usePlayerStore.getState()
101 usePlayerStore.getState().setPlaying(true)
101 usePlayerStore.getState()
118 usePlayerStore.getState()
125 remotePlay() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300); }) .catch((e) => { console.error("[transport] remote play error, falling back to local playback:", e); const store = usePlayerStore.getState(); store.setIsRemotePlayback(false); if (store.currentTrack) { resumeOrPlay(store.currentTrack.id, store.positionMs).catch(() => {}); } else { store.setPlaying(false); toast.error("Remote device unavailable"); } })
127 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300)
129 usePlayerStore.getState().syncRemotePlayback(st)
129 usePlayerStore.getState()
134 usePlayerStore.getState()
153 usePlayerStore.getState()
160 remotePause() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300); }) .catch((e) => { console.error("[transport] remote pause error:", e); usePlayerStore.getState().setPlaying(true); toast.error("Unable to pause remote device"); })
162 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 300)
164 usePlayerStore.getState().syncRemotePlayback(st)
164 usePlayerStore.getState()
169 usePlayerStore.getState().setPlaying(true)
169 usePlayerStore.getState()
181 usePlayerStore.getState()
187 remoteNext() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 350); }) .catch((e) => { console.error("[transport] remote next error:", e); toast.error("Unable to skip on remote device"); })
189 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 350)
191 usePlayerStore.getState().syncRemotePlayback(st)
191 usePlayerStore.getState()
201 useQueueStore.getState()
203 useQueueStore.getState().advance(currentTrack)
203 useQueueStore.getState()
206 usePlayerStore.getState().setPlaying(true)
206 usePlayerStore.getState()
207 usePlayerStore.getState().setTargetState("playing")
207 usePlayerStore.getState()
210 useQueueStore.getState().peek(n)
210 useQueueStore.getState()
212 setTimeout(() => preloadTrack(upcoming.id).catch(() => {}), 1500)
218 usePlayerStore.getState()
227 remotePrevious() .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 350); }) .catch((e) => { console.error("[transport] remote prev error:", e); toast.error("Unable to skip on remote device"); })
229 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 350)
231 usePlayerStore.getState().syncRemotePlayback(st)
231 usePlayerStore.getState()
245 useQueueStore.getState().previous(currentTrack)
245 useQueueStore.getState()
248 usePlayerStore.getState().setPlaying(true)
248 usePlayerStore.getState()
249 usePlayerStore.getState().setTargetState("playing")
249 usePlayerStore.getState()
251 useQueueStore.getState().peek(p)
251 useQueueStore.getState()
253 setTimeout(() => preloadTrack(upcoming.id).catch(() => {}), 1500)
260 usePlayerStore.getState()
267 remoteSeek(ms) .then(() => { setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 400); }) .catch((e) => { console.error("[transport] remote seek error:", e); toast.error("Unable to seek on remote device"); })
269 setTimeout(async () => { const st = await getPlaybackState().catch(() => null); if (st) usePlayerStore.getState().syncRemotePlayback(st); }, 400)
271 usePlayerStore.getState().syncRemotePlayback(st)
271 usePlayerStore.getState()
286 useCallback(() => transportTogglePlay(), [])
287 useCallback(() => transportPlay(), [])
288 useCallback(() => transportPause(), [])
289 useCallback(() => transportNext(), [])
290 useCallback(() => transportPrev(), [])
291 useCallback((ms: number) => transportSeek(ms), [])
```

## src/hooks/usePlaylist.ts

Declarations: usePlaylist:8.

```text
9 useQuery({ queryKey: ["playlist", id, "cached"], queryFn: () => getCachedPlaylist(id!), enabled: !!id, staleTime: Infinity, })
16 useQuery({ queryKey: ["playlist", id], queryFn: () => getPlaylist(id!), enabled: !!id, staleTime: 120_000, placeholderData: cached.data ?? undefined, })
```

## src/hooks/usePlaylistFolders.ts

Declarations: usePlaylistFolders:7.

```text
8 useAuthStore((s) => s.loggedIn)
9 useQuery({ queryKey: ["library", "rootlist"], queryFn: getPlaylistFolders, enabled: loggedIn, staleTime: 5 * 60_000, retry: false, })
```

## src/hooks/usePositionTicker.ts

Declarations: usePositionTicker:16.

```text
17 usePlayerStore((s) => s.isPlaying)
18 usePlayerStore((s) => s.incrementPos)
20 useEffect(() => { if (!isPlaying) return; const timer = setInterval(incrementPos, 1000); return () => clearInterval(timer); }, [isPlaying, incrementPos])
22 setInterval(incrementPos, 1000)
23 clearInterval(timer)
```

## src/hooks/useReflowPulse.ts

Declarations: onResize:25, subscribe:34, getSnapshot:46, useReflowPulse:51.

```text
27 requestAnimationFrame(() => { raf = 0; tick++; listeners.forEach((l) => l()); })
30 listeners.forEach((l) => l())
36 listeners.add(cb)
38 listeners.delete(cb)
41 cancelAnimationFrame(raf)
53 useSyncExternalStore(subscribe, getSnapshot, getSnapshot)
59 useUIStore((s) => s.sidebarCollapsed)
```

## src/hooks/useSearch.ts

Declarations: useSearch:7.

```text
8 useQuery({ queryKey: ["search", query], queryFn: () => search(query), enabled: query.trim().length > 0, staleTime: 300_000, placeholderData: keepPreviousData, })
```

## src/hooks/useVideoAmbient.ts

Declarations: useVideoAmbient:11.

```text
12 useState<Ambient | null>(null)
14 useEffect(() => { setAmbient(null); if (!video) return; let dead = false; const sample = () => { if (dead || video.paused || document.hidden) return; const a = ambientFromFrame(video); if (a && a.art) setAmbient(a); }; const onReady = () => sample(); video.addEventListener("loadeddata", onReady); video.addEventListener("playing", onReady); const iv = window.setInterval(sample, EVERY_MS); sample(); return () => { dead = true; window.clearInterval(iv); video.removeEventListener("loadeddata", onReady); video.removeEventListener("playing", onReady); }; }, [video])
26 window.setInterval(sample, EVERY_MS)
30 window.clearInterval(iv)
```

## src/hooks/useWindowActive.ts

Declarations: onStateChange:19, subscribe:23, getSnapshot:42, getServerSnapshot:47, useWindowActive:51.

```text
20 listeners.forEach((l) => l())
24 listeners.add(callback)
32 listeners.delete(callback)
52 useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)
```

## src/lib/ambient.ts

Declarations: Ambient:23, grade:48, preblur:61, read:86, peekAmbient:181, loadAmbient:186, ambientFromFrame:215.

```text
74 ctx.drawImage(img, -over / 2, -over / 2, BLUR_PX + over, BLUR_PX + over)
95 ctx.drawImage(img, 0, 0, SAMPLE, SAMPLE)
96 ctx.getImageData(0, 0, SAMPLE, SAMPLE)
116 bin .filter((c) => c.n > 0) .map((c) => ({ rgb: { r: c.r / c.n, g: c.g / c.n, b: c.b / c.n }, score: c.n, sat: c.s / c.n })) .sort((a, b) => b.score - a.score)
116 bin .filter((c) => c.n > 0) .map((c) => ({ rgb: { r: c.r / c.n, g: c.g / c.n, b: c.b / c.n }, score: c.n, sat: c.s / c.n }))
116 bin .filter((c) => c.n > 0)
134 ranked.slice(0, 3).sort((a, b) => b.sat - a.sat)
143 picks.map((p, i) => trip(grade(p, [0.45, 0.85], 0.58 - i * 0.05)))
187 Promise.resolve(AMBIENT_FALLBACK)
190 Promise.resolve(hit)
```

## src/lib/asyncListeners.test.ts

Declarations: module constants/functions.

```text
8 Promise.resolve(first)
8 Promise.reject(new Error("registration failed"))
8 Promise.resolve(last)
10 Promise.resolve()
26 expect(unlisten).toHaveBeenCalledTimes(1)
```

## src/lib/asyncListeners.ts

Declarations: cleanupAsyncListeners:2.

```text
6 registration.then((unlisten) => { if (disposed) unlisten(); else unlisteners.push(unlisten); }).catch(() => {})
7 unlisten()
8 unlisteners.push(unlisten)
14 unlisteners.forEach((unlisten) => unlisten())
14 unlisten()
```

## src/lib/authSession.test.ts

Declarations: deferred:34.

```text
43 localStorage.clear()
45 useAuthStore.setState({ phase: "idle", generation: 0 })
46 useAuthStore.getState().setFromStatus(ACCOUNT)
46 useAuthStore.getState()
53 usePlayerStore.setState({ currentId: "old-song", isPlaying: true })
54 useJamStore.setState({ meta: { stale: {} as never } })
55 localStorage.setItem("theme", "dark")
56 qc.setQueryData(["profile", "me"], { name: "Alice" })
58 expect(useAuthStore.getState()).toMatchObject({ loggedIn: false, imageUrl: null, phase: "logout" })
58 useAuthStore.getState()
59 expect(usePlayerStore.getState()).toMatchObject({ currentId: null, isPlaying: false })
59 usePlayerStore.getState()
60 expect(useJamStore.getState().meta).toEqual({})
60 useJamStore.getState()
63 expect(localStorage.getItem("spotify-player")).toBeNull()
63 localStorage.getItem("spotify-player")
64 expect(localStorage.getItem("theme")).toBe("dark")
64 localStorage.getItem("theme")
67 expect(useAuthStore.getState().phase).toBe("idle")
67 useAuthStore.getState()
79 expect(useAuthStore.getState().loggedIn).toBe(false)
79 useAuthStore.getState()
94 expect(useAuthStore.getState().loginLinks).toEqual(links)
94 useAuthStore.getState()
97 expect(useAuthStore.getState().loginLinks).toBeNull()
97 useAuthStore.getState()
108 expect(useAuthStore.getState()).toMatchObject({ phase: "idle", loginLinks: null })
108 useAuthStore.getState()
114 useAuthStore.getState()
116 Promise.all([logoutSession(qc), loginSession(qc)])
128 usePlayerStore.setState({ currentId: "alice-song" })
129 useQueueStore.setState({ queue: [TRACK] })
131 expect(useAuthStore.getState()).toMatchObject({ userId: "b", displayName: "Bob", imageUrl: null })
131 useAuthStore.getState()
132 expect(usePlayerStore.getState().currentId).toBeNull()
132 usePlayerStore.getState()
133 expect(useQueueStore.getState().queue).toEqual([])
133 useQueueStore.getState()
140 expect(useAuthStore.getState()).toMatchObject({ loggedIn: false, phase: "idle" })
140 useAuthStore.getState()
154 Promise.resolve()
155 Promise.resolve()
```

## src/lib/authSession.ts

Declarations: currentAuthRequest:17, clearLocalAccountState:22, loginSession:35, logoutSession:60.

```text
18 useAuthStore.getState()
23 useAuthStore.getState().clear()
23 useAuthStore.getState()
24 usePlayerStore.getState().clear()
24 usePlayerStore.getState()
25 useQueueStore.getState().clearAll()
25 useQueueStore.getState()
26 usePinsStore.getState().clear()
26 usePinsStore.getState()
27 useSpeedDialStore.getState().clear()
27 useSpeedDialStore.getState()
28 useJamStore.setState({ session: null, connect: null, meta: {}, held: false, localActionAt: 0 })
30 localStorage.removeItem(key)
36 useAuthStore.getState().begin("login")
36 useAuthStore.getState()
38 useAuthStore.getState()
42 useAuthStore.getState().setLoginLinks(generation, links)
42 useAuthStore.getState()
44 useAuthStore.getState()
47 useAuthStore.getState().setFromStatus(data)
47 useAuthStore.getState()
48 qc.setQueryData(["auth-status"], data)
49 useAuthStore.getState().finish(generation)
49 useAuthStore.getState()
51 qc.invalidateQueries()
53 useAuthStore.getState()
54 useAuthStore.getState().finish(generation)
54 useAuthStore.getState()
55 qc.invalidateQueries({ queryKey: ["auth-status"] })
61 useAuthStore.getState().begin("logout")
61 useAuthStore.getState()
66 qc.setQueryData(["auth-status"], LOGGED_OUT)
68 Promise.all([apiLogout(), cancelled])
71 qc.setQueryData(["auth-status"], LOGGED_OUT)
72 useAuthStore.getState().finish(generation)
72 useAuthStore.getState()
73 qc.invalidateQueries({ queryKey: ["auth-status"] })
```

## src/lib/backdrop.test.ts

Declarations: module constants/functions.

```text
```

## src/lib/backdrop.ts

Declarations: backdropScrim:16.

```text
```

## src/lib/color.ts

Declarations: RGB:4, clamp:6, hexToRgb:10, rgbToHex:17, rgbToHsl:22, hslToRgb:38, luminance:54, normalizeBrightness:63, extractVibrant:75, cacheColor:118, loadCoverAccent:128, dataUrlAccent:181, applyAccent:193.

```text
12 h.split("").map((c) => c + c)
82 ctx.drawImage(img, 0, 0, S, S)
83 ctx.getImageData(0, 0, S, S)
133 Promise.resolve(v)
135 localStorage.getItem("cover-accent-v2:" + url)
139 Promise.resolve(v)
145 new Promise<string | null>((resolve) => { const img = new Image(); img.crossOrigin = "anonymous"; // spotifys i.scdn.co sends ACAO:* so canvas can read it img.referrerPolicy = "no-referrer"; img.onload = () => { const raw = extractVibrant(img); const hex = raw ? rgbToHex(normalizeBrightness(raw)) : null; cacheColor(url, hex); try { localStorage.setItem("cover-accent-v2:" + url, hex ?? ""); } catch { /* quota */ } resolve(hex); }; /* the image couldn't be read here (no CORS on that host, blocked, gone). spotify has already picked a colour for its own artwork, so ask it */ img.onerror = () => { getExtractedColors([url]) .then(([c]) => { const hex = c?.raw ? rgbToHex(normalizeBrightness(hexToRgb(c.raw))) : null; cacheColor(url, hex); if (hex) { try { localStorage.setItem("cover-accent-v2:" + 
153 localStorage.setItem("cover-accent-v2:" + url, hex ?? "")
159 getExtractedColors([url]) .then(([c]) => { const hex = c?.raw ? rgbToHex(normalizeBrightness(hexToRgb(c.raw))) : null; cacheColor(url, hex); if (hex) { try { localStorage.setItem("cover-accent-v2:" + url, hex); } catch { /* quota */ } } resolve(hex); }) .catch(() => { cacheColor(url, null); resolve(null); })
164 localStorage.setItem("cover-accent-v2:" + url, hex)
```

## src/lib/coverUrl.test.ts

Declarations: setDpr:22.

```text
```

## src/lib/coverUrl.ts

Declarations: Family:27, coverUrl:44.

```text
54 FAMILIES.find((f) => f.some(([p]) => p === prefix))
54 f.some(([p]) => p === prefix)
59 family.find(([p]) => p === prefix)
```

## src/lib/err.ts

Declarations: errMsg:2.

```text
```

## src/lib/icons.tsx

Declarations: IconProps:17, LucideProps:26, LucideIcon:27, gear:44, IconName:181, Solid:186, svgProps:262, createIcon:275, createVolumeIcon:313, RuneIcon:416.

```text
282 useId().replace(/[^a-zA-Z0-9_-]/g, "")
282 useId()
```

## src/lib/jam.ts

Declarations: uriToId:39, ensureMeta:52, positionOf:66, syncNowPlaying:74, applyConnectState:92, refreshConnectState:101, sessionId:110, guestMayControl:115, send:124, jamSetPlaying:133, jamNext:148, jamPrev:157, jamSeek:169, jamToggleShuffle:180, jamCycleRepeat:189, jamEnqueue:200, jamSkipTo:210, jamPlay:222, seedJamFromLocal:256, leftJam:265.

```text
53 useJamStore.getState()
54 [...new Set(ids)].filter((id) => !meta[id] && !inflight.has(id))
79 useJamStore.getState()
84 usePlayerStore.getState()
89 usePlayerStore.getState().setPosition(positionOf(state))
89 usePlayerStore.getState()
93 useJamStore.getState()
97 state.next.slice(0, 40).map((e) => uriToId(e.uri)).filter((id): id is string => !!id)
97 state.next.slice(0, 40).map((e) => uriToId(e.uri))
104 useJamStore.getState().setConnect(state)
104 useJamStore.getState()
105 useJamStore.getState()
111 useJamStore.getState()
116 useJamStore.getState()
126 Promise.resolve()
134 usePlayerStore.getState()
140 useJamStore.getState().setHeld(!play)
140 useJamStore.getState()
145 (play ? resumePlayback() : pausePlayback()).catch(fail(play ? "Couldn't resume" : "Couldn't pause"))
145 pausePlayback()
158 usePlayerStore.getState()
171 usePlayerStore.getState().setPosition(ms)
171 usePlayerStore.getState()
176 usePlayerStore.getState().setPosition(ms)
176 usePlayerStore.getState()
185 useJamStore.getState()
194 useJamStore.getState()
223 useQueueStore.getState().queue.map((t) => t.id)
223 useQueueStore.getState()
224 upcoming.filter((u) => u !== id)
225 useJamStore.getState().markLocalAction()
225 useJamStore.getState()
237 usePlayerStore.getState()
238 useJamStore.getState()
247 ids.map((t) => ({ uri: idToUri(t) }))
257 usePlayerStore.getState()
259 useQueueStore.getState().queue.map((t) => t.id)
259 useQueueStore.getState()
260 queue.filter((id) => id !== p.currentTrack!.id)
266 useJamStore.getState()
271 usePlayerStore.getState().setPlaying(false)
271 usePlayerStore.getState()
272 pausePlayback().catch(() => {})
272 pausePlayback()
```

## src/lib/keyLayers.ts

Declarations: pushKeyLayer:10, popKeyLayer:16, isTopKeyLayer:21.

```text
```

## src/lib/layout.ts

Declarations: module constants/functions.

```text
```

## src/lib/lyrics.tsx

Declarations: RenderWord:15, mapWords:18, useLyricClock:34, useActiveRow:110, Tone:168, lyricTone:181, subscribeContrast:203, useMoreContrast:208, toneText:231, weldPunctuation:276, lyricWords:385, charPos:411.

```text
20 ws.map((w, i) => ({ text: cleanSpaces(w.text), startMs: w.time_ms, endMs: w.end_ms > w.time_ms ? w.end_ms : (ws[i + 1]?.time_ms ?? w.time_ms + 400), }))
35 usePlayerStore.getState()
36 useRef({ pos: initial.positionMs, at: performance.now() })
37 useRef(initial.isPlaying)
38 useRef(initial.lyricsOffsetMs)
42 useEffect(() => startLatencyPolling(), [])
44 useEffect(() => { const init = usePlayerStore.getState(); baseRef.current = { pos: init.positionMs, at: performance.now() }; playingRef.current = init.isPlaying; offsetRef.current = init.lyricsOffsetMs; return usePlayerStore.subscribe((state, prev) => { offsetRef.current = state.lyricsOffsetMs; if (state.positionMs !== prev.positionMs) { const now = performance.now(); const predicted = playingRef.current ? baseRef.current.pos + (now - baseRef.current.at) : baseRef.current.pos; const drift = state.positionMs - predicted; if (Math.abs(drift) > 1200) { baseRef.current = { pos: state.positionMs, at: now }; // seek / track change } else { baseRef.current = { pos: predicted + drift * 0.2, at: now }; // gentle pull } } // Match the original effects: correct position using the previous play // sta
45 usePlayerStore.getState()
50 usePlayerStore.subscribe((state, prev) => { offsetRef.current = state.lyricsOffsetMs; if (state.positionMs !== prev.positionMs) { const now = performance.now(); const predicted = playingRef.current ? baseRef.current.pos + (now - baseRef.current.at) : baseRef.current.pos; const drift = state.positionMs - predicted; if (Math.abs(drift) > 1200) { baseRef.current = { pos: state.positionMs, at: now }; // seek / track change } else { baseRef.current = { pos: predicted + drift * 0.2, at: now }; // gentle pull } } // Match the original effects: correct position using the previous play // state before rebasing a simultaneous pause/resume update. if (state.isPlaying !== prev.isPlaying) { const now = performance.now(); baseRef.current = { pos: playingRef.current ? baseRef.current.pos + (now - baseRef
81 useCallback(() => { const c = playingRef.current ? baseRef.current.pos + (performance.now() - baseRef.current.at) : baseRef.current.pos; /* the reported position is where the DECODER is; the listener is hearing whatever left it a buffer ago. subtracting the measured output latency is what actually puts the highlight on the beat - the manual offset on top is now purely taste, not compensation. */ return c - getOutputLatencyMs() + offsetRef.current; }, [])
92 useCallback((ms: number) => { baseRef.current = { pos: ms, at: performance.now() }; }, [])
111 useState(-1)
112 useRef(-1)
113 usePlayerStore((s) => s.isPlaying)
115 useEffect(() => { if (!synced || !rowStarts.length) { activeRef.current = -1; setActive(-1); return; } let timer = 0; const run = () => { const t = getClock(); let low = 0; let high = rowStarts.length - 1; let idx = -1; while (low <= high) { const mid = (low + high) >> 1; if (rowStarts[mid] <= t) { idx = mid; low = mid + 1; } else { high = mid - 1; } } if (idx !== activeRef.current) { activeRef.current = idx; setActive(idx); } if (!usePlayerStore.getState().isPlaying) return; const next = rowStarts[idx + 1]; timer = window.setTimeout(run, next === undefined ? 1000 : Math.min(1000, Math.max(16, next - t))); }; run(); // Re-arm on every reported position, including small seeks while paused, // without making unchanged active rows render on each tick. const unsub = usePlayerStore.subscribe((s
141 usePlayerStore.getState()
143 window.setTimeout(run, next === undefined ? 1000 : Math.min(1000, Math.max(16, next - t)))
150 usePlayerStore.subscribe((state, prev) => { if (state.positionMs !== prev.positionMs || state.currentId !== prev.currentId) { window.clearTimeout(timer); run(); } })
152 window.clearTimeout(timer)
158 window.clearTimeout(timer)
209 useSyncExternalStore( subscribeContrast, () => !!window.matchMedia?.(CONTRAST_Q).matches, () => false, )
474 useRef<(HTMLSpanElement | null)[]>([])
475 useRef<{ a: string; b: string; g: string }[]>([])
476 usePlayerStore((s) => s.isPlaying)
482 useWindowActive()
487 useMemo(() => { const starts: number[] = []; const lens: number[] = []; let at = 0; for (const w of toks) { const n = Math.max(w.text.trim().length, 1); starts.push(at); lens.push(n); at += n; } return { starts, lens, total: Math.max(at, 1) }; }, [toks])
500 useEffect(() => { if (!active) return; const { starts, lens, total } = geom; let raf = 0; let pos = charPos(getClock(), toks, starts, lens, total); // start where we are, not at zero let last = performance.now(); lastProps.current = []; const tick = (now: number) => { const t = getClock(); const dt = Math.min(now - last, 64); // a dropped frame must not overshoot last = now; // critically-damped chase, frame-rate independent pos += (charPos(t, toks, starts, lens, total) - pos) * (1 - Math.exp(-dt / CHASE_MS)); for (let i = 0; i < toks.length; i++) { const el = spans.current[i]; if (!el) continue; const n = lens[i]; const local = (pos - starts[i]) / n; const fill = local < 0 ? 0 : local > 1 ? 1 : local; /* The soft edge is about a character and a half wide whatever the word is, rather than 
551 requestAnimationFrame(tick)
555 requestAnimationFrame(tick)
556 cancelAnimationFrame(raf)
574 toks.map((w, i) => ( <span key={i} className={active ? "lyr-word lyr-word-lit" : "lyr-word"} ref={(el) => { spans.current[i] = el; }} style={ active ? { backgroundImage: `linear-gradient(90deg, rgba(${inkRgb}, 0.99) var(--a,-50%), rgba(${inkRgb}, ${dim}) var(--b,-10%))` } : unlit } > {w.text} </span> ))
```

## src/lib/lyricsRows.test.ts

Declarations: module constants/functions.

```text
32 rows.map((r) => lead(r).text)
154 rows.map((r) => lead(r).text)
190 voiceLayout(row(L(0, "a"), L(0, "(a)", "bg"), L(0, "b", "duet"), L(0, "(b)", "bg"))).map((v) => v.side)
197 voiceLayout(r).map((v) => v.side)
```

## src/lib/lyricsRows.ts

Declarations: Row:8, leadLineEnd:25, buildRows:36, VoiceSide:113, voiceLayout:121.

```text
19 (line.words || []).map((w) => ({ ...w, text: cleanSpaces(w.text) }))
53 plain.split(/\r?\n/).map((t) => ({ startMs: -1, endMs: -1, voices: [{ time_ms: -1, text: cleanSpaces(t), words: [] }], }))
123 row.voices.map((v, i) => { const secondary = i > 0 && v.role !== "duet"; if (secondary) return { side: owner, secondary }; owner = v.role === "duet" ? "right" : "left"; return { side: owner, secondary }; })
```

## src/lib/lyricsSource.ts

Declarations: sourceLabel:14.

```text
```

## src/lib/lyricsUpgrades.test.ts

Declarations: Handler:8.

```text
12 Promise.resolve()
12 Promise.resolve()
24 native.listen.mockReset()
25 native.listen.mockImplementation((_event: string, handler: Handler) => { handlers.push(handler); return new Promise<() => void>((resolve, reject) => registrations.push({ resolve, reject })); })
35 subscribe(client, () => "a")
36 subscribe(client, () => "a")
37 expect(native.listen).toHaveBeenCalledTimes(1)
61 subscribe(firstClient, () => selected)
61 subscribe(secondClient, () => selected)
62 subscribe(otherClient, () => "other")
82 subscribe(client, () => "a")
94 subscribe(client, () => "a")()
94 subscribe(client, () => "a")
95 subscribe(client, () => "a")
96 expect(native.listen).toHaveBeenCalledTimes(1)
109 subscribe(client, () => "a")
112 subscribe(client, () => "a")
113 expect(native.listen).toHaveBeenCalledTimes(2)
```

## src/lib/lyricsUpgrades.ts

Declarations: Subscriber:5, connect:14, subscribeLyricsUpgrades:35.

```text
17 listen<Lyrics>("lyrics:upgraded", ({ payload }) => { if (!payload) return; const updated = new Set<QueryClient>(); for (const subscriber of subscribers) { if (payload.track_id !== subscriber.trackId() || updated.has(subscriber.client)) continue; updated.add(subscriber.client); subscriber.client.setQueryData(["lyrics", payload.track_id], payload); } }).then((release) => { pending = false; // A consumer may remount before registration resolves. Reuse that // registration rather than creating another native listener. if (subscribers.size) unlisten = release; else release(); }).catch(() => { pending = false; })
17 listen<Lyrics>("lyrics:upgraded", ({ payload }) => { if (!payload) return; const updated = new Set<QueryClient>(); for (const subscriber of subscribers) { if (payload.track_id !== subscriber.trackId() || updated.has(subscriber.client)) continue; updated.add(subscriber.client); subscriber.client.setQueryData(["lyrics", payload.track_id], payload); } }).then((release) => { pending = false; // A consumer may remount before registration resolves. Reuse that // registration rather than creating another native listener. if (subscribers.size) unlisten = release; else release(); })
17 listen<Lyrics>("lyrics:upgraded", ({ payload }) => { if (!payload) return; const updated = new Set<QueryClient>(); for (const subscriber of subscribers) { if (payload.track_id !== subscriber.trackId() || updated.has(subscriber.client)) continue; updated.add(subscriber.client); subscriber.client.setQueryData(["lyrics", payload.track_id], payload); } })
21 subscriber.trackId()
23 subscriber.client.setQueryData(["lyrics", payload.track_id], payload)
37 subscribers.add(subscriber)
40 subscribers.delete(subscriber)
42 unlisten()
```

## src/lib/lyricsWeld.test.ts

Declarations: expectWellFormed:19, expectMonotonic:36.

```text
14 ws.map((x) => x.text)
92 out.map((t) => [t.startMs, t.endMs])
```

## src/lib/mesh.ts

Declarations: hash:19, meshPaletteIndex:26, meshGradient:31.

```text
```

## src/lib/motion.ts

Declarations: module constants/functions.

```text
```

## src/lib/outputLatency.ts

Declarations: poll:42, getOutputLatencyMs:56, startLatencyPolling:63.

```text
44 invoke<number>("get_output_latency_ms")
67 setInterval(() => void poll(), 10_000)
72 clearInterval(timer)
```

## src/lib/persistStorage.test.ts

Declarations: module constants/functions.

```text
83 expect(localStorage.getItem("rem-key")).toBeNull()
83 localStorage.getItem("rem-key")
89 vi.spyOn(localStorage, "setItem").mockImplementationOnce(() => { throw new Error("quota"); })
115 localStorage.setItem("middleware", JSON.stringify({ state: { volume: 50 }, version: 0 }))
```

## src/lib/persistStorage.ts

Declarations: shallowEqual:17, dedupedStorage:35.

```text
41 localStorage.getItem(name)
61 localStorage.setItem(name, serialized)
68 localStorage.removeItem(name)
```

## src/lib/platform.ts

Declarations: OS:2, detect:4.

```text
```

## src/lib/prefetch.ts

Declarations: prefetchAlbum:8, prefetchArtist:17, prefetchPlaylist:26, prefetchLyrics:36.

```text
```

## src/lib/railFlip.ts

Declarations: pinRailGrids:33, FlipMode:76, playFrom:78, flipChildren:119, useRailPin:137.

```text
34 root.querySelectorAll<HTMLElement>("[data-rail-lock]")
36 root.getBoundingClientRect()
40 grids.map((g) => g.getBoundingClientRect().width)
40 g.getBoundingClientRect()
42 c.getBoundingClientRect()
44 grids.map((g) => g.style.width)
79 card.getBoundingClientRect()
125 kids.map((k) => [k, k.getBoundingClientRect()])
125 k.getBoundingClientRect()
138 useRef(measure)
140 useEffect(() => { const el = ref.current; if (!el) return; const onPin = () => flushSync(() => latest.current()); el.addEventListener(RAIL_PIN, onPin); return () => el.removeEventListener(RAIL_PIN, onPin); }, [ref])
```

## src/lib/rootlist.test.ts

Declarations: module constants/functions.

```text
16 hit?.trail.map((f) => f.id)
27 open.map((r) => r.id)
28 open.find((r) => r.id === "c")
30 closed.map((r) => r.id)
```

## src/lib/rootlist.ts

Declarations: FolderItem:3, findFolder:6, countPlaylists:17, SidebarRow:21, flattenRows:27, hasFolders:41.

```text
42 tree.some((it) => it.kind === "folder")
```

## src/lib/scrollSpring.ts

Declarations: ScrollSpring:23, createScrollSpring:35.

```text
51 cancelAnimationFrame(raf)
75 requestAnimationFrame(step)
87 requestAnimationFrame(step)
```

## src/lib/share.ts

Declarations: ShareKind:4, spotifyUrl:7, copy:12, shareSpotifyLink:34, shareUniversalLink:41.

```text
```

## src/lib/updater.ts

Declarations: runUpdateCheck:12, startDownload:27, restartApp:59.

```text
17 useUpdaterStore .getState() .showAvailable(update.version, update.body ?? null)
17 useUpdaterStore .getState()
28 useUpdaterStore.getState()
```

## src/lib/zoom.ts

Declarations: nearestIndex:22, canZoom:30, announce:38, setZoom:44, stepZoom:51, apply:59, chromePx:74, installZoom:80.

```text
31 usePrefsStore.getState()
39 useToastStore.getState()
46 usePrefsStore.getState()
47 usePrefsStore.getState().setUiZoom(next)
47 usePrefsStore.getState()
52 usePrefsStore.getState()
84 usePrefsStore.getState()
85 usePrefsStore.subscribe((s, prev) => { if (s.uiZoom !== prev.uiZoom) apply(s.uiZoom); })
115 clearTimeout(accTimer)
116 setTimeout(() => { acc = 0; }, 250)
```

## src/main.tsx

Declarations: module constants/functions.

```text
```

## src/pages/AlbumPage.tsx

Declarations: AlbumPage:34, startAt:79.

```text
35 useReflowPulse()
36 useParams<{ id: string }>()
37 useAlbum(id)
38 usePlayerStore((s) => s.setCurrentTrack)
39 useQueueStore((s) => s.enqueue)
40 useQueueStore((s) => s.playContext)
41 usePinsStore((s) => Boolean(data?.id && s.pins.some((p) => p.id === data.id)))
41 s.pins.some((p) => p.id === data.id)
42 usePinsStore((s) => s.togglePin)
43 useToggleLike()
44 useContextMenu()
45 useArtist(data?.artists[0]?.id)
46 useMemo( () => (artistDetail?.albums ?? []).filter((a) => a.id !== data?.id).slice(0, 10), [artistDetail?.albums, data?.id], )
47 (artistDetail?.albums ?? []).filter((a) => a.id !== data?.id)
50 useCarousel([moreBy.length])
53 useMemo(() => tracks.map((t) => t.id), [tracks])
53 tracks.map((t) => t.id)
54 useSavedTrackIds(trackIds)
55 useMemo(() => new Set(savedIds), [savedIds])
57 useTrackTools(tracks, "Album order")
85 useSpeedDialStore.getState().recordAlbum({ id: data.id, name: data.name, image_url: data.image_url })
85 useSpeedDialStore.getState()
109 data.artists.map((a, i) => ( <span key={a.id}> {i > 0 && ", "} <Link to={`/artist/${a.id}`} style={{ color: "inherit", textDecoration: "none" }} className="hover:underline focus-ring"> {a.name} </Link> </span> ))
141 view.map((t, i) => ( <motion.div key={keys[i]} layout="position" transition={{ layout: REFLOW }} style={{ position: "relative" }} whileHover={{ zIndex: 40 }} > <TrackRow track={t} index={i} showCover={false} liked={likedSet.has(t.id)} onPlay={() => startAt(i)} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })} /> </motion.div> ))
178 data.artists.map((a) => a.name)
```

## src/pages/ArtistPage.tsx

Declarations: ArtistPage:40, startTop:108.

```text
41 useReflowPulse()
42 useParams<{ id: string }>()
43 useArtist(id)
44 useIsArtistFollowed(id)
47 useQuery({ queryKey: ["artist-extras", id], queryFn: () => getArtistExtras(id!), enabled: !!id, staleTime: 30 * 60_000, retry: false, })
54 useQuery({ queryKey: ["artist-overview", id], queryFn: () => getArtistOverview(id!), enabled: !!id, staleTime: 30 * 60_000, retry: false, })
61 useToggleFollow()
63 usePlayerStore((s) => s.setCurrentTrack)
64 useQueueStore((s) => s.enqueue)
65 useQueueStore((s) => s.playContext)
66 useToggleLike()
68 useState(false)
69 useContextMenu()
72 useMemo(() => topTracks.map((t) => t.id), [topTracks])
72 topTracks.map((t) => t.id)
73 useSavedTrackIds(topTrackIds)
74 useMemo(() => new Set(savedIds), [savedIds])
113 useSpeedDialStore.getState().recordArtist({ id: data!.id, name: data!.name, image_url: data!.image_url })
113 useSpeedDialStore.getState()
152 overview.monthly_listeners.toLocaleString()
169 genres.map((g, i) => ( <span key={g} style={{ display: "inline-flex", alignItems: "center", gap: 6 }}> {i > 0 && ( <span aria-hidden style={{ color: "rgba(255, 255, 255, 0.35)", fontSize: 10, userSelect: "none" }}>•</span> )} {g} </span> ))
221 shownTop.map((t, i) => ( <motion.div key={t.id} layout="position" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ layout: REFLOW_SPRING, duration: 0.28, delay: Math.min(i % TOP_TRACKS_COLLAPSED, 6) * 0.03, ease: EASE_OUT, }} style={{ position: "relative" }} whileHover={{ zIndex: 40 }} > <TrackRow track={t} index={i} showAlbum liked={likedSet.has(t.id)} onPlay={() => startTop(i)} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) }) } /> </motion.div> ))
298 [...(overview?.gallery ?? []), ...(extras?.gallery ?? []), ...(data.image_url ? [data.image_url] : [])].filter((v, i, arr) => arr.indexOf(v) === i)
```

## src/pages/Home.tsx

Declarations: QuickItem:54, handlePlay:96, startPlayback:121, QuickActionsShelf:392, ScoredCandidate:457, addOrUpdateCandidate:465, TileSkeleton:688, RecTile:694, TrackTiles:813, play:817, MadeForYou:839, play:852, RecentlyPlayed:881, play:888, RangeSlider:928, EmptyHint:947, Refreshing:954, ArtistTiles:968, TopTracks:982, TopArtists:1010, NewReleases:1038, greeting:1066, Home:1083.

```text
78 useState(false)
79 useState(false)
81 useState(false)
82 useNavigate()
83 usePlayerStore((s) => s.isPlaying)
84 usePlayerStore((s) => Boolean(item.track && s.currentTrack?.id === item.track.id))
85 useQueueStore((s) => s.contextId)
122 useQueueStore.getState()
123 usePlayerStore.getState()
124 usePlayerStore.getState().setPlaying(val)
124 usePlayerStore.getState()
168 recentTracks.filter((t) => t.id !== target!.id)
360 [0.4, 1.0, 0.6].map((_, i) => ( <div key={i} className="eq-bar" style={{ flex: 1, height: "100%", borderRadius: 1, background: "var(--color-accent)", ["--eq-dur" as string]: `${0.55 + i * 0.15}s`, }} /> ))
393 useRef<HTMLDivElement>(null)
395 useState<number>(() => { if (typeof window !== "undefined") { const w = Math.max(320, window.innerWidth - (window.innerWidth < 768 ? 88 : 280)); return w >= 680 ? 3 : w >= 400 ? 2 : 1; } return 3; })
408 useRef(cols)
411 useLayoutEffect(() => { const el = containerRef.current; if (!el) return; let rafId = 0; const ro = new ResizeObserver((entries) => { const w = entries[entries.length - 1]?.contentRect.width ?? 0; if (rafId) cancelAnimationFrame(rafId); rafId = requestAnimationFrame(() => { rafId = 0; if (w <= 0) return; const next = colsFor(w); if (next === colsRef.current) return; flipChildren(el, () => flushSync(() => setCols(next)), "size"); }); }); ro.observe(el); return () => { if (rafId) cancelAnimationFrame(rafId); ro.disconnect(); }; }, [])
417 cancelAnimationFrame(rafId)
418 requestAnimationFrame(() => { rafId = 0; if (w <= 0) return; const next = colsFor(w); if (next === colsRef.current) return; flipChildren(el, () => flushSync(() => setCols(next)), "size"); })
428 cancelAnimationFrame(rafId)
434 useRailPin(containerRef, () => { const w = containerRef.current?.getBoundingClientRect().width ?? 0; if (w > 0) setCols(colsFor(w)); })
435 containerRef.current?.getBoundingClientRect()
440 useState(() => !quickShelfHasEntered)
441 useEffect(() => { quickShelfHasEntered = true; }, [])
445 useSpeedDialStore((s) => s.entries)
446 useMyPlaylists()
447 useRecentlyPlayed()
448 useTopTracks("short_term")
449 useTopArtists("short_term")
451 useMemo<QuickItem[]>(() => { const inAppMap = new Map<string, SpeedDialEntry>(); for (const entry of speedDialEntries) { inAppMap.set(entry.id, entry); } interface ScoredCandidate { item: QuickItem; score: number; lastPlayedAt: number; } const candidateMap = new Map<string, ScoredCandidate>(); function addOrUpdateCandidate( item: QuickItem, baseScore: number, lastPlayedAt: number = 0 ) { const inApp = inAppMap.get(item.id); const inAppPlays = inApp?.playCount || 0; const inAppLastPlayed = inApp?.lastPlayedAt || inApp?.timestamp || 0; // In-app plays add heavy rotation weight (+5 pts per play) const totalScore = baseScore + inAppPlays * 5; const effectiveLastPlayed = Math.max(lastPlayedAt, inAppLastPlayed); const existing = candidateMap.get(item.id); if (!existing || totalScore > existing.s
606 Array.from(candidateMap.values()).sort((a, b) => { if (b.score !== a.score) { return b.score - a.score; } return b.lastPlayedAt - a.lastPlayedAt; })
673 items.map((item, i) => ( <QuickActionCard key={item.id} item={item} recentTracks={recentTracks} index={i} animateIn={animateIn} /> ))
695 useState(false)
696 usePlayerStore((s) => s.currentTrack)
697 usePlayerStore((s) => s.isPlaying)
789 [0.4, 1.0, 0.6].map((_, i) => ( <div key={i} className="eq-bar" style={{ flex: 1, height: "100%", borderRadius: 1, background: "var(--color-accent)", ["--eq-dur" as string]: `${0.55 + i * 0.15}s`, }} /> ))
806 track.artists.map((a) => a.name)
814 usePlayerStore((s) => s.setCurrentTrack)
815 useQueueStore((s) => s.playContext)
840 usePlayerStore((s) => s.setCurrentTrack)
841 useQueueStore((s) => s.playContext)
842 useQuery({ queryKey: ["recommendations", "home"], queryFn: () => getRecommendations(undefined, 16), staleTime: 10 * 60_000, gcTime: 24 * 60 * 60_000, placeholderData: (prev) => prev, refetchOnWindowFocus: false, })
850 useCarousel([recs.length])
882 useRecentlyPlayed()
883 usePlayerStore((s) => s.setCurrentTrack)
884 useQueueStore((s) => s.playContext)
886 useCarousel([shown.length])
983 useState<TimeRange>("medium_term")
984 useTopTracks(range)
985 useTopTracks("medium_term")
1011 useState<TimeRange>("medium_term")
1012 useTopArtists(range)
1013 useTopArtists("medium_term")
1039 useNewReleases()
1041 useCarousel([shown.length])
1084 useReflowPulse()
1085 useAuth()
```

## src/pages/Library.tsx

Declarations: TabKey:34, LikedSongsTab:38, AlbumsTab:111, ArtistsTab:155, PodcastsTab:200, Library:225.

```text
39 useLikedSongs(200, 0)
40 useLikedSongsCount()
41 usePlayerStore((s) => s.setCurrentTrack)
42 useQueueStore((s) => s.enqueue)
43 useQueueStore((s) => s.playContext)
44 useToggleLike()
46 useSortTools(tracks, "Find in songs")
47 useMemo(() => tracks.map((t) => t.id), [tracks])
47 tracks.map((t) => t.id)
48 useSavedTrackIds(trackIds)
49 useMemo(() => new Set(savedIds), [savedIds])
71 useSpeedDialStore.getState().recordLikedSongs()
71 useSpeedDialStore.getState()
89 view.map((t, i) => ( <TrackRow key={t.id} track={t} index={i} showAlbum liked={likedSet.has(t.id)} onPlay={() => handlePlay(i)} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })} /> ))
112 useSavedAlbums()
113 useSortTools(albums, "Find in albums")
144 view.map((al, i) => <AlbumCard key={al.id} album={al} index={i} />)
156 useFollowedArtists()
157 useSortTools(artists, "Find in artists")
188 view.map((a, i) => <ArtistCard key={a.id} artist={a} index={i} />)
201 useQuery({ queryKey: ["library", "shows"], queryFn: getSavedShows, staleTime: 300_000, })
218 shows.map((s, i) => ( <MediaTile key={s.id} to={`/show/${s.id}`} imageUrl={s.image_url} title={s.name} subtitle={s.publisher} index={i} /> ))
226 useReflowPulse()
227 useAuth()
228 useSearchParams()
230 (TAB_KEYS as readonly string[]).includes(rawTab ?? "")
```

## src/pages/Playground.tsx

Declarations: Playground:24.

```text
25 useReflowPulse()
26 useState("Hi-Res Studio")
27 useState(true)
28 useState(false)
```

## src/pages/PlaylistPage.tsx

Declarations: PlaylistPage:41, toggleFollow:70, deletePlaylist:84, removeFromPlaylist:132, startAt:166.

```text
43 useReflowPulse()
44 useParams<{ id: string }>()
45 usePlaylist(id)
46 usePlayerStore((s) => s.setCurrentTrack)
47 useQueueStore((s) => s.enqueue)
48 useQueueStore((s) => s.playContext)
49 usePinsStore((s) => Boolean(data?.id && s.pins.some((p) => p.id === data.id)))
49 s.pins.some((p) => p.id === data.id)
50 usePinsStore((s) => s.togglePin)
51 useToggleLike()
52 useAuthStore((s) => s.displayName)
53 useAuthStore((s) => s.userId)
54 useQueryClient()
55 useNavigate()
56 useState(false)
57 useState(false)
58 useState(false)
64 useQuery({ queryKey: ["library", "playlist-followed", id], queryFn: () => isPlaylistFollowed(id!), enabled: !!id && !!data && !isOwner, })
73 qc.setQueryData(key, !following)
76 qc.invalidateQueries({ queryKey: ["library", "playlists"] })
79 qc.setQueryData(key, following)
88 qc.invalidateQueries({ queryKey: ["library", "playlists"] })
98 useContextMenu()
101 useMemo(() => tracks.map((t) => t.id), [tracks])
101 tracks.map((t) => t.id)
102 useSavedTrackIds(trackIds)
103 useMemo(() => new Set(savedIds), [savedIds])
105 useTrackTools(tracks, "Playlist order")
138 qc.invalidateQueries({ queryKey: ["playlist", playlistId] })
139 removeTrackFromPlaylist(playlistId, track.id) .then(() => { refresh(); toast(`Removed from ${playlistName}`, { action: { label: "Undo", onClick: () => { const restore = positions.length ? positions.reduce<Promise<void>>( (p, pos) => p.then(() => addTrackToPlaylist(playlistId, track.id, pos)), Promise.resolve(), ) : addTrackToPlaylist(playlistId, track.id); restore .then(() => { refresh(); toast(`Restored to ${playlistName}`); }) .catch((e) => toast.error(`Couldn't restore track: ${errMsg(e)}`)); }, }, }); }) .catch((e) => toast.error(`Couldn't remove track: ${errMsg(e)}`))
149 Promise.resolve()
172 useSpeedDialStore.getState().recordPlaylist({ id: data.id, name: data.name, image_url: data.image_url })
172 useSpeedDialStore.getState()
248 qc.invalidateQueries({ queryKey: ["playlist", data.id] })
249 qc.invalidateQueries({ queryKey: ["library", "playlists"] })
270 view.map((t, i) => ( <TrackRow key={keys[i]} track={t} index={i} showAlbum liked={likedSet.has(t.id)} onPlay={() => startAt(i)} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })} onRemoveFromPlaylist={canEdit ? removeFromPlaylist : undefined} /> ))
```

## src/pages/Playlists.tsx

Declarations: folderCovers:165, Playlists:243, cardMenu:302.

```text
37 useReflowPulse()
38 useState(false)
39 useQueryClient()
40 useRef<number | null>(null)
44 window.setTimeout(() => { qc.prefetchQuery({ queryKey: ["playlist", playlist.id], queryFn: () => getPlaylist(playlist.id), staleTime: 120_000, }); }, 40)
56 window.clearTimeout(prefetchTimer.current)
179 useState(false)
217 (covers.length >= 4 ? covers.slice(0, 4) : covers.slice(0, 1)).map((c, i) => ( <img key={i} src={coverUrl(c, covers.length >= 4 ? 100 : 200) ?? c} alt="" loading="lazy" decoding="async" style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }} /> ))
244 useReflowPulse()
245 useId()
246 useAuth()
247 useMyPlaylists()
248 usePlaylistFolders()
252 useQuery({ queryKey: ["library", "home-feed"], queryFn: () => getHomeFeed(Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC"), staleTime: 30 * 60_000, retry: false, })
261 madeForYou.some((m) => m.id === it.id)
264 useSearchParams()
272 (level ?? []).filter((it): it is FolderItem => it.kind === "folder")
273 synced.map((p) => [p.id, p])
288 useSyncLibrary()
289 usePinsStore((s) => s.isPinned)
290 usePinsStore((s) => s.togglePin)
291 useContextMenu()
292 useRef(false)
295 useEffect(() => { if (loggedIn && !isLoading && synced.length === 0 && !isPending && !autoSynced.current) { autoSynced.current = true; sync(); } }, [loggedIn, isLoading, synced.length, isPending, sync])
328 openFolder.trail.map((f) => ( <span key={f.id} style={{ display: "inline-flex", alignItems: "center", gap: 8 }}> <ChevronRight size={18} style={{ color: "var(--color-text-dim)" }} /> <Link to={`/playlists?folder=${encodeURIComponent(f.id)}`} style={{ color: "var(--color-text-dim)", textDecoration: "none" }}>{f.name}</Link> </span> ))
385 folders.map((f, i) => ( <FolderCard key={`folder-${f.id}`} folder={f} index={i} /> ))
388 playlists.map((p: PlaylistSummary, i: number) => ( <PlaylistCard key={p.id} playlist={p} index={i} onContextMenu={openMenu(cardMenu(p))} /> ))
```

## src/pages/ProfilePage.tsx

Declarations: Tab:35, PeopleGrid:37, FollowList:55, ProfilePage:69, toggleFollow:113.

```text
40 people.map((p, i) => ( <MediaTile key={`${p.kind}-${p.id}`} to={p.kind === "artist" ? `/artist/${p.id}` : `/user/${p.id}`} imageUrl={p.image_url} title={p.name} subtitle={p.kind === "artist" ? "Artist" : p.followers != null ? `${p.followers.toLocaleString()} followers` : "Profile"} index={i} round /> ))
56 useQuery({ queryKey: ["profile", which, userId], queryFn: () => (which === "followers" ? getUserFollowers(userId) : getUserFollowing(userId)), staleTime: 300_000, })
70 useReflowPulse()
71 useParams<{ id: string }>()
72 useAuthStore((s) => s.userId)
75 useState<Tab>("overview")
76 useQueryClient()
77 useContextMenu()
79 useQuery({ queryKey: ["profile", "view", profileId ?? "me"], queryFn: () => getUserProfile(profileId), staleTime: 300_000, })
84 useQuery({ queryKey: ["profile", "is-followed", profileId], queryFn: () => isUserFollowed(profileId!), enabled: !!profileId, })
91 useTopArtists("short_term")
92 useTopTracks("short_term")
93 useMemo(() => (isMe ? topTracks.slice(0, 5) : []), [isMe, topTracks])
94 useMemo(() => shownTop.map((t) => t.id), [shownTop])
94 shownTop.map((t) => t.id)
95 useSavedTrackIds(shownTrackIds)
96 useMemo(() => new Set(savedIds), [savedIds])
97 useToggleLike()
98 usePlayerStore((s) => s.setCurrentTrack)
99 useQueueStore((s) => s.playContext)
100 useQueueStore((s) => s.enqueue)
115 qc.setQueryData(key, !following)
118 qc.invalidateQueries({ queryKey: ["profile", "view"] })
120 qc.setQueryData(key, following)
141 [ stat(data.total_public_playlists ?? data.playlists.length, "public playlist", "public playlists"), stat(data.followers, "follower", "followers", "followers"), stat(data.following, "following", "following", "following"), ].filter(Boolean)
155 stats.map((s, i) => ( <span key={i} style={{ display: "inline-flex", gap: 6, alignItems: "center" }}> {i > 0 && <span aria-hidden>·</span>} {s} </span> ))
214 shownTop.map((t, i) => ( <TrackRow key={t.id} track={t} index={i} showAlbum liked={liked.has(t.id)} onPlay={() => { const start = playContext(topTracks, i, "profile-top"); if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); } }} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: liked.has(track.id) })} /> ))
```

## src/pages/Search.tsx

Declarations: Category:45, TopResult:64, getTopResult:69, topResultImage:101, handleClick:152, SearchEpisodes:366, PlaylistGrid:396, Search:411, handlePlayTopResult:453, playFromSearch:522.

```text
55 CATEGORIES.map((c) => [CATEGORY_LABEL[c], c])
74 data.artists.find((a) => a.name.toLowerCase() === q)
77 data.tracks.find((t) => t.name.toLowerCase() === q)
80 data.albums.find((al) => al.name.toLowerCase() === q)
84 data.artists.find((a) => a.name.toLowerCase().startsWith(q))
87 data.tracks.find((t) => t.name.toLowerCase().startsWith(q))
90 data.albums.find((al) => al.name.toLowerCase().startsWith(q))
120 useState(false)
121 useState(false)
122 useNavigate()
134 result.item.artists.map((a) => a.name)
367 usePlayEpisodes(episodes, "search-episodes")
368 usePlayerStore((s) => s.currentId)
369 usePlayerStore((s) => s.isPlaying)
372 episodes.map((ep, i) => { const active = currentId === `${EPISODE_PREFIX}${ep.id}`; return <EpisodeRow key={ep.id} episode={ep} active={active} playing={active && isPlaying} onPlay={() => play(i)} showShow />; })
412 useReflowPulse()
413 useAuth()
414 useSearchParams()
416 useState<Category>("all")
417 usePlayerStore((s) => s.setCurrentTrack)
418 usePlayerStore((s) => s.isPlaying)
419 useQueueStore((s) => s.enqueue)
420 useQueueStore((s) => s.playContext)
421 useUIStore((s) => s.setPageTint)
422 useToggleLike()
424 useSearch(query)
427 useState(false)
428 useMemo(() => data?.tracks.map((t) => t.id) ?? [], [data?.tracks])
428 data?.tracks.map((t) => t.id)
429 useSavedTrackIds(trackIds)
430 useMemo(() => new Set(savedIds), [savedIds])
432 useMemo(() => { if (!data) return null; return getTopResult(data, query); }, [data, query])
440 useEffect(() => { setPageTint(tintUrl); return () => setPageTint(null); }, [tintUrl, setPageTint])
445 usePlayerStore((s) => { if (!s.isPlaying || !topResult || !s.currentTrack) return false; if (topResult.type === "track") return s.currentTrack.id === topResult.item.id; if (topResult.type === "artist") return s.currentTrack.artists.some((a) => a.id === topResult.item.id); if (topResult.type === "album") return s.currentTrack.album?.id === topResult.item.id; return false; })
448 s.currentTrack.artists.some((a) => a.id === topResult.item.id)
466 usePlayerStore.getState()
475 playTrack(trackToPlay.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(console.error)
475 usePlayerStore.getState().setPlaying(true)
475 usePlayerStore.getState()
493 playTrack(start.id).then(() => usePlayerStore.getState().setPlaying(true)).catch(console.error)
493 usePlayerStore.getState().setPlaying(true)
493 usePlayerStore.getState()
532 tracks.map((t, i) => ( <TrackRow key={t.id} track={t} index={i} showAlbum liked={likedSet.has(t.id)} onPlay={() => playFromSearch(i)} onQueue={(track) => enqueue(track)} onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })} /> ))
598 CATEGORIES.map((c) => CATEGORY_LABEL[c])
739 data.artists.map((a, i) => <ArtistCard key={a.id} artist={a} index={i} />)
749 data.albums.map((al, i) => <AlbumCard key={al.id} album={al} index={i} />)
761 data.shows!.map((s, i) => ( <MediaTile key={s.id} to={`/show/${s.id}`} imageUrl={s.image_url} title={s.name} subtitle={s.publisher} index={i} /> ))
781 data.playlists.map((pl, i) => <PlaylistResultCard key={pl.id} playlist={pl} index={i} />)
```

## src/pages/Settings.tsx

Declarations: StatusBadge:61, Field:97, PrimaryBtn:123, GhostBtn:144, RevertBtn:181, SettingRow:214, Segmented:279, ZoomStepper:346, AppearanceCard:401, backendOptions:503, PlaybackCard:521, GeneralCard:642, toggleDiscord:650, FeaturesCard:686, VisualCard:716, choose:722, LastfmCard:840, saveAndConnect:858, disconnect:885, forget:889, Card:1062, Divider:1101, Settings:1105, autoValidate:1117.

```text
297 options.map((o) => [o.label, o.value])
298 options.find((o) => o.value === value)
301 options.filter((o) => o.disabled).map((o) => o.label)
301 options.filter((o) => o.disabled)
303 options.filter((o) => o.info).map((o) => [o.label, o.info])
303 options.filter((o) => o.info)
308 options.map((o) => o.label)
347 usePrefsStore((s) => s.uiZoom)
402 useThemeStore((s) => s.source)
403 useThemeStore((s) => s.setSource)
404 useThemeStore((s) => s.albumColors)
405 useThemeStore((s) => s.setAlbumColors)
406 usePrefsStore((s) => s.ambientMotion)
407 usePrefsStore((s) => s.sidebarMode)
408 usePrefsStore((s) => s.setSidebarMode)
409 usePrefsStore((s) => s.setAmbientMotion)
410 usePrefsStore((s) => s.showCanvas)
411 usePrefsStore((s) => s.setShowCanvas)
522 usePrefsStore((s) => s.audioQuality)
523 usePrefsStore((s) => s.setAudioQuality)
524 usePrefsStore((s) => s.audioCacheLimitMb)
525 usePrefsStore((s) => s.setAudioCacheLimitMb)
530 useQueryClient()
531 usePlaybackBackend()
535 useState<PlaybackBackend | null>(null)
538 useRef(0)
542 setPlaybackBackend(mode) .then(() => qcBackend.invalidateQueries({ queryKey: ["playback-backend"] })) .catch((e) => { if (seq === backendReq.current) toast.error(errMsg(e)); }) .finally(() => { if (seq === backendReq.current) setPendingBackend(null); })
542 setPlaybackBackend(mode) .then(() => qcBackend.invalidateQueries({ queryKey: ["playback-backend"] })) .catch((e) => { if (seq === backendReq.current) toast.error(errMsg(e)); })
543 qcBackend.invalidateQueries({ queryKey: ["playback-backend"] })
643 usePrefsStore((s) => s.notifyOnTrack)
644 usePrefsStore((s) => s.setNotifyOnTrack)
645 usePrefsStore((s) => s.promptOnClose)
646 usePrefsStore((s) => s.setPromptOnClose)
647 usePrefsStore((s) => s.discordPresence)
648 usePrefsStore((s) => s.setDiscordPresence)
687 usePrefsStore((s) => s.showStats)
688 usePrefsStore((s) => s.setShowStats)
689 usePrefsStore((s) => s.showFriends)
690 usePrefsStore((s) => s.setShowFriends)
701 usePlayerStore.getState()
701 usePlayerStore.getState().toggleFriends()
701 usePlayerStore.getState()
717 useUIStore((s) => s.windowEffect)
718 useUIStore((s) => s.setWindowEffect)
719 useUIStore((s) => s.materialTransparency)
720 useUIStore((s) => s.setMaterialTransparency)
841 useQueryClient()
842 useQuery({ queryKey: ["lastfm", "status"], queryFn: lastfmStatus, })
847 useState("")
848 useState("")
849 useState<"idle" | "saving" | "connecting">("idle")
850 useState<string | null>(null)
853 useState(false)
856 qc.invalidateQueries({ queryKey: ["lastfm", "status"] })
1106 useReflowPulse()
1107 useQueryClient()
1109 useCredentialsStore()
1111 useState("")
1112 useState("")
1113 useState(false)
1114 useState<string | null>(null)
1115 useState(true)
1127 useQuery({ queryKey: ["credentials"], queryFn: async () => { const creds = await getCredentials(); setFromCredentials(creds); if (creds && creds.is_custom) { setClientId(creds.client_id); if (creds.has_secret) autoValidate(); } else { setClientId(""); } return creds; }, })
1142 useMutation({ mutationFn: () => saveCredentials(clientId, clientSecret || undefined), onSuccess: () => { setClientSecret(""); setValidationError(null); qc.invalidateQueries({ queryKey: ["credentials"] }); autoValidate(); }, })
1147 qc.invalidateQueries({ queryKey: ["credentials"] })
1152 useMutation({ mutationFn: clearCredentials, onSuccess: () => { setClientId(""); setClientSecret(""); setValidationError(null); clear(); qc.invalidateQueries({ queryKey: ["credentials"] }); }, })
1159 qc.invalidateQueries({ queryKey: ["credentials"] })
```

## src/pages/ShowPage.tsx

Declarations: Order:28, ShowPage:35, toggleSave:93, loadMore:105, startFrom:119.

```text
36 useReflowPulse()
37 useParams<{ id: string }>()
38 useQueryClient()
39 useContextMenu()
40 useQuery({ queryKey: ["show", id], queryFn: () => getShow(id!), enabled: !!id, staleTime: 120_000, })
46 useQuery({ queryKey: ["library", "show-saved", id], queryFn: () => isShowSaved(id!), enabled: !!id, })
53 useState<EpisodeItem[]>([])
54 useState<number | null>(null)
55 useState(false)
56 useEffect(() => { setMore([]); setNextOffset(data?.next_offset ?? null); }, [data])
61 useState("")
62 useState<Order>("newest")
63 useMemo(() => [...(data?.episodes ?? []), ...more], [data?.episodes, more])
64 useMemo(() => { const q = query.trim().toLowerCase(); let arr = q ? all.filter((e) => e.name.toLowerCase().includes(q) || (e.description ?? "").toLowerCase().includes(q)) : all; if (order === "oldest") arr = [...arr].reverse(); if (order === "unplayed") arr = [...arr].sort((a, b) => Number(a.fully_played) - Number(b.fully_played)); return arr; }, [all, query, order])
66 all.filter((e) => e.name.toLowerCase().includes(q) || (e.description ?? "").toLowerCase().includes(q))
66 e.name.toLowerCase().includes(q)
66 (e.description ?? "").toLowerCase().includes(q)
68 [...arr].sort((a, b) => Number(a.fully_played) - Number(b.fully_played))
72 usePlayEpisodes(view, `show-${id}`)
73 usePlayerStore((s) => s.currentId)
74 usePlayerStore((s) => s.isPlaying)
77 useMemo(() => { const firstUnplayed = all.findIndex((e) => !e.fully_played && e.is_playable); return all.slice(Math.max(0, firstUnplayed)).map(episodeToTrack); }, [all])
79 all.slice(Math.max(0, firstUnplayed)).map(episodeToTrack)
95 qc.setQueryData(key, !saved)
98 qc.invalidateQueries({ queryKey: ["library", "shows"] })
100 qc.setQueryData(key, saved)
120 all.find((e) => `${EPISODE_PREFIX}${e.id}` === track.id)
125 [ data.total_episodes != null ? `${data.total_episodes.toLocaleString()} episodes` : null, data.languages.length > 0 ? data.languages.slice(0, 2).join(", ").toUpperCase() : null, data.explicit ? "Explicit" : null, ].filter(Boolean)
140 meta.map((m) => ( <span key={m} className="tnum" style={{ textTransform: "uppercase", letterSpacing: "0.05em", fontWeight: 600 }}> · {m}</span> ))
204 view.map((ep, i) => { const active = currentId === `${EPISODE_PREFIX}${ep.id}`; return <EpisodeRow key={ep.id} episode={ep} index={i} active={active} playing={active && isPlaying} onPlay={() => play(i)} />; })
```

## src/pages/StatsPage.tsx

Declarations: Tile:35, Heatmap:47, Genres:102, RankedTracks:123, play:128, StatTiles:178, StatsPage:197.

```text
48 useMemo(() => heatMax(grid), [grid])
49 useState<{ day: number; hour: number } | null>(null)
50 useMemo(() => peakSlot(grid), [grid])
62 ROW_ORDER.map((day) => ( <div key={day} role="row" style={{ display: "contents" }}> <span className="t-caption" style={{ fontSize: 11, color: "var(--color-text-dim)", alignSelf: "center" }}>{DAYS[day]}</span> {grid[day].map((plays, hour) => { const level = heatLevel(plays, max); const on = hover?.day === day && hover?.hour === hour; return ( <div key={hour} role="gridcell" aria-label={`${DAYS[day]} ${hourLabel(hour)}: ${plays} ${plays === 1 ? "play" : "plays"}`} title={`${DAYS[day]} ${hourLabel(hour)} · ${plays} ${plays === 1 ? "play" : "plays"}`} onMouseEnter={() => setHover({ day, hour })} onMouseLeave={() => setHover(null)} style={{ aspectRatio: "1 / 1", borderRadius: 4, background: level > 0 ? `color-mix(in srgb, var(--color-accent) ${Math.round(level * 100)}%, transparent)` : "rgba(25
65 grid[day].map((plays, hour) => { const level = heatLevel(plays, max); const on = hover?.day === day && hover?.hour === hour; return ( <div key={hour} role="gridcell" aria-label={`${DAYS[day]} ${hourLabel(hour)}: ${plays} ${plays === 1 ? "play" : "plays"}`} title={`${DAYS[day]} ${hourLabel(hour)} · ${plays} ${plays === 1 ? "play" : "plays"}`} onMouseEnter={() => setHover({ day, hour })} onMouseLeave={() => setHover(null)} style={{ aspectRatio: "1 / 1", borderRadius: 4, background: level > 0 ? `color-mix(in srgb, var(--color-accent) ${Math.round(level * 100)}%, transparent)` : "rgba(255,255,255,0.05)", outline: on ? "2px solid var(--color-text-hi)" : "none", outlineOffset: 1, }} /> ); })
109 genres.map((g) => ( <div key={g.genre} style={{ display: "grid", gridTemplateColumns: "minmax(90px, 160px) 1fr 44px", alignItems: "center", gap: 10 }} title={`${g.genre}: ${Math.round(g.share * 100)}%`}> <span style={{ fontSize: 13, color: "var(--color-text)", textTransform: "capitalize", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{g.genre}</span> <div style={{ height: 10, borderRadius: 4, background: "rgba(255,255,255,0.05)", overflow: "hidden" }}> <div style={{ width: `${(g.share / max) * 100}%`, height: "100%", borderRadius: 4, background: "var(--color-accent)" }} /> </div> <span className="tnum" style={{ fontSize: 12.5, color: "var(--color-text-dim)", textAlign: "right" }}>{Math.round(g.share * 100)}%</span> </div> ))
124 usePlayerStore((s) => s.setCurrentTrack)
125 useQueueStore((s) => s.playContext)
127 rows.map((r) => r.item)
140 rows.slice(0, 10).map((r, i) => ( <li key={r.item.id}> <button type="button" className="q-row focus-ring" data-clickable onClick={() => play(i)} style={{ all: "unset", boxSizing: "border-box", width: "100%", display: "flex", alignItems: "center", gap: 12, // the same geometry as TrackRow, so every track list in the app lines up padding: "8px 12px", margin: "1.5px 0", borderRadius: 8, cursor: "pointer", background: i % 2 === 0 ? "rgba(255,255,255,0.032)" : "transparent", }} > <span className="tnum" style={{ width: 28, flexShrink: 0, textAlign: "center", fontSize: 13, color: "var(--color-text-muted)" }}>{i + 1}</span> <CoverArt url={r.item.album?.image_url} alt="" size={38} style={{ width: 38, height: 38, flexShrink: 0 }} /> <span style={{ flex: 1, minWidth: 0, display: "flex", flexDirection
159 r.item.artists.map((a) => a.name)
179 useRef<HTMLDivElement>(null)
180 useEvenColumns(ref, 6, TILE_MIN, TILE_GAP)
183 listenTime(data.biggest_day[1])
187 listenTime(data.total_ms)
198 useReflowPulse()
199 useState<StatsRange>("month")
200 usePrefsStore((s) => s.showStats)
201 useQuery({ queryKey: ["stats", range], queryFn: () => getListeningStats(range), staleTime: 60_000, enabled, })
216 RANGES.map((r) => r.label)
217 RANGES.find((r) => r.value === range)
218 RANGES.find((r) => r.label === label)
```

## src/store/addToPlaylist.store.ts

Declarations: AddToPlaylistStore:4.

```text
```

## src/store/auth.store.ts

Declarations: AuthStore:5.

```text
```

## src/store/credentials.store.ts

Declarations: ConnectionStatus:4, CredentialsState:11.

```text
```

## src/store/credits.store.ts

Declarations: CreditsStore:5.

```text
```

## src/store/jam.store.ts

Declarations: JamStore:9, JamRole:52, jamRole:54.

```text
55 useJamStore.getState()
60 useJamStore((s) => (s.session ? (s.session.is_host ? "host" : "guest") : null))
```

## src/store/pins.store.ts

Declarations: PinnedItem:5, PinsStore:12.

```text
25 get().pins.some((p) => p.id === id)
29 s.pins.some((p) => p.id === item.id)
30 s.pins.filter((p) => p.id !== item.id)
35 s.pins.filter((p) => p.id !== id)
```

## src/store/player.store.ts

Declarations: PlayerStore:7.

```text
185 ["off", "context", "track"].includes(state.repeat_state)
```

## src/store/prefs.store.ts

Declarations: SidebarMode:15, PrefsStore:17.

```text
125 getAudioQuality() .then((q) => { if (q === "96" || q === "160" || q === "320") { usePrefsStore.setState({ audioQuality: q }); } }) .catch(() => {})
128 usePrefsStore.setState({ audioQuality: q })
133 getAudioCacheLimit() .then((mb) => { if (typeof mb === "number" && mb > 0) { usePrefsStore.setState({ audioCacheLimitMb: mb }); } }) .catch(() => {})
136 usePrefsStore.setState({ audioCacheLimitMb: mb })
```

## src/store/queue.store.ts

Declarations: Repeat:17, shuffled:19, QueueStore:28.

```text
88 usePlayerStore.getState()
89 remoteAddToQueue(track.id) .then(() => toast(`Queued on ${usePlayerStore.getState().activeDevice?.name ?? "device"}`)) .catch(() => toast.error("Couldn't queue on the remote device"))
90 usePlayerStore.getState()
137 s.queue.map((t) => t.id)
138 tracks.filter((t) => !existing.has(t.id))
143 s.queue.filter((_, i) => i !== idx)
164 usePlayerStore.getState()
169 usePlayerStore.getState().setRemoteShuffle(!next)
169 usePlayerStore.getState()
181 usePlayerStore.getState()
182 s.history.map((t) => t.id)
184 s.contextTracks.filter((t) => !played.has(t.id))
196 usePlayerStore.getState()
202 usePlayerStore.getState().setRemoteRepeat(prev)
202 usePlayerStore.getState()
277 contextTracks.filter((t) => t.id !== current?.id)
281 history.filter((t) => t.id !== current?.id)
```

## src/store/speedDial.store.ts

Declarations: SpeedDialType:6, SpeedDialEntry:8, SpeedDialStore:25.

```text
54 s.entries.find((e) => e.id === entry.id)
57 s.entries.filter((e) => e.id !== entry.id)
65 [updated, ...filtered].sort((a, b) => { const pa = a.playCount ?? 1; const pb = b.playCount ?? 1; if (pb !== pa) return pb - pa; return (b.lastPlayedAt ?? b.timestamp ?? 0) - (a.lastPlayedAt ?? a.timestamp ?? 0); })
87 track.artists?.find((a) => a.id === artistId)
113 track.artists?.map((a) => a.name)
```

## src/store/theme.store.ts

Declarations: ThemeSource:6, ThemeStore:8.

```text
```

## src/store/toast.store.ts

Declarations: ToastKind:5, ToastAction:7, Toast:12, ToastOptions:19, ToastStore:24.

```text
41 setTimeout(() => set((s) => ({ toasts: s.toasts.filter((x) => x.id !== id) })), ttl)
41 s.toasts.filter((x) => x.id !== id)
44 s.toasts.filter((t) => t.id !== id)
49 useToastStore.getState().push(text, opts)
49 useToastStore.getState()
51 useToastStore.getState().push(text, { kind: "error" })
51 useToastStore.getState()
52 useToastStore.getState().push(text, { kind: "info" })
52 useToastStore.getState()
```

## src/store/ui.store.ts

Declarations: UIState:6.

```text
```

## src/store/updater.store.ts

Declarations: UpdateStage:9, UpdaterState:12.

```text
```

## src/store/ytMatch.store.ts

Declarations: YtMatchStore:3.

```text
```

## src/types/ipc.ts

Declarations: AppError:3, AuthStatus:8.

```text
```

## src/types/library.ts

Declarations: PlaylistSummary:1, LibraryStatus:10, SyncResult:15, TimeRange:26.

```text
```

## src/types/podcast.ts

Declarations: ShowItem:1, EpisodeItem:12, ShowDetail:28, EpisodePage:35, TranscriptLine:41, Transcript:48, EpisodeMedia:54.

```text
```

## src/types/spotify.ts

Declarations: ArtistItem:3, AlbumItem:10, TrackItem:20, PlaylistCard:33, SearchResults:41, Profile:50, ArtistDetail:63, AlbumDetail:75, TrackDetail:88, PlaylistDetail:99.

```text
```

## src/utils/episode.test.ts

Declarations: module constants/functions.

```text
```

## src/utils/episode.ts

Declarations: isEpisodeId:10, episodeToTrack:14, episodeLength:34, episodeProgress:42.

```text
```

## src/utils/fmt.test.ts

Declarations: module constants/functions.

```text
```

## src/utils/fmt.ts

Declarations: fmtMs:2, releaseYear:13, releaseDateMs:18, msUntilRelease:26, isUpcoming:31, fmtCountdown:36, cleanDescription:52.

```text
```

## src/utils/listenTracker.ts

Declarations: trackStarted:11, trackCompleted:29, trackSkipped:37.

```text
```

## src/utils/palette.ts

Declarations: rgbToHsl:5, extractPalette:21.

```text
33 ctx.drawImage(img, 0, 0, S, S)
34 ctx.getImageData(0, 0, S, S)
49 [...buckets.values()] .map((c) => ({ r: c.r / c.n, g: c.g / c.n, b: c.b / c.n, score: c.n * (0.4 + c.sat / c.n), // frequency x saturation })) .sort((a, b) => b.score - a.score) .slice(0, 5) .map((c) => `rgb(${Math.round(c.r)}, ${Math.round(c.g)}, ${Math.round(c.b)})`)
49 [...buckets.values()] .map((c) => ({ r: c.r / c.n, g: c.g / c.n, b: c.b / c.n, score: c.n * (0.4 + c.sat / c.n), // frequency x saturation })) .sort((a, b) => b.score - a.score)
49 [...buckets.values()] .map((c) => ({ r: c.r / c.n, g: c.g / c.n, b: c.b / c.n, score: c.n * (0.4 + c.sat / c.n), // frequency x saturation }))
```

## src/utils/radio.ts

Declarations: replenishQueue:14, playList:70, playTrackRadio:80, playStation:95, startRadio:109.

```text
17 useQueueStore.getState()
22 usePlayerStore.getState()
37 [...q.history.slice(-30).map((t) => t.id), seed.id].filter((id) => !isEpisodeId(id))
37 q.history.slice(-30).map((t) => t.id)
39 auto.filter((t) => !exclude.has(t.id))
41 useQueueStore.getState().queue.map((t) => t.id)
41 useQueueStore.getState()
42 useQueueStore.getState().appendTracks(fresh.filter((t) => !existing.has(t.id)).slice(0, 25))
42 useQueueStore.getState()
42 fresh.filter((t) => !existing.has(t.id))
56 useQueueStore.getState().queue.map((t) => t.id)
56 useQueueStore.getState()
57 recs.filter((t) => !existing.has(t.id) && t.id !== seed?.id)
59 useQueueStore.getState().appendTracks(fresh)
59 useQueueStore.getState()
71 useQueueStore.getState().playContext(tracks, 0, contextId)
71 useQueueStore.getState()
73 usePlayerStore.getState().setCurrentTrack(start)
73 usePlayerStore.getState()
83 radio.tracks.filter((t) => t.id !== seed.id)
111 useQueueStore.getState()
116 q.history.slice(-50).map((t) => t.id)
117 (radio?.tracks ?? []).filter((t) => t.id !== seed.id && !played.has(t.id))
136 useQueueStore.getState().playContext(recs, 0, "radio")
136 useQueueStore.getState()
138 usePlayerStore.getState().setCurrentTrack(start)
138 usePlayerStore.getState()
146 useQueueStore.getState().advance(seed)
146 useQueueStore.getState()
148 usePlayerStore.getState().setCurrentTrack(next)
148 usePlayerStore.getState()
157 useQueueStore.getState()
159 useQueueStore.getState().advance(seed)
159 useQueueStore.getState()
161 usePlayerStore.getState().setCurrentTrack(next)
161 usePlayerStore.getState()
```

## src/utils/romanize.ts

Declarations: Script:2, loadPinyin:5, loadWanakana:11, countMatches:25, detectLyricScript:30, canRomanize:45, scriptLabel:49, decomposeHangul:68, romanizeKorean:74, romanizeCyrillic:113, romanizeJapanese:126, romanizeLines:133.

```text
37 (["hangul", "han", "cyrillic"] as Script[]) .map((s) => ({ s, n: counts[s] ?? 0 })) .sort((a, b) => b.n - a.n)
37 (["hangul", "han", "cyrillic"] as Script[]) .map((s) => ({ s, n: counts[s] ?? 0 }))
137 texts.map((t) => pinyin(t, { toneType: "symbol", type: "string", nonZh: "consecutive" }).trim())
140 texts.map(romanizeKorean)
142 texts.map(romanizeCyrillic)
145 texts.map((t) => (t ? romanizeJapanese(t, toRomaji) : ""))
```

## src/utils/stats.test.ts

Declarations: module constants/functions.

```text
6 expect(listenTime(0)).toBe("0 min")
6 listenTime(0)
7 expect(listenTime(42 * 60000)).toBe("42 min")
7 listenTime(42 * 60000)
8 expect(listenTime(125 * 60000)).toBe("2 hr 5 min")
8 listenTime(125 * 60000)
9 expect(listenTime(120 * 60000)).toBe("2 hr")
9 listenTime(120 * 60000)
```

## src/utils/stats.ts

Declarations: listenTime:4, heatLevel:14, heatMax:19, peakSlot:24, hourLabel:34.

```text
```

## src/vite-env.d.ts

Declarations: module constants/functions.

```text
```

## vite.config.ts

Declarations: module constants/functions.

```text
```
