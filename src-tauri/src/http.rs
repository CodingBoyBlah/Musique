// Shared HTTP clients.
//
// Every outbound call used to build its own `reqwest::Client::new()`. A Client
// owns its connection pool, so a throwaway client means a throwaway pool: every
// single Spotify API call paid a fresh DNS lookup + TCP handshake + full TLS
// handshake (1-2 extra round trips, ~100-300ms on a normal connection) and then
// dropped the connection instead of keeping it warm. Library sync, search,
// album/artist/playlist loads and token refreshes all paid that tax on every
// request.
//
// One process-wide client keeps the connections to api.spotify.com and
// accounts.spotify.com alive, so everything after the first request skips
// DNS/TCP/TLS entirely. HTTP/2 also lets concurrent requests (the API semaphore
// allows 8) share a single connection instead of opening 8 of them.
use std::sync::OnceLock;
use std::time::Duration;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static COOKIE_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn base() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        // keep connections hot between requests - this is the whole point
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(8)
        .tcp_keepalive(Duration::from_secs(60))
        // Nagle off: our requests are small and latency-sensitive
        .tcp_nodelay(true)
        // Spotify JSON pages (liked songs, playlists, search) compress ~5-8x.
        // Explicit rather than relying on the feature default.
        .gzip(true)
        .brotli(true)
}

/// Shared pooled client for the Spotify Web API, the token endpoint, last.fm,
/// Odesli and friends. Reuses connections across every call in the process.
pub fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        base()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(8))
            .build()
            .unwrap_or_default()
    })
}

/// Same pooling, but with a cookie jar - the lyrics provider chain needs
/// musixmatch's token cookies to carry over into its macro call.
pub fn cookie_client() -> &'static reqwest::Client {
    COOKIE_CLIENT.get_or_init(|| {
        base()
            .timeout(Duration::from_secs(9))
            .connect_timeout(Duration::from_secs(5))
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}
