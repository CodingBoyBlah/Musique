//! Voice structuring: turn a provider's flat list of timed lines into *lead*
//! lines that carry their simultaneous backing vocals.
//!
//! This is a sync fix before it is a presentation feature.
//!
//! Only AMLL's TTML tells us which lines are backing vocals. Every other source
//! - including Spotify's own reference - emits a backing vocal as just another
//! timed line. Downstream, that wrecks two separate things:
//!
//! 1. **Rendering.** The player decides which line is "currently being sung" by
//!    walking row boundaries, and sets each row's end to the next row's start. A
//!    backing vocal starting 300ms after its lead becomes its own row, which
//!    truncates the lead's sung window and then steals the highlight mid-phrase.
//!    Every timestamp can be correct and it still reads as broken sync.
//!
//! 2. **Alignment.** `align.rs` pairs candidate lines against reference lines to
//!    derive the offset. If one source lists backing vocals and another doesn't,
//!    the two line grids disagree structurally - the pairing mismatches and the
//!    derived offset is pulled off by whatever the backing vocals contribute.
//!
//! Folding backing vocals onto their lead fixes both at once: the renderer gets
//! rows whose ends are defined by lead lines only, and `align.rs` automatically
//! compares lead-to-lead because the backing vocals are no longer in the list.
//!
//! The bar for folding is deliberately high. Consecutive sung lines routinely
//! overlap by 100-300ms (legato phrasing, a line beginning under the tail of the
//! last one); merging those would destroy the sequential rhythm that makes
//! lyrics readable. We only fold on evidence, never on a hunch.

use super::types::{Candidate, LineRole, LyricLine};

/// Two lines starting within this of each other are simultaneous voices - a
/// duet or a doubled line - rather than one following the other.
const SAME_INSTANT_MS: i64 = 80;

/// How much a line must overlap its predecessor's *sung* window before we'll
/// believe it's a concurrent backing vocal rather than the next lead line.
/// Legato overlap tops out around 300ms, so 400 is the first value that can't
/// be ordinary phrasing.
const MIN_OVERLAP_MS: i64 = 400;

/// A backing vocal that lands after its lead has finished is still a response
/// ("...my love" / "(my love)") if it lands promptly. Beyond this it's just the
/// next line.
const RESPONSE_GAP_MS: i64 = 700;

/// A parenthetical backing vocal is nearly always short - "(ooh)", "(yeah)",
/// "(my love)". A long parenthesised line is usually a transcription note or a
/// whole sung passage, so length is a useful guard against folding real lyrics.
const MAX_PARENTHETICAL_CHARS: usize = 40;

/// Is the whole line wrapped in parentheses? Across essentially every lyric
/// transcription convention this marks a backing/answering vocal, which makes it
/// the single strongest signal available to us without provider metadata.
fn is_parenthetical(text: &str) -> bool {
    let t = text.trim();
    if t.chars().count() > MAX_PARENTHETICAL_CHARS {
        return false;
    }
    let pairs = [('(', ')'), ('（', '）'), ('[', ']')];
    for (open, close) in pairs {
        if t.starts_with(open) && t.ends_with(close) && t.chars().count() > 2 {
            // reject "(a) b (c)" - only a single enclosing pair counts
            let inner: String = t.chars().skip(1).take(t.chars().count() - 2).collect();
            if !inner.contains(open) && !inner.contains(close) {
                return true;
            }
        }
    }
    false
}

/// When the line stops being sung. Word-level sources know exactly; line-level
/// ones don't, and guessing would manufacture the very overlap we're testing
/// for - so they report `None` and only the parenthetical rule can apply.
fn sung_end(line: &LyricLine) -> Option<i64> {
    line.end_ms()
}

/// Should `next` be folded onto `lead` as a concurrent backing vocal?
fn is_backing(lead: &LyricLine, next: &LyricLine) -> bool {
    // a provider that already told us wins outright
    if next.role == LineRole::Bg {
        return true;
    }
    // never fold a second voice onto a lead that already has one - the type
    // holds a single bg, and a third simultaneous voice is vanishingly rare
    if lead.bg.is_some() {
        return false;
    }

    let gap = next.time_ms - lead.time_ms;

    // simultaneous starts: a doubled or answering voice
    if gap.abs() <= SAME_INSTANT_MS {
        return true;
    }
    // a line can't back one that hasn't started
    if gap < 0 {
        return false;
    }

    match sung_end(lead) {
        // word-level: we know when the lead ends, so we can measure real overlap
        Some(end) => {
            let overlaps_well = next.time_ms + MIN_OVERLAP_MS < end;
            if overlaps_well {
                return true;
            }
            // a prompt parenthetical answer just after the lead still belongs to it
            is_parenthetical(&next.text) && next.time_ms <= end + RESPONSE_GAP_MS
        }
        // line-level: no honest end to measure against, so parentheses are the
        // only evidence we'll accept, and only while the answer is prompt
        None => is_parenthetical(&next.text) && gap <= RESPONSE_GAP_MS,
    }
}

/// Fold backing vocals onto their lead lines.
///
/// Runs on every candidate from every provider, so that by the time a candidate
/// reaches `align.rs` or the frontend, `lines` holds lead lines only and each
/// carries its simultaneous voice in `bg`.
pub fn structure(cand: &mut Candidate) {
    if cand.lines.len() < 2 {
        return;
    }

    // a source that already nests its backing vocals (AMLL TTML) is authoritative
    // and must not be second-guessed by heuristics
    if cand.lines.iter().any(|l| l.bg.is_some()) {
        return;
    }

    let lines = std::mem::take(&mut cand.lines);
    let mut out: Vec<LyricLine> = Vec::with_capacity(lines.len());

    for line in lines {
        let Some(lead) = out.last_mut() else {
            out.push(line);
            continue;
        };

        if is_backing(lead, &line) {
            let mut bg = line;
            bg.role = LineRole::Bg;
            lead.bg = Some(Box::new(bg));
        } else {
            out.push(line);
        }
    }

    cand.lines = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lyrics::types::LyricWord;

    fn worded(t: i64, text: &str, dur: i64) -> LyricLine {
        LyricLine::worded(
            t,
            Some(text.to_string()),
            vec![LyricWord { time_ms: t, end_ms: t + dur, text: text.to_string() }],
        )
    }

    fn cand(lines: Vec<LyricLine>) -> Candidate {
        Candidate::new("test", lines)
    }

    #[test]
    fn folds_a_parenthetical_answer() {
        let mut c = cand(vec![
            worded(0, "I need you", 1_500),
            worded(1_700, "(need you)", 500), // lands just after, parenthesised
            worded(4_000, "every single day", 1_500),
        ]);
        structure(&mut c);

        assert_eq!(c.lines.len(), 2, "the answering vocal must not be its own line");
        assert_eq!(c.lines[0].text, "I need you");
        assert_eq!(c.lines[0].bg.as_ref().unwrap().text, "(need you)");
        assert_eq!(c.lines[0].bg.as_ref().unwrap().role, LineRole::Bg);
        // the bg keeps its OWN timing - folding changes row membership, not sync
        assert_eq!(c.lines[0].bg.as_ref().unwrap().time_ms, 1_700);
        assert_eq!(c.lines[1].text, "every single day");
    }

    #[test]
    fn folds_a_deeply_overlapping_line() {
        let mut c = cand(vec![
            worded(0, "hold me closer", 4_000), // sung until 4000
            worded(1_000, "ooh ooh", 800),      // starts 3s before the lead ends
        ]);
        structure(&mut c);
        assert_eq!(c.lines.len(), 1);
        assert!(c.lines[0].bg.is_some());
    }

    #[test]
    fn legato_overlap_does_not_merge() {
        // this is the case that must NOT fold: consecutive lead lines whose
        // phrasing overlaps by ~250ms. merging them destroys the reading rhythm.
        let mut c = cand(vec![
            worded(0, "the first line here", 2_000),
            worded(1_800, "the second line here", 2_000),
            worded(3_700, "the third line here", 2_000),
        ]);
        structure(&mut c);
        assert_eq!(c.lines.len(), 3, "legato phrasing is not a backing vocal");
        assert!(c.lines.iter().all(|l| l.bg.is_none()));
    }

    #[test]
    fn simultaneous_starts_are_concurrent_voices() {
        let mut c = cand(vec![worded(5_000, "we go together", 2_000), worded(5_040, "together", 1_000)]);
        structure(&mut c);
        assert_eq!(c.lines.len(), 1);
        assert_eq!(c.lines[0].bg.as_ref().unwrap().text, "together");
    }

    #[test]
    fn line_level_sources_only_fold_on_parentheses() {
        // no word timings -> no honest sung end -> a plain following line is
        // just the next line, however close it lands
        let mut plain = cand(vec![LyricLine::line(0, "first"), LyricLine::line(300, "second")]);
        structure(&mut plain);
        assert_eq!(plain.lines.len(), 2);

        let mut parens = cand(vec![LyricLine::line(0, "first"), LyricLine::line(300, "(oooh)")]);
        structure(&mut parens);
        assert_eq!(parens.lines.len(), 1);
        assert!(parens.lines[0].bg.is_some());
    }

    #[test]
    fn a_source_that_already_nests_is_left_alone() {
        let mut lead = worded(0, "lead", 1_000);
        lead.bg = Some(Box::new(worded(100, "(bg)", 400)));
        let mut c = cand(vec![lead, worded(2_000, "next", 1_000)]);
        structure(&mut c);
        assert_eq!(c.lines.len(), 2, "AMLL's own structure is authoritative");
    }

    #[test]
    fn long_parenthetical_is_treated_as_a_real_line() {
        let mut c = cand(vec![
            LyricLine::line(0, "first"),
            LyricLine::line(300, "(this is a whole sung passage in brackets, not a backing vocal)"),
        ]);
        structure(&mut c);
        assert_eq!(c.lines.len(), 2);
    }

    #[test]
    fn only_one_backing_vocal_per_lead() {
        let mut c = cand(vec![
            worded(0, "lead line", 5_000),
            worded(1_000, "(one)", 300),
            worded(2_000, "(two)", 300),
        ]);
        structure(&mut c);
        // the second parenthetical stays a line of its own rather than
        // overwriting the first
        assert_eq!(c.lines.len(), 2);
        assert_eq!(c.lines[0].bg.as_ref().unwrap().text, "(one)");
    }
}
