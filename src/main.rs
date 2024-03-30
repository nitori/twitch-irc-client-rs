mod irc;

fn main() {
    dotenv::dotenv().ok();
    //let token = std::env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN is missing!");

    let msg_result = irc::protocol::parse_line(&":amc PRIVMSG #channel :Hello World".into());

    match msg_result {
        Ok(msg) => {
            match msg.command {
                irc::protocol::Command::Privmsg if msg.params.len() == 2 && msg.prefix.as_ref().is_some_and(|p| p.nick.is_some()) => {
                    println!("<{}@{}> {}", msg.prefix.unwrap().nick.unwrap(), msg.params[0], msg.params[1])
                }
                _ => {
                    println!("{:?}", msg)
                }
            }
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
}
