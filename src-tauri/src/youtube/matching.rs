// Spotify track -> YouTube Music song matching.
//
// The governing rule is that **refusing is cheaper than guessing**. A wrong
// song is the single worst failure this feature can produce: it is silent, it
// looks like a bug in the library rather than in playback, and the user has no
// way to tell the app is wrong except by listening. Silence with a clear error
// is strictly better.
//
// So this is gate-then-score, never score-alone. Candidates must clear every
// hard gate to be *eligible*; scoring only ranks survivors. If nothing
// survives, the answer is `None` and the caller plays nothing.
//
// The subtle failure mode is not "completely different song" - duration and
// artist gates catch that easily. It is the near-miss: the instrumental, the
// karaoke backing track, the sped-up edit, the live cut, the clean edit. Those
// share title, artist, album and often duration. They are caught by treating
// title *qualifiers* as a set that must match symmetrically: if YouTube says
// "(Instrumental)" and Spotify does not, that is a different recording, full
// stop - regardless of how well everything else scores.

use std::collections::BTreeSet;

use super::search::SongResult;

/// The Spotify side of a match request.
#[derive(Debug, Clone)]
pub struct TrackQuery {
    pub title:       String,
    /// Spotify's artist list, primary first.
    pub artists:     Vec<String>,
    pub album:       Option<String>,
    pub duration_ms: u64,
    pub explicit:    bool,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub video_id:    String,
    pub score:       f64,
    /// Human-readable justification, persisted with the cache row so a bad
    /// match can be diagnosed later without re-running the search.
    pub reason:      String,
    pub duration_ms: Option<u64>,
}

/// Maximum duration difference, in milliseconds, for a candidate to be
/// eligible. Different masters of the same recording drift by a second or two
/// and YouTube rounds to whole seconds, so this cannot be zero - but a genuine
/// alternate take or edit almost always lands well outside it.
const DURATION_TOLERANCE_MS: u64 = 3_000;

/// Minimum similarity between normalised title cores.
const TITLE_CORE_THRESHOLD: f64 = 0.85;

/// Minimum similarity for a primary-artist name to count as the same artist.
const ARTIST_THRESHOLD: f64 = 0.80;

/// Minimum final score. Every candidate reaching scoring has already cleared
/// the gates, so this only rejects weak-but-legal matches.
const MIN_SCORE: f64 = 0.55;

/// Qualifiers that denote a *different recording* of the same composition.
///
/// Matched against parsed qualifier segments only - never as a bare substring
/// of the whole title. That distinction is load-bearing: "Live and Let Die"
/// would otherwise self-reject as a live recording.
const VARIANT_MARKERS: &[&str] = &[
    "instrumental", "karaoke", "backing track", "cover", "tribute", "made famous by",
    "live", "unplugged", "acoustic", "demo", "rehearsal",
    "remix", "mix", "edit", "rework", "flip", "bootleg", "vip",
    "sped up", "spedup", "slowed", "reverb", "nightcore", "8d", "lofi", "lo fi",
    "acapella", "a cappella", "clean", "censored",
    "mashup", "medley", "parody", "piano version", "workout", "cardio",
    "extended", "reprise", "interlude", "intro", "outro", "skit",
];

/// Qualifiers that are *not* meaningful differences - reissue noise that
/// Spotify and YouTube disagree about constantly for the same audio.
const IGNORABLE_MARKERS: &[&str] = &[
    "remaster", "remastered", "remasterized", "mono", "stereo",
    "album version", "original", "original version", "original mix",
    "single version", "single", "lp version", "bonus track", "deluxe",
    "anniversary", "expanded", "digital", "explicit", "explicit version",
];

/// Best match for `query` among `candidates`, or `None` if nothing qualifies.
///
/// `candidates` is expected in YouTube's own relevance order; that order is
/// used only as a final tiebreak between otherwise equal scores.
pub fn best_match(query: &TrackQuery, candidates: &[SongResult]) -> Option<Match> {
    let q_title = ParsedTitle::parse(&query.title);
    // Featured artists are frequently in Spotify's title but in YouTube's
    // artist list (or vice versa), so pool both sides before comparing.
    let q_artists: Vec<String> = query
        .artists
        .iter()
        .map(|a| normalize(a))
        .chain(q_title.featured.iter().cloned())
        .filter(|a| !a.is_empty())
        .collect();

    let mut best: Option<Match> = None;

    for (rank, cand) in candidates.iter().enumerate() {
        match score_candidate(query, &q_title, &q_artists, cand, rank) {
            Ok(m) => {
                if best.as_ref().is_none_or(|b| m.score > b.score) {
                    best = Some(m);
                }
            }
            Err(why) => {
                eprintln!("[youtube/match] reject {} ({}): {why}", cand.video_id, cand.title);
            }
        }
    }

    best.filter(|m| m.score >= MIN_SCORE)
}

/// `Ok` if the candidate clears every gate, `Err(reason)` otherwise.
fn score_candidate(
    query: &TrackQuery,
    q_title: &ParsedTitle,
    q_artists: &[String],
    cand: &SongResult,
    rank: usize,
) -> Result<Match, String> {
    // --- Gate: must be an Art Track ---------------------------------------
    // The songs filter should guarantee this, but YouTube controls that
    // behaviour and we do not. Verify rather than assume.
    if let Some(vt) = &cand.video_type {
        if !vt.contains("ATV") {
            return Err(format!("not an Art Track ({vt})"));
        }
    }

    // --- Gate: duration ---------------------------------------------------
    let cand_duration = cand
        .duration_ms
        .ok_or_else(|| "no duration on candidate".to_string())?;
    let delta = query.duration_ms.abs_diff(cand_duration);
    if delta > DURATION_TOLERANCE_MS {
        return Err(format!(
            "duration off by {delta}ms ({} vs {cand_duration})",
            query.duration_ms
        ));
    }

    // --- Gate: title core -------------------------------------------------
    let c_title = ParsedTitle::parse(&cand.title);
    let title_sim = similarity(&q_title.core, &c_title.core);
    if title_sim < TITLE_CORE_THRESHOLD {
        return Err(format!(
            "title core {title_sim:.2} < {TITLE_CORE_THRESHOLD} ({:?} vs {:?})",
            q_title.core, c_title.core
        ));
    }

    // --- Gate: variant markers must match symmetrically -------------------
    // This is the gate that catches instrumentals, karaoke, live cuts and
    // sped-up edits, which all pass duration and artist checks.
    if q_title.variants != c_title.variants {
        let only_yt: Vec<_> = c_title.variants.difference(&q_title.variants).collect();
        let only_sp: Vec<_> = q_title.variants.difference(&c_title.variants).collect();
        return Err(format!(
            "variant mismatch (youtube-only: {only_yt:?}, spotify-only: {only_sp:?})"
        ));
    }

    // --- Gate: primary artist ---------------------------------------------
    let c_artists: Vec<String> = cand
        .artists
        .iter()
        .map(|a| normalize(a))
        .chain(c_title.featured.iter().cloned())
        .filter(|a| !a.is_empty())
        .collect();

    if c_artists.is_empty() {
        return Err("candidate has no artist".into());
    }
    // Spotify's primary artist must appear somewhere on the YouTube side.
    // Direction matters: YouTube frequently lists fewer artists than Spotify,
    // but the lead artist being absent means it is someone else's recording.
    let primary = q_artists.first().ok_or_else(|| "query has no artist".to_string())?;
    let artist_hit = c_artists
        .iter()
        .any(|c| similarity(primary, c) >= ARTIST_THRESHOLD || contains_name(c, primary));
    if !artist_hit {
        return Err(format!("primary artist {primary:?} not in {c_artists:?}"));
    }

    // --- Score (ranking only; all gates already passed) --------------------
    let artist_overlap = overlap_ratio(q_artists, &c_artists);
    let album_sim = match (&query.album, &cand.album) {
        (Some(a), Some(b)) => similarity(&normalize(a), &normalize(b)),
        // Unknown album is neutral, not a penalty - YouTube omits it on singles.
        _ => 0.5,
    };
    let duration_score = 1.0 - (delta as f64 / DURATION_TOLERANCE_MS as f64);
    let explicit_score = if query.explicit == cand.explicit { 1.0 } else { 0.6 };
    // Decays across the result list so a tie resolves to YouTube's own ranking.
    let rank_score = 1.0 / (1.0 + rank as f64 * 0.15);

    let score = 0.34 * title_sim
        + 0.24 * artist_overlap
        + 0.16 * duration_score
        + 0.12 * album_sim
        + 0.08 * explicit_score
        + 0.06 * rank_score;

    Ok(Match {
        video_id: cand.video_id.clone(),
        score,
        reason: format!(
            "title {title_sim:.2} artists {artist_overlap:.2} dur+-{delta}ms \
             album {album_sim:.2} rank {rank}"
        ),
        duration_ms: cand.duration_ms,
    })
}

// ---------------------------------------------------------------------------
// title parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct ParsedTitle {
    /// Title with all qualifier segments removed, normalised.
    core:     String,
    /// Recording-variant markers found in qualifiers, as a set.
    variants: BTreeSet<String>,
    /// Normalised artist names lifted out of "feat. ..." qualifiers.
    featured: Vec<String>,
}

impl ParsedTitle {
    /// Split a title into core, variant markers and featured artists.
    ///
    /// Qualifiers are the bracketed groups and anything after a " - " separator
    /// - i.e. exactly the places where both Spotify and YouTube put version
    /// information. Scanning only these regions (rather than the raw string) is
    /// what keeps "Live and Let Die" from registering as a live recording.
    fn parse(raw: &str) -> Self {
        let mut qualifiers = Vec::new();
        let mut core = String::new();
        let mut depth = 0usize;
        let mut current = String::new();

        for ch in raw.chars() {
            match ch {
                '(' | '[' => {
                    if depth == 0 {
                        core.push_str(&current);
                        current.clear();
                    }
                    depth += 1;
                }
                ')' | ']' => {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            qualifiers.push(std::mem::take(&mut current));
                        }
                    }
                }
                _ => current.push(ch),
            }
        }
        if !current.is_empty() {
            if depth > 0 {
                // Unbalanced bracket: treat the tail as a qualifier so a
                // truncated "(Live" still registers.
                qualifiers.push(current);
            } else {
                core.push_str(&current);
            }
        }

        // Dash-separated suffix, e.g. "Song - Remastered 2011".
        if let Some(idx) = core.find(" - ") {
            let tail = core[idx + 3..].to_string();
            core.truncate(idx);
            qualifiers.push(tail);
        }

        let mut variants = BTreeSet::new();
        let mut featured = Vec::new();

        for q in &qualifiers {
            let nq = normalize(q);
            if nq.is_empty() {
                continue;
            }
            if let Some(names) = strip_featured(&nq) {
                featured.extend(names);
                continue;
            }
            // Ignorable reissue noise never counts as a variant. Checked first
            // so "Original Mix" is not caught by the "mix" marker.
            if IGNORABLE_MARKERS.iter().any(|m| nq == *m || nq.starts_with(m)) {
                continue;
            }
            for m in VARIANT_MARKERS {
                if contains_word(&nq, m) {
                    variants.insert((*m).to_string());
                }
            }
        }

        Self { core: normalize(&core), variants, featured }
    }
}

/// Returns the normalised artist names if `q` is a "feat./ft./with" qualifier.
/// `q` is already normalised, so punctuation is gone and words are space-separated.
fn strip_featured(q: &str) -> Option<Vec<String>> {
    for prefix in ["feat ", "featuring ", "ft ", "with "] {
        if let Some(rest) = q.strip_prefix(prefix) {
            return Some(
                rest.split(" and ")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            );
        }
    }
    None
}

// ---------------------------------------------------------------------------
// text helpers
// ---------------------------------------------------------------------------

/// Lowercase, strip accents, drop punctuation, collapse whitespace.
fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_space = true;
    for ch in s.chars() {
        let ch = fold_accent(ch);
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim_end().to_string()
}

/// Map the Latin-1/Latin-Extended-A letters that actually show up in artist and
/// track names onto ASCII. Full Unicode normalisation would need a dependency;
/// this covers the realistic cases.
fn fold_accent(c: char) -> char {
    match c {
        '\u{e0}'..='\u{e5}' | '\u{101}' | '\u{103}' | '\u{105}' => 'a',
        '\u{e8}'..='\u{eb}' | '\u{113}' | '\u{115}' | '\u{117}' | '\u{119}' | '\u{11b}' => 'e',
        '\u{ec}'..='\u{ef}' | '\u{129}' | '\u{12b}' | '\u{12d}' | '\u{12f}' => 'i',
        '\u{f2}'..='\u{f6}' | '\u{f8}' | '\u{14d}' | '\u{14f}' | '\u{151}' => 'o',
        '\u{f9}'..='\u{fc}' | '\u{169}' | '\u{16b}' | '\u{16d}' | '\u{16f}' | '\u{171}' => 'u',
        '\u{e7}' | '\u{107}' | '\u{109}' | '\u{10b}' | '\u{10d}' => 'c',
        '\u{f1}' | '\u{144}' | '\u{146}' | '\u{148}' => 'n',
        '\u{fd}' | '\u{ff}' => 'y',
        '\u{161}' | '\u{15b}' | '\u{15f}' => 's',
        '\u{17e}' | '\u{17a}' | '\u{17c}' => 'z',
        '\u{142}' => 'l',
        '\u{111}' => 'd',
        '\u{11f}' => 'g',
        '\u{159}' => 'r',
        '\u{165}' => 't',
        _ => c,
    }
}

/// Whole-word containment on already-normalised text.
///
/// Substring matching would fire on "mix" inside "Mixtape" and "live" inside
/// "Delivered"; multi-word markers like "sped up" still work because the
/// haystack is space-normalised.
fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(needle) {
        let start = from + pos;
        let end = start + needle.len();
        let before_ok = start == 0 || bytes[start - 1] == b' ';
        let after_ok = end == haystack.len() || bytes[end] == b' ';
        if before_ok && after_ok {
            return true;
        }
        from = start + needle.len().max(1);
        if from >= haystack.len() {
            break;
        }
    }
    false
}

/// True when one name contains the other as a whole word - handles "Tyler" vs
/// "Tyler, The Creator" style list differences between the two services.
fn contains_name(a: &str, b: &str) -> bool {
    !a.is_empty() && !b.is_empty() && (contains_word(a, b) || contains_word(b, a))
}

/// Fraction of `a`'s entries that have a counterpart in `b`.
fn overlap_ratio(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() {
        return 0.0;
    }
    let hits = a
        .iter()
        .filter(|x| b.iter().any(|y| similarity(x, y) >= ARTIST_THRESHOLD || contains_name(x, y)))
        .count();
    hits as f64 / a.len() as f64
}

/// Levenshtein-based similarity in `0.0..=1.0`.
fn similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    if a.is_ascii() && b.is_ascii() {
        let a_bytes = a.as_bytes();
        let b_bytes = b.as_bytes();
        let longest = a_bytes.len().max(b_bytes.len());
        let dist = levenshtein_slice(a_bytes, b_bytes);
        1.0 - (dist as f64 / longest as f64)
    } else {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let longest = a_chars.len().max(b_chars.len());
        let dist = levenshtein_slice(&a_chars, &b_chars);
        1.0 - (dist as f64 / longest as f64)
    }
}

/// Two-row Levenshtein over generic slices.
/// Uses a stack array for common title/artist lengths (<= 63 items) to avoid heap allocation.
fn levenshtein_slice<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    if a.len() < b.len() {
        return levenshtein_slice(b, a);
    }
    let b_len = b.len();
    if b_len == 0 {
        return a.len();
    }
    if b_len + 1 <= 64 {
        let mut previous = [0usize; 64];
        let mut current = [0usize; 64];
        let mut prev = &mut previous[..=b_len];
        let mut curr = &mut current[..=b_len];
        for j in 0..=b_len {
            prev[j] = j;
        }
        for (i, ca) in a.iter().enumerate() {
            curr[0] = i + 1;
            for (j, cb) in b.iter().enumerate() {
                let cost = usize::from(ca != cb);
                curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[b_len]
    } else {
        let mut prev: Vec<usize> = (0..=b_len).collect();
        let mut curr = vec![0usize; b_len + 1];
        for (i, ca) in a.iter().enumerate() {
            curr[0] = i + 1;
            for (j, cb) in b.iter().enumerate() {
                let cost = usize::from(ca != cb);
                curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[b_len]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn reference_levenshtein(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        let mut curr = vec![0usize; b.len() + 1];
        for (i, ca) in a.iter().enumerate() {
            curr[0] = i + 1;
            for (j, cb) in b.iter().enumerate() {
                let cost = usize::from(ca != cb);
                curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            }
            std::mem::swap(&mut prev, &mut curr);
        }
        prev[b.len()]
    }

    #[test]
    fn levenshtein_slice_matches_reference() {
        let cases = [
            ("", ""),
            ("a", ""),
            ("", "abc"),
            ("kitten", "sitting"),
            ("flaw", "lawn"),
            ("blinding lights", "blinding light"),
            ("the weeknd", "weeknd"),
            ("beyoncé", "beyonce"),
            ("longer string that tests stack buffer capacity", "longer string that tests stack buffer cap"),
        ];
        for (s1, s2) in cases {
            let ref_dist = reference_levenshtein(s1, s2);
            let s1_c: Vec<char> = s1.chars().collect();
            let s2_c: Vec<char> = s2.chars().collect();
            assert_eq!(levenshtein_slice(&s1_c, &s2_c), ref_dist, "failed on ({s1}, {s2})");
            if s1.is_ascii() && s2.is_ascii() {
                assert_eq!(levenshtein_slice(s1.as_bytes(), s2.as_bytes()), ref_dist, "failed on bytes ({s1}, {s2})");
            }
        }
    }


    fn song(title: &str, artists: &[&str], album: &str, dur_ms: u64) -> SongResult {
        SongResult {
            video_id:    format!("vid_{}", normalize(title).replace(' ', "_")),
            title:       title.into(),
            artists:     artists.iter().map(|s| (*s).into()).collect(),
            album:       Some(album.into()),
            duration_ms: Some(dur_ms),
            explicit:    false,
            video_type:  Some("MUSIC_VIDEO_TYPE_ATV".into()),
        }
    }

    fn query(title: &str, artists: &[&str], album: &str, dur_ms: u64) -> TrackQuery {
        TrackQuery {
            title:       title.into(),
            artists:     artists.iter().map(|s| (*s).into()).collect(),
            album:       Some(album.into()),
            duration_ms: dur_ms,
            explicit:    false,
        }
    }

    // -- the headline requirement -----------------------------------------

    /// The instrumental shares title, artist, album and duration. Only the
    /// qualifier distinguishes it, and it must be rejected outright.
    #[test]
    fn rejects_instrumental_of_the_right_song() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        let cands = vec![song(
            "Blinding Lights (Instrumental)",
            &["The Weeknd"],
            "Blinding Lights (Instrumental)",
            200_000,
        )];
        assert!(best_match(&q, &cands).is_none());
    }

    #[test]
    fn rejects_karaoke_live_cover_and_sped_up() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        for bad in [
            "Blinding Lights (Karaoke Version)",
            "Blinding Lights (Live)",
            "Blinding Lights (Sped Up)",
            "Blinding Lights (Acoustic)",
            "Blinding Lights - Remix",
            "Blinding Lights (Slowed + Reverb)",
        ] {
            let cands = vec![song(bad, &["The Weeknd"], "After Hours", 200_000)];
            assert!(best_match(&q, &cands).is_none(), "should have rejected {bad:?}");
        }
    }

    /// A cover by a different artist with an identical title and duration.
    #[test]
    fn rejects_other_artist() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        let cands = vec![song("Blinding Lights", &["Some Cover Band"], "Covers", 200_000)];
        assert!(best_match(&q, &cands).is_none());
    }

    #[test]
    fn rejects_duration_outlier() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        // An extended cut: right title, right artist, wrong recording.
        let cands = vec![song("Blinding Lights", &["The Weeknd"], "After Hours", 260_000)];
        assert!(best_match(&q, &cands).is_none());
    }

    #[test]
    fn rejects_non_art_track() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        let mut c = song("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        c.video_type = Some("MUSIC_VIDEO_TYPE_OMV".into());
        assert!(best_match(&q, &[c]).is_none());
    }

    #[test]
    fn empty_candidates_is_no_match_not_a_panic() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        assert!(best_match(&q, &[]).is_none());
    }

    // -- true positives ----------------------------------------------------

    #[test]
    fn accepts_exact_match() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        let cands = vec![song("Blinding Lights", &["The Weeknd"], "After Hours", 202_000)];
        let m = best_match(&q, &cands).expect("should match");
        assert_eq!(m.video_id, "vid_blinding_lights");
    }

    /// Remaster/album-version noise is not a recording difference.
    #[test]
    fn accepts_remaster_noise_on_either_side() {
        let q = query("Bohemian Rhapsody - Remastered 2011", &["Queen"], "A Night at the Opera", 354_000);
        let cands = vec![song("Bohemian Rhapsody", &["Queen"], "A Night at the Opera", 355_000)];
        assert!(best_match(&q, &cands).is_some());
    }

    /// Spotify puts the feature in the title, YouTube puts it in the artists.
    #[test]
    fn accepts_feat_in_title_vs_artist_list() {
        let q = query("Starboy (feat. Daft Punk)", &["The Weeknd"], "Starboy", 230_000);
        let cands = vec![song("Starboy", &["The Weeknd", "Daft Punk"], "Starboy", 231_000)];
        assert!(best_match(&q, &cands).is_some());
    }

    #[test]
    fn accepts_accented_artist_name() {
        let q = query("Halo", &["Beyonc\u{e9}"], "I Am... Sasha Fierce", 261_000);
        let cands = vec![song("Halo", &["Beyonce"], "I Am... Sasha Fierce", 261_000)];
        assert!(best_match(&q, &cands).is_some());
    }

    /// The real-world shape: the right track buried among near-misses.
    #[test]
    fn picks_the_right_one_from_a_mixed_result_list() {
        let q = query("Blinding Lights", &["The Weeknd"], "After Hours", 200_000);
        let cands = vec![
            song("Blinding Lights (Instrumental)", &["The Weeknd"], "Blinding Lights (Instrumental)", 200_000),
            song("Blinding Lights", &["The Weeknd"], "After Hours", 201_000),
            song("Blinding Lights (Live)", &["The Weeknd"], "After Hours", 200_000),
        ];
        let m = best_match(&q, &cands).expect("should match the studio cut");
        assert_eq!(m.video_id, "vid_blinding_lights");
    }

    // -- the tricky bit: variant words that are really part of a title -----

    #[test]
    fn title_words_that_look_like_markers_are_not_markers() {
        // "Live" here is part of the song name, not a qualifier.
        let q = query("Live and Let Die", &["Wings"], "Band on the Run", 193_000);
        let cands = vec![song("Live and Let Die", &["Wings"], "Band on the Run", 193_000)];
        assert!(best_match(&q, &cands).is_some());
    }

    #[test]
    fn substring_markers_do_not_fire() {
        assert!(!contains_word("mixtape vol 1", "mix"));
        assert!(!contains_word("delivered", "live"));
        assert!(contains_word("slowed and reverb", "reverb"));
        assert!(contains_word("sped up", "sped up"));
        assert!(contains_word("live", "live"));
    }

    // -- unit-level checks -------------------------------------------------

    #[test]
    fn parses_title_qualifiers() {
        let t = ParsedTitle::parse("Song Name (Live) [Remastered] - Single Version");
        assert_eq!(t.core, "song name");
        assert!(t.variants.contains("live"));
        // Remastered and Single Version are ignorable, not variants.
        assert!(!t.variants.contains("remastered"));
        assert!(!t.variants.contains("single"));
    }

    #[test]
    fn extracts_featured_artists() {
        let t = ParsedTitle::parse("Track (feat. Daft Punk)");
        assert_eq!(t.core, "track");
        assert_eq!(t.featured, vec!["daft punk"]);
        assert!(t.variants.is_empty());
    }

    #[test]
    fn normalizes_punctuation_and_case() {
        assert_eq!(normalize("  The Weeknd!! "), "the weeknd");
        assert_eq!(normalize("Tyler, The Creator"), "tyler the creator");
    }

    #[test]
    fn similarity_bounds() {
        assert_eq!(similarity("abc", "abc"), 1.0);
        assert_eq!(similarity("", "abc"), 0.0);
        assert!(similarity("blinding lights", "blinding light") > 0.9);
        // Non-ASCII unicode path
        assert!(similarity("beyoncé", "beyonce") > 0.8);
        // Long string exceeding stack buffer (> 64 chars) to exercise heap branch
        let long_a = "this is a very long track title that exceeds the sixty four characters stack buffer limit";
        let long_b = "this is a very long track title that exceeds the sixty four characters stack buffer limit!";
        assert!(similarity(long_a, long_b) > 0.95);
    }
}
