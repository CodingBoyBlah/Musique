use serde::{Deserialize, Serialize};

// ─── spotify api internal types, deserialize only ────────────────────────────

#[derive(Deserialize)]
pub(crate) struct SpImage {
    pub url: String,
}

#[derive(Deserialize)]
pub(crate) struct SpArtistSimple {
    pub id:   String,
    pub name: String,
}

#[derive(Deserialize)]
pub(crate) struct SpArtist {
    pub id:         String,
    pub name:       String,
    pub images:     Option<Vec<SpImage>>,
    pub genres:     Option<Vec<String>>,
    pub popularity: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct SpAlbumSimple {
    pub id:           String,
    pub name:         String,
    pub album_type:   String,
    pub images:       Option<Vec<SpImage>>,
    pub release_date: Option<String>,
    pub artists:      Option<Vec<SpArtistSimple>>,
}

#[derive(Deserialize)]
pub(crate) struct SpAlbum {
    pub id:           String,
    pub name:         String,
    pub album_type:   String,
    pub images:       Option<Vec<SpImage>>,
    pub release_date: Option<String>,
    pub total_tracks: Option<i64>,
    pub artists:      Option<Vec<SpArtistSimple>>,
    pub tracks:       Option<SpPage<SpAlbumTrack>>,
    pub genres:       Option<Vec<String>>,
    pub popularity:   Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct SpAlbumTrack {
    pub id:           String,
    pub name:         String,
    pub duration_ms:  i64,
    pub explicit:     bool,
    pub artists:      Vec<SpArtistSimple>,
    pub preview_url:  Option<String>,
    pub track_number: Option<i64>,
    pub disc_number:  Option<i64>,
    pub is_local:     Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct SpTrack {
    pub id:           String,
    pub name:         String,
    pub duration_ms:  i64,
    pub explicit:     bool,
    pub popularity:   Option<i64>,
    pub preview_url:  Option<String>,
    pub artists:      Vec<SpArtistSimple>,
    pub album:        Option<SpAlbumSimple>,
    pub track_number: Option<i64>,
    pub disc_number:  Option<i64>,
    pub is_local:     Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct SpPage<T> {
    pub items: Vec<T>,
}

#[derive(Deserialize)]
pub(crate) struct SpTopTracks {
    pub tracks: Vec<SpTrack>,
}

/// `GET /artists?ids=` batch response, used to read genres for seed artists
#[derive(Deserialize)]
pub(crate) struct SpArtists {
    pub artists: Vec<SpArtist>,
}

#[derive(Deserialize)]
pub(crate) struct SpPlaylistTrackPage {
    pub items: Vec<SpPlaylistTrackItem>,
    pub total: i64,
    #[serde(default)]
    pub next:  Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpPlaylistTrackItem {
    pub track:    Option<SpTrack>,
    #[serde(default)]
    pub added_at: Option<String>,
}

// ─── discovery surfaces ───────────────────────────────────────────────────────

/// `GET /me/player/recently-played`, a cursor page of play history items
#[derive(Deserialize)]
pub(crate) struct SpRecentlyPlayedPage {
    pub items: Vec<SpPlayHistory>,
}

#[derive(Deserialize)]
pub(crate) struct SpPlayHistory {
    pub track:     SpTrack,
    pub played_at: String,
}

/// `GET /browse/new-releases`, looks like `{ "albums": { "items": [...] } }`
#[derive(Deserialize)]
pub(crate) struct SpNewReleases {
    pub albums: SpPage<SpAlbumSimple>,
}

#[derive(Deserialize)]
pub(crate) struct SpSearchResponse {
    pub tracks:    Option<SpPage<SpTrack>>,
    pub artists:   Option<SpPage<SpArtist>>,
    pub albums:    Option<SpPage<SpAlbumSimple>>,
    pub playlists: Option<SpPlaylistSearchPage>,
}

// spotifys search returns `null` entries in the playlists array so just tolerate em
#[derive(Deserialize)]
pub(crate) struct SpPlaylistSearchPage {
    pub items: Vec<Option<SpSearchPlaylist>>,
}

#[derive(Deserialize)]
pub(crate) struct SpSearchPlaylist {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub images:      Option<Vec<SpImage>>,
    pub owner:       Option<SpOwner>,
}

// ─── library sync response types ─────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct SpSavedTrackPage {
    pub items: Vec<SpSavedTrackItem>,
    pub total: i64,
    pub next:  Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpSavedTrackItem {
    pub added_at: String,
    pub track:    Option<SpTrack>,
}

#[derive(Deserialize)]
pub(crate) struct SpSavedAlbumPage {
    // spotify returns `null` entries here for albums that went unavailable so
    // just treat em as None instead of blowing up the whole page decode
    pub items: Vec<Option<SpSavedAlbumItem>>,
    pub next:  Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpSavedAlbumItem {
    pub added_at: String,
    pub album:    SpAlbum,
}

#[derive(Deserialize)]
pub(crate) struct SpUserPlaylistPage {
    pub items: Vec<SpUserPlaylistItem>,
    pub total: i64,
    pub next:  Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpUserPlaylistItem {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub images:      Option<Vec<SpImage>>,
    pub owner:       Option<SpOwner>,
    pub tracks:      Option<SpTracksTotal>,
    pub snapshot_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpTracksTotal {
    pub total: i64,
}

#[derive(Deserialize)]
pub(crate) struct SpFollowedArtistsResponse {
    pub artists: SpArtistCursorPage,
}

#[derive(Deserialize)]
pub(crate) struct SpArtistCursorPage {
    pub items:   Vec<SpArtist>,
    pub next:    Option<String>,
    pub cursors: Option<SpCursors>,
}

#[derive(Deserialize)]
pub(crate) struct SpCursors {
    pub after: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpPlaylist {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub images:      Option<Vec<SpImage>>,
    pub owner:       Option<SpOwner>,
    pub tracks:      Option<SpPlaylistTrackPage>,
    #[serde(default)]
    pub snapshot_id: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SpOwner {
    pub display_name: Option<String>,
}

// ─── ipc output types, public, serialize + deserialize ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistItem {
    pub id:         String,
    pub name:       String,
    pub image_url:  Option<String>,
    #[serde(default)]
    pub popularity: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumItem {
    pub id:           String,
    pub name:         String,
    pub album_type:   String,
    pub image_url:    Option<String>,
    pub release_date: Option<String>,
    pub artists:      Vec<ArtistItem>,
    #[serde(default)]
    pub popularity:   Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackItem {
    pub id:          String,
    pub name:        String,
    pub duration_ms: i64,
    pub explicit:    bool,
    pub artists:     Vec<ArtistItem>,
    pub album:       Option<AlbumItem>,
    #[serde(default)]
    pub popularity:  Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistCard {
    pub id:          String,
    pub name:        String,
    pub description: Option<String>,
    pub image_url:   Option<String>,
    pub owner_name:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub tracks:    Vec<TrackItem>,
    pub artists:   Vec<ArtistItem>,
    pub albums:    Vec<AlbumItem>,
    pub playlists: Vec<PlaylistCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDetail {
    pub id:         String,
    pub name:       String,
    pub image_url:  Option<String>,
    pub genres:     Vec<String>,
    pub popularity: Option<i64>,
    pub albums:     Vec<AlbumItem>,
    pub singles:    Vec<AlbumItem>,
    pub top_tracks: Vec<TrackItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumDetail {
    pub id:           String,
    pub name:         String,
    pub album_type:   String,
    pub image_url:    Option<String>,
    pub release_date: Option<String>,
    pub total_tracks: i64,
    pub popularity:   Option<i64>,
    pub artists:      Vec<ArtistItem>,
    pub tracks:       Vec<TrackItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackDetail {
    pub id:          String,
    pub name:        String,
    pub duration_ms: i64,
    pub explicit:    bool,
    pub popularity:  Option<i64>,
    pub preview_url: Option<String>,
    pub artists:     Vec<ArtistItem>,
    pub album:       Option<AlbumItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDetail {
    pub id:           String,
    pub name:         String,
    pub description:  Option<String>,
    pub image_url:    Option<String>,
    pub owner_name:   Option<String>,
    pub total_tracks: i64,
    pub tracks:       Vec<TrackItem>,
}
