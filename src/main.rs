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

fn main() {
}
