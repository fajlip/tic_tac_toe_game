use clap::App;
use std::{net::IpAddr, str::FromStr};

#[derive(Debug, Eq, PartialEq)]
pub enum HostType {
    Server,
    Client,
}

// Strum macros not used due to case insensitiveness.
impl FromStr for HostType {
    type Err = ();

    fn from_str(host_type: &str) -> Result<HostType, Self::Err> {
        // Do not respect letter case.
        match host_type.to_lowercase().as_str() {
            "server" => Ok(HostType::Server),
            "client" => Ok(HostType::Client),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StartOrder {
    First,
    Second,
}

// Strum macros not used due to case insensitiveness.
impl FromStr for StartOrder {
    type Err = ();

    fn from_str(start_order: &str) -> Result<StartOrder, Self::Err> {
        match start_order.to_lowercase().as_str() {
            "first" => Ok(StartOrder::First),
            "second" => Ok(StartOrder::Second),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Arguments {
    pub host_type: HostType,
    pub port: Option<u16>,
    pub ip_addr: Option<IpAddr>,
    pub start_order: StartOrder,
}

pub fn process_cli_arguments() -> Arguments {
    let yaml = load_yaml!("settings/cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    fn print_error(element: &str) -> String {
        format!("{} is not specified correctly. Use -h for help.", element)
    }

    let host_type: HostType = match HostType::from_str(matches.value_of("hostType").unwrap()) {
        Ok(host_type) => host_type,
        Err(_) => panic!("{}", print_error("Host type")),
    };

    let port: Option<u16> = match matches.value_of("port").unwrap_or("_").parse::<u16>() {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    if host_type != HostType::Server && port == None {
        panic!("{}", print_error("Port"))
    }

    let ip_addr: Option<IpAddr> = match matches.value_of("ipAddr").unwrap_or("_").parse::<IpAddr>()
    {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    if host_type == HostType::Client && ip_addr == None {
        panic!("{}", print_error("Ip address"))
    } else if host_type == HostType::Server && ip_addr != None {
        println!("Ip address specified for server will be ignored. Invalid option.")
    }

    let start_order: StartOrder =
        match StartOrder::from_str(matches.value_of("startOrder").unwrap()) {
            Ok(start_order) => start_order,
            Err(_) => panic!("{}", print_error("Start order")),
        };

    Arguments {
        host_type,
        port,
        ip_addr,
        start_order,
    }
}
