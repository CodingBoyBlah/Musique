use crate::errors::AppError;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub const CALLBACK_PORT: u16 = 8989;
pub const PLAYBACK_PORT: u16 = 8898;
const LOGIN_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes

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

pub async fn wait_for_callback(expected_state: &str) -> Result<String, AppError> {
    wait_for_callback_port(CALLBACK_PORT, expected_state).await
}

pub async fn wait_for_callback_port(port: u16, expected_state: &str) -> Result<String, AppError> {
    let address: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = TcpListener::bind(address)
        .await
        .map_err(|e| AppError::Auth(format!("Unable to listen on 127.0.0.1:{port} for Spotify redirect: {e}")))?;

    let deadline = tokio::time::sleep(LOGIN_TIMEOUT);
    tokio::pin!(deadline);

    loop {
        let (mut stream, _) = tokio::select! {
            accepted = listener.accept() => accepted.map_err(|e| AppError::Auth(format!("Redirect listener accept failed: {e}")))?,
            _ = &mut deadline => return Err(AppError::Auth("Sign-in timed out; please try again".into())),
        };

        let mut reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).await.is_err() {
            continue;
        }

        let outcome = parse_request_line(&request_line, expected_state);
        let (status, body) = match &outcome {
            Ok(_) => ("200 OK", success_page()),
            Err(error) => ("400 Bad Request", failure_page(&error.to_string())),
        };

        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.shutdown().await;

        match outcome {
            Ok(code) => return Ok(code),
            Err(error) => {
                // An ignored request (e.g. favicon.ico or pre-connection) - keep waiting in loop!
                eprintln!("[auth listener] ignored request on redirect port {port}: {error}");
                continue;
            }
        }
    }
}

fn parse_request_line(line: &str, expected_state: &str) -> Result<String, AppError> {
    let target = line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AppError::Auth("Malformed HTTP request".into()))?;

    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if path != "/login" && path != "/callback" {
        return Err(AppError::Auth(format!("Unexpected path {path}")));
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

    if let Some(err) = error {
        return Err(AppError::Auth(format!("Spotify refused the sign-in: {err}")));
    }

    if state.as_deref() != Some(expected_state) {
        return Err(AppError::Auth("State mismatch / possible CSRF".into()));
    }

    code.ok_or_else(|| AppError::Auth("Spotify did not return an authorization code".into()))
}
