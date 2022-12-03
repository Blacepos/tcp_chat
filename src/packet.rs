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

    /// Client requesting to change their name
    ClientRename(String),

    /// Host Client requesting to kick someone by id
    ClientKick(u64),

    /// Host Client requesting the list of client ids
    ClientRequestIDs,
    
    /// The server sending a message to client B by distributing a message from client A
    /// Use cases: distribution of client message or server update (e.g., someone leaving)
    ServerText(String, String), // sender name, text

    /// Server notifying all the clients that the room is closing
    ServerShutdown,

    /// Server notifying the person being kicked
    ServerNotifyKick,

    /// Server responding to a client with a list of (name, id) pairs
    ServerResponseIDs(Vec<(String, u64)>),
}
