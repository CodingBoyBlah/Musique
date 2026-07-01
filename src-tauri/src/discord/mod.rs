use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{
    activity::{Activity, ActivityType, Assets, Button, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

// default discord application id. discord rich presence ALWAYS needs a valid
// registered app id, theres no "no id" mode (cider and the rest just bake
// their own in, thats why they "just work"). the old default was a dead
// app so the handshake just got rejected quietly and nothing ever showed. this id
// is a verified live public app so presence works right outta the box
//
// discord ties the shown app NAME to the id so it reads as that apps name
// until u register ur own (like 30s at discord.com/developers) and point
// DISCORD_CLIENT_ID at it to show a custom name + ur own asset art
const DEFAULT_CLIENT_ID: &str = "1108588077900898414";

struct Meta {
    title:       String,
    artist:      String,
    album:       String,
    cover_url:   Option<String>,
    track_url:   Option<String>,
    duration_ms: u64,
}

/// thin wrapper over the discord ipc client. connection is lazy and kinda self
/// healing, the first update connects, any failed call drops the connection so
/// the next try reconnects, and a periodic `tick()` retries while a track is
/// active so presence shows up even if discord opens AFTER the app. all
/// errors just get swallowed, presence is best effort and never blocks playback
pub struct DiscordPresence {
    client:    Option<DiscordIpcClient>,
    connected: bool,
    enabled:   bool,
    meta:      Option<Meta>,
    // last known playback state so a reconnect or a settings re-enable can
    // bring back the right timeline without waiting for the next player event
    last_pos:     u64,
    last_playing: bool,
}

impl DiscordPresence {
    pub fn new() -> Self {
        let client_id =
            std::env::var("DISCORD_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        // v1s `new` cant fail, it just stores the id, `connect` does the
        // actual work and thats where stuff blows up
        let client = Some(DiscordIpcClient::new(&client_id));
        Self {
            client,
            connected: false,
            enabled: true,
            meta: None,
            last_pos: 0,
            last_playing: false,
        }
    }

    fn ensure_connected(&mut self) -> bool {
        if self.connected {
            return true;
        }
        if let Some(c) = self.client.as_mut() {
            if c.connect().is_ok() {
                self.connected = true;
                return true;
            }
        }
        false
    }

    /// flip presence on/off (user setting). off just wipes whatever activity is showing
    pub fn set_enabled(&mut self, enabled: bool, position_ms: u64, playing: bool) {
        self.enabled = enabled;
        if enabled {
            self.apply(position_ms, playing);
        } else {
            self.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn now_playing(
        &mut self,
        title:       String,
        artist:      String,
        album:       String,
        cover_url:   Option<String>,
        track_url:   Option<String>,
        duration_ms: u64,
        position_ms: u64,
        playing:     bool,
    ) {
        self.meta = Some(Meta { title, artist, album, cover_url, track_url, duration_ms });
        self.apply(position_ms, playing);
    }

    pub fn set_state(&mut self, position_ms: u64, playing: bool) {
        self.apply(position_ms, playing);
    }

    /// periodic retry. while a track is active but discord wasnt reachable yet
    /// (closed at launch, just opened, dropped connection) this re-applies the
    /// current presence so it shows up without needing a track change
    pub fn tick(&mut self) {
        if self.enabled && !self.connected && self.meta.is_some() {
            self.apply(self.last_pos, self.last_playing);
        }
    }

    pub fn clear(&mut self) {
        if self.connected {
            if let Some(c) = self.client.as_mut() {
                let _ = c.clear_activity();
            }
        }
    }

    fn apply(&mut self, position_ms: u64, playing: bool) {
        self.last_pos = position_ms;
        self.last_playing = playing;
        if !self.enabled {
            return;
        }
        // copy the metadata out so the immutable borrow of `self.meta` ends before
        // we grab a mutable borrow of `self.client` down below
        let (title, artist, album, cover, track_url, duration_ms) = match &self.meta {
            Some(m) => (
                m.title.clone(),
                m.artist.clone(),
                m.album.clone(),
                m.cover_url.clone(),
                m.track_url.clone(),
                m.duration_ms,
            ),
            None => return,
        };
        if title.is_empty() || !self.ensure_connected() {
            return;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let start = now - (position_ms / 1000) as i64;
        let end   = start + (duration_ms / 1000) as i64;

        let mut assets = Assets::new();
        if let Some(url) = cover.as_deref() {
            if !url.is_empty() {
                assets = assets.large_image(url);
            }
        }
        if !album.is_empty() {
            assets = assets.large_text(&album);
        }

        // "listening to <app>" reads right for music instead of the default
        // "playing". discord wants details/state to be 2..=128 chars btw
        let mut activity = Activity::new()
            .activity_type(ActivityType::Listening)
            .details(pad2(&title))
            .assets(assets);
        if artist.len() >= 2 {
            activity = activity.state(&artist);
        }

        // clickable button that opens the track in spotify, only if we actually have a url
        let valid_url = track_url
            .as_deref()
            .filter(|u| u.starts_with("https://") || u.starts_with("http://"));
        if let Some(u) = valid_url {
            activity = activity.buttons(vec![Button::new("Play on Spotify", u)]);
        }

        // progress bar only while playing, a paused track shows no timeline
        let timed = Timestamps::new().start(start).end(end);
        let activity = if playing { activity.timestamps(timed) } else { activity };

        if let Some(c) = self.client.as_mut() {
            if c.set_activity(activity).is_err() {
                // drop the connection so the next call just reconnects on its own
                self.connected = false;
            }
        }
    }
}

// discord rejects details/state strings shorter than 2 chars so pad just in case
fn pad2(s: &str) -> String {
    if s.len() >= 2 { s.to_string() } else { format!("{s} ") }
}

impl Default for DiscordPresence {
    fn default() -> Self {
        Self::new()
    }
}
