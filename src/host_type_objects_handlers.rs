use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::cli_args_processing::StartOrder;
use crate::playboard::{display_board, GameState, Playboard};
use crate::settings::commands::*;
use crate::settings::playboard_options::PLAYBOARD_ROW_COL_SIZE;
use colored::Colorize;

fn read_stream_data(arc_stream: &Arc<Mutex<TcpStream>>) -> (String, usize) {
    const READ_STREAM_DATA_TIMEOUT: Duration = Duration::from_millis(50);

    let mut data = String::new();

    let guard_stream = arc_stream.lock().unwrap();

    guard_stream
        .set_read_timeout(Some(READ_STREAM_DATA_TIMEOUT))
        .expect("Could not set a read timeout");

    let mut reader = BufReader::new(guard_stream.try_clone().unwrap());

    let size = reader.read_line(&mut data).unwrap_or(0);

    (data, size)
}

fn decode_place(data: String) -> Option<(usize, usize)> {
    let data_tail = &data[PLACE_ON_PLAYBOARD_COMMAND.len()..];

    let pattern = regex::Regex::new(r"\s*\(\s*(?P<row>\d*)\s*,\s*(?P<col>\d*)\s*\)").unwrap();

    let res = match pattern.captures(data_tail) {
        Some(val) => val,
        None => return None,
    };

    Some((
        // index starts from zero
        res["row"].to_string().parse::<usize>().unwrap() - 1,
        res["col"].to_string().parse::<usize>().unwrap() - 1,
    ))
}

pub fn place_on_board(
    data: String,
    arc_playboard: Arc<Mutex<Playboard>>,
    arc_run_game: &Arc<AtomicBool>,
    start_order: StartOrder,
    who: String,
) -> bool {
    let place_fields = decode_place(data);

    match place_fields {
        Some((row, col)) => {
            let mut guard_playboard = arc_playboard.lock().unwrap();
            let game_state: GameState = guard_playboard.place_on_grid(row, col, start_order);

            match game_state {
                GameState::InvalidPlace => {
                    let msg = format!(
                        "Invalid row: {} and col: {} specified for {}. Try again.",
                        row, col, PLACE_ON_PLAYBOARD_COMMAND
                    );
                    println!("{}", msg.red().bold());
                    return false;
                }
                GameState::Placed => {
                    let msg = format!("{} placed on ({}, {}).", who, row, col);
                    println!("{}", msg.green().bold());
                }
                GameState::GameOver => {
                    arc_run_game.store(false, Ordering::Relaxed);

                    let msg = format!(
                        "\nGame over. {} won!{}.\nYou can play again by {}.",
                        who,
                        if who == "You" {
                            ""
                        } else {
                            " ( ＾◡ ＾)っ 💗"
                        },
                        PLAY_AGAIN_COMMAND
                    );
                    println!("{}", msg.magenta().bold());
                }
                GameState::Draw => {
                    arc_run_game.store(false, Ordering::Relaxed);

                    let msg = format!("\nDraw.\nYou can play again by {}.", PLAY_AGAIN_COMMAND);
                    println!("{}", msg.magenta().bold());
                }
            }

            display_board(guard_playboard, PLAYBOARD_ROW_COL_SIZE);
        }
        None => {
            let msg = format!(
                "Invalid option for {}. You can use {} if in doubts.",
                PLACE_ON_PLAYBOARD_COMMAND, HELP_COMMAND
            );
            println!("{}", msg.red().bold());
            return false;
        }
    }

    true
}

fn get_oponent_start_order(start_order: StartOrder) -> StartOrder {
    match start_order {
        StartOrder::First => StartOrder::Second,
        _ => StartOrder::First,
    }
}

pub fn restart_game(
    arc_playboard: Arc<Mutex<Playboard>>,
    arc_run_game: &Arc<AtomicBool>,
    arc_new_game_req: &Arc<AtomicBool>,
    arc_new_game_desirable: &Arc<AtomicBool>,
) {
    println!("{}", "\nRestarting game for both players...".green().bold());

    let mut guard_playboard = arc_playboard.lock().unwrap();
    guard_playboard.clear_board();

    arc_run_game.store(true, Ordering::Relaxed);
    arc_new_game_req.store(false, Ordering::Relaxed);
    arc_new_game_desirable.store(false, Ordering::Relaxed);
}

fn process_received_data_meaning(
    data: String,
    arc_playboard: Arc<Mutex<Playboard>>,
    arc_run_game: &Arc<AtomicBool>,
    start_order: StartOrder,
    arc_my_turn: &Arc<AtomicBool>,
    arc_new_game_req: &Arc<AtomicBool>,
    arc_new_game_desirable: &Arc<AtomicBool>,
) {
    let data = data.trim().to_string();

    if data.starts_with(PRIVATE_MESSAGE_COMMAND) {
        println!("{}", &data[4..]);
    } else if data.starts_with(PLACE_ON_PLAYBOARD_COMMAND) {
        if place_on_board(
            data,
            arc_playboard,
            arc_run_game,
            get_oponent_start_order(start_order),
            "Your oponent".to_string(),
        ) {
            arc_my_turn.store(true, Ordering::Relaxed);
        }
    } else if data == PLAY_AGAIN_COMMAND {
        println!(
            "{}",
            format!(
                "Your oponent wants to restart the game, you can agree with {}",
                AGREE_COMMAND
            )
            .green()
            .bold()
        );
        arc_new_game_req.store(true, Ordering::Relaxed);
    } else if data == AGREE_COMMAND && arc_new_game_desirable.load(Ordering::Relaxed) {
        restart_game(
            arc_playboard,
            arc_run_game,
            arc_new_game_req,
            arc_new_game_desirable,
        );
    }
}

pub fn handle_client(
    arc_stream: Arc<Mutex<TcpStream>>,
    arc_run_app: Arc<AtomicBool>,
    arc_run_game: Arc<AtomicBool>,
    arc_playboard: Arc<Mutex<Playboard>>,
    start_order: StartOrder,
    arc_my_turn: Arc<AtomicBool>,
    arc_new_game_req: Arc<AtomicBool>,
    arc_new_game_desirable: Arc<AtomicBool>,
) {
    while arc_run_app.load(Ordering::Relaxed) {
        let (data, size) = read_stream_data(&arc_stream);

        if size != 0 {
            let arc_playboard = arc_playboard.clone();
            process_received_data_meaning(
                data,
                arc_playboard,
                &arc_run_game,
                start_order,
                &arc_my_turn,
                &arc_new_game_req,
                &arc_new_game_desirable,
            );
        }
    }
}

pub fn handle_server(
    arc_stream: Arc<Mutex<TcpStream>>,
    arc_run_app: Arc<AtomicBool>,
    arc_run_game: Arc<AtomicBool>,
    arc_playboard: Arc<Mutex<Playboard>>,
    start_order: StartOrder,
    arc_my_turn: Arc<AtomicBool>,
    arc_new_game_req: Arc<AtomicBool>,
    arc_new_game_desirable: Arc<AtomicBool>,
) {
    while arc_run_app.load(Ordering::Relaxed) {
        let (data, size) = read_stream_data(&arc_stream);

        if size != 0 {
            let arc_playboard = arc_playboard.clone();
            process_received_data_meaning(
                data,
                arc_playboard,
                &arc_run_game,
                start_order,
                &arc_my_turn,
                &arc_new_game_req,
                &arc_new_game_desirable,
            );
        }
    }
}
