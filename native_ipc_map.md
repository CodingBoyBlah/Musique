# Complete native IPC and event network

The 149 registered commands were reconciled with all static literal frontend invoke sites. Native definitions are source locations, not dynamically resolved call stacks. Every registered command remains documented even when no literal frontend caller is found. Tauri window/opener/updater APIs also have plugin IPC outside this custom registry.

| Registered command | Native definition | Frontend literal invoke sites |
| --- | --- | --- |
| get_setting | src-tauri/src/commands/settings.rs:8 | src/api/settings.ts:4 |
| set_setting | src-tauri/src/commands/settings.rs:22 | src/api/settings.ts:7 |
| get_all_settings | src-tauri/src/commands/settings.rs:48 | src/api/settings.ts:10 |
| save_credentials | src-tauri/src/commands/credentials.rs:73 | src/api/credentials.ts:18 |
| get_credentials | src-tauri/src/commands/credentials.rs:110 | src/api/credentials.ts:24 |
| validate_credentials | src-tauri/src/commands/credentials.rs:150 | src/api/credentials.ts:27 |
| clear_credentials | src-tauri/src/commands/credentials.rs:206 | src/api/credentials.ts:30 |
| start_login | src-tauri/src/commands/auth.rs:164 | src/api/auth.ts:10 |
| authorize_playback | src-tauri/src/commands/auth.rs:327 | No static literal invoke; native/indirect/test/diagnostic reachability needs caller context. |
| logout | src-tauri/src/commands/auth.rs:534 | src/api/auth.ts:12 |
| get_auth_status | src-tauri/src/commands/auth.rs:578 | src/api/auth.ts:13 |
| get_profile | src-tauri/src/commands/auth.rs:589 | src/api/auth.ts:14 |
| search | src-tauri/src/commands/spotify.rs:184 | src/api/spotify.ts:13 |
| get_artist | src-tauri/src/commands/spotify.rs:242 | src/api/spotify.ts:15 |
| get_album | src-tauri/src/commands/spotify.rs:288 | src/api/spotify.ts:16 |
| get_track | src-tauri/src/commands/spotify.rs:334 | src/api/spotify.ts:17 |
| get_playlist | src-tauri/src/commands/spotify.rs:411 | src/api/spotify.ts:18 |
| get_recommendations | src-tauri/src/commands/spotify.rs:387 | src/api/spotify.ts:26 |
| record_listen_event | src-tauri/src/commands/spotify.rs:361 | src/api/spotify.ts:42 |
| get_tracks_metadata | src-tauri/src/commands/internal.rs:11 | src/api/internal.ts:10 |
| get_friend_activity | src-tauri/src/commands/social.rs:70 | src/api/social.ts:20 |
| get_listening_stats | src-tauri/src/commands/stats.rs:200 | src/api/stats.ts:33 |
| get_jam | src-tauri/src/commands/social.rs:336 | src/api/social.ts:58 |
| start_jam | src-tauri/src/commands/social.rs:354 | src/api/social.ts:59 |
| join_jam | src-tauri/src/commands/social.rs:369 | src/api/social.ts:60 |
| leave_jam | src-tauri/src/commands/social.rs:389 | src/api/social.ts:62 |
| set_jam_queue_only | src-tauri/src/commands/social.rs:412 | src/api/social.ts:64 |
| kick_jam_member | src-tauri/src/commands/social.rs:426 | src/api/social.ts:66 |
| jam_command | src-tauri/src/commands/social.rs:443 | src/api/social.ts:78 |
| get_connect_state | src-tauri/src/commands/playback.rs:1194 | src/api/playback.ts:88 |
| connect_load_tracks | src-tauri/src/commands/playback.rs:1204 | src/api/playback.ts:90 |
| connect_add_to_queue | src-tauri/src/commands/playback.rs:1224 | src/api/playback.ts:91 |
| connect_skip_to | src-tauri/src/commands/playback.rs:1230 | src/api/playback.ts:92 |
| connect_next | src-tauri/src/commands/playback.rs:1236 | src/api/playback.ts:93 |
| connect_prev | src-tauri/src/commands/playback.rs:1241 | src/api/playback.ts:94 |
| connect_set_shuffle | src-tauri/src/commands/playback.rs:1246 | src/api/playback.ts:95 |
| connect_set_repeat | src-tauri/src/commands/playback.rs:1251 | src/api/playback.ts:96 |
| jam_hold | src-tauri/src/commands/playback.rs:1258 | src/api/playback.ts:98 |
| get_home_feed | src-tauri/src/commands/home_feed.rs:144 | src/api/internal.ts:103 |
| get_artist_extras | src-tauri/src/commands/artist_extras.rs:105 | src/api/internal.ts:76 |
| get_artist_overview | src-tauri/src/commands/artist_extras.rs:302 | src/api/internal.ts:122 |
| get_canvas | src-tauri/src/commands/internal.rs:79 | src/api/internal.ts:21 |
| get_track_credits | src-tauri/src/commands/internal.rs:177 | src/api/internal.ts:37 |
| get_playlist_folders | src-tauri/src/commands/internal.rs:332 | src/api/internal.ts:44 |
| get_track_radio | src-tauri/src/commands/internal.rs:479 | src/api/internal.ts:55 |
| get_station | src-tauri/src/commands/internal.rs:519 | src/api/internal.ts:59 |
| get_autoplay_tracks | src-tauri/src/commands/internal.rs:534 | src/api/internal.ts:63 |
| get_extracted_colors | src-tauri/src/commands/internal.rs:687 | src/api/internal.ts:86 |
| get_user_profile | src-tauri/src/commands/profile.rs:185 | src/api/profile.ts:27 |
| get_show | src-tauri/src/commands/podcasts.rs:128 | src/api/podcasts.ts:6 |
| get_episode_transcript | src-tauri/src/commands/episode_extras.rs:78 | src/api/podcasts.ts:23 |
| get_episode_media | src-tauri/src/commands/episode_extras.rs:151 | src/api/podcasts.ts:26 |
| get_show_episodes | src-tauri/src/commands/podcasts.rs:163 | src/api/podcasts.ts:9 |
| get_episode | src-tauri/src/commands/podcasts.rs:175 | src/api/podcasts.ts:11 |
| get_saved_shows | src-tauri/src/commands/podcasts.rs:183 | src/api/podcasts.ts:13 |
| save_show | src-tauri/src/commands/podcasts.rs:212 | src/api/podcasts.ts:15 |
| unsave_show | src-tauri/src/commands/podcasts.rs:220 | src/api/podcasts.ts:17 |
| is_show_saved | src-tauri/src/commands/podcasts.rs:228 | src/api/podcasts.ts:19 |
| get_user_followers | src-tauri/src/commands/profile.rs:232 | src/api/profile.ts:30 |
| get_user_following | src-tauri/src/commands/profile.rs:237 | src/api/profile.ts:33 |
| set_user_followed | src-tauri/src/commands/profile.rs:243 | src/api/profile.ts:36 |
| is_user_followed | src-tauri/src/commands/profile.rs:257 | src/api/profile.ts:39 |
| warmup_playback | src-tauri/src/commands/playback.rs:648 | src/api/playback.ts:46 |
| play_track | src-tauri/src/commands/playback.rs:717 | src/api/playback.ts:55 |
| retry_play_track | src-tauri/src/commands/playback.rs:763 | src/api/playback.ts:58 |
| pause_playback | src-tauri/src/commands/playback.rs:783 | src/api/playback.ts:99 |
| resume_playback | src-tauri/src/commands/playback.rs:804 | src/api/playback.ts:100 |
| resume_or_play | src-tauri/src/commands/playback.rs:851 | src/api/playback.ts:117 |
| stop_playback | src-tauri/src/commands/playback.rs:919 | src/api/playback.ts:119 |
| seek_playback | src-tauri/src/commands/playback.rs:945 | src/api/playback.ts:120 |
| preload_track | src-tauri/src/commands/playback.rs:960 | src/api/playback.ts:121 |
| get_playback_backend | src-tauri/src/commands/playback.rs:483 | src/api/playback.ts:153 |
| set_playback_backend | src-tauri/src/commands/playback.rs:503 | src/api/playback.ts:156 |
| get_yt_match | src-tauri/src/commands/playback.rs:567 | src/api/playback.ts:180 |
| search_yt_candidates | src-tauri/src/commands/playback.rs:591 | src/api/playback.ts:187 |
| pin_yt_match | src-tauri/src/commands/playback.rs:622 | src/api/playback.ts:191 |
| forget_yt_match | src-tauri/src/commands/playback.rs:636 | src/api/playback.ts:195 |
| set_volume | src-tauri/src/commands/playback.rs:979 | src/api/playback.ts:122 |
| set_playback_speed | src-tauri/src/commands/playback.rs:61 | src/api/playback.ts:205 |
| set_muted | src-tauri/src/commands/playback.rs:1009 | src/api/playback.ts:123 |
| get_volume | src-tauri/src/commands/playback.rs:1038 | src/api/playback.ts:124 |
| get_audio_quality | src-tauri/src/commands/playback.rs:1059 | src/api/playback.ts:129 |
| set_audio_quality | src-tauri/src/commands/playback.rs:1076 | src/api/playback.ts:132 |
| get_audio_cache_limit | src-tauri/src/commands/playback.rs:1104 | src/api/playback.ts:198 |
| set_audio_cache_limit | src-tauri/src/commands/playback.rs:1122 | src/api/playback.ts:201 |
| get_output_latency_ms | src-tauri/src/commands/playback.rs:1147 | src/lib/outputLatency.ts:44 |
| sync_library | src-tauri/src/commands/library.rs:29 | src/api/library.ts:6 |
| get_liked_songs | src-tauri/src/commands/library.rs:140 | src/api/library.ts:9 |
| get_liked_songs_count | src-tauri/src/commands/library.rs:193 | src/api/library.ts:12 |
| get_my_playlists | src-tauri/src/commands/library.rs:202 | src/api/library.ts:15 |
| get_saved_albums | src-tauri/src/commands/library.rs:226 | src/api/library.ts:18 |
| get_followed_artists | src-tauri/src/commands/library.rs:252 | src/api/library.ts:21 |
| get_library_status | src-tauri/src/commands/library.rs:741 | src/api/library.ts:24 |
| get_top_tracks | src-tauri/src/commands/library.rs:899 | src/api/library.ts:27 |
| get_top_artists | src-tauri/src/commands/library.rs:952 | src/api/library.ts:30 |
| get_recently_played | src-tauri/src/commands/library.rs:981 | src/api/library.ts:33 |
| get_new_releases | src-tauri/src/commands/library.rs:1033 | src/api/library.ts:36 |
| get_cached_playlist | src-tauri/src/commands/library.rs:1067 | src/api/spotify.ts:19 |
| save_track | src-tauri/src/commands/library.rs:285 | src/api/library.ts:39 |
| unsave_track | src-tauri/src/commands/library.rs:311 | src/api/library.ts:42 |
| get_saved_track_ids | src-tauri/src/commands/library.rs:489 | src/api/library.ts:45 |
| follow_artist | src-tauri/src/commands/library.rs:335 | src/api/library.ts:48 |
| unfollow_artist | src-tauri/src/commands/library.rs:362 | src/api/library.ts:51 |
| is_artist_followed | src-tauri/src/commands/library.rs:477 | src/api/library.ts:63 |
| save_album | src-tauri/src/commands/library.rs:386 | src/api/library.ts:54 |
| unsave_album | src-tauri/src/commands/library.rs:413 | src/api/library.ts:57 |
| is_album_saved | src-tauri/src/commands/library.rs:439 | src/api/library.ts:60 |
| add_track_to_playlist | src-tauri/src/commands/library.rs:640 | src/api/library.ts:67 |
| remove_track_from_playlist | src-tauri/src/commands/library.rs:670 | src/api/library.ts:70 |
| create_playlist | src-tauri/src/commands/library.rs:701 | src/api/library.ts:73 |
| update_playlist_details | src-tauri/src/commands/library.rs:514 | src/api/library.ts:79 |
| follow_playlist | src-tauri/src/commands/library.rs:578 | src/api/library.ts:88 |
| unfollow_playlist | src-tauri/src/commands/library.rs:598 | src/api/library.ts:92 |
| is_playlist_followed | src-tauri/src/commands/library.rs:626 | src/api/library.ts:95 |
| update_now_playing | src-tauri/src/commands/media.rs:12 | src/api/media.ts:5 |
| set_discord_enabled | src-tauri/src/commands/media.rs:32 | src/api/media.ts:21 |
| show_playback_notification | src-tauri/src/commands/media.rs:40 | src/api/media.ts:58 |
| set_window_effect | src-tauri/src/commands/window.rs:18 | src/api/window.ts:7 |
| get_backdrop_active | src-tauri/src/commands/window.rs:11 | src/api/window.ts:13 |
| get_lyrics | src-tauri/src/commands/lyrics.rs:22 | src/api/lyrics.ts:77 |
| set_lyrics_source | src-tauri/src/commands/lyrics.rs:66 | No static literal invoke; native/indirect/test/diagnostic reachability needs caller context. |
| resolve_odesli | src-tauri/src/commands/share.rs:14 | src/api/share.ts:5 |
| lastfm_status | src-tauri/src/commands/lastfm.rs:6 | src/api/lastfm.ts:10 |
| lastfm_save_api | src-tauri/src/commands/lastfm.rs:12 | src/api/lastfm.ts:13 |
| lastfm_start_auth | src-tauri/src/commands/lastfm.rs:18 | src/api/lastfm.ts:17 |
| lastfm_finish_auth | src-tauri/src/commands/lastfm.rs:36 | src/api/lastfm.ts:21 |
| lastfm_disconnect | src-tauri/src/commands/lastfm.rs:42 | src/api/lastfm.ts:24 |
| lastfm_clear | src-tauri/src/commands/lastfm.rs:48 | src/api/lastfm.ts:27 |
| lastfm_now_playing | src-tauri/src/commands/lastfm.rs:54 | src/api/lastfm.ts:30 |
| lastfm_scrobble | src-tauri/src/commands/lastfm.rs:64 | src/api/lastfm.ts:34 |
| get_wallpaper_data_url | src-tauri/src/commands/theme.rs:11 | src/api/theme.ts:4 |
| get_system_accent | src-tauri/src/commands/theme.rs:59 | src/api/theme.ts:8 |
| get_devices | src-tauri/src/commands/connect.rs:120 | src/api/connect.ts:31 |
| get_playback_state | src-tauri/src/commands/connect.rs:147 | src/api/connect.ts:34 |
| transfer_playback | src-tauri/src/commands/connect.rs:197 | src/api/connect.ts:37 |
| remote_play | src-tauri/src/commands/connect.rs:295 | src/api/connect.ts:51 |
| remote_play_track | src-tauri/src/commands/connect.rs:222 | src/api/connect.ts:44 |
| remote_pause | src-tauri/src/commands/connect.rs:305 | src/api/connect.ts:54 |
| remote_next | src-tauri/src/commands/connect.rs:315 | src/api/connect.ts:57 |
| remote_previous | src-tauri/src/commands/connect.rs:325 | src/api/connect.ts:60 |
| remote_seek | src-tauri/src/commands/connect.rs:335 | src/api/connect.ts:63 |
| remote_set_volume | src-tauri/src/commands/connect.rs:342 | src/api/connect.ts:66 |
| get_musique_device_id | src-tauri/src/commands/connect.rs:349 | src/api/connect.ts:69 |
| get_remote_queue | src-tauri/src/commands/connect.rs:410 | src/api/connect.ts:78 |
| remote_set_shuffle | src-tauri/src/commands/connect.rs:420 | src/api/connect.ts:81 |
| remote_set_repeat | src-tauri/src/commands/connect.rs:428 | src/api/connect.ts:86 |
| remote_add_to_queue | src-tauri/src/commands/connect.rs:446 | src/api/connect.ts:90 |
| remote_play_context | src-tauri/src/commands/connect.rs:457 | src/api/connect.ts:99 |
| trim_memory | src-tauri/src/mem_trim.rs:124 | No static literal invoke; native/indirect/test/diagnostic reachability needs caller context. |

Unregistered static literal invoke names: [].

## Native application events

| Event | Emit sites |
| --- | --- |
| media:toggle | src-tauri/src/lib.rs:865, src-tauri/src/media_controls/mod.rs:72 |
| media:next | src-tauri/src/lib.rs:868, src-tauri/src/media_controls/mod.rs:73 |
| media:prev | src-tauri/src/lib.rs:871, src-tauri/src/media_controls/mod.rs:74 |
| library:synced | src-tauri/src/library/mod.rs:722 |
| media:play | src-tauri/src/media_controls/mod.rs:70 |
| media:pause | src-tauri/src/media_controls/mod.rs:71 |
| media:stop | src-tauri/src/media_controls/mod.rs:75 |
| connect:state | src-tauri/src/playback/mod.rs:846 |
| player:event | src-tauri/src/playback/mod.rs:1010, src-tauri/src/playback/youtube.rs:469 |

The nine literal emit names above are supplemented by four constant/table-driven names. Their data flow was checked manually so the event network does not omit variable emit arguments.

| Event | Definition and emit | Frontend consumer/effect |
| --- | --- | --- |
| connect:cluster-changed | internal/dealer.rs:29; channel stream to emit at :60-86 | hooks/useDevices.ts:33; refresh shared device state after coalesced cluster pushes. |
| social:jam-updated | internal/dealer.rs:30; channel stream to emit at :60-86 | hooks/useJamSync.ts:57; receive trimmed reason/session update and reconcile Jam state. |
| library:playlist-changed | internal/dealer.rs:31; channel stream to emit at :60-86 | App.tsx:98; invalidate/refetch affected library queries. |
| lyrics:upgraded | lyrics/mod.rs:58 UPGRADE_EVENT; emit at :380 | lib/lyricsUpgrades.ts:17; update current-track query subscribers. Future-track prefetch handling is an audit finding. |

All paths in the definition/emit column are relative to src-tauri/src; consumer paths are relative to src. Total application event names: thirteen. Platform/input events and internal channel messages are separate networks covered in system_map.md and interaction_map.md.

## Every native module import edge

Qualified crate imports and declared child modules are listed below. References through super/self, macro expansion, traits, callbacks and inline calls are covered by manual workflow maps rather than presented as a complete compiler-resolved call graph.

## src-tauri/src/auth/http.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/auth/mod.rs

Declared children: http, pkce, token, profile.

Crate dependencies:

- `crate::errors::AppError`
- `crate::state::AuthState`

## src-tauri/src/auth/pkce.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/auth/profile.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`
- `crate::auth::{build_auth_status, init_auth_state_with, upsert_setting}`

## src-tauri/src/auth/token.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/commands/artist_extras.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, internal::{cache, metadata}, spotify::types::{AlbumItem, ArtistItem}, }`
- `crate::internal::pathfinder::{get, get_str}`

## src-tauri/src/commands/auth.rs

Declared children: none.

Crate dependencies:

- `crate::{ auth::{self, http, pkce, token}, errors::AppError, state::AppState, }`

## src-tauri/src/commands/connect.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, spotify::types::{AlbumItem, ArtistItem, SpAlbumSimple, SpArtistSimple, SpTrack, TrackItem}, state::AppState, }`

## src-tauri/src/commands/credentials.rs

Declared children: none.

Crate dependencies:

- `crate::{auth, errors::AppError, state::AppState}`

## src-tauri/src/commands/episode_extras.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, internal::{cache, pathfinder, spclient}, state::AppState, }`

## src-tauri/src/commands/home_feed.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, internal::{ cache, pathfinder::{self, get, get_str}, spclient, }, state::AppState, }`

## src-tauri/src/commands/internal.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, spotify::types::TrackItem}`
- `crate::internal::wire`
- `crate::internal::spclient::{image_uri_to_url, uri_id}`
- `crate::internal::pathfinder::get_str`

## src-tauri/src/commands/lastfm.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, lastfm, state::AppState}`

## src-tauri/src/commands/library.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, library::{sync_all, SyncResult}, spotify::types::{AlbumItem, ArtistItem, PlaylistDetail, TrackItem}, state::AppState, }`

## src-tauri/src/commands/lyrics.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, lyrics::{self, Lyrics, TrackRef}, state::AppState, }`

## src-tauri/src/commands/media.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, media_controls::MediaMsg, state::AppState, }`

## src-tauri/src/commands/mod.rs

Declared children: artist_extras, auth, credentials, episode_extras, home_feed, internal, lastfm, library, lyrics, media, playback, podcasts, profile, settings, share, social, stats, spotify, window, theme, connect.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/commands/playback.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, state::AppState, }`

## src-tauri/src/commands/podcasts.rs

Declared children: none.

Crate dependencies:

- `crate::{ commands::spotify::{tok, BASE}, errors::AppError, internal::cache, state::AppState, }`

## src-tauri/src/commands/profile.rs

Declared children: none.

Crate dependencies:

- `crate::{ commands::spotify::{tok, BASE}, errors::AppError, internal::{cache, spclient}, spotify::types::{ArtistItem, PlaylistCard}, state::AppState, }`

## src-tauri/src/commands/settings.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, state::AppState}`

## src-tauri/src/commands/share.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/commands/social.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, internal::spclient}`

## src-tauri/src/commands/spotify.rs

Declared children: none.

Crate dependencies:

- `crate::db::repos::{albums, artists, tracks}`
- `crate::errors::AppError`
- `crate::spotify::{self, types::*}`
- `crate::state::AppState`

## src-tauri/src/commands/stats.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, spotify::types::{AlbumItem, ArtistItem, TrackItem}, state::AppState, }`

## src-tauri/src/commands/theme.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/commands/window.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`
- `crate::state::AppState`

## src-tauri/src/db/connection.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/db/mod.rs

Declared children: connection, repos.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/db/repos/albums.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/artists.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/mod.rs

Declared children: albums, artists, playback_history, playlists, saved_tracks, search_history, tracks, users, yt_match.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/db/repos/playback_history.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/playlists.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/saved_tracks.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/search_history.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/tracks.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/users.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/db/repos/yt_match.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/discord/mod.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/episode_audio/feed.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/episode_audio/live_tests.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/episode_audio/mod.rs

Declared children: feed, mp4, remote, source, live_tests.

Crate dependencies:

- `crate::{errors::AppError, http, internal::cache}`

## src-tauri/src/episode_audio/mp4.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/episode_audio/remote.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, http}`

## src-tauri/src/episode_audio/source.rs

Declared children: none.

Crate dependencies:

- `crate::stretch::TimeStretch`

## src-tauri/src/errors.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/http.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/internal/cache.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/internal/dealer.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/internal/metadata.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, spotify::types::{AlbumItem, ArtistItem, TrackItem}, state::AppState, }`

## src-tauri/src/internal/mod.rs

Declared children: cache, dealer, metadata, pathfinder, spclient, wire.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/internal/pathfinder.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, state::AppState}`

## src-tauri/src/internal/spclient.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, state::AppState}`

## src-tauri/src/internal/wire.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/lastfm/mod.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/lib.rs

Declared children: auth, commands, db, discord, episode_audio, errors, http, internal, lastfm, library, lyrics, media_controls, playback, playout, recommend, resample, sink, spotify, stretch, youtube, state, mem_trim.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/library/mod.rs

Declared children: none.

Crate dependencies:

- `crate::{ db::repos::{albums, artists, tracks}, errors::AppError, spotify::{self, types::*}, }`

## src-tauri/src/lyrics/align.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::types::{LyricLine, LyricWord}`

## src-tauri/src/lyrics/cache.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`
- `crate::lyrics::types::{LyricLine, LyricWord}`

## src-tauri/src/lyrics/mod.rs

Declared children: align, cache, parse, providers, ttml, types, voices.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/lyrics/parse.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/lyrics/providers/amll.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::scrub_spaces`
- `crate::lyrics::ttml::parse_ttml`
- `crate::lyrics::types::{Candidate, TrackRef}`

## src-tauri/src/lyrics/providers/applemusic.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::ttml::parse_ttml`
- `crate::lyrics::types::{Candidate, TrackRef}`

## src-tauri/src/lyrics/providers/betterlyrics.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::ttml::parse_ttml`
- `crate::lyrics::types::{Candidate, TrackRef}`

## src-tauri/src/lyrics/providers/kugou.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::{scrub_spaces, strip_noise}`
- `crate::lyrics::types::{Candidate, LyricLine, LyricWord, TrackRef}`

## src-tauri/src/lyrics/providers/lrclib.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::{parse_lrc, strip_noise}`
- `crate::lyrics::types::{Candidate, TrackRef}`

## src-tauri/src/lyrics/providers/mod.rs

Declared children: amll, applemusic, betterlyrics, kugou, lrclib, musixmatch, netease, qq, spotify.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/lyrics/providers/musixmatch.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::{parse_lrc, parse_richsync, strip_noise}`
- `crate::lyrics::types::{Candidate, TrackRef}`

## src-tauri/src/lyrics/providers/netease.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::{merge_parallel, parse_lrc, parse_yrc, strip_noise}`
- `crate::lyrics::types::{Candidate, LyricLine, TrackRef}`

## src-tauri/src/lyrics/providers/qq.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::parse::{scrub_spaces, strip_noise}`
- `crate::lyrics::types::{Candidate, LyricLine, LyricWord, TrackRef}`

## src-tauri/src/lyrics/providers/spotify.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::{ parse::scrub_spaces, types::{Candidate, LyricLine, LyricWord, TrackRef}, }`

## src-tauri/src/lyrics/ttml.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/lyrics/types.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/lyrics/voices.rs

Declared children: none.

Crate dependencies:

- `crate::lyrics::types::LyricWord`

## src-tauri/src/main.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/media_controls/mod.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/mem_trim.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/playback/listening.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/playback/mod.rs

Declared children: listening, youtube.

Crate dependencies:

- `crate::{ auth, errors::AppError, state::AuthState, }`

## src-tauri/src/playback/youtube.rs

Declared children: none.

Crate dependencies:

- `crate::{ errors::AppError, media_controls::MediaMsg, playout::{self, Ring}, youtube::ExtractedStream, }`

## src-tauri/src/playout.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/recommend/mod.rs

Declared children: none.

Crate dependencies:

- `crate::commands::spotify::{item_from_album_simple, item_from_album_track, item_from_track}`
- `crate::errors::AppError`
- `crate::spotify::{self, types::*}`

## src-tauri/src/resample.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/sink.rs

Declared children: none.

Crate dependencies:

- `crate::playout::{self, Chunk, Ring, RingSource}`
- `crate::resample::Resampler`

## src-tauri/src/spotify/mod.rs

Declared children: types.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/spotify/types.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/state.rs

Declared children: none.

Crate dependencies:

- `crate::media_controls::MediaMsg`

## src-tauri/src/stretch.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/youtube/client.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/youtube/format.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/youtube/innertube.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, http}`

## src-tauri/src/youtube/live_tests.rs

Declared children: none.

Crate dependencies:

- `crate::youtube::{matching::TrackQuery, search}`

## src-tauri/src/youtube/matching.rs

Declared children: none.

Crate dependencies:

No explicit crate-qualified use declarations; other dependencies may use super/self or inline qualified calls.

## src-tauri/src/youtube/mod.rs

Declared children: client, format, innertube, matching, player, search, stream, visitor, live_tests.

Crate dependencies:

- `crate::{db::repos::yt_match, errors::AppError}`

## src-tauri/src/youtube/player.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/youtube/search.rs

Declared children: none.

Crate dependencies:

- `crate::errors::AppError`

## src-tauri/src/youtube/stream.rs

Declared children: none.

Crate dependencies:

- `crate::{errors::AppError, http}`

## src-tauri/src/youtube/visitor.rs

Declared children: none.

Crate dependencies:

- `crate::http`
