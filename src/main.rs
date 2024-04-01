use std::fs::{OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use irc::client::Client;
use crate::irc::protocol::{Command, RichText};
use crate::irc::utils::Color;

mod irc;

fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is missing!");
    let nickname = std::env::var("NICKNAME").expect("NICKNAME is missing!");
    let channel_string = std::env::var("CHANNELS").expect("CHANNELS is missing!");
    let channels = channel_string.split(",").map(|c| {
        if c.starts_with("#") {
            c.into()
        } else {
            format!("#{}", c)
        }
    }).collect::<Vec<String>>();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("logs.txt").unwrap();

    file.seek(SeekFrom::End(0)).unwrap();
    file.write("----------------\n".as_bytes()).unwrap();
    file.write("- Reconnecting -\n".as_bytes()).unwrap();
    file.write("----------------\n".as_bytes()).unwrap();

    let mut client = Client::new(&token, &nickname);
    client.connect();

    for msg in client.iter() {
        match msg.command {
            Command::Part => {}
            Command::Join => {}
            _ => {
                file.write(msg.original_line().as_bytes()).unwrap();
                file.write("\n".as_bytes()).unwrap();
            }
        }
        match msg.command {
            Command::Ready => {
                for channel in channels.iter() {
                    client.send_line(&format!("JOIN {}", channel)).unwrap();
                }
            }
            Command::Privmsg if msg.is_channel_message() => {
                let display_name = msg.display_name().unwrap();
                let colored_name = if let Some(hex) = msg.tags.get("color") {
                    match Color::from_string(hex) {
                        Ok(color) => color.wrap_ansi(display_name),
                        Err(_) => display_name.into()
                    }
                } else {
                    display_name.into()
                };

                if let Ok(emotes) = msg.emotes() {
                    let text_items = emotes.iter().map(|e| {
                        match e {
                            RichText::Text(s) => s.into(),
                            RichText::Emote(e) => {
                                let mut s = String::new();
                                s.push_str("\x1b[32m[");
                                s.push_str(&e.emote);
                                s.push_str("]\x1b[0m");
                                s
                            }
                        }
                    }).collect::<Vec<String>>();
                    let text_message = text_items.join("");
                    println!("{} <{}> {}", msg.params[0], colored_name, text_message);
                } else {
                    println!("{} <{}> {}", msg.params[0], colored_name, msg.params[1]);
                }
            }
            Command::Privmsg if msg.is_private_message() => {
                println!("(private) <{}> {}", msg.display_name().unwrap(), msg.params[1]);
            }
            Command::EndOfNames => {
                println!("Joined: {}", msg.params[1]);
            }
            Command::UserNotice => {
                if let Some(system_msg) = msg.tags.get("system-msg") {
                    println!("System: {}", system_msg.replace("\\s", " "));
                }
            }
            _ => ()
        }
    }
}
