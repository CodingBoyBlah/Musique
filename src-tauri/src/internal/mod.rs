//! spotify's internal apis - the stuff the official client runs on.
//!
//! the public web api lost most of its interesting surfaces for new apps in
//! nov 2024 (recommendations, related artists, algorithmic playlists, audio
//! features...). all of that still exists behind spclient, which librespot
//! already talks to for playback. this module is the one place that reaches in:
//!
//!   spclient  - session handle + json/protobuf helpers over the live session
//!   metadata  - batched extended-metadata (TRACK_V4/ALBUM_V4/ARTIST_V4) into
//!               the same TrackItem/AlbumItem/ArtistItem the rest of the app uses
//!   pathfinder - the graphql api behind the official home feed / artist pages
//!   dealer    - pushes (device/jam/playlist changes) forwarded as tauri events
//!   cache     - (entity, kind) -> bytes, so none of it is refetched needlessly
//!   wire      - bare protobuf encode/decode for the endpoints whose protos
//!               librespot ships incomplete
//!
//! every call here is best-effort. an internal endpoint moving or going away
//! must degrade a feature, never break the page it sits on.

pub mod cache;
pub mod dealer;
pub mod metadata;
pub mod pathfinder;
pub mod spclient;
pub mod wire;
