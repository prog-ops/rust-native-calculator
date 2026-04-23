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

        egui::CentralPanel::default().show(ctx, |ui| {
            // Style the text
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(32.0),
            );
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(24.0),
            );

            // Display
            ui.vertical_centered_justified(|ui| {
                ui.add_space(10.0);
                ui.heading(&self.display);
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
            });

            // Layout Buttons
            let button_size = egui::vec2(50.0, 50.0);

            egui::Grid::new("calc_grid").spacing([10.0, 10.0]).show(ui, |ui| {
                // Row 1
                if ui.add_sized(button_size, egui::Button::new("7")).clicked() { self.input_digit("7"); }
                if ui.add_sized(button_size, egui::Button::new("8")).clicked() { self.input_digit("8"); }
                if ui.add_sized(button_size, egui::Button::new("9")).clicked() { self.input_digit("9"); }
                if ui.add_sized(button_size, egui::Button::new("/")).clicked() { self.apply_op(Op::Div); }
                ui.end_row();

                // Row 2
                if ui.add_sized(button_size, egui::Button::new("4")).clicked() { self.input_digit("4"); }
                if ui.add_sized(button_size, egui::Button::new("5")).clicked() { self.input_digit("5"); }
                if ui.add_sized(button_size, egui::Button::new("6")).clicked() { self.input_digit("6"); }
                if ui.add_sized(button_size, egui::Button::new("*")).clicked() { self.apply_op(Op::Mul); }
                ui.end_row();

                // Row 3
                if ui.add_sized(button_size, egui::Button::new("1")).clicked() { self.input_digit("1"); }
                if ui.add_sized(button_size, egui::Button::new("2")).clicked() { self.input_digit("2"); }
                if ui.add_sized(button_size, egui::Button::new("3")).clicked() { self.input_digit("3"); }
                if ui.add_sized(button_size, egui::Button::new("-")).clicked() { self.apply_op(Op::Sub); }
                ui.end_row();

                // Row 4
                if ui.add_sized(button_size, egui::Button::new("C")).clicked() { *self = Default::default(); }
                if ui.add_sized(button_size, egui::Button::new("0")).clicked() { self.input_digit("0"); }
                if ui.add_sized(button_size, egui::Button::new(".")).clicked() {
                    if !self.display.contains('.') {
                        if self.new_input {
                            self.display = "0".to_string();
                            self.new_input = false;
                        }
                        self.input_digit(".");
                    }
                }
                if ui.add_sized(button_size, egui::Button::new("+")).clicked() { self.apply_op(Op::Add); }
                ui.end_row();
            });

            ui.add_space(10.0);

            // Put Equal button spanning 200 units width and 50 units height
            ui.vertical_centered_justified(|ui| {
                if ui.add_sized(egui::vec2(230.0, 50.0), egui::Button::new("=")).clicked() {
                    self.calculate();
                    self.current_op = None;
                    self.new_input = true;
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([260.0, 420.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Kalkulator",
        options,
        Box::new(|_cc| Box::new(Calculator::default())),
    )
}
