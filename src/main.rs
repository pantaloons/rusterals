extern crate core;
extern crate ws;
#[macro_use]
extern crate json;

use ws::{connect, Handler, Sender, Handshake, Result, Message, CloseCode};
use ws::util::Token;
use json::JsonValue;

mod state;

use state::State;

const BOT_NAME: &'static str = "Rusterals";
const BOT_PASSWORD: &'static str = "Rusterals09817qwr";
const PRIVATE_TEST_ROOM: &'static str = "private124124214";
const PING_TOKEN: Token = Token(0);
const ACTION_TOKEN: Token = Token(1);

struct Client {
    out: Sender,
    in_game: bool,
    state: State,
    replay_id: String,
}

impl Client {
    fn emit(&mut self, data: JsonValue) {
        self.out.send(format!("42{}", data)).unwrap();
    }

    fn join_test_room(&mut self) {
        self.emit(array!["join_private", PRIVATE_TEST_ROOM, BOT_PASSWORD]);
        self.emit(array!["set_force_start", PRIVATE_TEST_ROOM, true]);
        println!("Waiting for custom game: http://bot.generals.io/games/{}", PRIVATE_TEST_ROOM);
    }

    fn join_ffa(&mut self) {
        self.emit(array!["play", BOT_PASSWORD]);
        println!("Waiting for FFA game.");
    }
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.out.timeout(1_000, PING_TOKEN).unwrap();
        self.emit(array!["set_username", BOT_PASSWORD, BOT_NAME]);
        self.join_ffa();
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
                    self.replay_id = msg[1]["replay_id"].as_str().unwrap().to_string();
                    println!("FFA starting. Replay will be at http://bot.generals.io/replays/{}", self.replay_id);
                    self.state.handle_game_start(&msg[1]);
                    self.out.timeout(500, ACTION_TOKEN).unwrap();
                },
                "game_update" => self.state.handle_game_update(&msg[1]),
                "game_lost" => {
                    println!("Game lost.");
                    self.in_game = false;
                    self.emit(array!["leave_game"]);
                },
                "game_won" => {
                    println!("Game won.");
                    self.in_game = false;
                    self.emit(array!["leave_game"]);
                },
                "queue_update" | "chat_message" | "pre_game_start" | "game_over" => (),
                _ => panic!("Unknown message {:?}", msg),
            }
        }
        else if operation != '0' && operation != '3' && operation != '4' {
            panic!("Got unknown operation: {:?}", operation);
        }
        Ok(())
    }

    fn on_timeout(&mut self, token: Token) -> Result<()> {
        if token == PING_TOKEN {
            self.out.send("2").unwrap();
            self.out.timeout(10_000, PING_TOKEN).unwrap();
        }
        else if token == ACTION_TOKEN {
            if self.in_game {
                if let Some((src, dst, half)) = self.state.next_move() {
                    if half {
                        self.emit(array!["attack", src, dst, true]);
                    }
                    else {
                        self.emit(array!["attack", src, dst]);
                    }
                }

                self.out.timeout(500, ACTION_TOKEN).unwrap();
            }
            else {
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
    connect("ws://botws.generals.io/socket.io/?EIO=3&transport=websocket", |out| Client {
        out: out,
        state: State::new(),
        in_game: false,
        replay_id: "".to_string(),
    }).unwrap();
}
