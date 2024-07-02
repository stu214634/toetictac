use std::{thread::sleep, time::Duration};

use rand::seq::SliceRandom;

use crate::{announce, draw_board, draw_o, draw_x, move_to_field, request_input, ANNOTATIONS};

const BRAINS: [&str; 3] = ["Perfect", "Human", "Random"];

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

const DRAW_BOARD: u16 = 0b111_111_111;

#[derive(Clone)]
pub struct Game {
    xs: u16,
    os: u16,
    x_turn: bool,
    rematch: bool,
    has_human: bool,
}

impl Game {
    pub fn new() -> Game {
        Game {
            xs: 0,
            os: 0,
            x_turn: true,
            rematch: false,
            has_human: false,
        }
    }

    pub fn make_move_copy(&self, m: u16) -> Game {
        let mut game = Game {
            xs: self.xs,
            os: self.os,
            x_turn: self.x_turn,
            rematch: false,
            has_human: self.has_human,
        };
        game.make_move(m);
        game
    }

    fn start(&mut self) {
        self.xs = 0;
        self.os = 0;
        self.x_turn = true;
        self.rematch = false;
        self.has_human = false;
        draw_board(true);
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

    pub fn valid_moves_vec(&self) -> Vec<u16> {
        let mut valid = self.valid_moves();
        let mut vec_valid = Vec::new();
        for i in 0..9 {
            if valid % 2 != 0 {
                vec_valid.push(1 << i);
            }
            valid >>= 1;
        }
        vec_valid
    }

    pub fn play_games(&mut self) {
        loop {
            self.play_game();
            if !self.rematch {
                return;
            }
        }
    }

    fn announce_game_over(&mut self, x_won: bool, o_won: bool) {
        if x_won {
            announce("X has won!");
        } else if o_won {
            announce("O has won!");
        } else {
            announce("Draw!");
        }
        loop {
            let play_again = request_input("Play again? [y/n]:");
            match play_again.as_str() {
                "y" => {
                    self.rematch = true;
                    return;
                }
                "n" => std::process::exit(100),
                _ => (),
            }
            announce(&format!(
                "Invalid input: {} please choose [y/n]:",
                play_again
            ));
        }
    }

    pub fn game_over(&self) -> (bool, bool, bool) {
        let x_won = WINNING_BOARDS.into_iter().any(|e| e & self.xs == e);
        let o_won = WINNING_BOARDS.into_iter().any(|e| e & self.os == e);

        if x_won | o_won | (self.xs | self.os == DRAW_BOARD) {
            return (true, x_won, o_won);
        }

        (false, false, false)
    }

    fn draw_move(&self, x: usize, y: usize) {
        move_to_field(x, y);
        if self.x_turn {
            draw_x();
        } else {
            draw_o();
        }
    }

    fn human_move(&self) -> u16 {
        loop {
            let move_input: String = request_input("Enter a field to play:");
            let read_move = ANNOTATIONS.into_iter().find(|e| e.0 == move_input);
            if let Some((_, _, _, code)) = read_move {
                if (self.valid_moves() & code) == 0 {
                    announce("Move was invalid!");
                    continue;
                }
                return code;
            }
            announce(&format!(
                "Move {} could not be parsed. Enter one of the field labels.",
                move_input
            ));
        }
    }

    fn choose_brain(&mut self) -> Box<dyn Brain> {
        loop {
            let brain_input: String = request_input(&format!("Choose an AI {:?}: ", BRAINS));
            match brain_input.as_str() {
                "Perfect" => return Box::new(GameTree::from_game(self.clone())),
                "Random" => return Box::new(RandomBrain {}),
                "Human" => {
                    self.has_human = true;
                    return Box::new(HumanBrain {});
                }
                _ => {
                    announce(&format!(
                        "Brain {} could not be parsed. Enter one of {:?}",
                        brain_input, BRAINS
                    ));
                }
            }
        }
    }

    pub fn play_game(&mut self) {
        self.start();
        let mut x_brain = &*self.choose_brain();
        let mut o_brain = &*self.choose_brain();
        loop {
            let m = if self.x_turn {
                x_brain.best_move(self)
            } else {
                o_brain.best_move(self)
            };
            let (label, x, y, _) = ANNOTATIONS.into_iter().find(|c| c.3 == m).unwrap();
            self.draw_move(x, y);
            self.make_move(m);
            x_brain = x_brain.advance(m);
            o_brain = o_brain.advance(m);
            announce(&format!("Moved: {}", label));
            let (over, x_won, o_won) = self.game_over();
            if over {
                self.announce_game_over(x_won, o_won);
                return;
            }
            if !self.has_human {
                let dur = Duration::from_secs(1);
                sleep(dur);
            }
        }
    }
}

pub trait Brain {
    fn best_move(&self, game: &Game) -> u16;
    fn advance(&self, m: u16) -> &dyn Brain;
}

#[derive(Clone)]
struct RandomBrain;

impl Brain for RandomBrain {
    fn best_move(&self, game: &Game) -> u16 {
        *game
            .valid_moves_vec()
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    fn advance(&self, _m: u16) -> &dyn Brain {
        self
    }
}

struct HumanBrain;
impl Brain for HumanBrain {
    fn best_move(&self, game: &Game) -> u16 {
        game.human_move()
    }

    fn advance(&self, _m: u16) -> &dyn Brain {
        self
    }
}

#[derive(Clone)]
struct GameTree {
    x_inevitable: bool,
    o_inevitable: bool,
    draw_inevitable: bool,
    root: Game,
    children: Vec<(GameTree, u16)>,
}

impl GameTree {
    pub fn from_game(game: Game) -> GameTree {
        let (game_over, x_won, o_won) = game.game_over();

        if game_over {
            return GameTree {
                x_inevitable: x_won,
                o_inevitable: o_won,
                draw_inevitable: !(x_won || o_won),
                root: game,
                children: Vec::new(),
            };
        }

        let children = game
            .valid_moves_vec()
            .into_iter()
            .map(|m| (Self::from_game(game.make_move_copy(m)), m))
            .collect::<Vec<(GameTree, u16)>>();

        let x_inevitable = (game.x_turn && children.iter().any(|c| c.0.x_inevitable))
            || children.iter().all(|c| c.0.x_inevitable);

        let o_inevitable = (!game.x_turn && children.iter().any(|c| c.0.o_inevitable))
            || children.iter().all(|c| c.0.o_inevitable);

        let draw_inevitable = children.iter().all(|c| c.0.draw_inevitable);

        GameTree {
            x_inevitable,
            o_inevitable,
            draw_inevitable,
            root: game,
            children,
        }
    }
}

impl Brain for GameTree {
    fn best_move(&self, _game: &Game) -> u16 {
        let for_x = self.root.x_turn;
        if for_x {
            if let Some(wins) = self.children.iter().find(|c| c.0.x_inevitable) {
                announce("Found Win");
                wins.1
            } else {
                announce("Random move");
                let non_losing: Vec<u16> = self
                    .children
                    .iter()
                    .filter(|c| !c.0.o_inevitable)
                    .map(|c| c.1)
                    .collect();
                *non_losing.choose(&mut rand::thread_rng()).unwrap()
            }
        } else if let Some(wins) = self.children.iter().find(|c| c.0.o_inevitable) {
            announce("Found Win");
            wins.1
        } else {
            announce("Random move");
            let non_losing: Vec<u16> = self
                .children
                .iter()
                .filter(|c| !c.0.x_inevitable)
                .map(|c| c.1)
                .collect();
            *non_losing.choose(&mut rand::thread_rng()).unwrap()
        }
    }

    fn advance(&self, m: u16) -> &dyn Brain {
        &self.children.iter().find(|c| c.1 == m).unwrap().0
    }
}
