use relm4::gtk::{self, prelude::*};
use relm4::{ComponentSender, RelmWidgetExt};

use super::Probe;
use crate::config::Config;
use crate::modules::{ModuleRegistry, PlatformCapabilities};
use crate::ui::Message;

pub struct ModuleRowWidgets {
    pub id: &'static str,
    pub title_label: gtk::Label,
    pub desc_label: gtk::Label,
    pub switch: gtk::Switch,
    pub badge: Option<gtk::Label>,
}

pub struct ModulesPageWidgets {
    pub container: gtk::ScrolledWindow,
    pub title_label: gtk::Label,
    pub beta_badge: gtk::Label,
    pub rows: Vec<ModuleRowWidgets>,
    pub transparency_label: gtk::Label,
    pub transparency_scale: gtk::Scale,
    pub transparency_val_label: gtk::Label,
    pub transparency_status_label: gtk::Label,
}

impl ModulesPageWidgets {
    pub fn new(config: &Config, sender: &ComponentSender<Probe>) -> Self {
        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content_box.set_margin_all(24);
        content_box.set_halign(gtk::Align::Center);
        content_box.set_size_request(640, -1);

        // Заголовок экрана с бейджем Beta
        let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        header_box.set_halign(gtk::Align::Start);

        let title_label = gtk::Label::new(Some(config.language.modules_tab_title()));
        title_label.add_css_class("title-1");

        let beta_badge = gtk::Label::new(Some(config.language.modules_beta_label()));
        beta_badge.add_css_class("beta-badge");
        beta_badge.set_valign(gtk::Align::Center);

        header_box.append(&title_label);
        header_box.append(&beta_badge);
        content_box.append(&header_box);

        let mut row_widgets = Vec::new();
        let modules_list = ModuleRegistry::list_modules(config, config.language);

        for module in modules_list {
            let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
            card.add_css_class("internal-card");

            let top_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            top_row.set_valign(gtk::Align::Center);

            let text_col = gtk::Box::new(gtk::Orientation::Vertical, 4);
            text_col.set_hexpand(true);

            let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let name_label = gtk::Label::new(Some(module.title));
            name_label.set_halign(gtk::Align::Start);
            name_label.add_css_class("heading");
            title_box.append(&name_label);

            let badge = if module.is_required {
                let req_badge = gtk::Label::new(Some(match config.language {
                    crate::i18n::Language::Ru => "Обязательный",
                    crate::i18n::Language::En => "Required",
                }));
                req_badge.add_css_class("status-badge-supported");
                req_badge.set_valign(gtk::Align::Center);
                title_box.append(&req_badge);
                Some(req_badge)
            } else {
                None
            };

            text_col.append(&title_box);

            let desc_label = gtk::Label::new(Some(module.description));
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_wrap(true);
            desc_label.add_css_class("dim-label");
            text_col.append(&desc_label);

            if let Some(reason) = module.required_reason {
                let reason_label = gtk::Label::new(Some(reason));
                reason_label.set_halign(gtk::Align::Start);
                reason_label.set_wrap(true);
                reason_label.add_css_class("dim-label");
                text_col.append(&reason_label);
            }

            let switch = gtk::Switch::new();
            switch.set_active(module.is_enabled);
            switch.set_valign(gtk::Align::Center);

            if module.is_required {
                // Обязательный модуль нельзя отключить в UI
                switch.set_sensitive(false);
            } else {
                let s_toggle = sender.input_sender().clone();
                let mod_id = module.id.to_string();
                switch.connect_active_notify(move |sw| {
                    s_toggle.emit(Message::ToggleModule(mod_id.clone(), sw.is_active()));
                });
            }

            top_row.append(&text_col);
            top_row.append(&switch);
            card.append(&top_row);

            content_box.append(&card);

            row_widgets.push(ModuleRowWidgets {
                id: module.id,
                title_label: name_label,
                desc_label,
                switch,
                badge,
            });
        }

        // Дополнительная карточка: тонкая настройка прозрачности (модуль appearance)
        let trans_card = gtk::Box::new(gtk::Orientation::Vertical, 10);
        trans_card.add_css_class("internal-card");

        let trans_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let transparency_label = gtk::Label::new(Some(config.language.transparency_label()));
        transparency_label.set_hexpand(true);
        transparency_label.set_halign(gtk::Align::Start);
        transparency_label.add_css_class("heading");

        let transparency_val_label = gtk::Label::new(Some(&format!("{:.2}", config.transparency)));

        let transparency_status_label = gtk::Label::new(Some(config.language.feature_supported()));
        transparency_status_label.add_css_class("status-badge-supported");

        trans_header.append(&transparency_label);
        trans_header.append(&transparency_val_label);
        trans_header.append(&transparency_status_label);
        trans_card.append(&trans_header);

        let transparency_scale =
            gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        transparency_scale.set_value(config.transparency);
        transparency_scale.set_hexpand(true);

        let s_trans = sender.input_sender().clone();
        let val_label_clone = transparency_val_label.clone();
        transparency_scale.connect_value_changed(move |scale| {
            let val = scale.value();
            val_label_clone.set_text(&format!("{:.2}", val));
            s_trans.emit(Message::ChangeTransparency(val));
        });

        trans_card.append(&transparency_scale);
        content_box.append(&trans_card);

        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        scrolled.set_child(Some(&content_box));

        let mut res = Self {
            container: scrolled,
            title_label,
            beta_badge,
            rows: row_widgets,
            transparency_label,
            transparency_scale,
            transparency_val_label,
            transparency_status_label,
        };
        res.update(config);
        res
    }

    pub fn update(&mut self, config: &Config) {
        let caps = PlatformCapabilities::detect(config.transparency_enabled, false);

        self.title_label
            .set_text(config.language.modules_tab_title());
        self.beta_badge
            .set_text(config.language.modules_beta_label());

        let modules_list = ModuleRegistry::list_modules(config, config.language);

        for row in &self.rows {
            if let Some(m) = modules_list.iter().find(|item| item.id == row.id) {
                row.title_label.set_text(m.title);
                row.desc_label.set_text(m.description);

                if row.switch.is_active() != m.is_enabled {
                    row.switch.set_active(m.is_enabled);
                }

                if m.is_required {
                    row.switch.set_sensitive(false);
                    if let Some(b) = &row.badge {
                        b.set_text(match config.language {
                            crate::i18n::Language::Ru => "Обязательный",
                            crate::i18n::Language::En => "Required",
                        });
                    }
                }
            }
        }

        self.transparency_label
            .set_text(config.language.transparency_label());
        self.transparency_val_label
            .set_text(&format!("{:.2}", config.transparency));

        if (self.transparency_scale.value() - config.transparency).abs() > 0.001 {
            self.transparency_scale.set_value(config.transparency);
        }

        self.transparency_scale
            .set_sensitive(config.transparency_enabled);

        let trans_status_str = if caps.transparency.is_available() {
            config.language.feature_supported()
        } else {
            config.language.feature_not_available()
        };
        self.transparency_status_label.set_text(trans_status_str);
    }
}
