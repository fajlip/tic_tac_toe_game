use crate::cli_args_processing::Arguments;
use crate::host_type_objects::{HostTypeObject, HostTypeObjectFactory};
use crate::host_type_objects_utility::{print_game_help, print_game_welcome_message};
use crate::settings::commands::{
    AGREE_COMMAND, CLEAR_COMMAND, HELP_COMMAND, PLAY_AGAIN_COMMAND, QUIT_COMMAND,
};

pub fn handle_host_type_communication(arguments: Arguments) {
    let mut host_type_object: Box<dyn HostTypeObject> =
        HostTypeObjectFactory::create_host_type_object(arguments);

    print_game_welcome_message();

    let mut line = String::new();

    while let Ok(bytes_read) = std::io::stdin().read_line(&mut line) {
        if bytes_read == 0 {
            break;
        }

        let line = line[0..bytes_read].trim().to_string();

        if line == HELP_COMMAND {
            print_game_help();
        } else if line == CLEAR_COMMAND {
            if clearscreen::clear().is_err() {
                println!("Failed to clear window.");
            }

            print_game_help();
        } else if line == PLAY_AGAIN_COMMAND {
            host_type_object.new_game_request(line + "\n");
        } else if line == AGREE_COMMAND {
            host_type_object.new_game_agreement(line + "\n");
        } else if line == QUIT_COMMAND {
            break;
        } else {
            host_type_object.send_message(line + "\n");
        }
    }

    host_type_object.stop();
}
