use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use std::io;
use std::process::exit;

use crate::tcp_conn::TcpConn;
use crate::packet::Message::{self, *};
use crate::constants::*;



struct Client {
    id: u64,
    conn: TcpConn
}

/// A list of TcpConns which represents the active connections
type Clients = Arc<Mutex<Vec<Client>>>;

/// A map of id -> name to allow the server to lookup client names
type ClientNames = Arc<Mutex<HashMap<u64, String>>>;


/// Listens for new clients and distributes incoming messages
pub fn server() {
    
    // TcpListener will create a stream for each client
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let client_names: ClientNames = Arc::new(Mutex::new(HashMap::new()));
    
    let listener = TcpListener::bind(BIND_SOCKET).unwrap_or_else(|_| panic!(
        "[error] Unable to bind to port {PORT}",
    ));
    

    // listen for incoming connections in another thread
    let clients_clone = Arc::clone(&clients);
    let client_names_clone = Arc::clone(&client_names);
    thread::Builder::new()
        .name(String::from("server listener thread"))
        .spawn(move || {
            server_accept_connections(listener, clients_clone, client_names_clone)
        })
        .unwrap();

    // a queue to store messages while the `clients` mutex is locked and borrowed
    let mut queue = Vec::<(u64, Message)>::new();

    // process messages and distribute them
    loop {
        // the sockets are non-blocking, so sleep to avoid excessive cpu usage on the server, which
        // does not require high responsiveness
        thread::sleep(Duration::from_millis(SERVER_POLL_DELAY_MS));
        
        for client in clients.lock().unwrap().iter_mut() {

            match client.conn.receive() {
                Ok(msg) => queue.push((client.id, msg)),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {},

                // someone left without saying goodbye
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {
                    queue.push((client.id, ClientGoodbye));
                },
                // someone left without saying goodbye
                Err(e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                    queue.push((client.id, ClientGoodbye));
                },
                // client sent the wrong type
                Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                    println!("[server] Client sent an invalid type.");
                    client.conn.empty_buffer();
                },
                // insufficient data from the connection
                Err(e) if e.kind() == io::ErrorKind::Other => {},
                Err(e) => {
                    println!("[server] Error reading client's connection: {:?}", e);
                },
            }

        }

        // read back the messages received and determine what to do with them
        for (id, msg) in queue.iter() {
            
            server_handle_message(msg, id, &clients, &client_names);
        }

        queue.clear();
    }
}


/// Respond to the given message
fn server_handle_message(msg: &Message, sender: &u64, clients: &Clients, client_names: &ClientNames) {
    match msg {
        ServerShutdown => {

            println!("[server] Server shutting down");
            server_distribute_message(clients, msg, &[]);

            thread::sleep(Duration::from_secs(1));
            exit(0);

        },
        ClientText(text) => {

            if let Some(name) = client_names.lock().unwrap().get(sender) {

                server_distribute_message(
                    clients,
                    &ServerText(name.clone(), text.clone()),
                    &[*sender]
                );

            } else {
                println!("[server] Unable to get client name by id.");
            }

        }
        ClientGoodbye => {
            // perform removal of client
            clients.lock().unwrap().retain(|client| &client.id != sender);

            // let everyone else know they left
            if let Some(name) = client_names.lock().unwrap().remove(sender) {                     

                server_distribute_message(
                    clients,
                    &ServerText("[server]".to_string(), format!("{name} has left the room")),
                    &[]
                );

            } else {
                println!("[server] A client was removed, but I was unable to tell the other clients");
            }

        },
        ClientRename(new_name) => {
            client_names.lock().unwrap().insert(*sender, new_name.clone());
        },
        ClientKick(who) => {

            if who == &0 {
                println!("[server] Host cannot kick themselves");
                return;
            }
            
            match clients.lock().unwrap().iter_mut().find(|client| &client.id == who) {
                Some(kickee) => {
                    if kickee.conn.send(&ServerNotifyKick).is_err() {
                        println!("[server] Unable to notify client that they were kicked.");
                    }

                    // perform removal of client
                    clients.lock().unwrap().retain(|client| &client.id != who);


                    // let everyone else know they left
                    if let Some(name) = client_names.lock().unwrap().remove(sender) {                     

                        server_distribute_message(
                            clients,
                            &ServerText("[server]".to_string(), format!("{name} was kicked by the host")),
                            &[]
                        );

                    } else {
                        println!("[server] A client was kicked, but I was unable to tell the other clients");
                    }
                },
                None => println!("[server] Client with id {who} does not exist"),
            }
        },
        ClientRequestIDs => {
            let mut unlocked = clients.lock().unwrap();
            
            let sender_client = unlocked.iter_mut()
                .find(|client| &client.id == sender);
            
            match sender_client {
                Some(client) => {
                    let list: Vec<_> = client_names.lock().unwrap().iter()
                        .map(|(x, y)| (y.clone(), *x))
                        .collect();
                    if client.conn.send(&ServerResponseIDs(list)).is_err() {
                        println!("[server] Unable to reply to client that requested IDs");
                    }
                },
                None => println!("[server] Client with id {sender} could not be found")
            }
        },
        other => println!("[server] Got unusual message from client: {other:?}."),
    }  
}


/// Continuously listen for incoming connections
fn server_accept_connections(listener: TcpListener, clients: Clients, client_names: ClientNames) {

    println!("[server] Open for connections");

    let mut next_id = 0u64;

    // Receive incoming client connections forever. This will not exit.
    for client in listener.incoming().flatten() {
        
        // block for first message from new client before moving on so we can get their name
        let Ok(mut conn) = TcpConn::new(client) else {continue;};

        let client_name = match conn.receive_timeout::<Message>(Duration::from_secs(5)) {
            Ok(ClientHello(name)) => {
                let msg = ServerText("[server]".to_string(), format!("{name} has joined the room!"));
                
                // let the new client and everyone else know someone joined
                server_distribute_message(
                    &clients,
                    &msg,
                    &[]
                );
                
                match conn.send(&msg) {
                    Ok(_) => name,
                    Err(_) => {
                        // let everyone know this client could not be connected with
                        server_distribute_message(
                            &clients,
                            &ServerText("[server]".to_string(), format!("{name} left the room")),
                            &[]
                        );
                        // skip adding the client
                        continue;
                    }
                }
            },
            Ok(other) => {
                println!("[server] Client sent invalid response. Expected `ClientHello(<some name>)`, got `{:?}`", other);
                // we skip this bad client
                continue;
            },
            Err(e) => {
                println!("[server] Error reading client's connection: {}", e);
                // skip client
                continue;
            },
        };

        conn.set_nonblocking(true).unwrap();

        let new_client = Client {
            id: next_id,
            conn
        };
        
        clients.lock().unwrap().push(new_client);
        client_names.lock().unwrap().insert(next_id, client_name);
        next_id += 1;
    }
    println!("[server] Stopped listening for connections");
}


/// Send `msg` to every client. Improvement idea: accept iterator instead of `&Clients` to allow
/// easy filtering of which clients receive messages
fn server_distribute_message(clients: &Clients, msg: &Message, exclude: &[u64]) {
    for client in clients.lock().unwrap().iter_mut() {
        if !exclude.contains(&client.id) && client.conn.send(msg).is_err() {
            println!("[server] A client did not receive a message!");
        }
    }
}
