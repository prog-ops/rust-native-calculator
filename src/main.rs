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
    Acrylic,
    Blur,
    FullTransparent,
}

struct Calculator {
    display: String,
    previous_value: Option<f64>,
    current_op: Option<Op>,
    new_input: bool,
    theme: Theme,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: "0".to_owned(),
            previous_value: None,
            current_op: None,
            new_input: true,
            theme: Theme::Default,
        }
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

    fn calculate(&mut self) {
        if let (Some(prev), Some(op)) = (self.previous_value, self.current_op) {
            if let Ok(current) = self.display.parse::<f64>() {
                let res = match op {
                    Op::Add => prev + current,
                    Op::Sub => prev - current,
                    Op::Mul => prev * current,
                    Op::Div => if current != 0.0 { prev / current } else { f64::NAN },
                };
                self.display = res.to_string();
                self.previous_value = Some(res); // allow chaining equals or further ops
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

    fn apply_theme_to_window(&self) {
        unsafe {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;
            use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DwmEnableBlurBehindWindow, DWM_BLURBEHIND, DWM_BB_ENABLE};
            
            let title: Vec<u16> = std::ffi::OsStr::new("Kalkulator").encode_wide().chain(std::iter::once(0)).collect();
            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if hwnd != 0 {
                // Konfigurasi Blur lawas (Windows 10 / standard BlurBehind)
                let mut bb: DWM_BLURBEHIND = std::mem::zeroed();
                bb.dwFlags = DWM_BB_ENABLE;
                bb.fEnable = if self.theme == Theme::Blur { 1 } else { 0 };
                DwmEnableBlurBehindWindow(hwnd, &bb);

                // Konfigurasi Backdrop modern (Windows 11)
                let backdrop_type: i32 = match self.theme {
                    Theme::Default => 1, // DWMSBT_NONE
                    Theme::Acrylic => 3, // DWMSBT_TRANSIENTWINDOW (Acrylic)
                    Theme::Blur => 1,    // DWMSBT_NONE (Karena sudah pakai DwmEnableBlurBehindWindow)
                    Theme::FullTransparent => 1, // DWMSBT_NONE
                };
                
                DwmSetWindowAttribute(
                    hwnd,
                    38, // DWMWA_SYSTEMBACKDROP_TYPE
                    &backdrop_type as *const _ as *const _,
                    std::mem::size_of::<i32>() as u32,
                );
            }
        }
    }
}

impl eframe::App for Calculator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        
        // Sesuaikan transparansi background aplikasi eframe
        match self.theme {
            Theme::Default => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 30);
                visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            }
            Theme::Acrylic | Theme::Blur => {
                // Gunakan transparansi penuh (0 opacity) agar efek Windows terlihat jelas
                // Jika butuh sedikit gelap, bisa pakai (0, 0, 0, 10). Tapi 0 adalah paling transparan.
                visuals.panel_fill = egui::Color32::TRANSPARENT;
                visuals.window_fill = egui::Color32::TRANSPARENT;
            }
            Theme::FullTransparent => {
                visuals.panel_fill = egui::Color32::TRANSPARENT;
                visuals.window_fill = egui::Color32::TRANSPARENT;
            }
        }
        ctx.set_visuals(visuals);

        let total_size = ctx.screen_rect().size();
        
        // Menghitung proporsi font responsif
        let display_font_size = (total_size.y * 0.08).clamp(24.0, 100.0);
        let button_font_size = (total_size.y * 0.05).clamp(16.0, 60.0);

        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(display_font_size),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(button_font_size),
        );
        ctx.set_style(style);

        // Menu Bar Atas (Modern Egui Menu)
        egui::TopBottomPanel::top("menu_panel")
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(8.0, 4.0))) // Beri margin agar tidak nempel ke ujung
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    // Membuat tombol menu terlihat lebih lega dan modern
                    ui.style_mut().spacing.button_padding = egui::vec2(12.0, 6.0);
                    
                    ui.menu_button("🎨 Tema", |ui| {
                        // Atur lebar dropdown menu
                        ui.set_min_width(160.0);
                        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 8.0);
                        
                        if ui.button("Default").clicked() {
                            self.theme = Theme::Default;
                            self.apply_theme_to_window();
                        }
                        if ui.button("Acrylic Transparent").clicked() {
                            self.theme = Theme::Acrylic;
                            self.apply_theme_to_window();
                        }
                        if ui.button("Blur Transparent").clicked() {
                            self.theme = Theme::Blur;
                            self.apply_theme_to_window();
                        }
                        if ui.button("Full Transparent").clicked() {
                            self.theme = Theme::FullTransparent;
                            self.apply_theme_to_window();
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Kita bagi sisa window jadi 25% area teks display dan 75% area tombol-tombol
            let display_height = ui.available_height() * 0.25;

            // Panel Atas untuk Teks Angka
            egui::TopBottomPanel::top("display_panel")
                .exact_height(display_height)
                .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(16.0, 10.0)))
                .show_inside(ui, |ui| {
                    // Posisi kanan-bawah agar mirip layar kalkulator asli
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                        ui.heading(&self.display);
                    });
                });

            ui.separator();

            // Panel Tengah untuk Tombol yang Flexibel
            egui::CentralPanel::default()
                .frame(egui::Frame::none().inner_margin(8.0))
                .show_inside(ui, |ui| {
                    let available = ui.available_size();
                    let spacing = 8.0;

                    // Mengkalkulasi tinggi dan lebar masing-masing tombol secara dinamis
                    // karena kita memiliki 4 kolom dan 5 baris.
                    let btn_w = (available.x - spacing * 3.0) / 4.0;
                    let btn_h = (available.y - spacing * 4.0) / 5.0;
                    let btn_size = egui::vec2(btn_w, btn_h);

                    ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                    // Baris 1
                    ui.horizontal(|ui| {
                        if ui.add_sized(egui::vec2(btn_w * 3.0 + spacing * 2.0, btn_h), egui::Button::new("C")).clicked() { 
                            *self = Default::default(); 
                        }
                        if ui.add_sized(btn_size, egui::Button::new("/")).clicked() { self.apply_op(Op::Div); }
                    });

                    // Baris 2
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("7")).clicked() { self.input_digit("7"); }
                        if ui.add_sized(btn_size, egui::Button::new("8")).clicked() { self.input_digit("8"); }
                        if ui.add_sized(btn_size, egui::Button::new("9")).clicked() { self.input_digit("9"); }
                        if ui.add_sized(btn_size, egui::Button::new("*")).clicked() { self.apply_op(Op::Mul); }
                    });

                    // Baris 3
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("4")).clicked() { self.input_digit("4"); }
                        if ui.add_sized(btn_size, egui::Button::new("5")).clicked() { self.input_digit("5"); }
                        if ui.add_sized(btn_size, egui::Button::new("6")).clicked() { self.input_digit("6"); }
                        if ui.add_sized(btn_size, egui::Button::new("-")).clicked() { self.apply_op(Op::Sub); }
                    });

                    // Baris 4
                    ui.horizontal(|ui| {
                        if ui.add_sized(btn_size, egui::Button::new("1")).clicked() { self.input_digit("1"); }
                        if ui.add_sized(btn_size, egui::Button::new("2")).clicked() { self.input_digit("2"); }
                        if ui.add_sized(btn_size, egui::Button::new("3")).clicked() { self.input_digit("3"); }
                        if ui.add_sized(btn_size, egui::Button::new("+")).clicked() { self.apply_op(Op::Add); }
                    });

                    // Baris 5
                    ui.horizontal(|ui| {
                        if ui.add_sized(egui::vec2(btn_w * 2.0 + spacing, btn_h), egui::Button::new("0")).clicked() { self.input_digit("0"); }
                        if ui.add_sized(btn_size, egui::Button::new(".")).clicked() { 
                            if !self.display.contains('.') {
                                if self.new_input {
                                    self.display = "0".to_string();
                                    self.new_input = false;
                                }
                                self.input_digit("."); 
                            }
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
        Box::new(|_cc| Box::new(Calculator::default())),
    )
}
