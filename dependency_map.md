# Frontend dependency network

Generated from the complete TypeScript syntax trees. Imports and dynamic imports are recorded per file. This supports the manual audit; it is not a runtime trace.

## src/App.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `./components/layout/Layout`
- `./components/ui/Loader`
- `./components/ThemeEngine`
- `./components/UpdatePrompt`
- `./pages/Home`
- `dynamic "./pages/Settings"`
- `dynamic "./pages/Search"`
- `dynamic "./pages/Library"`
- `dynamic "./pages/Playlists"`
- `dynamic "./pages/PlaylistPage"`
- `dynamic "./pages/ArtistPage"`
- `dynamic "./pages/AlbumPage"`
- `dynamic "./pages/Playground"`
- `dynamic "./pages/ProfilePage"`
- `dynamic "./pages/ShowPage"`
- `dynamic "./pages/StatsPage"`
- `./api/credentials`
- `./hooks/useAuth`
- `./api/playback`
- `./api/media`
- `./store/credentials.store`
- `./store/auth.store`
- `./store/ui.store`
- `./api/window`
- `./api/spotify`
- `@tanstack/react-query`
- `@tauri-apps/api/event`
- `./store/player.store`
- `./store/queue.store`
- `./store/speedDial.store`
- `./store/prefs.store`
- `./store/toast.store`
- `./hooks/useLibrary`
- `./utils/radio`
- `./utils/listenTracker`
- `./hooks/usePlayerControls`
- `./lib/prefetch`
- `./api/lastfm`
- `./utils/episode`
- `@tauri-apps/api/window`
- `./hooks/useJamSync`
- `./store/jam.store`
- `./lib/asyncListeners`

## src/api/auth.ts

- `@tauri-apps/api/core`
- `../types/ipc`
- `../types/spotify`

## src/api/connect.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/credentials.ts

- `@tauri-apps/api/core`

## src/api/internal.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/lastfm.ts

- `@tauri-apps/api/core`

## src/api/library.ts

- `@tauri-apps/api/core`
- `../types/spotify`
- `../types/library`

## src/api/lyrics.test.ts

- `vitest`
- `@tauri-apps/api/core`
- `./lyrics`
- `../types/spotify`

## src/api/lyrics.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/media.ts

- `@tauri-apps/api/core`
- `../types/spotify`
- `@tauri-apps/plugin-notification`

## src/api/playback.ts

- `@tauri-apps/api/core`
- `../store/player.store`
- `../store/queue.store`
- `../store/toast.store`
- `../lib/err`
- `../store/jam.store`
- `../lib/jam`

## src/api/podcasts.ts

- `@tauri-apps/api/core`
- `../types/podcast`

## src/api/profile.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/settings.ts

- `@tauri-apps/api/core`

## src/api/share.ts

- `@tauri-apps/api/core`

## src/api/social.ts

- `@tauri-apps/api/core`

## src/api/spotify.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/stats.ts

- `@tauri-apps/api/core`
- `../types/spotify`

## src/api/theme.ts

- `@tauri-apps/api/core`

## src/api/window.ts

- `@tauri-apps/api/core`

## src/components/ErrorBoundary.tsx

- `react`
- `../lib/err`

## src/components/ThemeEngine.tsx

- `react`
- `../store/theme.store`
- `../store/ui.store`
- `../api/theme`
- `../lib/color`

## src/components/UpdatePrompt.tsx

- `react`
- `framer-motion`
- `@/lib/motion`
- `./ui/Modal`
- `@/lib/icons`
- `../store/updater.store`
- `../lib/updater`

## src/components/auth/ProfileCard.tsx


## src/components/layout/DevicesPopover.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../hooks/useDevices`
- `../../api/connect`
- `../../lib/motion`

## src/components/layout/FriendsPanel.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../api/social`
- `../../store/player.store`
- `../../store/queue.store`
- `../../api/playback`
- `../ui/CoverArt`
- `./JamCard`
- `../ui/Tooltip`
- `../../lib/err`
- `../../lib/motion`
- `../../types/spotify`

## src/components/layout/Immersive.tsx

- `react`
- `../../hooks/useCanvas`
- `../../hooks/useVideoAmbient`
- `../ui/CreditsList`
- `../podcast/PodcastStage`
- `../podcast/TranscriptView`
- `../../utils/episode`
- `../../api/internal`
- `../../lib/coverUrl`
- `framer-motion`
- `@/lib/icons`
- `../../store/ui.store`
- `../../store/player.store`
- `../../store/queue.store`
- `../../hooks/usePlayerControls`
- `../../hooks/useLyrics`
- `../../hooks/useAmbient`
- `../../hooks/useWindowActive`
- `../../hooks/useLyricFollow`
- `./LyricReturnPill`
- `../../store/prefs.store`
- `../../lib/motion`
- `../../lib/ambient`
- `../../styles/lyrics.css`
- `./PlayerBar`
- `../../api/playback`
- `../../lib/lyrics`

## src/components/layout/JamCard.tsx

- `react`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../api/social`
- `../ui/CoverArt`
- `../ui/Tooltip`
- `../../store/toast.store`
- `../../store/jam.store`
- `../../lib/err`
- `../../lib/jam`
- `../../hooks/usePlaybackBackend`
- `../../lib/motion`

## src/components/layout/Layout.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `./TitleBar`
- `./Sidebar`
- `./PlayerBar`
- `../../hooks/usePositionTicker`
- `./QueuePanel`
- `./FriendsPanel`
- `./LyricsPanel`
- `../ui/QuitConfirm`
- `../ui/Toaster`
- `./Immersive`
- `../ui/AddToPlaylistModal`
- `../ui/CreditsModal`
- `../ui/YtMatchModal`
- `./DevicesPopover`
- `../../store/player.store`
- `../../store/ui.store`
- `../../api/window`
- `../../lib/backdrop`
- `../../lib/platform`
- `../../store/prefs.store`
- `../../lib/zoom`
- `../../lib/motion`
- `../../lib/railFlip`
- `../../styles/layout.css`

## src/components/layout/LyricReturnPill.tsx

- `framer-motion`
- `@/lib/icons`
- `../../lib/motion`
- `../../styles/lyrics.css`

## src/components/layout/LyricsPanel.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../store/player.store`
- `../../hooks/useLyrics`
- `../../lib/lyricsSource`
- `../../hooks/useAmbient`
- `../../api/playback`
- `../ui/Loader`
- `../ui/Tooltip`
- `../../lib/motion`
- `../../hooks/useLyricFollow`
- `./LyricReturnPill`
- `../../styles/lyrics.css`
- `../../utils/romanize`
- `../../lib/lyrics`
- `../podcast/TranscriptView`
- `../../utils/episode`

## src/components/layout/PlayerBar.tsx

- `react`
- `react-router-dom`
- `@tanstack/react-query`
- `framer-motion`
- `../../lib/prefetch`
- `@/lib/icons`
- `zustand/react/shallow`
- `../../store/player.store`
- `../../store/queue.store`
- `../../store/jam.store`
- `../../api/playback`
- `../../api/connect`
- `../../hooks/useDevices`
- `../ui/CoverArt`
- `../ui/Tooltip`
- `../../utils/fmt`
- `../../hooks/usePlayerControls`
- `../../lib/motion`
- `../ui/SpeedControl`
- `../../utils/episode`

## src/components/layout/QueuePanel.tsx

- `react`
- `../../lib/coverUrl`
- `framer-motion`
- `@/lib/icons`
- `../../store/player.store`
- `../../store/queue.store`
- `../../api/playback`
- `../../utils/fmt`
- `../../lib/mesh`
- `../../types/spotify`
- `../ui/Tooltip`
- `../../lib/motion`
- `@tanstack/react-query`
- `../../api/connect`
- `../../store/jam.store`
- `../../lib/jam`
- `../../api/social`

## src/components/layout/Sidebar.tsx

- `react`
- `../../lib/coverUrl`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../../store/pins.store`
- `../../hooks/useLibrary`
- `../../hooks/usePlaylistFolders`
- `../../lib/rootlist`
- `../../api/internal`
- `../../store/ui.store`
- `../ui/ContextMenu`
- `../../lib/motion`
- `@tanstack/react-query`
- `../../lib/prefetch`
- `../ui/Tooltip`
- `../ui/OverlayScrollbar`
- `../../lib/platform`
- `../../store/prefs.store`
- `../../lib/zoom`

## src/components/layout/TitleBar.tsx

- `react`
- `framer-motion`
- `@tauri-apps/api/window`
- `react-router-dom`
- `@/lib/icons`
- `../../hooks/useAuth`
- `../../store/auth.store`
- `../../store/credentials.store`
- `../../hooks/useDevices`
- `../ui/Tooltip`
- `../../lib/platform`
- `../../store/player.store`
- `../../store/prefs.store`
- `../../store/ui.store`
- `./TopSearch`
- `../../lib/motion`

## src/components/layout/TopSearch.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../../hooks/useSearch`
- `../../store/player.store`
- `../../api/playback`
- `../ui/CoverArt`
- `../../utils/fmt`
- `../../lib/platform`
- `../../lib/motion`
- `../../utils/episode`

## src/components/playground/AnimatedIcons.tsx

- `framer-motion`
- `@/lib/icons`

## src/components/playground/PlaygroundCards.tsx

- `react`
- `framer-motion`
- `../ui/CirclePlayButton`

## src/components/playground/PlaygroundControls.tsx

- `react`
- `framer-motion`
- `./AnimatedIcons`
- `../../styles/ui.css`

## src/components/playground/PlaygroundDials.tsx

- `react`
- `framer-motion`
- `./AnimatedIcons`

## src/components/playground/PlaygroundThemeBar.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../lib/color`

## src/components/playground/PlaygroundTracks.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `./AnimatedIcons`

## src/components/podcast/PodcastStage.tsx

- `react`
- `framer-motion`
- `@tanstack/react-query`
- `../../api/podcasts`
- `../../lib/coverUrl`
- `../../lib/motion`
- `../../store/prefs.store`
- `../../utils/episode`
- `../ui/EpisodeRow`
- `../../types/spotify`

## src/components/podcast/TranscriptView.tsx

- `react`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../api/podcasts`
- `../../store/player.store`
- `../../hooks/usePlayerControls`
- `../../utils/episode`
- `../../lib/motion`
- `../../types/podcast`

## src/components/podcast/transcript.test.ts

- `vitest`
- `./TranscriptView`
- `../../types/podcast`

## src/components/ui/AddToPlaylistModal.tsx

- `react`
- `../../lib/coverUrl`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../store/addToPlaylist.store`
- `../../hooks/useLibrary`
- `../../api/library`
- `../../store/toast.store`
- `./Modal`

## src/components/ui/AlbumCard.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@tanstack/react-query`
- `../../types/spotify`
- `./CoverArt`
- `./ReleaseCountdown`
- `../../utils/fmt`
- `../../lib/prefetch`
- `./CirclePlayButton`
- `../../store/player.store`
- `../../store/queue.store`
- `../../api/spotify`
- `../../api/playback`
- `../../hooks/usePlayerControls`
- `../../lib/motion`
- `../../styles/ui.css`
- `../../hooks/useReflowPulse`
- `../../lib/layout`

## src/components/ui/ArtistAbout.tsx

- `react`
- `framer-motion`
- `@tauri-apps/plugin-opener`
- `@/lib/icons`
- `./CoverArt`
- `./SectionTitle`
- `./Modal`
- `../../lib/coverUrl`
- `../../lib/motion`

## src/components/ui/ArtistCard.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `@tanstack/react-query`
- `./CoverArt`
- `../../lib/prefetch`
- `../../lib/motion`
- `../../styles/ui.css`
- `../../hooks/useReflowPulse`
- `../../lib/layout`

## src/components/ui/ArtistTour.tsx

- `@tauri-apps/plugin-opener`
- `framer-motion`
- `@/lib/icons`
- `./SectionTitle`
- `./CoverArt`
- `./Shelf`
- `../../lib/motion`
- `../../api/internal`

## src/components/ui/Carousel.tsx

- `react`
- `@/lib/icons`
- `../../lib/layout`

## src/components/ui/CirclePlayButton.tsx

- `framer-motion`
- `../playground/AnimatedIcons`
- `@/lib/motion`

## src/components/ui/ConfirmDialog.tsx

- `react`
- `./Modal`

## src/components/ui/ContextMenu.tsx

- `react`
- `react-dom`
- `framer-motion`
- `@/lib/motion`
- `@/hooks/useMenuKeyboard`

## src/components/ui/CoverArt.tsx

- `@/lib/icons`
- `../../lib/coverUrl`
- `react`

## src/components/ui/CreditsList.tsx

- `react-router-dom`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../api/internal`
- `./CoverArt`
- `../../lib/mesh`
- `../../lib/err`

## src/components/ui/CreditsModal.tsx

- `@/lib/icons`
- `./Modal`
- `./CreditsList`
- `./CoverArt`
- `../../store/credits.store`

## src/components/ui/Dropdown.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../lib/motion`
- `@/hooks/useMenuKeyboard`

## src/components/ui/EditPlaylistModal.tsx

- `react`
- `@/lib/icons`
- `./Tooltip`
- `../playground/PlaygroundControls`
- `./Modal`
- `./CoverArt`
- `../../api/library`
- `../../store/toast.store`
- `../../lib/err`
- `../../types/spotify`

## src/components/ui/EmptyState.tsx

- `framer-motion`

## src/components/ui/EpisodeRow.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../playground/AnimatedIcons`
- `./ContextMenu`
- `./Tooltip`
- `../../lib/coverUrl`
- `../../utils/fmt`
- `../../utils/episode`
- `../../store/queue.store`
- `../../lib/share`
- `../../store/toast.store`
- `../../lib/motion`
- `../../types/podcast`

## src/components/ui/EvenGrid.tsx

- `react`
- `framer-motion`
- `../../lib/motion`
- `../../lib/railFlip`

## src/components/ui/ExpandableDescription.tsx

- `react`
- `../../utils/fmt`

## src/components/ui/HomeFeedShelves.tsx

- `@tanstack/react-query`
- `../../api/internal`
- `./Shelf`
- `./MediaTile`

## src/components/ui/Loader.tsx


## src/components/ui/MediaTile.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `./CoverArt`
- `../../lib/motion`
- `../../hooks/useReflowPulse`
- `../../lib/layout`

## src/components/ui/Modal.tsx

- `react`
- `framer-motion`
- `@/lib/motion`
- `@/hooks/useModalA11y`

## src/components/ui/MusiqueLogo.tsx


## src/components/ui/OverlayScrollbar.tsx

- `react`

## src/components/ui/PageHeader.tsx

- `react`
- `./CoverArt`
- `../../store/ui.store`

## src/components/ui/PlayActions.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../store/player.store`
- `../../store/queue.store`
- `../../store/pins.store`
- `../../store/speedDial.store`
- `../../store/prefs.store`
- `../../api/playback`
- `../../api/connect`
- `../../store/toast.store`
- `../../types/spotify`
- `../../lib/motion`
- `../../styles/ui.css`
- `./ContextMenu`
- `../../lib/share`
- `./Tooltip`
- `../playground/AnimatedIcons`

## src/components/ui/QuitConfirm.tsx

- `react`
- `@tauri-apps/api/window`
- `../../store/ui.store`
- `./Modal`

## src/components/ui/ReleaseCountdown.tsx

- `react`
- `@/lib/icons`
- `../../utils/fmt`

## src/components/ui/SaveAlbumButton.tsx

- `framer-motion`
- `@/lib/icons`
- `./Tooltip`
- `../../hooks/useLibrary`
- `../../lib/motion`
- `../../store/toast.store`
- `../../lib/err`

## src/components/ui/SectionTitle.tsx


## src/components/ui/Shelf.tsx

- `./SectionTitle`
- `./Carousel`

## src/components/ui/SignInPrompt.tsx

- `react`
- `@tauri-apps/plugin-opener`
- `../../lib/icons`
- `../../lib/keyLayers`
- `../../api/auth`
- `../../hooks/useAuth`
- `./MusiqueLogo`
- `./SignInPrompt.css`

## src/components/ui/Skeletons.tsx

- `../../lib/layout`

## src/components/ui/SortDirectionToggle.tsx

- `framer-motion`
- `@/lib/icons`
- `../../lib/motion`
- `./Tooltip`

## src/components/ui/SortToolbar.tsx

- `react`
- `@/lib/icons`
- `./Dropdown`
- `./SortDirectionToggle`

## src/components/ui/SpeedControl.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../../store/prefs.store`
- `../../lib/motion`

## src/components/ui/Switch.tsx

- `framer-motion`
- `../../lib/motion`

## src/components/ui/SyncButton.tsx

- `react`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../hooks/useLibrary`
- `../../store/toast.store`
- `../../lib/err`

## src/components/ui/Toaster.tsx

- `framer-motion`
- `@/lib/icons`
- `../../store/toast.store`

## src/components/ui/Tooltip.tsx

- `react`
- `react-dom`
- `framer-motion`
- `@/lib/motion`

## src/components/ui/TrackRow.tsx

- `react`
- `../../lib/coverUrl`
- `framer-motion`
- `@/lib/icons`
- `../../store/credits.store`
- `../../utils/radio`
- `react-router-dom`
- `@tanstack/react-query`
- `../../types/spotify`
- `../../utils/fmt`
- `../../lib/motion`
- `../../lib/prefetch`
- `./ContextMenu`
- `../../lib/share`
- `../../store/addToPlaylist.store`
- `../../store/ytMatch.store`
- `../../hooks/usePlaybackBackend`
- `../../store/player.store`
- `../../store/queue.store`
- `../../store/toast.store`
- `../../hooks/usePlayerControls`
- `../playground/AnimatedIcons`
- `./Tooltip`
- `../../styles/ui.css`
- `../../utils/episode`
- `./queueMembership`

## src/components/ui/TrackToolbar.tsx

- `react`
- `@/lib/icons`
- `../../types/spotify`
- `./Dropdown`
- `./SortDirectionToggle`
- `./Tooltip`

## src/components/ui/YtMatchModal.tsx

- `react`
- `@tanstack/react-query`
- `@/lib/icons`
- `../../store/ytMatch.store`
- `../../api/playback`
- `../../store/toast.store`
- `../../lib/err`
- `../../utils/fmt`
- `./Modal`

## src/components/ui/queueMembership.test.ts

- `vitest`
- `./queueMembership`

## src/components/ui/queueMembership.ts


## src/hooks/useAlbum.ts

- `@tanstack/react-query`
- `../api/spotify`

## src/hooks/useAmbient.ts

- `react`
- `../lib/coverUrl`
- `../lib/ambient`

## src/hooks/useArtist.ts

- `@tanstack/react-query`
- `../api/spotify`

## src/hooks/useAuth.ts

- `@tanstack/react-query`
- `../api/auth`
- `../lib/authSession`
- `../lib/err`
- `../store/toast.store`
- `../store/auth.store`

## src/hooks/useCanvas.ts

- `@tanstack/react-query`
- `../api/internal`
- `../store/prefs.store`
- `../utils/episode`

## src/hooks/useDevices.ts

- `react`
- `@tauri-apps/api/event`
- `../store/player.store`
- `../store/auth.store`
- `../api/connect`
- `../api/playback`
- `../store/toast.store`
- `../lib/err`

## src/hooks/useEvenColumns.test.ts

- `vitest`
- `./useEvenColumns`

## src/hooks/useEvenColumns.ts

- `react`
- `../lib/railFlip`

## src/hooks/useJamSync.ts

- `react`
- `@tauri-apps/api/event`
- `@tanstack/react-query`
- `../api/social`
- `../api/playback`
- `../store/jam.store`
- `../store/auth.store`
- `../store/toast.store`
- `../lib/jam`

## src/hooks/useLibrary.ts

- `react`
- `@tanstack/react-query`
- `../api/library`
- `../types/library`

## src/hooks/useLyricFollow.ts

- `react`
- `../lib/scrollSpring`

## src/hooks/useLyrics.ts

- `../utils/episode`
- `react`
- `@tanstack/react-query`
- `../api/lyrics`
- `../lib/lyricsUpgrades`
- `../types/spotify`

## src/hooks/useMenuKeyboard.ts

- `react`
- `../lib/keyLayers`

## src/hooks/useModalA11y.ts

- `react`
- `../lib/keyLayers`

## src/hooks/usePlayEpisodes.ts

- `react`
- `../store/player.store`
- `../store/queue.store`
- `../api/playback`
- `./usePlayerControls`
- `../utils/episode`
- `../store/toast.store`
- `../lib/err`
- `../types/podcast`

## src/hooks/usePlaybackBackend.ts

- `@tanstack/react-query`
- `../api/playback`

## src/hooks/usePlayerControls.ts

- `react`
- `../store/player.store`
- `../store/queue.store`
- `../api/playback`
- `../api/connect`
- `../utils/radio`
- `../utils/listenTracker`
- `../store/toast.store`
- `../store/jam.store`
- `../lib/jam`

## src/hooks/usePlaylist.ts

- `@tanstack/react-query`
- `../api/spotify`

## src/hooks/usePlaylistFolders.ts

- `@tanstack/react-query`
- `../api/internal`
- `../store/auth.store`

## src/hooks/usePositionTicker.ts

- `react`
- `../store/player.store`

## src/hooks/useReflowPulse.ts

- `react`
- `../store/ui.store`

## src/hooks/useSearch.ts

- `@tanstack/react-query`
- `../api/spotify`

## src/hooks/useVideoAmbient.ts

- `react`
- `../lib/ambient`

## src/hooks/useWindowActive.ts

- `react`

## src/lib/ambient.ts

- `./color`

## src/lib/asyncListeners.test.ts

- `vitest`
- `./asyncListeners`

## src/lib/asyncListeners.ts


## src/lib/authSession.test.ts

- `vitest`
- `@tanstack/react-query`
- `../api/auth`
- `../api/library`
- `./authSession`
- `../store/auth.store`
- `../store/player.store`
- `../store/queue.store`
- `../store/jam.store`
- `../types/ipc`
- `../types/library`
- `../types/spotify`

## src/lib/authSession.ts

- `@tanstack/react-query`
- `../api/auth`
- `../api/library`
- `../store/auth.store`
- `../store/player.store`
- `../store/queue.store`
- `../store/pins.store`
- `../store/speedDial.store`
- `../store/jam.store`
- `../types/ipc`

## src/lib/backdrop.test.ts

- `vitest`
- `./backdrop`

## src/lib/backdrop.ts

- `../api/window`

## src/lib/color.ts

- `../api/internal`

## src/lib/coverUrl.test.ts

- `vitest`
- `./coverUrl`

## src/lib/coverUrl.ts


## src/lib/err.ts


## src/lib/icons.tsx

- `react`

## src/lib/jam.ts

- `../types/spotify`
- `../api/playback`
- `../api/internal`
- `../api/social`
- `../store/jam.store`
- `../store/player.store`
- `../store/queue.store`
- `../store/toast.store`
- `./err`
- `../utils/episode`

## src/lib/keyLayers.ts


## src/lib/layout.ts

- `react`

## src/lib/lyrics.tsx

- `react`
- `react`
- `../api/lyrics`
- `../store/player.store`
- `./outputLatency`
- `../hooks/useWindowActive`

## src/lib/lyricsRows.test.ts

- `vitest`
- `./lyricsRows`
- `../api/lyrics`

## src/lib/lyricsRows.ts

- `../api/lyrics`

## src/lib/lyricsSource.ts


## src/lib/lyricsUpgrades.test.ts

- `vitest`
- `@tanstack/react-query`
- `../api/lyrics`
- `dynamic "./lyricsUpgrades"`

## src/lib/lyricsUpgrades.ts

- `@tauri-apps/api/event`
- `@tanstack/react-query`
- `../api/lyrics`

## src/lib/lyricsWeld.test.ts

- `vitest`
- `./lyrics`

## src/lib/mesh.ts


## src/lib/motion.ts

- `react`
- `framer-motion`

## src/lib/outputLatency.ts

- `@tauri-apps/api/core`

## src/lib/persistStorage.test.ts

- `vitest`
- `zustand/vanilla`
- `zustand/middleware`
- `./persistStorage`

## src/lib/persistStorage.ts

- `zustand/middleware`

## src/lib/platform.ts


## src/lib/prefetch.ts

- `../utils/episode`
- `@tanstack/react-query`
- `../api/spotify`
- `../api/lyrics`
- `../types/spotify`

## src/lib/railFlip.ts

- `react`
- `react-dom`
- `./motion`

## src/lib/rootlist.test.ts

- `vitest`
- `./rootlist`
- `../api/internal`

## src/lib/rootlist.ts

- `../api/internal`

## src/lib/scrollSpring.ts


## src/lib/share.ts

- `../api/share`
- `../store/toast.store`

## src/lib/updater.ts

- `@tauri-apps/plugin-updater`
- `@tauri-apps/plugin-process`
- `../store/updater.store`

## src/lib/zoom.ts

- `@tauri-apps/api/webview`
- `../store/prefs.store`
- `../store/toast.store`
- `./platform`

## src/main.tsx

- `react`
- `react-dom/client`
- `@tanstack/react-query`
- `./App`
- `./components/ErrorBoundary`
- `./index.css`
- `./lib/zoom`

## src/pages/AlbumPage.tsx

- `react`
- `../lib/coverUrl`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../utils/radio`
- `../hooks/useAlbum`
- `../hooks/useArtist`
- `../components/ui/AlbumCard`
- `../components/ui/TrackRow`
- `../components/ui/PlayActions`
- `../components/ui/SaveAlbumButton`
- `../components/ui/PageHeader`
- `../components/ui/ExpandableDescription`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../components/ui/SectionTitle`
- `../components/ui/Carousel`
- `../components/ui/TrackToolbar`
- `../utils/fmt`
- `../api/playback`
- `../store/player.store`
- `../store/queue.store`
- `../store/pins.store`
- `../store/speedDial.store`
- `../hooks/useLibrary`
- `../components/ui/ContextMenu`
- `../lib/err`
- `../hooks/useReflowPulse`

## src/pages/ArtistPage.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../utils/radio`
- `../hooks/useArtist`
- `@tanstack/react-query`
- `../api/internal`
- `../components/ui/MediaTile`
- `../components/ui/ArtistTour`
- `../components/ui/ArtistAbout`
- `../components/ui/AlbumCard`
- `../components/ui/ArtistCard`
- `../components/ui/TrackRow`
- `../components/ui/PageHeader`
- `../components/ui/PlayActions`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../components/ui/SectionTitle`
- `../components/ui/Shelf`
- `../components/ui/ContextMenu`
- `../components/ui/Tooltip`
- `../lib/share`
- `../hooks/useLibrary`
- `../store/player.store`
- `../store/queue.store`
- `../store/speedDial.store`
- `../api/playback`
- `../lib/motion`
- `../lib/err`
- `../hooks/useReflowPulse`

## src/pages/Home.tsx

- `react`
- `../lib/coverUrl`
- `react-router-dom`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../hooks/useAuth`
- `../api/spotify`
- `../api/library`
- `../hooks/useLibrary`
- `../store/player.store`
- `../store/queue.store`
- `../store/speedDial.store`
- `../api/playback`
- `../hooks/usePlayerControls`
- `../components/ui/Loader`
- `../components/ui/SignInPrompt`
- `../components/ui/SectionTitle`
- `../components/ui/Carousel`
- `../components/ui/ArtistCard`
- `../components/ui/AlbumCard`
- `../components/ui/EvenGrid`
- `../components/ui/CirclePlayButton`
- `../components/playground/PlaygroundControls`
- `../lib/mesh`
- `../lib/motion`
- `../hooks/useReflowPulse`
- `../types/spotify`
- `../types/library`
- `../components/ui/HomeFeedShelves`
- `../lib/layout`
- `../lib/railFlip`
- `react-dom`

## src/pages/Library.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `@tanstack/react-query`
- `../api/podcasts`
- `../components/ui/MediaTile`
- `../hooks/useAuth`
- `../components/ui/EmptyState`
- `../components/ui/SignInPrompt`
- `../components/ui/Skeletons`
- `../components/ui/SyncButton`
- `../hooks/useLibrary`
- `../components/ui/AlbumCard`
- `../components/ui/ArtistCard`
- `../components/ui/TrackRow`
- `../components/ui/SortToolbar`
- `../store/player.store`
- `../store/queue.store`
- `../api/playback`
- `../store/speedDial.store`
- `../hooks/useReflowPulse`

## src/pages/Playground.tsx

- `react`
- `framer-motion`
- `@/lib/icons`
- `../hooks/useReflowPulse`
- `../components/playground/PlaygroundThemeBar`
- `../components/playground/PlaygroundControls`
- `../components/playground/PlaygroundCards`
- `../components/playground/PlaygroundTracks`
- `../components/playground/PlaygroundDials`

## src/pages/PlaylistPage.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../utils/radio`
- `../hooks/usePlaylist`
- `../components/ui/TrackRow`
- `../components/ui/PlayActions`
- `../components/ui/PageHeader`
- `../components/ui/ExpandableDescription`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../components/ui/TrackToolbar`
- `../api/playback`
- `../store/player.store`
- `../store/queue.store`
- `../store/pins.store`
- `../store/speedDial.store`
- `../store/auth.store`
- `../hooks/useLibrary`
- `../components/ui/ContextMenu`
- `@tanstack/react-query`
- `../api/library`
- `../components/ui/EditPlaylistModal`
- `../components/ui/ConfirmDialog`
- `../components/ui/Tooltip`
- `../lib/motion`
- `../store/toast.store`
- `../types/spotify`
- `../lib/err`
- `../hooks/useReflowPulse`

## src/pages/Playlists.tsx

- `react`
- `../lib/coverUrl`
- `react-router-dom`
- `framer-motion`
- `@/lib/icons`
- `../hooks/usePlaylistFolders`
- `../lib/rootlist`
- `../api/internal`
- `../components/ui/Shelf`
- `../components/ui/MediaTile`
- `../components/ui/SectionTitle`
- `@tanstack/react-query`
- `../api/spotify`
- `../hooks/useAuth`
- `../components/ui/EmptyState`
- `../components/ui/SignInPrompt`
- `../components/ui/SyncButton`
- `../components/ui/Skeletons`
- `../hooks/useLibrary`
- `../store/pins.store`
- `../components/ui/ContextMenu`
- `../types/library`
- `../hooks/useReflowPulse`
- `../lib/motion`
- `../lib/layout`

## src/pages/ProfilePage.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../api/profile`
- `../components/ui/PageHeader`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../components/ui/Shelf`
- `../components/ui/MediaTile`
- `../components/ui/ArtistCard`
- `../components/ui/SectionTitle`
- `../components/ui/TrackRow`
- `../components/ui/Tooltip`
- `../components/ui/ContextMenu`
- `../hooks/useLibrary`
- `../store/auth.store`
- `../store/player.store`
- `../store/queue.store`
- `../api/playback`
- `../lib/share`
- `../store/toast.store`
- `../lib/err`
- `../lib/motion`
- `../hooks/useReflowPulse`

## src/pages/Search.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `../hooks/useSearch`
- `../hooks/useAuth`
- `../components/ui/EmptyState`
- `../components/ui/SignInPrompt`
- `../components/ui/SectionTitle`
- `../components/ui/Skeletons`
- `../components/ui/AlbumCard`
- `../components/ui/ArtistCard`
- `../components/ui/Shelf`
- `../components/ui/CoverArt`
- `../components/ui/MediaTile`
- `../components/ui/EpisodeRow`
- `../hooks/usePlayEpisodes`
- `../utils/episode`
- `../types/podcast`
- `../components/ui/TrackRow`
- `../components/playground/PlaygroundControls`
- `../components/playground/AnimatedIcons`
- `../api/spotify`
- `../api/playback`
- `../hooks/usePlayerControls`
- `../store/player.store`
- `../store/queue.store`
- `../store/ui.store`
- `../hooks/useLibrary`
- `../lib/coverUrl`
- `../utils/fmt`
- `../lib/err`
- `../store/toast.store`
- `../hooks/useReflowPulse`
- `../lib/motion`
- `../types/spotify`
- `../lib/layout`

## src/pages/Settings.tsx

- `react`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../lib/zoom`
- `../components/playground/PlaygroundControls`
- `../components/ui/Tooltip`
- `../api/credentials`
- `../store/credentials.store`
- `../store/prefs.store`
- `../store/player.store`
- `../api/playback`
- `../hooks/usePlaybackBackend`
- `../store/toast.store`
- `../lib/err`
- `../store/ui.store`
- `../api/media`
- `../api/lastfm`
- `../api/window`
- `../lib/platform`
- `../store/theme.store`
- `../hooks/useReflowPulse`
- `../lib/motion`
- `../styles/ui.css`
- `../components/ui/Switch`

## src/pages/ShowPage.tsx

- `react`
- `react-router-dom`
- `framer-motion`
- `@tanstack/react-query`
- `@/lib/icons`
- `../api/podcasts`
- `../components/ui/PageHeader`
- `../components/ui/PlayActions`
- `../components/ui/ExpandableDescription`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../components/ui/EpisodeRow`
- `../components/ui/Dropdown`
- `../components/ui/Tooltip`
- `../components/ui/ContextMenu`
- `../hooks/usePlayEpisodes`
- `../store/player.store`
- `../lib/share`
- `../store/toast.store`
- `../lib/err`
- `../utils/episode`
- `../api/playback`
- `../lib/motion`
- `../hooks/useReflowPulse`
- `../types/podcast`
- `../types/spotify`

## src/pages/StatsPage.tsx

- `react`
- `../hooks/useEvenColumns`
- `react-router-dom`
- `../store/prefs.store`
- `@tanstack/react-query`
- `@/lib/icons`
- `../api/stats`
- `../components/playground/PlaygroundControls`
- `../components/ui/SectionTitle`
- `../components/ui/Shelf`
- `../components/ui/ArtistCard`
- `../components/ui/AlbumCard`
- `../components/ui/CoverArt`
- `../components/ui/Loader`
- `../components/ui/EmptyState`
- `../store/player.store`
- `../store/queue.store`
- `../api/playback`
- `../lib/err`
- `../utils/stats`
- `../hooks/useReflowPulse`
- `../types/spotify`

## src/store/addToPlaylist.store.ts

- `zustand`
- `../types/spotify`

## src/store/auth.store.ts

- `zustand`
- `../types/ipc`
- `../api/auth`

## src/store/credentials.store.ts

- `zustand`
- `../api/credentials`

## src/store/credits.store.ts

- `zustand`
- `../types/spotify`

## src/store/jam.store.ts

- `zustand`
- `../api/social`
- `../api/playback`
- `../types/spotify`

## src/store/pins.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`

## src/store/player.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`
- `../types/spotify`
- `../api/connect`

## src/store/prefs.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`
- `../api/playback`
- `../api/media`

## src/store/queue.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`
- `../types/spotify`
- `./player.store`
- `../api/connect`
- `./toast.store`
- `./jam.store`
- `../lib/jam`

## src/store/speedDial.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`
- `../types/spotify`

## src/store/theme.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`

## src/store/toast.store.ts

- `zustand`

## src/store/ui.store.ts

- `zustand`
- `zustand/middleware`
- `../lib/persistStorage`
- `../api/window`

## src/store/updater.store.ts

- `zustand`

## src/store/ytMatch.store.ts

- `zustand`

## src/types/ipc.ts


## src/types/library.ts


## src/types/podcast.ts


## src/types/spotify.ts

- `./podcast`

## src/utils/episode.test.ts

- `vitest`
- `./episode`
- `../types/podcast`

## src/utils/episode.ts

- `../types/spotify`
- `../types/podcast`

## src/utils/fmt.test.ts

- `vitest`
- `./fmt`

## src/utils/fmt.ts


## src/utils/listenTracker.ts

- `../api/spotify`
- `../types/spotify`
- `./episode`

## src/utils/palette.ts


## src/utils/radio.ts

- `./episode`
- `../api/spotify`
- `../api/internal`
- `../store/toast.store`
- `../api/playback`
- `../store/player.store`
- `../store/queue.store`
- `../types/spotify`

## src/utils/romanize.ts

- `dynamic "pinyin-pro"`
- `dynamic "wanakana"`

## src/utils/stats.test.ts

- `vitest`
- `./stats`

## src/utils/stats.ts


## src/vite-env.d.ts


## vite.config.ts

- `vite`
- `@vitejs/plugin-react`
- `@tailwindcss/vite`
- `path`
