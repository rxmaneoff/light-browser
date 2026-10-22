use crate::engine::Engine;
use crate::i18n::{AddressError, Language};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStatus {
    Waiting,
    Navigating,
    Loading,
    Finished,
    Failed,
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
            Self::ProcessTerminated => lang.status_process_terminated(),
            Self::Error(err) => err.message(lang),
        }
    }
}

pub struct Tab {
    id: TabId,
    title: String,
    address: String,
    status: TabStatus,
    failed: bool,
    is_probe: bool,
    engine: Engine,
}

impl Tab {
    pub fn new(
        id: TabId,
        engine: Engine,
        initial_title: impl Into<String>,
        is_probe: bool,
    ) -> Self {
        Self {
            id,
            title: initial_title.into(),
            address: String::new(),
            status: TabStatus::Waiting,
            failed: false,
            is_probe,
            engine,
        }
    }

    pub fn id(&self) -> TabId {
        self.id
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

    #[allow(dead_code)]
    pub fn status(&self) -> TabStatus {
        self.status
    }

    pub fn status_text(&self, lang: Language) -> &'static str {
        self.status.message(lang)
    }

    pub fn set_status(&mut self, status: TabStatus) {
        self.status = status;
    }

    pub fn failed(&self) -> bool {
        self.failed
    }

    pub fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    pub fn is_probe(&self) -> bool {
        self.is_probe
    }

    pub fn set_is_probe(&mut self, is_probe: bool) {
        self.is_probe = is_probe;
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
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

    pub fn insert_tab(
        &mut self,
        id: TabId,
        engine: Engine,
        initial_title: impl Into<String>,
        is_probe: bool,
    ) {
        let tab = Tab::new(id, engine, initial_title, is_probe);
        self.tabs.push(tab);
        self.active_id = Some(id);
    }

    #[allow(dead_code)]
    pub fn add_tab(
        &mut self,
        engine: Engine,
        initial_title: impl Into<String>,
        is_probe: bool,
    ) -> TabId {
        let id = self.reserve_id();
        self.insert_tab(id, engine, initial_title, is_probe);
        id
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

    #[allow(dead_code)]
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
    fn test_tab_manager_navigation_logic() {
        if relm4::gtk::init().is_err() {
            return;
        }

        let mut manager = TabManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.active_tab().is_none());

        let engine1 = Engine::new(|_| {});
        let id1 = manager.add_tab(engine1, "Вкладка 1", false);
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active_id(), Some(id1));

        let engine2 = Engine::new(|_| {});
        let id2 = manager.add_tab(engine2, "Вкладка 2", false);
        assert_eq!(manager.count(), 2);
        assert_eq!(manager.active_id(), Some(id2));

        // Переключение назад на вкладку 1
        assert!(manager.switch_to(id1));
        assert_eq!(manager.active_id(), Some(id1));

        // Закрытие неактивной вкладки 2
        let closed = manager.close_tab(id2);
        assert!(closed.is_some());
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active_id(), Some(id1));

        // Закрытие последней вкладки
        let closed_last = manager.close_tab(id1);
        assert!(closed_last.is_some());
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.active_id(), None);

        // Проверка reserve_id и insert_tab
        let res_id = manager.reserve_id();
        let engine3 = Engine::new(|_| {});
        manager.insert_tab(res_id, engine3, "Вкладка 3", true);
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.active_id(), Some(res_id));
        assert_eq!(manager.active_tab().unwrap().title(), "Вкладка 3");
        assert!(manager.active_tab().unwrap().is_probe());
    }

    #[test]
    fn test_tab_manager_active_switch_on_close() {
        if relm4::gtk::init().is_err() {
            return;
        }

        let mut manager = TabManager::new();
        let id1 = manager.add_tab(Engine::new(|_| {}), "T1", false);
        let id2 = manager.add_tab(Engine::new(|_| {}), "T2", false);
        let id3 = manager.add_tab(Engine::new(|_| {}), "T3", false);

        assert!(manager.switch_to(id2));
        assert_eq!(manager.active_id(), Some(id2));

        assert!(manager.close_tab(id2).is_some());
        assert_eq!(manager.active_id(), Some(id3));
        assert_eq!(manager.count(), 2);

        assert!(manager.close_tab(id3).is_some());
        assert_eq!(manager.active_id(), Some(id1));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_tab_metadata_mutation() {
        if relm4::gtk::init().is_err() {
            return;
        }

        let mut manager = TabManager::new();
        let id = manager.add_tab(Engine::new(|_| {}), "Начальный", true);
        let tab = manager.get_mut(id).unwrap();

        assert_eq!(tab.title(), "Начальный");
        tab.set_title("Новый заголовок".to_owned());
        assert_eq!(tab.title(), "Новый заголовок");

        tab.set_address("https://example.com".to_owned());
        assert_eq!(tab.address(), "https://example.com");

        tab.set_status(TabStatus::Finished);
        assert_eq!(tab.status_text(Language::Ru), "Загрузка завершена.");
        assert_eq!(tab.status_text(Language::En), "Loading completed.");

        assert!(!tab.failed());
        tab.set_failed(true);
        assert!(tab.failed());

        assert!(tab.is_probe());
        tab.set_is_probe(false);
        assert!(!tab.is_probe());
    }
}
