# Complete frontend interaction inventory

Each JSX event-bearing element is listed with its source line, labels, values and handler expressions. This includes buttons, menus, forms, pointers, keyboard handlers and drag interactions. Programmatic/global event listeners are covered by system_map.md and file review notes.

## src/App.tsx

No JSX event-bearing elements.

## src/api/auth.ts

No JSX event-bearing elements.

## src/api/connect.ts

No JSX event-bearing elements.

## src/api/credentials.ts

No JSX event-bearing elements.

## src/api/internal.ts

No JSX event-bearing elements.

## src/api/lastfm.ts

No JSX event-bearing elements.

## src/api/library.ts

No JSX event-bearing elements.

## src/api/lyrics.test.ts

No JSX event-bearing elements.

## src/api/lyrics.ts

No JSX event-bearing elements.

## src/api/media.ts

No JSX event-bearing elements.

## src/api/playback.ts

No JSX event-bearing elements.

## src/api/podcasts.ts

No JSX event-bearing elements.

## src/api/profile.ts

No JSX event-bearing elements.

## src/api/settings.ts

No JSX event-bearing elements.

## src/api/share.ts

No JSX event-bearing elements.

## src/api/social.ts

No JSX event-bearing elements.

## src/api/spotify.ts

No JSX event-bearing elements.

## src/api/stats.ts

No JSX event-bearing elements.

## src/api/theme.ts

No JSX event-bearing elements.

## src/api/window.ts

No JSX event-bearing elements.

## src/components/ErrorBoundary.tsx

- Line 55, `button`. className="pressable"

  `onClick={() => { this.setState({ error: null }); window.location.reload(); }}`

## src/components/ThemeEngine.tsx

No JSX event-bearing elements.

## src/components/UpdatePrompt.tsx

- Line 27, `Modal`. 

  `onClose={() => { if (stage === "available" || stage === "error") dismiss(); }}`

- Line 88, `button`. className="btn-pill"

  `onClick={dismiss}`

- Line 91, `button`. className="btn-primary"

  `onClick={startDownload}`

- Line 149, `button`. className="btn-primary"

  `onClick={restartApp}`

- Line 167, `button`. className="btn-pill"

  `onClick={dismiss}`

## src/components/auth/ProfileCard.tsx

No JSX event-bearing elements.

## src/components/layout/DevicesPopover.tsx

- Line 192, `button`. type="button"; title="Refresh"; aria-label="Refresh devices"; className="btn-icon"

  `onClick={handleRefresh}`

- Line 207, `button`. type="button"; title="Close"; aria-label="Close devices"; className="btn-icon"

  `onClick={() => close(true)}`

- Line 256, `button`. type="button"; className="dev-row"; disabled={isTransferring || isActive}

  `onClick={() => { if (device.id && !isActive && !isTransferring) { transfer(device.id); } }}`

## src/components/layout/FriendsPanel.tsx

- Line 129, `button`. type="button"; aria-label={`Play ${f.track_name}`}; className="fr-art focus-ring"

  `onClick={play}`

- Line 173, `button`. aria-label="Refresh friend activity"; className="btn-icon"; disabled={isFetching}

  `onClick={() => refetch()}`

- Line 180, `button`. aria-label="Close friend activity"; className="btn-icon"

  `onClick={toggleFriends}`

## src/components/layout/Immersive.tsx

- Line 252, `motion.video`. 

  `onError={() => { if (cors) setCors(false); }}`

- Line 372, `div`. className="lyr-row"

  `onClick={() => seekTo(ri)}`

- Line 464, `ReturnPill`. 

  `onClick={recenter}`

- Line 519, `button`. className="row-btn"

  `onClick={() => jump(i)}`

- Line 572, `motion.div`. 

  `onAnimationComplete={() => { if (usePlayerStore.getState().immersiveOpen) useUIStore.getState().setImmersiveCovered(true); }}`

- Line 595, `PodcastStage`. 

  `onVideo={setVideoEl}`

- Line 597, `Sleeve`. 

  `onVideo={setVideoEl}`

- Line 662, `PanelTab`. 

  `onClick={() => setPanel("lyrics")}`

- Line 663, `PanelTab`. 

  `onClick={() => setPanel("queue")}`

- Line 664, `PanelTab`. 

  `onClick={() => setPanel("credits")}`

- Line 666, `button`. className="imm-close"; title="Exit immersive view (Esc)"; aria-label="Exit immersive view"

  `onClick={() => setOpen(false)}`

- Line 699, `CreditsList`. 

  `onNavigate={() => setOpen(false)}`

- Line 738, `button`. className={active ? "lyr-tab lyr-tab-on" : "lyr-tab"}

  `onClick={onClick}`

## src/components/layout/JamCard.tsx

- Line 126, `button`. type="button"; className="focus-ring"

  `onClick={() => setShowPeople((v) => !v)}`

- Line 161, `button`. type="button"; className="btn-icon"; aria-label={`Remove ${m.name} from the Jam`}; disabled={busy}

  `onClick={() => run(() => kickJamMember(jam.session_id, m.id), 'Couldn't remove ${m.name}')}`

- Line 177, `input`. type="checkbox"; disabled={busy}

  `onChange={(e) => run(() => setJamQueueOnly(!e.target.checked), "Couldn't change the Jam").then((next) => { if (next) adopt(next); }) }`

- Line 201, `motion.button`. type="button"; className="btn-primary"

  `onClick={() => copy(jam.join_url!)}`

- Line 212, `button`. type="button"; className="btn-pill"; disabled={busy}

  `onClick={() => run(async () => { await leaveJam(jam.session_id, jam.is_host); await leftJam(jam.is_host); qc.setQueryData(["jam"], null); }, jam.is_host ? "Couldn't end the Jam" : "Couldn't leave the Jam") }`

- Line 248, `motion.form`. 

  `onSubmit={(e) => { e.preventDefault(); if (!link.trim()) return; run(() => joinJam(link), "Couldn't join the Jam").then((j) => { if (j === undefined) return; setJoining(false); setLink(""); adopt(j); if (j) { toast('Joined ${hostOf(j)?.name ?? "the"}${hostOf(j) ? "'s" : ""} Jam'); // social-connect hands the jam's playback over right after // the join; pick it up once it has landed setTimeout(() => refreshConnectState(true), 1500); } }); }}`

- Line 272, `input`. value={link}; placeholder="Paste invite link"; aria-label="Jam invite link"; className="focus-ring"

  `onChange={(e) => setLink(e.target.value)}`

  `onKeyDown={(e) => e.key === "Escape" && setJoining(false)}`

- Line 295, `motion.button`. type="button"; className="btn-primary"; disabled={busy}

  `onClick={() => run(async () => { // the jam starts with what's playing and what's queued here await seedJamFromLocal().catch(() => {}); return startJam(); }, "Couldn't start a Jam").then((j) => { if (!j) return; adopt(j); refreshConnectState(true); if (j.join_url) copy(j.join_url); }) }`

- Line 317, `button`. type="button"; className="btn-pill"

  `onClick={() => setJoining(true)}`

## src/components/layout/Layout.tsx

No JSX event-bearing elements.

## src/components/layout/LyricReturnPill.tsx

- Line 14, `motion.button`. type="button"; className="lyr-return focus-ring glass-solid-fallback"

  `onClick={onClick}`

## src/components/layout/LyricsPanel.tsx

- Line 243, `Pill`. 

  `onClick={() => setShowRoman(!showRoman)}`

- Line 254, `Pill`. 

  `onClick={() => setShowTranslation(!showTranslation)}`

- Line 270, `Pill`. 

  `onClick={() => setPron((v) => !v)}`

- Line 309, `RetryBtn`. 

  `onClick={() => refetch()}`

- Line 329, `div`. className="lyr-prow"

  `onClick={() => seekTo(ri)}`

- Line 447, `ReturnPill`. 

  `onClick={recenter}`

- Line 482, `motion.button`. className="focus-ring"

  `onClick={onClick}`

- Line 554, `button`. className="pressable"; disabled={busy}

  `onClick={onClick}`

## src/components/layout/PlayerBar.tsx

- Line 50, `motion.button`. title={title}; aria-label={ariaLabel}; disabled={disabled}; className="pb-btn focus-ring"

  `onClick={disabled ? undefined : onClick}`

- Line 100, `motion.button`. aria-label={isPlaying ? "Pause" : "Play"}; className="pb-play focus-ring"

  `onClick={onClick}`

- Line 357, `motion.div`. role="slider"; aria-label="Volume"; className="focus-ring"

  `onPointerDown={onPointerDown}`

  `onPointerMove={(e) => { if (rectRef.current) track(e.clientY); }}`

  `onPointerUp={release}`

  `onPointerCancel={release}`

  `onLostPointerCapture={release}`

  `onKeyDown={onKeyDown}`

- Line 400, `button`. type="button"; aria-label={muted ? "Unmute" : "Mute"}; className="pb-vol-mute"

  `onPointerDown={(e) => e.stopPropagation()}`

  `onClick={onToggleMute}`

- Line 558, `div`. className="scrub"; role="slider"; aria-label="Seek"

  `onKeyDown={handleKeyDown}`

  `onPointerDown={handlePointerDown}`

  `onPointerMove={handlePointerMove}`

  `onPointerUp={handlePointerUp}`

  `onPointerCancel={handlePointerCancel}`

  `onLostPointerCapture={() => endDrag(false)}`

  `onPointerEnter={() => setIsScrubHovered(true)}`

  `onPointerLeave={() => { if (!draggingRef.current) { setIsScrubHovered(false); setHoverFrac(null); } }}`

- Line 873, `motion.button`. type="button"; aria-label={`Listening on ${activeDevice.name}. Change device`}; className="glass-solid-fallback focus-ring t-caption"

  `onClick={toggleDevices}`

- Line 957, `IconBtn`. 

  `onClick={() => doSeek(Math.max(0, usePlayerStore.getState().positionMs - 15_000))}`

- Line 963, `IconBtn`. 

  `onClick={toggleShuffle}`

- Line 970, `IconBtn`. 

  `onClick={handlePrev}`

- Line 976, `PlayPauseButton`. 

  `onClick={togglePlay}`

- Line 983, `IconBtn`. 

  `onClick={handleNext}`

- Line 990, `IconBtn`. 

  `onClick={() => { const s = usePlayerStore.getState(); doSeek(Math.min(s.durationMs || Infinity, s.positionMs + 30_000)); }}`

- Line 1002, `IconBtn`. 

  `onClick={cycleRepeat}`

- Line 1025, `div`. className="group pressable"; role={currentTrack ? "button" : undefined}; aria-label={currentTrack ? "Open full-screen player" : undefined}; title={currentTrack ? "Open full-screen player" : undefined}

  `onClick={() => { if (currentTrack) setImmersiveOpen(true); }}`

  `onKeyDown={(e) => { if (currentTrack && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); setImmersiveOpen(true); } }}`

- Line 1094, `div`. 

  `onPointerEnter={() => setTitlesHovered(true)}`

  `onPointerLeave={() => setTitlesHovered(false)}`

- Line 1102, `Link`. to={`/album/${currentTrack.album.id}`}; className="pb-link"; title={`Go to album: ${currentTrack.album.name}`}

  `onClick={() => setImmersiveOpen(false)}`

  `onPointerEnter={() => prefetchAlbum(qc, currentTrack.album?.id)}`

- Line 1123, `Link`. to={`/artist/${a.id}`}; className="pb-link"

  `onClick={() => setImmersiveOpen(false)}`

  `onPointerEnter={() => prefetchArtist(qc, a.id)}`

- Line 1153, `IconBtn`. 

  `onClick={handleVolumeButtonClick}`

- Line 1164, `input`. className="vol"; aria-label="Volume"; type="range"; value={muted ? 0 : volume}

  `onChange={handleVolumeChange}`

  `onMouseDown={() => { if (muted) { storeSetMuted(false); apiSetMuted(false).catch(() => {}); } }}`

- Line 1181, `VolumePopover`. 

  `onVolume={applyVolume}`

  `onToggleMute={handleMuteToggle}`

## src/components/layout/QueuePanel.tsx

- Line 85, `div`. className="group q-row"; role={onPlay ? "button" : undefined}

  `onClick={onPlay}`

  `onKeyDown={onPlay ? (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onPlay(); } } : undefined}`

- Line 124, `button`. aria-label={`Remove ${track.name} from queue`}; className="queue-btn btn-icon q-remove"

  `onPointerDown={(e) => e.stopPropagation()}`

  `onClick={(e) => { e.stopPropagation(); onRemove(); }}`

- Line 196, `button`. className="btn-text t-caption"

  `onClick={onClear}`

- Line 271, `QueueTrackRow`. 

  `onPlay={canPick ? () => jamSkipTo(r.track.id) : undefined}`

- Line 363, `button`. aria-label="Close queue"; className="btn-icon"

  `onClick={toggleQueue}`

- Line 383, `SectionHead`. 

  `onClear={queue.length > 0 ? clearQueue : undefined}`

- Line 395, `Reorder.Group`. 

  `onReorder={onReorder}`

- Line 420, `QueueTrackRow`. 

  `onRemove={() => removeAt(i)}`

- Line 435, `SectionHead`. 

  `onClear={clearHistory}`

- Line 438, `QueueTrackRow`. 

  `onPlay={() => playItem(track)}`

## src/components/layout/Sidebar.tsx

- Line 47, `motion.button`. className="nav-item sb-item focus-ring"; aria-label={collapsed ? label : undefined}

  `onClick={onClick}`

- Line 244, `button`. className="sb-section-head focus-ring"

  `onClick={onToggle}`

- Line 480, `Section`. 

  `onToggle={() => setSpotifyOpen(v => !v)}`

- Line 481, `NavItem`. 

  `onClick={() => navigate("/")}`

- Line 482, `NavItem`. 

  `onClick={() => navigate("/playlists")}`

- Line 483, `NavItem`. 

  `onClick={() => navigate("/stats")}`

- Line 486, `Section`. 

  `onToggle={() => setLibraryOpen(v => !v)}`

- Line 487, `NavItem`. 

  `onClick={() => navigate("/library?tab=songs")}`

- Line 488, `NavItem`. 

  `onClick={() => navigate("/library?tab=albums")}`

- Line 489, `NavItem`. 

  `onClick={() => navigate("/library?tab=artists")}`

- Line 490, `NavItem`. 

  `onClick={() => navigate("/library?tab=podcasts")}`

- Line 493, `Section`. 

  `onToggle={() => setPinsOpen(v => !v)}`

- Line 535, `button`. type="button"; className="sb-item focus-ring"; title={`${row.name} · ${row.count} playlists (double-click to open)`}

  `onClick={() => toggleFolder(row.id)}`

  `onDoubleClick={() => navigate('/playlists?folder=${encodeURIComponent(row.id)}')}`

- Line 566, `motion.button`. title={isCollapsed ? undefined : p.name}; aria-label={isCollapsed ? p.name : undefined}; className="sb-item focus-ring"

  `onClick={() => navigate('/${p.type}/${p.id}')}`

  `onContextMenu={showAllPlaylists ? undefined : openMenu([ { label: "Unpin", icon: <PinOff size={14} />, onSelect: () => removePin(p.id) }, ])}`

  `onPointerEnter={() => { if (p.type === "album") prefetchAlbum(qc, p.id); else prefetchPlaylist(qc, p.id); }}`

## src/components/layout/TitleBar.tsx

- Line 46, `button`. aria-label={label}; className="cap-btn"

  `onClick={onClick}`

- Line 92, `button`. className="tb-btn"

  `onClick={(e) => { if (!off) onClick(e); }}`

- Line 155, `button`. role="menuitem"; disabled={disabled}; className="row-btn acct-item"

  `onClick={onClick}`

- Line 240, `CapsuleButton`. aria-label="Account"

  `onClick={(e) => { // detail 0 = activated by Enter/Space rather than a pointer setKeyboardOpened(e.detail === 0); setOpen((v) => !v); }}`

- Line 327, `AccountMenuItem`. 

  `onClick={() => go("/profile")}`

- Line 328, `AccountMenuItem`. 

  `onClick={() => go("/settings")}`

- Line 333, `AccountMenuItem`. 

  `onClick={() => { setOpen(false); logout(); }}`

- Line 335, `AccountMenuItem`. disabled={loggingIn || loggingOut}

  `onClick={() => { setOpen(false); login(); }}`

- Line 389, `CaptionBtn`. 

  `onClick={() => win.minimize()}`

- Line 396, `CaptionBtn`. 

  `onClick={() => win.toggleMaximize()}`

- Line 410, `CaptionBtn`. 

  `onClick={() => win.close()}`

- Line 476, `CapsuleButton`. aria-label="Back"

  `onClick={() => navigate(-1)}`

- Line 485, `CapsuleButton`. aria-label="Forward"

  `onClick={() => navigate(1)}`

- Line 498, `CapsuleButton`. aria-label={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}

  `onClick={toggleSidebar}`

- Line 525, `CapsuleButton`. aria-label="Lyrics"

  `onClick={toggleLyrics}`

- Line 536, `CapsuleButton`. aria-label="Queue"

  `onClick={toggleQueue}`

- Line 546, `CapsuleButton`. aria-label="Friend activity"

  `onClick={toggleFriends}`

- Line 558, `CapsuleButton`. aria-label="Devices"

  `onClick={toggleDevices}`

## src/components/layout/TopSearch.tsx

- Line 210, `div`. className="tb-search"

  `onMouseDown={(e) => { // clicks on the pill's padding still land in the field if (e.target !== inputRef.current) { e.preventDefault(); inputRef.current?.focus(); } }}`

- Line 215, `input`. value={query}; placeholder="What do you want to play?"; aria-label="Search"; role="combobox"

  `onChange={(e) => { setQuery(e.target.value.replace(/[\x00-\x1f\x7f]/g, "")); setSelectedIndex(0); }}`

  `onFocus={() => { setFocused(true); setSelectedIndex(0); }}`

  `onBlur={() => { setFocused(false); instantOpenRef.current = false; }}`

  `onKeyDown={handleKeyDown}`

- Line 234, `button`. className="tb-search-clear"; aria-label="Clear search"

  `onMouseDown={(e) => e.preventDefault()}`

  `onClick={() => { setQuery(""); inputRef.current?.focus(); }}`

- Line 249, `motion.div`. id="top-search-results"; role="listbox"; aria-label="Search suggestions"; className="glass-solid-fallback"

  `onMouseDown={(e) => e.preventDefault()}`

- Line 291, `div`. id="ts-opt-0"; className="ts-row"; role="option"

  `onClick={() => handleSelect(0)}`

  `onMouseEnter={() => setSelectedIndex(0)}`

- Line 328, `div`. id={`ts-opt-${idx}`}; className="ts-row"; role="option"

  `onClick={() => handleSelect(idx)}`

  `onMouseEnter={() => setSelectedIndex(idx)}`

- Line 354, `div`. id={`ts-opt-${idx}`}; className="ts-row"; role="option"

  `onClick={() => handleSelect(idx)}`

  `onMouseEnter={() => setSelectedIndex(idx)}`

- Line 373, `div`. id={`ts-opt-${idx}`}; className="ts-row"; role="option"

  `onClick={() => handleSelect(idx)}`

  `onMouseEnter={() => setSelectedIndex(idx)}`

- Line 391, `div`. id={`ts-opt-${idx}`}; className="ts-row"; role="option"

  `onClick={() => handleSelect(idx)}`

  `onMouseEnter={() => setSelectedIndex(idx)}`

- Line 408, `div`. id={`ts-opt-${idx}`}; className="ts-row"; role="option"

  `onClick={() => handleSelect(idx)}`

  `onMouseEnter={() => setSelectedIndex(idx)}`

## src/components/playground/AnimatedIcons.tsx

No JSX event-bearing elements.

## src/components/playground/PlaygroundCards.tsx

- Line 21, `motion.div`. 

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

- Line 52, `CirclePlayButton`. 

  `onClick={(e) => { e.stopPropagation(); setPlaying(!playing); }}`

- Line 187, `motion.div`. 

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onClick={cycleCrate}`

- Line 330, `CirclePlayButton`. 

  `onClick={(e) => { e.stopPropagation(); setPlaying(!playing); onPlay?.(); }}`

## src/components/playground/PlaygroundControls.tsx

- Line 25, `motion.button`. title={playing ? "Pause" : "Play"}

  `onClick={onToggle}`

- Line 89, `motion.button`. title={playing ? "Pause" : "Play"}

  `onClick={onToggle}`

- Line 211, `div`. role="radiogroup"

  `onPointerDown={handlePointerDown}`

  `onPointerMove={handlePointerMove}`

  `onPointerUp={handlePointerUp}`

  `onPointerCancel={handlePointerUp}`

- Line 242, `button`. role="radio"; className="seg-opt"

  `onClick={() => { if (!off) onChange(opt); }}`

- Line 297, `span`. 

  `onClick={(e) => e.stopPropagation()}`

  `onPointerDown={(e) => e.stopPropagation()}`

## src/components/playground/PlaygroundDials.tsx

- Line 16, `div`. 

  `onClick={() => setOn(!on)}`

- Line 101, `motion.button`. title={muted ? "Unmute" : "Mute"}

  `onClick={handleMuteToggle}`

- Line 126, `input`. className="vol"; type="range"; value={muted ? 0 : volume}

  `onChange={handleVolumeChange}`

  `onMouseDown={() => { if (muted) setMuted(false); }}`

## src/components/playground/PlaygroundThemeBar.tsx

- Line 77, `motion.button`. title={`${preset.name} (${preset.hex})`}

  `onClick={() => pickAccent(preset.hex)}`

- Line 122, `button`. title="Reset to Musique Royal Blue"

  `onClick={resetToDefault}`

## src/components/playground/PlaygroundTracks.tsx

- Line 33, `div`. 

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onClick={onTogglePlay}`

- Line 52, `motion.button`. 

  `onClick={(e) => { e.stopPropagation(); onTogglePlay(); }}`

- Line 121, `button`. title={liked ? "Remove from Liked Songs" : "Save to Liked Songs"}

  `onClick={(e) => { e.stopPropagation(); setLiked(!liked); }}`

- Line 143, `button`. title="Add to playlist"

  `onClick={(e) => e.stopPropagation()}`

- Line 176, `TrackRowItem`. 

  `onTogglePlay={() => setPlayingId(playingId === t.id ? "" : t.id)}`

## src/components/podcast/PodcastStage.tsx

- Line 55, `motion.video`. 

  `onError={() => { if (cors) setCors(false); }}`

## src/components/podcast/TranscriptView.tsx

- Line 118, `span`. role={synced ? "button" : undefined}; className="tr-sentence"

  `onClick={synced ? () => onSeek(line.start_ms) : undefined}`

  `onKeyDown={synced ? (e) => { if (e.key === "Enter") onSeek(line.start_ms); } : undefined}`

- Line 235, `div`. className="ovs-hide"

  `onWheel={() => setUserScrolledAt(Date.now())}`

  `onTouchMove={() => setUserScrolledAt(Date.now())}`

- Line 257, `Paragraph`. 

  `onSeek={seek}`

- Line 274, `motion.button`. type="button"; className="focus-ring"

  `onClick={() => setUserScrolledAt(0)}`

## src/components/podcast/transcript.test.ts

No JSX event-bearing elements.

## src/components/ui/AddToPlaylistModal.tsx

- Line 61, `Modal`. 

  `onClose={close}`

- Line 75, `button`. aria-label="Close"; title="Close"; className="btn-icon"

  `onClick={close}`

- Line 83, `input`. value={filter}; placeholder="Find or name a new playlist"; aria-label="Find or name a new playlist"

  `onChange={(e) => setFilter(e.target.value)}`

  `onKeyDown={(e) => { // Enter only acts when the intent is unambiguous, since both // outcomes write to the user's Spotify account: an exact name // (or the single match of a typed filter) adds; a name nothing // matches creates. Never before the list has loaded - an empty // list would read as "no match" and create a duplicate. if (e.key !== "Enter" || busy || isLoading) return; e.preventDefault(); const q = filter.trim().toLowerCase(); if (!q) return; const exact = playlists.find((p) => p.name.toLowerCase() === q); const target = exact ?? (shown.length === 1 ? shown[0] : undefined); if (target) add(target.id, target.name); else if (shown.length === 0) createAndAdd(); }}`

- Line 112, `button`. disabled={busy}; className="row-btn"

  `onClick={createAndAdd}`

- Line 125, `button`. disabled={busy}; className="row-btn"

  `onClick={() => add(p.id, p.name)}`

## src/components/ui/AlbumCard.tsx

- Line 89, `MotionLink`. to={`/album/${album.id}`}; className="card-link"

  `onMouseEnter={() => { setHover(true); prefetchAlbum(qc, album.id); }}`

  `onMouseLeave={() => setHover(false)}`

  `onFocus={() => setHover(true)}`

  `onBlur={() => setHover(false)}`

- Line 141, `CirclePlayButton`. 

  `onClick={handlePlayAlbum}`

## src/components/ui/ArtistAbout.tsx

- Line 39, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={() => openUrl(l.url).catch(() => {})}`

- Line 94, `Modal`. 

  `onClose={onClose}`

- Line 111, `button`. aria-label="Close"; className="btn-icon"

  `onClick={onClose}`

- Line 116, `button`. className="btn-icon"; aria-label="Previous photo"

  `onClick={() => setI((v) => (v - 1 + images.length) % images.length)}`

- Line 117, `button`. className="btn-icon"; aria-label="Next photo"

  `onClick={() => setI((v) => (v + 1) % images.length)}`

- Line 159, `motion.button`. type="button"; className="focus-ring"; aria-label={`About ${name}`}

  `onClick={() => setOpen(true)}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

- Line 232, `AboutModal`. 

  `onClose={() => setOpen(false)}`

## src/components/ui/ArtistCard.tsx

- Line 45, `MotionLink`. to={`/artist/${artist.id}`}; className="card-link"

  `onMouseEnter={() => { setHover(true); prefetchArtist(qc, artist.id); }}`

  `onMouseLeave={() => setHover(false)}`

  `onFocus={() => setHover(true)}`

  `onBlur={() => setHover(false)}`

## src/components/ui/ArtistTour.tsx

- Line 58, `motion.button`. type="button"; className="card-link focus-ring"; disabled={!m.url}

  `onClick={() => m.url && openUrl(m.url).catch(() => {})}`

## src/components/ui/Carousel.tsx

- Line 81, `button`. type="button"; className="btn-icon"; aria-label={`Scroll ${label} back`}

  `onClick={() => canPrev && page(-1)}`

- Line 91, `button`. type="button"; className="btn-icon"; aria-label={`Scroll ${label} forward`}

  `onClick={() => canNext && page(1)}`

## src/components/ui/CirclePlayButton.tsx

- Line 27, `motion.button`. aria-label={ariaLabel}; className="focus-ring"

  `onClick={onClick}`

## src/components/ui/ConfirmDialog.tsx

- Line 27, `Modal`. 

  `onClose={onClose}`

- Line 39, `button`. type="button"; className="btn-pill"

  `onClick={onClose}`

- Line 42, `button`. type="button"; className={danger ? "btn-danger" : "btn-primary"}; disabled={busy}

  `onClick={onConfirm}`

## src/components/ui/ContextMenu.tsx

- Line 50, `ContextMenuView`. 

  `onClose={close}`

- Line 127, `MenuRow`. 

  `onClose={onClose}`

- Line 136, `button`. role="menuitem"; className="row-btn"

  `onClick={() => { entry.onSelect(); onClose(); }}`

## src/components/ui/CoverArt.tsx

- Line 119, `img`. 

  `onLoad={() => setState((s) => (s === "cached" ? s : "loaded"))}`

## src/components/ui/CreditsList.tsx

- Line 102, `Link`. to={`/artist/${p.artist_id}`}; className="row-btn focus-ring credit-link"

  `onClick={onNavigate}`

## src/components/ui/CreditsModal.tsx

- Line 11, `Modal`. 

  `onClose={close}`

- Line 32, `button`. aria-label="Close"; className="btn-icon"

  `onClick={close}`

- Line 37, `CreditsList`. 

  `onNavigate={close}`

## src/components/ui/Dropdown.tsx

- Line 45, `motion.button`. title={title}; className="focus-ring"

  `onClick={() => setOpen((v) => !v)}`

  `onKeyDown={(e) => { if (!open && (e.key === "ArrowDown" || e.key === "ArrowUp")) { e.preventDefault(); setOpen(true); } }}`

- Line 103, `button`. role="menuitemradio"; className="row-btn"

  `onClick={() => { onChange(o.value); setOpen(false); }}`

## src/components/ui/EditPlaylistModal.tsx

- Line 130, `Modal`. 

  `onClose={onClose}`

- Line 137, `form`. 

  `onSubmit={(e) => { e.preventDefault(); save(); }}`

  `onKeyDown={(e) => { if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); save(); } }}`

- Line 153, `button`. type="button"; aria-label="Close"; className="btn-icon"

  `onClick={onClose}`

- Line 175, `input`. className="focus-ring"; value={name}; placeholder="Give it a name"

  `onChange={(e) => setName(e.target.value)}`

- Line 186, `textarea`. className="focus-ring"; value={description}; placeholder="Add an optional description"

  `onChange={(e) => setDescription(e.target.value)}`

- Line 218, `SegmentedControl`. value={VISIBILITY.find((v) => v.value === visibility)!.label}

  `onChange={(label) => setVisibility(VISIBILITY.find((v) => v.label === label)?.value ?? "public")}`

- Line 240, `button`. type="button"; className="btn-pill"

  `onClick={onClose}`

## src/components/ui/EmptyState.tsx

No JSX event-bearing elements.

## src/components/ui/EpisodeRow.tsx

- Line 67, `div`. role="button"; aria-label={`Play ${episode.name}`}; className="group track-row"

  `onClick={onPlay}`

  `onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onPlay(); } }}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onContextMenu={openMenu(menu)}`

- Line 99, `motion.button`. className="focus-ring"; aria-label={playing ? `Pause ${episode.name}` : `Play ${episode.name}`}

  `onClick={(e) => { e.stopPropagation(); onPlay(); }}`

- Line 138, `Link`. to={`/show/${episode.show_id}`}; className="link-inline"

  `onClick={(e) => e.stopPropagation()}`

## src/components/ui/EvenGrid.tsx

No JSX event-bearing elements.

## src/components/ui/ExpandableDescription.tsx

- Line 50, `button`. type="button"; className="btn-text t-caption"

  `onClick={(e) => { e.stopPropagation(); setExpanded(!expanded); }}`

## src/components/ui/HomeFeedShelves.tsx

No JSX event-bearing elements.

## src/components/ui/Loader.tsx

No JSX event-bearing elements.

## src/components/ui/MediaTile.tsx

- Line 35, `MotionLink`. to={to}; className="card-link"

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onFocus={() => setHover(true)}`

  `onBlur={() => setHover(false)}`

  `onContextMenu={onContextMenu}`

## src/components/ui/Modal.tsx

- Line 33, `motion.div`. 

  `onClick={onClose}`

- Line 44, `motion.div`. role="dialog"; className="glass-solid-fallback"

  `onClick={(e) => e.stopPropagation()}`

## src/components/ui/MusiqueLogo.tsx

No JSX event-bearing elements.

## src/components/ui/OverlayScrollbar.tsx

- Line 105, `div`. className="ovs-track"

  `onPointerDown={onTrackDown}`

- Line 116, `div`. className="ovs-thumb"

  `onPointerDown={onThumbDown}`

  `onPointerMove={onThumbMove}`

  `onPointerUp={endDrag}`

  `onPointerCancel={endDrag}`

## src/components/ui/PageHeader.tsx

No JSX event-bearing elements.

## src/components/ui/PlayActions.tsx

- Line 171, `motion.button`. disabled={empty}; className="focus-ring"

  `onClick={onPlay}`

- Line 231, `motion.button`. disabled={empty}; className="focus-ring"

  `onClick={onShuffle}`

- Line 308, `motion.button`. className="ghost-pill focus-ring"

  `onClick={() => togglePin(pinItem as PinnedItem)}`

- Line 335, `motion.button`. aria-label="Share"; className="ghost-pill focus-ring"

  `onClick={(e) => openMenu(shareEntries)(e)}`

  `onContextMenu={openMenu(shareEntries)}`

## src/components/ui/QuitConfirm.tsx

- Line 16, `Modal`. 

  `onClose={() => setOpen(false)}`

- Line 33, `button`. className="btn-pill"

  `onClick={quit}`

- Line 40, `button`. className="btn-primary"

  `onClick={() => setOpen(false)}`

## src/components/ui/ReleaseCountdown.tsx

No JSX event-bearing elements.

## src/components/ui/SaveAlbumButton.tsx

- Line 16, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={() => toggle.mutate( { id, saved }, { onError: (e) => toast.error('Couldn't update your library: ${errMsg(e)}') }, ) }`

## src/components/ui/SectionTitle.tsx

- Line 53, `button`. type="button"; className="btn-text t-caption"

  `onClick={onClick}`

## src/components/ui/Shelf.tsx

No JSX event-bearing elements.

## src/components/ui/SignInPrompt.tsx

- Line 83, `button`. type="button"; className="sign-in-copy-link"; disabled={!links}; aria-label={`Copy ${label} link`}

  `onClick={() => void copy(key)}`

- Line 107, `button`. type="button"; className="sign-in-open-link"; disabled={!links}; aria-label={`Open ${label} tab`}; title={`Open ${label} tab`}

  `onClick={() => void open(key)}`

- Line 119, `input`. className="sign-in-manual-link"; aria-label={`${label} sign-in link`}; value={links[key]}

  `onFocus={(event) => event.currentTarget.select()}`

- Line 132, `button`. type="button"; className="sign-in-copy-both"

  `onClick={() => void copy("both")}`

- Line 149, `button`. type="button"; className="sign-in-copy-both"

  `onClick={prepare}`

- Line 228, `button`. type="button"; className="btn-primary sign-in-login"; disabled={loggingIn}

  `onClick={() => { setLinksOpen(false); login(); }}`

- Line 247, `div`. className="sign-in-copy"

  `onBlur={(event) => { if ( event.relatedTarget && !event.currentTarget.contains(event.relatedTarget as Node) ) setLinksOpen(false); }}`

- Line 258, `button`. type="button"; className="btn-pill sign-in-copy-trigger"

  `onClick={() => (linksOpen ? setLinksOpen(false) : showLinks())}`

  `onKeyDown={(event) => { if (event.key === "ArrowDown") { event.preventDefault(); showLinks(true); } }}`

- Line 302, `button`. type="button"; className="sign-in-help-trigger"

  `onClick={() => { setLinksOpen(false); setHelpOpen(!helpOpen); }}`

- Line 325, `button`. type="button"; className="sign-in-help-close"; aria-label="Close sign-in help"

  `onClick={() => { setHelpOpen(false); helpButtonRef.current?.focus(); }}`

## src/components/ui/Skeletons.tsx

No JSX event-bearing elements.

## src/components/ui/SortDirectionToggle.tsx

- Line 21, `motion.button`. type="button"; aria-label={asc ? "Sorted ascending. Switch to descending" : "Sorted descending. Switch to ascending"}; className="focus-ring"

  `onClick={() => onChange(asc ? "desc" : "asc")}`

## src/components/ui/SortToolbar.tsx

- Line 53, `input`. value={query}; placeholder={placeholder}; aria-label={placeholder}

  `onChange={(e) => setQuery(e.target.value)}`

- Line 66, `button`. title="Clear"; aria-label="Clear"; className="btn-icon"

  `onClick={() => setQuery("")}`

- Line 79, `Dropdown`. value={sort}; title="Sort by"

  `onChange={setSort}`

- Line 80, `SortDirectionToggle`. 

  `onChange={setDir}`

## src/components/ui/SpeedControl.tsx

- Line 38, `motion.button`. type="button"; aria-label={`Playback speed ${speedLabel(speed)}`}; className="focus-ring tnum"

  `onClick={() => setOpen((v) => !v)}`

- Line 89, `button`. type="button"; role="menuitemradio"; className="row-btn tnum"

  `onClick={() => { setSpeed(v); setOpen(false); }}`

## src/components/ui/Switch.tsx

- Line 15, `button`. type="button"; role="switch"; aria-label={label}; className="switch"

  `onClick={() => onChange(!checked)}`

## src/components/ui/SyncButton.tsx

- Line 46, `button`. type="button"; className={prominent ? "btn-primary" : "btn-pill"}

  `onClick={() => { if (isPending) return; mutate(undefined, { onError: (e) => toast.error('Sync failed: ${errMsg(e)}') }); }}`

## src/components/ui/Toaster.tsx

- Line 53, `button`. type="button"; className="btn-pill"

  `onClick={() => { // the toast stays on screen for its exit animation with this // handler still attached; a double-click must not undo twice if (!useToastStore.getState().toasts.some((x) => x.id === t.id)) return; remove(t.id); t.action!.onClick(); }}`

## src/components/ui/Tooltip.tsx

- Line 165, `span`. 

  `onMouseEnter={handleMouseEnter}`

  `onMouseLeave={handleMouseLeave}`

  `onFocus={handleFocus}`

  `onBlur={handleMouseLeave}`

## src/components/ui/TrackRow.tsx

- Line 57, `motion.button`. aria-label={title || undefined}; className={`row-action ${className ?? ""}`}

  `onClick={onClick}`

- Line 168, `div`. role={onPlay ? "button" : undefined}; className="group track-row"

  `onClick={() => { if (onPlay) handlePlayToggle(); }}`

  `onKeyDown={(e) => { if (onPlay && (e.key === "Enter" || e.key === " ")) { e.preventDefault(); handlePlayToggle(); } }}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onContextMenu={openMenu(menuEntries)}`

- Line 201, `motion.button`. className="focus-ring"; aria-label={isThisPlaying ? `Pause ${track.name}` : `Play ${track.name}`}

  `onClick={(e) => { stop(e); handlePlayToggle(); }}`

- Line 300, `Link`. to={`/artist/${a.id}`}; className="link-inline"

  `onClick={stop}`

  `onMouseEnter={() => prefetchArtist(qc, a.id)}`

- Line 313, `Link`. to={`/album/${track.album.id}`}; className="link-inline"

  `onClick={stop}`

  `onMouseEnter={() => prefetchAlbum(qc, track.album!.id)}`

- Line 328, `ActionBtn`. title={liked ? "Remove from Liked Songs" : "Save to Liked Songs"}; className={liked ? "" : "queue-btn"}

  `onClick={(e) => { stop(e); onToggleLike(track); }}`

- Line 343, `ActionBtn`. title={isQueued || justAdded ? "In queue" : "Add to queue"}; className={isQueued || justAdded ? "" : "queue-btn"}

  `onClick={handleEnqueue}`

## src/components/ui/TrackToolbar.tsx

- Line 85, `input`. value={query}; placeholder="Find in tracks"; aria-label="Find in tracks"

  `onChange={(e) => setQuery(e.target.value)}`

- Line 99, `button`. aria-label="Clear filter"; className="btn-icon"

  `onClick={() => setQuery("")}`

- Line 112, `Dropdown`. value={order}; title="Sort by"

  `onChange={setOrder}`

- Line 113, `SortDirectionToggle`. 

  `onChange={setDir}`

## src/components/ui/YtMatchModal.tsx

- Line 83, `Modal`. 

  `onClose={close}`

- Line 101, `button`. aria-label="Close"; title="Close"; className="btn-icon"

  `onClick={close}`

- Line 143, `button`. disabled={busy}; className="row-btn"

  `onClick={() => pick(c.video_id)}`

- Line 175, `button`. disabled={busy || !match}; className="btn-pill"

  `onClick={reset}`

## src/components/ui/queueMembership.test.ts

No JSX event-bearing elements.

## src/components/ui/queueMembership.ts

No JSX event-bearing elements.

## src/hooks/useAlbum.ts

No JSX event-bearing elements.

## src/hooks/useAmbient.ts

No JSX event-bearing elements.

## src/hooks/useArtist.ts

No JSX event-bearing elements.

## src/hooks/useAuth.ts

No JSX event-bearing elements.

## src/hooks/useCanvas.ts

No JSX event-bearing elements.

## src/hooks/useDevices.ts

No JSX event-bearing elements.

## src/hooks/useEvenColumns.test.ts

No JSX event-bearing elements.

## src/hooks/useEvenColumns.ts

No JSX event-bearing elements.

## src/hooks/useJamSync.ts

No JSX event-bearing elements.

## src/hooks/useLibrary.ts

No JSX event-bearing elements.

## src/hooks/useLyricFollow.ts

No JSX event-bearing elements.

## src/hooks/useLyrics.ts

No JSX event-bearing elements.

## src/hooks/useMenuKeyboard.ts

No JSX event-bearing elements.

## src/hooks/useModalA11y.ts

No JSX event-bearing elements.

## src/hooks/usePlayEpisodes.ts

No JSX event-bearing elements.

## src/hooks/usePlaybackBackend.ts

No JSX event-bearing elements.

## src/hooks/usePlayerControls.ts

No JSX event-bearing elements.

## src/hooks/usePlaylist.ts

No JSX event-bearing elements.

## src/hooks/usePlaylistFolders.ts

No JSX event-bearing elements.

## src/hooks/usePositionTicker.ts

No JSX event-bearing elements.

## src/hooks/useReflowPulse.ts

No JSX event-bearing elements.

## src/hooks/useSearch.ts

No JSX event-bearing elements.

## src/hooks/useVideoAmbient.ts

No JSX event-bearing elements.

## src/hooks/useWindowActive.ts

No JSX event-bearing elements.

## src/lib/ambient.ts

No JSX event-bearing elements.

## src/lib/asyncListeners.test.ts

No JSX event-bearing elements.

## src/lib/asyncListeners.ts

No JSX event-bearing elements.

## src/lib/authSession.test.ts

No JSX event-bearing elements.

## src/lib/authSession.ts

No JSX event-bearing elements.

## src/lib/backdrop.test.ts

No JSX event-bearing elements.

## src/lib/backdrop.ts

No JSX event-bearing elements.

## src/lib/color.ts

No JSX event-bearing elements.

## src/lib/coverUrl.test.ts

No JSX event-bearing elements.

## src/lib/coverUrl.ts

No JSX event-bearing elements.

## src/lib/err.ts

No JSX event-bearing elements.

## src/lib/icons.tsx

No JSX event-bearing elements.

## src/lib/jam.ts

No JSX event-bearing elements.

## src/lib/keyLayers.ts

No JSX event-bearing elements.

## src/lib/layout.ts

No JSX event-bearing elements.

## src/lib/lyrics.tsx

No JSX event-bearing elements.

## src/lib/lyricsRows.test.ts

No JSX event-bearing elements.

## src/lib/lyricsRows.ts

No JSX event-bearing elements.

## src/lib/lyricsSource.ts

No JSX event-bearing elements.

## src/lib/lyricsUpgrades.test.ts

No JSX event-bearing elements.

## src/lib/lyricsUpgrades.ts

No JSX event-bearing elements.

## src/lib/lyricsWeld.test.ts

No JSX event-bearing elements.

## src/lib/mesh.ts

No JSX event-bearing elements.

## src/lib/motion.ts

No JSX event-bearing elements.

## src/lib/outputLatency.ts

No JSX event-bearing elements.

## src/lib/persistStorage.test.ts

No JSX event-bearing elements.

## src/lib/persistStorage.ts

No JSX event-bearing elements.

## src/lib/platform.ts

No JSX event-bearing elements.

## src/lib/prefetch.ts

No JSX event-bearing elements.

## src/lib/railFlip.ts

No JSX event-bearing elements.

## src/lib/rootlist.test.ts

No JSX event-bearing elements.

## src/lib/rootlist.ts

No JSX event-bearing elements.

## src/lib/scrollSpring.ts

No JSX event-bearing elements.

## src/lib/share.ts

No JSX event-bearing elements.

## src/lib/updater.ts

No JSX event-bearing elements.

## src/lib/zoom.ts

No JSX event-bearing elements.

## src/main.tsx

No JSX event-bearing elements.

## src/pages/AlbumPage.tsx

- Line 68, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 91, `div`. className="flex flex-col"

  `onContextMenu={openMenu([ { label: pinned ? "Unpin from sidebar" : "Pin to sidebar", icon: <Pin size={14} active={pinned} />, onSelect: () => togglePin(pinItem) }, { label: "Album radio", icon: <Radio size={14} />, onSelect: () => { playStation('spotify:album:${data.id}', data.name); } }, ])}`

- Line 149, `TrackRow`. 

  `onPlay={() => startAt(i)}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}`

## src/pages/ArtistPage.tsx

- Line 84, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 119, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={() => toggleFollow.mutate({ id: data.id, following })}`

- Line 148, `div`. className="flex flex-col"

  `onContextMenu={openMenu(shareEntries)}`

- Line 187, `motion.button`. type="button"; aria-label={`${data.name} radio`}; className="ghost-pill focus-ring"

  `onClick={() => playStation('spotify:artist:${data.id}', data.name)}`

- Line 211, `ShowAllButton`. 

  `onClick={() => setShowAllTop((v) => !v)}`

- Line 236, `TrackRow`. 

  `onPlay={() => startTop(i)}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) }) }`

## src/pages/Home.tsx

- Line 247, `motion.div`. 

  `onClick={() => handlePlay()}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onFocus={() => setFocused(true)}`

  `onBlur={() => setFocused(false)}`

- Line 378, `CirclePlayButton`. 

  `onClick={(e) => handlePlay(e)}`

- Line 702, `motion.button`. 

  `onClick={onPlay}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

- Line 752, `CirclePlayButton`. 

  `onClick={(e) => { e.stopPropagation(); if (isThisTrackPlaying) { transportPause(); } else { onPlay(); } }}`

- Line 831, `RecTile`. 

  `onPlay={() => play(i)}`

- Line 874, `RecTile`. 

  `onPlay={() => play(i)}`

- Line 909, `RecTile`. 

  `onPlay={() => play(i)}`

- Line 938, `SegmentedControl`. value={KEY_TO_LABEL[value]}

  `onChange={(label) => onChange(LABEL_TO_KEY[label])}`

- Line 989, `RangeSlider`. value={range}

  `onChange={setRange}`

- Line 995, `button`. type="button"; className="btn-text"

  `onClick={() => refetch()}`

- Line 1017, `RangeSlider`. value={range}

  `onChange={setRange}`

- Line 1023, `button`. type="button"; className="btn-text"

  `onClick={() => refetch()}`

## src/pages/Library.tsx

- Line 90, `TrackRow`. 

  `onPlay={() => handlePlay(i)}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}`

## src/pages/Playground.tsx

- Line 73, `PrimaryPlayButton`. 

  `onToggle={() => setPrimaryPlaying(!primaryPlaying)}`

- Line 79, `PulsePlayButton`. 

  `onToggle={() => setPulsePlaying(!pulsePlaying)}`

- Line 84, `SegmentedControl`. value={segmentedMode}

  `onChange={setSegmentedMode}`

## src/pages/PlaylistPage.tsx

- Line 116, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 178, `div`. className="flex flex-col"

  `onContextMenu={openMenu([ { label: pinned ? "Unpin from sidebar" : "Pin to sidebar", icon: <Pin size={14} active={pinned} />, onSelect: () => togglePin(pinItem) }, { label: "Playlist radio", icon: <Radio size={14} />, onSelect: () => { playStation('spotify:playlist:${data.id}', data.name); } }, ...(isOwner ? [ { label: "Edit details", icon: <Pencil size={14} />, onSelect: () => setEditOpen(true) }, { label: "Delete playlist", icon: <Trash2 size={14} />, onSelect: () => setDeleteOpen(true) }, ] : [ { label: following ? "Remove from your library" : "Save to your library", icon: <Heart size={14} active={following} />, onSelect: toggleFollow, }, ]), ])}`

- Line 223, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={isOwner ? () => setEditOpen(true) : toggleFollow}`

- Line 243, `EditPlaylistModal`. 

  `onClose={() => setEditOpen(false)}`

  `onSaved={() => { qc.invalidateQueries({ queryKey: ["playlist", data.id] }); qc.invalidateQueries({ queryKey: ["library", "playlists"] }); }}`

- Line 252, `ConfirmDialog`. title="Delete playlist?"

  `onConfirm={deletePlaylist}`

  `onClose={() => setDeleteOpen(false)}`

- Line 271, `TrackRow`. 

  `onPlay={() => startAt(i)}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}`

  `onRemoveFromPlaylist={canEdit ? removeFromPlaylist : undefined}`

## src/pages/Playlists.tsx

- Line 62, `MotionLink`. to={`/playlist/${playlist.id}`}

  `onContextMenu={onContextMenu}`

  `onMouseEnter={handleMouseEnter}`

  `onMouseLeave={handleMouseLeave}`

- Line 187, `MotionLink`. to={`/playlists?folder=${encodeURIComponent(folder.id)}`}

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

- Line 389, `PlaylistCard`. 

  `onContextMenu={openMenu(cardMenu(p))}`

## src/pages/ProfilePage.tsx

- Line 108, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 129, `button`. type="button"; className="focus-ring"

  `onClick={() => setTab(tab === to ? "overview" : to)}`

- Line 152, `div`. className="flex flex-col"

  `onContextMenu={openMenu(shareEntries)}`

- Line 165, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={toggleFollow}`

- Line 180, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={() => shareSpotifyLink("user", data.id)}`

- Line 215, `TrackRow`. 

  `onPlay={() => { const start = playContext(topTracks, i, "profile-top"); if (start) { setCurrentTrack(start); playTrack(start.id).catch(() => {}); } }}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: liked.has(track.id) })}`

## src/pages/Search.tsx

- Line 179, `motion.div`. 

  `onClick={handleClick}`

  `onMouseEnter={() => setHover(true)}`

  `onMouseLeave={() => setHover(false)}`

  `onFocus={() => setFocused(true)}`

  `onBlur={() => setFocused(false)}`

- Line 288, `button`. type="button"; className="focus-ring"

  `onClick={(e) => { e.stopPropagation(); onPlay(e); }}`

- Line 297, `Link`. to={targetPath}; className="focus-ring"

  `onClick={(e) => e.stopPropagation()}`

- Line 325, `motion.button`. type="button"; className="focus-ring"; aria-label={isPlaying ? `Pause ${title}` : `Play ${title}`}

  `onClick={(e) => { e.stopPropagation(); onPlay(e); }}`

- Line 374, `EpisodeRow`. 

  `onPlay={() => play(i)}`

- Line 533, `TrackRow`. 

  `onPlay={() => playFromSearch(i)}`

  `onQueue={(track) => enqueue(track)}`

  `onToggleLike={(track) => toggleLike.mutate({ id: track.id, liked: likedSet.has(track.id) })}`

- Line 545, `ShowAllButton`. 

  `onClick={() => setCat(c)}`

- Line 597, `SegmentedControl`. value={CATEGORY_LABEL[cat]}

  `onChange={(label) => setCat(LABEL_TO_CATEGORY[label] ?? "all")}`

- Line 623, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 667, `TopResultCard`. 

  `onPlay={handlePlayTopResult}`

## src/pages/Settings.tsx

- Line 133, `button`. disabled={disabled}; className="btn-primary"

  `onClick={onClick}`

- Line 160, `button`. disabled={disabled}; className={danger ? "btn-danger" : subtle ? "btn-text ghost-subtle" : "btn-pill"}

  `onClick={onClick}`

- Line 189, `motion.button`. type="button"; aria-label="Revert to default"; className="btn-icon"

  `onClick={() => { if (!atDefault) onClick(); }}`

- Line 307, `SegmentedControl`. value={currentLabel}; disabled={disabled.length ? disabled : undefined}

  `onChange={(label) => { const targetVal = labelToValue.get(label); if (targetVal !== undefined) onChange(targetVal); }}`

- Line 358, `button`. type="button"; className="btn-icon"; aria-label="Zoom out"

  `onClick={() => zoom > ZOOM_MIN && stepZoom(-1, { quiet: true })}`

- Line 370, `button`. type="button"; className="btn-text tnum"; aria-label={`Zoom ${zoomLabel(zoom)}. Reset to 100%`}

  `onClick={() => resetZoom({ quiet: true })}`

- Line 381, `button`. type="button"; className="btn-icon"; aria-label="Zoom in"

  `onClick={() => zoom < ZOOM_MAX && stepZoom(1, { quiet: true })}`

- Line 425, `Segmented`. value={sidebarMode}

  `onChange={setSidebarMode}`

- Line 442, `Segmented`. value={source}

  `onChange={setSource}`

- Line 454, `Switch`. 

  `onChange={setAlbumColors}`

- Line 460, `Switch`. 

  `onChange={setAmbientMotion}`

- Line 466, `Switch`. 

  `onChange={setShowCanvas}`

- Line 569, `Segmented`. value={pendingBackend ?? backend?.active ?? "youtube"}

  `onChange={changeBackend}`

- Line 581, `Segmented`. value={audioQuality}

  `onChange={setAudioQuality}`

- Line 604, `input`. className="vol"; type="range"; value={audioCacheLimitMb}; aria-label="Disk cache limit"

  `onChange={(e) => setAudioCacheLimitMb(Number(e.target.value))}`

- Line 633, `RevertBtn`. 

  `onClick={() => setAudioCacheLimitMb(2048)}`

- Line 661, `Switch`. 

  `onChange={(v) => { setNotifyOnTrack(v); if (v) requestNotificationPermission(); }}`

- Line 674, `Switch`. 

  `onChange={setPromptOnClose}`

- Line 680, `Switch`. 

  `onChange={toggleDiscord}`

- Line 697, `Switch`. 

  `onChange={(v) => { setShowFriends(v); if (!v && usePlayerStore.getState().friendsOpen) usePlayerStore.getState().toggleFriends(); }}`

- Line 710, `Switch`. 

  `onChange={setShowStats}`

- Line 757, `Segmented`. value={windowEffect}

  `onChange={choose}`

- Line 764, `Segmented`. value={macValue}

  `onChange={choose}`

- Line 795, `input`. className="vol"; type="range"; value={tpct}; aria-label="Material transparency"

  `onChange={(e) => setTransparency(Number(e.target.value) / 100) }`

- Line 829, `RevertBtn`. 

  `onClick={() => setTransparency(0.4)}`

- Line 932, `div`. role="alertdialog"; aria-label="Forget Last.fm API keys"

  `onKeyDown={(e) => { if (e.key === "Escape") setConfirmForget(false); }}`

- Line 941, `GhostBtn`. 

  `onClick={() => setConfirmForget(false)}`

- Line 944, `GhostBtn`. 

  `onClick={forget}`

- Line 950, `GhostBtn`. 

  `onClick={disconnect}`

- Line 954, `GhostBtn`. 

  `onClick={() => setConfirmForget(true)}`

- Line 986, `input`. value={apiKey}; placeholder={ status?.configured ? "•••••••• (saved)" : "Paste your API key" }; disabled={busy}; aria-label="Last.fm API key"; className="settings-input"

  `onChange={(e) => setApiKey(e.currentTarget.value)}`

- Line 1001, `input`. type="password"; value={apiSecret}; placeholder={ status?.configured ? "•••••••• (saved)" : "Paste your shared secret" }; disabled={busy}; aria-label="Last.fm shared secret"; className="settings-input"

  `onChange={(e) => setApiSecret(e.currentTarget.value)}`

- Line 1042, `PrimaryBtn`. disabled={ busy || (!status?.configured && !(apiKey.trim() && apiSecret.trim())) }

  `onClick={saveAndConnect}`

- Line 1214, `button`. type="button"; className="disclosure"

  `onClick={() => setCollapsed((v) => !v)}`

- Line 1316, `input`. value={clientId}; placeholder={ isLoading ? "Loading…" : isCustom ? clientId : "Leave blank to use built-in access" }; disabled={busy}; aria-label="Spotify client ID"; className="settings-input"

  `onChange={(e) => setClientId(e.currentTarget.value)}`

- Line 1336, `input`. type={showSecret ? "text" : "password"}; value={clientSecret}; placeholder={ isLoading ? "Loading…" : isCustom && status === "configured" ? "•••••••••••••••• (saved)" : "Not needed for most setups" }; disabled={busy}; aria-label="Spotify client secret"; className="settings-input"

  `onChange={(e) => setClientSecret(e.currentTarget.value)}`

- Line 1354, `button`. type="button"; aria-label={showSecret ? "Hide client secret" : "Show client secret"}; title={showSecret ? "Hide" : "Show"}; className="btn-pill"

  `onClick={() => setShowSecret((v) => !v)}`

- Line 1387, `PrimaryBtn`. disabled={!canSave}

  `onClick={() => save()}`

- Line 1391, `GhostBtn`. disabled={!canReset}

  `onClick={() => resetToDefault()}`

## src/pages/ShowPage.tsx

- Line 88, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

- Line 132, `div`. className="flex flex-col"

  `onContextMenu={openMenu([ { label: saved ? "Unfollow" : "Follow", icon: <Heart size={14} active={saved} />, onSelect: toggleSave }, { label: "Copy Spotify link", icon: <Link2 size={14} />, onSelect: () => shareSpotifyLink("show", data.id) }, ])}`

- Line 144, `PlayActions`. 

  `onStart={startFrom}`

- Line 152, `motion.button`. type="button"; className="ghost-pill focus-ring"

  `onClick={toggleSave}`

- Line 180, `input`. value={query}; placeholder="Find in episodes"; aria-label="Find in episodes"

  `onChange={(e) => setQuery(e.target.value)}`

- Line 189, `button`. aria-label="Clear"; className="btn-icon"

  `onClick={() => setQuery("")}`

- Line 195, `Dropdown`. value={order}; title="Sort episodes"

  `onChange={setOrder}`

- Line 206, `EpisodeRow`. 

  `onPlay={() => play(i)}`

- Line 211, `button`. type="button"; className="btn-pill"; disabled={loadingMore}

  `onClick={loadMore}`

## src/pages/StatsPage.tsx

- Line 69, `div`. role="gridcell"; aria-label={`${DAYS[day]} ${hourLabel(hour)}: ${plays} ${plays === 1 ? "play" : "plays"}`}; title={`${DAYS[day]} ${hourLabel(hour)} · ${plays} ${plays === 1 ? "play" : "plays"}`}

  `onMouseEnter={() => setHover({ day, hour })}`

  `onMouseLeave={() => setHover(null)}`

- Line 142, `button`. type="button"; className="q-row focus-ring"

  `onClick={() => play(i)}`

- Line 215, `SegmentedControl`. value={RANGES.find((r) => r.value === range)!.label}

  `onChange={(label) => setRange(RANGES.find((r) => r.label === label)?.value ?? "month")}`

- Line 229, `button`. type="button"; className="btn-pill"

  `onClick={() => refetch()}`

## src/store/addToPlaylist.store.ts

No JSX event-bearing elements.

## src/store/auth.store.ts

No JSX event-bearing elements.

## src/store/credentials.store.ts

No JSX event-bearing elements.

## src/store/credits.store.ts

No JSX event-bearing elements.

## src/store/jam.store.ts

No JSX event-bearing elements.

## src/store/pins.store.ts

No JSX event-bearing elements.

## src/store/player.store.ts

No JSX event-bearing elements.

## src/store/prefs.store.ts

No JSX event-bearing elements.

## src/store/queue.store.ts

No JSX event-bearing elements.

## src/store/speedDial.store.ts

No JSX event-bearing elements.

## src/store/theme.store.ts

No JSX event-bearing elements.

## src/store/toast.store.ts

No JSX event-bearing elements.

## src/store/ui.store.ts

No JSX event-bearing elements.

## src/store/updater.store.ts

No JSX event-bearing elements.

## src/store/ytMatch.store.ts

No JSX event-bearing elements.

## src/types/ipc.ts

No JSX event-bearing elements.

## src/types/library.ts

No JSX event-bearing elements.

## src/types/podcast.ts

No JSX event-bearing elements.

## src/types/spotify.ts

No JSX event-bearing elements.

## src/utils/episode.test.ts

No JSX event-bearing elements.

## src/utils/episode.ts

No JSX event-bearing elements.

## src/utils/fmt.test.ts

No JSX event-bearing elements.

## src/utils/fmt.ts

No JSX event-bearing elements.

## src/utils/listenTracker.ts

No JSX event-bearing elements.

## src/utils/palette.ts

No JSX event-bearing elements.

## src/utils/radio.ts

No JSX event-bearing elements.

## src/utils/romanize.ts

No JSX event-bearing elements.

## src/utils/stats.test.ts

No JSX event-bearing elements.

## src/utils/stats.ts

No JSX event-bearing elements.

## src/vite-env.d.ts

No JSX event-bearing elements.

## vite.config.ts

No JSX event-bearing elements.
