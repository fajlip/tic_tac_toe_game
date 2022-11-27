use clap::App;
use std::{net::IpAddr, str::FromStr};

#[derive(Debug, Eq, PartialEq)]
pub enum HostType {
    Server,
    Client,
}
pub struct InvalidArgument;

fn handle_error(element: &str) -> InvalidArgument {
    println!("{} is not specified correctly. Use -h for help.", element);
    InvalidArgument
}

// Strum macros not used due to case insensitiveness.
impl FromStr for HostType {
    type Err = InvalidArgument;

    fn from_str(host_type: &str) -> Result<HostType, InvalidArgument> {
        // Do not respect letter case.
        match host_type.to_lowercase().as_str() {
            "server" => Ok(HostType::Server),
            "client" => Ok(HostType::Client),
            _ => Err(handle_error("Host type")),
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
    type Err = InvalidArgument;

    fn from_str(start_order: &str) -> Result<StartOrder, InvalidArgument> {
        match start_order.to_lowercase().as_str() {
            "first" => Ok(StartOrder::First),
            "second" => Ok(StartOrder::Second),
            _ => Err(handle_error("Start order")),
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

pub fn process_cli_arguments() -> Result<Arguments, InvalidArgument> {
    let yaml = load_yaml!("settings/cli.yaml");
    let matches = App::from_yaml(yaml).get_matches();

    let host_type: HostType = HostType::from_str(matches.value_of("hostType").unwrap())?;

    let port: Option<u16> = matches.value_of("port").unwrap_or("_").parse::<u16>().ok();

    if host_type != HostType::Server && port.is_none() {
        return Err(handle_error("Host type and port combination"));
    }

    let ip_addr: Option<IpAddr> = matches
        .value_of("ipAddr")
        .unwrap_or("_")
        .parse::<IpAddr>()
        .ok();

    if host_type == HostType::Client && ip_addr == None {
        return Err(handle_error("Host type and IP address combination"));
    } else if host_type == HostType::Server && ip_addr != None {
        println!("Ip address specified for server will be ignored. Invalid option, but the show goes on!")
    }

    let start_order: StartOrder = StartOrder::from_str(matches.value_of("startOrder").unwrap())?;

    Ok(Arguments {
        host_type,
        port,
        ip_addr,
        start_order,
    })
}
