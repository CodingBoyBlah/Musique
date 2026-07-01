use crate::errors::AppError;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub const CALLBACK_PORT: u16 = 8888;

const SUCCESS_HTML: &str = concat!(
    "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Login Successful</title></head>",
    "<body style='font-family:system-ui,sans-serif;max-width:420px;margin:80px auto;",
    "text-align:center;background:#121212;color:#fff;padding:2rem;border-radius:12px'>",
    "<h2 style='color:#1db954'>Login Successful</h2>",
    "<p style='color:#b3b3b3'>You can close this tab and return to Spotify Client.</p>",
    "</body></html>",
);

const ERROR_HTML: &str = concat!(
    "<!DOCTYPE html><html><head><meta charset='utf-8'><title>Login Failed</title></head>",
    "<body style='font-family:system-ui,sans-serif;max-width:420px;margin:80px auto;",
    "text-align:center;background:#121212;color:#fff;padding:2rem;border-radius:12px'>",
    "<h2 style='color:#ef4444'>Login Failed</h2>",
    "<p style='color:#b3b3b3'>Something went wrong. Please try again.</p>",
    "</body></html>",
);

pub async fn wait_for_callback(expected_state: &str) -> Result<String, AppError> {
    let listener = TcpListener::bind(format!("127.0.0.1:{CALLBACK_PORT}"))
        .await
        .map_err(|e| AppError::Auth(format!("Cannot bind port {CALLBACK_PORT}: {e}")))?;

    tokio::time::timeout(
        std::time::Duration::from_secs(300),
        accept_one(&listener, expected_state),
    )
    .await
    .map_err(|_| AppError::Auth("Login timed out (5 minutes)".into()))?
}

async fn accept_one(listener: &TcpListener, expected_state: &str) -> Result<String, AppError> {
    let (mut socket, _) = listener
        .accept()
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;

    let mut buf = vec![0u8; 8192];
    let n = socket
        .read(&mut buf)
        .await
        .map_err(|e| AppError::Auth(e.to_string()))?;

    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or("");

    let (html, result) = parse_callback(first_line, expected_state);

    let body = html.as_bytes();
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = socket.write_all(headers.as_bytes()).await;
    let _ = socket.write_all(body).await;

    result
}

fn parse_callback(
    request_line: &str,
    expected_state: &str,
) -> (&'static str, Result<String, AppError>) {
    // looks like "GET /callback?code=...&state=... HTTP/1.1" so we yank the bits out
    let path = request_line
        .strip_prefix("GET ")
        .and_then(|s| s.split(" HTTP/").next())
        .unwrap_or("");

    let query = path.find('?').map(|i| &path[i + 1..]).unwrap_or("");

    let mut code  = None;
    let mut state = None;
    let mut error = None;

    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        match k.as_ref() {
            "code"  => code  = Some(v.into_owned()),
            "state" => state = Some(v.into_owned()),
            "error" => error = Some(v.into_owned()),
            _ => {}
        }
    }

    if let Some(err) = error {
        return (ERROR_HTML, Err(AppError::Auth(format!("Spotify denied: {err}"))));
    }

    match (code, state) {
        (Some(c), Some(s)) if s == expected_state => (SUCCESS_HTML, Ok(c)),
        (Some(_), Some(_)) => (
            ERROR_HTML,
            Err(AppError::Auth("State mismatch — possible CSRF".into())),
        ),
        _ => (
            ERROR_HTML,
            Err(AppError::Auth("No authorization code in callback".into())),
        ),
    }
}
