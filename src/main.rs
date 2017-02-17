extern crate core;
extern crate ws;
#[macro_use]
extern crate json;
extern crate time;
extern crate rand;

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use ws::util::Token;
use json::JsonValue;

mod game;
mod strategy;
mod state;

use game::Game;
use strategy::MonteCarlo;

const BOT_NAME: &'static str = "[Bot] Dropbot3";
const BOT_PASSWORD: &'static str = "qwrqwrjasf21kj";
const PRIVATE_TEST_ROOM: &'static str = "private124124214";
const PING_TOKEN: Token = Token(0);
const ACTION_TOKEN: Token = Token(1);

struct Client {
    out: Sender,
    in_game: bool,
    has_action: bool,
    game: Game,
    strategy: MonteCarlo,
    replay_id: String,
}

impl Client {
    fn emit(&mut self, data: JsonValue) {
        self.out.send(format!("42{}", data)).unwrap();
    }

    #[allow(dead_code)]
    fn join_test_room(&mut self) {
        self.emit(array!["join_private", PRIVATE_TEST_ROOM, BOT_PASSWORD]);
        self.emit(array!["set_force_start", PRIVATE_TEST_ROOM, true]);
        println!("Waiting for custom game: http://bot.generals.io/games/{}",
                 PRIVATE_TEST_ROOM);
    }

    #[allow(dead_code)]
    fn join_ffa(&mut self) {
        self.emit(array!["play", BOT_PASSWORD]);
        println!("Waiting for FFA game.");
    }

    #[allow(dead_code)]
    fn join_1v1(&mut self) {
        self.emit(array!["join_1v1", BOT_PASSWORD]);
        println!("Waiting for 1v1 game.");
    }
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(1_000, PING_TOKEN).unwrap();
        //self.emit(array!["set_username", BOT_PASSWORD, BOT_NAME]);
        self.join_test_room();
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg = msg.as_text().unwrap();
        let operation = msg.chars().nth(0).unwrap();
        if operation == '4' && msg != "40" {
            assert_eq!(msg.chars().nth(1).unwrap(), '2');
            let msg = json::parse(&msg[2..]).unwrap();

            match msg[0].as_str().unwrap() {
                "game_start" => {
                    self.in_game = true;
                    self.has_action = false;
                    self.replay_id = msg[1]["replay_id"].as_str().unwrap().to_string();
                    println!("Game starting. Replay will be at http://bot.generals.io/replays/{}",
                             self.replay_id);
                    self.game.handle_game_start(&msg[1]);
                }
                "game_update" => {
                    if self.in_game {
                        self.game.handle_game_update(&msg[1]);
                        if !self.has_action {
                            self.has_action = true;
                            self.out.timeout(0, ACTION_TOKEN).unwrap();
                        }
                    }
                }
                "game_lost" => {
                    println!("Game lost.");
                    self.in_game = false;
                    self.game = Game::new();
                    self.emit(array!["leave_game"]);
                }
                "game_won" => {
                    println!("Game won.");
                    self.in_game = false;
                    self.game = Game::new();
                    self.emit(array!["leave_game"]);
                }
                "queue_update" | "chat_message" | "pre_game_start" | "game_over" | "stars" |
                "rank" => (),
                "error_set_username" => {
                    if !msg[1].as_str().unwrap().is_empty() {
                        panic!("Unknown message {:?}", msg);
                    }
                },
                _ => panic!("Unknown message {:?}", msg),
            }
        } else if operation != '0' && operation != '3' && operation != '4' {
            panic!("Got unknown operation: {:?}", operation);
        }
        Ok(())
    }

    fn on_timeout(&mut self, token: Token) -> Result<()> {
        if token == PING_TOKEN {
            self.out.send("2").unwrap();
            self.out.timeout(10_000, PING_TOKEN).unwrap();
        } else if token == ACTION_TOKEN {
            if self.in_game {
                self.out.timeout(250, ACTION_TOKEN).unwrap();
                if let Some((src, dst, half)) = self.strategy.next_move(&self.game) {
                    let src = src.y * self.game.width + src.x;
                    let dst = dst.y * self.game.width + dst.x;
                    println!("Attacking: {}, {}", src, dst);
                    self.emit(array!["attack", src, dst, half]);
                }
            } else {
                self.join_ffa();
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        panic!("Disconnected {:?} {:?}", code, reason);
    }
}

fn main() {
    connect("ws://botws.generals.io/socket.io/?EIO=3&transport=websocket",
            |out| {
        Client {
            out: out,
            game: Game::new(),
            strategy: MonteCarlo::new(),
            in_game: false,
            has_action: false,
            replay_id: "".to_string(),
        }
    })
        .unwrap();
}
