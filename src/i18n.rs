pub const PROBE_HTML_RU: &str = r#"<!doctype html>
<html lang="ru">
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>Light — проверка WebKitGTK</title>
<style>body { font: 20px sans-serif; margin: 3em; }</style>
<h1>Light Project</h1>
<p>Эта страница встроена в приложение и не требует сети.</p>
<p>Выделите текст и измените размер окна, чтобы проверить отображение.</p>
</html>"#;

pub const PROBE_HTML_EN: &str = r#"<!doctype html>
<html lang="en">
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>Light — WebKitGTK probe</title>
<style>body { font: 20px sans-serif; margin: 3em; }</style>
<h1>Light Project</h1>
<p>This page is built into the application and requires no network.</p>
<p>Select text and resize the window to verify rendering.</p>
</html>"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ru,
    En,
}

impl Language {
    pub fn toggle(self) -> Self {
        match self {
            Self::Ru => Self::En,
            Self::En => Self::Ru,
        }
    }

    pub fn button_label(self) -> &'static str {
        match self {
            Self::Ru => "⚙ RU",
            Self::En => "⚙ EN",
        }
    }

    pub fn settings_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Настройки: переключить интерфейс на английский язык",
            Self::En => "Settings: switch interface to Russian language",
        }
    }

    pub fn back_label(self) -> &'static str {
        match self {
            Self::Ru => "Назад",
            Self::En => "Back",
        }
    }

    pub fn forward_label(self) -> &'static str {
        match self {
            Self::Ru => "Вперёд",
            Self::En => "Forward",
        }
    }

    pub fn reload_label(self) -> &'static str {
        match self {
            Self::Ru => "Обновить",
            Self::En => "Reload",
        }
    }

    pub fn address_placeholder(self) -> &'static str {
        match self {
            Self::Ru => "Введите адрес сайта",
            Self::En => "Enter website address",
        }
    }

    pub fn new_tab_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Новая вкладка",
            Self::En => "New tab",
        }
    }

    pub fn close_tab_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Закрыть вкладку",
            Self::En => "Close tab",
        }
    }

    pub fn default_tab_title(self) -> &'static str {
        match self {
            Self::Ru => "Новая вкладка",
            Self::En => "New tab",
        }
    }

    pub fn status_waiting(self) -> &'static str {
        match self {
            Self::Ru => "Ожидание загрузки WebKitGTK…",
            Self::En => "Waiting for WebKitGTK…",
        }
    }

    pub fn status_navigating(self) -> &'static str {
        match self {
            Self::Ru => "Загрузка адреса…",
            Self::En => "Loading address…",
        }
    }

    pub fn status_loading(self) -> &'static str {
        match self {
            Self::Ru => "Загрузка страницы…",
            Self::En => "Loading page…",
        }
    }

    pub fn status_finished(self) -> &'static str {
        match self {
            Self::Ru => "Загрузка завершена.",
            Self::En => "Loading completed.",
        }
    }

    pub fn status_failed(self) -> &'static str {
        match self {
            Self::Ru => "Не удалось загрузить страницу. Проверьте адрес или повторите загрузку.",
            Self::En => "Failed to load page. Check the address or try again.",
        }
    }

    pub fn status_process_terminated(self) -> &'static str {
        match self {
            Self::Ru => "Процесс WebKitGTK завершился. Повторите загрузку.",
            Self::En => "WebKitGTK process terminated. Please reload.",
        }
    }

    pub fn probe_html(self) -> &'static str {
        match self {
            Self::Ru => PROBE_HTML_RU,
            Self::En => PROBE_HTML_EN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    EmptyOrWhitespace,
    UnsupportedScheme,
}

impl AddressError {
    pub fn message(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::EmptyOrWhitespace, Language::Ru) => "Введите корректный адрес без пробелов.",
            (Self::EmptyOrWhitespace, Language::En) => "Enter a valid address without spaces.",
            (Self::UnsupportedScheme, Language::Ru) => "Поддерживаются только адреса HTTP и HTTPS.",
            (Self::UnsupportedScheme, Language::En) => {
                "Only HTTP and HTTPS addresses are supported."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_toggle() {
        assert_eq!(Language::Ru.toggle(), Language::En);
        assert_eq!(Language::En.toggle(), Language::Ru);
    }

    #[test]
    fn test_localized_messages() {
        assert_eq!(
            AddressError::UnsupportedScheme.message(Language::Ru),
            "Поддерживаются только адреса HTTP и HTTPS."
        );
        assert_eq!(
            AddressError::UnsupportedScheme.message(Language::En),
            "Only HTTP and HTTPS addresses are supported."
        );
    }
}
