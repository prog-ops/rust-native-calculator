#![windows_subsystem = "windows"]

use eframe::egui;

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, PartialEq)]
enum Theme {
    Default,
    Blur,
    Dark,
}

struct Calculator {
    display: String,
    previous_value: Option<f64>,
    current_op: Option<Op>,
    new_input: bool,
    theme: Theme,
    initialized: bool,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: "0".to_owned(),
            previous_value: None,
            current_op: None,
            new_input: true,
            theme: Theme::Default,
            initialized: false,
        }
    }
}

/// Shown instead of `NaN` / `inf` (division by zero, overflow).
const ERROR_TEXT: &str = "Error";

/// Formats a result for the display without binary floating-point artefacts
/// (`0.1 + 0.2` is shown as `0.3`, not `0.30000000000000004`).
///
/// * rounded to 10 decimal places, trailing zeros dropped, no negative zero
/// * very large / very small magnitudes use scientific notation so the text
///   stays readable in the display panel
fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return ERROR_TEXT.to_owned();
    }
    let abs = value.abs();
    if abs != 0.0 && !(1e-9..1e15).contains(&abs) {
        let sci = format!("{value:.9e}"); // e.g. "1.234500000e20"
        let (mantissa, exponent) = sci.split_once('e').unwrap_or((&sci, "0"));
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        return format!("{mantissa}e{exponent}");
    }
    let rounded = (value * 1e10).round() / 1e10;
    if rounded == 0.0 {
        "0".to_owned()
    } else {
        rounded.to_string()
    }
}

impl Calculator {
    fn input_digit(&mut self, digit: &str) {
        if self.new_input {
            self.display = String::new();
            self.new_input = false;
        }

        if self.display == "0" && digit != "." {
            self.display = digit.to_owned();
        } else {
            self.display.push_str(digit);
        }
    }

    /// Handles the "." key.
    ///
    /// When a new number is being started (after an operator or "="), the old
    /// result is still on the display, so `display.contains('.')` must only be
    /// checked for the number currently being typed.
    fn input_decimal_point(&mut self) {
        if self.new_input {
            self.display = "0".to_owned();
            self.new_input = false;
            self.display.push('.');
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    fn reset(&mut self) {
        self.display = "0".to_owned();
        self.previous_value = None;
        self.current_op = None;
        self.new_input = true;
    }

    fn calculate(&mut self) {
        if let (Some(prev), Some(op)) = (self.previous_value, self.current_op) {
            if let Ok(current) = self.display.parse::<f64>() {
                let res = match op {
                    Op::Add => prev + current,
                    Op::Sub => prev - current,
                    Op::Mul => prev * current,
                    Op::Div => {
                        if current != 0.0 {
                            prev / current
                        } else {
                            f64::NAN
                        }
                    }
                };
                if res.is_finite() {
                    self.display = format_number(res);
                    self.previous_value = Some(res); // allow chaining equals or further ops
                } else {
                    // Division by zero / overflow: show an error and drop the pending
                    // operation, so the next key press starts a fresh calculation
                    // instead of silently propagating NaN / inf.
                    self.display = ERROR_TEXT.to_owned();
                    self.previous_value = None;
                    self.current_op = None;
                    self.new_input = true;
                }
            }
        }
    }

    fn apply_op(&mut self, op: Op) {
        if !self.new_input {
            self.calculate();
        }
        if let Ok(current) = self.display.parse::<f64>() {
            self.previous_value = Some(current);
            self.current_op = Some(op);
            self.new_input = true;
        }
    }

    /// Applies the selected theme to the native window (Windows DWM only).
    ///
    /// Takes the `HWND` straight from eframe's window handle instead of
    /// searching the desktop for a window titled "Kalkulator", which could
    /// match a different window (e.g. a second instance of this app).
    #[cfg(windows)]
    fn apply_theme_to_window(&self, frame: &eframe::Frame) {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::Graphics::Dwm::{
            DwmEnableBlurBehindWindow, DwmSetWindowAttribute, DWMSBT_NONE,
            DWMWA_SYSTEMBACKDROP_TYPE, DWM_BB_ENABLE, DWM_BLURBEHIND,
        };

        let Ok(handle) = frame.window_handle() else {
            return;
        };
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return;
        };
        // raw-window-handle stores the HWND as a non-zero integer; windows-sys models it as a pointer.
        let hwnd: HWND = win32.hwnd.get() as HWND;

        // Konfigurasi Blur lawas (Windows 10 / standard BlurBehind)
        // SAFETY: `DWM_BLURBEHIND` is a plain C struct for which all-zero bytes is a valid value.
        let mut bb: DWM_BLURBEHIND = unsafe { std::mem::zeroed() };
        bb.dwFlags = DWM_BB_ENABLE;
        bb.fEnable = i32::from(self.theme == Theme::Blur);

        // Konfigurasi Backdrop modern (Windows 11)
        let backdrop_type: i32 = match self.theme {
            Theme::Default => DWMSBT_NONE,
            Theme::Blur => DWMSBT_NONE, // Karena sudah pakai DwmEnableBlurBehindWindow
            Theme::Dark => DWMSBT_NONE,
        };

        // SAFETY: `hwnd` is the live window owned by eframe for the duration of this
        // call (we are inside its `update` loop on the UI thread). `bb` and
        // `backdrop_type` outlive the calls and the size passed matches the type.
        unsafe {
            DwmEnableBlurBehindWindow(hwnd, &bb);
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                std::ptr::addr_of!(backdrop_type).cast(),
                std::mem::size_of::<i32>() as u32,
            );
        }
    }

    /// macOS: use native window-server blur (`CGSSetWindowBackgroundBlurRadius`)
    /// to replicate the frosted-glass look that Windows achieves through DWM `BlurBehind`.
    ///
    /// This operates directly at the window compositor level behind the OpenGL surface,
    /// leaving all egui UI elements, buttons, and text completely visible and clickable
    /// without obstructing subviews.
    #[cfg(target_os = "macos")]
    fn apply_theme_to_window(&self, frame: &eframe::Frame) {
        use objc2::rc::Retained;
        use objc2_app_kit::{NSColor, NSView};
        use objc2_foundation::NSInteger;
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};

        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGSMainConnectionID() -> *mut std::ffi::c_void;
            fn CGSSetWindowBackgroundBlurRadius(
                connection_id: *mut std::ffi::c_void,
                window_id: NSInteger,
                radius: i64,
            ) -> i32;
        }

        let Ok(handle) = frame.window_handle() else {
            return;
        };
        let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
            return;
        };

        // SAFETY: `ns_view` is the live NSView owned by winit/eframe for the duration of this call.
        let ns_view: Retained<NSView> =
            unsafe { Retained::retain(appkit.ns_view.as_ptr().cast()) }.expect("non-null NSView");

        let Some(ns_window) = ns_view.window() else {
            return;
        };

        let window_number = ns_window.windowNumber();

        if self.theme == Theme::Blur {
            ns_window.setOpaque(false);
            let clear = NSColor::clearColor();
            ns_window.setBackgroundColor(Some(&clear));
            // SAFETY: CGSSetWindowBackgroundBlurRadius is the standard macOS compositor API
            // used by winit and Terminal.app to apply frosted-glass blur behind the window.
            unsafe {
                let conn = CGSMainConnectionID();
                CGSSetWindowBackgroundBlurRadius(conn, window_number, 64);
            }
            ns_window.invalidateShadow();
        } else {
            // Non-blur themes: clear blur and restore opaque background.
            // SAFETY: Setting blur radius to 0 disables background blurring.
            unsafe {
                let conn = CGSMainConnectionID();
                CGSSetWindowBackgroundBlurRadius(conn, window_number, 0);
            }
            ns_window.setOpaque(true);
            ns_window.setBackgroundColor(None);
            ns_window.invalidateShadow();
        }
    }

    /// Linux has no standard window-compositor blur API.
    /// The theme is handled purely by egui visuals + `clear_color`.
    #[cfg(not(any(windows, target_os = "macos")))]
    fn apply_theme_to_window(&self, _frame: &eframe::Frame) {}
}

impl eframe::App for Calculator {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if !self.initialized {
            self.apply_theme_to_window(frame);
            self.initialized = true;
        }

        let mut visuals = egui::Visuals::dark();

        // Sesuaikan transparansi background aplikasi eframe
        match self.theme {
            Theme::Default => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
                visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            }
            Theme::Blur => {
                visuals.panel_fill = egui::Color32::TRANSPARENT;
                visuals.window_fill = egui::Color32::from_rgba_premultiplied(30, 30, 30, 240);
            }
            Theme::Dark => {
                visuals.panel_fill = egui::Color32::BLACK;
                visuals.window_fill = egui::Color32::from_rgb(20, 20, 20);
            }
        }

        let total_size = ctx.content_rect().size();

        // Menghitung proporsi font responsif
        let display_font_size = (total_size.y * 0.08).clamp(24.0, 100.0);
        let button_font_size = (total_size.y * 0.05).clamp(16.0, 60.0);

        let mut style = (*ctx.global_style()).clone();
        style.visuals = visuals;
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(display_font_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(button_font_size),
        );
        // The root `Ui` drives the panels/buttons of this frame; the global style
        // is what popups such as the theme menu are created from.
        // Since egui 0.37 `Button` text defaults to `TextStyle::Body`, so every
        // button-only `Ui` below explicitly opts into `TextStyle::Button`.
        ui.set_style(style.clone());
        ctx.set_global_style(style);

        // Menu Bar Atas (Modern Egui Menu)
        egui::Panel::top("menu_panel")
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(8, 4))) // Beri margin agar tidak nempel ke ujung
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    // Membuat tombol menu terlihat lebih lega dan modern
                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 6.0);
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Button);

                    ui.menu_button("🎨", |ui| {
                        // Atur lebar dropdown menu
                        ui.set_min_width(160.0);
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 8.0);
                        // The button font scales with the window; let the menu grow
                        // with it instead of wrapping "Blur Transparent" onto two lines.
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Button);

                        if ui.button("Default").clicked() {
                            self.theme = Theme::Default;
                            self.apply_theme_to_window(frame);
                        }
                        if ui.button("Blur Transparent").clicked() {
                            self.theme = Theme::Blur;
                            self.apply_theme_to_window(frame);
                        }
                        if ui.button("Dark").clicked() {
                            self.theme = Theme::Dark;
                            self.apply_theme_to_window(frame);
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            // Kita bagi sisa window jadi 25% area teks display dan 75% area tombol-tombol
            let display_height = ui.available_height() * 0.25;

            // Panel Atas untuk Teks Angka
            egui::Panel::top("display_panel")
                .exact_size(display_height)
                .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(16, 10)))
                .show(ui, |ui| {
                    // Posisi kanan-bawah agar mirip layar kalkulator asli
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                        ui.heading(&self.display);
                    });
                });

            ui.separator();

            // Panel Tengah untuk Tombol yang Flexibel
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.inner_margin(8))
                .show(ui, |ui| {
                    let available = ui.available_size();
                    let spacing = 8.0;

                    // Mengkalkulasi tinggi dan lebar masing-masing tombol secara dinamis
                    // karena kita memiliki 4 kolom dan 5 baris.
                    let btn_w = (available.x - spacing * 3.0) / 4.0;
                    let btn_h = (available.y - spacing * 4.0) / 5.0;
                    let btn_size = egui::vec2(btn_w, btn_h);

                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Button);

                    // Baris 1
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                egui::vec2(btn_w * 3.0 + spacing * 2.0, btn_h),
                                egui::Button::new("C"),
                            )
                            .clicked()
                        {
                            self.reset();
                        }
                        if ui.add_sized(btn_size, egui::Button::new("/")).clicked() {
                            self.apply_op(Op::Div);
                        }
                    });

                    // Baris 2
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("7")).clicked() {
                            self.input_digit("7");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("8")).clicked() {
                            self.input_digit("8");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("9")).clicked() {
                            self.input_digit("9");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("*")).clicked() {
                            self.apply_op(Op::Mul);
                        }
                    });

                    // Baris 3
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("4")).clicked() {
                            self.input_digit("4");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("5")).clicked() {
                            self.input_digit("5");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("6")).clicked() {
                            self.input_digit("6");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("-")).clicked() {
                            self.apply_op(Op::Sub);
                        }
                    });

                    // Baris 4
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("1")).clicked() {
                            self.input_digit("1");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("2")).clicked() {
                            self.input_digit("2");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("3")).clicked() {
                            self.input_digit("3");
                        }
                        if ui.add_sized(btn_size, egui::Button::new("+")).clicked() {
                            self.apply_op(Op::Add);
                        }
                    });

                    // Baris 5
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                egui::vec2(btn_w * 2.0 + spacing, btn_h),
                                egui::Button::new("0"),
                            )
                            .clicked()
                        {
                            self.input_digit("0");
                        }
                        if ui.add_sized(btn_size, egui::Button::new(".")).clicked() {
                            self.input_decimal_point();
                        }
                        if ui.add_sized(btn_size, egui::Button::new("=")).clicked() {
                            self.calculate();
                            self.current_op = None;
                            self.new_input = true;
                        }
                    });
                });
        });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        match self.theme {
            Theme::Dark => [0.0, 0.0, 0.0, 1.0], // hitam pekat (opaque di semua OS)
            Theme::Blur => [0.0, 0.0, 0.0, 0.0], // biar DWM blur tembus
            Theme::Default => [0.118, 0.118, 0.118, 1.0], // rgb(30,30,30)
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([300.0, 480.0])
            .with_min_inner_size([200.0, 300.0])
            .with_resizable(true)
            .with_transparent(true), // Harus true agar background bisa transparent/acrylic
        ..Default::default()
    };
    eframe::run_native(
        "Kalkulator",
        options,
        Box::new(|cc| {
            // The calculator is designed around dark visuals; don't follow the OS
            // light/dark preference (egui's default since 0.29).
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(Calculator::default()))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives the calculator exactly like the buttons do ("C", digits, ".", "+", "-", "*", "/", "=").
    fn press(calc: &mut Calculator, keys: &str) {
        for key in keys.split_whitespace() {
            match key {
                "C" => calc.reset(),
                "+" => calc.apply_op(Op::Add),
                "-" => calc.apply_op(Op::Sub),
                "*" => calc.apply_op(Op::Mul),
                "/" => calc.apply_op(Op::Div),
                "=" => {
                    calc.calculate();
                    calc.current_op = None;
                    calc.new_input = true;
                }
                "." => calc.input_decimal_point(),
                digit => calc.input_digit(digit),
            }
        }
    }

    fn run(keys: &str) -> String {
        let mut calc = Calculator::default();
        press(&mut calc, keys);
        calc.display
    }

    #[test]
    fn basic_arithmetic() {
        assert_eq!(run("1 2 + 3 ="), "15");
        assert_eq!(run("9 - 1 2 ="), "-3");
        assert_eq!(run("6 * 7 ="), "42");
        assert_eq!(run("8 / 4 ="), "2");
    }

    #[test]
    fn chained_operations_and_equals() {
        assert_eq!(run("2 + 3 * 4 ="), "20"); // left-to-right, like a basic calculator
        assert_eq!(run("5 + 3 = + 2 ="), "10"); // continue from a result
        assert_eq!(run("5 + ="), "10"); // operand defaults to the display
        assert_eq!(run("5 + 3 = ="), "8"); // repeated "=" is stable
    }

    #[test]
    fn digit_entry_rules() {
        assert_eq!(run("0 0 7"), "7"); // no leading zeros
        assert_eq!(run(". 5"), "0.5"); // "." on a fresh input
        assert_eq!(run("1 . . 5"), "1.5"); // only one decimal point
        assert_eq!(run("1 + 2 = 7"), "7"); // a digit after "=" starts a new number
        assert_eq!(run("1 + ."), "0.");
        // "." right after an operator / "=" must start "0." even if the number
        // still on the display already has a decimal point
        assert_eq!(run("1 . 5 + . 5 ="), "2");
        assert_eq!(run("1 . 5 + 1 = . 5"), "0.5");
    }

    #[test]
    fn float_artifacts_are_not_shown() {
        assert_eq!(run(". 1 + . 2 ="), "0.3");
        assert_eq!(run("3 * 1 . 1 ="), "3.3");
        assert_eq!(run("1 / 3 ="), "0.3333333333");
        assert_eq!(run("1 0 0 / 7 ="), "14.2857142857");
    }

    #[test]
    fn division_by_zero_shows_error_and_recovers() {
        let mut calc = Calculator::default();
        press(&mut calc, "1 / 0 =");
        assert_eq!(calc.display, ERROR_TEXT);
        assert!(calc.previous_value.is_none());
        assert!(calc.current_op.is_none());

        // the error must not leak into the next calculation
        press(&mut calc, "2 + 2 =");
        assert_eq!(calc.display, "4");

        // ... also when the error is produced mid-chain by an operator key:
        // the operator that triggered the error is dropped, "5" starts fresh
        assert_eq!(run("1 / 0 + 5 ="), "5");
        // an operator pressed while "Error" is shown is ignored, a digit starts fresh
        assert_eq!(run("1 / 0 = + 5 ="), "5");
        assert_eq!(run("1 / 0 = 9"), "9");
    }

    #[test]
    fn format_number_edge_cases() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(-0.0), "0");
        assert_eq!(format_number(-1e-11), "-1e-11");
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(-2.5), "-2.5");
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(1e15), "1e15");
        assert_eq!(format_number(1.2345e20), "1.2345e20");
        assert_eq!(format_number(1e-10), "1e-10");
        assert_eq!(format_number(f64::NAN), ERROR_TEXT);
        assert_eq!(format_number(f64::INFINITY), ERROR_TEXT);
        assert_eq!(format_number(f64::NEG_INFINITY), ERROR_TEXT);
    }

    #[test]
    fn large_results_stay_parseable_for_chaining() {
        // "1e15" must round-trip through `display.parse::<f64>()` in `apply_op`
        assert_eq!(run("1 0 0 0 0 0 0 0 * 1 0 0 0 0 0 0 0 0 = * 1 0 ="), "1e16");
    }

    #[test]
    fn reset_only_clears_calculation_state() {
        let mut calc = Calculator {
            theme: Theme::Blur,
            initialized: true,
            ..Calculator::default()
        };
        press(&mut calc, "1 + 2");
        calc.reset();
        assert_eq!(calc.display, "0");
        assert!(calc.previous_value.is_none());
        assert!(calc.current_op.is_none());
        assert!(calc.new_input);
        assert!(calc.theme == Theme::Blur);
        assert!(calc.initialized);
    }
}
