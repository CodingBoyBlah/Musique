//! shared lyric types. these cross the ipc boundary, so every field here is
//! part of the frontend contract (`src/api/lyrics.ts`) - keep them in sync.

use serde::{Deserialize, Serialize};

/// which voice a line belongs to. lets the ui style backing vocals / duet parts
/// differently instead of flattening everything into one column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LineRole {
    #[default]
    Main,
    /// backing vocal sung underneath the lead line
    Bg,
    /// second voice in a duet (ttml agent 2)
    Duet,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LyricWord {
    pub time_ms: i64, // start, ms from track start
    pub end_ms:  i64, // end
    pub text:    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub time_ms: i64,
    pub text:    String,
    /// real per-word timings when the source has them (yrc / richsync / ttml /
    /// qrc / krc). empty for line-level sources, so the ui has no word data then
    #[serde(default)]
    pub words: Vec<LyricWord>,
    /// provider-supplied translation of this line, when the source ships one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
    /// provider-supplied romanization (pinyin / romaji), when the source ships one
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roman: Option<String>,
    #[serde(default)]
    pub role: LineRole,
    /// a concurrent backing-vocal line attached to this one (ttml bg spans).
    /// boxed so `LyricLine` stays a fixed size
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<Box<LyricLine>>,
}

impl LyricLine {
    /// plain line-level entry, no word timings
    pub fn line(time_ms: i64, text: impl Into<String>) -> Self {
        Self {
            time_ms,
            text: text.into(),
            words: Vec::new(),
            translation: None,
            roman: None,
            role: LineRole::Main,
            bg: None,
        }
    }

    /// word-timed entry. `text` is derived from the words when not given
    pub fn worded(time_ms: i64, text: Option<String>, words: Vec<LyricWord>) -> Self {
        let text = text.filter(|t| !t.trim().is_empty()).unwrap_or_else(|| {
            words.iter().map(|w| w.text.as_str()).collect::<String>().trim().to_string()
        });
        Self { time_ms, text, words, translation: None, roman: None, role: LineRole::Main, bg: None }
    }

    /// absolute ms this line stops being sung, when the source knows
    pub fn end_ms(&self) -> Option<i64> {
        self.words.iter().map(|w| w.end_ms).max()
    }

    pub fn is_word_level(&self) -> bool {
        !self.words.is_empty()
    }
}

/// what the frontend finally renders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyrics {
    pub track_id:     String,
    pub lines:        Vec<LyricLine>, // time-synced lines, empty if there are none
    pub plain:        Option<String>, // unsynced fallback text
    pub synced:       bool,
    pub word_level:   bool, // the lines carry real word timings
    pub instrumental: bool,
    pub source:       String, // spotify | musixmatch | netease | amll | qq | kugou | lrclib | none
    pub found:        bool,
    /// ms the chosen candidate was shifted by to line up with the sync reference
    #[serde(default)]
    pub offset_ms: i64,
    /// every other source we already hold for this track, for the ui's source
    /// switcher. offline-instant because they're cached parsed
    #[serde(default)]
    pub alternates: Vec<Alternate>,
    /// true while word-level candidates are still being raced in the background,
    /// so the ui knows a silent upgrade may still land
    #[serde(default)]
    pub upgrading: bool,
    #[serde(default)]
    pub has_translation: bool,
    #[serde(default)]
    pub has_roman: bool,
}

impl Lyrics {
    /// the empty answer - nothing found anywhere
    pub fn none(track_id: &str) -> Self {
        Self {
            track_id:     track_id.to_string(),
            lines:        Vec::new(),
            plain:        None,
            synced:       false,
            word_level:   false,
            instrumental: false,
            source:       "none".to_string(),
            found:        false,
            offset_ms:    0,
            alternates:   Vec::new(),
            upgrading:    false,
            has_translation: false,
            has_roman:    false,
        }
    }
}

/// a cached alternate rendering of the same track from a different provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternate {
    pub source:     String,
    pub word_level: bool,
    pub synced:     bool,
    pub lines:      usize,
}

/// everything a provider needs to identify the track it's being asked about.
/// `isrc` is the strongest key when present - it pins the exact recording, which
/// is the whole fix for version mismatch
#[derive(Debug, Clone, Default)]
pub struct TrackRef {
    pub id:          String, // spotify base62 track id
    pub name:        String,
    pub artist:      String,
    pub album:       String,
    pub isrc:        Option<String>,
    pub duration_ms: i64,
}

impl TrackRef {
    pub fn duration_sec(&self) -> i64 {
        (self.duration_ms as f64 / 1000.0).round() as i64
    }

    /// search-friendly title: strips the version suffixes that make providers
    /// return a different master ("- Remastered 2011", "(feat. X)", ...)
    pub fn clean_name(&self) -> String {
        clean_title(&self.name)
    }
}

/// drop `- Remastered` / `(feat. ...)` / `- Live` style suffixes. providers
/// search better on the bare title and we verify the match by duration anyway
pub fn clean_title(raw: &str) -> String {
    let mut s = raw.to_string();

    // " - <qualifier>" tail
    if let Some(idx) = s.find(" - ") {
        let tail = s[idx + 3..].to_ascii_lowercase();
        const TAILS: [&str; 10] = [
            "remaster", "remastered", "live", "radio edit", "single version",
            "album version", "mono", "stereo", "deluxe", "bonus track",
        ];
        if TAILS.iter().any(|t| tail.contains(t)) {
            s.truncate(idx);
        }
    }

    // "(feat. ...)" / "[feat. ...]" anywhere
    loop {
        let Some(open) = s.find(['(', '[']) else { break };
        let close_ch = if s.as_bytes()[open] == b'(' { ')' } else { ']' };
        let Some(rel) = s[open..].find(close_ch) else { break };
        let inner = s[open + 1..open + rel].to_ascii_lowercase();
        if inner.starts_with("feat") || inner.starts_with("ft.") || inner.starts_with("with ") {
            s.replace_range(open..open + rel + 1, "");
        } else {
            break;
        }
    }

    s.trim().trim_end_matches('-').trim().to_string()
}

/// one provider's answer, already parsed into absolute-ms lines
#[derive(Debug, Clone)]
pub struct Candidate {
    pub source:       &'static str,
    pub lines:        Vec<LyricLine>,
    pub plain:        Option<String>,
    pub instrumental: bool,
    /// Was this matched to the EXACT recording (by ISRC or Spotify track id), or
    /// found by fuzzy title+artist+duration search?
    ///
    /// It decides whether we're allowed to move its timestamps. A fuzzy match got
    /// its absolute timing from whatever master the search returned, so that
    /// timing is meaningless on its own and has to be shifted onto the reference.
    /// An exact match is already on this recording's clock, and shifting it would
    /// only import the reference's own error.
    pub exact: bool,
}

impl Candidate {
    pub fn new(source: &'static str, lines: Vec<LyricLine>) -> Self {
        Self { source, lines, plain: None, instrumental: false, exact: false }
    }

    pub fn word_level(&self) -> bool {
        self.lines.iter().any(|l| l.is_word_level())
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.plain.is_none() && !self.instrumental
    }

    /// line onsets, used by the alignment check
    pub fn onsets(&self) -> Vec<i64> {
        self.lines.iter().map(|l| l.time_ms).collect()
    }

    /// shift every timestamp by `offset_ms` (applied after alignment)
    pub fn shift(&mut self, offset_ms: i64) {
        if offset_ms == 0 {
            return;
        }
        for l in &mut self.lines {
            l.time_ms += offset_ms;
            for w in &mut l.words {
                w.time_ms += offset_ms;
                w.end_ms += offset_ms;
            }
            if let Some(bg) = l.bg.as_mut() {
                bg.time_ms += offset_ms;
                for w in &mut bg.words {
                    w.time_ms += offset_ms;
                    w.end_ms += offset_ms;
                }
            }
        }
    }
}
