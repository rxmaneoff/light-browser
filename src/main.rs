mod config;
mod engine;
mod i18n;
mod tabs;
mod ui;

fn main() -> std::process::ExitCode {
    if relm4::gtk::init().is_err() {
        eprintln!("Не удалось подключиться к графической сессии GTK.");
        return std::process::ExitCode::FAILURE;
    }

    relm4::RelmApp::new("org.lightproject.IntegrationProbe").run::<ui::Probe>(());
    std::process::ExitCode::SUCCESS
}
