# Complete frontend render network

All 194 original TypeScript/TSX files were parsed. Every JSX tag/component occurrence is retained with its source line, including icons, conditional branches, skeletons, portals and demonstration components. These are static render edges; a listed branch need not mount at runtime. Follow component imports in dependency_map.md and exact event handlers in interaction_inventory.md. Manually traced state/workflows are in system_map.md and interaction_map.md.

## src/App.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| MotionConfig | 441 |
| BrowserRouter | 442 |
| AppInit | 443 |
| ThemeEngine | 444 |
| UpdatePrompt | 445 |
| Routes | 446 |
| Route | 447, 448, 451, 459, 468, 476, 484, 492, 500, 508, 516, 524, 532, 540, 548 |
| Layout | 447 |
| Home | 448 |
| Suspense | 454, 462, 471, 479, 487, 495, 503, 511, 519, 527, 535, 543 |
| Loader | 454, 462, 471, 479, 487, 495, 503, 511, 519, 527, 535, 543 |
| Settings | 455 |
| Search | 463 |
| Library | 472 |
| PlaylistPage | 480 |
| Playlists | 488 |
| ArtistPage | 496 |
| AlbumPage | 504 |
| ProfilePage | 512, 536 |
| ShowPage | 520 |
| StatsPage | 528 |
| Playground | 544 |
| Navigate | 548 |

## src/api/auth.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/connect.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/credentials.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/internal.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/lastfm.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/library.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/lyrics.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/lyrics.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/media.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/playback.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/podcasts.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/profile.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/settings.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/share.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/social.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/spotify.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/stats.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/theme.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/api/window.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/components/ErrorBoundary.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 32 |
| h1 | 41 |
| p | 42 |
| code | 46 |
| button | 55 |

## src/components/ThemeEngine.tsx

No JSX rendering. Pure service/hook/store/test/config module.

## src/components/UpdatePrompt.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 27 |
| div | 46, 73, 87, 104, 105, 148, 166 |
| AlertTriangle | 60 |
| RotateCw | 62 |
| Sparkles | 64 |
| h2 | 70, 100, 144, 162 |
| p | 71, 74, 101, 126, 145, 163 |
| button | 88, 91, 149, 167 |
| motion.div | 114 |

## src/components/auth/ProfileCard.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 14, 26, 34 |
| img | 19 |
| span | 35, 39, 44 |

## src/components/layout/DevicesPopover.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| AnimatePresence | 59 |
| DevicesPopoverCard | 60 |
| motion.div | 136 |
| div | 172, 191, 221, 233, 285, 286, 303, 335 |
| span | 181, 304, 319, 345 |
| button | 192, 207, 256 |
| RefreshCw | 200 |
| X | 215 |
| Icon | 301 |
| Volume2 | 329 |
| Loader2 | 337 |

## src/components/layout/FriendsPanel.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| span | 56, 58, 89, 94, 95, 106, 110, 112, 119, 122, 137, 170, 195, 198, 199 |
| p | 65, 190, 192 |
| div | 84, 100, 101, 169, 171, 187, 194 |
| Link | 85, 102, 117 |
| CoverArt | 87, 136 |
| Bars | 105 |
| ctx.Icon | 118 |
| Tooltip | 128, 172, 179 |
| button | 129, 173, 180 |
| Play | 138 |
| motion.div | 162 |
| motion.span | 174 |
| RefreshCw | 175 |
| X | 181 |
| JamCard | 188 |
| Users | 196 |
| Label | 207, 213 |
| FriendRow | 208, 214 |

## src/components/layout/Immersive.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.canvas | 100 |
| div | 140, 144, 145, 170, 249, 363, 364, 372, 432, 490, 516, 528, 529, 604, 618, 626, 660, 661, 679, 698 |
| AnimatePresence | 146, 250, 570, 680 |
| motion.div | 148, 572, 636, 681, 712 |
| LiveVideoRoom | 159 |
| motion.video | 252 |
| motion.img | 276 |
| Centered | 358, 359, 360, 502 |
| LyricRowText | 445 |
| ReturnPill | 464 |
| p | 517, 530, 531 |
| button | 519, 666, 738 |
| img | 527 |
| Music | 528 |
| AmbientBg | 593 |
| PodcastStage | 595 |
| Sleeve | 597 |
| PanelTab | 662, 663, 664 |
| Captions | 662 |
| Queue | 663 |
| Info | 664 |
| Minimize2 | 672 |
| TranscriptView | 692 |
| ImmersiveLyrics | 694 |
| EpisodeAbout | 696 |
| CreditsList | 699 |
| ImmersiveQueue | 702 |
| PlayerBar | 720 |
| span | 744 |

## src/components/layout/JamCard.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| span | 26, 56, 61, 155, 158, 235 |
| CoverArt | 45 |
| div | 54, 125, 134, 135, 138, 153, 191, 199, 233, 234, 238, 239, 240 |
| Face | 57, 154 |
| button | 126, 161, 212, 282, 317 |
| Faces | 133 |
| AnimatePresence | 142, 246 |
| motion.div | 144, 287 |
| Tooltip | 160 |
| X | 169 |
| label | 176 |
| input | 177, 272 |
| motion.button | 201, 295 |
| Link2 | 209 |
| Users | 236 |
| motion.form | 248 |
| ArrowRight | 283 |

## src/components/layout/Layout.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 226, 236, 252, 253, 257, 259, 282, 321, 337, 353 |
| Sidebar | 248 |
| TitleBar | 254 |
| AnimatePresence | 296, 395, 398, 401 |
| motion.div | 298, 367, 385 |
| main | 354 |
| Outlet | 373 |
| PlayerBar | 381 |
| LyricsPanel | 396 |
| QueuePanel | 399 |
| FriendsPanel | 402 |
| DevicesPopover | 410 |
| Immersive | 411 |
| QuitConfirm | 412 |
| AddToPlaylistModal | 413 |
| CreditsModal | 414 |
| YtMatchModal | 415 |
| Toaster | 416 |
| WindowCaptionControls | 420 |

## src/components/layout/LyricReturnPill.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| AnimatePresence | 12 |
| motion.button | 14 |
| ArrowRight | 26 |

## src/components/layout/LyricsPanel.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| TranscriptPanel | 50 |
| SongLyricsPanel | 50 |
| motion.div | 58, 181 |
| div | 66, 69, 210, 224, 280, 312, 329, 375, 523, 535 |
| TranscriptView | 70 |
| CenterNote | 76, 300, 306 |
| Tooltip | 239, 250, 264 |
| Pill | 243, 254, 270 |
| Languages | 244, 271 |
| Globe | 258 |
| span | 272 |
| Loader | 298 |
| Music2 | 301 |
| RetryBtn | 309 |
| LyricRowText | 393 |
| p | 409, 412, 415, 425, 536, 546 |
| ReturnPill | 447 |
| motion.button | 482 |
| button | 554 |
| RefreshCw | 574 |

## src/components/layout/PlayerBar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.button | 50, 100, 873 |
| span | 89, 180, 349, 554, 662, 910, 911, 1120, 1133, 1152 |
| AnimatePresence | 119, 624, 868, 1049, 1179 |
| motion.span | 120 |
| Pause | 135 |
| Play | 136 |
| p | 175 |
| motion.div | 310, 357, 626, 1050 |
| div | 388, 552, 558, 594, 609, 849, 917, 953, 1010, 1025, 1062, 1079, 1094, 1099, 1149 |
| button | 400 |
| Icon | 424 |
| Devices | 909 |
| strong | 910 |
| Tooltip | 956, 962, 969, 975, 982, 989, 1001, 1151, 1163 |
| IconBtn | 957, 963, 970, 983, 990, 1002, 1153 |
| SkipSeconds | 958, 997 |
| Shuffle | 964 |
| SkipBack | 971 |
| PlayPauseButton | 976 |
| SkipForward | 984 |
| RepeatIcon | 1003 |
| CoverArt | 1058 |
| Maximize2 | 1074 |
| Marquee | 1100, 1117 |
| Link | 1102, 1123 |
| PlayerScrubber | 1144 |
| SpeedControl | 1150 |
| VolumeIcon | 1158 |
| input | 1164 |
| VolumePopover | 1181 |

## src/components/layout/QueuePanel.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| img | 41, 67 |
| div | 49, 85, 112, 166, 175, 190, 235, 285, 290, 358, 360, 374, 385, 436 |
| Tooltip | 56, 123, 195, 362 |
| span | 57, 103, 135, 146, 148, 361, 387 |
| GripVertical | 108 |
| Cover | 111, 174 |
| p | 113, 116, 176, 179, 191, 211 |
| AddedBy | 121 |
| button | 124, 196, 363 |
| X | 131, 369 |
| Equaliser | 183 |
| SectionHead | 227, 281, 289, 376, 383, 435 |
| EmptyRow | 229, 231, 233, 283, 293, 379 |
| QueueTrackRow | 237, 271, 420, 438 |
| motion.div | 340 |
| NowPlayingCard | 378 |
| JamQueue | 382 |
| RemoteQueue | 382 |
| Queue | 386 |
| Reorder.Group | 395 |
| AnimatePresence | 402 |
| Reorder.Item | 404 |

## src/components/layout/Sidebar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.button | 47, 566 |
| motion.div | 82, 185, 272, 607 |
| span | 95, 106, 260, 516, 521, 550, 551, 554, 555, 605, 620, 630 |
| Icon | 103 |
| Tooltip | 112, 638 |
| div | 225, 227, 243, 279, 451, 457, 468, 477, 478, 497, 498, 624 |
| button | 244, 535 |
| motion.span | 261 |
| ChevronDown | 267, 556 |
| AnimatePresence | 270 |
| motion.nav | 425 |
| RailIndicator | 479 |
| Section | 480, 486, 493 |
| NavItem | 481, 482, 483, 487, 488, 489, 490 |
| ListMusic | 515, 625 |
| Pin | 520 |
| Folder | 552 |
| PinOff | 572 |
| img | 622 |
| OverlayScrollbar | 645 |

## src/components/layout/TitleBar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| button | 46, 92, 155 |
| Tooltip | 91, 388, 395, 409 |
| span | 168, 169, 264, 316, 317, 318, 474, 494, 556 |
| div | 239, 304, 308, 312, 325, 330, 370, 449, 467, 471, 514, 524 |
| CapsuleButton | 240, 476, 485, 498, 525, 536, 546, 558 |
| img | 255, 306 |
| User | 261, 309, 327 |
| AnimatePresence | 275 |
| motion.div | 277 |
| p | 313 |
| AccountMenuItem | 327, 328, 333, 335 |
| Settings | 328 |
| LogOut | 333 |
| LogIn | 335 |
| CaptionBtn | 389, 396, 410 |
| svg | 390, 398, 403, 411 |
| path | 391, 399, 412 |
| rect | 400, 404 |
| AccountMenu | 472 |
| ChevronLeft | 482 |
| ChevronRight | 491 |
| PanelLeft | 504 |
| PanelLeftClose | 505 |
| TopSearch | 510 |
| Captions | 533 |
| Queue | 543 |
| Users | 553 |
| Devices | 568 |

## src/components/layout/TopSearch.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 209, 210, 277, 291, 301, 310, 317, 320, 323, 328, 330, 333, 338, 349, 354, 356, 359, 368, 373, 375, 378, 386, 391, 393, 396, 408, 410, 413, 418 |
| Search | 214, 308 |
| input | 215 |
| button | 234 |
| X | 240 |
| kbd | 243 |
| AnimatePresence | 247 |
| motion.div | 249 |
| ArrowRight | 313 |
| CoverArt | 331, 357, 376, 394, 411 |
| Play | 334, 414 |
| p | 339, 340, 360, 361, 379, 397, 398, 419, 420 |
| span | 342, 363, 381, 400, 422 |

## src/components/playground/AnimatedIcons.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| span | 18, 30, 45, 76 |
| Pause | 43 |
| Play | 58 |
| Icon | 87 |
| motion.span | 103 |
| Heart | 118 |

## src/components/playground/PlaygroundCards.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.div | 21, 187, 254 |
| div | 44, 211, 227, 293 |
| img | 45, 276 |
| CirclePlayButton | 52, 330 |
| span | 63, 82, 316 |

## src/components/playground/PlaygroundControls.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.button | 25, 89 |
| AnimatedPlayPause | 53, 111 |
| span | 54, 295, 297 |
| div | 70, 211 |
| AnimatePresence | 71 |
| motion.div | 73, 277 |
| button | 242 |

## src/components/playground/PlaygroundDials.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 16, 38, 87 |
| span | 34, 142 |
| motion.div | 51 |
| motion.button | 101 |
| AnimatedVolumeIcon | 122 |
| input | 126 |

## src/components/playground/PlaygroundThemeBar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 29, 46, 47, 63, 64, 67, 73 |
| Sparkles | 61 |
| motion.button | 77 |
| span | 95, 109, 140 |
| Check | 107 |
| button | 122 |
| RefreshCw | 139 |

## src/components/playground/PlaygroundTracks.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 33, 50, 83, 84, 101, 120, 174 |
| motion.button | 52 |
| AnimatedPlayPause | 73 |
| span | 76, 162 |
| p | 85, 114 |
| motion.div | 103 |
| button | 121, 143 |
| AnimatedHeart | 140 |
| Plus | 159 |
| TrackRowItem | 176 |

## src/components/podcast/PodcastStage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 34, 85, 86, 89, 103, 104 |
| AnimatePresence | 53 |
| motion.video | 55 |
| motion.img | 73 |
| p | 105 |

## src/components/podcast/TranscriptView.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 90, 99, 109, 227, 230, 234, 235 |
| span | 91, 92, 93, 110, 111, 118 |
| p | 114 |
| Paragraph | 257 |
| AnimatePresence | 272 |
| motion.button | 274 |
| ArrowRight | 304 |

## src/components/podcast/transcript.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/components/ui/AddToPlaylistModal.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 61 |
| div | 71, 80, 81, 111 |
| h2 | 72 |
| button | 75, 112, 125 |
| X | 76 |
| Search | 82 |
| input | 83 |
| span | 113, 114, 128, 129, 130, 131 |
| Plus | 113 |
| p | 120, 122 |
| img | 127 |
| ListMusic | 128 |

## src/components/ui/AlbumCard.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| LayoutGroup | 33 |
| motion.div | 34 |
| MotionLink | 89 |
| div | 126, 174 |
| CoverArt | 138 |
| CirclePlayButton | 141 |
| span | 153, 188 |
| ReleaseCountdown | 186 |

## src/components/ui/ArtistAbout.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 37, 63, 71, 72, 82, 83, 84, 95, 110, 115, 122, 123, 135, 136, 158, 191, 198, 200, 217, 218, 219, 224 |
| motion.button | 39, 159 |
| ArrowUpRight | 49 |
| ol | 60 |
| li | 62 |
| span | 64, 65, 67, 69, 193, 194, 195, 201, 202, 210 |
| Modal | 94 |
| AnimatePresence | 96 |
| motion.img | 98, 181 |
| button | 111, 116, 117 |
| X | 112 |
| ChevronLeft | 116 |
| ChevronRight | 117, 211 |
| h2 | 120 |
| StatTile | 124, 125, 126, 129, 130, 225, 226 |
| p | 133, 206 |
| Cities | 137, 222 |
| Links | 140, 228 |
| section | 156 |
| SectionTitle | 157 |
| CoverArt | 189 |
| Globe | 220 |
| AboutModal | 232 |

## src/components/ui/ArtistCard.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| LayoutGroup | 26 |
| motion.div | 27, 98, 154 |
| MotionLink | 45 |
| div | 83, 111, 130, 141, 183, 205 |
| CoverArt | 103 |
| User | 125 |
| AnimatePresence | 152 |
| ArrowUpRight | 176 |
| span | 184, 206, 216 |

## src/components/ui/ArtistTour.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| section | 27 |
| SectionTitle | 28 |
| div | 29, 33, 37, 38, 39, 41, 42, 43, 44, 68 |
| Shelf | 52 |
| motion.button | 58 |
| CoverArt | 69 |
| span | 71, 76, 77 |
| ArrowUpRight | 72 |

## src/components/ui/Carousel.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 80, 134, 157 |
| button | 81, 91 |
| ChevronLeft | 89 |
| ChevronRight | 99 |

## src/components/ui/CirclePlayButton.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.button | 27 |
| svg | 56 |
| circle | 62 |
| AnimatedPlayPause | 69 |

## src/components/ui/ConfirmDialog.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 27 |
| h2 | 34 |
| div | 37, 38 |
| button | 39, 42 |

## src/components/ui/ContextMenu.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| ContextMenuView | 50 |
| motion.div | 102 |
| MenuRow | 127 |
| button | 136 |
| span | 150, 151 |

## src/components/ui/CoverArt.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| FadeImg | 35 |
| div | 47 |
| span | 61, 109 |
| Music | 64 |
| img | 119 |

## src/components/ui/CreditsList.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| CoverArt | 11 |
| span | 15, 34, 35, 45, 46, 47, 48, 86, 87, 89, 91 |
| div | 42, 44, 77, 81, 107 |
| Skeleton | 70 |
| p | 71, 73, 114 |
| section | 79 |
| h3 | 80 |
| Avatar | 85 |
| Link | 102 |
| ChevronRight | 104 |

## src/components/ui/CreditsModal.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 11 |
| div | 19, 20, 23, 24, 28, 36 |
| CoverArt | 21 |
| h2 | 25 |
| button | 32 |
| X | 33 |
| CreditsList | 37 |

## src/components/ui/Dropdown.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 44 |
| motion.button | 45 |
| span | 71, 117 |
| motion.span | 72 |
| ChevronDown | 73 |
| AnimatePresence | 77 |
| motion.div | 79 |
| button | 103 |
| Check | 118 |

## src/components/ui/EditPlaylistModal.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| label | 47 |
| span | 48, 49, 50, 201, 205, 207, 212, 236 |
| Modal | 130 |
| form | 137 |
| div | 149, 158, 159, 173, 199, 200, 217, 227, 239 |
| h2 | 150 |
| button | 153, 240, 241 |
| X | 154 |
| CoverArt | 170 |
| Field | 174, 185 |
| input | 175 |
| textarea | 186 |
| Tooltip | 202 |
| b | 207 |
| Info | 213 |
| SegmentedControl | 218 |

## src/components/ui/EmptyState.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.div | 23 |
| div | 41, 79 |
| h2 | 55 |
| p | 67, 81 |

## src/components/ui/EpisodeRow.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Play | 53 |
| Plus | 56 |
| Link2 | 62 |
| div | 67, 97, 122, 127 |
| motion.button | 99 |
| AnimatedPlayPause | 110 |
| span | 113, 133, 144, 151, 157, 158, 164 |
| Mic | 115, 123 |
| img | 120 |
| p | 128, 131 |
| Link | 138 |
| Tooltip | 150, 156 |
| Check | 152 |

## src/components/ui/EvenGrid.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| LayoutGroup | 178 |
| motion.div | 179, 196 |
| div | 273, 284 |

## src/components/ui/ExpandableDescription.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 27 |
| p | 28 |
| button | 50 |

## src/components/ui/HomeFeedShelves.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Shelf | 31 |
| MediaTile | 38 |

## src/components/ui/Loader.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 17 |
| svg | 32 |
| circle | 39, 43 |
| span | 50 |

## src/components/ui/MediaTile.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| MotionLink | 35 |
| div | 64 |
| CoverArt | 75 |
| span | 77, 95 |

## src/components/ui/Modal.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| AnimatePresence | 31 |
| motion.div | 33, 44 |

## src/components/ui/MusiqueLogo.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| svg | 13 |
| defs | 26 |
| linearGradient | 27 |
| stop | 28, 29, 30, 33, 34, 35 |
| radialGradient | 32 |
| path | 38, 42 |

## src/components/ui/OverlayScrollbar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 105, 116 |

## src/components/ui/PageHeader.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 39, 52, 66, 77 |
| CoverArt | 67 |
| p | 81 |
| h1 | 88 |

## src/components/ui/PlayActions.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Link2 | 88 |
| Globe | 89 |
| div | 164, 300 |
| Tooltip | 165, 230, 307, 334 |
| motion.button | 171, 231, 308, 335 |
| motion.span | 204, 214, 266, 284 |
| AnimatedPlayPause | 205 |
| AnimatePresence | 212, 282 |
| Shuffle | 280 |
| Pin | 329 |
| span | 330, 370 |
| Share2 | 369 |

## src/components/ui/QuitConfirm.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 16 |
| h2 | 24 |
| p | 27 |
| div | 30 |
| button | 33, 40 |

## src/components/ui/ReleaseCountdown.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| span | 18, 39 |
| Clock | 38 |

## src/components/ui/SaveAlbumButton.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Tooltip | 15 |
| motion.button | 16 |
| Heart | 43 |
| span | 44 |

## src/components/ui/SectionTitle.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 19, 45 |
| Tag | 31 |
| button | 53 |

## src/components/ui/Shelf.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| section | 28 |
| SectionTitle | 29 |
| CarouselControls | 34 |
| CarouselTrack | 40 |

## src/components/ui/SignInPrompt.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 75, 79, 81, 82, 130, 220, 226, 227, 247, 280, 296, 323 |
| h3 | 76, 324 |
| p | 77, 145, 153, 225, 297, 340, 347, 356, 362 |
| button | 83, 107, 132, 149, 228, 258, 302, 325 |
| span | 90, 93, 95, 97, 349, 350 |
| Globe | 91 |
| strong | 94, 339, 346, 355 |
| Check | 100, 138 |
| ArrowUpRight | 115 |
| input | 119 |
| Link2 | 140 |
| Loader2 | 146, 240 |
| h1 | 221 |
| section | 222, 318 |
| MusiqueLogo | 223 |
| h2 | 224 |
| ChevronDown | 273, 313 |
| LoginLinksPanel | 286 |
| X | 334 |
| ol | 337 |
| li | 338, 345, 354 |

## src/components/ui/Skeletons.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 12, 14, 26, 27, 28, 29, 30, 32, 51, 57, 58, 59, 60, 70, 78, 79, 80, 82, 83 |
| TrackRowsSkeleton | 84 |

## src/components/ui/SortDirectionToggle.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Tooltip | 20 |
| motion.button | 21 |
| motion.span | 36 |
| ChevronDown | 42 |

## src/components/ui/SortToolbar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 42, 43, 78 |
| Search | 52 |
| input | 53 |
| button | 66 |
| X | 73 |
| Dropdown | 79 |
| SortDirectionToggle | 80 |

## src/components/ui/SpeedControl.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 37, 87 |
| motion.button | 38 |
| AnimatePresence | 63 |
| motion.div | 65 |
| button | 89 |
| span | 116 |
| Check | 117 |
| svg | 130 |
| path | 132, 134 |
| text | 136 |

## src/components/ui/Switch.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| button | 15 |
| motion.div | 38 |

## src/components/ui/SyncButton.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 40 |
| span | 42 |
| button | 46 |
| RefreshCw | 57 |

## src/components/ui/Toaster.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 17 |
| AnimatePresence | 24 |
| motion.div | 28 |
| Icon | 50 |
| button | 53 |

## src/components/ui/Tooltip.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| span | 165 |
| AnimatePresence | 186 |
| motion.span | 188 |

## src/components/ui/TrackRow.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.button | 57, 201 |
| Play | 126 |
| Check | 129, 360 |
| Plus | 129, 371 |
| Heart | 134 |
| ListPlus | 137 |
| Trash2 | 140 |
| Radio | 145 |
| Info | 146 |
| User | 148 |
| Disc3 | 149 |
| Music | 157, 250 |
| Link2 | 161 |
| Globe | 162 |
| div | 168, 199, 249, 256, 257, 273, 275, 342 |
| AnimatedPlayPause | 224 |
| span | 232, 293, 298, 381 |
| img | 247 |
| p | 258, 291 |
| Link | 300, 313 |
| Tooltip | 327, 341 |
| ActionBtn | 328, 343 |
| AnimatedHeart | 335 |
| AnimatePresence | 350 |
| motion.span | 352, 363 |

## src/components/ui/TrackToolbar.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 73, 75, 111 |
| Search | 84 |
| input | 85 |
| Tooltip | 98 |
| button | 99 |
| X | 105 |
| Dropdown | 112 |
| SortDirectionToggle | 113 |

## src/components/ui/YtMatchModal.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Modal | 83 |
| div | 92, 93, 105, 106, 128, 132, 174 |
| h2 | 94 |
| p | 97, 134, 136, 138 |
| button | 101, 143, 175 |
| X | 101 |
| strong | 113 |
| span | 114, 152, 157, 158, 160, 162 |
| br | 115, 120 |
| Check | 154 |
| Music | 155 |
| RefreshCw | 176 |

## src/components/ui/queueMembership.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/components/ui/queueMembership.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useAlbum.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useAmbient.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useArtist.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useAuth.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useCanvas.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useDevices.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useEvenColumns.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useEvenColumns.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useJamSync.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useLibrary.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useLyricFollow.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useLyrics.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useMenuKeyboard.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useModalA11y.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePlayEpisodes.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePlaybackBackend.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePlayerControls.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePlaylist.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePlaylistFolders.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/usePositionTicker.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useReflowPulse.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useSearch.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useVideoAmbient.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/hooks/useWindowActive.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/ambient.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/asyncListeners.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/asyncListeners.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/authSession.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/authSession.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/backdrop.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/backdrop.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/color.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/coverUrl.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/coverUrl.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/err.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/icons.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| svg | 285, 319 |
| path | 286, 290, 291, 292, 320, 321 |
| mask | 289 |
| rect | 294 |
| Comp | 418 |

## src/lib/jam.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/keyLayers.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/layout.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyrics.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| p | 562 |
| span | 575 |

## src/lib/lyricsRows.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyricsRows.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyricsSource.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyricsUpgrades.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyricsUpgrades.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/lyricsWeld.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/mesh.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/motion.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/outputLatency.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/persistStorage.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/persistStorage.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/platform.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/prefetch.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/railFlip.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/rootlist.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/rootlist.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/scrollSpring.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/share.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/updater.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/lib/zoom.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/main.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| React.StrictMode | 25 |
| QueryClientProvider | 26 |
| ErrorBoundary | 27 |
| App | 28 |

## src/pages/AlbumPage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Loader | 60 |
| EmptyState | 64 |
| button | 68 |
| div | 91, 99, 107, 169 |
| Pin | 94 |
| Radio | 95 |
| PageHeader | 98 |
| img | 101 |
| span | 108, 110, 119, 120, 121 |
| Link | 112, 189 |
| ExpandableDescription | 129 |
| PlayActions | 131 |
| SaveAlbumButton | 131 |
| section | 134, 184 |
| motion.div | 136, 142 |
| TrackRow | 149 |
| p | 161, 170, 177 |
| SectionTitle | 185 |
| CarouselControls | 187 |
| CarouselTrack | 193 |
| AlbumCard | 199 |

## src/pages/ArtistPage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Loader | 76 |
| EmptyState | 80 |
| button | 84 |
| Radio | 101, 196 |
| Link2 | 102 |
| Globe | 103 |
| Tooltip | 118, 186 |
| motion.button | 119, 187 |
| Check | 141 |
| UserPlus | 141 |
| span | 142, 170, 172 |
| div | 148, 156, 204, 220 |
| PageHeader | 149 |
| p | 151 |
| PlayActions | 179 |
| section | 206 |
| SectionTitle | 207 |
| ShowAllButton | 211 |
| motion.div | 222 |
| TrackRow | 236 |
| Shelf | 253, 260, 267, 274, 281, 288 |
| AlbumCard | 258, 265, 272 |
| MediaTile | 279, 286 |
| ArtistCard | 293 |
| ArtistAbout | 295 |
| ArtistTour | 308 |

## src/pages/Home.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.div | 247, 1110 |
| div | 285, 316, 342, 359, 361, 377, 729, 750, 769, 788, 790, 956, 1091, 1109 |
| Heart | 299 |
| img | 302, 742 |
| ListMusic | 331 |
| Users | 333 |
| Disc3 | 335 |
| Music2 | 337 |
| span | 343, 770, 805, 1120 |
| CirclePlayButton | 378, 752 |
| motion.section | 657, 858, 895, 988, 1016, 1045 |
| QuickActionCard | 674 |
| EvenGridSkeleton | 689, 1019 |
| motion.button | 702 |
| EvenGrid | 823, 970 |
| RecTile | 831, 874, 909 |
| SectionTitle | 859, 896, 989, 1017, 1046 |
| CarouselControls | 859, 896, 1046 |
| TileSkeleton | 863, 900, 991, 1050 |
| EmptyHint | 865, 993, 1000, 1021, 1028 |
| CarouselTrack | 867, 902, 1052 |
| SegmentedControl | 938 |
| p | 948 |
| ArtistCard | 977 |
| RangeSlider | 989, 1017 |
| button | 995, 1023 |
| Refreshing | 1002, 1030 |
| TrackTiles | 1003 |
| ArtistTiles | 1031 |
| AlbumCard | 1059 |
| h1 | 1092, 1115 |
| Loader | 1093 |
| SignInPrompt | 1100 |
| QuickActionsShelf | 1134 |
| HomeFeedShelves | 1136 |
| MadeForYou | 1137 |
| RecentlyPlayed | 1138 |
| TopTracks | 1139 |
| TopArtists | 1140 |
| NewReleases | 1141 |

## src/pages/Library.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 52, 88, 116, 160 |
| TrackRowsSkeleton | 52 |
| EmptyState | 57, 121, 165, 209 |
| Heart | 58 |
| SyncButton | 61, 125, 169, 256 |
| motion.div | 76, 85, 131, 140, 175, 184, 243, 248, 261 |
| motion.p | 77, 132, 176 |
| TrackRow | 90 |
| p | 103, 147, 191 |
| CardGridSkeleton | 116, 160, 206 |
| Disc3 | 122 |
| AlbumGrid | 143, 217 |
| AlbumCard | 144 |
| Users | 166 |
| ArtistGrid | 187 |
| ArtistCard | 188 |
| Mic | 210 |
| MediaTile | 219 |
| SignInPrompt | 234 |
| h1 | 253 |
| LikedSongsTab | 271 |
| AlbumsTab | 272 |
| ArtistsTab | 273 |
| PodcastsTab | 274 |

## src/pages/Playground.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 31, 42, 43, 60, 63, 72, 94, 103, 112, 121, 128, 137 |
| Sparkles | 44 |
| span | 45 |
| h1 | 47 |
| p | 50, 67, 98, 116, 132 |
| motion.div | 56 |
| PlaygroundThemeBar | 57 |
| motion.section | 62, 93, 111, 127 |
| h2 | 64, 95, 113, 129 |
| PrimaryPlayButton | 73 |
| PulsePlayButton | 79 |
| SegmentedControl | 84 |
| MusiqueAlbumCard | 104 |
| GenreBucketCard | 105, 106 |
| AlbumTrackRowsSection | 122 |
| SpringToggle | 138, 139 |
| PlayerBarVolumeSlider | 140 |

## src/pages/PlaylistPage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Loader | 108 |
| EmptyState | 112 |
| button | 116 |
| div | 178 |
| Pin | 181 |
| Radio | 182 |
| Pencil | 185, 236 |
| Trash2 | 186 |
| Heart | 191, 236 |
| PageHeader | 197 |
| ExpandableDescription | 199 |
| p | 201, 284 |
| Link | 203 |
| span | 208, 210, 237 |
| PlayActions | 214 |
| Tooltip | 219 |
| motion.button | 223 |
| EditPlaylistModal | 243 |
| ConfirmDialog | 252 |
| b | 255 |
| section | 263 |
| motion.div | 265 |
| TrackRow | 271 |

## src/pages/Playlists.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| MotionLink | 62, 187 |
| div | 89, 106, 201, 202, 203, 204, 213, 345 |
| img | 91, 218 |
| ListMusic | 118, 352 |
| p | 122, 141, 233, 236, 360 |
| Folder | 214, 229 |
| span | 221, 329, 335 |
| PinOff | 306 |
| Pin | 306 |
| SignInPrompt | 313 |
| motion.div | 322, 323, 379 |
| h1 | 324 |
| Link | 327, 331 |
| ChevronRight | 330, 334 |
| SyncButton | 339, 355 |
| CardGridSkeleton | 346 |
| EmptyState | 351 |
| Shelf | 364 |
| MediaTile | 369 |
| SectionTitle | 374 |
| LayoutGroup | 378 |
| FolderCard | 386 |
| PlaylistCard | 389 |

## src/pages/ProfilePage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| ArtistGrid | 39 |
| MediaTile | 41, 247 |
| Loader | 61, 102 |
| p | 62, 64, 154, 251 |
| PeopleGrid | 66 |
| EmptyState | 105 |
| button | 108, 129 |
| span | 138, 156, 157, 176, 189 |
| Link2 | 148, 188 |
| div | 152, 162, 194 |
| PageHeader | 153 |
| Tooltip | 164 |
| motion.button | 165, 180 |
| Check | 175 |
| UserPlus | 175 |
| section | 196, 212 |
| SectionTitle | 197, 213 |
| FollowList | 198 |
| Shelf | 203, 234, 241 |
| ArtistCard | 208, 239 |
| TrackRow | 215 |

## src/pages/Search.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| motion.div | 179, 398, 640, 650 |
| div | 212, 228, 238, 248, 267, 311, 371, 548, 574, 596, 607, 666, 685, 758 |
| CoverArt | 258 |
| p | 268, 575 |
| h3 | 275 |
| button | 288, 623 |
| Link | 297 |
| span | 315, 317, 356 |
| motion.button | 325 |
| AnimatedPlayPause | 355 |
| EpisodeRow | 374 |
| MediaTile | 386, 722, 762 |
| SignInPrompt | 504 |
| Navigate | 512 |
| TrackRow | 533 |
| ShowAllButton | 545 |
| motion.header | 562 |
| h1 | 581 |
| SegmentedControl | 597 |
| TopResultSkeleton | 609 |
| TrackRowsSkeleton | 611 |
| CardGridSkeleton | 613 |
| EmptyState | 619, 631, 732, 742, 752, 774, 784 |
| section | 664, 678, 730, 767 |
| SectionTitle | 665, 679, 768 |
| TopResultCard | 667 |
| Shelf | 693, 701, 709, 717 |
| ArtistCard | 698, 739 |
| AlbumCard | 706, 749 |
| PlaylistResultCard | 714, 781 |
| ArtistGrid | 738 |
| AlbumGrid | 748, 760 |
| SearchEpisodes | 769 |
| PlaylistGrid | 780 |

## src/pages/Settings.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 64, 107, 113, 224, 234, 265, 356, 593, 784, 932, 949, 953, 1041, 1102, 1168, 1386, 1390 |
| span | 65, 75, 115, 243, 253, 483, 495, 620, 771, 817, 912, 913, 921, 938, 1230, 1231, 1244, 1251 |
| label | 108 |
| button | 133, 160, 358, 370, 381, 1214, 1354 |
| motion.button | 189 |
| RotateCcw | 207 |
| SegmentedControl | 307 |
| Tooltip | 357, 369, 380, 482, 494, 632, 828 |
| Minus | 366 |
| Plus | 389 |
| Card | 414, 558, 656, 692, 745, 901 |
| SettingRow | 415, 421, 434, 451, 457, 463, 559, 577, 589, 657, 671, 677, 693, 707, 746, 780, 904 |
| ZoomStepper | 418 |
| Divider | 420, 433, 450, 456, 462, 670, 676, 706, 779, 930 |
| Segmented | 425, 442, 569, 581, 757, 764 |
| Switch | 454, 460, 466, 661, 674, 680, 697, 710 |
| Info | 486, 498 |
| input | 604, 795, 986, 1001, 1316, 1336 |
| RevertBtn | 633, 829 |
| GhostBtn | 941, 944, 950, 954, 1391 |
| p | 962, 1019, 1031, 1288, 1378 |
| a | 971 |
| Field | 985, 1000, 1315, 1335 |
| PrimaryBtn | 1042, 1387 |
| motion.section | 1070, 1199 |
| h2 | 1085 |
| h1 | 1180 |
| PlaybackCard | 1191 |
| GeneralCard | 1192 |
| FeaturesCard | 1193 |
| AppearanceCard | 1194 |
| VisualCard | 1195 |
| LastfmCard | 1196 |
| StatusBadge | 1252 |
| motion.span | 1253 |
| ChevronDown | 1267 |
| AnimatePresence | 1272 |
| motion.div | 1274 |
| code | 1299 |
| EyeOff | 1370 |
| Eye | 1372 |

## src/pages/ShowPage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| Loader | 82 |
| EmptyState | 85 |
| button | 88, 189, 211 |
| div | 132, 171, 172, 194, 210 |
| Heart | 133, 162 |
| Link2 | 134 |
| PageHeader | 136 |
| ExpandableDescription | 137 |
| p | 138, 200 |
| span | 139, 141, 163 |
| PlayActions | 144 |
| Tooltip | 151 |
| motion.button | 152 |
| section | 170 |
| Search | 179 |
| input | 180 |
| X | 190 |
| Dropdown | 195 |
| EpisodeRow | 206 |

## src/pages/StatsPage.tsx

| Rendered tag/component | Source lines |
| --- | --- |
| div | 37, 38, 39, 40, 54, 55, 63, 69, 108, 110, 112, 113, 186, 212, 213 |
| section | 52, 106, 136 |
| SectionTitle | 53, 107, 137 |
| span | 56, 58, 64, 111, 115, 154, 156, 157, 158, 162 |
| p | 90, 138, 261 |
| ol | 139 |
| li | 141 |
| button | 142, 229 |
| CoverArt | 155 |
| Tile | 187, 188, 189, 190, 191, 192 |
| Navigate | 209 |
| h1 | 214 |
| SegmentedControl | 215 |
| Loader | 224 |
| EmptyState | 226, 232 |
| Stats | 233 |
| StatTiles | 239 |
| Heatmap | 241 |
| RankedTracks | 242, 258, 259, 260 |
| Shelf | 243, 250 |
| ArtistCard | 248 |
| AlbumCard | 255 |
| Genres | 257 |
| Link | 262 |

## src/store/addToPlaylist.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/auth.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/credentials.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/credits.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/jam.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/pins.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/player.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/prefs.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/queue.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/speedDial.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/theme.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/toast.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/ui.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/updater.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/store/ytMatch.store.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/types/ipc.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/types/library.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/types/podcast.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/types/spotify.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/episode.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/episode.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/fmt.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/fmt.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/listenTracker.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/palette.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/radio.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/romanize.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/stats.test.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/utils/stats.ts

No JSX rendering. Pure service/hook/store/test/config module.

## src/vite-env.d.ts

No JSX rendering. Pure service/hook/store/test/config module.

## vite.config.ts

No JSX rendering. Pure service/hook/store/test/config module.
