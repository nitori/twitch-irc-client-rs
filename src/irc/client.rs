use std::{fmt, thread};
use std::fmt::Formatter;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver};
use crate::irc::protocol::{Command, Message, parse_line};

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
    nickname: String,
    receiver: Option<Receiver<Message>>,
    pub stream: Option<TcpStream>,
}


impl Client {
    pub fn new(token: &str, nickname: &str) -> Client {
        Client {
            token: Secret { value: token.into() },
            nickname: nickname.into(),
            receiver: None,
            stream: None,
        }
    }

    pub fn connect(&mut self) {
        let (ts, tx) = channel::<Message>();
        self.receiver = Some(tx);

        let mut stream = TcpStream::connect("irc.twitch.tv:6667").unwrap();
        self.stream = Some(stream.try_clone().unwrap());

        self.send_line("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands").unwrap();
        self.send_line(&format!("PASS {}\r\n", self.token.value)).unwrap();
        self.send_line(&format!("NICK {}\r\n", self.nickname)).unwrap();

        thread::spawn(move || {
            let mut vbuf: Vec<u8> = vec![];
            'mainloop: loop {
                let mut buf = [0u8; 8096];
                let chunk;
                match stream.read(&mut buf) {
                    Ok(size) => {
                        if size == 0 {
                            println!("Stream read returned 0 bytes");
                            break 'mainloop;
                        }
                        chunk = &buf[..size];
                    }
                    Err(e) => {
                        println!("Failed to read from stream {:?}", e);
                        break 'mainloop;
                    }
                }
                vbuf.extend(chunk);

                while let Some((pos, _)) = vbuf.iter().enumerate().find(|(_, c)| **c == 10) {
                    let line_vec = vbuf[..pos].to_vec();
                    vbuf = vbuf[pos + 1..].to_vec();
                    if let Ok(line) = String::from_utf8(line_vec.clone()) {
                        let final_line = line.trim_end_matches(|c| c == '\r' || c == '\n');
                        match parse_line(final_line) {
                            Ok(msg) => {
                                match msg.command {
                                    Command::Ping => {
                                        println!("Ping? Pong!");
                                        let write_result = stream.write(
                                            format!("{}\r\n", msg.with_command(Command::Pong)).as_bytes()
                                        );
                                        if let Err(e) = write_result {
                                            println!("Error writing to stream {:?}", e);
                                            break 'mainloop;
                                        }
                                    }
                                    _ => {
                                        if let Err(e) = ts.send(msg) {
                                            println!("Error sending to channel {:?}", e);
                                            break 'mainloop;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error: {:?} - {:?}", e, final_line);
                            }
                        }
                    } else {
                        println!("*** Couldn't parse line {:?}", line_vec);
                    }
                }
            }
        });
    }

    pub fn send_line(&self, line: &str) -> Result<(), std::io::Error> {
        if let Some(mut stream) = self.stream.as_ref() {
            stream.write(format!("{}\r\n", line).as_bytes())?;
        }
        Ok(())
    }
}

pub struct ClientIterator<'a> {
    receiver: &'a Receiver<Message>,
}

impl<'a> Iterator for ClientIterator<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(msg) = self.receiver.recv() {
            Some(msg)
        } else {
            None
        }
    }
}

impl Client {
    pub fn iter(&self) -> ClientIterator {
        ClientIterator {
            receiver: self.receiver.as_ref().unwrap(),
        }
    }
}
