pub mod modules_page;
pub mod settings_page;
pub mod style;

use std::sync::Arc;

use relm4::{gtk, gtk::gdk, gtk::glib, gtk::prelude::*, prelude::*};

use self::modules_page::ModulesPageWidgets;
use self::settings_page::SettingsPageWidgets;
use crate::config::{Config, ThemeMode};
use crate::engine::{ConnectionSecurity, Engine, EngineEvent};
use crate::i18n::Language;
use crate::modules::ModuleRegistry;
use crate::performance::{DiagnosticsSnapshot, PageLoadTimer, PerformanceCounters};
use crate::searchengine::{NavigationTarget, SearchEngineConfig, SearchProvider, parse_input};
use crate::shortcuts::{BrowserAction, standard_commands};
use crate::tab_manager::{SleepMode, TabManagerModuleConfig, TabPolicyEngine, TabSleepCandidate};
use crate::tabs::{PageLoadStatus, Tab, TabId, TabKind, TabManager, TabStatus};

pub struct Probe {
    tabs: TabManager,
    config: Config,
    network_session: Option<webkit6::NetworkSession>,
    tab_box: gtk::Box,
    stack: gtk::Stack,
    address_entry: gtk::Entry,
    css_provider: gtk::CssProvider,
    settings_widgets: Option<SettingsPageWidgets>,
    modules_widgets: Option<ModulesPageWidgets>,
    error_revealer: gtk::Revealer,
    error_label: gtk::Label,
    page_status_label: gtk::Label,
    performance_counters: Arc<PerformanceCounters>,
    page_timer: PageLoadTimer,
    palette_window: Option<gtk::Window>,
}

#[derive(Debug)]
pub enum Message {
    Navigate(String),
    Back,
    Forward,
    Reload,
    NewTab,
    OpenSettings,
    OpenModules,
    OpenPerformance,
    SwitchTab(TabId),
    CloseTab(TabId),
    SleepTab(TabId),
    WakeTab(TabId),
    ChangeLanguage(Language),
    ChangeTheme(ThemeMode),
    ChangeAccentColor(String),
    ToggleCompactMode(bool),
    ToggleTransparency(bool),
    ChangeSleepMode(SleepMode),
    ChangeSearchProvider(SearchProvider),
    TogglePageStatus(bool),
    ToggleModule(String, bool),
    ChangeTransparency(f64),
    DismissSaveError,
    OpenCommandPalette,
    CloseCommandPalette,
    ExecuteAction(BrowserAction),
    Engine(TabId, EngineEvent),
}

impl Probe {
    fn can_go_back(&self) -> bool {
        self.tabs
            .active_tab()
            .and_then(|tab| tab.engine())
            .is_some_and(|engine| engine.can_go_back())
    }

    fn can_go_forward(&self) -> bool {
        self.tabs
            .active_tab()
            .and_then(|tab| tab.engine())
            .is_some_and(|engine| engine.can_go_forward())
    }

    fn is_web_tab_active(&self) -> bool {
        self.tabs.active_tab().is_some_and(|t| !t.is_internal())
    }

    fn active_security(&self) -> ConnectionSecurity {
        self.tabs
            .active_tab()
            .map_or(ConnectionSecurity::Internal, |tab| {
                tab.connection_security()
            })
    }

    fn active_page_status_text(&self) -> String {
        if !self.config.show_page_status {
            return String::new();
        }

        self.tabs
            .active_tab()
            .and_then(|t| t.page_load_status().message(self.config.language))
            .unwrap_or("")
            .to_string()
    }

    fn apply_theme_and_styles(&self) {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(matches!(
                self.config.theme,
                ThemeMode::Dark
            ));
        }

        let css = style::generate_css(&self.config);
        self.css_provider.load_from_string(&css);
    }

    fn handle_config_save_result(&mut self, res: Result<(), std::io::Error>) {
        match res {
            Ok(()) => {
                self.error_revealer.set_reveal_child(false);
            }
            Err(e) => {
                eprintln!("Ошибка сохранения конфигурации: {}", e);
                self.error_label
                    .set_text(self.config.language.save_error_msg());
                self.error_revealer.set_reveal_child(true);
            }
        }
    }

    /// Создает домашнюю вкладку `light://home` с поисковой строкой.
    fn create_home_tab(&mut self, sender: &ComponentSender<Self>) -> TabId {
        let tab_id = self.tabs.reserve_id();
        let input = sender.input_sender().clone();
        let engine = Engine::new(self.network_session.as_ref(), move |event| {
            input.emit(Message::Engine(tab_id, event));
        });

        engine.load_home(
            self.config.language,
            self.config.theme,
            &self.config.accent_color,
            self.config.module_searchengine_enabled,
            self.config.search_provider.display_name(),
            self.config.transparency_enabled,
            self.config.transparency,
        );

        let tab_widget = engine.widget();
        let tab = Tab::new_home(tab_id, engine, self.config.language.home_tab_title());
        self.tabs.insert_existing_tab(tab);

        let id_str = tab_id.0.to_string();
        self.stack.add_named(&tab_widget, Some(&id_str));
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text("");
        self.sync_tab_bar(sender);
        tab_id
    }

    /// Создаёт web-вкладку с немедленной загрузкой адреса, не перезаписывая её домашней страницей.
    fn create_web_tab(&mut self, sender: &ComponentSender<Self>, initial_url: &str) -> TabId {
        let tab_id = self.tabs.reserve_id();
        let input = sender.input_sender().clone();
        let engine = Engine::new(self.network_session.as_ref(), move |event| {
            input.emit(Message::Engine(tab_id, event));
        });

        if let Err(error) = engine.navigate(initial_url) {
            eprintln!(
                "Ошибка навигации web-вкладки: {}",
                error.message(self.config.language)
            );
        }

        let tab_widget = engine.widget();
        let mut tab = Tab::new_web(
            tab_id,
            engine,
            self.config.language.default_tab_title(),
            false,
        );
        tab.start_navigation(initial_url.to_string());
        self.tabs.insert_existing_tab(tab);

        let id_str = tab_id.0.to_string();
        self.stack.add_named(&tab_widget, Some(&id_str));
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text(initial_url);
        self.sync_tab_bar(sender);
        tab_id
    }

    fn open_settings_tab(&mut self, sender: &ComponentSender<Self>) {
        if let Some(existing_id) = self.tabs.find_by_kind(TabKind::Settings) {
            self.tabs.switch_to(existing_id);
            self.stack
                .set_visible_child_name(&existing_id.0.to_string());
            self.address_entry.set_text("about:settings");
            self.sync_tab_bar(sender);
            return;
        }

        let tab_id = self.tabs.reserve_id();
        let settings_widgets = SettingsPageWidgets::new(&self.config, sender);
        let id_str = tab_id.0.to_string();
        self.stack
            .add_named(&settings_widgets.container, Some(&id_str));

        self.tabs.insert_internal_tab(
            tab_id,
            TabKind::Settings,
            self.config.language.settings_tab_title(),
        );

        self.settings_widgets = Some(settings_widgets);
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text("about:settings");
        self.sync_tab_bar(sender);
    }

    fn open_modules_tab(&mut self, sender: &ComponentSender<Self>) {
        if let Some(existing_id) = self.tabs.find_by_kind(TabKind::Modules) {
            self.tabs.switch_to(existing_id);
            self.stack
                .set_visible_child_name(&existing_id.0.to_string());
            self.address_entry.set_text("about:modules");
            self.sync_tab_bar(sender);
            return;
        }

        let tab_id = self.tabs.reserve_id();
        let modules_widgets = ModulesPageWidgets::new(&self.config, sender);
        let id_str = tab_id.0.to_string();
        self.stack
            .add_named(&modules_widgets.container, Some(&id_str));

        self.tabs.insert_internal_tab(
            tab_id,
            TabKind::Modules,
            self.config.language.modules_tab_title(),
        );

        self.modules_widgets = Some(modules_widgets);
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text("about:modules");
        self.sync_tab_bar(sender);
    }

    fn open_performance_tab(&mut self, sender: &ComponentSender<Self>) {
        if let Some(existing_id) = self.tabs.find_by_kind(TabKind::Performance) {
            self.tabs.switch_to(existing_id);
            self.stack
                .set_visible_child_name(&existing_id.0.to_string());
            self.address_entry.set_text("light://performance");
            self.sync_tab_bar(sender);
            return;
        }

        let tab_id = self.tabs.reserve_id();
        let input = sender.input_sender().clone();
        let engine = Engine::new(self.network_session.as_ref(), move |event| {
            input.emit(Message::Engine(tab_id, event));
        });

        let snapshot = DiagnosticsSnapshot {
            total_tabs: self.tabs.count(),
            active_tabs: self.tabs.tabs().iter().filter(|t| !t.is_sleeping()).count(),
            sleeping_tabs: self.tabs.tabs().iter().filter(|t| t.is_sleeping()).count(),
            memory_rss_mb: DiagnosticsSnapshot::read_process_rss_mb(),
            last_load_duration_ms: self.page_timer.last_duration(),
            load_errors_count: self
                .performance_counters
                .load_errors
                .load(std::sync::atomic::Ordering::Relaxed),
            tls_errors_count: self
                .performance_counters
                .tls_errors
                .load(std::sync::atomic::Ordering::Relaxed),
            process_crashes_count: self
                .performance_counters
                .process_crashes
                .load(std::sync::atomic::Ordering::Relaxed),
        };

        engine.load_performance(
            self.config.module_performance_enabled,
            Some(&snapshot),
            self.config.language,
            self.config.theme,
            &self.config.accent_color,
        );

        let tab_widget = engine.widget();
        let mut tab = Tab::new_internal(
            tab_id,
            TabKind::Performance,
            self.config.language.module_performance_name(),
        );
        tab.set_address("light://performance".to_string());
        self.tabs.insert_existing_tab(tab);

        let id_str = tab_id.0.to_string();
        self.stack.add_named(&tab_widget, Some(&id_str));
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text("light://performance");
        self.sync_tab_bar(sender);
    }

    /// Проверяет кандидатов на усыпление согласно политике модуля `tab_manager`.
    fn evaluate_tab_sleep_policy(&mut self) {
        if !self.config.module_tab_manager_enabled
            || self.config.tab_sleep_mode == SleepMode::Manual
        {
            return;
        }

        let active_id = self.tabs.active_id();
        let candidates: Vec<TabSleepCandidate> = self
            .tabs
            .tabs()
            .iter()
            .map(|t| TabSleepCandidate {
                id: t.id(),
                is_active: Some(t.id()) == active_id,
                is_internal: t.is_internal(),
                is_loading: t.page_load_status() == PageLoadStatus::Loading,
                is_playing_audio: t.is_playing_audio(),
                state: t.lifecycle_state(),
                last_interaction: t.last_interaction(),
            })
            .collect();

        let to_sleep = TabPolicyEngine::evaluate_candidates(
            &candidates,
            &TabManagerModuleConfig {
                enabled: self.config.module_tab_manager_enabled,
                mode: self.config.tab_sleep_mode,
                idle_timeout_minutes: self.config.tab_idle_timeout_minutes,
                max_active_tabs: self.config.tab_max_active_tabs,
            },
            std::time::Instant::now(),
        );

        for id in to_sleep {
            if let Some(tab) = self.tabs.get_mut(id) {
                tab.sleep();
            }
        }
    }

    fn sync_tab_bar(&self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.tab_box.first_child() {
            self.tab_box.remove(&child);
        }

        let active_id = self.tabs.active_id();

        for tab in self.tabs.tabs() {
            let tab_id = tab.id();
            let is_active = Some(tab_id) == active_id;

            let item_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            item_box.add_css_class("tab-item-box");
            item_box.set_valign(gtk::Align::Center);
            if is_active {
                item_box.add_css_class("tab-item-active");
            }
            item_box.set_cursor_from_name(Some("pointer"));

            // Индикатор сна
            if let Some(badge) = tab.lifecycle_state().badge_symbol() {
                let badge_label = gtk::Label::new(Some(badge));
                badge_label.add_css_class("tab-sleep-badge");
                badge_label.set_valign(gtk::Align::Center);
                item_box.append(&badge_label);
            }

            let title_str = match tab.kind() {
                TabKind::Settings => self.config.language.settings_tab_title().to_string(),
                TabKind::Modules => self.config.language.modules_tab_title().to_string(),
                TabKind::Performance => self.config.language.module_performance_name().to_string(),
                TabKind::Home => self.config.language.home_tab_title().to_string(),
                TabKind::Probe => self.config.language.default_tab_title().to_string(),
                TabKind::Web => tab.title().to_string(),
            };

            let title_label = gtk::Label::new(Some(&title_str));
            title_label.add_css_class("tab-title-label");
            title_label.set_ellipsize(relm4::gtk::pango::EllipsizeMode::End);
            title_label.set_max_width_chars(18);
            title_label.set_valign(gtk::Align::Center);
            title_label.set_halign(gtk::Align::Start);

            let click_gesture = gtk::GestureClick::new();
            let input_sender = sender.input_sender().clone();
            click_gesture.connect_released(move |_, _, _, _| {
                input_sender.emit(Message::SwitchTab(tab_id));
            });
            item_box.add_controller(click_gesture);

            let close_btn = gtk::Button::with_label("✕");
            close_btn.add_css_class("flat");
            close_btn.add_css_class("tab-close-btn");
            close_btn.set_valign(gtk::Align::Center);
            close_btn.set_halign(gtk::Align::Center);
            close_btn.set_tooltip_text(Some(self.config.language.close_tab_tooltip()));

            let input_sender = sender.input_sender().clone();
            close_btn.connect_clicked(move |_| {
                input_sender.emit(Message::CloseTab(tab_id));
            });

            item_box.append(&title_label);
            item_box.append(&close_btn);

            self.tab_box.append(&item_box);
        }

        // Кнопка добавления вкладки `+`
        let add_btn = gtk::Button::new();
        add_btn.add_css_class("tab-add-btn");
        add_btn.set_size_request(28, 28);
        add_btn.set_valign(gtk::Align::Center);
        add_btn.set_halign(gtk::Align::Center);
        add_btn.set_tooltip_text(Some(self.config.language.new_tab_tooltip()));
        let add_icon = gtk::Label::new(Some("+"));
        add_icon.add_css_class("tab-add-icon");
        add_icon.set_valign(gtk::Align::Center);
        add_icon.set_halign(gtk::Align::Center);
        add_btn.set_child(Some(&add_icon));
        let input_sender = sender.input_sender().clone();
        add_btn.connect_clicked(move |_| {
            input_sender.emit(Message::NewTab);
        });
        self.tab_box.append(&add_btn);
    }
}

#[relm4::component(pub)]
impl SimpleComponent for Probe {
    type Init = ();
    type Input = Message;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Light"),
            set_default_size: (980, 660),
            add_css_class: "main-window",

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                // Верхняя панель: компактные вкладки + кнопки действий
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 3,
                    set_margin_start: 4,
                    set_margin_end: 4,
                    set_margin_top: 2,
                    set_margin_bottom: 1,
                    add_css_class: "header-panel",

                    gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Never),

                        #[local_ref]
                        tab_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 3,
                        },
                    },

                    // Кнопка палитры команд (Ctrl+Shift+P)
                    gtk::Button {
                        set_icon_name: "system-search-symbolic",
                        #[watch]
                        set_tooltip_text: Some(model.config.language.command_palette_title()),
                        #[watch]
                        set_visible: model.config.module_shortcuts_enabled,
                        set_valign: gtk::Align::Center,
                        add_css_class: "flat",
                        connect_clicked => Message::OpenCommandPalette,
                    },

                    // Кнопка открытия вкладки модулей
                    gtk::Button {
                        set_icon_name: "application-x-addon-symbolic",
                        #[watch]
                        set_tooltip_text: Some(model.config.language.modules_tooltip()),
                        set_valign: gtk::Align::Center,
                        add_css_class: "flat",
                        connect_clicked => Message::OpenModules,
                    },

                    // Кнопка открытия вкладки настроек
                    gtk::Button {
                        set_icon_name: "emblem-system-symbolic",
                        #[watch]
                        set_tooltip_text: Some(model.config.language.settings_tooltip()),
                        set_valign: gtk::Align::Center,
                        add_css_class: "flat",
                        connect_clicked => Message::OpenSettings,
                    },
                },

                // Панель навигации с индикатором безопасности
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_margin_start: 8,
                    set_margin_end: 8,
                    set_margin_top: 4,
                    set_margin_bottom: 4,
                    add_css_class: "navigation-panel",

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.back_label(),
                        #[watch]
                        set_sensitive: model.can_go_back(),
                        set_valign: gtk::Align::Center,
                        add_css_class: "nav-btn",
                        connect_clicked => Message::Back,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.forward_label(),
                        #[watch]
                        set_sensitive: model.can_go_forward(),
                        set_valign: gtk::Align::Center,
                        add_css_class: "nav-btn",
                        connect_clicked => Message::Forward,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.reload_label(),
                        #[watch]
                        set_sensitive: model.is_web_tab_active(),
                        set_valign: gtk::Align::Center,
                        add_css_class: "nav-btn",
                        connect_clicked => Message::Reload,
                    },

                    // Индикатор безопасности
                    gtk::Image {
                        #[watch]
                        set_icon_name: Some(model.active_security().icon_name()),
                        #[watch]
                        set_tooltip_text: Some(model.active_security().tooltip(model.config.language)),
                        set_valign: gtk::Align::Center,
                        set_margin_start: 4,
                        set_margin_end: 2,
                    },

                    #[local_ref]
                    address_entry -> gtk::Entry {
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        add_css_class: "address-entry",
                        #[watch]
                        set_placeholder_text: Some(model.config.language.address_placeholder()),
                        connect_activate[sender] => move |entry| {
                            sender.input(Message::Navigate(entry.text().to_string()));
                        },
                    },
                },

                // Баннер ошибок и подсказок
                #[local_ref]
                error_revealer -> gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 8,
                        add_css_class: "error-banner",

                        #[local_ref]
                        error_label -> gtk::Label {
                            set_hexpand: true,
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            set_label: "✕",
                            add_css_class: "flat",
                            connect_clicked => Message::DismissSaveError,
                        },
                    },
                },

                // Индикатор состояния страницы
                #[local_ref]
                page_status_label -> gtk::Label {
                    #[watch]
                    set_label: &model.active_page_status_text(),
                    #[watch]
                    set_visible: model.config.show_page_status && !model.active_page_status_text().is_empty(),
                    set_wrap: true,
                    set_margin_bottom: 4,
                    add_css_class: "dim-label",
                },

                #[local_ref]
                stack -> gtk::Stack {
                    set_hexpand: true,
                    set_vexpand: true,
                },
            },
        }
    }

    fn init(
        (): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let network_session = match Engine::init_persistent_storage() {
            Ok(session) => Some(session),
            Err(e) => {
                eprintln!("Ошибка инициализации постоянного хранилища: {}", e);
                None
            }
        };

        let config = Config::load();
        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let stack = gtk::Stack::new();
        let address_entry = gtk::Entry::new();

        let css_provider = gtk::CssProvider::new();
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
        }

        let error_label = gtk::Label::new(None);
        let error_revealer = gtk::Revealer::new();
        let page_status_label = gtk::Label::new(None);

        let mut model = Self {
            tabs: TabManager::new(),
            config,
            network_session,
            tab_box,
            stack,
            address_entry,
            css_provider,
            settings_widgets: None,
            modules_widgets: None,
            error_revealer,
            error_label,
            page_status_label,
            performance_counters: Arc::new(PerformanceCounters::new()),
            page_timer: PageLoadTimer::new(),
            palette_window: None,
        };

        model.apply_theme_and_styles();
        // Создаем домашнюю вкладку при старте
        model.create_home_tab(&sender);

        let tab_box = &model.tab_box;
        let stack = &model.stack;
        let address_entry = &model.address_entry;
        let error_revealer = &model.error_revealer;
        let error_label = &model.error_label;
        let page_status_label = &model.page_status_label;

        // Контроллер горячих клавиш (Ctrl+Shift+P, Ctrl+T, Ctrl+W, Ctrl+R, Ctrl+L)
        let key_controller = gtk::EventControllerKey::new();
        let s_keys = sender.input_sender().clone();
        key_controller.connect_key_pressed(move |_, keyval, _, modifier| {
            let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);
            let shift = modifier.contains(gdk::ModifierType::SHIFT_MASK);

            if ctrl && shift && (keyval == gdk::Key::P || keyval == gdk::Key::p) {
                s_keys.emit(Message::OpenCommandPalette);
                return glib::Propagation::Stop;
            }
            if ctrl && (keyval == gdk::Key::t || keyval == gdk::Key::T) {
                s_keys.emit(Message::NewTab);
                return glib::Propagation::Stop;
            }
            if ctrl && (keyval == gdk::Key::r || keyval == gdk::Key::R) {
                s_keys.emit(Message::Reload);
                return glib::Propagation::Stop;
            }

            glib::Propagation::Proceed
        });
        root.add_controller(key_controller);

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Message, sender: ComponentSender<Self>) {
        match message {
            Message::Navigate(input) => {
                let parse_cfg = SearchEngineConfig {
                    enabled: self.config.module_searchengine_enabled,
                    provider: self.config.search_provider,
                };

                match parse_input(&input, &parse_cfg) {
                    Ok(NavigationTarget::Internal(uri)) => {
                        self.error_revealer.set_reveal_child(false);
                        match uri.as_str() {
                            "about:settings" => self.open_settings_tab(&sender),
                            "about:modules" => self.open_modules_tab(&sender),
                            "light://performance" => self.open_performance_tab(&sender),
                            "light://home" => {
                                self.create_home_tab(&sender);
                            }
                            _ => {}
                        }
                    }
                    Ok(NavigationTarget::Url(url)) | Ok(NavigationTarget::Search { url, .. }) => {
                        self.error_revealer.set_reveal_child(false);
                        if let Some(tab) = self.tabs.active_tab_mut() {
                            if tab.is_internal() {
                                // Если текущая вкладка внутренняя (настройки/модули), открываем URL в новой вкладке
                                self.create_web_tab(&sender, &url);
                            } else if let Some(engine) = tab.engine() {
                                let _ = engine.navigate(&url);
                                tab.start_navigation(url.clone());
                                self.address_entry.set_text(&url);
                            }
                        }
                    }
                    Err(crate::searchengine::SearchEngineError::SearchDisabled) => {
                        self.error_label
                            .set_text(self.config.language.search_disabled_hint());
                        self.error_revealer.set_reveal_child(true);
                    }
                    Err(err) => {
                        self.error_label.set_text(err.message(self.config.language));
                        self.error_revealer.set_reveal_child(true);
                    }
                }
            }
            Message::Back => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.on_load_started();
                    if let Some(engine) = tab.engine() {
                        engine.go_back();
                    }
                }
            }
            Message::Forward => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.on_load_started();
                    if let Some(engine) = tab.engine() {
                        engine.go_forward();
                    }
                }
            }
            Message::Reload => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.on_load_started();
                    let is_probe = tab.is_probe();
                    let kind = tab.kind();
                    if let Some(engine) = tab.engine() {
                        if kind == TabKind::Home {
                            engine.load_home(
                                self.config.language,
                                self.config.theme,
                                &self.config.accent_color,
                                self.config.module_searchengine_enabled,
                                self.config.search_provider.display_name(),
                                self.config.transparency_enabled,
                                self.config.transparency,
                            );
                        } else if is_probe {
                            engine.load_probe(
                                self.config.language,
                                self.config.theme,
                                &self.config.accent_color,
                            );
                        } else {
                            engine.reload();
                        }
                    }
                }
            }
            Message::NewTab => {
                self.create_home_tab(&sender);
                self.evaluate_tab_sleep_policy();
            }
            Message::OpenSettings => {
                self.open_settings_tab(&sender);
            }
            Message::OpenModules => {
                self.open_modules_tab(&sender);
            }
            Message::OpenPerformance => {
                self.open_performance_tab(&sender);
            }
            Message::SwitchTab(id) => {
                if self.tabs.switch_to(id) {
                    if let Some(tab) = self.tabs.get_mut(id) {
                        if tab.is_sleeping() {
                            tab.wake();
                        } else {
                            tab.touch_interaction();
                        }
                        self.stack.set_visible_child_name(&id.0.to_string());
                        self.address_entry.set_text(tab.address());
                    }
                    self.sync_tab_bar(&sender);
                    self.evaluate_tab_sleep_policy();
                }
            }
            Message::CloseTab(id) => {
                if let Some(removed) = self.tabs.close_tab(id) {
                    let id_str = id.0.to_string();
                    if let Some(child) = self.stack.child_by_name(&id_str) {
                        self.stack.remove(&child);
                    }

                    if removed.kind() == TabKind::Settings {
                        self.settings_widgets = None;
                    } else if removed.kind() == TabKind::Modules {
                        self.modules_widgets = None;
                    }

                    if self.tabs.is_empty() {
                        self.create_home_tab(&sender);
                    } else if let Some(active_id) = self.tabs.active_id() {
                        self.stack.set_visible_child_name(&active_id.0.to_string());
                        if let Some(tab) = self.tabs.active_tab() {
                            self.address_entry.set_text(tab.address());
                        }
                        self.sync_tab_bar(&sender);
                    }
                }
            }
            Message::SleepTab(id) => {
                let active_id = self.tabs.active_id();
                if let Some(tab) = self.tabs.get_mut(id)
                    && !tab.is_internal()
                    && Some(id) != active_id
                {
                    tab.sleep();
                    self.sync_tab_bar(&sender);
                }
            }
            Message::WakeTab(id) => {
                if let Some(tab) = self.tabs.get_mut(id) {
                    tab.wake();
                    self.sync_tab_bar(&sender);
                }
            }
            Message::ChangeLanguage(new_lang) => {
                if self.config.language != new_lang {
                    self.config.language = new_lang;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);

                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    if let Some(w) = self.modules_widgets.as_mut() {
                        w.update(&self.config);
                    }

                    self.sync_tab_bar(&sender);
                }
            }
            Message::ChangeTheme(new_theme) => {
                if self.config.theme != new_theme {
                    self.config.theme = new_theme;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);

                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    if let Some(w) = self.modules_widgets.as_mut() {
                        w.update(&self.config);
                    }

                    self.apply_theme_and_styles();
                    self.sync_tab_bar(&sender);
                }
            }
            Message::ChangeAccentColor(hex) => {
                if self.config.accent_color != hex {
                    self.config.accent_color = hex;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);

                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    if let Some(w) = self.modules_widgets.as_mut() {
                        w.update(&self.config);
                    }

                    self.apply_theme_and_styles();
                    self.sync_tab_bar(&sender);
                }
            }
            Message::ToggleCompactMode(enabled) => {
                if self.config.compact_mode != enabled {
                    self.config.compact_mode = enabled;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);
                    self.apply_theme_and_styles();
                }
            }
            Message::ToggleTransparency(enabled) => {
                if self.config.transparency_enabled != enabled {
                    self.config.transparency_enabled = enabled;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);
                    self.apply_theme_and_styles();
                }
            }
            Message::ChangeSleepMode(mode) => {
                if self.config.tab_sleep_mode != mode {
                    self.config.tab_sleep_mode = mode;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);
                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    self.evaluate_tab_sleep_policy();
                }
            }
            Message::ChangeSearchProvider(provider) => {
                if self.config.search_provider != provider {
                    self.config.search_provider = provider;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);
                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                }
            }
            Message::TogglePageStatus(enabled) => {
                if self.config.show_page_status != enabled {
                    self.config.show_page_status = enabled;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);
                    if let Some(w) = self.settings_widgets.as_mut() {
                        w.update(&self.config);
                    }
                }
            }
            Message::ToggleModule(module_id, enabled) => {
                if ModuleRegistry::try_set_enabled(&mut self.config, &module_id, enabled).is_ok() {
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);

                    if let Some(w) = self.modules_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    self.apply_theme_and_styles();
                }
            }
            Message::ChangeTransparency(val) => {
                if (self.config.transparency - val).abs() > 0.001 {
                    self.config.transparency = val;
                    let save_res = self.config.save();
                    self.handle_config_save_result(save_res);

                    if let Some(w) = self.modules_widgets.as_mut() {
                        w.update(&self.config);
                    }
                    self.apply_theme_and_styles();
                }
            }
            Message::DismissSaveError => {
                self.error_revealer.set_reveal_child(false);
            }
            Message::OpenCommandPalette => {
                if !self.config.module_shortcuts_enabled {
                    return;
                }

                let pal_win = gtk::Window::new();
                pal_win.set_title(Some(self.config.language.command_palette_title()));
                pal_win.set_default_size(480, 320);
                pal_win.add_css_class("command-palette-window");

                let box_content = gtk::Box::new(gtk::Orientation::Vertical, 8);
                box_content.set_margin_all(12);

                let search_entry = gtk::Entry::new();
                search_entry
                    .set_placeholder_text(Some(self.config.language.command_palette_placeholder()));
                box_content.append(&search_entry);

                let list_box = gtk::ListBox::new();
                list_box.add_css_class("command-list");

                let commands = standard_commands(self.config.language);
                for cmd in &commands {
                    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                    row.add_css_class("command-item");
                    let title = gtk::Label::new(Some(&cmd.title));
                    title.set_hexpand(true);
                    title.set_halign(gtk::Align::Start);
                    row.append(&title);

                    if let Some(hint) = cmd.shortcut_hint {
                        let hint_lbl = gtk::Label::new(Some(hint));
                        hint_lbl.add_css_class("dim-label");
                        row.append(&hint_lbl);
                    }
                    list_box.append(&row);
                }

                let scroll = gtk::ScrolledWindow::new();
                scroll.set_hexpand(true);
                scroll.set_vexpand(true);
                scroll.set_child(Some(&list_box));
                box_content.append(&scroll);

                pal_win.set_child(Some(&box_content));
                pal_win.present();
                self.palette_window = Some(pal_win);
            }
            Message::CloseCommandPalette => {
                if let Some(w) = self.palette_window.take() {
                    w.close();
                }
            }
            Message::ExecuteAction(action) => {
                self.palette_window = None;
                match action {
                    BrowserAction::NewTab => sender.input(Message::NewTab),
                    BrowserAction::CloseTab => {
                        if let Some(id) = self.tabs.active_id() {
                            sender.input(Message::CloseTab(id));
                        }
                    }
                    BrowserAction::Reload => sender.input(Message::Reload),
                    BrowserAction::GoBack => sender.input(Message::Back),
                    BrowserAction::GoForward => sender.input(Message::Forward),
                    BrowserAction::FocusAddressBar => {
                        self.address_entry.grab_focus();
                    }
                    BrowserAction::OpenSettings => sender.input(Message::OpenSettings),
                    BrowserAction::OpenModules => sender.input(Message::OpenModules),
                    BrowserAction::OpenPerformance => sender.input(Message::OpenPerformance),
                    BrowserAction::ToggleTheme => {
                        let next = match self.config.theme {
                            ThemeMode::Light => ThemeMode::Dark,
                            ThemeMode::Dark => ThemeMode::Light,
                        };
                        sender.input(Message::ChangeTheme(next));
                    }
                    BrowserAction::ToggleLanguage => {
                        let next = self.config.language.toggle();
                        sender.input(Message::ChangeLanguage(next));
                    }
                    BrowserAction::SwitchToTab(id) => sender.input(Message::SwitchTab(id)),
                }
            }
            Message::Engine(tab_id, event) => {
                let is_active = self.tabs.active_id() == Some(tab_id);
                if let Some(tab) = self.tabs.get_mut(tab_id) {
                    match event {
                        EngineEvent::Started => {
                            self.page_timer.start();
                            tab.on_load_started();
                        }
                        EngineEvent::Finished => {
                            self.page_timer.finish();
                            tab.on_load_finished();
                        }
                        EngineEvent::Failed { failing_uri: _ } => {
                            self.performance_counters.record_load_error();
                            tab.on_load_failed();
                        }
                        EngineEvent::Cancelled { failing_uri: _ } => {
                            tab.on_load_cancelled();
                        }
                        EngineEvent::TlsError { failing_uri: _ } => {
                            self.performance_counters.record_tls_error();
                            tab.on_tls_error();
                        }
                        EngineEvent::ProcessTerminated => {
                            self.performance_counters.record_process_crash();
                            tab.set_failed(true);
                            tab.set_status(TabStatus::ProcessTerminated);
                            tab.set_page_load_status(PageLoadStatus::Failed);
                        }
                        EngineEvent::TitleChanged(title) => {
                            tab.set_title(title);
                            self.sync_tab_bar(&sender);
                        }
                        EngineEvent::UriChanged(uri) => {
                            let is_web_uri =
                                uri.starts_with("http://") || uri.starts_with("https://");
                            if is_web_uri {
                                // WebKit может прислать отложенный URI старого документа при
                                // смене активной вкладки; такой сигнал не должен отменять новую навигацию.
                                tab.set_address(uri.clone());
                                if is_active {
                                    self.address_entry.set_text(&uri);
                                }
                            }
                        }
                        EngineEvent::SubmitInput(input) => {
                            sender.input(Message::Navigate(input));
                        }
                    }
                }
            }
        }
    }
}
