use std::fs;
use std::path::PathBuf;

use relm4::gtk::glib;

use crate::i18n::Language;

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

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub language: Language,
    pub theme: ThemeMode,
    pub accent_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::Ru,
            theme: ThemeMode::Dark,
            accent_color: "#3584e4".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        glib::user_config_dir()
            .join("light-project")
            .join("config.toml")
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
            if line.is_empty() || line.starts_with('#') {
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
                    _ => {}
                }
            }
        }

        config
    }

    pub fn serialize(&self) -> String {
        format!(
            "# Light Browser Configuration\n\
             language = \"{}\"\n\
             theme = \"{}\"\n\
             accent_color = \"{}\"\n",
            self.language.code(),
            self.theme.as_str(),
            self.accent_color
        )
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = self.serialize();
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, content)?;
        fs::rename(tmp_path, path)?;
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
    }

    #[test]
    fn test_config_parse_custom() {
        let toml = r##"
            # Comment
            language = "en"
            theme = "light"
            accent_color = "#e01b24"
            unknown_setting = "ignored"
        "##;
        let parsed = Config::parse(toml);
        assert_eq!(parsed.language, Language::En);
        assert_eq!(parsed.theme, ThemeMode::Light);
        assert_eq!(parsed.accent_color, "#e01b24");
    }

    #[test]
    fn test_config_parse_invalid_values_fallback_to_defaults() {
        let toml = r##"
            language = "invalid"
            theme = "unknown"
            accent_color = "not_a_color"
        "##;
        let parsed = Config::parse(toml);
        assert_eq!(parsed.language, Language::Ru);
        assert_eq!(parsed.theme, ThemeMode::Dark);
        assert_eq!(parsed.accent_color, "#3584e4");
    }
}
