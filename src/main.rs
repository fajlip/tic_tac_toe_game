#[macro_use]
extern crate clap;
extern crate matrix_display;

mod host_type_objects;
mod host_type_objects_handlers;
mod host_type_objects_utility;
mod playboard;
mod settings;

mod cli_args_processing;
use cli_args_processing::{process_cli_arguments, Arguments};

mod host_type_communication_handler;
use host_type_communication_handler::handle_host_type_communication;

fn main() {
    let arguments: Arguments = process_cli_arguments();

    handle_host_type_communication(arguments);
}
