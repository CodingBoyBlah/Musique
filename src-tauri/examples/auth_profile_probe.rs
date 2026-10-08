// Read-only account diagnostics. Prints no usernames, URLs or credentials.
use librespot_core::{authentication::Credentials, config::SessionConfig, Session};
use std::{path::PathBuf, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = PathBuf::from(std::env::var("APPDATA")?).join("dev.boyblah.musique");
    let credentials: Credentials = serde_json::from_slice(&std::fs::read(data.join("credentials/credentials.json"))?)?;
    let session = Session::new(SessionConfig::default(), None);
    tokio::time::timeout(Duration::from_secs(20), session.connect(credentials, false)).await??;
    let username = session.username();
    let token = tokio::time::timeout(Duration::from_secs(15), session.login5().auth_token()).await??;
    let client = reqwest::Client::builder().timeout(Duration::from_secs(15)).build()?;
    let response = client.get("https://api.spotify.com/v1/me").bearer_auth(&token.access_token).send().await?;
    println!("first_party_profile_status={}", response.status().as_u16());
    if response.status().is_success() {
        let p: serde_json::Value = response.json().await?;
        println!("first_party_premium={} avatar_present={} account_matches_session={}",
            p["product"].as_str() == Some("premium"),
            p["images"].as_array().is_some_and(|v| !v.is_empty()),
            p["id"].as_str() == Some(username.as_str()));
    }
    let endpoint = format!("/user-profile-view/v3/profile/{username}?playlist_limit=0&artist_limit=0");
    let bytes = tokio::time::timeout(Duration::from_secs(15),
        session.spclient().request_as_json(&reqwest::Method::GET, &endpoint, None, None)).await??;
    let view: serde_json::Value = serde_json::from_slice(&bytes)?;
    println!("session_premium={} profile_name_present={} avatar_present={}",
        session.get_user_attribute("type").as_deref() == Some("premium"),
        view["name"].as_str().is_some_and(|s| !s.is_empty()),
        view["image_url"].as_str().is_some_and(|s| !s.is_empty()));
    session.shutdown();
    Ok(())
}
