use Command::*;

pub const CLIENT_COMMANDS: [&str; 3] = ["!help", "!exit", "!rename <name>"];
pub const HOST_COMMANDS: [&str; 2] = ["!kick <who>", "!ids"];

pub fn parse_command(cmd: &str, is_host: bool) -> Option<Command> {

    let cmd_args: Vec<&str> = cmd.split(' ').collect();
    let &cmd = cmd_args.first()?;
    let args = cmd_args.get(1..)?;

    if is_host {
        if cmd.starts_with("!help") {
            return Some(HostHelp);
        }
        if cmd.starts_with("!exit") {
            return Some(HostExit);
        }
        if cmd.starts_with("!kick") {
            let &who = args.first()?;
            return Some(Kick(who.parse().ok()?));
            
        }
        if cmd.starts_with("!ids") {
            return Some(RequestIDs)
        }
    }

    if cmd.starts_with("!help") {
        return Some(Help)
    }
    if cmd.starts_with("!exit") {
        return Some(Exit);
    }
    if cmd.starts_with("!rename") {
        let &name = args.first()?;
        return Some(Rename(name.to_string()))
    }
    
    None
}

pub enum Command {
    Help,
    HostHelp,
    Exit,
    HostExit,
    Rename(String),
    Kick(u64),
    RequestIDs,
}
