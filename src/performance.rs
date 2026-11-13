//! Модуль мониторинга производительности и внутренней диагностики (`performance.rs`).
//!
//! Отвечает за:
//! - Сбор метрик работы браузера (память RSS, время загрузки, счетчики ошибок);
//! - Различение статусов «нет данных», «не поддерживается» и реальных измерений;
//! - Генерацию содержимого внутренней страницы `light://performance`;
//! - Отображение понятного локализованного состояния при выключенном модуле.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use crate::config::ThemeMode;
use crate::i18n::Language;

/// Счетчики системных событий и ошибок движка.
#[derive(Debug, Default)]
pub struct PerformanceCounters {
    pub load_errors: AtomicUsize,
    pub tls_errors: AtomicUsize,
    pub process_crashes: AtomicUsize,
}

impl PerformanceCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_load_error(&self) {
        self.load_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_tls_error(&self) {
        self.tls_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_process_crash(&self) {
        self.process_crashes.fetch_add(1, Ordering::Relaxed);
    }
}

/// Состояние метрик для отображения на диагностической странице.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticsSnapshot {
    pub total_tabs: usize,
    pub active_tabs: usize,
    pub sleeping_tabs: usize,
    pub memory_rss_mb: Option<f64>,
    pub last_load_duration_ms: Option<u64>,
    pub load_errors_count: usize,
    pub tls_errors_count: usize,
    pub process_crashes_count: usize,
}

impl DiagnosticsSnapshot {
    /// Считывает текущее потребление памяти Resident Set Size (RSS) процесса браузера из `/proc/self/statm`.
    pub fn read_process_rss_mb() -> Option<f64> {
        let content = fs::read_to_string("/proc/self/statm").ok()?;
        let mut parts = content.split_whitespace();
        // Второе число в statm — резидентные страницы (resident pages)
        let _size = parts.next()?;
        let resident_pages: u64 = parts.next()?.parse().ok()?;

        let page_size_kb = 4; // Стандартный размер страницы в Linux x86_64 — 4 КБ
        let rss_kb = resident_pages * page_size_kb;
        Some(rss_kb as f64 / 1024.0)
    }
}

/// Измеритель времени загрузки отдельной страницы.
#[derive(Debug)]
pub struct PageLoadTimer {
    start: Option<Instant>,
    last_duration_ms: Option<u64>,
}

impl Default for PageLoadTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl PageLoadTimer {
    pub fn new() -> Self {
        Self {
            start: None,
            last_duration_ms: None,
        }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn finish(&mut self) -> Option<u64> {
        if let Some(start) = self.start.take() {
            let duration = start.elapsed().as_millis() as u64;
            self.last_duration_ms = Some(duration);
            Some(duration)
        } else {
            None
        }
    }

    pub fn last_duration(&self) -> Option<u64> {
        self.last_duration_ms
    }
}

/// Генерирует HTML для страницы `light://performance`.
pub fn generate_performance_html(
    is_module_enabled: bool,
    snapshot: Option<&DiagnosticsSnapshot>,
    lang: Language,
    theme: ThemeMode,
    accent_color: &str,
) -> String {
    let (bg_color, text_color, card_bg, border_color, secondary_text) = match theme {
        ThemeMode::Light => ("#f6f8fa", "#1f2328", "#ffffff", "#d0d7de", "#656d76"),
        ThemeMode::Dark => ("#1c1c1f", "#f0f2f5", "#28282d", "#3c3c43", "#98989f"),
    };

    let title = match lang {
        Language::Ru => "Диагностика производительности — Light",
        Language::En => "Performance Diagnostics — Light",
    };

    let content_html = if !is_module_enabled {
        match lang {
            Language::Ru => r#"
                <div class="card warning-card">
                    <h2>Модуль мониторинга выключен</h2>
                    <p>Сбор метрик и анализ ресурсов отключены в настройках браузера. Включите модуль «Производительность» в разделе «Модули», чтобы увидеть диагностические данные.</p>
                </div>
            "#,
            Language::En => r#"
                <div class="card warning-card">
                    <h2>Performance monitoring module is disabled</h2>
                    <p>Metrics collection and resource analysis are disabled in browser settings. Enable the "Performance" module in "Modules" tab to view diagnostic data.</p>
                </div>
            "#,
        }.to_string()
    } else if let Some(snap) = snapshot {
        let memory_str = snap
            .memory_rss_mb
            .map(|m| format!("{m:.1} МБ / MB"))
            .unwrap_or_else(|| "Нет данных / No data".to_string());

        let load_time_str = snap
            .last_load_duration_ms
            .map(|t| format!("{t} мс / ms"))
            .unwrap_or_else(|| "Нет измерений / No measurement".to_string());

        match lang {
            Language::Ru => format!(
                r#"
                <div class="grid">
                    <div class="card">
                        <h3>Вкладки</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Всего открыто: {} | Активных: {} | В режиме сна: {}</p>
                    </div>
                    <div class="card">
                        <h3>Память процесса браузера</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Resident Set Size (RSS) основного процесса оболочки</p>
                    </div>
                    <div class="card">
                        <h3>Последняя загрузка страницы</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Время от старта до события завершения WebKitGTK</p>
                    </div>
                    <div class="card">
                        <h3>События надёжности</h3>
                        <p class="stat-desc">Ошибок загрузки: <b>{}</b></p>
                        <p class="stat-desc">Ошибок TLS: <b>{}</b></p>
                        <p class="stat-desc">Сбоев веб-процессов: <b>{}</b></p>
                    </div>
                </div>
            "#,
                snap.total_tabs,
                snap.total_tabs,
                snap.active_tabs,
                snap.sleeping_tabs,
                memory_str,
                load_time_str,
                snap.load_errors_count,
                snap.tls_errors_count,
                snap.process_crashes_count
            ),
            Language::En => format!(
                r#"
                <div class="grid">
                    <div class="card">
                        <h3>Tabs</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Total open: {} | Active: {} | Sleeping: {}</p>
                    </div>
                    <div class="card">
                        <h3>Browser Process Memory</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Resident Set Size (RSS) of main UI process</p>
                    </div>
                    <div class="card">
                        <h3>Last Page Load Time</h3>
                        <p class="stat-value">{}</p>
                        <p class="stat-desc">Duration between start and WebKitGTK finish event</p>
                    </div>
                    <div class="card">
                        <h3>Reliability Events</h3>
                        <p class="stat-desc">Load errors: <b>{}</b></p>
                        <p class="stat-desc">TLS errors: <b>{}</b></p>
                        <p class="stat-desc">Web process crashes: <b>{}</b></p>
                    </div>
                </div>
            "#,
                snap.total_tabs,
                snap.total_tabs,
                snap.active_tabs,
                snap.sleeping_tabs,
                memory_str,
                load_time_str,
                snap.load_errors_count,
                snap.tls_errors_count,
                snap.process_crashes_count
            ),
        }
    } else {
        "<p>Нет данных / No data</p>".to_string()
    };

    format!(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>{title}</title>
<style>
  :root {{
    --bg-color: {bg_color};
    --text-color: {text_color};
    --card-bg: {card_bg};
    --border-color: {border_color};
    --secondary-text: {secondary_text};
    --accent-color: {accent_color};
  }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background-color: var(--bg-color);
    color: var(--text-color);
    margin: 0;
    padding: 2.5rem;
    display: flex;
    justify-content: center;
  }}
  .container {{
    max-width: 800px;
    width: 100%;
  }}
  h1 {{
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
  }}
  .grid {{
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 1rem;
  }}
  .card {{
    background: var(--card-bg);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 1.25rem;
  }}
  .warning-card {{
    border-left: 4px solid #e6a23c;
  }}
  .stat-value {{
    font-size: 1.75rem;
    font-weight: bold;
    color: var(--accent-color);
    margin: 0.5rem 0;
  }}
  .stat-desc {{
    font-size: 0.875rem;
    color: var(--secondary-text);
    margin: 0.25rem 0;
  }}
</style>
</head>
<body>
  <div class="container">
    <h1>{title}</h1>
    {content_html}
  </div>
</body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_load_timer() {
        let mut timer = PageLoadTimer::new();
        assert!(timer.finish().is_none());

        timer.start();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let duration = timer.finish().expect("Must measure duration");
        assert!(duration >= 4);
    }

    #[test]
    fn test_performance_html_disabled_state() {
        let html = generate_performance_html(false, None, Language::Ru, ThemeMode::Dark, "#3584e4");
        assert!(html.contains("Модуль мониторинга выключен"));
    }

    #[test]
    fn test_performance_html_enabled_state() {
        let snap = DiagnosticsSnapshot {
            total_tabs: 3,
            active_tabs: 2,
            sleeping_tabs: 1,
            memory_rss_mb: Some(42.5),
            last_load_duration_ms: Some(150),
            load_errors_count: 0,
            tls_errors_count: 0,
            process_crashes_count: 0,
        };

        let html =
            generate_performance_html(true, Some(&snap), Language::En, ThemeMode::Dark, "#3584e4");
        assert!(html.contains("Tabs"));
        assert!(html.contains("42.5 МБ / MB"));
        assert!(html.contains("150 мс / ms"));
    }
}
