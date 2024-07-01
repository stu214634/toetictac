use std::{
    io::{self, stdout, Read, Write},
    sync::Mutex,
};

use game::Game;

use crate::cursor::*;

mod cursor;
mod game;

const FIELD_SIZE: usize = 5usize;
const ANNOTATIONS: [(&str, usize, usize, u16); 9] = [
    ("tl", 0, 0, 0b100_000_000),
    ("t", 1, 0, 0b010_000_000),
    ("tr", 2, 0, 0b001_000_000),
    ("l", 0, 1, 0b000_100_000),
    ("m", 1, 1, 0b000_010_000),
    ("r", 2, 1, 0b000_001_000),
    ("bl", 0, 2, 0b000_000_100),
    ("b", 1, 2, 0b000_000_010),
    ("br", 2, 2, 0b000_000_001),
];
const ANNOUNCEMENT_LINES: usize = 3usize;
static ANNOUNCEMENT_BUFFER: Mutex<Vec<String>> = Mutex::new(Vec::new());
const INPUT_LINE: usize = 3 * FIELD_SIZE + ANNOUNCEMENT_LINES + 5;

fn main() {
    announce("Welcome");
    loop {
        let mut game = Game::new();
        game.visible = true;
        game.play_game();
        if !game.rematch {
            break;
        }
    }
}

fn drawline() {
    move_right(FIELD_SIZE * 2);
    print!("|");
    move_right(FIELD_SIZE * 2);
    print!("|");
    move_down(1);
    move_start();
}

fn draw_board(clear: bool) {
    announce("drawing board");
    const FIELD_SIZE_DOUBLE: usize = FIELD_SIZE * 2;
    if clear {
        announce("clearing board");
        move_to_field(0, 0);
        for i in 0..(FIELD_SIZE * 3) {
            clear_line(Some(i));
        }
    }
    move_to_field(0, 0);
    for i in 1..(FIELD_SIZE * 3) {
        match i {
            FIELD_SIZE | FIELD_SIZE_DOUBLE => {
                print!("{}", "-".repeat(FIELD_SIZE * 6usize));
                move_down(1);
                move_start();
            }
            _ => drawline(),
        }
    }
    for (label, x, y, _) in ANNOTATIONS {
        move_to_field(x, y);
        center_in_current_field();
        print!("{}", label);
    }
    let _ = stdout().flush();
    announce("board drawn");
}

fn announce(message: &str) {
    hide_cursor();
    let _ = stdout().flush();
    let mut buffer = ANNOUNCEMENT_BUFFER.lock().unwrap();
    if buffer.len() == ANNOUNCEMENT_LINES {
        buffer.remove(0);
    }
    buffer.push(message.to_string());
    for (i, announcement) in buffer.iter().enumerate() {
        clear_line(Some(FIELD_SIZE * 3 + i));
        move_down(ANNOUNCEMENT_LINES);
        clear_line(None);
        print!("{}", announcement);
    }
    let _ = stdout().flush();
    show_cursor();
}

fn request_input(message: &str) -> String {
    clear_line(Some(INPUT_LINE));
    print!("{}", message);
    let _ = stdout().flush();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    return name.trim().to_string();
}

fn draw_x() {
    for y in 1..FIELD_SIZE {
        move_right(y * 2 - 1);
        print!("\\_");
        move_left((y * 2) + 1);
        move_right(FIELD_SIZE * 2 - y * 2 - 1);
        print!("_/");
        move_down(1);
        move_left(FIELD_SIZE * 2 - y * 2 + 1);
    }
    let _ = stdout().flush();
}

fn draw_o() {
    move_right(FIELD_SIZE - (FIELD_SIZE + 1) / 2);
    print!("{}", "-".repeat(FIELD_SIZE + 1));
    move_left(FIELD_SIZE + 2);
    for _ in 1..(FIELD_SIZE - 2) {
        move_down(1);
        print!("|{}|", " ".repeat(FIELD_SIZE + 1));
        move_left(FIELD_SIZE + 3);
    }
    move_down(1);
    move_right(1);
    print!("{}", "-".repeat(FIELD_SIZE + 1));
    let _ = stdout().flush();
}
