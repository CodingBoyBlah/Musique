import { sqliteTable, text, integer } from "drizzle-orm/sqlite-core";

export const settings = sqliteTable("settings", {
  key:       text("key").primaryKey(),
  value:     text("value").notNull(),
  updatedAt: integer("updated_at").notNull(),
});

export const users = sqliteTable("users", {
  id:          text("id").primaryKey(),
  displayName: text("display_name"),
  email:       text("email"),
  product:     text("product"),
  imageUrl:    text("image_url"),
  country:     text("country"),
  updatedAt:   integer("updated_at").notNull(),
});

export const artists = sqliteTable("artists", {
  id:         text("id").primaryKey(),
  name:       text("name").notNull(),
  imageUrl:   text("image_url"),
  genres:     text("genres"),    // JSON array
  popularity: integer("popularity"),
  updatedAt:  integer("updated_at").notNull(),
});

export const albums = sqliteTable("albums", {
  id:          text("id").primaryKey(),
  name:        text("name").notNull(),
  albumType:   text("album_type").notNull().default("album"),
  imageUrl:    text("image_url"),
  releaseDate: text("release_date"),
  totalTracks: integer("total_tracks").notNull().default(0),
  genres:      text("genres"),
  popularity:  integer("popularity"),
  updatedAt:   integer("updated_at").notNull(),
});

export const albumArtists = sqliteTable("album_artists", {
  albumId:  text("album_id").notNull().references(() => albums.id),
  artistId: text("artist_id").notNull().references(() => artists.id),
  position: integer("position").notNull().default(0),
});

export const tracks = sqliteTable("tracks", {
  id:          text("id").primaryKey(),
  name:        text("name").notNull(),
  albumId:     text("album_id").references(() => albums.id),
  durationMs:  integer("duration_ms").notNull().default(0),
  trackNumber: integer("track_number").notNull().default(0),
  discNumber:  integer("disc_number").notNull().default(1),
  explicit:    integer("explicit", { mode: "boolean" }).notNull().default(false),
  popularity:  integer("popularity"),
  previewUrl:  text("preview_url"),
  isLocal:     integer("is_local", { mode: "boolean" }).notNull().default(false),
  updatedAt:   integer("updated_at").notNull(),
});

export const trackArtists = sqliteTable("track_artists", {
  trackId:  text("track_id").notNull().references(() => tracks.id),
  artistId: text("artist_id").notNull().references(() => artists.id),
  position: integer("position").notNull().default(0),
});

export const playlists = sqliteTable("playlists", {
  id:          text("id").primaryKey(),
  name:        text("name").notNull(),
  description: text("description"),
  ownerId:     text("owner_id"),
  imageUrl:    text("image_url"),
  totalTracks: integer("total_tracks").notNull().default(0),
  isPublic:    integer("is_public", { mode: "boolean" }).notNull().default(false),
  isLocal:     integer("is_local",  { mode: "boolean" }).notNull().default(false),
  snapshotId:  text("snapshot_id"),
  updatedAt:   integer("updated_at").notNull(),
});

export const playlistTracks = sqliteTable("playlist_tracks", {
  playlistId: text("playlist_id").notNull().references(() => playlists.id),
  trackId:    text("track_id").notNull().references(() => tracks.id),
  position:   integer("position").notNull().default(0),
  addedAt:    integer("added_at").notNull(),
  addedBy:    text("added_by"),
});

export const savedTracks = sqliteTable("saved_tracks", {
  trackId: text("track_id").primaryKey().references(() => tracks.id),
  addedAt: integer("added_at").notNull(),
});

export const searchHistory = sqliteTable("search_history", {
  id:         integer("id").primaryKey({ autoIncrement: true }),
  query:      text("query").notNull(),
  resultType: text("result_type"),
  resultId:   text("result_id"),
  searchedAt: integer("searched_at").notNull(),
});

export const playbackHistory = sqliteTable("playback_history", {
  id:          integer("id").primaryKey({ autoIncrement: true }),
  trackId:     text("track_id").notNull(),
  contextType: text("context_type"),
  contextId:   text("context_id"),
  playedAt:    integer("played_at").notNull(),
  durationMs:  integer("duration_ms"),
});
