# Light

**An experimental, lightweight and customizable web browser for Linux.**

Light is an open-source browser project built with Rust, GTK4, Relm4 and WebKitGTK. The goal is to create a lightweight, modular browsing experience with an emphasis on performance, customization and user control.

The project is currently in an early development stage.

## Goals

- **Rust-powered** — a modern systems programming language.
- **Linux-first** — designed with the Linux desktop in mind.
- **Modular architecture** — optional features and customizable behavior.
- **Performance-focused** — investigating ways to reduce unnecessary resource usage.
- **Customizable interface** — planned themes, transparency and optional visual effects.
- **Privacy-conscious** — no browser-owned telemetry is intended.
- **WebKitGTK** — web content rendering through the WebKit engine.

## Tech Stack

- Rust
- GTK4
- Relm4
- WebKitGTK

## Development Status

Light is under active development. The current prototype can launch a browser window, navigate to websites and display web content.

Features such as tab management, download management, session recovery and advanced customization are planned or under development.

**Expect bugs, incomplete features and breaking changes.**

## &#x20;Roadmap

- [x] Initial browser window
- [x] Basic navigation controls
- [x] Address bar
- [x] Web content rendering
- [ ] Tab management
- [ ] Download manager
- [ ] Session persistence and recovery
- [ ] Configurable appearance
- [ ] Optional UI modules
- [ ] Tab suspension and memory management

## How to run (0.1-alpha)

Install base-devel, rust, gtk4, webkitgtk-6.0, pkgconf, then run the following commands:

```
git clone https://github.com/rxmaneoff/light-browser ~/Light
cd Light
cargo run --release
```
## &#x20;Contributing

Contributions, ideas, bug reports and feedback are welcome as the project evolves.

##

