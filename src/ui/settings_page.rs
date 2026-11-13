use relm4::gtk::{self, gdk, prelude::*};
use relm4::{ComponentSender, RelmWidgetExt};

use super::Probe;
use crate::config::{Config, ThemeMode};
use crate::i18n::Language;
use crate::searchengine::SearchProvider;
use crate::tab_manager::SleepMode;
use crate::ui::Message;

pub struct SettingsPageWidgets {
    pub container: gtk::ScrolledWindow,
    pub lang_label: gtk::Label,
    pub ru_btn: gtk::Button,
    pub en_btn: gtk::Button,
    pub theme_label: gtk::Label,
    pub light_btn: gtk::Button,
    pub dark_btn: gtk::Button,
    pub accent_label: gtk::Label,
    pub color_btn: gtk::ColorDialogButton,
    pub compact_label: gtk::Label,
    pub compact_switch: gtk::Switch,
    pub transparency_switch: gtk::Switch,
    pub tab_mode_label: gtk::Label,
    pub mode_manual_btn: gtk::Button,
    pub mode_balanced_btn: gtk::Button,
    pub mode_aggressive_btn: gtk::Button,
    pub search_label: gtk::Label,
    pub ddg_btn: gtk::Button,
    pub google_btn: gtk::Button,
    pub yandex_btn: gtk::Button,
    pub status_label: gtk::Label,
    pub status_switch: gtk::Switch,
}

impl SettingsPageWidgets {
    pub fn new(config: &Config, sender: &ComponentSender<Probe>) -> Self {
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content_box.set_margin_all(24);
        content_box.set_halign(gtk::Align::Center);
        content_box.set_size_request(640, -1);

        // 1. Секция Внешний вид
        let appearance_card = gtk::Box::new(gtk::Orientation::Vertical, 12);
        appearance_card.add_css_class("internal-card");

        let app_title = gtk::Label::new(Some(config.language.module_appearance_name()));
        app_title.set_halign(gtk::Align::Start);
        app_title.add_css_class("title-2");
        appearance_card.append(&app_title);

        // Язык
        let lang_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let lang_label = gtk::Label::new(Some(config.language.language_section()));
        lang_label.set_hexpand(true);
        lang_label.set_halign(gtk::Align::Start);

        let lang_btns = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let ru_btn = gtk::Button::with_label("Русский");
        ru_btn.add_css_class("settings-btn");
        let en_btn = gtk::Button::with_label("English");
        en_btn.add_css_class("settings-btn");

        let s_ru = sender.input_sender().clone();
        ru_btn.connect_clicked(move |_| s_ru.emit(Message::ChangeLanguage(Language::Ru)));
        let s_en = sender.input_sender().clone();
        en_btn.connect_clicked(move |_| s_en.emit(Message::ChangeLanguage(Language::En)));

        lang_btns.append(&ru_btn);
        lang_btns.append(&en_btn);
        lang_row.append(&lang_label);
        lang_row.append(&lang_btns);
        appearance_card.append(&lang_row);

        // Тема
        let theme_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let theme_label = gtk::Label::new(Some(config.language.theme_section()));
        theme_label.set_hexpand(true);
        theme_label.set_halign(gtk::Align::Start);

        let theme_btns = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let light_btn = gtk::Button::with_label(config.language.theme_light());
        light_btn.add_css_class("settings-btn");
        let dark_btn = gtk::Button::with_label(config.language.theme_dark());
        dark_btn.add_css_class("settings-btn");

        let s_light = sender.input_sender().clone();
        light_btn.connect_clicked(move |_| s_light.emit(Message::ChangeTheme(ThemeMode::Light)));
        let s_dark = sender.input_sender().clone();
        dark_btn.connect_clicked(move |_| s_dark.emit(Message::ChangeTheme(ThemeMode::Dark)));

        theme_btns.append(&light_btn);
        theme_btns.append(&dark_btn);
        theme_row.append(&theme_label);
        theme_row.append(&theme_btns);
        appearance_card.append(&theme_row);

        // Акцентный цвет
        let accent_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let accent_label = gtk::Label::new(Some(config.language.accent_section()));
        accent_label.set_hexpand(true);
        accent_label.set_halign(gtk::Align::Start);

        let colors_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let color_dialog = gtk::ColorDialog::new();
        let color_btn = gtk::ColorDialogButton::new(Some(color_dialog));
        color_btn.set_size_request(28, 28);
        if let Ok(rgba) = gdk::RGBA::parse(&config.accent_color) {
            color_btn.set_rgba(&rgba);
        }
        let s_custom = sender.input_sender().clone();
        color_btn.connect_rgba_notify(move |btn| {
            let rgba = btn.rgba();
            let hex = format!(
                "#{:02x}{:02x}{:02x}",
                (rgba.red() * 255.0).round() as u8,
                (rgba.green() * 255.0).round() as u8,
                (rgba.blue() * 255.0).round() as u8
            );
            s_custom.emit(Message::ChangeAccentColor(hex));
        });

        colors_box.append(&color_btn);
        accent_row.append(&accent_label);
        accent_row.append(&colors_box);
        appearance_card.append(&accent_row);

        // Компактный режим
        let compact_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let compact_label = gtk::Label::new(Some(config.language.compact_mode_label()));
        compact_label.set_hexpand(true);
        compact_label.set_halign(gtk::Align::Start);

        let compact_switch = gtk::Switch::new();
        compact_switch.set_active(config.compact_mode);
        compact_switch.set_valign(gtk::Align::Center);
        let s_comp = sender.input_sender().clone();
        compact_switch.connect_active_notify(move |sw| {
            s_comp.emit(Message::ToggleCompactMode(sw.is_active()));
        });

        compact_row.append(&compact_label);
        compact_row.append(&compact_switch);
        appearance_card.append(&compact_row);

        // Включение прозрачности
        let trans_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let trans_label = gtk::Label::new(Some(config.language.transparency_label()));
        trans_label.set_hexpand(true);
        trans_label.set_halign(gtk::Align::Start);

        let transparency_switch = gtk::Switch::new();
        transparency_switch.set_active(config.transparency_enabled);
        transparency_switch.set_valign(gtk::Align::Center);
        let s_tr = sender.input_sender().clone();
        transparency_switch.connect_active_notify(move |sw| {
            s_tr.emit(Message::ToggleTransparency(sw.is_active()));
        });

        trans_row.append(&trans_label);
        trans_row.append(&transparency_switch);
        appearance_card.append(&trans_row);

        content_box.append(&appearance_card);

        // 2. Секция Вкладки и память (tab_manager)
        let tab_card = gtk::Box::new(gtk::Orientation::Vertical, 12);
        tab_card.add_css_class("internal-card");

        let tab_title = gtk::Label::new(Some(config.language.module_tab_manager_name()));
        tab_title.set_halign(gtk::Align::Start);
        tab_title.add_css_class("title-2");
        tab_card.append(&tab_title);

        let tab_mode_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let tab_mode_label = gtk::Label::new(Some(config.language.tab_sleep_mode_label()));
        tab_mode_label.set_hexpand(true);
        tab_mode_label.set_halign(gtk::Align::Start);

        let mode_btns = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let mode_manual_btn = gtk::Button::with_label("Manual");
        mode_manual_btn.add_css_class("settings-btn");
        let mode_balanced_btn = gtk::Button::with_label("Balanced");
        mode_balanced_btn.add_css_class("settings-btn");
        let mode_aggressive_btn = gtk::Button::with_label("Aggressive");
        mode_aggressive_btn.add_css_class("settings-btn");

        let s_man = sender.input_sender().clone();
        mode_manual_btn
            .connect_clicked(move |_| s_man.emit(Message::ChangeSleepMode(SleepMode::Manual)));
        let s_bal = sender.input_sender().clone();
        mode_balanced_btn
            .connect_clicked(move |_| s_bal.emit(Message::ChangeSleepMode(SleepMode::Balanced)));
        let s_agg = sender.input_sender().clone();
        mode_aggressive_btn
            .connect_clicked(move |_| s_agg.emit(Message::ChangeSleepMode(SleepMode::Aggressive)));

        mode_btns.append(&mode_manual_btn);
        mode_btns.append(&mode_balanced_btn);
        mode_btns.append(&mode_aggressive_btn);
        tab_mode_row.append(&tab_mode_label);
        tab_mode_row.append(&mode_btns);
        tab_card.append(&tab_mode_row);

        content_box.append(&tab_card);

        // 3. Секция Поиск (searchengine)
        let search_card = gtk::Box::new(gtk::Orientation::Vertical, 12);
        search_card.add_css_class("internal-card");

        let search_title = gtk::Label::new(Some(config.language.module_searchengine_name()));
        search_title.set_halign(gtk::Align::Start);
        search_title.add_css_class("title-2");
        search_card.append(&search_title);

        let search_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let search_label = gtk::Label::new(Some(config.language.search_provider_label()));
        search_label.set_hexpand(true);
        search_label.set_halign(gtk::Align::Start);

        let search_btns = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let ddg_btn = gtk::Button::with_label("DuckDuckGo");
        ddg_btn.add_css_class("settings-btn");
        let google_btn = gtk::Button::with_label("Google");
        google_btn.add_css_class("settings-btn");
        let yandex_btn = gtk::Button::with_label("Яндекс");
        yandex_btn.add_css_class("settings-btn");

        let s_ddg = sender.input_sender().clone();
        ddg_btn.connect_clicked(move |_| {
            s_ddg.emit(Message::ChangeSearchProvider(SearchProvider::DuckDuckGo))
        });
        let s_g = sender.input_sender().clone();
        google_btn.connect_clicked(move |_| {
            s_g.emit(Message::ChangeSearchProvider(SearchProvider::Google))
        });
        let s_ya = sender.input_sender().clone();
        yandex_btn.connect_clicked(move |_| {
            s_ya.emit(Message::ChangeSearchProvider(SearchProvider::Yandex))
        });

        search_btns.append(&ddg_btn);
        search_btns.append(&google_btn);
        search_btns.append(&yandex_btn);
        search_row.append(&search_label);
        search_row.append(&search_btns);
        search_card.append(&search_row);

        content_box.append(&search_card);

        // 4. Секция Дополнительно
        let extra_card = gtk::Box::new(gtk::Orientation::Vertical, 12);
        extra_card.add_css_class("internal-card");

        let status_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let status_label = gtk::Label::new(Some(config.language.show_page_status_label()));
        status_label.set_hexpand(true);
        status_label.set_halign(gtk::Align::Start);

        let status_switch = gtk::Switch::new();
        status_switch.set_active(config.show_page_status);
        status_switch.set_valign(gtk::Align::Center);
        let s_status = sender.input_sender().clone();
        status_switch.connect_active_notify(move |sw| {
            s_status.emit(Message::TogglePageStatus(sw.is_active()));
        });

        status_row.append(&status_label);
        status_row.append(&status_switch);
        extra_card.append(&status_row);

        content_box.append(&extra_card);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&content_box));

        let mut res = Self {
            container: scrolled,
            lang_label,
            ru_btn,
            en_btn,
            theme_label,
            light_btn,
            dark_btn,
            accent_label,
            color_btn,
            compact_label,
            compact_switch,
            transparency_switch,
            tab_mode_label,
            mode_manual_btn,
            mode_balanced_btn,
            mode_aggressive_btn,
            search_label,
            ddg_btn,
            google_btn,
            yandex_btn,
            status_label,
            status_switch,
        };
        res.update(config);
        res
    }

    pub fn update(&mut self, config: &Config) {
        self.lang_label.set_text(config.language.language_section());
        self.theme_label.set_text(config.language.theme_section());
        self.light_btn.set_label(config.language.theme_light());
        self.dark_btn.set_label(config.language.theme_dark());
        self.accent_label.set_text(config.language.accent_section());
        self.compact_label
            .set_text(config.language.compact_mode_label());
        self.tab_mode_label
            .set_text(config.language.tab_sleep_mode_label());
        self.search_label
            .set_text(config.language.search_provider_label());
        self.status_label
            .set_text(config.language.show_page_status_label());

        // Активность языка
        self.ru_btn.set_sensitive(config.language != Language::Ru);
        self.en_btn.set_sensitive(config.language != Language::En);

        // Активность темы
        self.light_btn
            .set_sensitive(config.theme != ThemeMode::Light);
        self.dark_btn.set_sensitive(config.theme != ThemeMode::Dark);

        // Активность режима сна
        self.mode_manual_btn
            .set_sensitive(config.tab_sleep_mode != SleepMode::Manual);
        self.mode_balanced_btn
            .set_sensitive(config.tab_sleep_mode != SleepMode::Balanced);
        self.mode_aggressive_btn
            .set_sensitive(config.tab_sleep_mode != SleepMode::Aggressive);

        // Активность поиска
        self.ddg_btn
            .set_sensitive(config.search_provider != SearchProvider::DuckDuckGo);
        self.google_btn
            .set_sensitive(config.search_provider != SearchProvider::Google);
        self.yandex_btn
            .set_sensitive(config.search_provider != SearchProvider::Yandex);

        if self.compact_switch.is_active() != config.compact_mode {
            self.compact_switch.set_active(config.compact_mode);
        }

        if self.transparency_switch.is_active() != config.transparency_enabled {
            self.transparency_switch
                .set_active(config.transparency_enabled);
        }

        if self.status_switch.is_active() != config.show_page_status {
            self.status_switch.set_active(config.show_page_status);
        }
    }
}
