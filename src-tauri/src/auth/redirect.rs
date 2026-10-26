// ncspot registers /ncspot_login; custom apps keep the /login URI documented
// in Settings. Use the same URI for authorization and the code exchange.
pub const SHARED_CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";

pub fn login_redirect_uri(client_id: &str, port: u16) -> String {
    let path = if client_id == SHARED_CLIENT_ID {
        "/ncspot_login"
    } else {
        "/login"
    };
    format!("http://127.0.0.1:{port}{path}")
}

pub fn is_callback_path(path: &str) -> bool {
    matches!(path, "/ncspot_login" | "/login" | "/callback")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_login_uses_ncspots_registered_path() {
        assert_eq!(
            login_redirect_uri(SHARED_CLIENT_ID, 8989),
            "http://127.0.0.1:8989/ncspot_login"
        );
    }

    #[test]
    fn custom_and_desktop_clients_keep_their_registered_path() {
        for (client_id, port) in [
            ("custom-client", 8989),
            ("65b708073fc0480ea92a077233ca87bd", 8898),
        ] {
            assert_eq!(
                login_redirect_uri(client_id, port),
                format!("http://127.0.0.1:{port}/login")
            );
        }
    }

    #[test]
    fn every_generated_redirect_is_accepted_by_the_listener() {
        for client in [SHARED_CLIENT_ID, "custom-client", "65b708073fc0480ea92a077233ca87bd"] {
            let uri = login_redirect_uri(client, 8989);
            let path = uri.strip_prefix("http://127.0.0.1:8989").unwrap();
            assert!(is_callback_path(path), "Listener must accept {uri}");
        }
        assert!(is_callback_path("/callback"));
        for path in ["/", "/favicon.ico", "/ncspot_login/", "/ncspot_login-extra", "/login/extra"] {
            assert!(!is_callback_path(path), "Unexpected callback path: {path}");
        }
    }
}
