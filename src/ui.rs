use relm4::{gtk, gtk::prelude::*, prelude::*};

use crate::engine::{Engine, EngineEvent};

pub struct Probe {
    engine: Engine,
    status: String,
    failed: bool,
}

#[derive(Debug)]
pub enum Message {
    Navigate(String),
    Back,
    Forward,
    Reload,
    Engine(EngineEvent),
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
                set_spacing: 8,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 6,
                    set_margin_all: 8,

                    gtk::Button {
                        set_label: "Назад",
                        #[watch]
                        set_sensitive: model.engine.can_go_back(),
                        connect_clicked => Message::Back,
                    },

                    gtk::Button {
                        set_label: "Вперёд",
                        #[watch]
                        set_sensitive: model.engine.can_go_forward(),
                        connect_clicked => Message::Forward,
                    },

                    gtk::Button {
                        set_label: "Обновить",
                        connect_clicked => Message::Reload,
                    },

                    #[name(address_entry)]
                    gtk::Entry {
                        set_hexpand: true,
                        set_placeholder_text: Some("Введите адрес сайта"),
                        connect_activate[sender] => move |entry| {
                            sender.input(Message::Navigate(entry.text().to_string()));
                        },
                    },
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                    set_wrap: true,
                },

                #[local_ref]
                engine_widget -> gtk::Widget {
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
        let input = sender.input_sender().clone();
        let engine = Engine::new(move |event| input.emit(Message::Engine(event)));
        let model = Self {
            engine,
            status: "Ожидание загрузки WebKitGTK…".to_owned(),
            failed: false,
        };
        let engine_widget = &model.engine.widget();
        let widgets = view_output!();
        model.engine.load_probe();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Message, _sender: ComponentSender<Self>) {
        match message {
            Message::Navigate(address) => match self.engine.navigate(&address) {
                Ok(()) => self.status = "Загрузка адреса…".to_owned(),
                Err(message) => self.status = message.to_owned(),
            },
            Message::Back => self.engine.go_back(),
            Message::Forward => self.engine.go_forward(),
            Message::Reload => self.engine.reload(),
            Message::Engine(EngineEvent::Started) => {
                self.failed = false;
                self.status = "Загрузка страницы…".to_owned();
            }
            Message::Engine(EngineEvent::Finished) => {
                // Finished приходит и после load-failed; ошибка должна остаться видимой.
                if !self.failed {
                    self.status = "Загрузка завершена.".to_owned();
                }
            }
            Message::Engine(EngineEvent::Failed) => {
                self.failed = true;
                self.status =
                    "Не удалось загрузить страницу. Проверьте адрес или повторите загрузку."
                        .to_owned();
            }
            Message::Engine(EngineEvent::ProcessTerminated) => {
                self.failed = true;
                self.status = "Процесс WebKitGTK завершился. Повторите загрузку.".to_owned();
            }
        }
    }
}
