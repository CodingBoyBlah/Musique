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
