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

fn main() {
}
