use crate::{announce, draw_board, draw_o, draw_x, move_to_field, request_input, ANNOTATIONS};

const WINNING_BOARDS: [u16; 8] = [
    0b000_000_111,
    0b000_111_000,
    0b111_000_000,
    0b100_100_100,
    0b010_010_010,
    0b001_001_001,
    0b100_010_001,
    0b001_010_100,
];

pub struct Game {
    pub visible: bool,
    xs: u16,
    os: u16,
    x_turn: bool,
    pub rematch: bool,
}

impl Game {
    pub fn new() -> Game {
        Game {
            visible: false,
            xs: 0,
            os: 0,
            x_turn: true,
            rematch: false,
        }
    }

    fn start(&mut self) {
        self.xs = 0;
        self.os = 0;
        self.x_turn = true;
        self.rematch = false;
        if self.visible {
            draw_board(true);
        }
    }

    fn make_move(&mut self, m: u16) {
        if self.x_turn {
            self.xs |= m;
        } else {
            self.os |= m;
        }
        self.x_turn = !self.x_turn;
    }

    fn valid_moves(&self) -> u16 {
        !(0xFE00 | self.xs | self.os)
    }

    pub fn play_game(&mut self) {
        self.start();
        let x_won = WINNING_BOARDS.contains(&self.xs);
        let o_won = WINNING_BOARDS.contains(&self.os);
        if x_won | o_won {
            unsafe { announce(&format!("{} has won!", if x_won { "X" } else { "O" })) };
            let mut play_again;
            loop {
                play_again = request_input("Play again? [y/n]:");
                match play_again.as_str() {
                    "y" => {
                        self.rematch = true;
                        return;
                    }
                    "n" => std::process::exit(100),
                    _ => (),
                }
                unsafe {
                    announce(&format!(
                        "Invalid input: {} please choose [y/n]:",
                        play_again
                    ))
                };
            }
        }
        loop {
            let move_input: String = request_input("Enter a field to play:").trim().to_string();
            let read_move = ANNOTATIONS.into_iter().find(|e| e.0 == move_input);
            if let Some((_, x, y, code)) = read_move {
                if (self.valid_moves() | code) == self.valid_moves() {
                    unsafe { announce("Move was invalid!") };
                    continue;
                }
                move_to_field(x, y);
                self.make_move(code);
                if self.x_turn {
                    draw_x();
                } else {
                    draw_o();
                }
            }
            unsafe {
                announce(&format!(
                    "Move {} could not be parsed. Enter one of the field labels.",
                    move_input
                ))
            };
        }
    }
}
