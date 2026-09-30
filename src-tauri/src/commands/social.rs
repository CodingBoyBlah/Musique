//! the social layer: friend activity (what the people you follow are
//! playing) and jam sessions. both live on spclient only.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::{errors::AppError, internal::spclient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendActivity {
    pub user_id:      String,
    pub name:         String,
    pub image_url:    Option<String>,
    /// ms since epoch of the play
    pub timestamp:    i64,
    pub track_id:     Option<String>,
    pub track_name:   Option<String>,
    pub track_image:  Option<String>,
    pub artist_id:    Option<String>,
    pub artist_name:  Option<String>,
    pub album_id:     Option<String>,
    pub album_name:   Option<String>,
    /// what they're playing from (a playlist, album, artist...)
    pub context_uri:  Option<String>,
    pub context_name: Option<String>,
}

fn s(v: &Value, path: &[&str]) -> Option<String> {
    crate::internal::pathfinder::get_str(v, path).map(str::to_string)
}

fn id_of(uri: Option<String>) -> Option<String> {
    uri.map(|u| spclient::uri_id(&u).to_string()).filter(|u| !u.is_empty())
}

pub(crate) fn friends_from(v: &Value) -> Vec<FriendActivity> {
    let mut out: Vec<FriendActivity> = v
        .get("friends")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let user_uri = s(f, &["user", "uri"])?;
                    Some(FriendActivity {
                        user_id: spclient::uri_id(&user_uri).to_string(),
                        name: s(f, &["user", "name"]).unwrap_or_else(|| spclient::uri_id(&user_uri).to_string()),
                        image_url: s(f, &["user", "imageUrl"]),
                        timestamp: f.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0),
                        track_id: id_of(s(f, &["track", "uri"]).filter(|u| u.starts_with("spotify:track:"))),
                        track_name: s(f, &["track", "name"]),
                        track_image: s(f, &["track", "imageUrl"]),
                        artist_id: id_of(s(f, &["track", "artist", "uri"])),
                        artist_name: s(f, &["track", "artist", "name"]),
                        album_id: id_of(s(f, &["track", "album", "uri"])),
                        album_name: s(f, &["track", "album", "name"]),
                        context_uri: s(f, &["track", "context", "uri"]),
                        context_name: s(f, &["track", "context", "name"]),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    out
}

/// what the people you follow are listening to, most recent first
#[tauri::command]
pub async fn get_friend_activity(app: AppHandle) -> Result<Vec<FriendActivity>, AppError> {
    let v = spclient::get_json_value(&app, "/presence-view/v1/buddylist").await?;
    Ok(friends_from(&v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_buddylist() {
        let v: Value = serde_json::from_str(
            r#"{"friends": [
              {"timestamp": 100, "user": {"uri": "spotify:user:old", "name": "Old"}, "track": {"uri": "spotify:track:t0", "name": "A"}},
              {"timestamp": 200, "user": {"uri": "spotify:user:ann", "name": "Ann", "imageUrl": "https://i/a"},
               "track": {"uri": "spotify:track:t1", "name": "Song", "imageUrl": "https://i/t",
                         "album": {"uri": "spotify:album:al", "name": "LP"},
                         "artist": {"uri": "spotify:artist:ar", "name": "Band"},
                         "context": {"uri": "spotify:playlist:pl", "name": "Mix"}}}
            ]}"#,
        )
        .unwrap();
        let f = friends_from(&v);
        assert_eq!(f[0].name, "Ann", "newest first");
        assert_eq!(f[0].track_id.as_deref(), Some("t1"));
        assert_eq!(f[0].artist_id.as_deref(), Some("ar"));
        assert_eq!(f[0].context_name.as_deref(), Some("Mix"));
        assert_eq!(f[1].user_id, "old");
    }
}

// ── jam (social-connect) ─────────────────────────────────────────────────────
//
// how spotify's own clients do it (desktop core, go-librespot, captures):
//
// - a jam is bound to a Connect *device*. every call carries
//   `local_device_id` = this app's librespot device, and the dealer
//   connection it registered on (`Spotify-Connection-Id`). without them the
//   jam has nowhere to send its music - which is why jams used to join fine
//   here and then never play anything.
// - social-connect then drives that device like any Connect remote, from a
//   virtual `social-connect-<session>` device: it transfers the whole jam
//   (context, queue, position) over, and relays every skip, pause and queue
//   edit as ordinary connect commands. spirc plays them; see the "jam:"
//   patches in vendor/librespot-connect.
// - a guest's own skip / add-to-queue / play goes to the jam, not to their
//   player: POST sessions/{id}/commands/from/{device} with the same command
//   json a connect remote sends (the desktop core's redirectPlayerCommand).
// - a guest's pause is theirs alone, and resuming rejoins the jam where it is
//   now (spirc's jam hold).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamMember {
    pub id:              String,
    pub name:            String,
    pub image_url:       Option<String>,
    pub is_host:         bool,
    pub is_listening:    bool,
    pub is_current_user: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamSession {
    pub session_id: String,
    pub join_token: Option<String>,
    /// share this; opening it on another spotify client joins the jam
    pub join_url:   Option<String>,
    pub is_host:    bool,
    pub members:    Vec<JamMember>,
    /// guests can only add songs; playing and skipping is the host's
    pub queue_only_mode: bool,
    /// guest commands have to go through social-connect (see jam_command)
    pub redirect_commands: bool,
    /// the connect device the jam plays on, on the host's side
    pub host_device_id:   Option<String>,
    pub host_device_name: Option<String>,
    pub is_paused: bool,
}

/// first of several spellings: pushes come as protobuf json (camelCase),
/// REST replies as snake_case, and a few fields are camelCase in both
fn pick(v: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| v.get(*k))
        .and_then(|x| match x {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        })
        .filter(|s| !s.is_empty())
}

fn pick_bool(v: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter().find_map(|k| v.get(*k)).and_then(|x| x.as_bool())
}

pub(crate) fn jam_from(v: &Value) -> Option<JamSession> {
    let session_id = pick(v, &["session_id", "sessionId"])?;
    let owner = pick(v, &["session_owner_id", "sessionOwnerId"]);
    /* spotify hands back its internal address here
       (hm://social-connect/v2/sessions/join/<token>), which nothing outside
       spotify can open. the shareable link is the open.spotify.com one built
       from the same token */
    let raw_url = pick(v, &["join_session_url", "joinSessionUrl"]);
    let join_token = pick(v, &["join_session_token", "joinSessionToken"]).or_else(|| {
        raw_url.as_deref().and_then(|u| u.trim_end_matches('/').rsplit('/').next()).map(str::to_string)
    });
    let join_url = raw_url
        .filter(|u| u.starts_with("https://"))
        .or_else(|| join_token.as_ref().map(|t| format!("https://open.spotify.com/socialsession/{t}")));
    let members: Vec<JamMember> = ["session_members", "sessionMembers"]
        .iter()
        .find_map(|k| v.get(*k))
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = pick(m, &["id", "username"])?;
                    Some(JamMember {
                        name: pick(m, &["display_name", "displayName", "username"]).unwrap_or_else(|| id.clone()),
                        image_url: pick(m, &["image_url", "imageUrl", "large_image_url", "largeImageUrl"]),
                        is_host: owner.as_deref() == Some(id.as_str()),
                        is_listening: pick_bool(m, &["is_listening", "isListening", "listen_mode"]).unwrap_or(false),
                        is_current_user: pick_bool(m, &["is_current_user", "isCurrentUser"]).unwrap_or(false),
                        id,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let is_host = pick_bool(v, &["is_session_owner", "isSessionOwner"]).unwrap_or(false);
    let host_info = v.get("host_device_info").or_else(|| v.get("hostDeviceInfo"));
    Some(JamSession {
        session_id,
        join_token,
        join_url,
        is_host,
        members,
        queue_only_mode: pick_bool(v, &["queue_only_mode", "queueOnlyMode"]).unwrap_or(false),
        // absent on older replies; redirecting is what the official client
        // does for a guest either way
        redirect_commands: pick_bool(v, &["redirect_commands", "redirectCommands"]).unwrap_or(!is_host),
        host_device_id: pick(v, &["hostActiveDeviceId", "host_active_device_id"]),
        host_device_name: host_info.and_then(|h| pick(h, &["device_name", "deviceName"])),
        is_paused: pick_bool(v, &["is_jam_paused", "isJamPaused", "is_paused", "isPaused"]).unwrap_or(false),
    })
}

fn safe_segment(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// a social-connect call the way the official client makes it: bound to this
/// connect device's dealer connection, and never a bodyless POST (the google
/// frontend in front of spclient answers those with 411 before spotify sees
/// them)
async fn social_connect(app: &AppHandle, method: reqwest::Method, endpoint: &str, body: Option<Value>) -> Result<Value, AppError> {
    let session = spclient::session(app).await?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));
    if let Ok(id) = reqwest::header::HeaderValue::from_str(&session.connection_id()) {
        if !id.is_empty() {
            headers.insert("Spotify-Connection-Id", id);
        }
    }
    let body = match (&method, body) {
        (_, Some(b)) => Some(b.to_string()),
        (&reqwest::Method::GET, None) => None,
        (_, None) => Some("{}".to_string()),
    };
    let bytes = session
        .spclient()
        .request_as_json(&method, endpoint, Some(headers), body.as_deref())
        .await
        .map_err(spclient::map_err)?;
    if bytes.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_slice(&bytes).map_err(|e| AppError::Network(format!("decode {endpoint}: {e}")))
}

/// several endpoint spellings exist across client versions; take the first
/// one that answers
async fn first_ok(app: &AppHandle, calls: &[(reqwest::Method, String)]) -> Result<Value, AppError> {
    let mut last = AppError::Network("no endpoint tried".into());
    for (method, endpoint) in calls {
        match social_connect(app, method.clone(), endpoint, None).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                eprintln!("[jam] {method} {endpoint} failed: {e}");
                last = e;
            }
        }
    }
    Err(last)
}

async fn local_device(app: &AppHandle) -> Result<String, AppError> {
    Ok(spclient::session(app).await?.device_id().to_string())
}

/// the join token out of anything a person might paste: the open.spotify.com
/// link (any locale / path shape), spotify:socialsession:<token>, a bare
/// token, or a spotify.link short link (which has to be followed first - its
/// own last segment is a short-link id, and joining with it is a 400)
async fn join_token_from(link: &str) -> Result<String, AppError> {
    fn last_segment(s: &str) -> String {
        s.split(['?', '#'])
            .next()
            .unwrap_or_default()
            .trim_end_matches('/')
            .rsplit(['/', ':'])
            .next()
            .unwrap_or_default()
            .to_string()
    }
    fn token_in(text: &str) -> Option<String> {
        let at = text.find("socialsession")?;
        let mut rest = &text[at + "socialsession".len()..];
        // the separator, possibly url-encoded inside a redirect
        loop {
            if let Some(r) = rest.strip_prefix('/').or_else(|| rest.strip_prefix(':')) {
                rest = r;
            } else if rest.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("%2F") || p.eq_ignore_ascii_case("%3A")) {
                rest = &rest[3..];
            } else {
                break;
            }
        }
        let token: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
        (!token.is_empty()).then_some(token)
    }

    let link = link.trim();
    if link.contains("socialsession") {
        if let Some(t) = token_in(link) {
            return Ok(t);
        }
    }
    let is_short = link.contains("spotify.link/") || link.contains("spotify.app.link/");
    if is_short {
        let url = if link.starts_with("http") { link.to_string() } else { format!("https://{link}") };
        let resp = crate::http::client()
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("couldn't open the invite link: {e}")))?;
        let landed = resp.url().to_string();
        if let Some(t) = token_in(&landed) {
            return Ok(t);
        }
        // branch links often answer a non-browser with a page that redirects
        // in script; the destination is still in it
        let body = resp.text().await.unwrap_or_default();
        return token_in(&body).ok_or_else(|| AppError::InvalidInput("that link isn't a Jam invite".into()));
    }
    let token = last_segment(link);
    if safe_segment(&token) {
        Ok(token)
    } else {
        Err(AppError::InvalidInput("that doesn't look like a jam link".into()))
    }
}

/// the jam you're in, if any
#[tauri::command]
pub async fn get_jam(app: AppHandle) -> Result<Option<JamSession>, AppError> {
    let device = local_device(&app).await?;
    match social_connect(
        &app,
        reqwest::Method::GET,
        &format!("/social-connect/v2/sessions/current?local_device_id={device}"),
        None,
    )
    .await
    {
        Ok(v) => Ok(jam_from(&v)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// start a jam hosted on this device (or return the one already running)
#[tauri::command]
pub async fn start_jam(app: AppHandle) -> Result<JamSession, AppError> {
    let device = local_device(&app).await?;
    let v = social_connect(
        &app,
        reqwest::Method::GET,
        &format!("/social-connect/v2/sessions/current_or_new?activate=true&local_device_id={device}&type=REMOTE"),
        None,
    )
    .await?;
    jam_from(&v).ok_or_else(|| AppError::Network("jam: no session in response".into()))
}

/// join someone's jam from its share link or bare token. social-connect then
/// transfers the jam's playback onto this device
#[tauri::command]
pub async fn join_jam(app: AppHandle, link: String) -> Result<Option<JamSession>, AppError> {
    let token = join_token_from(&link).await?;
    if !safe_segment(&token) {
        return Err(AppError::InvalidInput("that doesn't look like a jam link".into()));
    }
    let device = local_device(&app).await?;
    let v = social_connect(
        &app,
        reqwest::Method::POST,
        &format!(
            "/social-connect/v2/sessions/join/{token}?playback_control=listen_and_control&join_type=deeplinking&local_device_id={device}"
        ),
        None,
    )
    .await?;
    Ok(jam_from(&v))
}

/// the host ends the jam for everyone; a guest just leaves
#[tauri::command]
pub async fn leave_jam(app: AppHandle, session_id: String, is_host: bool) -> Result<(), AppError> {
    if !safe_segment(&session_id) {
        return Err(AppError::InvalidInput("bad session id".into()));
    }
    let device = local_device(&app).await?;
    let calls = if is_host {
        vec![
            (reqwest::Method::DELETE, format!("/social-connect/v3/sessions/{session_id}")),
            (reqwest::Method::DELETE, format!("/social-connect/v2/sessions/{session_id}")),
        ]
    } else {
        vec![
            (reqwest::Method::POST, format!("/social-connect/v3/sessions/{session_id}/leave")),
            (reqwest::Method::POST, format!("/social-connect/v2/sessions/{session_id}/leave")),
            (reqwest::Method::POST, format!("/social-connect/v2/sessions/leave?local_device_id={device}")),
        ]
    };
    first_ok(&app, &calls).await?;
    Ok(())
}

/// host: whether guests may only add songs (on) or also play and skip (off)
#[tauri::command]
pub async fn set_jam_queue_only(app: AppHandle, enabled: bool) -> Result<Option<JamSession>, AppError> {
    let mode = if enabled { "enabled" } else { "disabled" };
    let v = social_connect(
        &app,
        reqwest::Method::PUT,
        &format!("/social-connect/v2/sessions/current/queue_only_mode/{mode}"),
        None,
    )
    .await?;
    Ok(jam_from(&v))
}

/// host: remove someone from the jam
#[tauri::command]
pub async fn kick_jam_member(app: AppHandle, session_id: String, member_id: String) -> Result<(), AppError> {
    if !safe_segment(&session_id) || !safe_segment(&member_id) {
        return Err(AppError::InvalidInput("bad jam member".into()));
    }
    social_connect(
        &app,
        reqwest::Method::POST,
        &format!("/social-connect/v3/sessions/{session_id}/member/{member_id}/kick"),
        None,
    )
    .await?;
    Ok(())
}

/// a guest's player command, sent to the jam instead of their own player.
/// `command` is connect's command json ({"endpoint": "skip_next", ...})
#[tauri::command]
pub async fn jam_command(app: AppHandle, session_id: String, mut command: Value) -> Result<(), AppError> {
    if !safe_segment(&session_id) {
        return Err(AppError::InvalidInput("bad session id".into()));
    }
    let endpoint = command.get("endpoint").and_then(|e| e.as_str()).unwrap_or_default();
    const ALLOWED: &[&str] = &["add_to_queue", "skip_next", "skip_prev", "play", "set_queue", "pause", "resume", "seek_to"];
    if !ALLOWED.contains(&endpoint) {
        return Err(AppError::InvalidInput(format!("not a jam command: {endpoint}")));
    }
    let device = local_device(&app).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let command_id: String = (0..32).map(|_| format!("{:x}", rand::random::<u8>() & 0xf)).collect();
    if let Some(obj) = command.as_object_mut() {
        obj.entry("logging_params").or_insert(json!({
            "command_initiated_time": now,
            "device_identifier": device,
            "command_id": command_id,
        }));
    }
    social_connect(
        &app,
        reqwest::Method::POST,
        &format!("/social-connect/v2/sessions/{session_id}/commands/from/{device}"),
        Some(json!({ "command": command })),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod jam_tests {
    use super::*;

    #[test]
    fn parses_session() {
        let v: Value = serde_json::from_str(
            r#"{"session_id": "abc", "join_session_token": "tok123", "session_owner_id": "me", "is_session_owner": true,
                "queue_only_mode": true,
                "session_members": [{"id": "me", "display_name": "Me", "is_current_user": true}, {"id": "pal", "username": "pal", "is_listening": true}]}"#,
        )
        .unwrap();
        let j = jam_from(&v).unwrap();
        assert_eq!(j.join_url.as_deref(), Some("https://open.spotify.com/socialsession/tok123"));
        assert!(j.is_host);
        assert!(j.queue_only_mode);
        assert!(!j.redirect_commands, "a host plays on their own device");
        assert!(j.members[0].is_host);
        assert!(j.members[0].is_current_user);
        assert_eq!(j.members[1].name, "pal");
        assert!(j.members[1].is_listening);

        // the internal hm:// address is never what gets shared
        let v: Value = serde_json::from_str(
            r#"{"session_id": "abc", "join_session_url": "hm://social-connect/v2/sessions/join/4Oxc"}"#,
        )
        .unwrap();
        let guest = jam_from(&v).unwrap();
        assert_eq!(guest.join_url.as_deref(), Some("https://open.spotify.com/socialsession/4Oxc"));
        assert!(guest.redirect_commands, "a guest's commands go to the jam");
    }

    #[test]
    fn parses_pushed_session() {
        // dealer pushes are protobuf json
        let v: Value = serde_json::from_str(
            r#"{"sessionId": "s1", "joinSessionToken": "t", "sessionOwnerId": "h", "isSessionOwner": false,
                "sessionMembers": [{"id": "h", "displayName": "Host"}], "hostActiveDeviceId": "dev"}"#,
        )
        .unwrap();
        let j = jam_from(&v).unwrap();
        assert_eq!(j.session_id, "s1");
        assert_eq!(j.members[0].name, "Host");
        assert!(j.members[0].is_host);
        assert_eq!(j.host_device_id.as_deref(), Some("dev"));
    }

    #[tokio::test]
    async fn join_tokens() {
        for link in [
            "https://open.spotify.com/socialsession/AbC123?si=x",
            "https://open.spotify.com/intl-de/socialsession/AbC123",
            "spotify:socialsession:AbC123",
            "https://x/?u=spotify%3Asocialsession%3AAbC123&y",
            "AbC123",
        ] {
            assert_eq!(join_token_from(link).await.unwrap(), "AbC123", "{link}");
        }
        assert!(join_token_from("not a link!").await.is_err());
    }
}
