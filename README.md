# Tcp Chat
A remake of a program I made a several years ago using C++ and SFML. The original was very messy and single-threaded. I wanted to challenge myself using my new experience to make something more organized and robust.

## TcpConn
In the first iteration of this remake, I quickly ran into a problem: I wasn't able to read arbitrary length data from a `TcpSteam`. Something I wasn't aware of was that SFML made sockets really nice by providing this capability via `sf::Packet` and this isn't the default behavior with sockets.

Turns out not being able to send discrete and arbitrary data is pretty inconvenient.

My first idea was to use some special byte as a terminator, but then I realized that since Rust strings are UTF-8 encoded, that byte could appear randomly in the middle of the message unless I started truncating what the user entered into ASCII. My second idea, which is what I ended up going with, was to have an 8-byte header encoding the length of the rest of the message.

I also wanted easy access to the data the message encoded, so I decided to use serde to automatically serialize and deserialize whatever values I send.

To abstract away the complexity, I made a wrapper for `TcpSteam` called `TcpConn` which would provide simple `send<T>` and `receive<T>` functions.

`TcpConn` ended up working great in practice. The message type I decided on was an enum with each variant representing some kind of "command" that the recipient could patten match on. 

## Potential improvements
- Instead of using `Arc<Mutex<...>>` to share the list of clients between the listener thread and the communication thread, it would probably be better to use `sync::mpsc::channel` to send the new client object to the communication thread as soon as it is ready.
- I didn't realize there was a `TcpStream::shutdown` method and was just discarding the steams when a client left. Using this might be able to help me remove that unstable `io::ErrorKind::Uncategorized` error in the client's message-receiving thread.
- `TcpConn` at this point should probably return a custom error type instead of forcing the user to check the `io::ErrorKind` of the errors.
- Have `TcpConn` wrap the creation of `TcpStream` as well as provide its own listener to return `TcpConn`s.
- CoLOrEd TExT
