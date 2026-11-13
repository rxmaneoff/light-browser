use std::time::Instant;

use crate::engine::{ConnectionSecurity, Engine};
use crate::i18n::{AddressError, Language};
use crate::tab_manager::TabLifeCycleState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

/// Тип содержимого вкладки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    /// Обычная веб-страница.
    Web,
    /// Домашняя страница со строкой поиска (light://home).
    Home,
    /// Встроенная тестовая страница проверки WebKitGTK.
    Probe,
    /// Внутренняя страница настроек браузера (about:settings).
    Settings,
    /// Внутренняя страница менеджера модулей (about:modules).
    Modules,
    /// Внутренняя страница диагностики (light://performance).
    Performance,
}

/// Статус загрузки веб-страницы для ненавязчивого индикатора.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLoadStatus {
    None,
    Loading,
    Loaded,
    Failed,
    Cancelled,
}

impl PageLoadStatus {
    pub fn message(self, lang: Language) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Loading => Some(lang.page_loading()),
            Self::Loaded => Some(lang.page_loaded()),
            Self::Failed => Some(lang.page_load_failed()),
            Self::Cancelled => Some(lang.page_load_cancelled()),
        }
    }
}

/// Детальный статус жизненного цикла вкладки.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStatus {
    Waiting,
    Navigating,
    Loading,
    Finished,
    Failed,
    Cancelled,
    ProcessTerminated,
    Error(AddressError),
}

impl TabStatus {
    pub fn message(self, lang: Language) -> &'static str {
        match self {
            Self::Waiting => lang.status_waiting(),
            Self::Navigating => lang.status_navigating(),
            Self::Loading => lang.status_loading(),
            Self::Finished => lang.status_finished(),
            Self::Failed => lang.status_failed(),
            Self::Cancelled => lang.page_load_cancelled(),
            Self::ProcessTerminated => lang.status_process_terminated(),
            Self::Error(err) => err.message(lang),
        }
    }
}

pub struct Tab {
    id: TabId,
    kind: TabKind,
    title: String,
    address: String,
    status: TabStatus,
    page_load_status: PageLoadStatus,
    lifecycle_state: TabLifeCycleState,
    last_interaction: Instant,
    is_playing_audio: bool,
    failed: bool,
    tls_error: bool,
    engine: Option<Engine>,
}

impl Tab {
    /// Создает вкладку с веб-движком (веб-страница или домашняя страница).
    pub fn new_web(
        id: TabId,
        engine: Engine,
        initial_title: impl Into<String>,
        is_probe: bool,
    ) -> Self {
        Self {
            id,
            kind: if is_probe {
                TabKind::Probe
            } else {
                TabKind::Web
            },
            title: initial_title.into(),
            address: String::new(),
            status: TabStatus::Waiting,
            page_load_status: PageLoadStatus::None,
            lifecycle_state: TabLifeCycleState::Active,
            last_interaction: Instant::now(),
            is_playing_audio: false,
            failed: false,
            tls_error: false,
            engine: Some(engine),
        }
    }

    /// Создает домашнюю вкладку со строкой поиска.
    pub fn new_home(id: TabId, engine: Engine, title: impl Into<String>) -> Self {
        Self {
            id,
            kind: TabKind::Home,
            title: title.into(),
            address: "light://home".to_string(),
            status: TabStatus::Finished,
            page_load_status: PageLoadStatus::None,
            lifecycle_state: TabLifeCycleState::Active,
            last_interaction: Instant::now(),
            is_playing_audio: false,
            failed: false,
            tls_error: false,
            engine: Some(engine),
        }
    }

    /// Создает внутреннюю системную вкладку (Настройки, Модули, Диагностика).
    pub fn new_internal(id: TabId, kind: TabKind, title: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            title: title.into(),
            address: match kind {
                TabKind::Settings => "about:settings".to_string(),
                TabKind::Modules => "about:modules".to_string(),
                TabKind::Performance => "light://performance".to_string(),
                _ => String::new(),
            },
            status: TabStatus::Finished,
            page_load_status: PageLoadStatus::None,
            lifecycle_state: TabLifeCycleState::Active,
            last_interaction: Instant::now(),
            is_playing_audio: false,
            failed: false,
            tls_error: false,
            engine: None,
        }
    }

    pub fn id(&self) -> TabId {
        self.id
    }

    pub fn kind(&self) -> TabKind {
        self.kind
    }

    pub fn is_internal(&self) -> bool {
        matches!(
            self.kind,
            TabKind::Settings | TabKind::Modules | TabKind::Performance
        )
    }

    pub fn lifecycle_state(&self) -> TabLifeCycleState {
        self.lifecycle_state
    }

    pub fn set_lifecycle_state(&mut self, state: TabLifeCycleState) {
        self.lifecycle_state = state;
    }

    pub fn is_sleeping(&self) -> bool {
        self.lifecycle_state == TabLifeCycleState::Sleeping
    }

    pub fn last_interaction(&self) -> Instant {
        self.last_interaction
    }

    pub fn touch_interaction(&mut self) {
        self.last_interaction = Instant::now();
    }

    pub fn is_playing_audio(&self) -> bool {
        self.is_playing_audio
    }

    pub fn set_is_playing_audio(&mut self, playing: bool) {
        self.is_playing_audio = playing;
    }

    /// Переводит вкладку в спящий режим.
    pub fn sleep(&mut self) {
        self.lifecycle_state = TabLifeCycleState::Sleeping;
        if let Some(engine) = &self.engine {
            engine.load_blank();
        }
    }

    /// Пробуждает вкладку и восстанавливает навигацию на сохранённый адрес.
    pub fn wake(&mut self) {
        self.lifecycle_state = TabLifeCycleState::Waking;
        self.touch_interaction();
        if let Some(engine) = &self.engine
            && !self.address.is_empty()
            && self.address != "light://home"
        {
            let _ = engine.navigate(&self.address);
        }
        self.lifecycle_state = TabLifeCycleState::Active;
    }

    pub fn is_probe(&self) -> bool {
        self.kind == TabKind::Probe
    }

    pub fn set_is_probe(&mut self, is_probe: bool) {
        if is_probe {
            self.kind = TabKind::Probe;
        } else if self.kind == TabKind::Probe {
            self.kind = TabKind::Web;
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: String) {
        if !title.trim().is_empty() {
            self.title = title;
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn set_address(&mut self, address: String) {
        self.address = address;
    }

    pub fn status(&self) -> TabStatus {
        self.status
    }

    pub fn status_text(&self, lang: Language) -> &'static str {
        self.status.message(lang)
    }

    pub fn set_status(&mut self, status: TabStatus) {
        self.status = status;
    }

    pub fn page_load_status(&self) -> PageLoadStatus {
        self.page_load_status
    }

    pub fn set_page_load_status(&mut self, s: PageLoadStatus) {
        self.page_load_status = s;
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    pub fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    pub fn tls_error(&self) -> bool {
        self.tls_error
    }

    pub fn set_tls_error(&mut self, error: bool) {
        self.tls_error = error;
    }

    /// Начало навигации: сбрасывает устаревшие ошибки TLS и статусы загрузки.
    pub fn start_navigation(&mut self, address: String) {
        self.tls_error = false;
        self.failed = false;
        self.address = address;
        self.status = TabStatus::Navigating;
        self.page_load_status = PageLoadStatus::Loading;
        self.lifecycle_state = TabLifeCycleState::Active;
        self.touch_interaction();
        if self.kind == TabKind::Probe || self.kind == TabKind::Home {
            self.kind = TabKind::Web;
        }
    }

    pub fn on_load_started(&mut self) {
        self.tls_error = false;
        self.failed = false;
        self.status = TabStatus::Loading;
        self.page_load_status = PageLoadStatus::Loading;
        self.touch_interaction();
    }

    pub fn on_load_finished(&mut self) {
        if !self.failed && !self.tls_error {
            self.status = TabStatus::Finished;
            self.page_load_status = PageLoadStatus::Loaded;
        }
    }

    pub fn on_load_failed(&mut self) {
        self.failed = true;
        self.status = TabStatus::Failed;
        self.page_load_status = PageLoadStatus::Failed;
    }

    pub fn on_load_cancelled(&mut self) {
        self.status = TabStatus::Cancelled;
        self.page_load_status = PageLoadStatus::Cancelled;
    }

    pub fn on_tls_error(&mut self) {
        self.tls_error = true;
        self.failed = true;
        self.status = TabStatus::Failed;
        self.page_load_status = PageLoadStatus::Failed;
    }

    pub fn connection_security(&self) -> ConnectionSecurity {
        match self.kind {
            TabKind::Settings
            | TabKind::Modules
            | TabKind::Performance
            | TabKind::Home
            | TabKind::Probe => ConnectionSecurity::Internal,
            TabKind::Web => {
                // Ошибка TLS имеет абсолютный приоритет
                if self.tls_error {
                    return ConnectionSecurity::TlsError;
                }

                // Во время загрузки состояние защищенности еще не подтверждено
                if self.page_load_status == PageLoadStatus::Loading {
                    return ConnectionSecurity::Loading;
                }

                let addr = self.address.to_lowercase();
                if addr.starts_with("https://") {
                    ConnectionSecurity::Https
                } else if addr.starts_with("http://") {
                    ConnectionSecurity::Http
                } else if addr.is_empty() || addr == "about:blank" {
                    ConnectionSecurity::Internal
                } else {
                    ConnectionSecurity::Unknown
                }
            }
        }
    }

    pub fn engine(&self) -> Option<&Engine> {
        self.engine.as_ref()
    }
}

pub struct TabManager {
    tabs: Vec<Tab>,
    active_id: Option<TabId>,
    next_id: u64,
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_id: None,
            next_id: 1,
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn tabs_mut(&mut self) -> &mut [Tab] {
        &mut self.tabs
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.active_id
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_id.and_then(|id| self.get(id))
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        let active_id = self.active_id?;
        self.get_mut(active_id)
    }

    pub fn get(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id == id)
    }

    pub fn get_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id == id)
    }

    pub fn reserve_id(&mut self) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Ищет уже открытую вкладку определенного типа (например, Settings или Modules).
    pub fn find_by_kind(&self, kind: TabKind) -> Option<TabId> {
        self.tabs.iter().find(|t| t.kind == kind).map(|t| t.id)
    }

    pub fn insert_existing_tab(&mut self, tab: Tab) {
        let id = tab.id();
        self.tabs.push(tab);
        self.active_id = Some(id);
    }

    pub fn insert_tab(
        &mut self,
        id: TabId,
        engine: Engine,
        initial_title: impl Into<String>,
        is_probe: bool,
    ) {
        let tab = Tab::new_web(id, engine, initial_title, is_probe);
        self.tabs.push(tab);
        self.active_id = Some(id);
    }

    pub fn insert_internal_tab(
        &mut self,
        id: TabId,
        kind: TabKind,
        initial_title: impl Into<String>,
    ) {
        let tab = Tab::new_internal(id, kind, initial_title);
        self.tabs.push(tab);
        self.active_id = Some(id);
    }

    pub fn switch_to(&mut self, id: TabId) -> bool {
        if self.tabs.iter().any(|t| t.id == id) {
            self.active_id = Some(id);
            true
        } else {
            false
        }
    }

    /// Закрывает вкладку с указанным id.
    /// Возвращает `Some(removed_tab)` если вкладка найдена.
    /// Если закрыта активная вкладка, `active_id` автоматически переключается на соседнюю.
    pub fn close_tab(&mut self, id: TabId) -> Option<Tab> {
        let index = self.tabs.iter().position(|t| t.id == id)?;
        let was_active = self.active_id == Some(id);
        let removed = self.tabs.remove(index);

        if was_active {
            if self.tabs.is_empty() {
                self.active_id = None;
            } else if index < self.tabs.len() {
                self.active_id = Some(self.tabs[index].id);
            } else {
                self.active_id = Some(self.tabs[index - 1].id);
            }
        }

        Some(removed)
    }

    pub fn count(&self) -> usize {
        self.tabs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_tabs_creation_and_search() {
        let mut manager = TabManager::new();
        assert_eq!(manager.count(), 0);

        let id1 = manager.reserve_id();
        manager.insert_internal_tab(id1, TabKind::Settings, "Настройки");
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active_id(), Some(id1));
        assert_eq!(manager.find_by_kind(TabKind::Settings), Some(id1));
        assert_eq!(manager.find_by_kind(TabKind::Modules), None);

        let id2 = manager.reserve_id();
        manager.insert_internal_tab(id2, TabKind::Modules, "Модули");
        assert_eq!(manager.count(), 2);
        assert_eq!(manager.active_id(), Some(id2));
        assert_eq!(manager.find_by_kind(TabKind::Modules), Some(id2));

        let tab_settings = manager.get(id1).unwrap();
        assert_eq!(
            tab_settings.connection_security(),
            ConnectionSecurity::Internal
        );
        assert!(tab_settings.is_internal());
        assert!(!tab_settings.is_probe());
    }

    #[test]
    fn test_tab_navigation_and_security_state_transitions() {
        let id = TabId(1);
        let mut tab = Tab::new_internal(id, TabKind::Web, "Test");

        // 1. Исходное состояние
        assert_eq!(tab.connection_security(), ConnectionSecurity::Internal);

        // 2. Старт навигации по HTTPS: состояние переходит в Loading
        tab.start_navigation("https://example.com".to_string());
        assert_eq!(tab.page_load_status(), PageLoadStatus::Loading);
        assert_eq!(tab.connection_security(), ConnectionSecurity::Loading);

        // 3. Успешное завершение загрузки: состояние HTTPS
        tab.on_load_finished();
        assert_eq!(tab.page_load_status(), PageLoadStatus::Loaded);
        assert_eq!(tab.connection_security(), ConnectionSecurity::Https);

        // 4. Ошибка TLS на HTTPS: статус TLS Error с наивысшим приоритетом
        tab.on_tls_error();
        assert_eq!(tab.connection_security(), ConnectionSecurity::TlsError);
        assert_eq!(tab.page_load_status(), PageLoadStatus::Failed);

        // 5. Новая навигация сбрасывает ошибку TLS!
        tab.start_navigation("https://secure.site".to_string());
        assert!(!tab.tls_error());
        assert!(!tab.failed());
        assert_eq!(tab.connection_security(), ConnectionSecurity::Loading);

        // 6. Переход на HTTP
        tab.start_navigation("http://insecure.site".to_string());
        tab.on_load_finished();
        assert_eq!(tab.connection_security(), ConnectionSecurity::Http);
        assert_eq!(tab.page_load_status(), PageLoadStatus::Loaded);

        // 7. Обычная ошибка загрузки без ошибки TLS:
        // статус Failed, но индикатор безопасности показывает HTTP, а не TLS Error!
        tab.on_load_failed();
        assert_eq!(tab.page_load_status(), PageLoadStatus::Failed);
        assert_eq!(tab.connection_security(), ConnectionSecurity::Http);

        // 8. Отмена загрузки
        tab.start_navigation("https://slow.site".to_string());
        tab.on_load_cancelled();
        assert_eq!(tab.page_load_status(), PageLoadStatus::Cancelled);
    }

    #[test]
    fn test_tabs_switching_and_closing_order() {
        let mut manager = TabManager::new();
        let id1 = manager.reserve_id();
        manager.insert_internal_tab(id1, TabKind::Settings, "T1");
        let id2 = manager.reserve_id();
        manager.insert_internal_tab(id2, TabKind::Modules, "T2");
        let id3 = manager.reserve_id();
        manager.insert_internal_tab(id3, TabKind::Settings, "T3");

        assert_eq!(manager.active_id(), Some(id3));

        // Переключение на T2
        assert!(manager.switch_to(id2));
        assert_eq!(manager.active_id(), Some(id2));

        // Закрытие активной T2 -> переключается на T3
        let closed2 = manager.close_tab(id2);
        assert!(closed2.is_some());
        assert_eq!(manager.active_id(), Some(id3));
        assert_eq!(manager.count(), 2);

        // Закрытие активной T3 (последней в списке) -> переключается на T1
        let closed3 = manager.close_tab(id3);
        assert!(closed3.is_some());
        assert_eq!(manager.active_id(), Some(id1));
        assert_eq!(manager.count(), 1);

        // Закрытие последней оставшейся вкладки
        let closed1 = manager.close_tab(id1);
        assert!(closed1.is_some());
        assert_eq!(manager.active_id(), None);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_page_load_status_messages() {
        assert_eq!(PageLoadStatus::None.message(Language::Ru), None);
        assert_eq!(
            PageLoadStatus::Loading.message(Language::Ru),
            Some("Страница загружается…")
        );
        assert_eq!(
            PageLoadStatus::Loaded.message(Language::Ru),
            Some("Страница загружена")
        );
        assert_eq!(
            PageLoadStatus::Failed.message(Language::Ru),
            Some("Не удалось загрузить страницу")
        );
        assert_eq!(
            PageLoadStatus::Cancelled.message(Language::Ru),
            Some("Загрузка прервана")
        );

        assert_eq!(
            PageLoadStatus::Loading.message(Language::En),
            Some("Page loading…")
        );
    }
}
