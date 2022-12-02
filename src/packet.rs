use serde::{self, Serialize, Deserialize};



/// The universal message type to make reading and sending messages significantly nicer
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    /// A generic message to the server
    ClientText(String),         // text

    /// Client's first message to server
    ClientHello(String),        // name

    /// Client letting the server know that it is leaving the room
    ClientGoodbye,
    
    /// The server sending a message to client B by distributing a message from client A
    /// Use cases: distribution of client message or server update (e.g., someone leaving)
    ServerText(String, String), // sender name, text

    /// Server notifying all the clients that the room is closing
    ServerShutdown
}
