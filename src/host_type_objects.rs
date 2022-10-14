use local_ip_address::local_ip;
use std::io::Write;
use std::net::{IpAddr, Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::cli_args_processing::{Arguments, HostType, StartOrder};
use crate::host_type_objects_handlers::{
    handle_client, handle_server, place_on_board, restart_game,
};
use crate::host_type_objects_utility::{get_first_free_port, print_server_game_setup};
use crate::playboard::Playboard;
use crate::settings::commands::{AGREE_COMMAND, PLACE_ON_PLAYBOARD_COMMAND, PLAY_AGAIN_COMMAND};
use colored::Colorize;

fn run_func_in_thread(
    func_host_type_object_handler: fn(
        Arc<Mutex<TcpStream>>,
        Arc<AtomicBool>,
        Arc<AtomicBool>,
        Arc<Mutex<Playboard>>,
        StartOrder,
        Arc<AtomicBool>,
        Arc<AtomicBool>,
        Arc<AtomicBool>,
    ),
    stream: TcpStream,
    start_order: StartOrder,
) -> (
    JoinHandle<()>,
    Arc<Mutex<TcpStream>>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
    Arc<Mutex<Playboard>>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
) {
    let arc_stream = Arc::new(Mutex::new(stream));
    let arc_run_app = Arc::new(AtomicBool::new(true));
    let arc_run_game = Arc::new(AtomicBool::new(true));
    let arc_playboard = Arc::new(Mutex::new(Playboard::new()));
    let arc_my_turn = Arc::new(AtomicBool::new(start_order == StartOrder::First));
    let arc_new_game_req = Arc::new(AtomicBool::new(false));
    let arc_new_game_desirable = Arc::new(AtomicBool::new(false));

    let handler_thread;
    {
        let arc_stream = arc_stream.clone();
        let arc_run_app = arc_run_app.clone();
        let arc_run_game = arc_run_game.clone();
        let arc_playboard = arc_playboard.clone();
        let arc_my_turn = arc_my_turn.clone();
        let arc_new_game_req = arc_new_game_req.clone();
        let arc_new_game_desirable = arc_new_game_desirable.clone();

        handler_thread = thread::spawn(move || {
            func_host_type_object_handler(
                arc_stream,
                arc_run_app,
                arc_run_game,
                arc_playboard,
                start_order,
                arc_my_turn,
                arc_new_game_req,
                arc_new_game_desirable,
            );
        });
    }

    (
        handler_thread,
        arc_stream,
        arc_run_app,
        arc_run_game,
        arc_playboard,
        arc_my_turn,
        arc_new_game_req,
        arc_new_game_desirable,
    )
}

pub trait HostTypeObject {
    fn send_message(&self, msg: String);
    fn new_game_request(&mut self, msg: String);
    fn new_game_agreement(&mut self, msg: String);
    fn stop(&mut self);
}

pub struct Server {
    handler_thread: Option<JoinHandle<()>>,
    arc_stream: Arc<Mutex<TcpStream>>,
    arc_run_app: Arc<AtomicBool>,
    arc_run_game: Arc<AtomicBool>,
    arc_playboard: Arc<Mutex<Playboard>>,
    start_order: StartOrder,
    arc_my_turn: Arc<AtomicBool>,
    arc_new_game_req: Arc<AtomicBool>,
    arc_new_game_desirable: Arc<AtomicBool>,
}

impl Server {
    pub fn new(port: Option<u16>, start_order: StartOrder) -> Self {
        let port: u16 = match port {
            Some(port) => port,
            None => get_first_free_port(),
        };

        let local_ip = local_ip().unwrap();

        print_server_game_setup(local_ip, port);

        let listener = TcpListener::bind(format!("{}:{}", local_ip, port))
            .expect("Cannot bind socket for tcp listener.");

        let (stream, client_addr) = listener
            .accept()
            .expect("Failed to accept client connection.");

        println!("Second player connected from {}.\n", client_addr);

        let (
            handler_thread,
            arc_stream,
            arc_run_app,
            arc_run_game,
            arc_playboard,
            arc_my_turn,
            arc_new_game_req,
            arc_new_game_desirable,
        ) = run_func_in_thread(handle_client, stream, start_order);

        drop(listener);

        Self {
            handler_thread: Some(handler_thread),
            arc_stream,
            arc_run_app,
            arc_run_game,
            arc_playboard,
            start_order,
            arc_my_turn,
            arc_new_game_req,
            arc_new_game_desirable,
        }
    }
}

impl HostTypeObject for Server {
    fn send_message(&self, msg: String) {
        if msg.starts_with(PLACE_ON_PLAYBOARD_COMMAND) {
            if !self.arc_run_game.load(Ordering::Relaxed) {
                let msg = format!("You can no longer play symbol; game finished.\nYou can start another one with {}.", PLAY_AGAIN_COMMAND);
                println!("{}", msg.red().bold());
                return;
            }

            if !self.arc_my_turn.load(Ordering::Relaxed) {
                println!(
                    "{}",
                    "It is not your turn, you cannot place symbol. Wait for your oponent."
                        .red()
                        .bold()
                );
                return;
            }

            let arc_playboard = self.arc_playboard.clone();
            place_on_board(
                msg.clone(),
                arc_playboard,
                &self.arc_run_game,
                self.start_order,
                "You".to_string(),
            );

            self.arc_my_turn.store(false, Ordering::Relaxed);
        }

        let mut guard_stream = self.arc_stream.lock().unwrap();

        guard_stream.write(msg.as_bytes()).unwrap();
        guard_stream.flush().expect("Flush failed.");
    }

    fn new_game_request(&mut self, msg: String) {
        println!(
            "{}",
            format!(
                "Sending new game request, your oponent must agree by {}.",
                AGREE_COMMAND
            )
            .green()
            .bold()
        );
        self.arc_new_game_desirable.store(true, Ordering::Relaxed);

        self.send_message(msg);
    }

    fn new_game_agreement(&mut self, msg: String) {
        if self.arc_new_game_req.load(Ordering::Relaxed) {
            let arc_playboard = self.arc_playboard.clone();

            restart_game(
                arc_playboard,
                &self.arc_run_game,
                &self.arc_new_game_req,
                &self.arc_new_game_desirable,
            );

            self.send_message(msg);
        }
    }

    fn stop(&mut self) {
        println!("Stopping tic tac toe game...");

        let guard_stream = self.arc_stream.lock().unwrap();

        // stop reading thread
        self.arc_run_app.store(false, Ordering::Relaxed);

        guard_stream
            .shutdown(Shutdown::Both)
            .expect("Stream shutdown failed.");

        if let Some(handler_thread) = self.handler_thread.take() {
            handler_thread.join().expect("Failed to join thread.");
        }
    }
}

pub struct Client {
    handler_thread: Option<JoinHandle<()>>,
    arc_stream: Arc<Mutex<TcpStream>>,
    arc_run_app: Arc<AtomicBool>,
    arc_run_game: Arc<AtomicBool>,
    arc_playboard: Arc<Mutex<Playboard>>,
    start_order: StartOrder,
    arc_my_turn: Arc<AtomicBool>,
    arc_new_game_req: Arc<AtomicBool>,
    arc_new_game_desirable: Arc<AtomicBool>,
}

impl Client {
    pub fn new(port: Option<u16>, ip_addr: Option<IpAddr>, start_order: StartOrder) -> Self {
        assert!(port.is_some());
        assert!(ip_addr.is_some());

        let ip_addr = ip_addr.unwrap();
        let port = port.unwrap();

        let stream = TcpStream::connect(format!("{}:{}", ip_addr, port))
            .expect("Client failed to connect to the server.");

        println!(
            "Successfully connected to player one to {} on port {}.\n",
            ip_addr, port
        );

        let (
            handler_thread,
            arc_stream,
            arc_run_app,
            arc_run_game,
            arc_playboard,
            arc_my_turn,
            arc_new_game_req,
            arc_new_game_desirable,
        ) = run_func_in_thread(handle_server, stream, start_order);

        Self {
            handler_thread: Some(handler_thread),
            arc_stream,
            arc_run_app,
            arc_run_game,
            arc_playboard,
            start_order,
            arc_my_turn,
            arc_new_game_req,
            arc_new_game_desirable,
        }
    }
}

impl HostTypeObject for Client {
    fn send_message(&self, msg: String) {
        if msg.starts_with(PLACE_ON_PLAYBOARD_COMMAND) {
            if !self.arc_run_game.load(Ordering::Relaxed) {
                let msg = format!("You can no longer play symbol; game finished.\nYou can start another one with {}.", PLAY_AGAIN_COMMAND);
                println!("{}", msg.red().bold());
                return;
            }

            if !self.arc_my_turn.load(Ordering::Relaxed) {
                println!(
                    "{}",
                    "It is not your turn, you cannot place symbol. Wait for your oponent."
                        .red()
                        .bold()
                );
                return;
            }

            let arc_playboard = self.arc_playboard.clone();
            if place_on_board(
                msg.clone(),
                arc_playboard,
                &self.arc_run_game,
                self.start_order,
                "You".to_string(),
            ) {
                self.arc_my_turn.store(false, Ordering::Relaxed);
            }
        }

        let mut guard_stream = self.arc_stream.lock().unwrap();

        guard_stream.write(msg.as_bytes()).unwrap();
        guard_stream.flush().expect("Flush failed.");
    }

    fn new_game_request(&mut self, msg: String) {
        println!(
            "{}",
            format!(
                "Sending new game request, your oponent must agree by {}.",
                AGREE_COMMAND
            )
            .green()
            .bold()
        );
        self.arc_new_game_desirable.store(true, Ordering::Relaxed);

        self.send_message(msg);
    }

    fn new_game_agreement(&mut self, msg: String) {
        if self.arc_new_game_req.load(Ordering::Relaxed) {
            let arc_playboard = self.arc_playboard.clone();

            restart_game(
                arc_playboard,
                &self.arc_run_game,
                &self.arc_new_game_req,
                &self.arc_new_game_desirable,
            );

            self.send_message(msg);
        }
    }

    fn stop(&mut self) {
        println!("Stopping tic tac toe game...");

        let guard_stream = self.arc_stream.lock().unwrap();

        // stop reading thread
        self.arc_run_app.store(false, Ordering::Relaxed);

        guard_stream
            .shutdown(Shutdown::Both)
            .expect("Stream shutdown failed.");

        if let Some(handler_thread) = self.handler_thread.take() {
            handler_thread.join().expect("Failed to join thread.");
        }
    }
}

pub struct HostTypeObjectFactory;
impl HostTypeObjectFactory {
    pub fn create_host_type_object(arguments: Arguments) -> Box<dyn HostTypeObject> {
        match arguments.host_type {
            HostType::Server => Box::new(Server::new(arguments.port, arguments.start_order)),
            HostType::Client => Box::new(Client::new(
                arguments.port,
                arguments.ip_addr,
                arguments.start_order,
            )),
        }
    }
}
