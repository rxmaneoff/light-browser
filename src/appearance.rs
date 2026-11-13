//! Модуль оформления и визуального слоя (`appearance.rs`).
//!
//! Является обязательным модулем (всегда включен).
//! Отвечает за:
//! - Темы оформления (светлая, тёмная);
//! - Пользовательский акцентный цвет и системную гармонию;
//! - Компактный режим панели инструментов и вкладок;
//! - Управление прозрачностью интерфейса и проверку возможностей композитора;
//! - Дизайн-концепцию Quiet Native: спокойный, утилитарный Linux-интерфейс,
//!   где во главе стоит веб-контент, а акцентный цвет обозначает интерактивность.

use crate::config::{Config, ThemeMode};

/// Настройки визуального оформления.
#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceConfig {
    pub theme: ThemeMode,
    pub accent_color: String,
    pub compact_mode: bool,
    pub transparency: bool,
    pub transparency_value: f64,
    pub blur: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            accent_color: "#3584e4".to_string(),
            compact_mode: false,
            transparency: false,
            transparency_value: 1.0,
            blur: false,
        }
    }
}

impl AppearanceConfig {
    pub fn from_config(config: &Config) -> Self {
        Self {
            theme: config.theme,
            accent_color: config.accent_color.clone(),
            compact_mode: config.compact_mode,
            transparency: config.transparency_enabled,
            transparency_value: config.transparency,
            blur: config.blur_enabled,
        }
    }
}

/// Цветовые токены интерфейса Quiet Native.
#[derive(Debug, Clone)]
pub struct ColorTokens {
    pub bg: &'static str,
    pub fg: &'static str,
    pub surface: &'static str,
    pub surface_hover: &'static str,
    pub border: &'static str,
    pub border_subtle: &'static str,
    pub tab_bg: &'static str,
    pub tab_bg_active: &'static str,
    pub tab_fg_active: &'static str,
    pub entry_bg: &'static str,
}

impl ColorTokens {
    pub fn for_theme(theme: ThemeMode) -> Self {
        match theme {
            ThemeMode::Light => Self {
                bg: "#f6f8fa",
                fg: "#1f2328",
                surface: "#ffffff",
                surface_hover: "#f0f2f5",
                border: "#d0d7de",
                border_subtle: "rgba(0, 0, 0, 0.08)",
                tab_bg: "rgba(0, 0, 0, 0.04)",
                tab_bg_active: "#ffffff",
                tab_fg_active: "#1f2328",
                entry_bg: "#ffffff",
            },
            ThemeMode::Dark => Self {
                bg: "#1c1c1f",
                fg: "#f0f2f5",
                surface: "#28282d",
                surface_hover: "#323238",
                border: "#3c3c43",
                border_subtle: "rgba(255, 255, 255, 0.08)",
                tab_bg: "rgba(255, 255, 255, 0.05)",
                tab_bg_active: "#2c2c32",
                tab_fg_active: "#ffffff",
                entry_bg: "#242428",
            },
        }
    }
}

/// Генерирует CSS для GTK4 на основе темы и параметров оформления.
pub fn generate_gtk_css(config: &AppearanceConfig) -> String {
    let tokens = ColorTokens::for_theme(config.theme);
    let accent = &config.accent_color;

    // Вычисление фонового цвета с учетом прозрачности
    let window_bg = if config.transparency {
        let alpha = config.transparency_value.clamp(0.0, 1.0);
        let (r, g, b) = match config.theme {
            ThemeMode::Light => (246, 248, 250),
            ThemeMode::Dark => (28, 28, 31),
        };
        format!("rgba({r}, {g}, {b}, {alpha:.2})")
    } else {
        tokens.bg.to_string()
    };

    // Размеры элементов в зависимости от компактного режима (Quiet Native)
    let (tab_height, nav_height, tab_padding_y, font_size) = if config.compact_mode {
        ("24px", "24px", "0px", "11px")
    } else {
        ("28px", "28px", "2px", "12px")
    };

    format!(
        r#"
        @define-color accent_color {accent};
        @define-color accent_bg_color {accent};
        @define-color accent_fg_color #ffffff;
        @define-color theme_selected_bg_color {accent};
        @define-color theme_selected_fg_color #ffffff;

        window, .main-window {{
            background-color: {window_bg};
            color: {fg};
        }}

        .main-window label,
        .main-window button {{
            color: {fg};
        }}

        .main-window label.dim-label {{
            opacity: 0.68;
        }}

        .header-panel, .navigation-panel {{
            background-color: transparent;
        }}

        /* Кнопки вкладок в стиле Quiet Native:
           Мягкий фон, тонкая рамка, активная вкладка выделена акцентным индикатором */
        .tab-item-box {{
            border-radius: 8px;
            border: 1px solid {border_subtle};
            background-color: {tab_bg};
            padding: {tab_padding_y} 8px;
            margin: 0;
            min-height: {tab_height};
            transition: background-color 140ms ease, border-color 140ms ease;
        }}

        .tab-item-box:hover {{
            background-color: {surface_hover};
            border-color: {border};
        }}

        .tab-item-box.tab-item-active {{
            background-color: {tab_bg_active};
            border-color: {accent};
        }}

        .tab-title-label {{
            color: {fg};
            font-size: {font_size};
            line-height: {tab_height};
            margin-right: 6px;
            padding: 0;
        }}

        .tab-item-active .tab-title-label {{
            color: {tab_fg_active};
            font-weight: 600;
        }}

        .tab-close-btn {{
            color: {accent};
            opacity: 0.8;
            padding: 0;
            margin: 0;
            min-height: 16px;
            min-width: 16px;
            font-size: 10px;
            line-height: 10px;
            border-radius: 6px;
            transition: opacity 120ms ease, background-color 120ms ease;
        }}

        .tab-close-btn:hover {{
            opacity: 1.0;
            background-color: {surface_hover};
        }}

        /* Круглая кнопка добавления вкладки `+` */
        .tab-add-btn {{
            border-radius: 9999px;
            border: 1px solid transparent;
            background-color: transparent;
            color: {fg};
            padding: 0;
            min-height: 17px;
            min-width: 17px;
            font-size: 13px;
            transition: background-color 140ms ease;
        }}

        .tab-add-btn:hover {{
            background-color: transparent;
        }}

        .tab-add-icon {{
            border-radius: 9999px;
            min-height: 20px;
            min-width: 20px;
            background-color: transparent;
            transition: background-color 140ms ease;
        }}

        .tab-add-btn:hover .tab-add-icon {{
            background-color: {surface_hover};
        }}

        /* Панель навигации: кнопки Назад, Вперёд, Обновить */
        .navigation-panel button.nav-btn,
        .navigation-panel button {{
            min-height: {nav_height};
            padding: 0 8px;
            font-size: {font_size};
            border-radius: 8px;
            border: 1px solid {border_subtle};
            background-color: {tab_bg};
            color: {fg};
        }}

        .navigation-panel button:hover {{
            background-color: {surface_hover};
            border-color: {border};
        }}

        .main-window button.settings-btn {{
            border-radius: 8px;
            border: 1px solid {border_subtle};
            background-color: {surface};
            color: {fg};
        }}

        .main-window button.settings-btn:hover {{
            background-color: {surface_hover};
            border-color: {border};
        }}

        .main-window button.settings-btn:active {{
            background-color: {tab_bg_active};
        }}

        /* Адресная строка */
        .navigation-panel entry,
        entry.address-entry {{
            min-height: {nav_height};
            padding-top: 0;
            padding-bottom: 0;
            padding-left: 8px;
            padding-right: 8px;
            font-size: {font_size};
            border-radius: 8px;
            border: 1px solid {border};
            background-color: {entry_bg};
            color: {fg};
        }}

        .navigation-panel entry:focus-within {{
            border-color: {accent};
            box-shadow: 0 0 0 1px {accent};
        }}

        /* Карточки внутренних страниц (Настройки, Модули) */
        .internal-card {{
            background-color: {surface};
            border: 1px solid {border};
            border-radius: 10px;
            padding: 16px;
        }}

        /* Индикатор спящей вкладки */
        .tab-sleep-badge {{
            color: {fg};
            opacity: 0.6;
            font-size: 10px;
            margin-right: 4px;
        }}

        /* Баннер ошибок сохранения */
        .error-banner {{
            background-color: rgba(230, 60, 60, 0.12);
            border: 1px solid rgba(230, 60, 60, 0.35);
            border-radius: 8px;
            padding: 6px 12px;
            margin: 4px 8px;
            color: #e63946;
        }}

        /* Палитра команд (Command Palette) */
        .command-palette-window {{
            background-color: {surface};
            border: 1px solid {border};
            border-radius: 10px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
            padding: 8px;
        }}

        .command-item {{
            padding: 6px 10px;
            border-radius: 6px;
            color: {fg};
        }}

        .command-item:hover, .command-item:selected {{
            background-color: {accent};
            color: #ffffff;
        }}
    "#,
        fg = tokens.fg,
        surface = tokens.surface,
        surface_hover = tokens.surface_hover,
        border = tokens.border,
        border_subtle = tokens.border_subtle,
        tab_bg = tokens.tab_bg,
        tab_bg_active = tokens.tab_bg_active,
        tab_fg_active = tokens.tab_fg_active,
        entry_bg = tokens.entry_bg,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_css_generation() {
        let mut cfg = AppearanceConfig::default();
        let css_dark = generate_gtk_css(&cfg);
        assert!(css_dark.contains("@define-color accent_color #3584e4"));
        assert!(css_dark.contains(".tab-item-box"));

        cfg.theme = ThemeMode::Light;
        cfg.compact_mode = true;
        let css_light = generate_gtk_css(&cfg);
        assert!(css_light.contains("24px")); // компактная высота
    }

    #[test]
    fn test_transparency_alpha_in_css() {
        let cfg = AppearanceConfig {
            transparency: true,
            transparency_value: 0.85,
            ..Default::default()
        };
        let css = generate_gtk_css(&cfg);
        assert!(css.contains("rgba(28, 28, 31, 0.85)"));
    }
}
