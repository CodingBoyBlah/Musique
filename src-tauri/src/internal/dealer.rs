//! push instead of poll. librespot already holds spotify's dealer websocket
//! open for connect; this taps three of its channels and turns them into
//! tauri events the frontend reacts to:
//!
//!   hm://connect-state/v1/cluster      -> "connect:cluster-changed"
//!       devices appearing/leaving, playback moving between them, remote
//!       play/pause/skip. replaces most of the device-list polling.
//!   social-connect/v2/session_update   -> "social:jam-updated"
//!       {reason, session} - someone joined/left, the jam ended, you were
//!       removed. the session is the same shape get_jam returns.
//!   hm://playlist/v2/playlist/         -> "library:playlist-changed"
//!
//! cluster and playlist payloads aren't decoded - those events just mean "go
//! refetch", which keeps this independent of the protobuf shapes.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use futures_util::{stream, StreamExt};
use librespot_core::{dealer::protocol::PayloadValue, Session};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

const CLUSTER: &str = "hm://connect-state/v1/cluster";
const JAM: &str = "social-connect/v2/session_update";
const PLAYLIST: &str = "hm://playlist/v2/playlist/";

const CHANNELS: &[(&str, &str)] = &[
    (CLUSTER, "connect:cluster-changed"),
    (JAM, "social:jam-updated"),
    (PLAYLIST, "library:playlist-changed"),
];

/// cluster updates arrive in bursts (every seek/volume nudge); one refetch a
/// second is plenty. jam updates each say something different, so they all go
const MIN_GAP: Duration = Duration::from_millis(1000);

/// the jam push, trimmed to {reason, session}
fn jam_payload(payload: &PayloadValue) -> Value {
    let PayloadValue::Json(raw) = payload else {
        return Value::Null;
    };
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return Value::Null;
    };
    let session = v
        .get("session")
        .and_then(crate::commands::social::jam_from)
        .map(|s| serde_json::to_value(s).unwrap_or(Value::Null))
        .unwrap_or(Value::Null);
    json!({
        "reason": v.get("reason").and_then(|r| r.as_str()).unwrap_or("UNKNOWN_UPDATE_TYPE"),
        "session": session,
    })
}

pub fn spawn(app: AppHandle, session: Session) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let mut subs = Vec::new();
        for (uri, event) in CHANNELS {
            match session.dealer().add_listen_for(*uri) {
                Ok(sub) => subs.push(
                    sub.map(move |msg| {
                        let payload = if *uri == JAM { jam_payload(&msg.payload) } else { Value::Null };
                        (*event, payload)
                    })
                    .boxed(),
                ),
                Err(e) => eprintln!("[dealer] can't listen for {uri}: {e}"),
            }
        }
        if subs.is_empty() {
            return;
        }
        let mut merged = stream::select_all(subs);
        let mut last: HashMap<&str, Instant> = HashMap::new();
        // ends when the session's dealer closes (session dropped / rebuilt)
        while let Some((event, payload)) = merged.next().await {
            if payload.is_null() {
                let now = Instant::now();
                if last.get(event).is_some_and(|t| now.duration_since(*t) < MIN_GAP) {
                    continue;
                }
                last.insert(event, now);
            }
            let _ = app.emit(event, payload);
        }
    })
}
