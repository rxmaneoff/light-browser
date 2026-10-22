use relm4::{gtk, gtk::prelude::*, prelude::*};

use crate::engine::{Engine, EngineEvent};
use crate::i18n::Language;
use crate::tabs::{TabId, TabManager, TabStatus};

pub struct Probe {
    tabs: TabManager,
    language: Language,
    tab_box: gtk::Box,
    stack: gtk::Stack,
    address_entry: gtk::Entry,
}

#[derive(Debug)]
pub enum Message {
    Navigate(String),
    Back,
    Forward,
    Reload,
    NewTab,
    SwitchTab(TabId),
    CloseTab(TabId),
    ToggleLanguage,
    Engine(TabId, EngineEvent),
}

impl Probe {
    fn can_go_back(&self) -> bool {
        self.tabs
            .active_tab()
            .is_some_and(|tab| tab.engine().can_go_back())
    }

    fn can_go_forward(&self) -> bool {
        self.tabs
            .active_tab()
            .is_some_and(|tab| tab.engine().can_go_forward())
    }

    fn active_status(&self) -> &str {
        self.tabs
            .active_tab()
            .map_or("", |tab| tab.status_text(self.language))
    }

    fn create_tab(&mut self, sender: &ComponentSender<Self>) -> TabId {
        let tab_id = self.tabs.reserve_id();
        let input = sender.input_sender().clone();
        let engine = Engine::new(move |event| {
            input.emit(Message::Engine(tab_id, event));
        });
        engine.load_probe(self.language);
        let tab_widget = engine.widget();
        self.tabs.insert_tab(
            tab_id,
            engine,
            self.language.default_tab_title(),
            true, // начальная встроенная страница
        );

        let id_str = tab_id.0.to_string();
        self.stack.add_named(&tab_widget, Some(&id_str));
        self.stack.set_visible_child_name(&id_str);
        self.address_entry.set_text("");
        self.sync_tab_bar(sender);
        tab_id
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
            item_box.add_css_class("linked");

            let title = if tab.is_probe() {
                self.language.default_tab_title()
            } else {
                tab.title()
            };

            let title_btn = gtk::Button::with_label(title);
            if let Some(label) = title_btn.child().and_downcast::<gtk::Label>() {
                label.set_ellipsize(relm4::gtk::pango::EllipsizeMode::End);
                label.set_max_width_chars(20);
            }

            if is_active {
                title_btn.add_css_class("suggested-action");
            }

            let input_sender = sender.input_sender().clone();
            title_btn.connect_clicked(move |_| {
                input_sender.emit(Message::SwitchTab(tab_id));
            });

            let close_btn = gtk::Button::with_label("✕");
            close_btn.set_tooltip_text(Some(self.language.close_tab_tooltip()));
            if is_active {
                close_btn.add_css_class("suggested-action");
            }

            let input_sender = sender.input_sender().clone();
            close_btn.connect_clicked(move |_| {
                input_sender.emit(Message::CloseTab(tab_id));
            });

            item_box.append(&title_btn);
            item_box.append(&close_btn);

            self.tab_box.append(&item_box);
        }
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
            set_default_size: (960, 640),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                // Панель вкладок и кнопка настроек языка
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
                    set_margin_all: 4,

                    gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Never),

                        #[local_ref]
                        tab_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 4,
                        },
                    },

                    gtk::Button {
                        set_label: "+",
                        #[watch]
                        set_tooltip_text: Some(model.language.new_tab_tooltip()),
                        connect_clicked => Message::NewTab,
                    },

                    // Небольшая кнопка настроек в углу окна для переключения языка
                    gtk::Button {
                        #[watch]
                        set_label: model.language.button_label(),
                        #[watch]
                        set_tooltip_text: Some(model.language.settings_tooltip()),
                        connect_clicked => Message::ToggleLanguage,
                    },
                },

                // Панель навигации
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_margin_all: 8,

                    gtk::Button {
                        #[watch]
                        set_label: model.language.back_label(),
                        #[watch]
                        set_sensitive: model.can_go_back(),
                        connect_clicked => Message::Back,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.language.forward_label(),
                        #[watch]
                        set_sensitive: model.can_go_forward(),
                        connect_clicked => Message::Forward,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.language.reload_label(),
                        connect_clicked => Message::Reload,
                    },

                    #[local_ref]
                    address_entry -> gtk::Entry {
                        set_hexpand: true,
                        #[watch]
                        set_placeholder_text: Some(model.language.address_placeholder()),
                        connect_activate[sender] => move |entry| {
                            sender.input(Message::Navigate(entry.text().to_string()));
                        },
                    },
                },

                gtk::Label {
                    #[watch]
                    set_label: model.active_status(),
                    set_wrap: true,
                    set_margin_bottom: 4,
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
        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let stack = gtk::Stack::new();
        let address_entry = gtk::Entry::new();

        let mut model = Self {
            tabs: TabManager::new(),
            language: Language::Ru,
            tab_box,
            stack,
            address_entry,
        };

        model.create_tab(&sender);

        let tab_box = &model.tab_box;
        let stack = &model.stack;
        let address_entry = &model.address_entry;

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Message, sender: ComponentSender<Self>) {
        match message {
            Message::Navigate(address) => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    match tab.engine().navigate(&address) {
                        Ok(()) => {
                            tab.set_is_probe(false);
                            tab.set_address(address);
                            tab.set_status(TabStatus::Navigating);
                        }
                        Err(err) => {
                            tab.set_status(TabStatus::Error(err));
                        }
                    }
                }
            }
            Message::Back => {
                if let Some(tab) = self.tabs.active_tab() {
                    tab.engine().go_back();
                }
            }
            Message::Forward => {
                if let Some(tab) = self.tabs.active_tab() {
                    tab.engine().go_forward();
                }
            }
            Message::Reload => {
                if let Some(tab) = self.tabs.active_tab() {
                    if tab.is_probe() {
                        tab.engine().load_probe(self.language);
                    } else {
                        tab.engine().reload();
                    }
                }
            }
            Message::NewTab => {
                self.create_tab(&sender);
            }
            Message::SwitchTab(id) => {
                if self.tabs.switch_to(id) {
                    self.stack.set_visible_child_name(&id.0.to_string());
                    if let Some(tab) = self.tabs.active_tab() {
                        self.address_entry.set_text(tab.address());
                    }
                    self.sync_tab_bar(&sender);
                }
            }
            Message::CloseTab(id) => {
                if let Some(removed) = self.tabs.close_tab(id) {
                    self.stack.remove(&removed.engine().widget());

                    if self.tabs.is_empty() {
                        self.create_tab(&sender);
                    } else if let Some(active_id) = self.tabs.active_id() {
                        self.stack.set_visible_child_name(&active_id.0.to_string());
                        if let Some(tab) = self.tabs.active_tab() {
                            self.address_entry.set_text(tab.address());
                        }
                        self.sync_tab_bar(&sender);
                    }
                }
            }
            Message::ToggleLanguage => {
                self.language = self.language.toggle();
                for tab in self.tabs.tabs_mut() {
                    if tab.is_probe() {
                        tab.engine().load_probe(self.language);
                    }
                }
                self.sync_tab_bar(&sender);
            }
            Message::Engine(tab_id, event) => {
                let is_active = self.tabs.active_id() == Some(tab_id);
                if let Some(tab) = self.tabs.get_mut(tab_id) {
                    match event {
                        EngineEvent::Started => {
                            tab.set_failed(false);
                            tab.set_status(TabStatus::Loading);
                        }
                        EngineEvent::Finished => {
                            if !tab.failed() {
                                tab.set_status(TabStatus::Finished);
                            }
                        }
                        EngineEvent::Failed => {
                            tab.set_failed(true);
                            tab.set_status(TabStatus::Failed);
                        }
                        EngineEvent::ProcessTerminated => {
                            tab.set_failed(true);
                            tab.set_status(TabStatus::ProcessTerminated);
                        }
                        EngineEvent::TitleChanged(title) => {
                            tab.set_title(title);
                            self.sync_tab_bar(&sender);
                        }
                        EngineEvent::UriChanged(uri) => {
                            tab.set_address(uri.clone());
                            if is_active {
                                self.address_entry.set_text(&uri);
                            }
                        }
                    }
                }
            }
        }
    }
}
