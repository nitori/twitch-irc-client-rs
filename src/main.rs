use irc::client::{Client, StreamingIterator};
use crate::irc::protocol::Command;

mod irc;

fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is missing!");
    let mut client = Client::new(&token, &"SaniSensei");
    client.connect();

    while let Some(msg) = client.stream_next() {
        match msg.command {
            Command::Ready => {
                client.send_line("JOIN #bloodstainedvt").unwrap();
            }
            Command::Privmsg if msg.params.len() == 2 && msg.prefix.as_ref().is_some_and(|p| p.nick.is_some()) => {
                println!("{} <{}> {}", msg.params[0], msg.display_name().unwrap(), msg.params[1]);
                //println!("tags: {:#?}", msg.tags);
            }
            Command::Names => {
                println!("Joined: {}", msg.params[2]);
            }
            _ => ()
        }
    }
}
