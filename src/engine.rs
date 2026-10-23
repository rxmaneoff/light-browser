use std::fs;
use std::path::PathBuf;

use relm4::gtk::{self, glib, prelude::*};
use webkit6::prelude::*;

use crate::config::ThemeMode;
use crate::i18n::{AddressError, Language, generate_probe_html};

pub fn normalize_address(input: &str) -> Result<String, AddressError> {
    let input = input.trim();
    if input.is_empty() || input.chars().any(char::is_whitespace) {
        return Err(AddressError::EmptyOrWhitespace);
    }

    if input.starts_with("http://") || input.starts_with("https://") {
        Ok(input.to_owned())
    } else if input.contains("://") {
        Err(AddressError::UnsupportedScheme)
    } else {
        Ok(format!("https://{input}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSecurity {
    Internal,
    Http,
    Https,
    TlsError,
    Unknown,
}

impl ConnectionSecurity {
    pub fn icon_name(self) -> &'static str {
        match self {
            Self::Internal => "emblem-default-symbolic",
            Self::Https => "channel-secure-symbolic",
            Self::Http => "channel-insecure-symbolic",
            Self::TlsError => "dialog-warning-symbolic",
            Self::Unknown => "network-wireless-acquiring-symbolic",
        }
    }

    pub fn tooltip(self, lang: Language) -> &'static str {
        match self {
            Self::Internal => lang.conn_internal(),
            Self::Https => lang.conn_https(),
            Self::Http => lang.conn_http(),
            Self::TlsError => lang.conn_tls_error(),
            Self::Unknown => lang.conn_unknown(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Started,
    Finished,
    Failed,
    ProcessTerminated,
    TitleChanged(String),
    UriChanged(String),
    TlsError,
}

pub struct Engine {
    view: webkit6::WebView,
}

impl Engine {
    pub fn user_data_dir() -> PathBuf {
        glib::user_data_dir().join("light-project")
    }

    #[allow(dead_code)]
    pub fn user_cache_dir() -> PathBuf {
        glib::user_cache_dir().join("light-project")
    }

    pub fn init_persistent_storage() {
        let data_dir = Self::user_data_dir();

        if let Err(e) = fs::create_dir_all(&data_dir) {
            eprintln!("Не удалось создать каталог пользовательских данных: {}", e);
            return;
        }

        let cookie_file = data_dir.join("cookies.sqlite");
        let cookie_path_str = cookie_file.to_string_lossy();

        if let Some(cookie_manager) =
            webkit6::NetworkSession::default().and_then(|s| s.cookie_manager())
        {
            cookie_manager
                .set_persistent_storage(&cookie_path_str, webkit6::CookiePersistentStorage::Sqlite);
        }
    }

    // WebView создаётся и используется только в главном потоке GTK.
    pub fn new(on_event: impl Fn(EngineEvent) + Clone + 'static) -> Self {
        let view = webkit6::WebView::new();

        let on_load = on_event.clone();
        view.connect_load_changed(move |_, event| match event {
            webkit6::LoadEvent::Started => on_load(EngineEvent::Started),
            webkit6::LoadEvent::Finished => on_load(EngineEvent::Finished),
            _ => {}
        });

        let on_failure = on_event.clone();
        view.connect_load_failed(move |_, _, _, _| {
            on_failure(EngineEvent::Failed);
            true
        });

        let on_tls_failure = on_event.clone();
        view.connect_load_failed_with_tls_errors(move |_, _, _, _| {
            on_tls_failure(EngineEvent::TlsError);
            false
        });

        let on_proc = on_event.clone();
        view.connect_web_process_terminated(move |_, _| {
            on_proc(EngineEvent::ProcessTerminated);
        });

        let on_title = on_event.clone();
        view.connect_title_notify(move |v| {
            if let Some(title) = v.title() {
                on_title(EngineEvent::TitleChanged(title.to_string()));
            }
        });

        let on_uri = on_event;
        view.connect_uri_notify(move |v| {
            if let Some(uri) = v.uri() {
                on_uri(EngineEvent::UriChanged(uri.to_string()));
            }
        });

        Self { view }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.view.clone().upcast()
    }

    #[allow(dead_code)]
    pub fn title(&self) -> Option<String> {
        self.view.title().map(|s| s.to_string())
    }

    #[allow(dead_code)]
    pub fn uri(&self) -> Option<String> {
        self.view.uri().map(|s| s.to_string())
    }

    pub fn load_probe(&self, lang: Language, theme: ThemeMode, accent_color: &str) {
        let html = generate_probe_html(lang, theme, accent_color);
        self.view.load_html(&html, Some("about:blank"));
    }

    pub fn navigate(&self, input: &str) -> Result<(), AddressError> {
        let address = normalize_address(input)?;
        self.view.load_uri(&address);
        Ok(())
    }

    pub fn can_go_back(&self) -> bool {
        self.view.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.can_go_forward()
    }

    pub fn go_back(&self) {
        if self.can_go_back() {
            self.view.go_back();
        }
    }

    pub fn go_forward(&self) {
        if self.can_go_forward() {
            self.view.go_forward();
        }
    }

    pub fn reload(&self) {
        self.view.reload();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_http_and_https_addresses() {
        assert_eq!(
            normalize_address("http://example.com"),
            Ok("http://example.com".into())
        );
        assert_eq!(
            normalize_address("https://example.com"),
            Ok("https://example.com".into())
        );
    }

    #[test]
    fn defaults_bare_host_to_https() {
        assert_eq!(
            normalize_address("example.com/path"),
            Ok("https://example.com/path".into())
        );
    }

    #[test]
    fn rejects_non_web_schemes_and_whitespace() {
        assert_eq!(
            normalize_address("file:///etc/passwd"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("custom://example.com"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("https://example.com bad"),
            Err(AddressError::EmptyOrWhitespace)
        );
    }

    #[test]
    fn test_connection_security_icons() {
        assert_eq!(
            ConnectionSecurity::Https.icon_name(),
            "channel-secure-symbolic"
        );
        assert_eq!(
            ConnectionSecurity::Http.icon_name(),
            "channel-insecure-symbolic"
        );
        assert_eq!(
            ConnectionSecurity::TlsError.icon_name(),
            "dialog-warning-symbolic"
        );
        assert_eq!(
            ConnectionSecurity::Internal.icon_name(),
            "emblem-default-symbolic"
        );
    }
}
