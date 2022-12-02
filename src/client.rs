use std::{io, thread};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use crate::packet::Message::{self, *};
use crate::constants::*;
use crate::tcp_conn::TcpConn;
use crate::helpers::{input, input_msg};



/// Console interface for client
pub fn client(name: &str, is_host: bool) {

    // Ask the user for the host address. If the user is the host, use loopback.
    let socket = if is_host {
        LOOPBACK_SOCKET
    } else {
        println!("Enter the address of the server");

        let ip: IpAddr = loop {
            let ip_str = input();

            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                break ip;
            } else if ip_str == "localhost" {
                break LOOPBACK;
            }
        };

        SocketAddr::new(ip, PORT)
    };

    let mut conn = connect_to_server(socket).expect("[error] Problem connecting to server.");

    // send an initial message so the server can display who joined and keep track of name
    conn.send(ClientHello(name.to_string()))
        .expect("[error] Failed to join room. Could not send greeting");

    // begin the messaging loop
    loop {
        let raw_msg = input_msg();

        // Check if input is a command or a message
        if raw_msg.starts_with('!') {

            if raw_msg.to_lowercase().as_str() == "!exit" {
                if is_host && conn.send(ServerShutdown).is_err(){

                    println!("[error] Failed to gracefully shutdown the server.");

                } else if conn.send(ClientGoodbye).is_err() {

                    println!("[error] Failed to gracefully leave the room.")

                }
                
                // give time for message to send
                sleep(Duration::from_secs(1));
                exit(0);
            }
        } else {
            let msg = ClientText(raw_msg);

            if conn.send(msg).is_err() {
                println!("[error] Could not send message.");
            }
        }
    }
}

/// Send a connection request to the specified server address. Upon successful connection, this
/// function will spawn a thread for receiving server messages
fn connect_to_server(addr: SocketAddr) -> io::Result<TcpConn> {

    let connection = TcpStream::connect_timeout(&addr, Duration::from_secs(10))?;
    
    let conn_clone = connection
        .try_clone()
        .expect("[error] Unable to clone the TcpStream connection");

    thread::spawn(move || receive_messages(TcpConn::new(conn_clone)));

    Ok(TcpConn::new(connection))
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
            Ok(other) => println!("Some other message was received: {:?}", other),

            // we ignore errors referring to incomplete data
            Err(e) if e.kind() == io::ErrorKind::Other => {},

            // this seems to be an indicator that the server removed the socket
            Err(e) if e.kind() == io::ErrorKind::Uncategorized => {
                
                exit(0);
            },
            Err(e) => {
                println!("[error] Connection to server lost. Reason: {}", e.kind());
                exit(0);
            }
        }
    }
}
