use irc::client::Client;
use crate::irc::protocol::Command;

mod irc;

fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is missing!");
    let nickname = std::env::var("NICKNAME").expect("NICKNAME is missing!");
    let mut channel = std::env::var("CHANNEL").expect("CHANNEL is missing!");
    if !channel.starts_with("#") {
        channel = format!("#{}", channel);
    }

    let mut client = Client::new(&token, &nickname);
    client.connect();

    for msg in client.iter() {
        match msg.command {
            Command::Ready => {
                client.send_line(&format!("JOIN {}", channel)).unwrap();
            }
            Command::Privmsg if msg.is_channel_message() => {
                println!("{} <{}> {}", msg.params[0], msg.display_name().unwrap(), msg.params[1]);
            }
            Command::Privmsg if msg.is_private_message() => {
                println!("(private) <{}> {}", msg.display_name().unwrap(), msg.params[1]);
            }
            Command::EndOfNames => {
                println!("Joined: {}", msg.params[1]);
            }
            _ => ()
        }
    }
}
