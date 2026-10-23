use relm4::{gtk, gtk::gdk, gtk::prelude::*, prelude::*};

use crate::config::{Config, ThemeMode};
use crate::engine::{ConnectionSecurity, Engine, EngineEvent};
use crate::i18n::Language;
use crate::tabs::{TabId, TabManager, TabStatus};

struct PresetButtonInfo {
    hex: &'static str,
    button: gtk::Button,
    swatch: gtk::Box,
}

struct SettingsWidgets {
    lang_label: gtk::Label,
    theme_label: gtk::Label,
    light_btn: gtk::Button,
    dark_btn: gtk::Button,
    accent_label: gtk::Label,
    ru_btn: gtk::Button,
    en_btn: gtk::Button,
    color_btn: gtk::ColorDialogButton,
    preset_buttons: Vec<PresetButtonInfo>,
}

pub struct Probe {
    tabs: TabManager,
    config: Config,
    tab_box: gtk::Box,
    stack: gtk::Stack,
    address_entry: gtk::Entry,
    css_provider: gtk::CssProvider,
    settings_widgets: SettingsWidgets,
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
    ChangeLanguage(Language),
    ChangeTheme(ThemeMode),
    ChangeAccentColor(String),
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
            .map_or("", |tab| tab.status_text(self.config.language))
    }

    fn active_security(&self) -> ConnectionSecurity {
        self.tabs
            .active_tab()
            .map_or(ConnectionSecurity::Internal, |tab| {
                tab.connection_security()
            })
    }

    fn update_settings_labels(&self) {
        self.settings_widgets
            .lang_label
            .set_text(self.config.language.language_section());
        self.settings_widgets
            .theme_label
            .set_text(self.config.language.theme_section());
        self.settings_widgets
            .light_btn
            .set_label(self.config.language.theme_light());
        self.settings_widgets
            .dark_btn
            .set_label(self.config.language.theme_dark());
        self.settings_widgets
            .accent_label
            .set_text(self.config.language.accent_section());
        self.settings_widgets
            .color_btn
            .set_tooltip_text(Some(self.config.language.custom_color_tooltip()));

        for info in &self.settings_widgets.preset_buttons {
            let tooltip = match info.hex {
                "#2563eb" => self.config.language.preset_blue_tooltip(),
                "#7c3aed" => self.config.language.preset_purple_tooltip(),
                "#059669" => self.config.language.preset_emerald_tooltip(),
                "#d97706" => self.config.language.preset_amber_tooltip(),
                "#e11d48" => self.config.language.preset_rose_tooltip(),
                _ => "",
            };
            if !tooltip.is_empty() {
                info.button.set_tooltip_text(Some(tooltip));
            }
        }

        self.update_settings_active_states();
    }

    fn update_settings_active_states(&self) {
        if self.config.language == Language::Ru {
            self.settings_widgets
                .ru_btn
                .add_css_class("settings-active-btn");
            self.settings_widgets
                .ru_btn
                .remove_css_class("suggested-action");
            self.settings_widgets
                .en_btn
                .remove_css_class("settings-active-btn");
            self.settings_widgets
                .en_btn
                .remove_css_class("suggested-action");
        } else {
            self.settings_widgets
                .ru_btn
                .remove_css_class("settings-active-btn");
            self.settings_widgets
                .ru_btn
                .remove_css_class("suggested-action");
            self.settings_widgets
                .en_btn
                .add_css_class("settings-active-btn");
            self.settings_widgets
                .en_btn
                .remove_css_class("suggested-action");
        }

        if self.config.theme == ThemeMode::Light {
            self.settings_widgets
                .light_btn
                .add_css_class("settings-active-btn");
            self.settings_widgets
                .light_btn
                .remove_css_class("suggested-action");
            self.settings_widgets
                .dark_btn
                .remove_css_class("settings-active-btn");
            self.settings_widgets
                .dark_btn
                .remove_css_class("suggested-action");
        } else {
            self.settings_widgets
                .light_btn
                .remove_css_class("settings-active-btn");
            self.settings_widgets
                .light_btn
                .remove_css_class("suggested-action");
            self.settings_widgets
                .dark_btn
                .add_css_class("settings-active-btn");
            self.settings_widgets
                .dark_btn
                .remove_css_class("suggested-action");
        }

        let current_accent = self.config.accent_color.to_lowercase();
        for info in &self.settings_widgets.preset_buttons {
            while let Some(child) = info.swatch.first_child() {
                info.swatch.remove(&child);
            }

            if info.hex.eq_ignore_ascii_case(&current_accent) {
                info.button.add_css_class("preset-selected");
                let check_label = gtk::Label::new(Some("✓"));
                check_label.add_css_class("preset-check-label");
                check_label.set_valign(gtk::Align::Center);
                check_label.set_halign(gtk::Align::Center);
                info.swatch.append(&check_label);
            } else {
                info.button.remove_css_class("preset-selected");
            }
        }

        if let Ok(rgba) = gdk::RGBA::parse(&self.config.accent_color)
            && self.settings_widgets.color_btn.rgba() != rgba
        {
            self.settings_widgets.color_btn.set_rgba(&rgba);
        }
    }

    fn apply_theme_and_styles(&self) {
        let (
            bg,
            fg,
            surface,
            border,
            tab_inactive_bg,
            tab_inactive_hover,
            tab_inactive_border,
            btn_inactive_bg,
            btn_inactive_border,
            btn_inactive_hover,
        ) = match self.config.theme {
            ThemeMode::Light => (
                "#f6f8fa",
                "#1f2328",
                "#ffffff",
                "#d0d7de",
                "rgba(0, 0, 0, 0.04)",
                "rgba(0, 0, 0, 0.08)",
                "rgba(0, 0, 0, 0.10)",
                "#f0f2f5",
                "#d0d7de",
                "#e4e7eb",
            ),
            ThemeMode::Dark => (
                "#1c1c1f",
                "#f0f2f5",
                "#28282d",
                "#3c3c43",
                "rgba(255, 255, 255, 0.06)",
                "rgba(255, 255, 255, 0.12)",
                "rgba(255, 255, 255, 0.12)",
                "#35353c",
                "#484852",
                "#404048",
            ),
        };

        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(matches!(
                self.config.theme,
                ThemeMode::Dark
            ));
        }

        let css = format!(
            r#"
            @define-color accent_color {accent};
            @define-color accent_bg_color {accent};
            @define-color accent_fg_color #ffffff;
            @define-color theme_selected_bg_color {accent};
            @define-color theme_selected_fg_color #ffffff;

            window, .main-window {{
                background-color: {bg};
                color: {fg};
            }}

            /* Меню настроек строго соответствует теме оформления */
            popover,
            popover > contents,
            popover.popover-menu,
            popover.popover-menu > contents,
            .popover-menu,
            .popover-menu > contents {{
                background-color: {surface};
                background: {surface};
                color: {fg};
                border: 1px solid {border};
                border-radius: 12px;
                padding: 10px;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.25);
            }}

            popover > arrow,
            popover.popover-menu > arrow {{
                background-color: {surface};
                background: {surface};
                border-color: {border};
            }}

            /* Базовые стили контейнера вкладки (монолитная форма капсулы) */
            .tab-item-box {{
                border-radius: 20px;
                border: 1px solid {tab_inactive_border};
                background-color: {tab_inactive_bg};
                padding: 0 8px 0 14px;
                margin: 0;
                min-height: 32px;
                transition: background-color 150ms ease, border-color 150ms ease, box-shadow 150ms ease;
            }}
            .tab-item-box:hover {{
                background-color: {tab_inactive_hover};
            }}

            /* Активная вкладка: монолитная заливка акцентным цветом всей капсулы */
            .tab-item-box.tab-item-active {{
                background-color: {accent};
                border-color: {accent};
                box-shadow: 0 1px 3px rgba(0, 0, 0, 0.20);
            }}

            /* Текст заголовка вкладки (без отдельного виджета кнопки и фона) */
            .tab-title-label {{
                color: {fg};
                font-size: 13px;
                margin-right: 6px;
            }}
            .tab-item-active .tab-title-label {{
                color: #ffffff;
                font-weight: 600;
            }}

            /* Кнопка закрытия вкладки (✕) */
            .tab-close-btn,
            button.tab-close-btn,
            button.flat.tab-close-btn {{
                background: transparent;
                background-color: transparent;
                background-image: none;
                border: none;
                outline: none;
                box-shadow: none;
                color: {fg};
                opacity: 0.0;
                padding: 0;
                margin: 0;
                min-height: 20px;
                min-width: 20px;
                border-radius: 9999px;
                transition: opacity 160ms ease, color 140ms ease;
            }}
            .tab-item-box:hover .tab-close-btn {{
                opacity: 0.75;
            }}
            .tab-item-box.tab-item-active .tab-close-btn,
            .tab-item-box.tab-item-active button.tab-close-btn {{
                color: #ffffff;
                opacity: 0.85;
            }}
            .tab-close-btn:hover {{
                opacity: 1.0;
            }}
            .tab-item-box.tab-item-active .tab-close-btn:hover {{
                opacity: 1.0;
            }}

            /* Кнопка создания новой вкладки `+` (квадратные пропорции со скруглением) */
            .tab-add-btn {{
                border-radius: 8px;
                border: 1px solid {tab_inactive_border};
                background-color: {tab_inactive_bg};
                color: {fg};
                padding: 0;
                min-height: 32px;
                min-width: 32px;
                margin-left: 4px;
                font-size: 16px;
                font-weight: 500;
                transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
            }}
            .tab-add-btn:hover {{
                background-color: {tab_inactive_hover};
                border-color: {accent};
                color: {accent};
            }}

            /* Неактивные кнопки в меню настроек (фон строго соответствует теме) */
            button.settings-btn,
            .settings-btn,
            popover button.settings-btn,
            popover contents button.settings-btn {{
                border-radius: 8px;
                padding: 8px 16px;
                font-size: 13px;
                background-color: {btn_inactive_bg};
                background: {btn_inactive_bg};
                background-image: none;
                border: 1px solid {btn_inactive_border};
                border-color: {btn_inactive_border};
                color: {fg};
                box-shadow: none;
                transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
            }}

            button.settings-btn:hover,
            .settings-btn:hover,
            popover button.settings-btn:hover,
            popover contents button.settings-btn:hover {{
                background-color: {btn_inactive_hover};
                background: {btn_inactive_hover};
                border: 1px solid {border};
                border-color: {border};
                color: {fg};
            }}

            /* Активные кнопки в меню настроек (строго соответствуют выбранному акцентному цвету) */
            button.settings-btn.settings-active-btn,
            button.settings-btn.suggested-action,
            button.settings-active-btn,
            .settings-active-btn,
            popover button.settings-active-btn,
            popover contents button.settings-active-btn,
            popover button.suggested-action,
            popover contents button.suggested-action,
            .accent-btn {{
                background-color: {accent};
                background: {accent};
                background-image: none;
                border: 1px solid {accent};
                border-color: {accent};
                color: #ffffff;
                font-weight: 600;
                box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
            }}

            button.settings-btn.settings-active-btn:hover,
            button.settings-btn.suggested-action:hover,
            button.settings-active-btn:hover,
            .settings-active-btn:hover,
            popover button.settings-active-btn:hover,
            popover contents button.settings-active-btn:hover,
            popover button.suggested-action:hover,
            popover contents button.suggested-action:hover {{
                background-color: {accent};
                background: {accent};
                background-image: none;
                border: 1px solid {accent};
                border-color: {accent};
                color: #ffffff;
                filter: brightness(1.12);
            }}

            /* Быстрые пресеты акцентных цветов: наглядные круглые образцы */
            button.color-preset-btn,
            popover button.color-preset-btn,
            popover contents button.color-preset-btn {{
                border-radius: 9999px;
                min-width: 28px;
                min-height: 28px;
                padding: 0;
                margin: 0;
                border: 2px solid {border};
                background: transparent;
                box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
                transition: transform 120ms ease, box-shadow 120ms ease, border-color 120ms ease;
            }}

            button.color-preset-btn:hover,
            popover button.color-preset-btn:hover,
            popover contents button.color-preset-btn:hover {{
                transform: scale(1.15);
                border-color: {fg};
                box-shadow: 0 2px 6px rgba(0, 0, 0, 0.30);
            }}

            button.color-preset-btn.preset-selected,
            popover button.color-preset-btn.preset-selected,
            popover contents button.color-preset-btn.preset-selected {{
                border: 2px solid #ffffff;
                box-shadow: 0 0 0 2px {fg}, 0 2px 6px rgba(0, 0, 0, 0.40);
            }}

            .preset-swatch {{
                border-radius: 9999px;
                min-width: 24px;
                min-height: 24px;
            }}

            .swatch-blue {{
                background-color: #2563eb;
            }}

            .swatch-purple {{
                background-color: #7c3aed;
            }}

            .swatch-emerald {{
                background-color: #059669;
            }}

            .swatch-amber {{
                background-color: #d97706;
            }}

            .swatch-rose {{
                background-color: #e11d48;
            }}

            .preset-check-label {{
                color: #ffffff;
                font-size: 13px;
                font-weight: 900;
                margin: 0;
                padding: 0;
                text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
            }}

            button.color-dialog-btn,
            popover button.color-dialog-btn,
            popover contents button.color-dialog-btn {{
                border-radius: 9999px;
                min-width: 28px;
                min-height: 28px;
                padding: 0;
                border: 2px solid {border};
            }}
            "#,
            bg = bg,
            fg = fg,
            surface = surface,
            border = border,
            tab_inactive_bg = tab_inactive_bg,
            tab_inactive_hover = tab_inactive_hover,
            tab_inactive_border = tab_inactive_border,
            btn_inactive_bg = btn_inactive_bg,
            btn_inactive_border = btn_inactive_border,
            btn_inactive_hover = btn_inactive_hover,
            accent = self.config.accent_color,
        );

        self.css_provider.load_from_string(&css);
    }

    fn create_tab(&mut self, sender: &ComponentSender<Self>) -> TabId {
        let tab_id = self.tabs.reserve_id();
        let input = sender.input_sender().clone();
        let engine = Engine::new(move |event| {
            input.emit(Message::Engine(tab_id, event));
        });
        engine.load_probe(
            self.config.language,
            self.config.theme,
            &self.config.accent_color,
        );
        let tab_widget = engine.widget();
        self.tabs.insert_tab(
            tab_id,
            engine,
            self.config.language.default_tab_title(),
            true,
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
            item_box.add_css_class("tab-item-box");
            if is_active {
                item_box.add_css_class("tab-item-active");
            }
            item_box.set_cursor_from_name(Some("pointer"));

            let title = if tab.is_probe() {
                self.config.language.default_tab_title()
            } else {
                tab.title()
            };

            let title_label = gtk::Label::new(Some(title));
            title_label.add_css_class("tab-title-label");
            title_label.set_ellipsize(relm4::gtk::pango::EllipsizeMode::End);
            title_label.set_max_width_chars(20);
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
            close_btn.set_size_request(20, 20);
            if let Some(label) = close_btn.child().and_downcast::<gtk::Label>() {
                label.set_valign(gtk::Align::Center);
                label.set_halign(gtk::Align::Center);
            }
            close_btn.set_tooltip_text(Some(self.config.language.close_tab_tooltip()));

            let input_sender = sender.input_sender().clone();
            close_btn.connect_clicked(move |_| {
                input_sender.emit(Message::CloseTab(tab_id));
            });

            item_box.append(&title_label);
            item_box.append(&close_btn);

            self.tab_box.append(&item_box);
        }

        // Кнопка добавления вкладки `+` непосредственно после последней вкладки
        let add_btn = gtk::Button::with_label("+");
        add_btn.add_css_class("tab-add-btn");
        add_btn.set_valign(gtk::Align::Center);
        add_btn.set_halign(gtk::Align::Center);
        add_btn.set_size_request(32, 32);
        if let Some(label) = add_btn.child().and_downcast::<gtk::Label>() {
            label.set_valign(gtk::Align::Center);
            label.set_halign(gtk::Align::Center);
        }
        add_btn.set_tooltip_text(Some(self.config.language.new_tab_tooltip()));
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

                // Панель вкладок и правая кнопка меню настроек
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
                    set_margin_start: 4,
                    set_margin_end: 4,
                    set_margin_top: 3,
                    set_margin_bottom: 2,

                    gtk::ScrolledWindow {
                        set_hexpand: true,
                        set_policy: (gtk::PolicyType::Automatic, gtk::PolicyType::Never),

                        #[local_ref]
                        tab_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 4,
                        },
                    },

                    // Шестерёнка настроек в правом верхнем углу
                    #[name(settings_menubutton)]
                    gtk::MenuButton {
                        set_icon_name: "emblem-system-symbolic",
                        #[watch]
                        set_tooltip_text: Some(model.config.language.settings_tooltip()),
                        set_valign: gtk::Align::Center,
                    },
                },

                // Панель навигации с индикатором безопасности
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_margin_start: 8,
                    set_margin_end: 8,
                    set_margin_top: 4,
                    set_margin_bottom: 6,

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.back_label(),
                        #[watch]
                        set_sensitive: model.can_go_back(),
                        connect_clicked => Message::Back,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.forward_label(),
                        #[watch]
                        set_sensitive: model.can_go_forward(),
                        connect_clicked => Message::Forward,
                    },

                    gtk::Button {
                        #[watch]
                        set_label: model.config.language.reload_label(),
                        connect_clicked => Message::Reload,
                    },

                    // Индикатор безопасности HTTP / HTTPS / Internal
                    gtk::Image {
                        #[watch]
                        set_icon_name: Some(model.active_security().icon_name()),
                        #[watch]
                        set_tooltip_text: Some(model.active_security().tooltip(model.config.language)),
                        set_margin_start: 4,
                        set_margin_end: 2,
                    },

                    #[local_ref]
                    address_entry -> gtk::Entry {
                        set_hexpand: true,
                        #[watch]
                        set_placeholder_text: Some(model.config.language.address_placeholder()),
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
        Engine::init_persistent_storage();

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

        let lang_label = gtk::Label::new(None);
        lang_label.set_halign(gtk::Align::Start);
        lang_label.add_css_class("heading");

        let ru_btn = gtk::Button::with_label("Русский");
        ru_btn.add_css_class("settings-btn");
        let en_btn = gtk::Button::with_label("English");
        en_btn.add_css_class("settings-btn");

        let theme_label = gtk::Label::new(None);
        theme_label.set_halign(gtk::Align::Start);
        theme_label.add_css_class("heading");

        let light_btn = gtk::Button::new();
        light_btn.add_css_class("settings-btn");
        let dark_btn = gtk::Button::new();
        dark_btn.add_css_class("settings-btn");

        let accent_label = gtk::Label::new(None);
        accent_label.set_halign(gtk::Align::Start);
        accent_label.add_css_class("heading");

        let presets = [
            ("swatch-blue", "#2563eb"),
            ("swatch-purple", "#7c3aed"),
            ("swatch-emerald", "#059669"),
            ("swatch-amber", "#d97706"),
            ("swatch-rose", "#e11d48"),
        ];

        let mut preset_buttons = Vec::new();
        for (cls, hex) in presets {
            let p_btn = gtk::Button::new();
            p_btn.add_css_class("color-preset-btn");
            p_btn.set_valign(gtk::Align::Center);
            p_btn.set_cursor_from_name(Some("pointer"));
            p_btn.set_size_request(28, 28);

            let swatch = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            swatch.add_css_class("preset-swatch");
            swatch.add_css_class(cls);
            swatch.set_valign(gtk::Align::Center);
            swatch.set_halign(gtk::Align::Center);
            swatch.set_size_request(24, 24);
            p_btn.set_child(Some(&swatch));

            let s_color = sender.input_sender().clone();
            let hex_str = hex.to_string();
            p_btn.connect_clicked(move |_| {
                s_color.emit(Message::ChangeAccentColor(hex_str.clone()));
            });
            preset_buttons.push(PresetButtonInfo {
                hex,
                button: p_btn,
                swatch,
            });
        }

        let color_btn = gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new()));
        color_btn.add_css_class("color-dialog-btn");
        color_btn.set_valign(gtk::Align::Center);
        if let Ok(rgba) = gdk::RGBA::parse(&config.accent_color) {
            color_btn.set_rgba(&rgba);
        }

        let s_color = sender.input_sender().clone();
        color_btn.connect_rgba_notify(move |btn| {
            let rgba = btn.rgba();
            let hex = format!(
                "#{:02x}{:02x}{:02x}",
                (rgba.red() * 255.0).round() as u8,
                (rgba.green() * 255.0).round() as u8,
                (rgba.blue() * 255.0).round() as u8
            );
            s_color.emit(Message::ChangeAccentColor(hex));
        });

        let settings_widgets = SettingsWidgets {
            lang_label,
            theme_label,
            light_btn,
            dark_btn,
            accent_label,
            ru_btn,
            en_btn,
            color_btn,
            preset_buttons,
        };

        let mut model = Self {
            tabs: TabManager::new(),
            config,
            tab_box,
            stack,
            address_entry,
            css_provider,
            settings_widgets,
        };

        model.apply_theme_and_styles();
        model.update_settings_labels();
        model.create_tab(&sender);

        let tab_box = &model.tab_box;
        let stack = &model.stack;
        let address_entry = &model.address_entry;

        let widgets = view_output!();

        // Поповер настроек
        let popover = gtk::Popover::new();
        popover.add_css_class("popover-menu");

        let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        popover_box.set_margin_all(8);

        // 1. Язык
        popover_box.append(&model.settings_widgets.lang_label);

        let lang_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let s_ru = sender.input_sender().clone();
        model
            .settings_widgets
            .ru_btn
            .connect_clicked(move |_| s_ru.emit(Message::ChangeLanguage(Language::Ru)));

        let s_en = sender.input_sender().clone();
        model
            .settings_widgets
            .en_btn
            .connect_clicked(move |_| s_en.emit(Message::ChangeLanguage(Language::En)));

        lang_box.append(&model.settings_widgets.ru_btn);
        lang_box.append(&model.settings_widgets.en_btn);
        popover_box.append(&lang_box);

        popover_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // 2. Тема оформления
        popover_box.append(&model.settings_widgets.theme_label);

        let theme_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let s_light = sender.input_sender().clone();
        model
            .settings_widgets
            .light_btn
            .connect_clicked(move |_| s_light.emit(Message::ChangeTheme(ThemeMode::Light)));

        let s_dark = sender.input_sender().clone();
        model
            .settings_widgets
            .dark_btn
            .connect_clicked(move |_| s_dark.emit(Message::ChangeTheme(ThemeMode::Dark)));

        theme_box.append(&model.settings_widgets.light_btn);
        theme_box.append(&model.settings_widgets.dark_btn);
        popover_box.append(&theme_box);

        popover_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // 3. Акцентный цвет
        popover_box.append(&model.settings_widgets.accent_label);

        let colors_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        colors_box.set_valign(gtk::Align::Center);

        for info in &model.settings_widgets.preset_buttons {
            colors_box.append(&info.button);
        }

        colors_box.append(&model.settings_widgets.color_btn);
        popover_box.append(&colors_box);

        popover.set_child(Some(&popover_box));
        widgets.settings_menubutton.set_popover(Some(&popover));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Message, sender: ComponentSender<Self>) {
        match message {
            Message::Navigate(address) => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.set_tls_error(false);
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
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.set_tls_error(false);
                    tab.engine().go_back();
                }
            }
            Message::Forward => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.set_tls_error(false);
                    tab.engine().go_forward();
                }
            }
            Message::Reload => {
                if let Some(tab) = self.tabs.active_tab_mut() {
                    tab.set_tls_error(false);
                    if tab.is_probe() {
                        tab.engine().load_probe(
                            self.config.language,
                            self.config.theme,
                            &self.config.accent_color,
                        );
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
            Message::ChangeLanguage(new_lang) => {
                if self.config.language != new_lang {
                    self.config.language = new_lang;
                    let _ = self.config.save();
                    self.update_settings_labels();
                    for tab in self.tabs.tabs_mut() {
                        if tab.is_probe() {
                            tab.engine().load_probe(
                                self.config.language,
                                self.config.theme,
                                &self.config.accent_color,
                            );
                        }
                    }
                    self.sync_tab_bar(&sender);
                }
            }
            Message::ChangeTheme(new_theme) => {
                if self.config.theme != new_theme {
                    self.config.theme = new_theme;
                    let _ = self.config.save();
                    self.update_settings_active_states();
                    self.apply_theme_and_styles();
                    self.sync_tab_bar(&sender);
                    for tab in self.tabs.tabs_mut() {
                        if tab.is_probe() {
                            tab.engine().load_probe(
                                self.config.language,
                                self.config.theme,
                                &self.config.accent_color,
                            );
                        }
                    }
                }
            }
            Message::ChangeAccentColor(hex) => {
                if self.config.accent_color != hex {
                    self.config.accent_color = hex;
                    let _ = self.config.save();
                    self.apply_theme_and_styles();
                    self.update_settings_active_states();
                    self.sync_tab_bar(&sender);
                    for tab in self.tabs.tabs_mut() {
                        if tab.is_probe() {
                            tab.engine().load_probe(
                                self.config.language,
                                self.config.theme,
                                &self.config.accent_color,
                            );
                        }
                    }
                }
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
                        EngineEvent::TlsError => {
                            tab.set_tls_error(true);
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
