#![windows_subsystem = "windows"]

use eframe::egui;

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

struct Calculator {
    display: String,
    previous_value: Option<f64>,
    current_op: Option<Op>,
    new_input: bool,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            display: "0".to_owned(),
            previous_value: None,
            current_op: None,
            new_input: true,
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
}

impl eframe::App for Calculator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

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

        egui::CentralPanel::default().show(ctx, |ui| {
            // Kita bagi window jadi 25% area teks display dan 75% area tombol-tombol
            let display_height = total_size.y * 0.25;

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
            .with_resizable(true), // Diatur true agar responsif
        ..Default::default()
    };
    eframe::run_native(
        "Kalkulator",
        options,
        Box::new(|_cc| Box::new(Calculator::default())),
    )
}
