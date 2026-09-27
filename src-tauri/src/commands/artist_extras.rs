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
