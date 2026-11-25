//! Apple Music as a full provider: account, catalog and library API, and Widevine playback.
//!
//! - `auth`: the web player's bearer token and the account's `media-user-token`.
//! - `client`: the amp-api client the rest of the app reads Apple Music through.
//! - `wire`: Apple JSON into the app's own models.
//! - `api`: the calls the command layer makes, answering in the app's models.
//! - `library`: writes the Apple library into the local database a sync fills.
//! - `playback`: resolving, licensing and decrypting a track into clear audio.

#![allow(dead_code)]
