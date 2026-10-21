use relm4::gtk::{self, prelude::*};
use webkit6::prelude::*;

const PROBE_HTML: &str = r#"<!doctype html>
<html lang="ru">
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>Light — проверка WebKitGTK</title>
<style>body { font: 20px sans-serif; margin: 3em; }</style>
<h1>Light Project</h1>
<p>Эта страница встроена в приложение и не требует сети.</p>
<p>Выделите текст и измените размер окна, чтобы проверить отображение.</p>
</html>"#;

fn normalize_address(input: &str) -> Result<String, &'static str> {
    let input = input.trim();
    if input.is_empty() || input.chars().any(char::is_whitespace) {
        return Err("Введите корректный адрес без пробелов.");
    }

    if input.starts_with("http://") || input.starts_with("https://") {
        Ok(input.to_owned())
    } else if input.contains("://") {
        Err("Поддерживаются только адреса HTTP и HTTPS.")
    } else {
        Ok(format!("https://{input}"))
    }
}

#[derive(Debug)]
pub enum EngineEvent {
    Started,
    Finished,
    Failed,
    ProcessTerminated,
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
        view.connect_web_process_terminated(move |_, _| {
            on_event(EngineEvent::ProcessTerminated);
        });
        Self { view }
    }

    pub fn widget(&self) -> gtk::Widget {
        self.view.clone().upcast()
    }

    pub fn load_probe(&self) {
        self.view.load_html(PROBE_HTML, Some("about:blank"));
    }

    pub fn navigate(&self, input: &str) -> Result<(), &'static str> {
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
    use super::normalize_address;

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
        assert!(normalize_address("file:///etc/passwd").is_err());
        assert!(normalize_address("custom://example.com").is_err());
        assert!(normalize_address("https://example.com bad").is_err());
    }
}
