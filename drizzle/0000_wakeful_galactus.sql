CREATE TABLE `album_artists` (
	`album_id` text NOT NULL,
	`artist_id` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	FOREIGN KEY (`album_id`) REFERENCES `albums`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`artist_id`) REFERENCES `artists`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `albums` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`album_type` text DEFAULT 'album' NOT NULL,
	`image_url` text,
	`release_date` text,
	`total_tracks` integer DEFAULT 0 NOT NULL,
	`genres` text,
	`popularity` integer,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `artists` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`image_url` text,
	`genres` text,
	`popularity` integer,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `playback_history` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`track_id` text NOT NULL,
	`context_type` text,
	`context_id` text,
	`played_at` integer NOT NULL,
	`duration_ms` integer
);
--> statement-breakpoint
CREATE TABLE `playlist_tracks` (
	`playlist_id` text NOT NULL,
	`track_id` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	`added_at` integer NOT NULL,
	`added_by` text,
	FOREIGN KEY (`playlist_id`) REFERENCES `playlists`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`track_id`) REFERENCES `tracks`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `playlists` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`description` text,
	`owner_id` text,
	`image_url` text,
	`total_tracks` integer DEFAULT 0 NOT NULL,
	`is_public` integer DEFAULT false NOT NULL,
	`is_local` integer DEFAULT false NOT NULL,
	`snapshot_id` text,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `saved_tracks` (
	`track_id` text PRIMARY KEY NOT NULL,
	`added_at` integer NOT NULL,
	FOREIGN KEY (`track_id`) REFERENCES `tracks`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `search_history` (
	`id` integer PRIMARY KEY AUTOINCREMENT NOT NULL,
	`query` text NOT NULL,
	`result_type` text,
	`result_id` text,
	`searched_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `settings` (
	`key` text PRIMARY KEY NOT NULL,
	`value` text NOT NULL,
	`updated_at` integer NOT NULL
);
--> statement-breakpoint
CREATE TABLE `track_artists` (
	`track_id` text NOT NULL,
	`artist_id` text NOT NULL,
	`position` integer DEFAULT 0 NOT NULL,
	FOREIGN KEY (`track_id`) REFERENCES `tracks`(`id`) ON UPDATE no action ON DELETE no action,
	FOREIGN KEY (`artist_id`) REFERENCES `artists`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `tracks` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`album_id` text,
	`duration_ms` integer DEFAULT 0 NOT NULL,
	`track_number` integer DEFAULT 0 NOT NULL,
	`disc_number` integer DEFAULT 1 NOT NULL,
	`explicit` integer DEFAULT false NOT NULL,
	`popularity` integer,
	`preview_url` text,
	`is_local` integer DEFAULT false NOT NULL,
	`updated_at` integer NOT NULL,
	FOREIGN KEY (`album_id`) REFERENCES `albums`(`id`) ON UPDATE no action ON DELETE no action
);
--> statement-breakpoint
CREATE TABLE `users` (
	`id` text PRIMARY KEY NOT NULL,
	`display_name` text,
	`email` text,
	`product` text,
	`image_url` text,
	`country` text,
	`updated_at` integer NOT NULL
);
