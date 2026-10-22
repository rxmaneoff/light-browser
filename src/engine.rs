use relm4::gtk::{self, prelude::*};
use webkit6::prelude::*;

use crate::i18n::{AddressError, Language};

pub fn normalize_address(input: &str) -> Result<String, AddressError> {
    let input = input.trim();
    if input.is_empty() || input.chars().any(char::is_whitespace) {
        return Err(AddressError::EmptyOrWhitespace);
    }

    if input.starts_with("http://") || input.starts_with("https://") {
        Ok(input.to_owned())
    } else if input.contains("://") {
        Err(AddressError::UnsupportedScheme)
    } else {
        Ok(format!("https://{input}"))
    }
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Started,
    Finished,
    Failed,
    ProcessTerminated,
    TitleChanged(String),
    UriChanged(String),
}

pub struct Engine {
    view: webkit6::WebView,
}

impl Engine {
    // WebView создаётся и используется только в главном потоке GTK.
    pub fn new(on_event: impl Fn(EngineEvent) + Clone + 'static) -> Self {
        let view = webkit6::WebView::new();
        let on_load = on_event.clone();
        view.connect_load_changed(move |_, event| match event {
            webkit6::LoadEvent::Started => on_load(EngineEvent::Started),
            webkit6::LoadEvent::Finished => on_load(EngineEvent::Finished),
            _ => {}
        });
        let on_failure = on_event.clone();
        view.connect_load_failed(move |_, _, _, _| {
            // Сообщение UI не содержит URL или текста ошибки с данными страницы.
            on_failure(EngineEvent::Failed);
            true
        });
        let on_proc = on_event.clone();
        view.connect_web_process_terminated(move |_, _| {
            on_proc(EngineEvent::ProcessTerminated);
        });

        let on_title = on_event.clone();
        view.connect_title_notify(move |v| {
            if let Some(title) = v.title() {
                on_title(EngineEvent::TitleChanged(title.to_string()));
            }
        });

        let on_uri = on_event;
        view.connect_uri_notify(move |v| {
            if let Some(uri) = v.uri() {
                on_uri(EngineEvent::UriChanged(uri.to_string()));
            }
        });

        Self { view }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.view.clone().upcast()
    }

    #[allow(dead_code)]
    pub fn title(&self) -> Option<String> {
        self.view.title().map(|s| s.to_string())
    }

    #[allow(dead_code)]
    pub fn uri(&self) -> Option<String> {
        self.view.uri().map(|s| s.to_string())
    }

    pub fn load_probe(&self, lang: Language) {
        self.view.load_html(lang.probe_html(), Some("about:blank"));
    }

    pub fn navigate(&self, input: &str) -> Result<(), AddressError> {
        let address = normalize_address(input)?;
        self.view.load_uri(&address);
        Ok(())
    }

    pub fn can_go_back(&self) -> bool {
        self.view.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.can_go_forward()
    }

    pub fn go_back(&self) {
        if self.can_go_back() {
            self.view.go_back();
        }
    }

    pub fn go_forward(&self) {
        if self.can_go_forward() {
            self.view.go_forward();
        }
    }

    pub fn reload(&self) {
        self.view.reload();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::AddressError;

    #[test]
    fn keeps_http_and_https_addresses() {
        assert_eq!(
            normalize_address("http://example.com"),
            Ok("http://example.com".into())
        );
        assert_eq!(
            normalize_address("https://example.com"),
            Ok("https://example.com".into())
        );
    }

    #[test]
    fn defaults_bare_host_to_https() {
        assert_eq!(
            normalize_address("example.com/path"),
            Ok("https://example.com/path".into())
        );
    }

    #[test]
    fn rejects_non_web_schemes_and_whitespace() {
        assert_eq!(
            normalize_address("file:///etc/passwd"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("custom://example.com"),
            Err(AddressError::UnsupportedScheme)
        );
        assert_eq!(
            normalize_address("https://example.com bad"),
            Err(AddressError::EmptyOrWhitespace)
        );
    }
}
