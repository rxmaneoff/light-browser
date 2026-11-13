use std::fs;
use std::io::Write;
use std::path::PathBuf;

use relm4::gtk::glib;

use crate::i18n::Language;
use crate::searchengine::SearchProvider;
use crate::tab_manager::SleepMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

/// Ограничивает значение в заданный диапазон [min, max].
pub fn clamp_f64(val: f64, min: f64, max: f64) -> f64 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

fn float_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn parse_bool(val: &str, default: bool) -> bool {
    match val.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => default,
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    // --- Общие параметры (0.3-alpha / 0.4-alpha) ---
    pub language: Language,
    pub theme: ThemeMode,
    pub accent_color: String,
    pub show_page_status: bool,

    // --- Модули (0.5-alpha) ---
    pub module_tab_manager_enabled: bool,
    pub module_performance_enabled: bool,
    pub module_appearance_enabled: bool, // Обязательный: всегда true
    pub module_shortcuts_enabled: bool,
    pub module_security_enabled: bool, // Обязательный: всегда true
    pub module_searchengine_enabled: bool,

    // --- Настройки tab_manager ---
    pub tab_sleep_mode: SleepMode,
    pub tab_idle_timeout_minutes: u32,
    pub tab_max_active_tabs: usize,

    // --- Настройки searchengine ---
    pub search_provider: SearchProvider,

    // --- Настройки appearance ---
    pub compact_mode: bool,
    pub transparency_enabled: bool,
    pub transparency: f64,
    pub blur_enabled: bool,
    pub blur_intensity: f64,
    pub blur_passes: u32,
    pub bg_brightness: f64,
    pub bg_saturation: f64,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.language == other.language
            && self.theme == other.theme
            && self.accent_color == other.accent_color
            && self.show_page_status == other.show_page_status
            && self.module_tab_manager_enabled == other.module_tab_manager_enabled
            && self.module_performance_enabled == other.module_performance_enabled
            && self.module_appearance_enabled == other.module_appearance_enabled
            && self.module_shortcuts_enabled == other.module_shortcuts_enabled
            && self.module_security_enabled == other.module_security_enabled
            && self.module_searchengine_enabled == other.module_searchengine_enabled
            && self.tab_sleep_mode == other.tab_sleep_mode
            && self.tab_idle_timeout_minutes == other.tab_idle_timeout_minutes
            && self.tab_max_active_tabs == other.tab_max_active_tabs
            && self.search_provider == other.search_provider
            && self.compact_mode == other.compact_mode
            && self.transparency_enabled == other.transparency_enabled
            && float_eq(self.transparency, other.transparency)
            && self.blur_enabled == other.blur_enabled
            && float_eq(self.blur_intensity, other.blur_intensity)
            && self.blur_passes == other.blur_passes
            && float_eq(self.bg_brightness, other.bg_brightness)
            && float_eq(self.bg_saturation, other.bg_saturation)
    }
}

impl Eq for Config {}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            theme: ThemeMode::Dark,
            accent_color: "#3584e4".to_string(),
            show_page_status: false,

            // Модули по матрице:
            module_tab_manager_enabled: true,
            module_performance_enabled: false,
            module_appearance_enabled: true,
            module_shortcuts_enabled: true,
            module_security_enabled: true,
            module_searchengine_enabled: true,

            tab_sleep_mode: SleepMode::Manual,
            tab_idle_timeout_minutes: 15,
            tab_max_active_tabs: 5,

            search_provider: SearchProvider::DuckDuckGo,

            compact_mode: false,
            transparency_enabled: false,
            transparency: 1.0,
            blur_enabled: false,
            blur_intensity: 10.0,
            blur_passes: 1,
            bg_brightness: 1.0,
            bg_saturation: 1.0,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        glib::user_config_dir()
            .join("light-project")
            .join("config.toml")
    }

    /// Валидирует значения полей и принудительно восстанавливает обязательные модули.
    pub fn validate(&mut self) {
        // Обязательные модули НИКОГДА не могут быть отключены
        self.module_appearance_enabled = true;
        self.module_security_enabled = true;

        self.transparency = clamp_f64(self.transparency, 0.0, 1.0);
        self.blur_intensity = clamp_f64(self.blur_intensity, 0.0, 40.0);
        self.blur_passes = self.blur_passes.clamp(1, 4);
        self.bg_brightness = clamp_f64(self.bg_brightness, 0.5, 1.5);
        self.bg_saturation = clamp_f64(self.bg_saturation, 0.0, 2.0);

        self.tab_idle_timeout_minutes = self.tab_idle_timeout_minutes.clamp(1, 1440);
        self.tab_max_active_tabs = self.tab_max_active_tabs.clamp(1, 100);
    }

    pub fn load() -> Self {
        let path = Self::config_path();

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => Self::parse(&content),
            Err(e) => {
                eprintln!(
                    "Не удалось прочитать файл настроек {}: {}",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn parse(content: &str) -> Self {
        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');

                match key {
                    "language" => {
                        if let Some(lang) = Language::from_code(val) {
                            config.language = lang;
                        }
                    }
                    "theme" => {
                        if let Some(theme) = ThemeMode::from_str(val) {
                            config.theme = theme;
                        }
                    }
                    "accent_color" => {
                        let clean = val.trim();
                        if clean.starts_with('#')
                            && (clean.len() == 7 || clean.len() == 4)
                            && clean[1..].chars().all(|c| c.is_ascii_hexdigit())
                        {
                            config.accent_color = clean.to_string();
                        }
                    }
                    "show_page_status" => {
                        config.show_page_status = parse_bool(val, config.show_page_status);
                    }
                    // Модули
                    "tab_manager" | "module_tab_manager_enabled" => {
                        config.module_tab_manager_enabled =
                            parse_bool(val, config.module_tab_manager_enabled);
                    }
                    "performance" | "module_performance_enabled" => {
                        config.module_performance_enabled =
                            parse_bool(val, config.module_performance_enabled);
                    }
                    "appearance" | "module_appearance_enabled" => {
                        // Обязательный модуль всегда остается true
                        config.module_appearance_enabled = true;
                    }
                    "shortcuts" | "module_shortcuts_enabled" => {
                        config.module_shortcuts_enabled =
                            parse_bool(val, config.module_shortcuts_enabled);
                    }
                    "security" | "module_security_enabled" => {
                        // Обязательный модуль всегда остается true
                        config.module_security_enabled = true;
                    }
                    "searchengine" | "module_searchengine_enabled" => {
                        config.module_searchengine_enabled =
                            parse_bool(val, config.module_searchengine_enabled);
                    }
                    // Tab manager settings
                    "mode" | "tab_sleep_mode" => {
                        if let Some(mode) = SleepMode::from_str(val) {
                            config.tab_sleep_mode = mode;
                        }
                    }
                    "idle_timeout_minutes" | "tab_idle_timeout_minutes" => {
                        if let Ok(v) = val.parse::<u32>() {
                            config.tab_idle_timeout_minutes = v;
                        }
                    }
                    "max_active_tabs" | "tab_max_active_tabs" => {
                        if let Ok(v) = val.parse::<usize>() {
                            config.tab_max_active_tabs = v;
                        }
                    }
                    // Search provider
                    "provider" | "search_provider" => {
                        if let Some(p) = SearchProvider::from_str(val) {
                            config.search_provider = p;
                        }
                    }
                    // Appearance settings
                    "compact_mode" => {
                        config.compact_mode = parse_bool(val, config.compact_mode);
                    }
                    "transparency" => {
                        // Поддержка булева значения и float
                        if val == "true" || val == "false" {
                            config.transparency_enabled = parse_bool(val, false);
                        } else if let Ok(v) = val.parse::<f64>() {
                            config.transparency = v;
                        }
                    }
                    "transparency_enabled" | "module_transparency_blur_enabled" => {
                        config.transparency_enabled = parse_bool(val, config.transparency_enabled);
                    }
                    "blur" | "blur_enabled" => {
                        config.blur_enabled = parse_bool(val, config.blur_enabled);
                    }
                    "blur_intensity" => {
                        if let Ok(v) = val.parse::<f64>() {
                            config.blur_intensity = v;
                        }
                    }
                    "blur_passes" => {
                        if let Ok(v) = val.parse::<u32>() {
                            config.blur_passes = v;
                        }
                    }
                    "bg_brightness" => {
                        if let Ok(v) = val.parse::<f64>() {
                            config.bg_brightness = v;
                        }
                    }
                    "bg_saturation" => {
                        if let Ok(v) = val.parse::<f64>() {
                            config.bg_saturation = v;
                        }
                    }
                    _ => {}
                }
            }
        }

        config.validate();
        config
    }

    pub fn serialize(&self) -> String {
        format!(
            "# Light Browser Configuration (v0.5-alpha)\n\
             \n\
             # Основные параметры\n\
             language = \"{}\"\n\
             theme = \"{}\"\n\
             accent_color = \"{}\"\n\
             show_page_status = {}\n\
             \n\
             [modules]\n\
             tab_manager = {}\n\
             performance = {}\n\
             appearance = true\n\
             shortcuts = {}\n\
             security = true\n\
             searchengine = {}\n\
             \n\
             [tab_manager]\n\
             mode = \"{}\"\n\
             idle_timeout_minutes = {}\n\
             max_active_tabs = {}\n\
             \n\
             [searchengine]\n\
             provider = \"{}\"\n\
             \n\
             [appearance]\n\
             compact_mode = {}\n\
             transparency_enabled = {}\n\
             transparency = {:.2}\n\
             blur = {}\n\
             blur_intensity = {:.1}\n\
             blur_passes = {}\n\
             bg_brightness = {:.2}\n\
             bg_saturation = {:.2}\n",
            self.language.code(),
            self.theme.as_str(),
            self.accent_color,
            self.show_page_status,
            self.module_tab_manager_enabled,
            self.module_performance_enabled,
            self.module_shortcuts_enabled,
            self.module_searchengine_enabled,
            self.tab_sleep_mode.as_str(),
            self.tab_idle_timeout_minutes,
            self.tab_max_active_tabs,
            self.search_provider.as_str(),
            self.compact_mode,
            self.transparency_enabled,
            self.transparency,
            self.blur_enabled,
            self.blur_intensity,
            self.blur_passes,
            self.bg_brightness,
            self.bg_saturation,
        )
    }

    /// Атомарно сохраняет конфигурацию на диск через временный файл.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(self.serialize().as_bytes())?;
        file.sync_all()?;
        drop(file);

        fs::rename(&tmp_path, &path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_and_parse() {
        let default = Config::default();
        let serialized = default.serialize();
        let parsed = Config::parse(&serialized);

        assert_eq!(parsed, default);
        assert!(parsed.module_appearance_enabled);
        assert!(parsed.module_security_enabled);
    }

    #[test]
    fn test_backward_compat_0_3_and_0_4_alpha() {
        let old_content = r##"
            language = "en"
            theme = "light"
            accent_color = "#e63946"
            show_page_status = true
            module_transparency_blur_enabled = true
            transparency = 0.85
        "##;

        let parsed = Config::parse(old_content);
        assert_eq!(parsed.language, Language::En);
        assert_eq!(parsed.theme, ThemeMode::Light);
        assert_eq!(parsed.accent_color, "#e63946");
        assert!(parsed.show_page_status);
        assert!(parsed.transparency_enabled);
        assert!(float_eq(parsed.transparency, 0.85));

        // Новые модули должны получить безопасные defaults
        assert!(parsed.module_tab_manager_enabled);
        assert!(!parsed.module_performance_enabled);
        assert!(parsed.module_appearance_enabled);
        assert!(parsed.module_shortcuts_enabled);
        assert!(parsed.module_security_enabled);
        assert!(parsed.module_searchengine_enabled);
        assert_eq!(parsed.search_provider, SearchProvider::DuckDuckGo);
        assert_eq!(parsed.tab_sleep_mode, SleepMode::Manual);
    }

    #[test]
    fn test_required_modules_cannot_be_disabled_via_config() {
        let malicious_content = r#"
            appearance = false
            security = false
            module_appearance_enabled = false
            module_security_enabled = false
        "#;

        let parsed = Config::parse(malicious_content);
        assert!(
            parsed.module_appearance_enabled,
            "appearance обязан быть true!"
        );
        assert!(parsed.module_security_enabled, "security обязан быть true!");
    }

    #[test]
    fn test_range_clamping() {
        let bad_content = r#"
            transparency = 2.5
            idle_timeout_minutes = 999999
            max_active_tabs = 0
            blur_intensity = 100.0
        "#;

        let parsed = Config::parse(bad_content);
        assert_eq!(parsed.transparency, 1.0);
        assert_eq!(parsed.tab_idle_timeout_minutes, 1440);
        assert_eq!(parsed.tab_max_active_tabs, 1);
        assert_eq!(parsed.blur_intensity, 40.0);
    }

    #[test]
    fn test_save_and_load() {
        let tmp_dir = std::env::temp_dir().join("light_test_cfg_0_5");
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        let cfg = Config {
            language: Language::En,
            theme: ThemeMode::Light,
            compact_mode: true,
            tab_sleep_mode: SleepMode::Balanced,
            search_provider: SearchProvider::Google,
            ..Default::default()
        };

        let file_path = tmp_dir.join("test.toml");
        fs::write(&file_path, cfg.serialize()).unwrap();

        let loaded = Config::parse(&fs::read_to_string(&file_path).unwrap());
        assert_eq!(loaded, cfg);

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
