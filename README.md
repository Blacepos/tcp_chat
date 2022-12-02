# Tcp Chat
A remake of a program I made a several years ago using C++ and SFML. The original was very messy and single-threaded. I wanted to challenge myself using my new experience to make something more organized and robust. Rust certainly helped with the robustness.

## TcpConn
In the first iteration of this remake, I quickly ran into a problem: I wasn't able to read arbitrary length data from a `TcpSteam`. Something I wasn't aware of was that SFML made sockets really nice by providing this capability and this isn't the default behavior with sockets.

Turns out not being able to send discrete and arbitrary data is pretty inconvenient.

My first idea was to use some special byte as a terminator, but then I realized that since Rust strings are UTF-8 encoded, that byte could appear randomly in the middle of the message unless I started truncating what the user entered into ASCII. My second idea, which is what I ended up going with, was to have an 8-byte header encoding the length of the rest of the message.

I also wanted easy access to the data the message encoded, so I decided to use serde to automatically serialize and deserialize whatever values I send.

To abstract away the complexity, I made a wrapper for `TcpSteam` called `TcpConn` which would provide simple `send<T>` and `receive<T>` functions.

*Side note: can I just take a moment to say I implemented this entire thing in one go and it worked on the first try? I've been wary to call Rust my favorite language due to the fact that I only started learning it several months ago, but I'm officially declaring it now. It's such a surreal experience to have something you've been conditioned to expect to fail actually work, which seems to be a common occurence with Rust.*

Despite some potential security vulnerabilities, `TcpConn` ended up working great in practice. The message type I decided on was an enum with each variant representing some kind of "command" that the recipeint could patten match on. 

## Potential improvements
- instead of using `Arc<Mutex<...>>` to share the list of clients between the listener thread and the communication thread, it would probably be better to use `sync::mpsc::channel` to send the new client object to the communication thread as soon as it is ready.