# Rust Native Calculator

A modern, highly responsive, and themeable desktop calculator built with **Rust**, **eframe (egui)**, and **Win32 Native APIs**. 

## 📌 Project Overview
The main objective of this project is to create a dynamic, fluid desktop calculator application that deviates from traditional, statically-sized native interfaces. Instead, the application behaves more like a modern responsive web app (akin to React.js + CSS Flexbox/Grid) while executing directly as a native Windows binary. 

## 🛠 Challenge & Solution

### The Challenge
1. **Fluid Responsiveness:** The calculator needed to be extremely "luwes" (flexible and resizable). Components such as typography, button dimensions, and panel spaces had to recalculate and scale proportionately in real-time as the user resizes the window, without relying on fixed grids.
2. **Native OS-Level Theming:** Beyond typical app-level color changes, the user requested distinct native-level themes, notably a **"Blur Transparent"** mode. Achieving true glass-like transparency requires deep integration with the OS composition engine, which cross-platform frameworks like `eframe` do not handle out-of-the-box.
3. **State Integrity:** Interactions within the UI (such as resetting the calculator's input state via the 'C' button) must strictly avoid causing side effects to the global application state (like the active theme or window initialization flags).

### The Solution
*   **Proportional Layout Engineering:** Instead of hardcoded pixel values, the UI relies on fractional subdivision using `egui`'s `ui.available_size()`. 25% of the viewport is dynamically reserved for the display panel. Font sizes use mathematical `clamp()` functions bounded to the `screen_rect().size().y`, and button dimensions are calculated recursively across the available width and height of the grid layout. This achieves a butter-smooth resizing experience identical to a modern web application.
*   **Bridging eframe with Desktop Window Manager (DWM):** To achieve the *Blur Transparent* theme, an Unsafe FFI layer via the `windows-sys` crate was integrated. By intercepting the window handle (`HWND`) using `FindWindowW`, the application invokes `DwmEnableBlurBehindWindow` and `DwmSetWindowAttribute`. The `egui` environment's `window_fill` and `panel_fill` alpha channels are systematically stripped (set to `TRANSPARENT`) when this theme is active, allowing the native Windows composition effect to render beautifully behind the app logic.
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
