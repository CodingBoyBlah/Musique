//! the social layer: friend activity (what the people you follow are
//! playing) and jam sessions. both live on spclient only.

use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamMember {
    pub id:          String,
    pub name:        String,
    pub image_url:   Option<String>,
    pub is_host:     bool,
    pub is_listening: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamSession {
    pub session_id: String,
    pub join_token: Option<String>,
    /// share this; opening it on another spotify client joins the jam
    pub join_url:   Option<String>,
    pub is_host:    bool,
    pub members:    Vec<JamMember>,
}

pub(crate) fn jam_from(v: &Value) -> Option<JamSession> {
    let session_id = s(v, &["session_id"])?;
    let owner = s(v, &["session_owner_id"]);
    let join_token = s(v, &["join_session_token"]);
    let join_url = s(v, &["join_session_url"])
        .or_else(|| join_token.as_ref().map(|t| format!("https://open.spotify.com/socialsession/{t}")));
    let members = v
        .get("session_members")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = s(m, &["id"]).or_else(|| s(m, &["username"]))?;
                    Some(JamMember {
                        name: s(m, &["display_name"]).or_else(|| s(m, &["username"])).unwrap_or_else(|| id.clone()),
                        image_url: s(m, &["image_url"]).or_else(|| s(m, &["large_image_url"])),
                        is_host: owner.as_deref() == Some(id.as_str()),
                        is_listening: m.get("is_listening").and_then(|x| x.as_bool()).unwrap_or(false),
                        id,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(JamSession {
        session_id,
        join_token,
        join_url,
        is_host: v.get("is_session_owner").and_then(|x| x.as_bool()).unwrap_or(false),
        members,
    })
}

/// the jam you're in, if any
#[tauri::command]
pub async fn get_jam(app: AppHandle) -> Result<Option<JamSession>, AppError> {
    match spclient::get_json_value(&app, "/social-connect/v2/sessions/current").await {
        Ok(v) => Ok(jam_from(&v)),
        Err(AppError::NotFound(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// start a jam hosted on this device (or return the one already running)
#[tauri::command]
pub async fn start_jam(app: AppHandle) -> Result<JamSession, AppError> {
    let device = spclient::session(&app).await?.device_id().to_string();
    let v = spclient::get_json_value(
        &app,
        &format!("/social-connect/v2/sessions/current_or_new?local_device_id={device}&type=REMOTE"),
    )
    .await?;
    jam_from(&v).ok_or_else(|| AppError::Network("jam: no session in response".into()))
}

/// join someone's jam from its share link or bare token
#[tauri::command]
pub async fn join_jam(app: AppHandle, link: String) -> Result<Option<JamSession>, AppError> {
    let token = link
        .trim()
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::InvalidInput("that doesn't look like a jam link".into()));
    }
    let v = spclient::send_json(
        &app,
        reqwest::Method::POST,
        &format!("/social-connect/v2/sessions/join/{token}?playback_control=listen_and_control&join_type=deeplinking"),
        None,
    )
    .await?;
    Ok(jam_from(&v))
}

/// the host ends the jam for everyone; a guest just leaves
#[tauri::command]
pub async fn leave_jam(app: AppHandle, session_id: String, is_host: bool) -> Result<(), AppError> {
    if !session_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::InvalidInput("bad session id".into()));
    }
    if is_host {
        spclient::send_json(&app, reqwest::Method::DELETE, &format!("/social-connect/v2/sessions/{session_id}"), None).await?;
    } else {
        spclient::send_json(&app, reqwest::Method::POST, &format!("/social-connect/v2/sessions/{session_id}/leave"), None).await?;
    }
    Ok(())
}

#[cfg(test)]
mod jam_tests {
    use super::*;

    #[test]
    fn parses_session() {
        let v: Value = serde_json::from_str(
            r#"{"session_id": "abc", "join_session_token": "tok123", "session_owner_id": "me", "is_session_owner": true,
                "session_members": [{"id": "me", "display_name": "Me"}, {"id": "pal", "username": "pal", "is_listening": true}]}"#,
        )
        .unwrap();
        let j = jam_from(&v).unwrap();
        assert_eq!(j.join_url.as_deref(), Some("https://open.spotify.com/socialsession/tok123"));
        assert!(j.is_host);
        assert!(j.members[0].is_host);
        assert_eq!(j.members[1].name, "pal");
        assert!(j.members[1].is_listening);
    }
}
