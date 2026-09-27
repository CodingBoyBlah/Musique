//! user profiles - yours and anyone else's.
//!
//! the rich view (follower/following counts, public playlists, recently played
//! artists, the follower lists themselves) only exists on spclient's
//! `user-profile-view`. the public web api gives name/avatar/followers and the
//! public playlists, so it's the fallback when the internal call fails.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
    commands::spotify::{tok, BASE},
    errors::AppError,
    internal::{cache, spclient},
    spotify::types::{ArtistItem, PlaylistCard},
    state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id:                      String,
    pub name:                    String,
    pub image_url:               Option<String>,
    pub followers:               Option<i64>,
    pub following:               Option<i64>,
    pub total_public_playlists:  Option<i64>,
    pub playlists:               Vec<PlaylistCard>,
    pub recently_played_artists: Vec<ArtistItem>,
    pub is_verified:             bool,
    pub spotify_url:             String,
}

/// one row in a followers / following list. `kind` is "user" or "artist" -
/// "following" mixes both
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCard {
    pub id:        String,
    pub kind:      String,
    pub name:      String,
    pub image_url: Option<String>,
    pub followers: Option<i64>,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(str::to_string).filter(|x| !x.is_empty())
}

fn i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(|x| x.as_i64())
}

/// profile-view images come as https urls, `spotify:image:<id>`, or for
/// generated playlist covers `spotify:mosaic:<id>:<id>:...` - first tile wins
pub(crate) fn image(raw: Option<String>) -> Option<String> {
    let raw = raw?;
    if let Some(rest) = raw.strip_prefix("spotify:mosaic:") {
        let first = rest.split([':', ',']).next()?;
        return spclient::image_uri_to_url(first);
    }
    spclient::image_uri_to_url(&raw)
}

fn card_from_view(v: &Value) -> Option<ProfileCard> {
    let uri = s(v, "uri")?;
    let kind = if uri.starts_with("spotify:artist:") { "artist" } else { "user" };
    Some(ProfileCard {
        id: spclient::uri_id(&uri).to_string(),
        kind: kind.into(),
        name: s(v, "name").unwrap_or_else(|| spclient::uri_id(&uri).to_string()),
        image_url: image(s(v, "image_url")),
        followers: i(v, "followers_count"),
    })
}

fn profile_from_view(id: &str, v: &Value) -> UserProfile {
    let playlists = v
        .get("public_playlists")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let uri = s(p, "uri")?;
                    Some(PlaylistCard {
                        id: spclient::uri_id(&uri).to_string(),
                        name: s(p, "name").unwrap_or_default(),
                        description: None,
                        image_url: image(s(p, "image_url")),
                        owner_name: s(p, "owner_name"),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let artists = v
        .get("recently_played_artists")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| {
                    let uri = s(a, "uri")?;
                    Some(ArtistItem {
                        id: spclient::uri_id(&uri).to_string(),
                        name: s(a, "name").unwrap_or_default(),
                        image_url: image(s(a, "image_url")),
                        popularity: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    UserProfile {
        id: id.to_string(),
        name: s(v, "name").unwrap_or_else(|| id.to_string()),
        image_url: image(s(v, "image_url")),
        followers: i(v, "followers_count"),
        following: i(v, "following_count"),
        total_public_playlists: i(v, "total_public_playlists_count"),
        playlists,
        recently_played_artists: artists,
        is_verified: v.get("is_verified").and_then(|x| x.as_bool()).unwrap_or(false),
        spotify_url: format!("https://open.spotify.com/user/{id}"),
    }
}

async fn profile_from_web(app: &AppHandle, id: &str) -> Result<UserProfile, AppError> {
    let token = tok(app).await?;
    let user: Value = crate::spotify::spotify_get(&token, &format!("{BASE}/users/{id}")).await?;
    let pls: Value = crate::spotify::spotify_get(&token, &format!("{BASE}/users/{id}/playlists?limit=50"))
        .await
        .unwrap_or(Value::Null);

    let playlists: Vec<PlaylistCard> = pls
        .get("items")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|p| !p.is_null())
                .filter_map(|p| {
                    Some(PlaylistCard {
                        id: s(p, "id")?,
                        name: s(p, "name").unwrap_or_default(),
                        description: s(p, "description"),
                        image_url: p
                            .get("images")
                            .and_then(|x| x.as_array())
                            .and_then(|a| a.first())
                            .and_then(|im| s(im, "url")),
                        owner_name: p.get("owner").and_then(|o| s(o, "display_name")),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(UserProfile {
        id: id.to_string(),
        name: s(&user, "display_name").unwrap_or_else(|| id.to_string()),
        image_url: user
            .get("images")
            .and_then(|x| x.as_array())
            .and_then(|a| a.first())
            .and_then(|im| s(im, "url")),
        followers: user.get("followers").and_then(|f| i(f, "total")),
        following: None,
        total_public_playlists: pls.get("total").and_then(|x| x.as_i64()),
        playlists,
        recently_played_artists: Vec::new(),
        is_verified: false,
        spotify_url: format!("https://open.spotify.com/user/{id}"),
    })
}

fn clean_id(id: &str) -> Result<String, AppError> {
    let id = spclient::uri_id(id.trim()).to_string();
    if id.is_empty() || id.contains('/') || id.contains('?') {
        return Err(AppError::InvalidInput(format!("bad user id: {id}")));
    }
    Ok(id)
}

/// the profile page. `id` is a spotify username / user id; omitted means you
#[tauri::command]
pub async fn get_user_profile(app: AppHandle, id: Option<String>) -> Result<UserProfile, AppError> {
    let id = match id {
        Some(id) => clean_id(&id)?,
        None => spclient::username(&app).await?,
    };
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:user:{id}");

    cache::cached_json(&pool, &uri, "profile", cache::HOUR, || async {
        let endpoint = format!("/user-profile-view/v3/profile/{id}?playlist_limit=50&artist_limit=20&market=from_token");
        match spclient::get_json_value(&app, &endpoint).await {
            Ok(v) if v.is_object() => {
                let mut p = profile_from_view(&id, &v);
                // profile-view can omit the playlist list for some accounts;
                // the web api always has the public ones
                if p.playlists.is_empty() {
                    if let Ok(web) = profile_from_web(&app, &id).await {
                        p.playlists = web.playlists;
                        p.total_public_playlists = p.total_public_playlists.or(web.total_public_playlists);
                    }
                }
                Ok(p)
            }
            _ => profile_from_web(&app, &id).await,
        }
    })
    .await
}

async fn follow_list(app: &AppHandle, id: Option<String>, which: &str) -> Result<Vec<ProfileCard>, AppError> {
    let id = match id {
        Some(id) => clean_id(&id)?,
        None => spclient::username(app).await?,
    };
    let pool = app.state::<AppState>().db.clone();
    let uri = format!("spotify:user:{id}");
    cache::cached_json(&pool, &uri, which, cache::HOUR, || async {
        let v = spclient::get_json_value(app, &format!("/user-profile-view/v3/profile/{id}/{which}?market=from_token")).await?;
        Ok(v.get("profiles")
            .and_then(|x| x.as_array())
            .map(|arr| arr.iter().filter_map(card_from_view).collect())
            .unwrap_or_default())
    })
    .await
}

#[tauri::command]
pub async fn get_user_followers(app: AppHandle, id: Option<String>) -> Result<Vec<ProfileCard>, AppError> {
    follow_list(&app, id, "followers").await
}

#[tauri::command]
pub async fn get_user_following(app: AppHandle, id: Option<String>) -> Result<Vec<ProfileCard>, AppError> {
    follow_list(&app, id, "following").await
}

/// follow / unfollow another user (web api, needs user-follow-modify)
#[tauri::command]
pub async fn set_user_followed(app: AppHandle, id: String, follow: bool) -> Result<(), AppError> {
    let id = clean_id(&id)?;
    let token = tok(&app).await?;
    let method = if follow { reqwest::Method::PUT } else { reqwest::Method::DELETE };
    crate::spotify::spotify_write(&token, method, &format!("{BASE}/me/following?type=user&ids={id}")).await?;
    // counts on both profiles just changed
    let pool = app.state::<AppState>().db.clone();
    let _ = sqlx::query("DELETE FROM extension_cache WHERE kind IN ('profile', 'followers', 'following')")
        .execute(&pool)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn is_user_followed(app: AppHandle, id: String) -> Result<bool, AppError> {
    let id = clean_id(&id)?;
    let token = tok(&app).await?;
    let v: Vec<bool> =
        crate::spotify::spotify_get(&token, &format!("{BASE}/me/following/contains?type=user&ids={id}")).await?;
    Ok(v.first().copied().unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_profile_view() {
        let v: Value = serde_json::from_str(
            r#"{
              "uri": "spotify:user:abc",
              "name": "Someone",
              "image_url": "https://i.scdn.co/image/ab67",
              "followers_count": 12,
              "following_count": 3,
              "total_public_playlists_count": 1,
              "public_playlists": [
                {"uri": "spotify:playlist:pl1", "name": "Mix", "image_url": "spotify:mosaic:aa:bb:cc:dd", "owner_name": "Someone"}
              ],
              "recently_played_artists": [
                {"uri": "spotify:artist:ar1", "name": "Band", "image_url": "spotify:image:ff"}
              ]
            }"#,
        )
        .unwrap();
        let p = profile_from_view("abc", &v);
        assert_eq!(p.name, "Someone");
        assert_eq!(p.followers, Some(12));
        assert_eq!(p.playlists[0].id, "pl1");
        assert_eq!(p.playlists[0].image_url.as_deref(), Some("https://i.scdn.co/image/aa"));
        assert_eq!(p.recently_played_artists[0].image_url.as_deref(), Some("https://i.scdn.co/image/ff"));
    }

    #[test]
    fn follow_cards_know_their_kind() {
        let v: Value = serde_json::from_str(r#"{"uri": "spotify:artist:x", "name": "A"}"#).unwrap();
        assert_eq!(card_from_view(&v).unwrap().kind, "artist");
        let v: Value = serde_json::from_str(r#"{"uri": "spotify:user:y"}"#).unwrap();
        let c = card_from_view(&v).unwrap();
        assert_eq!((c.kind.as_str(), c.name.as_str()), ("user", "y"));
    }
}
