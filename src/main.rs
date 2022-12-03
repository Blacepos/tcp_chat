#![feature(fn_traits, const_socketaddr, core_intrinsics, iterator_try_collect, io_error_uncategorized)]

use std::net::{TcpListener, TcpStream};
use std::process::exit;
use std::thread;

use constants::*;
use helpers::*;
use result_repeat::*;
use packet::Message::{self, *};
use tcp_conn::TcpConn;
use server::server;
use client::client;

mod constants;
mod client;
mod server;
mod helpers;
mod result_repeat;
mod packet;
mod tcp_conn;
mod commands;



fn main() {
    ctrlc::set_handler(|| exit(0)).expect("Unable to set Ctrl-C handler");

    println!("Welcome to TCP chat!");
    println!("Please enter your username");
    let name = input();

    println!("Are you going to host the room? (y/n)");

    let will_host = input.until_valid(validate_yn).is_yes(); // traits are cool

    // Spawn the server thread if user wishes to host
    if will_host {
        thread::Builder::new()
            .name(String::from("server main"))
            .spawn(server)
            .unwrap();
    }
    
    client(name.as_str(), will_host);
}


// This is unused, but here to demonstrate how `TcpConn` works
#[allow(unused)]
fn demonstrate() -> Result<(), Box<dyn std::error::Error>> {

    if std::env::args().nth(1) == Some(String::from("c")) {
        println!("Client");

        let stream = TcpStream::connect(LOOPBACK_SOCKET)?;
        let mut conn = TcpConn::new(stream)?;

        conn.send(&ClientText(String::from("Hello, server! I am sending this to you because it is a really long message and I just wanted to see if you like that I'm sending long messages. Also, I just wanted to tell you that I kind of like the way that you send me handshake messages and I was kind of um wondering if you would like to maybe possibly consider entering a long-term connection with me. Thanks bye.")))?;

        let msg1: Message = conn.receive()?;
        let msg2: Message = conn.receive()?;
        println!("{:?}\n{:?}", msg1, msg2);

    } else {
        println!("Server");

        let listen = TcpListener::bind(LOOPBACK_SOCKET)?;
        let (stream, _) = listen.accept()?;
        let mut conn = TcpConn::new(stream)?;

        let client_message: Message = conn.receive()?;

        println!("{:?}", client_message);

        conn.send(&ServerText(String::from("server"), String::from("No. Get owned lmao")))?;
        conn.send(&ServerShutdown)?;
    }

    Ok(())
}
