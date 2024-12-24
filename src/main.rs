use crate::discord::message;
use discord::client::DiscordClient;
mod discord;
mod https;
use std::env;
mod minecraft;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut client = DiscordClient::new(args.get(1).unwrap());

    let msg = message::MessageBuilder::new()
        .content("Welcome from Rust!")
        .build();

    let response = client.send_message(msg, "1296137217604849704");
    println!("Done!")
}
