# Rust Native Calculator

A modern, highly responsive, and themeable desktop calculator built with **Rust**, **eframe (egui)**, and **Win32 Native APIs** (on Windows).

| Platform | Build | Notes |
|----------|-------|-------|
| Windows  | ✅ | All three themes, incl. native DWM *Blur Transparent* |
| macOS    | ✅ | *Blur Transparent* = transparent window (macOS has no DWM, so no blur) |
| Linux    | ✅ | same as macOS |

## 🚀 Getting Started

**Prerequisites:** [rustup](https://rustup.rs). The toolchain (stable + clippy + rustfmt) is pinned in
`rust-toolchain.toml` and installed automatically the first time you build. `Cargo.lock` is committed,
so everybody builds the exact same dependency versions. Minimum supported Rust version: **1.95**.

### RustRover (recommended)
1. `File ▸ Open…` the project folder (the one containing `Cargo.toml`).
2. Shared run configurations are picked up from `.run/` and appear in the run-configuration dropdown:
   * **Run Calculator** – `cargo run` (debug)
   * **Run Calculator (release)** – optimised build
   * **Test** – unit tests of the calculator engine
   * **Lint (clippy -D warnings)** / **Format check** – exactly what CI enforces
3. Select **Run Calculator** and press ▶. No warnings are emitted on Windows or macOS.

> Tip: in *Settings ▸ Rust ▸ External Linters* choose **Clippy** so the editor shows the same
> diagnostics as CI.

### Terminal
```bash
cargo run            # debug build
cargo run --release  # optimised build
cargo test           # calculator-engine unit tests
cargo lint           # alias: clippy --all-targets -- -D warnings
cargo fmt-check      # alias: fmt --all -- --check
```

Cross-platform compile/lint check from **any** host OS (catches platform-specific compile errors
without needing the other machine; linking is the only step that still requires the native SDK):
```bash
rustup target add x86_64-pc-windows-msvc aarch64-apple-darwin x86_64-apple-darwin   # once
cargo check-win      # Windows
cargo check-mac      # macOS (Apple Silicon)
cargo check-mac-intel
```

## 📌 Project Overview
The main objective of this project is to create a dynamic, fluid desktop calculator application that deviates from traditional, statically-sized native interfaces. Instead, the application behaves more like a modern responsive web app (akin to React.js + CSS Flexbox/Grid) while executing directly as a native desktop binary.

## 🛠 Challenge & Solution

### The Challenge
1. **Fluid Responsiveness:** The calculator needed to be extremely flexible and resizable. Components such as typography, button dimensions, and panel spaces had to recalculate and scale proportionately in real-time as the user resizes the window, without relying on fixed grids.
2. **Native OS-Level Theming:** Beyond typical app-level color changes, the user requested distinct native-level themes, notably a **"Blur Transparent"** mode. Achieving true glass-like transparency requires deep integration with the OS composition engine, which cross-platform frameworks like `eframe` do not handle out-of-the-box.
3. **State Integrity:** Interactions within the UI (such as resetting the calculator's input state via the 'C' button) must strictly avoid causing side effects to the global application state (like the active theme or window initialization flags).

### The Solution
*   **Proportional Layout Engineering:** Instead of hardcoded pixel values, the UI relies on fractional subdivision using `egui`'s `ui.available_size()`. 25% of the viewport is dynamically reserved for the display panel. Font sizes use mathematical `clamp()` functions bounded to the `screen_rect().size().y`, and button dimensions are calculated recursively across the available width and height of the grid layout. This achieves a butter-smooth resizing experience identical to a modern web application.
*   **Bridging eframe with Desktop Window Manager (DWM):** To achieve the *Blur Transparent* theme, an Unsafe FFI layer via the `windows-sys` crate was integrated. The window handle (`HWND`) is taken directly from eframe's `Frame` (`raw-window-handle`), and the application invokes `DwmEnableBlurBehindWindow` and `DwmSetWindowAttribute`. The `egui` environment's `window_fill` and `panel_fill` alpha channels are systematically stripped (set to `TRANSPARENT`) when this theme is active, allowing the native Windows composition effect to render beautifully behind the app logic. The whole FFI path is compiled only on Windows (`#[cfg(windows)]` + a target-specific dependency); other platforms get a no-op, which keeps the crate building everywhere.
*   **Targeted State Mutability:** The `Calculator`'s global state (`struct Calculator`) was refined. Rather than utilizing `Default::default()` for runtime operations (which destructively wipes out theme persistence and initialization markers), a bespoke `reset()` method was implemented. This isolates the clearing of the calculation-specific fields (`display`, `current_op`, `previous_value`) from the application-level lifecycle fields (`theme`, `initialized`).

## 🏗 Architecture & Clean Code Notes

As a Senior Engineering endeavor, it is important to clarify the architectural scope of this iteration.

**Was Clean Architecture Applied Here?**
**No, strict Clean Architecture was intentionally deferred.** 

The primary focus of this project phase was purely **functional delivery and technical feasibility**—specifically proving that complex behaviors (fluid immediate-mode UI layouts coupled with low-level Win32 FFI hooks) could operate seamlessly together. 

As a result:
*   The application currently utilizes a **Monolithic State Pattern** where the Domain Logic (math calculations), Presentation Logic (egui rendering), and Infrastructure Logic (Win32 FFI calls) all reside together inside a single `Calculator` struct within `main.rs`.
*   While the code is clean, readable, and well-commented, it lacks boundary separations (e.g., isolating the FFI logic into a standalone `windows_integration` module, or decoupling the math state-machine so it can be independently unit-tested).

**Future Improvements:**
If this project were to be scaled, the next step would be applying Domain-Driven Design (DDD) principles: extracting the core calculator engine out of the UI layer, writing unit tests for the operator logic, and encapsulating the OS-specific native API calls behind an abstraction trait to maintain true cross-platform viability.

## 🐛 Bug Fixes (this revision)

| # | Bug | Fix |
|---|-----|-----|
| 1 | **Did not compile on macOS/Linux** – `std::os::windows::ffi::OsStrExt` and `windows_sys::Win32::*` were used unconditionally and `windows-sys` was a dependency for every target. | Win32 code is behind `#[cfg(windows)]` with a no-op counterpart; `windows-sys`/`raw-window-handle` moved to `[target.'cfg(windows)'.dependencies]`. |
| 2 | **Crash at start-up on current macOS** ("window-frame quit unexpectedly") and a *future-incompatibility* warning on macOS builds – eframe 0.27's macOS backend (winit 0.29, `cocoa`, `icrate`, pre-release `objc2 0.3-beta` / `block2 alpha` via `accesskit_macos 0.10`) is unmaintained and predates macOS 14/15/26. | Upgraded to the current **eframe 0.37 / winit 0.30 / objc2 0.6** stack (also silences RustRover's "newer version available" inspection, together with `windows-sys 0.61`). winit 0.30 uses the same DWM blur-behind mechanism on Windows, so the Windows theming is unchanged. Migrated the egui API changes: `App::update` → `App::ui`, `TopBottomPanel` → `Panel`, per-theme styles, `menu::bar` → `MenuBar`, `Frame::none()` → `Frame::NONE`, integer `Margin`; buttons opt into `TextStyle::Button` explicitly (egui 0.37 buttons default to `Body`), and the bundled emoji font is enabled via `egui/monochrome_emoji_fonts`. |
| 3 | `0.1 + 0.2` displayed `0.30000000000000004` (raw `f64::to_string`). | `format_number()` rounds to 10 decimals, trims zeros, avoids `-0`, uses scientific notation for huge/tiny values. |
| 4 | `1 / 0` showed `NaN` (overflow showed `inf`) and – because Rust parses `"NaN"` back into a float – the error silently propagated through every following operation. | Shows **Error**, clears the pending operation; the next key starts a fresh calculation. |
| 5 | The **"." key was ignored** after an operator or `=` whenever the number still on the display already contained a decimal point (`0.1 + .2` actually computed `0.1 + 2`). | `input_decimal_point()` checks `new_input` before looking at the old display text. |
| 6 | The window was located by **title** (`FindWindowW(null, "Kalkulator")`) – with two instances open the theme was applied to the wrong window. | Uses the real `HWND` from `eframe::Frame`. |
| 7 | **Dark** theme was black on Windows only as a side effect of disabling DWM blur-behind; on macOS it would have been see-through. | Explicit opaque black `clear_color` / `panel_fill` – same look on Windows, correct look elsewhere. |
| 8 | Theme menu label wrapped onto two lines once the window (and therefore the scaled button font) got large. | Menu uses `TextWrapMode::Extend`. |
| 9 | `clippy::collapsible_if` warning + rustfmt drift. | Fixed; `cargo lint` and `cargo fmt-check` are clean on Windows, macOS and Linux. |

Regression tests for 3 – 5 and the operator/chaining rules live in `src/main.rs` (`#[cfg(test)]`, not part of the binary): `cargo test`.

## 🧰 Development Tooling

| File | Purpose |
|------|---------|
| `rust-toolchain.toml` | Same channel + `clippy`/`rustfmt` components for everyone, auto-installed by rustup. |
| `Cargo.lock` | Committed (this is a binary crate) → reproducible builds; CI builds with `--locked`. |
| `Cargo.toml` → explicit eframe features | `glow` renderer (eframe's new default, wgpu, can't do the transparent window the Blur theme needs on Windows and currently fails to compile there), no `clipboard`/`wayland` (both reference the yanked `clipboard-win` crate and break fresh lockfile generation); Linux gets Wayland through `winit`/`glutin` features directly, loading libwayland at runtime (`dlopen`) so no dev headers are needed. |
| `Cargo.toml` → `[lints]` | Project-wide lint policy (`unsafe_op_in_unsafe_fn`, `undocumented_unsafe_blocks`, …) applied by `cargo clippy`/RustRover. |
| `Cargo.toml` → `[profile.dev.package."*"]` | Dependencies (egui/winit/glow) are optimised in *debug* builds, so the immediate-mode UI stays smooth while developing without losing debug info for our own code. |
| `Cargo.toml` → `[profile.release]` | LTO, single codegen unit, stripped symbols → small, fast binary. |
| `.cargo/config.toml` | `cargo lint`, `cargo fmt-check`, `cargo check-win`, `cargo check-mac` aliases. |
| `.run/*.run.xml` | Shared RustRover run configurations (Run / Release / Test / Lint / Format). |
| `.github/workflows/ci.yml` | fmt + clippy (`-D warnings`) + build + test on **Windows, macOS and Linux**; uploads the release binary as an artifact. |
| `rustfmt.toml`, `.editorconfig`, `.gitattributes` | Consistent formatting and **LF line endings** across Windows/macOS contributors – no CRLF churn in diffs. |
| `.gitignore` | No longer ignores the project's own `.cargo/config.toml`; `.run/` is versioned while `.idea/` stays local. |
