use crate::irc::protocol::ParseError::MissingCommand;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnknownCommand(String),
    MissingCommand,
}


#[derive(Debug)]
pub enum Command {
    Ready,
    Privmsg,
    Notice,
    Join,
}

#[derive(Debug)]
pub struct Prefix {
    pub nick: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
}

#[derive(Debug)]
pub struct Message {
    pub prefix: Option<Prefix>,
    pub command: Command,
    pub params: Vec<String>,
    _had_trailing: bool,
    _original_line: String,
}

fn extract_prefix(line: &str) -> Result<(Option<Prefix>, &str)> {
    if !line.starts_with(":") {
        return Ok((None, line));
    }

    if let Some(end_pos) = line.find(" ") {
        let prefix_string = &line[1..end_pos];
        let mut prefix = Prefix {
            nick: None,
            user: None,
            host: None,
        };
        if let Some((nick_user, host)) = prefix_string.split_once("@") {
            prefix.host = Some(host.into());
            if let Some((nick, user)) = nick_user.split_once("!") {
                prefix.nick = Some(nick.into());
                prefix.user = Some(user.into());
            } else {
                prefix.nick = Some(nick_user.into());
            }
        } else {
            prefix.host = Some(prefix_string.into());
        }
        Ok((Some(prefix), &line[end_pos + 1..]))
    } else {
        // if no space, it implies there is no command.
        Err(MissingCommand)
    }
}

fn extract_command(line: &str) -> Result<(Command, &str)> {
    if let Some((cmd, rest)) = line.split_once(" ") {
        Ok((map_command(cmd)?, rest))
    } else {
        Err(MissingCommand)
    }
}


fn extract_params(line: &str) -> Result<(Vec<String>, bool, &str)> {
    if line.starts_with(":") {
        return Ok((vec![line[1..].into()], true, &line[line.len()..]));
    }

    if let Some(trail_pos) = line.find(" :") {
        let param_string = &line[..trail_pos];
        let trailing = &line[trail_pos + 2..];
        let params = to_params(param_string, Some(trailing));
        Ok((params, true, ""))
    } else {
        let params = to_params(line, None);
        Ok((params, false, ""))
    }
}

fn to_params(param_string: &str, trailing: Option<&str>) -> Vec<String> {
    let mut params = param_string.split(" ").map(|s| s.into()).collect::<Vec<String>>();
    if trailing.is_some() {
        params.push(trailing.unwrap().into());
    }
    params
}


pub fn parse_line(line: &str) -> Result<Message> {
    let original_line: String = line.into();

    let (prefix, line) = extract_prefix(line)?;
    let (command, line) = extract_command(line)?;
    let (params, had_trailing, _) = extract_params(line)?;

    Ok(Message {
        prefix,
        command,
        params,
        _had_trailing: had_trailing,
        _original_line: original_line,
    })
}


fn map_command(cmd: &str) -> Result<Command> {
    match cmd {
        "001" => Ok(Command::Ready),
        "PRIVMSG" => Ok(Command::Privmsg),
        "NOTICE" => Ok(Command::Notice),
        "JOIN" => Ok(Command::Join),
        _ => Err(ParseError::UnknownCommand(cmd.to_string()))
    }
}
