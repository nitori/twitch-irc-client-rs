use std::fmt;
use std::fmt::Formatter;
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::irc::protocol::{Command, parse_line};

struct Secret {
    value: String,
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "*****")
    }
}

#[derive(Debug)]
pub struct Client {
    token: Secret,
}


impl Client {
    pub fn new(token: &String) -> Client {
        Client { token: Secret { value: token.clone() } }
    }

    pub fn connect(&self) {
        let mut stream = TcpStream::connect("irc.twitch.tv:6667").unwrap();
        let mut vbuf: Vec<u8> = vec![];

        //stream.write("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands\r\n".as_bytes()).unwrap();
        stream.write(format!("PASS {}\r\n", self.token.value).as_bytes()).unwrap();
        stream.write("NICK SaniSensei\r\n".as_bytes()).unwrap();


        loop {
            let mut buf = [0u8; 1024];
            let size = stream.read(&mut buf).unwrap();
            let chunk = &buf[..size];
            vbuf.extend(chunk);

            while let Some((pos, _)) = vbuf.iter().enumerate().find(|(_, c)| **c == 10) {
                let line_vec = vbuf[..pos].to_vec();
                vbuf = vbuf[pos + 1..].to_vec();
                if let Ok(line) = String::from_utf8(line_vec) {
                    let final_line = line.trim_end_matches(|c| c == '\r' || c == '\n');
                    match parse_line(final_line) {
                        Ok(msg) => {
                            match msg.command {
                                Command::Ready => {
                                    stream.write("JOIN #bloodstainedvt\r\n".as_bytes()).unwrap();
                                }
                                Command::Privmsg if msg.params.len() == 2 && msg.prefix.as_ref().is_some_and(|p| p.nick.is_some()) => {
                                    println!("<{}@{}> {}", msg.prefix.unwrap().nick.unwrap(), msg.params[0], msg.params[1])
                                }
                                _ => ()
                            }
                        }
                        Err(e) => {
                            println!("Error: {:?} - {:?}", e, final_line);
                        }
                    }
                }
            }
        }
    }
}