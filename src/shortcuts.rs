//! Модуль клавиатурных сочетаний и палитры команд (`shortcuts.rs`).
//!
//! Отвечает за:
//! - Регистрацию и обработку сочетаний клавиш;
//! - Палитру команд `Ctrl+Shift+P` (Command Palette);
//! - Быстрый поиск и переключение между открытыми вкладками;
//! - Сохранение базовых системных клавиш (Ctrl+T, Ctrl+W, Ctrl+R) даже при выключении модуля.

use crate::i18n::Language;
use crate::tabs::TabId;

/// Идентификаторы действий браузера.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserAction {
    NewTab,
    CloseTab,
    Reload,
    GoBack,
    GoForward,
    FocusAddressBar,
    OpenSettings,
    OpenModules,
    OpenPerformance,
    ToggleTheme,
    ToggleLanguage,
    SwitchToTab(TabId),
}

/// Элемент палитры команд для отображения пользователю.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandItem {
    pub action: BrowserAction,
    pub title: String,
    pub description: Option<String>,
    pub shortcut_hint: Option<&'static str>,
}

/// Фильтрует список команд по пользовательскому поисковому запросу.
pub fn filter_commands(query: &str, items: &[CommandItem]) -> Vec<CommandItem> {
    let clean = query.trim().to_lowercase();
    if clean.is_empty() {
        return items.to_vec();
    }

    items
        .iter()
        .filter(|item| {
            item.title.to_lowercase().contains(&clean)
                || item
                    .description
                    .as_ref()
                    .is_some_and(|d| d.to_lowercase().contains(&clean))
        })
        .cloned()
        .collect()
}

/// Возвращает стандартный список доступных команд браузера.
pub fn standard_commands(lang: Language) -> Vec<CommandItem> {
    match lang {
        Language::Ru => vec![
            CommandItem {
                action: BrowserAction::NewTab,
                title: "Новая вкладка".to_string(),
                description: Some("Открыть пустую вкладку браузера".to_string()),
                shortcut_hint: Some("Ctrl+T"),
            },
            CommandItem {
                action: BrowserAction::CloseTab,
                title: "Закрыть текущую вкладку".to_string(),
                description: Some("Закрыть активную вкладку".to_string()),
                shortcut_hint: Some("Ctrl+W"),
            },
            CommandItem {
                action: BrowserAction::Reload,
                title: "Обновить страницу".to_string(),
                description: Some("Перезагрузить текущую веб-страницу".to_string()),
                shortcut_hint: Some("Ctrl+R"),
            },
            CommandItem {
                action: BrowserAction::GoBack,
                title: "Назад".to_string(),
                description: Some("Вернуться на предыдущую страницу".to_string()),
                shortcut_hint: Some("Alt+Left"),
            },
            CommandItem {
                action: BrowserAction::GoForward,
                title: "Вперёд".to_string(),
                description: Some("Перейти на следующую страницу".to_string()),
                shortcut_hint: Some("Alt+Right"),
            },
            CommandItem {
                action: BrowserAction::FocusAddressBar,
                title: "Выделить адресную строку".to_string(),
                description: Some("Перевести фокус в поле ввода адреса".to_string()),
                shortcut_hint: Some("Ctrl+L"),
            },
            CommandItem {
                action: BrowserAction::OpenSettings,
                title: "Настройки".to_string(),
                description: Some("Открыть вкладку параметров браузера".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::OpenModules,
                title: "Менеджер модулей".to_string(),
                description: Some("Управление встроенными модулями (Beta)".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::OpenPerformance,
                title: "Диагностика производительности".to_string(),
                description: Some("Просмотр памяти и метрик браузера".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::ToggleTheme,
                title: "Переключить тему".to_string(),
                description: Some("Смена светлой и тёмной темы".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::ToggleLanguage,
                title: "Сменить язык".to_string(),
                description: Some("Переключение между RU и EN".to_string()),
                shortcut_hint: None,
            },
        ],
        Language::En => vec![
            CommandItem {
                action: BrowserAction::NewTab,
                title: "New Tab".to_string(),
                description: Some("Open a new browser tab".to_string()),
                shortcut_hint: Some("Ctrl+T"),
            },
            CommandItem {
                action: BrowserAction::CloseTab,
                title: "Close Current Tab".to_string(),
                description: Some("Close active tab".to_string()),
                shortcut_hint: Some("Ctrl+W"),
            },
            CommandItem {
                action: BrowserAction::Reload,
                title: "Reload Page".to_string(),
                description: Some("Reload current web page".to_string()),
                shortcut_hint: Some("Ctrl+R"),
            },
            CommandItem {
                action: BrowserAction::GoBack,
                title: "Back".to_string(),
                description: Some("Navigate to previous page".to_string()),
                shortcut_hint: Some("Alt+Left"),
            },
            CommandItem {
                action: BrowserAction::GoForward,
                title: "Forward".to_string(),
                description: Some("Navigate to next page".to_string()),
                shortcut_hint: Some("Alt+Right"),
            },
            CommandItem {
                action: BrowserAction::FocusAddressBar,
                title: "Focus Address Bar".to_string(),
                description: Some("Focus URL address entry".to_string()),
                shortcut_hint: Some("Ctrl+L"),
            },
            CommandItem {
                action: BrowserAction::OpenSettings,
                title: "Settings".to_string(),
                description: Some("Open browser settings tab".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::OpenModules,
                title: "Module Manager".to_string(),
                description: Some("Manage built-in modules (Beta)".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::OpenPerformance,
                title: "Performance Diagnostics".to_string(),
                description: Some("Inspect memory and metrics".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::ToggleTheme,
                title: "Toggle Theme".to_string(),
                description: Some("Switch between light and dark theme".to_string()),
                shortcut_hint: None,
            },
            CommandItem {
                action: BrowserAction::ToggleLanguage,
                title: "Switch Language".to_string(),
                description: Some("Toggle between RU and EN".to_string()),
                shortcut_hint: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_commands() {
        let items = standard_commands(Language::Ru);

        // Пустой запрос возвращает все команды
        let all = filter_commands("", &items);
        assert_eq!(all.len(), items.len());

        // Поиск по "вкладк"
        let filtered = filter_commands("вкладк", &items);
        assert!(filtered.len() >= 2);
        assert!(filtered.iter().any(|c| c.action == BrowserAction::NewTab));

        // Поиск по "settings" на английском
        let items_en = standard_commands(Language::En);
        let settings = filter_commands("settings", &items_en);
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].action, BrowserAction::OpenSettings);
    }
}
