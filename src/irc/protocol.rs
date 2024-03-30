use crate::irc::protocol::ParseError::MissingCommand;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    UnknownCommand(String),
    MissingCommand(String),
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


pub fn parse_line(line: &String) -> Result<Message> {
    let original_line: String = line.into();

    let mut pos = 0;

    let prefix = if line.starts_with(":") {
        if let Some(i) = line.find(" ") {
            let prefix_string: String = line[1..i].into();
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
                prefix.host = Some(prefix_string);
            }

            pos = i + 1;
            Some(prefix)
        } else {
            // if there is no space, it means there is no command after the prefix.
            return Err(MissingCommand(original_line))
        }
    } else {
        None
    };

    let line = &line[pos..];

    let (command, line) = if let Some(params_pos) = line.find(" ") {
        (line[..params_pos].to_string(), line[params_pos..].to_string())
    } else {
        (line.trim().to_string(), "".to_string())
    };

    if command.is_empty() {
        return Err(MissingCommand(original_line));
    }

    let (param_string, trailing) = if let Some(trail_pos) = line.find(" :") {
        (
            line[..trail_pos].trim().to_string(),
            Some(line[trail_pos + 2..].to_string())
        )
    } else {
        (line.trim().to_string(), None)
    };

    let params: Vec<&str> = if param_string.is_empty() {
        vec![]
    } else {
        param_string.split(" ").collect()
    };
    let mut new_params: Vec<String> = params.iter().map(|s| (*s).into()).collect();
    let had_trailing = match trailing {
        Some(t) => {
            new_params.push(t);
            true
        }
        None => false
    };

    Ok(Message {
        prefix,
        command: map_command(&command)?,
        params: new_params,
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
