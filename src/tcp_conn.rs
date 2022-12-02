use std::intrinsics::type_name;
use std::io::{self, Write, Read};
use std::net::TcpStream;
use serde::Serialize;
use serde::de::DeserializeOwned;



/// The number of bytes pulled from the TcpStream at a time. Smaller means more system calls, larger
/// means bulkier stack.
const POLL_SIZE: usize = 4096;

/// Wraps a TcpStream to provide an interface for sending arbitrary data over the network.
/// 
/// # Security
/// This should not be used in professional settings as no security protocols are implemented. Some
/// potential vulnerabilities that may arise include:
/// - the leaking of information due to lack of encryption
/// - the ability for an attacker to easily construct a custom message that will be deserialized
/// into a type used by the application.
pub struct TcpConn {
    stream: TcpStream,

    // VecDeque would be better because draining is faster, however any gains are nullified due to
    // the fact that there's currently no way in std to construct a string from an iterator of
    // bytes, thereby forcing a copy of the data anyways by going through an intermediate container
    // (issue occurs in `receive`).
    buffer: Vec<u8>
}

impl TcpConn {
    /// Construct a `TcpConn` by wrapping a `TcpStream`.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::new()
        }
    }

    /// Set the connection's blocking state. Simply wraps `TcpStream::set_nonblocking`
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.stream.set_nonblocking(nonblocking)
    }

    /// Send an arbitrary message across the network.
    /// 
    /// # Errors
    /// This function may return an error if the underlying TcpStream decides to return an error or
    /// if serialization of the message fails.
    pub fn send<T>(&mut self, data: T) -> io::Result<()>
    where T: Serialize {

        let bytes = serde_json::to_string(&data)?.into_bytes();
        let mut packet = bytes.len().to_le_bytes().to_vec();

        packet.extend(bytes);

        self.stream.write_all(&packet)?;
        self.stream.flush()?;

        Ok(())
    }

    /// Receive the next incoming message and attempt to deserialize it into some type.
    /// 
    /// # Errors
    /// This function may return errors for several reasons, some are perfectly normal and expected
    /// while others are more significant problems like failure to interact with the TCP socket.
    /// 
    /// The "normal" errors include inability to reconstruct the original data due to insufficient
    /// bytes (which are `io::ErrorKind::Other`).
    /// 
    /// The "unexpected" errors include failure to deserialize supposedly complete data into the
    /// wrong type (`io::ErrorKind::InvalidData`) and failure to read from the `TcpStream`, which
    /// could be any of the errors returned by `TcpStream`
    pub fn receive<T>(&mut self) -> io::Result<T>
    where T: DeserializeOwned {

        // try receiving some data by polling the TcpStream until it is empty
        let mut readbuf = [0u8; POLL_SIZE];
        loop {
            // grab POLL_SIZE bytes from the TcpStream and add them to self.buffer
            let bytes_read = self.stream.read(&mut readbuf)?;
            self.buffer.extend(&readbuf[..bytes_read]);

            // check if there are no more bytes to read (even if we don't have enough bytes to
            // deserialize into `T`)
            if bytes_read < POLL_SIZE {
                break;
            }
        }

        // attempt to read the 8 bytes representing the payload size
        let size_bytes: [u8; 8] = self.buffer.get(..8)
            .ok_or_else(incomplete_buffer_error::<T>)?
            .try_into()
            .map_err(|_| incomplete_buffer_error::<T>())?;
        
        let payload_size = usize::from_le_bytes(size_bytes);

        // make sure theres enough bytes to reconstruct the original data type
        if self.buffer.len() < payload_size + 8 {
            return Err(incomplete_buffer_error::<T>());
        }

        // convert bytes to str
        let payload_str = std::str::from_utf8(&self.buffer[8..payload_size+8])
            .map_err(|_| reconstruction_error::<T>())?;
        
        // deserialize the str into `T`
        let data = serde_json::from_str(payload_str)
            .map_err(|_| reconstruction_error::<T>())?;

        // remove size+8 bytes from the buffer. this is last because we don't want to drain if
        // the previous operations fail
        self.buffer.drain(..payload_size+8);
        
        Ok(data)
    }
}

/// A helper function to return an error which is used frequently
fn incomplete_buffer_error<T>() -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        format!(
            "Unable to reconstruct a value of type `{}` due to insufficient data. This should be handled by waiting until enough bytes have arrived.",
            type_name::<T>()
        )
    )
}

/// A helper function to return an error which is used frequently
fn reconstruction_error<T>() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!(
            "Unable to reconstruct a value of type `{}` due to invalid type. This error indicates that all the data from the message has arrived, but it cannot be deserialized into the given type.",
            type_name::<T>()
        )
    )
}
