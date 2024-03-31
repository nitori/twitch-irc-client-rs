use std::fmt::{Debug, Display, Formatter};
use indexmap::IndexMap;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnknownCommand(String),
    MissingCommand,
    InvalidEmoteString,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Misc(String),
    Ping,
    Pong,
    // 001
    Ready,
    // 353
    Names,
    // 366
    EndOfNames,
    Privmsg,
    Notice,
    Join,
    Part,
    Cap,
    // twitch specific
    GlobalUserState,
    UserState,
    RoomState,
    UserNotice,
}

#[derive(Debug, Clone)]
pub struct Prefix {
    pub nick: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if let Some(nick) = &self.nick {
            s.push_str(nick);
        }
        if let Some(user) = &self.user {
            s.push_str("!");
            s.push_str(user);
        }
        if let Some(host) = &self.host {
            s.push_str("@");
            s.push_str(host);
        }

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub tags: IndexMap<String, String>,
    pub prefix: Option<Prefix>,
    pub command: Command,
    pub params: Vec<String>,
    _had_trailing: bool,
    _original_line: String,
}

#[derive(Debug, PartialEq)]
pub struct EmoteInfo {
    pub id: String,
    pub emote: String,
}

#[derive(Debug, PartialEq)]
pub enum RichText {
    Text(String),
    Emote(EmoteInfo),
}

impl Message {
    pub fn display_name(&self) -> Option<&String> {
        if let Some(name) = self.tags.get("display-name") {
            Some(name)
        } else {
            let prefix = self.prefix.as_ref().unwrap();
            let nick = prefix.nick.as_ref().unwrap();
            Some(nick)
        }
    }

    pub fn is_valid_privmsg(&self) -> bool {
        return self.params.len() == 2 && self.prefix.as_ref().is_some_and(|p| p.nick.is_some());
    }

    pub fn is_channel_message(&self) -> bool {
        return self.is_valid_privmsg() && self.params[0].starts_with("#");
    }

    pub fn is_private_message(&self) -> bool {
        return self.is_valid_privmsg() && !self.params[0].starts_with("#");
    }

    pub fn with_command(&self, new_command: Command) -> Message {
        Message {
            tags: self.tags.clone(),
            prefix: self.prefix.clone(),
            command: new_command,
            params: self.params.clone(),
            _had_trailing: self._had_trailing,
            _original_line: self._original_line.clone(),
        }
    }

    pub fn emotes(&self) -> Result<Vec<RichText>> {
        if !self.is_channel_message() {
            return Ok(vec![]);
        }

        let emotes_string = match self.tags.get("emotes") {
            None => return Ok(vec![]),
            Some(s) => s
        };

        let mut emotes: Vec<(&str, usize, usize)> = vec![];
        let parts = emotes_string.split('/').collect::<Vec<_>>();
        for part in parts {
            if let Some((emote, places)) = part.split_once(":") {
                if let Some((start, end)) = places.split_once("-") {
                    let start = start.parse::<usize>().map_err(|_| ParseError::InvalidEmoteString)?;
                    let end = end.parse::<usize>().map_err(|_| ParseError::InvalidEmoteString)?;
                    emotes.push((emote, start, end));
                }
            }
        }

        // not sure if emotes are guaranteed to be sorted, but just to be safe.
        emotes.sort_by(|a, b| a.1.cmp(&b.1));

        let mut rich_parts: Vec<RichText> = vec![];

        let text = self.params[1].as_str();
        let mut last_end: usize = 0;

        for (emote, start, end) in emotes {
            let text_part = &text[last_end..start];
            let emote_part = &text[start..=end];
            last_end = end + 1;

            if !text_part.is_empty() {
                rich_parts.push(RichText::Text(text_part.into()));
            }
            rich_parts.push(RichText::Emote(EmoteInfo {
                id: emote.into(),
                emote: emote_part.into(),
            }))
        }

        if last_end < text.len() {
            let last_text_part = &text[last_end..];
            rich_parts.push(RichText::Text(last_text_part.into()));
        }

        Ok(rich_parts)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let tags = self.tags.iter().map(|(k, v)| {
            format!("{}={}", k, v)
        }).collect::<Vec<_>>().join(";");

        if tags.len() > 0 {
            s.push_str("@");
            s.push_str(&tags);
            s.push_str(" ");
        }

        if let Some(prefix) = &self.prefix {
            s.push_str(&format!("{}", prefix));
            s.push_str(" ");
        }

        s.push_str(&map_command_back(&self.command));

        for (i, param) in self.params.iter().enumerate() {
            if i == self.params.len() - 1 && (self._had_trailing || param.contains(' ') || param.starts_with(":")) {
                s.push_str(" :");
            } else {
                s.push_str(" ");
            }
            s.push_str(param);
        }

        write!(f, "{}", s)
    }
}

fn extract_tags(line: &str) -> Result<(IndexMap<String, String>, &str)> {
    if !line.starts_with("@") {
        return Ok((IndexMap::new(), line));
    }

    let (tag_string, rest) = if let Some(end_pos) = line.find(" ") {
        (&line[1..end_pos], &line[end_pos + 1..])
    } else {
        (&line[1..], "")
    };

    let mut tags = IndexMap::new();

    for part in tag_string.split(";") {
        if let Some((key, value)) = part.split_once("=") {
            tags.insert(key.into(), value.into());
        } else {
            tags.insert(part.into(), "".into());
        }
    }

    Ok((tags, rest))
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
        Err(ParseError::MissingCommand)
    }
}

fn extract_command(line: &str) -> Result<(Command, &str)> {
    if let Some((cmd, rest)) = line.split_once(" ") {
        Ok((map_command(cmd)?, rest))
    } else if line.len() > 0 {
        Ok((map_command(line)?, ""))
    } else {
        Err(ParseError::MissingCommand)
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
    let mut params = if param_string.is_empty() {
        vec![]
    } else {
        param_string.split(" ").map(|s| s.into()).collect::<Vec<String>>()
    };
    if trailing.is_some() {
        params.push(trailing.unwrap().into());
    }
    params
}


pub fn parse_line(line: &str) -> Result<Message> {
    let original_line: String = line.into();

    let (tags, line) = extract_tags(line)?;
    let (prefix, line) = extract_prefix(line)?;
    let (command, line) = extract_command(line)?;
    let (params, had_trailing, _) = extract_params(line)?;

    Ok(Message {
        tags,
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
        "002" | "003" | "004" | "375" | "372" | "376" => Ok(Command::Misc(cmd.into())),
        "353" => Ok(Command::Names),
        "366" => Ok(Command::EndOfNames),
        "PING" => Ok(Command::Ping),
        "PONG" => Ok(Command::Pong),
        "PRIVMSG" => Ok(Command::Privmsg),
        "NOTICE" => Ok(Command::Notice),
        "JOIN" => Ok(Command::Join),
        "PART" => Ok(Command::Part),
        "CAP" => Ok(Command::Cap),
        "GLOBALUSERSTATE" => Ok(Command::GlobalUserState),
        "USERSTATE" => Ok(Command::UserState),
        "ROOMSTATE" => Ok(Command::RoomState),
        "USERNOTICE" => Ok(Command::UserNotice),
        _ => Err(ParseError::UnknownCommand(cmd.to_string()))
    }
}

fn map_command_back(cmd: &Command) -> String {
    match cmd {
        Command::Misc(v) => v.into(),
        Command::Ready => "001".into(),
        Command::Names => "353".into(),
        Command::EndOfNames => "366".into(),
        Command::Ping => "PING".into(),
        Command::Pong => "PONG".into(),
        Command::Privmsg => "PRIVMSG".into(),
        Command::Notice => "NOTICE".into(),
        Command::Join => "JOIN".into(),
        Command::Part => "PART".into(),
        Command::Cap => "CAP".into(),
        Command::GlobalUserState => "GLOBALUSERSTATE".into(),
        Command::UserState => "USERSTATE".into(),
        Command::RoomState => "ROOMSTATE".into(),
        Command::UserNotice => "USERNOTICE".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privmsg() {
        let line = ":nick!user@host PRIVMSG #channel :Hello World!";
        let result = parse_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert!(msg.prefix.is_some());

        let prefix = msg.prefix.unwrap();
        assert!(prefix.nick.is_some());

        assert_eq!(prefix.nick.unwrap(), "nick");
        assert_eq!(prefix.user.unwrap(), "user");
        assert_eq!(prefix.host.unwrap(), "host");


        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.params, vec!["#channel", "Hello World!"]);
    }

    #[test]
    fn test_no_prefix() {
        let line = "PRIVMSG #channel :Hello World!";
        let result = parse_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert!(msg.prefix.is_none());

        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.params, vec!["#channel", "Hello World!"]);
    }

    #[test]
    fn test_command_only() {
        let line = "PRIVMSG";
        let result = parse_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert!(msg.prefix.is_none());

        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.params.len(), 0);
    }

    #[test]
    fn test_command_and_trailing() {
        let line = "PRIVMSG :Hello World!";
        let result = parse_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert!(msg.prefix.is_none());

        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.params.len(), 1);
        assert_eq!(msg.params, vec!["Hello World!"]);
    }

    #[test]
    fn test_command_and_params() {
        let line = "PRIVMSG param1 param2";
        let result = parse_line(line);
        assert!(result.is_ok());

        let msg = result.unwrap();
        assert!(msg.prefix.is_none());

        assert_eq!(msg.command, Command::Privmsg);
        assert_eq!(msg.params.len(), 2);
        assert_eq!(msg.params, vec!["param1", "param2"]);
    }

    #[test]
    fn test_empty_string() {
        let line = "";
        let result = parse_line(line);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::MissingCommand);
    }

    #[test]
    fn test_prefix_no_command() {
        let line = ":nick!user@host";
        let result = parse_line(line);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::MissingCommand);
    }

    #[test]
    fn test_unknown_command() {
        let line = "UNKNOWNCOMMAND";
        let result = parse_line(line);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::UnknownCommand("UNKNOWNCOMMAND".into()))
    }

    #[test]
    fn test_emotes_none() {
        let line = "@emotes= :nick!user@host PRIVMSG #channel :nothing";
        let result = parse_line(line);
        assert!(result.is_ok());
        let msg = result.unwrap();
        let emotes_result = msg.emotes();
        assert!(emotes_result.is_ok());
        let emotes = emotes_result.unwrap();
        assert_eq!(emotes.len(), 1);
        assert_eq!(emotes[0], RichText::Text("nothing".into()));
    }

    #[test]
    fn test_emotes_single() {
        let line = "@emotes=86:0-9 :nick!user@host PRIVMSG #channel :BibleThump";
        let result = parse_line(line);
        assert!(result.is_ok());
        let msg = result.unwrap();
        let emotes_result = msg.emotes();
        assert!(emotes_result.is_ok());
        let emotes = emotes_result.unwrap();
        assert_eq!(emotes.len(), 1);
        assert_eq!(emotes[0], RichText::Emote(EmoteInfo {
            id: "86".into(),
            emote: "BibleThump".into(),
        }));
    }

    #[test]
    fn test_emotes_mixed() {
        let line = "@emotes=86:10-19/46:38-43 :nick!user@host PRIVMSG #channel :This is a BibleThump test with emotes SSSsss yay \\o/";
        let result = parse_line(line);
        assert!(result.is_ok());
        let msg = result.unwrap();
        let emotes_result = msg.emotes();
        assert!(emotes_result.is_ok());
        let emotes = emotes_result.unwrap();
        assert_eq!(emotes.len(), 5);

        assert_eq!(emotes[0], RichText::Text("This is a ".into()));
        assert_eq!(emotes[1], RichText::Emote(EmoteInfo {
            id: "86".into(),
            emote: "BibleThump".into(),
        }));
        assert_eq!(emotes[2], RichText::Text(" test with emotes ".into()));
        assert_eq!(emotes[3], RichText::Emote(EmoteInfo {
            id: "46".into(),
            emote: "SSSsss".into(),
        }));
        assert_eq!(emotes[4], RichText::Text(" yay \\o/".into()));
    }
}

