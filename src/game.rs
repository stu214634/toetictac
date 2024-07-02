use std::any;

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

const DRAW_BOARD: u16 = 0b111_111_111;

#[derive(Clone)]
pub struct Game {
    xs: u16,
    os: u16,
    x_turn: bool,
    rematch: bool,
    against_ai: bool,
    ai_plays_x: bool,
}

impl Game {
    pub fn new(against_ai: bool) -> Game {
        Game {
            xs: 0,
            os: 0,
            x_turn: true,
            rematch: false,
            against_ai,
            ai_plays_x: false,
        }
    }

    pub fn make_move_copy(&self, m: u16) -> Game {
        let mut game = Game {
            xs: self.xs,
            os: self.os,
            x_turn: self.x_turn,
            rematch: false,
            against_ai: self.against_ai,
            ai_plays_x: self.ai_plays_x,
        };
        game.make_move(m);
        game
    }

    fn start(&mut self) {
        self.xs = 0;
        self.os = 0;
        self.x_turn = true;
        self.rematch = false;
        self.ai_plays_x = !self.ai_plays_x;
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
    fn human_move(&mut self) -> u16 {
        loop {
            let move_input: String = request_input("Enter a field to play:");
            let read_move = ANNOTATIONS.into_iter().find(|e| e.0 == move_input);
            if let Some((_, x, y, code)) = read_move {
                if (self.valid_moves() & code) == 0 {
                    announce("Move was invalid!");
                    continue;
                }
                self.draw_move(x, y);
                self.make_move(code);
                return code;
            }
            announce(&format!(
                "Move {} could not be parsed. Enter one of the field labels.",
                move_input
            ));
        }
    }
    pub fn play_game(&mut self) {
        self.start();
        let mut game_tree: &GameTree = &GameTree::from_game(self.clone());
        loop {
            let m = self.human_move();
            let mut game_over = self.game_over();
            if game_over.0 {
                self.announce_game_over(game_over.1, game_over.2);
                return;
            }
            game_tree = &game_tree.children.iter().find(|c| c.1 == m).unwrap().0;
            if !self.against_ai {
                continue;
            }
            let ai_m = game_tree.best_move();
            let (label, x, y, _) = ANNOTATIONS.into_iter().find(|c| c.3 == ai_m.1).unwrap();
            self.draw_move(x, y);
            self.make_move(ai_m.1);
            game_tree = &ai_m.0;
            announce(&format!("AI has made move: {}", label));
            game_over = self.game_over();
            if game_over.0 {
                self.announce_game_over(game_over.1, game_over.2);
                return;
            }
        }
    }
}

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

    pub fn best_move(&self) -> &(GameTree, u16) {
        let for_x = self.root.x_turn;
        if for_x {
            if let Some(wins) = self.children.iter().find(|c| c.0.x_inevitable) {
                wins
            } else {
                self.children.iter().find(|c| !c.0.o_inevitable).unwrap()
            }
        } else if let Some(wins) = self.children.iter().find(|c| c.0.o_inevitable) {
            announce("Found Win");
            wins
        } else {
            announce("Random move");
            self.children.iter().find(|c| !c.0.x_inevitable).unwrap()
        }
    }
}
