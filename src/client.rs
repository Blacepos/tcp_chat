use std::{io, thread};
use std::net::{SocketAddr, TcpStream, IpAddr};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use crate::commands::{parse_command, Command::*, CLIENT_COMMANDS, HOST_COMMANDS};
use crate::packet::Message::{self, *};
use crate::constants::*;
use crate::tcp_conn::TcpConn;
use crate::helpers::{input, input_msg};

/// Ask the user to input a domain name or ip address
fn prompt_address() -> Vec<SocketAddr> {
    println!("Enter the address of the server");

    let (ips, port): (Vec<IpAddr>, u16) = loop {
        let unparsed_str = input();

        let temp: Vec<_> = unparsed_str.split(':').take(2).collect();

        let Some(&addr_str) = temp.first() else {continue};

        let port: u16 = if let Some(&ps) = temp.get(1) {
            if let Ok(p) = ps.parse() {
                p
            } else {
                continue
            }
        } else {
            PORT
        };
        
        if let Ok(results) = dns_lookup::lookup_host(addr_str) {
            break (results, port)
        }
    };

    ips.iter().map(|&x| SocketAddr::new(x, port)).collect()
}

/// Console interface for client
pub fn client(name: &str, is_host: bool) {

    // Ask the user for the host address. If the user is the host, use loopback.
    let socket = if is_host {vec![LOOPBACK_SOCKET]} else {prompt_address()};

    let mut conn = connect_to_server(socket)
        .expect("[error] Problem connecting to server.");

    // send an initial message so the server can display who joined and keep track of name
    conn.send(&ClientHello(name.to_string()))
        .expect("[error] Failed to join room. Could not send greeting");

    // begin the messaging loop
    loop {
        let raw_msg = input_msg();

        if raw_msg.starts_with('!') {
            match parse_command(&raw_msg, is_host) {
                Some(cmd) => {
                    match cmd {
                        Help => {
                            println!("Commands: {}", CLIENT_COMMANDS.join(", "));
                        },
                        HostHelp => {
                            let list = [&CLIENT_COMMANDS[..], &HOST_COMMANDS[..]]
                                .concat()
                                .join(", ");
                            println!("Commands: {}", list);
                        },
                        Exit => {
                            if conn.send(&ClientGoodbye).is_err() {
                                println!("[error] Failed to gracefully leave the room.")
                            }
                            // give time for message to send
                            sleep(Duration::from_secs(1));
                            exit(0);
                        },
                        HostExit => {
                            if conn.send(&ServerShutdown).is_err() {
                                println!("[error] Failed to gracefully shutdown the server.");
                            }
                            // give time for message to send
                            sleep(Duration::from_secs(1));
                            exit(0);
                        },
                        Rename(new_name) => {
                            conn.send(&ClientRename(new_name))
                                .expect("[error] Could not send message");
                        },
                        Kick(who) => {
                            conn.send(&ClientKick(who))
                                .expect("[error] Could not send message");
                        },
                        RequestIDs => {
                            conn.send(&ClientRequestIDs)
                                .expect("[error] Could not send message");
                        },
                    }
                },
                None => {
                    println!("Invalid command, try !help for available commands.");
                },
            }

        } else {
            let msg = ClientText(raw_msg);

            conn.send(&msg).expect("[error] Could not send message.");
        }
    }
}

/// Send a connection request to the specified server address. Upon successful connection, this
/// function will spawn a thread for receiving server messages
fn connect_to_server(addr: Vec<SocketAddr>) -> io::Result<TcpConn> {
    println!("Resolved addresses: {addr:?}");
    let stream = TcpStream::connect(&addr[..])?;
    
    let stream_clone = stream
        .try_clone()
        .expect("[error] Unable to clone the TcpStream connection");
    
    let conn = TcpConn::new(stream)?;
    let conn_clone = TcpConn::new(stream_clone)?;

    thread::Builder::new()
        .name(String::from("client receive messages"))
        .spawn(move || receive_messages(conn_clone))
        .unwrap();

    Ok(conn)
}

/// Receive messages and print them to the console window
fn receive_messages(mut conn: TcpConn) {
    loop {
        match conn.receive::<Message>() {
            Ok(ServerText(name, text)) => println!("{name}: {text}"),
            Ok(ServerShutdown) => {
                println!("The host has closed the room");
                exit(0);
            },
            Ok(ServerResponseIDs(ids)) => {
                let list: String = ids.iter()
                    .map(|(x, y)| format!("{x}: {y}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                println!("Client IDs: {list}");
            },
            Ok(ServerNotifyKick) => {
                println!("The host has kicked you");
                exit(0);
            }
            Ok(other) => println!("Some other message was received: {:?}", other),
            // we ignore errors referring to incomplete data
            Err(e) if e.kind() == io::ErrorKind::Other => {},
            // this seems to be an indicator that the server removed the socket
            Err(e) if e.kind() == io::ErrorKind::Uncategorized => exit(0),
            Err(e) => {
                println!("[error] Connection to server lost. Reason: {}", e.kind());
                exit(0);
            }
        }
    }
}
