use crate::errors::AppError;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{Mutex, OwnedMutexGuard};

pub const CALLBACK_PORT: u16 = 8989;
pub const PLAYBACK_PORT: u16 = 8898;
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes
/// How often a waiting flow checks whether a newer one has superseded it.
const SUPERSEDE_POLL: Duration = Duration::from_millis(200);

// ─── interactive-flow arbitration ────────────────────────────────────────────
//
// There are two browser authorizations (the Web API login on CALLBACK_PORT and
// the playback grant on PLAYBACK_PORT) and several things that can start one:
// the account menu, the post-login warmup, the startup session warm, and a play
// that finds no session. Without arbitration two of them overlap and the second
// one dies on `Address already in use` - which is what "the tab opens and it
// immediately says sign-in failed" was. Worse, an abandoned flow kept its
// listener bound for the whole login timeout, so *every* retry failed until it
// expired.
//
// So: one browser tab at a time (`flow_lock`), and a newer attempt at the SAME
// flow always wins (the per-port generation). A flow that is waiting for its
// redirect notices within SUPERSEDE_POLL that its generation is stale, drops
// its listener and releases the lock for the newcomer.
//
// The generation is per port on purpose. A single shared counter would let the
// playback grant - which the app can start on its own, off the back of a play -
// cancel a login the user is halfway through typing their password for.

fn flow_lock() -> &'static Arc<Mutex<()>> {
    static LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();
    LOCK.get_or_init(|| Arc::new(Mutex::new(())))
}

static LOGIN_GEN: AtomicU64 = AtomicU64::new(0);
static PLAYBACK_GEN: AtomicU64 = AtomicU64::new(0);

fn generation_for(port: u16) -> &'static AtomicU64 {
    if port == PLAYBACK_PORT { &PLAYBACK_GEN } else { &LOGIN_GEN }
}

/// Abandon every browser authorization that is still waiting for a redirect.
///
/// Signing out in the middle of one has to be possible: the flow may be a
/// playback grant started behind the user's back, and its caller holds the
/// playback lock while it waits. Bumping both generations makes every waiter
/// give up within `SUPERSEDE_POLL` and release what it is holding.
pub fn cancel_pending_flows() {
    LOGIN_GEN.fetch_add(1, Ordering::SeqCst);
    PLAYBACK_GEN.fetch_add(1, Ordering::SeqCst);
}

/// A bound redirect listener, held for the duration of one browser authorization.
///
/// Binding happens *before* the browser is opened so a port conflict surfaces as
/// an error instead of a tab the user fills in for nothing.
pub struct Redirect {
    listener:   TcpListener,
    port:       u16,
    generation: u64,
    _guard:     OwnedMutexGuard<()>,
}

/// Claim the interactive-auth slot and bind the redirect port.
pub async fn open_redirect(port: u16) -> Result<Redirect, AppError> {
    // Claim the newest generation first: any flow currently waiting sees this
    // and steps aside, releasing the lock (and the port) for us.
    let generation = generation_for(port).fetch_add(1, Ordering::SeqCst) + 1;

    let guard = tokio::time::timeout(Duration::from_secs(10), flow_lock().clone().lock_owned())
        .await
        .map_err(|_| {
            AppError::Auth(
                "Another Spotify authorization is still open in the browser.                  Finish or close it, then try again."
                    .into(),
            )
        })?;

    // A third attempt may have arrived while we queued; the newest one wins.
    if generation_for(port).load(Ordering::SeqCst) != generation {
        return Err(AppError::Auth("Sign-in restarted".into()));
    }

    let listener = bind_redirect(port).await?;
    Ok(Redirect { listener, port, generation, _guard: guard })
}

/// Bind 127.0.0.1:port, retrying briefly while the OS releases a just-closed
/// listener (Windows in particular holds it for a moment after the process that
/// owned it drops it).
async fn bind_redirect(port: u16) -> Result<TcpListener, AppError> {
    let address: SocketAddr = ([127, 0, 0, 1], port).into();
    let mut last_err = None;

    for _ in 0..16 {
        match TcpListener::bind(address).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Err(e) => {
                return Err(AppError::Auth(format!(
                    "Unable to listen on 127.0.0.1:{port} for the Spotify redirect: {e}"
                )))
            }
        }
    }

    eprintln!("[auth] redirect port {port} still busy: {last_err:?}");
    Err(AppError::Auth(format!(
        "Port {port} is already in use, so Spotify has nowhere to send you back to. \
         Close any other copy of Musique (or whatever is using that port) and try again."
    )))
}

fn page(title: &str, heading: &str, body: &str, accent: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title>\
<style>:root{{color-scheme:dark}}body{{margin:0;min-height:100vh;display:grid;place-items:center;background:#0f1114;color:#e8eaed;font-family:system-ui,-apple-system,BlinkMacSystemFont,Inter,sans-serif}}\
main{{max-width:28rem;padding:2.5rem;border-radius:1.25rem;background:#181b20;box-shadow:0 20px 60px rgba(0,0,0,.5);text-align:center}}\
.mark{{width:64px;height:64px;border-radius:50%;background:{accent};display:grid;place-items:center;margin:0 auto 1.25rem}}\
.mark svg{{width:30px;height:30px;fill:#0f1114}}h1{{font-size:1.4rem;margin:.25rem 0 .5rem}}p{{color:#a5adba;line-height:1.5;margin:0}}</style>\
<main><div class=\"mark\"><svg viewBox=\"0 0 24 24\"><path d=\"M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z\"/></svg></div>\
<h1>{heading}</h1><p>{body}</p></main><script>setTimeout(function(){{window.close()}},1500)</script></html>"
    )
}

fn success_page() -> String {
    page(
        "Signed in to Musique",
        "You're signed in",
        "You can close this tab and return to Musique.",
        "#1ed760",
    )
}

fn failure_page(reason: &str) -> String {
    page(
        "Sign-in failed",
        "Sign-in didn't complete",
        &format!("{reason}. Return to Musique and try again."),
        "#f5717f",
    )
}

/// What a request arriving on the redirect port turned out to be.
enum Callback {
    /// The authorization code we were waiting for.
    Code(String),
    /// Spotify (or the user) refused: there is nothing left to wait for.
    Refused(String),
    /// Not our callback - a favicon probe, a browser pre-connect, or a stale
    /// redirect from an earlier attempt. Answer it and keep waiting.
    Unrelated(String),
}

impl Redirect {
    /// Wait for Spotify to redirect back with an authorization code.
    ///
    /// Consumes the redirect so the port and the interactive-flow slot are
    /// released the moment the flow ends, however it ends.
    pub async fn wait(self, expected_state: &str) -> Result<String, AppError> {
        let port = self.port;
        let deadline = tokio::time::sleep(LOGIN_TIMEOUT);
        tokio::pin!(deadline);
        let mut supersede_check = tokio::time::interval(SUPERSEDE_POLL);

        loop {
            let (mut stream, _) = tokio::select! {
                accepted = self.listener.accept() => accepted.map_err(|e| {
                    AppError::Auth(format!("Redirect listener accept failed: {e}"))
                })?,
                _ = &mut deadline => {
                    return Err(AppError::Auth("Sign-in timed out; please try again".into()))
                }
                _ = supersede_check.tick() => {
                    if generation_for(port).load(Ordering::SeqCst) != self.generation {
                        eprintln!("[auth listener] port {port} released to a newer sign-in attempt");
                        return Err(AppError::Auth("Sign-in restarted".into()));
                    }
                    continue;
                }
            };

            let mut reader = BufReader::new(&mut stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).await.is_err() {
                continue;
            }

            let outcome = classify_request_line(&request_line, expected_state);
            let (status, body) = match &outcome {
                Callback::Code(_) => ("200 OK", success_page()),
                Callback::Refused(reason) => ("400 Bad Request", failure_page(reason)),
                Callback::Unrelated(_) => ("404 Not Found", failure_page("That wasn't the sign-in redirect")),
            };

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;

            match outcome {
                Callback::Code(code) => return Ok(code),
                // A real refusal is final - waiting out the timeout after Spotify
                // already said no just leaves the user staring at a dead tab.
                Callback::Refused(reason) => return Err(AppError::Auth(reason)),
                Callback::Unrelated(reason) => {
                    eprintln!("[auth listener] ignored request on redirect port {port}: {reason}");
                    continue;
                }
            }
        }
    }
}

fn classify_request_line(line: &str, expected_state: &str) -> Callback {
    let Some(target) = line.split_whitespace().nth(1) else {
        return Callback::Unrelated("Malformed HTTP request".into());
    };

    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != "/login" && path != "/callback" {
        return Callback::Unrelated(format!("Unexpected path {path}"));
    }

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        match k.as_ref() {
            "code" => code = Some(v.into_owned()),
            "state" => state = Some(v.into_owned()),
            "error" => error = Some(v.into_owned()),
            _ => {}
        }
    }

    // A stale redirect from a superseded attempt carries the *old* state. It is
    // not this flow's callback, so keep waiting for the real one rather than
    // failing the sign-in the user is actually in the middle of.
    if state.as_deref() != Some(expected_state) {
        return Callback::Unrelated("State mismatch (stale or forged redirect)".into());
    }

    if let Some(err) = error {
        let message = match err.as_str() {
            "access_denied" => "Sign-in was cancelled in the browser".to_string(),
            other => format!("Spotify refused the sign-in: {other}"),
        };
        return Callback::Refused(message);
    }

    match code {
        Some(code) => Callback::Code(code),
        None => Callback::Refused("Spotify did not return an authorization code".into()),
    }
}
