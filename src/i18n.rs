use crate::config::ThemeMode;

/// Генерирует HTML для новой домашней страницы `light://home`.
pub fn generate_home_html(
    lang: Language,
    theme: ThemeMode,
    accent_color: &str,
    is_search_enabled: bool,
    search_provider_name: &str,
    transparency_enabled: bool,
    transparency_value: f64,
) -> String {
    let (bg_color, text_color, card_bg, border_color, secondary_text, input_bg) = match theme {
        ThemeMode::Light => (
            "#f6f8fa",
            "#1f2328",
            "rgba(255, 255, 255, 0.90)",
            "#d0d7de",
            "#656d76",
            "#ffffff",
        ),
        ThemeMode::Dark => (
            "#1c1c1f",
            "#f0f2f5",
            "rgba(40, 40, 45, 0.85)",
            "#3c3c43",
            "#98989f",
            "#242428",
        ),
    };

    let body_bg = if transparency_enabled {
        let alpha = transparency_value.clamp(0.0, 1.0);
        let (r, g, b) = match theme {
            ThemeMode::Light => (246, 248, 250),
            ThemeMode::Dark => (28, 28, 31),
        };
        format!("rgba({r}, {g}, {b}, {alpha:.2})")
    } else {
        bg_color.to_string()
    };

    let (title, wordmark, subtitle, placeholder_text) = match lang {
        Language::Ru => (
            "Light — Новая вкладка",
            "Light",
            "Быстрый и чистый веб-браузер",
            if is_search_enabled {
                format!("Поиск в {search_provider_name} или ввод адреса сайта…")
            } else {
                "Введите адрес веб-сайта…".to_string()
            },
        ),
        Language::En => (
            "Light — New Tab",
            "Light",
            "Fast and minimal web browser",
            if is_search_enabled {
                format!("Search with {search_provider_name} or enter web address…")
            } else {
                "Enter a website address…".to_string()
            },
        ),
    };

    format!(
        r#"<!doctype html>
<html lang="{lang_code}">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'">
<title>{title}</title>
<style>
  :root {{
    --bg-color: {body_bg};
    --text-color: {text_color};
    --card-bg: {card_bg};
    --border-color: {border_color};
    --secondary-text: {secondary_text};
    --accent-color: {accent_color};
    --input-bg: {input_bg};
  }}
  html, body {{
    height: 100%;
    margin: 0;
    padding: 0;
  }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: var(--bg-color);
    color: var(--text-color);
    display: flex;
    justify-content: center;
    align-items: center;
    box-sizing: border-box;
    transition: background-color 0.15s ease, color 0.15s ease;
  }}
  .home-container {{
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 640px;
    padding: 2rem 1.5rem;
    box-sizing: border-box;
    text-align: center;
  }}
  .wordmark {{
    font-size: 42px;
    font-weight: 700;
    letter-spacing: -1px;
    margin-bottom: 6px;
    color: var(--text-color);
  }}
  .subtitle {{
    font-size: 14px;
    color: var(--secondary-text);
    margin-bottom: 28px;
  }}
  .search-box {{
    width: 100%;
    position: relative;
    background-color: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 6px 14px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
    display: flex;
    align-items: center;
    box-sizing: border-box;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }}
  .search-box:focus-within {{
    border-color: var(--accent-color);
    box-shadow: 0 0 0 2px var(--accent-color);
  }}
  .search-input {{
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    font-size: 15px;
    color: var(--text-color);
    padding: 8px 4px;
  }}
  .search-input::placeholder {{
    color: var(--secondary-text);
    opacity: 0.8;
  }}
  .search-icon {{
    margin-right: 10px;
    color: var(--secondary-text);
    font-size: 16px;
  }}
  .hint {{
    margin-top: 18px;
    font-size: 12px;
    color: var(--secondary-text);
  }}
</style>
</head>
<body>
  <div class="home-container">
    <div class="wordmark">{wordmark}</div>
    <div class="subtitle">{subtitle}</div>
    <form class="search-box" onsubmit="window.webkit.messageHandlers.lightNavigate.postMessage(document.getElementById('q').value); return false;">
      <span class="search-icon">🔍</span>
      <input type="text" id="q" name="q" class="search-input" placeholder="{placeholder_text}" autofocus autocomplete="off">
    </form>
    <div class="hint">Нажмите Enter для перехода или поиска</div>
  </div>
</body>
</html>"#,
        lang_code = lang.code(),
    )
}

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

    #[allow(dead_code)]
    pub fn conn_loading(self) -> &'static str {
        match self {
            Self::Ru => "Подключение…",
            Self::En => "Connecting…",
        }
    }

    #[allow(dead_code)]
    pub fn settings_tab_title(self) -> &'static str {
        match self {
            Self::Ru => "Настройки",
            Self::En => "Settings",
        }
    }

    pub fn modules_tab_title(self) -> &'static str {
        match self {
            Self::Ru => "Модули",
            Self::En => "Modules",
        }
    }

    pub fn modules_beta_label(self) -> &'static str {
        match self {
            Self::Ru => "Beta",
            Self::En => "Beta",
        }
    }

    pub fn module_enabled(self) -> &'static str {
        match self {
            Self::Ru => "Включён",
            Self::En => "Enabled",
        }
    }

    pub fn module_disabled(self) -> &'static str {
        match self {
            Self::Ru => "Выключен",
            Self::En => "Disabled",
        }
    }

    pub fn module_transparency_blur_name(self) -> &'static str {
        match self {
            Self::Ru => "Прозрачность",
            Self::En => "Transparency",
        }
    }

    pub fn module_transparency_blur_desc(self) -> &'static str {
        match self {
            Self::Ru => {
                "Настройка прозрачности окна приложения. Доступность зависит от оконного композитора."
            }
            Self::En => {
                "Window transparency customization. Availability depends on the window compositor."
            }
        }
    }

    pub fn transparency_label(self) -> &'static str {
        match self {
            Self::Ru => "Прозрачность",
            Self::En => "Transparency",
        }
    }

    pub fn blur_enabled_label(self) -> &'static str {
        match self {
            Self::Ru => "Размытие",
            Self::En => "Blur",
        }
    }

    pub fn blur_intensity_label(self) -> &'static str {
        match self {
            Self::Ru => "Интенсивность",
            Self::En => "Intensity",
        }
    }

    pub fn blur_passes_label(self) -> &'static str {
        match self {
            Self::Ru => "Проходы",
            Self::En => "Passes",
        }
    }

    pub fn bg_brightness_label(self) -> &'static str {
        match self {
            Self::Ru => "Яркость фона",
            Self::En => "Background brightness",
        }
    }

    pub fn bg_saturation_label(self) -> &'static str {
        match self {
            Self::Ru => "Насыщенность фона",
            Self::En => "Background saturation",
        }
    }

    pub fn feature_supported(self) -> &'static str {
        match self {
            Self::Ru => "Поддерживается",
            Self::En => "Supported",
        }
    }

    pub fn feature_not_available(self) -> &'static str {
        match self {
            Self::Ru => "Недоступно на текущей платформе",
            Self::En => "Not available on current platform",
        }
    }

    pub fn feature_init_failed(self) -> &'static str {
        match self {
            Self::Ru => "Не удалось инициализировать",
            Self::En => "Failed to initialize",
        }
    }

    pub fn save_error_msg(self) -> &'static str {
        match self {
            Self::Ru => {
                "Не удалось сохранить настройки на диск. Изменения действуют в текущем сеансе."
            }
            Self::En => "Failed to save settings to disk. Changes apply to the current session.",
        }
    }

    pub fn show_page_status_label(self) -> &'static str {
        match self {
            Self::Ru => "Отображать состояние страницы",
            Self::En => "Show page status",
        }
    }

    pub fn page_loading(self) -> &'static str {
        match self {
            Self::Ru => "Страница загружается…",
            Self::En => "Page loading…",
        }
    }

    pub fn page_loaded(self) -> &'static str {
        match self {
            Self::Ru => "Страница загружена",
            Self::En => "Page loaded",
        }
    }

    pub fn page_load_failed(self) -> &'static str {
        match self {
            Self::Ru => "Не удалось загрузить страницу",
            Self::En => "Page failed to load",
        }
    }

    pub fn page_load_cancelled(self) -> &'static str {
        match self {
            Self::Ru => "Загрузка прервана",
            Self::En => "Loading cancelled",
        }
    }

    pub fn modules_tooltip(self) -> &'static str {
        match self {
            Self::Ru => "Модули",
            Self::En => "Modules",
        }
    }

    // --- Локализация 6 модулей браузера (v0.5-alpha) ---

    pub fn module_tab_manager_name(self) -> &'static str {
        match self {
            Self::Ru => "Управление вкладками",
            Self::En => "Tab Lifecycle Manager",
        }
    }

    pub fn module_tab_manager_desc(self) -> &'static str {
        match self {
            Self::Ru => "Умное усыпление фоновых вкладок и освобождение оперативной памяти.",
            Self::En => "Smart background tab sleeping and memory optimization.",
        }
    }

    pub fn module_performance_name(self) -> &'static str {
        match self {
            Self::Ru => "Производительность",
            Self::En => "Performance Diagnostics",
        }
    }

    pub fn module_performance_desc(self) -> &'static str {
        match self {
            Self::Ru => "Локальный мониторинг использования памяти и стабильности браузера.",
            Self::En => "Local memory usage and browser reliability metrics.",
        }
    }

    pub fn module_appearance_name(self) -> &'static str {
        match self {
            Self::Ru => "Внешний вид",
            Self::En => "Appearance & Theme",
        }
    }

    pub fn module_appearance_desc(self) -> &'static str {
        match self {
            Self::Ru => "Темы оформления, акцентный цвет, компактный режим и прозрачность.",
            Self::En => "Theme styling, accent color, compact mode and transparency.",
        }
    }

    pub fn module_appearance_required_reason(self) -> &'static str {
        match self {
            Self::Ru => {
                "Обязательный модуль: обеспечивает базовую тему и визуальную целостность интерфейса."
            }
            Self::En => "Required module: provides core theming and visual integrity.",
        }
    }

    pub fn module_shortcuts_name(self) -> &'static str {
        match self {
            Self::Ru => "Горячие клавиши",
            Self::En => "Shortcuts & Palette",
        }
    }

    pub fn module_shortcuts_desc(self) -> &'static str {
        match self {
            Self::Ru => "Палитра команд (Ctrl+Shift+P) и быстрый поиск по открытым вкладкам.",
            Self::En => "Command palette (Ctrl+Shift+P) and fast open tab search.",
        }
    }

    pub fn module_security_name(self) -> &'static str {
        match self {
            Self::Ru => "Безопасность",
            Self::En => "Security & TLS",
        }
    }

    pub fn module_security_desc(self) -> &'static str {
        match self {
            Self::Ru => "Проверка TLS-сертификатов и достоверная индикация защиты соединений.",
            Self::En => "TLS certificate verification and honest connection status.",
        }
    }

    pub fn module_security_required_reason(self) -> &'static str {
        match self {
            Self::Ru => {
                "Обязательный модуль: защищает пользователя и проверяет безопасность соединений."
            }
            Self::En => "Required module: protects user and validates connection security.",
        }
    }

    pub fn module_searchengine_name(self) -> &'static str {
        match self {
            Self::Ru => "Поисковые системы",
            Self::En => "Search Engine",
        }
    }

    pub fn module_searchengine_desc(self) -> &'static str {
        match self {
            Self::Ru => "Интеграция DuckDuckGo, Google или Яндекса прямо в адресной строке.",
            Self::En => "DuckDuckGo, Google or Yandex integration in the address bar.",
        }
    }

    pub fn search_provider_label(self) -> &'static str {
        match self {
            Self::Ru => "Поисковая система",
            Self::En => "Search Provider",
        }
    }

    pub fn search_disabled_hint(self) -> &'static str {
        match self {
            Self::Ru => "Поиск отключен. Введите адрес сайта или включите поиск в модулях.",
            Self::En => "Search is disabled. Enter a website address or enable search in modules.",
        }
    }

    pub fn compact_mode_label(self) -> &'static str {
        match self {
            Self::Ru => "Компактный режим панелей",
            Self::En => "Compact Toolbar Mode",
        }
    }

    pub fn tab_sleep_mode_label(self) -> &'static str {
        match self {
            Self::Ru => "Режим сна вкладок",
            Self::En => "Tab Sleep Mode",
        }
    }

    pub fn idle_timeout_label(self) -> &'static str {
        match self {
            Self::Ru => "Тайм-аут бездействия (минут)",
            Self::En => "Idle Timeout (minutes)",
        }
    }

    pub fn max_active_tabs_label(self) -> &'static str {
        match self {
            Self::Ru => "Максимум активных вкладок",
            Self::En => "Max Active Tabs",
        }
    }

    pub fn command_palette_title(self) -> &'static str {
        match self {
            Self::Ru => "Палитра команд",
            Self::En => "Command Palette",
        }
    }

    pub fn command_palette_placeholder(self) -> &'static str {
        match self {
            Self::Ru => "Поиск команды или действия…",
            Self::En => "Search command or action…",
        }
    }

    pub fn sleep_tab_action(self) -> &'static str {
        match self {
            Self::Ru => "Усыпить вкладку",
            Self::En => "Sleep Tab",
        }
    }

    pub fn wake_tab_action(self) -> &'static str {
        match self {
            Self::Ru => "Пробудить вкладку",
            Self::En => "Wake Tab",
        }
    }

    pub fn home_tab_title(self) -> &'static str {
        match self {
            Self::Ru => "Новая вкладка",
            Self::En => "New Tab",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressError {
    EmptyOrWhitespace,
    UnsupportedScheme,
    InvalidHost,
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
            (Self::InvalidHost, Language::Ru) => "Некорректный адрес хоста.",
            (Self::InvalidHost, Language::En) => "Invalid host address.",
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

    #[test]
    fn test_new_i18n_methods_no_panic() {
        for lang in [Language::Ru, Language::En] {
            // Вкладки настроек и модулей
            let _ = lang.settings_tab_title();
            let _ = lang.modules_tab_title();
            let _ = lang.modules_beta_label();
            let _ = lang.modules_tooltip();

            // Состояние модуля
            let _ = lang.module_enabled();
            let _ = lang.module_disabled();

            // Модуль прозрачности
            let _ = lang.module_transparency_blur_name();
            let _ = lang.module_transparency_blur_desc();
            let _ = lang.transparency_label();
            let _ = lang.blur_enabled_label();
            let _ = lang.blur_intensity_label();
            let _ = lang.blur_passes_label();
            let _ = lang.bg_brightness_label();
            let _ = lang.bg_saturation_label();

            // Поддержка функций
            let _ = lang.feature_supported();
            let _ = lang.feature_not_available();
            let _ = lang.feature_init_failed();

            // Ошибка сохранения
            let _ = lang.save_error_msg();

            // Состояние страницы
            let _ = lang.show_page_status_label();
            let _ = lang.page_loading();
            let _ = lang.page_loaded();
            let _ = lang.page_load_failed();
            let _ = lang.page_load_cancelled();

            // Модули 0.5-alpha
            let _ = lang.module_tab_manager_name();
            let _ = lang.module_tab_manager_desc();
            let _ = lang.module_performance_name();
            let _ = lang.module_performance_desc();
            let _ = lang.module_appearance_name();
            let _ = lang.module_appearance_desc();
            let _ = lang.module_appearance_required_reason();
            let _ = lang.module_shortcuts_name();
            let _ = lang.module_shortcuts_desc();
            let _ = lang.module_security_name();
            let _ = lang.module_security_desc();
            let _ = lang.module_security_required_reason();
            let _ = lang.module_searchengine_name();
            let _ = lang.module_searchengine_desc();

            // Настройки и действия
            let _ = lang.search_provider_label();
            let _ = lang.compact_mode_label();
            let _ = lang.tab_sleep_mode_label();
            let _ = lang.idle_timeout_label();
            let _ = lang.max_active_tabs_label();
            let _ = lang.command_palette_title();
            let _ = lang.command_palette_placeholder();
            let _ = lang.sleep_tab_action();
            let _ = lang.wake_tab_action();
            let _ = lang.home_tab_title();

            // Подключение
            let _ = lang.conn_loading();

            // Новый вариант AddressError
            let _ = AddressError::InvalidHost.message(lang);
        }
    }

    #[test]
    fn test_generate_home_html() {
        let home_ru = generate_home_html(
            Language::Ru,
            ThemeMode::Dark,
            "#3584e4",
            true,
            "DuckDuckGo",
            true,
            0.8,
        );
        assert!(home_ru.contains("Light"));
        assert!(home_ru.contains("DuckDuckGo"));
        assert!(home_ru.contains("rgba(28, 28, 31, 0.80)"));

        let home_en = generate_home_html(
            Language::En,
            ThemeMode::Light,
            "#3584e4",
            false,
            "",
            false,
            1.0,
        );
        assert!(home_en.contains("Enter a website address"));
    }
}
