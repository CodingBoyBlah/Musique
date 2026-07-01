use crate::errors::AppError;
use serde::Deserialize;

#[derive(Deserialize)]
struct OdesliResp {
    #[serde(rename = "pageUrl")]
    page_url: Option<String>,
}

/// turn a spotify entity url into its odesli (song.link) universal page url,
/// one link that opens in whatever streaming service the other person uses.
/// routed thru rust not the webview so we dodge cors and keep it offline safe
#[tauri::command]
pub async fn resolve_odesli(url: String) -> Result<String, AppError> {
    let resp = reqwest::Client::new()
        .get("https://api.song.link/v1-alpha.1/links")
        .query(&[("url", url.as_str())])
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(AppError::Network(format!("song.link returned {}", resp.status())));
    }

    let body: OdesliResp = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    body.page_url
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::NotFound("no universal link for this item".into()))
}
