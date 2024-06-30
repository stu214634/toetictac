use std::io::{stdin, Read};

use crate::FIELD_SIZE;

pub fn move_up(c: usize) {
    print!("\x1B[{}A", c);
}

pub fn move_down(c: usize) {
    print!("\x1B[{}B", c);
}

pub fn move_right(c: usize) {
    print!("\x1B[{}C", c);
}

pub fn move_left(c: usize) {
    print!("\x1B[{}D", c);
}

pub fn move_col(c: usize) {
    print!("\x1B[{}G", c);
}

pub fn move_row(r: usize) {
    print!("\x1B[{};H", r + 1);
}

pub fn move_pos(r: usize, c: usize) {
    print!("\x1B[{};{}H", r + 1, c + 1);
}

pub fn move_start() {
    print!("\r");
}

pub fn backspace() {
    print!("{}", (8u8 as char));
}

pub fn clear_line(line: Option<usize>) {
    if let Some(line) = line {
        move_pos(line, 0);
    }
    move_start();
    print!("\x1B[2K\r");
}

#[deprecated(since = "0.1.1", note = "leaks memory")]
pub fn position() {
    let mut bytes = [0; 8];
    print!("\x1B[6n");
    let _ = stdin().read(&mut bytes[..]);
    print!("{}", String::from_utf8(bytes.to_vec()).unwrap());
}

pub fn move_to_field(x: usize, y: usize) {
    move_pos(y * (FIELD_SIZE), x * (FIELD_SIZE * 2 + 1));
}

pub fn center_in_current_field() {
    move_right(FIELD_SIZE - 1);
    move_down(FIELD_SIZE / 2 - 1);
}

pub fn hide_cursor() {
    print!("\x1B[25l");
}

pub fn show_cursor() {
    print!("\x1B[25h");
}
