use crate::config::ThemeMode;

pub fn generate_probe_html(lang: Language, theme: ThemeMode, accent_color: &str) -> String {
    let (bg_color, text_color, card_bg, border_color, secondary_text) = match theme {
        ThemeMode::Light => ("#f6f8fa", "#1f2328", "#ffffff", "#d0d7de", "#656d76"),
        ThemeMode::Dark => ("#1c1c1f", "#f0f2f5", "#28282d", "#3c3c43", "#98989f"),
    };

    let (title, heading, desc_1, desc_2, badge) = match lang {
        Language::Ru => (
            "Light — проверка WebKitGTK",
            "Light Browser",
            "Эта страница встроена в приложение и не требует сети.",
            "Выделите текст и измените размер окна, чтобы проверить отображение.",
            "Встроенная страница",
        ),
        Language::En => (
            "Light — WebKitGTK Probe",
            "Light Browser",
            "This page is built into the application and requires no network.",
            "Select text and resize the window to verify rendering.",
            "Offline page",
        ),
    };

    format!(
        r#"<!doctype html>
<html lang="{lang_code}">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>{title}</title>
<style>
  :root {{
    --bg-color: {bg_color};
    --text-color: {text_color};
    --card-bg: {card_bg};
    --border-color: {border_color};
    --secondary-text: {secondary_text};
    --accent-color: {accent_color};
  }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: var(--bg-color);
    color: var(--text-color);
    margin: 0;
    padding: 3rem 1.5rem;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 80vh;
    box-sizing: border-box;
    transition: background-color 0.2s ease, color 0.2s ease;
  }}
  .card {{
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 2.5rem;
    max-width: 580px;
    width: 100%;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
  }}
  .badge {{
    display: inline-block;
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    background-color: var(--accent-color);
    color: #ffffff;
    margin-bottom: 1rem;
  }}
  h1 {{
    margin: 0 0 1rem 0;
    font-size: 28px;
    letter-spacing: -0.5px;
  }}
  p {{
    font-size: 16px;
    line-height: 1.6;
    margin: 0 0 0.75rem 0;
    color: var(--secondary-text);
  }}
  .highlight {{
    color: var(--text-color);
    font-weight: 500;
  }}
</style>
</head>
<body>
  <div class="card">
    <div class="badge">{badge}</div>
    <h1>{heading}</h1>
    <p class="highlight">{desc_1}</p>
    <p>{desc_2}</p>
  </div>
</body>
</html>"#,
        lang_code = lang.code(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Ru,
    En,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::Ru => "ru",
            Self::En => "en",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "ru" => Some(Self::Ru),
            "en" => Some(Self::En),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn toggle(self) -> Self {
        match self {
            Self::Ru => Self::En,
            Self::En => Self::Ru,
        }
    }

    #[allow(dead_code)]
    pub fn settings_label(self) -> &'static str {
        match self {
            Self::Ru => "Настройки",
            Self::En => "Settings",
        }
    }

    pub fn settings_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Настройки браузера",
            Self::En => "Browser settings",
        }
    }

    pub fn language_section(self) -> &'static str {
        match self {
            Self::Ru => "Язык интерфейса",
            Self::En => "Interface language",
        }
    }

    pub fn theme_section(self) -> &'static str {
        match self {
            Self::Ru => "Тема оформления",
            Self::En => "Appearance theme",
        }
    }

    pub fn theme_light(self) -> &'static str {
        match self {
            Self::Ru => "Светлая",
            Self::En => "Light",
        }
    }

    pub fn theme_dark(self) -> &'static str {
        match self {
            Self::Ru => "Тёмная",
            Self::En => "Dark",
        }
    }

    pub fn accent_section(self) -> &'static str {
        match self {
            Self::Ru => "Акцентный цвет",
            Self::En => "Accent color",
        }
    }

    pub fn preset_blue_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Синий (#2563eb)",
            Self::En => "Blue (#2563eb)",
        }
    }

    pub fn preset_purple_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Фиолетовый (#7c3aed)",
            Self::En => "Purple (#7c3aed)",
        }
    }

    pub fn preset_emerald_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Изумрудный (#059669)",
            Self::En => "Emerald (#059669)",
        }
    }

    pub fn preset_amber_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Янтарный (#d97706)",
            Self::En => "Amber (#d97706)",
        }
    }

    pub fn preset_rose_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Розовый (#e11d48)",
            Self::En => "Rose (#e11d48)",
        }
    }

    pub fn custom_color_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Выбрать свой цвет",
            Self::En => "Custom color",
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

    pub fn conn_internal(self) -> &'static str {
        match self {
            Self::Ru => "Внутренняя страница",
            Self::En => "Internal page",
        }
    }

    pub fn conn_https(self) -> &'static str {
        match self {
            Self::Ru => "Защищённое соединение (HTTPS)",
            Self::En => "Secure connection (HTTPS)",
        }
    }

    pub fn conn_http(self) -> &'static str {
        match self {
            Self::Ru => "Незащищённое соединение (HTTP)",
            Self::En => "Insecure connection (HTTP)",
        }
    }

    pub fn conn_tls_error(self) -> &'static str {
        match self {
            Self::Ru => "Ошибка сертификата TLS",
            Self::En => "TLS certificate error",
        }
    }

    pub fn conn_unknown(self) -> &'static str {
        match self {
            Self::Ru => "Соединение",
            Self::En => "Connection",
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

    #[test]
    fn test_generate_probe_html() {
        let html_ru = generate_probe_html(Language::Ru, ThemeMode::Dark, "#3584e4");
        assert!(html_ru.contains("Light Browser"));
        assert!(html_ru.contains("Встроенная страница"));
        assert!(html_ru.contains("#1c1c1f"));

        let html_en = generate_probe_html(Language::En, ThemeMode::Light, "#e01b24");
        assert!(html_en.contains("Offline page"));
        assert!(html_en.contains("#f6f8fa"));
    }
}
