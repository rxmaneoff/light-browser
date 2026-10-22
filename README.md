# Light

**An experimental, lightweight and customizable web browser for Linux.**

Light is an open-source browser project built with Rust, GTK4, Relm4 and WebKitGTK. The goal is to create a lightweight, modular browsing experience with an emphasis on performance, customization and user control.

The project is currently in the **0.2-alpha** development stage.

## Goals

- **Rust-powered** — safe, fast, and modern systems language.
- **Linux-first** — designed natively for Wayland and X11 desktops.
- **Modular architecture** — clean separation between UI, Core, and WebKit adapter.
- **Performance-focused** — manual tab lifecycle by default without unexpected background interruptions.
- **Customizable interface** — language switching, planned themes, transparency and optional visual effects.
- **Privacy-conscious** — strict zero telemetry policy; session cookies are kept in memory and never written unencrypted to disk.
- **WebKitGTK 6.0** — modern and secure web content rendering.

## Tech Stack

- Rust (Edition 2024)
- GTK4
- Relm4
- WebKitGTK 6.0 (`webkit6` crate)

## Development Status (0.2-alpha)

Light 0.2-alpha provides a functional browser shell with:
- **Full tab management**: create new tabs (`+`), switch between tabs, and close tabs (`✕`) with automatic fallback to neighbor tabs.
- **Smooth multi-tab stack**: tabs preserve their independent WebKit views and background loading without interruption.
- **Settings & Localization**: quick corner button (`⚙ RU` / `⚙ EN`) switching interface labels, dynamic status notices, and the built-in offline test page.
- **Safe address bar & navigation**: URL normalization (HTTPS by default, scheme validation), Back, Forward, and Reload controls.
- **Privacy & Security verified**: no telemetry, no unencrypted credential files, and cookies remain strictly in-memory by default.

## Roadmap

- [x] Initial browser window
- [x] Basic navigation controls (Back, Forward, Reload)
- [x] Address bar with HTTPS auto-resolution and scheme safety
- [x] Web content rendering via WebKitGTK 6.0
- [x] Tab management (create, switch, close, background loading preservation)
- [x] Interface language switcher (RU / EN) and dynamic status localization
- [ ] Download manager
- [ ] Session persistence and crash recovery
- [ ] Configurable appearance (themes, window transparency)
- [ ] Optional UI modules
- [ ] Tab suspension policies (Balanced, Aggressive)

## How to Run (0.2-alpha)

### Prerequisites

Install system dependencies:

```
base-devel rust gtk4 webkitgtk-6.0 pkgconf
```

Verify your environment:

```
pkg-config --modversion gtk4
pkg-config --modversion webkitgtk-6.0
```

### Build and Launch

```
git clone https://github.com/rxmaneoff/light-browser ~/Light
cd ~/Light
cargo run --release
```

Or for development and running tests:

```
cargo test
cargo run
```

## Contributing

Contributions, ideas, bug reports and feedback are welcome as the project evolves.
