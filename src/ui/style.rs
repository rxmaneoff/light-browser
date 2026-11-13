use crate::appearance::{AppearanceConfig, generate_gtk_css};
use crate::config::Config;

/// Генерирует динамический CSS для GTK4 с учетом темы, акцентного цвета,
/// компактного режима и параметров прозрачности в стиле Quiet Native.
pub fn generate_css(config: &Config) -> String {
    let app_config = AppearanceConfig::from_config(config);
    generate_gtk_css(&app_config)
}
