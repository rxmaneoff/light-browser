# Light

[Русская версия](README.md)

**An experimental, lightweight, and customizable browser for Linux.**

Light is an open-source web browser built with Rust using GTK4, Relm4, and WebKitGTK. The main goal of the project is to provide a fast, modular, and convenient tool for web surfing with a focus on performance, flexible UI customization, and complete user control over their data.

The project is currently in the **0.3-alpha** development stage.

---

## Screenshots

| Dark Theme (English interface) | Light Theme (English interface) |
| :---: | :---: |
| ![Dark theme (EN)](docs/screenshots/dark_en.png) | ![Light theme (EN)](docs/screenshots/light_en.png) |

| Dark Theme (Russian interface) | Light Theme (Russian interface) |
| :---: | :---: |
| ![Dark theme (RU)](docs/screenshots/dark_ru.png) | ![Light theme (RU)](docs/screenshots/light_ru.png) |

| Web Browsing (Arch Linux) |
| :---: |
| ![Web browsing](docs/screenshots/archlinux.png) |

---

## Key Features & Goals

- **Rust Reliability** — memory safety, high performance, and a modern programming language.
- **Linux-First** — native operation on Wayland and X11.
- **Modular Architecture** — strict separation of concerns: UI (GTK4 / Relm4), tab management core, configuration, and WebKitGTK adapter (in future versions).
- **Predictable Memory Management** — `Manual` tab lifecycle mode by default: no background unloads or loss of unsaved site data.
- **Flexible Customization** — Light and Dark themes, custom accent color picker palette, pre-made color swatches, dynamic language switching on the fly, and additional settings in future versions.
- **Privacy & Security** — strict zero-telemetry policy: no hidden network requests, telemetry reporting, or trackers.
- **WebKitGTK 6.0** — modern web standards rendering engine with hardware acceleration and process isolation.

---

## Tech Stack (Brief)

- **Language**: Rust (Edition 2024)
- **Graphical Interface**: GTK4 (`gtk4` crate)
- **Reactive UI Architecture**: Relm4 (`relm4` crate)
- **Rendering Engine**: WebKitGTK 6.0 (`webkit6` crate)
- **Configuration**: TOML (`toml`, `serde`)
- **Session & Cookie Database**: SQLite via WebKitGTK

---

## Work Done in 0.3-alpha (Compared to 0.2-alpha)

Version 0.3-alpha represents a qualitative leap forward compared to 0.2-alpha: from a basic window and preliminary navigation elements to a cohesive, personalized browser with persistent state.

### 1. Capsule Tab Design
- **Capsule-shaped tabs**: each tab is designed as a rounded block with a comfortable height.
- **Accent fill**: the active tab is completely filled with the selected accent color and high-contrast white text.
- **Updated tab close button (`✕`)**:
  - On inactive tabs, the button is hidden and only appears on mouse hover.
  - Visible on the active tab immediately.
  - The background behind the close button seamlessly matches the tab surface without gaps or seams.
- **Square new tab button (`+`)**: placed in the tab strip directly behind the last open tab with square proportions and rounded corners.

### 2. Persistent SQLite Session & Cookie Storage
- Integrated WebKitGTK SQLite-based storage (`~/.local/share/light-project/cookies.sqlite`).
- Authorization sessions, website logins, and cookies are now preserved between browser restarts.

### 3. Connection Security Indicator
- Added a dynamic connection status indicator right before the address bar with descriptive tooltips:
  - **HTTPS** — secure encrypted connection.
  - **HTTP** — open unencrypted connection.
  - **TLS Errors** — warning about invalid or self-signed certificates.
  - **Internal Pages** — built-in Light pages.

### 4. Interactive Settings Menu & Personalization
- Opened via the gear button in the top-right toolbar.
- **Instant language switching (RU / EN)**: all interface labels, statuses, and settings menus update immediately.
- **Appearance themes (Light / Dark)**: menu, panel, and button backgrounds strictly adhere to the chosen theme.
- **Ready-made presets**: 5 round accent color swatches: blue, purple, emerald, amber, rose.
- **Custom color selection**: built-in dialog for choosing any custom shade.
- **Live style updates**: active tabs, settings buttons, and the built-in probe page are recolored instantly without a restart.

### 5. Configuration Persistence in TOML
- User settings are saved in `~/.config/light-project/config.toml` (language, theme, accent color).
- Provides safe default values and error handling for reading corrupt files.

### 6. GTK4 CSS Cleanup & Stabilization
- Complete compatibility with the GTK4 CSS parser: incompatible properties and directives removed, ensuring 0 style parsing errors.

---

## Development Progress

- [x] Main browser window
- [x] Basic navigation controls (Back, Forward, Reload)
- [x] Address bar with automatic HTTPS resolution and scheme safety validation
- [x] Full tab management (create, switch, safe close)
- [x] Capsule tab design with smooth close button
- [x] New tab button `+` in the tab strip
- [x] Interface language switching (RU / EN)
- [x] Connection security indicator (HTTPS / HTTP / certificate errors)
- [x] Quick settings menu (language, theme, quick color presets, and palette)
- [x] Persistent browser configuration (`config.toml`)
- [x] Persistent cookies and site logins in SQLite
- [ ] Download manager (progress tracking, destination picker, cancellation)
- [ ] Session and tab restoration after restart
- [ ] Graphical built-in module manager
- [ ] Inactive tab unloading and suspension policies (Balanced, Aggressive)

---

## Building and Running

### System Requirements

Development libraries for GTK4 and WebKitGTK 6.0 are required for building and running:

- **Arch Linux / Manjaro**:
  ```
  sudo pacman -S base-devel rust gtk4 webkitgtk-6.0 pkgconf
  ```

- **Ubuntu / Debian** (24.04+):
  ```
  sudo apt install build-essential cargo libgtk-4-dev libwebkitgtk-6.0-dev pkg-config
  ```

- **Fedora**:
  ```
  sudo dnf install gcc rust cargo gtk4-devel webkitgtk6.0-devel pkgconf-pkg-config
  ```

Check system library versions:
```
pkg-config --modversion gtk4
pkg-config --modversion webkitgtk-6.0
```

### Running the Browser

Clone the repository and launch the project via Cargo:

```
git clone https://github.com/rxmaneoff/light-browser ~/Light
cd ~/Light
cargo run --release
```

To build in development mode and run tests:

```
cargo test
cargo run
```

---

## Contributing

The project is in its early stages of active development. Bug reports, interface suggestions, and architectural feedback are warmly welcome!
