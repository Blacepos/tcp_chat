// Some of these are helpers to go along with "result_repeat.rs". Those, and the functions in
// "result_repeat.rs" only serve the impractical role of saving a few lines in `main()`

use std::io::{self, Write};

/// Simple input wrapper for my use case.
pub fn input_msg() -> String {

    let mut buf = String::new();

    io::stdout()
        .flush()
        .expect("[error] Unable to write to buffer!");
    io::stdin()
        .read_line(&mut buf)
        .expect("[error] Unable to read input!");

    buf.trim().to_owned()
}

/// Simple input wrapper for my use case. Adds a little "> " prompt
pub fn input() -> String {

    let mut buf = String::new();

    print!("> ");

    io::stdout()
        .flush()
        .expect("[error] Unable to write to buffer!");
    io::stdin()
        .read_line(&mut buf)
        .expect("[error] Unable to read input!");

    buf.trim().to_owned()
}



pub trait CmdResponse {
    fn is_yes(&self) -> bool;
}

impl CmdResponse for String {
    /// Interpret a string as either "Yes" or "No" (`true` or `false`). "No" is the default if no
    /// match.
    fn is_yes(&self) -> bool {
        let s = self.to_lowercase();
        s == "yes" || s == "y"
    }
}

/// Validator for `UntilValid` to determine if a given string is like "yes" or "no". This is a
/// helper to work with `input()`, which is why it takes a `&String` instead of a `&str`
#[allow(clippy::ptr_arg)]
pub fn validate_yn(s: &String) -> bool {
    let sl = s.to_lowercase();
    let sl = sl.as_str();
    let valid = sl == "y" || sl == "yes" || sl == "n" || sl == "no";

    if !valid {
        println!("Please enter a valid option");
    }
    valid
}
