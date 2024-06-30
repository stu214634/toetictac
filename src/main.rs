mod cursor;
mod game;

use core::time;
use std::{
    io::{self, stdin, stdout, Read, Write},
    thread::sleep,
    time::Duration,
};

use game::Game;

use crate::cursor::*;

const FIELD_SIZE: usize = 4usize;
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
static mut ANNOUNCEMENT_BUFFER: Vec<String> = Vec::new();
const INPUT_LINE: usize = 3 * FIELD_SIZE + ANNOUNCEMENT_LINES + 5;

fn main() {
    unsafe { announce("Welcome") };
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
    unsafe { announce("drawing board") };
    const FIELD_SIZE_DOUBLE: usize = FIELD_SIZE * 2;
    if clear {
        unsafe { announce("clearing board") };
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
    unsafe { announce("board drawn") };
}

unsafe fn announce(message: &str) {
    hide_cursor();
    let _ = stdout().flush();
    if ANNOUNCEMENT_BUFFER.len() == ANNOUNCEMENT_LINES {
        ANNOUNCEMENT_BUFFER.remove(0);
    }
    ANNOUNCEMENT_BUFFER.push(message.to_string());
    for (i, announcement) in ANNOUNCEMENT_BUFFER.iter().enumerate() {
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
    let mut buf = [0; 256];
    let _ = stdin().read(&mut buf);
    String::from_utf8(buf.to_vec()).unwrap_or("NO_PARSE".to_string())
}

fn draw_x() {}

fn draw_o() {}
