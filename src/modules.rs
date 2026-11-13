//! Реестр встроенных модулей браузера Light (версия 0.5-alpha).
//!
//! Управляет 6 встроенными подсистемами приложения в соответствии с матрицей:
//! 1. `tab_manager` — жизненный цикл и память вкладок (опциональный, по умолчанию включен);
//! 2. `performance` — мониторинг и диагностика (опциональный, по умолчанию выключен);
//! 3. `appearance` — единый визуальный слой Quiet Native (ОБЯЗАТЕЛЬНЫЙ, всегда включен);
//! 4. `shortcuts` — клавиатурные сочетания и палитра команд (опциональный, по умолчанию включен);
//! 5. `security` — проверка TLS и безопасность соединений (ОБЯЗАТЕЛЬНЫЙ, всегда включен);
//! 6. `searchengine` — поиск и разбор ввода (опциональный, по умолчанию включен).
//!
//! Сторонняя экосистема внешних плагинов не поддерживается.

use crate::config::Config;
use crate::i18n::Language;

/// Статус поддержки функции на текущей графической платформе.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureStatus {
    /// Функция поддерживается и активна.
    Active,
    /// Функция поддерживается платформой, но выключена в настройках.
    SupportedDisabled,
    /// Недоступно на текущей платформе или оконном композиторе.
    Unavailable,
    /// Не удалось инициализировать эффект.
    InitFailed,
}

impl FeatureStatus {
    pub fn is_available(self) -> bool {
        matches!(self, Self::Active | Self::SupportedDisabled)
    }
}

/// Возможности графической платформы (GTK4 / Wayland / X11 / Композитор).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformCapabilities {
    /// Прозрачность UI-поверхностей (поддерживается через CSS rgba).
    pub transparency: FeatureStatus,
    /// Размытие фона оконным композитором.
    pub blur: FeatureStatus,
    /// Имя активного дисплейного бэкенда.
    pub backend_name: String,
}

impl PlatformCapabilities {
    pub fn detect(transparency_requested: bool, blur_requested: bool) -> Self {
        let backend_name = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            "Wayland".to_string()
        } else if std::env::var_os("DISPLAY").is_some() {
            "X11".to_string()
        } else {
            "Headless / Unknown".to_string()
        };

        let transparency = if transparency_requested {
            FeatureStatus::Active
        } else {
            FeatureStatus::SupportedDisabled
        };

        let _ = blur_requested;
        // Размытие фона требует внешних нестандартных протоколов композитора (ext-blur),
        // честно указываем статус Unavailable
        let blur = FeatureStatus::Unavailable;

        Self {
            transparency,
            blur,
            backend_name,
        }
    }
}

/// Описание модуля для отображения в реестре и менеджере модулей.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub is_required: bool,
    pub required_reason: Option<&'static str>,
    pub is_enabled: bool,
}

/// Идентификаторы шести встроенных модулей.
pub struct ModuleId;

impl ModuleId {
    pub const TAB_MANAGER: &'static str = "tab_manager";
    pub const PERFORMANCE: &'static str = "performance";
    pub const APPEARANCE: &'static str = "appearance";
    pub const SHORTCUTS: &'static str = "shortcuts";
    pub const SECURITY: &'static str = "security";
    pub const SEARCH_ENGINE: &'static str = "searchengine";
}

/// Реестр встроенных модулей браузера.
pub struct ModuleRegistry;

impl ModuleRegistry {
    /// Возвращает метаданные и текущие состояния всех 6 модулей браузера.
    pub fn list_modules(config: &Config, lang: Language) -> Vec<ModuleInfo> {
        vec![
            // 1. Управление вкладками и памятью
            ModuleInfo {
                id: ModuleId::TAB_MANAGER,
                title: lang.module_tab_manager_name(),
                description: lang.module_tab_manager_desc(),
                is_required: false,
                required_reason: None,
                is_enabled: config.module_tab_manager_enabled,
            },
            // 2. Диагностика и производительность
            ModuleInfo {
                id: ModuleId::PERFORMANCE,
                title: lang.module_performance_name(),
                description: lang.module_performance_desc(),
                is_required: false,
                required_reason: None,
                is_enabled: config.module_performance_enabled,
            },
            // 3. Внешний вид (ОБЯЗАТЕЛЬНЫЙ)
            ModuleInfo {
                id: ModuleId::APPEARANCE,
                title: lang.module_appearance_name(),
                description: lang.module_appearance_desc(),
                is_required: true,
                required_reason: Some(lang.module_appearance_required_reason()),
                is_enabled: true, // Всегда включен!
            },
            // 4. Горячие клавиши и палитра команд
            ModuleInfo {
                id: ModuleId::SHORTCUTS,
                title: lang.module_shortcuts_name(),
                description: lang.module_shortcuts_desc(),
                is_required: false,
                required_reason: None,
                is_enabled: config.module_shortcuts_enabled,
            },
            // 5. Безопасность и TLS (ОБЯЗАТЕЛЬНЫЙ)
            ModuleInfo {
                id: ModuleId::SECURITY,
                title: lang.module_security_name(),
                description: lang.module_security_desc(),
                is_required: true,
                required_reason: Some(lang.module_security_required_reason()),
                is_enabled: true, // Всегда включен!
            },
            // 6. Поисковые системы и разбор ввода
            ModuleInfo {
                id: ModuleId::SEARCH_ENGINE,
                title: lang.module_searchengine_name(),
                description: lang.module_searchengine_desc(),
                is_required: false,
                required_reason: None,
                is_enabled: config.module_searchengine_enabled,
            },
        ]
    }

    /// Проверяет, является ли модуль обязательным для работы браузера.
    pub fn is_required(id: &str) -> bool {
        matches!(id, ModuleId::APPEARANCE | ModuleId::SECURITY)
    }

    /// Пытается изменить состояние модуля в конфигурации.
    ///
    /// Для обязательных модулей (`appearance`, `security`) возвращает ошибку,
    /// гарантируя невозможность их отключения.
    pub fn try_set_enabled(
        config: &mut Config,
        id: &str,
        enabled: bool,
    ) -> Result<(), &'static str> {
        if Self::is_required(id) && !enabled {
            return Err("Обязательный модуль не может быть отключен.");
        }

        match id {
            ModuleId::TAB_MANAGER => {
                config.module_tab_manager_enabled = enabled;
            }
            ModuleId::PERFORMANCE => {
                config.module_performance_enabled = enabled;
            }
            ModuleId::APPEARANCE => {
                config.module_appearance_enabled = true; // Защита
            }
            ModuleId::SHORTCUTS => {
                config.module_shortcuts_enabled = enabled;
            }
            ModuleId::SECURITY => {
                config.module_security_enabled = true; // Защита
            }
            ModuleId::SEARCH_ENGINE => {
                config.module_searchengine_enabled = enabled;
            }
            _ => return Err("Неизвестный идентификатор модуля."),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_capabilities_detection() {
        let caps = PlatformCapabilities::detect(true, false);
        assert_eq!(caps.transparency, FeatureStatus::Active);
        assert_eq!(caps.blur, FeatureStatus::Unavailable);
        assert!(!caps.backend_name.is_empty());
    }

    #[test]
    fn test_required_modules_cannot_be_disabled() {
        let mut config = Config::default();

        // Попытка выключить appearance должна завершиться ошибкой
        let res_app = ModuleRegistry::try_set_enabled(&mut config, ModuleId::APPEARANCE, false);
        assert!(res_app.is_err());
        assert!(config.module_appearance_enabled);

        // Попытка выключить security должна завершиться ошибкой
        let res_sec = ModuleRegistry::try_set_enabled(&mut config, ModuleId::SECURITY, false);
        assert!(res_sec.is_err());
        assert!(config.module_security_enabled);
    }

    #[test]
    fn test_optional_modules_can_be_toggled() {
        let mut config = Config::default();

        assert!(config.module_searchengine_enabled);
        ModuleRegistry::try_set_enabled(&mut config, ModuleId::SEARCH_ENGINE, false).unwrap();
        assert!(!config.module_searchengine_enabled);

        ModuleRegistry::try_set_enabled(&mut config, ModuleId::PERFORMANCE, true).unwrap();
        assert!(config.module_performance_enabled);
    }

    #[test]
    fn test_list_modules_count_and_properties() {
        let config = Config::default();
        let list = ModuleRegistry::list_modules(&config, Language::Ru);
        assert_eq!(list.len(), 6);

        let req_count = list.iter().filter(|m| m.is_required).count();
        assert_eq!(req_count, 2, "Ровно 2 модуля должны быть обязательными");
    }
}
