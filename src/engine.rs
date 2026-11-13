use std::fs;
use std::path::PathBuf;

use relm4::gtk::{self, glib, prelude::*};
use webkit6::prelude::*;

use crate::config::ThemeMode;
use crate::i18n::{AddressError, Language, generate_probe_html};

/// Нормализует введенный пользователем адрес.
///
/// Правила:
/// 1. Удаляются пробелы по краям.
/// 2. Пустые строки и строки с внутренними пробелами отклоняются (`EmptyOrWhitespace`).
/// 3. Опасные и локальные схемы (`javascript:`, `file:`, `data:`, `blob:` и др.) отклоняются (`UnsupportedScheme`).
/// 4. Поддерживаются схемы `http://` и `https://` (регистронезависимо).
/// 5. Адрес без схемы дополняется префиксом `https://`.
/// 6. Корректность хоста и структуры проверяется встроенным парсером `glib::Uri`.
pub fn normalize_address(input: &str) -> Result<String, AddressError> {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return Err(AddressError::EmptyOrWhitespace);
    }

    let lower = trimmed.to_ascii_lowercase();

    // Неподдерживаемые или небезопасные схемы
    for scheme_prefix in &[
        "javascript:",
        "file:",
        "data:",
        "blob:",
        "about:",
        "chrome:",
    ] {
        if lower.starts_with(scheme_prefix) {
            return Err(AddressError::UnsupportedScheme);
        }
    }

    let candidate = if lower.starts_with("http://") || lower.starts_with("https://") {
        trimmed.to_string()
    } else if trimmed.contains("://") {
        // Любая другая схема вида something://
        return Err(AddressError::UnsupportedScheme);
    } else {
        // По умолчанию HTTPS
        format!("https://{trimmed}")
    };

    match glib::Uri::parse(&candidate, glib::UriFlags::NONE) {
        Ok(uri) => {
            let scheme = uri.scheme();
            if scheme != "http" && scheme != "https" {
                return Err(AddressError::UnsupportedScheme);
            }
            match uri.host() {
                Some(h) if !h.trim().is_empty() => Ok(candidate),
                _ => Err(AddressError::InvalidHost),
            }
        }
        Err(_) => Err(AddressError::InvalidHost),
    }
}

/// Статус безопасности соединения.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSecurity {
    /// Внутренняя страница браузера (настройки, модули, встроенная проверка).
    Internal,
    /// Страница в процессе загрузки (состояние TLS еще проверяется).
    Loading,
    /// Незащищенное открытое соединение HTTP.
    Http,
    /// Защищенное соединение HTTPS без ошибок сертификата.
    Https,
    /// Ошибка проверки TLS-сертификата.
    TlsError,
    /// Неизвестное состояние соединения.
    Unknown,
}

impl ConnectionSecurity {
    pub fn icon_name(self) -> &'static str {
        match self {
            Self::Internal => "emblem-default-symbolic",
            Self::Loading => "network-wireless-acquiring-symbolic",
            Self::Https => "channel-secure-symbolic",
            Self::Http => "channel-insecure-symbolic",
            Self::TlsError => "dialog-warning-symbolic",
            Self::Unknown => "dialog-question-symbolic",
        }
    }

    pub fn tooltip(self, lang: Language) -> &'static str {
        match self {
            Self::Internal => lang.conn_internal(),
            Self::Loading => lang.conn_loading(),
            Self::Https => lang.conn_https(),
            Self::Http => lang.conn_http(),
            Self::TlsError => lang.conn_tls_error(),
            Self::Unknown => lang.conn_unknown(),
        }
    }
}

/// События движка WebKitGTK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    Started,
    Finished,
    Failed { failing_uri: Option<String> },
    Cancelled { failing_uri: Option<String> },
    TlsError { failing_uri: String },
    ProcessTerminated,
    TitleChanged(String),
    UriChanged(String),
    SubmitInput(String),
}

/// Адаптер движка WebKitGTK.
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

    /// Инициализирует постоянное сетевое хранилище (cookies и кэш) до начала навигации.
    /// Возвращает сконфигурированную `NetworkSession` для привязки к `WebView`.
    pub fn init_persistent_storage() -> Result<webkit6::NetworkSession, String> {
        let data_dir = Self::user_data_dir();
        let cache_dir = Self::user_cache_dir();

        if let Err(e) = fs::create_dir_all(&data_dir) {
            return Err(format!(
                "Не удалось создать каталог данных {}: {}",
                data_dir.display(),
                e
            ));
        }
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!(
                "Предупреждение: не удалось создать каталог кэша {}: {}",
                cache_dir.display(),
                e
            );
        }

        let cookie_file = data_dir.join("cookies.sqlite");
        let cookie_path_str = cookie_file.to_string_lossy();

        let session = match webkit6::NetworkSession::default() {
            Some(s) => s,
            None => {
                let data_str = data_dir.to_string_lossy();
                let cache_str = cache_dir.to_string_lossy();
                webkit6::NetworkSession::new(Some(&data_str), Some(&cache_str))
            }
        };

        if let Some(cookie_manager) = session.cookie_manager() {
            cookie_manager
                .set_persistent_storage(&cookie_path_str, webkit6::CookiePersistentStorage::Sqlite);
        }

        Ok(session)
    }

    /// Создает новый экземпляр `Engine`.
    /// `WebView` создаётся и привязывается к указанной `NetworkSession` в главном потоке GTK.
    pub fn new(
        network_session: Option<&webkit6::NetworkSession>,
        on_event: impl Fn(EngineEvent) + Clone + 'static,
    ) -> Self {
        let content_manager = webkit6::UserContentManager::new();
        if content_manager.register_script_message_handler("lightNavigate", None) {
            let on_home_submit = on_event.clone();
            content_manager.connect_script_message_received(
                Some("lightNavigate"),
                move |_, value| {
                    on_home_submit(EngineEvent::SubmitInput(value.to_str().to_string()));
                },
            );
        } else {
            eprintln!(
                "Предупреждение: не удалось зарегистрировать мост навигации домашней страницы"
            );
        }

        let view = if let Some(session) = network_session {
            webkit6::WebView::builder()
                .network_session(session)
                .user_content_manager(&content_manager)
                .build()
        } else if let Some(def) = webkit6::NetworkSession::default() {
            webkit6::WebView::builder()
                .network_session(&def)
                .user_content_manager(&content_manager)
                .build()
        } else {
            webkit6::WebView::builder()
                .user_content_manager(&content_manager)
                .build()
        };

        let on_load = on_event.clone();
        view.connect_load_changed(move |_, event| match event {
            webkit6::LoadEvent::Started => on_load(EngineEvent::Started),
            webkit6::LoadEvent::Finished => on_load(EngineEvent::Finished),
            _ => {}
        });

        let on_failure = on_event.clone();
        view.connect_load_failed(move |_, _, failing_uri, error| {
            let uri = if failing_uri.is_empty() {
                None
            } else {
                Some(failing_uri.to_string())
            };

            if error.matches(webkit6::NetworkError::Cancelled) {
                on_failure(EngineEvent::Cancelled { failing_uri: uri });
            } else {
                on_failure(EngineEvent::Failed { failing_uri: uri });
            }
            true
        });

        let on_tls_failure = on_event.clone();
        view.connect_load_failed_with_tls_errors(move |_, failing_uri, _, _| {
            on_tls_failure(EngineEvent::TlsError {
                failing_uri: failing_uri.to_string(),
            });
            // Не подавляем ошибку: даем WebKit продолжить обычный путь отказа
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

    pub fn load_blank(&self) {
        self.view.load_html(
            "<!doctype html><html><head><meta charset='utf-8'></head><body></body></html>",
            Some("about:blank"),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn load_home(
        &self,
        lang: Language,
        theme: ThemeMode,
        accent_color: &str,
        is_search_enabled: bool,
        search_provider_name: &str,
        transparency_enabled: bool,
        transparency_value: f64,
    ) {
        let html = crate::i18n::generate_home_html(
            lang,
            theme,
            accent_color,
            is_search_enabled,
            search_provider_name,
            transparency_enabled,
            transparency_value,
        );
        self.view.load_html(&html, Some("light://home"));
    }

    pub fn load_performance(
        &self,
        is_enabled: bool,
        snapshot: Option<&crate::performance::DiagnosticsSnapshot>,
        lang: Language,
        theme: ThemeMode,
        accent_color: &str,
    ) {
        let html = crate::performance::generate_performance_html(
            is_enabled,
            snapshot,
            lang,
            theme,
            accent_color,
        );
        self.view.load_html(&html, Some("light://performance"));
    }

    pub fn navigate(&self, input: &str) -> Result<String, AddressError> {
        let address = normalize_address(input)?;
        self.view.load_uri(&address);
        Ok(address)
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
            normalize_address("example.com"),
            Ok("https://example.com".into())
        );
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
            normalize_address("javascript:alert(1)"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("data:text/html,test"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("blob:https://example.com/123"),
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
        assert_eq!(
            normalize_address("   "),
            Err(AddressError::EmptyOrWhitespace)
        );
    }

    #[test]
    fn test_case_insensitive_scheme() {
        assert_eq!(
            normalize_address("HTTP://EXAMPLE.COM"),
            Ok("HTTP://EXAMPLE.COM".into())
        );
        assert_eq!(
            normalize_address("Https://Example.Com/Path"),
            Ok("Https://Example.Com/Path".into())
        );
    }

    #[test]
    fn test_preserves_query_and_fragment() {
        assert_eq!(
            normalize_address("example.com/search?q=rust&lang=ru#top"),
            Ok("https://example.com/search?q=rust&lang=ru#top".into())
        );
    }

    #[test]
    fn test_ipv4_and_ipv6_addresses() {
        assert_eq!(
            normalize_address("127.0.0.1:8080"),
            Ok("https://127.0.0.1:8080".into())
        );
        assert_eq!(
            normalize_address("http://192.168.1.1/router"),
            Ok("http://192.168.1.1/router".into())
        );
        assert_eq!(
            normalize_address("http://[::1]:8080"),
            Ok("http://[::1]:8080".into())
        );
    }

    #[test]
    fn test_invalid_hosts() {
        assert_eq!(
            normalize_address("https://"),
            Err(AddressError::InvalidHost)
        );
        assert_eq!(normalize_address("http://"), Err(AddressError::InvalidHost));
    }

    #[test]
    fn test_connection_security_icons_and_tooltips() {
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
        assert_eq!(
            ConnectionSecurity::Loading.icon_name(),
            "network-wireless-acquiring-symbolic"
        );
        assert_eq!(
            ConnectionSecurity::Unknown.icon_name(),
            "dialog-question-symbolic"
        );

        assert_eq!(
            ConnectionSecurity::Https.tooltip(Language::Ru),
            "Защищённое соединение (HTTPS)"
        );
        assert_eq!(
            ConnectionSecurity::Loading.tooltip(Language::En),
            "Connecting…"
        );
    }
}
