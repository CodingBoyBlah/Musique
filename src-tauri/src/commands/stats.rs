//! your listening, computed locally from the taste engine's `listen_events`.
//! nothing leaves the machine and nothing here needs a network - it's the
//! offline, spotify-independent half of the stats (a wrapped you can open any
//! day of the year).

use std::collections::{BTreeSet, HashMap};

use serde::Serialize;
use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};

use crate::{
    errors::AppError,
    spotify::types::{AlbumItem, ArtistItem, TrackItem},
    state::AppState,
};

#[derive(Debug, Clone, Serialize)]
pub struct Ranked<T> {
    pub item:  T,
    pub plays: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenreShare {
    pub genre: String,
    /// 0..1 of all genre-weighted plays in the range
    pub share: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ListeningStats {
    pub range:            String,
    pub total_ms:         i64,
    pub plays:            i64,
    pub distinct_tracks:  i64,
    pub distinct_artists: i64,
    pub current_streak:   i64,
    pub longest_streak:   i64,
    /// plays by [weekday 0=sun][hour], local time
    pub heatmap:          Vec<Vec<i64>>,
    pub top_tracks:       Vec<Ranked<TrackItem>>,
    pub top_artists:      Vec<Ranked<ArtistItem>>,
    pub top_albums:       Vec<Ranked<AlbumItem>>,
    pub top_genres:       Vec<GenreShare>,
    /// the most-played in the last 30 days, regardless of range
    pub on_repeat:        Vec<Ranked<TrackItem>>,
    /// finished often once, not played in two months
    pub forgotten:        Vec<Ranked<TrackItem>>,
    /// first-ever plays that happened inside the range, most played first
    pub discoveries:      Vec<Ranked<TrackItem>>,
    /// local date (yyyy-mm-dd) + ms of the biggest listening day in range
    pub biggest_day:      Option<(String, i64)>,
}

const DAY_MS: i64 = 86_400_000;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// range start in ms. "year" is the calendar year so far (wrapped-style)
async fn range_start(pool: &SqlitePool, range: &str) -> Result<i64, AppError> {
    Ok(match range {
        "week" => now_ms() - 7 * DAY_MS,
        "month" => now_ms() - 30 * DAY_MS,
        "year" => {
            let (s,): (i64,) = sqlx::query_as("SELECT CAST(strftime('%s', date('now', 'localtime', 'start of year'), 'utc') AS INTEGER) * 1000")
                .fetch_one(pool)
                .await?;
            s
        }
        _ => 0,
    })
}

/// current and longest runs of consecutive local days with a play.
/// `days` are day numbers (days since epoch), `today` likewise
pub(crate) fn streaks(days: &BTreeSet<i64>, today: i64) -> (i64, i64) {
    let mut longest = 0;
    let mut run = 0;
    let mut prev: Option<i64> = None;
    for &d in days {
        run = if prev == Some(d - 1) { run + 1 } else { 1 };
        longest = longest.max(run);
        prev = Some(d);
    }
    // the current streak survives until the end of today even with no play yet
    let mut current = 0;
    let mut d = if days.contains(&today) { today } else { today - 1 };
    while days.contains(&d) {
        current += 1;
        d -= 1;
    }
    (current, longest)
}

/// weight each artist's plays across their genres, return the top shares
pub(crate) fn genre_shares(artist_plays: &[(i64, Option<String>)], top: usize) -> Vec<GenreShare> {
    let mut weights: HashMap<String, f64> = HashMap::new();
    let mut total = 0.0;
    for (plays, genres) in artist_plays {
        let Some(g) = genres else { continue };
        let list: Vec<String> = serde_json::from_str(g).unwrap_or_default();
        if list.is_empty() {
            continue;
        }
        let w = *plays as f64 / list.len() as f64;
        for genre in list {
            *weights.entry(genre).or_default() += w;
            total += w;
        }
    }
    if total <= 0.0 {
        return Vec::new();
    }
    let mut out: Vec<GenreShare> = weights
        .into_iter()
        .map(|(genre, w)| GenreShare { genre, share: w / total })
        .collect();
    out.sort_by(|a, b| b.share.partial_cmp(&a.share).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(top);
    out
}

/// catalog rows for these track ids, straight from the local db. anything the
/// db doesn't know is filled in from spotify's metadata when online
async fn load_tracks(app: &AppHandle, pool: &SqlitePool, ids: &[String]) -> HashMap<String, TrackItem> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return out;
    }
    let ph = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT t.id, t.name, t.duration_ms, t.explicit, t.album_id, al.name, al.album_type, al.image_url, al.release_date
         FROM tracks t LEFT JOIN albums al ON al.id = t.album_id WHERE t.id IN ({ph})"
    );
    let mut q = sqlx::query_as::<_, (String, String, i64, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(&sql);
    for id in ids {
        q = q.bind(id);
    }
    for (id, name, dur, explicit, album_id, album_name, album_type, image, date) in q.fetch_all(pool).await.unwrap_or_default() {
        out.insert(
            id.clone(),
            TrackItem {
                id,
                name,
                duration_ms: dur,
                explicit,
                artists: Vec::new(),
                album: album_id.map(|aid| AlbumItem {
                    id: aid,
                    name: album_name.unwrap_or_default(),
                    album_type: album_type.unwrap_or_else(|| "album".into()),
                    image_url: image,
                    release_date: date,
                    artists: Vec::new(),
                    popularity: None,
                }),
                popularity: None,
            },
        );
    }
    let sql = format!(
        "SELECT ta.track_id, a.id, a.name FROM track_artists ta JOIN artists a ON a.id = ta.artist_id
         WHERE ta.track_id IN ({ph}) ORDER BY ta.track_id, ta.position"
    );
    let mut q = sqlx::query_as::<_, (String, String, String)>(&sql);
    for id in ids {
        q = q.bind(id);
    }
    for (tid, aid, aname) in q.fetch_all(pool).await.unwrap_or_default() {
        if let Some(t) = out.get_mut(&tid) {
            t.artists.push(ArtistItem { id: aid, name: aname, image_url: None, popularity: None });
        }
    }
    let missing: Vec<String> = ids.iter().filter(|i| !out.contains_key(*i)).cloned().collect();
    if !missing.is_empty() {
        if let Ok(fetched) = crate::internal::metadata::tracks(app, &missing).await {
            for t in fetched {
                out.insert(t.id.clone(), t);
            }
        }
    }
    out
}

async fn ranked_tracks(app: &AppHandle, pool: &SqlitePool, rows: Vec<(String, i64)>) -> Vec<Ranked<TrackItem>> {
    let ids: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    let map = load_tracks(app, pool, &ids).await;
    rows.into_iter()
        .filter_map(|(id, plays)| map.get(&id).cloned().map(|item| Ranked { item, plays }))
        .collect()
}

#[tauri::command]
pub async fn get_listening_stats(app: AppHandle, range: Option<String>) -> Result<ListeningStats, AppError> {
    let pool = app.state::<AppState>().db.clone();
    let range = match range.as_deref() {
        Some(r @ ("week" | "month" | "year" | "all")) => r.to_string(),
        _ => "month".to_string(),
    };
    let since = range_start(&pool, &range).await?;
    // podcast episodes never reach listen_events (the frontend skips them), so
    // everything below is music

    let (total_ms,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(ms_played), 0) FROM listen_events
         WHERE event_type IN ('complete', 'skip') AND occurred_at >= ?",
    )
    .bind(since)
    .fetch_one(&pool)
    .await?;
    let (plays, distinct_tracks): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COUNT(DISTINCT track_id) FROM listen_events WHERE event_type = 'play' AND occurred_at >= ?",
    )
    .bind(since)
    .fetch_one(&pool)
    .await?;

    let mut heatmap = vec![vec![0i64; 24]; 7];
    let cells: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT CAST(strftime('%w', occurred_at / 1000, 'unixepoch', 'localtime') AS INTEGER),
                CAST(strftime('%H', occurred_at / 1000, 'unixepoch', 'localtime') AS INTEGER),
                COUNT(*)
         FROM listen_events WHERE event_type = 'play' AND occurred_at >= ?
         GROUP BY 1, 2",
    )
    .bind(since)
    .fetch_all(&pool)
    .await?;
    for (wd, h, n) in cells {
        if (0..7).contains(&wd) && (0..24).contains(&h) {
            heatmap[wd as usize][h as usize] = n;
        }
    }

    // streaks look at all history, not just the range
    let day_rows: Vec<(i64,)> = sqlx::query_as(
        "SELECT DISTINCT CAST(julianday(date(occurred_at / 1000, 'unixepoch', 'localtime')) - 2440587.5 AS INTEGER)
         FROM listen_events WHERE event_type = 'play'",
    )
    .fetch_all(&pool)
    .await?;
    let (today,): (i64,) = sqlx::query_as("SELECT CAST(julianday(date('now', 'localtime')) - 2440587.5 AS INTEGER)")
        .fetch_one(&pool)
        .await?;
    let days: BTreeSet<i64> = day_rows.into_iter().map(|r| r.0).collect();
    let (current_streak, longest_streak) = streaks(&days, today);

    let biggest_day: Option<(String, i64)> = sqlx::query_as(
        "SELECT date(occurred_at / 1000, 'unixepoch', 'localtime') AS d, SUM(ms_played) AS ms
         FROM listen_events WHERE event_type IN ('complete', 'skip') AND occurred_at >= ?
         GROUP BY d ORDER BY ms DESC LIMIT 1",
    )
    .bind(since)
    .fetch_optional(&pool)
    .await?
    .filter(|(_, ms): &(String, i64)| *ms > 0);

    let top_track_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT track_id, COUNT(*) AS n FROM listen_events WHERE event_type = 'play' AND occurred_at >= ?
         GROUP BY track_id ORDER BY n DESC, MAX(occurred_at) DESC LIMIT 20",
    )
    .bind(since)
    .fetch_all(&pool)
    .await?;
    let top_tracks = ranked_tracks(&app, &pool, top_track_rows).await;

    let artist_rows: Vec<(String, String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT a.id, a.name, a.image_url, a.genres, COUNT(*) AS n
         FROM listen_events e JOIN track_artists ta ON ta.track_id = e.track_id JOIN artists a ON a.id = ta.artist_id
         WHERE e.event_type = 'play' AND e.occurred_at >= ?
         GROUP BY a.id ORDER BY n DESC",
    )
    .bind(since)
    .fetch_all(&pool)
    .await?;
    let distinct_artists = artist_rows.len() as i64;
    let top_genres = genre_shares(
        &artist_rows.iter().map(|r| (r.4, r.3.clone())).collect::<Vec<_>>(),
        8,
    );
    let top_artists = artist_rows
        .into_iter()
        .take(20)
        .map(|(id, name, image_url, _, plays)| Ranked { item: ArtistItem { id, name, image_url, popularity: None }, plays })
        .collect();

    let album_rows: Vec<(String, String, Option<String>, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT al.id, al.name, al.album_type, al.image_url, al.release_date, COUNT(*) AS n
         FROM listen_events e JOIN tracks t ON t.id = e.track_id JOIN albums al ON al.id = t.album_id
         WHERE e.event_type = 'play' AND e.occurred_at >= ?
         GROUP BY al.id ORDER BY n DESC LIMIT 20",
    )
    .bind(since)
    .fetch_all(&pool)
    .await?;
    let top_albums = album_rows
        .into_iter()
        .map(|(id, name, album_type, image_url, release_date, plays)| Ranked {
            item: AlbumItem {
                id,
                name,
                album_type: album_type.unwrap_or_else(|| "album".into()),
                image_url,
                release_date,
                artists: Vec::new(),
                popularity: None,
            },
            plays,
        })
        .collect();

    let repeat_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT track_id, COUNT(*) AS n FROM listen_events WHERE event_type = 'play' AND occurred_at >= ?
         GROUP BY track_id HAVING n >= 3 ORDER BY n DESC LIMIT 20",
    )
    .bind(now_ms() - 30 * DAY_MS)
    .fetch_all(&pool)
    .await?;
    let on_repeat = ranked_tracks(&app, &pool, repeat_rows).await;

    let forgotten_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT track_id, complete_count FROM track_stats
         WHERE complete_count >= 3 AND last_played_at IS NOT NULL AND last_played_at < ?
         ORDER BY complete_count DESC, last_played_at DESC LIMIT 20",
    )
    .bind(now_ms() - 60 * DAY_MS)
    .fetch_all(&pool)
    .await?;
    let forgotten = ranked_tracks(&app, &pool, forgotten_rows).await;

    let discovery_rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT track_id, COUNT(*) AS n FROM listen_events WHERE event_type = 'play'
         GROUP BY track_id HAVING MIN(occurred_at) >= ? AND n >= 2
         ORDER BY n DESC LIMIT 20",
    )
    .bind(since.max(1))
    .fetch_all(&pool)
    .await?;
    let discoveries = if range == "all" { Vec::new() } else { ranked_tracks(&app, &pool, discovery_rows).await };

    Ok(ListeningStats {
        range,
        total_ms,
        plays,
        distinct_tracks,
        distinct_artists,
        current_streak,
        longest_streak,
        heatmap,
        top_tracks,
        top_artists,
        top_albums,
        top_genres,
        on_repeat,
        forgotten,
        discoveries,
        biggest_day,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_math() {
        let days: BTreeSet<i64> = [1, 2, 3, 7, 8, 10, 11].into_iter().collect();
        assert_eq!(streaks(&days, 11), (2, 3));
        // nothing yet today: yesterday's run still counts
        assert_eq!(streaks(&days, 12), (2, 3));
        // a gap day breaks it
        assert_eq!(streaks(&days, 13), (0, 3));
        assert_eq!(streaks(&BTreeSet::new(), 5), (0, 0));
    }

    #[test]
    fn genre_weighting() {
        let rows = vec![
            (6, Some(r#"["indie rock", "shoegaze"]"#.to_string())),
            (2, Some(r#"["indie rock"]"#.to_string())),
            (9, None),
        ];
        let g = genre_shares(&rows, 5);
        assert_eq!(g[0].genre, "indie rock");
        assert!((g[0].share - 5.0 / 8.0).abs() < 1e-9);
        assert!((g[1].share - 3.0 / 8.0).abs() < 1e-9);
    }
}

