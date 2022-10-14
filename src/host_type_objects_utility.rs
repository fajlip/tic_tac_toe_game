use colored::*;
use core::panic;
use portpicker::is_free_tcp;
use std::net::IpAddr;

use crate::settings::commands::*;

pub fn get_first_free_port() -> u16 {
    const MIN_DYNAMIC_PRIVATE_PORT: u16 = 49152;
    const MAX_DYNAMIC_PRIVATE_PORT: u16 = 65535;

    for port in MIN_DYNAMIC_PRIVATE_PORT..MAX_DYNAMIC_PRIVATE_PORT {
        if is_free_tcp(port) {
            return port;
        }
    }

    panic!("Port not specified and none other port is free.")
}

pub fn print_server_game_setup(local_ip: IpAddr, port: u16) {
    // todo: refactor
    println!(
        "{} is running on IP address {} and {}.\n{}\n",
        "Tic tac toe game".magenta().bold(),
        local_ip.to_string().magenta().bold(),
        port.to_string().magenta().bold(),
        "Connect second player as a client.".yellow().bold()
    );
}

pub fn print_game_welcome_message() {
    println!(
        "Welcome to {}. Write {} if unsure what to do and {}.\n",
        "tic tac toe game".magenta().bold(),
        HELP_COMMAND.magenta().bold(),
        "enjoy the game".magenta().bold(),
    );
}

pub fn print_game_help() {
    println!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        "Valid commands to use:\n".magenta().bold(),
        format!(
            "{}            Writes message for other player.",
            PRIVATE_MESSAGE_COMMAND
        )
        .green()
        .bold(),
        format!(
            "{}(2, 1)   Places player symbol to playboard. In this example on row 2 and col 1.",
            PLACE_ON_PLAYBOARD_COMMAND
        )
        .green()
        .bold(),
        format!("{}          Clears window chat history.", CLEAR_COMMAND)
            .green()
            .bold(),
        format!("{}           Quits application.", QUIT_COMMAND)
            .green()
            .bold(),
        format!(
            "{}      Sends request for game restart. Oponent must agree with {}",
            PLAY_AGAIN_COMMAND, AGREE_COMMAND
        )
        .green()
        .bold(),
    );
}
