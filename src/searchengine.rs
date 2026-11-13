//! Модуль поисковых систем и разбора пользовательского ввода (`searchengine.rs`).
//!
//! Отвечает за:
//! - Выбор провайдера (DuckDuckGo, Google, Яндекс);
//! - Различение веб-адресов, внутренних URI (`light://`, `about:`) и поисковых запросов;
//! - Безопасное URL-кодирование текстовых запросов (включая UTF-8 и спецсимволы);
//! - Запрет скрытой отправки произвольного текста в сеть при выключенном модуле.

use relm4::gtk::glib;

use crate::i18n::Language;

/// Доступные провайдеры поиска.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchProvider {
    #[default]
    DuckDuckGo,
    Google,
    Yandex,
}

impl SearchProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DuckDuckGo => "duckduckgo",
            Self::Google => "google",
            Self::Yandex => "yandex",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "duckduckgo" | "ddg" => Some(Self::DuckDuckGo),
            "google" => Some(Self::Google),
            "yandex" | "ya" => Some(Self::Yandex),
            _ => None,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::DuckDuckGo => "DuckDuckGo",
            Self::Google => "Google",
            Self::Yandex => "Яндекс",
        }
    }

    /// Формирует полный поисковый URL для закодированного запроса.
    pub fn build_search_url(self, encoded_query: &str) -> String {
        match self {
            Self::DuckDuckGo => format!("https://duckduckgo.com/?q={encoded_query}"),
            Self::Google => format!("https://www.google.com/search?q={encoded_query}"),
            Self::Yandex => format!("https://ya.ru/search/?text={encoded_query}"),
        }
    }
}

/// Результат разбора ввода пользователя.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationTarget {
    /// Прямой веб-адрес (HTTP/HTTPS).
    Url(String),
    /// Внутренний системный URI (`light://...`, `about:...`).
    Internal(String),
    /// Поисковый запрос через провайдера.
    Search {
        provider: SearchProvider,
        query: String,
        url: String,
    },
}

/// Ошибки разбора ввода.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEngineError {
    EmptyInput,
    /// Модуль поиска выключен, а ввод не является валидным адресом.
    SearchDisabled,
    /// Неподдерживаемая или потенциально опасная схема (`javascript:`, `data:` и т.п.).
    UnsupportedScheme,
    /// Некорректный адрес хоста.
    InvalidHost,
}

impl SearchEngineError {
    pub fn message(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::EmptyInput, Language::Ru) => "Введите адрес или поисковый запрос.",
            (Self::EmptyInput, Language::En) => "Enter an address or search query.",
            (Self::SearchDisabled, Language::Ru) => "Введите адрес сайта или включите поиск.",
            (Self::SearchDisabled, Language::En) => "Enter a website address or enable search.",
            (Self::UnsupportedScheme, Language::Ru) => {
                "Поддерживаются только адреса HTTP, HTTPS и внутренние страницы."
            }
            (Self::UnsupportedScheme, Language::En) => {
                "Only HTTP, HTTPS and internal addresses are supported."
            }
            (Self::InvalidHost, Language::Ru) => "Некорректный адрес хоста.",
            (Self::InvalidHost, Language::En) => "Invalid host address.",
        }
    }
}

/// Выполняет процентное URL-кодирование строки по стандартам RFC 3986.
///
/// Символы ASCII a-z, A-Z, 0-9, `-`, `_`, `.`, `~` сохраняются.
/// Пробелы преобразуются в `%20` (или `+`), остальные байты кодируются `%XX`.
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{b:02X}"));
            }
        }
    }
    encoded
}

/// Конфигурация модуля поисковой системы.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEngineConfig {
    pub enabled: bool,
    pub provider: SearchProvider,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: SearchProvider::DuckDuckGo,
        }
    }
}

/// Разбирает и классифицирует пользовательский ввод в адресной строке.
pub fn parse_input(
    input: &str,
    config: &SearchEngineConfig,
) -> Result<NavigationTarget, SearchEngineError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SearchEngineError::EmptyInput);
    }

    let lower = trimmed.to_ascii_lowercase();

    // 1. Опасные схемы отклоняем сразу
    for dangerous in &["javascript:", "data:", "blob:", "file:"] {
        if lower.starts_with(dangerous) {
            return Err(SearchEngineError::UnsupportedScheme);
        }
    }

    // 2. Внутренние URI приложения
    if lower.starts_with("light://") || lower.starts_with("about:") {
        return Ok(NavigationTarget::Internal(trimmed.to_string()));
    }

    // 3. Явные схемы HTTP / HTTPS
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return validate_web_url(trimmed);
    }

    // Если есть неизвестная схема вида "foo://bar"
    if trimmed.contains("://") {
        return Err(SearchEngineError::UnsupportedScheme);
    }

    // 4. Проверяем, является ли ввод потенциальным сетевым адресом (домен, IP, localhost):
    // Признаки адреса без схемы:
    // - отсутствие пробелов внутри
    // - наличие точки с допустимым TLD/доменом или это localhost[:port] или IPv4/IPv6
    if is_likely_host_address(trimmed) {
        let candidate = if trimmed.starts_with("localhost") || trimmed.starts_with("127.0.0.1") {
            format!("http://{trimmed}")
        } else {
            format!("https://{trimmed}")
        };

        if let Ok(target) = validate_web_url(&candidate) {
            return Ok(target);
        }
    }

    // 5. Ввод не является распознанным адресом: это поисковый запрос
    if !config.enabled {
        // Поиск выключен: строгий запрет отправки в сеть
        return Err(SearchEngineError::SearchDisabled);
    }

    let encoded = url_encode(trimmed);
    let search_url = config.provider.build_search_url(&encoded);

    Ok(NavigationTarget::Search {
        provider: config.provider,
        query: trimmed.to_string(),
        url: search_url,
    })
}

/// Проверяет, похож ли ввод на адрес сайта (а не на поисковый запрос).
fn is_likely_host_address(input: &str) -> bool {
    // В адресах не должно быть пробелов
    if input.chars().any(char::is_whitespace) {
        return false;
    }

    // localhost с портом или без
    if input == "localhost" || input.starts_with("localhost:") {
        return true;
    }

    // Проверка IPv4 (например 127.0.0.1 или 192.168.1.1)
    let host_part = input.split('/').next().unwrap_or(input);
    let host_without_port = host_part.split(':').next().unwrap_or(host_part);

    if host_without_port
        .split('.')
        .all(|part| part.parse::<u8>().is_ok())
        && host_without_port.matches('.').count() == 3
    {
        return true;
    }

    // Проверка доменного имени: должна быть хотя бы одна точка, и часть после последней точки
    // должна состоять из ASCII-букв (TLD) длиной от 2 символов
    if let Some((_, tld_part)) = host_without_port.rsplit_once('.')
        && !tld_part.is_empty()
        && tld_part.chars().all(|c| c.is_ascii_alphabetic())
        && tld_part.len() >= 2
    {
        return true;
    }

    false
}

/// Валидирует HTTP/HTTPS URL через `glib::Uri`.
fn validate_web_url(candidate: &str) -> Result<NavigationTarget, SearchEngineError> {
    match glib::Uri::parse(candidate, glib::UriFlags::NONE) {
        Ok(uri) => {
            let scheme = uri.scheme();
            if scheme != "http" && scheme != "https" {
                return Err(SearchEngineError::UnsupportedScheme);
            }
            match uri.host() {
                Some(h) if !h.trim().is_empty() => Ok(NavigationTarget::Url(candidate.to_string())),
                _ => Err(SearchEngineError::InvalidHost),
            }
        }
        Err(_) => Err(SearchEngineError::InvalidHost),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_provider_serialization() {
        assert_eq!(SearchProvider::DuckDuckGo.as_str(), "duckduckgo");
        assert_eq!(SearchProvider::Google.as_str(), "google");
        assert_eq!(SearchProvider::Yandex.as_str(), "yandex");

        assert_eq!(
            SearchProvider::from_str("duckduckgo"),
            Some(SearchProvider::DuckDuckGo)
        );
        assert_eq!(
            SearchProvider::from_str("google"),
            Some(SearchProvider::Google)
        );
        assert_eq!(
            SearchProvider::from_str("yandex"),
            Some(SearchProvider::Yandex)
        );
        assert_eq!(SearchProvider::from_str("unknown"), None);
    }

    #[test]
    fn test_url_encode_utf8_and_symbols() {
        assert_eq!(url_encode("hello world"), "hello+world");
        assert_eq!(url_encode("rust & gtk4?"), "rust+%26+gtk4%3F");
        // Кириллица
        let encoded_ru = url_encode("привет");
        assert!(encoded_ru.contains("%D0") || encoded_ru.contains("%D1"));
    }

    #[test]
    fn test_parse_explicit_urls() {
        let cfg = SearchEngineConfig::default();

        let res = parse_input("https://example.com/path?arg=1", &cfg).unwrap();
        assert_eq!(
            res,
            NavigationTarget::Url("https://example.com/path?arg=1".to_string())
        );

        let res_http = parse_input("http://insecure.site", &cfg).unwrap();
        assert_eq!(
            res_http,
            NavigationTarget::Url("http://insecure.site".to_string())
        );
    }

    #[test]
    fn test_parse_domain_without_scheme() {
        let cfg = SearchEngineConfig::default();

        let res = parse_input("example.com", &cfg).unwrap();
        assert_eq!(
            res,
            NavigationTarget::Url("https://example.com".to_string())
        );

        let res_sub = parse_input("sub.domain.co.uk/test", &cfg).unwrap();
        assert_eq!(
            res_sub,
            NavigationTarget::Url("https://sub.domain.co.uk/test".to_string())
        );

        let res_local = parse_input("localhost:8080", &cfg).unwrap();
        assert_eq!(
            res_local,
            NavigationTarget::Url("http://localhost:8080".to_string())
        );

        let res_ip = parse_input("127.0.0.1:3000", &cfg).unwrap();
        assert_eq!(
            res_ip,
            NavigationTarget::Url("http://127.0.0.1:3000".to_string())
        );
    }

    #[test]
    fn test_parse_internal_uris() {
        let cfg = SearchEngineConfig::default();

        let res = parse_input("light://home", &cfg).unwrap();
        assert_eq!(res, NavigationTarget::Internal("light://home".to_string()));

        let res_perf = parse_input("light://performance", &cfg).unwrap();
        assert_eq!(
            res_perf,
            NavigationTarget::Internal("light://performance".to_string())
        );

        let res_about = parse_input("about:settings", &cfg).unwrap();
        assert_eq!(
            res_about,
            NavigationTarget::Internal("about:settings".to_string())
        );
    }

    #[test]
    fn test_parse_search_queries_enabled() {
        let mut cfg = SearchEngineConfig {
            enabled: true,
            provider: SearchProvider::DuckDuckGo,
        };

        let res_ddg = parse_input("rust language", &cfg).unwrap();
        match res_ddg {
            NavigationTarget::Search { provider, url, .. } => {
                assert_eq!(provider, SearchProvider::DuckDuckGo);
                assert!(url.starts_with("https://duckduckgo.com/?q=rust+language"));
            }
            _ => panic!("Expected Search target"),
        }

        cfg.provider = SearchProvider::Google;
        let res_google = parse_input("gtk4 relm4", &cfg).unwrap();
        match res_google {
            NavigationTarget::Search { provider, url, .. } => {
                assert_eq!(provider, SearchProvider::Google);
                assert!(url.starts_with("https://www.google.com/search?q=gtk4+relm4"));
            }
            _ => panic!("Expected Search target"),
        }

        cfg.provider = SearchProvider::Yandex;
        let res_ya = parse_input("погода москва", &cfg).unwrap();
        match res_ya {
            NavigationTarget::Search { provider, url, .. } => {
                assert_eq!(provider, SearchProvider::Yandex);
                assert!(url.starts_with("https://ya.ru/search/?text="));
            }
            _ => panic!("Expected Search target"),
        }
    }

    #[test]
    fn test_search_disabled_rejects_plain_text() {
        let cfg = SearchEngineConfig {
            enabled: false,
            provider: SearchProvider::DuckDuckGo,
        };

        // Текстовый поисковый запрос отклоняется
        let err = parse_input("rust language tutorial", &cfg).unwrap_err();
        assert_eq!(err, SearchEngineError::SearchDisabled);

        // При этом валидный адрес открывается без проблем!
        let ok_url = parse_input("https://rust-lang.org", &cfg).unwrap();
        assert_eq!(
            ok_url,
            NavigationTarget::Url("https://rust-lang.org".to_string())
        );

        let ok_domain = parse_input("rust-lang.org", &cfg).unwrap();
        assert_eq!(
            ok_domain,
            NavigationTarget::Url("https://rust-lang.org".to_string())
        );
    }

    #[test]
    fn test_rejects_dangerous_schemes_and_empty() {
        let cfg = SearchEngineConfig::default();

        assert_eq!(
            parse_input("", &cfg).unwrap_err(),
            SearchEngineError::EmptyInput
        );
        assert_eq!(
            parse_input("   ", &cfg).unwrap_err(),
            SearchEngineError::EmptyInput
        );
        assert_eq!(
            parse_input("javascript:alert(1)", &cfg).unwrap_err(),
            SearchEngineError::UnsupportedScheme
        );
        assert_eq!(
            parse_input("data:text/html,<h1>Hi</h1>", &cfg).unwrap_err(),
            SearchEngineError::UnsupportedScheme
        );
    }
}
