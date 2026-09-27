//! push instead of poll. librespot already holds spotify's dealer websocket
//! open for connect; this taps three of its channels and turns them into
//! tauri events the frontend reacts to:
//!
//!   hm://connect-state/v1/cluster      -> "connect:cluster-changed"
//!       devices appearing/leaving, playback moving between them, remote
//!       play/pause/skip. replaces most of the device-list polling.
//!   social-connect/v2/session_update   -> "social:jam-updated"
//!   hm://playlist/v2/playlist/         -> "library:playlist-changed"
//!
//! the payloads aren't decoded - each event just means "go refetch", which
//! keeps this independent of the protobuf shapes.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures_util::{stream, StreamExt};
use librespot_core::Session;
use tauri::{AppHandle, Emitter};

const CHANNELS: &[(&str, &str)] = &[
    ("hm://connect-state/v1/cluster", "connect:cluster-changed"),
    ("social-connect/v2/session_update", "social:jam-updated"),
    ("hm://playlist/v2/playlist/", "library:playlist-changed"),
];

/// cluster updates arrive in bursts (every seek/volume nudge); one refetch a
/// second is plenty
const MIN_GAP: Duration = Duration::from_millis(1000);

pub fn spawn(app: AppHandle, session: Session) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut subs = Vec::new();
        for (uri, event) in CHANNELS {
            match session.dealer().add_listen_for(*uri) {
                Ok(sub) => subs.push(sub.map(move |_| *event).boxed()),
                Err(e) => eprintln!("[dealer] can't listen for {uri}: {e}"),
            }
        }
        if subs.is_empty() {
            return;
        }
        let mut merged = stream::select_all(subs);
        let mut last: HashMap<&str, Instant> = HashMap::new();
        // ends when the session's dealer closes (session dropped / rebuilt)
        while let Some(event) = merged.next().await {
            let now = Instant::now();
            if last.get(event).is_some_and(|t| now.duration_since(*t) < MIN_GAP) {
                continue;
            }
            last.insert(event, now);
            let _ = app.emit(event, ());
        }
    })
}
