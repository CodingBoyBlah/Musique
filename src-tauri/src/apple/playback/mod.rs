//! Apple Music playback: stream resolution, Widevine licensing, lazy CENC decryption,
//! decoder opening, preload cache, and diagnostics probe.

pub mod aac;
pub mod fmp4;
pub mod license;
pub mod resolve;
pub mod source;
pub mod stream;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use tokio::sync::{Mutex, RwLock};

use crate::errors::AppError;

pub use self::resolve::AppleQuality;
use self::resolve::ResolvedTrack;
use self::source::AppleSource;
use self::stream::MediaStream;

/// Maximum prepared tracks held in memory (active + preloaded next).
const PREPARED_CAPACITY: usize = 2;

/// Apple Widevine licenses expire after ~13–15 minutes. Refresh before resuming or seeking
/// if playback has been paused longer than 5 minutes, or if the license is older than 10 minutes.
const STALE_PAUSE: Duration = Duration::from_secs(5 * 60);
const MAX_LICENSE_AGE: Duration = Duration::from_secs(10 * 60);

static APPLE_REQUEST: AtomicU64 = AtomicU64::new(0);

static PREPARED: RwLock<Vec<(String, PreparedTrack)>> = RwLock::const_new(Vec::new());

static INFLIGHT: std::sync::Mutex<Vec<(String, Arc<Mutex<()>>)>> =
    std::sync::Mutex::new(Vec::new());

static ACTIVE: Mutex<Option<ActiveSession>> = Mutex::const_new(None);

/// Resolved, licensed, and primed Apple Music track ready to hand to [`AppleSource::open`].
#[derive(Clone)]
pub struct PreparedTrack {
    pub track_id:    String,
    pub quality:     AppleQuality,
    pub resolved:    ResolvedTrack,
    pub media:       MediaStream,
    pub licensed_at: Option<Instant>,
}

impl PreparedTrack {
    pub fn duration_ms(&self) -> u64 {
        self.resolved
            .duration
            .map_or(0, |d| d.as_millis().min(u64::MAX as u128) as u64)
    }
}

struct ActiveSession {
    prepared:  PreparedTrack,
    paused_at: Option<Instant>,
}

/// Whether a resume or seek can reuse the live `MediaStream` or needs a full reload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshOutcome {
    Ready,
    NeedsReload,
}

/// Claims a new transport request ticket, cancelling any earlier in-flight load.
pub fn claim() -> u64 {
    APPLE_REQUEST.fetch_add(1, Ordering::SeqCst) + 1
}

/// Whether `ticket` is still the latest transport command.
pub fn is_current(ticket: u64) -> bool {
    APPLE_REQUEST.load(Ordering::SeqCst) == ticket
}

/// Reads the user's configured `audio_quality` setting mapped to Apple's AAC flavors.
pub async fn configured_quality(pool: &SqlitePool) -> AppleQuality {
    let setting: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'audio_quality'")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    AppleQuality::from_setting(setting.as_deref())
}

async fn current_client() -> Result<crate::apple::client::AppleClient, AppError> {
    crate::apple::client::current().await
}

fn inflight_lock(track_id: &str) -> Arc<Mutex<()>> {
    let mut map = INFLIGHT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, lock)) = map.iter().find(|(id, _)| id == track_id) {
        return Arc::clone(lock);
    }
    let lock = Arc::new(Mutex::new(()));
    map.push((track_id.to_owned(), Arc::clone(&lock)));
    lock
}

fn release_inflight(track_id: &str, lock: Arc<Mutex<()>>) {
    let mut map = INFLIGHT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if Arc::strong_count(&lock) <= 2 {
        map.retain(|(id, _)| id != track_id);
    }
}

async fn take_cached(track_id: &str, quality: AppleQuality) -> Option<PreparedTrack> {
    PREPARED
        .read()
        .await
        .iter()
        .find(|(id, p)| id == track_id && p.quality == quality)
        .map(|(_, p)| p.clone())
}

async fn store_cached(prepared: PreparedTrack) {
    let mut cache = PREPARED.write().await;
    cache.retain(|(id, _)| id != &prepared.track_id);
    cache.push((prepared.track_id.clone(), prepared));
    while cache.len() > PREPARED_CAPACITY {
        cache.remove(0);
    }
}

/// Drops one track from the preload cache (used on explicit retry).
pub async fn evict_prepared(track_id: &str) {
    let lock = inflight_lock(track_id);
    {
        let _guard = lock.lock().await;
        PREPARED.write().await.retain(|(id, _)| id != track_id);
    }
    release_inflight(track_id, lock);
}

/// Drops all preloaded Apple Music streams from the cache.
pub async fn clear_prepared() {
    PREPARED.write().await.clear();
}

async fn try_reuse_cached(
    client: &crate::apple::client::AppleClient,
    bearer: &str,
    track_id: &str,
    quality: AppleQuality,
) -> Option<PreparedTrack> {
    let mut hit = take_cached(track_id, quality).await?;
    if hit.media.failed().is_some() {
        return None;
    }
    refresh_license_if_stale(&mut hit, client.http(), bearer, client.user_token(), false)
        .await
        .ok()?;
    Some(hit)
}

/// Resolves, licenses, starts streaming, and primes `track_id` (reusing any fresh cached entry).
pub async fn load_prepared(pool: &SqlitePool, track_id: &str) -> Result<PreparedTrack, AppError> {
    let quality = configured_quality(pool).await;
    let client = current_client().await?;
    let bearer = client.bearer();

    if let Some(hit) = try_reuse_cached(&client, &bearer, track_id, quality).await {
        return Ok(hit);
    }

    let lock = inflight_lock(track_id);
    let result = {
        let _guard = lock.lock().await;
        match try_reuse_cached(&client, &bearer, track_id, quality).await {
            Some(hit) => Ok(hit),
            None => {
                prepare_with_creds(
                    client.http(),
                    &bearer,
                    client.user_token(),
                    track_id,
                    quality,
                )
                .await
            }
        }
    };
    release_inflight(track_id, lock);
    result
}

/// Resolves, licenses (in parallel with opening the asset response), and primes a track
/// using explicit `(http, bearer, user_token)` parameters.
pub async fn prepare_with_creds(
    http: &reqwest::Client,
    bearer: &str,
    user_token: &str,
    track_id: &str,
    quality: AppleQuality,
) -> Result<PreparedTrack, AppError> {
    let resolved = resolve::resolve(http, track_id, bearer, user_token, quality).await?;
    if resolved.key.is_some() && !crate::widevine::available() {
        return Err(AppError::Playback(
            "Widevine CDM is not installed; cannot play protected Apple Music tracks".into(),
        ));
    }

    let (asset_res, license_res, licensed_at) = match &resolved.key {
        Some(key) => {
            let (a, l) = tokio::join!(
                license::open_asset(&resolved.asset_url, user_token),
                license::acquire_license(http, bearer, user_token, key, resolved.is_library),
            );
            (a, Some(l), Some(Instant::now()))
        }
        None => (
            license::open_asset(&resolved.asset_url, user_token).await,
            None,
            None,
        ),
    };

    let resp = asset_res?;
    let mut media = MediaStream::start(resp, resolved.key.is_none());
    if let (Some(key), Some(cdm_res)) = (&resolved.key, license_res) {
        let cdm = cdm_res?;
        media.install_license(cdm, key.id.clone())?;
        media.release_when_downloaded();
    }
    media.prime().await?;

    let prepared = PreparedTrack {
        track_id: track_id.to_owned(),
        quality,
        resolved,
        media,
        licensed_at,
    };
    store_cached(prepared.clone()).await;
    Ok(prepared)
}

/// Preloads the next track in the background; errors are logged and ignored.
pub async fn preload(pool: &SqlitePool, track_id: &str) {
    if let Err(err) = load_prepared(pool, track_id).await {
        eprintln!("[apple] preload {track_id} failed: {err}");
    }
}

/// Opens a `rodio::Source` over `prepared` positioned at `position_ms` off the async runtime.
pub async fn open_at(
    prepared: &PreparedTrack,
    position_ms: u32,
) -> Result<(AppleSource, u32), AppError> {
    let media = prepared.media.clone();
    let duration = prepared.resolved.duration;
    let at = Duration::from_millis(u64::from(position_ms));
    tokio::task::spawn_blocking(move || AppleSource::open(media, duration, at))
        .await
        .map_err(|e| AppError::Playback(format!("apple decoder open task failed: {e}")))?
}

/// Records `prepared` as the actively playing Apple Music track.
pub async fn set_active(prepared: PreparedTrack) {
    *ACTIVE.lock().await = Some(ActiveSession {
        prepared,
        paused_at: None,
    });
}

/// Clears the active Apple Music session (e.g. on stop or backend switch).
pub async fn clear_active() {
    *ACTIVE.lock().await = None;
}

/// Notes that the active Apple track was paused so resume can refresh a stale Widevine license.
pub async fn note_paused() {
    if let Some(active) = ACTIVE.lock().await.as_mut() {
        active.paused_at = Some(Instant::now());
    }
}

/// Ensures the active track's Widevine license is still valid before resuming from pause.
pub async fn refresh_before_resume(track_id: &str) -> RefreshOutcome {
    refresh_active(track_id, true).await
}

/// Ensures the active track's Widevine license is still valid before seeking.
pub async fn refresh_before_seek(track_id: &str) -> RefreshOutcome {
    refresh_active(track_id, false).await
}

async fn refresh_active(track_id: &str, check_pause: bool) -> RefreshOutcome {
    let mut guard = ACTIVE.lock().await;
    let Some(active) = guard.as_mut() else {
        return RefreshOutcome::NeedsReload;
    };
    if active.prepared.track_id != track_id || active.prepared.media.failed().is_some() {
        return RefreshOutcome::NeedsReload;
    }

    let paused_stale = check_pause
        && active
            .paused_at
            .is_some_and(|since| since.elapsed() >= STALE_PAUSE);
    active.paused_at = None;

    if active.prepared.media.is_all_clear() {
        return RefreshOutcome::Ready;
    }

    let needs_refresh = paused_stale
        || active
            .prepared
            .licensed_at
            .is_some_and(|at| at.elapsed() >= MAX_LICENSE_AGE);
    if !needs_refresh {
        return RefreshOutcome::Ready;
    }

    let Ok(client) = current_client().await else {
        return RefreshOutcome::NeedsReload;
    };
    let bearer = client.bearer();
    match refresh_license_if_stale(
        &mut active.prepared,
        client.http(),
        &bearer,
        client.user_token(),
        paused_stale,
    )
    .await
    {
        Ok(()) => RefreshOutcome::Ready,
        Err(err) => {
            log::warn!("apple: license refresh failed for {track_id}, reloading: {err}");
            RefreshOutcome::NeedsReload
        }
    }
}

async fn refresh_license_if_stale(
    prepared: &mut PreparedTrack,
    http: &reqwest::Client,
    bearer: &str,
    user_token: &str,
    force: bool,
) -> Result<(), AppError> {
    let Some(key) = &prepared.resolved.key else {
        return Ok(());
    };
    if prepared.media.is_all_clear() {
        return Ok(());
    }
    let expired = force
        || prepared
            .licensed_at
            .is_some_and(|at| at.elapsed() >= MAX_LICENSE_AGE);
    if !expired {
        return Ok(());
    }

    let cdm = license::acquire_license(
        http,
        bearer,
        user_token,
        key,
        prepared.resolved.is_library,
    )
    .await?;
    prepared.media.install_license(cdm, key.id.clone())?;
    if prepared.media.done() {
        prepared.media.release_when_downloaded();
    }
    prepared.licensed_at = Some(Instant::now());
    Ok(())
}

/// Fire-and-forget play-activity reporting to Apple once a track begins playing.
pub fn report_play_async(track_id: &str) {
    let id = track_id.to_owned();
    tokio::spawn(async move {
        if let Err(err) = crate::apple::api::report_play(&id).await {
            log::debug!("apple: report_play {id} failed: {err}");
        }
    });
}

/// Self-test probe: resolves, licenses, streams, decrypts initial fragments, and decodes
/// 2 seconds of audio for `track_id`, returning measured stage timings.
///
/// Reads `MUSIQUE_APPLE_MEDIA_USER_TOKEN` (or stored keyring credentials via `apple::auth`).
pub async fn probe(track_id: &str) -> Result<String, AppError> {
    probe_at(track_id, AppleQuality::High256).await
}

/// [`probe`] at a chosen quality, so the 64 kbps HE-AAC path can be timed too.
pub async fn probe_at(track_id: &str, quality: AppleQuality) -> Result<String, AppError> {
    use rodio::Source as _;

    let total_start = Instant::now();
    let client = current_client().await?;
    let http = client.http();
    let bearer = client.bearer();
    let user_token = client.user_token();

    let t_resolve = Instant::now();
    let resolved =
        resolve::resolve(http, track_id, &bearer, user_token, quality).await?;
    let resolve_ms = t_resolve.elapsed().as_millis();

    if resolved.key.is_some() && !crate::widevine::available() {
        return Err(AppError::Playback(
            "Widevine CDM is not available for Apple playback probe".into(),
        ));
    }

    let t_license = Instant::now();
    let (asset_res, license_res) = match &resolved.key {
        Some(key) => {
            let (a, l) = tokio::join!(
                license::open_asset(&resolved.asset_url, user_token),
                license::acquire_license(http, &bearer, user_token, key, resolved.is_library),
            );
            (a, Some(l))
        }
        None => (
            license::open_asset(&resolved.asset_url, user_token).await,
            None,
        ),
    };
    let license_open_ms = t_license.elapsed().as_millis();

    let resp = asset_res?;
    let mut media = MediaStream::start(resp, resolved.key.is_none());
    if let (Some(key), Some(cdm_res)) = (&resolved.key, license_res) {
        let cdm = cdm_res?;
        media.install_license(cdm, key.id.clone())?;
        media.release_when_downloaded();
    }

    let t_prime = Instant::now();
    media.prime().await?;
    let prime_ms = t_prime.elapsed().as_millis();

    let duration = resolved.duration;
    let media_for_decode = media.clone();
    let (rate, channels, decoded_samples, open_ms, decode_ms) =
        tokio::task::spawn_blocking(move || -> Result<(u32, u16, usize, u128, u128), AppError> {
            let t_open = Instant::now();
            let (mut source, _) = AppleSource::open(media_for_decode, duration, Duration::ZERO)?;
            let open_ms = t_open.elapsed().as_millis();

            // MUSIQUE_APPLE_PROBE_REALTIME=<seconds>: play through the app's own playout ring,
            // pulled at the speaker's pace, and count how often it ran dry
            if let Some(secs) = std::env::var("MUSIQUE_APPLE_PROBE_REALTIME").ok().and_then(|v| v.parse::<u64>().ok()) {
                let rate = source.sample_rate();
                let channels = source.channels();
                let (mut ring_source, ring) = crate::playout::prebuffer(source, Duration::ZERO, false, None);
                let per_tick = rate as usize * channels as usize / 100; // 10 ms
                let started = Instant::now();
                let mut ticks = 0u64;
                while started.elapsed() < Duration::from_secs(secs) {
                    for _ in 0..per_tick {
                        let _ = ring_source.next();
                    }
                    ticks += 1;
                    let due = started + Duration::from_millis(ticks * 10);
                    if let Some(wait) = due.checked_duration_since(Instant::now()) {
                        std::thread::sleep(wait);
                    }
                }
                eprintln!("realtime: {secs}s pulled, underruns {}", ring.underruns());
                return Ok((rate, channels, 1, open_ms, 0));
            }

            let rate = source.sample_rate();
            let channels = source.channels();
            // MUSIQUE_APPLE_PROBE_FULL=1 decodes the whole track and reports the slowest stretch
            let full = std::env::var_os("MUSIQUE_APPLE_PROBE_FULL").is_some();
            let target_samples = if full { usize::MAX } else { rate as usize * channels as usize * 2 };
            let chunk = rate as usize * channels as usize / 10; // 100 ms of audio

            let t_decode = Instant::now();
            let mut decoded = 0usize;
            let mut chunk_started = Instant::now();
            let mut slowest = Duration::ZERO;
            let mut slow_chunks = 0usize;
            while decoded < target_samples {
                if source.next().is_none() {
                    break;
                }
                decoded += 1;
                if full && decoded % chunk == 0 {
                    let took = chunk_started.elapsed();
                    if took > Duration::from_millis(100) {
                        slow_chunks += 1;
                    }
                    slowest = slowest.max(took);
                    chunk_started = Instant::now();
                }
            }
            if full {
                let audio_s = decoded as f64 / (rate as f64 * channels as f64);
                eprintln!(
                    "full decode: {audio_s:.1}s of audio in {:.2}s ({:.1}x realtime), slowest 100ms chunk {:?}, {slow_chunks} chunks slower than realtime",
                    t_decode.elapsed().as_secs_f64(),
                    audio_s / t_decode.elapsed().as_secs_f64().max(1e-9),
                    slowest
                );
            }
            let decode_ms = t_decode.elapsed().as_millis();
            Ok((rate, channels, decoded, open_ms, decode_ms))
        })
        .await
        .map_err(|e| AppError::Playback(format!("probe decode task failed: {e}")))??;

    if decoded_samples == 0 {
        return Err(AppError::Playback(format!(
            "probe decoded 0 samples for track {track_id}"
        )));
    }

    let (decrypted, indexed, decrypt_time) = media.metrics();
    Ok(format!(
        "track={track_id} flavor={:?} rate={rate}Hz ch={channels} decoded_samples={decoded_samples} \
         resolve={resolve_ms}ms license_and_open={license_open_ms}ms prime={prime_ms}ms \
         decoder_open={open_ms}ms decode_2s={decode_ms}ms decrypted={decrypted}/{indexed} ({}ms in CDM) \
         total={}ms",
        resolved.flavor,
        decrypt_time.as_millis(),
        total_start.elapsed().as_millis(),
    ))
}
