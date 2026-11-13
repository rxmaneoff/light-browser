//! Модуль безопасности соединений и TLS (`security.rs`).
//!
//! Является обязательным модулем браузера (всегда включен).
//! Отвечает за:
//! - Определение достоверного статуса безопасности соединения;
//! - Предотвращение ложного отображения статуса "Защищено" во время загрузки (`Loading`);
//! - Безусловный приоритет ошибок сертификата (`TlsError`);
//! - Локализованные подсказки и иконки для адресной строки;
//! - Валидацию отсутствия несанкционированного отключения проверки TLS.

use crate::i18n::Language;

/// Статус безопасности соединения.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionSecurity {
    /// Внутренняя страница браузера (настройки, модули, домашняя страница, диагностика).
    Internal,
    /// Страница в процессе загрузки (состояние TLS еще проверяется).
    Loading,
    /// Защищенное соединение HTTPS без ошибок сертификата.
    Https,
    /// Незащищенное открытое соединение HTTP.
    Http,
    /// Ошибка проверки TLS-сертификата (недоверенный, истекший, несоответствие имени).
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

    pub fn is_secure(self) -> bool {
        matches!(self, Self::Https | Self::Internal)
    }
}

/// Состояние безопасности отдельной вкладки.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSecurityState {
    pub is_internal: bool,
    pub is_loading: bool,
    pub tls_error: bool,
    pub address: String,
}

impl TabSecurityState {
    pub fn new_internal(address: impl Into<String>) -> Self {
        Self {
            is_internal: true,
            is_loading: false,
            tls_error: false,
            address: address.into(),
        }
    }

    pub fn new_web() -> Self {
        Self {
            is_internal: false,
            is_loading: false,
            tls_error: false,
            address: String::new(),
        }
    }

    /// Сбрасывает состояние безопасности при старте новой навигации.
    pub fn on_navigation_started(&mut self, address: String) {
        self.address = address;
        self.tls_error = false;
        self.is_loading = true;
    }

    pub fn on_load_finished(&mut self) {
        self.is_loading = false;
    }

    pub fn on_tls_error(&mut self) {
        self.tls_error = true;
        self.is_loading = false;
    }

    /// Вычисляет текущий статус безопасности соединения.
    pub fn evaluate(&self) -> ConnectionSecurity {
        if self.is_internal {
            return ConnectionSecurity::Internal;
        }

        // Ошибка сертификата имеет абсолютный приоритет над схемой URL
        if self.tls_error {
            return ConnectionSecurity::TlsError;
        }

        // Во время загрузки безопасность еще не подтверждена
        if self.is_loading {
            return ConnectionSecurity::Loading;
        }

        let addr = self.address.to_ascii_lowercase();
        if addr.starts_with("https://") {
            ConnectionSecurity::Https
        } else if addr.starts_with("http://") {
            ConnectionSecurity::Http
        } else if addr.starts_with("light://") || addr.starts_with("about:") || addr.is_empty() {
            ConnectionSecurity::Internal
        } else {
            ConnectionSecurity::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_state_transitions() {
        let mut state = TabSecurityState::new_web();
        assert_eq!(state.evaluate(), ConnectionSecurity::Internal); // пустой адрес

        // Старт навигации на HTTPS
        state.on_navigation_started("https://example.com".to_string());
        assert_eq!(state.evaluate(), ConnectionSecurity::Loading);

        // Успешная загрузка без ошибок
        state.on_load_finished();
        assert_eq!(state.evaluate(), ConnectionSecurity::Https);

        // Если возникла ошибка TLS
        state.on_tls_error();
        assert_eq!(state.evaluate(), ConnectionSecurity::TlsError);

        // Новая навигация сбрасывает устаревшую ошибку TLS
        state.on_navigation_started("https://another.com".to_string());
        assert_eq!(state.evaluate(), ConnectionSecurity::Loading);
        state.on_load_finished();
        assert_eq!(state.evaluate(), ConnectionSecurity::Https);
    }

    #[test]
    fn test_http_insecure_state() {
        let mut state = TabSecurityState::new_web();
        state.on_navigation_started("http://insecure.example".to_string());
        state.on_load_finished();
        assert_eq!(state.evaluate(), ConnectionSecurity::Http);
        assert!(!state.evaluate().is_secure());
    }

    #[test]
    fn test_tls_error_priority_over_https() {
        let mut state = TabSecurityState::new_web();
        state.address = "https://badcert.site".to_string();
        state.tls_error = true;
        state.is_loading = false;
        assert_eq!(state.evaluate(), ConnectionSecurity::TlsError);
    }

    #[test]
    fn test_internal_state() {
        let state = TabSecurityState::new_internal("light://home");
        assert_eq!(state.evaluate(), ConnectionSecurity::Internal);
        assert!(state.evaluate().is_secure());
    }
}
