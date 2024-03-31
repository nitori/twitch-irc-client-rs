use irc::client::Client;
use crate::irc::protocol::Command;

mod irc;

fn main() {
    dotenv::dotenv().ok();
    let token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is missing!");
    let mut client = Client::new(&token, &"SaniSensei");
    client.connect();

    for msg in client.iter() {
        match msg.command {
            Command::Ready => {
                client.send_line("JOIN #bloodstainedvt").unwrap();
            }
            Command::Privmsg if msg.is_channel_message() => {
                println!("{} <{}> {}", msg.params[0], msg.display_name().unwrap(), msg.params[1]);
            }
            Command::Privmsg if msg.is_private_message() => {
                println!("(private) <{}> {}", msg.display_name().unwrap(), msg.params[1]);
            }
            Command::Names => {
                println!("Joined: {}", msg.params[2]);
            }
            _ => ()
        }
    }
}
