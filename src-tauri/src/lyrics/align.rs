//! Apple Music owns lyric content. Other providers donate matching timings.
//! When Apple is absent, fallbacks still use recording alignment and consensus.

use super::types::{is_apple_source, Candidate};

// tuning constants. the reasoning matters more than the numbers, so:
//
// PAIR_WINDOW_MS - how far from its expected position a line may sit and still
//   be considered "the same line". deliberately generous: we apply it AFTER a
//   coarse global shift, so it only has to absorb per-line drift, not the offset
//   itself. too tight and a genuinely-aligned document with a slow drift pairs
//   nothing and gets rejected; too loose and lines start pairing with their
//   neighbours. 4s is roughly one sung line, which is the natural spacing floor.
const PAIR_WINDOW_MS: i64 = 4_000;

// MAX_RESIDUAL_MS - the MAD ceiling. a listener notices a lyric line landing
//   late at around 300-400ms and finds it unusable past ~1s. 700ms sits just
//   inside "annoying but recognisably this song", which is the right place to
//   draw the trust line: anything looser is not a latency offset we can correct,
//   it's a different arrangement pretending to be one.
const MAX_RESIDUAL_MS: i64 = 700;

// MAX_OFFSET_MS - beyond this an "offset" stops being plausible. real offsets
//   come from an intro/count-in difference or a provider timing from a video
//   with a logo sting: a few seconds at most. 15s is more than any of those and
//   less than the gap you'd get from a track with a different intro length, so
//   it separates "correctable latency" from "different recording" cleanly.
const MAX_OFFSET_MS: i64 = 15_000;

// MIN_MATCHED / MIN_MATCH_RATIO - evidence floor. a median over 3 points is
//   noise, so demand at least 4 pairs outright, and at least 60% of the
//   reference's lines so a candidate can't "align" on a chorus while getting
//   every verse wrong. 60% (not 90%) because providers legitimately disagree on
//   credit lines, repeated choruses and interlude markers.
const MIN_MATCHED: usize = 4;
const MIN_MATCH_RATIO: f64 = 0.60;

// MAX_COUNT_SKEW - an arrangement check that runs before any arithmetic. if one
//   document has 40% more lines than the other it is not the same cut of the
//   song, and pairing it would just find whichever 60% happens to line up.
const MAX_COUNT_SKEW: f64 = 0.40;

// MAX_SPAN_SKEW - the second arrangement check, and the one that catches what
//   the MAD cannot. a master with different PACING drifts linearly away from the
//   reference, but when both documents have a dense line grid every candidate
//   line still lands near SOME reference line, so mispairing folds that drift
//   into a small sawtooth that measures as tight. the giveaway is the span: the
//   same recording's lyrics cover the same stretch of time, whatever either side
//   thinks about credit lines. 20% leaves room for a document that skips a long
//   outro or adds a trailing credit, and still rejects a differently-paced cut.
//   only applied to documents long enough for the span to mean anything.
const MAX_SPAN_SKEW: f64 = 0.20;
const SPAN_FLOOR_MS: i64 = 20_000;

// how many leading characters two normalised lines must share to corroborate a
// time pairing, as a fraction of the shorter line - 8 shared characters is
// strong evidence on a 10-character line and weak evidence on a 40-character
// one. clamped so a very short line still needs 4 and a very long one is never
// asked for more than 12 (providers disagree about the tails of long lines).
const TEXT_PREFIX_MIN: usize = 4;
const TEXT_PREFIX_MAX: usize = 12;

// what a text match is worth, expressed as proximity. it outranks 2s of timing
// distance, so a line pairs with its real counterpart rather than a neighbour
// that happens to sit closer - but it never overrides the window outright,
// because two providers do genuinely transcribe the same line differently.
const TEXT_BONUS_MS: i64 = 2_000;

// residuals within the same bucket are "equally good" - a 40ms and a 180ms MAD
// are both inaudible, so source trust should decide between them rather than
// meaningless decimal places.
const RESIDUAL_BUCKET_MS: i64 = 250;

/// the verdict on whether a candidate can be trusted for this exact recording.
///
/// `offset_ms` measures candidate minus reference. For Apple it summarizes
/// per-line corrections; for fallbacks it describes one global shift.
/// [`choose`] already applies all corrections to the returned candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub offset_ms:   i64,
    pub residual_ms: i64,   // median absolute deviation of the per-line deltas
    pub matched:     usize, // how many lines we could pair up at all
    pub aligned:     bool,  // the only field callers should branch on
}

impl Alignment {
    /// nothing could be measured, or what we measured failed the gates
    pub fn unknown() -> Self {
        Self { offset_ms: 0, residual_ms: i64::MAX, matched: 0, aligned: false }
    }

    /// ms to ADD to the candidate's timestamps to put it on the reference
    pub fn applied_shift(&self) -> i64 {
        -self.offset_ms
    }
}

/// source trust order. amll is hand-corrected to ~±100ms, spotify is keyed to
/// the exact track id, musixmatch/netease are large and mostly machine-timed,
/// lrclib is community-timed and by far the loosest. only ever used to break a
/// tie between candidates that already passed the same correctness gates.
pub fn source_rank(source: &str) -> usize {
    match source {
        // apple's own word-level TTML, keyed by ISRC - exact recording, real
        // syllable timings, background vocals and duet voices all marked up
        "applemusic"   => 0,
        // the same format, hand-corrected, keyed by spotify track id
        "amll"         => 2,
        // the same format again, but found by fuzzy search
        "betterlyrics" => 1,
        "spotify"      => 3,
        "musixmatch"   => 4,
        "netease"      => 5,
        "qq"           => 6,
        "kugou"        => 7,
        "lrclib"       => 8,
        _              => 9,
    }
}

/// check a candidate against the spotify sync reference.
///
/// the match floor is measured against the REFERENCE's line count, because the
/// reference is the one document we know describes this recording.
pub fn align_to_reference(cand: &Candidate, reference: &Candidate) -> Alignment {
    measure(cand, reference, reference.lines.len(), reference.lines.len())
}

/// when there is no reference: do two independent candidates agree with each
/// other? same arithmetic, but neither side is privileged, so the floors are
/// measured against the shorter document and the arrangement check against the
/// longer one.
pub fn cross_agrees(a: &Candidate, b: &Candidate) -> Alignment {
    let lo = a.lines.len().min(b.lines.len());
    let hi = a.lines.len().max(b.lines.len());
    measure(a, b, lo, hi)
}

/// pick the winner from a set of candidates given an optional reference.
///
/// The returned candidate already includes timing corrections. Store the
/// representative applied shift in `Lyrics::offset_ms`; do not apply it again.
pub fn choose(reference: Option<&Candidate>, candidates: Vec<Candidate>) -> Option<(Candidate, Alignment)> {
    let mut pool: Vec<Candidate> = candidates.into_iter().filter(|c| !c.is_empty()).collect();

    // Timing disagreement must never turn an available Apple document into
    // Spotify text. Match lines individually, retaining Apple's words/voices.
    if let Some(i) = (0..pool.len()).filter(|&i| is_apple_source(pool[i].source))
        .min_by_key(|&i| source_rank(pool[i].source)) {
        let mut apple = pool.remove(i);
        let mut donors: Vec<&Candidate> = reference.into_iter().chain(pool.iter()).collect();
        donors.sort_by_key(|c| (c.source != "spotify", !c.exact, source_rank(c.source)));
        let alignment = sync_apple(&mut apple, &donors);
        return Some((apple, alignment));
    }

    // path 1: we have ground truth for this exact recording
    if let Some(reference) = reference.filter(|r| !r.lines.is_empty()) {
        // Apple was handled above. Fallbacks compete on verified word timing.
        let mut best: Option<(usize, Alignment)> = None;
        for (i, c) in pool.iter().enumerate() {
            if !c.word_level() {
                continue;
            }
            let a = align_to_reference(c, reference);
            if !a.aligned {
                continue;
            }
            let win = match &best {
                None => true,
                Some((bi, ba)) => beats(&a, c.source, ba, pool[*bi].source),
            };
            if win {
                best = Some((i, a));
            }
        }
        if let Some((i, mut a)) = best {
            let mut winner = pool.swap_remove(i);
            if winner.exact {
                // it already sits on this recording's clock; the measured offset
                // describes the reference's error, not the candidate's
                a.offset_ms = 0;
            } else {
                winner.shift(a.applied_shift());
            }
            return Some((winner, a));
        }

        // nothing proved itself, so we keep the reference: line-level but
        // correct. this is the governing rule made concrete - we throw away
        // every word-level upgrade rather than ship one that drifts
        return Some((
            reference.clone(),
            Alignment {
                offset_ms:   0,
                residual_ms: 0,
                matched:     reference.lines.len(),
                aligned:     true,
            },
        ));
    }

    if pool.is_empty() {
        // a plain-text-only or instrumental reference is still better than nothing
        return reference.map(|r| (r.clone(), Alignment::unknown()));
    }

    // path 2: no reference - look for consensus instead. two independent
    // providers landing on the same timings is evidence they found the same
    // master; a lone document has nothing corroborating it at all
    if let Some(picked) = cluster_pick(&mut pool) {
        return Some(picked);
    }

    // path 3: nothing verified anything. take the most trustworthy document and
    // be honest about it - `aligned: false` lets the caller keep racing for an
    // upgrade instead of treating this as settled
    let best = (0..pool.len()).min_by_key(|&i| fallback_key(&pool[i]))?;
    Some((pool.swap_remove(best), Alignment::unknown()))
}

/// Ordered, one-to-one text matches. Unlike the fallback alignment heuristic,
/// a shared prefix or a nearby timestamp cannot donate another line's timing.
fn matching_lines(apple: &Candidate, donor: &Candidate) -> Vec<(usize, usize)> {
    let a: Vec<_> = apple.lines.iter().map(|l| normalize(&l.text)).collect();
    let b: Vec<_> = donor.lines.iter().map(|l| normalize(&l.text)).collect();
    // Prefer the closest occurrence when a repeated chorus has equally many
    // ordered matches. A missing repeat must not steal the next chorus's timing.
    let mut lengths = vec![vec![(0usize, 0i64); b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            let mut best = lengths[i + 1][j].max(lengths[i][j + 1]);
            if !a[i].is_empty() && a[i] == b[j] {
                let (count, cost) = lengths[i + 1][j + 1];
                best = best.max((count + 1, cost - (apple.lines[i].time_ms - donor.lines[j].time_ms).abs()));
            }
            lengths[i][j] = best;
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut pairs = Vec::new();
    while i < a.len() && j < b.len() {
        let (count, cost) = lengths[i + 1][j + 1];
        let matched = (count + 1, cost - (apple.lines[i].time_ms - donor.lines[j].time_ms).abs());
        if !a[i].is_empty() && a[i] == b[j] && lengths[i][j] == matched {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if lengths[i + 1][j] > lengths[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

fn shift_line(line: &mut super::types::LyricLine, delta: i64) {
    line.time_ms += delta;
    line.end_ms = line.end_ms.map(|end| end + delta);
    for word in &mut line.words {
        word.time_ms += delta;
        word.end_ms += delta;
    }
    if let Some(bg) = line.bg.as_mut() {
        shift_line(bg, delta);
    }
}

fn sync_apple(apple: &mut Candidate, donors: &[&Candidate]) -> Alignment {
    let mut synced = vec![false; apple.lines.len()];
    let mut deltas = Vec::new();
    for donor in donors {
        for (ai, di) in matching_lines(apple, donor) {
            let line = &mut apple.lines[ai];
            let timing = &donor.lines[di];
            if !synced[ai] {
                // ISRC-matched Apple already has this recording's clock.
                // A fuzzy lookup needs per-line correction, not a median shift.
                let delta = if apple.exact { 0 } else { timing.time_ms - line.time_ms };
                shift_line(line, delta);
                deltas.push(delta);
                synced[ai] = true;
            }
            if line.words.is_empty() && !timing.words.is_empty() {
                // Borrow real word timings only when every word corresponds.
                // Keep Apple's spelling, punctuation and whitespace verbatim.
                let tokens: Vec<_> = line.text.split_inclusive(char::is_whitespace).collect();
                let tokens: Vec<_> = tokens.into_iter().fold(Vec::<String>::new(), |mut out, t| {
                    if t.trim().is_empty() {
                        if let Some(last) = out.last_mut() { last.push_str(t); }
                    } else { out.push(t.to_string()); }
                    out
                });
                if tokens.len() == timing.words.len() && tokens.iter().zip(&timing.words)
                    .all(|(a, b)| normalize(a) == normalize(&b.text)) {
                    let delta = line.time_ms - timing.time_ms;
                    line.words = tokens.into_iter().zip(&timing.words).map(|(text, word)| {
                        super::types::LyricWord { text, time_ms: word.time_ms + delta, end_ms: word.end_ms + delta }
                    }).collect();
                }
            }
        }
    }
    apple.lines.sort_by_key(|line| line.time_ms);
    if deltas.is_empty() { return Alignment::unknown(); }
    let delta = median(&mut deltas.clone());
    Alignment { offset_ms: -delta, residual_ms: mad(&deltas, delta), matched: deltas.len(), aligned: true }
}

// the measurement core

/// pair lines up, take the median delta as the offset and the median absolute
/// deviation around it as the residual. MAD, not stddev: a handful of mispaired
/// lines is normal and inevitable (credit lines, `♪` markers, a chorus repeated
/// a different number of times), and a mean/stddev would let those few outliers
/// drag the verdict around. the median and the MAD simply ignore them.
fn measure(cand: &Candidate, refr: &Candidate, match_denom: usize, skew_denom: usize) -> Alignment {
    if cand.lines.is_empty() || refr.lines.is_empty() {
        return Alignment::unknown();
    }

    // arrangement checks first: both are cheap, and neither failure is something
    // an offset could ever fix. a line-count gap this large is a different cut of
    // the song...
    if skew_denom > 0 {
        let gap = (cand.lines.len() as i64 - refr.lines.len() as i64).abs() as f64;
        if gap / skew_denom as f64 > MAX_COUNT_SKEW {
            return Alignment::unknown();
        }
    }

    // ...and a different span is a different pacing, i.e. a different master
    let (cs, rs) = (span(cand), span(refr));
    if cs >= SPAN_FLOOR_MS && rs >= SPAN_FLOOR_MS {
        let gap = (cs - rs).abs() as f64 / cs.max(rs) as f64;
        if gap > MAX_SPAN_SKEW {
            return Alignment::unknown();
        }
    }

    // Normalize each line once. Every coarse shift and admissible pair shares
    // this text instead of allocating two strings for each comparison.
    let texts = TextEvidence::new(cand, refr);

    // try a few coarse global shifts and keep whichever explains the most lines.
    // without this the pairing window would have to be as wide as the offset we
    // are trying to measure, which would let lines pair with their neighbours
    let mut best = Alignment::unknown();
    for coarse in coarse_hypotheses(cand, refr, &texts) {
        let deltas = pair_deltas(cand, refr, coarse, &texts);
        if deltas.is_empty() {
            continue;
        }
        let offset   = median(&mut deltas.clone());
        let residual = mad(&deltas, offset);
        let better = deltas.len() > best.matched
            || (deltas.len() == best.matched && residual < best.residual_ms);
        if better {
            best = Alignment { offset_ms: offset, residual_ms: residual, matched: deltas.len(), aligned: false };
        }
    }

    if best.matched == 0 {
        return Alignment::unknown();
    }

    let need = MIN_MATCHED.max((match_denom as f64 * MIN_MATCH_RATIO).ceil() as usize);
    best.aligned = best.matched >= need
        && best.residual_ms <= MAX_RESIDUAL_MS
        // a 20s "offset" is not latency, it's a different recording that happens
        // to have a similarly-shaped line grid
        && best.offset_ms.abs() <= MAX_OFFSET_MS;
    best
}

/// starting guesses for the global shift. cheap, and one of them is almost
/// always within the pairing window of the truth.
fn coarse_hypotheses(cand: &Candidate, refr: &Candidate, texts: &TextEvidence) -> Vec<i64> {
    let mut out = vec![0];

    // medians of the two onset lists: robust to missing lines at either end,
    // which is exactly how providers differ (credits, intro markers)
    let mut co = cand.onsets();
    let mut ro = refr.onsets();
    out.push(median(&mut co) - median(&mut ro));

    // first-line delta: right whenever both documents start at the same lyric
    out.push(cand.lines[0].time_ms - refr.lines[0].time_ms);

    // densest delta among text-corroborated pairs. this is the one that survives
    // a document with extra lines bolted onto the front
    if let Some(d) = densest_text_delta(cand, refr, texts) {
        out.push(d);
    }

    out.sort_unstable();
    out.dedup();
    out
}

/// look only at pairs whose TEXT matches, collect their deltas, and return the
/// value with the most neighbours within one residual window - the mode of a
/// histogram, without building one.
fn densest_text_delta(cand: &Candidate, refr: &Candidate, texts: &TextEvidence) -> Option<i64> {
    const MAX_SAMPLES: usize = 400;
    const INDEX_WINDOW: usize = 24; // documents never reorder by more than this

    let mut deltas: Vec<i64> = Vec::new();
    for (ri, r) in refr.lines.iter().enumerate() {
        let start = ri.saturating_sub(INDEX_WINDOW);
        let end = (ri + INDEX_WINDOW + 1).min(cand.lines.len());
        for ci in start..end {
            let c = &cand.lines[ci];
            if !texts.candidate[ci].identifies(&texts.reference[ri]) {
                continue;
            }
            deltas.push(c.time_ms - r.time_ms);
            if deltas.len() >= MAX_SAMPLES {
                break;
            }
        }
        if deltas.len() >= MAX_SAMPLES {
            break;
        }
    }
    if deltas.is_empty() {
        return None;
    }

    let mut best = (0usize, deltas[0]);
    for &d in &deltas {
        let n = deltas.iter().filter(|&&o| (o - d).abs() <= MAX_RESIDUAL_MS).count();
        if n > best.0 {
            best = (n, d);
        }
    }
    Some(best.1)
}

/// One-to-one pairing between reference and candidate lines, assigned
/// BEST-FIRST rather than in reference order.
///
/// The order matters more than it looks. Walking the reference in sequence and
/// letting each line take its nearest free partner cascades whenever the
/// candidate is missing a line (credits, an interlude marker): the reference
/// line whose partner vanished reaches past the gap and takes the NEXT
/// candidate line, that one steals its successor's, and every pair to the end of
/// the document is off by one. Because the resulting deltas are all inflated by
/// exactly one line spacing, the residual stays at zero and the verdict comes
/// back "aligned" a whole line out - confidently wrong, the worst thing this
/// module can do.
///
/// Assigning the closest pairs first removes it: once the coarse shift is
/// applied, true partners sit at ~0ms and are locked in before any line a full
/// spacing away is even considered. A dropped line then costs exactly one
/// unmatched line instead of corrupting the tail.
///
/// One-to-one still matters on its own: without it a single candidate line could
/// "explain" three reference lines and inflate `matched`.
fn pair_deltas(cand: &Candidate, refr: &Candidate, coarse: i64, texts: &TextEvidence) -> Vec<i64> {
    // every admissible pair, then sort by quality and take greedily
    let mut pairs: Vec<(i64, bool, usize, usize)> = Vec::new(); // dist, text, ri, ci
    for (ri, r) in refr.lines.iter().enumerate() {
        let target = r.time_ms + coarse;
        for (ci, c) in cand.lines.iter().enumerate() {
            let dist = (c.time_ms - target).abs();
            if dist > PAIR_WINDOW_MS {
                continue;
            }
            pairs.push((dist, texts.candidate[ci].identifies(&texts.reference[ri]), ri, ci));
        }
    }

    // text corroboration outranks proximity, so a line that drifted closer to
    // its neighbour still pairs with its real counterpart; distance breaks the
    // rest. `!text` sorts corroborated pairs first.
    pairs.sort_unstable_by_key(|&(dist, txt, ri, ci)| (!txt, dist, ri, ci));

    let mut used_r = vec![false; refr.lines.len()];
    let mut used_c = vec![false; cand.lines.len()];
    let mut deltas = Vec::with_capacity(refr.lines.len());

    for (_, _, ri, ci) in pairs {
        if used_r[ri] || used_c[ci] {
            continue;
        }
        used_r[ri] = true;
        used_c[ci] = true;
        deltas.push(cand.lines[ci].time_ms - refr.lines[ri].time_ms);
    }

    deltas
}

// text corroboration

struct TextEvidence {
    candidate: Vec<NormalizedText>,
    reference: Vec<NormalizedText>,
}

impl TextEvidence {
    fn new(cand: &Candidate, refr: &Candidate) -> Self {
        let lines = |c: &Candidate| c.lines.iter()
            .map(|line| NormalizedText::new(&line.text)).collect();
        Self { candidate: lines(cand), reference: lines(refr) }
    }
}

struct NormalizedText {
    text: String,
    prefix_len: usize,
}

impl NormalizedText {
    fn new(text: &str) -> Self {
        let text = normalize(text);
        let prefix_len = text.chars().take(TEXT_PREFIX_MAX).count();
        Self { text, prefix_len }
    }

    /// Containment or a shared prefix corroborates a possible time pairing.
    fn identifies(&self, other: &Self) -> bool {
        let (a, b) = (&self.text, &other.text);
        if a.is_empty() || b.is_empty() {
            return false;
        }
        if a.contains(b.as_str()) || b.contains(a.as_str()) {
            return true;
        }
        let n = self.prefix_len.min(other.prefix_len);
        n >= TEXT_PREFIX_MIN && a.chars().take(n).eq(b.chars().take(n))
    }
}

/// lowercase, drop everything that isn't alphanumeric. providers disagree about
/// punctuation, capitalisation, apostrophes and spacing constantly; none of that
/// tells us anything about whether it's the same line.
fn normalize(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// cheap "is this plausibly the same line" test. containment or a shared prefix
/// is enough - we only need it to break ties in the time pairing, so a real
/// fuzzy-distance crate would be a dependency bought for nothing.
#[cfg(test)]
fn text_identifies(a: &str, b: &str) -> bool {
    NormalizedText::new(a).identifies(&NormalizedText::new(b))
}

// statistics

/// how long the document is actually sung over: first onset to last. two cuts of
/// the same song share a span; a radio edit and an album version do not, and no
/// single offset can reconcile that - which is exactly what the caller uses this
/// for. lines are kept sorted, so the ends are the extremes.
fn span(c: &Candidate) -> i64 {
    match (c.lines.first(), c.lines.last()) {
        (Some(f), Some(l)) => {
            // prefer a real sung end when the source carries word timings
            let end = l.end_ms().unwrap_or(l.time_ms);
            (end - f.time_ms).max(0)
        }
        _ => 0,
    }
}

fn median(v: &mut Vec<i64>) -> i64 {
    if v.is_empty() {
        return 0;
    }
    v.sort_unstable();
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2 }
}

/// median absolute deviation around `center`
fn mad(deltas: &[i64], center: i64) -> i64 {
    let mut devs: Vec<i64> = deltas.iter().map(|d| (d - center).abs()).collect();
    median(&mut devs)
}

// selection helpers

/// does alignment `a` (from source `a_src`) beat `b`? both have already passed
/// the gates, so this is purely a preference order.
fn beats(a: &Alignment, a_src: &str, b: &Alignment, b_src: &str) -> bool {
    let (ba, bb) = (a.residual_ms / RESIDUAL_BUCKET_MS, b.residual_ms / RESIDUAL_BUCKET_MS);
    if ba != bb {
        return ba < bb;
    }
    let (ra, rb) = (source_rank(a_src), source_rank(b_src));
    if ra != rb {
        return ra < rb;
    }
    a.matched > b.matched
}

/// ordering for the last-resort pick, lowest wins: timed lines beat plain text
/// beats a bare instrumental flag, then trust order, then word timing, then the
/// fuller document.
fn fallback_key(c: &Candidate) -> (u8, usize, u8, std::cmp::Reverse<usize>) {
    let tier = if !c.lines.is_empty() {
        0
    } else if c.plain.is_some() {
        1
    } else {
        2
    };
    let word = if c.word_level() { 0 } else { 1 };
    (tier, source_rank(c.source), word, std::cmp::Reverse(c.lines.len()))
}

/// find a cluster of mutually-agreeing candidates and return its best member.
///
/// note we do NOT shift the winner: with no reference there's no ground truth to
/// shift toward, and the cluster members agree to within the residual tolerance
/// anyway. the agreement is used as a trust signal, not as a correction.
fn cluster_pick(pool: &mut Vec<Candidate>) -> Option<(Candidate, Alignment)> {
    let n = pool.len();
    if n < 2 {
        return None;
    }

    let mut partners: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut evidence: Vec<Alignment> = vec![Alignment::unknown(); n];

    for i in 0..n {
        for j in (i + 1)..n {
            let a = cross_agrees(&pool[i], &pool[j]);
            if !a.aligned {
                continue;
            }
            partners[i].push(j);
            partners[j].push(i);
            // keep the tightest agreement each side achieved, as its evidence
            for k in [i, j] {
                if a.residual_ms < evidence[k].residual_ms {
                    evidence[k] = a;
                }
            }
        }
    }

    // anchor = the document the most others corroborate
    let anchor = (0..n).max_by_key(|&i| (partners[i].len(), std::cmp::Reverse(fallback_key(&pool[i]))))?;
    if partners[anchor].is_empty() {
        return None;
    }

    // best member of the anchor's cluster - word timing is safe to prefer here
    // because every member has been corroborated by at least the anchor
    let mut cluster = partners[anchor].clone();
    cluster.push(anchor);
    let pick = *cluster.iter().min_by_key(|&&i| fallback_key(&pool[i]))?;

    let mut verdict = evidence[pick];
    // we return the document untouched, so say so
    verdict.offset_ms = 0;
    verdict.aligned = true;

    Some((pool.swap_remove(pick), verdict))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lyrics::types::{LyricLine, LyricWord};

    #[test]
    fn apple_survives_sync_disagreement_and_is_corrected_line_by_line() {
        let reference = doc("spotify", 20, 5_000, 4_000);
        let other = worded_doc("amll", 20, 5_000, 4_000);
        let mut apple = worded_doc("applemusic", 20, 5_000, 4_000);
        for (i, line) in apple.lines.iter_mut().enumerate() {
            let delta = if i % 2 == 0 { 450 } else { -450 };
            line.time_ms += delta;
            for word in &mut line.words {
                word.time_ms += delta;
                word.end_ms += delta;
            }
        }
        assert!(align_to_reference(&apple, &reference).aligned);
        assert_eq!(align_to_reference(&apple, &reference).residual_ms, 450);
        for pool in [vec![other.clone(), apple.clone()], vec![apple.clone(), other.clone()]] {
            assert_eq!(choose(Some(&reference), pool).unwrap().0.source, "applemusic");
        }

        for (i, line) in apple.lines.iter_mut().enumerate() {
            let delta = i as i64 * 2_000;
            line.time_ms += delta;
            for word in &mut line.words {
                word.time_ms += delta;
                word.end_ms += delta;
            }
        }
        assert!(!align_to_reference(&apple, &reference).aligned);
        let (won, _) = choose(Some(&reference), vec![apple, other]).unwrap();
        assert_eq!(won.source, "applemusic");
        assert_eq!(won.onsets(), reference.onsets());
    }

    /// a document whose lines sit `step` apart starting at `start`, each with
    /// distinct text so the pairing has something real to corroborate with
    fn doc(source: &'static str, n: usize, start: i64, step: i64) -> Candidate {
        let lines = (0..n)
            .map(|i| LyricLine::line(start + i as i64 * step, format!("line number {i} of the song")))
            .collect();
        Candidate::new(source, lines)
    }

    /// same, but carrying real per-word timings
    fn worded_doc(source: &'static str, n: usize, start: i64, step: i64) -> Candidate {
        let lines = (0..n)
            .map(|i| {
                let t = start + i as i64 * step;
                let words = vec![
                    LyricWord { time_ms: t,       end_ms: t + 400, text: format!("line{i} ") },
                    LyricWord { time_ms: t + 400, end_ms: t + 900, text: "words".to_string() },
                ];
                LyricLine::worded(t, Some(format!("line number {i} of the song")), words)
            })
            .collect();
        Candidate::new(source, lines)
    }

    fn shifted(mut c: Candidate, by: i64) -> Candidate {
        c.shift(by);
        c
    }

    /// deterministic pseudo-jitter, roughly uniform over ±2500ms
    fn jitter(i: usize) -> i64 {
        ((i as i64 * 7919) % 5001) - 2500
    }

    #[test]
    fn clean_shift_aligns() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let cand = shifted(doc("musixmatch", 20, 1_000, 3_000), 300);

        let a = align_to_reference(&cand, &reference);
        assert!(a.aligned, "a clean global shift is exactly what we're meant to correct: {a:?}");
        assert_eq!(a.offset_ms, 300);
        assert_eq!(a.matched, 20);
        assert_eq!(a.residual_ms, 0);
        // applying the shift puts it back on the reference
        assert_eq!(a.applied_shift(), -300);
    }

    #[test]
    fn negative_shift_aligns() {
        let reference = doc("spotify", 16, 4_000, 3_500);
        let cand = shifted(doc("netease", 16, 4_000, 3_500), -450);

        let a = align_to_reference(&cand, &reference);
        assert!(a.aligned, "{a:?}");
        assert_eq!(a.offset_ms, -450);
    }

    #[test]
    fn per_line_jitter_is_rejected() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mut cand = doc("lrclib", 20, 1_000, 3_000);
        for (i, l) in cand.lines.iter_mut().enumerate() {
            l.time_ms += 300 + jitter(i);
        }
        cand.lines.sort_by_key(|l| l.time_ms);

        let a = align_to_reference(&cand, &reference);
        assert!(
            !a.aligned,
            "internally inconsistent timing is not a correctable offset: {a:?}"
        );
        assert!(a.residual_ms > MAX_RESIDUAL_MS, "{a:?}");
    }

    #[test]
    fn different_arrangement_is_rejected() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        // twice the lines: a different cut of the song, not a timing problem
        let cand = doc("netease", 40, 1_000, 1_500);

        let a = align_to_reference(&cand, &reference);
        assert!(!a.aligned, "{a:?}");
        assert_eq!(a.matched, 0, "the arrangement check should short-circuit before pairing");
    }

    #[test]
    fn huge_offset_is_rejected() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let cand = shifted(doc("musixmatch", 20, 1_000, 3_000), 30_000);

        let a = align_to_reference(&cand, &reference);
        assert!(
            !a.aligned,
            "30s is a different recording, not latency: {a:?}"
        );
    }

    #[test]
    fn borderline_offset_still_aligns() {
        // 4s is a plausible intro/count-in difference and must survive
        let reference = doc("spotify", 20, 2_000, 3_000);
        let cand = shifted(doc("amll", 20, 2_000, 3_000), 4_000);

        let a = align_to_reference(&cand, &reference);
        assert!(a.aligned, "{a:?}");
        assert_eq!(a.offset_ms, 4_000);
    }

    #[test]
    fn too_few_lines_to_judge() {
        // 3 pairs is not evidence, whatever the residual says
        let reference = doc("spotify", 3, 1_000, 3_000);
        let cand = shifted(doc("lrclib", 3, 1_000, 3_000), 200);

        assert!(!align_to_reference(&cand, &reference).aligned);
    }

    #[test]
    fn choose_prefers_aligned_word_level_over_the_reference() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let word = shifted(worded_doc("musixmatch", 20, 1_000, 3_000), 300);
        let line = doc("lrclib", 20, 1_000, 3_000);

        let (won, a) = choose(Some(&reference), vec![line, word]).expect("something must win");
        assert_eq!(won.source, "musixmatch");
        assert!(won.word_level());
        assert!(a.aligned);
        // returned already corrected: it now sits exactly on the reference
        assert_eq!(won.lines[0].time_ms, reference.lines[0].time_ms);
        assert_eq!(won.lines[0].words[0].time_ms, reference.lines[0].time_ms);
    }

    #[test]
    fn apple_line_lyrics_beat_other_word_sources_and_use_reference_sync() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let apple = shifted(doc("applemusic", 20, 1_000, 3_000), 600);
        let other = worded_doc("amll", 20, 1_000, 3_000);
        let text = apple.lines[0].text.clone();
        let (won, alignment) = choose(Some(&reference), vec![other, apple]).unwrap();
        assert_eq!(won.source, "applemusic");
        assert!(!won.word_level());
        assert_eq!(won.lines[0].text, text);
        assert_eq!(won.lines[0].time_ms, reference.lines[0].time_ms);
        assert_eq!(alignment.applied_shift(), -600);
    }

    #[test]
    fn apple_is_canonical_even_when_no_donor_text_matches() {
        for source in ["applemusic", "betterlyrics"] {
            let apple = Candidate::new(source, vec![LyricLine::line(1234, "Apple wording")]);
            let spotify = doc("spotify", 20, 1000, 3000);
            let (won, alignment) = choose(Some(&spotify), vec![apple.clone()]).unwrap();
            assert_eq!(won.source, source);
            assert_eq!(won.lines[0].text, "Apple wording");
            assert_eq!(won.lines[0].time_ms, 1234);
            assert!(!alignment.aligned);
            let (won, _) = choose(None, vec![spotify, apple]).unwrap();
            assert_eq!(won.source, source);
        }
    }

    #[test]
    fn donor_words_keep_apple_text_and_cannot_reuse_a_repeated_line() {
        let apple = Candidate::new("applemusic", vec![
            LyricLine::line(1100, "Hello, WORLD!"),
            LyricLine::line(5100, "Hello, WORLD!"),
        ]);
        let donor = Candidate::new("musixmatch", vec![LyricLine::worded(5000, None, vec![
            LyricWord { time_ms: 5000, end_ms: 5300, text: "hello ".into() },
            LyricWord { time_ms: 5300, end_ms: 5700, text: "world".into() },
        ])]);
        let (won, _) = choose(None, vec![donor, apple]).unwrap();
        assert_eq!(won.source, "applemusic");
        assert!(won.lines[0].words.is_empty(), "one donor line cannot time both repeats");
        assert_eq!(won.lines[0].time_ms, 1100);
        assert_eq!(won.lines[1].time_ms, 5000);
        assert_eq!(won.lines[1].text, "Hello, WORLD!");
        assert_eq!(won.lines[1].words.iter().map(|w| w.text.as_str()).collect::<String>(), "Hello, WORLD!");
        assert_eq!(won.lines[1].words[1].time_ms, 5300);
    }

    #[test]
    fn exact_apple_keeps_native_words_and_overlapping_singers() {
        let mut apple = worded_doc("applemusic", 10, 1000, 3000);
        apple.exact = true;
        apple.lines[1].time_ms = 1100;
        apple.lines[1].agent = Some("guest".into());
        apple.lines[1].role = super::super::types::LineRole::Duet;
        let before = format!("{:?}", apple.lines);
        let reference = doc("spotify", 10, 2000, 4000);
        let (won, _) = choose(Some(&reference), vec![apple]).unwrap();
        assert_eq!(format!("{:?}", won.lines), before);
    }

    #[test]
    fn second_apple_endpoint_precedes_community_lyrics_after_sync_check() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let apple = doc("betterlyrics", 20, 1_000, 3_000);
        let other = worded_doc("amll", 20, 1_000, 3_000);
        assert_eq!(choose(Some(&reference), vec![other, apple.clone()]).unwrap().0.source, "betterlyrics");
        assert_eq!(choose(None, vec![reference.clone(), apple.clone()]).unwrap().0.source, "betterlyrics");

        let primary = doc("applemusic", 20, 1_000, 3_000);
        assert_eq!(choose(Some(&reference), vec![apple, primary]).unwrap().0.source, "applemusic");
    }

    #[test]
    fn choose_keeps_the_reference_when_word_level_drifts() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mut word = worded_doc("musixmatch", 20, 1_000, 3_000);
        for (i, l) in word.lines.iter_mut().enumerate() {
            let d = 300 + jitter(i);
            l.time_ms += d;
            for w in &mut l.words {
                w.time_ms += d;
                w.end_ms += d;
            }
        }
        word.lines.sort_by_key(|l| l.time_ms);

        let (won, a) = choose(Some(&reference), vec![word]).expect("the reference is always a fallback");
        assert_eq!(won.source, "spotify", "a drifting word-level upgrade must be thrown away");
        assert!(!won.word_level());
        assert!(a.aligned);
        assert_eq!(a.offset_ms, 0);
    }

    #[test]
    fn choose_rejects_a_word_level_document_for_another_master() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let other_master = worded_doc("netease", 20, 1_000, 4_100); // same lines, different pacing

        let (won, _) = choose(Some(&reference), vec![other_master]).unwrap();
        assert_eq!(won.source, "spotify");
    }

    #[test]
    fn cross_agreement_needs_two_sources() {
        let a = doc("netease", 18, 1_500, 3_200);
        let b = shifted(doc("lrclib", 18, 1_500, 3_200), 120);

        let v = cross_agrees(&a, &b);
        assert!(v.aligned, "{v:?}");
        assert_eq!(v.offset_ms, -120);
    }

    #[test]
    fn cross_agreement_rejects_disagreement() {
        let a = doc("netease", 18, 1_500, 3_200);
        let b = doc("lrclib", 18, 1_500, 4_400);

        assert!(!cross_agrees(&a, &b).aligned);
    }

    #[test]
    fn choose_without_reference_takes_the_agreeing_cluster() {
        // two sources agree; a third is off in its own world and must not win
        // despite being word-level
        let a = doc("lrclib", 18, 1_500, 3_200);
        let b = worded_doc("netease", 18, 1_500, 3_200);
        let rogue = worded_doc("qq", 18, 40_000, 3_200);

        let (won, v) = choose(None, vec![rogue, a, b]).expect("a cluster exists");
        assert_eq!(won.source, "netease", "word-level member of the agreeing cluster");
        assert!(v.aligned);
        assert_eq!(v.offset_ms, 0, "no reference means nothing to shift toward");
        // untouched timings
        assert_eq!(won.lines[0].time_ms, 1_500);
    }

    #[test]
    fn choose_without_reference_falls_back_to_trust_order() {
        // nothing corroborates anything, so trust order decides and the result
        // is honestly marked unverified
        let a = doc("lrclib", 18, 1_500, 3_200);
        let b = doc("musixmatch", 18, 9_000, 4_400);

        let (won, v) = choose(None, vec![a, b]).unwrap();
        assert_eq!(won.source, "musixmatch");
        assert!(!v.aligned);
    }

    #[test]
    fn choose_ignores_empty_candidates() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let empty = Candidate::new("kugou", Vec::new());

        let (won, _) = choose(Some(&reference), vec![empty]).unwrap();
        assert_eq!(won.source, "spotify");
    }

    #[test]
    fn choose_returns_none_when_there_is_nothing() {
        assert!(choose(None, Vec::new()).is_none());
        assert!(choose(None, vec![Candidate::new("qq", Vec::new())]).is_none());
    }

    #[test]
    fn choose_prefers_plain_text_over_a_bare_instrumental_flag() {
        let mut plain = Candidate::new("lrclib", Vec::new());
        plain.plain = Some("some words".to_string());
        let mut inst = Candidate::new("netease", Vec::new());
        inst.instrumental = true;

        let (won, _) = choose(None, vec![inst, plain]).unwrap();
        assert_eq!(won.source, "lrclib");
    }

    #[test]
    fn an_id_keyed_winner_keeps_its_own_timing() {
        // AMLL is keyed by spotify track id and hand-corrected, so a measured
        // delta against the line-level reference is the REFERENCE's error. we
        // must verify it belongs to this recording and then leave it alone.
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mut amll = shifted(worded_doc("amll", 20, 1_000, 3_000), 300);
        amll.exact = true; // as the real provider marks it
        let first_before = amll.lines[0].time_ms;

        let (won, a) = choose(Some(&reference), vec![amll]).unwrap();
        assert_eq!(won.source, "amll");
        assert_eq!(a.offset_ms, 0, "an id-keyed source is not shifted onto the reference");
        assert_eq!(won.lines[0].time_ms, first_before, "its timings must be untouched");
    }

    #[test]
    fn a_fuzzy_matched_winner_is_shifted_onto_the_reference() {
        // musixmatch was found by name+artist+duration, so its absolute timing
        // came from whatever master the search returned - it MUST be corrected
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mxm = shifted(worded_doc("musixmatch", 20, 1_000, 3_000), 300);

        let (won, a) = choose(Some(&reference), vec![mxm]).unwrap();
        assert_eq!(won.source, "musixmatch");
        assert_eq!(a.offset_ms, 300);
        assert_eq!(won.lines[0].time_ms, reference.lines[0].time_ms, "shifted onto the reference");
    }

    #[test]
    fn missing_and_extra_lines_still_align() {
        // the realistic case: the candidate skips a couple of lines the
        // reference has (credits, an interlude marker) but is otherwise correct
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mut cand = shifted(doc("amll", 20, 1_000, 3_000), 250);
        cand.lines.remove(11);
        cand.lines.remove(4);

        let a = align_to_reference(&cand, &reference);
        assert!(a.aligned, "a few dropped lines must not cost us the alignment: {a:?}");
        assert_eq!(a.offset_ms, 250);
        assert_eq!(a.matched, 18);
    }

    #[test]
    fn a_few_mispaired_lines_do_not_move_the_verdict() {
        // exactly why MAD and not stddev: three wild outliers, everything else
        // perfect. a mean/stddev would be dragged past the tolerance
        let reference = doc("spotify", 20, 1_000, 3_000);
        let mut cand = shifted(doc("musixmatch", 20, 1_000, 3_000), 200);
        for i in [2usize, 9, 17] {
            cand.lines[i].time_ms += 3_000;
        }
        cand.lines.sort_by_key(|l| l.time_ms);

        let a = align_to_reference(&cand, &reference);
        assert!(a.aligned, "{a:?}");
        assert_eq!(a.offset_ms, 200);
        assert!(a.residual_ms <= MAX_RESIDUAL_MS, "{a:?}");
    }

    #[test]
    fn trust_order_breaks_ties_between_equally_aligned_candidates() {
        let reference = doc("spotify", 20, 1_000, 3_000);
        let lrclib = shifted(worded_doc("lrclib", 20, 1_000, 3_000), 100);
        let amll   = shifted(worded_doc("amll", 20, 1_000, 3_000), 100);

        let (won, _) = choose(Some(&reference), vec![lrclib, amll]).unwrap();
        assert_eq!(won.source, "amll", "hand-corrected timing wins a tie");
    }

    #[test]
    fn a_clearly_tighter_residual_outranks_trust() {
        let reference = doc("spotify", 24, 1_000, 3_000);

        // lrclib is dead-on; amll is loose but still inside tolerance
        let lrclib = shifted(worded_doc("lrclib", 24, 1_000, 3_000), 100);
        let mut amll = worded_doc("amll", 24, 1_000, 3_000);
        for (i, l) in amll.lines.iter_mut().enumerate() {
            l.time_ms += 100 + if i % 2 == 0 { 600 } else { -600 };
        }
        amll.lines.sort_by_key(|l| l.time_ms);

        let (won, _) = choose(Some(&reference), vec![amll, lrclib]).unwrap();
        assert_eq!(won.source, "lrclib", "audible drift beats brand trust");
    }

    #[test]
    fn text_corroboration_basics() {
        assert!(text_identifies("Don't stop me now!", "dont stop me now"));
        assert!(text_identifies("Hello, world", "hello world (ad-lib)"));
        assert!(!text_identifies("hello world", "goodbye world"));
        assert!(!text_identifies("", "hello world"), "an empty line cannot corroborate");
    }

    #[test]
    fn median_and_mad_basics() {
        assert_eq!(median(&mut vec![5, 1, 3]), 3);
        assert_eq!(median(&mut vec![4, 2]), 3);
        assert_eq!(median(&mut vec![]), 0);
        assert_eq!(mad(&[100, 100, 100, 5_000], 100), 0);
    }
}
