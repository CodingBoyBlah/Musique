//! the parts of an artist page the public api doesn't have: biography, related
//! artists ("fans also like" - the public endpoint is gone for new apps),
//! portrait gallery, active years, and appears-on. all from ARTIST_V4.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{
    errors::AppError,
    internal::{cache, metadata},
    spotify::types::{AlbumItem, ArtistItem},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtistExtras {
    pub biography:    Option<String>,
    pub gallery:      Vec<String>,
    pub active_years: Option<String>,
    pub related:      Vec<ArtistItem>,
    pub appears_on:   Vec<AlbumItem>,
    pub compilations: Vec<AlbumItem>,
}

/// spotify biographies are light html (`<a href="spotify:artist:..">Name</a>`,
/// the odd <br>, entities). the page renders text, so flatten it
pub(crate) fn strip_html(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in raw.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let t = tag.trim().to_ascii_lowercase();
                if t.starts_with("br") || t == "p" || t == "/p" {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(c),
            _ => out.push(c),
        }
    }
    let out = out
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    // collapse runs of blank lines the tag removal left behind
    let mut lines: Vec<&str> = Vec::new();
    for line in out.lines().map(str::trim) {
        if line.is_empty() && lines.last().map(|l| l.is_empty()).unwrap_or(true) {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_string()
}

fn years(periods: &[librespot_protocol::metadata::ActivityPeriod]) -> Option<String> {
    let starts = periods.iter().filter_map(|p| {
        if p.has_start_year() {
            Some(p.start_year())
        } else if p.has_decade() {
            Some(p.decade())
        } else {
            None
        }
    });
    let start = starts.min()?;
    let open = periods.iter().any(|p| !p.has_end_year() && !p.has_decade());
    let end = periods.iter().filter(|p| p.has_end_year()).map(|p| p.end_year()).max();
    Some(match (open, end) {
        (true, _) | (false, None) => format!("{start} - present"),
        (false, Some(e)) if e == start => format!("{start}"),
        (false, Some(e)) => format!("{start} - {e}"),
    })
}

fn group_ids(groups: &[librespot_protocol::metadata::AlbumGroup], max: usize) -> Vec<String> {
    let mut ids = Vec::new();
    for g in groups {
        // each group lists regional versions of one release; the first stands in
        if let Some(a) = g.album.first() {
            if let Some(id) = metadata::gid_to_id(a.gid()) {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        if ids.len() >= max {
            break;
        }
    }
    ids
}

#[tauri::command]
pub async fn get_artist_extras(app: AppHandle, id: String) -> Result<ArtistExtras, AppError> {
    use tauri::Manager;
    let pool = app.state::<crate::state::AppState>().db.clone();
    let uri = format!("spotify:artist:{id}");
    cache::cached_json(&pool, &uri, "artist-extras", cache::DAY, || async {
        let a = metadata::artist_proto(&app, &id, cache::DAY).await?;

        let biography = a
            .biography
            .iter()
            .map(|b| strip_html(b.text()))
            .find(|t| !t.is_empty());

        let mut gallery: Vec<String> = Vec::new();
        for b in &a.biography {
            for g in &b.portrait_group {
                if let Some(url) = metadata::best_image(Some(g), &[]) {
                    if !gallery.contains(&url) {
                        gallery.push(url);
                    }
                }
            }
            if let Some(url) = metadata::best_image(None, &b.portrait) {
                if !gallery.contains(&url) {
                    gallery.push(url);
                }
            }
        }

        let related_ids: Vec<String> = a.related.iter().filter_map(|r| metadata::gid_to_id(r.gid())).take(20).collect();
        let related = metadata::artists(&app, &related_ids).await.unwrap_or_default();
        let appears_on = metadata::albums(&app, &group_ids(&a.appears_on_group, 20)).await.unwrap_or_default();
        let compilations = metadata::albums(&app, &group_ids(&a.compilation_group, 20)).await.unwrap_or_default();

        Ok(ArtistExtras {
            biography,
            gallery,
            active_years: years(&a.activity_period),
            related,
            appears_on,
            compilations,
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_bios() {
        let raw = r#"Formed in <a href="spotify:artist:x">Leeds</a> &amp; London.<br/><br/><br/>Still going."#;
        assert_eq!(strip_html(raw), "Formed in Leeds & London.\n\nStill going.");
    }

    #[test]
    fn active_years() {
        use librespot_protocol::metadata::ActivityPeriod;
        let mut a = ActivityPeriod::new();
        a.set_start_year(1994);
        assert_eq!(years(&[a.clone()]).as_deref(), Some("1994 - present"));
        a.set_end_year(2008);
        assert_eq!(years(&[a]).as_deref(), Some("1994 - 2008"));
        assert_eq!(years(&[]), None);
    }
}

// ── artist overview (pathfinder) ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopCity {
    pub city:      String,
    pub country:   Option<String>,
    pub listeners: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Concert {
    pub title: String,
    /// iso-8601
    pub date:  Option<String>,
    pub venue: Option<String>,
    pub city:  Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MerchItem {
    pub name:      String,
    pub price:     Option<String>,
    pub url:       Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalLink {
    pub name: String,
    pub url:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtistOverview {
    pub monthly_listeners: Option<i64>,
    pub followers:         Option<i64>,
    pub world_rank:        Option<i64>,
    pub verified:          bool,
    pub top_cities:        Vec<TopCity>,
    pub header_image:      Option<String>,
    pub gallery:           Vec<String>,
    pub external_links:    Vec<ExternalLink>,
    /// playlists the artist was discovered on / is featured in
    pub discovered_on:     Vec<crate::commands::home_feed::HomeItem>,
    pub featuring:         Vec<crate::commands::home_feed::HomeItem>,
    pub concerts:          Vec<Concert>,
    pub merch:             Vec<MerchItem>,
}

fn arr<'a>(v: &'a serde_json::Value, path: &[&str]) -> Vec<&'a serde_json::Value> {
    crate::internal::pathfinder::get(v, path)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn first_source(v: &serde_json::Value) -> Option<String> {
    v.get("sources")?
        .as_array()?
        .iter()
        .filter_map(|s| Some((s.get("url")?.as_str()?, s.get("width").and_then(|w| w.as_i64()).unwrap_or(0))))
        .max_by_key(|(_, w)| *w)
        .map(|(u, _)| u.to_string())
}

pub(crate) fn overview_from(data: &serde_json::Value) -> ArtistOverview {
    use crate::internal::pathfinder::{get, get_str};
    let a = get(data, &["artistUnion"]).cloned().unwrap_or_default();
    let n = |path: &[&str]| get(&a, path).and_then(|x| x.as_i64());
    let items = |path: &[&str]| -> Vec<crate::commands::home_feed::HomeItem> {
        arr(&a, path).into_iter().filter_map(crate::commands::home_feed::item_from).collect()
    };
    ArtistOverview {
        monthly_listeners: n(&["stats", "monthlyListeners"]),
        followers: n(&["stats", "followers"]),
        world_rank: n(&["stats", "worldRank"]).filter(|r| *r > 0),
        verified: get(&a, &["profile", "verified"]).and_then(|x| x.as_bool()).unwrap_or(false),
        top_cities: arr(&a, &["stats", "topCities", "items"])
            .into_iter()
            .filter_map(|c| {
                Some(TopCity {
                    city: get_str(c, &["city"])?.to_string(),
                    country: get_str(c, &["country"]).map(str::to_string),
                    listeners: c.get("numberOfListeners").and_then(|x| x.as_i64()),
                })
            })
            .collect(),
        header_image: get(&a, &["visuals", "headerImage"]).and_then(first_source),
        gallery: arr(&a, &["visuals", "gallery", "items"]).into_iter().filter_map(first_source).collect(),
        external_links: arr(&a, &["profile", "externalLinks", "items"])
            .into_iter()
            .filter_map(|l| {
                let url = get_str(l, &["url"])?;
                url.starts_with("https://").then(|| ExternalLink {
                    name: get_str(l, &["name"]).unwrap_or("Link").to_string(),
                    url: url.to_string(),
                })
            })
            .collect(),
        discovered_on: items(&["relatedContent", "discoveredOnV2", "items"]),
        featuring: items(&["relatedContent", "featuringV2", "items"]),
        concerts: arr(&a, &["goods", "events", "concerts", "items"])
            .into_iter()
            .filter_map(|c| {
                Some(Concert {
                    title: get_str(c, &["title"]).or_else(|| get_str(c, &["venue", "name"]))?.to_string(),
                    date: get_str(c, &["date", "isoString"]).map(str::to_string),
                    venue: get_str(c, &["venue", "name"]).map(str::to_string),
                    city: get_str(c, &["venue", "location", "name"]).map(str::to_string),
                })
            })
            .collect(),
        merch: arr(&a, &["goods", "merch", "items"])
            .into_iter()
            .filter_map(|m| {
                Some(MerchItem {
                    name: get_str(m, &["name"])?.to_string(),
                    price: get_str(m, &["price"]).map(str::to_string),
                    url: get_str(m, &["url"]).filter(|u| u.starts_with("https://")).map(str::to_string),
                    image_url: get(m, &["image"]).and_then(first_source),
                })
            })
            .collect(),
    }
}

/// monthly listeners, world rank, top cities, discovered-on, concerts, merch,
/// socials - the artist page's "about" data from pathfinder
#[tauri::command]
pub async fn get_artist_overview(app: AppHandle, id: String) -> Result<ArtistOverview, AppError> {
    use tauri::Manager;
    let pool = app.state::<crate::state::AppState>().db.clone();
    let uri = format!("spotify:artist:{id}");
    cache::cached_json(&pool, &uri, "artist-overview", 6 * cache::HOUR, || async {
        let data = crate::internal::pathfinder::query(
            &app,
            "queryArtistOverview",
            serde_json::json!({ "uri": uri, "locale": "", "preReleaseV2": false }),
        )
        .await?;
        Ok(overview_from(&data))
    })
    .await
}

#[cfg(test)]
mod overview_tests {
    use super::*;

    #[test]
    fn parses_overview() {
        let v = serde_json::json!({"artistUnion": {
            "profile": {"verified": true, "externalLinks": {"items": [
                {"name": "INSTAGRAM", "url": "https://instagram.com/x"}, {"name": "BAD", "url": "javascript:alert(1)"}
            ]}},
            "stats": {"monthlyListeners": 1234567, "followers": 99, "worldRank": 0,
                      "topCities": {"items": [{"city": "London", "country": "GB", "numberOfListeners": 5000}]}},
            "visuals": {"headerImage": {"sources": [{"url": "https://h/1", "width": 100}, {"url": "https://h/2", "width": 2000}]}},
            "relatedContent": {"discoveredOnV2": {"items": [
                {"data": {"uri": "spotify:playlist:p1", "name": "Hits", "images": {"items": [{"sources": [{"url": "https://i/p"}]}]}}}
            ]}},
            "goods": {"events": {"concerts": {"items": [
                {"title": "Tour", "date": {"isoString": "2026-10-01T20:00:00Z"}, "venue": {"name": "Hall", "location": {"name": "Paris"}}}
            ]}}}
        }});
        let o = overview_from(&v);
        assert_eq!(o.monthly_listeners, Some(1_234_567));
        assert_eq!(o.world_rank, None, "rank 0 means unranked");
        assert!(o.verified);
        assert_eq!(o.external_links.len(), 1, "non-https links are dropped");
        assert_eq!(o.header_image.as_deref(), Some("https://h/2"));
        assert_eq!(o.discovered_on[0].id, "p1");
        assert_eq!(o.concerts[0].city.as_deref(), Some("Paris"));
        assert_eq!(o.top_cities[0].listeners, Some(5000));
    }
}
